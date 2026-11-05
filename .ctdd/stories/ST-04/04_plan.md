# Implementation Plan — ST-04: Modules you can enable, kept apart, with one generic conformance test

## Traceability
- Specs: `.ctdd/stories/ST-04/02_specs.md` v2: § Resolution (Q1 to Q7 answered (a), 2026-11-05) and § DA iteration 1 → specs v2, both binding.
- Story: `.ctdd/stories/ST-04/01_story.md`.
- Frame: `.ctdd/cadre/04_plan.md` § ST-04 (Supporting / Std, size M, arbitration "moyenne"), § Collisions, F-12, F-18, F-27, F-37.
- ST-03: `04_plan.md` P-3, P-4, P-11, "ST-04 handover"; `zones.toml`.
- DA: `05_da_review.md` (iteration 1, BLOCKED); v1 of this plan in `history/04_plan_v1.md`.
- Date: 2026-11-05 · Status: v2 (DA F-1 to F-15 folded; Q-DA1 settled (a), see § Plan questions).

## Technical context
- **Codebase** (`st-03/story`, read 2026-11-05):
  - `crates/core` (`arbitre-core`): `#![no_std]` + `alloc`, no normal dependency. `Clock { now }`, `SimClock` (feature `sim`), `Store` (`async fn`, `&self`, no `Send`; `records(module, category)` needs a category, no category listing), `SimStore` (feature `sim`), `RecordId { module, category, id }`, `DeviceId([u8; 16])`, `Author` (`System` exists), `catalog.rs` (`Catalog::build(&[(&str, Vec<ActionDescriptor>)])`, refuses a module id given twice; an empty enum is refused), `error.rs`, `value.rs`.
  - `crates/cli` (`arbitre`): `modules.rs` (`Wiring`, `Context`, `shipped()` empty, `all()`, `catalog()`, unit test `shipped_catalogue_builds`), `store.rs` (`location`, `open` = `create_dir_all` + `SqliteStore::open`, `map_store`, `map_apply`, `block_on` = one poll on `Waker::noop`), `cli/mod.rs` (runtime clap tree; `--json`, `--store` global), `main.rs` (JSON pre-scan, `catch_unwind`, one checked write), `wiring/test.rs` (module `test`; `arbitre.test-foundation` under id `arbitre`; `test.read` uses `writes_since`). Tests: `tests/{usage,output,codes,location,docs}.rs`, `tests/support/mod.rs`.
  - `crates/sqlite`: `SqliteStore::open(&Path, &Options)`; on a file with no table and `application_id = 0` it runs `create` (writes).
  - CI: `10` tests (`--all-features --locked`), `20` wasm32 check of `arbitre-core` alone, `30` crate graph, `40` gitleaks, `50` `cargo mutants --in-diff --all-features` (Gen excluded, every test target of the package run per mutant). Judge reds a lowered zone.
- **Stack**: Rust 1.99, edition 2024; `clap` 4.6.7 builder; `jiff` 0.2.39 (shared, `default-features = false`); `serde_json` dev only.
- **Patterns**: functional core / imperative shell; logic pushed into core where mutation is cheap (ST-03 R-2, Q-P2); red tests laid by the Skeleton; zones per story.

## `jiff` check (done 2026-11-05)

Scratch crate outside the workspace (`$SCRATCH/jiffcheck`, `#![no_std]` + `extern crate alloc`), `jiff = "=0.2.39"`, `default-features = false`, features `["alloc", "tzdb-bundle-always"]`:
- `cargo build --target wasm32-unknown-unknown`: **green**.
- `cargo tree -e features`: only `alloc` and `tzdb-bundle-always` on `jiff` (plus `jiff-core/alloc`, `jiff-tzdb`); **no `std`**.
- Native test: `TimeZone::get("Europe/Paris")` at `2026-03-29T01:30:00Z` → +02:00, at `00:30Z` → +01:00, from the bundled database.
- DA check (scratch `dyncheck`): `TimeZone::try_system` and `TimeZone::UTC` exist in 0.2.39.
- New core dependencies: `jiff`, `jiff-core`, `jiff-tzdb` (all no-std). Size measured by S1-A: about +120 to +135 KB gzip in the PWA.

