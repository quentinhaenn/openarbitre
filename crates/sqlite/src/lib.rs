//! SQLite adapter of the `Store` port, layout v1 (ST-02 archi "SQLite adapter"; plan P-1, P-7 to P-11).
#![expect(dead_code, reason = "skeleton: bodies land in T-9")]

use std::cell::RefCell;
use std::path::Path;
use std::time::Duration;

use arbitre_core::Damage;
use arbitre_core::{
    ApplyError, Cause, ClockState, DeviceId, LocalChange, Position, RecordId, RecordState, Report,
    Store, StoreError, Watermark, Write,
};
use rusqlite::{Connection, OpenFlags, OptionalExtension, Transaction, TransactionBehavior};

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

/// `PRAGMA application_id` of a store ("ARBT").
const APP_ID: i32 = 0x4152_4254;
/// Layout version supported (`PRAGMA user_version`).
const VERSION: u32 = 1;
const TABLES: [&str; 4] = ["device", "seen", "watermarks", "writes"];

/// Layout v1 (plan P-7; archi "SQLite layout v1"). Identifiers are BLOBs of UTF-8 bytes.
const DDL: &str = "
CREATE TABLE device (
    id INTEGER PRIMARY KEY CHECK (id = 0),
    device BLOB NOT NULL CHECK (length(device) = 16),
    clock_l INTEGER NOT NULL CHECK (clock_l >= 0),
    clock_c INTEGER NOT NULL CHECK (clock_c BETWEEN 0 AND 4294967295)
) STRICT;
CREATE TABLE writes (
    pos INTEGER PRIMARY KEY,
    module BLOB NOT NULL, category BLOB NOT NULL, id BLOB NOT NULL,
    field BLOB NOT NULL, value BLOB,
    l INTEGER NOT NULL, c INTEGER NOT NULL,
    a BLOB NOT NULL CHECK (length(a) = 16),
    author INTEGER NOT NULL CHECK (author IN (0, 1, 2, 3)),
    CHECK ((field = x'') = (value IS NULL))
) STRICT;
CREATE INDEX writes_record ON writes (module, category, id);
CREATE INDEX writes_stamp ON writes (l, c);
CREATE TABLE seen (
    pos INTEGER NOT NULL, kind INTEGER NOT NULL, field BLOB NOT NULL,
    l INTEGER NOT NULL, c INTEGER NOT NULL,
    a BLOB NOT NULL CHECK (length(a) = 16),
    CHECK (kind IN (0, 1)),
    CHECK (kind = 1 OR field = x''),
    PRIMARY KEY (pos, kind, field)
) STRICT, WITHOUT ROWID;
CREATE TABLE watermarks (
    peer BLOB PRIMARY KEY CHECK (length(peer) = 16),
    mark BLOB NOT NULL CHECK (length(mark) = 12)
) STRICT, WITHOUT ROWID;
";

fn io(text: impl ToString) -> StoreError {
    StoreError::Unavailable(Cause::Io(text.to_string()))
}

fn unsigned(n: i64) -> Option<u64> {
    u64::try_from(n).ok()
}

impl SqliteStore {
    /// Opens or creates the store at `path` (archi "Open sequence", P-10 open-time check).
    /// Writes nothing unless creating.
    pub fn open(path: &Path, options: &Options) -> Result<Self, StoreError> {
        let flags = OpenFlags::default();
        let mut conn = match &options.vfs {
            Some(vfs) => Connection::open_with_flags_and_vfs(path, flags, vfs.as_str()),
            None => Connection::open_with_flags(path, flags),
        }
        .map_err(map)?;
        conn.busy_timeout(options.wait).map_err(map)?;
        for (name, value) in [
            ("synchronous", "FULL"),
            ("fullfsync", "ON"),
            ("checkpoint_fullfsync", "ON"),
        ] {
            conn.pragma_update(None, name, value).map_err(map)?;
        }
        // T-13: the P-10 / P-11 open-time check lands in `inspect`.
        let device = match inspect(&mut conn)? {
            Some(d) => d,
            None => match create(&mut conn)? {
                Some(d) => d,
                None => inspect(&mut conn)?.ok_or(StoreError::Corrupted(Damage::Layout))?,
            },
        };
        Ok(Self {
            conn: RefCell::new(conn),
            device,
            poisoned: RefCell::new(None),
        })
    }

    /// Test hook: settings in effect on this connection.
    pub fn settings(&self) -> Settings {
        let conn = self.conn.borrow();
        let int = |name: &str| {
            conn.pragma_query_value(None, name, |r| r.get::<_, i64>(0))
                .expect("pragma")
        };
        Settings {
            journal_mode: conn
                .pragma_query_value(None, "journal_mode", |r| r.get::<_, String>(0))
                .expect("pragma"),
            synchronous: int("synchronous"),
            fullfsync: int("fullfsync") != 0,
            checkpoint_fullfsync: int("checkpoint_fullfsync") != 0,
        }
    }
}

