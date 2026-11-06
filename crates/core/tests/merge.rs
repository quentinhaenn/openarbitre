//! Log, merge, projection, discarded, anomalies (T-5); conflicts and sorting (T-6).
//! Matrices US-1, US-2, US-3; A-1 to A-4, A-10.

mod support;

use std::collections::BTreeSet;
use std::time::Instant;

use arbitre_core::*;
use proptest::prelude::*;
use support::*;

const A: u128 = 1;
const B: u128 = 2;
const C: u128 = 3;
const X: u128 = 9;

fn log_of(n: u128) -> Log {
    Log::new(dev(n))
}

fn put(log: &mut Log, pt: u64, d: &str, change: Change) -> Write {
    log.write(pt, record(d), change, Author::CliUser).unwrap()
}

fn received(batch: &[Write]) -> Log {
    let mut x = log_of(X);
    x.merge(batch).unwrap();
    x
}

fn value_of(x: &Log, d: &str, f: &str) -> Change {
    x.projection()[&record(d)].fields[&Field(f.into())]
        .change
        .clone()
}

fn status(x: &Log, d: &str) -> Status {
    x.projection()[&record(d)].status.clone()
}

fn conflict(retained: &Write, discarded: &Write, f: &str) -> Conflict {
    Conflict {
        record: retained.record.clone(),
        field: Field(f.into()),
        retained: retained.clone(),
        discarded: discarded.clone(),
    }
}

// ---- T-5: US-1 matrix ----

#[test]
fn us1_case1_different_fields_two_devices() {
    let wa = put(&mut log_of(A), 10, "d", Change::field("title", b"v1"));
    let wb = put(&mut log_of(B), 12, "d", Change::field("due", b"v2"));
    let x = received(&[wa, wb]);
    assert_eq!(value_of(&x, "d", "title"), Change::field("title", b"v1"));
    assert_eq!(value_of(&x, "d", "due"), Change::field("due", b"v2"));
    assert_eq!(status(&x, "d"), Status::Present);
    assert!(x.discarded().is_empty());
}

#[test]
fn us1_case2_same_field_different_stamps() {
    let wa = put(&mut log_of(A), 10, "d", Change::field("f", b"v1"));
    let wb = put(&mut log_of(B), 12, "d", Change::field("f", b"v2"));
    let x = received(&[wa.clone(), wb]);
    assert_eq!(value_of(&x, "d", "f"), Change::field("f", b"v2"));
    assert_eq!(x.discarded(), vec![wa]);
}

#[test]
fn us1_case3_same_l_c_tie_broken_by_device() {
    let wa = put(&mut log_of(A), 10, "d", Change::field("f", b"v1"));
    let wb = put(&mut log_of(B), 10, "d", Change::field("f", b"v2"));
    assert_eq!((wa.stamp.l, wa.stamp.c), (wb.stamp.l, wb.stamp.c));
    for batch in [[wa.clone(), wb.clone()], [wb.clone(), wa.clone()]] {
        let x = received(&batch);
        assert_eq!(value_of(&x, "d", "f"), Change::field("f", b"v2"));
        assert_eq!(x.discarded(), vec![wa.clone()]);
    }
}

#[test]
fn us1_case7_empty_batch() {
    let mut x = received(&[put(&mut log_of(A), 10, "d", Change::field("f", b"v1"))]);
    let before = (outputs_of(&x), x.anomalies());
    assert_eq!(x.merge(&[]).unwrap(), Report::default());
    assert_eq!((outputs_of(&x), x.anomalies()), before);
}

#[test]
fn us1_case8_ten_thousand_sequential_writes() {
    let mut a = log_of(A);
    for k in 0..10_000u64 {
        put(
            &mut a,
            k / 3,
            "d",
            Change::field("f", k.to_string().as_bytes()),
        );
    }
    assert_eq!(value_of(&a, "d", "f"), Change::field("f", b"9999"));
    assert_eq!(a.discarded().len(), 9_999);
}

