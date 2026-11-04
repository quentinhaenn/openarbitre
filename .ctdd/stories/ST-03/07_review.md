# Code Review — ST-03: Action catalogue, derived CLI, exit codes

## Verdict: APPROVED WITH CHANGES

- Reviewed: `git diff origin/st-02/store...HEAD -- crates/ README.md CONTRIBUTING.md Cargo.toml` (29 files, +3872/−34; ST-03 sits on the unmerged ST-02 branch), at `6894d8e`.
- Against: `01_story.md`, `02_specs.md` v2, `04_plan.md` v2 and its arbitrations, `05_da_review.md`, `06_skeleton.md`, `zones.toml`, `CONTRIBUTING.md`.
- Tests: `cargo test --workspace --all-features --locked`: green, 265 tests passed in 23 suites (467 s, under load from other sessions).
- Extra check, run by hand: a build **without** `test-actions` (the shipped binary). See R-1 and R-5.
- Date: 2026-11-04 · Phase 8, zone Supporting.

## Summary

The implementation follows the plan closely. The core holds the frozen contract and stays dependency-free: `Value`, a hand-written canonical encoder, `Category`, the result document, the descriptor and the catalogue checks. The binary is a thin shell over it, with the plan's two hard parts done right: a single checked write to stdout (DA F-5) and `catch_unwind` around the whole pipeline. Every DA finding is addressed. Three findings must be fixed before the PR:
- the shipped binary crashes on a bare `arbitre` (exit 3 instead of 2, R-1);
- actions are routed to the wrong module when two wirings share a module id, which ST-04 will trigger (R-2);
- the mutation measurement that Q-P2 required at the end of T-4 was never made (R-3).

The rest is minor, plus a short ponytail list. The largest cut is `leak()`, which clap's `string` feature replaces.

## Conformance to the specs

| Rule | Status | Notes |
|---|---|---|
| RG-ST03-1.1 malformed ⇒ 2 (INV-ST03-1) | ⚠️ | Covered with `test-actions` (`usage.rs`, 14 defects × 2 modes × 2 store states, plus rows 1, 2, 6, 8, 9, 15). **Broken in the shipped build**: a bare `arbitre` or `arbitre --json` exits 3 `internal_error` (R-1) |
| RG-ST03-1.2 schema checks are usage | ✅ | `parse_text` in the value parser; dates, bounds, enum, length covered in core and in binary tests |
| RG-ST03-1.3 store never opened | ✅ | Store opened in `execute` only, after the parse; absent path stays absent; file bytes and reads unchanged |
| RG-ST03-1.4 help/version 0 | ✅ | Human mode prints clap's text as is; JSON mode gives `{"help"\|"version": …}` |
| RG-ST03-1.5 not-enabled beats usage | — | ST-04 (frozen rule, no ST-03 mechanism) |
| RG-ST03-2.1 one document (INV-ST03-4) | ✅ | `document` golden bytes; every binary test parses exactly one line; optional failure `data` (Q-DA1) |
| RG-ST03-2.2 human streams | ✅ | `finish`: `error: <msg>` then `render(data)` on stderr |
| RG-ST03-2.3 rendering from `Value` (INV-ST03-2) | ✅ | `render(&Value)` only; leaf property in core (1,000 cases); fixed binary cases |
| RG-ST03-2.4 canonical JSON (INV-ST03-5) | ✅ | Round trip through `serde_json`, re-encoding, escape goldens |
| RG-ST03-3.1, 3.2 codes (INV-ST03-3) | ✅ | `codes.rs`: five categories × two modes, document `code` = exit code |
| RG-ST03-3.3 ids | ✅ | `store.rs` maps every `StoreError`/`Cause`/`Rejected`; `no_default_store`, `catalog_invalid`, `output_mismatch`, `internal_error`, `output_failed` emitted |
| RG-ST03-3.4 action `Usage` = contract breach | ✅ | `settle` → `internal_error`, unit-tested |
| RG-ST03-3.5 crash (INV-ST03-6) | ✅ | `test panic` ± `--json` → 3 `internal_error` |
| RG-ST03-3.6 nothing changes (PROP-ST03-1) | ✅ | `--fail refusal\|tool-error\|panic` × modes, store reads unchanged |
| RG-ST03-3.7 published table | ✅ | `docs.rs` parses the README table |
| RG-ST03-4.1 to 4.3 descriptor, names, subset | ✅ | |
| RG-ST03-4.4, 4.5 conformance (INV-ST03-8) | ✅ | |
| RG-ST03-4.6 catalogue refusals (INV-ST03-7) | ⚠️ | Every bullet tested **except** "module id `arbitre` outside the foundation": `Catalog::build` cannot tell (skeleton decision 7). See R-4 and Q-R1 |
| RG-ST03-4.7 derived CLI | ✅ | Sub-commands sorted, long options only, global `--json`/`--store` |
| RG-ST03-5.1 to 5.4 location (PROP-ST03-2) | ✅ | All platform branches unit-tested on any OS (P-13); `location.rs` rows 1–6 |
| RG-ST03-G.1 offline | ✅ | No network crate added |
| INV-ST03-9 core dependency-free | ✅ | `crates/core/Cargo.toml` gains a dev-dependency only |

