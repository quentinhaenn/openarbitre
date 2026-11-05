# Devil's Advocate Review — ST-03: Action catalogue, derived CLI, exit codes

- Reviewed: `.ctdd/stories/ST-03/04_plan.md` (DRAFT v1), against `01_story.md` (its "Quentin's arbitrations (2026-11-04)" are binding), `02_specs.md`, `zones.toml`.
- Code read (`origin/st-02/store`): `crates/core/src/store.rs` (`Store`, `StoreError`, `Cause`, `ApplyError`), `crates/sqlite/src/lib.rs` (`SqliteStore` = `RefCell<Connection>`, `open`, `Options`, `map`), `crates/cli/{Cargo.toml,src/main.rs}`, root `Cargo.toml` (no `panic = "abort"` profile), `scripts/ci.d/50-st01-mutation.sh`, `CONTRIBUTING.md` (Zones, Equivalent mutants).
- Frame read: `.ctdd/cadre/04_plan.md` § ST-03, ST-04, ST-07, ST-22; `.ctdd/cadre/02_specs.md` RG-3.3, 3.4, 3.5, RG-15.3.
- Iteration 1 of 1 (supporting zone) · Date: 2026-11-04.
- Pending questions taken at their recommendations: Q-B (a), Q-P1 (a), Q-P2 (a), Q-P3 (a), Q-P4 (a). See "Plan questions".

## Overall verdict
**VALIDATED WITH CHANGES.**
The plan is lean and mostly sound: no JSON parser, no `serde` in core, logic pushed into the pure crate, a single-poll executor that matches ST-02's synchronous futures. One flaw hits the core purpose of the story, which is to freeze a contract that later lanes will not amend. The frozen failure document has nowhere to put structured refusal details, and ST-22 provably needs that slot (F-1). F-1 and F-2 are contract choices: Quentin answers Q-DA1 and Q-DA2 **before the Skeleton**, because they change T-1's types and T-2's golden bytes. The other changes are plan edits that need no new DA iteration.

## Findings

### [B] F-1: The frozen failure document cannot carry structured refusal details, and ST-22 needs them
- **Axis**: Regressions (a later lane must amend a frozen contract) / Completeness
- **Source**: specs RG-ST03-2.1 (frozen failure shape `{"error":{category,code,id,message},"format":1,"ok":false}`), RG-ST03-2.2 (failure → `error: <message>` only), RG-ST03-G.2 (shape frozen at the end of ST-03); frame `02_specs.md` RG-15.3 ("échec ⇔ ∃ violation …; **chaque violation est rapportée avec : nom de l'invariant, description, fichier, ligne**"), RG-3.4 ("erreur → la sortie structurée porte aussi l'erreur" and the same information in both outputs); frame `04_plan.md` § ST-22 ("codes « violation » … pris dans la table de ST-03"); story US-3 nominal case (`arbitre diff check` exits 1 in CI).
- **Tasks**: T-1 (`ActionError` type), T-2 (`document`, golden bytes), T-6 (human rendering of a failure), T-7 (README shape).
- **Description**: `arbitre diff check` with violations is a **refusal** (exit 1), and the frame requires a per-violation report: name, description, file, line. Under the frozen shape, that report can only go into `message`, which is free text and explicitly not stable. ST-22 then has three outcomes, all bad. It can make scripts parse `message`, which breaks US-2 ("my scripts never parse human text"). It can return `ok:true` with exit 1, which breaks `code = 0 ⇔ Success` (RG-ST03-3.1) and the frozen shape. Or it can amend the shape (`format: 2`), which is a foundation amendment at the first lane that uses refusals seriously. The same gap hits any refusal that should name structured items (ST-05 "invalid transition" naming the item).
- **Proof**: RG-15.3 requires a structured list on a failing outcome. `ActionError { category, id, message }` and the RG-ST03-2.1 failure shape have no field that can hold a `Value`. Changing either after ST-03 is an amendment by RG-ST03-G.2.
- **Required plan change**: put **Q-DA1** to Quentin (below) before the Skeleton. Then update RG-ST03-2.1/2.2 in the specs, the `ActionError` type in T-1, `document` and its golden bytes in T-2, the human failure rendering in T-6, and the README shape in T-7.

