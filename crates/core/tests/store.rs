//! `store.rs` pure logic (T-4, T-5) and `SimStore` (T-7), plus the contract suite on `SimStore`.
mod support;

#[path = "support/contract.rs"]
mod contract;

use arbitre_core::*;
use contract::{Harness, ready};
use proptest::prelude::*;
use proptest::test_runner::Config as ProptestConfig;
use support::*;

/// Local device `D`; peers 3 (< D) and 7 (> D).
const D: u128 = 5;
const PEER: u128 = 7;
const LOW_PEER: u128 = 3;

fn k(l: u64, c: u32) -> ClockState {
    ClockState { l, c }
}

fn local(id: &str, change: Change) -> LocalChange {
    LocalChange {
        record: record(id),
        change,
        author: Author::CliUser,
    }
}

fn fresh_log(kk: ClockState) -> Log {
    rebuild(dev(D), kk, &[]).expect("empty store rebuilds")
}

// ---------------------------------------------------------------- T-4: restored clock

#[derive(Debug, Clone)]
enum ClockOp {
    Issue(u64),
    Receive(u64, u32),
}

fn clock_op() -> impl Strategy<Value = ClockOp> {
    prop_oneof![
        (0u64..20).prop_map(ClockOp::Issue),
        (0u64..20, 0u32..5).prop_map(|(l, c)| ClockOp::Receive(l, c)),
    ]
}

proptest! {
    #![proptest_config(config())]

    /// GAP-ST02-1, PROP-ST02-5: a clock restored from `(l, c)` issues like the original.
    #[test]
    fn restored_clock_issues_like_the_original(
        ops in prop::collection::vec(clock_op(), 0..30),
        pt in 0u64..25,
    ) {
        let mut original = HybridClock::new(dev(D));
        let mut kk = k(0, 0);
        for op in &ops {
            match *op {
                ClockOp::Issue(pt) => {
                    if let Ok(s) = original.issue(pt) {
                        kk = k(s.l, s.c);
                    }
                }
                ClockOp::Receive(l, c) => {
                    original.receive(&stamp(l, c, PEER));
                    kk = kk.max(k(l, c));
                }
            }
        }
        let mut restored = rebuild(dev(D), kk, &[]).unwrap();
        let issued = restored.write(pt, record("r"), field("f", "v"), Author::CliUser);
        prop_assert_eq!(
            issued.map(|w| w.stamp),
            original.issue(pt).map_err(WriteError::from)
        );
    }
}

// ---------------------------------------------------------------- T-4: stamp_local

#[test]
fn stamp_local_stamps_in_list_order_from_k() {
    let mut log = fresh_log(k(100, 2));
    let changes = [
        local("r", field("f", "a")),
        local("r", field("f", "b")),
        local("s", Change::Delete),
    ];
    let (writes, kk) = stamp_local(&mut log, 50, &changes).unwrap();
    assert_eq!(writes.len(), 3);
    assert_eq!(writes[0].stamp, stamp(100, 3, D));
    assert!(writes.windows(2).all(|p| p[0].stamp < p[1].stamp));
    assert!(writes.iter().all(|w| w.stamp.a == dev(D)), "US-4 case 6");
    let last = writes[2].stamp;
    assert_eq!(kk, k(last.l, last.c));
    for (w, ch) in writes.iter().zip(&changes) {
        assert_eq!(
            (&w.record, &w.change, w.author),
            (&ch.record, &ch.change, ch.author)
        );
        assert!(log.writes().contains(w));
    }
}

/// A-1, RG-ST02-3.1: `seen` of the i-th change covers the earlier changes of the list.
#[test]
fn stamp_local_seen_covers_earlier_changes_of_the_list() {
    let cases = [
        (field("f", "a"), field("f", "b"), kind("f")),
        (field("f", "a"), Change::Delete, kind("f")),
        (Change::Delete, field("f", "a"), TargetKind::Delete),
    ];
    for (first, second, seen_kind) in cases {
        let mut log = fresh_log(k(0, 0));
        let (w, _) = stamp_local(&mut log, 10, &[local("r", first), local("r", second)]).unwrap();
        assert_eq!(w[1].seen, Seen::from([(seen_kind, w[0].stamp)]));
    }
}