#[test]
fn us1_case10_identity_collision() {
    let w1 = write(record("d"), Change::field("f", b"v1"), stamp(10, 0, A));
    let w2 = write(record("d"), Change::field("f", b"v2"), stamp(10, 0, A));
    let mut x = received(std::slice::from_ref(&w1));
    assert!(x.anomalies().is_empty());
    let report = x.merge(std::slice::from_ref(&w2)).unwrap();
    assert_eq!(x.writes(), vec![w1.clone(), w2.clone()]);
    assert_eq!(value_of(&x, "d", "f"), Change::field("f", b"v2"));
    assert_eq!(x.discarded(), vec![w1.clone()]);
    assert_eq!(report.new_anomalies.len(), 1);
    assert_eq!(x.anomalies().len(), 1);
    let before = outputs_of(&x);
    let second = x.merge(&[w1]).unwrap();
    assert!(second.new_anomalies.is_empty());
    assert_eq!(outputs_of(&x), before);
}

#[test]
fn us1_case11_malformed_batch_rejected_whole() {
    let mut x = received(&[put(&mut log_of(A), 10, "d", Change::field("f", b"v1"))]);
    let before = x.clone();
    let good = put(&mut log_of(B), 20, "d", Change::field("f", b"v2"));
    let bad = write(record("d"), Change::field("", b"v3"), stamp(21, 0, B));
    assert_eq!(
        x.merge(&[good, bad]),
        Err(BatchError {
            index: 1,
            reason: Reason::EmptyIdentifier
        })
    );
    assert_eq!(x, before);
}

// ---- T-5: US-3 matrix (state and discarded) ----

#[test]
fn us3_case1_deletion_then_concurrent_edit() {
    let s = put(&mut log_of(A), 10, "d", Change::Delete);
    let w = put(&mut log_of(B), 12, "d", Change::field("f", b"v"));
    let x = received(&[s.clone(), w]);
    assert_eq!(status(&x, "d"), Status::Present);
    assert_eq!(value_of(&x, "d", "f"), Change::field("f", b"v"));
    assert_eq!(x.discarded(), vec![s]);
}

#[test]
fn us3_case2_edit_then_concurrent_deletion() {
    let w = put(&mut log_of(A), 10, "d", Change::field("f", b"v"));
    let s = put(&mut log_of(B), 12, "d", Change::Delete);
    let x = received(&[w, s.clone()]);
    assert_eq!(status(&x, "d"), Status::Deleted(s));
    assert_eq!(value_of(&x, "d", "f"), Change::field("f", b"v"));
    assert!(x.discarded().is_empty());
}

#[test]
fn us3_case3_sequential_deletion() {
    let w = put(&mut log_of(A), 10, "d", Change::field("f", b"v"));
    let mut b = received(&[w]);
    let s = put(&mut b, 11, "d", Change::Delete);
    assert_eq!(status(&b, "d"), Status::Deleted(s));
}

#[test]
fn us3_case4_sequential_resurrection() {
    let s = put(&mut log_of(A), 10, "d", Change::Delete);
    let mut b = received(std::slice::from_ref(&s));
    put(&mut b, 11, "d", Change::field("f", b"v"));
    assert_eq!(status(&b, "d"), Status::Present);
    assert_eq!(b.discarded(), vec![s]);
}

#[test]
fn us3_case5_same_l_c_deletion_vs_field() {
    let s = put(&mut log_of(A), 10, "d", Change::Delete);
    let w = put(&mut log_of(B), 10, "d", Change::field("f", b"v"));
    for batch in [[s.clone(), w.clone()], [w.clone(), s.clone()]] {
        assert_eq!(status(&received(&batch), "d"), Status::Present);
    }
}

#[test]
fn us3_case6_concurrent_double_deletion() {
    let sa = put(&mut log_of(A), 10, "d", Change::Delete);
    let sb = put(&mut log_of(B), 12, "d", Change::Delete);
    let x = received(&[sa.clone(), sb.clone()]);
    assert_eq!(status(&x, "d"), Status::Deleted(sb));
    assert_eq!(x.discarded(), vec![sa]);
}

#[test]
fn us3_case7_resurrection_all_fields() {
    let mut a = log_of(A);
    let f1a = put(&mut a, 5, "d", Change::field("f1", b"a"));
    put(&mut a, 6, "d", Change::field("f2", b"b"));
    let s = put(&mut a, 8, "d", Change::Delete);
    put(&mut a, 9, "d", Change::field("f1", b"c"));
    assert_eq!(status(&a, "d"), Status::Present);
    assert_eq!(value_of(&a, "d", "f1"), Change::field("f1", b"c"));
    assert_eq!(value_of(&a, "d", "f2"), Change::field("f2", b"b"));
    assert_eq!(a.discarded(), vec![s, f1a]);
}

