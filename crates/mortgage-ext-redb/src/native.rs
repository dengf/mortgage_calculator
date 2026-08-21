//! Native construction: a plain `redb` file on disk.

use std::path::Path;

use mortgage_ports::StoreError;
use redb::Builder;

use crate::RedbScenarioStore;

/// redb defaults to a 1 GiB cache ceiling, sized for large server-side
/// databases. Saved scenarios here are small JSON blobs (a few hundred
/// bytes to a couple KB each) — a few MiB is generous headroom for this
/// app's realistic data scale and a more predictable ceiling on
/// memory-constrained mobile targets.
const CACHE_SIZE_BYTES: usize = 4 * 1024 * 1024;

impl RedbScenarioStore {
    /// Opens (or creates) a redb file at `path`.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let db = Builder::new()
            .set_cache_size(CACHE_SIZE_BYTES)
            .create(path.as_ref())
            .map_err(|e| StoreError::Backend(e.to_string()))?;
        Ok(Self::from_database(db))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mortgage_ports::{CalculatorKind, Scenario, ScenarioStore};

    fn scenario(id: &str, name: &str) -> Scenario {
        Scenario {
            id: id.to_string(),
            calculator: CalculatorKind::Payment,
            name: name.to_string(),
            created_at: 0,
            inputs_json: "{}".to_string(),
        }
    }

    #[tokio::test]
    async fn save_list_load_delete_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let store = RedbScenarioStore::open(dir.path().join("scenarios.redb")).unwrap();

        store.save(scenario("a", "30yr fixed")).await.unwrap();
        store.save(scenario("b", "15yr fixed")).await.unwrap();

        let all = store.list(None).await.unwrap();
        assert_eq!(all.len(), 2);

        let loaded = store.load("a").await.unwrap();
        assert_eq!(loaded.name, "30yr fixed");

        store.delete("a").await.unwrap();
        let remaining = store.list(None).await.unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].id, "b");

        assert!(store.load("a").await.is_err());
    }

    #[tokio::test]
    async fn list_filters_by_calculator_kind() {
        let dir = tempfile::tempdir().unwrap();
        let store = RedbScenarioStore::open(dir.path().join("scenarios.redb")).unwrap();

        store.save(scenario("a", "payment scenario")).await.unwrap();
        let mut refi = scenario("b", "refi scenario");
        refi.calculator = CalculatorKind::Refinance;
        store.save(refi).await.unwrap();

        let payments = store.list(Some(CalculatorKind::Payment)).await.unwrap();
        assert_eq!(payments.len(), 1);
        assert_eq!(payments[0].id, "a");
    }

    #[tokio::test]
    async fn reopening_the_same_file_persists_data() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("scenarios.redb");

        {
            let store = RedbScenarioStore::open(&path).unwrap();
            store.save(scenario("a", "persisted")).await.unwrap();
        }

        let store = RedbScenarioStore::open(&path).unwrap();
        let all = store.list(None).await.unwrap();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].name, "persisted");
    }
}