/// US-1 case 8.
#[test]
fn us1_case8_clock_went_back() {
    let known = [write(
        record("r"),
        field("f", "a"),
        stamp(1_000_000_000, 3, D),
    )];
    let mut log = rebuild(dev(D), k(1_000_000_000, 3), &known).unwrap();
    let (w, kk) = stamp_local(&mut log, 0, &[local("r", field("f", "b"))]).unwrap();
    assert_eq!(w[0].stamp, stamp(1_000_000_000, 4, D));
    assert_eq!(kk, k(1_000_000_000, 4));
}

/// US-3 case 10: rejected, no stamp consumed, `Log` and `K` unchanged.
#[test]
fn us3_case10_invalid_change_rejects_the_whole_list() {
    let mut log = fresh_log(k(10, 0));
    let before = log.clone();
    let changes = [
        local("r", field("f", "a")),
        local("r", field("", "b")),
        local("r", field("g", "c")),
    ];
    assert_eq!(
        stamp_local(&mut log, 20, &changes),
        Err(Rejection::Local {
            index: 1,
            error: WriteError::Invalid(Reason::EmptyIdentifier)
        })
    );
    assert_eq!(log, before);
}

#[test]
fn stamp_local_clock_exhausted_is_rejected() {
    let one = [local("r", field("f", "a"))];
    let two = [local("r", field("f", "a")), local("r", field("f", "b"))];
    let cases: [(ClockState, u64, &[LocalChange], usize); 3] = [
        (k(10, u32::MAX), 5, &one, 0),
        (k(L_MAX - 1, 0), L_MAX, &one, 0),
        (k(10, u32::MAX - 1), 5, &two, 1),
    ];
    for (kk, pt, changes, index) in cases {
        let mut log = fresh_log(kk);
        let before = log.clone();
        assert_eq!(
            stamp_local(&mut log, pt, changes),
            Err(Rejection::Local {
                index,
                error: WriteError::ClockExhausted
            })
        );
        assert_eq!(log, before);
    }
}

// ---------------------------------------------------------------- T-4: rebuild

#[test]
fn rebuild_known_write_failing_validate_is_damage() {
    let mut w = write(record("r"), field("f", "a"), stamp(10, 0, D));
    w.seen.insert(kind("f"), stamp(10, 0, PEER));
    assert_eq!(
        rebuild(dev(D), k(10, 0), &[w]).err(),
        Some(Damage::StoredWrite { position: None })
    );
}

/// Q-A (b): a local-device pair is damage; a peer pair is an anomaly (US-5 case 4).
#[test]
fn rebuild_local_identity_pair_is_damage_peer_pair_is_not() {
    let a = write(record("r"), field("f", "a"), stamp(10, 0, D));
    let b = write(record("r"), field("f", "b"), stamp(10, 0, D));
    assert_eq!(
        rebuild(dev(D), k(10, 0), &[a.clone(), b]).err(),
        Some(Damage::LocalIdentityPair {
            target: Box::new(a.target()),
            stamp: a.stamp
        })
    );
    let pa = write(record("r"), field("f", "a"), stamp(10, 0, PEER));
    let pb = write(record("r"), field("f", "b"), stamp(10, 0, PEER));
    let log = rebuild(dev(D), k(10, 0), &[pa, pb]).unwrap();
    assert_eq!(log.anomalies().len(), 1);
}

/// DA F-2: the clock check compares `(l, c)` only, never a full `Stamp`.
#[test]
fn rebuild_clock_check_on_l_c_only() {
    let at = |l, c, a| [write(record("r"), field("f", "a"), stamp(l, c, a))];
    assert!(rebuild(dev(D), k(10, 5), &at(10, 5, PEER)).is_ok());
    assert!(rebuild(dev(D), k(10, 5), &at(10, 5, LOW_PEER)).is_ok());
    assert_eq!(
        rebuild(dev(D), k(10, 5), &at(10, 6, PEER)).err(),
        Some(Damage::ClockState)
    );
    assert_eq!(
        rebuild(dev(D), k(10, u32::MAX), &at(11, 0, LOW_PEER)).err(),
        Some(Damage::ClockState)
    );
}

// ---------------------------------------------------------------- T-5: receive

/// US-1 case 9.
#[test]
fn us1_case9_received_stamp_far_ahead() {
    let kk = k(1_000_000_000, 0);
    let mut log = fresh_log(kk);
    let batch = [write(
        record("r"),
        field("f", "a"),
        stamp(1_000_000_000_000, 0, PEER),
    )];
    let r = receive(dev(D), &mut log, kk, &batch).unwrap();
    assert_eq!(r.clock, k(1_000_000_000_000, 0));
    let (w, _) = stamp_local(&mut log, 0, &[local("s", field("f", "b"))]).unwrap();
    assert!(w[0].stamp > batch[0].stamp);
}

