//! Interleaved processes (E-ST02-3; US-3 cases 1-10, RG-ST02-3.1 to 3.5); plan T-11.
//! Replay a failure with `ARBITRE_SEED=<printed seed>`.

#[path = "support/child.rs"]
mod child;
#[path = "support/common.rs"]
mod common;

use std::collections::HashSet;
use std::path::Path;
use std::time::{Duration, Instant};

use arbitre_core::{Author, Change, Log, Position, Store, ready};
use child::Child;
use common::{Rng, fresh_dir, open, rec, seed};

const T: Duration = Duration::from_secs(20);
/// Busy wait of the writers (δ) and the upper bound on an observed wait (ε).
const DELTA: Duration = Duration::from_millis(200);
const EPSILON: Duration = Duration::from_secs(1);

#[test]
fn child_entry() {
    child::child_main();
}

/// One schedule: two writers and a reader driven step by step; `pt` drawn from a small domain
/// (equal values, rewinds) so a cached `K` would repeat or rewind stamps.
fn schedule(path: &Path, rng: &mut Rng, tag: u64) {
    let main = open(path);
    let env = [("ARBITRE_CHILD_WAIT_MS", DELTA.as_millis().to_string())];
    let mut w: Vec<Child> = (0..2).map(|_| Child::spawn("steps", path, &env)).collect();
    let mut reader = Child::spawn("steps", path, &env);
    for c in w.iter_mut().chain([&mut reader]) {
        c.expect("opened", T);
    }
    let mut reference = Log::new(ready(main.device()).unwrap());
    for op in 0..2 + rng.below(19) {
        let who = rng.below(2) as usize;
        let pt = [0, 1, 1, 2, 3, 2][rng.below(6) as usize];
        let mut spec = Vec::new();
        let mut changes = Vec::new();
        for i in 0..1 + rng.below(2) {
            let id = ["r1", "r2"][rng.below(2) as usize];
            if rng.chance(15) {
                spec.push(format!("{id}=-"));
                changes.push((id, Change::Delete));
            } else {
                let f = ["f", "g"][rng.below(2) as usize];
                let v = format!("v{tag}.{op}.{i}");
                spec.push(format!("{id}={f}={v}"));
                changes.push((id, Change::field(f, v.as_bytes())));
            }
        }
        // Held 2δ: the other writer must time out; held δ/2: it must wait and succeed.
        let held = if rng.chance(8) {
            let ms = if rng.chance(50) { 2 * DELTA } else { DELTA / 2 };
            w[1 - who].send(&format!("hold {}", ms.as_millis()));
            w[1 - who].expect("held", T);
            Some(ms)
        } else {
            None
        };
        let t0 = Instant::now();
        w[who].send(&format!("local {pt} {}", spec.join(";")));
        let reply = w[who].next(T).expect("reply");
        let waited = t0.elapsed();
        if held == Some(2 * DELTA) {
            assert!(
                reply.starts_with("err ") && reply.contains("Busy"),
                "{reply}"
            );
            assert!(
                waited >= DELTA - Duration::from_millis(10) && waited <= EPSILON + DELTA,
                "{waited:?}"
            );
        } else {
            let expected: Vec<_> = changes
                .iter()
                .map(|(id, c)| {
                    reference
                        .write(pt, rec("c", id), c.clone(), Author::CliUser)
                        .unwrap()
                })
                .collect();
            assert_eq!(reply, format!("ok {expected:?}"), "op {op} by {who}");
            assert!(waited <= EPSILON + DELTA, "{waited:?}");
        }
        if held.is_some() {
            w[1 - who].expect("released", T);
        }
        reader.send("since");
        let stored = ready(main.writes_since(Position(0))).unwrap();
        assert_eq!(
            reader.expect("since", T),
            format!("since {stored:?}"),
            "op {op}"
        );
    }
    // Commit order = stamp order, strictly; the store equals the sequential application.
    let stored = ready(main.writes_since(Position(0))).unwrap();
    let stamps: Vec<_> = stored.iter().map(|(_, w)| (w.stamp.l, w.stamp.c)).collect();
    assert!(stamps.windows(2).all(|p| p[0] < p[1]), "stamps {stamps:?}");
    let mut got: Vec<_> = stored.into_iter().map(|(_, w)| w).collect();
    let mut want = reference.writes();
    got.sort();
    want.sort();
    assert_eq!(got, want);
    assert!(reference.conflicts().is_empty() && reference.anomalies().is_empty());
    for (r, state) in reference.projection() {
        assert_eq!(ready(main.state(&r)).unwrap(), Some(state));
    }
}

#[test]
fn stepped_100_schedules() {
    let seed = seed();
    println!("seed={seed}");
    for s in 0..100 {
        println!("schedule={s}");
        let dir = fresh_dir("steps");
        schedule(&dir.join("store.db"), &mut Rng::new(seed ^ (s << 20)), s);
        let _ = std::fs::remove_dir_all(dir);
    }
}

/// RG-ST02-3.5 / US-3 case 9: a reader looping against a committing writer never sees a torn
/// batch nor a write without its `seen` entries.
#[test]
fn free_running_10_seeds_2s() {
    let seed = seed();
    println!("seed={seed}");
    let mut rng = Rng::new(seed ^ 9);
    for run in 0..10 {
        let dir = fresh_dir("free");
        let path = dir.join("store.db");
        drop(open(&path));
        let k = 5 + rng.below(16);
        let env = [("ARBITRE_CHILD_K", k.to_string())];
        let mut writer = Child::spawn("free_writer", &path, &env);
        let mut reader = Child::spawn("free_reader", &path, &env);
        writer.expect("opened", T);
        reader.expect("opened", T);
        std::thread::sleep(Duration::from_secs(2));
        writer.close();
        reader.close();
        let (wl, rl) = (writer.finish(), reader.finish());
        let ctx = format!("seed={seed} run={run} k={k}");
        assert!(!wl.iter().any(|l| l.starts_with("failed")), "{ctx}: {wl:?}");
        let reported: HashSet<_> = wl.iter().filter(|l| l.starts_with("w ")).collect();
        let seen: Vec<_> = rl.iter().filter(|l| l.starts_with("w ")).collect();
        assert!(
            reported.len() > k as usize,
            "{ctx}: writer committed too little"
        );
        assert!(!seen.is_empty(), "{ctx}: reader saw nothing");
        for l in &rl {
            assert!(!l.starts_with("bad"), "{ctx}: {l}");
        }
        for l in seen {
            assert!(
                reported.contains(l),
                "{ctx}: read a write the writer never reported: {l}"
            );
        }
        println!(
            "run={run} k={k} {} {}",
            wl.last().unwrap(),
            rl.last().unwrap()
        );
        let _ = std::fs::remove_dir_all(dir);
    }
}
