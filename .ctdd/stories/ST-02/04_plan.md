# Implementation Plan — ST-02: Persistence, async `Store` port and SQLite adapter

## Traceability
- Specs: `.ctdd/stories/ST-02/02_specs.md`. Its "Quentin's arbitration (2026-11-04)" is binding: **Q-A → (b) plus a guard on the received path**.
- Archi: `.ctdd/stories/ST-02/03_archi.md`. Its "Quentin's arbitration (2026-11-04)" is binding: **Q-M → (e)**, no CI mutation of Gen-zone files, shipped as a separate `ci-change` PR merged before the ST-02 PR.
- Story: `.ctdd/stories/ST-02/01_story.md` (Q-1 to Q-5 arbitrations binding).
- Frame: `.ctdd/cadre/04_plan.md` § ST-02 (size M, arbitration "moyenne (2)", zone Gen with blocking properties); § "Amendement du socle (F-27)".
- Format: `.ctdd/stories/ST-01/04_plan.md`. Repo rules: `CONTRIBUTING.md`, `scripts/ci.sh`, `scripts/ci.d/*.sh`, `scripts/lib/{mutation,judge}.py`.
- DA: `.ctdd/stories/ST-02/05_da_review.md` (iteration 2, PASS WITH RESERVATIONS); iteration 1 in `history/05_da_review_v1.md`. Plan v1 and v2 archived as `history/04_plan_v1.md`, `history/04_plan_v2.md`.
- Quentin's arbitrations of 2026-11-04 (end of this file) are binding: Q-P5 → (a), Q-P1 → (b), Q-P2 → (a), Q-P3 → layout fix, Q-P4 → `Std`.
- Date: 2026-11-04 · Status: DRAFT v3.

## Resolution of DA v2 reservations

| Finding | Change in v3 | Section |
|---|---|---|
| **F-18 [M]** free-running red step targets an equivalent mutant | Red step retargeted at an observable split read: `writes_since` reads `seen` **before** `writes` (newer writes come back with missing `seen` entries, caught by `==` against the written write). Batch atomicity on `records`: each batch writes field `v` on every record of one category with value = batch tag, so a whole batch is visible iff all those records carry the same tag. Second observable mutant: `records` with one `SELECT` per record outside a transaction. Structural rule kept in T-16 | T-11, R-10 |
| **F-19 [M]** Q-4 (a) held per handle only | Settled by **Q-P5 → (a)**: open-time check of every stored write (P-10), built in T-13, red step across two processes; `SimStore` damage survives `reopen()` (T-7); 100 ms budget at 10,000 writes measured in T-15 with the escalation rule. The "for information" note and the PR body note are removed | P-10, T-7, T-13, T-15, T-16 |
| F-20 [m] non-colliding orphan `seen` row grafted onto the next write | Detected at open (P-11), part of the open-time check: `Corrupted(StoredWrite { position })`. T-13 tests both the non-colliding orphan (refused at open) and the colliding one injected after open (`Corrupted(Engine)`, rolled back) | P-11, T-13 |
| F-21 [m] R-9 fallback can leave a store out of WAL | If R-9 moves the WAL switch, it runs at **every** open (no-op when already WAL). US-4 case 4 also checks `settings().journal_mode = WAL` after reopen | R-9, T-10 |
| F-22 [m] `Harness` signature inconsistent | Pinned in the Convention: `fn fresh() -> Self; fn store(&self) -> &Self::Store; fn reopen(&mut self)` | Convention, T-6, T-9 |
| DA notes on F-3, F-6, F-7 | Stated: the reference merges on a clone; T-9's local loop passes `--features sim`; T-12 resets the budget per case | T-6, T-9, T-12 |

## Resolution of DA v1 findings

| Finding | Change in v2 | Section |
|---|---|---|
| **F-1 [B]** `seen` cannot store a `Field("")` key | Layout v1 fixed in the adapter, core untouched: `seen` gets a `kind INTEGER` column (0 = `Delete`, 1 = `Field`), PK `(pos, kind, field)`. Write generator draws `seen` keys from {`Delete`, own field, other field, `Field("")`}. Named US-1 case 6b. Full SQLite error mapping written out, `SQLITE_CONSTRAINT` included, with a per-row unit test | P-7, P-8, T-8, T-9, T-6, T-13 |
| F-2 [M] "known stamp above `K`" not pinned to `(l, c)` | Comparison stated on `(l, c)` only. Named boundaries in T-4 (`(l, c) = K`, `a > D` → no damage; `c + 1` → damage) and in the contract suite (received write at exactly the new `K`, then read) | T-4, T-6 |
| F-3 [M] Q-A guard not tested through adapters | Named guard cases on `SimStore` (T-7) and SQLite (T-9): `Rejected(LocalIdentityReused)`, `Σ` unchanged, handle not poisoned, reopen OK; unknown local identity accepted, `K` raised. Generator includes local-device received writes; the reference implements the guard | T-6, T-7, T-9 |
| F-4 [M] interleaving harness misses the stamp race | Coordinator sends `pt` to each writer from a small domain with equal values and rewinds. Red step 1: cache `K` in the handle; red step 2: `BEGIN DEFERRED` (caught by the busy checks). RG-ST02-3.5: free-running phase (reader loops while a writer commits batches in a loop) plus the code-shape rule stated | T-11 |
| F-5 [M] poisoning hides which read detects damage | T-13 uses one fresh handle per operation, expected result per operation. T-9 states `writes_since` decodes **and** validates each row | T-9, T-13 |
| F-6 [M] `#[cfg(feature = "sim")]` in an included `support/mod.rs` | `arbitre-sqlite` declares `[features] sim = ["arbitre-core/sim"]` (dev path only); the SQLite `contract` test target has `required-features = ["sim"]`, like core's `hlc`/`convergence`. Contrary under-engineering bullet deleted | P-9, T-3, T-9 |
| F-7 [M] shared fault-VFS budget | One VFS name per test, budget reached through `pAppData`; `io_methods` copied per opened file and kept alive with it; each level asserts the budget was crossed | T-12 |
| F-8 [M] `Report` and stamped writes unchecked | Contract suite compares `apply`'s `Report` with the reference's, and `apply_local`'s writes with the reference's `Log::write` results and with the next `writes_since` | T-6 |
| F-9 [M] concurrent first opening: one run, no red | Repeated N = 50 seeded runs, both children released by one "go"; red step "skip the re-check inside `BEGIN IMMEDIATE`". Loser's `Busy` is a failure (spec: "both handles report it"); fix path stated | T-10, R-9 |
| F-10 [m] `xSync` assertion too strong | Accepted: `Σ_live = Σ_before` on the same handle, `Σ ∈ {before, before ⊕ b}` after reopen; level (c) budget from frame boundaries without padding | T-12 |
| F-11 [m] `mapfile` swallows listing failure | Accepted: listing written to `$CI_OUT/gen.txt` with `|| exit 1`, then `mapfile < file`; shell demonstration added | T-1 |
| F-12 [m] `checkpoint_fullfsync` not asserted | Accepted: `settings()` returns and the test asserts all four values | T-8 |
| F-13 [m] untested error cases | Accepted: `δ = 0` in T-9; `P = Position(u64::MAX)` named contract case; `apply`/`apply_local` on the damaged record in T-13. `RandomSource`: **deferred, untested** — no injection seam in Gen code, and adding an argument to `open` for one test is not worth it; stated in the PR body | T-6, T-9, T-13 |
| F-14 [m] `sim_store.rs` `Std` vs specs `Full` | Q-P4 → `Std` (arbitrated) | T-3, T-7 |
| F-15 [m] who writes `contract.rs` and T-7 tests | Stated: the Skeleton writes all red unit tests (T-4, T-5, T-7) and a `contract.rs` stub with its entry point and both call sites; T-6 owns the suite body only | Convention, T-6, collisions |
| F-16 [m] timing measured on the wrong platform | Accepted: T-15 takes timings from the PR 2 CI run (Linux), applies Q-P1 to `10` measured there, records `50`'s duration and mutant count | T-15 |
| F-17 [m] Q-4 (a) only for damage visible at open | Superseded by Q-P5 → (a) (see DA v2 table) | P-10, T-13 |

## Technical context
- **Codebase** (read 2026-11-04, `main` = `d6c2ec4`):
  - `crates/core` (`arbitre-core`): `#![no_std]` + `alloc`, no normal dependency; `hlc.rs`, `write.rs`, `merge.rs` (Full), `clock.rs` (Std, feature `sim`); tests in `crates/core/tests/` with `support/mod.rs` (`config()`: 1,000 cases, `SEED`; generators and oracle under `#[cfg(feature = "sim")]`); test targets `hlc`, `convergence` declare `required-features = ["sim"]`.
  - `crates/cli`: clap stub, zone Gen (`.ctdd/stories/ST-01/zones.toml`). Not touched.
  - `Log` owns a private `HybridClock` at `(0, 0)`; `Log::write` is the only local write path (GAP-ST02-1).
  - `write.rs::validate` checks `seen` **values** (`< stamp`), not `seen` keys; the change's field name must be non-empty (`identifiers_non_empty`). So a valid write may carry `seen[Field("")]` (DA F-1).
  - CI: `scripts/ci.sh` runs `ci.d/10` (tests, `--all-features`), `20` (wasm32 on core), `30` (graph, `arbitre-sqlite → arbitre-core` already allowed), `40` (gitleaks), `50` (`cargo mutants --in-diff`, verdict by `mutation.py`; `set -uo pipefail`, no `-e`). Warnings do not fail CI. The judge (`judge.py`, run from `main`) reds any PR touching `scripts/` or `.github/` without `ci-change`, and any `ci-change` PR touching `crates/`.
