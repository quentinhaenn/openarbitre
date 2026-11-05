# Code Review — ST-04: Modules you can enable, kept apart, with one generic conformance test

## Verdict: APPROVED WITH FIXES

- Reviewed: `git diff st-03/story...HEAD` (44 files, +4722/−255) at `ccb7e46`. T-2 to T-7 are done; T-8 (docs, PR) is not started.
- Against: `01_story.md`, `02_specs.md` v2 (§ Resolution and § DA iteration 1 binding), `04_plan.md` v2, `05_da_review.md`, `06_skeleton.md`, `zones.toml`, `CONTRIBUTING.md`.
- Checks run by the reviewer: see § Checks.
- Date: 2026-11-05 · Phase 8, zone Supporting.

## Summary

The core contract is what Q1 to Q7 arbitrated, and it is small. `Module<S>` is dyn-compatible with boxed local futures. `ModuleStore` has private fields, so no method can name a module, and a `compile_fail` doctest guards it. `Problem`, `Ai`/`AiError` and `Clock::timezone` match the specs. `arbitre-core` has `jiff` as its only normal dependency, with no `std` feature on wasm32. The binary's pre-parse pipeline does what DA F-5 and F-6 asked: it reads `--store` anywhere, resolves the location once, checks the SQLite/`ARBT` header before opening, and never creates or writes a non-store file. Mutation on the Std files: 211 caught, 0 missed.