/// US-5 case 7, RG-ST02-5.3: second delivery stores nothing; `K' = max_lex(K, max batch)`.
#[test]
fn us5_case7_same_batch_twice() {
    let batch = [
        write(record("r"), field("f", "a"), stamp(100, 5, PEER)),
        write(record("s"), Change::Delete, stamp(90, 0, PEER)),
    ];
    for kk in [k(100, 4), k(100, 5), k(100, 6)] {
        let mut log = fresh_log(kk);
        let first = receive(dev(D), &mut log, kk, &batch).unwrap();
        assert_eq!(first.new, batch.to_vec());
        assert_eq!(first.clock, kk.max(k(100, 5)));
        let second = receive(dev(D), &mut log, kk, &batch).unwrap();
        assert!(second.new.is_empty());
        assert_eq!(second.clock, kk.max(k(100, 5)));
        assert_eq!(second.report, Report::default());
    }
}

#[test]
fn receive_new_is_unknown_writes_deduplicated_in_batch_order() {
    let w1 = write(record("r"), field("f", "a"), stamp(1, 0, PEER));
    let w2 = write(record("r"), field("f", "b"), stamp(2, 0, PEER));
    let w3 = write(record("s"), field("f", "c"), stamp(3, 0, PEER));
    let mut log = rebuild(dev(D), k(1, 0), std::slice::from_ref(&w1)).unwrap();
    let r = receive(
        dev(D),
        &mut log,
        k(1, 0),
        &[w3.clone(), w1, w2.clone(), w3.clone()],
    )
    .unwrap();
    assert_eq!(r.new, vec![w3, w2]);
}

/// US-6 case 6: invalid batch rejected, `Log` and `K` unchanged.
#[test]
fn us6_case6_invalid_batch_rejected() {
    let mut bad = write(record("r"), field("f", "b"), stamp(5, 0, PEER));
    bad.seen.insert(kind("f"), stamp(5, 0, PEER));
    let batch = [write(record("r"), field("f", "a"), stamp(4, 0, PEER)), bad];
    let mut log = fresh_log(k(3, 0));
    let before = log.clone();
    assert_eq!(
        receive(dev(D), &mut log, k(3, 0), &batch).err(),
        Some(Rejection::Batch(BatchError {
            index: 1,
            reason: Reason::SeenNotBefore
        }))
    );
    assert_eq!(log, before);
}

/// Q-A guard: stored local identity re-sent with another content.
#[test]
fn qa_guard_stored_local_identity_with_other_content() {
    let stored = write(record("r"), field("f", "a"), stamp(10, 0, D));
    let mut log = rebuild(dev(D), k(10, 0), &[stored]).unwrap();
    let before = log.clone();
    let batch = [
        write(record("s"), field("f", "x"), stamp(11, 0, PEER)),
        write(record("r"), field("f", "b"), stamp(10, 0, D)),
    ];
    assert_eq!(
        receive(dev(D), &mut log, k(10, 0), &batch).err(),
        Some(Rejection::LocalIdentityReused { index: 1 })
    );
    assert_eq!(log, before);
}

/// Q-A guard: two contents of one local identity inside the batch.
#[test]
fn qa_guard_two_contents_of_a_local_identity_in_the_batch() {
    let mut log = fresh_log(k(0, 0));
    let batch = [
        write(record("s"), field("f", "x"), stamp(11, 0, PEER)),
        write(record("r"), field("f", "a"), stamp(10, 0, D)),
        write(record("t"), field("f", "y"), stamp(12, 0, PEER)),
        write(record("r"), field("f", "b"), stamp(10, 0, D)),
    ];
    assert_eq!(
        receive(dev(D), &mut log, k(0, 0), &batch).err(),
        Some(Rejection::LocalIdentityReused { index: 1 })
    );
}

/// Q-A guard: the reported index is the write sharing target **and** stamp, not just one of them.
#[test]
fn qa_guard_index_needs_target_and_stamp_both() {
    let stored = write(record("r"), field("f", "a"), stamp(10, 0, D));
    let reused = write(record("r"), field("f", "b"), stamp(10, 0, D));
    let same_target = write(record("r"), field("f", "x"), stamp(11, 0, PEER));
    let same_stamp = write(record("s"), field("f", "y"), stamp(10, 0, D));
    for first in [same_target, same_stamp] {
        let mut log = rebuild(dev(D), k(10, 0), std::slice::from_ref(&stored)).unwrap();
        assert_eq!(
            receive(dev(D), &mut log, k(10, 0), &[first, reused.clone()]).err(),
            Some(Rejection::LocalIdentityReused { index: 1 })
        );
    }
}

