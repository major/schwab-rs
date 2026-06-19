#![cfg_attr(coverage_nightly, coverage(off))]
use super::*;

impl OrderBuilder {
    /// Convert an existing Schwab order into a reusable builder payload.
    ///
    /// The conversion keeps only fields that can be submitted back to Schwab as
    /// an order request. Response metadata such as order IDs, status, fill
    /// history, timestamps, and account numbers is intentionally omitted.
    ///
    /// Supports `SINGLE`, `TRIGGER`, and `OCO` order strategies with equity or
    /// option legs. Malformed partial orders and unsupported strategy, type, or
    /// instrument values return [`crate::Error::OrderConversion`].
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::OrderConversion`] when required request fields are
    /// missing or the historical order uses a strategy, type, or leg this builder
    /// cannot safely recreate.
    ///
    /// # Examples
    ///
    /// ```
    /// use schwab::{Order, OrderBuilder};
    ///
    /// fn rebuild(order: &Order) -> schwab::Result<OrderBuilder> {
    ///     OrderBuilder::try_from_order(order)
    /// }
    /// ```
    pub fn try_from_order(order: &Order) -> Result<Self> {
        Self::from_order(order)
    }

    pub(super) fn empty(order_strategy_type: OrderStrategyType) -> Self {
        Self {
            order_type: None,
            session: None,
            duration: None,
            order_strategy_type,
            complex_order_strategy_type: None,
            price: None,
            price_link_basis: None,
            price_link_type: None,
            stop_price: None,
            stop_price_link_basis: None,
            stop_price_link_type: None,
            stop_price_offset: None,
            stop_type: None,
            activation_price: None,
            special_instruction: None,
            order_leg_collection: Vec::new(),
            child_order_strategies: Vec::new(),
        }
    }

    fn from_order(order: &Order) -> Result<Self> {
        let strategy = required(order.order_strategy_type.clone(), "orderStrategyType")?;
        let mut builder = Self::empty(strategy.clone());
        builder.copy_optional_fields(order)?;

        match strategy {
            OrderStrategyType::Single => {
                if order
                    .child_order_strategies
                    .as_ref()
                    .is_some_and(|children| !children.is_empty())
                {
                    return Err(conversion_error(
                        "SINGLE order cannot include childOrderStrategies".to_string(),
                    ));
                }

                Ok(builder)
            }
            OrderStrategyType::Trigger => {
                let children = required_children(order, 1, "TRIGGER")?;
                builder.child_order_strategies = vec![Self::from_order(&children[0])?];
                Ok(builder)
            }
            OrderStrategyType::Oco => {
                let children = required_children(order, 2, "OCO")?;
                builder.child_order_strategies = vec![
                    Self::from_order(&children[0])?,
                    Self::from_order(&children[1])?,
                ];
                Ok(builder)
            }
            _ => Err(conversion_error(format!(
                "unsupported orderStrategyType {strategy:?}"
            ))),
        }
    }

    fn copy_optional_fields(&mut self, order: &Order) -> Result<()> {
        reject_unsupported_order_fields(order)?;

        self.order_type = order
            .order_type
            .clone()
            .map(request_order_type)
            .transpose()?;
        self.session.clone_from(&order.session);
        self.duration.clone_from(&order.duration);
        self.complex_order_strategy_type
            .clone_from(&order.complex_order_strategy_type);
        self.price = clone_number(&order.price);
        self.price_link_basis.clone_from(&order.price_link_basis);
        self.price_link_type.clone_from(&order.price_link_type);
        self.stop_price = clone_number(&order.stop_price);
        self.stop_price_link_basis
            .clone_from(&order.stop_price_link_basis);
        self.stop_price_link_type
            .clone_from(&order.stop_price_link_type);
        self.stop_price_offset = clone_number(&order.stop_price_offset);
        self.stop_type.clone_from(&order.stop_type);
        self.activation_price = clone_number(&order.activation_price);
        self.special_instruction
            .clone_from(&order.special_instruction);

        if let Some(legs) = &order.order_leg_collection {
            self.order_leg_collection = legs
                .iter()
                .enumerate()
                .map(|(index, leg)| convert_leg(index, leg))
                .collect::<Result<Vec<_>>>()?;
        }
        validate_order_quantity(order, self)?;

        if self.order_strategy_type != OrderStrategyType::Oco {
            require_submit_fields(self)?;
        }

        Ok(())
    }
}

