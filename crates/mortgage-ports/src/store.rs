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
}
