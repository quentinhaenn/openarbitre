//! Open, creation, verification and settings of the SQLite adapter (T-8; US-1 case 11, US-4, US-7).

#[path = "support/common.rs"]
mod common;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use arbitre_core::{
    ApplyError, Author, Cause, Change, Damage, DeviceId, Field, FieldValue, LocalChange, Position,
    RecordId, Rejection, Seen, Stamp, Store, StoreError, TargetKind, Watermark, Write, ready,
};
use arbitre_sqlite::{Options, SqliteStore};
use common::{fresh_dir, open, raw_k, rec};
use rusqlite::Connection;

fn path() -> PathBuf {
    fresh_dir("settings").join("store.db")
}

fn device(s: &SqliteStore) -> DeviceId {
    ready(s.device()).unwrap()
}

fn check_settings(s: &SqliteStore) {
    let st = s.settings();
    assert_eq!(st.journal_mode, "wal");
    assert_eq!(st.synchronous, 2);
    assert!(st.fullfsync);
    assert!(st.checkpoint_fullfsync);
}

#[test]
fn settings_on_fresh_and_reopened_handles() {
    let p = path();
    check_settings(&open(&p));
    check_settings(&open(&p));
}

#[test]
fn creation_and_reopen() {
    let p = path();
    let d = device(&open(&p));
    assert_ne!(d, DeviceId([0; 16]));
    assert_eq!(device(&open(&p)), d);
    let raw = Connection::open(&p).unwrap();
    let get = |n: &str| {
        raw.pragma_query_value(None, n, |r| r.get::<_, i64>(0))
            .unwrap()
    };
    assert_eq!(get("user_version"), 1);
    assert_eq!(get("application_id"), 0x4152_4254);
    assert_eq!(raw_k(&raw), (0, 0));
}

#[test]
fn thousand_distinct_devices() {
    let dir = path().parent().unwrap().to_owned();
    let ids: BTreeSet<_> = (0..1000)
        .map(|i| {
            let d = device(&open(&dir.join(format!("s{i}.db"))));
            assert_ne!(d, DeviceId([0; 16]));
            d
        })
        .collect();
    assert_eq!(ids.len(), 1000);
}

#[test]
fn newer_version_refused_file_untouched() {
    let p = path();
    drop(open(&p));
    {
        let raw = Connection::open(&p).unwrap();
        raw.pragma_update(None, "user_version", 2).unwrap();
    }
    let before = std::fs::read(&p).unwrap();
    let err = SqliteStore::open(&p, &Options::default())
        .err()
        .expect("refused");
    assert_eq!(
        err,
        StoreError::Unavailable(Cause::IncompatibleVersion {
            stored: 2,
            supported: 1
        })
    );
    assert_eq!(std::fs::read(&p).unwrap(), before);
}