impl TryFrom<&Order> for OrderBuilder {
    type Error = Error;

    fn try_from(order: &Order) -> Result<Self> {
        Self::try_from_order(order)
    }
}

impl TryFrom<Order> for OrderBuilder {
    type Error = Error;

    fn try_from(order: Order) -> Result<Self> {
        Self::try_from_order(&order)
    }
}

fn required<T>(value: Option<T>, field: impl Into<String>) -> Result<T> {
    value.ok_or_else(|| conversion_error(format!("missing {}", field.into())))
}

fn clone_number(value: &Option<Number>) -> Option<Number> {
    *value
}

fn conversion_error(message: String) -> Error {
    Error::OrderConversion(message)
}

fn required_children<'a>(order: &'a Order, count: usize, strategy: &str) -> Result<&'a [Order]> {
    let children = order.child_order_strategies.as_deref().ok_or_else(|| {
        conversion_error(format!("{strategy} order is missing childOrderStrategies"))
    })?;

    if children.len() != count {
        return Err(conversion_error(format!(
            "{strategy} order requires {count} childOrderStrategies, found {}",
            children.len()
        )));
    }

    Ok(children)
}

fn require_submit_fields(builder: &OrderBuilder) -> Result<()> {
    let order_type = builder
        .order_type
        .as_ref()
        .ok_or_else(|| conversion_error("missing orderType".to_string()))?;
    if builder.session.is_none() {
        return Err(conversion_error("missing session".to_string()));
    }
    if builder.duration.is_none() {
        return Err(conversion_error("missing duration".to_string()));
    }
    if builder.order_leg_collection.is_empty() {
        return Err(conversion_error("missing orderLegCollection".to_string()));
    }

    match order_type {
        OrderTypeRequest::Limit | OrderTypeRequest::LimitOnClose => {
            require_number(builder.price, "price")?;
        }
        OrderTypeRequest::Stop => {
            require_number(builder.stop_price, "stopPrice")?;
        }
        OrderTypeRequest::StopLimit => {
            require_number(builder.price, "price")?;
            require_number(builder.stop_price, "stopPrice")?;
        }
        _ => {}
    }

    Ok(())
}

fn require_number(value: Option<Number>, field: &'static str) -> Result<()> {
    value
        .map(|_| ())
        .ok_or_else(|| conversion_error(format!("missing {field}")))
}

fn reject_unsupported_order_fields(order: &Order) -> Result<()> {
    reject_present(order.tax_lot_method.is_some(), "taxLotMethod")?;

    Ok(())
}

fn validate_order_quantity(order: &Order, builder: &OrderBuilder) -> Result<()> {
    let Some(quantity) = order.quantity else {
        return Ok(());
    };

    if builder.order_leg_collection.len() != 1 {
        return Err(conversion_error(
            "unsupported request field quantity for orders without exactly one leg".to_string(),
        ));
    }

    let leg_quantity = builder.order_leg_collection[0].quantity;
    if leg_quantity != quantity {
        return Err(conversion_error(format!(
            "quantity {quantity:?} does not match orderLegCollection[0].quantity {leg_quantity:?}"
        )));
    }

    Ok(())
}

fn reject_present(present: bool, field: &str) -> Result<()> {
    if present {
        Err(conversion_error(format!(
            "unsupported request field {field}"
        )))
    } else {
        Ok(())
    }
}

fn request_order_type(order_type: OrderType) -> Result<OrderTypeRequest> {
    match order_type {
        OrderType::Market => Ok(OrderTypeRequest::Market),
        OrderType::Limit => Ok(OrderTypeRequest::Limit),
        OrderType::Stop => Ok(OrderTypeRequest::Stop),
        OrderType::StopLimit => Ok(OrderTypeRequest::StopLimit),
        OrderType::TrailingStop => Ok(OrderTypeRequest::TrailingStop),
        OrderType::Cabinet => Ok(OrderTypeRequest::Cabinet),
        OrderType::NonMarketable => Ok(OrderTypeRequest::NonMarketable),
        OrderType::MarketOnClose => Ok(OrderTypeRequest::MarketOnClose),
        OrderType::Exercise => Ok(OrderTypeRequest::Exercise),
        OrderType::TrailingStopLimit => Ok(OrderTypeRequest::TrailingStopLimit),
        OrderType::NetDebit => Ok(OrderTypeRequest::NetDebit),
        OrderType::NetCredit => Ok(OrderTypeRequest::NetCredit),
        OrderType::NetZero => Ok(OrderTypeRequest::NetZero),
        OrderType::LimitOnClose => Ok(OrderTypeRequest::LimitOnClose),
        OrderType::Unknown => Err(conversion_error(
            "unsupported orderType UNKNOWN".to_string(),
        )),
    }
}