/// A local-device write with an unknown identity is accepted (re-receiving after a loss).
#[test]
fn qa_guard_unknown_local_identity_is_accepted() {
    let mut log = fresh_log(k(5, 0));
    let w = write(record("r"), field("f", "a"), stamp(10, 0, D));
    let r = receive(dev(D), &mut log, k(5, 0), std::slice::from_ref(&w)).unwrap();
    assert_eq!(r.new, vec![w]);
    assert_eq!(r.clock, k(10, 0));
}

#[test]
fn qa_guard_resent_identical_local_write_is_a_duplicate() {
    let w = write(record("r"), field("f", "a"), stamp(10, 0, D));
    let mut log = rebuild(dev(D), k(10, 0), std::slice::from_ref(&w)).unwrap();
    let r = receive(dev(D), &mut log, k(10, 0), &[w]).unwrap();
    assert!(r.new.is_empty());
    assert_eq!(r.report, Report::default());
}

/// US-5 case 4: a peer pair is accepted and reported as an anomaly.
#[test]
fn us5_case4_peer_second_content_is_an_anomaly() {
    let a = write(record("r"), field("f", "a"), stamp(10, 0, PEER));
    let b = write(record("r"), field("f", "b"), stamp(10, 0, PEER));
    let mut log = rebuild(dev(D), k(10, 0), &[a]).unwrap();
    let r = receive(dev(D), &mut log, k(10, 0), std::slice::from_ref(&b)).unwrap();
    assert_eq!(r.new, vec![b]);
    assert_eq!(r.report.new_anomalies.len(), 1);
}

// ---------------------------------------------------------------- T-7: SimStore

struct SimHarness(SimStore);

impl Harness for SimHarness {
    type Store = SimStore;

    fn fresh() -> Self {
        Self(SimStore::new(dev(D)))
    }

    fn store(&self) -> &SimStore {
        &self.0
    }

    fn reopen(&mut self) {
        self.0.reopen();
    }
}

/// PROP-ST02-9, US-6 case 9 (suite body: T-6).
#[test]
fn prop_st02_9_contract_suite_on_sim_store() {
    contract::run::<SimHarness>();
}

/// Observable `Σ`: writes with positions, `K`, the peer's watermark.
#[derive(Debug, PartialEq)]
struct Sigma {
    writes: Vec<(Position, Write)>,
    clock: ClockState,
    mark: Option<Watermark>,
}

fn sigma(s: &SimStore) -> Sigma {
    Sigma {
        writes: ready(s.writes_since(Position::START)).unwrap(),
        clock: s.clock_state(),
        mark: ready(s.watermark(dev(PEER))).unwrap(),
    }
}

fn sim() -> SimStore {
    SimStore::new(dev(D))
}

fn changes(n: usize, tag: u32) -> Vec<LocalChange> {
    (0..n)
        .map(|i| local(&format!("r{}", i % 3), field("f", &format!("{tag}-{i}"))))
        .collect()
}

#[derive(Debug, Clone, Copy)]
enum Level {
    Before,
    Inside,
    Commit,
    Watermark,
}

/// E-ST02-2 on `SimStore`: `StorageFull`, `Σ` unchanged, reads work, retry = one success.
fn full_case(level: Level, prior: usize, n: usize, cut: prop::sample::Index) {
    let s = sim();
    ready(s.apply_local(1, &changes(prior, 0))).unwrap();
    ready(s.advance_watermark(dev(PEER), Watermark(1, 0))).unwrap();
    let before = sigma(&s);
    let batch = changes(n, 1);
    let op = || match level {
        Level::Watermark => ready(s.advance_watermark(dev(PEER), Watermark(2, 0)))
            .map_err(ApplyError::from)
            .map(|_| ()),
        _ => ready(s.apply_local(2, &batch)).map(|_| ()),
    };
    s.full_after(match level {
        Level::Before | Level::Watermark => 0,
        Level::Inside if n > 1 => 1 + cut.index(n - 1),
        Level::Inside => 0,
        Level::Commit => n,
    });
    assert_eq!(op(), Err(ApplyError::Store(StoreError::StorageFull)));
    assert_eq!(sigma(&s), before, "RG-ST02-2.1, 2.3");
    s.full_after(usize::MAX);
    op().expect("retry after freeing space (RG-ST02-2.2)");
    let after = sigma(&s);
    match level {
        Level::Watermark => assert_eq!(after.mark, Some(Watermark(2, 0))),
        _ => {
            assert_eq!(after.writes.len(), before.writes.len() + n);
            assert_eq!(after.writes[..before.writes.len()], before.writes[..]);
        }
    }
}

