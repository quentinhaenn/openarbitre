# Architectural Frame — ST-02: Persistence, async `Store` port and SQLite adapter

## Traceability
- Sources: `.ctdd/stories/ST-02/02_specs.md` (its "Quentin's arbitration (2026-11-04)", Q-A → (b) plus a guard on the received path, is binding), `.ctdd/stories/ST-02/01_story.md` (its "Quentin's arbitrations (2026-11-04)" are binding: Q-1 fsync before confirm, Q-2 one persisted clock per device and stamping inside one `BEGIN IMMEDIATE` transaction, Q-3 5 s wait setting, Q-4 refuse a damaged store, Q-5 watermarks in the port), `.ctdd/cadre/03_archi.md` (port `Store`, `arbitre-sqlite`, write paths, migrations), `.ctdd/spikes/S1-A.md` (trait shape), `.ctdd/stories/ST-01/03_archi.md` (format).
- Codebase read on 2026-11-04: `Cargo.toml`, `crates/core/**`, `crates/cli/Cargo.toml`, `CONTRIBUTING.md`, `docs/decisions/ST-01-dependencies.md`, `scripts/ci.sh`, `scripts/ci.d/*.sh`, `scripts/lib/{graph,mutation}.py`, `.ctdd/stories/ST-01/zones.toml`. Cargo registry cache: `rusqlite` 0.41.1, `libsqlite3-sys` 0.39.1, `getrandom` 0.4.3.
- Date: 2026-11-04 · Status: DRAFT · Zones: Core/Full (port and store logic), Std (simulated store), Gen (SQLite adapter).

## Summary

1. **The store logic is pure and lives in the core; adapters only do I/O.** Stamping from the persisted clock, `seen`, the Q-A guard, damage checks on loaded writes, idempotence and the merge `Report` are computed by pure functions of `arbitre-core` (`store.rs`, Full). An adapter opens a transaction, loads the writes of the touched records and the clock state, calls the core, stores what it returns, commits. The simulated store, the SQLite adapter and the future browser adapter (ST-20) share the same semantics by construction.
2. **GAP-ST02-1 is closed with one `pub(crate)` constructor** in `merge.rs`: `Log::with_clock`. No public signature of ST-01 changes; `Log::write` stays the only path that stamps a local write.
3. **Two write operations in the port** (CONFLICT-ST02-1): `apply_local` takes unstamped changes and the physical time and stamps them inside the transaction; `apply` takes a stamped batch (received, imported). `apply` returns the merge `Report` now (GAP-ST02-5), so the port does not need an amendment in ST-19.
4. **SQLite layout v1 is relational and generic**: STRICT tables, identifiers stored as BLOB, `seen` in a child table, no byte codec to maintain. Version in `PRAGMA user_version`, store marker in `PRAGMA application_id`.
5. **Durability and concurrency are SQLite settings**: WAL, `synchronous = FULL`, `fullfsync = ON`, `busy_timeout` = the wait setting (5 s), every write in `BEGIN IMMEDIATE`, every read in one read transaction.
6. **No new shared dependency.** `rusqlite` 0.41.1 (`bundled`) is local to `arbitre-sqlite`; `getrandom` and `proptest` are already in `[workspace.dependencies]`. No CI file changes.

---

## Codebase scope

### Existing state

| Element | Content | Finding |
|---|---|---|
| `crates/core` (`arbitre-core`) | `no_std` + `alloc`, no normal dependency; modules `hlc`, `write`, `merge`, `clock`; feature `sim` (`SimClock`); `tests/support/mod.rs` (generators, fixed seed, 1,000 cases) | `Log` owns a private `HybridClock` created at `(0, 0)`; `Log::write` is the only local write path. This is GAP-ST02-1. |
| `HybridClock` | `new`, `issue`, `receive`; no accessor | Restorable from `K = (l, c)` with the current API: `new(D)` then `receive(&Stamp { l, c, a: D })` gives exactly `(l, c)`. State after `issue` = `(stamp.l, stamp.c)`. **No change needed in `hlc.rs`.** |
| `Log::merge` | validates, inserts, `receive(max stamp)`, returns a `Report` | Reusable as is to rebuild the log of the touched records and to compute the received `Report`. |
| `crates/cli` | clap stub | Not touched (ST-03 wires the store). |
| `Cargo.toml` | `[workspace.dependencies]` has `getrandom` 0.4.3 and `proptest` 1.12.0 | Enough for ST-02. |
| `scripts/lib/graph.py` | `ALLOWED["arbitre-sqlite"] = {arbitre-core}`; `rusqlite`, `libsqlite3-sys` forbidden in the browser list only; rule (e): a dependency used by ≥ 2 crates must be in the workspace | `arbitre-sqlite` fits without a CI change. |
| `scripts/ci.d/10-st01-tests.sh` | `cargo test --workspace --all-features --locked` | Runs the new crate's tests, the `sim` feature included. |
| `scripts/ci.d/50-st01-mutation.sh` | `cargo mutants --in-diff` on the PR diff, `--all-features` | Mutates `crates/sqlite/src/**` too and runs its test suite per mutant: see risk R-1 and the open question. |