#[test]
fn us3_case9_deletion_alone() {
    let s = put(&mut log_of(A), 10, "d", Change::Delete);
    let x = received(std::slice::from_ref(&s));
    assert_eq!(status(&x, "d"), Status::Deleted(s));
    assert!(x.projection()[&record("d")].fields.is_empty());
}

#[test]
fn us2_case9_full_discarded_write() {
    let wa = Write {
        author: Author::Assistant,
        ..put(&mut log_of(A), 10, "d", Change::field("f", b"v1"))
    };
    let wb = put(&mut log_of(B), 12, "d", Change::field("f", b"v2"));
    let discarded = &received(&[wa.clone(), wb]).discarded()[0];
    assert_eq!(discarded, &wa);
    assert_eq!(discarded.device(), dev(A));
}

// ---- T-5: local write and log clock (A-1, A-2) ----

#[test]
fn write_computes_seen_per_target_kind() {
    let mut a = log_of(A);
    let w1 = put(&mut a, 1, "d", Change::field("f1", b"a"));
    let w2 = put(&mut a, 2, "d", Change::field("f2", b"b"));
    let s = put(&mut a, 3, "d", Change::Delete);
    let w4 = put(&mut a, 4, "d", Change::field("f1", b"c"));
    assert!(w1.seen.is_empty());
    assert!(w2.seen.is_empty());
    assert_eq!(
        s.seen,
        Seen::from([(kind("f1"), w1.stamp), (kind("f2"), w2.stamp)])
    );
    assert_eq!(
        w4.seen,
        Seen::from([(TargetKind::Delete, s.stamp), (kind("f1"), w1.stamp)])
    );
}

#[test]
fn write_ignores_other_records() {
    let mut a = log_of(A);
    put(&mut a, 1, "other", Change::field("f", b"a"));
    put(&mut a, 2, "other", Change::Delete);
    assert!(
        put(&mut a, 3, "d", Change::field("f", b"b"))
            .seen
            .is_empty()
    );
}

#[test]
fn write_empty_identifier_consumes_no_stamp() {
    let mut a = log_of(A);
    assert_eq!(
        put(&mut a, 10, "d", Change::field("f", b"a")).stamp,
        stamp(10, 0, A)
    );
    let before = a.clone();
    assert_eq!(
        a.write(10, record(""), Change::field("f", b"b"), Author::System),
        Err(WriteError::Invalid(Reason::EmptyIdentifier))
    );
    assert_eq!(a, before);
    assert_eq!(
        put(&mut a, 10, "d", Change::field("f", b"c")).stamp,
        stamp(10, 1, A)
    );
}

#[test]
fn write_clock_exhausted() {
    let mut a = log_of(A);
    a.merge(&[write(
        record("d"),
        Change::field("f", b"v"),
        stamp(10, u32::MAX, B),
    )])
    .unwrap();
    assert_eq!(
        a.write(10, record("d"), Change::field("f", b"w"), Author::System),
        Err(WriteError::ClockExhausted)
    );
}

#[test]
fn merge_invalid_batch_log_and_clock_unchanged() {
    let mut a = log_of(A);
    put(&mut a, 10, "d", Change::field("f", b"a"));
    let before = a.clone();
    let batch = [
        write(record("d"), Change::field("f", b"b"), stamp(1_000, 0, B)),
        write(record("d"), Change::field("f", b"c"), stamp(L_MAX, 0, B)),
    ];
    assert_eq!(
        a.merge(&batch),
        Err(BatchError {
            index: 1,
            reason: Reason::StampOutOfBounds
        })
    );
    assert_eq!(a, before);
    assert_eq!(
        put(&mut a, 10, "d", Change::field("f", b"d")).stamp,
        stamp(10, 1, A)
    );
}

#[test]
fn merge_valid_batch_clock_adopts_maximum() {
    let mut a = log_of(A);
    let batch = [
        write(record("d"), Change::field("f", b"b"), stamp(500, 4, B)),
        write(record("e"), Change::field("f", b"c"), stamp(900, 2, C)),
    ];
    a.merge(&batch).unwrap();
    assert!(put(&mut a, 10, "d", Change::field("f", b"x")).stamp > stamp(900, 2, C));
}

// ---- T-6: conflicts (US-2, US-3 matrices) ----

