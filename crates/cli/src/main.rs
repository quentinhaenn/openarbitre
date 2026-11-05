mod cli;
mod clock;
mod modules;
mod signals;
mod store;

use std::collections::BTreeSet;
use std::ffi::{OsStr, OsString};
use std::io::Write;
use std::path::PathBuf;

use arbitre_core::{
    ActionDescriptor, ActionError, Author, Category, Context, Module, ModuleStore, conforms_output,
    document, encode, pre_scan, render,
};
use arbitre_sqlite::SqliteStore;
use clap::ArgMatches;
use clap::error::ErrorKind;

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

/// The store located before parsing: the existing handle, else opened (created) at `path`.
struct Located {
    path: Result<PathBuf, ActionError>,
    handle: Option<SqliteStore>,
}

/// Before parsing: the location, once, and the enabled state of an existing store only
/// (RG-ST04-2.3; plan P-8). A location error gives the default state; the action reports it.
fn locate(
    store_flag: Option<&str>,
    modules: &Modules,
) -> Result<(Located, BTreeSet<&'static str>), ActionError> {
    let path = store::location(
        store_flag.map(OsStr::new),
        |k| std::env::var_os(k),
        std::env::consts::OS,
    );
    let handle = match &path {
        Ok(p) => store::open_existing(p)?,
        Err(_) => None,
    };
    let enabled = match &handle {
        Some(s) => foundation::enabled(s, modules)?,
        None => BTreeSet::new(),
    };
    Ok((Located { path, handle }, enabled))
}

/// A first command word naming a disabled module (RG-ST04-2.2).
fn not_enabled(
    first_word: Option<&str>,
    modules: &Modules,
    enabled: &BTreeSet<&str>,
) -> Option<ActionError> {
    let m = modules::ids(modules)
        .into_iter()
        .find(|m| first_word == Some(*m) && !enabled.contains(m))?;
    Some(ActionError::module_not_enabled(
        "module_not_enabled",
        format!("module {m} is not enabled; run: arbitre module enable --module {m}"),
    ))
}

fn execute(
    action: &ActionDescriptor,
    leaf: &ArgMatches,
    modules: &Modules,
    located: Located,
) -> Outcome {
    let store = match located.handle {
        Some(s) => s,
        None => store::open(&located.path?)?,
    };
    let clock = SystemClock;
    // `test.signal` stands for a background task (RG-ST04-6.3), not a user command: showing
    // here would consume the signals it is meant to pile up (`tests/signals.rs` oldest_first).
    if action.name != "arbitre.test.signal" {
        signals::show_and_mark(&store, &clock, &mut std::io::stderr());
    }
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
    // The whole build is validated, whatever is enabled (plan P-4).
    if let Err(e) = modules::catalog(&modules, |_| true) {
        return finish(
            Err(ActionError::tool_error("catalog_invalid", e.message)),
            json,
        );
    }
    let words: Vec<_> = args.iter().skip(1).map(|a| a.to_string_lossy()).collect();
    let words: Vec<&str> = words.iter().map(|w| w.as_ref()).collect();
    let scan = pre_scan(&words);
    let (located, enabled) = match locate(scan.store, &modules) {
        Ok(x) => x,
        Err(e) => return unreadable(args, &modules, e, json),
    };
    if let Some(e) = not_enabled(scan.first_word, &modules, &enabled) {
        return finish(Err(e), json);
    }
    let catalog = modules::catalog(&modules, |m| enabled.contains(m))
        .expect("a subset of a valid build is valid");
    let matches = match cli::command(&catalog).try_get_matches_from(args) {
        Ok(m) => m,
        Err(e) => return clap_exit(e, json),
    };
    let (action, leaf) = cli::selected(&matches, &catalog);
    finish(execute(action, leaf, &modules, located), json)
}

/// Help, version or usage error.
fn clap_exit(e: clap::Error, json: bool) -> Done {
    if json || e.use_stderr() {
        return finish(cli::usage_error(e), json);
    }
    // Human help/version: the text itself, not a `help: …` rendering.
    Done {
        out: e.to_string(),
        err: String::new(),
        code: 0,
    }
}

/// An unreadable store stops everything but `--version` (CONFLICT-ST04-2).
fn unreadable(args: &[OsString], modules: &Modules, e: ActionError, json: bool) -> Done {
    let foundation = modules::catalog(modules, |_| false).expect("valid build");
    match cli::command(&foundation).try_get_matches_from(args) {
        Err(v) if v.kind() == ErrorKind::DisplayVersion => clap_exit(v, json),
        _ => finish(Err(e), json),
    }
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
        // An absent store, never the machine's default one.
        let absent = std::env::temp_dir()
            .join(format!("arbitre-help-{}", std::process::id()))
            .join("absent.db");
        let store = absent.to_str().expect("utf-8 temp dir");
        let human = run(&os(&["arbitre", "--help", "--store", store]), false);
        assert!(human.out.starts_with("Usage:") && human.code == 0);
        let json = run(
            &os(&["arbitre", "--help", "--json", "--store", store]),
            true,
        );
        assert!(json.out.contains("\"help\"") && json.code == 0);
    }
}
