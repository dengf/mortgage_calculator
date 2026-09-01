use async_trait::async_trait;

use crate::error::StoreError;
use crate::scenario::{CalculatorKind, Scenario};

/// Local persistence for saved scenarios.
///
/// `?Send`: on `wasm32-unknown-unknown` futures generally aren't `Send`
/// (`JsValue` isn't thread-safe), and this trait needs to be implementable
/// by a wasm-backed store, so it drops the `Send` bound entirely rather
/// than cfg-gating two versions of the trait.
#[async_trait(?Send)]
pub trait ScenarioStore {
    async fn save(&self, scenario: Scenario) -> Result<(), StoreError>;

    /// Lists scenarios, optionally filtered to one calculator, newest first.
    async fn list(&self, calculator: Option<CalculatorKind>) -> Result<Vec<Scenario>, StoreError>;

    async fn load(&self, id: &str) -> Result<Scenario, StoreError>;

    async fn delete(&self, id: &str) -> Result<(), StoreError>;

    /// Removes every scenario, across every calculator.
    async fn clear(&self) -> Result<(), StoreError>;

    /// Persists the in-progress (not explicitly named/saved) inputs for one
    /// host-chosen `key`, overwriting whatever was stored under it before.
    ///
    /// Deliberately keyed by an opaque string rather than [`CalculatorKind`]:
    /// two calculators can share one `CalculatorKind` (US and Singapore
    /// affordability both do) while having incompatible field shapes, so the
    /// host layer must be free to key this more finely than the named-save
    /// feature does.
    async fn save_current(&self, key: &str, inputs_json: String) -> Result<(), StoreError>;

    /// Returns the most recently persisted current-inputs for `key`, or
    /// `None` if nothing has been saved under it yet.
    async fn load_current(&self, key: &str) -> Result<Option<String>, StoreError>;

    /// Removes every persisted current-inputs entry, across every key.
    ///
    /// Deliberately separate from [`Self::clear`]: `clear` is also invoked
    /// as part of import-replace, and wiping current-inputs there would
    /// silently blank whatever a user is mid-typing on an unrelated tab with
    /// no warning in the import confirmation.
    async fn clear_current_inputs(&self) -> Result<(), StoreError>;
}
