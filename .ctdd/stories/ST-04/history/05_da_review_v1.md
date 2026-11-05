# Devil's Advocate Review — ST-04: Modules you can enable, kept apart, with one generic conformance test

- Reviewed: `.ctdd/stories/ST-04/04_plan.md` (v1), against `01_story.md`, `02_specs.md` (its § Resolution is binding: Q1 to Q7 answered (a)).
- Frame read: `.ctdd/cadre/04_plan.md` § ST-04 and § Collisions; `.ctdd/cadre/02_specs.md` RG-3.7, RG-5.4, CONFLIT-6, INV-15, INV-23.
- ST-03 read: `05_da_review.md`, `07_review.md` (mutation numbers), `zones.toml`.
- Code read (`st-04/story`): `crates/core/src/{lib.rs, clock.rs, store.rs, catalog.rs, write.rs}`, `crates/cli/src/{main.rs, modules.rs, store.rs, cli/mod.rs, wiring/test.rs}`, `crates/cli/tests/{support/mod.rs, codes.rs, location.rs, output.rs, docs.rs}`, `crates/sqlite/src/lib.rs` (open sequence), `scripts/ci.d/*.sh`, `scripts/lib/graph.py`, `CONTRIBUTING.md` (Zones, Equivalent mutants).
- Iteration 1 of 1 (supporting zone) · Date: 2026-11-05.

## Claims verified by experiment

| Claim | How | Result |
|---|---|---|
| P-1 to P-3 compile: `dyn Module<S>` with boxed local futures borrowing `&'a Context<'a, S>`, a `!Send` store with `async fn`, a `revalidate` borrowing the view, `Box<dyn Module<Mem>>` registry, one-poll executor | Scratch crate `$SCRATCH/dyncheck` (edition 2024, Rust 1.99), a `RefCell` store standing in for `SqliteStore` | Builds and runs; `fake-a` and `fake-b` see only their own key |
| `jiff::tz::TimeZone::try_system` exists in 0.2.39 | Same crate, `jiff = "=0.2.39"`, features `std, tz-system, alloc, tzdb-bundle-always` | Exists; returned `Europe/Paris`. `TimeZone::UTC` exists for the fallback |
| A small or schema-less file is treated as an empty database by SQLite | Python `sqlite3` (same engine family): a 1-byte file, and an 8 KB file whose only table was dropped | Both open with zero tables, `application_id = 0`, `user_version = 0`. `inspect` returns `None` on exactly that, so `SqliteStore::open` runs `create` and writes. Files of 50 bytes or more of garbage give `NOTADB` |
| `reqwest` and `ureq` honour the proxy variables by default | Sources in `~/.cargo/registry`: `reqwest-0.13.5/src/lib.rs` § Proxies; `ureq-3.5.0/src/config.rs:1073` (`proxy: Proxy::try_from_env()`) | Yes for both, upper and lower case, `ALL_PROXY` included |
| Spawn cost of the binary | Release build of `st-04/story`, 50 spawns each (macOS, `fullfsync`) | `test write` 27 ms, `test noop` 3.5 ms, `--help` 2.3 ms |

## Overall verdict
**BLOCKED.**
The design core is sound: the trait compiles as planned, the store view gives isolation by type, and the observer works with the HTTP clients a later lane will pick. But the generic conformance test, the one thing frozen for every later lane, cannot be green on the build the plan describes. ST-03's `test` module stays a toggleable module, enabled by default in test builds, and it holds actions that panic or fail on purpose (F-1). The isolation check also fails on every module as written (F-2). F-1 changes a resolved spec point (GAP-ST04-3, RG-ST04-1.6), so Quentin answers Q-DA1 before the Skeleton. The other findings are plan changes. After Q-DA1 and those changes, the Skeleton can start with no second DA iteration.

## Findings

