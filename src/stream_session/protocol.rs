//! Schwab streaming protocol request and response types.
//!
//! These types model the JSON wire format used over the WebSocket
//! connection. They are crate-internal and not exposed to users.

use serde::{Deserialize, Serialize};

// Request types

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
#[derive(Default, Serialize)]
pub(crate) struct StreamParameters {
    #[serde(rename = "Authorization", skip_serializing_if = "Option::is_none")]
    pub(crate) authorization: Option<String>,
    #[serde(
        rename = "SchwabClientChannel",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) schwab_client_channel: Option<String>,
    #[serde(
        rename = "SchwabClientFunctionId",
        skip_serializing_if = "Option::is_none"
    )]
    pub(crate) schwab_client_function_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) keys: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) fields: Option<String>,
}

impl std::fmt::Debug for StreamParameters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StreamParameters")
            .field(
                "authorization",
                &self.authorization.as_ref().map(|_| "<redacted>"),
            )
            .field("schwab_client_channel", &self.schwab_client_channel)
            .field("schwab_client_function_id", &self.schwab_client_function_id)
            .field("keys", &self.keys)
            .field("fields", &self.fields)
            .finish()
    }
}

// Response types

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
    #[serde(default, deserialize_with = "deserialize_optional_string")]
    pub(crate) requestid: Option<String>,
    #[serde(rename = "SchwabClientCorrelId")]
    // Keep server metadata in the wire model even though current session events
    // only expose service, command, request ID, code, and message.
    #[allow(dead_code)]
    pub(crate) schwab_client_correl_id: Option<String>,
    // Keep server metadata in the wire model for future diagnostics and parity
    // with Schwab protocol payloads.
    #[allow(dead_code)]
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
    // Preserve server metadata in the parsed data message even though dispatch
    // currently routes by service and content only.
    #[allow(dead_code)]
    pub(crate) timestamp: Option<i64>,
    // Preserve the original command for protocol parity and future diagnostics.
    #[allow(dead_code)]
    pub(crate) command: Option<String>,
    pub(crate) content: Option<Vec<serde_json::Map<String, serde_json::Value>>>,
}

// Parsed message enum

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

fn deserialize_optional_string<'de, D>(
    deserializer: D,
) -> std::result::Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    Ok(value.map(|value| match value {
        serde_json::Value::String(text) => text,
        other => other.to_string(),
    }))
}

// Message parser

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
            match n.heartbeat.as_deref().and_then(|s| s.parse::<i64>().ok()) {
                Some(ts) => result.push(ParsedMessage::Heartbeat(ts)),
                None => tracing::warn!("skipping malformed heartbeat: {:?}", n.heartbeat),
            }
        }
    }
    if let Some(data) = msg.data {
        for d in data {
            result.push(ParsedMessage::Data(d));
        }
    }
    Ok(result)
}