## Conformance to the plan

| Task | Status | Notes |
|---|---|---|
| T-1 Skeleton | ✅ | Deviations recorded in `06_skeleton.md` |
| T-2 value, error | ✅ | Render layout `path: leaf\n` (dotted paths, list items repeat the path): the layout is free under RG-ST03-2.3 |
| T-3 catalog | ✅ | Extra refusals (duplicate property, non-object top-level schema, ill-formed enum value) all derive from RG-ST03-4.3 |
| T-4 store | ⚠️ | Code and tests conform. **The Q-P2 mutation measurement is missing** (R-3) |
| T-5 modules, test module | ⚠️ | `#[allow(unused_mut)]` in `all()` (acceptable, see S-6); routing flaw shared with T-6 (R-2) |
| T-6 cli, main | ⚠️ | `leak()` instead of clap's `string` feature (R-6, S-1); `selected` and `wiring_of` added (reasonable, see R-2); R-1 |
| T-7 README, glossary | ⚠️ | Error id example and "ids are stable" claim (R-7) |
| T-8 | — | Not started. It must carry R-3 and the PR body items (F-7 CI gap, Q-P1 to Q-P4, Q-DA1/2) |
| Amendment `Value` → `FieldValue` | ✅ | Mechanical rename across core, sqlite and tests; glossary updated. One leftover: the `write.rs:23` doc (R-9) |

## Failles DA adressées

| Finding | Status | Notes |
|---|---|---|
| F-1 [B] failure `data` | ✅ | `ActionError::with_data`, document and stderr rendering, `test refuse` |
| F-2 [M] signals | ✅ | Decision recorded (AMB-ST03-2), no code |
| F-3 [M] binary test cost | ✅ | Enumerated E-ST03-1, fixed E-ST03-2, no `proptest` in `arbitre`. The measurement half is R-3 |
| F-4 [M] `cfg` in Std | ✅ | `location(…, os)`, no `cfg` in `store.rs` |
| F-5 [M] `output_failed` | ✅ | `write_all` + `flush`, closed-pipe test in both modes |
| F-6 [m] row 9 | ✅ | Uses `test read` |
| F-7 [m] shipped catalogue | ⚠️ | `shipped_catalogue_builds` exists, but the shipped **binary** still crashes (R-1) and warns (R-5). A catalogue-only test cannot see either |
| F-8 [m] `Output` | ✅ | `pub type Output = Value;` |

## Findings

