# Implementation Plan — ST-04: Modules you can enable, kept apart, with one generic conformance test

## Traceability
- Specs: `.ctdd/stories/ST-04/02_specs.md` v1, with § Resolution (Q1 to Q7 answered (a), 2026-11-05), binding.
- Story: `.ctdd/stories/ST-04/01_story.md`.
- Frame: `.ctdd/cadre/04_plan.md` § ST-04 (Supporting / Std, size M, arbitration "moyenne"), § Collisions, F-12, F-18, F-27, F-37.
- ST-03: `04_plan.md` P-3, P-4, P-11, "ST-04 handover"; `zones.toml`.
- Date: 2026-11-05 · Status: v1.

## Technical context
- **Codebase** (`st-03/story`, read 2026-11-05):
  - `crates/core` (`arbitre-core`): `#![no_std]` + `alloc`, no normal dependency. `Clock { now }`, `SimClock` (feature `sim`), `Store` (`async fn`, `&self`, no `Send`), `SimStore` (feature `sim`), `RecordId { module, category, id }`, `DeviceId([u8; 16])`, `Author`, `catalog.rs` (`Catalog::build(&[(&str, Vec<ActionDescriptor>)])`, refuses a module id given twice), `error.rs`, `value.rs`.
  - `crates/cli` (`arbitre`): `modules.rs` (`Wiring`, `Context`, `all()`, `catalog()`), `store.rs` (`location`, `open` = `create_dir_all` + `SqliteStore::open`, `map_store`, `map_apply`, `block_on` = one poll on `Waker::noop`), `cli/mod.rs` (runtime clap tree), `main.rs` (JSON pre-scan, `catch_unwind`, one checked write), `wiring/test.rs` (module `test` + `arbitre.test-foundation` under id `arbitre`). Tests: `tests/{usage,output,codes,location,docs}.rs`, `tests/support/mod.rs` (`tmp`, `cmd`, `arbitre`, `doc`, `ready`, `populate`, `snapshot`).
  - `crates/sqlite`: `SqliteStore::open(&Path, &Options)` creates the store when absent, sets WAL.
  - CI: `10` tests (`--all-features --locked`), `20` wasm32 check of `arbitre-core`, `30` crate graph, `40` gitleaks, `50` `cargo mutants --in-diff --all-features` (Gen excluded). Judge reds a lowered zone.
- **Stack**: Rust 1.99, edition 2024; `clap` 4.6.7 builder; `jiff` 0.2.39 (shared, `default-features = false`); `serde_json` dev only.
- **Patterns**: functional core / imperative shell; logic pushed into core where mutation is cheap (ST-03 R-2, Q-P2); red tests laid by the Skeleton; zones per story.

## `jiff` check (done 2026-11-05)

Scratch crate outside the workspace (`$SCRATCH/jiffcheck`, `#![no_std]` + `extern crate alloc`), dependency `jiff = "=0.2.39"`, `default-features = false`, features `["alloc", "tzdb-bundle-always"]`:
- `cargo build --target wasm32-unknown-unknown`: **green**.
- `cargo tree -e features`: only `alloc` and `tzdb-bundle-always` on `jiff` (plus `jiff-core/alloc`, `jiff-tzdb`); **no `std`** is pulled in.
- Native test: `TimeZone::get("Europe/Paris")` at `2026-03-29T01:30:00Z` → +02:00, at `00:30Z` → +01:00. The bundled database resolves a named zone with no system database.
- Dependency added to core: `jiff`, `jiff-core`, `jiff-tzdb` (all no-std). Size measured by S1-A: about +120 to +135 KB gzip in the PWA.

Result: **no fallback needed.** Limit of the check: `wasm32-unknown-unknown` has a `std`, so the build proves the feature set, not a target without `std`; the feature tree shows `std` off, which is what `no_std` needs. If a later `jiff` release breaks this, the fallback is a core type `TimeZoneId(String)` (IANA name) in `Clock`, with resolution done in `arbitre-plan` (also wasm). Not built now.

## Decisions taken by this plan (within the specs' latitude)

