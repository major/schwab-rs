use crate::{Error, Result};

pub(crate) fn required_text<'a>(parameter: &'static str, value: &'a str) -> Result<&'a str> {
    let value = value.trim();
    if value.is_empty() {
        return Err(Error::MissingRequiredParameter(parameter));
    }
    Ok(value)
}

pub(crate) fn comma_separated_symbols<S>(symbols: impl IntoIterator<Item = S>) -> Result<String>
where
    S: AsRef<str>,
{
    comma_separated_nonempty(symbols, || Error::EmptySymbols)
}

pub(crate) fn comma_separated_required<S>(
    parameter: &'static str,
    values: impl IntoIterator<Item = S>,
) -> Result<String>
where
    S: AsRef<str>,
{
    comma_separated_nonempty(values, || Error::MissingRequiredParameter(parameter))
}

pub(crate) fn push_optional<T>(
    query: &mut Vec<(&'static str, String)>,
    key: &'static str,
    value: Option<T>,
) where
    T: AsRef<str>,
{
    if let Some(value) = value {
        let value = value.as_ref().trim();
        if !value.is_empty() {
            query.push((key, value.to_string()));
        }
    }
}

fn comma_separated_nonempty<S>(
    values: impl IntoIterator<Item = S>,
    empty_error: impl FnOnce() -> Error,
) -> Result<String>
where
    S: AsRef<str>,
{
    let values: Vec<String> = values
        .into_iter()
        .map(|value| value.as_ref().trim().to_string())
        .filter(|value| !value.is_empty())
        .collect();

    if values.is_empty() {
        return Err(empty_error());
    }
    Ok(values.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comma_separated_symbols_rejects_empty_values() {
        assert!(matches!(
            comma_separated_symbols([" "]),
            Err(Error::EmptySymbols)
        ));
    }
}
