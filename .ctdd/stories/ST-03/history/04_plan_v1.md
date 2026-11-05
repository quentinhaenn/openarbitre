# Implementation Plan — ST-03: Action catalogue, derived CLI, exit codes

## Traceability
- Specs: `.ctdd/stories/ST-03/02_specs.md` (phase 3 skipped: its "Architecture notes" replace the archi document).
- Story: `.ctdd/stories/ST-03/01_story.md`. Its "Quentin's arbitrations (2026-11-04)" are binding: Q-1 (a), Q-2 (a), Q-3 (a), Q-4 (a), Q-5 (a), Q-6 (b), Q-7 (a), Q-8 (a), Q-9 (a); MCP name rule `.` → `_` (spike S5).
- **Q-B (specs) taken on the recommendation (a), revisable**: a command path may be both an action and a group (`plan` and `plan add`). Only T-3 (one catalogue check) and T-5/T-6 (`test.group` + `test.group.child`, one clap setting) depend on it; switching to (b) removes code, it adds none.
- Frame: `.ctdd/cadre/04_plan.md` § ST-03 (Supporting / Std, size M, arbitration "forte"), § ST-04, § Collisions, F-6, F-38.
- Codebase planned against: `origin/st-02/store` (ST-02, PR #6, open). Repo rules: `CONTRIBUTING.md` (on that branch: Zones, Equivalent mutants), `scripts/ci.d/*.sh`, `scripts/lib/{graph,judge,mutation}.py`.
- Format: `.ctdd/stories/ST-02/04_plan.md`.
- Date: 2026-11-04 · Status: DRAFT v1 · DA: one iteration max (`state.json`).

## Technical context
- **Codebase** (`origin/st-02/store`, read 2026-11-04):
  - `crates/core` (`arbitre-core`): `#![no_std]` + `alloc`, no normal dependency; modules `clock`, `hlc`, `merge`, `store`, `write`, `sim_store` (feature `sim`), all re-exported with `pub use *`. `Store` trait (`apply_local(pt, &[LocalChange])`, `writes_since`, `device`, …), `StoreError { StorageFull, Corrupted(Damage), Unavailable(Cause { Busy, Io, IncompatibleVersion, RandomSource }) }`, `ApplyError { Rejected(Rejection), Store(StoreError) }`, `Author::CliUser`.
  - `crates/sqlite` (`arbitre-sqlite`): `SqliteStore::open(&Path, &Options) -> Result<Self, StoreError>` (creates the store if absent; parent directories: see R-4), futures ready on first poll (S1-A).
  - `crates/cli` (`arbitre`): `clap` 4.6.7 (`derive`), a 3-line `main.rs`; zone Gen through ST-01's `crates/cli/src/**`.
  - CI: `10` (`cargo test --workspace --all-features --locked` + `test_ci.py`), `20` (wasm32 check of `arbitre-core`), `30` (graph: `arbitre → arbitre-sqlite` allowed; shared deps inherited with `workspace = true`, dev-deps included), `40` (gitleaks), `50` (`cargo mutants --in-diff --all-features`, Gen files excluded). The judge reds a lowered zone or a `scripts/`/`.github/` change without `ci-change`.
- **Stack**: Rust 1.99, edition 2024; `clap` 4.6.7 (builder API); `serde_json` and `proptest` from `[workspace.dependencies]`, **dev only**.
- **Patterns to follow**: functional core / imperative shell (everything decidable without I/O in `arbitre-core`, the binary only does argv, env, store, stdout, exit); validation before effect; zones per story, strictest wins; `required-features` on test targets that need a feature; red tests laid down by the Skeleton.

## Decisions taken by this plan (within the specs' latitude)

| # | Decision | Why |
|---|---|---|
| P-1 | Core split in **three files**: `value.rs` (`Value`, canonical encoder, human rendering), `error.rs` (`Category`, `ActionError`, result document), `catalog.rs` (descriptor, schema subset, name rule, catalogue build, conformance, per-type text parsers, JSON Schema form). All `Std` | Specs allow the split; three files of ~150–300 lines give the parallel tasks disjoint files (T-2 ∥ T-3) |
| P-2 | **No JSON parser in production code**, anywhere. Core has the encoder only; the CLI never reads JSON; ST-07 converts `rmcp`'s `serde_json::Value` into `Value` at its own edge, in `arbitre`. `serde_json` is a dev-dependency of `arbitre-core` and `arbitre` (independent parser in tests) | Keeps core minimal (Q-P1); the only consumer of incoming JSON is MCP, which already links `serde_json` through `rmcp` |
| P-3 | Dates checked by a hand-written `YYYY-MM-DD` parser in `catalog.rs` (Gregorian leap rule, years 0001–9999), no `jiff` | AC-4.4: no new core dependency. ST-04 brings `jiff` into core (`Clock::fuseau`) and may replace it then |
| P-4 | Execution seam, provisional until ST-04, in `crates/cli/src/modules.rs`: `struct Wiring { id: &'static str, descriptors: fn() -> Vec<ActionDescriptor>, execute: fn(&str, Value, &Context) -> Result<Value, ActionError> }`, `struct Context<'a> { store: &'a SqliteStore, pt: u64, author: Author }`, and `fn all() -> Vec<Wiring>` | The specs' seam, nothing more; ST-04 replaces it by `Module`/`Context` in core |
| P-5 | **Adding a module = one entry in `modules.rs`**: the `#[path = "wiring/<m>.rs"] mod <m>;` line and its line in `all()`, adjacent; no `wiring/mod.rs` | No other shared file is touched (AC-4.5). A macro to fold both lines into one is not worth it; the test module needs a `#[cfg(feature = "test-actions")]` on both lines anyway |
| P-6 | Store location, opening, `StoreError`/`ApplyError` → `ActionError` mapping and the single-poll executor live in `crates/cli/src/store.rs`, outside `cli/` | Shared by every surface of the binary (ST-07's `mcp/` reuses it unchanged) |
| P-7 | The command-line surface is **one file**, `crates/cli/src/cli/mod.rs` (derivation, matches → input `Value`, clap error → usage id, help/version). Human rendering is in core (`value.rs`), not here | "One directory per surface" (frame); one file until it needs two |
| P-8 | Result document built in core: `fn document(&Result<Value, ActionError>) -> Value` in `error.rs`, encoded with the core encoder | Frozen shape (RG-ST03-2.1) tested in the pure crate, where mutation is cheap |
| P-9 | `test-actions` is a feature of `arbitre`; the test module's `[[test]]` targets declare `required-features = ["test-actions"]`; CI's `--all-features` builds it, a release build does not | Same pattern as `sim` in ST-01/ST-02 |
| P-10 | `clap`'s `derive` feature is dropped (builder only) | Unused after ST-03 |

## Pre-plan analysis

### Over-engineering detected
- **Constraint keywords in the frozen subset** (`minimum`, `maximum`, `minLength`, `maxLength`, GAP-ST03-2): four keywords that no ST-03 action needs for itself. Kept: without them Q-3 (a)'s "range" and "length limit" would be refusals, against the arbitration. Each is one comparison in the conformance function. Needs Quentin's confirmation because it freezes (Q-P3).
- **`--json` help/version documents** (`{"help": …}`): a few lines; kept because "one document for every outcome" (Q-4 (a)) is simpler for scripts than an exception.
- **Test actions** (13): each exercises one rule; all in one Gen file, not shipped. `test.noisy` and `test.read` are kept (E-ST03-2 diagnostics, US-5 row 6). Nothing to cut.
- **Not added**: a JSON parser in core (P-2), `jiff` (P-3), a `Module` trait (ST-04), a macro for module registration (P-5), short options, shell completion.

### Under-engineering detected
- **JSON mode must be known before clap runs**: a parse failure or an invalid catalogue still needs a JSON document. `main.rs` scans the raw `OsString` args for a literal `--json` before the first `--` (RG-ST03-2.1), and clap still declares `--json` as a global flag so it is accepted at every level.
- **The catalogue is built before parsing**, `--help` included, so an invalid catalogue is `catalog_invalid` (exit 3) everywhere (RG-ST03-4.6). Cost: a few µs per run.
- **`catch_unwind` needs everything in one closure** (`AssertUnwindSafe`), the document built in memory and written once (RG-ST03-3.5). The default panic hook already writes to stderr; no custom hook.
- **clap prints and exits on its own** if `get_matches` is used: only `try_get_matches_from(args_os)` is allowed; `DisplayHelp`/`DisplayVersion`/`DisplayHelpOnMissingArgumentOrSubcommand` are mapped explicitly (the last one is usage, exit 2, RG-ST03-1.1 row 9).
- **Rust ignores `SIGPIPE`**: writing to a closed stdout returns an error instead of killing the process, so `output_failed` (exit 3) is reachable as specified (CONFLICT-ST03-3).
- **Default-location tests depend on the environment**: each test clears `XDG_DATA_HOME`, `ARBITRE_STORE`, and sets `HOME` to a temporary directory; the macOS branch is tested only on macOS (`cfg(target_os)`), CI runs the Linux branch.

### Identified risks

| # | Risk | Probability | Impact | Mitigation |
|---|---|---|---|---|
| R-1 | ST-02 (PR #6) changes before merging (Q-P1…Q-P6 still revisable) | Medium | Store mapping (T-4) breaks | ST-03 code starts only after PR #6 merges; `st-03/*` is rebased on `main` then. ST-03 uses only `SqliteStore::open`, `apply_local`, `writes_since`, `device` and the error enums |
| R-2 | Mutation of `Std` files in `crates/cli` is slow: each mutant rebuilds `arbitre` and re-runs binary-spawning tests (E-ST03-1/2: 200 cases each) | Medium | `50-st01-mutation.sh` takes too long | Logic is pushed into core (P-1, P-8) so the CLI files stay small; measured on the PR's CI run (T-8). Escalation rule in Q-P2 |
| R-3 | clap's error kinds do not map one-to-one to the specs' usage ids, or a clap message does not contain the usage | Low | Ids differ | Ids are not frozen except those of RG-ST03-3.3 (none of them is a usage id); map by `ErrorKind`, `usage` as fallback |
| R-4 | `SqliteStore::open` does not create missing parent directories | Medium | US-5 row 1 (default location under a fresh `HOME`) fails | `store.rs` calls `std::fs::create_dir_all(parent)` before `open`; failure → `storage_unavailable` (RG-ST03-3.3) |
| R-5 | A value-parser closure that fails gives a clap `ValueValidation` error whose text hides which rule failed | Low | Poor message | The closure returns core's error message; clap shows it with the usage |
| R-6 | Q-B or Q-P3 answered differently after the Skeleton | Low | Rework | Both touch one catalogue check and one or two test actions; marked revisable |

## Tasks

**Convention (who writes what).** The Skeleton (phase 6, T-1) lays down every type and signature below and **all red tests**: core unit tests (E-ST03-7, 8, 9 and the rendering property) and the acceptance tests in `crates/cli/tests/` (E-ST03-1 to 6, 10). Implementation tasks then only make tests green, each in its own files. "Green" for every task also means `cargo test --workspace --all-features --locked`, `20-st01-browser.sh` and `30-st01-graph.sh` pass.

### T-1: Skeleton — scaffold, types, red tests, zones
- **DDD zone**: Generic
- **Spec refs**: architecture notes; AC-4.4; RG-ST03-G.1, G.2; all E-ST03-n
- **Dependencies**: PR #6 merged; branch rebased on `main`
- **Description**:
  - `crates/core/src/{value,error,catalog}.rs` declared in `lib.rs` and re-exported (`pub use`), with the types of the glossary: `Value { Bool, Int(i64), Str(String), List(Vec<Value>), Object(BTreeMap<String, Value>) }`; `Category` (`exit_code`, `json_name`); `ActionError { category, id, message }`; `ActionDescriptor` (fields of RG-ST03-4.1); `LoggedAs`; `Network` + `BTreeSet<Network>`; `Schema`/`Property`; `Catalog`, `CatalogError`. Bodies `todo!()`.
  - `crates/cli`: `main.rs`, `modules.rs`, `store.rs`, `cli/mod.rs`, `wiring/test.rs` with signatures (P-4); `Cargo.toml`: `arbitre-sqlite` (path), `clap` without `derive`, feature `test-actions`, dev-deps `serde_json`, `proptest` (`workspace = true`), `[[test]]` targets with `required-features = ["test-actions"]`. `crates/core/Cargo.toml`: dev-dep `serde_json` (`workspace = true`). `Cargo.lock` regenerated.
  - Red tests: `crates/core/tests/catalog.rs` (E-ST03-7, 9; conformance and text-parser cases of matrix US-1 rows 3–7), `crates/core/tests/value.rs` (E-ST03-8; leaf-in-render property, RG-ST03-2.3); `crates/cli/tests/{usage,output,codes,location,docs}.rs` (E-ST03-1, 2 + 10, 3 + 4, 5, 6), sharing a `tests/support.rs` included with `#[path]` (spawn helper on `CARGO_BIN_EXE_arbitre`, temporary `HOME`/store).
  - `.ctdd/stories/ST-03/zones.toml` (written in phase 4, kept).
- **Deliverable**: files above, compiling, tests red.
- **Completion criterion**: `cargo check --workspace --all-features --locked` green; every new test compiles and fails; `20-st01-browser.sh`, `30-st01-graph.sh` green; `cargo tree -p arbitre-core -e normal` shows no dependency (INV-ST03-9); judge locally green without label (no zone lowered).
- **Effort**: agent 35 min · arbitration 0

### T-2: `Value`, canonical JSON, human rendering, categories, result document
- **DDD zone**: Supporting (Std) — `value.rs`, `error.rs`
- **Spec refs**: RG-ST03-2.1 (shape), 2.3, 2.4, 3.1, 3.4; INV-ST03-2, 3, 4 (shape), 5
- **Dependencies**: T-1
- **Description**:
  - `encode(&Value) -> String`: sorted keys (`BTreeMap` order = UTF-8 byte order), no whitespace, shortest decimal `i64`, escaping table of RG-ST03-2.4 (lowercase `\u00xx`).
  - `render(&Value) -> String`: deterministic walk in key order, one `key: value` line per leaf (nested keys indented or dotted, layout free), strings verbatim; empty object → empty text.
  - `Category::exit_code` (0–4), `json_name`; `ActionError` constructors `refusal`, `tool_error`, `module_not_enabled` (the last used by `test.not-enabled` until ST-04); `Success` is unrepresentable in an error, and `Usage` has its own constructor used only by the CLI (RG-ST03-3.4).
  - `document(&Result<Value, ActionError>) -> Value` (P-8).
- **Deliverable**: `crates/core/src/value.rs`, `crates/core/src/error.rs`.
- **Completion criterion**: `core/tests/value.rs` green: `serde_json` reads the bytes back as the same value (1,000 random `Value`s, strings drawn with controls, quotes, `\`, non-Latin), re-encoding byte-identical, golden bytes for each escape; every leaf of a random value appears in `render` on a line with its key; `exit_code` table and injectivity; `document` golden bytes for success and each failure category.
- **Effort**: agent 20 min · arbitration 0

### T-3: Descriptor, schema subset, catalogue build, conformance
- **DDD zone**: Supporting (Std) — `catalog.rs`
- **Spec refs**: RG-ST03-1.2, 4.1 to 4.6; Q-B (a); GAP-ST03-1, 2; INV-ST03-7, 8
- **Dependencies**: T-1 (types); uses T-2's `encode` only in tests of the JSON Schema form (red until T-2 lands)
- **Description**:
  - Name rule and command path (RG-ST03-4.2), MCP name (`.` → `_`), length ≤ 64.
  - `Catalog::build(&[(module_id, Vec<ActionDescriptor>)]) -> Result<Catalog, CatalogError>`: every refusal of RG-ST03-4.6, messages naming the action(s) and module(s). With Q-B (a), a path that is both an action and a prefix of another action is allowed.
  - `parse_text(&Schema, &str) -> Result<Value, String>` per scalar type (decimal `i64`, date P-3, enum exact match, string with length bounds); `conforms_input(&Schema, &Value)` and `conforms_output(&Schema, &Value)` (RG-ST03-4.4, 4.5), first failure reported.
  - `ActionDescriptor::input_json_schema()`/`output_json_schema()` → `Value` in the JSON Schema form of RG-ST03-4.3.
- **Deliverable**: `crates/core/src/catalog.rs`.
- **Completion criterion**: `core/tests/catalog.rs` green: each refusal of RG-ST03-4.6 (one case per bullet, duplicate names both named); Q-B case accepted; date boundaries (`2024-02-29` ok, `2023-02-29`, `2026-02-31`, `0000-01-01`, `2026-1-01` refused); integer and length bounds at `min`, `max`, `max + 1`; golden JSON Schema of `test.echo`'s input and of each subset type (E-ST03-9).
- **Effort**: agent 35 min · arbitration 0

### T-4: Store location, opening, error mapping
- **DDD zone**: Supporting (Std) — `crates/cli/src/store.rs`
- **Spec refs**: RG-ST03-3.3, 5.1 to 5.4; PROP-ST03-2; R-4
- **Dependencies**: T-1
- **Description**:
  - `location(flag: Option<&OsStr>, env) -> Result<PathBuf, ActionError>`: `--store` > non-empty `ARBITRE_STORE` > platform default (RG-ST03-5.1); `no_default_store` otherwise. Env read through a parameter (a closure or map), so unit tests do not mutate the process environment.
  - `open(&Path) -> Result<SqliteStore, ActionError>`: `create_dir_all(parent)` then `SqliteStore::open(path, &Options::default())`.
  - `map_store(StoreError)`, `map_apply(ApplyError)` → ids of RG-ST03-3.3 (`write_rejected` for `Rejected`).
  - `block_on(fut)`: one poll with `Waker::noop()`; `Pending` → `ToolError`/`internal_error`.
- **Deliverable**: `crates/cli/src/store.rs` (+ its unit tests in the same file).
- **Completion criterion**: unit tests: precedence of the three sources, empty `--store` refused upstream (T-6) and empty env ignored, every `StoreError`/`Cause` row of RG-ST03-3.3 mapped to its id; `cli/tests/location.rs` (E-ST03-5) green once T-6 lands.
- **Effort**: agent 15 min · arbitration 0

### T-5: Test module and module list
- **DDD zone**: Generic (`wiring/test.rs`) · Supporting (Std, `modules.rs`)
- **Spec refs**: architecture notes "Test module"; RG-ST03-3.6; AC-4.5; Q-B (a)
- **Dependencies**: T-1
- **Description**:
  - `modules.rs`: `Wiring`, `Context`, `all()` with the `test` entry under `#[cfg(feature = "test-actions")]` (P-4, P-5).
  - `wiring/test.rs`: descriptors and execution of `test.echo` (every input kind; returns the input plus a `remark`), `test.noop`, `test.refuse`, `test.fail`, `test.not-enabled` (returns `ModuleNotEnabled` until ST-04), `test.panic`, `test.bad-output`, `test.noisy` (writes to stderr), `test.write` (mutating; `--key`, optional `--fail refusal|tool-error|panic` acted on **before** `apply_local`), `test.read`, `test.group` + `test.group.child`, and `arbitre.test-foundation`.
- **Deliverable**: `crates/cli/src/modules.rs`, `crates/cli/src/wiring/test.rs`.
- **Completion criterion**: the catalogue with and without `test-actions` builds (E-ST03-7, second half); the test module's descriptors cover every subset type of RG-ST03-4.3 in `test.echo`.
- **Effort**: agent 15 min · arbitration 0

### T-6: Derived command line and the `main` pipeline
- **DDD zone**: Supporting (Std) — `cli/mod.rs`, `main.rs`
- **Spec refs**: RG-ST03-1.1 to 1.4, 2.1, 2.2, 3.2, 3.5, 3.6, 4.7, 5.2; INV-ST03-1, 3, 4, 6, 10; PROP-ST03-1
- **Dependencies**: T-2, T-3, T-4, T-5
- **Description**:
  - `cli/mod.rs`: `command(&Catalog) -> clap::Command` (one sub-command per path segment, sorted; long options only; `SetTrue` booleans, `Append` lists, possible values for enums, `value_parser` closures calling `parse_text`; `subcommand_required` on groups that are not actions; with Q-B (a) `args_conflicts_with_subcommands(true)` on action-groups; global `--json`, `--store`; `propagate_version`, `disable_help_subcommand`); `input(&ArgMatches, &ActionDescriptor) -> Value`; `usage_error(clap::Error) -> Outcome` (help/version → success, other kinds → `Usage` with an id by `ErrorKind`).
  - `main.rs`: raw `args_os` → JSON pre-scan → `catch_unwind` { build catalogue → parse → resolve location → open → run → `conforms_output` → document or rendering } → write once → `std::process::exit(code)`. Human mode: rendering on stdout, `error: <message>` (+ usage) on stderr.
- **Deliverable**: `crates/cli/src/cli/mod.rs`, `crates/cli/src/main.rs`.
- **Completion criterion**: `cli/tests/{usage,output,codes,location}.rs` green (E-ST03-1 to 5, 10), including: malformed command on an absent store path leaves it absent, on a populated store leaves `writes_since(START)` and `device` identical; `--help --json` and `--version` exit 0 with no store; crash ± `--json` → 3 `internal_error`, one document; `test write --fail …` → 1/3/3, store unchanged; `arbitre test group` and `arbitre test group child` both run.
- **Effort**: agent 40 min · arbitration 0

### T-7: Published exit-code table and glossary
- **DDD zone**: Generic
- **Spec refs**: RG-ST03-3.7; AC-3.7; specs glossary
- **Dependencies**: T-1 (T-2 for the test to pass)
- **Description**: `README.md` section "Exit codes": the table of RG-ST03-3.1, the JSON result shape (RG-ST03-2.1), the statement that both never change (foundation amendment), and the store location rule (RG-ST03-5.1). `CONTRIBUTING.md` glossary: the new rows of the story's glossary table.
- **Deliverable**: `README.md`, `CONTRIBUTING.md`.
- **Completion criterion**: `cli/tests/docs.rs` (E-ST03-6) parses the README table and finds it equal to `Category` names and `exit_code()` for all five.
- **Effort**: agent 10 min · arbitration 0

### T-8: Finish and merge the ST-03 PR
- **DDD zone**: Supporting (story completion)
- **Spec refs**: frame § ST-03 completion criteria; RG-ST03-G.2
- **Dependencies**: T-2 to T-7
- **Description**:
  - Branch rebased on `main` (PR #6 merged); `git diff main --name-only` has no `scripts/` or `.github/` path.
  - Push and open the PR on Quentin's go. Body: what is frozen (table, JSON shape, encoding, name form, subset), P-2 (no parser in core), CONFLICT-ST03-3 (`output_failed` exception), Q-B and Q-P1–Q-P4 as answered, measured `50-st01-mutation.sh` time (Q-P2).
  - Each surviving mutant of a Std file: a killing test, or an entry in `.ctdd/stories/ST-03/mutants_equivalents.toml`.
  - **Quentin**: reviews the frozen contract and the equivalents, merges (squash, hand-written message).
- **Deliverable**: PR merged.
- **Completion criterion**: `ci` and `judge` green; `mutation.md`: every ST-03 `Std` file ≥ 70%, no row for `wiring/test.rs`; README table present.
- **Effort**: agent 20 min · arbitration 20 min (frozen contract read, equivalents, merge)

## Execution order

```
T-1 (Skeleton) ─┬─► T-2 ─────────┐
                ├─► T-3 ─────────┤
                ├─► T-4 ─────────┼─► T-6 ─► T-8
                ├─► T-5 ─────────┘     ▲
                └─► T-7 ───────────────┘
```

### Parallelism (separate agents) and collisions

| Can run together | Files owned | Collision |
|---|---|---|
| T-2 ∥ T-3 ∥ T-4 ∥ T-5 ∥ T-7 | T-2: `value.rs`, `error.rs`; T-3: `catalog.rs`; T-4: `cli/src/store.rs`; T-5: `modules.rs`, `wiring/test.rs`; T-7: `README.md`, `CONTRIBUTING.md` | None: types, `lib.rs` re-exports, `Cargo.toml` and all test files are fixed by T-1 |
| T-6 alone | `cli/mod.rs`, `main.rs` | Needs the other four |

With N = 3 concurrent cycles, two agents are enough: one on core (T-2 → T-3), one on the binary (T-4 → T-5 → T-7), then T-6.

### Estimate (agent time and arbitration time)

| Lane | Tasks | Agent (min) | Arbitration (min) |
|---|---|---|---|
| Skeleton | T-1 | 35 | 0 |
| Core | T-2, T-3 | 55 | 0 |
| Binary | T-4, T-5, T-7, T-6 | 80 | 0 |
| Closing | T-8 | 20 | 20 |
| Plan questions | Q-P1 to Q-P4 | — | 10 |
| **Total** | | **≈ 190 cumulated, ≈ 135 in duration** | **≈ 30** |

- Frame: M (≈ 100 min machine) + "forte". Machine ≈ 1.9 × M; most arbitration was already spent in phases 1–2 (Q-1 to Q-9). Recorded by the orchestrator, not corrected.
- Not counted: the DA iteration, CI wall time.

## C-TDD invariants to check after implementation

| Invariant / Property | Task(s) | Verification |
|---|---|---|
| INV-ST03-1 malformed ⇒ 2, store never opened | T-3, T-6 | E-ST03-1 (property, 200, absent and populated store) |
| INV-ST03-2 rendering from `Value` only, every leaf shown | T-2, T-6 | `render(&Value)` signature; leaf property (core); E-ST03-2 |
| INV-ST03-3 five codes, injective, same in both modes | T-2, T-6 | `exit_code` unit tests; E-ST03-3; E-ST03-6 |
| INV-ST03-4 one document per outcome in JSON mode | T-2, T-6 | `document` golden bytes; E-ST03-1 to 3 (stdout parses as exactly one document) |
| INV-ST03-5 canonical encoding | T-2 | E-ST03-8 |
| INV-ST03-6 crash ⇒ 3 `internal_error` | T-6 | E-ST03-3 (`test panic` ± `--json`), E-ST03-4 (`--fail panic`) |
| INV-ST03-7 catalogue refusals | T-3, T-5 | E-ST03-7 |
| INV-ST03-8 conforming input in, output checked | T-3, T-6 | catalog unit tests; `test bad-output` → `output_mismatch` |
| INV-ST03-9 core dependency-free, wasm32 | T-1 | `20-st01-browser.sh`; `cargo tree -p arbitre-core -e normal` |
| INV-ST03-10 help/version 0, no store | T-6 | E-ST03-10 |
| PROP-ST03-1 failure ⇒ store unchanged | T-5, T-6 | E-ST03-4 |
| PROP-ST03-2 location and opening failures | T-4, T-6 | E-ST03-5; mapping unit tests |

## Traceability matrix (AC → rule → task → test)

| AC | Rule | Task | Test |
|---|---|---|---|
| AC-1.1, 1.4 | RG-ST03-1.1 | T-3, T-6 | `cli/tests/usage.rs` (E-ST03-1), matrix US-1 rows 1–9, 15 |
| AC-1.2, 1.3 | RG-ST03-1.3, 5.2 | T-6 | `usage.rs` (absent / populated store) |
| AC-1.5 | RG-ST03-1.4 | T-6 | `output.rs` (E-ST03-10) |
| AC-1.6 | RG-ST03-1.2 | T-3, T-6 | `core/tests/catalog.rs`; `codes.rs` :: `test refuse` → 1 |
| AC-2.1, 2.3, 2.5 | RG-ST03-2.1, 2.2 | T-2, T-6 | `output.rs`, `codes.rs` (one document, `test noisy`) |
| AC-2.2 | RG-ST03-2.3 | T-2, T-6 | `core/tests/value.rs` (leaf property); `output.rs` (E-ST03-2) |
| AC-2.4 | RG-ST03-2.4 | T-2 | `core/tests/value.rs` (E-ST03-8) |
| AC-3.1 to 3.4, 3.8 | RG-ST03-3.1, 3.2 | T-2, T-6 | `codes.rs` (E-ST03-3) |
| AC-3.5 | RG-ST03-3.5 | T-6 | `codes.rs` :: panic ± `--json` |
| AC-3.6, 5.3 | RG-ST03-3.3, 5.3 | T-4, T-6 | `store.rs` unit tests; `location.rs` rows 4, 5; `codes.rs` :: busy, damaged, newer layout |
| AC-3.7 | RG-ST03-3.7 | T-7 | `docs.rs` (E-ST03-6) |
| AC-4.1, 4.6 | RG-ST03-4.1, 4.2, 4.6 | T-3 | `core/tests/catalog.rs` (E-ST03-7) |
| AC-4.2, 4.3 | RG-ST03-4.3 to 4.5, 4.7 | T-3, T-5, T-6 | E-ST03-9; `output.rs` :: every input kind, help lists every option |
| AC-4.4 | INV-ST03-9 | T-1 | `20-st01-browser.sh`; `cargo tree` |
| AC-4.5 | P-5 | T-5 | review in T-8 (adding `test` touched only `modules.rs` + `wiring/test.rs`) |
| AC-5.1, 5.2 | RG-ST03-5.1 | T-4, T-6 | `location.rs` (E-ST03-5) rows 1–3, 6 |
| (global) nothing changes on failure | RG-ST03-3.6 | T-5, T-6 | `codes.rs` (E-ST03-4) |

## Open questions for Quentin

The recommendation applies by default; none blocks the Skeleton except Q-P3 (it changes which cases T-1 writes).

| # | Question | Options | Recommendation |
|---|---|---|---|
| Q-B | **(specs, provisional)** May a command path be both an action and a group? | (a) yes, (b) prefix-free names | **(a)**, applied by this plan, revisable (see Traceability) |
| Q-P1 | **CONFLICT-ST03-1: JSON in core.** The archi expects `serde` in core; AC-4.4 forbids a new core dependency. | (a) Core owns `Value` and a canonical **encoder only** (~60 lines); no JSON parser in production code; JSON *parsing* happens at the edge, in `arbitre`, only when ST-07 receives MCP calls (from `rmcp`'s `serde_json` values); `serde_json` is a dev-dependency for tests. (b) Foundation amendment: `serde` (no_std, `alloc`) in core with `serde_json` (`alloc`) for the encoding. | **(a).** Minimal core, browser check unchanged, and canonical bytes do not depend on `serde_json` features unified by `rmcp` (`preserve_order` would change key order). (b) saves ~60 lines for one amendment and a feature-unification risk. Applied by P-2 |
| Q-P2 | **Zone of the binary's files.** The frame classes ST-03 Std; ST-01 left `crates/cli/src/**` Gen. Mutating CLI files re-runs binary-spawning tests per mutant (R-2). | (a) Std for `main.rs`, `modules.rs`, `store.rs`, `cli/**` (as `zones.toml` says), `wiring/test.rs` Gen; if the CI mutation step of the ST-03 PR exceeds 30 min, stop and ask. (b) Keep all of `crates/cli/src/**` Gen; only the three core files are Std. | **(a).** The derivation and exit mapping *are* the frozen contract; P-1/P-8 keep the CLI files thin, so the mutant count should be small |
| Q-P3 | **GAP-ST03-1/2 extend the frozen schema subset** that Q-7 (a) arbitrated: `object` (top level only in inputs, free nesting in outputs) and `minimum`/`maximum`/`minLength`/`maxLength`. | (a) Accept both. (b) Accept `object` only; ranges and lengths become refusals by the action (contradicts Q-3 (a) "range"). | **(a).** `object` is unavoidable (every input and output is one); the four keywords make Q-3 (a) hold as written |
| Q-P4 | **CONFLICT-ST03-3: one documented exception to "nothing changes on failure".** A mutating action commits, then stdout is closed (`… --json \| head -c1`): exit 3 `output_failed`, but the state has changed. | (a) Accept the exception, documented in the README "Exit codes" section. (b) Exit 0 anyway (the action succeeded; only the report was lost). | **(a).** A script that lost the report must not believe it got one; the README says a `output_failed` mutating command may have been applied. Not a correctness hole of the store (all or nothing still holds) |

Not raised: CONFLICT-ST03-2 (settled by Q-3 (a), binding); AMB-ST03-2 (where ST-04 reads the enabled state) is ST-04's decision and does not touch ST-03 code.
