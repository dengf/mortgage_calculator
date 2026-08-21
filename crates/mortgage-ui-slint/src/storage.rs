//! Opens the same [`mortgage_ext_redb::RedbScenarioStore`] used by the
//! original React frontend, but via its native (plain file) constructor on
//! desktop/iOS/Android, and its wasm (IndexedDB-backed) constructor on the
//! web target. One concrete type either way.

use std::rc::Rc;

use mortgage_ext_redb::RedbScenarioStore;

/// Populated by `android_main` (before `open_store()` runs) from
/// `AndroidApp::internal_data_path()` — the only way to reach the app's
/// actual sandboxed storage (`Context.getFilesDir()`), since Android has no
/// `$HOME`-based convention for it. See [`set_android_data_dir`].
#[cfg(target_os = "android")]
static ANDROID_DATA_DIR: std::sync::OnceLock<std::path::PathBuf> = std::sync::OnceLock::new();

/// Records the app-private data directory for [`data_dir`] to use. Must be
/// called from `android_main` before the first `open_store()` call.
#[cfg(target_os = "android")]
pub fn set_android_data_dir(path: std::path::PathBuf) {
    let _ = ANDROID_DATA_DIR.set(path);
}

pub async fn open_store() -> Rc<RedbScenarioStore> {
    #[cfg(not(target_family = "wasm"))]
    {
        let dir = data_dir();
        std::fs::create_dir_all(&dir).expect("failed to create app data directory");
        Rc::new(
            RedbScenarioStore::open(dir.join("scenarios.redb"))
                .expect("failed to open local scenario database"),
        )
    }
    #[cfg(target_family = "wasm")]
    {
        Rc::new(
            RedbScenarioStore::open_wasm()
                .await
                .expect("failed to open browser scenario database"),
        )
    }
}

#[cfg(target_os = "android")]
fn data_dir() -> std::path::PathBuf {
    // Falls back to a generic temp dir only if `android_main` somehow ran
    // without setting this (internal_data_path() returning None, or the
    // setter never being called) — not expected in practice, but preferable
    // to a hard panic on startup.
    ANDROID_DATA_DIR
        .get()
        .cloned()
        .unwrap_or_else(|| std::env::temp_dir().join("mortgage-calculator"))
}

#[cfg(not(any(target_family = "wasm", target_os = "android")))]
fn data_dir() -> std::path::PathBuf {
    // iOS/macOS sandbox HOME already scopes this per-app.
    match std::env::var_os("HOME") {
        Some(home) => {
            std::path::PathBuf::from(home).join("Library/Application Support/MortgageCalculator")
        }
        None => std::env::temp_dir().join("mortgage-calculator"),
    }
}

pub fn now_millis() -> i64 {
    #[cfg(target_family = "wasm")]
    {
        js_sys::Date::now() as i64
    }
    #[cfg(not(target_family = "wasm"))]
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