Result: **no fallback needed.** Limit: `wasm32-unknown-unknown` has a `std`, so the build proves the feature set, not a `std`-less target; the feature tree shows `std` off. Fallback if a later `jiff` breaks this: a core type `TimeZoneId(String)` (IANA name) in `Clock`, resolved in `arbitre-plan`. Not built. `jiff` 0.2 becomes part of the frozen contract through `Clock::timezone` (DA F-15): a major `jiff` upgrade is a foundation amendment.

## Decisions taken by this plan (within the specs' latitude)

| # | Decision | Why |
|---|---|---|
| P-1 | `pub type LocalBoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>` in `module.rs`; `Module<S>` and `Ai` return it | Q3 (a); verified to compile by the DA |
| P-2 | `ModuleStore<'a, S>`: **private** fields `store: &'a S`, `module: &'static str`, `author`, `clock: &'a dyn Clock`; one `pub fn new(…)`; no getter returning the store. Reads take `(category, id)`; `apply(&[(category, id, Change)])` stamps with `clock.now()` and `author`. A `compile_fail` doctest shows `view.store` is unreachable (F-4) | RG-ST04-3.3 by construction |
| P-3 | `Context<'a, S> { author, clock: &'a dyn Clock, store: ModuleStore<'a, S>, ai: Option<&'a dyn Ai> }` (public fields; `store` is the view) | RG-ST04-3.2 |
| P-4 | `Catalog::build(foundation: &[ActionDescriptor], modules: &[(&str, Vec<ActionDescriptor>)])`. Refused: a listed id `arbitre`; an id not of the name-segment form; an id equal to a first command word of the foundation (`module`, `test`) (F-8) | RG-ST04-3.4 |
| P-5 | Pure helpers in core (`module.rs`): `pre_scan(args: &[&str]) -> PreScan { json, store: Option<&str>, first_word: Option<&str> }` reading `--store x` and `--store=x` **anywhere** before `--` (last wins), the first word being the first argument that is neither an option nor `--store`'s value (F-6); `enabled_from(record_state, build_ids) -> BTreeSet<&str>` (only `b"1"` enables; other values and unknown ids ignored, F-15); `state_change(module, on)` | Logic where mutation is cheap |
| P-6 | Enabled-state record `arbitre/module-state/<device hex>`, field = module id, value `b"1"`/`b"0"` | Q1 (a) |
| P-7 | Signals: pure parts in core `signal.rs` (record id `<device hex>-<ms>-<source>`, fields `at`, `source`, `message`, `read`; `unread(records, device)` sorted by `(at, id)`; `line(at, tz, message)`); `crates/cli/src/signals.rs` does the I/O | Same as P-5 |
| P-8 | Before parsing: location resolved **once** from `pre_scan` (a location error ⇒ default state; the action later reports `no_default_store`). `store::open_existing` reads the first 100 bytes: absent or 0 bytes ⇒ default state, no open; not the SQLite magic, or `application_id` ≠ `ARBT` at offset 68 ⇒ exit 3 `storage_corrupted`, no open; otherwise `SqliteStore::open` (which cannot reach `create`). The handle is kept for the action; an action with no handle opens (creates) at the same path (F-5, F-6) | RG-ST04-2.3 |
| P-9 | **The foundation is not a `Module` impl** (F-3). `wiring/arbitre.rs` is a CLI-side dispatcher holding the raw `&SqliteStore`, the registry and the clock. It owns `arbitre.module.*` and, under `test-actions`, ST-03's test actions renamed `arbitre.test.*` plus `arbitre.test-foundation` and `arbitre.test.signal` (F-1, F-2). `module.enable`/`.disable` are declared only if the build has a toggleable module; `module.list` always (F-7). `modules.rs`: `all() -> Vec<Box<dyn Module<SqliteStore>>>`, one line per module; no default-enabled list | Q-DA1 (a), F-1, F-3, F-7 |
| P-10 | Records of module `m` for revalidation = `writes_since(START)` filtered on `record.module == m`, deduplicated on `(category, id)`, deleted ones left out (projected state). Revalidation runs **before** the state write; a `StoreError` ⇒ exit 3, state unchanged | F-3 |
| P-11 | `block_on` (one poll) unchanged; a `Pending` stays `internal_error` | No real async I/O in ST-04 |
| P-12 | Native clock `crates/cli/src/clock.rs`: `SystemClock` (`now` from `SystemTime`, or from `ARBITRE_TEST_NOW` in ms when built with `test-actions`; `timezone` = `TimeZone::try_system()` else `TimeZone::UTC`). `arbitre` adds `jiff` features `std`, `tz-system` | RG-ST04-8.1; F-10 |
| P-13 | Observer `tests/support/network.rs`, frozen API `Observer::start(Mode::{Open, Cut}) -> Observer`, `env() -> Vec<(OsString, OsString)>`, `take() -> Vec<Attempt { host, port }>`: the attempts since the last call, read after each command (07_review R-8). `std::net::TcpListener` on `127.0.0.1:0`, one thread per connection, reads the request line (`CONNECT host:port` or absolute `http://…`); cut closes, open forwards. `env()` sets `HTTP_PROXY`, `HTTPS_PROXY`, `ALL_PROXY` in both cases and clears `NO_PROXY`/`no_proxy` (F-11) | Q5 (a) |
| P-14 | Surfaces `tests/support/surfaces/mod.rs`: `Env { store: PathBuf, vars: Vec<(OsString, OsString)> }`; `Called { code: u8, doc: serde_json::Value }` (a surface with no exit code takes `doc.error.code`, 0 on success); `trait Surface { fn name(&self) -> &str; fn actions(&self, env: &Env) -> BTreeSet<String>; fn call(&self, env: &Env, action: &str, input: &serde_json::Value) -> Called }`; `pub fn all() -> Vec<Box<dyn Surface>>`, one line per surface. `cli.rs::actions` walks `--help` recursively (what the user sees), not `module list` (F-9) | RG-ST04-5.4 |
| P-15 | Conformance discovery through `arbitre module list --json` (the catalogue source); visibility through each surface's `actions()`. Per surface: checks 1, 2 (visibility) and 7. CLI only: explicit call ⇒ 4, checks 3 to 6 (F-9). Each check starts from a fresh store with no signal. Isolation (check 4) compares writes after the setup position only, with a test-side `SqliteStore` (F-2). Comparisons (checks 5, 7) run on byte copies of one prepared store with `ARBITRE_TEST_NOW` set (F-10) | RG-ST04-5.1, 5.2 |
| P-16 | Every spawned command in the conformance and observer tests has a 10 s timeout (`try_wait` + 50 ms sleep); on timeout `kill()` then `wait()`, and the check fails "hang" (F-12) | RG-ST04-4.2, 4.3 |

