# Implementation Plan — ST-02: Persistence, async `Store` port and SQLite adapter

## Traceability
- Specs: `.ctdd/stories/ST-02/02_specs.md`. Its "Quentin's arbitration (2026-11-04)" is binding: **Q-A → (b) plus a guard on the received path**.
- Archi: `.ctdd/stories/ST-02/03_archi.md`. Its "Quentin's arbitration (2026-11-04)" is binding: **Q-M → (e)**, no CI mutation of Gen-zone files, shipped as a separate `ci-change` PR merged before the ST-02 PR.
- Story: `.ctdd/stories/ST-02/01_story.md` (Q-1 to Q-5 arbitrations binding).
- Frame: `.ctdd/cadre/04_plan.md` § ST-02 (size M, arbitration "moyenne (2)", zone Gen with blocking properties); § "Amendement du socle (F-27)".
- Format: `.ctdd/stories/ST-01/04_plan.md`. Repo rules: `CONTRIBUTING.md`, `scripts/ci.sh`, `scripts/ci.d/*.sh`, `scripts/lib/{mutation,judge}.py`.
- Date: 2026-11-04 · Status: DRAFT v1.

## Technical context
- **Codebase** (read 2026-11-04, `main` = `d6c2ec4`):
  - `crates/core` (`arbitre-core`): `#![no_std]` + `alloc`, no normal dependency; `hlc.rs`, `write.rs`, `merge.rs` (Full), `clock.rs` (Std, feature `sim`); tests in `crates/core/tests/` with `support/mod.rs` (`config()`: 1,000 cases, `SEED`).
  - `crates/cli`: clap stub, zone Gen (`.ctdd/stories/ST-01/zones.toml`). Not touched.
  - `Log` owns a private `HybridClock` at `(0, 0)`; `Log::write` is the only local write path (GAP-ST02-1).
  - CI: `scripts/ci.sh` runs `ci.d/10` (tests, `--all-features`), `20` (wasm32 on core), `30` (graph, `arbitre-sqlite → arbitre-core` already allowed), `40` (gitleaks), `50` (`cargo mutants --in-diff`, verdict by `mutation.py`). The judge (`judge.py`, run from `main`) reds any PR touching `scripts/` or `.github/` without `ci-change`, and any `ci-change` PR touching `crates/`.
- **Stack**: Rust 1.99, edition 2024; `rusqlite` 0.41.1 `bundled` (local to `arbitre-sqlite`); `getrandom` 0.4.3 and `proptest` 1.12.0 (workspace); Python ≥ 3.13 stdlib; cargo-mutants 27.1.0.
- **Patterns to follow**: functional core / imperative shell (store semantics pure in `store.rs`, adapters only do I/O); port + real adapter + simulator (`Clock`/`SimClock` → `Store`/`SimStore`/`SqliteStore`); validation before effect; fixed-seed property tests from `support/mod.rs`; zones per story, strictest wins.
- **cargo-mutants facts checked in its 27.1.0 source** (for T-1): an `--exclude` glob containing `/` matches the whole relative path (`glob.rs`), so an exact file path works; a diff whose only Rust changes are in excluded files ends with `NoMutants` → exit 0, no `outcomes.json`, which `50-st01-mutation.sh` already treats as neutral (`in_diff.rs`). With mutants only in `arbitre-core`, only `arbitre-core`'s tests run per mutant, so the SQLite process harnesses never run under mutation.

## Decisions taken by this plan (within the archi's latitude)

| # | Decision | Why |
|---|---|---|
| P-1 | `schema.rs` is merged into `crates/sqlite/src/lib.rs` | Fewest files; the archi allows it; Gen, not mutated in CI after T-2. Split later only if the file passes ~600 lines |
| P-2 | No `tests/support/mod.rs` in `crates/sqlite`: each test file includes its helper with `#[path = "support/<x>.rs"] mod <x>;` | Lets T-10, T-11, T-12 and T-13 run in parallel without colliding on a shared `mod.rs` |
| P-3 | The contract suite is a plain proptest over a generated `Vec<Op>` driven against the reference model | `proptest-state-machine` would be a new dependency (foundation amendment); not needed |
| P-4 | Gen exclusion passes **resolved file paths** (one `--exclude <path>` per Gen `.rs` file), not the `zones.toml` patterns | Pattern semantics differ (`full_match` vs cargo-mutants globs) and a file covered by Gen and Std must stay mutated (strictest wins). Same resolution as the judge: `zone_of` |
| P-5 | "Unreadable store" (US-5 case 11) is tested with a **directory** at the store path, not `chmod 000` | Works when CI runs as root; `SQLITE_CANTOPEN` → `Unavailable(Io)` |
| P-6 | Busy-store timing (US-3 cases 6–8, US-6 case 5) is tested **in-process** with two connections in `T-9`, and again cross-process in E-ST02-3 (T-11) | SQLite locks are per connection; the in-process test is cheap and deterministic |

## Pre-plan analysis

### Over-engineering detected
- **`Options.vfs: Option<String>`** is a public field that only tests use. Kept: it is the one-line way to open with a fault VFS (`Connection::open_with_flags_and_vfs`), and the alternative (a test-only constructor behind a feature) is more code. Doc comment: "test hook".
- **`SqliteStore::settings()`** exists only for the RG-ST02-1.1 test. Kept: `synchronous` and `fullfsync` are per-connection, so a second raw connection cannot read them.
- **Six SQLite test files** (archi tree): kept, because they are separate test binaries that run in parallel and give parallel agents disjoint files. No other split.