| Id | Severity | File:line | Finding | Proposed fix |
|---|---|---|---|---|
| R-1 | **MAJOR** | `crates/cli/src/cli/mod.rs:70-74`, `:116-120` | In the shipped build (empty catalogue) the root has no children, so `node_command` sets no `subcommand_required`. A bare `arbitre` (or `arbitre --json`) parses, then `selected` hits `expect("clap only matches derived paths")`, and the crash maps to exit 3 `internal_error`. Reproduced on a `cargo build -p arbitre` binary. This breaks RG-ST03-1.1 ("bare `arbitre` ⇒ 2") in the only build users get. ST-04 adds `arbitre.module.*`, which hides it without fixing it | Make every node without an action require a sub-command: `(None, _) => c.subcommand_required(true)` before the `(_, true)` arm. Add a unit test in `cli/mod.rs` (no feature gate) that runs `command(&catalog(&[]))` on `["arbitre"]` and on `["arbitre", "--json"]` and expects a `Usage` error |
| R-2 | **MAJOR** | `crates/cli/src/main.rs:59-64`, `crates/core/src/catalog.rs:192-213` | `wiring_of` routes an action to the **first** wiring whose id equals its first segment. `Catalog::build` accepts two wirings with the same module id. The test module already registers id `arbitre`, and ST-04 will add the real foundation wiring with the same id. With `test-actions` on, every `arbitre.*` action would then go to whichever wiring is listed first, and the test executor answers any unknown name with `Ok({})` (`wiring/test.rs:217`): a silent success | Refuse a module id given twice in `Catalog::build` (one `BTreeSet` insert, one test), so the clash fails at catalogue build. ST-04 then decides where `arbitre.test-foundation` lives (inside its foundation wiring under `cfg`). Alternative: keep the declaring module per action in `Catalog` and route on it. Larger, and the `Catalog` type would change anyway at ST-04 |
| R-3 | **MAJOR** | process (plan T-4, Q-P2; T-8) | No trace of the Q-P2 measurement (`cargo mutants --all-features -f crates/cli/src/store.rs`, per-mutant time × ≈100 mutants, stop above 30 min). Not in the commits, not in `.ctdd/`. T-6 went ahead without that checkpoint. The ≥ 70% Std threshold on 7 files is still unverified | Before opening the PR: run the measurement, record it (T-8 PR body and a note in `state.json`), then run `cargo mutants --in-diff` on the Std files and kill or justify each survivor in `mutants_equivalents.toml`. If it is above 30 min, Quentin decides (Q-P2) |
| R-4 | MINOR (deferral to record) | `crates/core/src/catalog.rs:189-213`; specs RG-ST03-4.6 bullet 3 | The refusal "module id `arbitre` outside the foundation" is neither implemented nor tested. As written, this is an uncovered part of INV-ST03-7. It cannot be implemented in ST-03, because nothing distinguishes the foundation from any wiring that declares id `arbitre` (the test module does exactly that) | Record the deferral explicitly as an ST-04 input (ST-04 story/specs: "`Module` carries foundation status; `Catalog::build` refuses `arbitre` from a non-foundation module"), and add a line to the T-8 PR body. See Q-R1 |
| R-5 | MINOR | `crates/cli/src/modules.rs:13-18`, `crates/cli/src/store.rs:80, 90` | The shipped build compiles with three `dead_code` warnings (`Context` fields, `map_apply`, `block_on`). With `-D warnings` it would not build. This is the CI gap R-6/F-7, now concrete | Accept until ST-05 brings a real module that uses them, and state it in the T-8 PR body. Or `#[cfg_attr(not(feature = "test-actions"), expect(dead_code))]` on the three items (removed at ST-05) |
| R-6 | MINOR | `crates/cli/src/cli/mod.rs:15-18` (+8 call sites) | `leak()` turns every name, summary and description into `&'static str` because clap's `string` feature is off. It is harmless today (one command tree per process), but ST-07's MCP server is long-lived, and any rebuild of the tree there leaks each time. The function also exists only to work around a feature switch | `clap = { version = "4.6.7", features = ["string"] }` (no new crate, so `Cargo.lock` is unchanged; it is a feature of a direct dependency, not a new shared dependency, so not an amendment) and pass `String`s. `leak` is deleted (S-1) |
| R-7 | MINOR | `CONTRIBUTING.md` glossary row "identifiant d'erreur"; `README.md` "Exit codes" | The glossary's example id `unknown_argument` (copied from the specs) does not exist: the code emits `unknown_option`, `unknown_command`, `missing_option`, `missing_command`, `repeated_option`, `invalid_value`, `usage`. The README says "Error ids are stable", but RG-ST03-G.2 only makes the RG-ST03-3.3 ids non-removable. The usage ids are therefore published as stable without having been arbitrated | Fix the glossary example to `unknown_option`. For the README wording, see Q-R2 |
| R-8 | MINOR | `crates/cli/src/cli/mod.rs:166` | `ArgumentConflict` → `repeated_option`. clap also uses that kind when an action-group gets options together with a sub-command (`args_conflicts_with_subcommands`, Q-B), which would get a misleading id | Rename to `conflicting_option`, or check the action-group case and leave the id as it is if clap reports it differently. Ids are not frozen, so this costs nothing now |
| R-9 | NIT | `crates/core/src/write.rs:23`; `crates/core/src/lib.rs:1` | Leftover from the rename: "FieldValue serialized by the module". The crate doc still lists only "stamps, hybrid clock, writes, merge" | "Field value serialized by the module; …"; add "action catalogue" to the crate doc |
| R-10 | NIT | `crates/core/src/catalog.rs:347` | `Value::Int(*n as i64)` on a `u64` length wraps above `i64::MAX` (negative `maxLength` in the JSON Schema). It cannot happen in practice | `i64::try_from(*n).unwrap_or(i64::MAX)`, or type the lengths `u32` before ST-04 freezes `Schema` |