fn convert_leg(index: usize, leg: &OrderLegCollection) -> Result<Leg> {
    reject_unsupported_leg_fields(index, leg)?;

    let instruction = required(
        leg.instruction.clone(),
        format!("orderLegCollection[{index}].instruction"),
    )?;
    let quantity = required(
        clone_number(&leg.quantity),
        format!("orderLegCollection[{index}].quantity"),
    )?;
    let instrument = required(
        leg.instrument.as_ref(),
        format!("orderLegCollection[{index}].instrument"),
    )?;
    let (symbol, instrument_asset_type) = instrument_symbol_and_asset(index, instrument)?;
    let asset_type = match leg.order_leg_type.clone().or(instrument_asset_type.clone()) {
        Some(InstrumentAssetType::Equity) => InstrumentAssetType::Equity,
        Some(InstrumentAssetType::Option) => InstrumentAssetType::Option,
        Some(asset_type) => {
            return Err(conversion_error(format!(
                "unsupported orderLegCollection[{index}].orderLegType {asset_type:?}"
            )));
        }
        None => {
            return Err(conversion_error(format!(
                "missing orderLegCollection[{index}].orderLegType or instrument.assetType"
            )));
        }
    };

    if let Some(instrument_asset_type) = instrument_asset_type
        && instrument_asset_type != asset_type
    {
        return Err(conversion_error(format!(
            "orderLegCollection[{index}].instrument assetType is {instrument_asset_type:?}, expected {asset_type:?}"
        )));
    }

    Ok(Leg {
        instruction,
        quantity,
        instrument: LegInstrument { symbol, asset_type },
    })
}

fn reject_unsupported_leg_fields(index: usize, leg: &OrderLegCollection) -> Result<()> {
    let fields = [
        (leg.quantity_type.is_some(), "quantityType"),
        (leg.position_effect.is_some(), "positionEffect"),
        (leg.div_cap_gains.is_some(), "divCapGains"),
        (leg.to_symbol.is_some(), "toSymbol"),
    ];

    for (present, field) in fields {
        if present {
            return Err(conversion_error(format!(
                "unsupported orderLegCollection[{index}].{field}"
            )));
        }
    }

    Ok(())
}

fn instrument_symbol_and_asset(
    index: usize,
    instrument: &AccountsInstrument,
) -> Result<(String, Option<InstrumentAssetType>)> {
    match instrument {
        AccountsInstrument::Equity(instrument) => {
            symbol_and_asset(index, &instrument.symbol, &instrument.asset_type)
        }
        AccountsInstrument::Option(instrument) => {
            symbol_and_asset(index, &instrument.symbol, &instrument.asset_type)
        }
        AccountsInstrument::FixedIncome(instrument) => {
            symbol_and_asset(index, &instrument.symbol, &instrument.asset_type)
        }
        AccountsInstrument::CashEquivalent(instrument) => {
            symbol_and_asset(index, &instrument.symbol, &instrument.asset_type)
        }
        AccountsInstrument::MutualFund(instrument) => {
            symbol_and_asset(index, &instrument.symbol, &instrument.asset_type)
        }
    }
}

fn symbol_and_asset(
    index: usize,
    symbol: &Option<String>,
    asset_type: &Option<InstrumentAssetType>,
) -> Result<(String, Option<InstrumentAssetType>)> {
    let symbol = required(
        symbol.clone(),
        format!("orderLegCollection[{index}].instrument.symbol"),
    )?;

    if symbol.trim().is_empty() {
        return Err(conversion_error(format!(
            "orderLegCollection[{index}].instrument symbol is empty"
        )));
    }

    Ok((symbol, asset_type.clone()))
}
