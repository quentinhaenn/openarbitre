//! Simulated devices, causal oracle, convergence (T-7). PROP-ST01-2, INV-5, INV-ST01-1, INV-ST01-2.
//! Matrices US-1 cases 4, 5, 6, 9; US-2 case 5; US-3 case 8 (generated scenarios with deletions).

mod support;

use std::cell::RefCell;
use std::collections::BTreeSet;

use arbitre_core::*;
use proptest::prelude::*;
use proptest::test_runner::TestRunner;
use support::*;

fn received(batches: &[&[Write]]) -> Log {
    let mut x = Log::new(dev(100));
    for batch in batches {
        x.merge(batch).unwrap();
    }
    x
}

/// All writes of a fully delivered scenario, and masks to split them into batches.
fn s_and_masks() -> impl Strategy<Value = (Vec<Write>, Vec<bool>, Vec<bool>)> {
    scenario().prop_flat_map(|scn| {
        let mut sim = Simulation::new(&scn);
        for action in &scn.actions {
            sim.apply(action);
        }
        sim.deliver_all();
        let s = sim.devices[0].log.writes();
        let n = s.len();
        (
            Just(s),
            prop::collection::vec(any::<bool>(), n),
            prop::collection::vec(any::<bool>(), n),
        )
    })
}

fn split(s: &[Write], mask: &[bool]) -> (Vec<Write>, Vec<Write>) {
    let (a, b): (Vec<_>, Vec<_>) = s.iter().cloned().zip(mask).partition(|(_, m)| **m);
    (
        a.into_iter().map(|(w, _)| w).collect(),
        b.into_iter().map(|(w, _)| w).collect(),
    )
}

proptest! {
    #![proptest_config(config())]

    /// INV-ST01-2, F-2, D-10: after each action, on each device, `conflicts()` = oracle;
    /// the concatenation of `new_conflicts` has no duplicates and equals `conflicts()` as a set.
    #[test]
    fn concurrent_iff_conflict(scn in scenario()) {
        let mut sim = Simulation::new(&scn);
        let mut accumulated: Vec<Vec<Conflict>> = vec![Vec::new(); sim.devices.len()];
        let mut check = |sim: &Simulation, reports: Vec<(usize, Report)>| -> Result<(), TestCaseError> {
            for (i, r) in reports {
                accumulated[i].extend(r.new_conflicts);
            }
            let ledger = sim.ledger();
            for (i, device) in sim.devices.iter().enumerate() {
                let conflicts = device.log.conflicts();
                prop_assert_eq!(&conflicts, &expected_conflicts(&device.log.writes(), &ledger));
                let as_set: BTreeSet<Conflict> = accumulated[i].iter().cloned().collect();
                prop_assert_eq!(as_set.len(), accumulated[i].len());
                prop_assert_eq!(as_set, conflicts.into_iter().collect::<BTreeSet<_>>());
            }
            Ok(())
        };
        for action in &scn.actions {
            let reports = sim.apply(action);
            check(&sim, reports)?;
        }
        let reports = sim.deliver_all();
        check(&sim, reports)?;
    }

    /// PROP-ST01-2, US-1 cases 4 and 5, US-2 case 5: at the end of a scenario, `R(S)` is identical everywhere.
    #[test]
    fn convergence(scn in scenario()) {
        let mut sim = Simulation::new(&scn);
        for action in &scn.actions {
            sim.apply(action);
        }
        sim.deliver_all();
        let baseline = outputs_of(&sim.devices[0].log);
        for device in &sim.devices[1..] {
            prop_assert_eq!(&outputs_of(&device.log), &baseline);
        }
    }

    /// INV-5: commutativity, associativity, idempotence, neutral element (US-1 case 6, US-3 case 8, AC-3.5).
    #[test]
    fn inv_5_algebra((s, m1, m2) in s_and_masks()) {
        let (a, rest) = split(&s, &m1);
        let (b, c) = split(&rest, &m2[..rest.len()]);
        let ab_c = outputs_of(&received(&[&a, &b, &c]));
        prop_assert_eq!(&outputs_of(&received(&[&b, &a, &c])), &ab_c);
        prop_assert_eq!(&outputs_of(&received(&[&c, &b, &a])), &ab_c);
        let bc: Vec<Write> = b.iter().chain(&c).cloned().collect();
        prop_assert_eq!(&outputs_of(&received(&[&a, &bc])), &ab_c);
        let mut x = received(&[&a, &b, &c]);
        prop_assert_eq!(x.merge(&a).unwrap(), Report::default());
        prop_assert_eq!(x.merge(&[]).unwrap(), Report::default());
        prop_assert_eq!(&outputs_of(&x), &ab_c);
    }

    /// INV-ST01-1, US-1 case 9: identical outputs under permutation of a batch's internal order.
    #[test]
    fn determinism_permutation((s, _, _) in s_and_masks(), seed in any::<u64>()) {
        let mut permuted = s.clone();
        let mut r = seed;
        for i in (1..permuted.len()).rev() {
            r = r.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            permuted.swap(i, (r >> 33) as usize % (i + 1));
        }
        let (mut x, mut y) = (Log::new(dev(100)), Log::new(dev(100)));
        prop_assert_eq!(x.merge(&s).unwrap(), y.merge(&permuted).unwrap());
        prop_assert_eq!(outputs_of(&x), outputs_of(&y));
    }
}

/// R-7: over the 1,000 scenarios of the seed, each E-1.1 family appears at least once.
#[test]
fn family_coverage() {
    let seen_families = RefCell::new(BTreeSet::new());
    TestRunner::new(config())
        .run(&scenario(), |scn| {
            let mut sim = Simulation::new(&scn);
            for action in &scn.actions {
                sim.apply(action);
            }
            sim.deliver_all();
            seen_families.borrow_mut().extend(sim.families());
            Ok(())
        })
        .unwrap();
    let all_families = BTreeSet::from([
        Family::EmptyBatch,
        Family::Duplicate,
        Family::ThreeDevices,
        Family::SameL,
        Family::SameLCTwoDevices,
        Family::Delete,
        Family::ConcurrentSameValue,
        Family::SequentialOverwrite,
        Family::CounterExampleF1,
    ]);
    assert_eq!(seen_families.into_inner(), all_families);
}
