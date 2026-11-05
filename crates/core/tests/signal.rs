//! Signals, pure parts (specs RG-ST04-6.1, 6.2; plan P-7, T-6).
mod support;

use arbitre_core::*;
use jiff::tz::TimeZone;

fn deposit(s: &SimStore, device: DeviceId, at: u64, source: &str, message: &str) -> RecordId {
    let record = signal_record(device, at, source);
    let changes: Vec<_> = signal_changes(at, source, message)
        .into_iter()
        .map(|change| LocalChange {
            record: record.clone(),
            change,
            author: Author::System,
        })
        .collect();
    ready(s.apply_local(at, &changes)).expect("deposit");
    record
}

fn signals(s: &SimStore) -> Vec<(RecordId, RecordState)> {
    ready(s.records("arbitre", "signal")).expect("records")
}

#[test]
fn record_id_names_device_time_and_source() {
    let r = signal_record(DeviceId::from_u128(0xab), 1_700, "calendar_refresh");
    assert_eq!(
        (r.module.as_str(), r.category.as_str()),
        ("arbitre", "signal")
    );
    assert_eq!(r.id, format!("{:032x}-1700-calendar_refresh", 0xab));
}

#[test]
fn unread_is_this_device_oldest_first_and_skips_read() {
    // Matrix US-6 rows 4-5.
    let (me, other) = (DeviceId::from_u128(1), DeviceId::from_u128(2));
    let s = SimStore::new(me);
    let late = deposit(&s, me, 2_000, "b", "late");
    deposit(&s, me, 1_000, "a", "early");
    deposit(&s, other, 1_500, "c", "foreign");
    let got: Vec<_> = unread(&signals(&s), me)
        .into_iter()
        .map(|g| (g.at, g.message))
        .collect();
    assert_eq!(got, [(1_000, "early".into()), (2_000, "late".to_string())]);
    let mark = LocalChange {
        record: late,
        change: read_change(),
        author: Author::System,
    };
    ready(s.apply_local(3_000, &[mark])).expect("mark");
    assert_eq!(unread(&signals(&s), me).len(), 1);
}

#[test]
fn line_uses_the_local_time() {
    // 2026-01-15T14:05:00Z.
    let at = 1_768_485_900_000;
    let ny = TimeZone::get("America/New_York").expect("bundled");
    assert_eq!(
        line(at, &ny, "token expired"),
        "signal (2026-01-15 09:05): token expired"
    );
    assert_eq!(
        line(at, &TimeZone::UTC, "x"),
        "signal (2026-01-15 14:05): x"
    );
}