| # | Decision | Why |
|---|---|---|
| P-1 | `pub type LocalBoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>` in `module.rs`; `Module<S>` and `Ai` return it | Q3 (a); `alloc` only |
| P-2 | `ModuleStore<'a, S> { store: &'a S, module: &'static str, author: Author, clock: &'a dyn Clock }`; reads take `(category, id)`; `apply(&[(category, id, Change)])` stamps with `clock.now()` and `author` | RG-ST04-3.3 by construction: no method names a module |
| P-3 | `Context<'a, S> { author, clock: &'a dyn Clock, store: ModuleStore<'a, S>, ai: Option<&'a dyn Ai> }`; built by the CLI per action | RG-ST04-3.2 |
| P-4 | `Catalog::build(foundation: &[ActionDescriptor], modules: &[(&str, Vec<ActionDescriptor>)])`; a listed module with id `arbitre` → `CatalogError` | RG-ST04-3.4; handover |
| P-5 | Pure helpers in core (`module.rs`): `pre_scan(args: &[&str]) -> PreScan { json, store: Option<&str>, first_word: Option<&str> }`; `enabled_from(record_state, toggleable, default) -> BTreeSet<&str>`; `state_change(module, on) -> (category, id, Change)` | Logic where mutation is cheap; `main.rs` only wires |
| P-6 | Enabled-state record `arbitre/module-state/<device hex>`, field = module id, value `b"1"`/`b"0"`; a missing field = default | Q1 (a); hex of the 16 device bytes, lowercase |
| P-7 | Signals: pure parts in core `signal.rs` (record id `<device hex>-<ms>-<source>`, fields `at`, `source`, `message`, `read`; `unread(records, device)` sorted by `(at, id)`; `line(at, tz, message)`); `crates/cli/src/signals.rs` does the I/O | Same as P-5 |
| P-8 | Before parsing: resolve the location from `pre_scan`; if the file exists **and is not empty**, open it (one handle, kept for the action); otherwise default state. After parsing, an action reuses that handle or opens (creates) one | RG-ST04-2.3. An empty file would be initialised by SQLite on open, which is a write |
| P-9 | `modules.rs`: `all() -> Vec<Box<dyn Module<SqliteStore>>>` (one line per module) and `DEFAULT_ENABLED: &[&str]` (`"test"` under `cfg(feature = "test-actions")`); the foundation is built in `main.rs` from `wiring/arbitre.rs` | RG-ST04-1.6, GAP-ST04-3; AC-3.6 |
| P-10 | `block_on` (one poll) unchanged; a `Pending` stays `internal_error` | No real async I/O in ST-04 |
| P-11 | Native clock in `crates/cli/src/clock.rs`: `SystemClock` (`now` from `SystemTime`, `timezone` = `jiff::tz::TimeZone::try_system()` else UTC). `arbitre` adds `jiff` features `std`, `tz-system` | RG-ST04-8.1 |
| P-12 | Observer = `tests/support/network.rs`: `std::net::TcpListener` on `127.0.0.1:0`, one thread per connection, reads the request line (`CONNECT host:port` or absolute `http://host[:port]/…`), records `(host, port)` with the argv the spawn helper registered, cut mode closes, open mode forwards (`TcpStream` + two `io::copy` threads). The spawn helper sets `HTTP_PROXY`, `HTTPS_PROXY`, `ALL_PROXY` (both cases), clears `NO_PROXY`/`no_proxy` | Q5 (a); no dependency |
| P-13 | Surfaces: `tests/support/surfaces/mod.rs` with `trait Surface { fn name(&self) -> &str; fn actions(&self, env: &Env) -> BTreeSet<String>; fn call(&self, env: &Env, action: &str, input: &serde_json::Value) -> Called }` and `pub fn all() -> Vec<Box<dyn Surface>>` (one line per surface); `cli.rs` turns input into options by the RG-ST03-4.3 table | RG-ST04-5.4 |
| P-14 | Conformance discovery through `arbitre module list --json` (RG-ST04-5.1); isolation compared with a test-side `SqliteStore` (`arbitre-sqlite` is a normal dependency of `arbitre`, usable by its integration tests) | No module list in tests |
| P-15 | Every spawned command in the conformance and observer tests has a 10 s timeout (`Child::wait_timeout` emulated with `try_wait` + 50 ms sleep); timeout = failure "hang" | RG-ST04-4.2, 4.3 |

