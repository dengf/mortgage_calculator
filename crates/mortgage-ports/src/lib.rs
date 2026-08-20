//! Hexagonal *port* definition for local scenario persistence: the
//! [`Scenario`] value type and the [`ScenarioStore`] trait, with no
//! concrete backend. Adapters (e.g. `mortgage-ext-redb`) depend *down* on
//! this crate, and callers (`mortgage-wasm`) depend on the trait, never on
//! a specific backend.

mod error;
mod scenario;
mod store;

pub use error::StoreError;
pub use scenario::{CalculatorKind, Scenario};
pub use store::ScenarioStore;