### [M] F-2: ST-04's "signals at opening" have no place in the frozen JSON shape and contradict "store never opened"
- **Axis**: Regressions / Consistency
- **Source**: frame `04_plan.md` § ST-04 deliverable, "signaux à l'ouverture (F-27): `main.rs` affiche les signaux non lus **au début de toute commande**, puis les marque lus"; specs RG-ST03-1.3 (usage, help, version ⇒ store not opened), RG-ST03-2.1 (stdout = exactly one document), INV-ST03-2 / RG-3.4 (same information in both modes); specs AMB-ST03-2 (covers only "module not enabled", not signals).
- **Tasks**: none (the gap is that no task or note settles it); specs AMB-ST03-2.
- **Description**: ST-04 must show unread signals at the start of every command and mark them read. That means (1) reading **and writing** the store on every command, including malformed ones and `--help`, which RG-ST03-1.3 forbids, and (2) putting signals somewhere in JSON mode. The only frozen top-level keys are `data`, `error`, `format`, `ok`. If signals go to stderr, human mode shows information that JSON mode lacks (RG-3.4). A top-level `signals` key would be an amendment. AMB-ST03-2 anticipated the store conflict for "module not enabled" (read only, never create) but not for signals, which also **write**.
- **Required plan change**: put **Q-DA2** to Quentin. Record the answer in the specs (extend AMB-ST03-2) so that ST-04 inherits a decision and not an amendment. No ST-03 code change if the answer is (a).