### Under-engineering detected
- **STRICT tables and CHECK constraints block the damage tests.** E-ST02-4 (a)/(b)/(d) cannot be produced by plain SQL on a STRICT table with CHECKs (wrong type, `author = 7`, `length(a) ≠ 16`). The damaging connection uses `PRAGMA ignore_check_constraints = ON` and type-valid but semantically bad values (non-UTF-8 identifier BLOB, 15-byte device, `l ≥ L_MAX`, `c > u32::MAX`, `seen ≥ stamp`). The adapter still decodes and validates every field: constraints are a second line, not the check.
- **`cargo test` runs tests on several threads**, and the process harnesses spawn children and fsync heavily: the timing assertions (`δ ≤ wait ≤ δ + ε`) can flake under load. Each timing test uses its own store directory, `ε = 1 s`, and the timing schedules run sequentially inside one test function.
- **Child re-exec**: the child is selected by an env var and `--exact <child test> --test-threads=1 --nocapture`; a child test run without the env var is a no-op success, so `cargo test` alone never runs a stray child.
- **`arbitre-sqlite` declares no Cargo feature**: `10-st01-tests.sh` runs `--all-features`, and an optional feature there would be enabled silently.

### Identified risks

| # | Risk | Probability | Impact | Mitigation |
|---|---|---|---|---|
| R-1 | Fault VFS (`unsafe` FFI, every I/O method delegated) costs more than planned | Medium | T-12 slips | Delegate through the default VFS's `pAppData`, override only `xWrite`/`xSync` of the file methods. Fallback `PRAGMA max_page_count` loses level (c) "at commit": that weakens E-ST02-2 and goes back to Quentin (archi R-2) |
| R-2 | CI time of `10-st01-tests.sh` too high (1,000 SQLite sequences with fsync per commit, 250 crash runs, 400 full cases, 100 schedules) | Medium | Slow PRs | Measured in T-15; lowering rule = open question Q-P1. Never below the specs' floor (half) |
| R-3 | Timing flakiness of busy-store tests on a loaded runner | Medium | Red CI by chance | P-6; ε = 1 s; failing seed printed for replay |
| R-4 | Bundled SQLite of `libsqlite3-sys` 0.39.1 older than 3.37 (`STRICT`) | Low | Layout v1 invalid | Checked in T-3 (`SELECT sqlite_version()`); fallback: drop `STRICT`, keep CHECKs (no spec impact) |
| R-5 | Mutation score of `store.rs` < 85% (bounds in `max_lex`, idempotence, Q-A guard) | Medium | Story blocked | Named boundary tests in T-4/T-5; equivalents justified in `.ctdd/stories/ST-02/mutants_equivalents.toml`, reviewed by Quentin (T-16) |
| R-6 | Kill instants rarely land inside an fsync (archi R-3) | Medium | E-ST02-1 weaker than it looks | T-10 measures commit duration first and draws half of the kill delays inside `[0, 2 × commit]` after a batch starts |
| R-7 | In WAL mode, SQLite may spill frames before commit (`cache_spill`); level (b) "after the k-th write" maps to a byte budget only approximately | Low | Level (b) and (c) differ less than intended | Both must give `StorageFull` with `Σ` unchanged; the test asserts the budget is spent strictly inside the batch's frames (WAL size before/after) |
| R-8 | The ST-02 PR is opened before the `ci-change` PR is merged | Low | 50 mutates `crates/sqlite/src` for hours | T-16 starts with `git rebase origin/main` and checks that `50-st01-mutation.sh` on the branch contains the Gen exclusion |

## PR split

| PR | Branch (from `main`) | Content | Agent does | **Quentin does** |
|---|---|---|---|---|
| **PR 1 — `ci-change` amendment** | `st-02/gen-mutation` | `scripts/lib/mutation.py`, `scripts/lib/test_ci.py`, `scripts/ci.d/50-st01-mutation.sh`, `CONTRIBUTING.md` (Zones, Equivalent mutants). **Nothing under `crates/`** | T-1, T-2: code, local checks, push and open the PR on Quentin's go | Read the diff; **set the `ci-change` label** (agents never do); check `ci` and `judge` green; **merge** (squash, message written by hand) |
| **PR 2 — ST-02** | `st-02/store`, rebased on `main` **after PR 1 is merged** | `crates/core/**`, `crates/sqlite/**`, `Cargo.lock`, `CONTRIBUTING.md` (glossary rows), `.ctdd/stories/ST-02/**` (cycle docs, `zones.toml`, `mutants_equivalents.toml` if any). **Nothing under `scripts/` or `.github/`** | T-3 to T-16 | **No label.** Review equivalents if any, the measured CI time (Q-P1), `mutation.md`; check `ci` and `judge` green; **merge** (squash, hand-written message) |

- PR 1 has no Rust diff: its mutation check is neutral. Its judge is red until the label is set, then green (no `crates/` path).
- PR 1 can be opened, reviewed and merged while the ST-02 code is being written: it blocks only T-16.
- F-27 procedure: see Q-P2.

## Tasks

Convention (as ST-01): the Skeleton (phase 6) lays down the types and signatures of the archi and writes the **red** tests of T-4 to T-9 (they compile and fail). For T-10 to T-13 the harness *is* the test: the task writes it first, runs it red against a deliberately broken adapter step (named per task), then makes it green. "Green" for every task also means `cargo test --workspace --all-features --locked`, `20-st01-browser.sh` and `30-st01-graph.sh` pass.

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
  - `50-st01-mutation.sh`: before `cargo mutants`, read that list (`mapfile`) and append `--exclude <path>` per file to the existing command line (the only place `--exclude` is allowed). A failure of the listing makes the check red.
  - `CONTRIBUTING.md`: in "Zones", `Gen` = "not mutated in CI (excluded by `50-st01-mutation.sh`, same resolution as the judge); local run: `cargo mutants --all-features -f 'crates/sqlite/src/**'`"; in "Equivalent mutants", the "only exclusion channel" sentence names the Gen exclusion as the one automatic exception.
