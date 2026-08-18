//! Opens the same [`mortgage_ext_redb::RedbScenarioStore`] used by the wasm
//! frontend, but via its native (plain file) constructor on desktop/iOS/
//! Android, and its wasm (IndexedDB-backed) constructor on the web target.
//! One concrete type either way — only *how it's opened* differs by
//! platform — so the rest of the app (see [`crate::components::SavedScenarios`])
//! never needs to know which target it's running on.

use std::rc::Rc;

use mortgage_ext_redb::RedbScenarioStore;

pub async fn open_store() -> Rc<RedbScenarioStore> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let dir = data_dir();
        std::fs::create_dir_all(&dir).expect("failed to create app data directory");
        Rc::new(
            RedbScenarioStore::open(dir.join("scenarios.redb"))
                .expect("failed to open local scenario database"),
        )
    }
    #[cfg(target_arch = "wasm32")]
    {
        Rc::new(
            RedbScenarioStore::open_wasm()
                .await
                .expect("failed to open browser scenario database"),
        )
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn data_dir() -> std::path::PathBuf {
    // iOS/macOS sandbox HOME already scopes this per-app; this is a
    // reasonable v1 for Apple platforms specifically (Android would need
    // its own path convention, e.g. via `ndk-context`).
    match std::env::var_os("HOME") {
        Some(home) => {
            std::path::PathBuf::from(home).join("Library/Application Support/MortgageCalculator")
        }
        None => std::env::temp_dir().join("mortgage-calculator"),
    }
}

pub fn now_millis() -> i64 {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::now() as i64
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before Unix epoch")
            .as_millis() as i64
    }
}

pub fn new_scenario_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("{}-{n:x}", now_millis())
}