### Impacted modules

| Module | Current role | Impact | Action |
|---|---|---|---|
| `crates/core/src/merge.rs` | merge engine | `pub(crate) fn Log::with_clock(clock: HybridClock) -> Log` | Minimal extension (GAP-ST02-1) |
| `crates/core/src/store.rs` | — | Port `Store`, its types and errors, pure store logic | Creation |
| `crates/core/src/sim_store.rs` | — | `SimStore` (feature `sim`) with fault injection | Creation |
| `crates/core/src/lib.rs` | module list | `mod store; mod sim_store;` and re-exports | Modification |
| `crates/core/Cargo.toml` | `sim` feature, tests | `[[test]] store` with `required-features = ["sim"]` | Modification |
| `crates/core/tests/` | ST-01 tests | `store.rs` (pure logic, `SimStore`), `support/contract.rs` (contract suite and reference model) | Creation |
| `crates/sqlite/` (`arbitre-sqlite`) | — | SQLite adapter and its process-level tests | Creation |
| `.ctdd/stories/ST-02/zones.toml` | — | Zones of the new files | Creation |
| `Cargo.lock` | — | `rusqlite`, `libsqlite3-sys`, `getrandom` resolved | Regenerated |
| `CONTRIBUTING.md` | glossary, rules | Glossary rows for the new terms (store, local apply, received apply, position, watermark, clock state). Additive, no rule changes | Extension |
| `crates/core/src/{hlc,write,clock}.rs`, `crates/cli/`, `scripts/`, `.github/` | — | none | Untouched |

### Target tree

```
crates/
├── core/
│   ├── Cargo.toml                 ← [[test]] store (required-features = ["sim"])
│   ├── src/
│   │   ├── lib.rs                 ← mod store, sim_store; pub use
│   │   ├── merge.rs               ← + pub(crate) Log::with_clock (only change)
│   │   ├── store.rs               ← port Store, types, errors, pure store logic   (Full)
│   │   └── sim_store.rs           ← SimStore, #[cfg(feature = "sim")]              (Std)
│   └── tests/
│       ├── store.rs               ← unit tests of store.rs, contract suite on SimStore
│       └── support/
│           ├── mod.rs             ← (ST-01, unchanged)
│           └── contract.rs        ← contract suite + reference model, generic over Store
└── sqlite/                        ← arbitre-sqlite                                  (Gen)
    ├── Cargo.toml
    ├── src/
    │   ├── lib.rs                 ← SqliteStore, Options, open, impl Store, error mapping
    │   └── schema.rs              ← layout v1 (DDL), create / verify at open, row ↔ Write
    └── tests/
        ├── contract.rs            ← includes core's support/contract.rs by #[path], runs it on SqliteStore
        ├── settings.rs            ← durability pragmas, first poll, non-Send / two futures
        ├── crash.rs               ← E-ST02-1 (SIGKILL)
        ├── full.rs                ← E-ST02-2
        ├── interleave.rs          ← E-ST02-3 (two writers + one reader)
        ├── damage.rs              ← E-ST02-4, newer layout
        └── support/
            ├── child.rs           ← child-process protocol (re-exec of the test binary)
            └── full_vfs.rs        ← fault VFS: byte budget, SQLITE_FULL on write / sync
```

The Planner may merge `schema.rs` into `lib.rs` or split test files differently; the boundaries that matter are "pure logic in `store.rs`" and "SQLite only in `crates/sqlite`".

## Existing patterns identified