### [B] F-1: The conformance test cannot pass on ST-03's `test` module, which the plan keeps toggleable
- **Axis**: Consistency / Completeness of the frozen tooling
- **Source**: specs RG-ST04-1.6 and GAP-ST04-3 (`test` enabled by default in test builds), RG-ST04-4.2 (every `∅` action in cut mode: exit ∈ {0, 1}, no panic), RG-ST04-5.1 (no list of modules written in the test), RG-ST04-5.2 ("for every toggleable module"), matrix US-5 row 1 ("green on `test`, `fake-a`, `fake-b`"); `crates/cli/src/wiring/test.rs` (`test.panic`, `test.fail`, `test.bad-output`, `test.write --fail panic`).
- **Tasks**: T-1, T-4, T-7.
- **Description**: the conformance test discovers modules from `module list` and runs check 6 on every `∅` action of every toggleable module. `test` is toggleable. `test.panic` panics (exit 3 `internal_error`), `test.fail` exits 3, `test.bad-output` exits 3 `output_mismatch`. All three are `∅` actions, so check 6 is red by design. The test may not name modules (RG-ST04-5.1), so it cannot skip `test` without a written exception. Matrix US-5 row 1 cannot hold as specified.
- **Required plan change**: Quentin answers **Q-DA1**. With (a): T-1 renames the test actions `test.*` to `arbitre.test.*`, owned by the foundation under `test-actions`. Command paths do not change (`command_path` drops a leading `arbitre`, so `arbitre test echo` still works). `DEFAULT_ENABLED` and its `cfg` branch disappear: nothing is enabled by default in any build. GAP-ST04-3 becomes moot. `test.write` writes in namespace `arbitre`. ST-03 tests that read records of namespace `test` change their expected namespace. The specs (RG-ST04-1.6, GAP-ST04-3, matrix US-1 row 8, US-5 row 1) are updated to match.

### [M] F-2: The isolation check fails on every module as written, and `test.signal` breaks it too
- **Axis**: Consistency (check 4 vs setup and signals)
- **Source**: specs RG-ST04-5.2 item 4 ("compared on `writes_since(START)`"), RG-ST04-1.5 (enabling writes `arbitre/module-state/<device>`), RG-ST04-6.1 (signals live in namespace `arbitre`), RG-ST04-6.3 (simulator `test.signal`); plan T-6 (`test.signal` in `test.rs`), P-7.
- **Tasks**: T-1, T-6, T-7.
- **Description**: three problems.
  1. Check 4 must first enable `m`, which writes a record in namespace `arbitre`. `writes_since(START)` therefore always holds a write outside `m`, and check 4 is red for every module.
  2. `test.signal` deposits into namespace `arbitre` from module `test`. Through `Context` it only has the `test` view, so it cannot be written without a back door to the whole store. If it gets one, check 4 catches it and goes red, rightly.
  3. Any action that runs after an unread signal marks it read, which is a write in `arbitre`. Check 4 would blame that action.
- **Required plan change**: (1) Check 4 takes the position after setup (last `writes_since` position once `m` is enabled and the fixtures are written), runs the action, then asserts every newer write has `record.module == m`. (2) `test.signal` becomes a foundation action under `test-actions` (`arbitre.test.signal` under Q-DA1 (a), `arbitre.test-signal` otherwise), laid by T-1 in `wiring/arbitre.rs` with an empty body that T-6 fills. (3) T-7 states that each conformance check starts from a fresh store, and that no signal is deposited in it.

### [M] F-3: `module.enable` cannot run through `Module::execute`, and the store has no query for "all records of a module"
- **Axis**: Soundness of the frozen contract (RG-ST04-3.2, 3.3, 7.2)
- **Source**: specs RG-ST04-3.2 ("no other field"), RG-ST04-3.3 (no `writes_since` in the view), RG-ST04-7.2 (enable revalidates **all** records of the module); `crates/core/src/store.rs` (`records(module, category)` needs a category; no category listing); plan P-3, P-9, T-4, T-5.
- **Tasks**: T-1, T-4, T-5.
- **Description**: `module.enable` must write its own record, list every record of module `m` in every category, build `m`'s view, and call `m.revalidate`. A `Module` impl only gets a `Context` with its own view (`arbitre`). It sees neither the other modules nor their namespaces. The `Store` port has no way to list a module's categories. The plan does not say how the foundation runs. The natural shortcut, a new field in `Context` or a `writes_since` in the view, would break the frozen contract on its first day.
- **Required plan change**: P-9 and T-4 state that the foundation is **not** a `Module` impl. It is a CLI-side dispatcher in `wiring/arbitre.rs` that holds the raw `&SqliteStore`, the registry and the clock. It builds `ModuleStore` values for the module it acts on. Records of `m` are enumerated with `writes_since(START)`, filtered on `record.module == m`, deduplicated on `(category, id)`, and deleted records are left out (the projected state says so). Revalidation runs **before** the state write. A `StoreError` from it then exits 3 and leaves the state unchanged. A cost note: this is one log scan per enable, the same order as the open-time check `SqliteStore::open` already does.