### [M] F-3: Binary-level property tests make Std mutation of `crates/cli` costly, and the escalation point comes too late
- **Axis**: Over-engineering / Risk (R-2, Q-P2)
- **Source**: specs E-ST03-1 and E-ST03-2 (200 random cases each, spawning the binary), matrix US-3 row 8 (store busy "past the wait"; ST-02 `Options::default().wait` = 5 s, not configurable from the CLI); plan R-2, Q-P2 ("if the CI mutation step exceeds 30 min, stop and ask", measured in T-8); `50-st01-mutation.sh` (cargo-mutants runs the mutated package's tests, here every `crates/cli/tests/*`, and `mutants.toml` is forbidden, so test scope cannot be narrowed by config).
- **Tasks**: T-1 (red tests), T-4, T-6, T-8.
- **Description**: every mutant of `main.rs`, `cli/mod.rs`, `store.rs`, `modules.rs` reruns the whole `arbitre` test suite. E-ST03-1 is about 600 spawns (populated store setup, malformed run, read back), E-ST03-2 about 400, and the busy row waits at least 5 s. Roughly 20–60 s per mutant times about 100 mutants gives 30–100 min. Q-P2's 30-minute escape is measured only at T-8, after all the code exists. Two of these tests are also over-built for what they prove:
  - E-ST03-1's input space is finite and small: about 9 defect kinds × ±`--json` × 2 store states, so about 36 cases. Random sampling of 200 cases covers it less surely than enumeration does, at 5 times the cost.
  - E-ST03-2's leaf property is already a core property (T-2: 1,000 random `Value`s through `render`). At binary level, a few fixed cases prove the wiring. Matrix US-2 row 4's string covers the escapes.
  - Row 8 (busy) at binary level repeats ST-02's busy tests plus T-4's pure `Unavailable(Busy) → storage_busy` mapping test.
- **Required plan change**: (1) E-ST03-1 = exhaustive enumeration of defect kind × mode × store state, no proptest. (2) E-ST03-2 = fixed cases (matrix US-2 rows 1–8), with the random leaf property kept in core only. (3) Matrix US-3 row 8 = covered by T-4's mapping unit test, no binary-level busy test. (4) Drop `proptest` from `arbitre`'s dev-dependencies if nothing else uses it. (5) Move the Q-P2 measurement to the end of T-4: a local `cargo mutants --all-features -f crates/cli/src/store.rs`, per-mutant time × the estimated Std mutant count. Escalate then, not at T-8.

### [M] F-4: A `cfg(target_os = "macos")` branch in a Std file gives guaranteed surviving mutants on Linux CI
- **Axis**: Edge cases (mutation gate)
- **Source**: plan "Under-engineering detected" ("the macOS branch is tested only on macOS (`cfg(target_os)`), CI runs the Linux branch"); `zones.toml` (`crates/cli/src/store.rs` Std, 70%); `CONTRIBUTING.md` (only exclusion channel = `mutants_equivalents.toml`); cargo-mutants mutates source text, so it does not evaluate `cfg`.
- **Tasks**: T-4, T-8.
- **Description**: on Linux CI, a mutant inside the macOS-only code compiles and no test runs it, so it is reported **missed**. These are not equivalent mutants (the behaviour on macOS does change), so they cannot honestly go into `mutants_equivalents.toml`, and they lower `store.rs`'s score. The same holds for any `cfg(unix)`/other-platform fallback (`no_default_store` "on another platform").
- **Required plan change**: T-4 states that `location` takes the platform as a parameter (for example `std::env::consts::OS` passed in by the caller), with no `cfg` in `store.rs`. Unit tests then cover Linux (XDG absolute, XDG relative ignored, HOME fallback), macOS, an unknown platform and a missing `HOME` on every OS.

### [M] F-5: `output_failed` (a listed id) has no test, and the plan leaves room for an exit 101 or a silent 0
- **Axis**: Error cases
- **Source**: specs RG-ST03-3.3 (`output_failed` in the listed ids, which may only be added to), RG-ST03-3.1 ("no other exit code is produced by a normal return"), CONFLICT-ST03-3; plan "Under-engineering" (SIGPIPE), T-6 ("write once"), acceptance tests E-ST03-1 to 10 (none reaches `output_failed`).
- **Tasks**: T-6.
- **Description**: the plan relies on Rust ignoring SIGPIPE, but it does not fix how stdout is written. With `print!`/`println!`, a write error **panics**. The write happens after `catch_unwind` (the document is built inside, written once outside), so the panic is uncaught and the exit code is 101. If the code writes through a buffered handle without an explicit checked `flush`, `std::process::exit` flushes at exit and ignores the error, and the exit code is 0. Neither is caught by a test, so the mapping code in `main.rs` (Std) also yields surviving mutants. The `… | head -c1` scenario of CONFLICT-ST03-3 cannot serve as a test: a small document usually fits in the pipe buffer before `head` exits.
- **Required plan change**: T-6 states `write_all` + `flush` on a locked stdout, each error → exit 3 `output_failed` (with a stderr line), and that the store handle is dropped before `std::process::exit`. Add a deterministic case to `codes.rs`: spawn with stdout on a pipe whose read end is closed before the spawn, then expect exit 3 in human and JSON mode.

### [m] F-6: Matrix US-3 row 9 pins "every action opens the store", which ST-22 will want to undo
- **Axis**: Regressions (provisional behaviour frozen by a test)
- **Source**: specs RG-ST03-5.2 (store opened for every well-formed action), matrix US-3 row 9 (`test noop` on a damaged file → `storage_corrupted`); frame § ST-22 (`arbitre diff check` runs in CI and reads code and the invariants file, not the store).
- **Tasks**: T-6 (`codes.rs`).
- **Description**: with eager opening, `arbitre diff check` on a CI runner silently creates `~/.local/share/arbitre/arbitre.db`. Without `HOME` (some containers) it exits 3 `no_default_store` for a command that needs no data. RG-ST03-5.2 is not in G.2's frozen list, so this is not an amendment. But row 9 tests that a **pure** action opens the store, so changing the behaviour later means changing an ST-03 acceptance test.
- **Required plan change**: row 9 uses `test read` (an action that reads), and the plan notes that "opened before every action" is behaviour of the provisional seam (P-4), for ST-04's `Context` to revisit.

### [m] F-7: The shipped configuration (without `test-actions`) is neither compiled nor checked in CI
- **Axis**: Completeness / Regressions
- **Source**: specs RG-ST03-4.6 and E-ST03-7 ("the shipped catalogue builds with and without `test-actions`"); plan P-9, T-5 completion criterion; CI step `10` (`--all-features` only).
- **Tasks**: T-5, T-8.
- **Description**: every CI build turns `test-actions` on, so the "without" half of E-ST03-7 cannot run as written, and the release binary's feature set is never compiled. The catalogue is empty in ST-03, so the risk today is low. It grows at ST-05/ST-22, when real modules sit beside a `cfg`-gated test module.
- **Required plan change**: T-5's "without `test-actions`" check = a unit test that builds the catalogue from `all()` without the `test` wiring. State in the T-8 PR body that the no-feature compile is not covered by CI (a later `ci-change` PR, at Quentin's discretion).

### [m] F-8: `Output` (AC-4.4, glossary) has no type or definition in the plan
- **Axis**: Completeness (traceability)
- **Source**: story AC-4.4 ("the descriptor, `Output` and the error categories live in the core"); specs glossary (`Output` = success `Value` + derived rendering); plan T-1 type list (no `Output`).
- **Tasks**: T-1.
- **Description**: the plan rightly uses `Value` plus `render(&Value)`, but no artifact says that `Output` is `Value`. ST-04 freezes "the Rust types" (RG-ST03-G.2) and will look for it.
- **Required plan change**: T-1 states that `Output` is `Value` (a one-line doc on `Value`, or `pub type Output = Value;`). Pick one and say which.

