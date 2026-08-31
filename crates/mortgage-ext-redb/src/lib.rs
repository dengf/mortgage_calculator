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

// Compiled for tests on every target (not just wasm32) so its unit tests
// run under a plain `cargo test` — see the module doc comment for why.
#[cfg(any(test, target_arch = "wasm32"))]
mod buffer;

use async_trait::async_trait;
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};

use mortgage_ports::{CalculatorKind, Scenario, ScenarioStore, StoreError};

const SCENARIOS: TableDefinition<&str, &[u8]> = TableDefinition::new("scenarios");

/// Shared [`ScenarioStore`] implementation over an already-open
/// [`redb::Database`], regardless of what [`redb::StorageBackend`] backs it.
pub struct RedbScenarioStore {
    db: Database,
    // Only wasm32 has anything to wait for -- redb's native file backend
    // fsyncs synchronously inside `commit()`, so a write is already durable
    // by the time it returns.
    #[cfg(target_arch = "wasm32")]
    persist: Option<std::rc::Rc<wasm::PersistState>>,
}

impl RedbScenarioStore {
    #[cfg(not(target_arch = "wasm32"))]
    fn from_database(db: Database) -> Self {
        Self { db }
    }

    #[cfg(target_arch = "wasm32")]
    fn from_database_with_persist(db: Database, persist: std::rc::Rc<wasm::PersistState>) -> Self {
        Self {
            db,
            persist: Some(persist),
        }
    }

    fn backend_err(e: impl std::fmt::Display) -> StoreError {
        StoreError::Backend(e.to_string())
    }

    fn serialization_err(e: impl std::fmt::Display) -> StoreError {
        StoreError::Serialization(e.to_string())
    }

    /// Waits for a just-committed write to actually reach durable storage.
    /// A no-op on native (see the `persist` field's doc comment); on
    /// wasm32, waits for the in-flight IndexedDB flush this write kicked
    /// off to finish, so a caller only reports success once a page reload
    /// would actually see the change.
    #[cfg(not(target_arch = "wasm32"))]
    async fn wait_for_durability(&self) {}

    #[cfg(target_arch = "wasm32")]
    async fn wait_for_durability(&self) {
        if let Some(persist) = &self.persist {
            wasm::wait_idle(persist).await;
        }
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
        self.wait_for_durability().await;

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

        scenarios.sort_by_key(|s| std::cmp::Reverse(s.created_at));
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
        self.wait_for_durability().await;

        Ok(())
    }

    async fn clear(&self) -> Result<(), StoreError> {
        let write_txn = self.db.begin_write().map_err(Self::backend_err)?;
        // Dropping the whole table rather than removing each row: `list`
        // and `load` already treat a missing table as "empty" (see their
        // `TableDoesNotExist` handling above), and `save` re-creates it on
        // the next write via `open_table`, so this is a safe, atomic clear.
        write_txn
            .delete_table(SCENARIOS)
            .map_err(Self::backend_err)?;
        write_txn.commit().map_err(Self::backend_err)?;
        self.wait_for_durability().await;

        Ok(())
    }
}
