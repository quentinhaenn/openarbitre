//! Signals left by background tasks, pure parts (ST-04 specs RG-ST04-6.1, 6.2; plan P-7).

use alloc::string::String;
use alloc::vec::Vec;

use crate::hlc::DeviceId;
use crate::merge::RecordState;
use crate::write::{Change, RecordId};

/// One signal, as read back from its record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signal {
    pub record: RecordId,
    /// Physical time, ms since the Unix epoch.
    pub at: u64,
    pub source: String,
    pub message: String,
}

/// `arbitre/signal/<device hex>-<at>-<source>`.
pub fn signal_record(device: DeviceId, at: u64, source: &str) -> RecordId {
    todo!("T-6 {device:?} {at} {source}")
}

/// Field changes of a new signal (`at`, `source`, `message`).
pub fn signal_changes(at: u64, source: &str, message: &str) -> Vec<Change> {
    todo!("T-6 {at} {source} {message}")
}

/// Change marking a signal read.
pub fn read_change() -> Change {
    todo!("T-6")
}

/// Unread signals of `device` among the records of category `signal`, oldest first
/// (`(at, id)` order). Other devices' signals and malformed records are skipped.
pub fn unread(records: &[(RecordId, RecordState)], device: DeviceId) -> Vec<Signal> {
    todo!("T-6 {} {device:?}", records.len())
}

/// `signal (<YYYY-MM-DD HH:MM>): <message>`, the time in `tz`.
pub fn line(at: u64, tz: &jiff::tz::TimeZone, message: &str) -> String {
    todo!("T-6 {at} {tz:?} {message}")
}
