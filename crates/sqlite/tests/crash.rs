//! Forced stop (E-ST02-1) and concurrent first opening (US-4 cases 3, 4); plan T-10.
//! Replay a failure with `ARBITRE_SEED=<printed seed>`.

#[path = "support/child.rs"]
mod child;
#[path = "support/common.rs"]
mod common;

use std::collections::BTreeMap;
use std::path::Path;
use std::time::{Duration, Instant};

use arbitre_core::{Author, Change, DeviceId, LocalChange, Log, Position, Store, ready};
use child::Child;
use common::{Rng, fresh_dir, open, raw_k, rec, seed};
use rusqlite::Connection;

const T: Duration = Duration::from_secs(20);

#[test]
fn child_entry() {
    child::child_main();
}

/// Average duration of one 5-change batch (R-6).
fn commit_micros() -> u64 {
    let dir = fresh_dir("crash-calib");
    let store = open(&dir.join("store.db"));
    let changes: Vec<LocalChange> = (0..5)
        .map(|i| LocalChange {
            record: rec("c", &format!("calib{i}")),
            change: Change::field("f", b"x"),
            author: Author::System,
        })
        .collect();
    let t0 = Instant::now();
    for _ in 0..10 {
        ready(store.apply_local(0, &changes)).unwrap();
    }
    let us = t0.elapsed().as_micros() as u64 / 10;
    drop(store);
    let _ = std::fs::remove_dir_all(dir);
    us.max(200)
}

/// Kill instant: half inside `[0, 2 × commit]` after a batch started, half anywhere (R-6).
fn kill_after_delay(c: &mut Child, rng: &mut Rng, commit_us: u64) {
    if rng.chance(50) {
        for _ in 0..=rng.below(3) {
            c.expect("start", T);
        }
        std::thread::sleep(Duration::from_micros(rng.below(2 * commit_us)));
    } else {
        std::thread::sleep(Duration::from_micros(rng.below(80_000)));
    }
    c.kill();
}

/// Checks the reopened store against what the children reported (US-1 cases 1-5, 7, 12).
fn verify(path: &Path, children: &[(u64, Vec<String>)]) {
    let store = open(path);
    let stored = ready(store.writes_since(Position(0))).unwrap();
    let mut groups: BTreeMap<(u64, u64), Vec<_>> = BTreeMap::new();
    for (_, w) in &stored {
        let mut p = w.record.id.split('.');
        let key = (
            p.next().unwrap().parse().unwrap(),
            p.next().unwrap().parse().unwrap(),
        );
        groups.entry(key).or_default().push(w.clone());
    }
    for (id, log) in children {
        let mut started = BTreeMap::new();
        let mut confirmed = BTreeMap::new();
        for l in log {
            assert!(
                !l.starts_with("failed") && !l.starts_with("open_failed"),
                "{l}"
            );
            if let Some(r) = l.strip_prefix("start ") {
                let (b, n) = r.split_once(' ').unwrap();
                started.insert(b.parse::<u64>().unwrap(), n.parse::<usize>().unwrap());
            } else if let Some(r) = l.strip_prefix("confirmed ") {
                let (b, d) = r.split_once(' ').unwrap();
                confirmed.insert(b.parse::<u64>().unwrap(), d.to_string());
            }
        }
        let unconfirmed: Vec<_> = started
            .keys()
            .filter(|b| !confirmed.contains_key(b))
            .collect();
        assert!(unconfirmed.len() <= 1, "child {id}: {unconfirmed:?}");
        for (b, dbg) in &confirmed {
            let got = groups.get(&(*id, *b)).cloned().unwrap_or_default();
            assert_eq!(
                &format!("{got:?}"),
                dbg,
                "child {id} batch {b}: confirmed write lost or altered"
            );
        }
        for b in unconfirmed {
            let n = groups.get(&(*id, *b)).map_or(0, Vec::len);
            assert!(
                n == 0 || n == started[b],
                "child {id} batch {b}: torn ({n} of {})",
                started[b]
            );
        }
        for (_, b) in groups.keys().filter(|(i, _)| i == id) {
            assert!(
                started.contains_key(b),
                "child {id}: stored batch {b} never started"
            );
        }
    }
    // R(W) = reference log fed with every stored write.
    let mut reference = Log::new(ready(store.device()).unwrap());
    let all: Vec<_> = stored.iter().map(|(_, w)| w.clone()).collect();
    reference.merge(&all).expect("stored writes merge");
    for (r, state) in reference.projection() {
        assert_eq!(ready(store.state(&r)).unwrap(), Some(state), "{r:?}");
    }
    // K covers every stored stamp and the next local stamp is above all of them.
    let top = all.iter().map(|w| (w.stamp.l, w.stamp.c)).max();
    let k = raw_k(&Connection::open(path).unwrap());
    if let Some((l, c)) = top {
        assert!(
            k >= (l as i64, i64::from(c)),
            "K {k:?} below stored stamp {top:?}"
        );
        let next = ready(store.apply_local(
            0,
            &[LocalChange {
                record: rec("c", "next"),
                change: Change::field("f", b"n"),
                author: Author::System,
            }],
        ))
        .unwrap();
        assert!(
            (next[0].stamp.l, next[0].stamp.c) > (l, c),
            "next stamp {:?} not above {top:?}",
            next[0].stamp
        );
    }
}

