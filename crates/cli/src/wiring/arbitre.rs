//! Foundation `arbitre`: a dispatcher holding the whole store, not a `Module` impl
//! (ST-04 specs RG-ST04-1.1 to 1.7, 7.2; plan P-9, P-10).

use std::collections::BTreeSet;

use arbitre_core::{
    ActionDescriptor, ActionError, Author, Clock, LoggedAs, Property, Schema, Value,
};
use arbitre_sqlite::SqliteStore;

use super::Modules;

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
    let mutating = |a: ActionDescriptor| ActionDescriptor {
        mutating: true,
        ..a
    };
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

/// `enable` revalidates the whole module before writing the state (P-10); idempotent.
fn switch(input: &Value, on: bool, env: &Env<'_>) -> Result<Value, ActionError> {
    todo!(
        "T-4/T-5 {input:?} {on} {} {:p}",
        env.modules.len(),
        env.store
    )
}

fn list_modules(env: &Env<'_>) -> Result<Value, ActionError> {
    todo!("T-4 {:?} {}", env.author, env.clock.now())
}