#[test]
fn us2_case1_same_field_offline_on_both_sides() {
    let wa = put(&mut log_of(A), 10, "d", Change::field("f", b"v1"));
    let wb = put(&mut log_of(B), 12, "d", Change::field("f", b"v2"));
    let mut x = log_of(X);
    let report = x.merge(&[wa.clone(), wb.clone()]).unwrap();
    assert_eq!(x.conflicts(), vec![conflict(&wb, &wa, "f")]);
    assert_eq!(report.new_conflicts, x.conflicts());
    assert!(report.new_anomalies.is_empty());
    assert_eq!(report.new_discarded, vec![wa]);
}

#[test]
fn us2_case2_sequential_overwrite() {
    let wa = put(&mut log_of(A), 10, "d", Change::field("f", b"v1"));
    let mut b = received(std::slice::from_ref(&wa));
    let wb = put(&mut b, 11, "d", Change::field("f", b"v2"));
    assert!(b.conflicts().is_empty());
    assert!(received(&[wb, wa]).conflicts().is_empty());
}

#[test]
fn us2_case3_sequential_overwrite_lagging_receiver() {
    let mut a = log_of(A);
    let mut b = log_of(B);
    let w0 = put(&mut b, 5, "d", Change::field("f", b"v0"));
    a.merge(&[w0]).unwrap();
    let wa = put(&mut a, 10, "d", Change::field("f", b"v1"));
    b.merge(&[wa]).unwrap();
    let wb = put(&mut b, 11, "d", Change::field("f", b"v2"));
    let report = a.merge(&[wb]).unwrap();
    assert!(report.new_conflicts.is_empty());
    assert!(a.conflicts().is_empty());
}

#[test]
fn us2_case4_concurrent_same_value() {
    let wa = put(&mut log_of(A), 10, "d", Change::field("f", b"v"));
    let wb = put(&mut log_of(B), 12, "d", Change::field("f", b"v"));
    let x = received(&[wa.clone(), wb.clone()]);
    assert_eq!(x.projection()[&record("d")].fields[&Field("f".into())], wb);
    assert_eq!(x.discarded(), vec![wa]);
    assert!(x.conflicts().is_empty());
}

#[test]
fn us2_case6_three_concurrent_one_conflict_per_pair() {
    let wa = put(&mut log_of(A), 10, "d", Change::field("f", b"v1"));
    let wb = put(&mut log_of(B), 11, "d", Change::field("f", b"v2"));
    let wc = put(&mut log_of(C), 12, "d", Change::field("f", b"v3"));
    let x = received(&[wc.clone(), wa.clone(), wb.clone()]);
    assert_eq!(value_of(&x, "d", "f"), Change::field("f", b"v3"));
    assert_eq!(x.discarded(), vec![wa.clone(), wb.clone()]);
    assert_eq!(
        x.conflicts(),
        vec![
            conflict(&wb, &wa, "f"),
            conflict(&wc, &wa, "f"),
            conflict(&wc, &wb, "f")
        ]
    );
}

#[test]
fn us2_case7_never_synced_peer() {
    let wp = put(&mut log_of(B), 10, "d", Change::field("f", b"v1"));
    let mut local = log_of(A);
    let wl = put(&mut local, 20, "d", Change::field("f", b"v2"));
    let report = local.merge(std::slice::from_ref(&wp)).unwrap();
    assert_eq!(report.new_conflicts, vec![conflict(&wl, &wp, "f")]);
}

#[test]
fn us2_case8_replay_same_lists_same_order() {
    let batch = [
        put(&mut log_of(A), 10, "d", Change::field("f", b"v1")),
        put(&mut log_of(B), 12, "d", Change::field("f", b"v2")),
        put(&mut log_of(C), 11, "e", Change::Delete),
        put(&mut log_of(A), 9, "e", Change::field("g", b"x")),
    ];
    let (mut x, mut y) = (log_of(X), log_of(X));
    assert_eq!(x.merge(&batch).unwrap(), y.merge(&batch).unwrap());
    assert_eq!(outputs_of(&x), outputs_of(&y));
}