| Pattern | Location | Follow? | Notes |
|---|---|---|---|
| Functional core, imperative shell | `merge.rs` pure, I/O nowhere in core | Yes | Store semantics in `store.rs` (pure), transactions in adapters. |
| Port + real adapter + simulator | `Clock` / `SimClock` (feature `sim`) | Yes | `Store` / `SimStore` (feature `sim`) / `SqliteStore`. |
| Illegal states unrepresentable | `Author` closed enum, device read from the stamp, `Threshold` non-zero | Yes | `Position`, `Watermark` newtypes; deletion stored with no value; error causes are enums. |
| Validation before effect | `validate` before insert, identifiers checked before `issue` | Yes | Every rejection happens before the transaction stores anything; the transaction is rolled back. |
| Shared test support, fixed seed | `crates/core/tests/support/mod.rs` (`config()`, `SEED`) | Yes | Contract suite reuses its generators and config; process-level harnesses take a seed and print it on failure. |
| Zones per story, strictest wins | `.ctdd/stories/*/zones.toml` | Yes | See "DDD classification and zones". |
| Async port, no `Send`, `&self`, native `async fn` | S1-A | Yes | `#[allow(async_fn_in_trait)]`. |

## Data model

### Persisted state Σ ↔ storage

| Σ part | Core type | SQLite v1 |
|---|---|---|
| `D` | `DeviceId` | `device.device BLOB` (16 bytes) |
| `K` | `ClockState { l: u64, c: u32 }` | `device.clock_l`, `device.clock_c` (same single row as `D`) |
| `W`, `pos` | `Write`, `Position` | `writes` (one row per stored content) + `seen` |
| `M` | `DeviceId → Watermark` | `watermarks` |
| `V` | `u32` | `PRAGMA user_version` (= 1) |
| store marker | — | `PRAGMA application_id` = a fixed constant |

### New core types (`store.rs`)

```
pub struct LocalChange { pub record: RecordId, pub change: Change, pub author: Author }
pub struct ClockState { pub l: u64, pub c: u32 }                 // K, persisted by adapters
pub struct Position(pub u64);   Position::START = Position(0)    // ⊥P; stored positions start at 1
pub struct Watermark(pub u64, pub u32);                          // opaque, lexicographic Ord (AMB-ST02-1)
```

- `Watermark` holds either a peer stamp's `(l, c)` (the peer is the key) or a position as `(pos, 0)`. ST-19 fixes the meaning; the store only keeps `max`.
- Positions: strictly increasing in commit order, in batch (or list) order inside a batch, never reused. Their values are not contractual beyond order (the browser adapter keeps that freedom).

### SQLite layout v1

All tables `STRICT`. Identifiers are BLOB holding the UTF-8 bytes: bytewise order equals `String` order, and an identifier containing NUL round-trips (TEXT does not reliably).

| Table | Columns | Constraints / indexes |
|---|---|---|
| `device` | `id INTEGER PRIMARY KEY`, `device BLOB`, `clock_l INTEGER`, `clock_c INTEGER` | `CHECK (id = 0)` (one row), `length(device) = 16` |
| `writes` | `pos INTEGER PRIMARY KEY`, `module BLOB`, `category BLOB`, `id BLOB`, `field BLOB`, `value BLOB NULL`, `l INTEGER`, `c INTEGER`, `a BLOB`, `author INTEGER` | `field = x''` ⇔ deletion ⇔ `value IS NULL` (empty field names are invalid, so `''` is unambiguous); `length(a) = 16`; `author IN (0,1,2,3)`; index `(module, category, id)`; index `(l, c)` |
| `seen` | `pos INTEGER` (→ `writes.pos`), `field BLOB` (`x''` = `TargetKind::Delete`), `l`, `c`, `a` | `PRIMARY KEY (pos, field)`, `WITHOUT ROWID` |
| `watermarks` | `peer BLOB PRIMARY KEY`, `mark BLOB` | `mark` = 12 bytes big-endian `(u64, u32)`, so bytewise order = `Watermark` order |

- `Author` codes `0..3` = `CliUser, UiUser, Assistant, System`; part of layout v1 (a change is a new layout version).
- `pos INTEGER PRIMARY KEY` without `AUTOINCREMENT`: rows are never deleted and a rolled-back transaction consumes no rowid, so `max + 1` is enough.
- One identity with several contents = several `writes` rows with the same `(module, category, id, field, l, c, a)`. Uniqueness of `(identity, content)` is decided by the core (idempotence), not by an SQL constraint.
- **No codec.** A row decodes into a `Write` field by field; any type, length, range or UTF-8 failure is damage. RG-ST02-1.4 therefore stays a `@Property` on the store (round trip through SQLite and a reopening), not an `@Invariant` on an encode/decode pair.
- Generic (RG-ST02-7.4): no table or column knows a module, category or field name.

