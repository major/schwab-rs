//! Streaming event types for the Schwab WebSocket API.

/// Account activity streaming data.
pub mod acct_activity;
/// Equity chart streaming data.
pub mod chart_equity;
/// Futures chart streaming data.
pub mod chart_futures;
/// Level-one equity streaming data.
pub mod equities;
/// Level-one forex streaming data.
pub mod forex;
/// Level-one futures streaming data.
pub mod futures;
/// Level-one futures option streaming data.
pub mod futures_options;
/// Level-one option streaming data.
pub mod options;
/// Equity screener streaming data.
pub mod screener_equity;
/// Option screener streaming data.
pub mod screener_option;

pub use acct_activity::{AccountActivity, AccountActivityField};
pub use chart_equity::{ChartEquity, ChartEquityField};
pub use chart_futures::{ChartFutures, ChartFuturesField};
pub use equities::{EquityField, LevelOneEquity};
pub use forex::{ForexField, LevelOneForex};
pub use futures::{FuturesField, LevelOneFutures};
pub use futures_options::{FuturesOptionField, LevelOneFuturesOption};
pub use options::{LevelOneOption, OptionField};
pub use screener_equity::{ScreenerEquity, ScreenerEquityField, ScreenerItem};
pub use screener_option::{ScreenerOption, ScreenerOptionField};

/// Event received from the Schwab streaming WebSocket connection.
///
/// # Examples
///
/// ```
/// use schwab::{StreamEvent, StreamData};
///
/// fn handle(event: StreamEvent) {
///     match event {
///         StreamEvent::Data(data) => println!("data: {data:?}"),
///         StreamEvent::Heartbeat(ts) => println!("heartbeat {ts}"),
///         _ => {}
///     }
/// }
/// ```
#[non_exhaustive]
#[derive(Clone, Debug)]
#[allow(missing_docs)]
pub enum StreamEvent {
    Data(StreamData),
    Response(StreamResponse),
    Heartbeat(i64),
    Disconnected { error: Option<String> },
    Reconnecting { attempt: u32 },
    Reconnected,
}

/// Streaming payload delivered within a [`StreamEvent::Data`] event.
///
/// # Examples
///
/// ```
/// use schwab::{StreamData, LevelOneEquity};
///
/// fn handle(data: StreamData) {
///     if let StreamData::LevelOneEquities(updates) = data {
///         for update in updates {
///             println!("{:?}", update.symbol);
///         }
///     }
/// }
/// ```
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
#[allow(missing_docs)]
pub enum StreamData {
    AccountActivity(Vec<AccountActivity>),
    LevelOneEquities(Vec<LevelOneEquity>),
    LevelOneOptions(Vec<LevelOneOption>),
    LevelOneFutures(Vec<LevelOneFutures>),
    LevelOneFuturesOptions(Vec<LevelOneFuturesOption>),
    LevelOneForex(Vec<LevelOneForex>),
    ChartEquities(Vec<ChartEquity>),
    ChartFutures(Vec<ChartFutures>),
    ScreenerEquities(Vec<ScreenerEquity>),
    ScreenerOptions(Vec<ScreenerOption>),
}

/// Acknowledgement returned by the Schwab streaming server for a command request.
///
/// # Examples
///
/// ```
/// use schwab::StreamResponse;
///
/// let resp = StreamResponse {
///     service: Some("LEVELONE_EQUITIES".to_string()),
///     command: Some("SUBS".to_string()),
///     request_id: None,
///     code: Some(0),
///     message: Some("OK".to_string()),
/// };
/// assert_eq!(resp.code, Some(0));
/// ```
#[derive(Clone, Debug, PartialEq)]
#[allow(missing_docs)]
pub struct StreamResponse {
    pub service: Option<String>,
    pub command: Option<String>,
    pub request_id: Option<String>,
    pub code: Option<u32>,
    pub message: Option<String>,
}
