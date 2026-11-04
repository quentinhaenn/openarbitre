//! Log, local write, merge, projection, discarded, conflicts, anomalies
//! (ST-01 design; amendments A-1 to A-4, A-10; T-5, T-6).

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec::Vec;
use core::ops::Bound;

use crate::hlc::{ClockExhausted, DeviceId, HybridClock, Stamp};
use crate::write::{
    Author, BatchError, Change, Field, Reason, RecordId, Seen, Target, TargetKind, Value, Write,
    identifiers_non_empty, validate,
};

const STAMP_MIN: Stamp = Stamp {
    l: 0,
    c: 0,
    a: DeviceId([0; 16]),
};

/// Rank of a write `(stamp, target, content)` (A-3).
fn rank(w: &Write) -> (Stamp, Target, Option<&Value>, &Seen, Author) {
    let value = match &w.change {
        Change::Field { value, .. } => Some(value),
        Change::Delete => None,
    };
    (w.stamp, w.target(), value, &w.seen, w.author)
}

/// Sort key of conflicts (A-4).
fn conflict_key(k: &Conflict) -> impl Ord + '_ {
    (
        &k.record,
        &k.field,
        k.retained.stamp,
        k.discarded.stamp,
        rank(&k.retained),
        rank(&k.discarded),
    )
}

fn to_write(target: &Target, stamp: &Stamp, content: &Content) -> Write {
    let change = match (&target.kind, &content.value) {
        (TargetKind::Field(name), Some(value)) => Change::Field {
            name: name.clone(),
            value: value.clone(),
        },
        _ => Change::Delete,
    };
    Write {
        record: target.record.clone(),
        change,
        stamp: *stamp,
        seen: content.seen.clone(),
        author: content.author,
    }
}

/// Elements of `after` missing from `before`, `after` order kept.
fn only_new<T: Ord>(before: Vec<T>, after: Vec<T>) -> Vec<T> {
    let before: BTreeSet<T> = before.into_iter().collect();
    after.into_iter().filter(|x| !before.contains(x)).collect()
}

/// `w` is retained in the projected state of its record.
fn retained(state: &RecordState, w: &Write) -> bool {
    match &w.change {
        Change::Field { name, .. } => state.fields.get(name) == Some(w),
        Change::Delete => matches!(&state.status, Status::Deleted(s) if s == w),
    }
}

/// Content of an identity `(target, stamp)`: `(value?, seen, author)` (A-3).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Content {
    pub value: Option<Value>,
    pub seen: Seen,
    pub author: Author,
}

/// Rejection of a local write (A-2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteError {
    ClockExhausted,
    Invalid(Reason),
}

impl From<ClockExhausted> for WriteError {
    fn from(_: ClockExhausted) -> Self {
        Self::ClockExhausted
    }
}

/// Status of a projected record; a deleted record carries its retained deletion (RG-2.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Status {
    Present,
    Deleted(Write),
}

/// Projected state of a record. Winners stay present, hidden, if it is deleted (AMB-ST01-1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordState {
    pub status: Status,
    pub fields: BTreeMap<Field, Write>,
}

/// Projected state `R(S)`, state part.
pub type Projection = BTreeMap<RecordId, RecordState>;

/// Conflicting pair (RG-ST01-2.1). `field` is the one of the field write.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Conflict {
    pub record: RecordId,
    pub field: Field,
    pub retained: Write,
    pub discarded: Write,
}

/// Identity carried by at least two contents (MANQUE-ST01-2, A-3).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Anomaly {
    pub target: Target,
    pub stamp: Stamp,
    pub contents: BTreeSet<Content>,
}

/// Incremental outputs of a merge: differences after \ before, sorted (RG-2.2, A-4, A-10).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Report {
    pub new_discarded: Vec<Write>,
    pub new_conflicts: Vec<Conflict>,
    pub new_anomalies: Vec<Anomaly>,
}

/// Known log `S` of a device and its clock (A-2, A-3). `S` grows by union; nothing leaves it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Log {
    clock: HybridClock,
    writes: BTreeMap<(Target, Stamp), BTreeSet<Content>>,
}

impl Log {
    /// Empty log of device `device`.
    pub fn new(device: DeviceId) -> Self {
        Self {
            clock: HybridClock::new(device),
            writes: BTreeMap::new(),
        }
    }

    /// Empty log driven by a restored clock (GAP-ST02-1, used by `store::rebuild`).
    pub(crate) fn with_clock(clock: HybridClock) -> Self {
        Self {
            clock,
            writes: BTreeMap::new(),
        }
    }

