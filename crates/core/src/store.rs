//! `Store` port, its errors and the pure store logic shared by every adapter (ST-02 archi).

use alloc::boxed::Box;
use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;

use crate::hlc::{DeviceId, HybridClock, Stamp};
use crate::merge::{Log, RecordState, Report, WriteError};
use crate::write::{Author, BatchError, Change, RecordId, Target, Write};

/// One unstamped change of a local apply (CONFLICT-ST02-1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalChange {
    pub record: RecordId,
    pub change: Change,
    pub author: Author,
}

/// Persisted clock state `K = (l, c)` of the local device; lexicographic `Ord` (GAP-ST02-1).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ClockState {
    pub l: u64,
    pub c: u32,
}

/// Place in the store's commit order. Only the order is contractual (RG-ST02-5.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Position(pub u64);

impl Position {
    /// `⊥P`: before every stored write; stored positions start at 1.
    pub const START: Position = Position(0);
}

/// Opaque per-peer mark, lexicographic `Ord`; the store keeps the max (AMB-ST02-1, RG-ST02-5.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Watermark(pub u64, pub u32);

/// The three storage errors, a closed set (RG-ST02-6.5, AC-6.6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreError {
    StorageFull,
    /// Poisons the handle: every later operation returns it (RG-ST02-7.2).
    Corrupted(Damage),
    Unavailable(Cause),
}

/// What is damaged (RG-ST02-7.2; archi "Errors"; plan P-7, P-10, P-11).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Damage {
    /// Undecodable or fails `validate`; the adapter fills in the position.
    StoredWrite {
        position: Option<Position>,
    },
    /// Missing, undecodable, or `K` below a stored stamp on `(l, c)`.
    ClockState,
    DeviceId,
    Layout,
    NotAStore,
    /// Two contents for one identity stamped by the local device (Q-A (b)); boxed to keep errors small.
    LocalIdentityPair {
        target: Box<Target>,
        stamp: Stamp,
    },
    Engine(String),
}

/// Why the store is unavailable; retrying may help.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cause {
    Busy,
    Io(String),
    IncompatibleVersion { stored: u32, supported: u32 },
    RandomSource,
}

/// Input refused before anything is stored: `Σ` unchanged, handle not poisoned (CONFLICT-ST02-2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rejection {
    Batch(BatchError),
    Local {
        index: usize,
        error: WriteError,
    },
    /// Q-A guard: index of the first batch write of the reused local identity.
    LocalIdentityReused {
        index: usize,
    },
}

/// Error of a write operation: an input rejection or a storage error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApplyError {
    Rejected(Rejection),
    Store(StoreError),
}

impl From<StoreError> for ApplyError {
    fn from(e: StoreError) -> Self {
        Self::Store(e)
    }
}

/// Outcome of `receive`: merge report, writes to store (batch order), new `K`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Received {
    pub report: Report,
    pub new: Vec<Write>,
    pub clock: ClockState,
}

/// Storage port. `&self`, no `Send`/`Sync` bound (RG-ST02-6.2); opening is adapter-specific.
#[allow(async_fn_in_trait)]
pub trait Store {
    /// Local device identifier `D` (RG-ST02-4.1).
    async fn device(&self) -> Result<DeviceId, StoreError>;

    /// Stamps `changes` from the persisted `K` and stores them all or none (RG-ST02-1.6, 3.1, 3.2).
    /// Empty list: no-op success. Returns the stamped writes, list order.
    async fn apply_local(&self, pt: u64, changes: &[LocalChange])
    -> Result<Vec<Write>, ApplyError>;

    /// Stores a received batch all or none; `K' = max_lex(K, max batch)` (RG-ST02-1.6, 5.3, Q-A guard).
    async fn apply(&self, batch: &[Write]) -> Result<Report, ApplyError>;

    /// Writes of a record, `Log::writes` order (RG-ST02-5.1).
    async fn writes_of(&self, record: &RecordId) -> Result<Vec<Write>, StoreError>;

    /// Projected state of a record; `None` if unknown (RG-ST02-5.2).
    async fn state(&self, record: &RecordId) -> Result<Option<RecordState>, StoreError>;

    /// Records of a category, sorted by `RecordId`, deleted ones included (RG-ST02-5.2).
    async fn records(
        &self,
        module: &str,
        category: &str,
    ) -> Result<Vec<(RecordId, RecordState)>, StoreError>;

