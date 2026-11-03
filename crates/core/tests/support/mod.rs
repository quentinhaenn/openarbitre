//! Core test support: proptest config, constructors, simulated devices and oracle (T-3, T-7).
#![allow(dead_code)]

use arbitre_core::*;
use proptest::test_runner::{Config as ProptestConfig, RngSeed};

/// Fixed seed of the properties: changing it goes through a PR (F-3).
pub const SEED: u64 = 0x5701_0001;

/// Single configuration of all properties: 1,000 cases, fixed seed (MANQUE-ST01-7, F-3).
pub fn config() -> ProptestConfig {
    ProptestConfig {
        cases: 1000,
        rng_seed: RngSeed::Fixed(SEED),
        ..ProptestConfig::default()
    }
}

pub fn dev(n: u128) -> DeviceId {
    DeviceId::from_u128(n)
}

pub fn stamp(l: u64, c: u32, a: u128) -> Stamp {
    Stamp { l, c, a: dev(a) }
}

pub fn record(id: &str) -> RecordId {
    RecordId {
        module: "m".into(),
        category: "g".into(),
        id: id.into(),
    }
}

pub fn field(name: &str, value: &str) -> Change {
    Change::Field {
        name: Field(name.into()),
        value: Value(value.as_bytes().to_vec()),
    }
}

pub fn kind(name: &str) -> TargetKind {
    TargetKind::Field(Field(name.into()))
}

/// Hand-built write (forged batch), `seen` empty unless stated.
pub fn write(record: RecordId, change: Change, stamp: Stamp) -> Write {
    Write {
        record,
        change,
        stamp,
        seen: Seen::new(),
        author: Author::CliUser,
    }
}

#[cfg(feature = "sim")]
#[allow(unused_imports)]
pub use sim::*;

/// Simulated devices and causal oracle (T-7, F-2).
#[cfg(feature = "sim")]
mod sim {
    use std::collections::{BTreeMap, BTreeSet};

    use super::{dev, field, record};
    use arbitre_core::*;
    use proptest::prelude::*;

    /// Identity of a write `(target, stamp)`.
    pub type Identity = (Target, Stamp);

    /// Ground-truth ledger: for each issued write, the identities known to its author just before `write`.
    pub type Ledger = BTreeMap<Identity, BTreeSet<Identity>>;

    /// A device = a log, a time source and the ledger of its writes.
    pub struct Device {
        pub log: Log,
        pub clock: SimClock,
        pub known: Ledger,
    }

    /// Scenario action. Indices are reduced modulo the number of devices.
    #[derive(Debug, Clone)]
    pub enum Action {
        /// `record ∈ {0, 1}`, `kind ∈ {0: f1, 1: f2, 2: deletion}`, `value ∈ {0: a, 1: b}`, `delta_pt ∈ −2..=2`.
        Put {
            i: usize,
            record: u8,
            kind: u8,
            value: u8,
            delta_pt: i8,
        },
        /// Delivers to `to` the subset `selection` of the log of `from`, shuffled by `order`.
        Deliver {
            from: usize,
            to: usize,
            selection: Vec<bool>,
            order: u64,
        },
        /// Redelivers the last batch sent from `from` to `to`.
        Redeliver { from: usize, to: usize },
    }

    /// Scenario: 3 to 5 devices, initial time offset per device, sequence of actions.
    #[derive(Debug, Clone)]
    pub struct Scenario {
        pub offsets: Vec<u64>,
        pub actions: Vec<Action>,
    }

