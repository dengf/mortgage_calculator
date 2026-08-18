//! Pure mortgage math: no I/O, no serialization, no JS boundary concerns.
//!
//! This mirrors `convex-bonds` + `convex-analytics` in the `convex` sibling
//! project — [`Loan`] is the equivalent of a bond instrument, and each
//! submodule below is an analytics function family that operates on it:
//!
//! - [`payment`] — standard amortizing payment amount and lifetime summary
//! - [`amortization`] — full payment-by-payment schedule, extra-payment payoff impact
//! - [`affordability`] — maximum affordable home price from income/debt inputs
//! - [`refinance`] — refinance break-even and lifetime savings analysis
//!
//! [`mortgage_wasm`](../mortgage_wasm) is the only crate allowed to depend on
//! this one from the JS side; everything here stays pure Rust so it is
//! trivially unit-testable and reusable from a future CLI, FFI, or server
//! layer without touching wasm-bindgen at all.

mod loan;

pub mod affordability;
pub mod amortization;
pub mod payment;
pub mod refinance;

pub use loan::{Loan, LoanBuilder};
pub use mortgage_core::{MortgageError, MortgageResult, PaymentFrequency};
