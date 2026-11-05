//! In-memory `Store` with fault injection, feature `sim` (ST-02 archi "Simulated store").

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::RefCell;
use core::marker::PhantomData;

use crate::hlc::DeviceId;
use crate::merge::{Log, RecordState, Report};
use crate::store::{
    ApplyError, Cause, ClockState, Damage, LocalChange, Position, Store, StoreError, Watermark,
    rebuild, receive, stamp_local,
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

impl State {
    /// Fault gate of every operation; `write` ops may also hit the armed crash.
    fn gate(&mut self, write: bool) -> Result<(), StoreError> {
        let io = || StoreError::Unavailable(Cause::Io(String::from("simulated crash")));
        if self.crashed {
            return Err(io());
        }
        if let Some(d) = &self.damage {
            return Err(StoreError::Corrupted(d.clone()));
        }
        if self.unavailable > 0 {
            self.unavailable -= 1;
            return Err(StoreError::Unavailable(Cause::Busy));
        }
        if write && self.crash_armed {
            self.crash_armed = false;
            self.crashed = true;
            return Err(io());
        }
        Ok(())
    }

    /// Spends `cost` units or fails with `StorageFull`, nothing spent.
    fn charge(&mut self, cost: usize) -> Result<(), StoreError> {
        if cost > self.budget {
            return Err(StoreError::StorageFull);
        }
        self.budget -= cost;
        Ok(())
    }

    /// `Log` rebuilt from the stored writes selected by `pick`; damage poisons the handle.
    fn log(&mut self, pick: impl Fn(&RecordId) -> bool) -> Result<Log, StoreError> {
        let known: Vec<Write> = self
            .writes
            .iter()
            .filter(|(_, w)| pick(&w.record))
            .map(|(_, w)| w.clone())
            .collect();
        rebuild(self.device, self.clock, &known).map_err(|d| {
            self.damage = Some(d.clone());
            StoreError::Corrupted(d)
        })
    }

    fn store(&mut self, new: Vec<Write>, clock: ClockState) {
        for w in new {
            let p = Position(self.writes.len() as u64 + 1);
            self.writes.push((p, w));
        }
        self.clock = clock;
    }
}

impl SimStore {
    /// Empty store of `device`, `K = (0, 0)`, no budget limit.
    pub fn new(device: DeviceId) -> Self {
        Self {
            state: RefCell::new(State {
                device,
                clock: ClockState::default(),
                writes: Vec::new(),
                watermarks: BTreeMap::new(),
                budget: usize::MAX,
                unavailable: 0,
                crash_armed: false,
                crashed: false,
                damage: None,
            }),
            _not_send: PhantomData,
        }
    }

    /// Accepts `units` more storage units: one per stored write, one per commit
    /// (batch or watermark advance). Exceeding them → `StorageFull`, nothing applied.
    /// `usize::MAX` lifts the limit (E-ST02-2, RG-ST02-2.1, 2.2).
    pub fn full_after(&self, units: usize) {
        self.state.borrow_mut().budget = units;
    }

    /// The next `ops` operations return `Unavailable`, `Σ` unchanged (RG-ST02-5.5).
    pub fn unavailable(&self, ops: usize) {
        self.state.borrow_mut().unavailable = ops;
    }

    /// The next write is abandoned before its effects are visible (`Unavailable(Io)`);
    /// the handle then refuses everything until `reopen` (RG-ST02-1.3, US-6 case 8).
    pub fn crash_next_write(&self) {
        self.state.borrow_mut().crash_armed = true;
    }

    /// Damages the stored state: every later operation returns `Corrupted(damage)`,
    /// across `reopen` too (RG-ST02-7.2, Q-P5 → (a)).
    pub fn corrupt(&self, damage: Damage) {
        self.state.borrow_mut().damage = Some(damage);
    }

    /// Clears volatile state (crash, unavailability); keeps `Σ` and damage.
    pub fn reopen(&self) {
        let mut st = self.state.borrow_mut();
        st.crashed = false;
        st.crash_armed = false;
        st.unavailable = 0;
    }

    /// Test hook: current `K`.
    pub fn clock_state(&self) -> ClockState {
        self.state.borrow().clock
    }
}

impl Store for SimStore {
    async fn device(&self) -> Result<DeviceId, StoreError> {
        let mut st = self.state.borrow_mut();
        st.gate(false)?;
        Ok(st.device)
    }

    async fn apply_local(
        &self,
        pt: u64,
        changes: &[LocalChange],
    ) -> Result<Vec<Write>, ApplyError> {
        let mut st = self.state.borrow_mut();
        st.gate(true)?;
        if changes.is_empty() {
            return Ok(Vec::new());
        }
        let mut log = st.log(|r| changes.iter().any(|c| c.record == *r))?;
        let (writes, clock) = stamp_local(&mut log, pt, changes).map_err(ApplyError::Rejected)?;
        st.charge(writes.len() + 1)?;
        st.store(writes.clone(), clock);
        Ok(writes)
    }

    async fn apply(&self, batch: &[Write]) -> Result<Report, ApplyError> {
        let mut st = self.state.borrow_mut();
        st.gate(true)?;
        let mut log = st.log(|r| batch.iter().any(|w| w.record == *r))?;
        let (device, k) = (st.device, st.clock);
        let got = receive(device, &mut log, k, batch).map_err(ApplyError::Rejected)?;
        // No-op writes are free on full storage, as in SQLite (Q-R1, ST-02 04_plan.md).
        if !got.new.is_empty() || got.clock != k {
            st.charge(got.new.len() + 1)?;
            st.store(got.new, got.clock);
        }
        Ok(got.report)
    }

    async fn writes_of(&self, record: &RecordId) -> Result<Vec<Write>, StoreError> {
        let mut st = self.state.borrow_mut();
        st.gate(false)?;
        Ok(st.log(|r| r == record)?.writes())
    }

    async fn state(&self, record: &RecordId) -> Result<Option<RecordState>, StoreError> {
        let mut st = self.state.borrow_mut();
        st.gate(false)?;
        Ok(st.log(|r| r == record)?.projection().remove(record))
    }

    async fn records(
        &self,
        module: &str,
        category: &str,
    ) -> Result<Vec<(RecordId, RecordState)>, StoreError> {
        let mut st = self.state.borrow_mut();
        st.gate(false)?;
        let log = st.log(|r| r.module == module && r.category == category)?;
        Ok(log.projection().into_iter().collect())
    }

    async fn writes_since(&self, from: Position) -> Result<Vec<(Position, Write)>, StoreError> {
        let mut st = self.state.borrow_mut();
        st.gate(false)?;
        st.log(|_| true)?;
        Ok(st
            .writes
            .iter()
            .filter(|(p, _)| *p > from)
            .cloned()
            .collect())
    }

    async fn watermark(&self, peer: DeviceId) -> Result<Option<Watermark>, StoreError> {
        let mut st = self.state.borrow_mut();
        st.gate(false)?;
        Ok(st.watermarks.get(&peer).copied())
    }

    async fn advance_watermark(&self, peer: DeviceId, to: Watermark) -> Result<(), StoreError> {
        let mut st = self.state.borrow_mut();
        st.gate(true)?;
        // No-op advance is free on full storage, as in SQLite (Q-R1, ST-02 04_plan.md).
        if st.watermarks.get(&peer).is_none_or(|m| to > *m) {
            st.charge(1)?;
            st.watermarks.insert(peer, to);
        }
        Ok(())
    }
}
