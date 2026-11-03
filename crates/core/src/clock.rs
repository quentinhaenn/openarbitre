//! `Clock` port and its simulator (ST-01 design, Interfaces; T-3).

/// Physical time source, in UTC milliseconds since the Unix epoch (RG-ST01-4.4).
pub trait Clock {
    fn now(&self) -> u64;
}

/// Controllable time source, one per simulated device (PROP-ST01-1).
///
/// Contract (`06_skeleton.md`): `now()` returns the last time set;
/// after `freeze()`, `advance` and `rewind` do nothing until the next `set`.
#[cfg(feature = "sim")]
#[derive(Debug, Default)]
pub struct SimClock {
    t: core::cell::Cell<u64>,
    frozen: core::cell::Cell<bool>,
}

#[cfg(feature = "sim")]
impl SimClock {
    /// Source set to `t`, not frozen.
    pub fn new(t: u64) -> Self {
        Self {
            t: t.into(),
            frozen: false.into(),
        }
    }

    /// Sets the time to `t` and unfreezes.
    pub fn set(&self, t: u64) {
        self.t.set(t);
        self.frozen.set(false);
    }

    /// Moves forward by `delta` ms (no-op if frozen).
    pub fn advance(&self, delta: u64) {
        if !self.frozen.get() {
            self.t.set(self.t.get().saturating_add(delta));
        }
    }

    /// Moves back by `delta` ms, floored at 0 (no-op if frozen).
    pub fn rewind(&self, delta: u64) {
        if !self.frozen.get() {
            self.t.set(self.t.get().saturating_sub(delta));
        }
    }

    /// Freezes the current time.
    pub fn freeze(&self) {
        self.frozen.set(true);
    }
}

#[cfg(feature = "sim")]
impl Clock for SimClock {
    fn now(&self) -> u64 {
        self.t.get()
    }
}
