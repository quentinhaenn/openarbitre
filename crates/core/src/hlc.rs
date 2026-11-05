//! Stamps, hybrid logical clock, leads (ST-01 design, core types, A-11; T-3).

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::num::NonZeroU64;

use crate::write::Write;

/// Exclusive upper bound of `Stamp::l` (A-11).
pub const L_MAX: u64 = 1 << 53;

/// Device identifier, ordered byte by byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeviceId(pub [u8; 16]);

impl DeviceId {
    /// Builds the identifier from an integer; integer order is preserved.
    pub fn from_u128(n: u128) -> Self {
        Self(n.to_be_bytes())
    }

    /// 32 lowercase hex digits, byte order (record ids, P-6, P-7).
    pub fn hex(&self) -> String {
        self.0.iter().map(|b| format!("{b:02x}")).collect()
    }
}

/// Stamp `(l, c, a)`. Field order is contractual: lexicographic `Ord` (RG-ST01-4.1, A-4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Stamp {
    pub l: u64,
    pub c: u32,
    pub a: DeviceId,
}

/// No stamp can be issued any more: `c` saturated or `l' ≥ L_MAX` (MANQUE-ST01-3, A-11).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClockExhausted;

/// State `(l, c)` of a device's clock.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HybridClock {
    l: u64,
    c: u32,
    a: DeviceId,
}

impl HybridClock {
    /// Clock of device `a`, state `(0, 0)`.
    pub fn new(a: DeviceId) -> Self {
        Self { l: 0, c: 0, a }
    }

    /// Issues a stamp (RG-ST01-4.2): `l' = max(l, pt)`, `c' = c + 1` if `l' = l`, else 0.
    ///
    /// Post: stamp > everything issued or received; `l' ≥ pt` (INV-ST01-5).
    /// Error: `c` saturated or `l' ≥ L_MAX` ⇒ `ClockExhausted`, state unchanged (A-11).
    pub fn issue(&mut self, pt: u64) -> Result<Stamp, ClockExhausted> {
        let l = self.l.max(pt);
        let c = if l == self.l {
            self.c.checked_add(1).ok_or(ClockExhausted)?
        } else {
            0
        };
        if l >= L_MAX {
            return Err(ClockExhausted);
        }
        (self.l, self.c) = (l, c);
        Ok(Stamp { l, c, a: self.a })
    }

    /// Adopts `max_lex((l, c), (m.l, m.c))` (RG-ST01-4.3). Post: every later issue > `m` (INV-ST01-6).
    pub fn receive(&mut self, m: &Stamp) {
        (self.l, self.c) = (self.l, self.c).max((m.l, m.c));
    }
}

/// Lead threshold `σ` in ms; `σ = 0` is not representable (RG-ST01-4.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Threshold(pub NonZeroU64);

/// 60 min (RG-ST01-4.6).
pub const DEFAULT_THRESHOLD: Threshold = Threshold(NonZeroU64::new(3_600_000).unwrap());

/// Largest lead of a device within a batch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Lead {
    pub device: DeviceId,
    pub lead_ms: u64,
}

/// Devices with a write in the batch more than `threshold` ahead of `now_ms` (RG-ST01-4.6).
///
/// Post: one entry per device, the batch's largest lead, sorted by device;
/// a lead equal to the threshold is not reported (INV-ST01-8). Pure.
pub fn leads(batch: &[Write], now_ms: u64, threshold: Threshold) -> Vec<Lead> {
    let mut max_lead: BTreeMap<DeviceId, u64> = BTreeMap::new();
    for w in batch {
        let lead = max_lead.entry(w.device()).or_default();
        *lead = (*lead).max(w.stamp.l.saturating_sub(now_ms));
    }
    max_lead
        .into_iter()
        .filter(|&(_, lead_ms)| lead_ms > threshold.0.get())
        .map(|(device, lead_ms)| Lead { device, lead_ms })
        .collect()
}