// Command builders

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
// Kept with the other protocol builders so future session controls can use the
// same tested serialization path as LOGIN, LOGOUT, and SUBS.
#[allow(dead_code)]
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
// Kept with the other protocol builders so future session controls can use the
// same tested serialization path as LOGIN, LOGOUT, and SUBS.
#[allow(dead_code)]
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
// Kept with the other protocol builders so future session controls can use the
// same tested serialization path as LOGIN, LOGOUT, and SUBS.
#[allow(dead_code)]
pub(crate) fn build_view(
    request_id: &str,
    service_name: &str,
    customer_id: &str,
    correl_id: &str,
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
            schwab_client_customer_id: customer_id.to_string(),
            schwab_client_correl_id: correl_id.to_string(),
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
    use std::assert_matches;

    use super::*;
    use crate::test_support::fixture;

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
        // Bearer token must never appear in debug output.
        assert!(
            !debug.contains("SECRET_TOKEN"),
            "bearer token leaked in Debug output: {debug}"
        );
        assert!(
            debug.contains("<redacted>"),
            "redaction marker missing: {debug}"
        );
    }

    #[test]
    fn parse_response_message() {
        let json = r#"{"response":[{"service":"ADMIN","command":"LOGIN","requestid":"0","SchwabClientCorrelId":"c","timestamp":1234,"content":{"code":0,"msg":"SUCCESS"}}]}"#;
        let msgs = parse_message(json).unwrap();
        assert_eq!(msgs.len(), 1);
        assert_matches!(msgs[0], ParsedMessage::Response(_));
    }

    #[test]
    fn parse_heartbeat() {
        let json = r#"{"notify":[{"heartbeat":"1234567890"}]}"#;
        let msgs = parse_message(json).unwrap();
        assert_eq!(msgs.len(), 1);
        assert_matches!(msgs[0], ParsedMessage::Heartbeat(1234567890));
    }

    #[test]
    fn parse_data_message() {
        let json = r#"{"data":[{"service":"LEVELONE_EQUITIES","timestamp":1234,"command":"SUBS","content":[{"key":"AAPL","1":150.0}]}]}"#;
        let msgs = parse_message(json).unwrap();
        assert_eq!(msgs.len(), 1);
        assert_matches!(msgs[0], ParsedMessage::Data(_));
    }

    #[test]
    fn parse_malformed_returns_error() {
        let result = parse_message("not json");
        assert_matches!(result, Err(crate::Error::StreamProtocol(_)));
    }

    #[test]
    fn parse_login_success_fixture_accepts_numeric_request_id() {
        let msgs = parse_message(&fixture("streaming_login_success.json")).unwrap();

        let ParsedMessage::Response(response) = &msgs[0] else {
            panic!("expected response message");
        };

        assert_eq!(response.service.as_deref(), Some("ADMIN"));
        assert_eq!(response.command.as_deref(), Some("LOGIN"));
        assert_eq!(response.requestid.as_deref(), Some("1"));
        assert_eq!(
            response.content.as_ref().and_then(|content| content.code),
            Some(0)
        );
    }

    #[test]
    fn parse_login_denied_fixture() {
        let msgs = parse_message(&fixture("streaming_login_denied.json")).unwrap();

        let ParsedMessage::Response(response) = &msgs[0] else {
            panic!("expected response message");
        };

        assert_eq!(
            response.content.as_ref().and_then(|content| content.code),
            Some(3)
        );
        assert_eq!(
            response
                .content
                .as_ref()
                .and_then(|content| content.msg.as_deref()),
            Some("LOGIN_DENIED")
        );
    }

    #[test]
    fn parse_heartbeat_fixture_timestamp() {
        let msgs = parse_message(&fixture("streaming_heartbeat.json")).unwrap();

        assert_matches!(msgs[0], ParsedMessage::Heartbeat(1234567890));
    }

    #[test]
    fn parse_equity_data_fixture_shape() {
        let msgs = parse_message(&fixture("streaming_equity_data.json")).unwrap();

        let ParsedMessage::Data(data) = &msgs[0] else {
            panic!("expected data message");
        };
        let content = data.content.as_ref().expect("fixture has content");

        assert_eq!(data.service.as_deref(), Some("LEVELONE_EQUITIES"));
        assert_eq!(
            content[0].get("key").and_then(|value| value.as_str()),
            Some("AAPL")
        );
        assert!(content[0].contains_key("5"));
    }

    #[test]
    fn parse_options_data_fixture_shape() {
        let msgs = parse_message(&fixture("streaming_options_data.json")).unwrap();

        let ParsedMessage::Data(data) = &msgs[0] else {
            panic!("expected data message");
        };
        let content = data.content.as_ref().expect("fixture has content");

        assert_eq!(data.service.as_deref(), Some("LEVELONE_OPTIONS"));
        assert_eq!(
            content[0].get("0").and_then(|value| value.as_str()),
            Some("AAPL  251219C00200000")
        );
        assert!(content[0].contains_key("5"));
    }

    #[test]
    fn parse_account_activity_data_fixture_shape() {
        let msgs = parse_message(&fixture("streaming_account_activity_data.json")).unwrap();

        let ParsedMessage::Data(data) = &msgs[0] else {
            panic!("expected data message");
        };
        let content = data.content.as_ref().expect("fixture has content");

        assert_eq!(data.service.as_deref(), Some("ACCT_ACTIVITY"));
        assert_eq!(
            content[0].get("seq").and_then(|value| value.as_i64()),
            Some(42)
        );
        assert_eq!(
            content[0].get("key").and_then(|value| value.as_str()),
            Some("Account Activity")
        );
        assert!(content[0].contains_key("3"));
    }
}
