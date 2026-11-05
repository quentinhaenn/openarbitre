mod cli;
mod clock;
mod modules;
mod signals;
mod store;

use std::ffi::OsString;
use std::io::Write;
use std::path::PathBuf;

use arbitre_core::{
    ActionDescriptor, ActionError, Author, Category, Context, Module, ModuleStore, conforms_output,
    document, encode, render,
};
use arbitre_sqlite::SqliteStore;
use clap::ArgMatches;

use cli::Outcome;
use clock::SystemClock;
use modules::Modules;
use modules::foundation::{self, Env};

/// What to write and how to exit.
struct Done {
    out: String,
    err: String,
    code: u8,
}

/// `--json` among the raw arguments, before any `--` (RG-ST03-2.1).
fn json_mode(args: &[OsString]) -> bool {
    args.iter()
        .skip(1)
        .take_while(|a| *a != "--")
        .any(|a| a == "--json")
}

fn finish(r: Outcome, json: bool) -> Done {
    let code = r.as_ref().map_or_else(|e| e.category().exit_code(), |_| 0);
    let (out, err) = match (&r, json) {
        (_, true) => (encode(&document(&r)) + "\n", String::new()),
        (Ok(v), false) => (render(v), String::new()),
        (Err(e), false) => {
            let details = e.data().map(render).unwrap_or_default();
            (String::new(), format!("error: {}\n{details}", e.message()))
        }
    };
    Done { out, err, code }
}

/// Output checked against its schema; `Usage` from an action is a contract breach (RG-ST03-3.4, 4.5).
fn settle(action: &ActionDescriptor, r: Outcome) -> Outcome {
    match r {
        Ok(v) => conforms_output(&action.output, &v)
            .map(|()| v)
            .map_err(|m| ActionError::tool_error("output_mismatch", m)),
        Err(e) if e.category() == Category::Usage => Err(ActionError::tool_error(
            "internal_error",
            format!("action returned a usage error: {}", e.message()),
        )),
        r => r,
    }
}

/// The toggleable module owning `action`; `None` for the foundation.
fn module_of<'a>(modules: &'a Modules, action: &str) -> Option<&'a dyn Module<SqliteStore>> {
    let first = action.split('.').next();
    modules
        .iter()
        .find(|m| first == Some(m.id()))
        .map(|m| m.as_ref())
}

fn execute(action: &ActionDescriptor, leaf: &ArgMatches, modules: &Modules) -> Outcome {
    let flag = leaf.get_one::<PathBuf>("store");
    let path = store::location(
        flag.map(|p| p.as_os_str()),
        |k| std::env::var_os(k),
        std::env::consts::OS,
    )?;
    let store = store::open(&path)?;
    let clock = SystemClock;
    let author = Author::CliUser;
    let input = cli::input(leaf, action);
    let r = match module_of(modules, &action.name) {
        None => {
            let env = Env {
                store: &store,
                clock: &clock,
                author,
                modules,
            };
            foundation::execute(&action.name, input, &env)
        }
        Some(m) => {
            let ctx = Context {
                author,
                clock: &clock,
                store: ModuleStore::new(&store, m.id(), author, &clock),
                ai: None,
            };
            store::block_on(m.execute(&action.name, input, &ctx)).and_then(|r| r)
        }
    };
    settle(action, r)
}

/// Everything from catalogue to result; the store is dropped on return, before any write.
fn run(args: &[OsString], json: bool) -> Done {
    let modules = modules::all();
    let catalog = match modules::catalog(&modules) {
        Ok(c) => c,
        Err(e) => {
            return finish(
                Err(ActionError::tool_error("catalog_invalid", e.message)),
                json,
            );
        }
    };
    let matches = match cli::command(&catalog).try_get_matches_from(args) {
        Ok(m) => m,
        // Human help/version: the text itself, not a `help: …` rendering.
        Err(e) if !json && !e.use_stderr() => {
            return Done {
                out: e.to_string(),
                err: String::new(),
                code: 0,
            };
        }
        Err(e) => return finish(cli::usage_error(e), json),
    };
    let (action, leaf) = cli::selected(&matches, &catalog);
    finish(execute(action, leaf, &modules), json)
}

