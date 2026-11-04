//! E-ST02-2: full storage on the SQLite adapter, through a fault VFS (plan T-12; RG-ST02-2.1 to 2.4,
//! RG-ST02-1.3, RG-ST02-4.1). Four levels, 100 cases each; each case asserts the budget was crossed.
#[path = "support/full_vfs.rs"]
mod full_vfs;

use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::task::{Context, Poll, Waker};

use arbitre_core::{
    ApplyError, Author, Cause, Change, DeviceId, Field, FieldValue, LocalChange, Position,
    RecordId, Store, StoreError, Watermark, Write,
};
use arbitre_sqlite::SqliteStore;
use full_vfs::FullVfs;
use proptest::prelude::*;
use proptest::test_runner::{Config, RngSeed, TestRunner};
use rusqlite::Connection;

const CASES: u32 = 100;

fn block_on<T>(f: impl Future<Output = T>) -> T {
    match std::pin::pin!(f)
        .as_mut()
        .poll(&mut Context::from_waker(Waker::noop()))
    {
        Poll::Ready(r) => r,
        Poll::Pending => panic!("pending"),
    }
}

fn runner(seed: u64, cases: u32) -> TestRunner {
    TestRunner::new(Config {
        cases,
        rng_seed: RngSeed::Fixed(seed),
        failure_persistence: None,
        ..Config::default()
    })
}