## Interfaces to define

### Port `Store` (`arbitre-core::store`)

```
#[allow(async_fn_in_trait)]
pub trait Store {
    async fn device(&self) -> Result<DeviceId, StoreError>;
    async fn apply_local(&self, pt: u64, changes: &[LocalChange]) -> Result<Vec<Write>, ApplyError>;
    async fn apply(&self, batch: &[Write]) -> Result<Report, ApplyError>;            // received
    async fn writes_of(&self, record: &RecordId) -> Result<Vec<Write>, StoreError>;
    async fn state(&self, record: &RecordId) -> Result<Option<RecordState>, StoreError>;
    async fn records(&self, module: &str, category: &str) -> Result<Vec<(RecordId, RecordState)>, StoreError>;
    async fn writes_since(&self, from: Position) -> Result<Vec<(Position, Write)>, StoreError>;
    async fn watermark(&self, peer: DeviceId) -> Result<Option<Watermark>, StoreError>;
    async fn advance_watermark(&self, peer: DeviceId, to: Watermark) -> Result<(), StoreError>;
}
```

- `&self` everywhere, no `Send`/`Sync` bound anywhere in the core (RG-ST02-6.2). Opening is **not** in the port: it is adapter-specific (path and options for SQLite, a worker for the browser).
- `pt` comes from the `Clock` port at the call site; the store never reads time.
- `apply_local` with an empty list is a no-op success (nothing to stamp); `n ≥ 1` is the specified case.
- `writes_of`: `Log::writes` order. `records`: sorted by `RecordId`, deleted records included. `writes_since`: position order.

### Errors

```
pub enum StoreError { StorageFull, Corrupted(Damage), Unavailable(Cause) }

pub enum Damage {
    StoredWrite { position: Option<Position> },  // undecodable, or fails validate
    ClockState,                                   // missing, undecodable, or K < a stored stamp
    DeviceId,                                     // missing or undecodable
    Layout,                                       // user_version missing/0 with tables, unknown table shape
    NotAStore,                                    // SQLITE_NOTADB, or foreign application_id / tables
    LocalIdentityPair { target: Target, stamp: Stamp },   // Q-A (b)
    Engine(String),                               // SQLITE_CORRUPT, short read
}

pub enum Cause { Busy, Io(String), IncompatibleVersion { stored: u32, supported: u32 }, RandomSource }

pub enum Rejection {
    Batch(BatchError),                            // received batch fails validate
    Local { index: usize, error: WriteError },    // EmptyIdentifier, ClockExhausted
    LocalIdentityReused { index: usize },         // Q-A guard on the received path
}

pub enum ApplyError { Rejected(Rejection), Store(StoreError) }   // From<StoreError>
```

- The set of storage errors stays exactly three (AC-6.6). Input rejections are a separate arm (CONFLICT-ST02-2): `Rejected ⇒ Σ unchanged`.
- `Corrupted` poisons the handle: the adapter remembers the `Damage` and every later operation on that handle returns it, before touching storage (RG-ST02-7.2, AMB-ST02-3).

### Pure store logic (`store.rs`, used by every adapter inside its write transaction)

```
pub fn rebuild(device: DeviceId, k: ClockState, known: &[Write]) -> Result<Log, Damage>;
pub fn stamp_local(log: &mut Log, pt: u64, changes: &[LocalChange]) -> Result<(Vec<Write>, ClockState), Rejection>;
pub fn receive(device: DeviceId, log: &mut Log, k: ClockState, batch: &[Write]) -> Result<Received, Rejection>;
pub struct Received { pub report: Report, pub new: Vec<Write>, pub clock: ClockState }
```

