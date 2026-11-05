//! The command line as a surface: spawns the binary (ST-04 plan P-14).

use std::collections::BTreeSet;
use std::process::Command;

use serde_json::Value as Json;

use super::{Called, Env, Surface};
use crate::support::conformance::{modules, run_timed};
use crate::support::{cmd, doc};

pub struct Cli;

impl Surface for Cli {
    fn name(&self) -> &str {
        "cli"
    }

    /// Walks `arbitre --help`, then each group's `--help`, recursively.
    fn actions(&self, env: &Env) -> BTreeSet<String> {
        let ids: BTreeSet<_> = modules(home(env), &env.store)
            .into_iter()
            .map(|m| m.id)
            .collect();
        let mut found = BTreeSet::new();
        walk(env, &mut Vec::new(), &mut found);
        found
            .into_iter()
            .map(|path| {
                let name = path.join(".");
                if ids.contains(&path[0]) {
                    name
                } else {
                    format!("arbitre.{name}")
                }
            })
            .collect()
    }

    /// Input becomes options by the RG-ST03-4.3 table, with `--json --store <env.store>`.
    fn call(&self, env: &Env, action: &str, input: &serde_json::Value) -> Called {
        let mut args: Vec<String> = action
            .strip_prefix("arbitre.")
            .unwrap_or(action)
            .split('.')
            .map(String::from)
            .collect();
        for (k, v) in input.as_object().expect("input object") {
            let opt = format!("--{}", k.replace('_', "-"));
            match v {
                Json::Bool(on) => args.extend(on.then_some(opt)),
                Json::Array(items) => {
                    for i in items {
                        args.extend([opt.clone(), text(i)]);
                    }
                }
                v => args.extend([opt, text(v)]),
            }
        }
        args.push("--json".into());
        let r = run_timed(&mut spawn(env, &args)).expect("no hang");
        Called {
            code: u8::try_from(r.code).expect("exit code"),
            doc: doc(&r),
        }
    }
}

/// The binary on `env.store` with `env.vars`; `HOME` is the store's directory.
fn spawn(env: &Env, args: &[String]) -> Command {
    let store = env.store.display().to_string();
    let mut v: Vec<&str> = args.iter().map(String::as_str).collect();
    v.extend(["--store", &store]);
    let mut c = cmd(home(env), &v);
    c.envs(env.vars.iter().cloned());
    c
}

fn text(v: &Json) -> String {
    v.as_str().map_or_else(|| v.to_string(), String::from)
}

fn home(env: &Env) -> &std::path::Path {
    env.store.parent().expect("store in a directory")
}

/// Adds to `found` the command paths under `path` that run an action.
fn walk(env: &Env, path: &mut Vec<String>, found: &mut BTreeSet<Vec<String>>) {
    let mut args = path.clone();
    args.push("--help".into());
    let r = run_timed(&mut spawn(env, &args)).expect("no hang");
    assert_eq!(r.code, 0, "{args:?}: {}", r.stdout);
    let (usage, subs) = sections(&r.stdout);
    // A pure group's usage ends in `<COMMAND>`; a line that does not is a runnable action.
    if !path.is_empty() && usage.iter().any(|l| !l.ends_with("<COMMAND>")) {
        found.insert(path.clone());
    }
    for s in subs {
        path.push(s);
        walk(env, path, found);
        path.pop();
    }
}

/// Usage lines and sub-command names of a clap help text.
fn sections(help: &str) -> (Vec<&str>, Vec<String>) {
    let block = |title: &str| -> Vec<&str> {
        help.split_once(title)
            .map(|(_, rest)| rest.split("\n\n").next().unwrap_or("").lines().collect())
            .unwrap_or_default()
    };
    let usage = block("Usage:").into_iter().map(str::trim).collect();
    let subs = block("Commands:\n")
        .into_iter()
        .filter_map(|l| l.split_whitespace().next().map(String::from))
        .collect();
    (usage, subs)
}