- **Stack**: Rust 1.99, edition 2024; `rusqlite` 0.41.1 `bundled` (local to `arbitre-sqlite`); `getrandom` 0.4.3 and `proptest` 1.12.0 (workspace); Python ≥ 3.13 stdlib; cargo-mutants 27.1.0.
- **Patterns to follow**: functional core / imperative shell (store semantics pure in `store.rs`, adapters only do I/O); port + real adapter + simulator (`Clock`/`SimClock` → `Store`/`SimStore`/`SqliteStore`); validation before effect; fixed-seed property tests from `support/mod.rs`; `required-features = ["sim"]` on test targets that need the simulator; zones per story, strictest wins.
- **cargo-mutants facts checked in its 27.1.0 source** (for T-1): an `--exclude` glob containing `/` matches the whole relative path (`glob.rs`), so an exact file path works; a diff whose only Rust changes are in excluded files ends with `NoMutants` → exit 0, no `outcomes.json`, which `50-st01-mutation.sh` already treats as neutral (`in_diff.rs`). With mutants only in `arbitre-core`, only `arbitre-core`'s tests run per mutant, so the SQLite process harnesses never run under mutation.

## Decisions taken by this plan (within the archi's latitude)

| # | Decision | Why |
|---|---|---|
| P-1 | `schema.rs` is merged into `crates/sqlite/src/lib.rs` | Fewest files; the archi allows it; Gen, not mutated in CI after T-2. Split later only if the file passes ~600 lines |
| P-2 | No `tests/support/mod.rs` of its own in `crates/sqlite`: each test file includes its helpers with `#[path = "support/<x>.rs"] mod <x>;` | Lets T-10, T-11, T-12 and T-13 run in parallel without colliding on a shared `mod.rs` |
| P-3 | The contract suite is a plain proptest over a generated `Vec<Op>` driven against the reference model | `proptest-state-machine` would be a new dependency (foundation amendment); not needed |
| P-4 | Gen exclusion passes **resolved file paths** (one `--exclude <path>` per Gen `.rs` file), not the `zones.toml` patterns | Pattern semantics differ (`full_match` vs cargo-mutants globs) and a file covered by Gen and Std must stay mutated (strictest wins). Same resolution as the judge: `zone_of` |
| P-5 | "Unreadable store" (US-5 case 11) is tested with a **directory** at the store path, not `chmod 000` | Works when CI runs as root; `SQLITE_CANTOPEN` → `Unavailable(Io)` |
| P-6 | Busy-store timing (US-3 cases 6–8, US-6 case 5) is tested **in-process** with two connections in T-9, and again cross-process in E-ST02-3 (T-11) | SQLite locks are per connection; the in-process test is cheap and deterministic |
| **P-7** | **Layout v1, table `seen`: amended** (DA F-1). Columns `pos INTEGER`, `kind INTEGER`, `field BLOB`, `l`, `c`, `a`. `kind` 0 = `TargetKind::Delete` with `field = x''`; `kind` 1 = `TargetKind::Field(name)` with `field` = the name's bytes, **empty allowed**. Constraints: `CHECK (kind IN (0, 1))`, `CHECK (kind = 1 OR field = x'')`, `PRIMARY KEY (pos, kind, field)`, `WITHOUT ROWID`. Decode: unknown `kind`, or `kind = 0` with non-empty `field` → `Damage::StoredWrite`. `writes` is unchanged (`field = x''` ⇔ deletion stays unambiguous there, since `validate` forbids an empty change field name) | The map `TargetKind → (kind, field)` is injective, so every `Seen` (a `BTreeMap` key set) fits the PK and round-trips with `==`. **Not** changing `validate` in the core: that would change ST-01 behaviour in a Full zone (a write valid today becomes rejected, peers included) for an adapter encoding problem; the adapter is the one with the defect. The archi's layout table is superseded on this row only; `03_archi.md` is not edited (Q-P3 → layout fix, arbitrated) |
| **P-8** | **SQLite error mapping, closed** (archi table completed). One function `map(rusqlite::Error) -> StoreError`, exhaustive, with a final catch-all: see table below | No rusqlite error can escape the three-arm set (AC-6.6). Every row is unit-tested |
| **P-10** | **Open-time check (Q-P5 → (a))**, step 4 of the open sequence, after the archi's steps 1–3, in the same read transaction as the `K` check: (1) every `writes` row and its `seen` rows go through the **same decode + `validate` function as `writes_since`** → `Corrupted(StoredWrite { position })`; (2) one `GROUP BY module, category, id, field, l, c HAVING count(*) > 1` restricted to `author = D` → `Corrupted(LocalIdentityPair)`; (3) P-11. Runs on every open, creation excepted (empty store). Supersedes the archi's "No full integrity scan at open"; `03_archi.md` is not edited | Quentin's arbitration: a damaged store refuses to open in every process. Reusing `writes_since`'s path keeps one decoder. Cost O(n) per open, budget 100 ms at 10,000 writes (T-15). Damage found later by a read (introduced while a handle is open) still poisons that handle; the next open then refuses the store |
| **P-11** | **Orphan `seen` rows detected at open** (DA F-20): `SELECT pos FROM seen WHERE pos > (SELECT max(pos) FROM writes) LIMIT 1` → `Corrupted(StoredWrite { position: pos })` | One indexed query inside P-10. An orphan injected after open and colliding with a new write still ends as `Corrupted(Engine)` via P-8; a non-colliding one injected after open is caught at the next open |
| **P-9** | `arbitre-sqlite` declares `[features] sim = ["arbitre-core/sim"]`; its dev-dependency on `arbitre-core` has `features = ["sim"]`; its `[[test]] contract` has `required-features = ["sim"]` (DA F-6) | `support/mod.rs` uses `#[cfg(feature = "sim")]`, which is evaluated against the **including** crate. Declaring the feature makes the include compile the same way as in core, with no `unexpected_cfgs` warning, and lets `contract.rs` reuse core's generators. Harmless under `--all-features`: it only enables the simulator in `arbitre-core`, which no normal build of `arbitre-sqlite` uses. Same pattern as core's `hlc` and `convergence` targets |

**P-8 — SQLite error mapping (complete).** Applied in this order; first match wins.

| Source | Port error | Note |
|---|---|---|
| `SQLITE_FULL` | `StorageFull` | |
| `SQLITE_BUSY` (all extended codes, `BUSY_SNAPSHOT` included), `SQLITE_LOCKED` | `Unavailable(Busy)` | |
| `SQLITE_NOTADB` | `Corrupted(NotAStore)` | |
| `SQLITE_CORRUPT` (all extended), `SQLITE_IOERR_SHORT_READ` | `Corrupted(Engine)` | |
| `SQLITE_CONSTRAINT` (all extended: `CHECK`, `PRIMARYKEY`, `UNIQUE`, `NOTNULL`, `DATATYPE`) | `Corrupted(Engine("constraint: …"))` | After P-7, a write that passed the core checks cannot violate a constraint unless stored rows contradict the layout (e.g. an orphan `seen` row at the next position). Retrying cannot help, so not `Unavailable`. The transaction is rolled back first, so `Σ` is unchanged |
| rusqlite decode errors (`FromSqlConversionFailure`, `InvalidColumnType`, `IntegralValueOutOfRange`, `Utf8Error`) and the adapter's own field checks | `Corrupted(StoredWrite { position } / ClockState / DeviceId)` by the table being read | |
| other `SQLITE_IOERR_*` (fsync included), `SQLITE_CANTOPEN`, `SQLITE_PERM`, `SQLITE_READONLY`, `SQLITE_NOMEM`, `SQLITE_PROTOCOL` | `Unavailable(Io(text))` | `ENOSPC` on fsync surfaces as `IOERR_FSYNC` (archi) |
| anything else (other codes, `InvalidPath`, `NulError`, …) | `Unavailable(Io(text))` | Catch-all; keeps the set closed |

## Pre-plan analysis

### Over-engineering detected
- **`Options.vfs: Option<String>`** is a public field that only tests use. Kept: it is the one-line way to open with a fault VFS (`Connection::open_with_flags_and_vfs`), and the alternative (a test-only constructor behind a feature) is more code. Doc comment: "test hook".
- **`SqliteStore::settings()`** exists only for the RG-ST02-1.1 test. Kept: `synchronous`, `fullfsync`, `checkpoint_fullfsync` are per-connection, so a second raw connection cannot read them.
- **Six SQLite test files** (archi tree): kept, because they are separate test binaries that run in parallel and give parallel agents disjoint files. No other split.
- **`RandomSource` injection seam** (DA F-13): not added. One untested error branch (a `getrandom` failure) does not justify a new parameter on `open`. Stated in the PR body.

### Under-engineering detected
- **STRICT tables and CHECK constraints block the damage tests.** E-ST02-4 (a)/(b)/(d) cannot be produced by plain SQL on a STRICT table with CHECKs (wrong type, `author = 7`, `length(a) ≠ 16`). The damaging connection uses `PRAGMA ignore_check_constraints = ON` and type-valid but semantically bad values (non-UTF-8 identifier BLOB, 15-byte device, `l ≥ L_MAX`, `c > u32::MAX`, `seen ≥ stamp`, `seen.kind = 2`). The adapter still decodes and validates every field: constraints are a second line, not the check.
- **`cargo test` runs tests on several threads**, and the process harnesses spawn children and fsync heavily: the timing assertions (`δ ≤ wait ≤ δ + ε`) can flake under load. Each timing test uses its own store directory, `ε = 1 s`, and the timing schedules run sequentially inside one test function.
- **Process-global state in tests**: `sqlite3_vfs_register` is global, so the fault VFS never shares a budget between tests (T-12, one VFS name per test).
- **Child re-exec**: the child is selected by an env var and `--exact <child test> --test-threads=1 --nocapture`; a child test run without the env var is a no-op success, so `cargo test` alone never runs a stray child.
- **`writes_since` bypasses `rebuild`** (archi): without an explicit per-row `validate`, it could return a stored write that fails `validate` as if it were valid (RG-ST02-5.5). T-9 makes it decode **and** validate each row.

### Identified risks

