//! SQLite adapter of the `Store` port, layout v1 (ST-02 archi "SQLite adapter"; plan P-1, P-7 to P-11).

use std::cell::RefCell;
use std::path::Path;
use std::time::{Duration, Instant};

use std::collections::BTreeSet;

use arbitre_core::{
    ApplyError, Author, Cause, Change, ClockState, Damage, DeviceId, Field, LocalChange, Log,
    Position, RecordId, RecordState, Report, Seen, Stamp, Store, StoreError, Target, TargetKind,
    Value, Watermark, Write, rebuild, receive, stamp_local, validate,
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
        // `SQLITE_BUSY` can skip the busy handler while another process creates the store or
        // switches it to WAL (plan R-9): retry the whole sequence until `wait` elapses.
        let deadline = Instant::now() + options.wait;
        loop {
            match Self::open_once(path, options, deadline) {
                Err(StoreError::Unavailable(Cause::Busy)) if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(2));
                }
                other => return other,
            }
        }
    }

    fn open_once(path: &Path, options: &Options, deadline: Instant) -> Result<Self, StoreError> {
        let flags = OpenFlags::default();
        let mut conn = match &options.vfs {
            Some(vfs) => Connection::open_with_flags_and_vfs(path, flags, vfs.as_str()),
            None => Connection::open_with_flags(path, flags),
        }
        .map_err(map)?;
        // Each attempt gets only the time left, so `open` stays within `wait`.
        conn.busy_timeout(deadline.saturating_duration_since(Instant::now()))
            .map_err(map)?;
        for (name, value) in [
            ("synchronous", "FULL"),
            ("fullfsync", "ON"),
            ("checkpoint_fullfsync", "ON"),
        ] {
            conn.pragma_update(None, name, value).map_err(map)?;
        }
        let device = match inspect(&mut conn)? {
            Some(d) => d,
            None => match create(&mut conn)? {
                Some(d) => d,
                None => inspect(&mut conn)?.ok_or(StoreError::Corrupted(Damage::Layout))?,
            },
        };
        wal(&conn)?;
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
    let device = DeviceId(device);
    check_stored(&tx, device)?;
    // Page-level damage the row checks cannot see (Q-P6), last so row damage keeps its precise variant.
    let report = tx
        .prepare("PRAGMA quick_check")
        .map_err(map)?
        .query_map([], |r| r.get::<_, String>(0))
        .map_err(map)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(map)?;
    if report != ["ok"] {
        return Err(StoreError::Corrupted(Damage::Engine(report.join("; "))));
    }
    Ok(Some(device))
}

/// Open-time check (P-10, P-11): every stored write decodes and validates, no orphan `seen`
/// row, no local identity stored twice (cost measured by `tests/damage.rs :: open_check_timing`).
fn check_stored(tx: &Transaction, device: DeviceId) -> Result<(), StoreError> {
    load(tx, "", [])?;
    let orphan: Option<i64> = tx
        .query_row(
            "SELECT min(pos) FROM seen WHERE pos NOT IN (SELECT pos FROM writes)",
            [],
            |r| r.get(0),
        )
        .map_err(map)?;
    if let Some(pos) = orphan {
        return Err(bad(pos));
    }
    let pair = tx
        .query_row(
            "SELECT module, category, id, field, l, c FROM writes WHERE a = ?1
             GROUP BY module, category, id, field, l, c HAVING count(*) > 1 LIMIT 1",
            [&device.0[..]],
            |r| {
                Ok((
                    [r.get::<_, Vec<u8>>(0)?, r.get(1)?, r.get(2)?, r.get(3)?],
                    r.get::<_, i64>(4)?,
                    r.get::<_, i64>(5)?,
                ))
            },
        )
        .optional()
        .map_err(map)?;
    let Some(([module, category, id, field], l, c)) = pair else {
        return Ok(());
    };
    // Decoded writes passed `validate`: identifiers are UTF-8, `l`/`c` in range.
    let utf8 = |b| text(b).unwrap_or_default();
    let stamp = stamp_of(l, c, device.0.to_vec());
    let record = RecordId {
        module: utf8(module),
        category: utf8(category),
        id: utf8(id),
    };
    let kind = match utf8(field) {
        f if f.is_empty() => TargetKind::Delete,
        f => TargetKind::Field(Field(f)),
    };
    Err(StoreError::Corrupted(Damage::LocalIdentityPair {
        target: Box::new(Target { record, kind }),
        stamp: stamp.ok_or(StoreError::Corrupted(Damage::Layout))?,
    }))
}

/// WAL switch at every open (no-op when already WAL; plan R-9, F-21).
fn wal(conn: &Connection) -> Result<(), StoreError> {
    conn.pragma_update_and_check(None, "journal_mode", "WAL", |r| r.get::<_, String>(0))
        .map_err(map)?;
    Ok(())
}

