//! Streaming event types for the Schwab WebSocket API.

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

pub use equities::{EquityField, LevelOneEquity};
pub use forex::{ForexField, LevelOneForex};
pub use futures::{FuturesField, LevelOneFutures};
pub use futures_options::{FuturesOptionField, LevelOneFuturesOption};
pub use options::{LevelOneOption, OptionField};

/// Event received from the Schwab streaming WebSocket connection.
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

/// Market data payload delivered within a [`StreamEvent::Data`] event.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
#[allow(missing_docs)]
pub enum StreamData {
    LevelOneEquities(Vec<LevelOneEquity>),
    LevelOneOptions(Vec<LevelOneOption>),
    LevelOneFutures(Vec<LevelOneFutures>),
    LevelOneFuturesOptions(Vec<LevelOneFuturesOption>),
    LevelOneForex(Vec<LevelOneForex>),
}

/// Acknowledgement returned by the Schwab streaming server for a command request.
#[derive(Clone, Debug, PartialEq)]
#[allow(missing_docs)]
pub struct StreamResponse {
    pub service: Option<String>,
    pub command: Option<String>,
    pub request_id: Option<String>,
    pub code: Option<u32>,
    pub message: Option<String>,
}
