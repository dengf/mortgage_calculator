//! `wasm32` construction: `redb` running against a
//! [`redb::StorageBackend`] backed by an in-memory buffer, asynchronously
//! flushed to IndexedDB (via `rexie`) after each write and preloaded from
//! IndexedDB once at startup.
//!
//! This intentionally does *not* use the Origin Private File System's
//! synchronous access handles (the approach browser-native SQLite builds
//! use for real durability) — that API only exists inside a dedicated
//! Worker, which would mean restructuring the whole app around a
//! Worker/postMessage boundary just for storage. Keeping the working set
//! in memory and persisting it as a single blob is a well-understood
//! local-first tradeoff: writes are visible immediately (no I/O in the
//! hot path at all) and durable within roughly one JS microtask, rather
//! than fsync-before-return.

use std::cell::RefCell;
use std::io;
use std::rc::Rc;

use mortgage_ports::StoreError;
use redb::{Builder, StorageBackend};
use rexie::{ObjectStore, Rexie, TransactionMode};
use wasm_bindgen::JsValue;

use crate::RedbScenarioStore;

const DB_NAME: &str = "mortgage_calculator";
const STORE_NAME: &str = "redb_backend";
const BLOB_KEY: &str = "db";

impl RedbScenarioStore {
    /// Loads any previously-persisted database bytes from IndexedDB, then
    /// opens (or initializes) redb against an in-memory-backed
    /// [`IndexedDbBackend`]. Must run before any other wasm-bindgen call
    /// touches storage.
    pub async fn open_wasm() -> Result<Self, StoreError> {
        let bytes = load_blob().await.map_err(StoreError::Backend)?;

        let backend = IndexedDbBackend {
            buffer: RefCell::new(bytes),
            writer: Rc::new(PersistState::default()),
        };
        let db = Builder::new()
            .create_with_backend(backend)
            .map_err(|e| StoreError::Backend(e.to_string()))?;

        Ok(Self::from_database(db))
    }
}

#[derive(Debug)]
struct IndexedDbBackend {
    buffer: RefCell<Vec<u8>>,
    writer: Rc<PersistState>,
}

/// Coalescing single-flight writer: at most one IndexedDB persist runs at a
/// time. A `sync_data()` call that lands while a flush is already in
/// progress just replaces `pending` with its (newer) snapshot rather than
/// opening a second, independent IndexedDB transaction — two overlapping
/// writes have no ordering guarantee relative to each other and the older
/// one finishing last would silently leave stale data as the final state.
/// `pending` is drained in a loop after each flush so the *latest* snapshot
/// always eventually wins.
#[derive(Debug, Default)]
struct PersistState {
    flushing: RefCell<bool>,
    pending: RefCell<Option<Vec<u8>>>,
}

// Safety: wasm32-unknown-unknown is single-threaded (no shared-memory
// threads are in play here), so there is no actual concurrent access for
// these bounds to protect against — they exist only because
// `redb::StorageBackend` requires them.
unsafe impl Send for IndexedDbBackend {}
unsafe impl Sync for IndexedDbBackend {}

impl StorageBackend for IndexedDbBackend {
    fn len(&self) -> Result<u64, io::Error> {
        Ok(self.buffer.borrow().len() as u64)
    }

    fn read(&self, offset: u64, out: &mut [u8]) -> Result<(), io::Error> {
        let buffer = self.buffer.borrow();
        let start = offset as usize;
        let end = start
            .checked_add(out.len())
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "offset overflow"))?;
        if end > buffer.len() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "read past end of in-memory database",
            ));
        }
        out.copy_from_slice(&buffer[start..end]);
        Ok(())
    }

    fn write(&self, offset: u64, data: &[u8]) -> Result<(), io::Error> {
        let mut buffer = self.buffer.borrow_mut();
        let start = offset as usize;
        let end = start + data.len();
        if end > buffer.len() {
            buffer.resize(end, 0);
        }
        buffer[start..end].copy_from_slice(data);
        Ok(())
    }

    fn set_len(&self, len: u64) -> Result<(), io::Error> {
        self.buffer.borrow_mut().resize(len as usize, 0);
        Ok(())
    }

    fn sync_data(&self) -> Result<(), io::Error> {
        let snapshot = self.buffer.borrow().clone();

        if *self.writer.flushing.borrow() {
            *self.writer.pending.borrow_mut() = Some(snapshot);
            return Ok(());
        }

        *self.writer.flushing.borrow_mut() = true;
        spawn_flush_loop(self.writer.clone(), snapshot);
        Ok(())
    }
}

/// Persists `snapshot`, then keeps persisting whatever landed in `pending`
/// while that write was in flight, until nothing newer is waiting. wasm32
/// is single-threaded and this only yields at the `.await` inside
/// `persist_blob`, so there's no race between draining `pending` and
/// clearing `flushing` — see [`PersistState`].
fn spawn_flush_loop(writer: Rc<PersistState>, mut snapshot: Vec<u8>) {
    wasm_bindgen_futures::spawn_local(async move {
        loop {
            if let Err(e) = persist_blob(snapshot).await {
                web_sys::console::error_1(
                    &format!("mortgage-ext-redb: persist failed: {e}").into(),
                );
            }
            match writer.pending.borrow_mut().take() {
                Some(next) => snapshot = next,
                None => break,
            }
        }
        *writer.flushing.borrow_mut() = false;
    });
}

async fn open_object_store() -> Result<Rexie, String> {
    Rexie::builder(DB_NAME)
        .version(1)
        .add_object_store(ObjectStore::new(STORE_NAME))
        .build()
        .await
        .map_err(|e| e.to_string())
}

async fn load_blob() -> Result<Vec<u8>, String> {
    let rexie = open_object_store().await?;

    let txn = rexie
        .transaction(&[STORE_NAME], TransactionMode::ReadOnly)
        .map_err(|e| e.to_string())?;
    let store = txn.store(STORE_NAME).map_err(|e| e.to_string())?;

    let value = store
        .get(JsValue::from_str(BLOB_KEY))
        .await
        .map_err(|e| e.to_string())?;
    txn.done().await.map_err(|e| e.to_string())?;

    let Some(value) = value.filter(|v| !v.is_undefined() && !v.is_null()) else {
        return Ok(Vec::new());
    };
    Ok(js_sys::Uint8Array::new(&value).to_vec())
}

async fn persist_blob(bytes: Vec<u8>) -> Result<(), String> {
    let rexie = open_object_store().await?;

    let txn = rexie
        .transaction(&[STORE_NAME], TransactionMode::ReadWrite)
        .map_err(|e| e.to_string())?;
    let store = txn.store(STORE_NAME).map_err(|e| e.to_string())?;

    let array = js_sys::Uint8Array::from(bytes.as_slice());
    store
        .put(&array, Some(&JsValue::from_str(BLOB_KEY)))
        .await
        .map_err(|e| e.to_string())?;
    txn.done().await.map_err(|e| e.to_string())?;

    Ok(())
}
