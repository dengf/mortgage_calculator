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
//! - [`comparison`] — `calculate_comparison`, `get_common_rate_presets`
//! - [`storage`] (wasm32 only) — `init_storage`, `save_scenario`,
//!   `list_scenarios`, `load_scenario`, `delete_scenario`, backed by
//!   `mortgage-ext-redb`'s wasm/IndexedDB-persisted store. Gated to
//!   `wasm32` because it calls `RedbScenarioStore::open_wasm()`, which
//!   only exists on that target — everything else in this crate builds
//!   and its tests run on the native host too, which is how `cargo test
//!   --workspace` exercises it.
//!
//! The non-public modules ([`dto`], [`convert`], [`loan`]) hold the wire
//! types, parser/formatter helpers, and shared loan construction. No
//! business logic lives in this crate — every function here parses a
//! `JsValue`, calls into `mortgage-calc` or `mortgage-ext-redb`, and
//! serializes the result back.

use wasm_bindgen::prelude::*;

pub mod affordability;
pub mod amortization;
pub mod comparison;
pub mod convert;
pub mod dto;
pub mod loan;
pub mod payment;
pub mod refinance;
#[cfg(target_arch = "wasm32")]
pub mod storage;

pub use affordability::calculate_affordability;
pub use amortization::{calculate_amortization_schedule, calculate_extra_payment_impact};
pub use comparison::{calculate_comparison, get_common_rate_presets};
pub use dto::{
    AffordabilityParams, AffordabilityResultDto, AmortizationParams, AmortizationResult,
    AmortizationRowDto, ComparisonEntryParams, ComparisonParams, ComparisonResult,
    ComparisonRowDto, DeleteScenarioResult, ExtraPaymentImpactResult, LoanParams,
    PaymentSummaryResult, RatePresetDto, RateTypeDto, RefinanceParams, RefinanceResultDto,
    SaveScenarioParams, SaveScenarioResult, ScenarioDto, ScenarioListResult, ScenarioResult,
};
pub use payment::calculate_payment;
pub use refinance::calculate_refinance;
#[cfg(target_arch = "wasm32")]
pub use storage::{delete_scenario, init_storage, list_scenarios, load_scenario, save_scenario};

/// Initialize the WASM module (sets up panic hook for better error messages).
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