## Pre-plan analysis

### Over-engineering detected
- **Open-mode forwarding in the proxy**: no ST-04 action connects. Kept (≈ 15 lines) because PROP-2 for ST-10, 19 and 30 needs it and the tooling is frozen after ST-04.
- **Subset enumeration** (RG-ST04-1.4): 3 toggleable modules in the test build ⇒ 8 subsets × every action. Kept; bounded by the pairs rule above 6 modules.
- **Not added**: an `ai` fake action; a positional argument (Q7); lazy store opening (AMB-ST04-1); `tokio`, `reqwest`; a macro for module registration; a completion generator.

### Under-engineering detected
- **Empty store file** (P-8): `SqliteStore::open` on a 0-byte file creates the schema. Treated as absent before parsing.
- **`--store` before parsing**: `pre_scan` must read `--store <p>` and `--store=<p>`, skip option values, stop at `--`, and ignore non-UTF-8 arguments (they become usage errors later in clap). Non-UTF-8 first word ⇒ no RG-ST04-2.2 decision, clap reports usage.
- **The `module` enum is derived from `all()`**: the foundation's descriptors need the toggleable ids, so `wiring/arbitre.rs` takes them as a parameter.
- **ST-03 tests rely on `test` enabled with no store**: P-9's default list keeps them green.
- **Foundation actions must not be filtered**: the catalogue is foundation + enabled modules, always.
- **Signals are shown after the store is opened for an action, before `execute`**; marking read is a separate local apply with `Author::System`. A failure to show or mark signals never changes the action's outcome: it writes one stderr line and goes on.

### Identified risks