| # | Risk | Probability | Impact | Mitigation |
|---|---|---|---|---|
| R-1 | Fault VFS (`unsafe` FFI, every I/O method delegated) costs more than planned | Medium | T-12 slips | Delegate through the default VFS stored in `pAppData`; per-file `io_methods` copy overriding only `xWrite`/`xSync`. Fallback `PRAGMA max_page_count` loses level (c) "at commit": that weakens E-ST02-2 and goes back to Quentin (archi R-2) |
| R-2 | CI time of `10-st01-tests.sh` too high (1,000 SQLite sequences with fsync per commit, 250 crash runs, 50 concurrent creations, 400 full cases, 100 schedules + free-running phase; every reopen now runs the P-10 check, on small stores) | Medium | Slow PRs | Measured on the PR 2 CI run (T-15); lowering rule = Q-P1 → (b). Never below the specs' floor (half) |
| R-3 | Timing flakiness of busy-store tests on a loaded runner | Medium | Red CI by chance | P-6; ε = 1 s; failing seed printed for replay |
| R-4 | Bundled SQLite of `libsqlite3-sys` 0.39.1 older than 3.37 (`STRICT`) | Low | Layout v1 invalid | Checked in T-3 (`SELECT sqlite_version()`); fallback: drop `STRICT`, keep CHECKs (no spec impact) |
| R-5 | Mutation score of `store.rs` < 85% (bounds in `max_lex`, idempotence, Q-A guard) | Medium | Story blocked | Named boundary tests in T-4/T-5 (F-2 boundaries included); equivalents justified in `.ctdd/stories/ST-02/mutants_equivalents.toml`, reviewed by Quentin (T-16) |
| R-6 | Kill instants rarely land inside an fsync (archi R-3) | Medium | E-ST02-1 weaker than it looks | T-10 measures commit duration first and draws half of the kill delays inside `[0, 2 × commit]` after a batch starts |
| R-7 | In WAL mode, SQLite may spill frames before commit (`cache_spill`); level (b) "after the k-th write" maps to a byte budget only approximately | Low | Levels (b) and (c) differ less than intended | Both must give `StorageFull` with `Σ` unchanged; each test asserts the budget was crossed strictly inside the batch's frames (WAL size before/after) |
| R-8 | The ST-02 PR is opened before the `ci-change` PR is merged | Low | 50 mutates `crates/sqlite/src` for hours | T-16 starts with `git rebase origin/main` and checks that `50-st01-mutation.sh` on the branch contains the Gen exclusion |
| R-9 | Concurrent first opening: `PRAGMA journal_mode = WAL` runs outside `BEGIN IMMEDIATE` and may return `SQLITE_BUSY` to the loser without honouring `busy_timeout` | Medium | US-4 case 3 red: loser gets `Busy` | If T-10 shows it: the adapter sets WAL **after** the creation commit, and the switch then runs at **every** open (no-op when already WAL), retried until `wait` elapses, so a kill between commit and switch cannot leave the store out of WAL (DA F-21). Change confined to `src/lib.rs` |
| R-10 | The free-running reader phase (T-11) is CI-time heavy or its race window is rarely hit | Medium | RG-ST02-3.5 weaker than it looks | 2 s per seed, 10 seeds; red steps are observable split reads (DA F-18), each must be seen red locally; structural rule (one `BEGIN DEFERRED` per read) reviewed in T-16 |
| R-11 | The P-10 open-time check exceeds 100 ms at 10,000 writes on CI | Low | Q-P5 budget broken | Measured in T-15; if exceeded, **stop** and ask Quentin: narrow Q-4 to per-handle poisoning, or an incremental check. No tuning attempt beyond the obvious (one statement for all rows, prepared once) before asking |

## PR split

| PR | Branch (from `main`) | Content | Agent does | **Quentin does** |
|---|---|---|---|---|
| **PR 1 — `ci-change` amendment** | `st-02/gen-mutation` | `scripts/lib/mutation.py`, `scripts/lib/test_ci.py`, `scripts/ci.d/50-st01-mutation.sh`, `CONTRIBUTING.md` (Zones, Equivalent mutants). **Nothing under `crates/`** | T-1, T-2: code, local checks, push and open the PR on Quentin's go | Read the diff; **set the `ci-change` label** (agents never do); check `ci` and `judge` green; **merge** (squash, message written by hand) |
| **PR 2 — ST-02** | `st-02/store`, rebased on `main` **after PR 1 is merged** | `crates/core/**`, `crates/sqlite/**`, `Cargo.lock`, `CONTRIBUTING.md` (glossary rows), `.ctdd/stories/ST-02/**` (cycle docs, `zones.toml`, `mutants_equivalents.toml` if any). **Nothing under `scripts/` or `.github/`** | T-3 to T-16 | **No label.** Review equivalents if any, the measured CI time (Q-P1), `mutation.md`; check `ci` and `judge` green; **merge** (squash, hand-written message) |

- PR 1 has no Rust diff: its mutation check is neutral. Its judge is red until the label is set, then green (no `crates/` path).
- PR 1 can be opened, reviewed and merged while the ST-02 code is being written: it blocks only T-14 and T-16.
- F-27 procedure: Q-P2 → (a), no mini-cycle; Quentin's Q-M arbitration is the strong arbitration.

## Tasks

**Convention (who writes what, DA F-15).** The Skeleton (phase 6) lays down the types and signatures of the archi (with P-7, P-8, P-9) and writes:
- the **red** unit tests of T-4, T-5 and T-7 in `crates/core/tests/store.rs` (they compile and fail);
- a stub `crates/core/tests/support/contract.rs` exposing the suite's entry point (`run::<H: Harness>()`) and the `Harness` trait with this exact shape (DA F-22): `type Store: Store; fn fresh() -> Self; fn store(&self) -> &Self::Store; fn reopen(&mut self)` — the harness owns the handle (and, on SQLite, the path; `reopen` drops the handle, then opens again and panics on failure) — plus **both** call sites (`crates/core/tests/store.rs` on `SimStore`, `crates/sqlite/tests/contract.rs` on `SqliteStore`), so the call sites exist and fail from the start.

T-6 then owns only the body of `contract.rs`. For T-8 and T-9 the task writes its own red tests (Gen crate). For T-10 to T-13 the harness *is* the test: the task writes it first, runs it red against a deliberately broken adapter step (named per task), then makes it green; the broken step is never committed. "Green" for every task also means `cargo test --workspace --all-features --locked`, `20-st01-browser.sh` and `30-st01-graph.sh` pass.

### PR 1 — `ci-change` amendment

### T-1: Exclude Gen-zone files from CI mutation
- **DDD zone**: Generic (CI tooling)
- **Spec refs**: archi Q-M → (e); RG-ST02-G.2; CONTRIBUTING "Equivalent mutants" (only exclusion channel), "Zones"
- **Dependencies**: none (branch `st-02/gen-mutation` from `main`, own worktree)
- **Red**: in `scripts/lib/test_ci.py`, tests of a new `gen_files(paths, zones)`:
  - `crates/cli/src/main.rs` under `Gen = ["crates/cli/src/**"]` → listed;
  - a file covered by both `Gen` and `Std` → **not** listed (strictest wins);
  - an uncovered file → not listed (Full);
  - `crates/sqlite/src/lib.rs` under `Gen = ["crates/sqlite/src/**"]` → listed;
  - and one graph test for US-6 case 7: an edge `arbitre-plan → arbitre-sqlite` is red.
  - `python3 -m unittest scripts/lib/test_ci.py` fails (function missing).
- **Green**:
  - `mutation.py`: `gen_files` = paths whose `zone_of(...) == "Gen"` (reuses `read_zones`, `zone_of`); `main` accepts `--gen-files` and prints, one per line, the tracked `crates/**/*.rs` files resolved to Gen, then exits; no other behaviour change.
  - `50-st01-mutation.sh`: before `cargo mutants`, write the list to a file — `python3 scripts/lib/mutation.py --gen-files > "$CI_OUT/gen.txt" || exit 1` — then `mapfile -t gen < "$CI_OUT/gen.txt"` and append `--exclude <path>` per entry to the existing command line (the only place `--exclude` is allowed). Not a process substitution, whose exit status is lost (DA F-11).
  - `CONTRIBUTING.md`: in "Zones", `Gen` = "not mutated in CI (excluded by `50-st01-mutation.sh`, same resolution as the judge); local run: `cargo mutants --all-features -f 'crates/sqlite/src/**'`"; in "Equivalent mutants", the "only exclusion channel" sentence names the Gen exclusion as the one automatic exception.
- **Deliverable**: `scripts/lib/mutation.py`, `scripts/lib/test_ci.py`, `scripts/ci.d/50-st01-mutation.sh`, `CONTRIBUTING.md`.
- **Completion criterion**:
  - unit tests green;
  - local demonstration on a throwaway branch on top (never pushed): a diff touching `crates/cli/src/main.rs` (Gen) and `crates/core/src/clock.rs` (Std); `cargo mutants --list --in-diff <diff> --all-features` with the computed `--exclude`s lists only `clock.rs` mutants; with only the `main.rs` change, the full `50-st01-mutation.sh` is green (neutral, no `outcomes.json`);
  - shell demonstration of F-11: with `mutation.py --gen-files` made to exit 1 (temporary edit, not committed), `50-st01-mutation.sh` exits non-zero;
  - `scripts/ci.sh` green on `st-02/gen-mutation`;
  - `git diff main --name-only` contains no `crates/` path.
- **Effort**: agent 22 min · arbitration 0

### T-2: Open, label and merge PR 1
- **DDD zone**: Generic
- **Spec refs**: CONTRIBUTING "Branches and PRs", "The judge"; archi Q-M arbitration
- **Dependencies**: T-1
- **Description**: on Quentin's go, push `st-02/gen-mutation` and open PR 1. Body: the Q-M arbitration, what changes, the demonstrations of T-1, "foundation amendment". **Quentin** sets `ci-change`, reviews, merges.
- **Deliverable**: PR 1 merged into `main`.
- **Completion criterion**: `judge` red before the label (lists `scripts/…` paths), green after; `ci` green; merged.
- **Effort**: agent 5 min · arbitration 10 min (read diff, label, merge)

### PR 2 — ST-02

