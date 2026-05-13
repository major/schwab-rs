use serde::{Deserialize, Serialize};

// --- Market Data Enums ---

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum AssetMainType {
    Bond,
    Equity,
    Forex,
    Future,
    FutureOption,
    Index,
    MutualFund,
    Option,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum MarketType {
    Bond,
    Equity,
    Etf,
    Extended,
    Forex,
    Future,
    FutureOption,
    Fundamental,
    Index,
    Indicator,
    MutualFund,
    Option,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[non_exhaustive]
pub enum ContractType {
    #[serde(rename = "P")]
    Put,
    #[serde(rename = "C")]
    Call,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[non_exhaustive]
pub enum EquityAssetSubType {
    #[serde(rename = "COE")]
    CommonEquity,
    #[serde(rename = "PRF")]
    Preferred,
    #[serde(rename = "ADR")]
    Adr,
    #[serde(rename = "GDR")]
    Gdr,
    #[serde(rename = "CEF")]
    ClosedEndFund,
    #[serde(rename = "ETF")]
    Etf,
    #[serde(rename = "ETN")]
    Etn,
    #[serde(rename = "UIT")]
    Uit,
    #[serde(rename = "WAR")]
    Warrant,
    #[serde(rename = "RGT")]
    Right,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[non_exhaustive]
pub enum MarketErrorStatus {
    #[serde(rename = "400")]
    BadRequest,
    #[serde(rename = "401")]
    Unauthorized,
    #[serde(rename = "404")]
    NotFound,
    #[serde(rename = "500")]
    InternalServerError,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[non_exhaustive]
pub enum ExerciseType {
    #[serde(rename = "A")]
    American,
    #[serde(rename = "E")]
    European,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[non_exhaustive]
pub enum ExpirationType {
    #[serde(rename = "M")]
    Monthly,
    #[serde(rename = "Q")]
    Quarterly,
    #[serde(rename = "S")]
    Standard,
    #[serde(rename = "W")]
    Weekly,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[non_exhaustive]
pub enum FundStrategy {
    #[serde(rename = "A")]
    Active,
    #[serde(rename = "L")]
    Leveraged,
    #[serde(rename = "P")]
    Passive,
    #[serde(rename = "Q")]
    Quantitative,
    #[serde(rename = "S")]
    Short,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum MutualFundAssetSubType {
    Oef,
    Cef,
    Mmf,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum OptionStrategy {
    Single,
    Analytical,
    Covered,
    Vertical,
    Calendar,
    Strangle,
    Straddle,
    Butterfly,
    Condor,
    Diagonal,
    Collar,
    Roll,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum PutCall {
    Put,
    Call,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum QuoteType {
    Nbbo,
    Nfl,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum Direction {
    Up,
    Down,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[non_exhaustive]
pub enum SettlementType {
    #[serde(rename = "A")]
    Am,
    #[serde(rename = "P")]
    Pm,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum ExchangeName {
    Ind,
    Ase,
    Nys,
    Nas,
    Nap,
    Pac,
    Opr,
    Bats,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum MarketDataMarket {
    Equity,
    Option,
    Bond,
    Future,
    Forex,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum MoverSymbol {
    #[serde(rename = "$DJI")]
    Dji,
    #[serde(rename = "$COMPX")]
    Compx,
    #[serde(rename = "$SPX")]
    Spx,
    Nyse,
    Nasdaq,
    Otcbb,
    IndexAll,
    EquityAll,
    OptionAll,
    OptionPut,
    OptionCall,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum OptionContractType {
    Call,
    Put,
    All,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum Entitlement {
    Pn,
    Np,
    Pp,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum ExpirationMonth {
    Jan,
    Feb,
    Mar,
    Apr,
    May,
    Jun,
    Jul,
    Aug,
    Sep,
    Oct,
    Nov,
    Dec,
    All,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum Sort {
    Volume,
    Trades,
    PercentChangeUp,
    PercentChangeDown,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum FrequencyType {
    Minute,
    Daily,
    Weekly,
    Monthly,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum PeriodType {
    Day,
    Month,
    Year,
    Ytd,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[non_exhaustive]
pub enum Projection {
    #[serde(rename = "symbol-search")]
    SymbolSearch,
    #[serde(rename = "symbol-regex")]
    SymbolRegex,
    #[serde(rename = "desc-search")]
    DescSearch,
    #[serde(rename = "desc-regex")]
    DescRegex,
    #[serde(rename = "search")]
    Search,
    #[serde(rename = "fundamental")]
    Fundamental,
}

// --- Trader API Enums ---

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum ApiRuleAction {
    Accept,
    Alert,
    Reject,
    Review,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum ApiCurrencyType {
    Usd,
    Cad,
    Eur,
    Jpy,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum CashEquivalentType {
    SweepVehicle,
    Savings,
    MoneyMarketFund,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum OptionPutCall {
    Put,
    Call,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum OptionType {
    Vanilla,
    Binary,
    Barrier,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum InstrumentAssetType {
    Equity,
    Option,
    Index,
    MutualFund,
    CashEquivalent,
    FixedIncome,
    Currency,
    CollectiveInvestment,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum CollectiveInvestmentType {
    UnitInvestmentTrust,
    ExchangeTradedFund,
    ClosedEndFund,
    Index,
    Units,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum FeeType {
    Commission,
    SecFee,
    StrFee,
    RFee,
    CdscFee,
    OptRegFee,
    AdditionalFee,
    MiscellaneousFee,
    Ftt,
    FuturesClearingFee,
    FuturesDeskOfficeFee,
    FuturesExchangeFee,
    FuturesGlobexFee,
    FuturesNfaFee,
    FuturesPitBrokerageFee,
    FuturesTransactionFee,
    LowProceedsCommission,
    BaseCharge,
    GeneralCharge,
    GstFee,
    TafFee,
    IndexOptionFee,
    TefraTax,
    StateTax,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum ForexType {
    Standard,
    Nbbo,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum FutureType {
    Standard,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum IndexType {
    BroadBased,
    NarrowBased,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum OrderActivityType {
    Execution,
    OrderAction,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum ExecutionType {
    Fill,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum DividendCapitalGains {
    Reinvest,
    Payout,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum PositionEffect {
    Opening,
    Closing,
    Automatic,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum QuantityType {
    AllShares,
    Dollars,
    Shares,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum AdvancedOrderType {
    None,
    Oto,
    Oco,
    Otoco,
    #[serde(rename = "OT2OCO")]
    Ot2Oco,
    #[serde(rename = "OT3OCO")]
    Ot3Oco,
    BlastAll,
    Ota,
    Pair,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum ProductType {
    Tbd,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum SecuritiesAccountType {
    Cash,
    Margin,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum TransactionActivityType {
    ActivityCorrection,
    Execution,
    OrderAction,
    Transfer,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum TransactionStatus {
    Valid,
    Invalid,
    Pending,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum SubAccount {
    Cash,
    Margin,
    Short,
    Div,
    Income,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum TransactionEquityType {
    CommonStock,
    PreferredStock,
    DepositoryReceipt,
    PreferredDepositoryReceipt,
    RestrictedStock,
    ComponentUnit,
    Right,
    Warrant,
    ConvertiblePreferredStock,
    ConvertibleStock,
    LimitedPartnership,
    WhenIssued,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum TransactionFixedIncomeType {
    BondUnit,
    CertificateOfDeposit,
    ConvertibleBond,
    CollateralizedMortgageObligation,
    CorporateBond,
    GovernmentMortgage,
    GnmaBonds,
    MunicipalAssessmentDistrict,
    MunicipalBond,
    OtherGovernment,
    ShortTermPaper,
    UsTreasuryBond,
    UsTreasuryBill,
    UsTreasuryNote,
    UsTreasuryZeroCoupon,
    AgencyBond,
    WhenAsAndIfIssuedBond,
    AssetBackedSecurity,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum TransactionMutualFundType {
    NotApplicable,
    OpenEndNonTaxable,
    OpenEndTaxable,
    NoLoadNonTaxable,
    NoLoadTaxable,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum TransactionType {
    Trade,
    ReceiveAndDeliver,
    DividendOrInterest,
    AchReceipt,
    AchDisbursement,
    CashReceipt,
    CashDisbursement,
    ElectronicFund,
    WireOut,
    WireIn,
    Journal,
    Memorandum,
    MarginCall,
    MoneyMarket,
    SmaAdjustment,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum UserType {
    AdvisorUser,
    BrokerUser,
    ClientUser,
    SystemUser,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum AmountIndicator {
    Dollars,
    Shares,
    AllShares,
    Percentage,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum OrderStatus {
    AwaitingParentOrder,
    AwaitingCondition,
    AwaitingStopCondition,
    AwaitingManualReview,
    Accepted,
    AwaitingUrOut,
    PendingActivation,
    Queued,
    Working,
    Rejected,
    PendingCancel,
    Canceled,
    PendingReplace,
    Replaced,
    Filled,
    Expired,
    New,
    AwaitingReleaseTime,
    PendingAcknowledgement,
    PendingRecall,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum AssetType {
    Equity,
    MutualFund,
    Option,
    Future,
    Forex,
    Index,
    CashEquivalent,
    FixedIncome,
    Product,
    Currency,
    CollectiveInvestment,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum ComplexOrderStrategyType {
    None,
    Covered,
    Vertical,
    BackRatio,
    Calendar,
    Diagonal,
    Straddle,
    Strangle,
    CollarSynthetic,
    Butterfly,
    Condor,
    IronCondor,
    VerticalRoll,
    CollarWithStock,
    DoubleDiagonal,
    UnbalancedButterfly,
    UnbalancedCondor,
    UnbalancedIronCondor,
    UnbalancedVerticalRoll,
    MutualFundSwap,
    Custom,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum Duration {
    Day,
    GoodTillCancel,
    FillOrKill,
    ImmediateOrCancel,
    EndOfWeek,
    EndOfMonth,
    NextEndOfMonth,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum Instruction {
    Buy,
    Sell,
    BuyToCover,
    SellShort,
    BuyToOpen,
    BuyToClose,
    SellToOpen,
    SellToClose,
    Exchange,
    SellShortExempt,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum OrderStrategyType {
    Single,
    Cancel,
    Recall,
    Pair,
    Flatten,
    TwoDaySwap,
    BlastAll,
    Oco,
    Trigger,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum OrderType {
    Market,
    Limit,
    Stop,
    StopLimit,
    TrailingStop,
    Cabinet,
    NonMarketable,
    MarketOnClose,
    Exercise,
    TrailingStopLimit,
    NetDebit,
    NetCredit,
    NetZero,
    LimitOnClose,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum OrderTypeRequest {
    Market,
    Limit,
    Stop,
    StopLimit,
    TrailingStop,
    Cabinet,
    NonMarketable,
    MarketOnClose,
    Exercise,
    TrailingStopLimit,
    NetDebit,
    NetCredit,
    NetZero,
    LimitOnClose,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum PriceLinkBasis {
    Manual,
    Base,
    Trigger,
    Last,
    Bid,
    Ask,
    AskBid,
    Mark,
    Average,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum PriceLinkType {
    Value,
    Percent,
    Tick,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum RequestedDestination {
    Inet,
    EcnArca,
    Cboe,
    Amex,
    Phlx,
    Ise,
    Box,
    Nyse,
    Nasdaq,
    Bats,
    C2,
    Auto,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum Session {
    Normal,
    Am,
    Pm,
    Seamless,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum SettlementInstruction {
    Regular,
    Cash,
    NextDay,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum SpecialInstruction {
    AllOrNone,
    DoNotReduce,
    AllOrNoneDoNotReduce,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum StopPriceLinkBasis {
    Manual,
    Base,
    Trigger,
    Last,
    Bid,
    Ask,
    AskBid,
    Mark,
    Average,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum StopPriceLinkType {
    Value,
    Percent,
    Tick,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum StopType {
    Standard,
    Bid,
    Ask,
    Last,
    Mark,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum TaxLotMethod {
    Fifo,
    Lifo,
    HighCost,
    LowCost,
    AverageCost,
    SpecificLot,
    LossHarvester,
}
