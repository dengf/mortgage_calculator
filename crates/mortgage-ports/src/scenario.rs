use serde::{Deserialize, Serialize};

/// Which calculator a saved scenario belongs to. Scenarios are always
/// listed/filtered per calculator in the UI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CalculatorKind {
    Payment,
    Amortization,
    Affordability,
    Refinance,
    Comparison,
}

/// A user-named, saved set of inputs for one calculator.
///
/// `inputs_json` is an opaque, calculator-specific JSON blob (the same
/// shape `mortgage-wasm`'s DTOs already use) rather than a typed field per
/// calculator — that keeps this crate, and the storage backends built on
/// it, from needing to know about every calculator's input shape.
/// `created_at` is milliseconds since the Unix epoch; this crate has no
/// time source of its own (`SystemTime::now()` panics on
/// `wasm32-unknown-unknown`), so callers supply it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scenario {
    pub id: String,
    pub calculator: CalculatorKind,
    pub name: String,
    pub created_at: i64,
    pub inputs_json: String,
}
