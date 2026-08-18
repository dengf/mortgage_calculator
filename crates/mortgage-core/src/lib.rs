//! Foundational types shared by [`mortgage_calc`](../mortgage_calc) and
//! [`mortgage_wasm`](../mortgage_wasm).
//!
//! This crate is deliberately small: it holds only the vocabulary
//! (payment frequency, rounding, errors) that every layer above it needs to
//! agree on, and has no dependency on the calculation logic itself.

mod error;
mod frequency;
mod rounding;

pub use error::MortgageError;
pub use frequency::PaymentFrequency;
pub use rounding::round_currency;

/// Result alias used throughout the mortgage-calculator crates.
pub type MortgageResult<T> = Result<T, MortgageError>;