## Pre-plan analysis

### Over-engineering detected
- **Open-mode forwarding in the proxy**: no ST-04 action connects. Kept (≈ 15 lines): PROP-2 for ST-10, 19, 30 needs it and the tooling is frozen after ST-04.
- **Subset enumeration**: 2 toggleable modules in the test build (`fake-a`, `fake-b`) ⇒ 4 subsets. Bounded by the pairs rule above 6 modules.
- **Not added**: an `ai` fake action; a positional argument (Q7); lazy store opening (AMB-ST04-1); `tokio`, `reqwest`; a macro for module registration; a completion generator; a default-enabled list (F-1).

### Under-engineering detected
- **Files that are not a store** (P-8): a 1-byte file or a schema-less SQLite file would be initialised by `SqliteStore::open`. The header check stops it.
- **`--store` after the first word** (P-5): `arbitre fake-a list --store p` and the D-2 CI flow.
- **Shipped build has no toggleable module** (P-9): an empty enum is refused by `Catalog::build`; `enable`/`disable` are left out.
- **The whole build is validated at every start** (P-4), then filtered, so a defect never hides in a disabled module and an enabled one can always be disabled.
- **The `module` enum is derived from `all()`**: `wiring/arbitre.rs` takes the toggleable ids as a parameter.
- **Signals are shown after the store is opened for an action, before `execute`**; marking read is one local apply with `Author::System`. A failure to show or mark writes one stderr line and never changes the action's outcome.
- **`arbitre.test.read`** (was `test.read`) cannot use `writes_since` through a view; as a foundation action it keeps the raw store, but its summary and the ST-03 tests that read namespace `test` move to namespace `arbitre`.

### Identified risks