The weak part is the test tooling that ST-04 **freezes**. Once ST-04 merges, any change to it is a foundation amendment, so its defects should be fixed now:
- the network check mixes up attempts between actions and asserts nothing for the foundation actions (R-1);
- the open-mode property RG-ST04-4.4 / AC-4.3 has no check at all (R-2);
- the CLI derivation names options `--due_date` where the ST-03 spec and both frozen encoders say `--due-date` (R-3, an ST-03 defect that ST-04's tooling now depends on).

Every other finding is minor or a nit.

## Conformance to the specs

| Rule | Status | Notes |
|---|---|---|
| RG-ST04-1.1 three foundation actions | ✅ | `modules.rs` E-ST04-1; F-7 (`enable`/`disable` only with a toggleable module) in `shipped_catalogue_builds` |
| RG-ST04-1.2 idempotent | ✅ | Second `enable`: `changed: false`, `snapshot` identical |
| RG-ST04-1.3 data kept | ✅ | `disabling_keeps_the_data`; conformance check 3 |
| RG-ST04-1.4 any subset | ⚠️ | Check 5 compares the exit code and stdout, not the data; enumeration breaks above 16 modules (R-5, R-6) |
| RG-ST04-1.5 per-device state | ⚠️ | Layout tested (`state_record_is_per_device`). **Matrix US-1 row 10 / PROP-ST04-7 (another device's record applied) has no test** (R-10) |
| RG-ST04-1.6 default off | ✅ | `enabled_from` with `1`, `0`, `x` and an unknown id; fresh device |
| RG-ST04-1.7 foundation always on | ✅ | `--module arbitre`, `--module nope` ⇒ 2, store not created |
| RG-ST04-2.1 invisible | ✅ | Root help, conformance check 2 through every surface's `actions()` |
| RG-ST04-2.2 not enabled ⇒ 4 before parsing | ✅ | Beats usage, JSON mode, option before the word, `--store` after it |
| RG-ST04-2.3 read only, damaged store | ✅ | Text, 1-byte and header-only files byte-identical; schema-less SQLite in `store.rs`. Exit 3 now also covers usage errors (T-4 deviation, Q-R1) |
| RG-ST04-2.4 entry point = module id | ✅ | By derivation, on `fake-*` |
| RG-ST04-3.1 `Module` | ✅ | Q3 (a); `settle` unchanged around `execute` |
| RG-ST04-3.2 `Context` | ✅ | Four public fields; `ai: None` in `main.rs`. US-3 row 4 is not asserted by a test (construction only) |
| RG-ST04-3.3 view | ✅ | E-ST04-9 on `SimStore`, doctest, conformance check 4 (mutating actions only, R-4) |
| RG-ST04-3.4 foundation owns `arbitre` | ✅ | `arbitre`, `module`, `test`, `Fake`, `fake_a` refused; whole build validated at every start |
| RG-ST04-3.5 `Ai` declared | ✅ | Four ids mapped and tested |
| RG-ST04-3.6 seam replaced | ✅ | `Wiring` and the CLI `Context` are gone; one line in `modules.rs` per module |
| RG-ST04-4.1 observer | ⚠️ | Works and self-tests. API differs from the specs: `take()` with no `argv` (skeleton decision 2, R-8) |
| RG-ST04-4.2 no network, cut | ⚠️ | Toggleable modules ✅. Foundation actions (matrix US-4 row 6): attempts discarded, never asserted (R-1) |
| RG-ST04-4.3 declared network, cut | ⚠️ | Exit and error id checked. Attempts are not drained, so they are blamed on the next action (R-1). "Exit 0 only with a named degraded mode" is not checked |
| RG-ST04-4.4 / AC-4.3 open mode | ❌ | No check (R-2) |
| RG-ST04-5.1 discovery | ✅ | `module list --json`, then `--help` walk per surface |
| RG-ST04-5.2 checks 1–7 | ⚠️ | All present; gaps R-1, R-4, R-5, R-6, R-7 |
| RG-ST04-5.3 minimal input | ✅ | Unit test for each subset type |
| RG-ST04-5.4 surface registry | ✅ | `Env`, `Called`, `Surface`, `all()` as in P-14 |
| RG-ST04-5.5 fake module | ✅ | |
| RG-ST04-5.6 frozen tooling | ⚠️ | Frozen as written in `support/mod.rs`; see R-1 to R-3, R-7, R-9 before freezing |
| RG-ST04-6.1 deposit | ⚠️ | `source` is not checked against `snake_case` although the layout is frozen (R-11) |
| RG-ST04-6.2, 6.3 show once | ✅ | stderr only, oldest first, `TZ` case, not on help/usage. Other device: core `unread` test |
| RG-ST04-7.1, 7.2 revalidate | ✅ | Invalid found, deleted skipped, second `enable` does not revalidate |
| RG-ST04-8.1, 8.2 time zone | ✅ | Paris offset change, `SimClock` setter, browser build. The native "UTC when unreadable" fallback is untested (`[HYP]`, acceptable) |
| RG-ST04-G.1 offline | ✅ | No network crate |
| RG-ST04-G.2 frozen contract | ⚠️ | See § Frozen contract |

## Frozen contract (RG-ST04-G.2)

| Item | Status | Notes |
|---|---|---|
| `Module<S>` | ✅ | `id`, `actions`, `execute`, `revalidate`; boxed local futures (Q3 (a)); `revalidate` takes `(category, id)` pairs (skeleton decision 4, sound: the caller cannot name another namespace) |
| `Context<'a, S>` | ✅ | `author`, `clock`, `store` (the view), `ai`. Note: the view carries its own author and clock; the surface must pass the same ones twice. Acceptable, and the DA approved the shape |
| `ModuleStore` | ✅ | Private fields, one `new`, no store getter; `module()` getter extra (harmless); no `writes_since` |
| `Problem` | ✅ | `{record, id, message}` (Q4 (a)) |
| `Ai`, `AiError` | ✅ | `complete(&Value, timeout_ms)`; four variants, four ids |
| `Clock::timezone` | ✅ | Required method returning `jiff::tz::TimeZone`; `jiff` 0.2 enters the contract (DA F-15) |
| Observer API | ⚠️ | `start`, `env`, `take` and `Attempt { host, port }`; the specs still say `records()` with `argv` (R-8) |
| Surfaces contract | ✅ | As P-14; `Env.vars` should carry the pinned time (R-7) |
| Record layouts | ⚠️ | State record ✅; signal record ✅ but `source` is unchecked (R-11) |
| Frozen by mistake | — | `pub use module::*` and `signal::*` also export `pre_scan`, `enabled_from`, `state_change`, `state_record`, `unread`, `line`. G.2 does not list them; T-8's frozen list in `CONTRIBUTING.md` should name only the G.2 items so these stay free |

## Security and data safety

- **Non-store files**: `open_existing` refuses any non-empty file without the SQLite magic and `ARBT` at offset 68, before `SqliteStore::open`, on every pre-parse path, help included. An `ARBT` file with dropped tables reaches `inspect`, which refuses `Layout` without writing (`crates/sqlite/src/lib.rs:193`). An empty file still becomes a store on the first action (ST-02 behaviour, F-5 accepted it). ✅
- **Namespace isolation**: a module only ever holds a `ModuleStore`, whose fields are private (doctest), and its `RecordId`s are built from the view's `module`. Ids `arbitre`, `module` and `test` are refused for the whole build at every start. `RecordId` is structured, so no separator can be injected. ✅ The one remaining path, a module opening its own `SqliteStore` from a path, cannot be closed by types and is out of scope.
- **Network in core**: `cargo tree -p arbitre-core -e normal,features --target wasm32-unknown-unknown` gives `jiff` (`alloc`, `tzdb-bundle-always`) → `jiff-core`, `jiff-tzdb`; no `std`, no network crate. ✅
- `ARBITRE_TEST_NOW` is read only with `test-actions`. ✅

## Findings

| Id | Severity | File:line | Finding | Proposed fix |
|---|---|---|---|---|
| R-1 | **MAJOR** | `crates/cli/tests/conformance.rs:203-217` | Check 6 drains the observer (`o.take()`) only after actions with no network. Attempts made by a declared-network action stay queued and are blamed on the next action, which gives a false red as soon as a real adapter sits before an offline action. The `o.take()` at line 203 also throws away what the `module enable` calls of `fresh()` recorded, so matrix US-4 row 6 (foundation actions in cut mode, zero attempts) is never asserted. The file is frozen tooling | Take after **every** call: `let seen = o.take();` then assert `seen.is_empty()` only for undeclared actions. Replace the bare `o.take()` at line 203 with `assert!(o.take().is_empty(), "foundation actions tried the network")`, and run `module list` and one `disable`/`enable` pair inside the cut check |
| R-2 | **MAJOR** | `crates/cli/tests/conformance.rs` (absent); specs RG-ST04-4.4, story AC-4.3 | No test runs the binary in `Mode::Open`. The specs say "the check exists for the stories that add adapters". As written, the first adapter story would have to amend frozen tooling to get it | Add `check_6_network_open`: the same loop as check 6 with `Observer::start(Mode::Open)`; an undeclared action records nothing; a declared one records only hosts from an allow-list given per kind (empty in ST-04, so `fake-a push` must record nothing too). About 15 lines. Adapter stories then only fill the allow-list |
| R-3 | **MAJOR** | `crates/cli/src/cli/mod.rs:75` (ST-03 code) | `.long(p.name.clone())` keeps `_`, but RG-ST03-4.3 says "CLI option = `--` + name with `_` → `-`", and both frozen encoders (`conformance.rs:96`, `surfaces/cli.rs:49`) follow the spec. No property has a `_` today, so nothing is red. The first module with `due_date` (ST-05) would fail every conformance check with exit 2, and its users would see an option name the spec forbids | `.long(p.name.replace('_', "-"))`, plus one row in `usage.rs` or a `cli/mod.rs` unit test with a `snake_case` property. It conforms to ST-03's frozen rule, so it is not an amendment |
| R-4 | MINOR | `crates/cli/tests/conformance.rs:150` | Check 4 runs only the mutating actions. RG-ST04-5.2 item 4 says "running every action of `m` writes nothing outside namespace `m`" | Drop the `mutating` filter for the isolation part and keep it for the data-kept part (or run all actions; the data-kept comparison still holds) |
| R-5 | MINOR | `crates/cli/tests/conformance.rs:191-193` | Check 5 compares `(code, stdout)` only. RG-ST04-5.2 item 5 asks for "same exit code, same data", so a module whose writes depend on another module's state passes | Also compare the module's namespace after the call in both copies, as `(record, change)` pairs: stamps differ, because `part` has extra `disable` writes |
| R-6 | MINOR | `crates/cli/tests/conformance.rs:176-178` | `1u32..(1 << ids.len().min(16))` with `mask & (1 << i)`: modules past index 15 never get a pair, and `1 << i` overflows (a panic in debug) at 32 modules. Frozen tooling, and the pairs rule exists for large builds | When `ids.len() > 6`, enumerate the singletons and pairs directly (`for i in 0..n { for j in i..n { … } }`); keep the mask loop only for `n ≤ 6` |
| R-7 | MINOR | `crates/cli/tests/conformance.rs:26-31, 44-51` | T-7 deviation: `ARBITRE_TEST_NOW` is set by `Case::cmd` but not in `Env.vars`, so `Surface::call` (check 7) runs with wall time. With one surface this is invisible. When ST-07 adds MCP, any time-bearing output differs between surfaces and check 7 flakes, and the fix would touch frozen tooling | Put `("ARBITRE_TEST_NOW", NOW)` into `Env.vars` in `fresh()`, and drop the `.env(...)` in `Case::cmd` (it already applies `vars`) |
| R-8 | MINOR | `crates/cli/tests/support/network.rs:18-23, 61-64`; specs RG-ST04-4.1, plan P-13 | The frozen observer API is `take()` (drain) with `Attempt { host, port }`, while the specs and the DA F-11 API say `records()` and `(host, port, argv)`. Skeleton decision 2 explains why (a proxy cannot see the process) and the design is sound. But the frozen contract must be written as built | Amend RG-ST04-4.1 and P-13: "`take()` returns the attempts since the last call; tests call it after each command". Repeat it in T-8's frozen list. No code change |
| R-9 | MINOR | `crates/cli/tests/conformance.rs:93-112` vs `crates/cli/tests/support/surfaces/cli.rs:41-59` | Two encoders from input to options in the frozen tooling. `options()` silently drops `true` booleans and list items, so it only works because minimal inputs have none | One encoder: make `surfaces::cli::args(action, input) -> Vec<String>` public and use it in `call_cli`. About −12 lines |
| R-10 | MINOR | `crates/core/tests/module.rs` (absent); matrix US-1 row 10, PROP-ST04-7 | No test applies another device's state record and checks that this device's enabled set is unchanged. `state_record_is_per_device` checks only the id format | Core test: `SimStore` of device A merges a write of `state_record(B)` with `fake-a = 1`; `enabled_from(state(state_record(A)))` stays empty. About 10 lines |
| R-11 | MINOR | `crates/cli/src/signals.rs:27-39`; `crates/core/src/signal.rs:21` | T-6 deviation: `source` is not checked. The record layout is frozen with `source` as `snake_case`, and it is part of the record id | Check `[a-z][a-z0-9_]*` in `deposit` and return `internal_error` otherwise (callers pass constants, so a bad one is a programming error). Give `test.signal` a valid default in the tests (they already use `s`, `sync`, `calendar_refresh`) |
| R-12 | NIT | `crates/cli/src/wiring/arbitre.rs:278` | `describe` builds the public `logged_as` and `requires_network` strings from `Debug` + lowercase. MCP (ST-07) will need the same names | `as_str()` on `LoggedAs` and `Network` in core (adding a method is not an amendment), used here |
| R-13 | NIT | `crates/cli/tests/conformance.rs:296-305` | `a_module_with_no_action_passes_the_loops` asserts properties of an empty `Vec` and exercises none of the checks (US-5 row 4, DA F-15) | Delete it and note in the specs that row 4 holds by construction (every check is a loop over `m.actions`), or extract one check body into a function and call it on the empty `ModuleInfo` |
| R-14 | NIT | `crates/cli/src/wiring/arbitre.rs:109`, `arbitre_test.rs:58`, `fake.rs:51`; `arbitre_test.rs:43` / `fake.rs:38` | `mutating` closure written three times, `key` property twice, `object` helper twice | `pub fn mutating(a)` and `pub fn object(kv)` next to `action` in `arbitre.rs`; one `key()` |

### Deviations reported by the agents: assessment

| Deviation | Assessment |
|---|---|
| T-4: unreadable store ⇒ exit 3 for every command but `--version`, usage errors included | **Accept**, with a spec line. Without the enabled state, the binary cannot tell a usage error from "module not enabled" (`fake-a put --bogus` is 4 or 2 depending on the state), so 3 is the only honest answer. It does narrow RG-ST03-1.1 ("malformed ⇒ 2"), a published rule, so Quentin confirms it (Q-R1). Then amend CONFLICT-ST04-2 and RG-ST04-2.3 to read "every command except `--version`", and have the T-8 README exit-code section say so |
| T-4: `human_help…` unit test made hermetic by T-6 | Accept. It used the machine's default store before; it now points `--store` at an absent temp path |
| T-6: signals not shown before `arbitre.test.signal` | Accept. The 2-line comment in `main.rs:130-131` gives the counter-intuitive reason on the spot, as the comment rule requires. A string compare in the shipped binary is harmless; `cfg!(feature = "test-actions") &&` would make the intent explicit (optional) |
| T-6: `source` format not validated in `deposit` | Fix: R-11 |
| T-7: polling 5 ms instead of 50 ms | Accept. It shortens the conformance run; the CPU cost of `try_wait` every 5 ms is negligible |
| T-7: foundation actions discovered through `module list` plus `--help` | Accept. That is P-14/P-15: `module list` names the modules, and `--help` is what the user sees |
| T-7: `ARBITRE_TEST_NOW` absent from `Env.vars` | Fix now: R-7 (one line, frozen tooling) |
| T-7: proxy listener thread lives until process end (`ponytail:`) | Accept. Per test process, a few idle threads; the comment names the ceiling |
| T-2: F-8 rule changes an ST-03 refusal message | Accept. Messages are not frozen; only the RG-ST03-3.3 ids are, and `catalog_invalid` is unchanged |

## C-TDD metrics (Supporting zone)

| Metric | Value | Threshold | Status |
|---|---|---|---|
| Tests passing | 312 / 312 | 100% | ✅ |
| Mutation, ST-04 Std files, full suite (T-7 commit) | 268 mutants: 211 caught, 57 unviable, 0 missed (7 min) | ≥ 70% | ✅ 100% |
| `arbitre-core` normal dependencies | `jiff` (+ `jiff-core`, `jiff-tzdb`), no `std` | `jiff` only | ✅ |

## Checks

Run by the reviewer at `ccb7e46` (`PATH=$HOME/.cargo/bin:$PATH`):

| Check | Result |
|---|---|
| `cargo test --workspace --all-features` | ✅ 312 passed, 0 failed, 30 suites (doctests included). `conformance.rs`: 8 tests in 1.45 s, far under the 3 min ceiling (R-5 of the plan) |
| `cargo fmt --all --check` | ✅ |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | ✅ |
| `cargo clippy --workspace --lib --bins --all-features -- -D clippy::cognitive_complexity` | ✅ |
| `scripts/ci.d/20-st01-browser.sh` | ✅ `arbitre-core` checks for wasm32 with `jiff`, `jiff-core`, `jiff-tzdb` |
| `cargo tree -p arbitre-core -e normal,features --target wasm32-unknown-unknown` | ✅ `jiff` features `alloc`, `tzdb-bundle-always` only |

## Simplification (ponytail)

There is little to cut. The foundation is a plain `match`, the fake module is one implementation with two instances, and the proxy uses only std. What stands out:
- S-1 (R-9): one input→options encoder instead of two. −12 lines.
- S-2 (R-14): shared `mutating`, `object`, `key`. −15 lines.
- S-3 `crates/cli/src/main.rs:164, 180, 205`: the catalogue is built up to three times per run (whole build, enabled subset, foundation only). It costs nothing measurable; keep it for clarity.
- S-4 `crates/cli/src/wiring/arbitre.rs:225`: `revalidate` scans every write of the store to list one module's records. Fine at this scale. If ST-05 measures it on a large store, add a `categories(module)` read to the `Store` port then, not now.

## Questions for Quentin

**Q-R1 — An unreadable store makes every command exit 3, usage errors included (T-4).**
- (a) Keep it. Amend CONFLICT-ST04-2 and RG-ST04-2.3 to "every command except `--version`", and have the README say so. Rationale: with no enabled state, the binary cannot tell usage (2) from "not enabled" (4), and a damaged store is the more urgent message.
- (b) Narrow it to what the plan wrote: help ⇒ 3, while a usage error that does not depend on the enabled state (an unknown root option, a malformed foundation command) stays 2. This needs a second parse against the foundation-only catalogue and a rule for which errors "do not depend" on the state, which is fragile.
- **Recommendation: (a).** It is the simpler and more predictable rule, it only applies to a damaged store, and RG-ST03-1.1 already yields to RG-ST03-1.5 for the enabled state.

## Must fix before the PR (T-8)

1. R-1, R-2: network checks in the frozen conformance test (drain per call, foundation asserted, open-mode check).
2. R-3: `_` → `-` in derived option names, plus a test.
3. R-7, R-8: pinned time in `Env.vars`; specs describe the observer API as built.
4. Q-R1 answered and written into the specs and README.

R-4 to R-6 and R-9 to R-11 are recommended before the tooling freezes. R-12 to R-14 are optional.

## Resolution

Branch `st-04/fixes` (on `ccb7e46`). Every finding is applied.

**Q-R1: Quentin chose (a) on 2026-11-05.** The behaviour stays as built: with an unreadable store, every command exits 3 (help and usage errors included), and only `--version` exits 0. CONFLICT-ST04-2 and RG-ST04-2.3 in `02_specs.md` are amended with that date and arbitration, and the README "Exit codes" section says so. `tests/modules.rs` `non_store_files_are_never_written` pins it: on each non-store file, `--help`, a usage error (`module enable --bogus`) and an unknown command all exit 3 `storage_corrupted`, `--version` exits 0, and the file is unchanged.

| Id | What was done |
|---|---|
| R-1 | `check_6_network_cut`: runs `module list` and a `disable`/`enable` pair per module, then asserts `o.take()` is empty ("foundation actions tried the network", matrix US-4 row 6). In the loop, `o.take()` runs after **every** call; only undeclared actions assert it empty |
| R-2 | New `check_6_network_open` (`Mode::Open`): each recorded attempt must target a host of `ALLOWED` for a kind the action declares. `ALLOWED` is empty in ST-04, so every action, `fake-a push` included, must record nothing; adapter stories add rows |
| R-3 | `cli/mod.rs`: `.long(p.name.replace('_', "-"))`. Unit test `option_names_use_dashes`: `--due-date` parses into input key `due_date`, and `--due_date` is refused |
| R-4 | Check 4 runs every action of the module, not just the mutating ones. The data-kept comparison still holds |
| R-5 | Check 5 also compares the module's namespace in both copies as `(record, change)` pairs |
| R-6 | `support::conformance::subsets(n)`: every subset up to 6 modules, singletons and pairs above. No `.min(16)` and no shift past `n ≤ 6`. Test `subsets_all_up_to_six_then_pairs` (n = 3, 6, 40) |
| R-7 | `fresh()` puts `ARBITRE_TEST_NOW` into `Env.vars`, so `Surface::call` gets it too. The extra `.env(…)` is gone |
| R-8 | RG-ST04-4.1, DA F-11 (specs) and P-13 (plan) now describe `take()` and `Attempt { host, port }`. The T-8 frozen list does not exist yet; T-8 writes it from these lines |
| R-9 | One encoder, `pub fn surfaces::cli::args`. `options` and `call_cli` are deleted. Checks 1 to 6 call the `Cli` surface itself (`Cli.call`), so "CLI only" (P-15) uses the same encoder and spawn as check 7. Check 5 compares `(code, doc)`, the parsed form of the old `(code, stdout)` |
| R-10 | Core test `another_devices_state_record_leaves_this_device_unchanged`: device B's `fake-a = 1` merged into A leaves A's enabled set empty |
| R-11 | `signals::deposit` refuses a `source` outside `[a-z][a-z0-9_]*` with `internal_error`, before writing. Test `source_must_be_snake_case` |
| R-12 | `LoggedAs::as_str`, `Network::as_str` in core, used by `describe`. Core test `logged_as_and_network_public_names` |
| R-13 | The tautological test is deleted. US-5 row 4 is now `a_module_with_no_action_enables_lists_and_disables` (`wiring/arbitre.rs`): a zero-action module builds a catalogue, and `enable`/`list`/`disable` give the expected outputs. Row 4 of the matrix says the per-action checks are empty by construction |
| R-14 | `mutating`, `key`, `object` and `text` are shared from `wiring/arbitre.rs` (see below) |

### Simplification applied with the fixes (tooling freezes with ST-04)

1. One input→options encoder: `surfaces::cli::args` (R-9).
2. One command builder on a store, `support::on(store, vars, args)` (home = `store.parent()`, `--store`, vars). It replaces `Case::cmd`, `Dev::run`'s body, `signals.rs` `on`, `surfaces/cli.rs` `spawn`'s body, and the `module list` call in `support/conformance.rs`. `Case` is now plain `Env`.
3. `foundation::text(input, key) -> Option<&str>` replaces the four `let Value::Object … unreachable!` copies.
4. `Change::field(name, bytes)` in core `write.rs`, used in `signal.rs`, `module.rs`, `arbitre_test.rs`, `fake.rs`, `tests/support/mod.rs` and core `tests/module.rs` (×2).
5. `RecordState::text(name)` in core `merge.rs` replaces `signal.rs` `text` and `fake.rs` `text`. Fake values are now read as strict UTF-8 instead of lossy; CLI input is always UTF-8.
6. `mutating`, `key()` and `object` are shared. `object` takes any `(&str, Value)` iterator, so it also replaces the inline `Value::Object([..].into())` forms.
7. `ModuleInfo.enabled` is deleted (R-13 above).
8. `modules::get(&Modules, id)` replaces `main.rs` `module_of`, the lookup in `not_enabled`, and `revalidate`'s lookup. Its unit test `get_by_id` takes over the assertions of `module_by_first_segment`.
9. `PreScan.json` replaces `main.rs` `json_mode`. It keeps the old semantics (any `--json` before `--`, a `--store` value included). The old unit test moved to core `pre_scan_reads_json_before_the_terminator`.
10. `support::run` is `run_timed(c).expect("no hang")`, so every test command shares one `Run` construction and the 10 s timeout.
11. Not applied: `jiff::tz::TimeZone` has no `Default` impl in jiff 0.2.39, so `SimClock` (which derives `Default`) keeps `RefCell<Option<TimeZone>>`.
12. Subset enumeration (R-6).
13. `main.rs`: the `"arbitre.test.signal"` comparison is compiled only with `test-actions`.
14. The `support/network.rs` module doc is one line pointing to `CONTRIBUTING.md` § Network clients.

Also fixed: `tests/network.rs` `connect` sent its request with `write!`, which writes in fragments. In cut mode the proxy closes after the request line, so a later fragment could get `ECONNRESET` (seen once in a full run). It now sends with one `write_all`, and 30 repeated runs passed.

Checks on `st-04/fixes`: `cargo test --workspace --all-features`, 318 passed (312, minus 3 removed or merged, plus 9 new). `fmt` passes. Both clippy runs pass. `20-st01-browser.sh` and `30-st01-graph.sh` pass. Mutation `-f` on the 11 Std files plus `core/src/write.rs` (Full) and `cli/src/cli/mod.rs` (Std): 317 mutants, 250 caught, 67 unviable, **0 missed**. `core/src/merge.rs` (Full), `--re RecordState::text`: 3/3 caught. No new equivalent.

Not done: RG-ST04-4.3 "exit 0 only with a named degraded mode" is still not asserted. No field name for a degraded mode is specified yet, and no ST-04 action can return 0 while declaring a network kind.
