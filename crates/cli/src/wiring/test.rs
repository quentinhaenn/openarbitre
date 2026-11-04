//! Test module (feature `test-actions`, never shipped): one action per ST-03 rule.
//! Actions: specs "Architecture notes / Test module"; schemas are what `crates/cli/tests/` expect.

use std::collections::BTreeSet;

use arbitre_core::value::Value;
use arbitre_core::{
    ActionDescriptor, ActionError, Change, Field, LocalChange, LoggedAs, Position, Property,
    RecordId, Schema, Store, Value as WriteValue,
};

use super::{Context, Wiring};
use crate::store::{block_on, map_apply, map_store};

/// Module `test`, and `arbitre.test-foundation` under the foundation id `arbitre`.
pub fn wirings() -> Vec<Wiring> {
    vec![
        Wiring {
            id: "test",
            descriptors,
            execute,
        },
        Wiring {
            id: "arbitre",
            descriptors: foundation,
            execute,
        },
    ]
}

fn prop(name: &str, description: &str, required: bool, schema: Schema) -> Property {
    Property {
        name: name.into(),
        description: description.into(),
        required,
        schema,
    }
}

fn string(max_length: Option<u64>) -> Schema {
    Schema::String {
        min_length: None,
        max_length,
    }
}

fn action(
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

/// Inputs of `test.echo`: every subset type (mirror in `crates/core/tests/catalog.rs`).
fn echo_input() -> Vec<Property> {
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

fn descriptors() -> Vec<ActionDescriptor> {
    let mut echo_output = echo_input();
    echo_output.push(prop("remark", "A remark.", true, string(None)));
    let write = ActionDescriptor {
        mutating: true,
        ..action(
            "test.write",
            "Writes one record, or fails before writing.",
            vec![
                prop(
                    "key",
                    "Record key.",
                    true,
                    Schema::String {
                        min_length: Some(1),
                        max_length: None,
                    },
                ),
                prop(
                    "fail",
                    "Fail this way before writing.",
                    false,
                    Schema::Enum(vec!["refusal".into(), "tool-error".into(), "panic".into()]),
                ),
            ],
            vec![prop("key", "Record key.", true, string(None))],
        )
    };
    vec![
        action(
            "test.echo",
            "Echoes its input with a remark.",
            echo_input(),
            echo_output,
        ),
        action("test.noop", "Does nothing.", vec![], vec![]),
        action("test.refuse", "Refuses, with details.", vec![], vec![]),
        action("test.fail", "Fails as a tool error.", vec![], vec![]),
        action(
            "test.not-enabled",
            "Reports its module as not enabled.",
            vec![],
            vec![],
        ),
        action("test.panic", "Panics.", vec![], vec![]),
        action(
            "test.bad-output",
            "Returns an output that breaks its schema.",
            vec![],
            vec![prop("key", "A key.", true, string(None))],
        ),
        action(
            "test.noisy",
            "Writes diagnostics to standard error.",
            vec![],
            vec![],
        ),
        write,
        action(
            "test.read",
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
            "test.group",
            "An action that is also a group.",
            vec![],
            vec![],
        ),
        action(
            "test.group.child",
            "An action inside a group.",
            vec![],
            vec![],
        ),
    ]
}

fn foundation() -> Vec<ActionDescriptor> {
    vec![action(
        "arbitre.test-foundation",
        "A foundation action.",
        vec![],
        vec![],
    )]
}

/// Runs one test action. `test.refuse` attaches `{"item":"x"}`; `test.write --fail` acts
/// before `apply_local`; `test.echo` returns its input plus `remark`.
fn execute(name: &str, input: Value, ctx: &Context<'_>) -> Result<Value, ActionError> {
    let object = |kv: &[(&str, Value)]| {
        Value::Object(kv.iter().map(|(k, v)| (k.to_string(), v.clone())).collect())
    };
    match name {
        "test.echo" => {
            let Value::Object(mut o) = input else {
                unreachable!("input conforms to an object schema")
            };
            o.insert("remark".into(), Value::Str("echoed".into()));
            Ok(Value::Object(o))
        }
        "test.noisy" => {
            eprintln!("diagnostic on standard error");
            Ok(object(&[]))
        }
        "test.refuse" => Err(ActionError::refusal("refused", "refused on purpose")
            .with_data(object(&[("item", Value::Str("x".into()))]))),
        "test.fail" => Err(ActionError::tool_error("failed", "failed on purpose")),
        "test.not-enabled" => Err(ActionError::module_not_enabled(
            "module_not_enabled",
            "module test is not enabled",
        )),
        "test.panic" => panic!("test.panic"),
        "test.write" => write(&input, ctx),
        "test.read" => read(ctx),
        // noop, bad-output (misses its required key), group, group.child, foundation
        _ => Ok(object(&[])),
    }
}

fn write(input: &Value, ctx: &Context<'_>) -> Result<Value, ActionError> {
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
            module: "test".into(),
            category: "item".into(),
            id: key.clone(),
        },
        change: Change::Field {
            name: Field("v".into()),
            value: WriteValue(key.as_bytes().to_vec()),
        },
        author: ctx.author,
    };
    block_on(ctx.store.apply_local(ctx.pt, &[change]))?.map_err(map_apply)?;
    Ok(Value::Object(
        [("key".to_string(), Value::Str(key.clone()))].into(),
    ))
}

fn read(ctx: &Context<'_>) -> Result<Value, ActionError> {
    let writes = block_on(ctx.store.writes_since(Position::START))?.map_err(map_store)?;
    let keys = writes
        .into_iter()
        .filter(|(_, w)| w.record.module == "test")
        .map(|(_, w)| Value::Str(w.record.id))
        .collect();
    Ok(Value::Object(
        [("keys".to_string(), Value::List(keys))].into(),
    ))
}