### [M] F-4: `ModuleStore` must keep its store field private, or isolation does not hold by construction
- **Axis**: Security / frozen contract (INV-ST04-3)
- **Source**: plan P-2 (written as a struct with fields `store: &'a S`, `module`, `author`, `clock`), specs RG-ST04-3.2 ("no access to the whole `Store`"), RG-ST04-3.3, RG-ST04-G.2 (`ModuleStore` frozen).
- **Tasks**: T-1, T-2.
- **Description**: if `store` or `module` is `pub`, a module writes `ctx.store.store.records("fake-b", "entry")` or builds a view for another namespace. E-ST04-9 cannot see that, because the module under test does not try. Making the fields private later changes a frozen type, which is an amendment.
- **Required plan change**: T-1 declares every field of `ModuleStore` private, with one `pub fn new(store, module, author, clock)` constructor used by the CLI, and no getter that returns `&S`. `Context` fields can stay public, since `store` is the view. T-2 adds a `compile_fail` doctest: `ctx.store.store` does not compile outside the crate.

### [M] F-5: "File exists and is not empty" does not stop the pre-parse open from writing
- **Axis**: Edge cases (RG-ST04-2.3, AC-2.5)
- **Source**: plan P-8; `crates/sqlite/src/lib.rs` `open_once` → `inspect` returns `None` when there is no table, `user_version = 0` and `application_id = 0` → `create` writes the layout and a device id; experiment above.
- **Tasks**: T-4.
- **Description**: a 1-byte file and a schema-less SQLite file are both "not empty". SQLite reads both as empty databases, so `--help` would create a store inside them and overwrite the user's file. Two smaller gaps sit next to it. Without `HOME` or `ARBITRE_STORE`, `location` fails, and the plan does not say whether `--help` then exits 0 or 3. A `--store` that names a directory is not covered either.
- **Required plan change**: `store::open_existing` reads the first 100 bytes before opening. Missing path or 0 bytes: default state, no open. Anything other than the SQLite magic with `application_id` = `ARBT` at offset 68: `storage_corrupted` (`NotAStore`), no open. Otherwise: `SqliteStore::open`, which can no longer reach `create`, since `inspect` only returns `None` when `application_id` is 0. A location error before parsing gives the default state. The action then reports `no_default_store` as in ST-03. Add the 1-byte and the dropped-table files to E-ST04-2 row 5, with the bytes compared after `--help`.

### [M] F-6: `pre_scan` must find `--store` after the first word too
- **Axis**: Edge cases (RG-ST04-2.2, 2.3)
- **Source**: `crates/cli/src/cli/mod.rs` (`--store` and `--json` are `global(true)`, so they are valid after any sub-command); `crates/cli/tests/location.rs:87` (`test read --store <p> --json`); plan "Under-engineering" and the T-2 table (only "options before the word").
- **Tasks**: T-2, T-4.
- **Description**: `arbitre fake-a list --store p` is valid. If `pre_scan` stops at the first word, the enabled state is read from the default store, while the action runs on `p`. The result is a false exit 4 or a disabled module shown as enabled. The D-2 CI flow (`arbitre diff check --store <tmp>`) hits this directly. Re-resolving the location after parsing can also disagree with the pre-parse one.
- **Required plan change**: `pre_scan` reads `--store` and `--store=` anywhere up to `--`. The first word is still the first non-option argument that is not `--store`'s value. The location is resolved once, before parsing, and the action uses that path and that handle. The T-2 table adds: `--store` after the word, `--store` given twice (clap then gives usage 2; the pre-scan takes the last one, read only), `--store=` empty.

