//! The command line as a surface: spawns the binary (ST-04 plan P-14).

use std::collections::BTreeSet;

use super::{Called, Env, Surface};

pub struct Cli;

impl Surface for Cli {
    fn name(&self) -> &str {
        "cli"
    }

    /// Walks `arbitre --help`, then each group's `--help`, recursively.
    fn actions(&self, env: &Env) -> BTreeSet<String> {
        todo!("T-7 {env:?}")
    }

    /// Input becomes options by the RG-ST03-4.3 table, with `--json --store <env.store>`.
    fn call(&self, env: &Env, action: &str, input: &serde_json::Value) -> Called {
        todo!("T-7 {env:?} {action} {input}")
    }
}
