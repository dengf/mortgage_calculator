//! WebAssembly bindings for the mortgage-calculator library.
//!
//! This crate provides WASM bindings for `mortgage-calc`, enabling the same
//! calculations to run client-side in a browser. The public
//! `#[wasm_bindgen]` surface is split across submodules by responsibility:
//!
//! - [`payment`] — `calculate_payment`
//! - [`amortization`] — `calculate_amortization_schedule`, `calculate_extra_payment_impact`
//! - [`affordability`] — `calculate_affordability`
//! - [`refinance`] — `calculate_refinance`
//! - [`comparison`] — `calculate_comparison`, `get_common_rate_presets`
//! - [`singapore`] — `calculate_singapore`
//! - [`united_states`] — `calculate_united_states`
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
pub mod singapore;
#[cfg(target_arch = "wasm32")]
pub mod storage;
pub mod united_states;

pub use affordability::calculate_affordability;
pub use amortization::{calculate_amortization_schedule, calculate_extra_payment_impact};
pub use comparison::{calculate_comparison, get_common_rate_presets};
pub use dto::{
    AffordabilityParams, AffordabilityResultDto, AmortizationParams, AmortizationResult,
    AmortizationRowDto, ComparisonEntryParams, ComparisonParams, ComparisonResult,
    ComparisonRowDto, DeleteScenarioResult, ExtraPaymentImpactResult, LoanParams,
    PaymentSummaryResult, RatePresetDto, RateTypeDto, RefinanceParams, RefinanceResultDto,
    SaveScenarioParams, SaveScenarioResult, ScenarioDto, ScenarioListResult, ScenarioResult,
    SingaporeParams, SingaporeResult, UnitedStatesParams, UnitedStatesResult,
};
pub use payment::calculate_payment;
pub use refinance::calculate_refinance;
pub use singapore::calculate_singapore;
#[cfg(target_arch = "wasm32")]
pub use storage::{delete_scenario, init_storage, list_scenarios, load_scenario, save_scenario};
pub use united_states::calculate_united_states;

/// Guards against this crate silently falling behind `mortgage-calc`.
///
/// wasm-bindgen exports nothing automatically: every function JS can reach
/// is hand-written here. That means a capability can be added to the core
/// and simply never surface in the browser — no build error, because an
/// unexported function is just unused code that gets eliminated. Exactly
/// that happened to `mortgage_calc::singapore`, which shipped in the Slint
/// app for months while the web app had no idea it existed.
///
/// So: every analytics family `mortgage-calc` makes public must have a
/// bridge module here. If you add one there and this test fails, that's the
/// point — either write the binding, or add it to `NOT_BRIDGED` with a
/// reason.
#[cfg(test)]
mod bridge_coverage {
    /// Modules that deliberately have no binding of their own, and why.
    /// Keep this list short and justified; it is an escape hatch, not a
    /// dumping ground.
    const NOT_BRIDGED: &[(&str, &str)] = &[];

    fn public_modules(source: &str) -> Vec<String> {
        source
            .lines()
            .map(str::trim)
            .filter_map(|line| line.strip_prefix("pub mod "))
            .filter_map(|rest| rest.strip_suffix(';'))
            .map(str::to_string)
            .collect()
    }

    /// Core modules with no binding here and no entry in `NOT_BRIDGED`.
    fn unbridged(calc_source: &str, wasm_source: &str) -> Vec<String> {
        let bridged = public_modules(wasm_source);
        public_modules(calc_source)
            .into_iter()
            .filter(|m| !bridged.contains(m))
            .filter(|m| !NOT_BRIDGED.iter().any(|(name, _)| *name == m.as_str()))
            .collect()
    }

    #[test]
    fn every_public_mortgage_calc_module_has_a_wasm_binding() {
        let calc = include_str!("../../mortgage-calc/src/lib.rs");

        assert!(
            !public_modules(calc).is_empty(),
            "parsed no public modules from mortgage-calc; the `pub mod` parse likely broke"
        );

        let missing = unbridged(calc, include_str!("lib.rs"));
        assert!(
            missing.is_empty(),
            "mortgage-calc exposes {missing:?} with no matching binding in mortgage-wasm, \
             so the web app cannot reach it. Add a `pub mod` here wrapping it, or list it \
             in NOT_BRIDGED with a reason."
        );
    }

    /// The guard above only earns its keep if it actually fires. Exercising
    /// it against the real files can't show that — removing a binding to try
    /// it breaks the build on the `pub use` instead. So drive the detection
    /// with synthetic sources, which is the shape the real drift takes: a
    /// module added to the core and nothing added here, compiling fine.
    #[test]
    fn the_guard_detects_a_core_module_with_no_binding() {
        let calc = "pub mod payment;\npub mod brand_new_thing;\n";
        let wasm = "pub mod payment;\n";

        assert_eq!(unbridged(calc, wasm), vec!["brand_new_thing".to_string()]);
    }

    #[test]
    fn the_guard_is_quiet_when_everything_is_bridged() {
        let calc = "pub mod payment;\npub mod refinance;\n";
        let wasm = "pub mod payment;\npub mod refinance;\npub mod dto;\n";

        assert!(unbridged(calc, wasm).is_empty());
    }
}

/// Initialize the WASM module (sets up panic hook for better error messages).
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}
