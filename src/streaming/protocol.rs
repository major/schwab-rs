//! Schwab streaming protocol request and response types.
//!
//! These types model the JSON wire format used over the WebSocket
//! connection. They are crate-internal and not exposed to users.

use serde::{Deserialize, Serialize};

// ── Request types ──────────────────────────────────────────────

/// Top-level request wrapper sent over the WebSocket.
#[allow(missing_docs)]
#[derive(Debug, Serialize)]
pub(crate) struct StreamRequest {
    pub(crate) requests: Vec<StreamRequestItem>,
}

/// Individual command within a streaming request.
#[allow(missing_docs)]
#[derive(Debug, Serialize)]
pub(crate) struct StreamRequestItem {
    pub(crate) requestid: String,
    pub(crate) service: String,
    pub(crate) command: String,
    #[serde(rename = "SchwabClientCustomerId")]
    pub(crate) schwab_client_customer_id: String,
    #[serde(rename = "SchwabClientCorrelId")]
    pub(crate) schwab_client_correl_id: String,
    pub(crate) parameters: StreamParameters,
}

/// Parameters for a streaming command.
///
/// LOGIN requests use `authorization`, `schwab_client_channel`, and
/// `schwab_client_function_id`. SUBS/ADD requests use `keys` and `fields`.
#[allow(missing_docs)]
#[derive(Debug, Default, Serialize)]
pub(crate) struct StreamParameters {
    #[serde(rename = "Authorization", skip_serializing_if = "Option::is_none")]
    pub(crate) authorization: Option<String>,
    #[serde(rename = "SchwabClientChannel", skip_serializing_if = "Option::is_none")]
    pub(crate) schwab_client_channel: Option<String>,
    #[serde(rename = "SchwabClientFunctionId", skip_serializing_if = "Option::is_none")]
    pub(crate) schwab_client_function_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) keys: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) fields: Option<String>,
}

// ── Response types ─────────────────────────────────────────────

/// Top-level message received from the streaming server.
///
/// Each message contains one or more of `response`, `notify`, or `data`
/// arrays depending on the event type.
#[allow(missing_docs)]
#[derive(Debug, Deserialize)]
pub(crate) struct StreamMessage {
    pub(crate) response: Option<Vec<StreamResponseMessage>>,
    pub(crate) notify: Option<Vec<StreamNotifyMessage>>,
    pub(crate) data: Option<Vec<StreamDataMessage>>,
}

/// Command acknowledgment from the server.
#[allow(missing_docs)]
#[derive(Debug, Deserialize)]
pub(crate) struct StreamResponseMessage {
    pub(crate) service: Option<String>,
    pub(crate) command: Option<String>,
    pub(crate) requestid: Option<String>,
    #[serde(rename = "SchwabClientCorrelId")]
    pub(crate) schwab_client_correl_id: Option<String>,
    pub(crate) timestamp: Option<i64>,
    pub(crate) content: Option<StreamResponseContent>,
}

/// Result code and message from a command response.
#[allow(missing_docs)]
#[derive(Debug, Deserialize)]
pub(crate) struct StreamResponseContent {
    pub(crate) code: Option<u32>,
    pub(crate) msg: Option<String>,
}

/// Heartbeat notification from the server.
#[allow(missing_docs)]
#[derive(Debug, Deserialize)]
pub(crate) struct StreamNotifyMessage {
    pub(crate) heartbeat: Option<String>,
}

/// Market data payload from the server.
#[allow(missing_docs)]
#[derive(Debug, Deserialize)]
pub(crate) struct StreamDataMessage {
    pub(crate) service: Option<String>,
    pub(crate) timestamp: Option<i64>,
    pub(crate) command: Option<String>,
    pub(crate) content: Option<Vec<serde_json::Map<String, serde_json::Value>>>,
}