### T-3: Scaffold `arbitre-sqlite`, core test target, zones
- **DDD zone**: Generic
- **Spec refs**: RG-ST02-6.1 (INV-ST02-1), RG-ST02-6.6 (INV-ST02-5), RG-ST02-G.1 (INV-ST02-7), RG-ST02-G.2
- **Dependencies**: none (part of the Skeleton phase)
- **Description**:
  - `crates/sqlite/Cargo.toml`: `arbitre-core` (path), `rusqlite = { version = "0.41.1", features = ["bundled"] }` (local), `getrandom = { workspace = true }`; dev: `arbitre-core` with `features = ["sim"]`, `proptest = { workspace = true }`; `[features] sim = ["arbitre-core/sim"]` and `[[test]] name = "contract"`, `required-features = ["sim"]` (P-9).
  - `crates/core/Cargo.toml`: `[[test]] name = "store"`, `required-features = ["sim"]`.
  - `crates/core/src/lib.rs`: `mod store;`, `#[cfg(feature = "sim")] mod sim_store;`, re-exports.
  - `.ctdd/stories/ST-02/zones.toml` as in the archi (`Full` store.rs, `Std` sim_store.rs per Q-P4, `Gen` `crates/sqlite/src/**`).
  - Types and signatures of the archi (`Store`, `LocalChange`, `ClockState`, `Position`, `Watermark`, `StoreError`, `Damage`, `Cause`, `Rejection`, `ApplyError`, `Received`, `rebuild`/`stamp_local`/`receive`, `SimStore`, `SqliteStore`, `Options`, `Settings` with four fields: journal mode, `synchronous`, `fullfsync`, `checkpoint_fullfsync`) with bodies left for later tasks.
  - `Cargo.lock` regenerated by `cargo check --workspace`.
- **Deliverable**: files above.
- **Completion criterion**:
  - `cargo check --workspace --all-features --locked` green with no `unexpected_cfgs` warning; `20-st01-browser.sh` and `30-st01-graph.sh` green (INV-ST02-1, 5);
  - `cargo tree -p arbitre-sqlite -e normal` shows no network crate (INV-ST02-7), and no `arbitre-core` `sim` feature without `--features sim`;
  - bundled `sqlite_version()` ≥ 3.37 recorded in the task report (R-4);
  - judge locally: `python3 scripts/lib/judge.py main HEAD` green without label (no zone lowered).
- **Effort**: agent 15 min · arbitration 0

### T-4: Restored clock, `rebuild`, `stamp_local` (`store.rs` part 1)
- **DDD zone**: Core (Full)
- **Spec refs**: GAP-ST02-1; RG-ST02-1.6 (PROP-ST02-5); RG-ST02-3.1, 3.2; RG-ST02-6.4 (INV-ST02-3); RG-ST02-6.5 (INV-ST02-4, local part); RG-ST02-7.2 (b), (d), Q-A (b)
- **Dependencies**: T-3
- **Red** (`crates/core/tests/store.rs`, written by the Skeleton):
  - "issue from a restored state = issue from the original": for any sequence of issues/receives on a `HybridClock`, a clock restored from its `(l, c)` issues the same next stamp (property, 1,000 cases);
  - `stamp_local`: `K' = (sₙ.l, sₙ.c)`; stamps strictly increasing in list order; `seen` of the i-th change covers the earlier changes of the same list (A-1: same field, deletion, deletion after field); US-1 case 8 (`pt = 0` after `K = (10⁹, 3)` → `(10⁹, 4, D)`); every stamp has `a = D` (US-4 case 6);
  - rejections: US-3 case 10 (2nd change with empty field → `Rejection::Local { index: 1, Invalid(EmptyIdentifier) }`), `ClockExhausted` at `c = u32::MAX` / `l ≥ L_MAX` → rejected; in both cases the input `Log` and `K` are unchanged;
  - `rebuild`: a known write failing `validate` → `Damage::StoredWrite`; a known local-device pair (two contents, `stamp.a = D`) → `Damage::LocalIdentityPair`; a peer pair → no damage, reported as an anomaly;
  - `rebuild`, clock check **on `(l, c)` only** (DA F-2): known peer stamp with `(l, c) = K` and `a > D` → no damage; `(l, c) = K` with `a < D` → no damage; `(l, c + 1)` above `K` → `Damage::ClockState`; `(l + 1, 0)` with `K = (l, u32::MAX)` → `Damage::ClockState`.
- **Green**: `pub(crate) fn Log::with_clock` in `merge.rs` (only change there); `ClockState` ↔ `HybridClock` via `new` + `receive` inside `store.rs`; `rebuild` compares `max_lex{(w.l, w.c)}` with `(K.l, K.c)`, never a full `Stamp`; `stamp_local` as in the archi.
- **Deliverable**: `crates/core/src/merge.rs` (one constructor), `crates/core/src/store.rs`.
- **Completion criterion**: tests above green; ST-01 suites unchanged and green; `git diff main -- crates/core/src/{hlc,write,clock}.rs` empty and `merge.rs` diff = the constructor only (INV-ST02-3); `grep -n "store" crates/core/src/{merge,hlc,write}.rs` finds no reference to the port.
- **Effort**: agent 22 min · arbitration 0

### T-5: Received path, idempotence, Q-A guard (`store.rs` part 2)
- **DDD zone**: Core (Full)
- **Spec refs**: RG-ST02-1.6 (received), RG-ST02-5.3 (idempotence, late write), RG-ST02-6.5 (`BatchError`), Q-A guard; CONFLICT-ST02-1
- **Dependencies**: T-4 (same file)
- **Red** (`crates/core/tests/store.rs`, written by the Skeleton):
  - US-1 case 9: `K = (10⁹, 0)`, batch with `l = 10¹²` → `K' = (10¹², c)`; next local stamp above it;
  - US-5 case 7: same batch twice → second `new` empty, `K'` still `max_lex(K, max batch)` (boundaries: batch max `<`, `=`, `>` `K`);
  - `new` = unknown writes, deduplicated, batch order;
  - invalid batch (US-6 case 6, `seen` not before stamp) → `Rejection::Batch`, `Log` and `K` unchanged;
  - Q-A guard: local-device write whose identity is stored with another content → `LocalIdentityReused { index }` (index of the first write of that identity); two contents of one local identity inside the batch → same; local-device write with an unknown identity → accepted; peer write with a second content → accepted, reported as anomaly (US-5 case 4).
- **Green**: `receive` and `Received` as in the archi.
- **Deliverable**: `crates/core/src/store.rs`.
- **Completion criterion**: tests above green; `cargo mutants --in-diff` locally on `store.rs` ≥ 85% (early read of R-5).
- **Effort**: agent 15 min · arbitration 0