- **Deliverable**: `scripts/lib/mutation.py`, `scripts/lib/test_ci.py`, `scripts/ci.d/50-st01-mutation.sh`, `CONTRIBUTING.md`.
- **Completion criterion**:
  - unit tests green;
  - local demonstration on a throwaway branch on top (never pushed): a diff touching `crates/cli/src/main.rs` (Gen) and `crates/core/src/clock.rs` (Std); `cargo mutants --list --in-diff <diff> --all-features` with the computed `--exclude`s lists only `clock.rs` mutants; with only the `main.rs` change, the full `50-st01-mutation.sh` is green (neutral, no `outcomes.json`);
  - `scripts/ci.sh` green on `st-02/gen-mutation`;
  - `git diff main --name-only` contains no `crates/` path.
- **Effort**: agent 20 min · arbitration 0

### T-2: Open, label and merge PR 1
- **DDD zone**: Generic
- **Spec refs**: CONTRIBUTING "Branches and PRs", "The judge"; archi Q-M arbitration
- **Dependencies**: T-1
- **Description**: on Quentin's go, push `st-02/gen-mutation` and open PR 1. Body: the Q-M arbitration, what changes, the demonstration of T-1, "foundation amendment". **Quentin** sets `ci-change`, reviews, merges.
- **Deliverable**: PR 1 merged into `main`.
- **Completion criterion**: `judge` red before the label (lists `scripts/…` paths), green after; `ci` green; merged.
- **Effort**: agent 5 min · arbitration 10 min (read diff, label, merge)

### PR 2 — ST-02

### T-3: Scaffold `arbitre-sqlite`, core test target, zones
- **DDD zone**: Generic
- **Spec refs**: RG-ST02-6.1 (INV-ST02-1), RG-ST02-6.6 (INV-ST02-5), RG-ST02-G.1 (INV-ST02-7), RG-ST02-G.2
- **Dependencies**: none (part of the Skeleton phase)
- **Description**:
  - `crates/sqlite/Cargo.toml`: `arbitre-core` (path), `rusqlite = { version = "0.41.1", features = ["bundled"] }` (local), `getrandom = { workspace = true }`; dev: `arbitre-core` with `features = ["sim"]`, `proptest = { workspace = true }`. No `[features]`.
  - `crates/core/Cargo.toml`: `[[test]] name = "store"`, `required-features = ["sim"]`.
  - `crates/core/src/lib.rs`: `mod store;`, `#[cfg(feature = "sim")] mod sim_store;`, re-exports.
  - `.ctdd/stories/ST-02/zones.toml` as in the archi (`Full` store.rs, `Std` sim_store.rs, `Gen` `crates/sqlite/src/**`).
  - Types and signatures of the archi (`Store`, `LocalChange`, `ClockState`, `Position`, `Watermark`, `StoreError`, `Damage`, `Cause`, `Rejection`, `ApplyError`, `Received`, `rebuild`/`stamp_local`/`receive`, `SimStore`, `SqliteStore`, `Options`, `Settings`) with bodies left for later tasks.
  - `Cargo.lock` regenerated by `cargo check --workspace`.
- **Deliverable**: files above.
- **Completion criterion**:
  - `cargo check --workspace --all-features --locked` green; `20-st01-browser.sh` and `30-st01-graph.sh` green (INV-ST02-1, 5);
  - `cargo tree -p arbitre-sqlite -e normal` shows no network crate (INV-ST02-7);
  - bundled `sqlite_version()` ≥ 3.37 recorded in the task report (R-4);
  - judge locally: `python3 scripts/lib/judge.py main HEAD` green without label (no zone lowered).
- **Effort**: agent 15 min · arbitration 0

### T-4: Restored clock, `rebuild`, `stamp_local` (`store.rs` part 1)
- **DDD zone**: Core (Full)
- **Spec refs**: GAP-ST02-1; RG-ST02-1.6 (PROP-ST02-5); RG-ST02-3.1, 3.2; RG-ST02-6.4 (INV-ST02-3); RG-ST02-6.5 (INV-ST02-4, local part); RG-ST02-7.2 (b), (d), Q-A (b)
- **Dependencies**: T-3
- **Red** (`crates/core/tests/store.rs`):
  - "issue from a restored state = issue from the original": for any sequence of issues/receives on a `HybridClock`, a clock restored from its `(l, c)` issues the same next stamp (property, 1,000 cases);
  - `stamp_local`: `K' = (sₙ.l, sₙ.c)`; stamps strictly increasing in list order; `seen` of the i-th change covers the earlier changes of the same list (A-1: same field, deletion, deletion after field); US-1 case 8 (`pt = 0` after `K = (10⁹, 3)` → `(10⁹, 4, D)`); every stamp has `a = D` (US-4 case 6);
  - rejections: US-3 case 10 (2nd change with empty field → `Rejection::Local { index: 1, Invalid(EmptyIdentifier) }`), `ClockExhausted` at `c = u32::MAX` / `l ≥ L_MAX` → rejected; in both cases the input `Log` and `K` are unchanged;
  - `rebuild`: a known write failing `validate` → `Damage::StoredWrite`; a known stamp above `K` → `Damage::ClockState` (boundary: equal to `K` passes); a known local-device pair (two contents, `stamp.a = D`) → `Damage::LocalIdentityPair`; a peer pair → no damage, reported as an anomaly.
- **Green**: `pub(crate) fn Log::with_clock` in `merge.rs` (only change there); `ClockState` ↔ `HybridClock` via `new` + `receive` inside `store.rs`; `rebuild`, `stamp_local` as in the archi.
- **Deliverable**: `crates/core/src/merge.rs` (one constructor), `crates/core/src/store.rs`.
- **Completion criterion**: tests above green; ST-01 suites unchanged and green; `git diff main -- crates/core/src/{hlc,write,clock}.rs` empty and `merge.rs` diff = the constructor only (INV-ST02-3); `grep -n "store" crates/core/src/{merge,hlc,write}.rs` finds no reference to the port.
- **Effort**: agent 20 min · arbitration 0