fn main() {
    let args: Vec<OsString> = std::env::args_os().collect();
    let json = json_mode(&args);
    let done = std::panic::catch_unwind(|| run(&args, json)).unwrap_or_else(|_| {
        let crash = ActionError::tool_error("internal_error", "crash; details on standard error");
        finish(Err(crash), json)
    });
    // One checked write: a closed stdout is exit 3, never a panic (101) nor a silent 0 (F-5).
    let mut out = std::io::stdout().lock();
    let written = out
        .write_all(done.out.as_bytes())
        .and_then(|()| out.flush());
    let mut err = std::io::stderr().lock();
    let _ = err.write_all(done.err.as_bytes());
    let code = match written {
        Ok(()) => done.code,
        Err(e) => {
            let _ = writeln!(err, "error: output_failed: cannot write the result: {e}");
            3
        }
    };
    std::process::exit(code.into());
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use arbitre_core::Value;
    use arbitre_core::{LoggedAs, Schema};

    use super::*;

    fn descriptor() -> ActionDescriptor {
        ActionDescriptor {
            name: "x.y".into(),
            summary: "s".into(),
            input: Schema::Object(vec![]),
            output: Schema::Object(vec![]),
            mutating: false,
            logged_as: LoggedAs::None,
            requires_network: BTreeSet::new(),
        }
    }

    fn os(a: &[&str]) -> Vec<OsString> {
        a.iter().map(OsString::from).collect()
    }

    #[test]
    fn json_mode_stops_at_terminator_and_skips_program() {
        assert!(json_mode(&os(&["a", "x", "--json"])));
        assert!(!json_mode(&os(&["a", "x", "--", "--json"])));
        assert!(!json_mode(&os(&["--json"])));
        assert!(!json_mode(&os(&["a", "x"])));
    }

    #[test]
    fn settle_checks_output_and_usage() {
        let ok = Ok(Value::Object([].into()));
        assert_eq!(settle(&descriptor(), ok.clone()), ok);
        let bad = settle(&descriptor(), Ok(Value::Int(1))).unwrap_err();
        assert_eq!(bad.id(), "output_mismatch");
        let usage = settle(&descriptor(), Err(ActionError::usage("u", "m"))).unwrap_err();
        assert_eq!(
            (usage.category(), usage.id()),
            (Category::ToolError, "internal_error")
        );
        let refusal = ActionError::refusal("r", "m");
        assert_eq!(settle(&descriptor(), Err(refusal.clone())), Err(refusal));
    }

    #[test]
    fn finish_codes_and_streams() {
        let d = finish(Err(ActionError::refusal("r", "no")), false);
        assert_eq!(
            (d.code, d.out.as_str(), d.err.as_str()),
            (1, "", "error: no\n")
        );
        let d = finish(Err(ActionError::refusal("r", "no")), true);
        assert!(d.out.ends_with("}\n") && d.err.is_empty());
        let d = finish(
            Ok(Value::Object([("k".into(), Value::Int(1))].into())),
            false,
        );
        assert_eq!((d.code, d.out.as_str()), (0, "k: 1\n"));
    }

    #[test]
    fn module_by_first_segment() {
        let all = modules::all();
        assert!(module_of(&all, "arbitre.module.list").is_none());
        if cfg!(feature = "test-actions") {
            let m = module_of(&all, "fake-b.put").expect("fake-b");
            assert_eq!(m.id(), "fake-b");
            assert!(module_of(&all, "fake-c.put").is_none());
        }
    }

    #[test]
    fn human_help_is_the_text_json_help_is_a_document() {
        let human = run(&os(&["arbitre", "--help"]), false);
        assert!(human.out.starts_with("Usage:") && human.code == 0);
        let json = run(&os(&["arbitre", "--help", "--json"]), true);
        assert!(json.out.contains("\"help\"") && json.code == 0);
    }
}
