//! Native clock: system time and system time zone (ST-04 specs RG-ST04-8.1; plan P-12).

use arbitre_core::Clock;

/// `now` from the system (or `ARBITRE_TEST_NOW`, in ms, with `test-actions`); `timezone`
/// from the system, UTC when unreadable.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> u64 {
        // T-3: `ARBITRE_TEST_NOW` under `test-actions`.
        let ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock before 1970")
            .as_millis();
        u64::try_from(ms).expect("system clock beyond u64 ms")
    }

    fn timezone(&self) -> jiff::tz::TimeZone {
        todo!("T-3")
    }
}
