//! PROP-ST02-9 on `SqliteStore`: core's contract suite, included by path (plan T-6, T-9, P-9).
#[path = "../../core/tests/support/mod.rs"]
mod support;

#[path = "../../core/tests/support/contract.rs"]
mod contract;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use arbitre_sqlite::{Options, SqliteStore};
use contract::Harness;

/// Fresh store per sequence, in its own temp directory; `reopen` = drop, then open.
struct SqliteHarness {
    path: PathBuf,
    store: Option<SqliteStore>,
}

fn open(path: &Path) -> SqliteStore {
    SqliteStore::open(path, &Options::default()).expect("open store")
}

impl Harness for SqliteHarness {
    type Store = SqliteStore;

    fn fresh() -> Self {
        static N: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "arbitre-contract-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("store.db");
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
