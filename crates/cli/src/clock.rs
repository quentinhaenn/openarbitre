//! Native clock: system time and system time zone (ST-04 specs RG-ST04-8.1; plan P-12).

use arbitre_core::Clock;

/// `now` from the system (or `ARBITRE_TEST_NOW`, in ms, with `test-actions`); `timezone`
/// from the system, UTC when unreadable.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> u64 {
        #[cfg(feature = "test-actions")]
        if let Ok(t) = std::env::var("ARBITRE_TEST_NOW") {
            return t.parse().expect("ARBITRE_TEST_NOW: integer ms");
        }
        let ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock before 1970")
            .as_millis();
        u64::try_from(ms).expect("system clock beyond u64 ms")
    }

    fn timezone(&self) -> jiff::tz::TimeZone {
        jiff::tz::TimeZone::try_system().unwrap_or(jiff::tz::TimeZone::UTC)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn now_is_wall_time_without_override() {
        // Assumes `ARBITRE_TEST_NOW` unset in the test environment; 1_767_225_600_000 = 2026-01-01.
        assert!(SystemClock.now() > 1_767_225_600_000);
    }
}
