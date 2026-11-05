//! Surface registry (ST-04 specs RG-ST04-5.4; plan P-14). Frozen except `all()`: a new
//! surface adds its file and one line there (ST-07 `mcp.rs`, ST-21 `gui.rs`).

use std::collections::BTreeSet;
use std::ffi::OsString;
use std::path::PathBuf;

pub mod cli;

/// Where and how a surface runs: one store, extra variables for every process it starts.
#[derive(Debug, Clone)]
pub struct Env {
    pub store: PathBuf,
    pub vars: Vec<(OsString, OsString)>,
}

/// Outcome of one call: exit code (or `doc.error.code`, 0 on success) and result document.
#[derive(Debug, Clone)]
pub struct Called {
    pub code: u8,
    pub doc: serde_json::Value,
}

pub trait Surface {
    fn name(&self) -> &str;

    /// Action names the user can see on this surface (CLI: `--help`; MCP: `tools/list`).
    fn actions(&self, env: &Env) -> BTreeSet<String>;

    /// Calls `action` with `input` (a JSON object conforming to its schema).
    fn call(&self, env: &Env, action: &str, input: &serde_json::Value) -> Called;
}

/// Registered surfaces: one line per surface.
pub fn all() -> Vec<Box<dyn Surface>> {
    vec![Box::new(cli::Cli)]
}