| # | Risk | Probability | Impact | Mitigation |
|---|---|---|---|---|
| R-1 | ST-03 (PR #8) changes in review before merging | Medium | Rebase work | ST-04 code starts after PR #8 merges; `st-04/*` rebased on `main` then |
| R-2 | Mutation time of Std CLI files grows (more binary tests per mutant) | Medium | `50-st01-mutation.sh` long | Pure parts in core (P-5, P-7); the conformance test lives in its own `[[test]]` target; measured at end of T-5, same 30 min rule as ST-03 Q-P2 |
| R-3 | `dyn Module<S>` with a boxed future borrowing `&Context` hits lifetime errors | Medium | T-2 slips | Skeleton (T-1) writes and compiles the trait and one dummy impl first |
| R-4 | A proxy-env-based observer misses a client that ignores proxies | Low now | PROP-2 hole later | `CONTRIBUTING.md` rule (T-8); each adapter story asserts its traffic **is** recorded (specs RG-ST04-4.1) |
| R-5 | Conformance test is slow (every action × subsets × modes, each a process spawn) | Medium | CI time | Measured in T-7; parallel test threads; ceiling 3 min, else ask |
| R-6 | `jiff` features unify `std` into core on native builds | Low | None functionally | wasm check (`20`) builds `arbitre-core` alone, so the no-std set is what it checks |

## Tasks

**Convention.** The Skeleton (phase 6, T-1) lays down every type and signature and **all red tests**. Each later task makes its tests green in its own files. "Green" also means `cargo test --workspace --all-features --locked`, `20-st01-browser.sh`, `30-st01-graph.sh` pass.

### T-1: Skeleton — types, signatures, red tests, zones
- **DDD zone**: Generic
- **Spec refs**: architecture notes; RG-ST04-G.2; every E-ST04-n
- **Dependencies**: PR #8 merged; branch rebased on `main`
- **Description**:
  - Core: `module.rs` (`LocalBoxFuture`, `Module<S>`, `Context`, `ModuleStore`, `Problem`, `PreScan`, `pre_scan`, `enabled_from`, `state_change`), `ai.rs` (`Ai`, `AiError`, mapping to ids), `signal.rs` (P-7), `Clock::timezone` + `SimClock::set_timezone`, `Catalog::build` new signature; `lib.rs` lines; `Cargo.toml`: `jiff = { workspace = true, features = ["alloc", "tzdb-bundle-always"] }`. Bodies `todo!()`.
  - CLI: `modules.rs` (P-9), `wiring/arbitre.rs`, `wiring/fake.rs`, `signals.rs`, `clock.rs` signatures; `test.rs` ported to `Module` (bodies kept), `test.not-enabled` removed, `test.signal` declared; `Cargo.toml`: `jiff` features `std`, `tz-system`; `[[test]]` targets `modules`, `signals`, `network`, `conformance` with `required-features = ["test-actions"]`.
  - Red tests: core `tests/module.rs` (E-ST04-9, pre_scan cases, `enabled_from`, P-4 refusal), `tests/clock.rs` additions (E-ST04-8), `tests/signal.rs` (P-7 pure parts); CLI `tests/modules.rs` (E-ST04-1, 2, 3, 7), `tests/signals.rs` (E-ST04-6), `tests/network.rs` (E-ST04-4), `tests/conformance.rs` (E-ST04-5), `tests/support/{network.rs, conformance.rs, surfaces/{mod.rs, cli.rs}}` with signatures; `codes.rs` row "module not enabled" moved to a disabled `fake-a` (E-ST04-10).
  - `.ctdd/stories/ST-04/zones.toml` (below).
- **Deliverable**: files above, compiling, new tests red, ST-03 tests still green where they do not touch the new paths.
- **Completion criterion**: `cargo check --workspace --all-features --locked` green; `cargo tree -p arbitre-core -e normal` shows only `jiff`; `20`, `30` green; judge green.
- **Effort**: agent 45 min · arbitration 0

### T-2: Core module contract and store view
- **DDD zone**: Supporting (Std) — `module.rs`, `ai.rs`, `catalog.rs`
- **Spec refs**: RG-ST04-2.2 (pre-scan), 3.1 to 3.6, 1.5, 1.6; INV-ST04-3, 4
- **Dependencies**: T-1
- **Description**: `ModuleStore` operations over any `S: Store` (P-2); `pre_scan` (P-5, under-engineering bullet); `enabled_from`, `state_change` (P-6); `Catalog::build` with the foundation apart (P-4); `AiError` → ids.
- **Deliverable**: the three core files.
- **Completion criterion**: `core/tests/module.rs` green on `SimStore`: every view operation targets `module = m`; writes of `fake-a` invisible to `fake-b`'s view; `apply` stamps with the clock and author given; pre-scan table (options before the word, `--store=x`, `--store x`, `--`, no word); catalogue refuses a listed `arbitre`; foundation actions present whatever the enabled set.
- **Effort**: agent 35 min · arbitration 0

### T-3: Time zone
- **DDD zone**: Supporting (Std) — `clock.rs` (core), `crates/cli/src/clock.rs`
- **Spec refs**: RG-ST04-8.1, 8.2; INV-ST04-6
- **Dependencies**: T-1
- **Description**: `Clock::timezone`; `SimClock` default UTC and setter; `SystemClock` (P-11).
- **Deliverable**: both clock files.
- **Completion criterion**: E-ST04-8 green; `20-st01-browser.sh` green.
- **Effort**: agent 10 min · arbitration 0

### T-4: Foundation module, enabled state, `main` pipeline
- **DDD zone**: Supporting (Std) — `wiring/arbitre.rs`, `modules.rs`, `main.rs`, `store.rs`
- **Spec refs**: RG-ST04-1.1 to 1.7, 2.1 to 2.4, 3.4, 3.6; CONFLICT-ST04-1, 2; GAP-ST04-1, 2, 3; INV-ST04-1, 2, 5
- **Dependencies**: T-2, T-3
- **Description**:
  - `wiring/arbitre.rs`: `module.enable` / `.disable` / `.list` (output of RG-ST04-1.1; `problems` from T-5's hook, empty until then), `test-foundation` under `test-actions`.
  - `store.rs`: `open_existing(&Path) -> Result<Option<SqliteStore>, ActionError>` (P-8).
  - `main.rs`: pre-scan → location → `open_existing` → enabled set → disabled first word ⇒ exit 4 document → catalogue (foundation + enabled) → clap → action: reuse or open the store → signals (T-6 hook point, no-op until then) → `execute` through `block_on` → ST-03 tail unchanged. Help on an unreadable store ⇒ 3; `--version` ⇒ 0 (CONFLICT-ST04-2).
  - `test.rs` and `modules.rs` on `Box<dyn Module<SqliteStore>>`.
- **Deliverable**: files above.
- **Completion criterion**: `cli/tests/modules.rs` E-ST04-1 and E-ST04-2 green, with the store absent or byte-identical as specified; all ST-03 tests green (E-ST04-10).
- **Effort**: agent 45 min · arbitration 0

### T-5: Fake module and revalidation on enable
- **DDD zone**: Generic (`wiring/fake.rs`) · Supporting (Std, `wiring/arbitre.rs`)
- **Spec refs**: RG-ST04-5.5, 7.1, 7.2; US-3 rows 1–2; US-7 rows 1–3
- **Dependencies**: T-4
- **Description**: `Fake { id }` with `put`, `get`, `list`, `update`, `delete` (`logged_as = Delete`), `push` (`{Relay}`, `relay_not_configured`), `revalidate` (`invalid_value`); `fake-a`, `fake-b` in `all()` under `test-actions`; `module.enable` calls `revalidate` on all records of a module that was disabled.
- **Deliverable**: `wiring/fake.rs`, the `enable` change.
- **Completion criterion**: `modules.rs` E-ST04-3 rows 1–2 and E-ST04-7 green. **Mutation measurement** (R-2): `cargo mutants --all-features -f crates/cli/src/main.rs -f crates/cli/src/wiring/arbitre.rs`, per-mutant time × count; above 30 min, stop and ask.
- **Effort**: agent 25 min · arbitration 0 (unless the measurement escalates)

### T-6: Signals
- **DDD zone**: Supporting (Std) — `signal.rs` (core), `crates/cli/src/signals.rs`, the hook point in `main.rs`
- **Spec refs**: RG-ST04-6.1 to 6.3; CONFLICT-ST04-5
- **Dependencies**: T-3, T-4
- **Description**: pure parts (P-7); `deposit`, `show_and_mark`; `test.signal` in `test.rs`; `main.rs` calls `show_and_mark` for a well-formed action only.
- **Deliverable**: files above.
- **Completion criterion**: `core/tests/signal.rs` and `cli/tests/signals.rs` (E-ST04-6, matrix US-6 rows 1–5) green.
- **Effort**: agent 20 min · arbitration 0

### T-7: Observer, surface registry, generic conformance test
- **DDD zone**: tests (`crates/cli/tests/**`, not mutated)
- **Spec refs**: RG-ST04-4.1 to 4.4, 5.1 to 5.6; PROP-ST04-1 to 5; INV-ST04-7
- **Dependencies**: T-5 (and T-6 for a clean stderr check)
- **Description**: `support/network.rs` (P-12, P-15); `support/surfaces/{mod.rs, cli.rs}` (P-13); `support/conformance.rs` (minimal input of RG-ST04-5.3, discovery P-14, the seven checks of RG-ST04-5.2); `tests/network.rs` (self-test, std TCP client); `tests/conformance.rs` (one `#[test]` per check, each looping over every module and surface).
- **Deliverable**: files above.
- **Completion criterion**: E-ST04-4 and E-ST04-5 green; the fake CRUD scenario succeeds in cut mode; `fake-a push` → 3 `relay_not_configured` in < 10 s; total conformance time measured and written in the PR body (R-5).
- **Effort**: agent 50 min · arbitration 0

### T-8: Docs, glossary, close the PR
- **DDD zone**: Supporting (story completion)
- **Spec refs**: RG-ST04-G.2; § Collisions
- **Dependencies**: T-2 to T-7
- **Description**:
  - `README.md`: "Modules" section (enable, disable, list; nothing enabled on a fresh device; the CI flow of D-2 with `--module`).
  - `CONTRIBUTING.md`: glossary rows of the specs; "Network clients" rule (never disable proxy settings; each adapter story asserts its traffic is recorded); "Adding a module" (one line in `modules.rs`, one `wiring/<m>.rs`); "Adding a surface" (one file and one line in `tests/support/surfaces/`); the frozen list of RG-ST04-G.2.
  - Rebase on `main`, push and open the PR on Quentin's go. Body: frozen types and tooling, Q1–Q7, the `jiff` check, mutation and conformance times.
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
Gen = ["crates/cli/src/wiring/fake.rs"]
```

`crates/core/src/clock.rs` is already `Std` in ST-01's `zones.toml`; listing it again lowers nothing. The strictest declaration wins (judge rule), so the CLI files above are `Std` despite ST-01's `Gen = ["crates/cli/src/**"]`. `wiring/test.rs` stays `Gen` through that same ST-01 line.

## Execution order

```
T-1 ─┬─► T-2 ─┐
     └─► T-3 ─┴─► T-4 ─┬─► T-5 ─┬─► T-7 ─► T-8
                       └─► T-6 ─┘
```

| Can run together | Files owned | Collision |
|---|---|---|
| T-2 ∥ T-3 | T-2: `module.rs`, `ai.rs`, `catalog.rs`; T-3: both `clock.rs` | None |
| T-5 ∥ T-6 | T-5: `wiring/fake.rs`, `wiring/arbitre.rs`; T-6: `signal.rs`, `signals.rs`, `test.rs`, one call line in `main.rs` | `main.rs` hook point laid by T-4; T-6 only fills it |

### Estimate

| Lane | Tasks | Agent (min) | Arbitration (min) |
|---|---|---|---|
| Skeleton | T-1 | 45 | 0 |
| Core | T-2, T-3 | 45 | 0 |
| Binary | T-4, T-5, T-6 | 90 | 0 |
| Tooling | T-7 | 50 | 0 |
| Closing | T-8 | 20 | 20 |
| **Total** | | **≈ 250 cumulated, ≈ 210 in duration** | **≈ 20** (+ phases 1–2 and DA) |

Frame: M (100 min) + "moyenne". Machine ≈ 2.5 × M; recorded, not corrected (recalibration measures it).

## C-TDD invariants to check after implementation

| Invariant / Property | Task(s) | Verification |
|---|---|---|
| INV-ST04-1 invisible when disabled | T-4, T-7 | E-ST04-2 rows 1, 8; conformance check 2 on every surface |
| INV-ST04-2 explicit call ⇒ 4, store untouched | T-2, T-4 | E-ST04-2 rows 2–6 |
| INV-ST04-3 view cannot name another namespace | T-2 | E-ST04-9; conformance check 4 |
| INV-ST04-4 `arbitre` refused outside the foundation | T-2 | `core/tests/module.rs` |
| INV-ST04-5 idempotent, foundation on | T-4 | E-ST04-1 rows 2, 6, 7 |
| INV-ST04-6 core wasm32, `jiff` only | T-1, T-3 | `20-st01-browser.sh`; `cargo tree` |
| INV-ST04-7 minimal input conforms | T-7 | unit test in `support/conformance.rs` against `conforms_input` |
| PROP-ST04-1, 2 data kept, subsets | T-5, T-7 | E-ST04-1 rows 3–4; conformance checks 3, 5 |
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
| AC-3.2, 3.3 | RG-ST04-3.3 | T-2 | E-ST04-9, E-ST04-3 |
| AC-3.4 | RG-ST04-3.4 | T-2 | `core/tests/module.rs` |
| AC-3.5 | RG-ST04-3.5 | T-1, T-2 | id mapping unit test |
| AC-3.7, 8.2 | RG-ST04-8.2 | T-1, T-3 | `20-st01-browser.sh` |
| AC-4.1 to 4.5 | RG-ST04-4.1 to 4.4 | T-7 | E-ST04-4, E-ST04-5 |
| AC-5.1 to 5.5 | RG-ST04-5.1 to 5.6 | T-5, T-7, T-8 | E-ST04-5; CONTRIBUTING |
| AC-6.1 to 6.4 | RG-ST04-6.1 to 6.3 | T-6 | E-ST04-6 |
| AC-7.1 to 7.3 | RG-ST04-7.1, 7.2 | T-5 | E-ST04-7 |
| AC-8.1 | RG-ST04-8.1 | T-3 | E-ST04-8 |
| AC-3.8 | RG-ST04-G.2 | T-8 | PR body, CONTRIBUTING frozen list |

## Plan questions

None for Quentin: every decision above stays inside the specs as arbitrated.
