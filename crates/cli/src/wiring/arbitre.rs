//! Foundation `arbitre`: a dispatcher holding the whole store, not a `Module` impl
//! (ST-04 specs RG-ST04-1.1 to 1.7, 7.2; plan P-9, P-10).

use std::collections::BTreeSet;

use arbitre_core::{
    ActionDescriptor, ActionError, Author, Clock, LocalChange, LoggedAs, ModuleStore, Position,
    Property, RecordId, RecordState, Schema, Status, Store, Value, enabled_from, encode,
    state_change, state_record,
};
use arbitre_sqlite::SqliteStore;

use super::{Modules, get, ids};
use crate::store::{block_on, map_apply, map_store};

#[cfg(feature = "test-actions")]
#[path = "arbitre_test.rs"]
mod test;

/// What a foundation action runs with.
pub struct Env<'a> {
    pub store: &'a SqliteStore,
    pub clock: &'a dyn Clock,
    pub author: Author,
    pub modules: &'a Modules,
}

pub fn prop(name: &str, description: &str, required: bool, schema: Schema) -> Property {
    Property {
        name: name.into(),
        description: description.into(),
        required,
        schema,
    }
}

pub fn string(max_length: Option<u64>) -> Schema {
    Schema::String {
        min_length: None,
        max_length,
    }
}

pub fn list(item: Schema) -> Schema {
    Schema::List(Box::new(item))
}

/// A non-mutating, offline action.
pub fn action(
    name: &str,
    summary: &str,
    input: Vec<Property>,
    output: Vec<Property>,
) -> ActionDescriptor {
    ActionDescriptor {
        name: name.into(),
        summary: summary.into(),
        input: Schema::Object(input),
        output: Schema::Object(output),
        mutating: false,
        logged_as: LoggedAs::None,
        requires_network: BTreeSet::new(),
    }
}

pub fn mutating(a: ActionDescriptor) -> ActionDescriptor {
    ActionDescriptor {
        mutating: true,
        ..a
    }
}

/// Required non-empty `key`.
#[cfg(feature = "test-actions")]
pub fn key() -> Property {
    prop(
        "key",
        "Record key.",
        true,
        Schema::String {
            min_length: Some(1),
            max_length: None,
        },
    )
}

pub fn object<'k>(kv: impl IntoIterator<Item = (&'k str, Value)>) -> Value {
    Value::Object(kv.into_iter().map(|(k, v)| (k.into(), v)).collect())
}

/// Text property `key` of a conforming input; `None` if absent.
pub fn text<'a>(input: &'a Value, key: &str) -> Option<&'a str> {
    let Value::Object(o) = input else {
        unreachable!("input conforms to an object schema")
    };
    match o.get(key) {
        Some(Value::Str(s)) => Some(s),
        _ => None,
    }
}

/// `arbitre.module.*` (`enable`/`disable` only with a toggleable module, DA F-7), plus the
/// test actions with `test-actions`. Descriptor schemas are given as canonical JSON text.
pub fn descriptors(toggleable: &[&str]) -> Vec<ActionDescriptor> {
    let module = || {
        vec![prop(
            "module",
            "Module id.",
            true,
            Schema::Enum(toggleable.iter().map(|m| m.to_string()).collect()),
        )]
    };
    let flag = |n: &str, d: &str| prop(n, d, true, Schema::Boolean);
    let text = |n: &str, d: &str| prop(n, d, true, string(None));
    let problem = Schema::Object(vec![
        text("category", "Record category."),
        text("id", "Record id."),
        text("problem", "Problem id."),
        text("message", "What is wrong."),
    ]);
    let descriptor = Schema::Object(vec![
        text("name", "Action name."),
        text("summary", "One-line summary."),
        text("input", "Input JSON Schema, canonical JSON."),
        text("output", "Output JSON Schema, canonical JSON."),
        flag("mutating", "Changes data."),
        text("logged_as", "none, delete or move."),
        prop(
            "requires_network",
            "calendar, relay, ai.",
            true,
            list(string(None)),
        ),
    ]);
    let entry = Schema::Object(vec![
        text("id", "Module id."),
        flag("enabled", "Enabled on this device."),
        prop(
            "actions",
            "Its actions, empty when disabled.",
            true,
            list(descriptor),
        ),
    ]);
    let mut all = vec![action(
        "arbitre.module.list",
        "Lists the modules of this build and their state.",
        vec![],
        vec![prop("modules", "Modules.", true, list(entry))],
    )];
    if !toggleable.is_empty() {
        all.push(mutating(action(
            "arbitre.module.enable",
            "Enables a module on this device.",
            module(),
            vec![
                text("module", "Module id."),
                flag("enabled", "Always true."),
                flag("changed", "False if it was already enabled."),
                prop(
                    "problems",
                    "Problems found by revalidation.",
                    true,
                    list(problem),
                ),
            ],
        )));
        all.push(mutating(action(
            "arbitre.module.disable",
            "Disables a module on this device; its data is kept.",
            module(),
            vec![
                text("module", "Module id."),
                flag("enabled", "Always false."),
                flag("changed", "False if it was already disabled."),
            ],
        )));
    }
    #[cfg(feature = "test-actions")]
    all.extend(test::descriptors());
    all
}

