//! Signals: leave one, show and mark the unread ones (ST-04 specs RG-ST04-6.1, 6.2; plan P-7).
//! Pure parts: `arbitre_core::signal`.

use std::io::Write;

use arbitre_core::{
    ActionError, Author, Clock, LocalChange, Store, is_snake, line, read_change, signal_changes,
    signal_record, unread,
};
use arbitre_sqlite::SqliteStore;

use crate::store::{block_on, map_apply, wait};

fn system(record: &arbitre_core::RecordId, change: arbitre_core::Change) -> LocalChange {
    LocalChange {
        record: record.clone(),
        change,
        author: Author::System,
    }
}

/// Writes one signal of this device, dated `now` (ST-11, ST-19 call it). `source` is
/// snake_case (`is_snake`); callers pass constants, so another is an `internal_error`.
#[cfg_attr(
    not(feature = "test-actions"),
    expect(dead_code, reason = "called by `test.signal`, then by ST-11 and ST-19")
)]
pub fn deposit(
    store: &SqliteStore,
    now: u64,
    source: &str,
    message: &str,
) -> Result<(), ActionError> {
    if !is_snake(source) {
        return Err(ActionError::tool_error(
            "internal_error",
            format!("signal source {source:?} is not snake_case"),
        ));
    }
    let record = signal_record(wait(store.device())?, now, source);
    let changes: Vec<_> = signal_changes(now, source, message)
        .into_iter()
        .map(|c| system(&record, c))
        .collect();
    block_on(store.apply_local(now, &changes))?.map_err(map_apply)?;
    Ok(())
}

/// Writes the unread signals of this device to `err`, oldest first, then marks them read
/// (`Author::System`). A failure writes one line to `err` and is otherwise ignored.
pub fn show_and_mark(store: &SqliteStore, clock: &dyn Clock, err: &mut dyn Write) {
    if let Err(e) = try_show_and_mark(store, clock, err) {
        let _ = writeln!(err, "error: signals not shown: {}", e.message());
    }
}

fn try_show_and_mark(
    store: &SqliteStore,
    clock: &dyn Clock,
    err: &mut dyn Write,
) -> Result<(), ActionError> {
    let device = wait(store.device())?;
    let records = wait(store.records("arbitre", "signal"))?;
    let tz = clock.timezone();
    let signals = unread(&records, device);
    for s in &signals {
        let _ = writeln!(err, "{}", line(s.at, &tz, &s.message));
    }
    let marks: Vec<_> = signals
        .iter()
        .map(|s| system(&s.record, read_change()))
        .collect();
    block_on(store.apply_local(clock.now(), &marks))?.map_err(map_apply)?;
    Ok(())
}
