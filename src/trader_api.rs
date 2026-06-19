use reqwest::Method;
use serde::Serialize;
use tracing::instrument;

use crate::client::ApiBase;
use crate::models::trader::{
    Account, AccountNumberHash, Order, PreviewOrder, Transaction, UserPreference,
};
use crate::query::{push_optional, required_text};
use crate::{Client, Error, OrderListOptions, OrderResponse, Result, TransactionListOptions};

impl Client {
    /// Fetches all linked accounts from `GET /accounts`.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the request fails or the response cannot be decoded.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> schwab::Result<()> {
    /// use schwab::{Client, Config};
    ///
    /// let client = Client::new(Config::new().bearer_token("my-token"));
    /// let accounts = client.get_accounts(Some("positions")).await?;
    /// for account in &accounts {
    ///     println!("{account:?}");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip_all)]
    pub async fn get_accounts(&self, fields: Option<&str>) -> Result<Vec<Account>> {
        let url = self.endpoint_url(ApiBase::Trader, &["accounts"])?;
        let mut query = Vec::new();
        push_optional(&mut query, "fields", fields);
        self.send_json(Method::GET, url, &query, None).await
    }

    /// Fetches account-number hash mappings from `GET /accounts/accountNumbers`.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the request fails or the response cannot be decoded.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> schwab::Result<()> {
    /// use schwab::{Client, Config};
    ///
    /// let client = Client::new(Config::new().bearer_token("my-token"));
    /// let mappings = client.get_account_numbers().await?;
    /// for mapping in &mappings {
    ///     println!("{mapping:?}");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip_all)]
    pub async fn get_account_numbers(&self) -> Result<Vec<AccountNumberHash>> {
        let url = self.endpoint_url(ApiBase::Trader, &["accounts", "accountNumbers"])?;
        self.send_json(Method::GET, url, &[], None).await
    }

