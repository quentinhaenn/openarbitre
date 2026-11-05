//! Damage and incompatible versions (E-ST02-4); plan T-13. Replay with `ARBITRE_SEED=<seed>`.

#[path = "support/child.rs"]
mod child;

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use arbitre_core::{
    Author, Cause, Damage, DeviceId, LocalChange, Position, Seen, Stamp, Store, StoreError,
    TargetKind, Watermark, Write,
};
use arbitre_sqlite::{Options, SqliteStore};
use child::{Child, Rng, block_on, field_change, fresh_dir, rec, seed};
use rusqlite::Connection;

const T: Duration = Duration::from_secs(20);

#[test]
fn child_entry() {
    child::child_main();
}

fn open(path: &Path) -> Result<SqliteStore, StoreError> {
    SqliteStore::open(path, &Options::default())
}

fn open_err(path: &Path) -> StoreError {
    open(path).err().expect("open must be refused")
}

fn lc(id: &str, value: &str) -> LocalChange {
    LocalChange {
        record: rec("c", id),
        change: field_change("f", value.as_bytes()),
        author: Author::System,
    }
}

/// Store at `<dir>/store.db`: records `a` (3 writes, pos 1-3; pos 2-3 carry `seen`) and `b`
/// (pos 4), one watermark; handle dropped, WAL checkpointed. No later open: all writes are
/// past the checked marker (incremental open check).
fn build(tag: &str) -> (PathBuf, PathBuf) {
    let dir = fresh_dir(tag);
    let path = dir.join("store.db");
    let store = open(&path).unwrap();
    for (id, v) in [("a", "1"), ("a", "2"), ("a", "3"), ("b", "1")] {
        block_on(store.apply_local(1000, &[lc(id, v)])).unwrap();
    }
    // A watermark so that every page of the file holds data (truncation case).
    block_on(store.advance_watermark(DeviceId::from_u128(9), Watermark(1, 0))).unwrap();
    drop(store);
    (dir, path)
}

/// `D` read from outside, so that `build`'s writes stay past the checked marker.
fn device_of(path: &Path) -> DeviceId {
    let c = Connection::open(path).unwrap();
    DeviceId(
        c.query_row("SELECT device FROM device", [], |r| r.get(0))
            .unwrap(),
    )
}

/// Damage `name` (SQL and expected cause) for the store at `path`.
fn case(path: &Path, name: &str) -> (&'static str, String, Damage) {
    damages(device_of(path))
        .into_iter()
        .find(|c| c.0 == name)
        .unwrap()
}

fn raw(path: &Path, sql: &str) {
    let conn = Connection::open(path).unwrap();
    conn.execute_batch(&format!("PRAGMA ignore_check_constraints = ON; {sql}"))
        .unwrap();
}

fn pos(n: u64) -> Damage {
    Damage::StoredWrite {
        position: Some(Position(n)),
    }
}

/// Damaged files: name, SQL, expected `Corrupted` cause (all on `build`'s shape).
fn damages(d: DeviceId) -> Vec<(&'static str, String, Damage)> {
    let a = "x'00000000000000000000000000000000'";
    let copy = "INSERT INTO writes (module, category, id, field, value, l, c, a, author)
                SELECT module, category, id, field, x'7a', l, c, a, author FROM writes WHERE pos = 1;";
    let pair = Damage::LocalIdentityPair {
        target: Box::new(arbitre_core::Target {
            record: rec("c", "a"),
            kind: TargetKind::Field(arbitre_core::Field("f".into())),
        }),
        stamp: Stamp {
            l: 1000,
            c: 0,
            a: d,
        },
    };
    vec![
        (
            "non-utf8 field",
            "UPDATE writes SET field = x'ff' WHERE pos = 2;".into(),
            pos(2),
        ),
        (
            "seen >= stamp",
            "UPDATE seen SET l = l + 1000000 WHERE pos = 2;".into(),
            pos(2),
        ),
        (
            "seen kind 2",
            "UPDATE seen SET kind = 2 WHERE pos = 2;".into(),
            pos(2),
        ),
        (
            "seen kind 0 with field",
            "UPDATE seen SET kind = 0 WHERE pos = 3;".into(),
            pos(3),
        ),
        (
            "orphan seen",
            format!("INSERT INTO seen VALUES (5, 1, x'66', 1, 0, {a});"),
            pos(5),
        ),
        ("local identity pair", copy.into(), pair),
        (
            "K below stamp",
            "UPDATE device SET clock_l = 5;".into(),
            Damage::ClockState,
        ),
        (
            "missing device",
            "DELETE FROM device;".into(),
            Damage::DeviceId,
        ),
        (
            "foreign application_id",
            "PRAGMA application_id = 5;".into(),
            Damage::NotAStore,
        ),
        (
            "user_version 0",
            "PRAGMA user_version = 0;".into(),
            Damage::Layout,
        ),
    ]
}