fn dir() -> PathBuf {
    static N: AtomicUsize = AtomicUsize::new(0);
    let d = std::env::temp_dir().join(format!(
        "arbitre-full-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn peer(n: u128) -> DeviceId {
    DeviceId::from_u128(n)
}

/// `n` field changes over 3 records, values of `len` bytes, unique per `(tag, i)`.
fn changes(tag: u8, n: usize, len: usize) -> Vec<LocalChange> {
    (0..n)
        .map(|i| LocalChange {
            record: RecordId {
                module: "m".into(),
                category: "g".into(),
                id: format!("r{}", i % 3),
            },
            change: Change::Field {
                name: Field(format!("f{}", i % 2)),
                value: FieldValue((0..len).map(|j| tag ^ i as u8 ^ j as u8).collect()),
            },
            author: Author::CliUser,
        })
        .collect()
}

/// Σ of a store: what a full storage must leave untouched (RG-ST02-2.1).
#[derive(Debug, PartialEq)]
struct Sigma {
    device: DeviceId,
    writes: Vec<(Position, Write)>,
    marks: Vec<Option<Watermark>>,
    /// Persisted clock `K` and layout version `V`, read by a raw connection.
    k: (i64, i64),
    v: i64,
}

fn sigma(s: &SqliteStore, path: &Path) -> Sigma {
    let raw = Connection::open(path).unwrap();
    Sigma {
        device: block_on(s.device()).unwrap(),
        writes: block_on(s.writes_since(Position(0))).unwrap(),
        marks: [1, 2, 3]
            .map(|n| block_on(s.watermark(peer(n))).unwrap())
            .to_vec(),
        k: raw
            .query_row("SELECT clock_l, clock_c FROM device", [], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })
            .unwrap(),
        v: raw
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .unwrap(),
    }
}

fn open(vfs: &FullVfs, path: &Path) -> SqliteStore {
    SqliteStore::open(path, &vfs.options()).expect("open")
}

/// A store with `prefix` local batches and one watermark.
fn build(vfs: &FullVfs, path: &Path, prefix: &[usize], len: usize) -> SqliteStore {
    vfs.unlimited();
    let s = open(vfs, path);
    for (i, &n) in prefix.iter().enumerate() {
        block_on(s.apply_local(100 + i as u64, &changes(i as u8, n, len))).unwrap();
    }
    block_on(s.advance_watermark(peer(1), Watermark(3, 0))).unwrap();
    s
}

#[derive(Debug, Clone, Copy)]
enum Level {
    /// (a) before the batch: budget 0.
    Before,
    /// (b) inside the batch's frames.
    Inside,
    /// (c) all but the commit frame.
    AtCommit,
    /// (d) during `advance_watermark`.
    Watermark,
}

#[derive(Debug, Clone)]
struct Case {
    prefix: Vec<usize>,
    k: usize,
    len: usize,
    /// Apply the batch as received writes instead of a local apply.
    received: bool,
    /// Retry on the same handle instead of a reopened one.
    same_handle: bool,
    /// Position of the budget inside its range, in thousandths.
    frac: u64,
}

fn cases() -> impl Strategy<Value = Case> {
    (
        prop::collection::vec(1..=20usize, 0..=3),
        1..=20usize,
        1..=2000usize,
        any::<bool>(),
        any::<bool>(),
        0..1000u64,
    )
        .prop_map(|(prefix, k, len, received, same_handle, frac)| Case {
            prefix,
            k,
            len,
            received,
            same_handle,
            frac,
        })
}

fn is_full<T: std::fmt::Debug>(r: Result<T, ApplyError>) -> bool {
    matches!(r, Err(ApplyError::Store(StoreError::StorageFull)))
}

/// One case: dry run to measure the batch's `xWrite` calls, then the real run at `level`.
fn run_case(vfs: &FullVfs, level: Level, c: &Case) {
    let work = dir();
    let (dry_path, path) = (work.join("dry.db"), work.join("store.db"));
    let batch = changes(200, c.k, c.len);

    // Dry run on a twin store: frames of the batch (header + page per frame).
    let dry = build(vfs, &dry_path, &c.prefix, c.len);
    vfs.unlimited();
    let mut dry_writes = Vec::new();
    match level {
        Level::Watermark => block_on(dry.advance_watermark(peer(2), Watermark(5, 1))).unwrap(),
        _ => dry_writes = block_on(dry.apply_local(1000, &batch)).unwrap(),
    }
    let log = vfs.log();
    let total: u64 = log.iter().map(|&n| n as u64).sum();
    let commit_frame = (log[log.len() - 2] + log[log.len() - 1]) as u64;
    let body = total - commit_frame;
    assert!(body >= 2 || matches!(level, Level::Watermark), "{log:?}");
    let budget = match level {
        Level::Before => 0,
        Level::Inside => 1 + (body - 1) * c.frac / 1000,
        Level::AtCommit => body,
        Level::Watermark => (total - 1) * c.frac / 1000,
    };

    let store = build(vfs, &path, &c.prefix, c.len);
    let before = sigma(&store, &path);
    let attempt = |s: &SqliteStore| match (level, c.received) {
        (Level::Watermark, _) => block_on(s.advance_watermark(peer(2), Watermark(5, 1)))
            .map_err(ApplyError::Store)
            .map(|()| Vec::new()),
        (_, false) => block_on(s.apply_local(1000, &batch)),
        (_, true) => block_on(s.apply(&dry_writes)).map(|_| Vec::new()),
    };

    vfs.set_budget(budget);
    let first = attempt(&store);
    assert!(is_full(first), "{level:?} {c:?} budget {budget} of {total}");
    assert!(vfs.refused() >= 1, "budget {budget} of {total} not crossed");
    vfs.unlimited();
    assert_eq!(
        sigma(&store, &path),
        before,
        "live handle after StorageFull"
    );

    let store = if c.same_handle {
        store
    } else {
        drop(store);
        let s = open(vfs, &path);
        assert_eq!(sigma(&s, &path), before, "after reopen");
        s
    };
    let stamps = attempt(&store).expect("retry after space is freed");
    let after = sigma(&store, &path);
    let n0 = before.writes.len();
    match level {
        Level::Watermark => {
            assert_eq!(after.writes, before.writes);
            assert_eq!(after.marks[1], Some(Watermark(5, 1)));
            assert_eq!(after.k, before.k);
        }
        _ => {
            assert_eq!(
                after.writes.len(),
                n0 + c.k,
                "one application, no doubled write"
            );
            assert_eq!(after.writes[..n0], before.writes[..]);
            let last = before.writes.last().map_or(0, |(p, _)| p.0);
            for (i, (p, w)) in after.writes[n0..].iter().enumerate() {
                assert_eq!(p.0, last + 1 + i as u64, "no position gap");
                assert_eq!((&w.record, &w.change), (&batch[i].record, &batch[i].change));
                if let Some(s) = stamps.get(i) {
                    assert_eq!(s, w, "returned write = stored write");
                    let below = before
                        .writes
                        .iter()
                        .map(|(_, o)| (o.stamp.l, o.stamp.c))
                        .max();
                    assert!(
                        Some((w.stamp.l, w.stamp.c)) > below,
                        "stamp above `W_before`"
                    );
                }
            }
        }
    }
    if c.same_handle {
        drop(store);
        assert_eq!(sigma(&open(vfs, &path), &path), after, "retry is durable");
    }
    let _ = std::fs::remove_dir_all(work);
}

fn level_test(name: &str, level: Level, seed: u64) {
    let vfs = FullVfs::register(name);
    runner(seed, CASES)
        .run(&cases(), |c| {
            run_case(&vfs, level, &c);
            Ok(())
        })
        .unwrap();
}

#[test]
fn full_before_the_batch() {
    level_test("before", Level::Before, 0x5702_0001);
}

#[test]
fn full_inside_the_batch() {
    level_test("inside", Level::Inside, 0x5702_0002);
}

#[test]
fn full_at_commit() {
    level_test("commit", Level::AtCommit, 0x5702_0003);
}

#[test]
fn full_during_advance_watermark() {
    level_test("mark", Level::Watermark, 0x5702_0004);
}

/// US-2 case 5, RG-ST02-4.1: full on an empty location -> `StorageFull`, no `D`; a later open creates.
#[test]
fn full_at_first_opening() {
    let vfs = FullVfs::register("first");
    let path = dir().join("store.db");
    vfs.set_budget(0);
    let err = SqliteStore::open(&path, &vfs.options()).err();
    assert_eq!(err, Some(StoreError::StorageFull));
    assert!(vfs.refused() >= 1);
    if path.exists() {
        let raw = Connection::open(&path).unwrap();
        let tables: i64 = raw
            .query_row("SELECT count(*) FROM sqlite_master", [], |r| r.get(0))
            .unwrap();
        assert_eq!(tables, 0, "no `device` row, no layout");
    }
    vfs.unlimited();
    let s = open(&vfs, &path);
    assert_ne!(block_on(s.device()).unwrap(), peer(0));
    assert_eq!(block_on(s.writes_since(Position(0))).unwrap(), vec![]);
}

/// US-2 case 6: full storage, existing store -> open and read give the data.
#[test]
fn full_existing_store_reads() {
    let vfs = FullVfs::register("reads");
    let path = dir().join("store.db");
    let s = build(&vfs, &path, &[3, 5], 40);
    let expected = sigma(&s, &path);
    drop(s);
    vfs.set_budget(0);
    let s = open(&vfs, &path);
    assert_eq!(sigma(&s, &path), expected);
    let r = RecordId {
        module: "m".into(),
        category: "g".into(),
        id: "r0".into(),
    };
    assert!(block_on(s.state(&r)).unwrap().is_some());
}

/// RG-ST02-1.3, DA F-10: `xSync` failure -> `Unavailable(Io)`; live Σ unchanged; after reopen
/// Σ is `before` or `before ⊕ b`.
#[test]
fn sync_failure() {
    let vfs = FullVfs::register("sync");
    runner(0x5702_0005, 30)
        .run(&cases(), |c| {
            let work = dir();
            let path = work.join("store.db");
            let batch = changes(200, c.k, c.len);
            let store = build(&vfs, &path, &c.prefix, c.len);
            let before = sigma(&store, &path);
            vfs.fail_sync(true);
            let r = block_on(store.apply_local(1000, &batch));
            vfs.fail_sync(false);
            assert!(
                matches!(
                    r,
                    Err(ApplyError::Store(StoreError::Unavailable(Cause::Io(_))))
                ),
                "{r:?}"
            );
            assert_eq!(sigma(&store, &path), before, "live handle");
            drop(store);
            let after = sigma(&open(&vfs, &path), &path);
            let n0 = before.writes.len();
            assert_eq!(after.writes[..n0], before.writes[..]);
            if after.writes.len() == n0 {
                assert_eq!(after, before);
            } else {
                assert_eq!(after.writes.len(), n0 + c.k);
                for (i, (_, w)) in after.writes[n0..].iter().enumerate() {
                    assert_eq!((&w.record, &w.change), (&batch[i].record, &batch[i].change));
                }
            }
            let _ = std::fs::remove_dir_all(work);
            Ok(())
        })
        .unwrap();
}
