//! Fake module, two instances `fake-a` and `fake-b` (feature `test-actions`, never shipped):
//! what the conformance test runs on (ST-04 specs RG-ST04-5.5).

use std::collections::BTreeSet;

use arbitre_core::{
    ActionDescriptor, ActionError, Context, LocalBoxFuture, LoggedAs, Module, ModuleStore, Network,
    Output, Problem, Schema, Store, StoreError, Value,
};
use arbitre_sqlite::SqliteStore;

use super::foundation::{action, list, prop, string};

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
        Box::pin(async move { todo!("T-5 {action} {input:?} {}", ctx.store.module()) })
    }

    /// A `value` equal to `invalid` gives the problem `invalid_value`.
    fn revalidate<'a>(
        &'a self,
        records: &'a [(String, String)],
        view: &'a ModuleStore<'a, S>,
    ) -> LocalBoxFuture<'a, Result<Vec<Problem>, StoreError>> {
        Box::pin(async move { todo!("T-5 {records:?} {}", view.module()) })
    }
}