### T-5: Received path, idempotence, Q-A guard (`store.rs` part 2)
- **DDD zone**: Core (Full)
- **Spec refs**: RG-ST02-1.6 (received), RG-ST02-5.3 (idempotence, late write), RG-ST02-6.5 (`BatchError`), Q-A guard; CONFLICT-ST02-1
- **Dependencies**: T-4 (same file)
- **Red** (`crates/core/tests/store.rs`):
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
- **Spec refs**: PROP-ST02-9; RG-ST02-1.5 (PROP-ST02-4), RG-ST02-5.1 to 5.4, RG-ST02-6.7, RG-ST02-7.4 (INV-ST02-6, generic names)
- **Dependencies**: T-3 (types); can be written in parallel with T-4/T-5 (different file)
- **Red**: the suite itself, run on a `SimStore` stub, fails.
  - Test-only trait `Harness { fn fresh() -> Self::Store; fn reopen(&mut self) }`, generic over `S: Store`.
  - Generated `Vec<Op>` (≤ 50): local apply (1–5 changes, `pt` from a small domain including rewinds), received apply (fresh peer writes, duplicates of stored writes, peer anomalies, late writes with old stamps), advance watermark (lower, equal, higher), every read, reopen. Module/category/field names generated at random (INV-ST02-6).
  - Reference: ST-01 `Log` of `D` + ordered list of stored writes + `BTreeMap<DeviceId, Watermark>`. After each step every read is compared; positions compared by order (strictly increasing, stable across reopen), not by value.
  - Named cases from the matrices: US-5 cases 1, 2, 3 (10,000 writes on one field), 5, 6, 8, 9, 10, 12; US-1 case 6 (unusual write round trip).
  - `config()` from `support/mod.rs` (1,000 sequences, fixed seed).
- **Green**: comes with T-7 (SimStore) and T-9 (SQLite).
- **Deliverable**: `crates/core/tests/support/contract.rs`; its call site in `crates/core/tests/store.rs`.
- **Completion criterion**: compiles; fails on the stub; a deliberately wrong reference (e.g. positions reused) is caught by the suite (checked once, not committed).
- **Effort**: agent 25 min · arbitration 0

### T-7: `SimStore` with fault injection; non-`Send` compile test
- **DDD zone**: Supporting (Std)
- **Spec refs**: RG-ST02-6.7, AC-6.8; RG-ST02-6.2 (INV-ST02-2); RG-ST02-1.3, 2.1, 2.2, 5.5 on the simulator; E-ST02-2 on `SimStore`
- **Dependencies**: T-5, T-6
- **Red** (`crates/core/tests/store.rs`):
  - contract suite on `SimStore`;
  - `full_after(k)`: E-ST02-2 levels (a)–(d), 100 cases per level, batches 1–20; `StorageFull`, `Σ` unchanged (writes, positions, `K`, watermarks); retry after `full_after(∞)` gives `Σ` of a single success (RG-ST02-2.2); reads still work (RG-ST02-2.3);
  - `unavailable(n)`: next `n` operations → `Unavailable`, `Σ` unchanged, no partial read (RG-ST02-5.5);
  - `crash_next_write()`: US-6 case 8, batch absent after `reopen()`, handle refuses everything until then;
  - poison: an injected `Corrupted` makes every later operation return it until `reopen()`;
  - compile test: a generic `async fn use_store<S: Store>(s: &S)` instantiated with `SimStore` (not `Send`, not `Sync`, asserted with a negative static check); US-6 case 3, two futures created on one handle then polled with `Waker::noop()`, both `Ready`.
- **Green**: `sim_store.rs` as in the archi (`RefCell`, same pure functions, positions from 1).
- **Deliverable**: `crates/core/src/sim_store.rs`.
- **Completion criterion**: tests green; `20-st01-browser.sh` green with `--features sim` too (`cargo check --target wasm32-unknown-unknown -p arbitre-core --features sim`); local mutation of `sim_store.rs` ≥ 70%.
- **Effort**: agent 20 min · arbitration 0

### T-8: SQLite open: creation, verification, settings, error mapping
- **DDD zone**: Generic
- **Spec refs**: RG-ST02-1.1 (PROP-ST02-1), RG-ST02-4.1, 4.2 (PROP-ST02-10), RG-ST02-7.1 (INV-ST02-6), RG-ST02-7.3, archi "Open sequence", "Error mapping"
- **Dependencies**: T-3 only (types). **Runs in parallel with T-4 to T-7.**
- **Red** (`crates/sqlite/tests/settings.rs`):
  - US-1 case 11: `settings()` = WAL, `synchronous = FULL`, `fullfsync = ON` on every opened handle (fresh and reopened);
  - US-4 cases 1, 2: creation gives a random `D`, `K = (0, 0)`, `user_version = 1`, the fixed `application_id`; reopening gives the same `D`;
  - US-4 case 5: 1,000 stores created in a row → 1,000 distinct `D`, none all-zero;
  - US-7 case 1 (`V = 1`); US-7 case 2 (`user_version = 2` → `Unavailable(IncompatibleVersion { stored: 2, supported: 1 })`, file byte-identical).
- **Green**: `SqliteStore::open` (steps 1–3 of the archi), per-connection pragmas, `busy_timeout(options.wait)`, error mapping table, poison slot, `device()`.
- **Deliverable**: `crates/sqlite/src/lib.rs` (layout v1 DDL included, P-1).
- **Completion criterion**: tests green; the error mapping is one `match` on `rusqlite::ErrorCode` covering every row of the archi table.
- **Effort**: agent 25 min · arbitration 0