### [M] F-7: The shipped build has no toggleable module, so `module enable` has an empty enum and the catalogue is refused
- **Axis**: Regressions (shipped configuration)
- **Source**: `crates/core/src/catalog.rs` `check_schema` (`Schema::Enum(vs)` with `vs.is_empty()` ⇒ "empty or duplicate enum"); `crates/cli/src/modules.rs` `shipped()` (empty until ST-05) and unit test `shipped_catalogue_builds`; specs RG-ST04-1.1 (input = enum of the toggleable ids).
- **Tasks**: T-4.
- **Description**: without `test-actions`, the toggleable set is empty. The enum of `module.enable` and `module.disable` is empty, so `Catalog::build` fails. Every command of the release binary would exit 3 `catalog_invalid`. `shipped_catalogue_builds` turns red at T-4, with no decision in the plan.
- **Required plan change**: T-4 states that the foundation declares `module.enable` and `module.disable` only when at least one toggleable module exists. `module.list` is always present. `shipped_catalogue_builds` stays as the guard.

### [M] F-8: The catalogue is validated on the enabled subset only
- **Axis**: Edge cases / Regressions (RG-ST04-3.4, 2.2)
- **Source**: plan P-4 (`Catalog::build(foundation, modules)` called with the enabled modules); `catalog.rs` (duplicate ids, names and paths are checked only among the modules passed; the module id itself is never checked for form); specs RG-ST04-2.2 (precedence on the first word).
- **Tasks**: T-2, T-4.
- **Description**: a defect in a disabled module stays hidden until someone enables it. Two concrete cases:
  - a toggleable module with id `module`: `arbitre module enable --module module` has first word `module`, which is a disabled module id, so it exits 4 and can never be enabled;
  - two modules with the same id, or a bad descriptor, pass while one is off. Once enabled, every command exits 3, `module disable` included, so the user cannot undo it without editing the store by hand.
- **Required plan change**: at every start, `main.rs` validates foundation + **all** toggleable modules once, then filters to the enabled set. `Catalog::build` also refuses a module id that is ill-formed (the `dashed` rule) or that equals a first command segment of the foundation (`module`, the test groups). T-2 adds both cases to `core/tests/module.rs`.

### [M] F-9: The surface contract is too loose to freeze; ST-07 would have to amend it
- **Axis**: Frozen tooling (F-27, AC-5.3)
- **Source**: plan P-13 (`Surface { name, actions(env), call(env, action, input) -> Called }`, `Env` and `Called` undefined), P-14; specs RG-ST04-5.2 items 1, 2, 7; frame CONFLIT-6 ("appelé explicitement **en ligne de commande**"), INV-23.
- **Tasks**: T-7, T-8.
- **Description**: the MCP surface must join with one file and one line, so the trait must already carry what MCP needs:
  - `Env` is not defined. If it does not carry the store path and the observer's variables, an MCP surface cannot run on the same store or under cut mode.
  - `Called` holds "exit code and document". An MCP call has no exit code.
  - The plan does not say which checks loop over surfaces. Under CONFLIT-6, "explicit call ⇒ 4" is a command-line rule. A disabled MCP tool is simply absent.
  - If `cli.rs::actions` reads `module list --json`, which is also the catalogue source (P-14), check 7 compares a list with itself. Help invisibility (INV-ST04-1, AC-2.1) is then never tested by the generic test.
- **Required plan change**: P-13 fixes:
  - `Env { store: PathBuf, vars: Vec<(OsString, OsString)> }`, where `vars` holds the observer's variables; every surface applies it to every process it starts;
  - `Called { code: u8, doc: serde_json::Value }`; a surface without an exit code takes `doc.error.code`, or 0 on success;
  - `actions()` reads what the user would see: for `cli.rs`, a recursive walk of `--help`; for MCP, `tools/list`;
  - per surface: checks 1, 2 (visibility only) and 7; CLI only: check 2 (explicit call ⇒ 4), 3, 4, 5 and 6.

  T-8 writes this split in CONTRIBUTING "Adding a surface".

### [M] F-10: The subset and parity checks compare result documents that real modules will not reproduce
- **Axis**: Frozen tooling / Regressions (ST-05 onwards)
- **Source**: specs RG-ST04-5.2 item 5 ("same exit code, same data"), item 7 ("same result document"); plan T-7; `SystemClock` (P-11) reads the wall clock; each fresh store gets a random device id (`crates/sqlite/src/lib.rs` `create`).
- **Tasks**: T-7.
- **Description**: the fake module returns deterministic documents, so the checks pass in ST-04. ST-05's actions will return generated ids or timestamps. Two runs on two fresh stores, milliseconds apart, then give different documents, and checks 5 and 7 turn red. The fix would touch frozen tooling, which is an amendment at the first real module.
- **Required plan change**: the comparison runs are made on byte copies of one prepared store (same device id, same data). Physical time is pinned by a variable that `SystemClock` reads only under `cfg(feature = "test-actions")` (for example `ARBITRE_TEST_NOW`, in ms). The conformance test sets it for every comparison run. T-3 adds the variable; T-7 uses it. Document it in CONTRIBUTING as part of the frozen tooling.