    /// Only path of a local write (A-2).
    ///
    /// Pre: none. Validates identifiers **before** issuing (a rejection consumes no stamp).
    /// Post: `seen` = `seen_for` (A-1), write inserted and returned for sending.
    pub fn write(
        &mut self,
        pt: u64,
        record: RecordId,
        change: Change,
        author: Author,
    ) -> Result<Write, WriteError> {
        if !identifiers_non_empty(&record, &change) {
            return Err(WriteError::Invalid(Reason::EmptyIdentifier));
        }
        let stamp = self.clock.issue(pt)?;
        let seen = self.seen_for(&record, &change);
        let w = Write {
            record,
            change,
            stamp,
            seen,
            author,
        };
        self.insert_write(&w);
        Ok(w)
    }

    /// Merges a received batch, all or nothing (RG-ST01-4.5, INV-ST01-7).
    ///
    /// Invalid batch ⇒ `Err`, log **and** clock unchanged. Otherwise inserts, then the clock
    /// adopts the batch's largest stamp (RG-ST01-4.3). Report = differences (A-10).
    /// Accepts a hand-built batch (`seen` not verifiable, Q-6).
    pub fn merge(&mut self, batch: &[Write]) -> Result<Report, BatchError> {
        validate(batch)?;
        // ponytail: recomputed per touched record, quadratic on a stream of isolated writes (see ST-19)
        let touched: BTreeSet<&RecordId> = batch.iter().map(|w| &w.record).collect();
        let before: Vec<_> = touched.iter().map(|d| self.outputs(d)).collect();
        for w in batch {
            self.insert_write(w);
        }
        if let Some(m) = batch.iter().map(|w| w.stamp).max() {
            self.clock.receive(&m);
        }
        let mut report = Report::default();
        for (d, (discarded, conflicts, anomalies)) in touched.iter().zip(before) {
            let (discarded_after, conflicts_after, anomalies_after) = self.outputs(d);
            report
                .new_discarded
                .extend(only_new(discarded, discarded_after));
            report
                .new_conflicts
                .extend(only_new(conflicts, conflicts_after));
            report
                .new_anomalies
                .extend(only_new(anomalies, anomalies_after));
        }
        Ok(report)
    }

    /// All known writes, `S`, in `(target, stamp, content)` order.
    pub fn writes(&self) -> Vec<Write> {
        self.writes
            .iter()
            .flat_map(|((c, m), ks)| ks.iter().map(|k| to_write(c, m, k)))
            .collect()
    }

    /// Projected state (RG-2.1, RG-2.3): `deleted ⇔ t_s > t_f`; winner = max rank `(stamp, target, content)`.
    pub fn projection(&self) -> Projection {
        self.records()
            .into_iter()
            .map(|d| (d.clone(), self.project(&d)))
            .collect()
    }

    /// `discarded(S) = S \ retained(S)`, in `(target, stamp, content)` order (RG-2.4, INV-7, A-4).
    pub fn discarded(&self) -> Vec<Write> {
        self.records()
            .iter()
            .flat_map(|d| self.discarded_of(d))
            .collect()
    }

    /// Identities with several contents, sorted by identity (A-3).
    pub fn anomalies(&self) -> Vec<Anomaly> {
        self.records()
            .iter()
            .flat_map(|d| self.anomalies_of(d))
            .collect()
    }

    /// `conflicts(S)` (RG-ST01-2.1), by interval (A-10), sorted and deduplicated (A-4).
    pub fn conflicts(&self) -> Vec<Conflict> {
        self.records()
            .iter()
            .flat_map(|d| self.conflicts_of(d))
            .collect()
    }

    /// `seen` table of a new local write (A-1): field `f` ⇒ entries `Field(f)` and
    /// `Delete`; deletion ⇒ one entry per target kind known for the record. Largest known stamp.
    fn seen_for(&self, record: &RecordId, change: &Change) -> Seen {
        let mut seen = Seen::new();
        for ((c, m), _) in self.of(record) {
            let relevant = match change {
                Change::Delete => true,
                Change::Field { name, .. } => {
                    c.kind == TargetKind::Delete || c.kind == TargetKind::Field(name.clone())
                }
            };
            if relevant {
                seen.insert(c.kind.clone(), *m);
            }
        }
        seen
    }

    fn insert_write(&mut self, w: &Write) {
        let value = match &w.change {
            Change::Field { value, .. } => Some(value.clone()),
            Change::Delete => None,
        };
        let content = Content {
            value,
            seen: w.seen.clone(),
            author: w.author,
        };
        self.writes
            .entry((w.target(), w.stamp))
            .or_default()
            .insert(content);
    }

