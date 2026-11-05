//! Signals left by background tasks, pure parts (ST-04 specs RG-ST04-6.1, 6.2; plan P-7).

use alloc::string::String;
use alloc::vec::Vec;

use crate::hlc::DeviceId;
use crate::merge::{RecordState, Status};
use crate::write::{Change, Field, RecordId};

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
    RecordId {
        module: "arbitre".into(),
        category: "signal".into(),
        id: format!("{}-{at}-{source}", device.hex()),
    }
}

/// Field changes of a new signal (`at`, `source`, `message`).
pub fn signal_changes(at: u64, source: &str, message: &str) -> Vec<Change> {
    vec![
        Change::field("at", format!("{at}").as_bytes()),
        Change::field("source", source.as_bytes()),
        Change::field("message", message.as_bytes()),
    ]
}

/// Change marking a signal read.
pub fn read_change() -> Change {
    Change::field("read", b"1")
}

/// Unread signals of `device` among the records of category `signal`, oldest first
/// (`(at, id)` order). Other devices' signals and malformed records are skipped.
pub fn unread(records: &[(RecordId, RecordState)], device: DeviceId) -> Vec<Signal> {
    let mine = format!("{}-", device.hex());
    let mut out: Vec<Signal> = records
        .iter()
        .filter(|(r, s)| {
            r.id.starts_with(&mine)
                && s.status == Status::Present
                && !s.fields.contains_key(&Field("read".into()))
        })
        .filter_map(|(r, s)| {
            Some(Signal {
                record: r.clone(),
                at: s.text("at")?.parse().ok()?,
                source: s.text("source")?,
                message: s.text("message")?,
            })
        })
        .collect();
    out.sort_by(|a, b| (a.at, &a.record.id).cmp(&(b.at, &b.record.id)));
    out
}

/// `signal (<YYYY-MM-DD HH:MM>): <message>`, the time in `tz`.
pub fn line(at: u64, tz: &jiff::tz::TimeZone, message: &str) -> String {
    // Beyond jiff's range (after year 9999) is clamped to its last instant.
    let ts = i64::try_from(at)
        .ok()
        .and_then(|ms| jiff::Timestamp::from_millisecond(ms).ok())
        .unwrap_or(jiff::Timestamp::MAX);
    format!(
        "signal ({}): {message}",
        tz.to_datetime(ts).strftime("%Y-%m-%d %H:%M")
    )
}