/// Runs a foundation action on a conforming input.
pub fn execute(name: &str, input: Value, env: &Env<'_>) -> Result<Value, ActionError> {
    match name {
        "arbitre.module.enable" => switch(&input, true, env),
        "arbitre.module.disable" => switch(&input, false, env),
        "arbitre.module.list" => list_modules(env),
        #[cfg(feature = "test-actions")]
        _ => test::execute(name, input, env),
        #[cfg(not(feature = "test-actions"))]
        _ => unreachable!("{name} is not a foundation action"),
    }
}

/// This device's enabled-state record and its projected state (RG-ST04-1.5).
fn state(store: &SqliteStore) -> Result<(RecordId, Option<RecordState>), ActionError> {
    let record = state_record(block_on(store.device())?.map_err(map_store)?);
    let st = block_on(store.state(&record))?.map_err(map_store)?;
    Ok((record, st))
}

/// Toggleable modules enabled on this device (RG-ST04-1.6).
pub fn enabled(
    store: &SqliteStore,
    modules: &Modules,
) -> Result<BTreeSet<&'static str>, ActionError> {
    let (_, st) = state(store)?;
    Ok(enabled_from(st.as_ref(), &ids(modules)))
}

/// `enable` revalidates the whole module before writing the state (P-10); idempotent:
/// the state is written only when it changes (RG-ST04-1.2).
fn switch(input: &Value, on: bool, env: &Env<'_>) -> Result<Value, ActionError> {
    let module = text(input, "module").expect("module is required");
    let (record, st) = state(env.store)?;
    let changed = enabled_from(st.as_ref(), &ids(env.modules)).contains(module) != on;
    let problems = if on && changed {
        revalidate(module, env)?
    } else {
        Vec::new()
    };
    if changed {
        let change = LocalChange {
            record,
            change: state_change(module, on),
            author: env.author,
        };
        block_on(env.store.apply_local(env.clock.now(), &[change]))?.map_err(map_apply)?;
    }
    let mut out = vec![
        ("module", Value::Str(module.into())),
        ("enabled", Value::Bool(on)),
        ("changed", Value::Bool(changed)),
    ];
    if on {
        out.push(("problems", Value::List(problems)));
    }
    Ok(object(out))
}

/// Revalidates every present record of `module` (P-10): `{category, id, problem, message}`.
fn revalidate(module: &str, env: &Env<'_>) -> Result<Vec<Value>, ActionError> {
    let m = get(env.modules, module).expect("module is in the build enum");
    let keys: BTreeSet<_> = block_on(env.store.writes_since(Position::START))?
        .map_err(map_store)?
        .into_iter()
        .filter(|(_, w)| w.record.module == module)
        .map(|(_, w)| w.record)
        .collect();
    let mut records = Vec::new();
    for r in keys {
        let st = block_on(env.store.state(&r))?.map_err(map_store)?;
        if st.is_some_and(|s| s.status == Status::Present) {
            records.push((r.category, r.id));
        }
    }
    let view = ModuleStore::new(env.store, m.id(), env.author, env.clock);
    let problems = block_on(m.revalidate(&records, &view))?.map_err(map_store)?;
    Ok(problems
        .into_iter()
        .map(|p| {
            object([
                ("category", Value::Str(p.record.category)),
                ("id", Value::Str(p.record.id)),
                ("problem", Value::Str(p.id)),
                ("message", Value::Str(p.message)),
            ])
        })
        .collect())
}

