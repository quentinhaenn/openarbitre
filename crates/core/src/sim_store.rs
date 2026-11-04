//! In-memory `Store` with fault injection, feature `sim` (ST-02 archi "Simulated store").
#![expect(dead_code, reason = "skeleton: bodies land in T-7")]

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::marker::PhantomData;

use crate::hlc::DeviceId;
use crate::merge::{RecordState, Report};
use crate::store::{
    ApplyError, ClockState, Damage, LocalChange, Position, Store, StoreError, Watermark,
};
use crate::write::{RecordId, Write};

/// Simulated store, deliberately neither `Send` nor `Sync` (RG-ST02-6.2).
pub struct SimStore {
    state: RefCell<State>,
    _not_send: PhantomData<*const ()>,
}

/// `Σ` plus volatile fault state.
struct State {
    device: DeviceId,
    clock: ClockState,
    writes: Vec<(Position, Write)>,
    watermarks: BTreeMap<DeviceId, Watermark>,
    /// Units left before `StorageFull`.
    budget: usize,
    unavailable: usize,
    crash_armed: bool,
    crashed: bool,
    /// Survives `reopen` (Q-P5 → (a)).
    damage: Option<Damage>,
}

impl SimStore {
    /// Empty store of `device`, `K = (0, 0)`, no budget limit.
    pub fn new(device: DeviceId) -> Self {
        let _ = device;
        todo!("T-7: SimStore::new")
    }

    /// Accepts `units` more storage units: one per stored write, one per commit
    /// (batch or watermark advance). Exceeding them → `StorageFull`, nothing applied.
    /// `usize::MAX` lifts the limit (E-ST02-2, RG-ST02-2.1, 2.2).
    pub fn full_after(&self, units: usize) {
        let _ = units;
        todo!("T-7: full_after — E-ST02-2")
    }

    /// The next `ops` operations return `Unavailable`, `Σ` unchanged (RG-ST02-5.5).
    pub fn unavailable(&self, ops: usize) {
        let _ = ops;
        todo!("T-7: unavailable — RG-ST02-5.5")
    }

    /// The next write is abandoned before its effects are visible (`Unavailable(Io)`);
    /// the handle then refuses everything until `reopen` (RG-ST02-1.3, US-6 case 8).
    pub fn crash_next_write(&self) {
        todo!("T-7: crash_next_write — RG-ST02-1.3")
    }

    /// Damages the stored state: every later operation returns `Corrupted(damage)`,
    /// across `reopen` too (RG-ST02-7.2, Q-P5 → (a)).
    pub fn corrupt(&self, damage: Damage) {
        let _ = damage;
        todo!("T-7: corrupt — RG-ST02-7.2")
    }

    /// Clears volatile state (crash, unavailability); keeps `Σ` and damage.
    pub fn reopen(&self) {
        todo!("T-7: reopen")
    }

    /// Test hook: current `K`.
    pub fn clock_state(&self) -> ClockState {
        todo!("T-7: clock_state")
    }
}

impl Store for SimStore {
    async fn device(&self) -> Result<DeviceId, StoreError> {
        todo!("T-7: device")
    }

    async fn apply_local(
        &self,
        pt: u64,
        changes: &[LocalChange],
    ) -> Result<Vec<Write>, ApplyError> {
        let _ = (pt, changes);
        todo!("T-7: apply_local — rebuild + stamp_local")
    }

    async fn apply(&self, batch: &[Write]) -> Result<Report, ApplyError> {
        let _ = batch;
        todo!("T-7: apply — rebuild + receive")
    }

    async fn writes_of(&self, record: &RecordId) -> Result<Vec<Write>, StoreError> {
        let _ = record;
        todo!("T-7: writes_of")
    }

    async fn state(&self, record: &RecordId) -> Result<Option<RecordState>, StoreError> {
        let _ = record;
        todo!("T-7: state")
    }

    async fn records(
        &self,
        module: &str,
        category: &str,
    ) -> Result<Vec<(RecordId, RecordState)>, StoreError> {
        let _ = (module, category);
        todo!("T-7: records")
    }

    async fn writes_since(&self, from: Position) -> Result<Vec<(Position, Write)>, StoreError> {
        let _ = from;
        todo!("T-7: writes_since")
    }

    async fn watermark(&self, peer: DeviceId) -> Result<Option<Watermark>, StoreError> {
        let _ = peer;
        todo!("T-7: watermark")
    }

    async fn advance_watermark(&self, peer: DeviceId, to: Watermark) -> Result<(), StoreError> {
        let _ = (peer, to);
        todo!("T-7: advance_watermark")
    }
}
