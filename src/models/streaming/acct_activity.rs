//! Account activity streaming data types.

/// Field selector for account activity streaming subscriptions.
///
/// Each variant corresponds to a numeric field index in the Schwab streaming protocol.
///
/// # Examples
///
/// ```
/// use schwab::AccountActivityField;
///
/// assert_eq!(AccountActivityField::All.index(), 0);
/// assert_eq!(AccountActivityField::MessageData.index(), 3);
/// ```
#[allow(missing_docs)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum AccountActivityField {
    All = 0,
    Account = 1,
    MessageType = 2,
    MessageData = 3,
}

impl AccountActivityField {
    /// Return the numeric field index used in the Schwab streaming protocol.
    pub fn index(&self) -> u32 {
        *self as u32
    }

    /// Return all `AccountActivityField` variants in index order.
    pub fn all() -> &'static [AccountActivityField] {
        use AccountActivityField::*;
        &[All, Account, MessageType, MessageData]
    }
}

/// Account activity streaming data for a single activity message.
///
/// All fields are `Option<T>` because the Schwab API sends partial field sets.
/// Named metadata fields use string keys, while documented activity fields use
/// numeric string keys.
///
/// # Examples
///
/// ```
/// use schwab::AccountActivity;
///
/// let data = AccountActivity {
///     account: Some("123456789".to_string()),
///     message_type: Some("OrderEntryRequest".to_string()),
///     ..Default::default()
/// };
/// assert_eq!(data.account.as_deref(), Some("123456789"));
/// ```
#[allow(missing_docs)]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AccountActivity {
    pub seq: Option<i64>,
    pub key: Option<String>,
    pub account: Option<String>,
    pub message_type: Option<String>,
    pub message_data: Option<String>,
}

impl AccountActivity {
    /// Construct an [`AccountActivity`] from a streaming data map entry.
    ///
    /// The map uses named string keys for metadata (`"seq"`, `"key"`) and
    /// numeric string keys (`"1"`, `"2"`, `"3"`) for the documented activity
    /// fields. Returns `None` if `value` is not a JSON object.
    pub(crate) fn from_value(value: &serde_json::Value) -> Option<Self> {
        let map = value.as_object()?;
        let mut activity = Self {
            seq: map.get("seq").and_then(parse_i64),
            key: map.get("key").and_then(|v| v.as_str()).map(String::from),
            ..Self::default()
        };

        // Numeric-keyed data fields.
        for (key, val) in map {
            match key.as_str() {
                "1" => activity.account = val.as_str().map(String::from),
                "2" => activity.message_type = val.as_str().map(String::from),
                "3" => activity.message_data = val.as_str().map(String::from),
                _ => {}
            }
        }

        Some(activity)
    }
}

fn parse_i64(value: &serde_json::Value) -> Option<i64> {
    value
        .as_i64()
        .or_else(|| value.as_str().and_then(|text| text.parse::<i64>().ok()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn field_index_first() {
        assert_eq!(AccountActivityField::All.index(), 0);
    }

    #[test]
    fn field_index_last() {
        assert_eq!(AccountActivityField::MessageData.index(), 3);
    }

    #[test]
    fn all_fields_count() {
        assert_eq!(AccountActivityField::all().len(), 4);
    }

    #[test]
    fn all_fields_sequential_indices() {
        for (i, field) in AccountActivityField::all().iter().enumerate() {
            assert_eq!(
                field.index() as usize,
                i,
                "field at position {i} has wrong index"
            );
        }
    }

    #[test]
    fn from_value_parses_sample() {
        let input = json!({
            "seq": 42,
            "key": "Account Activity",
            "1": "123456789",
            "2": "OrderEntryRequest",
            "3": "{\"orderId\":12345}"
        });

        let activity = AccountActivity::from_value(&input).expect("object should parse");

        assert_eq!(activity.seq, Some(42));
        assert_eq!(activity.key.as_deref(), Some("Account Activity"));
        assert_eq!(activity.account.as_deref(), Some("123456789"));
        assert_eq!(activity.message_type.as_deref(), Some("OrderEntryRequest"));
        assert_eq!(
            activity.message_data.as_deref(),
            Some("{\"orderId\":12345}")
        );
    }

    #[test]
    fn from_value_accepts_string_sequence() {
        let input = json!({
            "seq": "43",
            "key": "Account Activity"
        });

        let activity = AccountActivity::from_value(&input).expect("object should parse");

        assert_eq!(activity.seq, Some(43));
    }

    #[test]
    fn from_value_keeps_null_message_data_empty() {
        let input = json!({
            "seq": 44,
            "key": "Account Activity",
            "1": "123456789",
            "2": "Heartbeat",
            "3": null
        });

        let activity = AccountActivity::from_value(&input).expect("object should parse");

        assert_eq!(activity.message_data, None);
    }

    #[test]
    fn from_value_rejects_non_object() {
        assert_eq!(AccountActivity::from_value(&json!(["not", "object"])), None);
    }
}