#[test]
fn layout_v1_schema() {
    let p = path();
    drop(open(&p));
    let raw = Connection::open(&p).unwrap();
    let seen: String = raw
        .query_row(
            "SELECT sql FROM sqlite_schema WHERE name = 'seen'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    let seen = seen.split_whitespace().collect::<Vec<_>>().join(" ");
    for part in [
        "pos INTEGER NOT NULL",
        "kind INTEGER NOT NULL",
        "field BLOB NOT NULL",
        "CHECK (kind IN (0, 1))",
        "CHECK (kind = 1 OR field = x'')",
        "PRIMARY KEY (pos, kind, field)",
        "STRICT, WITHOUT ROWID",
    ] {
        assert!(seen.contains(part), "missing {part:?} in {seen}");
    }
    let cols: Vec<String> = raw
        .prepare("SELECT name FROM pragma_table_info('seen')")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(cols, ["pos", "kind", "field", "l", "c", "a"]);
}

#[test]
fn foreign_file_is_not_a_store() {
    let p = path();
    std::fs::write(&p, b"this is not a database, just text ".repeat(200)).unwrap();
    assert_eq!(
        SqliteStore::open(&p, &Options::default()).err(),
        Some(StoreError::Corrupted(Damage::NotAStore))
    );
}

fn put(id: &str, v: &[u8]) -> LocalChange {
    LocalChange {
        record: rec("c", id),
        change: Change::field("f", v),
        author: Author::CliUser,
    }
}

fn peer_write(l: u64) -> Write {
    Write {
        record: rec("c", "p"),
        change: Change::Delete,
        stamp: Stamp {
            l,
            c: 0,
            a: DeviceId([9; 16]),
        },
        seen: Seen::new(),
        author: Author::System,
    }
}

fn busy() -> ApplyError {
    ApplyError::Store(StoreError::Unavailable(Cause::Busy))
}

#[test]
fn every_operation_is_ready_on_first_poll() {
    let s = open(&path());
    let peer = DeviceId([9; 16]);
    ready(s.device()).unwrap();
    ready(s.apply_local(1, &[put("a", b"x")])).unwrap();
    ready(s.apply(&[peer_write(5)])).unwrap();
    ready(s.writes_of(&rec("c", "a"))).unwrap();
    ready(s.state(&rec("c", "a"))).unwrap();
    ready(s.records("m", "c")).unwrap();
    ready(s.writes_since(Position::START)).unwrap();
    ready(s.watermark(peer)).unwrap();
    ready(s.advance_watermark(peer, Watermark(1, 2))).unwrap();
    assert_eq!(ready(s.watermark(peer)), Ok(Some(Watermark(1, 2))));
    ready(s.advance_watermark(peer, Watermark(1, 1))).unwrap();
    assert_eq!(ready(s.watermark(peer)), Ok(Some(Watermark(1, 2))));
}

/// Raw connection holding `BEGIN IMMEDIATE` for `d`; the lock is taken on return.
fn hold(p: &Path, d: Duration) -> std::thread::JoinHandle<()> {
    let raw = Connection::open(p).unwrap();
    raw.execute_batch("BEGIN IMMEDIATE").unwrap();
    std::thread::spawn(move || {
        std::thread::sleep(d);
        raw.execute_batch("ROLLBACK").unwrap();
    })
}

/// SQLite's busy handler sleeps in steps that can sum a hair under the
/// timeout (CI measured 199.9 ms for 200 ms), so lower bounds allow this slack.
const SLACK: Duration = Duration::from_millis(20);

fn open_wait(p: &Path, ms: u64) -> SqliteStore {
    let o = Options {
        wait: Duration::from_millis(ms),
        ..Options::default()
    };
    SqliteStore::open(p, &o).expect("open")
}

#[test]
fn busy_store_waits_then_reports_busy() {
    let p = path();
    let s = open_wait(&p, 200);
    let h = hold(&p, Duration::from_millis(400));
    let t = Instant::now();
    let r = ready(s.apply_local(1, &[put("a", b"x")]));
    let el = t.elapsed();
    assert_eq!(r, Err(busy()));
    assert!(
        el >= Duration::from_millis(200) - SLACK && el <= Duration::from_millis(1200),
        "{el:?}"
    );
    h.join().unwrap();
    assert_eq!(ready(s.writes_since(Position::START)), Ok(vec![]));
    ready(s.apply_local(1, &[put("a", b"x")])).expect("retry after release");
}

#[test]
fn short_hold_succeeds() {
    let p = path();
    let s = open_wait(&p, 200);
    let h = hold(&p, Duration::from_millis(100));
    ready(s.apply_local(1, &[put("a", b"x")])).expect("waits the lock out");
    h.join().unwrap();
    assert_eq!(ready(s.writes_since(Position::START)).unwrap().len(), 1);
}

#[test]
fn open_busy_is_bounded_by_wait() {
    let p = path();
    let raw = Connection::open(&p).unwrap();
    raw.execute_batch("CREATE TABLE t (x); BEGIN EXCLUSIVE")
        .unwrap();
    let wait = Duration::from_millis(200);
    let o = Options {
        wait,
        ..Options::default()
    };
    let t = Instant::now();
    let r = SqliteStore::open(&p, &o);
    let el = t.elapsed();
    assert_eq!(r.err(), Some(StoreError::Unavailable(Cause::Busy)));
    assert!(
        el >= wait - SLACK && el < wait + Duration::from_secs(1),
        "{el:?}"
    );
}

#[test]
fn zero_wait_fails_fast() {
    let p = path();
    let s = open_wait(&p, 0);
    let h = hold(&p, Duration::from_millis(300));
    let t = Instant::now();
    assert_eq!(ready(s.apply(&[peer_write(5)])), Err(busy()));
    assert!(t.elapsed() < Duration::from_millis(100));
    h.join().unwrap();
}

fn sigma(p: &Path) -> (i64, i64, i64) {
    let raw = Connection::open(p).unwrap();
    let n = raw
        .query_row("SELECT count(*) FROM writes", [], |r| r.get(0))
        .unwrap();
    let (l, c) = raw_k(&raw);
    (n, l, c)
}

#[test]
fn rejections_leave_the_store_unchanged_and_the_handle_usable() {
    let p = path();
    let s = open(&p);
    ready(s.apply_local(5, &[put("a", b"x")])).unwrap();
    let before = sigma(&p);
    let me = device(&s);

    let mut bad = peer_write(3);
    bad.record.module.clear();
    assert!(matches!(
        ready(s.apply(&[peer_write(100), bad])),
        Err(ApplyError::Rejected(Rejection::Batch(_)))
    ));
    let empty = LocalChange {
        record: rec("c", ""),
        ..put("a", b"y")
    };
    assert!(matches!(
        ready(s.apply_local(9, &[put("b", b"y"), empty])),
        Err(ApplyError::Rejected(Rejection::Local { index: 1, .. }))
    ));
    let mut reused = ready(s.writes_since(Position::START)).unwrap()[0].1.clone();
    reused.author = Author::System;
    assert_eq!(
        ready(s.apply(&[peer_write(100), reused])),
        Err(ApplyError::Rejected(Rejection::LocalIdentityReused {
            index: 1
        }))
    );
    assert_eq!(sigma(&p), before);
    assert_ne!(me, DeviceId([9; 16]));

    ready(s.apply_local(9, &[put("b", b"y")])).expect("handle not poisoned");
    drop(s);
    ready(open(&p).apply(&[])).unwrap();
}

#[test]
fn unknown_local_identity_is_accepted_and_raises_the_clock() {
    let p = path();
    let s = open(&p);
    let me = device(&s);
    let mut w = peer_write(50);
    w.stamp.a = me;
    ready(s.apply(&[w])).unwrap();
    assert_eq!(sigma(&p), (1, 50, 0));
    let out = ready(s.apply_local(1, &[put("a", b"x")])).unwrap();
    assert!(out[0].stamp > Stamp { l: 50, c: 0, a: me });
}

#[test]
fn empty_apply_local_is_a_noop() {
    let p = path();
    let s = open(&p);
    assert_eq!(ready(s.apply_local(7, &[])), Ok(vec![]));
    assert_eq!(sigma(&p), (0, 0, 0));
}

#[test]
fn round_trip_edge_values() {
    let p = path();
    let s = open(&p);
    let mut w = peer_write((1 << 53) - 1);
    w.stamp.c = u32::MAX;
    w.record = RecordId {
        module: "é\0😀".into(),
        category: "c".into(),
        id: "\0".into(),
    };
    w.change = Change::Field {
        name: Field("f".into()),
        value: FieldValue(vec![0xff, 0, 0xfe]),
    };
    let older = |l| Stamp {
        l,
        c: 0,
        a: DeviceId([1; 16]),
    };
    w.seen = [
        (TargetKind::Delete, older(1)),
        (TargetKind::Field(Field(String::new())), older(2)),
        (TargetKind::Field(Field("f".into())), older(3)),
    ]
    .into();
    ready(s.apply(std::slice::from_ref(&w))).unwrap();
    let reopened = open(&p);
    for s in [&s, &reopened] {
        assert_eq!(ready(s.writes_of(&w.record)), Ok(vec![w.clone()]));
        let all = ready(s.writes_since(Position::START)).unwrap();
        assert_eq!(all, vec![(all[0].0, w.clone())]);
    }
}

/// Q-3: the default busy wait is 5 s.
#[test]
fn default_wait_is_five_seconds() {
    assert_eq!(Options::default().wait, std::time::Duration::from_secs(5));
}