    /// E-1.1 families to cover (R-7).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    pub enum Family {
        EmptyBatch,
        Duplicate,
        ThreeDevices,
        SameL,
        SameLCTwoDevices,
        Delete,
        ConcurrentSameValue,
        SequentialOverwrite,
        CounterExampleF1,
    }

    /// Start time of each source, before offset.
    const BASE: u64 = 100;

    /// Scenario generator (E-1.1, R-7): small domains, frequent collisions.
    pub fn scenario() -> BoxedStrategy<Scenario> {
        let write = (0usize..5, 0u8..2, 0u8..3, 0u8..2, -2i8..=2).prop_map(
            |(i, record, kind, value, delta_pt)| Action::Put {
                i,
                record,
                kind,
                value,
                delta_pt,
            },
        );
        let deliver = (
            0usize..5,
            0usize..5,
            prop::collection::vec(any::<bool>(), 1..8),
            any::<u64>(),
        )
            .prop_map(|(from, to, selection, order)| Action::Deliver {
                from,
                to,
                selection,
                order,
            });
        let redeliver =
            (0usize..5, 0usize..5).prop_map(|(from, to)| Action::Redeliver { from, to });
        let action = prop_oneof![6 => write, 3 => deliver, 1 => redeliver];
        (3usize..=5)
            .prop_flat_map(move |n| {
                (
                    prop::collection::vec(0u64..4, n),
                    prop::collection::vec(action.clone(), 4..24),
                )
            })
            .prop_map(|(offsets, actions)| Scenario { offsets, actions })
            .boxed()
    }

    /// Execution of a scenario.
    pub struct Simulation {
        pub devices: Vec<Device>,
        last_sent: BTreeMap<(usize, usize), Vec<Write>>,
        events: BTreeSet<Family>,
    }

    fn identities(log: &Log) -> BTreeSet<Identity> {
        log.writes().iter().map(|w| (w.target(), w.stamp)).collect()
    }

    /// `y ∈ known` on the target of `x` with `stamp(y) ≥ stamp(x)`.
    fn knows(known: &BTreeSet<Identity>, x: &Write) -> bool {
        let target = x.target();
        known
            .range((target.clone(), x.stamp)..)
            .next()
            .is_some_and(|(c, _)| *c == target)
    }

    impl Simulation {
        pub fn new(scn: &Scenario) -> Self {
            let devices = scn
                .offsets
                .iter()
                .enumerate()
                .map(|(i, d)| Device {
                    log: Log::new(dev(i as u128 + 1)),
                    clock: SimClock::new(BASE + d),
                    known: Ledger::new(),
                })
                .collect();
            Self {
                devices,
                last_sent: BTreeMap::new(),
                events: BTreeSet::new(),
            }
        }

        /// Applies an action; returns the merge reports produced, per receiving device.
        pub fn apply(&mut self, action: &Action) -> Vec<(usize, Report)> {
            let n = self.devices.len();
            match action {
                Action::Put {
                    i,
                    record: d,
                    kind,
                    value,
                    delta_pt,
                } => {
                    let device = &mut self.devices[i % n];
                    match delta_pt {
                        1.. => device.clock.advance(*delta_pt as u64),
                        _ => device.clock.rewind(delta_pt.unsigned_abs() as u64),
                    }
                    let change = match kind {
                        2 => Change::Delete,
                        v => field(["f1", "f2"][*v as usize], ["a", "b"][*value as usize]),
                    };
                    let known = identities(&device.log);
                    let pt = device.clock.now();
                    let w = device.log.write(
                        pt,
                        record(["d0", "d1"][*d as usize]),
                        change,
                        Author::CliUser,
                    );
                    let w = w.expect("valid scenario write");
                    device.known.insert((w.target(), w.stamp), known);
                    Vec::new()
                }
                Action::Deliver {
                    from,
                    to,
                    selection,
                    order,
                } => {
                    let (from, to) = (from % n, (from % n + 1 + to % (n - 1)) % n);
                    let mut batch: Vec<Write> = self.devices[from]
                        .log
                        .writes()
                        .into_iter()
                        .enumerate()
                        .filter(|(k, _)| selection[k % selection.len()])
                        .map(|(_, w)| w)
                        .collect();
                    let mut r = *order;
                    for k in (1..batch.len()).rev() {
                        r = r
                            .wrapping_mul(6364136223846793005)
                            .wrapping_add(1442695040888963407);
                        batch.swap(k, (r >> 33) as usize % (k + 1));
                    }
                    self.last_sent.insert((from, to), batch.clone());
                    vec![self.deliver(to, &batch)]
                }
                Action::Redeliver { from, to } => {
                    let (from, to) = (from % n, (from % n + 1 + to % (n - 1)) % n);
                    let batch = self.last_sent.get(&(from, to)).cloned().unwrap_or_default();
                    vec![self.deliver(to, &batch)]
                }
            }
        }

        fn deliver(&mut self, to: usize, batch: &[Write]) -> (usize, Report) {
            let log = &mut self.devices[to].log;
            let known_writes = log.writes();
            if batch.is_empty() {
                self.events.insert(Family::EmptyBatch);
            }
            if batch.iter().any(|w| known_writes.contains(w)) {
                self.events.insert(Family::Duplicate);
            }
            (to, log.merge(batch).expect("valid simulator batch"))
        }

        /// Delivers everything to everyone; returns the reports produced.
        pub fn deliver_all(&mut self) -> Vec<(usize, Report)> {
            let n = self.devices.len();
            let mut reports = Vec::new();
            for from in 0..n {
                for to in (0..n).filter(|to| *to != from) {
                    let batch = self.devices[from].log.writes();
                    reports.push(self.deliver(to, &batch));
                }
            }
            reports
        }

        /// Union of the ledgers of all devices.
        pub fn ledger(&self) -> Ledger {
            self.devices.iter().flat_map(|a| a.known.clone()).collect()
        }

        /// E-1.1 families met since the start of the simulation.
        pub fn families(&self) -> BTreeSet<Family> {
            let mut families = self.events.clone();
            let ledger = self.ledger();
            let s: BTreeSet<Write> = self.devices.iter().flat_map(|a| a.log.writes()).collect();
            let s: Vec<Write> = s.into_iter().collect();
            if self.devices.len() >= 3 {
                families.insert(Family::ThreeDevices);
            }
            if s.iter().any(|w| matches!(w.change, Change::Delete)) {
                families.insert(Family::Delete);
            }
            for (k, w) in s.iter().enumerate() {
                for x in &s[k + 1..] {
                    if w.stamp == x.stamp {
                        continue;
                    }
                    if w.stamp.l == x.stamp.l {
                        families.insert(Family::SameL);
                        if w.stamp.c == x.stamp.c {
                            families.insert(Family::SameLCTwoDevices);
                        }
                    }
                    if !exclusive(w, x) {
                        continue;
                    }
                    let (wx, xw) = (
                        knows(&ledger[&(w.target(), w.stamp)], x),
                        knows(&ledger[&(x.target(), x.stamp)], w),
                    );
                    if wx != xw && w.change != x.change {
                        families.insert(Family::SequentialOverwrite);
                    }
                    if !wx && !xw {
                        if w.change == x.change {
                            families.insert(Family::ConcurrentSameValue);
                        }
                        let (field, deletion) = if matches!(w.change, Change::Delete) {
                            (x, w)
                        } else {
                            (w, x)
                        };
                        let autre_champ =
                            ledger[&(deletion.target(), deletion.stamp)]
                                .iter()
                                .any(|(c, m)| {
                                    c.record == field.record
                                        && c.kind != field.target().kind
                                        && *c != deletion.target()
                                        && *m > field.stamp
                                });
                        if matches!(deletion.change, Change::Delete) && autre_champ {
                            families.insert(Family::CounterExampleF1);
                        }
                    }
                }
            }
            families
        }
    }

    /// Same record, and same field or deletion against field.
    fn exclusive(w: &Write, x: &Write) -> bool {
        w.record == x.record
            && match (&w.change, &x.change) {
                (Change::Field { name: a, .. }, Change::Field { name: b, .. }) => a == b,
                (Change::Delete, Change::Delete) => false,
                _ => true,
            }
    }

    fn rank(w: &Write) -> (Stamp, Target, Option<Value>, Seen, Author) {
        let value = match &w.change {
            Change::Field { value, .. } => Some(value.clone()),
            Change::Delete => None,
        };
        (w.stamp, w.target(), value, w.seen.clone(), w.author)
    }

    /// Oracle (F-2): conflicts of `s` by brute force over all pairs, from the ledger alone.
    /// `has_seen*(w, x) ⇔ ∃ y ∈ known(w), target(y) = target(x) ∧ stamp(y) ≥ stamp(x)`; normalized and sorted (A-4).
    pub fn expected_conflicts(s: &[Write], ledger: &Ledger) -> Vec<Conflict> {
        let mut conflicts = Vec::new();
        for (k, w) in s.iter().enumerate() {
            for x in &s[k + 1..] {
                if !exclusive(w, x) || w.change == x.change {
                    continue;
                }
                let a_vu = |a: &Write, b: &Write| knows(&ledger[&(a.target(), a.stamp)], b);
                if a_vu(w, x) || a_vu(x, w) {
                    continue;
                }
                let (retained, discarded) = if rank(w) > rank(x) { (w, x) } else { (x, w) };
                let ((Change::Field { name, .. }, _) | (_, Change::Field { name, .. })) =
                    (&w.change, &x.change)
                else {
                    continue;
                };
                conflicts.push(Conflict {
                    record: w.record.clone(),
                    field: name.clone(),
                    retained: retained.clone(),
                    discarded: discarded.clone(),
                });
            }
        }
        let cle = |k: &Conflict| {
            (
                k.record.clone(),
                k.field.clone(),
                k.retained.stamp,
                k.discarded.stamp,
                rank(&k.retained),
                rank(&k.discarded),
            )
        };
        conflicts.sort_by_key(cle);
        conflicts
    }
}