- `rebuild`: `Log::with_clock(clock from K)`, then `merge(known)`. `known` failing `validate` → `Damage::StoredWrite` (the adapter fills in the position from the index). A known stamp above `K` → `Damage::ClockState`. A known `Anomaly` whose `stamp.a = device` → `Damage::LocalIdentityPair`. Used by every record-scoped read too.
- `stamp_local`: calls `Log::write` per change, in list order, so `seen` includes the writes issued earlier in the same list (A-1). New `K` = last stamp's `(l, c)`. Any `WriteError` → `Rejection::Local`; the adapter rolls back, so no stamp is consumed.
- `receive`: `merge(batch)` (gives `BatchError` and the `Report`); if the report has a new anomaly with `stamp.a = device` → `Rejection::LocalIdentityReused` (index of the first batch write of that identity). This covers both "already stored with another content" and "two contents inside the batch". `new` = batch writes not already known, deduplicated, batch order. New `K = max_lex(K, max batch stamp)`, also when `new` is empty (RG-ST02-5.3).
- A local-device write received with an identity not known locally is accepted (a device re-receiving its own writes after a loss).

### Extension of an existing interface (GAP-ST02-1)

```
impl Log { pub(crate) fn with_clock(clock: HybridClock) -> Log }   // merge.rs
```

