//! Open, creation, verification and settings of the SQLite adapter (T-8; US-1 case 11, US-4, US-7).

use std::collections::BTreeSet;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::task::{Context, Poll, Waker};

use arbitre_core::{Cause, Damage, DeviceId, Store, StoreError};
use arbitre_sqlite::{Options, SqliteStore};
use rusqlite::Connection;

fn path() -> PathBuf {
    static N: AtomicUsize = AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "arbitre-settings-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("store.db")
}

fn open(p: &Path) -> SqliteStore {
    SqliteStore::open(p, &Options::default()).expect("open")
}

fn device(s: &SqliteStore) -> DeviceId {
    let mut f = std::pin::pin!(s.device());
    match f.as_mut().poll(&mut Context::from_waker(Waker::noop())) {
        Poll::Ready(r) => r.unwrap(),
        Poll::Pending => panic!("pending"),
    }
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
    let (l, c): (i64, i64) = raw
        .query_row("SELECT clock_l, clock_c FROM device", [], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .unwrap();
    assert_eq!((l, c), (0, 0));
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