#[test]
fn us2_case10_sequential_write_after_conflict() {
    let wa = put(&mut log_of(A), 10, "d", Change::field("f", b"v1"));
    let wb = put(&mut log_of(B), 12, "d", Change::field("f", b"v2"));
    let mut a = log_of(A);
    a.merge(&[wa.clone(), wb.clone()]).unwrap();
    let w3 = put(&mut a, 13, "d", Change::field("f", b"v3"));
    let mut x = received(&[wa.clone(), wb.clone()]);
    let report = x.merge(&[w3]).unwrap();
    assert_eq!(value_of(&x, "d", "f"), Change::field("f", b"v3"));
    assert_eq!(report.new_discarded, vec![wb.clone()]);
    assert!(report.new_conflicts.is_empty());
    assert_eq!(x.conflicts(), vec![conflict(&wb, &wa, "f")]);
}

#[test]
fn us3_case1_conflict_retained_field() {
    let s = put(&mut log_of(A), 10, "d", Change::Delete);
    let w = put(&mut log_of(B), 12, "d", Change::field("f", b"v"));
    assert_eq!(
        received(&[s.clone(), w.clone()]).conflicts(),
        vec![conflict(&w, &s, "f")]
    );
}

#[test]
fn us3_case2_conflict_retained_deletion() {
    let w = put(&mut log_of(A), 10, "d", Change::field("f", b"v"));
    let s = put(&mut log_of(B), 12, "d", Change::Delete);
    assert_eq!(
        received(&[w.clone(), s.clone()]).conflicts(),
        vec![conflict(&s, &w, "f")]
    );
}

#[test]
fn us3_case3_and_4_sequential_no_conflict() {
    let w = put(&mut log_of(A), 10, "d", Change::field("f", b"v"));
    let mut b = received(&[w]);
    put(&mut b, 11, "d", Change::Delete);
    assert!(b.conflicts().is_empty());
    let s = put(&mut log_of(A), 10, "e", Change::Delete);
    let mut c = received(&[s]);
    put(&mut c, 11, "e", Change::field("f", b"v"));
    assert!(c.conflicts().is_empty());
}

#[test]
fn us3_case5_same_l_c_deletion_vs_field_in_conflict() {
    let s = put(&mut log_of(A), 10, "d", Change::Delete);
    let w = put(&mut log_of(B), 10, "d", Change::field("f", b"v"));
    assert_eq!(
        received(&[s.clone(), w.clone()]).conflicts(),
        vec![conflict(&w, &s, "f")]
    );
}

#[test]
fn us3_case6_double_deletion_no_conflict() {
    let sa = put(&mut log_of(A), 10, "d", Change::Delete);
    let sb = put(&mut log_of(B), 12, "d", Change::Delete);
    assert!(received(&[sa, sb]).conflicts().is_empty());
}

#[test]
fn counter_example_f1() {
    let title = put(&mut log_of(B), 10, "d", Change::field("title", b"t"));
    let mut a = log_of(A);
    let due = put(&mut a, 11, "d", Change::field("due", b"e"));
    let s = put(&mut a, 12, "d", Change::Delete);
    assert_eq!(
        received(&[title.clone(), due, s.clone()]).conflicts(),
        vec![conflict(&s, &title, "title")]
    );
}

#[test]
fn case_f12_same_stamp_deletion_and_field() {
    let m = stamp(10, 0, A);
    let s = write(record("d"), Change::Delete, m);
    let w = write(record("d"), Change::field("f", b"v"), m);
    let x = received(&[s.clone(), w.clone()]);
    assert_eq!(status(&x, "d"), Status::Present);
    assert_eq!(x.conflicts(), vec![conflict(&w, &s, "f")]);
}

#[test]
fn collision_of_different_values_is_a_conflict() {
    let w1 = write(record("d"), Change::field("f", b"v1"), stamp(10, 0, A));
    let w2 = write(record("d"), Change::field("f", b"v2"), stamp(10, 0, A));
    assert_eq!(
        received(&[w1.clone(), w2.clone()]).conflicts(),
        vec![conflict(&w2, &w1, "f")]
    );
}

#[test]
fn second_receipt_three_empty_lists() {
    let batch = [
        put(&mut log_of(A), 10, "d", Change::field("f", b"v1")),
        put(&mut log_of(B), 12, "d", Change::field("f", b"v2")),
        write(record("d"), Change::field("g", b"x"), stamp(5, 0, C)),
        write(record("d"), Change::field("g", b"y"), stamp(5, 0, C)),
    ];
    let mut x = received(&batch);
    assert_eq!(x.merge(&batch).unwrap(), Report::default());
}