    /// Writes stored after `from`, position order, each decoded and validated (RG-ST02-5.3, 5.5).
    async fn writes_since(&self, from: Position) -> Result<Vec<(Position, Write)>, StoreError>;

    /// Mark of `peer`; `None` = nothing received (RG-ST02-5.4).
    async fn watermark(&self, peer: DeviceId) -> Result<Option<Watermark>, StoreError>;

    /// `M(peer) := max(M(peer), to)` (RG-ST02-5.4).
    async fn advance_watermark(&self, peer: DeviceId, to: Watermark) -> Result<(), StoreError>;
}

/// Log of `device` restored from `K` and the known writes of the touched records.
///
/// Errors: a known write failing `validate` → `StoredWrite { position: None }`; a known stamp
/// above `K` on `(l, c)` → `ClockState`; a local-device anomaly → `LocalIdentityPair`.
/// Refs: T-4, GAP-ST02-1, RG-ST02-1.6, RG-ST02-7.2, Q-A (b).
pub fn rebuild(device: DeviceId, k: ClockState, known: &[Write]) -> Result<Log, Damage> {
    let mut clock = HybridClock::new(device);
    clock.receive(&Stamp {
        l: k.l,
        c: k.c,
        a: device,
    });
    let mut log = Log::with_clock(clock);
    log.merge(known)
        .map_err(|_| Damage::StoredWrite { position: None })?;
    if known.iter().any(|w| {
        ClockState {
            l: w.stamp.l,
            c: w.stamp.c,
        } > k
    }) {
        return Err(Damage::ClockState);
    }
    match log.anomalies().into_iter().find(|a| a.stamp.a == device) {
        Some(a) => Err(Damage::LocalIdentityPair {
            target: Box::new(a.target),
            stamp: a.stamp,
        }),
        None => Ok(log),
    }
}

/// Stamps `changes` in list order through `Log::write` (A-1); returns the writes and `K'`.
///
/// Post: stamps strictly increasing, `a = D`, `K' = (sₙ.l, sₙ.c)`; empty list → `K'` is
/// meaningless (adapters short-circuit it).
/// Error: first faulty change → `Rejection::Local`, `log` unchanged.
/// Refs: T-4, RG-ST02-3.1, 3.2, RG-ST02-6.5.
pub fn stamp_local(
    log: &mut Log,
    pt: u64,
    changes: &[LocalChange],
) -> Result<(Vec<Write>, ClockState), Rejection> {
    let mut work = log.clone();
    let mut writes = Vec::with_capacity(changes.len());
    for (index, c) in changes.iter().enumerate() {
        let w = work
            .write(pt, c.record.clone(), c.change.clone(), c.author)
            .map_err(|error| Rejection::Local { index, error })?;
        writes.push(w);
    }
    let clock = writes
        .last()
        .map_or_else(ClockState::default, |w| ClockState {
            l: w.stamp.l,
            c: w.stamp.c,
        });
    *log = work;
    Ok((writes, clock))
}

/// Merges a received batch: `new` = unknown writes, deduplicated, batch order;
/// `clock = max_lex(k, max batch)`, also when `new` is empty.
///
/// Errors: invalid batch → `Batch`; new anomaly with `stamp.a = device` → `LocalIdentityReused`.
/// On error `log` is unchanged. Refs: T-5, RG-ST02-1.6, RG-ST02-5.3, Q-A guard.
pub fn receive(
    device: DeviceId,
    log: &mut Log,
    k: ClockState,
    batch: &[Write],
) -> Result<Received, Rejection> {
    let mut known: BTreeSet<Write> = log.writes().into_iter().collect();
    let mut work = log.clone();
    let report = work.merge(batch).map_err(Rejection::Batch)?;
    if let Some(index) = report
        .new_anomalies
        .iter()
        .filter(|a| a.stamp.a == device)
        .filter_map(|a| {
            batch
                .iter()
                .position(|w| w.target() == a.target && w.stamp == a.stamp)
        })
        .min()
    {
        return Err(Rejection::LocalIdentityReused { index });
    }
    let new = batch
        .iter()
        .filter(|w| known.insert((*w).clone()))
        .cloned()
        .collect();
    let clock = batch
        .iter()
        .map(|w| ClockState {
            l: w.stamp.l,
            c: w.stamp.c,
        })
        .fold(k, ClockState::max);
    *log = work;
    Ok(Received { report, new, clock })
}