### Deviations reported by the agents: assessment

| Deviation | Assessment |
|---|---|
| T-6 `leak()` instead of clap `string` | Works, but replace it: R-6 / S-1 |
| T-6 added `selected` and `wiring_of` | Reasonable: they are the minimum glue between clap matches and the seam. `selected` must not be reachable on a bare root (R-1). `wiring_of` needs unique module ids (R-2) |
| Human help/version printed by clap's text | Correct: RG-ST03-1.4 asks for "the text on standard output". `!e.use_stderr()` is true only for `DisplayHelp`/`DisplayVersion`, so `DisplayHelpOnMissing…` still goes to usage (exit 2). Covered by `human_help_is_the_text_json_help_is_a_document` and `output.rs` |
| Usage ids per clap `ErrorKind` | Matches the specs' latitude (R-3 of the plan: not frozen, `usage` as fallback). Naming drift vs the glossary: R-7; one ambiguous mapping: R-8 |
| Integers accept `+5` and negatives | Negatives are required (`i64`, `minimum` may be negative; `allow_negative_numbers` is needed so clap does not read `-1` as a flag). `+5` comes from `i64::from_str`: lenient input, canonical output (`5`), and RG-ST03-4.4 only says "decimal integer". **Accept**; no test pins it either way |
| T-2 render layout (`path: leaf`, trailing newline) | Conforms: the layout is explicitly not frozen; the property (leaf on a line with its key) holds. Side effect: an empty list or empty object shows nothing in human mode. That is consistent with "every leaf" (no leaf) and acceptable for ST-03. ST-05 may want `(none)` for an empty listing, a rendering change that needs no amendment |
| T-3 extra catalogue refusals | Accept: duplicate property names, non-object top-level schemas and ill-formed enum values are all implied by RG-ST03-4.3 (object form, enum value pattern). They tighten catalogue validation, not the frozen contract |
| Test action ids outside the frozen list (`refused`, `failed`, `module_not_enabled`) | Accept: test-only, never shipped; RG-ST03-G.2 only freezes the RG-ST03-3.3 list against removal. `module_not_enabled` is a good name for ST-04 to reuse |
| `#[allow(unused_mut)]` in `modules::all()` | Accept. It is the honest cost of a `cfg`-gated `extend`. The skeleton note "no `allow` added" is now stale, which is cosmetic. Alternative without the attribute: S-6 |
| "Module id `arbitre` outside the foundation" untested | Not implementable in ST-03; to be recorded as an ST-04 obligation (R-4, Q-R1) |
| `Value` → `FieldValue` rename | Clean and mechanical; arbitrated (plan, "Value name clash → (c)"). Only the doc leftover R-9. `pub mod value` + a selective re-export is now unneeded (S-5) |

## C-TDD metrics (Supporting zone)

| Metric | Value | Threshold | Status |
|---|---|---|---|
| Tests passing | 265 / 265 | 100% | ✅ |
| Mutation score, Std files (`value.rs`, `error.rs`, `catalog.rs`, `main.rs`, `modules.rs`, `store.rs`, `cli/**`) | not measured | ≥ 70% | ❌ R-3 |
| `arbitre-core` normal dependencies | 0 | 0 | ✅ |

## Simplification (ponytail)

Over-engineering is low: no parser, no trait, no macro, three core files. What remains are small cuts.