#[test]
fn us1_case8_ten_thousand_writes_in_one_received_batch() {
    let mut a = log_of(A);
    let batch: Vec<Write> = (0..10_000u64)
        .map(|k| {
            put(
                &mut a,
                k / 3,
                "d",
                Change::field("f", k.to_string().as_bytes()),
            )
        })
        .collect();
    let mut x = log_of(X);
    let start = Instant::now();
    let report = x.merge(&batch).unwrap();
    assert!(start.elapsed().as_secs_f64() < 2.0);
    assert_eq!(report.new_discarded.len(), 9_999);
    assert!(report.new_conflicts.is_empty());
}

// ---- Properties (T-5 partition and monotonicity, T-6 sorting) ----

/// Forged writes over a small domain, empty `seen`: concurrent, frequent collisions (R-7).
fn any_write() -> impl Strategy<Value = Write> {
    (0u8..2, 0u8..3, 0u8..2, 0u64..4, 0u32..2, 1u128..4).prop_map(|(d, v, val, l, c, a)| {
        let d = ["d0", "d1"][d as usize];
        let change = match v {
            2 => Change::Delete,
            _ => Change::field(["f1", "f2"][v as usize], [b"a", b"b"][val as usize]),
        };
        write(record(d), change, stamp(l, c, a))
    })
}

fn any_batch() -> impl Strategy<Value = Vec<Write>> {
    prop::collection::vec(any_write(), 0..12)
}

fn retained_set(x: &Log) -> BTreeSet<Write> {
    let mut r = BTreeSet::new();
    for state in x.projection().into_values() {
        r.extend(state.fields.into_values());
        if let Status::Deleted(s) = state.status {
            r.insert(s);
        }
    }
    r
}

fn discarded_key(w: &Write) -> impl Ord {
    (w.target(), rank(w))
}

fn strict<T: Ord>(v: Vec<T>) -> bool {
    v.windows(2).all(|p| p[0] < p[1])
}

proptest! {
    #![proptest_config(config())]

    /// INV-7: `retained ⊎ discarded = S`.
    #[test]
    fn inv_7_partition(batch in any_batch()) {
        let x = received(&batch);
        let s: BTreeSet<Write> = x.writes().into_iter().collect();
        let e: BTreeSet<Write> = x.discarded().into_iter().collect();
        let r = retained_set(&x);
        prop_assert!(r.is_disjoint(&e));
        prop_assert_eq!(r.union(&e).cloned().collect::<BTreeSet<_>>(), s);
    }

    /// INV-7: `S ⊆ S' ⇒ discarded(S) ⊆ discarded(S')`.
    #[test]
    fn inv_7_monotonicity(a in any_batch(), b in any_batch()) {
        let mut x = received(&a);
        let before: BTreeSet<Write> = x.discarded().into_iter().collect();
        x.merge(&b).unwrap();
        let after: BTreeSet<Write> = x.discarded().into_iter().collect();
        prop_assert!(before.is_subset(&after));
    }

    /// INV-ST01-7: a batch containing an invalid write leaves the log unchanged.
    #[test]
    fn inv_st01_7_invalid_batch(a in any_batch(), b in any_batch(), position in any::<prop::sample::Index>()) {
        let mut x = received(&a);
        let before = x.clone();
        let mut batch = b;
        let i = position.index(batch.len() + 1);
        batch.insert(i, write(record(""), Change::Delete, stamp(1, 0, 1)));
        prop_assert!(x.merge(&batch).is_err());
        prop_assert_eq!(x, before);
    }

    /// INV-ST01-3: reports and `conflicts()` sorted per A-4, no duplicates.
    #[test]
    fn inv_st01_3_sorting(a in any_batch(), b in any_batch()) {
        let mut x = received(&a);
        let report = x.merge(&b).unwrap();
        prop_assert!(strict(report.new_discarded.iter().map(discarded_key).collect()));
        prop_assert!(strict(report.new_conflicts.iter().map(conflict_key).collect()));
        prop_assert!(strict(report.new_anomalies.iter().map(|k| (k.target.clone(), k.stamp)).collect()));
        prop_assert!(strict(x.discarded().iter().map(discarded_key).collect()));
        prop_assert!(strict(x.conflicts().iter().map(conflict_key).collect()));
    }
}
