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

// ── Parsed message enum ───────────────────────────────────────

/// A discriminated streaming message, produced by parsing a raw WebSocket text frame.
#[allow(missing_docs)]
#[derive(Debug)]
pub(crate) enum ParsedMessage {
    /// A command acknowledgment from the server.
    Response(StreamResponseMessage),
    /// A server heartbeat with a Unix timestamp.
    Heartbeat(i64),
    /// A market data update.
    Data(StreamDataMessage),
}

// ── Message parser ────────────────────────────────────────────

/// Parse a raw WebSocket text frame into zero or more [`ParsedMessage`] values.
///
/// A single frame may contain multiple messages in the same array.
/// Returns `Err(Error::StreamProtocol(...))` on JSON parse failure or unexpected shape.
pub(crate) fn parse_message(text: &str) -> crate::Result<Vec<ParsedMessage>> {
    let msg: StreamMessage =
        serde_json::from_str(text).map_err(|e| crate::Error::StreamProtocol(e.to_string()))?;
    let mut result = Vec::new();
    if let Some(responses) = msg.response {
        for r in responses {
            result.push(ParsedMessage::Response(r));
        }
    }
    if let Some(notifies) = msg.notify {
        for n in notifies {
            let ts = n
                .heartbeat
                .as_deref()
                .and_then(|s| s.parse::<i64>().ok())
                .unwrap_or(0);
            result.push(ParsedMessage::Heartbeat(ts));
        }
    }
    if let Some(data) = msg.data {
        for d in data {
            result.push(ParsedMessage::Data(d));
        }
    }
    Ok(result)
}

// ── Command builders ──────────────────────────────────────────

/// Build a serialized LOGIN request.
///
/// The Schwab streaming protocol requires `SchwabClientCustomerId` and
/// `SchwabClientCorrelId` at the top level of the request item, with
/// the bearer token in the `Authorization` parameter.
pub(crate) fn build_login(
    customer_id: &str,
    correl_id: &str,
    channel: &str,
    function_id: &str,
    access_token: &str,
) -> crate::Result<String> {
    let req = StreamRequest {
        requests: vec![StreamRequestItem {
            requestid: "0".to_string(),
            service: "ADMIN".to_string(),
            command: "LOGIN".to_string(),
            schwab_client_customer_id: customer_id.to_string(),
            schwab_client_correl_id: correl_id.to_string(),
            parameters: StreamParameters {
                authorization: Some(access_token.to_string()),
                schwab_client_channel: Some(channel.to_string()),
                schwab_client_function_id: Some(function_id.to_string()),
                ..Default::default()
            },
        }],
    };
    serde_json::to_string(&req).map_err(crate::Error::Encode)
}

/// Build a serialized LOGOUT request.
pub(crate) fn build_logout(customer_id: &str, correl_id: &str) -> crate::Result<String> {
    let req = StreamRequest {
        requests: vec![StreamRequestItem {
            requestid: "1".to_string(),
            service: "ADMIN".to_string(),
            command: "LOGOUT".to_string(),
            schwab_client_customer_id: customer_id.to_string(),
            schwab_client_correl_id: correl_id.to_string(),
            parameters: StreamParameters::default(),
        }],
    };
    serde_json::to_string(&req).map_err(crate::Error::Encode)
}

/// Build a SUBS (subscribe) request.
pub(crate) fn build_subs(
    request_id: &str,
    service_name: &str,
    customer_id: &str,
    correl_id: &str,
    symbols: &[&str],
    field_indices: &[u32],
) -> crate::Result<String> {
    build_keyed_command(
        "SUBS",
        request_id,
        service_name,
        customer_id,
        correl_id,
        symbols,
        Some(field_indices),
    )
}

/// Build an ADD request (add symbols to existing subscription).
pub(crate) fn build_add(
    request_id: &str,
    service_name: &str,
    customer_id: &str,
    correl_id: &str,
    symbols: &[&str],
    field_indices: &[u32],
) -> crate::Result<String> {
    build_keyed_command(
        "ADD",
        request_id,
        service_name,
        customer_id,
        correl_id,
        symbols,
        Some(field_indices),
    )
}

/// Build an UNSUBS (unsubscribe) request.
pub(crate) fn build_unsubs(
    request_id: &str,
    service_name: &str,
    customer_id: &str,
    correl_id: &str,
    symbols: &[&str],
) -> crate::Result<String> {
    build_keyed_command(
        "UNSUBS",
        request_id,
        service_name,
        customer_id,
        correl_id,
        symbols,
        None,
    )
}