### [m] F-11: PROP-2 has no generic open-mode check, and cut mode is not a real cut
- **Axis**: Completeness (frame § ST-04 "assertion PROP-2") / observer blind spots
- **Source**: specs RG-ST04-4.4 ("the check exists for the stories that add adapters"), RG-ST04-5.2 item 6 (4.2 and 4.3 only), RG-ST04-5.6; plan P-12, R-4, T-8.
- **Tasks**: T-7, T-8.
- **Description**: the generic test never runs open mode, and has no per-kind list of allowed hosts. ST-10, ST-19 and ST-30 therefore assert PROP-2 in their own tests, with the observer's API. That API is frozen, but the plan never names it. In cut mode, the proxy only refuses clients that honour the variables. Raw TCP, DNS lookups and most WebSocket clients (likely for the ST-19 relay) go straight to the real network on a CI runner, so they are neither recorded nor cut.
- **Required plan change**: P-12 names the frozen API: `Observer::start(Mode)`, `env()`, `records()`. T-8's "Network clients" rule says that (1) PROP-2 is asserted by each adapter story in its own `tests/<adapter>.rs` file through that API, not under `support/`; (2) cut mode is a proxy refusal, not a network cut; (3) a client that does not read the proxy variables, WebSocket included, needs its own proxy support before its story can pass.

### [m] F-12: Mutation cost is measured too early, and a timed-out child is not killed
- **Axis**: Risk R-2, R-5
- **Source**: plan R-2 ("the conformance test lives in its own `[[test]]` target"), T-5 (measurement), P-15; `50-st01-mutation.sh` (cargo-mutants runs every test target of the mutated package); ST-03 `07_review.md` (CLI: 61 mutants in 54 s).
- **Tasks**: T-5, T-7.
- **Description**: a separate `[[test]]` target does not cut mutation time: cargo-mutants still runs it for every CLI mutant that survives the earlier targets. The measurement at T-5 runs before the conformance and network tests exist, so it under-reads. P-15 detects a hang but does not kill the child. A hung child then outlives the test, keeps the store locked, and can stall the next check.
- **Required plan change**: move the measurement to the end of T-7, with the full suite, and keep the 30-minute rule. Drop the `[[test]]` argument from R-2's mitigation. P-15: on timeout, `kill()` then `wait()`.

### [m] F-13: The CLI clock's mutants survive unless a spawned test sets `TZ`
- **Axis**: Mutation gate (Std file with no direct test)
- **Source**: plan P-11, zones (`crates/cli/src/clock.rs` Std), E-ST04-8 (core only), E-ST04-6.
- **Tasks**: T-3, T-6.
- **Description**: `timezone()` mutated to UTC is not seen by any planned test. On a UTC CI runner the signal line is the same either way.
- **Required plan change**: one E-ST04-6 case spawns with `TZ=America/New_York` and asserts the local time in the signal line. jiff reads `TZ` first on Unix. With F-10's pinned time, the same case also covers `now()`.

### [m] F-14: Task order and file ownership
- **Axis**: Task dependencies and collisions
- **Source**: plan T-1, T-4, T-5, T-6, "Can run together"; `crates/cli/tests/output.rs` (help-order list includes `not-enabled`); `wiring/test.rs` `read` (uses `writes_since`).
- **Tasks**: T-1, T-4 to T-6.
- **Description**:
  - T-4's completion criterion requires every ST-03 test to pass, but the `codes.rs` "module not enabled" row needs `fake-a`, which comes in T-5.
  - T-1 says `test.rs` is ported "bodies kept". `test.read` calls `writes_since`, which the view does not have (RG-ST04-3.3). It must use `records("item")`, which sorts by id, not by write order.
  - `output.rs`'s help-order test names `not-enabled`, which is removed.
  - With F-2, T-6 touches `wiring/arbitre.rs`, which T-5 owns.
