//! `Clock::timezone` and the bundled database (E-ST04-8; specs RG-ST04-8.1, 8.2; plan T-3).
use arbitre_core::{Clock, SimClock};
use jiff::Timestamp;
use jiff::tz::TimeZone;

fn offset(tz: &TimeZone, at: &str) -> i32 {
    tz.to_offset(at.parse::<Timestamp>().expect("timestamp"))
        .seconds()
}

#[test]
fn named_zone_resolves_from_the_bundled_database() {
    // Matrix US-8 row 1: the 2026 change in Paris is at 01:00 UTC on 29 March.
    let paris = TimeZone::get("Europe/Paris").expect("bundled");
    assert_eq!(offset(&paris, "2026-03-29T00:30:00Z"), 3_600);
    assert_eq!(offset(&paris, "2026-03-29T01:30:00Z"), 7_200);
}

#[test]
fn sim_clock_is_utc_until_set() {
    // Matrix US-8 row 2.
    let c = SimClock::new(0);
    assert_eq!(c.timezone(), TimeZone::UTC);
    let ny = TimeZone::get("America/New_York").expect("bundled");
    c.set_timezone(ny.clone());
    assert_eq!(c.timezone(), ny);
    assert_eq!(c.now(), 0);
}