### T-6: Contract suite and reference model (`support/contract.rs`)
- **DDD zone**: Supporting (test code, not mutated)
- **Spec refs**: PROP-ST02-9; RG-ST02-1.4 (PROP-ST02-3, named case), RG-ST02-1.5 (PROP-ST02-4), RG-ST02-1.6, RG-ST02-5.1 to 5.4, RG-ST02-6.5, RG-ST02-6.7, RG-ST02-7.4 (INV-ST02-6, generic names); Q-A guard
- **Dependencies**: T-3 (types and the Skeleton's stub); can be written in parallel with T-4/T-5 (different file)
- **Red**: the suite, run on the Skeleton's `SimStore` stub, fails. T-6 writes only the body of `contract.rs` (entry point and call sites come from the Skeleton).
  - `Harness` as pinned in the Convention (DA F-22).
  - Generated `Vec<Op>` (≤ 50): local apply (1–5 changes, `pt` from a small domain including equal values and rewinds), received apply (fresh peer writes, duplicates of stored writes, peer anomalies, late writes with old stamps, **local-device writes**: re-sent stored local writes and unknown local identities, plus a share with a stored local identity and another content), advance watermark (lower, equal, higher), every read, reopen. Module/category/field names generated at random (INV-ST02-6). Write generator: `seen` keys drawn from {`Delete`, the write's own field, another field, `Field("")`} (DA F-1).
  - Reference: ST-01 `Log` of `D` + ordered list of stored writes + `BTreeMap<DeviceId, Watermark>`. The reference applies the **Q-A guard** itself: it merges a received batch on a **clone** of its `Log`; if the merge adds an anomaly with `stamp.a = D` → expected `Rejected(LocalIdentityReused)` and the clone is dropped, so the reference is left unchanged (DA F-3).
  - Compared after **each** step (DA F-8): every read; the result of the operation itself — `apply`'s `Report` equal to the reference's `merge` `Report`; `apply_local`'s returned writes equal to the reference's `Log::write` results **and** to the tail returned by the next `writes_since(last seen position)`; the error arm and `Rejection` variant on rejections. Positions compared by order (strictly increasing, stable across reopen), not by value.
  - Named cases: US-5 cases 1, 2, 3 (10,000 writes on one field), 5, 6, 8, 9, 10, 12; US-1 case 6 (unusual write round trip); **US-1 case 6b** "`seen` with key `Field("")` and a `Delete` entry, round trip `==`" (DA F-1); `writes_since(Position(u64::MAX))` → empty list (DA F-13); **received write at exactly the new `K`** with `a > D`, then `writes_of`/`state` of its record succeed (DA F-2); **Q-A on the adapter** (DA F-3): stored local identity re-sent with another content → `Rejected(LocalIdentityReused)`, `Σ` unchanged (writes, positions, `K`), the next operation on the same handle succeeds, `reopen` succeeds; unknown local identity received → accepted, `K` raised.
  - `config()` from `support/mod.rs` (1,000 sequences, fixed seed).
- **Green**: comes with T-7 (SimStore) and T-9 (SQLite).
- **Deliverable**: `crates/core/tests/support/contract.rs` (body).
- **Completion criterion**: compiles in both crates (P-9); fails on the stub; two deliberately wrong references are caught once each (not committed): positions reused, and the Q-A guard removed from the reference.
- **Effort**: agent 35 min · arbitration 0

### T-7: `SimStore` with fault injection; non-`Send` compile test
- **DDD zone**: Supporting (`Std`, Q-P4)
- **Spec refs**: RG-ST02-6.7, AC-6.8; RG-ST02-6.2 (INV-ST02-2); RG-ST02-1.3, 2.1, 2.2, 5.5 on the simulator; E-ST02-2 on `SimStore`; Q-A guard
- **Dependencies**: T-5, T-6
- **Red** (`crates/core/tests/store.rs`, written by the Skeleton):
  - contract suite on `SimStore` (call site from the Skeleton; includes the Q-A named cases of T-6);
  - `full_after(k)`: E-ST02-2 levels (a)–(d), 100 cases per level, batches 1–20; `StorageFull`, `Σ` unchanged (writes, positions, `K`, watermarks); retry after `full_after(∞)` gives `Σ` of a single success (RG-ST02-2.2); reads still work (RG-ST02-2.3);
  - `unavailable(n)`: next `n` operations → `Unavailable`, `Σ` unchanged, no partial read (RG-ST02-5.5);
  - `crash_next_write()`: US-6 case 8, batch absent after `reopen()`, handle refuses everything until then;
  - damage (Q-P5 → (a)): an injected `Corrupted(cause)` models damage in the stored state: every later operation returns it, **and `reopen()` keeps returning it** (a fresh open refuses a damaged store); a fresh `SimStore` is the only way out. A `Rejected(LocalIdentityReused)` does **not** poison (DA F-3);
  - compile test: a generic `async fn use_store<S: Store>(s: &S)` instantiated with `SimStore` (not `Send`, not `Sync`, asserted with a negative static check); US-6 case 3, two futures created on one handle then polled with `Waker::noop()`, both `Ready`.
- **Green**: `sim_store.rs` as in the archi (`RefCell`, same pure functions, positions from 1).
- **Deliverable**: `crates/core/src/sim_store.rs`.
- **Completion criterion**: tests green; `20-st01-browser.sh` green with `--features sim` too (`cargo check --target wasm32-unknown-unknown -p arbitre-core --features sim`); local mutation of `sim_store.rs` ≥ 70% (Q-P4 → `Std`).
- **Effort**: agent 22 min · arbitration 0

### T-8: SQLite open: creation, verification, settings, error mapping
- **DDD zone**: Generic
- **Spec refs**: RG-ST02-1.1 (PROP-ST02-1), RG-ST02-4.1, 4.2 (PROP-ST02-10), RG-ST02-6.6 (closed set, AC-6.6), RG-ST02-7.1 (INV-ST02-6), RG-ST02-7.3, archi "Open sequence", "Error mapping", P-7, P-8
- **Dependencies**: T-3 only (types). **Runs in parallel with T-4 to T-7.**
- **Red**:
  - `crates/sqlite/tests/settings.rs`:
    - US-1 case 11: `settings()` = WAL, `synchronous = FULL`, `fullfsync = ON`, `checkpoint_fullfsync = ON` on every opened handle, fresh and reopened (DA F-12);
    - US-4 cases 1, 2: creation gives a random `D`, `K = (0, 0)`, `user_version = 1`, the fixed `application_id`; reopening gives the same `D`;
    - US-4 case 5: 1,000 stores created in a row → 1,000 distinct `D`, none all-zero;
    - US-7 case 1 (`V = 1`); US-7 case 2 (`user_version = 2` → `Unavailable(IncompatibleVersion { stored: 2, supported: 1 })`, file byte-identical);
    - layout check: the created `seen` table has the P-7 columns, constraints and PK (read from `sqlite_schema`).
  - `crates/sqlite/src/lib.rs`, `#[cfg(test)] mod`: one test per row of the P-8 table, feeding a constructed `rusqlite::Error` (`SqliteFailure` with the primary and one extended code, `FromSqlConversionFailure`, `InvalidPath`, …) and asserting the port error. `SQLITE_CONSTRAINT_PRIMARYKEY` and `SQLITE_CONSTRAINT_CHECK` → `Corrupted(Engine)`.
- **Green**: `SqliteStore::open` (steps 1–3 of the archi), layout v1 DDL with P-7, per-connection pragmas (four), `busy_timeout(options.wait)`, the P-8 mapping function, poison slot, `device()`.
- **Deliverable**: `crates/sqlite/src/lib.rs` (layout v1 DDL included, P-1), `crates/sqlite/tests/settings.rs`.
- **Completion criterion**: tests green; the mapping is one exhaustive function whose catch-all is `Unavailable(Io)`; every P-8 row has its unit test.
- **Effort**: agent 30 min · arbitration 0

### T-9: SQLite `impl Store`; contract suite on SQLite; first poll; busy
- **DDD zone**: Generic (blocking properties)
- **Spec refs**: PROP-ST02-9 on SQLite; RG-ST02-1.4 (PROP-ST02-3); RG-ST02-3.4 (PROP-ST02-8, in-process); RG-ST02-5.5; RG-ST02-6.3 (PROP-ST02-11); RG-ST02-6.5 on SQLite; Q-A guard on SQLite
- **Dependencies**: T-5, T-6, T-8
- **Red**:
  - `crates/sqlite/tests/contract.rs` (call site from the Skeleton): includes `../../core/tests/support/{mod,contract}.rs` by `#[path]` (compiles with `sim` thanks to P-9), runs the suite on `SqliteStore` (fresh store per sequence; `reopen` = drop and open). The Q-A named cases of T-6 therefore run on SQLite too (DA F-3);
  - round trip (RG-ST02-1.4): property over valid writes with the archi's edge domain (empty and non-UTF-8 values, identifiers with NUL and non-ASCII, 0–3 `seen` entries **with keys drawn from {`Delete`, own field, other field, `Field("")`}**, `l = L_MAX − 1`, `c = u32::MAX`, every `Author`), received then read back through `writes_of`, `writes_since` and after reopen, `==` on `Write`; named case US-1 case 6b (DA F-1);
  - `settings.rs`: US-6 case 4, every operation polled once with `Waker::noop()` is `Ready`; US-6 case 5 and US-3 cases 6–8 in-process (P-6): a raw connection holds `BEGIN IMMEDIATE`; with `wait = 200 ms`, held 400 ms → `Ready(Unavailable(Busy))` after 200 ms ≤ t ≤ 1.2 s, batch absent; held 100 ms → success; retry after release → success; **`wait = 0`** → `Unavailable(Busy)` with t < 100 ms (DA F-13);
  - rejections on SQLite (`settings.rs`): US-3 case 10, US-6 case 6 and the Q-A guard leave the file's `Σ` unchanged (positions, `K`) and do not poison the handle.
- **Green**: reads in one `BEGIN DEFERRED` per operation (writes and `seen` rows from the same snapshot); writes in `BEGIN IMMEDIATE` (load `K` and touched records, call `rebuild` + `stamp_local`/`receive`, insert, update `K`, `COMMIT`); unconditional `ROLLBACK` on error; `advance_watermark` as `max`. **`writes_since` decodes and validates each row** (`validate` on the single write; failure → `Corrupted(StoredWrite { position })`); `LocalIdentityPair` is the only damage it does not detect (archi "Q-A detection scope") (DA F-5). `K` is never cached in the handle (Q-2).
- **Deliverable**: `crates/sqlite/src/lib.rs`, `crates/sqlite/tests/{contract,settings}.rs`.
- **Completion criterion**: contract suite green on SQLite with 1,000 sequences (locally `cargo test -p arbitre-sqlite --features sim --test contract`: without the feature the target is silently skipped, DA F-6 note); round trip green; first-poll and busy tests green 20 times in a row locally (`cargo test -p arbitre-sqlite --test settings` in a loop).
- **Effort**: agent 30 min · arbitration 0

### T-10: Forced-stop harness (E-ST02-1) and concurrent creation
- **DDD zone**: Generic (blocking)
- **Spec refs**: E-ST02-1; RG-ST02-1.2, 1.3, 1.5, 1.6 (PROP-ST02-2, 4, 5); RG-ST02-4.1 (US-4 cases 3, 4)
- **Dependencies**: T-9
- **Red** (written first, run against broken steps to prove it bites; never committed):
  - broken step 1: the adapter reports success **before** `COMMIT` → `crash.rs` must fail;
  - broken step 2 (DA F-9): `open` skips the re-check inside `BEGIN IMMEDIATE` → the concurrent-creation test must see two different `D`s or a store refused as damaged.
- **Harness**:
  - `tests/support/child.rs`: re-exec protocol (env var role, `--exact`, line-based stdout "confirmed <run> <batch>" printed after the success result, stdin "go" lines; the coordinator can send `pt` per operation, used by T-11);
  - `tests/crash.rs`: 200 single-writer runs + 50 two-writer runs (one killed); batches of 1–5, local and received mixed; unique value per (run, batch, index); `Child::kill()` after a seeded delay (R-6 bias); reopen and check: every recorded confirmation present and identical; at most one unconfirmed batch, entire or absent; `R(W)` equals the reference `Log` fed with present batches; `K ≥` every stored stamp on `(l, c)` and the next local stamp is above all of them; survivor's confirmed writes all present (US-1 cases 1–5, 7, 12);
  - US-4 case 3, **repeated 50 times** (seeded, fresh empty path each): two children wait on stdin, are released by the same "go", open, print their `D`; check: both succeed, same `D`, the store reopens with that `D`. `Unavailable(Busy)` for either child is a **failure** (spec: "both handles report it"); if seen, apply R-9;
  - US-4 case 4: kill during first open (repeated, seeded) → next open is a fresh creation or a complete store, never one without `D`, and `settings().journal_mode = WAL` on the reopened handle (DA F-21).
- **Green**: harness passes on the real adapter; any adapter defect found is fixed in `src/lib.rs` (collision rule below).
- **Deliverable**: `crates/sqlite/tests/{crash.rs,support/child.rs}`.
- **Completion criterion**: green at spec counts; both red steps observed red; seed printed on failure and replayable via env var; run time recorded for T-15.
- **Effort**: agent 40 min · arbitration 0

### T-11: Interleaved processes (E-ST02-3)
- **DDD zone**: Generic (blocking)
- **Spec refs**: E-ST02-3; RG-ST02-3.1 to 3.5 (PROP-ST02-7, 8); story Q-2 arbitration
- **Dependencies**: T-10 (reuses and extends `support/child.rs`)
- **Red** (never committed):
  - broken step 1 (the AC-3.2 defect, DA F-4): `SqliteStore` caches `K` after its first transaction and stamps from the cache → the stepped phase must catch duplicate or non-increasing stamps, or a `seen` that misses an earlier commit. It bites because the coordinator gives both writers the **same or a rewound `pt`**;
  - broken step 2: writes open with `BEGIN DEFERRED` instead of `IMMEDIATE` → the busy checks ("held δ/2 → success") must fail with an early `Unavailable(Busy)` (`SQLITE_BUSY_SNAPSHOT` on the read-to-write upgrade, no busy handler). The plan does not claim this mutant yields duplicate stamps.
- **Harness** `tests/interleave.rs`, two phases:
  - **Stepped phase**: two writer children + one reader child, driven step by step (go per operation); the coordinator sends `pt` to each writer, drawn from a small domain with equal values and rewinds; 100 generated schedules, 2–20 ops per process, high share of same-field writes, including "held 2δ" and "held δ/2" (δ = 200 ms, ε = 1 s); checks: commit order = stamp order; later `seen` covers earlier on same field/deletion; no conflict, no anomaly, distinct stamps; every confirmed write stored; `R(W)` = sequential application in commit order (US-3 cases 1–8, 10).
  - **Free-running phase** (RG-ST02-3.5, US-3 case 9; DA F-18): 2 s per seed, 10 seeds. Batch shape: batch `t` writes field `v` on every record `r1…rk` (k = 5–20) of one category, value = tag `t`; from the second batch on, each write's `seen` is non-empty (it covers the previous `v`). One writer child commits batches in a loop and prints each batch's returned writes on stdout; the reader child loops `writes_since(START)` and `records(category)` freely and prints what it read. Checks, after the run:
    - `writes_since`: each tag appears 0 or k times, and every returned write `==` the write the writer reported (stamp, change, `seen`);
    - `records`: all k records carry the same tag `t`, or none is present yet (a split read shows record 1 at `t` and record 2 at `t + 1`).
- **Red** for the free-running phase (observable split reads, never committed; DA F-18):
  - broken step 3: `writes_since` reads `seen` rows **before** `writes`, in two statements outside one transaction → a write committed between them comes back with its `seen` entries missing → `==` check red;
  - broken step 4: `records` issues one `SELECT` per record outside a transaction → records at different tags → tag check red.
  - Both are races: each must be observed red in at least one of 10 local runs; if one is not, report it to Quentin before relying on the structural rule (one `BEGIN DEFERRED` per read operation), which T-16 checks in the code (R-10).
- **Green**: passes on the real adapter.
- **Deliverable**: `crates/sqlite/tests/interleave.rs`, additions to `support/child.rs`.
- **Completion criterion**: green at spec count; broken steps 1 to 4 observed red (3 and 4 within 10 local runs each); replayable seed; run time recorded.
- **Effort**: agent 50 min · arbitration 0

### T-12: Full storage (E-ST02-2) with a fault VFS
- **DDD zone**: Generic (blocking)
- **Spec refs**: E-ST02-2; RG-ST02-2.1 to 2.4 (PROP-ST02-6); RG-ST02-1.3; RG-ST02-4.1 (full at first opening)
- **Dependencies**: T-9. **Parallel with T-10/T-11 and T-13.**
- **Red** (broken step: the adapter skips `ROLLBACK` after an error; level (b) must then fail on retry or leave a position gap; never committed).
- **Harness**:
  - `tests/support/full_vfs.rs` (DA F-7): VFS registered via `rusqlite::ffi` under a **unique name per test** (`full-<test>-<n>`), delegating to the default VFS; the byte budget lives in the VFS's own `pAppData` (shared by the main file and its `-wal`, never across tests); `xOpen` gives **each opened file** its own copy of the delegate's `io_methods` with `xWrite` (→ `SQLITE_FULL` once the budget is spent) and `xSync` (optional failure) overridden, kept alive as long as the file; a counter records whether the budget was crossed. Budget and counter are reset before each generated case (the cases of one test share the VFS sequentially).
  - `tests/full.rs`: 100 generated cases per level (a) before the batch, (b) inside the batch's frames, (c) all but the commit frame — budget computed from WAL frame boundaries, without sector padding (DA F-10) — (d) during `advance_watermark`; batches 1–20; each: `StorageFull`, `Σ` unchanged (writes, positions, `K`, `D`, watermarks, `V`), **the budget was crossed** (asserted for every level), then budget lifted and retry = single success (US-2 cases 1–4, 7, 8); US-2 case 6: full, existing store, open and read → data; US-2 case 5: full on empty location → `StorageFull`, no `D`, later open creates.
  - `xSync` failure (DA F-10): `Unavailable(Io)`; `Σ_live = Σ_before` on the same handle; after reopen, `Σ ∈ {Σ_before, Σ_before ⊕ b}` (RG-ST02-1.3).
- **Green**: passes; adapter defects fixed in `src/lib.rs` (collision rule).
- **Deliverable**: `crates/sqlite/tests/{full.rs,support/full_vfs.rs}`.
- **Completion criterion**: green at spec counts, also with `--test-threads` default (parallel tests in `full.rs`); if R-1 triggers, stop and report to Quentin before falling back.
- **Effort**: agent 45 min · arbitration 0 (unless R-1)

### T-13: Open-time check (Q-P5), damage and incompatible versions (E-ST02-4)
- **DDD zone**: Generic (blocking)
- **Spec refs**: E-ST02-4; RG-ST02-7.2, 7.3, 5.5 (PROP-ST02-12); RG-ST02-1.6 error case; AMB-ST02-3; story Q-4 (a); Q-P5 → (a); P-7, P-8, P-10, P-11
- **Dependencies**: T-9; T-10 for `support/child.rs` (re-exec protocol, used read-only). **Parallel with T-11 and T-12.**
- **Builds**: the P-10 open-time check (with P-11) in `crates/sqlite/src/lib.rs`. It is the task's own green, not a harness-found defect, but follows the same collision rule (separate commit touching only `src/lib.rs`).
- **Red**:
  - **Q-P5 two-process test** (written first; red on the T-9 adapter, which has no open-time check): process 1 (the test) opens the store, a raw connection then damages a stored write (non-UTF-8 identifier BLOB), process 1's `writes_of` of that record → `Corrupted(StoredWrite { position })`; process 1 drops its handle; a **fresh child process** (`support/child.rs`) opens the same path → must return the same `Corrupted(StoredWrite { position })`. Same test for `seen ≥ stamp` and for `LocalIdentityPair`. Before P-10 exists, the child's open succeeds → red.
  - broken step (never committed): `open` skips the `K` vs max-stamp check → case 10 must fail.
- **Harness** `tests/damage.rs`. Two families, each asserting the error **and** the byte-identical file (read-only attempts, opens included) or unchanged `Σ` (write attempts):
  - **Damage at rest → refused at open** (build, close with WAL checkpointed, damage from outside with a raw connection, `ignore_check_constraints`, see Under-engineering; then `open` → expected `Corrupted(cause)`, file byte-identical):
    - US-7 case 3: truncation at a random length (seeded, 20 lengths) → `Corrupted` at open, or at the first read reaching an index page if Q-P6 → (a);
    - US-7 case 4: non-UTF-8 identifier BLOB → `StoredWrite { position }`;
    - US-7 case 5: stored `seen ≥ stamp` → `StoredWrite { position }`;
    - P-7 decode: a `seen` row with `kind = 2`, and one with `kind = 0` and a non-empty `field` → `StoredWrite`;
    - P-11 (DA F-20): a non-colliding orphan `seen` row at `max + 1` → `StoredWrite { position: max + 1 }`;
    - Q-A (b): two contents for one local identity inserted by SQL → `LocalIdentityPair`;
    - US-1 case 10: `K` below the largest stored stamp → `ClockState`;
    - US-7 case 6: text file at the path → `NotAStore`; foreign `application_id` → `NotAStore`;
    - (c): missing `device` row → `DeviceId`; `user_version = 0` with tables → `Layout`;
    - US-5 case 11: a directory at the store path → `Unavailable(Io)` (P-5).
  - **Damage after open → found by the operation** (DA F-5 kept): one fresh store **and handle per operation**; the damage is injected by a raw connection after `open` succeeded; then the single operation (`writes_of`, `state`, `records`, `writes_since`, `apply`, `apply_local`, `advance_watermark`) → expected result per operation:
    - US-7 cases 4 and 5: `Corrupted(StoredWrite { position })` from every operation that reads that write (`writes_since` from before it included, it validates, T-9; `apply`/`apply_local` touching its record, DA F-13); `advance_watermark` and operations on other records succeed; on one handle, every op after the first `Corrupted` → same `Corrupted` (poison);
    - `LocalIdentityPair`: `Corrupted` on `writes_of` / `state` / `records` / `apply` / `apply_local` of that record; `writes_since` returns both rows (archi exception);
    - P-8 constraint: a colliding orphan `seen` row (`kind = 0`) at `max + 1`, then a received write with a `Delete` `seen` entry → `Corrupted(Engine)`, `Σ` unchanged (rolled back);
    - each case then reopens: `open` → the same `Corrupted` (P-10), closing the loop of the two-process test in-process.
  - **Timing probe for T-15** (`damage.rs :: open_check_timing`, asserts nothing): builds a store of 10,000 local writes with the US-5 case 3 shape (one field, so each write after the first has a `seen` entry), in batches of 100; measures the median of 5 `open`s of it minus the median of 5 `open`s of a one-write store; writes the result in ms to `$CI_OUT/open-check-ms.txt` when `CI_OUT` is set (exported by `scripts/ci.sh`; `target/ci/` is uploaded as a CI artifact), otherwise prints it.
- **Green**: P-10 and P-11 in `src/lib.rs`; other adapter defects fixed there too (collision rule).
- **Deliverable**: `crates/sqlite/src/lib.rs` (open-time check), `crates/sqlite/tests/damage.rs`.
- **Completion criterion**: two-process test observed red before P-10 and green after; all cases green; each asserts the error and the byte-identical file or unchanged `Σ`; the timing probe runs locally and writes its file when `CI_OUT` is set.
- **Effort**: agent 45 min · arbitration 0

### T-14: Glossary rows in `CONTRIBUTING.md`
- **DDD zone**: Generic (docs)
- **Spec refs**: archi "Impacted modules" (`CONTRIBUTING.md` additive)
- **Dependencies**: T-2 merged (PR 1 also edits `CONTRIBUTING.md`; write on the rebased branch to avoid a conflict)
- **Description**: glossary rows for store / `Store`, local apply / `apply_local`, received apply / `apply`, position / `Position`, watermark / `Watermark`, clock state / `ClockState`, `StorageFull` / `Corrupted` / `Unavailable` (French `StockagePlein`, `Corrompu`, `Indisponible`). No rule changes.
- **Deliverable**: `CONTRIBUTING.md` (glossary table only).
- **Completion criterion**: `git diff main -- CONTRIBUTING.md` touches only glossary rows.
- **Effort**: agent 5 min · arbitration 0

### T-15: Measure CI time on CI, settle run counts, full local verdict
- **DDD zone**: Generic
- **Spec refs**: GAP-ST02-2 (counts, floor = half); R-2; Q-P1 → (b); Q-P5 → (a) budget; R-11
- **Dependencies**: T-7, T-9 to T-14
- **Description** (DA F-16):
  - rebase on `main`; run `scripts/ci.sh` locally (correctness only; macOS timings are not used for Q-P1 because `F_FULLFSYNC` makes commits slower than on the Linux runner);
  - on Quentin's go, push `st-02/store` and open PR 2 as a **draft**; read from the CI step summary the duration of each check, of each harness binary, of `10-st01-tests` as a whole, and of `50-st01-mutation` with its mutant count (`store.rs`, `sim_store.rs` mutants rerun the core suites, `SimStore` contract suite included);
  - apply Q-P1 → (b) to `10` as measured on CI: above 10 min, halve E-ST02-1, -2, -3 counts (the floor), never PROP-ST02-9; if `50` alone exceeds 10 min, report to Quentin (no rule covers it yet);
  - **open-time check budget (Q-P5)**: read `open-check-ms.txt` from the CI artifact (T-13 probe, Linux runner). ≤ 100 ms → recorded, continue. > 100 ms → **stop**, no further task; ask Quentin to choose between narrowing Q-4 to per-handle poisoning and an incremental check (R-11). macOS local values are recorded for information only;
  - record final counts and timings in the task report (they go into the PR body).
- **Deliverable**: possibly lowered counts in `crates/sqlite/tests/*.rs` (never below the floor; PROP-ST02-9 never lowered).
- **Completion criterion**: `scripts/ci.sh` green locally; CI timings recorded, open-time check at 10,000 writes included and ≤ 100 ms (or Quentin's answer recorded); `mutation.md` of the CI run has rows for `store.rs` (≥ 85%) and `sim_store.rs` (≥ 70%) and **no** `crates/sqlite/src` row.
- **Effort**: agent 20 min · arbitration 5 min (go for the draft push), +10 min only if the 100 ms budget is exceeded

### T-16: Finish and merge PR 2
- **DDD zone**: Core (story completion)
- **Spec refs**: cadre § ST-02 completion criteria; RG-ST02-G.2
- **Dependencies**: T-2 (merged), T-15
- **Description**:
  - Check R-8: branch rebased on `main`; `scripts/ci.d/50-st01-mutation.sh` contains the Gen exclusion; `git diff main --name-only` has no `scripts/` or `.github/` path.
  - Structural check (R-10): every read operation of `SqliteStore` runs inside exactly one `BEGIN DEFERRED`; no `K` cached in the handle; the P-10 check runs on every non-creating open and reuses `writes_since`'s decode + `validate` function.
  - Mark PR 2 ready. Body: what ships, counts and timings from T-15, blocking properties and where they run, P-7 (layout v1 `seen` amended against the archi), P-10/P-11 (open-time check, superseding the archi's "no full scan at open", with its measured cost at 10,000 writes), `RandomSource` untested (F-13). No label needed.
  - For each surviving mutant of `store.rs` or `sim_store.rs`: a killing test, or an entry in `.ctdd/stories/ST-02/mutants_equivalents.toml` with a justification.
  - **Quentin**: reviews equivalents and the PR, merges (squash, hand-written message).
- **Deliverable**: PR 2 merged; `mutants_equivalents.toml` if needed.
- **Completion criterion**: `ci` and `judge` green on PR 2; `mutation.md` of the CI run: `store.rs` ≥ 85%, `sim_store.rs` ≥ 70% (Q-P4 → `Std`), `merge.rs` diff neutral or ≥ 85%, no Gen row; each equivalent reviewed by Quentin; merged.
- **Effort**: agent 30 min · arbitration 15 min (equivalents, PR read, merge)

## Execution order

```
PR 1:  T-1 ─► T-2 (Quentin: label + merge) ─────────────────────────────────┐
                                                                            │
PR 2:  T-3 ─┬─► T-4 ─► T-5 ─┬─► T-7 ───────────────────────────────┐        │
 (Skeleton) ├─► T-6 ────────┤                                      │        │
            └─► T-8 ────────┴─► T-9 ─┬─► T-10 ─► T-11 ─────────────┤        │
                                     │         └─► T-13 ───────────┤        │
                                     ├─► T-12 ─────────────────────┼─► T-15 ─┴─► T-16
                         (after T-2) T-14 ─────────────────────────┘
```

T-9 needs T-5 and T-6 (and T-8); T-7 needs T-5 and T-6; T-13 needs T-9 and T-10's `support/child.rs` (read-only), then runs in parallel with T-11 and T-12.

### Parallelism (separate agents) and collisions

| Can run together | Files owned | Collision and rule |
|---|---|---|
| T-1/T-2 ∥ all of PR 2 up to T-15 | `scripts/**`, `CONTRIBUTING.md` (Zones, Equivalents) on `st-02/gen-mutation` | `CONTRIBUTING.md` is also touched by T-14 (glossary): T-14 waits for T-2 and works on the rebased branch |
| T-4 → T-5 (one agent) ∥ T-6 ∥ T-8 | T-4/T-5: `store.rs`, `merge.rs`; T-6: body of `tests/support/contract.rs`; T-8: `crates/sqlite/src/lib.rs`, `tests/settings.rs` | None: the red tests in `tests/store.rs` and both suite call sites are written by the Skeleton (Convention), so no task edits `tests/store.rs` or `crates/sqlite/tests/contract.rs`. `lib.rs` re-exports are fixed by T-3 |
| T-7 ∥ T-9 | T-7: `sim_store.rs`; T-9: `crates/sqlite/**` | None |
| T-10 → T-11 (one agent) ∥ T-12; T-13 from the end of T-10 | T-10/T-11: `tests/{crash,interleave}.rs`, `support/child.rs`; T-12: `tests/full.rs`, `support/full_vfs.rs`; T-13: `tests/damage.rs`, P-10 in `src/lib.rs` | Test files are disjoint (P-2); T-13 reads `support/child.rs` but never edits it (a missing child role goes to the T-10/T-11 agent). **`crates/sqlite/src/lib.rs` is the hot spot** (R-9 fix in T-10, P-10 in T-13): a harness agent that finds an adapter defect commits the fix as a separate commit touching only `src/lib.rs`, and the agents rebase one at a time (first finished, first rebased). T-11 depends on T-10 because both write `support/child.rs` |

- Worktrees: PR 1 in its own worktree from `main`; PR 2's parallel agents each in a worktree from `st-02/store`, merged back in the order of the diagram.
- Pushes happen only in T-2, T-15 (draft) and T-16, on Quentin's go.

### Estimate (agent time and arbitration time)

| Lane | Tasks | Agent (min) | Arbitration (min) |
|---|---|---|---|
| PR 1 | T-1, T-2 | 27 | 10 |
| Core | T-3 → T-4 → T-5, T-6, T-7 | 109 | 0 |
| SQLite | T-8, T-9, T-10 → T-11 ∥ T-12 ∥ T-13 | 240 (≈ 170 in duration) | 0 (R-1 aside) |
| Closing | T-14, T-15, T-16 | 55 | 20 (+10 if the 100 ms budget is exceeded) |
| Open questions | Q-P6 | — | 5 |
| **Total** | | **≈ 431 cumulated, ≈ 275 in duration** (SQLite lane is the critical path, via T-11) | **≈ 35** (≈ 45 with the R-11 escalation) |

- v2 → v3: +25 min agent (T-13 +15: open-time check, two-process test, damage split at rest / after open; T-11 +5: retargeted red steps and `records` tag check; T-15 +5: budget read and escalation); −5 min arbitration (Q-P1 to Q-P5 arbitrated, Q-P6 added).
- v1 → v2: +71 min agent (mostly T-10, T-11, T-12: concurrent-creation repeats, free-running phase, per-test VFS), +10 min arbitration (Q-P3, Q-P4).
- Frame: M (100 min machine) + "moyenne (2)". Machine time is about 2.75 × M; arbitration fits "moyenne". Recorded in `state.json` by the orchestrator, not corrected.
- Not counted: DA iterations, CI wall time of the harnesses (measured in T-15).

## C-TDD invariants to check after implementation

| Invariant / Property | Task(s) | Verification |
|---|---|---|
| INV-ST02-1 core pure, wasm32 | T-3, T-7 | `20-st01-browser.sh`; wasm32 check with `--features sim` |
| INV-ST02-2 async, `&self`, no `Send` | T-7 | Compile test with non-`Send`/non-`Sync` `SimStore`; two futures on one handle |
| INV-ST02-3 merge engine port-free | T-4 | Diff of `hlc/write/clock.rs` empty; `merge.rs` diff = `with_clock`; ST-01 suites unchanged |
| INV-ST02-4 closed error set, `Rejected ⇒ Σ unchanged` | T-3, T-4, T-5, T-6, T-8, T-9 | Enums closed by type; P-8 mapping exhaustive with per-row tests; rejection tests on core, `SimStore` and SQLite, Q-A guard included |
| INV-ST02-5 SQLite only in `arbitre-sqlite` | T-1, T-3 | `30-st01-graph.sh`; test `arbitre-plan → arbitre-sqlite` red |
| INV-ST02-6 `V = 1`, generic layout | T-6, T-8, T-9 | `user_version = 1`; P-7 schema check; contract suite and round trip with random names |
| INV-ST02-7 no network | T-3 | `cargo tree -e normal` of `arbitre-sqlite` |
| PROP-ST02-1 durable confirmation | T-8, T-10 | `settings()` (four values); forced stop |
| PROP-ST02-2 confirmed survives, all or nothing | T-10 | `crash.rs` (200 + 50) |
| PROP-ST02-3 exact round trip | T-6, T-9 | Round-trip property through SQLite and reopen, `seen` keys incl. `Field("")` + `Delete`; US-1 cases 6, 6b |
| PROP-ST02-4 rebuilt log = reference | T-6, T-7, T-9, T-10 | Contract suite with reopen, `Report` compared; crash reference |
| PROP-ST02-5 clock persisted, monotone | T-4, T-5, T-6, T-10, T-11, T-13 | Unit cases 8, 9; `(l, c)` boundaries; received write at new `K`; post-crash check; cached-`K` mutant; damage case 10 |
| PROP-ST02-6 full storage | T-7, T-12 | `full_after` levels; fault VFS levels (a)–(d), budget crossed, `xSync` failure |
| PROP-ST02-7 interleaved writers | T-4, T-11 | `seen` within a list; `interleave.rs` stepped phase with controlled `pt` |
| PROP-ST02-8 busy, readers whole batches | T-9, T-11, T-16 | In-process busy timing incl. `δ = 0`; free-running reader phase with `==` on `writes_since` and tag check on `records`, two observable split-read red steps; structural check of one read transaction per read |
| PROP-ST02-9 contract suite | T-6, T-7, T-9 | Same suite on `SimStore` and `SqliteStore`, 1,000 sequences each, op results compared |
| PROP-ST02-10 device identifier | T-8, T-10, T-12 | Create/reopen, 1,000 distinct; concurrent creation × 50 with red step, kill during creation; full on empty location |
| PROP-ST02-11 first poll | T-9 | Every op `Ready` with `Waker::noop()`, busy included |
| PROP-ST02-12 damage, newer layout, unreadable | T-7, T-8, T-13, T-15 | Q-P5 two-process test; damage at rest refused at open (P-10, P-11); damage after open per operation, then refused at reopen; `SimStore` damage survives `reopen()`; `IncompatibleVersion`; open-check cost ≤ 100 ms at 10,000 writes |

## Traceability matrix (AC → rule/property → task → test)

| AC | Rule / property | Task | Test (file :: subject) |
|---|---|---|---|
| AC-1.1 | RG-ST02-1.1, PROP-ST02-1 | T-8, T-10 | `sqlite/tests/settings.rs` :: durability settings (four); `crash.rs` |
| AC-1.2, AC-1.4 | RG-ST02-1.2, PROP-ST02-2 | T-10 | `sqlite/tests/crash.rs` :: single writer, two writers one killed |
| AC-1.3 | RG-ST02-1.3 | T-7, T-9, T-10, T-12 | `core/tests/store.rs` :: crash/full injection; `crash.rs`; `full.rs` (incl. `xSync` before/after reopen); `settings.rs` :: busy batch absent |
| AC-1.5 | RG-ST02-1.4, PROP-ST02-3 | T-6, T-9 | `sqlite/tests/contract.rs` :: round trip, US-1 cases 6 and 6b |
| AC-1.6 | RG-ST02-1.5, PROP-ST02-4 | T-6, T-7, T-9, T-10 | `support/contract.rs` (reopen op, `Report` comparison); `crash.rs` :: rebuilt log |
| AC-1.7 | RG-ST02-1.6, PROP-ST02-5 | T-4, T-5, T-6, T-13 | `core/tests/store.rs` :: US-1 cases 8, 9, `(l, c)` boundaries; `support/contract.rs` :: write at new `K`; `damage.rs` :: clock below stamp |
| AC-2.1 to AC-2.4 | RG-ST02-2.1 to 2.4, PROP-ST02-6 | T-7, T-12 | `core/tests/store.rs` :: `full_after` levels; `sqlite/tests/full.rs` :: levels (a)–(d), retry, reads, empty location |
| AC-3.1, AC-3.5 | RG-ST02-3.1, 3.3, PROP-ST02-7 | T-4, T-11 | `core/tests/store.rs` :: seen within a list; `interleave.rs` :: stepped phase |
| AC-3.2 | RG-ST02-3.2 | T-4, T-11 | `core/tests/store.rs` :: distinct increasing stamps; `interleave.rs` :: distinct stamps with equal/rewound `pt` |
| AC-3.3 | RG-ST02-3.4, PROP-ST02-8 | T-9, T-11 | `settings.rs` :: busy held 2δ / δ/2 / retry / `δ = 0`; `interleave.rs` |
| AC-3.4 | RG-ST02-3.5, PROP-ST02-8 | T-11, T-16 | `interleave.rs` :: free-running reader phase (`writes_since` `==`, `records` tags); structural check |
| AC-4.1, AC-4.2 | RG-ST02-4.1, PROP-ST02-10 | T-8, T-10, T-12 | `settings.rs` :: create/reopen; `crash.rs` :: concurrent creation × 50, kill during creation; `full.rs` :: empty location |
| AC-4.3 | RG-ST02-4.2 | T-8 | `settings.rs` :: 1,000 distinct identifiers |
| AC-5.1 to AC-5.4 | RG-ST02-5.1 to 5.4, PROP-ST02-9 | T-6, T-7, T-9 | `support/contract.rs` :: model-based suite + US-5 named cases + `P = u64::MAX`, on both adapters |
| AC-5.5 | RG-ST02-5.5, PROP-ST02-12 | T-7, T-9, T-13 | `core/tests/store.rs` :: `unavailable(n)`; `damage.rs` :: directory at path, per-operation damaged reads (`writes_since` validates) |
| AC-6.1 | RG-ST02-6.1, INV-ST02-1 | T-3, T-7 | `ci.d/20-st01-browser.sh`; wasm32 check with `sim` |
| AC-6.2, AC-6.3 | RG-ST02-6.2, INV-ST02-2 | T-7 | `core/tests/store.rs` :: non-`Send` compile test, two futures |
| AC-6.4 | RG-ST02-6.3, PROP-ST02-11 | T-9 | `settings.rs` :: first poll, busy included |
| AC-6.5 | RG-ST02-6.4, INV-ST02-3, CONFLICT-ST02-1 | T-4, T-5 | diff check of core modules; `apply_local` / `apply` tests |
| AC-6.6 | RG-ST02-6.5, INV-ST02-4, CONFLICT-ST02-2 | T-3, T-4, T-5, T-6, T-8, T-9 | `core/tests/store.rs` :: rejections; `support/contract.rs` :: Q-A on adapters; `sqlite/src/lib.rs` :: P-8 mapping rows; `settings.rs` :: rejections on SQLite |
| AC-6.7 | RG-ST02-6.6, INV-ST02-5 | T-1, T-3 | `ci.d/30-st01-graph.sh`; `scripts/lib/test_ci.py` :: plan → sqlite red |
| AC-6.8 | RG-ST02-6.7, PROP-ST02-9 | T-7 | `core/tests/store.rs` :: contract suite on `SimStore`, injections |
| AC-7.1 | RG-ST02-7.1, INV-ST02-6 | T-8 | `settings.rs` :: `user_version = 1`, P-7 schema |
| AC-7.2 | RG-ST02-7.2, story Q-4 (a), Q-P5, Q-A (b), PROP-ST02-12 | T-4, T-5, T-6, T-7, T-9, T-13, T-15 | `core/tests/store.rs` :: `rebuild` damage, Q-A guard, no poison on rejection, damage survives `reopen()`; `damage.rs` :: two-process refusal, refused at open (P-7 decode, P-11 orphan, pair), per operation after open, constraint; `open_check_timing` |
| AC-7.3 | RG-ST02-7.3 | T-8 | `settings.rs` :: newer layout refused, file unchanged |
| AC-7.4 | RG-ST02-7.4, INV-ST02-6 | T-6, T-9 | contract suite and round trip with random names |
| (RG-ST02-G.1) | INV-ST02-7 | T-3 | `cargo tree` check |
| (RG-ST02-G.2, Q-M) | zones, Gen not mutated | T-1, T-3, T-15, T-16 | `test_ci.py` :: `gen_files`; `mutation.md` without Gen rows |

## Open questions for Quentin

The recommendation applies by default if Quentin says nothing. It does not block the Skeleton (T-13 writes its own tests).

| # | Question | Options | Recommendation |
|---|---|---|---|
| Q-P6 | **Engine damage outside the scanned rows.** P-10 reads every `writes` and `seen` row, so table pages are covered; damage confined to an index page (e.g. a truncation that cuts only index pages) is still found only by a later read, which poisons that handle, and the next open refuses only if P-10 hits it. | (a) Keep P-10 as arbitrated (stored writes only). (b) Add `PRAGMA quick_check` to the open-time check (one statement, any non-`ok` row → `Corrupted(Engine)`), counted in the same 100 ms budget; if it alone breaks the budget, drop it first. | **(b).** One line, makes "a damaged store refuses to open" hold for engine damage too, and makes T-13's truncation case deterministic ("at open"). |

## Quentin's arbitrations (2026-11-04)

Binding; override the open questions above.

- **Q-P5 → (a)** (DA v2 F-19 / F-17): every stored write is checked when the store opens (decode + validate, plus the local-identity-pair query), so a damaged store refuses to open in every process. Budget: if the open-time check exceeds 100 ms at 10,000 writes (measured in T-15), stop and ask Quentin to choose between narrowing Q-4 to per-handle poisoning and an incremental check.
- **Q-P1 → (b), Q-P2 → (a), Q-P3 → layout fix, Q-P4 → Std**: taken on the recommendations; Quentin may still revise them before the ST-02 PR is merged.