fn writer(path: &Path, id: u64, seed: u64) -> Child {
    Child::spawn(
        "writer",
        path,
        &[
            ("ARBITRE_CHILD_ID", id.to_string()),
            ("ARBITRE_CHILD_SEED", seed.to_string()),
        ],
    )
}

#[test]
fn single_writer_killed_200_runs() {
    let seed = seed();
    println!("seed={seed}");
    let commit_us = commit_micros();
    let mut rng = Rng::new(seed);
    for run in 0..200 {
        let path = fresh_dir("crash1").join("store.db");
        let mut c = writer(&path, 1, rng.next());
        kill_after_delay(&mut c, &mut rng, commit_us);
        let log = c.finish();
        println!("run={run}");
        verify(&path, &[(1, log)]);
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }
}

#[test]
fn two_writers_one_killed_50_runs() {
    let seed = seed();
    println!("seed={seed}");
    let commit_us = commit_micros();
    let mut rng = Rng::new(seed ^ 2);
    for run in 0..50 {
        let path = fresh_dir("crash2").join("store.db");
        let mut a = writer(&path, 1, rng.next());
        let mut b = writer(&path, 2, rng.next());
        kill_after_delay(&mut a, &mut rng, commit_us);
        let la = a.finish();
        // The survivor keeps committing after the kill.
        for _ in 0..2 {
            b.expect("confirmed", T);
        }
        b.close();
        let lb = b.finish();
        println!("run={run}");
        verify(&path, &[(1, la), (2, lb)]);
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }
}

/// US-4 case 3: two processes released by one "go" on an empty path agree on `D`.
#[test]
fn concurrent_first_opening_50_runs() {
    let seed = seed();
    println!("seed={seed}");
    for run in 0..50 {
        let path = fresh_dir("crash-create").join("store.db");
        let mut kids: Vec<Child> = (0..2).map(|_| Child::spawn("open", &path, &[])).collect();
        for k in &mut kids {
            k.expect("ready", T);
        }
        for k in &mut kids {
            k.send("go");
        }
        let ds: Vec<String> = kids
            .iter_mut()
            .map(|k| {
                let l = k.expect("open", T);
                assert!(l.starts_with("opened "), "seed={seed} run={run}: {l}");
                l["opened ".len()..].to_string()
            })
            .collect();
        assert_eq!(ds[0], ds[1], "seed={seed} run={run}: two different D");
        drop(kids);
        let d = ready(open(&path).device()).unwrap();
        assert_eq!(d.hex(), ds[0], "seed={seed} run={run}");
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }
}

/// US-4 case 4: a forced stop during the first opening leaves a creatable or complete store.
#[test]
fn kill_during_first_opening_50_runs() {
    let seed = seed();
    println!("seed={seed}");
    let mut rng = Rng::new(seed ^ 4);
    for run in 0..50 {
        let path = fresh_dir("crash-open").join("store.db");
        let mut c = Child::spawn("open", &path, &[]);
        c.expect("ready", T);
        c.send("go");
        std::thread::sleep(Duration::from_micros(rng.below(30_000)));
        c.kill();
        c.finish();
        let store = open(&path);
        let ctx = format!("seed={seed} run={run}");
        assert_ne!(ready(store.device()).unwrap(), DeviceId([0; 16]), "{ctx}");
        assert_eq!(store.settings().journal_mode, "wal", "{ctx}");
        let d = ready(store.device()).unwrap();
        drop(store);
        assert_eq!(ready(open(&path).device()).unwrap(), d, "{ctx}");
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }
}