    /// Fetches one account from `GET /accounts/{accountNumber}`.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::MissingRequiredParameter`] if `account_number` is empty.
    /// Returns an [`Error`](crate::Error) if the request fails or the response cannot be decoded.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> schwab::Result<()> {
    /// use schwab::{Client, Config};
    ///
    /// let client = Client::new(Config::new().bearer_token("my-token"));
    /// let account = client.get_account("account-hash", Some("positions")).await?;
    /// println!("{account:?}");
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip_all)]
    pub async fn get_account(
        &self,
        account_number: impl AsRef<str>,
        fields: Option<&str>,
    ) -> Result<Account> {
        let account_number = required_text("accountNumber", account_number.as_ref())?;
        let url = self.endpoint_url(ApiBase::Trader, &["accounts", account_number])?;
        let mut query = Vec::new();
        push_optional(&mut query, "fields", fields);
        self.send_json(Method::GET, url, &query, None).await
    }

    /// Fetches orders for one account from `GET /accounts/{accountNumber}/orders`.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::MissingRequiredParameter`] if `account_number` is empty.
    /// Returns an [`Error`](crate::Error) if the request fails or the response cannot be decoded.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> schwab::Result<()> {
    /// use schwab::{Client, Config, OrderListOptions};
    ///
    /// let client = Client::new(Config::new().bearer_token("my-token"));
    /// let orders = client
    ///     .get_orders(
    ///         "account-hash",
    ///         OrderListOptions::new("2024-01-01T00:00:00Z", "2024-01-31T00:00:00Z")
    ///             .status("FILLED"),
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip_all)]
    pub async fn get_orders(
        &self,
        account_number: impl AsRef<str>,
        options: OrderListOptions,
    ) -> Result<Vec<Order>> {
        let account_number = required_text("accountNumber", account_number.as_ref())?;
        let url = self.endpoint_url(ApiBase::Trader, &["accounts", account_number, "orders"])?;
        self.send_json(Method::GET, url, &options.into_query()?, None)
            .await
    }

    /// Places an order with `POST /accounts/{accountNumber}/orders`.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::MissingRequiredParameter`] if `account_number` is empty.
    /// Returns an [`Error`](crate::Error) if the request fails or the response cannot be decoded.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> schwab::Result<()> {
    /// use schwab::{Client, Config, Instruction, OrderBuilder};
    ///
    /// let client = Client::new(Config::new().bearer_token("my-token"));
    /// let quantity = "10".parse().unwrap();
    /// let order = OrderBuilder::equity_market("AAPL", Instruction::Buy, quantity);
    /// let response = client.place_order("account-hash", &order).await?;
    /// println!("order id: {:?}", response.order_id);
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip_all)]
    pub async fn place_order<B>(
        &self,
        account_number: impl AsRef<str>,
        order: &B,
    ) -> Result<OrderResponse>
    where
        B: Serialize + ?Sized,
    {
        let account_number = required_text("accountNumber", account_number.as_ref())?;
        let url = self.endpoint_url(ApiBase::Trader, &["accounts", account_number, "orders"])?;
        self.send_empty_with_location(Method::POST, url, order)
            .await
    }

    /// Cancels an order with `DELETE /accounts/{accountNumber}/orders/{orderId}`.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::MissingRequiredParameter`] if `account_number` is empty.
    /// Returns an [`Error`](crate::Error) if the request fails or the response cannot be decoded.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> schwab::Result<()> {
    /// use schwab::{Client, Config};
    ///
    /// let client = Client::new(Config::new().bearer_token("my-token"));
    /// client.cancel_order("account-hash", 9001).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip_all)]
    pub async fn cancel_order(&self, account_number: impl AsRef<str>, order_id: i64) -> Result<()> {
        let account_number = required_text("accountNumber", account_number.as_ref())?;
        let order_id = order_id.to_string();
        let url = self.endpoint_url(
            ApiBase::Trader,
            &["accounts", account_number, "orders", &order_id],
        )?;
        self.send_empty(Method::DELETE, url).await
    }

    /// Fetches one order with `GET /accounts/{accountNumber}/orders/{orderId}`.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::MissingRequiredParameter`] if `account_number` is empty.
    /// Returns an [`Error`](crate::Error) if the request fails or the response cannot be decoded.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> schwab::Result<()> {
    /// use schwab::{Client, Config};
    ///
    /// let client = Client::new(Config::new().bearer_token("my-token"));
    /// let order = client.get_order("account-hash", 9001).await?;
    /// println!("{order:?}");
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip_all)]
    pub async fn get_order(&self, account_number: impl AsRef<str>, order_id: i64) -> Result<Order> {
        let account_number = required_text("accountNumber", account_number.as_ref())?;
        let order_id = order_id.to_string();
        let url = self.endpoint_url(
            ApiBase::Trader,
            &["accounts", account_number, "orders", &order_id],
        )?;
        self.send_json(Method::GET, url, &[], None).await
    }

    /// Replaces an order with `PUT /accounts/{accountNumber}/orders/{orderId}`.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::MissingRequiredParameter`] if `account_number` is empty.
    /// Returns an [`Error`](crate::Error) if the request fails or the response cannot be decoded.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> schwab::Result<()> {
    /// use schwab::{Client, Config, Instruction, OrderBuilder};
    ///
    /// let client = Client::new(Config::new().bearer_token("my-token"));
    /// let quantity = "10".parse().unwrap();
    /// let price = "150.00".parse().unwrap();
    /// let order = OrderBuilder::equity_limit("AAPL", Instruction::Buy, quantity, price);
    /// let response = client.replace_order("account-hash", 9001, &order).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip_all)]
    pub async fn replace_order<B>(
        &self,
        account_number: impl AsRef<str>,
        order_id: i64,
        order: &B,
    ) -> Result<OrderResponse>
    where
        B: Serialize + ?Sized,
    {
        let account_number = required_text("accountNumber", account_number.as_ref())?;
        let order_id_text = order_id.to_string();
        let url = self.endpoint_url(
            ApiBase::Trader,
            &["accounts", account_number, "orders", &order_id_text],
        )?;
        let mut response = self
            .send_empty_with_location(Method::PUT, url, order)
            .await?;
        if response.order_id.is_none() {
            response.order_id = Some(order_id);
        }
        Ok(response)
    }

    /// Previews an order with `POST /accounts/{accountNumber}/previewOrder`.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::MissingRequiredParameter`] if `account_number` is empty.
    /// Returns an [`Error`](crate::Error) if the request fails or the response cannot be decoded.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> schwab::Result<()> {
    /// use schwab::{Client, Config, Instruction, OrderBuilder};
    ///
    /// let client = Client::new(Config::new().bearer_token("my-token"));
    /// let quantity = "10".parse().unwrap();
    /// let order = OrderBuilder::equity_market("AAPL", Instruction::Buy, quantity);
    /// let preview = client.preview_order("account-hash", &order).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip_all)]
    pub async fn preview_order<B>(
        &self,
        account_number: impl AsRef<str>,
        order: &B,
    ) -> Result<PreviewOrder>
    where
        B: Serialize + ?Sized,
    {
        let account_number = required_text("accountNumber", account_number.as_ref())?;
        let url = self.endpoint_url(
            ApiBase::Trader,
            &["accounts", account_number, "previewOrder"],
        )?;
        self.send_json(
            Method::POST,
            url,
            &[],
            Some(serde_json::to_value(order).map_err(Error::Encode)?),
        )
        .await
    }

    /// Fetches account transactions from `GET /accounts/{accountNumber}/transactions`.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::MissingRequiredParameter`] if `account_number` is empty.
    /// Returns an [`Error`](crate::Error) if the request fails or the response cannot be decoded.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> schwab::Result<()> {
    /// use schwab::{Client, Config, TransactionListOptions};
    ///
    /// let client = Client::new(Config::new().bearer_token("my-token"));
    /// let transactions = client
    ///     .get_transactions(
    ///         "account-hash",
    ///         TransactionListOptions::new(
    ///             "2024-01-01T00:00:00Z",
    ///             "2024-01-31T00:00:00Z",
    ///             "TRADE",
    ///         ),
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip_all)]
    pub async fn get_transactions(
        &self,
        account_number: impl AsRef<str>,
        options: TransactionListOptions,
    ) -> Result<Vec<Transaction>> {
        let account_number = required_text("accountNumber", account_number.as_ref())?;
        let url = self.endpoint_url(
            ApiBase::Trader,
            &["accounts", account_number, "transactions"],
        )?;
        self.send_json(Method::GET, url, &options.into_query()?, None)
            .await
    }

    /// Fetches one transaction from `GET /accounts/{accountNumber}/transactions/{transactionId}`.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::MissingRequiredParameter`] if `account_number` is empty.
    /// Returns an [`Error`](crate::Error) if the request fails or the response cannot be decoded.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> schwab::Result<()> {
    /// use schwab::{Client, Config};
    ///
    /// let client = Client::new(Config::new().bearer_token("my-token"));
    /// let transactions = client.get_transaction_by_id("account-hash", 123).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip_all)]
    pub async fn get_transaction_by_id(
        &self,
        account_number: impl AsRef<str>,
        transaction_id: i64,
    ) -> Result<Vec<Transaction>> {
        let account_number = required_text("accountNumber", account_number.as_ref())?;
        let transaction_id = transaction_id.to_string();
        let url = self.endpoint_url(
            ApiBase::Trader,
            &["accounts", account_number, "transactions", &transaction_id],
        )?;
        self.send_json(Method::GET, url, &[], None).await
    }

    /// Fetches orders across all accounts from `GET /orders`.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the request fails or the response cannot be decoded.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> schwab::Result<()> {
    /// use schwab::{Client, Config, OrderListOptions};
    ///
    /// let client = Client::new(Config::new().bearer_token("my-token"));
    /// let orders = client
    ///     .get_all_orders(OrderListOptions::new(
    ///         "2024-01-01T00:00:00Z",
    ///         "2024-01-31T00:00:00Z",
    ///     ))
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip_all)]
    pub async fn get_all_orders(&self, options: OrderListOptions) -> Result<Vec<Order>> {
        let url = self.endpoint_url(ApiBase::Trader, &["orders"])?;
        self.send_json(Method::GET, url, &options.into_query()?, None)
            .await
    }

    /// Fetches user preferences from `GET /userPreference`.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) if the request fails or the response cannot be decoded.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # async fn example() -> schwab::Result<()> {
    /// use schwab::{Client, Config};
    ///
    /// let client = Client::new(Config::new().bearer_token("my-token"));
    /// let prefs = client.get_user_preference().await?;
    /// # Ok(())
    /// # }
    /// ```
    #[instrument(skip_all)]
    pub async fn get_user_preference(&self) -> Result<Vec<UserPreference>> {
        let url = self.endpoint_url(ApiBase::Trader, &["userPreference"])?;
        self.send_json(Method::GET, url, &[], None).await
    }
}

