//! Stamps, hybrid clock, leads, simulated clock (T-3). US-4 matrix, cases 1 to 10.

mod support;

use std::num::NonZeroU64;

use arbitre_core::*;
use proptest::prelude::*;
use support::*;

const MIN: u64 = 60_000;
const HEURE: u64 = 3_600_000;

fn threshold(ms: u64) -> Threshold {
    Threshold(NonZeroU64::new(ms).unwrap())
}

fn write_at(l: u64, a: u128) -> Write {
    write(record("d"), Change::field("f", b"v"), stamp(l, 0, a))
}

#[test]
fn us4_case1_correction_on_lagging_device() {
    let t0 = 1_000_000;
    let source_ordi = SimClock::new(t0 + 5 * MIN);
    let source_tel = SimClock::new(t0);
    let mut ordi = HybridClock::new(dev(1));
    let mut tel = HybridClock::new(dev(2));
    let m = ordi.issue(source_ordi.now()).unwrap();
    tel.receive(&m);
    source_tel.advance(MIN);
    let m2 = tel.issue(source_tel.now()).unwrap();
    assert!(m2 > m);
}

#[test]
fn us4_case2_two_issues_same_instant() {
    let mut h = HybridClock::new(dev(1));
    let m1 = h.issue(100).unwrap();
    let m2 = h.issue(100).unwrap();
    assert!(m2 > m1);
    assert_eq!((m2.l, m2.c), (100, m1.c + 1));
}

#[test]
fn us4_case3_hundred_thousand_issues_same_instant() {
    let mut h = HybridClock::new(dev(1));
    let mut previous = h.issue(100).unwrap();
    for _ in 1..100_000 {
        let m = h.issue(100).unwrap();
        assert!(m > previous);
        previous = m;
    }
}

#[test]
fn us4_case4_time_goes_back() {
    let mut h = HybridClock::new(dev(1));
    let m1 = h.issue(100).unwrap();
    let m2 = h.issue(40).unwrap();
    assert!(m2 > m1);
    assert_eq!(m2.l, 100);
}

#[test]
fn us4_case5_time_advances_after_freeze() {
    let mut h = HybridClock::new(dev(1));
    for _ in 0..8 {
        h.issue(100).unwrap();
    }
    assert_eq!(h.issue(150).unwrap(), stamp(150, 0, 1));
}

#[test]
fn us4_case7_three_hour_lead_reported_and_adopted() {
    let now_ms = 10 * HEURE;
    let batch = [write_at(now_ms + 3 * HEURE, 7)];
    assert_eq!(
        leads(&batch, now_ms, threshold(HEURE)),
        vec![Lead {
            device: dev(7),
            lead_ms: 3 * HEURE
        }]
    );
    let mut h = HybridClock::new(dev(1));
    h.receive(&batch[0].stamp);
    assert_eq!(h.issue(now_ms).unwrap().l, now_ms + 3 * HEURE);
}

#[test]
fn us4_case8_lead_equal_to_threshold_not_reported() {
    let batch = [write_at(1_000 + HEURE, 7)];
    assert!(leads(&batch, 1_000, DEFAULT_THRESHOLD).is_empty());
}

#[test]
fn us4_case9_threshold_set_to_ten_minutes() {
    let batch = [write_at(1_000 + 20 * MIN, 7)];
    assert_eq!(
        leads(&batch, 1_000, threshold(10 * MIN)),
        vec![Lead {
            device: dev(7),
            lead_ms: 20 * MIN
        }]
    );
}

#[test]
fn us4_case10_late_write_nothing_to_report() {
    let batch = [write_at(10 * HEURE - 2 * HEURE, 7)];
    assert!(leads(&batch, 10 * HEURE, DEFAULT_THRESHOLD).is_empty());
}

#[test]
fn leads_max_per_device_sorted_by_device() {
    let m = 10 * HEURE;
    let batch = [
        write_at(m + 3 * HEURE, 9),
        write_at(m + 2 * HEURE, 3),
        write_at(m + 5 * HEURE, 9),
    ];
    assert_eq!(
        leads(&batch, m, DEFAULT_THRESHOLD),
        vec![
            Lead {
                device: dev(3),
                lead_ms: 2 * HEURE
            },
            Lead {
                device: dev(9),
                lead_ms: 5 * HEURE
            },
        ]
    );
}

#[test]
fn bound_counter_saturated_and_time_frozen() {
    let mut h = HybridClock::new(dev(1));
    h.receive(&stamp(100, u32::MAX, 2));
    let before = h.clone();
    assert_eq!(h.issue(100), Err(ClockExhausted));
    assert_eq!(h, before);
    assert_eq!(h.issue(101).unwrap(), stamp(101, 0, 1));
}