/// Open sequence steps 1 and 3: `None` = empty file (to create), else the verified `D`.
fn inspect(conn: &mut Connection) -> Result<Option<DeviceId>, StoreError> {
    let tx = conn.transaction().map_err(map)?;
    let app: i32 = tx
        .pragma_query_value(None, "application_id", |r| r.get(0))
        .map_err(map)?;
    let version: i64 = tx
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .map_err(map)?;
    let mut tables = tx
        .prepare("SELECT name FROM sqlite_schema WHERE type = 'table' AND name NOT LIKE 'sqlite_%'")
        .map_err(map)?
        .query_map([], |r| r.get::<_, String>(0))
        .map_err(map)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(map)?;
    if tables.is_empty() && version == 0 && app == 0 {
        return Ok(None);
    }
    if app != APP_ID {
        return Err(StoreError::Corrupted(Damage::NotAStore));
    }
    if version > i64::from(VERSION) {
        return Err(StoreError::Unavailable(Cause::IncompatibleVersion {
            stored: u32::try_from(version).unwrap_or(u32::MAX),
            supported: VERSION,
        }));
    }
    tables.sort();
    if version < 1 || tables != TABLES {
        return Err(StoreError::Corrupted(Damage::Layout));
    }
    let row = tx
        .query_row("SELECT device, clock_l, clock_c FROM device", [], |r| {
            Ok((
                r.get::<_, Vec<u8>>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, i64>(2)?,
            ))
        })
        .optional()
        .map_err(|e| match map(e) {
            StoreError::Corrupted(Damage::StoredWrite { .. }) => {
                StoreError::Corrupted(Damage::DeviceId)
            }
            other => other,
        })?;
    let Some((device, l, c)) = row else {
        return Err(StoreError::Corrupted(Damage::DeviceId));
    };
    let device =
        <[u8; 16]>::try_from(device).map_err(|_| StoreError::Corrupted(Damage::DeviceId))?;
    let clock = unsigned(l)
        .zip(u32::try_from(c).ok())
        .map(|(l, c)| ClockState { l, c })
        .ok_or(StoreError::Corrupted(Damage::ClockState))?;
    let top: Option<(i64, i64)> = tx
        .query_row(
            "SELECT l, c FROM writes ORDER BY l DESC, c DESC LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()
        .map_err(map)?;
    if let Some((l, c)) = top {
        match unsigned(l).zip(u32::try_from(c).ok()) {
            Some((l, c)) if clock >= (ClockState { l, c }) => {}
            _ => return Err(StoreError::Corrupted(Damage::ClockState)),
        }
    }
    Ok(Some(DeviceId(device)))
}

/// Open sequence step 2. `None` = another process created the store first.
fn create(conn: &mut Connection) -> Result<Option<DeviceId>, StoreError> {
    conn.pragma_update_and_check(None, "journal_mode", "WAL", |r| r.get::<_, String>(0))
        .map_err(map)?;
    let tx = Transaction::new(conn, TransactionBehavior::Immediate).map_err(map)?;
    let n: i64 = tx
        .query_row("SELECT count(*) FROM sqlite_schema", [], |r| r.get(0))
        .map_err(map)?;
    if n != 0 {
        return Ok(None);
    }
    let mut id = [0u8; 16];
    getrandom::fill(&mut id).map_err(|_| StoreError::Unavailable(Cause::RandomSource))?;
    tx.execute_batch(DDL).map_err(map)?;
    tx.execute("INSERT INTO device VALUES (0, ?1, 0, 0)", [&id[..]])
        .map_err(map)?;
    tx.pragma_update(None, "application_id", APP_ID)
        .map_err(map)?;
    tx.pragma_update(None, "user_version", VERSION)
        .map_err(map)?;
    tx.commit().map_err(map)?;
    Ok(Some(DeviceId(id)))
}

/// Closed SQLite error mapping, first match wins (plan P-8).
fn map(e: rusqlite::Error) -> StoreError {
    use rusqlite::Error as E;
    use rusqlite::ffi;
    if let E::SqliteFailure(f, msg) = &e {
        let ext = f.extended_code;
        return match ext & 0xff {
            ffi::SQLITE_FULL => StoreError::StorageFull,
            ffi::SQLITE_BUSY | ffi::SQLITE_LOCKED => StoreError::Unavailable(Cause::Busy),
            ffi::SQLITE_NOTADB => StoreError::Corrupted(Damage::NotAStore),
            ffi::SQLITE_CORRUPT => StoreError::Corrupted(Damage::Engine(e.to_string())),
            ffi::SQLITE_IOERR if ext == ffi::SQLITE_IOERR_SHORT_READ => {
                StoreError::Corrupted(Damage::Engine(e.to_string()))
            }
            ffi::SQLITE_CONSTRAINT => StoreError::Corrupted(Damage::Engine(format!(
                "constraint: {}",
                msg.as_deref().unwrap_or("unknown")
            ))),
            _ => io(e),
        };
    }
    match e {
        E::FromSqlConversionFailure(..)
        | E::InvalidColumnType(..)
        | E::IntegralValueOutOfRange(..)
        | E::Utf8Error(..) => StoreError::Corrupted(Damage::StoredWrite { position: None }),
        other => io(other),
    }
}

impl Store for SqliteStore {
    async fn device(&self) -> Result<DeviceId, StoreError> {
        Ok(self.device)
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

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::ffi;
    use rusqlite::types::Type;

    fn fail(code: i32) -> rusqlite::Error {
        rusqlite::Error::SqliteFailure(ffi::Error::new(code), Some("x".into()))
    }

    fn is_io(e: StoreError) -> bool {
        matches!(e, StoreError::Unavailable(Cause::Io(_)))
    }

    fn is_engine(e: StoreError) -> bool {
        matches!(e, StoreError::Corrupted(Damage::Engine(_)))
    }

    #[test]
    fn full() {
        assert_eq!(map(fail(ffi::SQLITE_FULL)), StoreError::StorageFull);
        assert_eq!(
            map(fail(ffi::SQLITE_FULL | (1 << 8))),
            StoreError::StorageFull
        );
    }

    #[test]
    fn busy_and_locked() {
        for c in [
            ffi::SQLITE_BUSY,
            ffi::SQLITE_BUSY_SNAPSHOT,
            ffi::SQLITE_BUSY_RECOVERY,
            ffi::SQLITE_LOCKED,
            ffi::SQLITE_LOCKED_SHAREDCACHE,
        ] {
            assert_eq!(map(fail(c)), StoreError::Unavailable(Cause::Busy), "{c}");
        }
    }

    #[test]
    fn not_a_db() {
        assert_eq!(
            map(fail(ffi::SQLITE_NOTADB)),
            StoreError::Corrupted(Damage::NotAStore)
        );
    }

    #[test]
    fn corrupt_and_short_read() {
        for c in [
            ffi::SQLITE_CORRUPT,
            ffi::SQLITE_CORRUPT_VTAB,
            ffi::SQLITE_IOERR_SHORT_READ,
        ] {
            assert!(is_engine(map(fail(c))), "{c}");
        }
    }

    #[test]
    fn constraint_is_corruption() {
        for c in [
            ffi::SQLITE_CONSTRAINT,
            ffi::SQLITE_CONSTRAINT_CHECK,
            ffi::SQLITE_CONSTRAINT_PRIMARYKEY,
            ffi::SQLITE_CONSTRAINT_UNIQUE,
            ffi::SQLITE_CONSTRAINT_NOTNULL,
            ffi::SQLITE_CONSTRAINT_DATATYPE,
        ] {
            match map(fail(c)) {
                StoreError::Corrupted(Damage::Engine(t)) => {
                    assert!(t.starts_with("constraint: "), "{c}");
                }
                other => panic!("{c}: {other:?}"),
            }
        }
    }

    #[test]
    fn decode_errors() {
        let damage = StoreError::Corrupted(Damage::StoredWrite { position: None });
        let bad_utf8 = String::from_utf8(vec![0xff]).unwrap_err().utf8_error();
        let cases = [
            rusqlite::Error::FromSqlConversionFailure(0, Type::Blob, "x".into()),
            rusqlite::Error::InvalidColumnType(0, "c".into(), Type::Null),
            rusqlite::Error::IntegralValueOutOfRange(0, -1),
            rusqlite::Error::Utf8Error(0, bad_utf8),
        ];
        for e in cases {
            assert_eq!(map(e), damage);
        }
    }

    #[test]
    fn io_family() {
        for c in [
            ffi::SQLITE_IOERR,
            ffi::SQLITE_IOERR_FSYNC,
            ffi::SQLITE_CANTOPEN,
            ffi::SQLITE_PERM,
            ffi::SQLITE_READONLY,
            ffi::SQLITE_NOMEM,
            ffi::SQLITE_PROTOCOL,
        ] {
            assert!(is_io(map(fail(c))), "{c}");
        }
    }

    #[test]
    fn catch_all() {
        assert!(is_io(map(fail(ffi::SQLITE_INTERNAL))));
        assert!(is_io(map(rusqlite::Error::InvalidPath("p".into()))));
        assert!(is_io(map(rusqlite::Error::QueryReturnedNoRows)));
        assert!(is_io(map(rusqlite::Error::InvalidQuery)));
    }
}