fn full_config() -> ProptestConfig {
    ProptestConfig {
        cases: 100,
        ..config()
    }
}

proptest! {
    #![proptest_config(full_config())]

    #[test]
    fn e_st02_2_full_before_the_batch(prior in 0usize..5, n in 1usize..=20, cut in any::<prop::sample::Index>()) {
        full_case(Level::Before, prior, n, cut);
    }

    #[test]
    fn e_st02_2_full_inside_the_batch(prior in 0usize..5, n in 1usize..=20, cut in any::<prop::sample::Index>()) {
        full_case(Level::Inside, prior, n, cut);
    }

    #[test]
    fn e_st02_2_full_at_commit(prior in 0usize..5, n in 1usize..=20, cut in any::<prop::sample::Index>()) {
        full_case(Level::Commit, prior, n, cut);
    }

    #[test]
    fn e_st02_2_full_while_advancing_a_watermark(prior in 0usize..5, n in 1usize..=20, cut in any::<prop::sample::Index>()) {
        full_case(Level::Watermark, prior, n, cut);
    }
}

/// A batch of `n` writes plus the commit costs `n + 1`: a budget of `n` fails, `n + 1` succeeds.
#[test]
fn budget_equal_to_the_cost_succeeds() {
    let n = 3;
    let s = sim();
    s.full_after(n);
    assert_eq!(
        ready(s.apply_local(1, &changes(n, 0))),
        Err(ApplyError::Store(StoreError::StorageFull))
    );
    s.full_after(n + 1);
    ready(s.apply_local(1, &changes(n, 0))).expect("budget = cost");
}

/// Same bound on the received path (E-ST02-2 level "full at commit").
#[test]
fn received_batch_full_at_commit() {
    let n = 3;
    let batch: Vec<Write> = (0..n as u64)
        .map(|i| {
            write(
                record("r"),
                field(&format!("f{i}"), "a"),
                stamp(1_000_000 + i, 0, PEER),
            )
        })
        .collect();
    let s = sim();
    let before = sigma(&s);
    s.full_after(n);
    assert_eq!(
        ready(s.apply(&batch)),
        Err(ApplyError::Store(StoreError::StorageFull))
    );
    assert_eq!(sigma(&s), before);
    s.full_after(n + 1);
    ready(s.apply(&batch)).expect("budget = cost");
}

/// `clock_state()` is the stamp of the last write after `apply_local`.
#[test]
fn clock_state_follows_the_last_stamp() {
    let s = sim();
    let ws = ready(s.apply_local(1_000, &changes(3, 0))).unwrap();
    let last = ws.last().unwrap().stamp;
    assert_eq!(
        s.clock_state(),
        ClockState {
            l: last.l,
            c: last.c
        }
    );
    assert_ne!(s.clock_state(), ClockState::default());
}

/// Q-R1 (b): a no-op write on full storage succeeds (`Store` trait doc).
#[test]
fn no_op_writes_on_full_storage_succeed() {
    let s = sim();
    let batch = [write(
        record("r"),
        field("f", "a"),
        stamp(1_000_000, 0, PEER),
    )];
    ready(s.apply(&batch)).unwrap();
    ready(s.advance_watermark(dev(PEER), Watermark(5, 0))).unwrap();
    let before = sigma(&s);
    s.full_after(0);
    ready(s.apply(&batch)).expect("stored batch");
    ready(s.apply(&[])).expect("empty batch");
    for to in [Watermark(5, 0), Watermark(1, 0)] {
        ready(s.advance_watermark(dev(PEER), to)).expect("no-op watermark");
    }
    assert_eq!(sigma(&s), before);
}

/// US-2 case 8: received batch on full storage, `K` unchanged.
#[test]
fn us2_case8_received_batch_on_full_storage() {
    let s = sim();
    let before = sigma(&s);
    s.full_after(0);
    let batch = [write(
        record("r"),
        field("f", "a"),
        stamp(1_000_000, 0, PEER),
    )];
    assert_eq!(
        ready(s.apply(&batch)),
        Err(ApplyError::Store(StoreError::StorageFull))
    );
    assert_eq!(sigma(&s), before);
}

