//! Technical analysis subcommands, indicators, and output types.

pub mod candles;
pub mod custom;
pub mod dashboard;
pub mod expected_move;
pub mod indicators;
pub mod interval;
pub mod types;

pub use dashboard::dashboard;

#[cfg(test)]
mod tests;
