//! Signals: leave one, show and mark the unread ones (ST-04 specs RG-ST04-6.1, 6.2; plan P-7).
//! Pure parts: `arbitre_core::signal`.

use std::io::Write;

use arbitre_core::{ActionError, Clock};
use arbitre_sqlite::SqliteStore;

/// Writes one signal of this device, dated `now` (ST-11, ST-19 call it).
#[expect(
    dead_code,
    reason = "T-6: called by `test.signal`; then cfg_attr on `test-actions`"
)]
pub fn deposit(
    store: &SqliteStore,
    now: u64,
    source: &str,
    message: &str,
) -> Result<(), ActionError> {
    todo!("T-6 {:p} {now} {source} {message}", store)
}

/// Writes the unread signals of this device to `err`, oldest first, then marks them read
/// (`Author::System`). A failure writes one line to `err` and is otherwise ignored.
#[expect(dead_code, reason = "called from main.rs by T-6")]
pub fn show_and_mark(store: &SqliteStore, clock: &dyn Clock, err: &mut dyn Write) {
    todo!("T-6 {:p} {} {:p}", store, clock.now(), err)
}