### T-9: SQLite `impl Store`; contract suite on SQLite; first poll; busy
- **DDD zone**: Generic (blocking properties)
- **Spec refs**: PROP-ST02-9 on SQLite; RG-ST02-1.4 (PROP-ST02-3); RG-ST02-3.4 (PROP-ST02-8, in-process); RG-ST02-6.3 (PROP-ST02-11); RG-ST02-6.5 on SQLite
- **Dependencies**: T-5, T-6, T-8
- **Red**:
  - `crates/sqlite/tests/contract.rs`: includes `../../core/tests/support/{mod,contract}.rs` by `#[path]`, runs the suite on `SqliteStore` (fresh store per sequence; `reopen` = drop and open);
  - round trip (RG-ST02-1.4): property over valid writes with the archi's edge domain (empty and non-UTF-8 values, identifiers with NUL and non-ASCII, 0–3 `seen` entries, `l = L_MAX − 1`, `c = u32::MAX`, every `Author`), received then read back through `writes_of`, `writes_since` and after reopen, `==` on `Write`;
  - `settings.rs`: US-6 case 4, every operation polled once with `Waker::noop()` is `Ready`; US-6 case 5 and US-3 cases 6–8 in-process (P-6): a raw connection holds `BEGIN IMMEDIATE`; with `wait = 200 ms`, held 400 ms → `Ready(Unavailable(Busy))` after 200 ms ≤ t ≤ 1.2 s, batch absent; held 100 ms → success; retry after release → success;
  - rejections on SQLite: US-3 case 10 and US-6 case 6 leave the file's `Σ` unchanged (positions, `K`).
- **Green**: reads in `BEGIN DEFERRED`, writes in `BEGIN IMMEDIATE` (load `K` and touched records, call `rebuild` + `stamp_local`/`receive`, insert, update `K`, `COMMIT`), unconditional `ROLLBACK` on error, `advance_watermark` as `max`.
- **Deliverable**: `crates/sqlite/src/lib.rs`, `crates/sqlite/tests/{contract,settings}.rs`.
- **Completion criterion**: contract suite green on SQLite with 1,000 sequences; round trip green; first-poll and busy tests green 20 times in a row locally (`cargo test -p arbitre-sqlite --test settings` in a loop).
- **Effort**: agent 25 min · arbitration 0

### T-10: Forced-stop harness (E-ST02-1) and concurrent creation
- **DDD zone**: Generic (blocking)
- **Spec refs**: E-ST02-1; RG-ST02-1.2, 1.3, 1.5, 1.6 (PROP-ST02-2, 4, 5); RG-ST02-4.1 (US-4 cases 3, 4)
- **Dependencies**: T-9
- **Red** (written first, run against a broken step to prove it bites: the adapter temporarily reports success **before** `COMMIT`; the harness must fail; the step is reverted):
  - `tests/support/child.rs`: re-exec protocol (env var role, `--exact`, line-based stdout "confirmed <run> <batch>" printed after the success result, stdin "go" lines);
  - `tests/crash.rs`: 200 single-writer runs + 50 two-writer runs (one killed); batches of 1–5, local and received mixed; unique value per (run, batch, index); `Child::kill()` after a seeded delay (R-6 bias); reopen and check: every recorded confirmation present and identical; at most one unconfirmed batch, entire or absent; `R(W)` equals the reference `Log` fed with present batches; `K ≥` every stored stamp and the next local stamp is above all of them; survivor's confirmed writes all present (US-1 cases 1–5, 7, 12);
  - US-4 case 3: two children open the same empty path at once → one `D`, both report it; US-4 case 4: kill during first open (repeated, seeded) → next open is a fresh creation or a complete store, never one without `D`.
- **Green**: harness passes on the real adapter; any adapter defect found is fixed in `src/lib.rs` (collision rule below).
- **Deliverable**: `crates/sqlite/tests/{crash.rs,support/child.rs}`.
- **Completion criterion**: green at spec counts; seed printed on failure and replayable via env var; run time recorded for T-15.
- **Effort**: agent 30 min · arbitration 0

### T-11: Interleaved processes (E-ST02-3)
- **DDD zone**: Generic (blocking)
- **Spec refs**: E-ST02-3; RG-ST02-3.1 to 3.5 (PROP-ST02-7, 8)
- **Dependencies**: T-10 (reuses and extends `support/child.rs`)
- **Red** (broken step: the adapter opens writes with `BEGIN DEFERRED` instead of `IMMEDIATE`, so `K` is read outside the lock; the harness must catch duplicate stamps or a conflict):
  - `tests/interleave.rs`: two writer children + one reader child, driven step by step; 100 generated schedules, 2–20 ops per process, high share of same-field writes, including "held 2δ" and "held δ/2" (δ = 200 ms, ε = 1 s); checks: commit order = stamp order; later `seen` covers earlier on same field/deletion; no conflict, no anomaly, distinct stamps; every confirmed write stored; `R(W)` = sequential application in commit order; reader sees 0 or all writes of a concurrent batch (US-3 cases 1–9).
- **Green**: passes on the real adapter.
- **Deliverable**: `crates/sqlite/tests/interleave.rs`, additions to `support/child.rs`.
- **Completion criterion**: green at spec count; replayable seed; run time recorded.
- **Effort**: agent 30 min · arbitration 0

### T-12: Full storage (E-ST02-2) with a fault VFS
- **DDD zone**: Generic (blocking)
- **Spec refs**: E-ST02-2; RG-ST02-2.1 to 2.4 (PROP-ST02-6); RG-ST02-1.3; RG-ST02-4.1 (full at first opening)
- **Dependencies**: T-9. **Parallel with T-10/T-11 and T-13.**
- **Red** (broken step: the adapter skips `ROLLBACK` after an error; level (b) must then fail on retry or leave a position gap):
  - `tests/support/full_vfs.rs`: VFS registered via `rusqlite::ffi`, delegating to the default VFS, shared byte budget, `xWrite` → `SQLITE_FULL` when spent, optional `xSync` failure;
  - `tests/full.rs`: 100 generated cases per level (a) before the batch, (b) inside the batch's frames, (c) all but the commit frame, (d) during `advance_watermark`; batches 1–20; each: `StorageFull`, `Σ` unchanged (writes, positions, `K`, `D`, watermarks, `V`), then budget lifted and retry = single success (US-2 cases 1–4, 7, 8); US-2 case 6: full, existing store, open and read → data; US-2 case 5: full on empty location → `StorageFull`, no `D`, later open creates; `xSync` failure → `Unavailable(Io)`, `Σ` unchanged.