    /// Entries of a record, in `(target, stamp)` order.
    fn of(&self, d: &RecordId) -> impl Iterator<Item = (&(Target, Stamp), &BTreeSet<Content>)> {
        let start = (
            Target {
                record: d.clone(),
                kind: TargetKind::Delete,
            },
            STAMP_MIN,
        );
        self.writes
            .range(start..)
            .take_while(move |((c, _), _)| c.record == *d)
    }

    fn records(&self) -> Vec<RecordId> {
        let mut v: Vec<RecordId> = Vec::new();
        for (c, _) in self.writes.keys() {
            if v.last() != Some(&c.record) {
                v.push(c.record.clone());
            }
        }
        v
    }

    fn outputs(&self, d: &RecordId) -> (Vec<Write>, Vec<Conflict>, Vec<Anomaly>) {
        (
            self.discarded_of(d),
            self.conflicts_of(d),
            self.anomalies_of(d),
        )
    }

    fn project(&self, d: &RecordId) -> RecordState {
        let mut fields = BTreeMap::new();
        let mut deletion: Option<Write> = None;
        // `(target, stamp, content)` order puts the winner last
        for ((c, m), ks) in self.of(d) {
            for k in ks {
                let w = to_write(c, m, k);
                match &c.kind {
                    TargetKind::Field(name) => {
                        fields.insert(name.clone(), w);
                    }
                    TargetKind::Delete => deletion = Some(w),
                }
            }
        }
        let t_f = fields.values().map(|w| w.stamp).max();
        let status = match deletion {
            Some(s) if Some(s.stamp) > t_f => Status::Deleted(s),
            _ => Status::Present,
        };
        RecordState { status, fields }
    }

    fn discarded_of(&self, d: &RecordId) -> Vec<Write> {
        let state = self.project(d);
        let all = self
            .of(d)
            .flat_map(|((c, m), ks)| ks.iter().map(|k| to_write(c, m, k)));
        all.filter(|w| !retained(&state, w)).collect()
    }

    fn anomalies_of(&self, d: &RecordId) -> Vec<Anomaly> {
        let collisions = self.of(d).filter(|(_, ks)| ks.len() > 1);
        collisions
            .map(|((c, m), ks)| Anomaly {
                target: c.clone(),
                stamp: *m,
                contents: ks.clone(),
            })
            .collect()
    }

    fn conflicts_of(&self, d: &RecordId) -> Vec<Conflict> {
        let mut field_targets: BTreeSet<Target> = BTreeSet::new();
        for ((c, _), _) in self.of(d) {
            if matches!(c.kind, TargetKind::Field(_)) {
                field_targets.insert(c.clone());
            }
        }
        let deletion = Target {
            record: d.clone(),
            kind: TargetKind::Delete,
        };
        let mut conflicts = Vec::new();
        for ((cw, mw), contents) in self.of(d) {
            for kw in contents {
                let w = to_write(cw, mw, kw);
                let own;
                let targets: &mut dyn Iterator<Item = &Target> = match cw.kind {
                    TargetKind::Delete => &mut field_targets.iter(),
                    TargetKind::Field(_) => {
                        own = [cw.clone(), deletion.clone()];
                        &mut own.iter()
                    }
                };
                for c in targets {
                    // interval `]seen(w)[c], stamp(w)]`: writes on `c` that the author of `w` had not seen
                    let low = match kw.seen.get(&c.kind) {
                        Some(m) => Bound::Excluded((c.clone(), *m)),
                        None => Bound::Included((c.clone(), STAMP_MIN)),
                    };
                    for ((cx, mx), ksx) in
                        self.writes.range((low, Bound::Included((c.clone(), *mw))))
                    {
                        for kx in ksx {
                            let x = to_write(cx, mx, kx);
                            let has_seen = |w: &Write, x: &Write| {
                                w.seen.get(&x.target().kind).is_some_and(|m| *m >= x.stamp)
                            };
                            if x == w || kx.value == kw.value || has_seen(&x, &w) {
                                continue;
                            }
                            let ((TargetKind::Field(name), _) | (_, TargetKind::Field(name))) =
                                (&cw.kind, &c.kind)
                            else {
                                continue;
                            };
                            let field = name.clone();
                            let (retained, discarded) = if rank(&w) > rank(&x) {
                                (w.clone(), x)
                            } else {
                                (x, w.clone())
                            };
                            conflicts.push(Conflict {
                                record: d.clone(),
                                field,
                                retained,
                                discarded,
                            });
                        }
                    }
                }
            }
        }
        conflicts.sort_by(|a, b| conflict_key(a).cmp(&conflict_key(b)));
        conflicts.dedup();
        conflicts
    }
}