#[cfg(test)]
mod tests {
    use mockito::Matcher;
    use serde_json::json;

    use crate::*;

    #[tokio::test]
    async fn get_account_uses_trader_base_url_path_escaping_and_bearer_auth() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/accounts/HASH%2FABC")
            .match_query(Matcher::UrlEncoded("fields".into(), "positions".into()))
            .match_header("authorization", "Bearer TOKEN")
            .with_status(200)
            .with_body(r#"{"securitiesAccount":{"type":"CASH"}}"#)
            .create_async()
            .await;

        let url = server.url();
        let config = Config::new()
            .trader_base_url(&url)
            .unwrap()
            .bearer_token("TOKEN");
        let client = Client::new(config);
        client
            .get_account("HASH/ABC", Some("positions"))
            .await
            .unwrap();

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn place_order_sends_json_body_and_parses_location() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/accounts/HASH/orders")
            .match_body(Matcher::Json(json!({"orderType":"MARKET"})))
            .with_status(201)
            .with_header("Location", "/trader/v1/accounts/HASH/orders/9001")
            .with_body("")
            .create_async()
            .await;

        let url = server.url();
        let client = Client::new(Config::new().trader_base_url(&url).unwrap());
        let response = client
            .place_order("HASH", &json!({"orderType":"MARKET"}))
            .await
            .unwrap();

        mock.assert_async().await;
        assert_eq!(response.order_id, Some(9001));
    }