- **Green**: passes; adapter defects fixed in `src/lib.rs` (collision rule).
- **Deliverable**: `crates/sqlite/tests/{full.rs,support/full_vfs.rs}`.
- **Completion criterion**: green at spec counts; if R-1 triggers, stop and report to Quentin before falling back.
- **Effort**: agent 35 min · arbitration 0 (unless R-1)

### T-13: Damage and incompatible versions (E-ST02-4)
- **DDD zone**: Generic (blocking)
- **Spec refs**: E-ST02-4; RG-ST02-7.2, 7.3, 5.5 (PROP-ST02-12); RG-ST02-1.6 error case; AMB-ST02-3
- **Dependencies**: T-9. **Parallel with T-10/T-11 and T-12.**
- **Red** (broken step: `open` skips the `K` vs max-stamp check; case 10 must fail):
  - `tests/damage.rs`: build, close (WAL checkpointed), damage from outside with a raw connection (`ignore_check_constraints`, see Under-engineering), then open and read everything; file byte-identical after each attempt:
    - US-7 case 3: truncation at a random length (seeded, 20 lengths) → `Corrupted`;
    - US-7 case 4: non-UTF-8 identifier BLOB → `Corrupted(StoredWrite { position })` from the first read reaching it; every later op on the handle → same `Corrupted`;
    - US-7 case 5: stored `seen ≥ stamp` → `Corrupted(StoredWrite)`;
    - US-7 case 6: text file at the path → `Corrupted(NotAStore)`; foreign `application_id` → `NotAStore`;
    - (c): missing `device` row → `DeviceId`; `user_version = 0` with tables → `Layout`;
    - US-1 case 10: `K` below the largest stored stamp → `Corrupted(ClockState)`, store refused;
    - Q-A (b): two contents for one local identity inserted by SQL → `Corrupted(LocalIdentityPair)` on `writes_of` / `state` / `records` of that record;
    - US-5 case 11: a directory at the store path → `Unavailable(Io)` (P-5).
- **Green**: passes; adapter defects fixed in `src/lib.rs` (collision rule).
- **Deliverable**: `crates/sqlite/tests/damage.rs`.
- **Completion criterion**: green; each case asserts both the error **and** the byte-identical file.
- **Effort**: agent 20 min · arbitration 0

### T-14: Glossary rows in `CONTRIBUTING.md`
- **DDD zone**: Generic (docs)
- **Spec refs**: archi "Impacted modules" (`CONTRIBUTING.md` additive)
- **Dependencies**: T-2 merged (PR 1 also edits `CONTRIBUTING.md`; write on the rebased branch to avoid a conflict)
- **Description**: glossary rows for store / `Store`, local apply / `apply_local`, received apply / `apply`, position / `Position`, watermark / `Watermark`, clock state / `ClockState`, `StorageFull` / `Corrupted` / `Unavailable` (French `StockagePlein`, `Corrompu`, `Indisponible`). No rule changes.
- **Deliverable**: `CONTRIBUTING.md` (glossary table only).
- **Completion criterion**: `git diff main -- CONTRIBUTING.md` touches only glossary rows.
- **Effort**: agent 5 min · arbitration 0

### T-15: Measure CI time, settle run counts, full local verdict
- **DDD zone**: Generic
- **Spec refs**: GAP-ST02-2 (counts, floor = half); R-2
- **Dependencies**: T-7, T-9 to T-14
- **Description**: rebase on `main`; run `scripts/ci.sh` locally; record the duration of each harness (`cargo test -p arbitre-sqlite -- --report-time` or per-binary timing) and of `10-st01-tests` as a whole; apply the rule of Q-P1; record final counts and timings in the task report (they go into the PR body).
- **Deliverable**: possibly lowered counts in `crates/sqlite/tests/*.rs` (never below the floor; PROP-ST02-9 never lowered).
- **Completion criterion**: `scripts/ci.sh` green locally; `target/ci/mutation.md` has rows for `store.rs` (≥ 85%) and `sim_store.rs` (≥ 70%) and **no** `crates/sqlite/src` row.
- **Effort**: agent 15 min · arbitration 5 min (Q-P1 if the threshold is crossed)

### T-16: Open and merge PR 2
- **DDD zone**: Core (story completion)
- **Spec refs**: cadre § ST-02 completion criteria; RG-ST02-G.2
- **Dependencies**: T-2 (merged), T-15
- **Description**:
  - Check R-8: branch rebased on `main`; `scripts/ci.d/50-st01-mutation.sh` contains the Gen exclusion; `git diff main --name-only` has no `scripts/` or `.github/` path.
  - On Quentin's go: push `st-02/store`, open PR 2 (body: what ships, counts and timings from T-15, blocking properties and where they run, no label needed).
  - For each surviving mutant of `store.rs` or `sim_store.rs`: a killing test, or an entry in `.ctdd/stories/ST-02/mutants_equivalents.toml` with a justification.
  - **Quentin**: reviews equivalents and the PR, merges (squash, hand-written message).
- **Deliverable**: PR 2 merged; `mutants_equivalents.toml` if needed.
- **Completion criterion**: `ci` and `judge` green on PR 2; `mutation.md` of the CI run: `store.rs` ≥ 85%, `sim_store.rs` ≥ 70%, `merge.rs` diff neutral or ≥ 85%, no Gen row; each equivalent reviewed by Quentin; merged.
- **Effort**: agent 30 min · arbitration 15 min (equivalents, PR read, merge)

## Execution order

```
PR 1:  T-1 ─► T-2 (Quentin: label + merge) ─────────────────────────────────┐
                                                                            │
PR 2:  T-3 ─┬─► T-4 ─► T-5 ─┬─► T-7 ───────────────────────────────┐        │
            ├─► T-6 ────────┤                                      │        │
            └─► T-8 ────────┴─► T-9 ─┬─► T-10 ─► T-11 ─────────────┤        │
                                     ├─► T-12 ─────────────────────┼─► T-15 ─┴─► T-16
                                     └─► T-13 ─────────────────────┤
                         (after T-2) T-14 ─────────────────────────┘
```