## Axes with no finding
- **Completeness**: every RG-ST03, INV-ST03 and PROP-ST03 has a task and a test row, except F-5's id and F-8's name.
- **"Store never opened on usage errors"**: holds. Location resolution and `open` come after `try_get_matches_from`, and `--store` is only known after parsing, so there is no path to an early open. The absent-path case of E-ST03-1 is the real check: it kills a reorder mutant through file creation. The populated-store reads cannot see an open that writes nothing, which is acceptable.
- **Single-poll executor on a non-`Send` store**: sound. `SqliteStore`'s async methods await nothing (synchronous `rusqlite` and SQLite busy wait inside), so the first poll is ready. `RefCell<Connection>` makes it `!Sync`, which does not matter in a single-threaded CLI. ST-07 can drive the same `block_on` under its mutex on a blocking thread. `Waker::noop` is stable on 1.99. The seam's shape (`&SqliteStore`, sync `execute`) is explicitly ST-04's to replace and freezes nothing.
- **Panic / exit codes**: sound apart from F-5. There is no `panic = "abort"` profile, `catch_unwind` covers catalogue build through the conformance check, the document is built in memory, and `--fail panic` acts before `apply_local` (a panic inside a transaction is ST-02's all or nothing).
- **Security**: no input reaches a shell or SQL; JSON is only emitted; the JSON pre-scan works on `OsStr` and stops at `--`.
- **Over-engineering** (beyond F-3): none. Three core files, no parser, no `jiff`, no macro, no `Module` trait. The four constraint keywords are required by Q-3 (a).
- **Regressions on existing code**: the 3-line `main.rs` is replaced, `clap`'s `derive` is dropped (unused), there is no ST-02 change, and the `arbitre → arbitre-sqlite` edge is already allowed by `graph.py`. No zone is lowered (`wiring/test.rs` stays Gen through ST-01's glob, the rest goes up to Std).

## Plan questions (DA position)
| # | DA position |
|---|---|
| Q-B | **Agree (a).** Path collisions between a foundation action and a module action-group are still refused by RG-ST03-4.6's same-path rule. Cost: one clap setting, one test action |
| Q-P1 | **Agree (a).** An encoder of about 60 lines, and canonical bytes that stay independent of `serde_json` feature unification. ST-07's `Value` ↔ `serde_json` conversion is its own edge work |
| Q-P2 | **Agree (a)**, with F-3's changes. The measurement moves to the end of T-4 |
| Q-P3 | **Agree (a).** `object` is unavoidable, and the four keywords are what make Q-3 (a)'s "range" and "length limit" usage errors |
| Q-P4 | **Agree (a)**, with F-5's test. Note that `output_failed` is rare in practice (small documents fit in the pipe buffer) |

## New questions for Quentin (before the Skeleton)

**Q-DA1 — Structured details on a failure. [FREEZES the JSON shape] (F-1)**
- (a) The failure document gets an optional `data` key, present only when the action gives one: `{"data":<Value>,"error":{…},"format":1,"ok":false}`. `ActionError` gains an optional `Value`. In human mode it is rendered on stderr after the `error:` line. ST-22 puts its violations there; ST-05 can name the refused item.
- (b) Keep the shape. Failure details live only in `message`; ST-22 amends the shape (`format: 2`) when it needs to.
- **DA recommendation: (a).** One optional field now, purely additive and with no cost to actions that do not use it. (b) is a foundation amendment that has already been scheduled by RG-15.3.

**Q-DA2 — ST-04's "signals at opening" (F-27) vs the frozen shape and "store never opened". (F-2)**
- (a) Signals are shown on **stderr** in both modes. They are not part of the command's result, so INV-ST03-2/RG-3.4 do not apply to them. They are shown only for a well-formed action, after the store is opened, so "every command" means "every action" and RG-ST03-1.3 stands. No ST-03 code change.
- (b) Reserve a top-level optional `signals` key in the frozen shape now; ST-04 fills it.
- **DA recommendation: (a).** Zero code and no freeze impact. If an MCP assistant later needs signals, an action (`arbitre signals`) can return them as data.

## Recommendation
Quentin answers Q-DA1 and Q-DA2. The Planner folds the answers and F-3 to F-8 into `04_plan.md` (v2) and the matching spec lines (RG-ST03-2.1/2.2, matrix US-2/US-3 rows, E-ST03-1/2, AMB-ST03-2). The Skeleton follows with no second DA iteration (supporting zone, one iteration max). The changes stay inside these findings.