    #[tokio::test]
    async fn endpoint_request_shapes_are_covered() {
        let mut server = mockito::Server::new_async().await;
        let url = server.url();
        let client = Client::new(Config::new().trader_base_url(&url).unwrap());

        // get_accounts with fields
        let m = server
            .mock("GET", "/accounts")
            .match_query(Matcher::UrlEncoded("fields".into(), "positions".into()))
            .with_status(200)
            .with_body("[]")
            .create_async()
            .await;
        client.get_accounts(Some("positions")).await.unwrap();
        m.assert_async().await;

        // get_account_numbers
        let m = server
            .mock("GET", "/accounts/accountNumbers")
            .with_status(200)
            .with_body("[]")
            .create_async()
            .await;
        client.get_account_numbers().await.unwrap();
        m.assert_async().await;

        // get_orders with query params
        let m = server
            .mock("GET", "/accounts/HASH/orders")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("fromEnteredTime".into(), "2024-01-01T00:00:00Z".into()),
                Matcher::UrlEncoded("toEnteredTime".into(), "2024-01-31T00:00:00Z".into()),
                Matcher::UrlEncoded("maxResults".into(), "10".into()),
                Matcher::UrlEncoded("status".into(), "FILLED".into()),
            ]))
            .with_status(200)
            .with_body("[]")
            .create_async()
            .await;
        client
            .get_orders(
                "HASH",
                OrderListOptions::new("2024-01-01T00:00:00Z", "2024-01-31T00:00:00Z")
                    .max_results(10)
                    .status("FILLED"),
            )
            .await
            .unwrap();
        m.assert_async().await;

        // cancel_order
        let m = server
            .mock("DELETE", "/accounts/HASH/orders/9001")
            .with_status(204)
            .with_body("")
            .create_async()
            .await;
        client.cancel_order("HASH", 9001).await.unwrap();
        m.assert_async().await;

        // get_order
        let m = server
            .mock("GET", "/accounts/HASH/orders/9001")
            .with_status(200)
            .with_body(r#"{"orderId":9001}"#)
            .create_async()
            .await;
        client.get_order("HASH", 9001).await.unwrap();
        m.assert_async().await;

        // replace_order
        let m = server
            .mock("PUT", "/accounts/HASH/orders/9001")
            .match_body(Matcher::Json(json!({"orderType":"LIMIT"})))
            .with_status(201)
            .with_body("")
            .create_async()
            .await;
        let response = client
            .replace_order("HASH", 9001, &json!({"orderType":"LIMIT"}))
            .await
            .unwrap();
        m.assert_async().await;
        assert_eq!(response.order_id, Some(9001));

        // preview_order
        let m = server
            .mock("POST", "/accounts/HASH/previewOrder")
            .match_body(Matcher::Json(json!({"orderType":"MARKET"})))
            .with_status(200)
            .with_body(r#"{"previewId":"abc"}"#)
            .create_async()
            .await;
        client
            .preview_order("HASH", &json!({"orderType":"MARKET"}))
            .await
            .unwrap();
        m.assert_async().await;

        // get_transactions
        let m = server
            .mock("GET", "/accounts/HASH/transactions")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("startDate".into(), "2024-01-01T00:00:00Z".into()),
                Matcher::UrlEncoded("endDate".into(), "2024-01-31T00:00:00Z".into()),
                Matcher::UrlEncoded("types".into(), "TRADE".into()),
                Matcher::UrlEncoded("symbol".into(), "AAPL".into()),
            ]))
            .with_status(200)
            .with_body("[]")
            .create_async()
            .await;
        client
            .get_transactions(
                "HASH",
                TransactionListOptions::new(
                    "2024-01-01T00:00:00Z",
                    "2024-01-31T00:00:00Z",
                    "TRADE",
                )
                .symbol("AAPL"),
            )
            .await
            .unwrap();
        m.assert_async().await;

        // get_transaction_by_id
        let m = server
            .mock("GET", "/accounts/HASH/transactions/123")
            .with_status(200)
            .with_body(r#"[{"transactionId":123}]"#)
            .create_async()
            .await;
        client.get_transaction_by_id("HASH", 123).await.unwrap();
        m.assert_async().await;

        // get_all_orders
        let m = server
            .mock("GET", "/orders")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("fromEnteredTime".into(), "2024-01-01T00:00:00Z".into()),
                Matcher::UrlEncoded("toEnteredTime".into(), "2024-01-31T00:00:00Z".into()),
            ]))
            .with_status(200)
            .with_body("[]")
            .create_async()
            .await;
        client
            .get_all_orders(OrderListOptions::new(
                "2024-01-01T00:00:00Z",
                "2024-01-31T00:00:00Z",
            ))
            .await
            .unwrap();
        m.assert_async().await;

        // get_user_preference
        let m = server
            .mock("GET", "/userPreference")
            .with_status(200)
            .with_body(r#"[{"streamerInfo":[]}]"#)
            .create_async()
            .await;
        client.get_user_preference().await.unwrap();
        m.assert_async().await;
    }
}