### Parallelism (separate agents) and collisions

| Can run together | Files owned | Collision and rule |
|---|---|---|
| T-1/T-2 ∥ all of PR 2 up to T-15 | `scripts/**`, `CONTRIBUTING.md` (Zones, Equivalents) on `st-02/gen-mutation` | `CONTRIBUTING.md` is also touched by T-14 (glossary): T-14 waits for T-2 and works on the rebased branch |
| T-4 → T-5 (one agent) ∥ T-6 ∥ T-8 | T-4/T-5: `store.rs`, `merge.rs`, `tests/store.rs`; T-6: `tests/support/contract.rs`; T-8: `crates/sqlite/src/lib.rs`, `tests/settings.rs` | `tests/store.rs` is touched by T-4/T-5 (unit tests) and T-6 (one call site): T-6 adds its call site in a separate `#[test]` at the end of the file. `lib.rs` re-exports are fixed by T-3 |
| T-10 → T-11 (one agent) ∥ T-12 ∥ T-13 | T-10/T-11: `tests/{crash,interleave}.rs`, `support/child.rs`; T-12: `tests/full.rs`, `support/full_vfs.rs`; T-13: `tests/damage.rs` | Test files are disjoint (P-2). **`crates/sqlite/src/lib.rs` is the hot spot**: a harness agent that finds an adapter defect commits the fix as a separate commit touching only `src/lib.rs`, and the agents rebase one at a time (first finished, first rebased). T-11 depends on T-10 because both write `support/child.rs` |
| T-7 ∥ T-9 | T-7: `sim_store.rs`; T-9: `crates/sqlite/**` | None |

- Worktrees: PR 1 in its own worktree from `main`; PR 2's parallel agents each in a worktree from `st-02/store`, merged back in the order of the diagram.
- Pushes happen only in T-2 and T-16, on Quentin's go.

### Estimate (agent time and arbitration time)

| Lane | Tasks | Agent (min) | Arbitration (min) |
|---|---|---|---|
| PR 1 | T-1, T-2 | 25 | 10 |
| Core | T-3 → T-4 → T-5, T-6, T-7 | 95 | 0 |
| SQLite | T-8, T-9, T-10 → T-11 ∥ T-12 ∥ T-13 | 165 (≈ 110 in duration) | 0 (R-1 aside) |
| Closing | T-14, T-15, T-16 | 50 | 20 |
| **Total** | | **≈ 335 cumulated, ≈ 210 in duration** (SQLite lane is the critical path) | **≈ 30** |

- Frame: M (100 min machine) + "moyenne (2)". Machine time is about twice M, like ST-01 (+80%); arbitration fits "moyenne". Recorded in `state.json` by the orchestrator, not corrected.
- Not counted: DA iterations, CI wall time of the harnesses (measured in T-15).

## C-TDD invariants to check after implementation

| Invariant / Property | Task(s) | Verification |
|---|---|---|
| INV-ST02-1 core pure, wasm32 | T-3, T-7 | `20-st01-browser.sh`; wasm32 check with `--features sim` |
| INV-ST02-2 async, `&self`, no `Send` | T-7 | Compile test with non-`Send`/non-`Sync` `SimStore`; two futures on one handle |
| INV-ST02-3 merge engine port-free | T-4 | Diff of `hlc/write/clock.rs` empty; `merge.rs` diff = `with_clock`; ST-01 suites unchanged |
| INV-ST02-4 closed error set, `Rejected ⇒ Σ unchanged` | T-3, T-4, T-5, T-9 | Enums closed by type; rejection tests on core and SQLite |
| INV-ST02-5 SQLite only in `arbitre-sqlite` | T-1, T-3 | `30-st01-graph.sh`; test `arbitre-plan → arbitre-sqlite` red |
| INV-ST02-6 `V = 1`, generic layout | T-6, T-8, T-9 | `user_version = 1`; contract suite and round trip with random names |
| INV-ST02-7 no network | T-3 | `cargo tree -e normal` of `arbitre-sqlite` |
| PROP-ST02-1 durable confirmation | T-8, T-10 | `settings()`; forced stop |
| PROP-ST02-2 confirmed survives, all or nothing | T-10 | `crash.rs` (200 + 50) |
| PROP-ST02-3 exact round trip | T-9 (T-6 named case) | Round-trip property through SQLite and reopen |
| PROP-ST02-4 rebuilt log = reference | T-6, T-7, T-9, T-10 | Contract suite with reopen; crash reference |
| PROP-ST02-5 clock persisted, monotone | T-4, T-5, T-10, T-13 | Unit cases 8, 9; post-crash check; damage case 10 |
| PROP-ST02-6 full storage | T-7, T-12 | `full_after` levels; fault VFS levels (a)–(d) |
| PROP-ST02-7 interleaved writers | T-4, T-11 | `seen` within a list; `interleave.rs` |
| PROP-ST02-8 busy, readers whole batches | T-9, T-11 | In-process busy timing; reader child |
| PROP-ST02-9 contract suite | T-6, T-7, T-9 | Same suite on `SimStore` and `SqliteStore`, 1,000 sequences each |
| PROP-ST02-10 device identifier | T-8, T-10, T-12 | Create/reopen, 1,000 distinct; concurrent creation, kill during creation; full on empty location |
| PROP-ST02-11 first poll | T-9 | Every op `Ready` with `Waker::noop()`, busy included |
| PROP-ST02-12 damage, newer layout, unreadable | T-7, T-8, T-13 | `damage.rs`; `IncompatibleVersion`; poison |

## Traceability matrix (AC → rule/property → task → test)

