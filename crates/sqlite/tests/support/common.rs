//! Shared by the SQLite adapter tests: temp dirs, open, records, seeded randomness, raw `K`.
#![allow(dead_code)]

use std::path::{Path, PathBuf};

use arbitre_core::RecordId;
use arbitre_sqlite::{Options, SqliteStore};
use rusqlite::Connection;

/// splitmix64; the base seed comes from `ARBITRE_SEED` (printed by the tests).
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }
    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    pub fn below(&mut self, n: u64) -> u64 {
        self.next() % n.max(1)
    }
    pub fn chance(&mut self, percent: u64) -> bool {
        self.below(100) < percent
    }
}

pub fn seed() -> u64 {
    std::env::var("ARBITRE_SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0x5EED)
}

pub fn rec(category: &str, id: &str) -> RecordId {
    RecordId {
        module: "m".into(),
        category: category.into(),
        id: id.into(),
    }
}

/// Fresh empty directory; the caller removes it.
pub fn fresh_dir(tag: &str) -> PathBuf {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static N: AtomicUsize = AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "arbitre-{tag}-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Store at `path` with default options; panics on failure.
pub fn open(path: &Path) -> SqliteStore {
    SqliteStore::open(path, &Options::default()).expect("open")
}

/// Persisted clock `K` `(clock_l, clock_c)`, read through a raw connection.
pub fn raw_k(raw: &Connection) -> (i64, i64) {
    raw.query_row("SELECT clock_l, clock_c FROM device", [], |r| {
        Ok((r.get(0)?, r.get(1)?))
    })
    .unwrap()
}