/// Every module of the build with its state; actions only when enabled (RG-ST04-2.1).
fn list_modules(env: &Env<'_>) -> Result<Value, ActionError> {
    let on = enabled(env.store, env.modules)?;
    let modules = env
        .modules
        .iter()
        .map(|m| {
            let enabled = on.contains(m.id());
            let actions = if enabled {
                m.actions().iter().map(describe).collect()
            } else {
                Vec::new()
            };
            object([
                ("id", Value::Str(m.id().into())),
                ("enabled", Value::Bool(enabled)),
                ("actions", Value::List(actions)),
            ])
        })
        .collect();
    Ok(object([("modules", Value::List(modules))]))
}

/// A descriptor as `module list` gives it: schemas as canonical JSON text (skeleton decision 1).
fn describe(a: &ActionDescriptor) -> Value {
    let s = |x: &str| Value::Str(x.into());
    object([
        ("name", Value::Str(a.name.clone())),
        ("summary", Value::Str(a.summary.clone())),
        ("input", Value::Str(encode(&a.input_json_schema()))),
        ("output", Value::Str(encode(&a.output_json_schema()))),
        ("mutating", Value::Bool(a.mutating)),
        ("logged_as", s(a.logged_as.as_str())),
        (
            "requires_network",
            Value::List(a.requires_network.iter().map(|n| s(n.as_str())).collect()),
        ),
    ])
}

#[cfg(test)]
mod tests {
    use arbitre_core::{Context, LocalBoxFuture, Output, Problem, StoreError};

    use super::*;

    struct NoAction;

    impl arbitre_core::Module<SqliteStore> for NoAction {
        fn id(&self) -> &'static str {
            "empty"
        }

        fn actions(&self) -> Vec<ActionDescriptor> {
            Vec::new()
        }

        fn execute<'a>(
            &'a self,
            _: &'a str,
            _: Value,
            _: &'a Context<'a, SqliteStore>,
        ) -> LocalBoxFuture<'a, Result<Output, ActionError>> {
            unreachable!("no action")
        }

        fn revalidate<'a>(
            &'a self,
            _: &'a [(String, String)],
            _: &'a ModuleStore<'a, SqliteStore>,
        ) -> LocalBoxFuture<'a, Result<Vec<Problem>, StoreError>> {
            Box::pin(async { Ok(Vec::new()) })
        }
    }

    #[test]
    fn a_module_with_no_action_enables_lists_and_disables() {
        // Matrix US-5 row 4.
        let modules: Modules = vec![Box::new(NoAction)];
        super::super::catalog(&modules, |_| true).expect("valid with no action");
        let dir = std::env::temp_dir().join(format!("arbitre-no-action-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("temp dir");
        let store = crate::store::open(&dir.join("s.db")).expect("store");
        let env = Env {
            store: &store,
            clock: &crate::clock::SystemClock,
            author: Author::CliUser,
            modules: &modules,
        };
        let id = || Value::Str("empty".into());
        let input = object([("module", id())]);
        let on = execute("arbitre.module.enable", input.clone(), &env).expect("enable");
        assert_eq!(
            on,
            object([
                ("module", id()),
                ("enabled", Value::Bool(true)),
                ("changed", Value::Bool(true)),
                ("problems", Value::List(vec![])),
            ])
        );
        let entry = object([
            ("id", id()),
            ("enabled", Value::Bool(true)),
            ("actions", Value::List(vec![])),
        ]);
        assert_eq!(
            execute("arbitre.module.list", object([]), &env).expect("list"),
            object([("modules", Value::List(vec![entry]))])
        );
        let off = execute("arbitre.module.disable", input, &env).expect("disable");
        assert_eq!(text(&off, "module"), Some("empty"));
        assert_eq!(enabled(&store, &modules).expect("state"), BTreeSet::new());
    }

    #[test]
    fn only_enable_and_disable_mutate() {
        let d = descriptors(&["m"]);
        let mutating = |n: &str| d.iter().find(|a| a.name == n).expect(n).mutating;
        assert!(mutating("arbitre.module.enable") && mutating("arbitre.module.disable"));
        assert!(!mutating("arbitre.module.list"));
    }
}