/// Open sequence step 2. `None` = another process created the store first.
fn create(conn: &mut Connection) -> Result<Option<DeviceId>, StoreError> {
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

const AUTHORS: [Author; 4] = [
    Author::CliUser,
    Author::UiUser,
    Author::Assistant,
    Author::System,
];

const WRITE_COLS: &str = "pos, module, category, id, field, value, l, c, a, author";

fn bad(pos: i64) -> StoreError {
    StoreError::Corrupted(Damage::StoredWrite {
        position: unsigned(pos).map(Position),
    })
}

fn stamp_of(l: i64, c: i64, a: Vec<u8>) -> Option<Stamp> {
    Some(Stamp {
        l: unsigned(l)?,
        c: u32::try_from(c).ok()?,
        a: DeviceId(<[u8; 16]>::try_from(a).ok()?),
    })
}

fn text(b: Vec<u8>) -> Option<String> {
    String::from_utf8(b).ok()
}

/// Decodes the `writes` row at the cursor, loads its `seen` rows and validates it (RG-ST02-5.5).
fn decode(tx: &Transaction, r: &rusqlite::Row) -> Result<(Position, Write), StoreError> {
    let pos: i64 = r.get(0).map_err(map)?;
    let bad = || bad(pos);
    let blob = |i| r.get::<_, Vec<u8>>(i).map_err(|_| bad());
    let (module, category, id) = (blob(1)?, blob(2)?, blob(3)?);
    let field = blob(4)?;
    let value: Option<Vec<u8>> = r.get(5).map_err(|_| bad())?;
    let change = match (field.is_empty(), value) {
        (true, None) => Change::Delete,
        (false, Some(v)) => Change::Field {
            name: Field(text(field).ok_or_else(bad)?),
            value: Value(v),
        },
        _ => return Err(bad()),
    };
    let int = |i| r.get::<_, i64>(i).map_err(|_| bad());
    let stamp = stamp_of(int(6)?, int(7)?, blob(8)?).ok_or_else(bad)?;
    let author = usize::try_from(int(9)?)
        .ok()
        .and_then(|i| AUTHORS.get(i).copied())
        .ok_or_else(bad)?;
    let mut seen = Seen::new();
    let mut st = tx
        .prepare_cached("SELECT kind, field, l, c, a FROM seen WHERE pos = ?1")
        .map_err(map)?;
    let mut rows = st.query([pos]).map_err(map)?;
    while let Some(s) = rows.next().map_err(map)? {
        let get = |i| s.get::<_, Vec<u8>>(i).map_err(|_| bad());
        let kind = match (s.get::<_, i64>(0).map_err(|_| bad())?, get(1)?) {
            (0, f) if f.is_empty() => TargetKind::Delete,
            (1, f) => TargetKind::Field(Field(text(f).ok_or_else(bad)?)),
            _ => return Err(bad()),
        };
        let m = stamp_of(
            s.get(2).map_err(|_| bad())?,
            s.get(3).map_err(|_| bad())?,
            get(4)?,
        )
        .ok_or_else(bad)?;
        seen.insert(kind, m);
    }
    let w = Write {
        record: RecordId {
            module: text(module).ok_or_else(bad)?,
            category: text(category).ok_or_else(bad)?,
            id: text(id).ok_or_else(bad)?,
        },
        change,
        stamp,
        seen,
        author,
    };
    validate(std::slice::from_ref(&w)).map_err(|_| bad())?;
    Ok((Position(pos as u64), w))
}

/// Stored writes matching `filter` (a `WHERE` tail), position order, decoded and validated.
fn load(
    tx: &Transaction,
    filter: &str,
    params: impl rusqlite::Params,
) -> Result<Vec<(Position, Write)>, StoreError> {
    let mut st = tx
        .prepare_cached(&format!(
            "SELECT {WRITE_COLS} FROM writes {filter} ORDER BY pos"
        ))
        .map_err(map)?;
    let mut rows = st.query(params).map_err(map)?;
    let mut out = Vec::new();
    while let Some(r) = rows.next().map_err(map)? {
        out.push(decode(tx, r)?);
    }
    Ok(out)
}

fn load_record(tx: &Transaction, r: &RecordId) -> Result<Vec<Write>, StoreError> {
    let rows = load(
        tx,
        "WHERE module = ?1 AND category = ?2 AND id = ?3",
        (r.module.as_bytes(), r.category.as_bytes(), r.id.as_bytes()),
    )?;
    Ok(rows.into_iter().map(|(_, w)| w).collect())
}

/// Log built from stored writes (read paths).
fn log_of(device: DeviceId, writes: &[Write]) -> Result<Log, StoreError> {
    let mut log = Log::new(device);
    log.merge(writes)
        .map_err(|_| StoreError::Corrupted(Damage::StoredWrite { position: None }))?;
    match log.anomalies().into_iter().find(|a| a.stamp.a == device) {
        Some(a) => Err(StoreError::Corrupted(Damage::LocalIdentityPair {
            target: Box::new(a.target),
            stamp: a.stamp,
        })),
        None => Ok(log),
    }
}

fn clock(tx: &Transaction) -> Result<ClockState, StoreError> {
    let (l, c): (i64, i64) = tx
        .query_row("SELECT clock_l, clock_c FROM device", [], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .map_err(|_| StoreError::Corrupted(Damage::ClockState))?;
    unsigned(l)
        .zip(u32::try_from(c).ok())
        .map(|(l, c)| ClockState { l, c })
        .ok_or(StoreError::Corrupted(Damage::ClockState))
}

fn insert(tx: &Transaction, w: &Write) -> Result<(), StoreError> {
    let (field, value): (&[u8], Option<&[u8]>) = match &w.change {
        Change::Field { name, value } => (name.0.as_bytes(), Some(&value.0)),
        Change::Delete => (b"", None),
    };
    // Exhaustive: a new `Author` must get a code here and in `AUTHORS`.
    let author: i64 = match w.author {
        Author::CliUser => 0,
        Author::UiUser => 1,
        Author::Assistant => 2,
        Author::System => 3,
    };
    tx.prepare_cached(
        "INSERT INTO writes (module, category, id, field, value, l, c, a, author)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
    )
    .map_err(map)?
    .execute((
        w.record.module.as_bytes(),
        w.record.category.as_bytes(),
        w.record.id.as_bytes(),
        field,
        value,
        w.stamp.l as i64,
        w.stamp.c,
        &w.stamp.a.0[..],
        author,
    ))
    .map_err(map)?;
    let pos = tx.last_insert_rowid();
    let mut st = tx
        .prepare_cached(
            "INSERT INTO seen (pos, kind, field, l, c, a) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )
        .map_err(map)?;
    for (kind, m) in &w.seen {
        let (k, f): (i64, &[u8]) = match kind {
            TargetKind::Delete => (0, b""),
            TargetKind::Field(n) => (1, n.0.as_bytes()),
        };
        st.execute((pos, k, f, m.l as i64, m.c, &m.a.0[..]))
            .map_err(map)?;
    }
    Ok(())
}

fn set_clock(tx: &Transaction, k: ClockState) -> Result<(), StoreError> {
    tx.execute(
        "UPDATE device SET clock_l = ?1, clock_c = ?2",
        (k.l as i64, k.c),
    )
    .map_err(map)?;
    Ok(())
}

/// Stored writes of the given records.
fn known(
    tx: &Transaction,
    records: impl Iterator<Item = RecordId>,
) -> Result<Vec<Write>, StoreError> {
    let mut out = Vec::new();
    for r in records.collect::<BTreeSet<_>>() {
        out.extend(load_record(tx, &r)?);
    }
    Ok(out)
}

fn mark(tx: &Transaction, peer: DeviceId) -> Result<Option<Watermark>, StoreError> {
    let b: Option<Vec<u8>> = tx
        .query_row(
            "SELECT mark FROM watermarks WHERE peer = ?1",
            [&peer.0[..]],
            |r| r.get(0),
        )
        .optional()
        .map_err(map)?;
    let Some(b) = b else { return Ok(None) };
    let b = <[u8; 12]>::try_from(b).map_err(|_| StoreError::Corrupted(Damage::Layout))?;
    let (l, c) = b.split_at(8);
    Ok(Some(Watermark(
        u64::from_be_bytes(l.try_into().expect("8 bytes")),
        u32::from_be_bytes(c.try_into().expect("4 bytes")),
    )))
}

impl SqliteStore {
    /// Fails with the stored damage once the handle is poisoned (RG-ST02-7.2).
    fn check_poison(&self) -> Result<(), StoreError> {
        match self.poisoned.borrow().as_ref() {
            Some(d) => Err(StoreError::Corrupted(d.clone())),
            None => Ok(()),
        }
    }

    /// Runs `f` in one transaction (`IMMEDIATE` for writes), commits on `Ok`, rolls back otherwise.
    /// The first `Corrupted` poisons the handle (RG-ST02-7.2).
    fn exec<T>(
        &self,
        behavior: TransactionBehavior,
        f: impl FnOnce(&Transaction) -> Result<T, ApplyError>,
    ) -> Result<T, ApplyError> {
        self.check_poison()?;
        let mut conn = self.conn.borrow_mut();
        let out = Transaction::new(&mut conn, behavior)
            .map_err(|e| ApplyError::Store(map(e)))
            .and_then(|tx| {
                let v = f(&tx)?;
                tx.commit().map_err(|e| ApplyError::Store(map(e)))?;
                Ok(v)
            });
        if let Err(ApplyError::Store(StoreError::Corrupted(d))) = &out {
            *self.poisoned.borrow_mut() = Some(d.clone());
        }
        out
    }

    fn run<T>(
        &self,
        behavior: TransactionBehavior,
        f: impl FnOnce(&Transaction) -> Result<T, StoreError>,
    ) -> Result<T, StoreError> {
        match self.exec(behavior, |tx| Ok(f(tx)?)) {
            Ok(v) => Ok(v),
            Err(ApplyError::Store(e)) => Err(e),
            Err(ApplyError::Rejected(_)) => unreachable!("no rejection here"),
        }
    }

    fn read<T>(
        &self,
        f: impl FnOnce(&Transaction) -> Result<T, StoreError>,
    ) -> Result<T, StoreError> {
        self.run(TransactionBehavior::Deferred, f)
    }
}

impl Store for SqliteStore {
    async fn device(&self) -> Result<DeviceId, StoreError> {
        self.check_poison()?;
        Ok(self.device)
    }

    async fn apply_local(
        &self,
        pt: u64,
        changes: &[LocalChange],
    ) -> Result<Vec<Write>, ApplyError> {
        self.check_poison()?;
        // `stamp_local` on an empty list returns a meaningless `K'`.
        if changes.is_empty() {
            return Ok(Vec::new());
        }
        self.exec(TransactionBehavior::Immediate, |tx| {
            let k = clock(tx)?;
            let known = known(tx, changes.iter().map(|c| c.record.clone()))?;
            let mut log = rebuild(self.device, k, &known).map_err(StoreError::Corrupted)?;
            let (writes, k) = stamp_local(&mut log, pt, changes).map_err(ApplyError::Rejected)?;
            for w in &writes {
                insert(tx, w)?;
            }
            set_clock(tx, k)?;
            Ok(writes)
        })
    }

    async fn apply(&self, batch: &[Write]) -> Result<Report, ApplyError> {
        self.exec(TransactionBehavior::Immediate, |tx| {
            let k = clock(tx)?;
            let known = known(tx, batch.iter().map(|w| w.record.clone()))?;
            let mut log = rebuild(self.device, k, &known).map_err(StoreError::Corrupted)?;
            let got = receive(self.device, &mut log, k, batch).map_err(ApplyError::Rejected)?;
            for w in &got.new {
                insert(tx, w)?;
            }
            set_clock(tx, got.clock)?;
            Ok(got.report)
        })
    }

    async fn writes_of(&self, record: &RecordId) -> Result<Vec<Write>, StoreError> {
        self.read(|tx| Ok(log_of(self.device, &load_record(tx, record)?)?.writes()))
    }

    async fn state(&self, record: &RecordId) -> Result<Option<RecordState>, StoreError> {
        self.read(|tx| {
            Ok(log_of(self.device, &load_record(tx, record)?)?
                .projection()
                .remove(record))
        })
    }

    async fn records(
        &self,
        module: &str,
        category: &str,
    ) -> Result<Vec<(RecordId, RecordState)>, StoreError> {
        self.read(|tx| {
            let rows = load(
                tx,
                "WHERE module = ?1 AND category = ?2",
                (module.as_bytes(), category.as_bytes()),
            )?;
            let writes: Vec<Write> = rows.into_iter().map(|(_, w)| w).collect();
            Ok(log_of(self.device, &writes)?
                .projection()
                .into_iter()
                .collect())
        })
    }

    async fn writes_since(&self, from: Position) -> Result<Vec<(Position, Write)>, StoreError> {
        let from = i64::try_from(from.0).unwrap_or(i64::MAX);
        self.read(|tx| load(tx, "WHERE pos > ?1", [from]))
    }

    async fn watermark(&self, peer: DeviceId) -> Result<Option<Watermark>, StoreError> {
        self.read(|tx| mark(tx, peer))
    }

    async fn advance_watermark(&self, peer: DeviceId, to: Watermark) -> Result<(), StoreError> {
        self.run(TransactionBehavior::Immediate, |tx| {
            let to = mark(tx, peer)?.map_or(to, |m| m.max(to));
            let mut b = to.0.to_be_bytes().to_vec();
            b.extend(to.1.to_be_bytes());
            tx.execute(
                "INSERT OR REPLACE INTO watermarks (peer, mark) VALUES (?1, ?2)",
                (&peer.0[..], b),
            )
            .map_err(map)?;
            Ok(())
        })
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