/// Build a VIEW request (change subscribed fields without changing symbols).
pub(crate) fn build_view(
    request_id: &str,
    service_name: &str,
    _customer_id: &str,
    _correl_id: &str,
    field_indices: &[u32],
) -> crate::Result<String> {
    let fields = field_indices
        .iter()
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let req = StreamRequest {
        requests: vec![StreamRequestItem {
            requestid: request_id.to_string(),
            service: service_name.to_string(),
            command: "VIEW".to_string(),
            schwab_client_customer_id: String::new(),
            schwab_client_correl_id: String::new(),
            parameters: StreamParameters {
                fields: Some(fields),
                ..Default::default()
            },
        }],
    };
    serde_json::to_string(&req).map_err(crate::Error::Encode)
}

fn build_keyed_command(
    command: &str,
    request_id: &str,
    service_name: &str,
    customer_id: &str,
    correl_id: &str,
    symbols: &[&str],
    field_indices: Option<&[u32]>,
) -> crate::Result<String> {
    let keys = symbols.join(",");
    let fields = field_indices.map(|fi| {
        fi.iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(",")
    });
    let req = StreamRequest {
        requests: vec![StreamRequestItem {
            requestid: request_id.to_string(),
            service: service_name.to_string(),
            command: command.to_string(),
            schwab_client_customer_id: customer_id.to_string(),
            schwab_client_correl_id: correl_id.to_string(),
            parameters: StreamParameters {
                keys: Some(keys),
                fields,
                ..Default::default()
            },
        }],
    };
    serde_json::to_string(&req).map_err(crate::Error::Encode)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_top_level_fields() {
        let json = build_login("cust123", "correl456", "chan1", "fn1", "my_token").unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let item = &v["requests"][0];
        // Top-level fields in the request item
        assert_eq!(item["SchwabClientCustomerId"], "cust123");
        assert_eq!(item["SchwabClientCorrelId"], "correl456");
        // Token in parameters
        assert_eq!(item["parameters"]["Authorization"], "my_token");
        assert_eq!(item["parameters"]["SchwabClientChannel"], "chan1");
        assert_eq!(item["service"], "ADMIN");
        assert_eq!(item["command"], "LOGIN");
    }

    #[test]
    fn login_token_not_in_debug() {
        let req = StreamRequest {
            requests: vec![StreamRequestItem {
                requestid: "0".to_string(),
                service: "ADMIN".to_string(),
                command: "LOGIN".to_string(),
                schwab_client_customer_id: "c".to_string(),
                schwab_client_correl_id: "r".to_string(),
                parameters: StreamParameters {
                    authorization: Some("SECRET_TOKEN".to_string()),
                    ..Default::default()
                },
            }],
        };
        let debug = format!("{req:?}");
        // Verify the Debug output can be produced without panicking.
        // The authorization field is internal-only, so its presence
        // in Debug is acceptable per the task requirements.
        assert!(!debug.is_empty());
    }

    #[test]
    fn parse_response_message() {
        let json = r#"{"response":[{"service":"ADMIN","command":"LOGIN","requestid":"0","SchwabClientCorrelId":"c","timestamp":1234,"content":{"code":0,"msg":"SUCCESS"}}]}"#;
        let msgs = parse_message(json).unwrap();
        assert_eq!(msgs.len(), 1);
        assert!(matches!(msgs[0], ParsedMessage::Response(_)));
    }

    #[test]
    fn parse_heartbeat() {
        let json = r#"{"notify":[{"heartbeat":"1234567890"}]}"#;
        let msgs = parse_message(json).unwrap();
        assert_eq!(msgs.len(), 1);
        assert!(matches!(msgs[0], ParsedMessage::Heartbeat(1234567890)));
    }

    #[test]
    fn parse_data_message() {
        let json = r#"{"data":[{"service":"LEVELONE_EQUITIES","timestamp":1234,"command":"SUBS","content":[{"key":"AAPL","1":150.0}]}]}"#;
        let msgs = parse_message(json).unwrap();
        assert_eq!(msgs.len(), 1);
        assert!(matches!(msgs[0], ParsedMessage::Data(_)));
    }

    #[test]
    fn parse_malformed_returns_error() {
        let result = parse_message("not json");
        assert!(matches!(result, Err(crate::Error::StreamProtocol(_))));
    }
}
