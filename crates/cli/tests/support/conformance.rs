//! Helpers of the generic conformance test (ST-04 specs RG-ST04-5.1 to 5.3; plan P-15, P-16).

use std::path::Path;
use std::process::Command;
use std::time::Duration;

use super::Run;

/// Longest a command may run before the check fails as a hang (RG-ST04-4.2, 4.3).
pub const TIMEOUT: Duration = Duration::from_secs(10);

/// One module as `arbitre module list --json` gives it.
#[derive(Debug, Clone)]
pub struct ModuleInfo {
    pub id: String,
    pub enabled: bool,
    /// Descriptor objects (`name`, `summary`, `input`, `output`, `mutating`, `logged_as`,
    /// `requires_network`); empty when disabled.
    pub actions: Vec<serde_json::Value>,
}

/// Modules of the build, read from the binary on `store` (RG-ST04-5.1).
pub fn modules(home: &Path, store: &Path) -> Vec<ModuleInfo> {
    todo!("T-7 {} {}", home.display(), store.display())
}

/// Minimal conforming input of an input JSON Schema (RG-ST04-5.3).
pub fn minimal_input(schema: &serde_json::Value) -> serde_json::Value {
    todo!("T-7 {schema}")
}

/// Runs `c`; `None` if it outlives `TIMEOUT` (then killed and reaped, DA F-12).
pub fn run_timed(c: &mut Command) -> Option<Run> {
    todo!("T-7 {c:?}")
}

/// Position of the last write in `store` (0 if none): check 4 compares what comes after.
pub fn last_position(store: &Path) -> u64 {
    todo!("T-7 {}", store.display())
}

/// Modules of the writes after `after` in `store`.
pub fn modules_written_after(store: &Path, after: u64) -> Vec<String> {
    todo!("T-7 {} {after}", store.display())
}
