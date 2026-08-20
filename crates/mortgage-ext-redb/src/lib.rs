//! [`mortgage_ports::ScenarioStore`] implemented on top of `redb`.
//!
//! The table logic (schema, save/list/load/delete) is identical on every
//! platform — only *how the bytes get durable* differs:
//!
//! - Native targets ([`native`]): `redb`'s ordinary file backend. A plain
//!   file on disk.
//! - `wasm32` ([`wasm`]): a custom [`redb::StorageBackend`] backed by an
//!   in-memory buffer, asynchronously flushed to the browser's IndexedDB
//!   after each write. Real `redb` (actual ACID transactions, actual
//!   schema) runs in-process; only the durability step is async and
//!   best-effort, which avoids the Origin Private File System's
//!   synchronous-access-handle API — and the dedicated-Worker +
//!   cross-origin-isolation-headers requirement that comes with it.

#[cfg(not(target_arch = "wasm32"))]
pub mod native;
#[cfg(target_arch = "wasm32")]
pub mod wasm;

use async_trait::async_trait;
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};

use mortgage_ports::{CalculatorKind, Scenario, ScenarioStore, StoreError};

const SCENARIOS: TableDefinition<&str, &[u8]> = TableDefinition::new("scenarios");

/// Shared [`ScenarioStore`] implementation over an already-open
/// [`redb::Database`], regardless of what [`redb::StorageBackend`] backs it.
pub struct RedbScenarioStore {
    db: Database,
}

impl RedbScenarioStore {
    fn from_database(db: Database) -> Self {
        Self { db }
    }

    fn backend_err(e: impl std::fmt::Display) -> StoreError {
        StoreError::Backend(e.to_string())
    }

    fn serialization_err(e: impl std::fmt::Display) -> StoreError {
        StoreError::Serialization(e.to_string())
    }
}

#[async_trait(?Send)]
impl ScenarioStore for RedbScenarioStore {
    async fn save(&self, scenario: Scenario) -> Result<(), StoreError> {
        let bytes = serde_json::to_vec(&scenario).map_err(Self::serialization_err)?;

        let write_txn = self.db.begin_write().map_err(Self::backend_err)?;
        {
            let mut table = write_txn.open_table(SCENARIOS).map_err(Self::backend_err)?;
            table
                .insert(scenario.id.as_str(), bytes.as_slice())
                .map_err(Self::backend_err)?;
        }
        write_txn.commit().map_err(Self::backend_err)?;

        Ok(())
    }

    async fn list(&self, calculator: Option<CalculatorKind>) -> Result<Vec<Scenario>, StoreError> {
        let read_txn = self.db.begin_read().map_err(Self::backend_err)?;
        let table = match read_txn.open_table(SCENARIOS) {
            Ok(table) => table,
            Err(redb::TableError::TableDoesNotExist(_)) => return Ok(Vec::new()),
            Err(e) => return Err(Self::backend_err(e)),
        };

        let mut scenarios = Vec::new();
        for entry in table.iter().map_err(Self::backend_err)? {
            let (_, value) = entry.map_err(Self::backend_err)?;
            let scenario: Scenario =
                serde_json::from_slice(value.value()).map_err(Self::serialization_err)?;
            if calculator.is_none_or(|kind| kind == scenario.calculator) {
                scenarios.push(scenario);
            }
        }

        scenarios.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(scenarios)
    }

    async fn load(&self, id: &str) -> Result<Scenario, StoreError> {
        let read_txn = self.db.begin_read().map_err(Self::backend_err)?;
        let table = match read_txn.open_table(SCENARIOS) {
            Ok(table) => table,
            Err(redb::TableError::TableDoesNotExist(_)) => {
                return Err(StoreError::NotFound(id.to_string()))
            }
            Err(e) => return Err(Self::backend_err(e)),
        };

        let value = table
            .get(id)
            .map_err(Self::backend_err)?
            .ok_or_else(|| StoreError::NotFound(id.to_string()))?;

        serde_json::from_slice(value.value()).map_err(Self::serialization_err)
    }

    async fn delete(&self, id: &str) -> Result<(), StoreError> {
        let write_txn = self.db.begin_write().map_err(Self::backend_err)?;
        {
            let mut table = write_txn.open_table(SCENARIOS).map_err(Self::backend_err)?;
            table.remove(id).map_err(Self::backend_err)?;
        }
        write_txn.commit().map_err(Self::backend_err)?;

        Ok(())
    }
}
