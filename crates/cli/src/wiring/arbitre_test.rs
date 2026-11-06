//! Test actions `arbitre.test.*` (feature `test-actions`, never shipped): one per ST-03 rule,
//! plus `test.signal` (ST-04 DA F-1, F-2). Schemas are what `crates/cli/tests/` expect.

use arbitre_core::{
    ActionDescriptor, ActionError, Change, LocalChange, Position, RecordId, Schema, Store, Value,
};

use super::{Env, action, key, mutating, object, prop, string, text};
use crate::store::{block_on, map_apply, wait};

/// Inputs of `test.echo`: every subset type (mirror in `crates/core/tests/catalog.rs`).
fn echo_input() -> Vec<arbitre_core::Property> {
    vec![
        prop("text", "Text to echo.", true, string(Some(200))),
        prop(
            "count",
            "A count.",
            false,
            Schema::Integer {
                minimum: Some(0),
                maximum: Some(100),
            },
        ),
        prop("flag", "A flag.", true, Schema::Boolean),
        prop(
            "level",
            "A level.",
            false,
            Schema::Enum(vec!["low".into(), "medium".into(), "high".into()]),
        ),
        prop("due", "A due date.", false, Schema::Date),
        prop(
            "tag",
            "A tag, repeatable.",
            true,
            Schema::List(Box::new(string(None))),
        ),
        prop("note", "An optional note.", false, string(None)),
    ]
}

pub fn descriptors() -> Vec<ActionDescriptor> {
    let mut echo_output = echo_input();
    echo_output.push(prop("remark", "A remark.", true, string(None)));
    vec![
        action(
            "arbitre.test.echo",
            "Echoes its input with a remark.",
            echo_input(),
            echo_output,
        ),
        action("arbitre.test.noop", "Does nothing.", vec![], vec![]),
        action(
            "arbitre.test.refuse",
            "Refuses, with details.",
            vec![],
            vec![],
        ),
        action(
            "arbitre.test.fail",
            "Fails as a tool error.",
            vec![],
            vec![],
        ),
        action("arbitre.test.panic", "Panics.", vec![], vec![]),
        action(
            "arbitre.test.bad-output",
            "Returns an output that breaks its schema.",
            vec![],
            vec![prop("key", "A key.", true, string(None))],
        ),
        action(
            "arbitre.test.noisy",
            "Writes diagnostics to standard error.",
            vec![],
            vec![],
        ),
        mutating(action(
            "arbitre.test.write",
            "Writes one record, or fails before writing.",
            vec![
                key(),
                prop(
                    "fail",
                    "Fail this way before writing.",
                    false,
                    Schema::Enum(vec!["refusal".into(), "tool-error".into(), "panic".into()]),
                ),
            ],
            vec![prop("key", "Record key.", true, string(None))],
        )),
        action(
            "arbitre.test.read",
            "Lists the keys written by test.write.",
            vec![],
            vec![prop(
                "keys",
                "Written keys, in write order.",
                true,
                Schema::List(Box::new(string(None))),
            )],
        ),
        action(
            "arbitre.test.group",
            "An action that is also a group.",
            vec![],
            vec![],
        ),
        action(
            "arbitre.test.group.child",
            "An action inside a group.",
            vec![],
            vec![],
        ),
        mutating(action(
            "arbitre.test.signal",
            "Leaves a signal, as a background task would.",
            vec![
                prop("source", "Signal source id.", true, string(None)),
                prop("message", "Signal text.", true, string(None)),
            ],
            vec![],
        )),
        action(
            "arbitre.test-foundation",
            "A foundation action.",
            vec![],
            vec![],
        ),
    ]
}

/// Runs one test action. `test.refuse` attaches `{"item":"x"}`; `test.write --fail` acts
/// before `apply_local`; `test.echo` returns its input plus `remark`.
pub fn execute(name: &str, input: Value, env: &Env<'_>) -> Result<Value, ActionError> {
    match name {
        "arbitre.test.echo" => {
            let Value::Object(mut o) = input else {
                unreachable!("input conforms to an object schema")
            };
            o.insert("remark".into(), Value::Str("echoed".into()));
            Ok(Value::Object(o))
        }
        "arbitre.test.noisy" => {
            eprintln!("diagnostic on standard error");
            Ok(object([]))
        }
        "arbitre.test.refuse" => Err(ActionError::refusal("refused", "refused on purpose")
            .with_data(object([("item", Value::Str("x".into()))]))),
        "arbitre.test.fail" => Err(ActionError::tool_error("failed", "failed on purpose")),
        "arbitre.test.panic" => panic!("test.panic"),
        "arbitre.test.write" => write(&input, env),
        "arbitre.test.read" => read(env),
        "arbitre.test.signal" => signal(&input, env),
        // noop, bad-output (misses its required key), group, group.child, test-foundation
        _ => Ok(object([])),
    }
}

fn write(input: &Value, env: &Env<'_>) -> Result<Value, ActionError> {
    let key = text(input, "key").expect("key is required");
    match text(input, "fail") {
        Some("refusal") => return Err(ActionError::refusal("refused", "refused before writing")),
        Some("tool-error") => {
            return Err(ActionError::tool_error("failed", "failed before writing"));
        }
        Some("panic") => panic!("test.write --fail panic"),
        _ => {}
    }
    let change = LocalChange {
        record: RecordId {
            module: "arbitre".into(),
            category: "item".into(),
            id: key.into(),
        },
        change: Change::field("v", key.as_bytes()),
        author: env.author,
    };
    block_on(env.store.apply_local(env.clock.now(), &[change]))?.map_err(map_apply)?;
    Ok(object([("key", Value::Str(key.into()))]))
}

fn read(env: &Env<'_>) -> Result<Value, ActionError> {
    let writes = wait(env.store.writes_since(Position::START))?;
    let keys = writes
        .into_iter()
        .filter(|(_, w)| w.record.module == "arbitre" && w.record.category == "item")
        .map(|(_, w)| Value::Str(w.record.id))
        .collect();
    Ok(object([("keys", Value::List(keys))]))
}

/// Deposits through `signals::deposit`, from this process (RG-ST04-6.3 simulator).
fn signal(input: &Value, env: &Env<'_>) -> Result<Value, ActionError> {
    let text = |k: &str| text(input, k).expect("required");
    crate::signals::deposit(env.store, env.clock.now(), text("source"), text("message"))?;
    Ok(object([]))
}