/// RG-ST02-5.5: no partial read while unavailable.
#[test]
fn unavailable_n_operations_then_back() {
    let s = sim();
    ready(s.apply_local(1, &changes(2, 0))).unwrap();
    let before = sigma(&s);
    s.unavailable(3);
    assert!(matches!(
        ready(s.writes_of(&record("r0"))),
        Err(StoreError::Unavailable(_))
    ));
    assert!(matches!(
        ready(s.apply_local(2, &changes(1, 1))),
        Err(ApplyError::Store(StoreError::Unavailable(_)))
    ));
    assert!(matches!(
        ready(s.writes_since(Position::START)),
        Err(StoreError::Unavailable(_))
    ));
    assert_eq!(sigma(&s), before);
}

/// US-6 case 8, RG-ST02-1.3.
#[test]
fn us6_case8_crash_before_commit() {
    let s = sim();
    ready(s.apply_local(1, &changes(2, 0))).unwrap();
    let before = sigma(&s);
    s.crash_next_write();
    assert!(matches!(
        ready(s.apply_local(2, &changes(3, 1))),
        Err(ApplyError::Store(StoreError::Unavailable(Cause::Io(_))))
    ));
    assert!(matches!(ready(s.device()), Err(StoreError::Unavailable(_))));
    assert!(matches!(
        ready(s.writes_since(Position::START)),
        Err(StoreError::Unavailable(_))
    ));
    s.reopen();
    assert_eq!(sigma(&s), before);
}

/// Q-P5 → (a): damage poisons every operation and survives `reopen`.
#[test]
fn damage_poisons_and_survives_reopen() {
    let s = sim();
    ready(s.apply_local(1, &changes(2, 0))).unwrap();
    s.corrupt(Damage::ClockState);
    let damaged = StoreError::Corrupted(Damage::ClockState);
    assert_eq!(ready(s.device()), Err(damaged.clone()));
    assert_eq!(
        ready(s.writes_since(Position::START)).err(),
        Some(damaged.clone())
    );
    assert_eq!(
        ready(s.apply_local(2, &changes(1, 1))).err(),
        Some(ApplyError::Store(damaged.clone()))
    );
    s.reopen();
    assert_eq!(ready(s.device()), Err(damaged));
    assert_eq!(ready(sim().device()), Ok(dev(D)));
}

/// DA F-3: a Q-A rejection leaves `Σ` unchanged and does not poison.
#[test]
fn qa_rejection_on_sim_store_does_not_poison() {
    let s = sim();
    let stored = ready(s.apply_local(10, &[local("r", field("f", "a"))])).unwrap();
    let before = sigma(&s);
    let mut reused = stored[0].clone();
    reused.change = field("f", "b");
    assert_eq!(
        ready(s.apply(&[reused])),
        Err(ApplyError::Rejected(Rejection::LocalIdentityReused {
            index: 0
        }))
    );
    assert_eq!(sigma(&s), before);
    s.reopen();
    assert_eq!(sigma(&s), before);
    assert!(ready(s.apply_local(11, &changes(1, 1))).is_ok());
}

// Negative static check: `<T as AmbiguousIf*<_>>` compiles only when `T` lacks the trait.
trait AmbiguousIfSend<A> {
    fn check() {}
}
impl<T: ?Sized> AmbiguousIfSend<()> for T {}
impl<T: ?Sized + Send> AmbiguousIfSend<u8> for T {}

trait AmbiguousIfSync<A> {
    fn check() {}
}
impl<T: ?Sized> AmbiguousIfSync<()> for T {}
impl<T: ?Sized + Sync> AmbiguousIfSync<u8> for T {}

async fn use_store<S: Store>(s: &S) -> Result<DeviceId, StoreError> {
    s.device().await
}

/// US-6 case 2, RG-ST02-6.2: generic code over a non-`Send`, non-`Sync` store.
#[test]
fn us6_case2_generic_code_over_non_send_store() {
    let _ = <SimStore as AmbiguousIfSend<_>>::check;
    let _ = <SimStore as AmbiguousIfSync<_>>::check;
    assert_eq!(ready(use_store(&sim())), Ok(dev(D)));
}

/// US-6 case 3: two futures on one handle, both ready.
#[test]
fn us6_case3_two_requests_in_flight() {
    let s = sim();
    let batch = changes(1, 0);
    let read = s.writes_since(Position::START);
    let written = s.apply_local(1, &batch);
    assert!(ready(written).is_ok());
    assert!(ready(read).is_ok());
}