- S-1 `crates/cli/src/cli/mod.rs:L15-18` + 8 calls: native: `leak()` helper. Use clap's `string` feature and pass `String`/`&String` directly (R-6). About −6 lines, one concept fewer.
- S-2 `crates/cli/src/cli/mod.rs:L124-135`: shrink: `values()` special-cases enums because `PossibleValuesParser` yields `String`. `PossibleValuesParser::new(…).map(Value::Str)` (clap `TypedValueParser::map`) makes every option yield `Value`, so `values` becomes one `get_many::<Value>` line. About −8 lines.
- S-3 `crates/cli/src/cli/mod.rs:L91-104`: shrink: nested `match` in `scalar`. Use one `match s { Enum => …, Integer => …value_name("INTEGER").allow_negative_numbers(true), Date => …value_name("DATE"), _ => a }`, then a single `value_parser(parse_text)` for the non-enum arms. About −3 lines, one nesting level fewer.
- S-4 `crates/cli/src/cli/mod.rs:L155-178`: shrink: `usage_error` computes `key` in an outer `match` and `return`s from inside the inner one. Flatten it: `DisplayHelp => return Ok(doc("help", text))`, `DisplayVersion => …`, then one `let id = match kind {…}`. This is the only production function with avoidable cognitive load. About −4 lines.
- S-5 `crates/core/src/lib.rs:L19,30`: shrink: `pub mod value` + `pub use value::{Output, Value, encode, render}` survives from the name clash. Use `mod value; pub use value::*;` like every other module. Callers' `arbitre_core::value::Value` (5 files) become `arbitre_core::Value`.
- S-6 `crates/cli/src/modules.rs:L34-37` (optional): `let all = shipped(); #[cfg(feature = "test-actions")] let all = [all, test::wirings()].concat();` needs `Wiring: Clone`. Otherwise keep the `allow`. Same size, so only worth it if the `allow` bothers you.
- S-7 `crates/core/src/catalog.rs:L326-335` (optional): `conforms_input` and `conforms_output` are the same one-line wrapper over `check`. Keep both names only if ST-04 is expected to make them diverge; otherwise one `conforms`. −5 lines. Low value: the two names document intent at call sites.
- S-8 `crates/cli/src/modules.rs:L58-63`: shrink: the `let _ = c;` exists to silence a variable unused without the feature. Use `assert!(cfg!(not(feature = "test-actions")) || c.actions().iter().any(…))`. −1 line, no `cfg` attribute.

Duplication check: `block_on` (`store.rs`) and `ready` (`tests/support/mod.rs`) are the same single poll, as are the two `tmp()` helpers. Integration tests cannot reach a binary crate's private items, so both copies are justified. No copy-pasted production logic was found. The two `for (k, v) in [(…), (…)]` loops in `schema_json` are already the loop form.

Cognitive complexity: `check` (`catalog.rs:276-324`) and `check_schema` are long but flat matches, one arm per schema type. They are fine as they are, and splitting them would hurt. `node_command`, `location` and `finish` read well. Only `usage_error` (S-4) earns a flattening.

net: about −30 lines possible (S-1 to S-5, S-8), plus the optional S-6 and S-7.

## Questions for Quentin

**Q-R1 — The "module id `arbitre` outside the foundation" refusal (RG-ST03-4.6) cannot be implemented in ST-03.**
- (a) Defer it to ST-04 explicitly: one line in ST-04's inputs ("`Module` carries foundation status; the catalogue refuses `arbitre` from a non-foundation module, with a test"), and one line in the T-8 PR body. ST-03 merges without it.
- (b) Implement it now with a `foundation: bool` on `Wiring`, passed to `Catalog::build`. This changes `Catalog::build`'s signature one story before ST-04 replaces the seam anyway.
- **Recommendation: (a).** The concept of "foundation" only exists with ST-04's `Module`. Today the only `arbitre` wiring is the test module's, so nothing ships that the rule would catch.

**Q-R2 — Which error ids does the README call stable?**
- (a) Narrow the README to the spec: "The ids listed below never disappear; others may change". List the RG-ST03-3.3 ids. Usage ids stay free to change.
- (b) Add the seven usage ids (`unknown_option`, `unknown_command`, `missing_option`, `missing_command`, `repeated_option`/`conflicting_option`, `invalid_value`, `usage`) to the RG-ST03-3.3 list now, and keep "Error ids are stable".
- **Recommendation: (a).** Scripts need the category (exit 2) for usage errors, not the sub-kind. Freezing ids that come from clap's `ErrorKind` mapping ties the contract to a dependency's classification. (b) is reasonable only if you foresee automation that branches on the kind of usage error.

## Must fix before the PR (T-8)

1. R-1: root without an action requires a sub-command, plus a no-feature unit test.
2. R-2: refuse a duplicate module id at catalogue build, plus a test.
3. R-3: the mutation measurement and the Std ≥ 70% run, survivors killed or justified.
4. R-4: record the deferral (per Q-R1).

The rest (R-5 to R-10, S-1 to S-8) is recommended. R-6/S-1 and S-2 are the most worthwhile.