- **Required plan change**: (1) Move the `codes.rs` row to T-5's completion criterion. (2) T-1 ports `test.read` to `records("item")`; the action's summary becomes "Lists the keys written by test.write, by key". (3) T-1 updates the help-order list. (4) T-1 lays the signal action's descriptor and dispatch arm, so T-6 only fills `signals.rs` and the body.

### [m] F-15: Small gaps
- **Axis**: Completeness
- **Source**: specs matrix US-5 row 4, RG-ST04-1.5, RG-ST04-G.2; plan P-6, T-7.
- **Description and required plan change**:
  - US-5 row 4 (a module with zero actions) has no task. T-7 adds it as a unit test of the `support/conformance.rs` helpers.
  - P-6 does not say what `enabled_from` does with a value other than `b"1"`/`b"0"`, or with a field for a module that is no longer in the build. Rule: anything other than `b"1"` or `b"0"` gives the default; fields for ids outside the build are ignored. Add both to `core/tests/module.rs`.
  - `Clock::timezone() -> jiff::tz::TimeZone` makes `jiff` 0.2 part of the frozen contract: a major `jiff` upgrade becomes a foundation amendment. T-8 writes this in the PR body and in the frozen list of CONTRIBUTING.

## Axes with no finding
- **Dyn and lifetime design (P-1 to P-3)**: sound, verified by experiment. One `'a` for the receiver, the context and the future is enough, and `Box<dyn Module<SqliteStore>>` is object-safe with a generic `S` on the trait. A boxed future per call costs nothing at this scale.
- **`jiff`**: `try_system` and `UTC` exist in 0.2.39. Feature unification of `std` into core on native builds is harmless, because `20-st01-browser.sh` checks `-p arbitre-core` alone for wasm32. `graph.py` has no rule against `jiff` in core.
- **Observer mechanism (Q5 (a))**: `reqwest` 0.13 and `ureq` 3 read `HTTP(S)_PROXY`/`ALL_PROXY` in both cases by default, so P-12's variables reach the clients a lane will likely pick. Parallel tests each have their own proxy, so records are not mixed between tests.
- **Per-device records under sync and export (Q1 (a))**: no device reads another device's `module-state` or signal ids, so a merge can never change what a device shows. Restoring an export on a new device gives that device the default state, which matches AMB-12. Foreign signals are never shown or marked; they cost a few bytes each.
- **Signals**: the stderr-only, well-formed-action-only rule follows ST-03 Q-DA2 (a). Marking read with `Author::System` (the variant exists) in one local apply holds "at most twice" under a race. A failure to show never changes the action's outcome.
- **Revalidate on enable**: no revalidation when `changed: false`. The cost is one log scan, the same order as the open-time check (with F-3's details).
- **Zones vs CONTRIBUTING**: no file is lowered. No core file is `Gen`. `wiring/fake.rs` in `Gen` is allowed. New core files are listed `Std`, so none falls back to `Full`. `lib.rs` has only `mod` lines and gives no mutant.
- **Executor**: one poll stays correct. Every planned future is ready on the first poll, because `SqliteStore` awaits nothing.
- **Frame deliverables**: every bullet of § ST-04 has a task, except the PROP-2 assertion, which F-11 settles. Shell completion is out of scope by the story.

## New questions for Quentin (before the Skeleton)

**Q-DA1 — Where do ST-03's test actions live? (F-1) [changes GAP-ST04-3 and RG-ST04-1.6]**
- (a) They become foundation actions `arbitre.test.*`, under `test-actions`. Same command paths (`arbitre test echo`). They are always on, and the conformance test, which only loops over toggleable modules, never sees them. Nothing is enabled by default in any build, so the test-build exception and its `cfg` branch in a Std file disappear. `test.write` writes in namespace `arbitre`.
- (b) `test` stays a toggleable module, enabled by default in test builds. The conformance test skips id `test` by one written exception, against RG-ST04-5.1, and every later module relies on that exception staying correct.
- **DA recommendation: (a).** It removes a special case instead of adding one. The CLI paths of the ST-03 suite stay unchanged. The only cost is the namespace of `test.write` records, which only ST-03's own tests read.

## Recommendation
Quentin answers Q-DA1. The Planner folds the answer and F-2 to F-15 into `04_plan.md` (v2), and Q-DA1's spec lines into `02_specs.md`. The Skeleton then starts with no second DA iteration (supporting zone, one iteration maximum). The changes stay inside these findings.
