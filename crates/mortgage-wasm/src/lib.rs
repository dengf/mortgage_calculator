//! WebAssembly bindings for the mortgage-calculator library.
//!
//! This crate provides WASM bindings for `mortgage-calc`, enabling the same
//! calculations to run client-side in a browser. The public
//! `#[wasm_bindgen]` surface is split across submodules by responsibility,
//! following the pattern used by `convex-wasm` in the sibling `convex`
//! project:
//!
//! - [`payment`] — `calculate_payment`
//! - [`amortization`] — `calculate_amortization_schedule`, `calculate_extra_payment_impact`
//! - [`affordability`] — `calculate_affordability`
//! - [`refinance`] — `calculate_refinance`
//!
//! The non-public modules ([`dto`], [`convert`], [`loan`]) hold the wire
//! types, parser/formatter helpers, and shared loan construction. No
//! business logic lives in this crate — every function here parses a
//! `JsValue`, calls into `mortgage-calc`, and serializes the result back.

use wasm_bindgen::prelude::*;

pub mod affordability;
pub mod amortization;
pub mod convert;
pub mod dto;
pub mod loan;
pub mod payment;
pub mod refinance;

pub use affordability::calculate_affordability;
pub use amortization::{calculate_amortization_schedule, calculate_extra_payment_impact};
pub use dto::{
    AffordabilityParams, AffordabilityResultDto, AmortizationParams, AmortizationResult,
    AmortizationRowDto, ExtraPaymentImpactResult, LoanParams, PaymentSummaryResult,
    RefinanceParams, RefinanceResultDto,
};
pub use payment::calculate_payment;
pub use refinance::calculate_refinance;

/// Initialize the WASM module (sets up panic hook for better error messages).
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
