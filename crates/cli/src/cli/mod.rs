//! Command-line surface derived from the catalogue (ST-03 specs RG-ST03-1.1 to 1.4, 4.7; plan P-7).

use arbitre_core::value::Value;
use arbitre_core::{ActionDescriptor, ActionError, Catalog};
use clap::{ArgMatches, Command};

/// Outcome of a command before rendering.
pub type Outcome = Result<Value, ActionError>;

/// Command tree: one sub-command per path segment, sorted; long options from input
/// properties; global `--json`, `--store`; no short options (RG-ST03-4.7).
pub fn command(catalog: &Catalog) -> Command {
    let _ = catalog;
    todo!("T-6: command — RG-ST03-4.7, 1.1")
}

/// Conforming input `Value` of `action` from its parsed options (RG-ST03-4.4).
pub fn input(matches: &ArgMatches, action: &ActionDescriptor) -> Value {
    let _ = (matches, action);
    todo!("T-6: input — RG-ST03-4.4")
}

/// Help/version → success `{"help"|"version": text}`; other clap errors → `Usage` (RG-ST03-1.1, 1.4).
pub fn usage_error(e: clap::Error) -> Outcome {
    let _ = e;
    todo!("T-6: usage_error — RG-ST03-1.1, 1.4")
}
