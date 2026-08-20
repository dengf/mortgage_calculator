//! Pure mortgage math: no I/O, no serialization, no JS boundary concerns.
//!
//! [`Loan`] is the shared instrument type, and each submodule below is an
//! analytics function family that operates on it:
//!
//! - [`payment`] — standard amortizing payment amount and lifetime summary
//! - [`amortization`] — full payment-by-payment schedule, extra-payment payoff impact
//! - [`affordability`] — maximum affordable home price from income/debt inputs
//! - [`refinance`] — refinance break-even and lifetime savings analysis
//! - [`comparison`] — side-by-side scenario comparison across rate types and terms
//! - [`singapore`] — CPF OA, MAS TDSR/MSR limits, and BSD/ABSD stamp duty
//!
//! [`mortgage_wasm`](../mortgage_wasm) is the only crate allowed to depend on
//! this one from the JS side; everything here stays pure Rust so it is
//! trivially unit-testable and reusable from a future CLI, FFI, or server
//! layer without touching wasm-bindgen at all.

mod loan;
mod rate;

pub mod affordability;
pub mod amortization;
pub mod comparison;
pub mod payment;
pub mod refinance;
pub mod singapore;

pub use loan::{Loan, LoanBuilder};
pub use mortgage_core::{MortgageError, MortgageResult, PaymentFrequency};
pub use rate::{common_presets, RatePreset, RateType};
