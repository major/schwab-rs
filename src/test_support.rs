use crate::models::Number;

/// Convert an `f64` literal to [`Number`] for use in test assertions.
///
/// When the `decimal` feature is off this is a no-op; when it is on the value
/// is converted via `Decimal::from_f64`, which mirrors the `serde-with-float`
/// deserialization path so assertions stay consistent.
#[cfg(not(feature = "decimal"))]
pub(crate) fn n(v: f64) -> Number {
    v
}

/// Convert an `f64` literal to [`Number`] for use in test assertions.
#[cfg(feature = "decimal")]
pub(crate) fn n(v: f64) -> Number {
    use rust_decimal::prelude::FromPrimitive;
    rust_decimal::Decimal::from_f64(v).expect("f64 -> Decimal conversion failed in test")
}

/// Load a golden JSON fixture file from `tests/fixtures/`.
///
/// Panics if the file doesn't exist or can't be read.
pub(crate) fn fixture(name: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(name);

    std::fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to load fixture {name}: {err}"))
}
