//! Fake module, two instances `fake-a` and `fake-b` (feature `test-actions`, never shipped):
//! what the conformance test runs on (ST-04 specs RG-ST04-5.5).

use std::collections::BTreeSet;

use arbitre_core::{
    ActionDescriptor, ActionError, Change, Context, LocalBoxFuture, LoggedAs, Module, ModuleStore,
    Network, Output, Problem, RecordId, Schema, Status, Store, StoreError, Value,
};
use arbitre_sqlite::SqliteStore;

use super::foundation::{action, key, list, mutating, object, prop, string, text};
use crate::store::{map_apply, map_store};

const ENTRY: &str = "entry";
const VALUE: &str = "value";

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
        let value = || prop("value", "Entry value.", true, string(None));
        let entry = || vec![prop("key", "Entry key.", true, string(None)), value()];
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
            let text = |k: &str| text(&input, k).expect("required").to_string();
            match verb {
                "list" => {
                    let entries = view
                        .records(ENTRY)
                        .await
                        .map_err(map_store)?
                        .into_iter()
                        .filter_map(|(key, st)| present_value(&st).map(|v| entry(key, v)))
                        .collect();
                    Ok(object([("entries", Value::List(entries))]))
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
                            Ok(object([]))
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
                if st.and_then(|s| s.text(VALUE)).as_deref() == Some("invalid") {
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

fn value_change(value: &str) -> Change {
    Change::field(VALUE, value.as_bytes())
}

/// `value` of a present entry; `None` if absent or deleted.
fn present_value(st: &arbitre_core::RecordState) -> Option<String> {
    match st.status {
        Status::Present => st.text(VALUE),
        Status::Deleted(_) => None,
    }
}

fn entry(key: String, value: String) -> Value {
    object([("key", Value::Str(key)), ("value", Value::Str(value))])
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