fn expect_corrupted(e: StoreError, want: &Damage, what: &str) {
    match e {
        StoreError::Corrupted(d) => assert_eq!(&d, want, "{what}"),
        other => panic!("{what}: expected Corrupted, got {other:?}"),
    }
}

#[test]
fn damage_at_rest_is_refused_at_open() {
    let names: Vec<_> = damages(DeviceId::from_u128(0))
        .into_iter()
        .map(|c| c.0)
        .collect();
    for name in names {
        let (d2, path) = build("dmg-rest-case");
        let (_, sql, want) = case(&path, name);
        raw(&path, &sql);
        let before = std::fs::read(&path).unwrap();
        expect_corrupted(open_err(&path), &want, name);
        assert_eq!(
            std::fs::read(&path).unwrap(),
            before,
            "{name}: file changed"
        );
        let _ = std::fs::remove_dir_all(d2);
    }
}

#[test]
fn truncation_is_refused() {
    let (dir, path) = build("dmg-trunc");
    let full = std::fs::read(&path).unwrap();
    let mut rng = Rng::new(seed());
    println!("seed {}", seed());
    for _ in 0..20 {
        // Any page, the last one included (Q-P6: `quick_check` at open); a cut that only drops
        // zero bytes loses nothing.
        let len = 1 + rng.below(full.len() as u64 - 1) as usize;
        if full[len..].iter().all(|&b| b == 0) {
            continue;
        }
        let (d2, p) = build("dmg-trunc-case");
        std::fs::OpenOptions::new()
            .write(true)
            .open(&p)
            .unwrap()
            .set_len(len as u64)
            .unwrap();
        let before = std::fs::read(&p).unwrap();
        match open(&p) {
            Err(StoreError::Corrupted(_)) => {}
            Err(other) => panic!("len {len}: {other:?}"),
            Ok(_) => panic!("len {len}: truncated store opened"),
        }
        assert_eq!(
            std::fs::read(&p).unwrap(),
            before,
            "len {len}: file changed"
        );
        let _ = std::fs::remove_dir_all(d2);
    }
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn not_a_store_and_directory() {
    let dir = fresh_dir("dmg-text");
    let text = dir.join("text.db");
    std::fs::write(
        &text,
        "this is not a database, just a long enough line of text".repeat(30),
    )
    .unwrap();
    let before = std::fs::read(&text).unwrap();
    expect_corrupted(open_err(&text), &Damage::NotAStore, "text file");
    assert_eq!(std::fs::read(&text).unwrap(), before);
    let sub = dir.join("sub.db");
    std::fs::create_dir(&sub).unwrap();
    assert!(matches!(
        open_err(&sub),
        StoreError::Unavailable(Cause::Io(_))
    ));
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn newer_version_is_unavailable_and_untouched() {
    let (dir, path) = build("dmg-version");
    raw(&path, "PRAGMA user_version = 2;");
    let before = std::fs::read(&path).unwrap();
    assert!(matches!(
        open_err(&path),
        StoreError::Unavailable(Cause::IncompatibleVersion {
            stored: 2,
            supported: 1
        })
    ));
    assert_eq!(std::fs::read(&path).unwrap(), before);
    let _ = std::fs::remove_dir_all(dir);
}

type Op = fn(&SqliteStore) -> Result<(), StoreError>;

fn ops() -> Vec<(&'static str, Op)> {
    vec![
        ("writes_of", |s| {
            block_on(s.writes_of(&rec("c", "a"))).map(drop)
        }),
        ("state", |s| block_on(s.state(&rec("c", "a"))).map(drop)),
        ("records", |s| block_on(s.records("m", "c")).map(drop)),
        ("writes_since", |s| {
            block_on(s.writes_since(Position(0))).map(drop)
        }),
        ("apply", |s| {
            let w = Write {
                record: rec("c", "a"),
                change: field_change("g", b"x"),
                stamp: Stamp {
                    l: 2000,
                    c: 0,
                    a: DeviceId::from_u128(9),
                },
                seen: Seen::new(),
                author: Author::System,
            };
            block_on(s.apply(&[w])).map(drop).map_err(store_err)
        }),
        ("apply_local", |s| {
            block_on(s.apply_local(1000, &[lc("a", "z")]))
                .map(drop)
                .map_err(store_err)
        }),
        ("advance_watermark", |s| {
            block_on(s.advance_watermark(DeviceId::from_u128(9), Watermark(1, 0)))
        }),
    ]
}

fn store_err(e: arbitre_core::ApplyError) -> StoreError {
    match e {
        arbitre_core::ApplyError::Store(e) => e,
        other => panic!("unexpected {other:?}"),
    }
}

/// Operations that must find the damage in record `a`; the others succeed (DA F-13).
fn finds(case: &str, op: &str) -> bool {
    match op {
        "advance_watermark" => false,
        // Archi exception: `writes_since` returns both rows of a local identity pair.
        "writes_since" => case != "local identity pair",
        _ => true,
    }
}

#[test]
fn damage_after_open_is_found_by_the_operation() {
    for name in [
        "non-utf8 field",
        "seen >= stamp",
        "seen kind 2",
        "seen kind 0 with field",
        "local identity pair",
    ] {
        for (op, run) in ops() {
            // One fresh store and handle per operation (DA F-5).
            let (d2, path) = build("dmg-live");
            let (_, sql, want) = case(&path, name);
            let (sql, want) = (&sql, &want);
            let store = open(&path).unwrap();
            raw(&path, sql);
            let got = run(&store);
            let tag = format!("{name} / {op}");
            if finds(name, op) {
                expect_corrupted(got.expect_err(&tag), want, &tag);
                // Poison: the handle keeps answering the same error, also for record `b`.
                let again = block_on(store.writes_of(&rec("c", "b"))).expect_err(&tag);
                expect_corrupted(again, want, &format!("{tag} (poison)"));
                let dev = block_on(store.device()).expect_err(&tag);
                expect_corrupted(dev, want, &format!("{tag} (poison, device)"));
                let empty = block_on(store.apply_local(1000, &[])).map_err(store_err);
                expect_corrupted(
                    empty.expect_err(&tag),
                    want,
                    &format!("{tag} (poison, empty apply_local)"),
                );
            } else {
                got.unwrap_or_else(|e| panic!("{tag}: {e:?}"));
            }
            drop(store);
            // Rows checked by the first open: only `check_all` still sees them. Exceptions: the
            // pair row is a new write past the marker; CHECK violations are caught at open by
            // `quick_check`, as engine damage.
            let tag = format!("{tag} (reopen)");
            if name == "local identity pair" {
                expect_corrupted(open_err(&path), want, &tag);
            } else if name.starts_with("seen kind") {
                assert!(
                    matches!(open_err(&path), StoreError::Corrupted(Damage::Engine(_))),
                    "{tag}"
                );
            } else {
                let again = open(&path).unwrap().check_all();
                expect_corrupted(again.expect_err(&tag), want, &tag);
            }
            let _ = std::fs::remove_dir_all(d2);
        }
    }
}

#[test]
fn other_records_survive_a_damaged_one() {
    let (dir, path) = build("dmg-other");
    let store = open(&path).unwrap();
    raw(&path, "UPDATE writes SET field = x'ff' WHERE pos = 2;");
    assert_eq!(block_on(store.writes_of(&rec("c", "b"))).unwrap().len(), 1);
    block_on(store.apply_local(1000, &[lc("b", "q")])).unwrap();
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn colliding_orphan_seen_rolls_back_a_received_write() {
    let (dir, path) = build("dmg-p8");
    let store = open(&path).unwrap();
    raw(
        &path,
        "INSERT INTO seen VALUES (5, 0, x'', 1, 0, x'00000000000000000000000000000000');",
    );
    let foreign = DeviceId::from_u128(9);
    let w = Write {
        record: rec("c", "z"),
        change: field_change("f", b"x"),
        stamp: Stamp {
            l: 2000,
            c: 0,
            a: foreign,
        },
        seen: Seen::from([(
            TargetKind::Delete,
            Stamp {
                l: 1999,
                c: 0,
                a: foreign,
            },
        )]),
        author: Author::System,
    };
    match block_on(store.apply(&[w])) {
        Err(arbitre_core::ApplyError::Store(StoreError::Corrupted(Damage::Engine(_)))) => {}
        other => panic!("expected Corrupted(Engine), got {other:?}"),
    }
    drop(store);
    let again = open(&path);
    assert!(again.is_err(), "orphan must be refused at reopen");
    expect_corrupted(again.err().unwrap(), &pos(5), "reopen");
    let _ = std::fs::remove_dir_all(dir);
}

/// Incremental open check: damage to rows checked by an earlier open is found by `check_all`
/// (not by `open`); damage to writes added since is refused at open.
#[test]
fn checked_rows_are_skipped_at_open_but_not_by_check_all() {
    let (dir, path) = build("dmg-incr");
    open(&path).unwrap().check_all().unwrap();
    let marker = |p: &Path| -> i64 {
        let c = Connection::open(p).unwrap();
        c.query_row("SELECT checked FROM device", [], |r| r.get(0))
            .unwrap()
    };
    assert_eq!(marker(&path), 4);
    raw(&path, "UPDATE writes SET field = x'ff' WHERE pos = 2;");
    let store = open(&path).expect("checked rows are not re-read at open");
    expect_corrupted(store.check_all().unwrap_err(), &pos(2), "check_all");
    // A new write after the marker is checked at the next open; the marker moves only on success.
    let (dir2, path2) = build("dmg-incr-new");
    open(&path2).unwrap();
    block_on(open(&path2).unwrap().apply_local(1000, &[lc("a", "4")])).unwrap();
    raw(&path2, "UPDATE writes SET field = x'ff' WHERE pos = 5;");
    let before = std::fs::read(&path2).unwrap();
    expect_corrupted(open_err(&path2), &pos(5), "new write");
    assert_eq!(std::fs::read(&path2).unwrap(), before);
    assert_eq!(marker(&path2), 4);
    for d in [dir, dir2] {
        let _ = std::fs::remove_dir_all(d);
    }
}

/// Q-P5: a fresh process's `open` returns the `Corrupted` that a read found.
#[test]
fn child_open_matches_the_read_that_found_the_damage() {
    for (name, sql) in [
        (
            "non-utf8 field",
            "UPDATE writes SET field = x'ff' WHERE pos = 5;",
        ),
        (
            "seen >= stamp",
            "UPDATE seen SET l = l + 1000000 WHERE pos = 5;",
        ),
        (
            "local identity pair",
            "INSERT INTO writes (module, category, id, field, value, l, c, a, author)
             SELECT module, category, id, field, x'7a', l, c, a, author FROM writes WHERE pos = 1;",
        ),
    ] {
        let (dir, path) = build("dmg-child");
        let store = open(&path).unwrap();
        // pos 5 is past the marker set by the open above.
        block_on(store.apply_local(1000, &[lc("a", "4")])).unwrap();
        raw(&path, sql);
        let found = block_on(store.writes_of(&rec("c", "a"))).expect_err(name);
        assert!(
            matches!(found, StoreError::Corrupted(_)),
            "{name}: {found:?}"
        );
        drop(store);
        let mut c = Child::spawn("open", &path, &[]);
        c.expect("ready", T);
        c.send("go");
        let line = c.expect("open", T);
        assert_eq!(line, format!("open_failed {found:?}"), "{name}");
        c.close();
        let _ = std::fs::remove_dir_all(dir);
    }
}

fn median_ms(mut f: impl FnMut()) -> f64 {
    let mut v: Vec<f64> = (0..5)
        .map(|_| {
            let t = Instant::now();
            f();
            t.elapsed().as_secs_f64() * 1000.0
        })
        .collect();
    v.sort_by(f64::total_cmp);
    v[2]
}

/// Timing probe for T-15 (asserts nothing), 10,000 writes: incremental open (second open, marker
/// at the top), `check_all`, and `quick_check` alone (Q-P6). Open times are net of a one-write store.
#[test]
fn open_check_timing() {
    let dir = fresh_dir("dmg-timing");
    let (big, small) = (dir.join("big.db"), dir.join("small.db"));
    let store = open(&big).unwrap();
    for b in 0..100 {
        let changes: Vec<LocalChange> = (0..100).map(|i| lc("r", &format!("{b}.{i}"))).collect();
        block_on(store.apply_local(1000, &changes)).unwrap();
    }
    drop(store);
    block_on(open(&small).unwrap().apply_local(1000, &[lc("r", "0")])).unwrap();
    let t = Instant::now();
    drop(open(&big).unwrap()); // first open after the writes: checks all 10,000, sets the marker
    let first_ms = t.elapsed().as_secs_f64() * 1000.0;
    let small_ms = median_ms(|| drop(open(&small).unwrap()));
    let incremental = median_ms(|| drop(open(&big).unwrap())) - small_ms;
    let handle = open(&big).unwrap();
    let full = median_ms(|| handle.check_all().unwrap());
    let quick = median_ms(|| {
        let c = Connection::open(&big).unwrap();
        let r: String = c.query_row("PRAGMA quick_check", [], |r| r.get(0)).unwrap();
        assert_eq!(r, "ok");
    });
    let line = format!(
        "open-check-ms {incremental:.1}; first-open-ms {first_ms:.1}; check-all-ms {full:.1}; quick_check-ms {quick:.1}"
    );
    println!("{line}");
    if let Ok(out) = std::env::var("CI_OUT") {
        std::fs::write(
            Path::new(&out).join("open-check-ms.txt"),
            format!("{line}\n"),
        )
        .unwrap();
    }
    let _ = std::fs::remove_dir_all(dir);
}
