//! Fake module, two instances `fake-a` and `fake-b` (feature `test-actions`, never shipped):
//! what the conformance test runs on (ST-04 specs RG-ST04-5.5).

use std::collections::BTreeSet;

use arbitre_core::{
    ActionDescriptor, ActionError, Change, Context, Field, FieldValue, LocalBoxFuture, LoggedAs,
    Module, ModuleStore, Network, Output, Problem, RecordId, Schema, Status, Store, StoreError,
    Value,
};
use arbitre_sqlite::SqliteStore;

use super::foundation::{action, list, prop, string};
use crate::store::{map_apply, map_store};

const ENTRY: &str = "entry";

pub struct Fake {
    id: &'static str,
}

pub fn modules() -> Vec<Box<dyn Module<SqliteStore>>> {
    vec![
        Box::new(Fake { id: "fake-a" }),
        Box::new(Fake { id: "fake-b" }),
    ]
}

impl<S: Store> Module<S> for Fake {
    fn id(&self) -> &'static str {
        self.id
    }

    /// `put`, `get`, `list`, `update`, `delete` on category `entry`, field `value`; `push`
    /// declared `relay`.
    fn actions(&self) -> Vec<ActionDescriptor> {
        let n = |verb: &str| format!("{}.{verb}", self.id);
        let key = || {
            prop(
                "key",
                "Entry key.",
                true,
                Schema::String {
                    min_length: Some(1),
                    max_length: None,
                },
            )
        };
        let value = || prop("value", "Entry value.", true, string(None));
        let entry = || vec![prop("key", "Entry key.", true, string(None)), value()];
        let mutating = |a: ActionDescriptor| ActionDescriptor {
            mutating: true,
            ..a
        };
        vec![
            mutating(action(
                &n("put"),
                "Creates an entry.",
                vec![key(), value()],
                entry(),
            )),
            action(&n("get"), "Reads an entry.", vec![key()], entry()),
            action(
                &n("list"),
                "Lists the entries, by key.",
                vec![],
                vec![prop(
                    "entries",
                    "Entries.",
                    true,
                    list(Schema::Object(entry())),
                )],
            ),
            mutating(action(
                &n("update"),
                "Changes an entry.",
                vec![key(), value()],
                entry(),
            )),
            ActionDescriptor {
                logged_as: LoggedAs::Delete,
                ..mutating(action(
                    &n("delete"),
                    "Deletes an entry.",
                    vec![key()],
                    vec![],
                ))
            },
            ActionDescriptor {
                requires_network: BTreeSet::from([Network::Relay]),
                ..action(
                    &n("push"),
                    "Pushes to a relay (never configured).",
                    vec![],
                    vec![],
                )
            },
        ]
    }

    fn execute<'a>(
        &'a self,
        action: &'a str,
        input: Value,
        ctx: &'a Context<'a, S>,
    ) -> LocalBoxFuture<'a, Result<Output, ActionError>> {
        Box::pin(async move {
            let view = &ctx.store;
            let verb = action.rsplit('.').next().unwrap_or(action);
            let Value::Object(o) = input else {
                unreachable!("input conforms to an object schema")
            };
            let text = |k: &str| match o.get(k) {
                Some(Value::Str(s)) => s.clone(),
                _ => unreachable!("{k} is required"),
            };
            match verb {
                "list" => {
                    let entries = view
                        .records(ENTRY)
                        .await
                        .map_err(map_store)?
                        .into_iter()
                        .filter_map(|(key, st)| present_value(&st).map(|v| entry(key, v)))
                        .collect();
                    Ok(Value::Object(
                        [("entries".to_string(), Value::List(entries))].into(),
                    ))
                }
                "push" => Err(ActionError::tool_error(
                    "relay_not_configured",
                    "no relay is configured",
                )),
                "put" => {
                    let (key, value) = (text("key"), text("value"));
                    write(view, &key, value_change(&value)).await?;
                    Ok(entry(key, value))
                }
                _ => {
                    let key = text("key");
                    let st = view.state(ENTRY, &key).await.map_err(map_store)?;
                    let Some(current) = st.as_ref().and_then(present_value) else {
                        return Err(ActionError::refusal("not_found", format!("no entry {key}")));
                    };
                    match verb {
                        "get" => Ok(entry(key, current)),
                        "update" => {
                            let value = text("value");
                            write(view, &key, value_change(&value)).await?;
                            Ok(entry(key, value))
                        }
                        _ => {
                            write(view, &key, Change::Delete).await?;
                            Ok(Value::Object(Default::default()))
                        }
                    }
                }
            }
        })
    }

    /// A `value` equal to `invalid` gives the problem `invalid_value`.
    fn revalidate<'a>(
        &'a self,
        records: &'a [(String, String)],
        view: &'a ModuleStore<'a, S>,
    ) -> LocalBoxFuture<'a, Result<Vec<Problem>, StoreError>> {
        Box::pin(async move {
            let mut problems = Vec::new();
            for (category, id) in records {
                let st = view.state(category, id).await?;
                if st.as_ref().and_then(|s| s.fields.get(&field())).map(text)
                    == Some("invalid".into())
                {
                    problems.push(Problem {
                        record: RecordId {
                            module: view.module().into(),
                            category: category.clone(),
                            id: id.clone(),
                        },
                        id: "invalid_value".into(),
                        message: format!("{category} {id}: value `invalid`"),
                    });
                }
            }
            Ok(problems)
        })
    }
}

fn field() -> Field {
    Field("value".into())
}

fn value_change(value: &str) -> Change {
    Change::Field {
        name: field(),
        value: FieldValue(value.as_bytes().to_vec()),
    }
}

fn text(w: &arbitre_core::Write) -> String {
    match &w.change {
        Change::Field { value, .. } => String::from_utf8_lossy(&value.0).into_owned(),
        Change::Delete => unreachable!("a field winner is a field write"),
    }
}

/// `value` of a present entry; `None` if absent or deleted.
fn present_value(st: &arbitre_core::RecordState) -> Option<String> {
    match st.status {
        Status::Present => st.fields.get(&field()).map(text),
        Status::Deleted(_) => None,
    }
}

fn entry(key: String, value: String) -> Value {
    Value::Object(
        [
            ("key".to_string(), Value::Str(key)),
            ("value".to_string(), Value::Str(value)),
        ]
        .into(),
    )
}

async fn write<S: Store>(
    view: &ModuleStore<'_, S>,
    key: &str,
    change: Change,
) -> Result<(), ActionError> {
    view.apply(&[(ENTRY.into(), key.into(), change)])
        .await
        .map(drop)
        .map_err(map_apply)
}