| # | Risk | Probability | Impact | Mitigation |
|---|---|---|---|---|
| R-1 | ST-03 (PR #8) changes in review before merging | Medium | Rebase work | ST-04 code starts after PR #8 merges; `st-04/*` rebased on `main` then |
| R-2 | Mutation time of Std CLI files grows (cargo-mutants runs every test target per mutant) | Medium | `50-st01-mutation.sh` long | Pure parts in core (P-5, P-7); measured at the **end of T-7** with the full suite (F-12); above 30 min, stop and ask |
| R-3 | `dyn Module<S>` lifetimes | Low (verified by the DA) | T-2 slips | Skeleton compiles the trait and one dummy impl first |
| R-4 | A client that ignores proxy variables (raw TCP, DNS, some WebSocket) bypasses the observer | Low now | PROP-2 hole later | CONTRIBUTING rule (T-8): cut mode is a proxy refusal; each adapter story asserts its traffic **is** recorded through the observer API, in its own test file |
| R-5 | Conformance test is slow (spawns: ≈ 3 ms per read, ≈ 27 ms per write, DA measure) | Low | CI time | Measured in T-7; ceiling 3 min, else ask |
| R-6 | `jiff` features unify `std` into core on native builds | Low | None functionally | `20` checks `arbitre-core` alone for wasm32 |

## Tasks

**Convention.** The Skeleton (phase 6, T-1) lays down every type and signature and **all red tests**. Each later task makes its tests green in its own files. "Green" also means `cargo test --workspace --all-features --locked`, `20-st01-browser.sh`, `30-st01-graph.sh` pass.

### T-1: Skeleton — types, signatures, red tests, zones
- **DDD zone**: Generic
- **Spec refs**: architecture notes; RG-ST04-G.2; every E-ST04-n; DA F-1, F-2, F-4, F-14
- **Dependencies**: PR #8 merged; branch rebased on `main`
- **Description**:
  - Core: `module.rs` (`LocalBoxFuture`, `Module<S>`, `Context`, `ModuleStore` with private fields and `new`, `Problem`, `PreScan`, `pre_scan`, `enabled_from`, `state_change`), `ai.rs` (`Ai`, `AiError`, id mapping), `signal.rs` (P-7), `Clock::timezone` + `SimClock::set_timezone`, `Catalog::build` new signature; `lib.rs` lines; `Cargo.toml`: `jiff = { workspace = true, features = ["alloc", "tzdb-bundle-always"] }`. Bodies `todo!()`.
  - CLI: `modules.rs` (P-9), `wiring/arbitre.rs` (dispatcher, `module.*`), `wiring/arbitre_test.rs` (ST-03's test actions renamed `arbitre.test.*`, `test-foundation`, and `test.signal` with its dispatch arm and an empty body; bodies kept; `test.not-enabled` removed; `test.read` reads `arbitre` records), `wiring/fake.rs`, `signals.rs`, `clock.rs` signatures. `wiring/test.rs` is deleted. `Cargo.toml`: `jiff` features `std`, `tz-system`; `[[test]]` targets `modules`, `signals`, `network`, `conformance` with `required-features = ["test-actions"]`.
  - ST-03 tests adjusted: help-order list without `not-enabled`; expected namespace `arbitre` where a test reads `test` records; the "module not enabled" row of `codes.rs` points at a disabled `fake-a` (red until T-5).
  - Red tests: core `tests/module.rs` (E-ST04-9; pre-scan table; `enabled_from`; P-4 refusals), `tests/clock.rs` additions (E-ST04-8), `tests/signal.rs` (P-7); CLI `tests/modules.rs` (E-ST04-1, 2, 3, 7), `tests/signals.rs` (E-ST04-6), `tests/network.rs` (E-ST04-4), `tests/conformance.rs` (E-ST04-5), `tests/support/{network.rs, conformance.rs, surfaces/{mod.rs, cli.rs}}` with signatures.
  - `.ctdd/stories/ST-04/zones.toml` (below).
- **Deliverable**: files above, compiling, new tests red.
- **Completion criterion**: `cargo check --workspace --all-features --locked` green; `cargo tree -p arbitre-core -e normal` shows only `jiff` (and its two no-std crates); `20`, `30` green; judge green.
- **Effort**: agent 50 min · arbitration 0

### T-2: Core module contract and store view
- **DDD zone**: Supporting (Std) — `module.rs`, `ai.rs`, `catalog.rs`
- **Spec refs**: RG-ST04-2.2 (pre-scan), 3.1 to 3.6, 1.5, 1.6; INV-ST04-3, 4; DA F-4, F-6, F-8, F-15
- **Dependencies**: T-1
- **Description**: `ModuleStore` over any `S: Store` (P-2); `pre_scan` (P-5); `enabled_from`, `state_change` (P-5, P-6); `Catalog::build` (P-4); `AiError` → ids.
- **Deliverable**: the three core files.
- **Completion criterion**: `core/tests/module.rs` green on `SimStore`: every view operation targets `module = m`; writes of `fake-a` invisible to `fake-b`'s view; `apply` stamps with the given clock and author; `compile_fail` doctest on `view.store`; pre-scan table (options before the word, `--store` after the word, `--store=x`, `--store` twice, `--store=` empty, `--`, no word); `enabled_from` with values `1`, `0`, `x` and an unknown id; catalogue refuses a listed `arbitre`, an ill-formed id, an id `module`; foundation actions present whatever the enabled set.
- **Effort**: agent 35 min · arbitration 0

### T-3: Time zone and pinned time
- **DDD zone**: Supporting (Std) — `clock.rs` (core), `crates/cli/src/clock.rs`
- **Spec refs**: RG-ST04-8.1, 8.2; INV-ST04-6; DA F-10
- **Dependencies**: T-1
- **Description**: `Clock::timezone`; `SimClock` default UTC and setter; `SystemClock` with `ARBITRE_TEST_NOW` under `test-actions` (P-12).
- **Deliverable**: both clock files.
- **Completion criterion**: E-ST04-8 green; `20-st01-browser.sh` green. The CLI clock is covered by T-6's `TZ` case.
- **Effort**: agent 10 min · arbitration 0

### T-4: Foundation dispatcher, enabled state, `main` pipeline
- **DDD zone**: Supporting (Std) — `wiring/arbitre.rs`, `modules.rs`, `main.rs`, `store.rs`
- **Spec refs**: RG-ST04-1.1 to 1.7, 2.1 to 2.4, 3.4, 3.6; CONFLICT-ST04-1, 2; GAP-ST04-1, 2, 3; INV-ST04-1, 2, 5; DA F-1, F-3, F-5, F-6, F-7, F-8
- **Dependencies**: T-2, T-3
- **Description**:
  - `wiring/arbitre.rs`: `module.enable` / `.disable` (only if a toggleable module exists) / `.list` (P-9); revalidation hook point in `enable` (P-10; empty problems until T-5); `test.*` actions on the raw store.
  - `store.rs`: `open_existing` with the header check (P-8).
  - `main.rs`: pre-scan → location once → `open_existing` → validate the whole build → enabled set → disabled first word ⇒ exit 4 document → catalogue (foundation + enabled) → clap → action: reuse or open the handle → signals hook point (no-op until T-6) → dispatch (foundation or `execute` through `block_on`) → ST-03 tail unchanged. Help on an unreadable store ⇒ 3; `--version` ⇒ 0.
- **Deliverable**: files above.
- **Completion criterion**: `cli/tests/modules.rs` E-ST04-1 and E-ST04-2 green except rows needing `fake-a` data actions, including: 1-byte file and dropped-table SQLite file byte-identical after `--help` (exit 3 `storage_corrupted`); `fake-a list --store p` reads the state of `p`. `shipped_catalogue_builds` green. ST-03 tests green except the `codes.rs` "module not enabled" row (moved to T-5).
- **Effort**: agent 50 min · arbitration 0

### T-5: Fake module and revalidation on enable
- **DDD zone**: Generic (`wiring/fake.rs`) · Supporting (Std, the `enable` body in `wiring/arbitre.rs`)
- **Spec refs**: RG-ST04-5.5, 7.1, 7.2; US-3 rows 1–2; US-7 rows 1–3; DA F-3
- **Dependencies**: T-4
- **Description**: `Fake { id }` with `put`, `get`, `list`, `update`, `delete` (`logged_as = Delete`), `push` (`{Relay}`, `relay_not_configured`), `revalidate` (`invalid_value`); `fake-a`, `fake-b` in `all()` under `test-actions`; `module.enable` enumerates and revalidates (P-10) before writing the state.
- **Deliverable**: `wiring/fake.rs`, the `enable` body.
- **Completion criterion**: E-ST04-1 (all rows), E-ST04-2 (all rows), E-ST04-3 rows 1–2, E-ST04-7 green; the `codes.rs` "module not enabled" row green (E-ST04-10).
- **Effort**: agent 25 min · arbitration 0

### T-6: Signals
- **DDD zone**: Supporting (Std) — `signal.rs` (core), `crates/cli/src/signals.rs`, the `test.signal` body in `wiring/arbitre_test.rs` (Gen)
- **Spec refs**: RG-ST04-6.1 to 6.3; CONFLICT-ST04-5; DA F-2, F-13
- **Dependencies**: T-3, T-4
- **Description**: pure parts (P-7); `deposit`, `show_and_mark`; the `arbitre.test.signal` body (descriptor and dispatch arm laid by T-1); `main.rs` hook point filled with one call.
- **Deliverable**: files above.
- **Completion criterion**: `core/tests/signal.rs` and `cli/tests/signals.rs` green (matrix US-6 rows 1–5), plus one case spawned with `TZ=America/New_York` and `ARBITRE_TEST_NOW` that checks the local time in the signal line.
- **Effort**: agent 20 min · arbitration 0

### T-7: Observer, surface registry, generic conformance test
- **DDD zone**: tests (`crates/cli/tests/**`, not mutated)
- **Spec refs**: RG-ST04-4.1 to 4.4, 5.1 to 5.6; PROP-ST04-1 to 5; INV-ST04-7; DA F-2, F-9, F-10, F-11, F-12, F-15
- **Dependencies**: T-5 and T-6
- **Description**: `support/network.rs` (P-13, P-16); `support/surfaces/{mod.rs, cli.rs}` (P-14); `support/conformance.rs` (minimal input of RG-ST04-5.3, discovery and split of P-15, the seven checks); `tests/network.rs` (self-test with a std TCP client); `tests/conformance.rs` (one `#[test]` per check, each looping over every toggleable module and, where P-15 says so, every surface). Unit tests of the helpers: minimal input conforms for each subset type; a module with no action passes (US-5 row 4).
- **Deliverable**: files above.
- **Completion criterion**: E-ST04-4 and E-ST04-5 green on `fake-a`, `fake-b`; the fake CRUD scenario succeeds in cut mode; `fake-a push` → 3 `relay_not_configured` in < 10 s. **Mutation measurement** with the full suite: `cargo mutants --all-features -f 'crates/cli/src/**'` on the ST-04 Std files, per-mutant time × count; above 30 min, stop and ask Quentin. Conformance time written in the PR body (R-5).
- **Effort**: agent 55 min · arbitration 0 (unless the measurement escalates)

### T-8: Docs, glossary, close the PR
- **DDD zone**: Supporting (story completion)
- **Spec refs**: RG-ST04-G.2; § Collisions; DA F-9, F-11, F-15
- **Dependencies**: T-2 to T-7
- **Description**:
  - `README.md`: "Modules" section (enable, disable, list; nothing enabled on a fresh device; the D-2 CI flow with `--module` and `--store`).
  - `CONTRIBUTING.md`: glossary rows; "Adding a module" (one line in `modules.rs`, one `wiring/<m>.rs`; reserved ids `arbitre`, `module`, `test`); "Adding a surface" (one file and one line in `tests/support/surfaces/`; which checks run per surface, P-15); "Network clients" (never disable proxy settings; cut mode is a proxy refusal, not a network cut; a client that ignores the proxy variables needs its own support; each adapter story asserts PROP-2 in its own `tests/<adapter>.rs` through `Observer`); `ARBITRE_TEST_NOW`; the frozen list of RG-ST04-G.2, `jiff` 0.2 included.
  - Rebase on `main`, push and open the PR on Quentin's go. Body: frozen types and tooling, Q1–Q7, Q-DA1 settled, the `jiff` check, mutation and conformance times.
  - **Quentin**: reads the frozen contract, merges (squash, hand-written message).
- **Completion criterion**: `ci` and `judge` green; every ST-04 Std file ≥ 70 % mutation, survivors killed or listed in `mutants_equivalents.toml`.
- **Effort**: agent 20 min · arbitration 20 min

## Zones (`.ctdd/stories/ST-04/zones.toml`, written by T-1)

```toml
# ST-04 zones: format in CONTRIBUTING.md. A file not listed gets Full.
Std = [
  "crates/core/src/module.rs",
  "crates/core/src/ai.rs",
  "crates/core/src/signal.rs",
  "crates/core/src/clock.rs",
  "crates/core/src/catalog.rs",
  "crates/cli/src/main.rs",
  "crates/cli/src/modules.rs",
  "crates/cli/src/store.rs",
  "crates/cli/src/signals.rs",
  "crates/cli/src/clock.rs",
  "crates/cli/src/wiring/arbitre.rs",
]
Gen = ["crates/cli/src/wiring/fake.rs", "crates/cli/src/wiring/arbitre_test.rs"]
```

`crates/core/src/clock.rs` is already `Std` in ST-01's `zones.toml`. The strictest declaration wins (judge rule), so the CLI files above are `Std` despite ST-01's `Gen = ["crates/cli/src/**"]`. The test actions move from `wiring/test.rs` (Gen) into `wiring/arbitre.rs` (Std): to keep their mutation cost out of the Std gate, they stay in a separate file `wiring/arbitre_test.rs` (Gen, added to the list above by T-1), included by `wiring/arbitre.rs` under `test-actions` (listed `Gen` above). `wiring/test.rs` is deleted.

## Execution order

```
T-1 ─┬─► T-2 ─┐
     └─► T-3 ─┴─► T-4 ─┬─► T-5 ─┬─► T-7 ─► T-8
                       └─► T-6 ─┘
```

| Can run together | Files owned | Collision |
|---|---|---|
| T-2 ∥ T-3 | T-2: `module.rs`, `ai.rs`, `catalog.rs`; T-3: both `clock.rs` | None |
| T-5 ∥ T-6 | T-5: `wiring/fake.rs`, the `enable` body in `wiring/arbitre.rs`; T-6: `signal.rs`, `signals.rs`, the `test.signal` body in `wiring/arbitre_test.rs`, one call line in `main.rs` | None: T-1 laid both dispatch arms in separate files; `main.rs` hook point laid by T-4 |

### Estimate

| Lane | Tasks | Agent (min) | Arbitration (min) |
|---|---|---|---|
| Skeleton | T-1 | 50 | 0 |
| Core | T-2, T-3 | 45 | 0 |
| Binary | T-4, T-5, T-6 | 95 | 0 |
| Tooling | T-7 | 55 | 0 |
| Closing | T-8 | 20 | 20 |
| **Total** | | **≈ 265 cumulated, ≈ 225 in duration** | **≈ 20** (+ phases 1–2 and DA) |

Frame: M (100 min) + "moyenne". Machine ≈ 2.6 × M; recorded, not corrected (recalibration measures it).

## C-TDD invariants to check after implementation

| Invariant / Property | Task(s) | Verification |
|---|---|---|
| INV-ST04-1 invisible when disabled | T-4, T-7 | E-ST04-2 rows 1, 8; conformance check 2 through each surface's `actions()` |
| INV-ST04-2 explicit call ⇒ 4, store untouched | T-2, T-4 | E-ST04-2 rows 2–7 (non-store files included) |
| INV-ST04-3 view cannot name another namespace | T-2 | E-ST04-9; `compile_fail` doctest; conformance check 4 |
| INV-ST04-4 reserved ids refused | T-2 | `core/tests/module.rs` |
| INV-ST04-5 idempotent, foundation on | T-4 | E-ST04-1 rows 2, 6, 7 |
| INV-ST04-6 core wasm32, `jiff` only | T-1, T-3 | `20-st01-browser.sh`; `cargo tree` |
| INV-ST04-7 minimal input conforms | T-7 | helper unit test against `conforms_input` |
| PROP-ST04-1, 2 data kept, subsets | T-5, T-7 | E-ST04-1 rows 3–4; conformance checks 3, 5 (byte copies, pinned time) |
| PROP-ST04-3, 4, 5 network | T-7 | E-ST04-5 check 6 (cut), E-ST04-4 |
| PROP-ST04-6 signal once | T-6 | E-ST04-6 |
| PROP-ST04-7 per-device state | T-2, T-4 | E-ST04-1 row 10 |

## Traceability matrix (AC → rule → task → test)

| AC | Rule | Task | Test |
|---|---|---|---|
| AC-1.1, 1.2, 1.7 | RG-ST04-1.1, 1.2, 1.7 | T-4 | `modules.rs` (E-ST04-1) |
| AC-1.3, 1.4 | RG-ST04-1.3, 1.4 | T-5, T-7 | E-ST04-1 rows 3–4; conformance 3, 5 |
| AC-1.5, 1.6 | RG-ST04-1.5, 1.6 | T-2, T-4 | E-ST04-1 rows 8–10 |
| AC-2.1 to 2.6 | RG-ST04-2.1 to 2.4 | T-2, T-4 | E-ST04-2; conformance 1, 2 |
| AC-3.1, 3.6 | RG-ST04-3.1, 3.6 | T-1, T-4 | compiles; review of the T-8 diff |
| AC-3.2, 3.3 | RG-ST04-3.3 | T-2 | E-ST04-9, E-ST04-3, doctest |
| AC-3.4 | RG-ST04-3.4 | T-2 | `core/tests/module.rs` |
| AC-3.5 | RG-ST04-3.5 | T-1, T-2 | id mapping unit test |
| AC-3.7, 8.2 | RG-ST04-8.2 | T-1, T-3 | `20-st01-browser.sh` |
| AC-4.1 to 4.5 | RG-ST04-4.1 to 4.4 | T-7 | E-ST04-4, E-ST04-5 |
| AC-5.1 to 5.5 | RG-ST04-5.1 to 5.6 | T-5, T-7, T-8 | E-ST04-5; CONTRIBUTING |
| AC-6.1 to 6.4 | RG-ST04-6.1 to 6.3 | T-6 | E-ST04-6 |
| AC-7.1 to 7.3 | RG-ST04-7.1, 7.2 | T-5 | E-ST04-7 |
| AC-8.1 | RG-ST04-8.1 | T-3, T-6 | E-ST04-8; `TZ` case |
| AC-3.8 | RG-ST04-G.2 | T-8 | PR body, CONTRIBUTING frozen list |

## DA iteration 1 → plan v2

| Flaw | Change |
|---|---|
| F-1 [B] | Q-DA1 (a): test actions become foundation actions `arbitre.test.*` (P-9); no default-enabled list; specs RG-ST04-1.6, GAP-ST04-3 updated |
| F-2 [M] | Check 4 from the setup position, fresh store per check (P-15); `arbitre.test.signal` is a foundation action (P-9) |
| F-3 [M] | Foundation = CLI dispatcher with the raw store (P-9); record enumeration and revalidation before the state write (P-10) |
| F-4 [M] | `ModuleStore` private fields + `compile_fail` doctest (P-2, T-2) |
| F-5 [M] | Header check before opening (P-8); non-store files in E-ST04-2 |
| F-6 [M] | `--store` anywhere, location resolved once (P-5, P-8) |
| F-7 [M] | `enable`/`disable` only with a toggleable module (P-9) |
| F-8 [M] | Whole build validated, reserved ids refused (P-4) |
| F-9 [M] | `Env`, `Called`, `actions()` from what the user sees, per-surface split (P-14, P-15, T-8) |
| F-10 [M] | Byte copies and `ARBITRE_TEST_NOW` (P-12, P-15) |
| F-11 [m] | Observer API named (P-13); CONTRIBUTING rule (T-8) |
| F-12 [m] | Measurement moved to T-7; kill on timeout (P-16) |
| F-13 [m] | `TZ` case in T-6 |
| F-14 [m] | `codes.rs` row in T-5; `test.read` and help-order list fixed in T-1; separate files for the two dispatch arms (§ Zones, § Execution) |
| F-15 [m] | US-5 row 4 helper test (T-7); `enabled_from` rules (P-5); `jiff` frozen (§ `jiff` check, T-8) |

## Plan questions

**Q-DA1 — settled (a) by the plan, not put to Quentin.** It only moves test-only code (feature `test-actions`, never shipped) and makes Q2 (a) hold in every build; it overturns no arbitration, since the test-build exception was a default of this cycle. Quentin may still reverse it: (b) keeps `test` toggleable with a written exception in the frozen conformance test, which the DA and this plan advise against.

No other question: every decision above stays inside the specs as arbitrated.
