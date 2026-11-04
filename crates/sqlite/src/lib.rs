//! SQLite adapter of the `Store` port, layout v1 (ST-02 archi "SQLite adapter"; plan P-1, P-7 to P-11).
#![expect(dead_code, reason = "skeleton: bodies land in T-8 and T-9")]

use std::cell::RefCell;
use std::path::Path;
use std::time::Duration;

use arbitre_core::{
    ApplyError, DeviceId, LocalChange, Position, RecordId, RecordState, Report, Store, StoreError,
    Watermark, Write,
};
use arbitre_core::Damage;
use rusqlite::Connection;

/// Opening options.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    /// Busy wait before `Unavailable(Busy)` (Q-3).
    pub wait: Duration,
    /// Test hook: name of a registered fault VFS (E-ST02-2).
    pub vfs: Option<String>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            wait: Duration::from_secs(5),
            vfs: None,
        }
    }
}

/// Durability settings in effect on a handle (RG-ST02-1.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    pub journal_mode: String,
    /// `PRAGMA synchronous` value; `2` = FULL.
    pub synchronous: i64,
    pub fullfsync: bool,
    pub checkpoint_fullfsync: bool,
}

/// One handle = one connection. Nothing but `D` cached between transactions (Q-2).
pub struct SqliteStore {
    conn: RefCell<Connection>,
    device: DeviceId,
    /// First `Corrupted` seen; returned by every later operation (RG-ST02-7.2).
    poisoned: RefCell<Option<Damage>>,
}

impl SqliteStore {
    /// Opens or creates the store at `path` (archi "Open sequence", P-10 open-time check).
    /// Writes nothing unless creating.
    pub fn open(path: &Path, options: &Options) -> Result<Self, StoreError> {
        let _ = (path, options);
        todo!("T-8: open — RG-ST02-4.1, 7.1, 7.3; T-13: P-10, P-11")
    }

    /// Test hook: settings in effect on this connection.
    pub fn settings(&self) -> Settings {
        todo!("T-8: settings — RG-ST02-1.1")
    }
}

/// Closed SQLite error mapping, first match wins (plan P-8).
fn map(e: rusqlite::Error) -> StoreError {
    let _ = e;
    todo!("T-8: map — P-8, AC-6.6")
}

impl Store for SqliteStore {
    async fn device(&self) -> Result<DeviceId, StoreError> {
        todo!("T-8: device")
    }

    async fn apply_local(
        &self,
        pt: u64,
        changes: &[LocalChange],
    ) -> Result<Vec<Write>, ApplyError> {
        let _ = (pt, changes);
        todo!("T-9: apply_local — BEGIN IMMEDIATE, rebuild + stamp_local")
    }

    async fn apply(&self, batch: &[Write]) -> Result<Report, ApplyError> {
        let _ = batch;
        todo!("T-9: apply — BEGIN IMMEDIATE, rebuild + receive")
    }

    async fn writes_of(&self, record: &RecordId) -> Result<Vec<Write>, StoreError> {
        let _ = record;
        todo!("T-9: writes_of — one BEGIN DEFERRED")
    }

    async fn state(&self, record: &RecordId) -> Result<Option<RecordState>, StoreError> {
        let _ = record;
        todo!("T-9: state — one BEGIN DEFERRED")
    }

    async fn records(
        &self,
        module: &str,
        category: &str,
    ) -> Result<Vec<(RecordId, RecordState)>, StoreError> {
        let _ = (module, category);
        todo!("T-9: records — one BEGIN DEFERRED")
    }

    async fn writes_since(&self, from: Position) -> Result<Vec<(Position, Write)>, StoreError> {
        let _ = from;
        todo!("T-9: writes_since — decode and validate each row")
    }

    async fn watermark(&self, peer: DeviceId) -> Result<Option<Watermark>, StoreError> {
        let _ = peer;
        todo!("T-9: watermark")
    }

    async fn advance_watermark(&self, peer: DeviceId, to: Watermark) -> Result<(), StoreError> {
        let _ = (peer, to);
        todo!("T-9: advance_watermark — max")
    }
}
