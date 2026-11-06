//! PROP-ST02-9 on `SqliteStore`: core's contract suite, included by path (plan T-6, T-9, P-9).
#[path = "../../core/tests/support/mod.rs"]
mod support;

#[path = "../../core/tests/support/contract.rs"]
mod contract;

#[path = "support/common.rs"]
mod common;

use std::path::PathBuf;

use arbitre_sqlite::SqliteStore;
use common::{fresh_dir, open};
use contract::Harness;

/// Fresh store per sequence, in its own temp directory; `reopen` = drop, then open.
struct SqliteHarness {
    path: PathBuf,
    store: Option<SqliteStore>,
}

impl Harness for SqliteHarness {
    type Store = SqliteStore;

    fn fresh() -> Self {
        let path = fresh_dir("contract").join("store.db");
        let store = Some(open(&path));
        Self { path, store }
    }

    fn store(&self) -> &SqliteStore {
        self.store.as_ref().expect("store open")
    }

    fn reopen(&mut self) {
        self.store = None;
        self.store = Some(open(&self.path));
    }
}

impl Drop for SqliteHarness {
    fn drop(&mut self) {
        self.store = None;
        if let Some(dir) = self.path.parent() {
            let _ = std::fs::remove_dir_all(dir);
        }
    }
}

#[test]
fn prop_st02_9_contract_suite_on_sqlite() {
    contract::run::<SqliteHarness>();
}