- `HybridClock` from `K`: `HybridClock::new(D)` then `receive(&Stamp { l, c, a: D })`, inside `store.rs`. No change to `hlc.rs` or `write.rs`, no public ST-01 signature changes.
- Impact on existing tests: none; the ST-01 suites run unchanged.
- Impact on mutation: the diff of `merge.rs` is a three-line constructor with no `Default` for `Log`, so cargo-mutants has no viable mutant there. `store.rs` is new Full code (85%): killed by `crates/core/tests/store.rs` (including the spec's "issue from a restored state = issue from the original") and by the contract suite on `SimStore`.
- `CONTRIBUTING.md` rule "a local write goes only through `Log::write`" stays true.

### Write paths (CONFLICT-ST02-1)

| | Local apply (`apply_local`) | Received apply (`apply`) |
|---|---|---|
| Input | `pt`, `[LocalChange]` | stamped `[Write]` |
| Transaction | `BEGIN IMMEDIATE` | `BEGIN IMMEDIATE` |
| Loads | `K`, writes of the touched records | `K`, writes of the touched records |
| Core call | `rebuild` → `stamp_local` | `rebuild` → `receive` |
| Stores | the `n` new writes (list order), `K'` | `Received.new` (batch order), `K'` |
| Returns | the stamped writes | `Report` |
| Rejections | `Local { index, WriteError }` | `Batch(BatchError)`, `LocalIdentityReused` |

Received apply serves peers (ST-19), import (ST-04b) and tests. The calling shell never stamps (Q-2).

### SQLite adapter (`arbitre-sqlite`)

```
pub struct Options { pub wait: Duration /* default 5 s, Q-3 */, pub vfs: Option<String> /* fault injection in tests */ }
pub struct SqliteStore { /* RefCell<rusqlite::Connection>, DeviceId, poisoned: RefCell<Option<Damage>> */ }
impl SqliteStore {
    pub fn open(path: &Path, options: &Options) -> Result<SqliteStore, StoreError>;
    pub fn settings(&self) -> Settings;   // journal mode, synchronous, fullfsync in effect (RG-ST02-1.1 test)
}
impl Store for SqliteStore { … }
```

- **One connection per handle**, in a `RefCell`. Every `async fn` body has no `.await`: the whole operation, waiting on a busy store included, runs in the first poll (RG-ST02-6.3). Two futures in flight on one handle are fine since each borrows the connection only while being polled.
- **Nothing cached between transactions** except `D` (immutable). `K` and writes are read inside each transaction (Q-2).
- **Per connection at open**: `busy_timeout(options.wait)`, `synchronous = FULL`, `fullfsync = ON`, `checkpoint_fullfsync = ON`. `journal_mode = WAL` is persistent and set at creation only.
- **Open sequence** (writes nothing unless creating):
  1. Read `application_id`, `user_version`, table list in a read transaction. `SQLITE_NOTADB` → `NotAStore`.
  2. No table and `user_version = 0` → creation: set WAL, `BEGIN IMMEDIATE`, re-check (another process may have created it), create tables, draw `D` with `getrandom` (failure → `Unavailable(RandomSource)`), `K = (0, 0)`, set `application_id` and `user_version = 1`, commit.
  3. Otherwise: foreign `application_id` or tables → `NotAStore`; `user_version > 1` → `Unavailable(IncompatibleVersion)`; `0` or missing rows → `Layout` / `DeviceId` / `ClockState`; `K` below the largest stored stamp (one indexed `ORDER BY l DESC, c DESC LIMIT 1`) → `ClockState`.
  - No full integrity scan at open (RG-ST02-7.2 allows detection by the first read reaching the damage). *Superseded by plan P-10: incremental row check (rows past `device.checked`) plus `PRAGMA quick_check` at open; full check on demand (`check_all`).*
- **Reads**: one `BEGIN DEFERRED` read transaction per operation (writes + seen rows from the same snapshot); WAL readers are not blocked by a writer (RG-ST02-3.5).
- **Writes**: `BEGIN IMMEDIATE` (waits up to `wait`), load, core call, insert, update, `COMMIT`. Any error → `ROLLBACK` (SQLite may or may not have rolled back itself after `SQLITE_FULL`/`IOERR`; the adapter always does it, ignoring "no transaction").
- **Error mapping**:

| SQLite result | Port error |
|---|---|
| `SQLITE_FULL` | `StorageFull` |
| `SQLITE_BUSY`, `SQLITE_LOCKED` | `Unavailable(Busy)` |
| `SQLITE_NOTADB` | `Corrupted(NotAStore)` |
| `SQLITE_CORRUPT`, `SQLITE_IOERR_SHORT_READ` | `Corrupted(Engine)` |
| decode failure of a row | `Corrupted(StoredWrite / ClockState / DeviceId)` |
| other `SQLITE_IOERR_*` (fsync included), `CANTOPEN`, `PERM`, `READONLY`, … | `Unavailable(Io)` |

An `ENOSPC` on fsync surfaces as `SQLITE_IOERR_FSYNC`, so it is `Unavailable`, not `StorageFull`: the spec allows it ("`Unavailable` otherwise") since the cause cannot be told apart.

### Simulated store (`sim_store.rs`, feature `sim`)

- `SimStore::new(device)`; Σ in a `RefCell`; same pure logic as SQLite (`rebuild`, `stamp_local`, `receive`); positions from 1.
- Fault injection: `full_after(units)` (the batch whose k-th inserted write exceeds the budget → `StorageFull`, nothing applied), `unavailable(n_ops)`, `crash_next_write()` (the operation is abandoned before its effects become visible, returns `Unavailable(Io)`, and the handle refuses everything until `reopen()`), `reopen()` (clears volatile state and poison, keeps Σ).
- It is the non-`Send`, non-`Sync` adapter of the compile test (RG-ST02-6.2). It compiles for wasm32 (`no_std`, `core::cell`).

## Test harnesses

| Requirement | Harness | Location |
|---|---|---|
| PROP-ST02-9 contract suite | Model-based: generated operation sequences (local apply, received apply with duplicates and peer anomalies, watermark advance, all reads, reopen) against a reference = ST-01 `Log` + list of stored writes + watermark map. Positions compared by order, not by value. Generic over a small test-only trait (`open fresh`, `reopen`) | `crates/core/tests/support/contract.rs`, run by `crates/core/tests/store.rs` on `SimStore` and by `crates/sqlite/tests/contract.rs` on `SqliteStore` (included with `#[path = "../../core/tests/support/contract.rs"]`, along with core's `support/mod.rs` generators) |
| E-ST02-1 forced stop | The test re-executes its own binary (`std::env::current_exe()`, `--exact <child test> --test-threads=1`, env var selects the child role). The child applies batches and prints one line per confirmation after the success result. The parent kills it with `Child::kill()` (SIGKILL on Unix) after a seeded random delay, reopens and checks. Each write carries a unique value (run, batch, index) so the all-or-nothing check works on local applies whose stamps are not known in advance. The reference includes at most one unconfirmed batch, only if it is present in full | `crates/sqlite/tests/crash.rs`, `tests/support/child.rs` |
| E-ST02-2 full storage | A fault VFS registered through `rusqlite::ffi`, delegating to the default VFS, with a shared byte budget: `xWrite` returns `SQLITE_FULL` once it is spent, and an optional `xSync` failure. Opened with `Options { vfs: Some(…) }`. Levels: budget 0 (before the batch, and first opening), budget inside the WAL frames of the batch (mid-batch), budget = all but the commit frame (at commit), during `advance_watermark`. Then the budget is lifted and the batch retried. In WAL mode all page writes of a transaction reach the file at commit, so this is the real `ENOSPC` path. `SimStore` uses `full_after` | `crates/sqlite/tests/full.rs`, `tests/support/full_vfs.rs` |
| E-ST02-3 interleaving | Same child mechanism, two writer children and one reader child, driven step by step over stdin/stdout by the test (go per operation). "Hold a transaction for t" is done by a child opening a raw `rusqlite` connection and holding `BEGIN IMMEDIATE`; the port needs no hook for it. Wait setting 200 ms, ε 1 s | `crates/sqlite/tests/interleave.rs` |
| E-ST02-4 damage | Build a store, close it (WAL checkpointed and removed), damage the main file from outside (raw `rusqlite` edits for (a)–(d), truncation, text file, foreign `application_id`, `user_version = 2`), open and read everything; check `Corrupted` / `Unavailable(IncompatibleVersion)` and that the file is byte-identical | `crates/sqlite/tests/damage.rs` |
| RG-ST02-1.1, 6.2, 6.3 | `settings()` values; every operation polled once with `Waker::noop()` is `Ready`, a busy one included; generic function over `S: Store` instantiated with `SimStore`; two futures created then polled | `crates/sqlite/tests/settings.rs`, `crates/core/tests/store.rs` |

- Temporary stores: a unique directory under `std::env::temp_dir()` per case; no `tempfile` dependency.
- Process-level harnesses take a seed (fixed default, overridable by env var) and print it on failure (replay).
- Run counts: those of the specs (GAP-ST02-2); the Planner measures CI time and may go down to the floor stated there.

## DDD classification and zones

| Component | Zone | C-TDD rigor |
|---|---|---|
| `store.rs`: port, errors, `rebuild` / `stamp_local` / `receive` | Core Domain | **Full** (mutation ≥ 85%) |
| `merge.rs` (`Log::with_clock`) | Core Domain | Full (unchanged zone) |
| `sim_store.rs` | Supporting (test simulator) | **Std** (≥ 70%), like `clock.rs` |
| `crates/sqlite/src/**` | Generic | **Gen** (informational score); blocking properties: E-ST02-1, E-ST02-2, E-ST02-3, E-ST02-4, RG-ST02-1.4, PROP-ST02-9 |

`.ctdd/stories/ST-02/zones.toml`:

```toml
Full = ["crates/core/src/store.rs"]
Std  = ["crates/core/src/sim_store.rs"]
Gen  = ["crates/sqlite/src/**"]
```

`crates/core/src/lib.rs` stays uncovered (Full by default, re-exports only). No existing file's zone is lowered, so the judge stays green.

## Dependencies

### Internal

| Dependency | Type | Coupling |
|---|---|---|
| `arbitre-sqlite → arbitre-core` | normal; dev with `features = ["sim"]` | Allowed edge (`graph.py`) |
| `store.rs → merge.rs, hlc.rs, write.rs` | in-crate | `Log::with_clock` (`pub(crate)`), `Log::write`, `Log::merge`, `validate`, `HybridClock::{new, receive}` |
| `Clock` port | caller side | `pt` is passed in; the store does not depend on `Clock` |

### External

| Dependency | Declared | Status | Risk |
|---|---|---|---|
| `rusqlite` 0.41.1, `features = ["bundled"]` | `crates/sqlite/Cargo.toml` only | **Local**, not shared (ST-01 dependency decision says it returns in `arbitre-sqlite` alone). **Not a foundation amendment.** | `bundled` compiles SQLite's C code with `cc`: a C compiler is needed natively (present on CI and macOS). Bundled SQLite ≥ 3.37 is required for `STRICT` (check the version shipped by `libsqlite3-sys` 0.39.1 when the Skeleton runs). |
| `getrandom` 0.4.3 | `{ workspace = true }` | Already shared | none |
| `proptest` 1.12.0 | dev, `{ workspace = true }` | Already shared | none |

No network crate (RG-ST02-G.1). No `tempfile`, no `libc`, no VFS crate: `std::process` gives SIGKILL, `rusqlite::ffi` gives the VFS.

## Constraints and limits

- **Backward compatibility**: no public ST-01 signature changes; ST-01 tests unchanged.
- **`arbitre-core` purity**: still no normal dependency; the port and `SimStore` are `no_std`; `20-st01-browser.sh` and graph rule (c) stay green.
- **Port frozen after ST-04**: the port includes `apply`'s `Report` and the watermark operations now to avoid an amendment later.
- **Durability**: `synchronous = FULL` in WAL mode syncs the WAL on each commit; `fullfsync` is what makes it reach the platter on macOS (plain `fsync` does not); a no-op elsewhere. Cost: a few ms per commit, accepted by Q-1.
- **WAL needs a local file system** (shared memory index). A store on a network share is unsupported; the path choice is ST-03's.
- **Closing the last connection checkpoints the WAL into the main file.** Damage tests must damage a cleanly closed store; an open on a damaged store with a non-empty WAL is not a case of the specs.
- **Q-A detection scope**: `LocalIdentityPair` is detected by operations that rebuild a record (`writes_of`, `state`, `records`, both applies). `writes_since` returns raw writes and does not group by identity, so it does not detect it (allowed by RG-ST02-7.2: damage is reported by the operation that reads it as such).
- **Performance**: every write loads the writes of its touched records; no target is set by the specs.
- **Migration**: none (first layout). A future layout change bumps `user_version`; v1 refuses newer stores with `IncompatibleVersion`.
- **Bounded context**: synchronization semantics of watermarks (ST-19), export/import (ST-04b), CLI wiring and messages (ST-03), namespace views (ST-04) and the browser adapter (ST-20) are out. The contract suite in `crates/core/tests/support/contract.rs` is the contract test the browser adapter will have to pass.

## Anti-patterns and risks

| # | Item | Location | Impact | Recommendation |
|---|---|---|---|---|
| R-1 | Mutation cost of the Gen crate | `50-st01-mutation.sh` mutates `crates/sqlite/src/**` from the diff and runs the whole `arbitre-sqlite` test suite per mutant, process harnesses included (hundreds of runs each) | CI time could reach hours, for a score that is informational | Keep `crates/sqlite/src` thin (logic lives in `store.rs`). See the open question. |
| R-2 | Fault VFS is unsafe FFI | `crates/sqlite/tests/support/full_vfs.rs` | Around a few hundred lines of `unsafe` test code delegating every I/O method | Test-only, outside the mutated tree. Fallback if it proves too costly: `PRAGMA max_page_count` covers levels (a), (b), (d) without `unsafe`, but not the commit-time path (c); that would weaken E-ST02-2 and must go back to the specs. |
| R-3 | Kill timing | E-ST02-1 | A uniform kill instant may rarely land inside an fsync | Many short batches and 200 runs; the Planner may bias delays towards commit duration, measured first. |
| R-4 | `INSIDE_MUTANTS` behavior not yet verified on cargo-mutants 27.1.0 | open question (a) | — | Check the variable name before relying on it. |

## Open questions for Quentin

**Q-M — Process-level harnesses under mutation testing (R-1).** cargo-mutants runs the full `arbitre-sqlite` suite for each mutant of `crates/sqlite/src`, whose score is informational (Gen).
- (a) The process-level harnesses (E-ST02-1, -2, -3) run a reduced count (for example 10 runs) when cargo-mutants runs them (it sets `INSIDE_MUTANTS`); the contract suite keeps its full count; normal CI runs keep the spec counts. Not an exclusion channel listed in `CONTRIBUTING.md`, but it is a test that behaves differently under mutation.
- (b) Accept the CI time as it comes.
- (c) Move the process harnesses to a test-only workspace crate that cargo-mutants does not mutate: needs a new row in `graph.py`'s allowed table, so a `ci-change` PR and a foundation amendment.
- **Recommendation: (a)**, documented in `CONTRIBUTING.md`. The blocking guarantee comes from the full-count runs in `10-st01-tests.sh`; the mutation run only needs the tests to fail fast on a broken adapter.

## Quentin's arbitration (2026-11-04)

Binding; overrides the open question above.

- **Q-M → (e): no mutation of Gen-zone files in CI.** Option (a) is void: cargo-mutants 27.1.0 does not set `INSIDE_MUTANTS` (checked on the installed binary). Since a Gen score has no threshold, `scripts/ci.d/50-st01-mutation.sh` excludes the files whose effective zone is Gen (patterns read from the `zones.toml` files, same resolution as the judge); Full and Std keep their thresholds. Gen mutation runs locally on demand, with the command documented in `CONTRIBUTING.md`.
- This is a **foundation amendment** (it changes a `*-st01-*` file and the CONTRIBUTING rule on Gen scores). It ships as its own PR with the `ci-change` label, merged before the ST-02 PR.
- Durability harnesses (crash, interleaving, full disk) stay in the normal test suite; no `#[ignore]`, no dedicated check script.