#[test]
fn bound_l_max_reached() {
    let mut h = HybridClock::new(dev(1));
    let before = h.clone();
    assert_eq!(h.issue(L_MAX), Err(ClockExhausted));
    assert_eq!(h, before);
}

#[test]
fn bound_l_max_minus_one_issues() {
    let mut h = HybridClock::new(dev(1));
    assert_eq!(h.issue(L_MAX - 1).unwrap(), stamp(L_MAX - 1, 0, 1));
}

#[test]
fn device_id_preserves_integer_order() {
    assert!(dev(1) < dev(2));
    assert!(dev(255) < dev(256));
    assert!(dev(u128::MAX - 1) < dev(u128::MAX));
}

#[test]
fn sim_clock_set_advance_rewind() {
    let s = SimClock::new(1_000);
    assert_eq!(s.now(), 1_000);
    s.advance(500);
    assert_eq!(s.now(), 1_500);
    s.rewind(1_200);
    assert_eq!(s.now(), 300);
    s.set(42);
    assert_eq!(s.now(), 42);
}

#[test]
fn sim_clock_freeze() {
    let s = SimClock::new(1_000);
    s.freeze();
    s.advance(500);
    s.rewind(200);
    assert_eq!(s.now(), 1_000);
    s.set(2_000);
    s.advance(1);
    assert_eq!(s.now(), 2_001);
}

fn any_stamp() -> impl Strategy<Value = Stamp> {
    (0u64..4, 0u32..3, 0u128..3).prop_map(|(l, c, a)| stamp(l, c, a))
}

/// Sequences of times: increasing, constant, decreasing or shuffled (INV-ST01-5).
fn any_times() -> impl Strategy<Value = Vec<u64>> {
    let brut = prop::collection::vec(0u64..10_000, 1..50);
    prop_oneof![
        brut.clone().prop_map(|mut v| {
            v.sort();
            v
        }),
        (0u64..10_000, 1usize..50).prop_map(|(t, n)| vec![t; n]),
        brut.clone().prop_map(|mut v| {
            v.sort_by(|a, b| b.cmp(a));
            v
        }),
        brut,
    ]
}

proptest! {
    #![proptest_config(config())]

    /// US-4 case 6, INV-ST01-4: total order, lexicographic on `(l, c, a)`.
    #[test]
    fn inv_st01_4_total_order(m1 in any_stamp(), m2 in any_stamp()) {
        prop_assert_eq!(m1.cmp(&m2), (m1.l, m1.c, m1.a).cmp(&(m2.l, m2.c, m2.a)));
        prop_assert_eq!(m1 == m2, m1.cmp(&m2).is_eq());
        prop_assert_eq!(m1.cmp(&m2), m2.cmp(&m1).reverse());
    }

    /// INV-ST01-5: strictly increasing issues, `l ≥ pt`.
    #[test]
    fn inv_st01_5_increasing_issues(times in any_times()) {
        let mut h = HybridClock::new(dev(1));
        let mut previous: Option<Stamp> = None;
        for pt in times {
            let m = h.issue(pt).unwrap();
            prop_assert!(m.l >= pt);
            if let Some(p) = previous {
                prop_assert!(m > p);
            }
            previous = Some(m);
        }
    }

    /// INV-ST01-6: after `receive(m)`, every issue is `> m`.
    #[test]
    fn inv_st01_6_receive(before in any_times(), l in 0u64..20_000, c in 0u32..1_000, a in 0u128..5, pt in 0u64..20_000) {
        let mut h = HybridClock::new(dev(2));
        for t in before {
            h.issue(t).unwrap();
        }
        let m = stamp(l, c, a);
        h.receive(&m);
        prop_assert!(h.issue(pt).unwrap() > m);
    }

    /// INV-ST01-8: only what exceeds `σ` is reported, largest lead per device, sorted by device.
    #[test]
    fn inv_st01_8_leads(ls in prop::collection::vec((0u64..200, 0u128..4), 0..20), now_ms in 0u64..100, s in 1u64..60) {
        let batch: Vec<Write> = ls.iter().map(|&(l, a)| write_at(l, a)).collect();
        let returned = leads(&batch, now_ms, threshold(s));
        prop_assert!(returned.windows(2).all(|p| p[0].device < p[1].device));
        for a in 0u128..4 {
            let max = ls.iter().filter(|&&(_, b)| b == a).map(|&(l, _)| l.saturating_sub(now_ms)).max();
            let expected = max.filter(|&av| av > s).map(|av| Lead { device: dev(a), lead_ms: av });
            prop_assert_eq!(returned.iter().find(|x| x.device == dev(a)).copied(), expected);
        }
    }
}