| AC | Rule / property | Task | Test (file :: subject) |
|---|---|---|---|
| AC-1.1 | RG-ST02-1.1, PROP-ST02-1 | T-8, T-10 | `sqlite/tests/settings.rs` :: durability settings; `crash.rs` |
| AC-1.2, AC-1.4 | RG-ST02-1.2, PROP-ST02-2 | T-10 | `sqlite/tests/crash.rs` :: single writer, two writers one killed |
| AC-1.3 | RG-ST02-1.3 | T-7, T-10, T-12, T-9 | `core/tests/store.rs` :: crash/full injection; `crash.rs`; `full.rs`; `settings.rs` :: busy batch absent |
| AC-1.5 | RG-ST02-1.4, PROP-ST02-3 | T-9 | `sqlite/tests/contract.rs` :: round trip |
| AC-1.6 | RG-ST02-1.5, PROP-ST02-4 | T-6, T-7, T-9, T-10 | `support/contract.rs` (reopen op); `crash.rs` :: rebuilt log |
| AC-1.7 | RG-ST02-1.6, PROP-ST02-5 | T-4, T-5, T-13 | `core/tests/store.rs` :: US-1 cases 8, 9; `damage.rs` :: clock below stamp |
| AC-2.1 to AC-2.4 | RG-ST02-2.1 to 2.4, PROP-ST02-6 | T-7, T-12 | `core/tests/store.rs` :: `full_after` levels; `sqlite/tests/full.rs` :: levels (a)–(d), retry, reads, empty location |
| AC-3.1, AC-3.5 | RG-ST02-3.1, 3.3, PROP-ST02-7 | T-4, T-11 | `core/tests/store.rs` :: seen within a list; `interleave.rs` |
| AC-3.2 | RG-ST02-3.2 | T-4, T-11 | `core/tests/store.rs` :: distinct increasing stamps; `interleave.rs` :: distinct stamps |
| AC-3.3 | RG-ST02-3.4, PROP-ST02-8 | T-9, T-11 | `settings.rs` :: busy held 2δ / δ/2 / retry; `interleave.rs` |
| AC-3.4 | RG-ST02-3.5, PROP-ST02-8 | T-11 | `interleave.rs` :: reader child |
| AC-4.1, AC-4.2 | RG-ST02-4.1, PROP-ST02-10 | T-8, T-10, T-12 | `settings.rs` :: create/reopen; `crash.rs` :: concurrent creation, kill during creation; `full.rs` :: empty location |
| AC-4.3 | RG-ST02-4.2 | T-8 | `settings.rs` :: 1,000 distinct identifiers |
| AC-5.1 to AC-5.4 | RG-ST02-5.1 to 5.4, PROP-ST02-9 | T-6, T-7, T-9 | `support/contract.rs` :: model-based suite + US-5 named cases, on both adapters |
| AC-5.5 | RG-ST02-5.5, PROP-ST02-12 | T-7, T-13 | `core/tests/store.rs` :: `unavailable(n)`; `damage.rs` :: directory at path, damaged reads |
| AC-6.1 | RG-ST02-6.1, INV-ST02-1 | T-3, T-7 | `ci.d/20-st01-browser.sh`; wasm32 check with `sim` |
| AC-6.2, AC-6.3 | RG-ST02-6.2, INV-ST02-2 | T-7 | `core/tests/store.rs` :: non-`Send` compile test, two futures |
| AC-6.4 | RG-ST02-6.3, PROP-ST02-11 | T-9 | `settings.rs` :: first poll, busy included |
| AC-6.5 | RG-ST02-6.4, INV-ST02-3, CONFLICT-ST02-1 | T-4, T-5 | diff check of core modules; `apply_local` / `apply` tests |
| AC-6.6 | RG-ST02-6.5, INV-ST02-4, CONFLICT-ST02-2 | T-3, T-4, T-5, T-9 | `core/tests/store.rs` :: rejections; `settings.rs` :: rejections on SQLite |
| AC-6.7 | RG-ST02-6.6, INV-ST02-5 | T-1, T-3 | `ci.d/30-st01-graph.sh`; `scripts/lib/test_ci.py` :: plan → sqlite red |
| AC-6.8 | RG-ST02-6.7, PROP-ST02-9 | T-7 | `core/tests/store.rs` :: contract suite on `SimStore`, injections |
| AC-7.1 | RG-ST02-7.1, INV-ST02-6 | T-8 | `settings.rs` :: `user_version = 1` |
| AC-7.2 | RG-ST02-7.2, Q-A (b), PROP-ST02-12 | T-4, T-5, T-13 | `core/tests/store.rs` :: `rebuild` damage, Q-A guard; `damage.rs` |
| AC-7.3 | RG-ST02-7.3 | T-8 | `settings.rs` :: newer layout refused, file unchanged |
| AC-7.4 | RG-ST02-7.4, INV-ST02-6 | T-6, T-9 | contract suite and round trip with random names |
| (RG-ST02-G.1) | INV-ST02-7 | T-3 | `cargo tree` check |
| (RG-ST02-G.2, Q-M) | zones, Gen not mutated | T-1, T-3, T-15, T-16 | `test_ci.py` :: `gen_files`; `mutation.md` without Gen rows |

## Open questions for Quentin

Each recommendation applies by default if Quentin says nothing.

| # | Question | Options | Recommendation |
|---|---|---|---|
| Q-P1 | **CI time threshold for lowering run counts** (GAP-ST02-2 says "if measured too high", with no number). | (a) Keep spec counts whatever the time. (b) If `10-st01-tests` exceeds **10 min** on CI, halve E-ST02-1, -2, -3 counts (the floor), never PROP-ST02-9. (c) Halve now. | **(b).** The floor is already accepted by the specs; 10 min keeps a PR under a coffee break. |
| Q-P2 | **F-27 procedure for PR 1.** The frame makes a foundation amendment a mini-cycle `ST-Ax` with a strong arbitration that freezes other lanes. | (a) No mini-cycle: your Q-M arbitration is the strong arbitration; PR 1 is planned here as T-1/T-2 and only the Socle lane is open. (b) Run `ST-A1` as its own cycle. | **(a).** Nothing else is open to freeze, and the change is ~30 lines plus tests. |
