//! Test actions `arbitre.test.*` (feature `test-actions`, never shipped): one per ST-03 rule,
//! plus `test.signal` (ST-04 DA F-1, F-2). Schemas are what `crates/cli/tests/` expect.

use arbitre_core::{
    ActionDescriptor, ActionError, Change, Field, FieldValue, LocalChange, Position, RecordId,
    Schema, Store, Value,
};

use super::{Env, action, prop, string};
use crate::store::{block_on, map_apply, map_store};

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

fn key() -> arbitre_core::Property {
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

pub fn descriptors() -> Vec<ActionDescriptor> {
    let mut echo_output = echo_input();
    echo_output.push(prop("remark", "A remark.", true, string(None)));
    let mutating = |a: ActionDescriptor| ActionDescriptor {
        mutating: true,
        ..a
    };
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
    let object = |kv: &[(&str, Value)]| {
        Value::Object(kv.iter().map(|(k, v)| (k.to_string(), v.clone())).collect())
    };
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
            Ok(object(&[]))
        }
        "arbitre.test.refuse" => Err(ActionError::refusal("refused", "refused on purpose")
            .with_data(object(&[("item", Value::Str("x".into()))]))),
        "arbitre.test.fail" => Err(ActionError::tool_error("failed", "failed on purpose")),
        "arbitre.test.panic" => panic!("test.panic"),
        "arbitre.test.write" => write(&input, env),
        "arbitre.test.read" => read(env),
        "arbitre.test.signal" => signal(&input, env),
        // noop, bad-output (misses its required key), group, group.child, test-foundation
        _ => Ok(object(&[])),
    }
}

fn write(input: &Value, env: &Env<'_>) -> Result<Value, ActionError> {
    let Value::Object(o) = input else {
        unreachable!("input conforms to an object schema")
    };
    let Some(Value::Str(key)) = o.get("key") else {
        unreachable!("key is required")
    };
    match o.get("fail") {
        Some(Value::Str(m)) if m == "refusal" => {
            return Err(ActionError::refusal("refused", "refused before writing"));
        }
        Some(Value::Str(m)) if m == "tool-error" => {
            return Err(ActionError::tool_error("failed", "failed before writing"));
        }
        Some(Value::Str(m)) if m == "panic" => panic!("test.write --fail panic"),
        _ => {}
    }
    let change = LocalChange {
        record: RecordId {
            module: "arbitre".into(),
            category: "item".into(),
            id: key.clone(),
        },
        change: Change::Field {
            name: Field("v".into()),
            value: FieldValue(key.as_bytes().to_vec()),
        },
        author: env.author,
    };
    block_on(env.store.apply_local(env.clock.now(), &[change]))?.map_err(map_apply)?;
    Ok(Value::Object(
        [("key".to_string(), Value::Str(key.clone()))].into(),
    ))
}

fn read(env: &Env<'_>) -> Result<Value, ActionError> {
    let writes = block_on(env.store.writes_since(Position::START))?.map_err(map_store)?;
    let keys = writes
        .into_iter()
        .filter(|(_, w)| w.record.module == "arbitre" && w.record.category == "item")
        .map(|(_, w)| Value::Str(w.record.id))
        .collect();
    Ok(Value::Object(
        [("keys".to_string(), Value::List(keys))].into(),
    ))
}

/// Deposits through `signals::deposit`, from this process (RG-ST04-6.3 simulator).
fn signal(input: &Value, env: &Env<'_>) -> Result<Value, ActionError> {
    let text = |k: &str| match input {
        Value::Object(o) => match o.get(k) {
            Some(Value::Str(s)) => s.clone(),
            _ => unreachable!("{k} is required"),
        },
        _ => unreachable!("input conforms to an object schema"),
    };
    crate::signals::deposit(
        env.store,
        env.clock.now(),
        &text("source"),
        &text("message"),
    )?;
    Ok(Value::Object([].into()))
}
