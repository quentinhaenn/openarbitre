# Functional Specifications — ST-04: Modules you can enable, kept apart, with one generic conformance test

## Traceability
- Source: `.ctdd/stories/ST-04/01_story.md`.
- Frame: `.ctdd/cadre/02_specs.md` RG-1.1, 1.2, 3.2, 3.7, 5.1 to 5.4, INV-11, 15, 20–23, PROP-1, PROP-2, CONFLIT-1, CONFLIT-6, AMB-12 (arbitrated 2026-11-02: per-device enabled state, data always synced); `.ctdd/cadre/03_archi.md` ("Interfaces à définir": `Module`, `Contexte`, `Port Ia`, `Clock::fuseau`; "Activation des modules"; "Validation après fusion"; D-2); `.ctdd/cadre/04_plan.md` § ST-04, F-12, F-18, F-27, F-37, § Collisions.
- ST-03 (binding, PR #8): RG-ST03-1.3, 1.4, 1.5, 3.3, 3.6, 4.2, 4.6, 5.2, G.2; AMB-ST03-2 with Q-DA2 (a); plan P-3, P-4, P-11; "ST-04 handover" (Q-R1 (a)).
- ST-02: `Store` port (`async fn`, no `Send`), `StoreError`, `SqliteStore::open` (creates if absent), `sim_store` (feature `sim`).
- Date: 2026-11-05 · Status: v2. Q1 to Q7 answered (a) on 2026-11-05 (§ Resolution); DA iteration 1 folded (§ DA iteration 1 → specs v2, which overrides the rules it names).
- Codebase context (branch `st-03/story`, read 2026-11-05): `arbitre-core` is `#![no_std]` + `alloc`, no normal dependency; `Clock` has `now()` only; `crates/cli/src/modules.rs` holds the provisional `Wiring { id, descriptors, execute }` and a `Context { store: &SqliteStore, pt, author }`; `wiring/test.rs` declares module `test` and, under id `arbitre`, `arbitre.test-foundation`; `test.not-enabled` returns `ModuleNotEnabled` by hand; futures are driven by one poll on a no-op waker; `crates/cli/tests/support/mod.rs` spawns the binary.

### Conventions
- `US-n`, `AC-n.m`: the stories and criteria **of ST-04**. Frame references are prefixed (`frame RG-5.3`); ST-03 rules are quoted as they are (`RG-ST03-1.3`).
- Identifiers created here: `RG-ST04-x.y`, `INV-ST04-n`, `PROP-ST04-n`, `E-ST04-n`, `CONFLICT-ST04-n`, `AMB-ST04-n`, `GAP-ST04-n`.
- `[ARB]` derives from an arbitration; `[HYP]` from a hypothesis of the requirements; `[DERIVED]` is needed for an AC to hold; `[Qn]` depends on open question Qn, written with its recommended answer.
- Phase 3 is skipped (supporting zone). **Architecture notes** carry what the planner needs.

### Completeness analysis
The requirements are complete enough to specify, given answers to Q1 to Q7. Three points shape everything else: where the per-device enabled state lives (Q1), the Rust shape of the frozen `Module` contract (Q3), and how the network observer sees traffic (Q5). The rest is settled below as defaults (§ Defaults settled here).

## Domain glossary

Terms of `CONTRIBUTING.md` keep their meaning. New terms (to add to the glossary):

| Term | Definition |
|---|---|
| Module | A set of actions with its own data namespace. Implements `Module`. Identified by its module id, the first segment of its action names. |
| Foundation module | The module with id `arbitre`. Always enabled. Owns `arbitre.module.*` and signals. Not listed in `modules.rs`. |
| Toggleable module | Every module of the build other than the foundation. |
| Enabled state | The set of toggleable modules enabled on this device. Not synchronised (frame AMB-12). |
| Namespace | The `module` part of a `RecordId`. A module's namespace is its id. |
| Module store view | `ModuleStore`: the store as one module sees it. Every record it names is in that module's namespace. |
| `Context` | What an action runs with: author, clock, module store view, optional `Ai` port. Built by the surface. |
| `Problem` | One finding of `revalidate`: the record, a stable `snake_case` id, an English message. |
| Signal | A dated note left in the foundation namespace by a background task, shown once at the next command on the same device. |
| Fake module | Module compiled only with feature `test-actions`, used by the conformance test. Two instances, `fake-a` and `fake-b`. Never shipped. |
| Surface | A way to reach actions: the CLI now, MCP (ST-07), GUI (ST-21). |
| Surface registry | `crates/cli/tests/support/surfaces/`: one file per surface and one registration line each. |
| Network observer | Test tooling that records every outgoing connection of the binary under test. |
| Cut mode | Observer mode in which every outgoing connection fails. |
| `Ai` port | Core trait for an AI provider: one request, one answer, four errors. Declared here; adapter in ST-30. |
| Conformance test | The generic test of § SPEC-US-5, run on every module of the build and every registered surface. |

## Specifications per User Story

### SPEC-US-1: Enable only the modules I use
**Ref:** US-1; frame RG-5.1, 5.2, AMB-12; archi D-2

**RG-ST04-1.1: Three foundation actions** `@Invariant` [DERIVED AC-1.1, Q7]
- **Actions** (module `arbitre`):
  - `arbitre.module.enable`, mutating, input `module` (enum of the toggleable module ids of the build), output `{module, enabled: true, changed, problems}` (`problems`: Q4);
  - `arbitre.module.disable`, mutating, same input, output `{module, enabled: false, changed}`;
  - `arbitre.module.list`, not mutating, no input, output `{modules: [{id, enabled, actions}]}` where `actions` lists the descriptors (name, summary, input and output JSON Schema, `mutating`, `logged_as`, `requires_network`) of an **enabled** module, and is empty for a disabled one.
- **CLI form** [Q7]: `arbitre module enable --module diff` (ST-03 derivation unchanged).
- None of them needs the network (`requires_network = ∅`).

**RG-ST04-1.2: Idempotent** `@Invariant` [AC-1.2]
- **Post-conditions:** enabling an enabled module, or disabling a disabled one, succeeds with `changed: false` and writes nothing.
- **Formal expression:** `enable(m) ; enable(m) ≡ enable(m)`; `changed ⇔ state before ≠ state after`.

**RG-ST04-1.3: Disabling keeps the data** `@Invariant` [frame RG-5.2, INV-21]
- **Formal expression:** `data(m)` after `disable(m) ; enable(m)` equals `data(m)` before. `disable` writes no record of namespace `m`; the store never filters by enabled state.

**RG-ST04-1.4: Any subset works alone** `@Invariant` [frame RG-5.1, INV-20]
- **Formal expression:** `∀ E ⊆ toggleable modules, enabled = E ⇒ ∀ m ∈ E, ∀ action a of m, outcome(a) does not depend on E \ {m}`.

**RG-ST04-1.5: Per-device enabled state** `@Invariant` [ARB AMB-12, Q1]
- **Post-conditions:** the enabled state of device D is read and written only by D. Sync may carry it, but no device reads another's (Q1 (a)).
- **Storage** [Q1 (a)]: one record per device in the foundation namespace, `arbitre/module-state/<device-id>`, one field per toggleable module id. Written by `enable`/`disable` in one local apply.

**RG-ST04-1.6: Default enabled state** `@Invariant` [Q2]
- **Post-conditions** [Q2 (a)]: a device with no enabled-state record has **no** toggleable module enabled, in every build. ST-03's test actions become foundation actions `arbitre.test.*` under `test-actions`, with unchanged command paths (`arbitre test echo`), so they are always on and outside the conformance test (GAP-ST04-3, DA F-1). The fake modules start disabled like real ones.
- A field value other than `1`/`0` gives the default; a field naming a module that is not in the build is ignored (DA F-15).

**RG-ST04-1.7: The foundation is always on** `@Invariant` [AC-1.7]
- `arbitre` is not in the `module` enum: `arbitre module disable --module arbitre` is a usage error (exit 2). Same for any id not in the build.

#### Case matrix — US-1

| # | Case | Type | Input | State | Expected result | Rule |
|---|---|---|---|---|---|---|
| 1 | Enable | Nominal | `module enable --module fake-a` | default | exit 0, `changed: true`; `fake-a` actions in help | 1.1, 1.5 |
| 2 | Enable twice | Edge | same again | `fake-a` on | exit 0, `changed: false`; store reads identical | 1.2 |
| 3 | Disable keeps data | Nominal | create in `fake-a`, disable, enable, read | — | same data | 1.3 |
| 4 | Every subset | Nominal | each subset of {`fake-a`, `fake-b`} | — | each enabled module's CRUD scenario succeeds | 1.4 |
| 5 | List | Nominal | `module list --json` | `fake-a` on | every module named with its state; actions only for `fake-a` | 1.1 |
| 6 | Unknown module | Error | `module enable --module nope` | — | exit 2; store not created | 1.7 |
| 7 | Disable foundation | Error | `module disable --module arbitre` | — | exit 2 | 1.7 |
| 8 | Fresh device | Nominal | `--help` | store absent | only foundation commands (`module`, plus `test` in test builds); store still absent | 1.6, 2.4 |
| 9 | CI flow (D-2) | Nominal | `ARBITRE_STORE=<tmp>`, enable `fake-a`, run a `fake-a` action | — | both exit 0 | 1.1, 1.6 |
| 10 | Other device's state | Edge | a write of device B's state record applied to A | A: `fake-a` off | A still sees `fake-a` off | 1.5 |

### SPEC-US-2: A disabled module stays out of sight, and says so when called
**Ref:** US-2; frame RG-3.2, 3.7, 5.4, CONFLIT-6, INV-11, 15, 23; RG-ST03-1.5

**RG-ST04-2.1: Invisible when disabled** `@Invariant` [frame RG-5.4, INV-23]
- **Formal expression:** `¬enabled(m) ⇒ ∀ surface s, actions(s) ∩ actions(m) = ∅ ∧ m ∉ help(s)`. The catalogue each surface derives from contains the foundation and the enabled modules only.
- `module list` names `m` and its state, with no action (US-1 edge, `[HYP]`).

**RG-ST04-2.2: Calling a disabled module** `@Invariant` [frame RG-3.7, CONFLIT-6, INV-15]
- **Pre-conditions:** the first command word (the first argument that is neither an option nor the value of `--store`) equals the id of a toggleable module that is not enabled.
- **Post-conditions:** exit 4, error id `module_not_enabled`, message naming `arbitre module enable --module <m>`; one result document in JSON mode; the store is not created and not written.
- **Precedence** [RG-ST03-1.5]: decided before parsing, so it wins over any usage error of that command.

**RG-ST04-2.3: Reading the enabled state never changes the store** `@Invariant` [AMB-ST03-2, CONFLICT-ST04-1]
- **Post-conditions:** to build the help or decide RG-ST04-2.2, the binary opens the store **only if the store file already exists**, and only reads. Absent file ⇒ default state (RG-ST04-1.6). Help, usage errors and "module not enabled" never create the store and leave its data identical (`writes_since(START)` and `device` unchanged).
- **Error cases:** an existing store that cannot be read ⇒ exit 3 with the RG-ST03-3.3 id, help included (CONFLICT-ST04-2); `--version` still exits 0.

**RG-ST04-2.4: Entry point = module id** `@Invariant` [frame RG-3.2, INV-11 form]
- **Formal expression:** the command path of every action of module `m` starts with `m` (RG-ST03-4.2). Checked on `fake-a`; `plan` and `diff` inherit it in ST-05 and ST-22.

#### Case matrix — US-2

| # | Case | Type | Input | State | Expected | Rule |
|---|---|---|---|---|---|---|
| 1 | Hidden in help | Nominal | `--help`, `fake-a --help` | `fake-a` off | `fake-a` absent from root help; `fake-a --help` exits 4 | 2.1, 2.2 |
| 2 | Explicit call | Error | `fake-a put --key k --value v` | off, store with data | exit 4, `module_not_enabled`; store reads identical | 2.2 |
| 3 | Beats usage | Error | `fake-a put --bogus` | off | exit 4, not 2 | 2.2 |
| 4 | JSON mode | Error | row 2 + `--json` | off | one document, category `module_not_enabled`, code 4 | 2.2 |
| 5 | No store yet | Edge | rows 1–3 | store absent | store still absent | 2.3 |
| 6 | Option before the word | Edge | `--json --store <p> fake-a put …` | off | exit 4 | 2.2 |
| 7 | Damaged store | Error | `--help` | store file is not a store | exit 3, `storage_corrupted`; `--version` exits 0 | 2.3 |
| 8 | Enabled again | Nominal | enable, then row 2 | on | exit 0 | 2.1 |

### SPEC-US-3: A module plugs in through one frozen contract and only sees its own data
**Ref:** US-3; frame RG-5.3, INV-22; archi "Trait Module", "Contexte", "Port Ia"; ST-03 handover

**RG-ST04-3.1: The `Module` contract** `@Invariant` [archi, Q3]
- A module gives: its id; its action descriptors; `execute(action name, conforming input, context) → Output or ActionError`; `revalidate(records, view) → problems or StoreError` (US-7).
- `execute` and `revalidate` are asynchronous, since the store is (PL-8).
- Shape [Q3 (a)]: `trait Module<S: Store>`, dyn-compatible, the two asynchronous methods returning a boxed local future (`Pin<Box<dyn Future<Output = …> + 'a>>`, `alloc` only). The registry holds `Box<dyn Module<S>>`. The rules of ST-03 still apply around `execute`: input conforms before the call, output is checked after it, `Usage` from an action is `internal_error` (RG-ST03-3.4, 4.4, 4.5).

**RG-ST04-3.2: The `Context`** `@Invariant` [archi "Contexte"]
- **Fields:** `author` (`Author`, set by the surface: `CliUser` for the CLI); `clock` (`&dyn Clock`); `store` (the module store view of the action's module); `ai` (`Option<&dyn Ai>`, `None` on every surface until ST-30).
- No other field. In particular the context gives no access to the whole `Store`.

**RG-ST04-3.3: The module store view** `@Invariant` [frame RG-5.3, INV-22]
- **Formal expression:** every read and write of `ModuleStore` for module `m` takes a `(category, id)` and targets `RecordId { module: m, category, id }`. There is no parameter that names a module. A record of another namespace cannot be named.
- **Operations:** the projected state of a record, the records of a category, the writes of a record, the device id, and one atomic local apply of a list of `(category, id, change)` (author and physical time from the context). No `writes_since`, no watermark: those belong to sync and export.
- **Error cases:** storage errors as in ST-02, mapped by RG-ST03-3.3.

**RG-ST04-3.4: The foundation owns `arbitre`** `@Invariant` [ST-03 handover, RG-ST03-4.6 bullet 3]
- **Formal expression:** the catalogue is built from the foundation module plus the list of `modules.rs`. A module of that list with id `arbitre` is refused at catalogue build: exit 3, `catalog_invalid`, message naming it. Only the foundation's view reaches namespace `arbitre`.
- `arbitre.test-foundation` (ST-03) moves into the foundation module, under feature `test-actions` (GAP-ST04-2).

**RG-ST04-3.5: The `Ai` port, declared** `@Invariant` [archi "Port Ia", F-32]
- **Signature:** one operation, `complete(request: Value, timeout in ms) → Value or AiError`, asynchronous, same future shape as RG-ST04-3.1.
- **`AiError`**: `NotConfigured`, `Unreachable`, `KeyRefused`, `Timeout`. Each maps to `ToolError` with ids `ai_not_configured`, `ai_unreachable`, `ai_key_refused`, `ai_timeout` (added to the RG-ST03-3.3 list).
- No adapter and no consent logic in ST-04 (ST-30).

**RG-ST04-3.6: The seam is replaced, nothing else moves** `@Invariant` [AC-3.1, AC-3.6]
- `Wiring` and the CLI-side `Context` disappear. `ActionDescriptor`, `Value`, `Output`, `Category`, `ActionError`, `document`, `encode`, `render` are unchanged. Adding a module is one line in `modules.rs` and one file `wiring/<m>.rs`.

#### Case matrix — US-3

| # | Case | Type | Input / state | Expected | Rule |
|---|---|---|---|---|---|
| 1 | Same key, two modules | Nominal | `fake-a put --key k --value 1`, `fake-b put --key k --value 2` | `fake-a get --key k` → 1; `fake-b get` → 2 | 3.3 |
| 2 | Other module's data invisible | Nominal | data in `fake-b` only | `fake-a list` → empty | 3.3 |
| 3 | Foundation id stolen | Error | catalogue with a listed module of id `arbitre` | build refused, `catalog_invalid` | 3.4 |
| 4 | No AI | Nominal | any action | `ctx.ai` is `None` | 3.2, 3.5 |
| 5 | Core still builds for the browser | Nominal | `20-st01-browser.sh` | green | AC-3.7 |

### SPEC-US-4: Everything works offline, and nothing leaves for an undeclared destination
**Ref:** US-4; frame RG-1.1, 1.2, PROP-1, PROP-2; plan F-18, F-37

**RG-ST04-4.1: The observer** `@Property` [AC-4.1, AC-4.2, Q5]
- [Q5 (a)] A recording proxy in `crates/cli/tests/support/`, started per test on a local port. The binary under test is spawned with `HTTP_PROXY`, `HTTPS_PROXY` and `ALL_PROXY` pointing at it and `NO_PROXY` empty. Each connection attempt is recorded as `(host, port, argv of the command)`. In cut mode the proxy refuses every attempt (connection closed, nothing forwarded). In open mode it forwards.
- Every later network client must honour these variables. The rule is written in `CONTRIBUTING.md`: no network client may disable the proxy settings, and the story of each adapter replays PROP-2 with an assertion that its traffic **was** recorded (so a bypass is caught there).
- The observer tests itself: a plain client in the test sends a request through it and checks it is recorded (open) or refused (cut).

**RG-ST04-4.2: No-network actions run with the network cut** `@Property` [frame PROP-1, AC-1.1, F-12, Q6]
- **Formal expression** [Q6 (a)]: `∀ module m, ∀ action a of m with requires_network(a) = ∅`, run in cut mode with a minimal conforming input (RG-ST04-5.3), on a store where `m` is enabled: exit ∈ {0, 1}, the process ends within 10 s, no panic, and the observer records **no** attempt.
- Plus, on the fake module: the full scenario put → get → list → update → delete → get succeeds in cut mode (frame AC-1.1 on real data operations).

**RG-ST04-4.3: Network actions fail explicitly when cut** `@Property` [F-37]
- **Formal expression:** `∀ a with requires_network(a) ≠ ∅`, in cut mode: exit ∈ {0, 1, 3} with an error id or a named degraded-mode field in the output, within 10 s, no panic. Exit 0 is allowed only for a declared degraded mode (the output says which).

**RG-ST04-4.4: Only declared destinations** `@Property` [frame PROP-2, RG-1.2]
- **Formal expression:** in open mode, every recorded attempt of an action `a` targets a host the test configured for one of the kinds in `requires_network(a)`. An action with `∅` records none.
- In ST-04 no action connects: the check exists for the stories that add adapters (ST-10, 15b, 17, 19, 30, 31, 32).

#### Case matrix — US-4

| # | Case | Type | Input | Mode | Expected | Rule |
|---|---|---|---|---|---|---|
| 1 | Observer records | Nominal | test client through the proxy | open | one record, right host | 4.1 |
| 2 | Observer cuts | Nominal | same | cut | refused, nothing forwarded | 4.1 |
| 3 | Fake CRUD offline | Nominal | scenario of 4.2 | cut | every step exit 0, zero records | 4.2 |
| 4 | Every no-network action | Nominal | each `∅` action of the build | cut | exit ∈ {0, 1}, zero records, < 10 s | 4.2 |
| 5 | Fake relay action | Error | `fake-a push` (declared `relay`) | cut | exit 3, `relay_not_configured`, < 10 s | 4.3 |
| 6 | Foundation actions | Nominal | `module enable/disable/list` | cut | exit 0, zero records | 4.2 |

### SPEC-US-5: One conformance test that every module and every surface replays
**Ref:** US-5; plan F-12, F-27, § Collisions "shared test tooling"

**RG-ST04-5.1: Discovery from the binary** `@Invariant` [AC-5.1]
- The test asks the binary itself: `arbitre module list --json` gives the module ids; enabling each module and listing again gives its descriptors. No list of modules or actions is written in the test.

**RG-ST04-5.2: What it checks, per module** `@Invariant` [AC-5.2]
For every toggleable module `m` of the build (test build, `--all-features`):
1. enable `m`: exit 0; its actions appear on every surface (RG-ST04-2.1);
2. disable `m`: its actions disappear from every surface; each action called explicitly exits 4 and changes nothing (RG-ST04-2.2);
3. data kept: every mutating action of `m` run once, then disable and enable; the store's `m` namespace is identical (RG-ST04-1.3);
4. isolation: running every action of `m` writes nothing outside namespace `m` (compared on `writes_since(START)` from a test-side store handle) (RG-ST04-3.3);
5. subsets: for each subset of the toggleable modules, every action of each enabled module behaves as with all enabled (same exit code, same data) (RG-ST04-1.4). If the build has more than 6 toggleable modules, pairs only `[DERIVED]` (bound on 2ⁿ).
6. network: RG-ST04-4.2 and 4.3;
7. parity: for each registered surface, the set of exposed actions equals the catalogue of enabled modules, and the same call on a fresh store gives the same exit code and the same result document (RG-ST04-5.4).

**RG-ST04-5.3: Minimal conforming input** `@Invariant` [DERIVED, Q6]
- **Formal expression:** for an input schema, the input with only the required properties, each set to: the first enum value; `minimum` (else 0) for an integer; a string of `max(minLength, 1)` letters `a`; `2000-01-01` for a date. Booleans `false`, lists empty, optional properties absent. It conforms by construction (RG-ST03-4.4).

**RG-ST04-5.4: The surface registry** `@Invariant` [F-27]
- `crates/cli/tests/support/surfaces/` holds one file per surface and a list with one registration line per surface. A surface gives: its name; the set of action names it exposes for a given store; and a call (action name, input `Value`) returning exit code and result document.
- ST-04 adds `cli.rs` (spawns the binary; the input becomes options by the RG-ST03-4.3 table). ST-07 adds `mcp.rs` and its line; ST-21 adds `gui.rs` and its line. Nothing else changes.

**RG-ST04-5.5: The fake module** `@Invariant` [AC-5.4]
- Compiled only with `test-actions`, in `wiring/fake.rs`. One implementation, two instances with ids `fake-a` and `fake-b`. Records in category `entry`, field `value`.
- **Actions** (same for each instance): `put` (mutating; `key`, `value`), `get` (`key`; refusal `not_found` if absent), `list`, `update` (mutating; refusal `not_found`), `delete` (mutating, `logged_as = Delete`; refusal `not_found`), `push` (`requires_network = {Relay}`; no relay is ever configured in ST-04: `ToolError`/`relay_not_configured`, Q5 (a)).
- **`revalidate`**: a record whose `value` is the text `invalid` gives the problem `invalid_value`; any other value gives none.

**RG-ST04-5.6: Frozen tooling** `@Invariant` [AC-5.5, § Collisions]
- After ST-04, `tests/support/` changes only by adding a file under `surfaces/` and its registration line. The observer, cut mode, fake module and conformance test are part of the foundation; changing them is a foundation amendment.

#### Case matrix — US-5

| # | Case | Type | Input | Expected | Rule |
|---|---|---|---|---|---|
| 1 | Whole test on the build | Nominal | `cargo test --all-features` | green on `test`, `fake-a`, `fake-b` | 5.2 |
| 2 | New module joins | Nominal | a third fake instance added to `modules.rs` only (checked once by hand at review, not committed) | covered with no test edit | 5.1 |
| 3 | Minimal input | Nominal | each subset type | conforms; accepted by the CLI | 5.3 |
| 4 | Module without actions | Edge | `fake-a` filtered to zero actions (unit level) | per-action checks empty, enable/disable/list pass | 5.2 |

### SPEC-US-6: A problem met in the background shows up at my next command
**Ref:** US-6; plan F-27; ST-03 Q-DA2 (a)

**RG-ST04-6.1: Leaving a signal** `@Invariant` [AC-6.1, AC-6.4]
- A function of the binary, `signals::deposit(store, now, source, message)`, writes one signal record in the foundation namespace: `arbitre/signal/<device>-<ms>-<source>`, fields `at` (ms), `source` (`snake_case`), `message`, `read` (absent until read). Two deposits with the same device, millisecond and source are one signal.
- ST-11 and ST-19 call it from their own files; `main.rs` is not touched.

**RG-ST04-6.2: Showing signals** `@Invariant` [AC-6.2, Q-DA2 (a)]
- **Pre-conditions:** a well-formed action is about to run (after the store is opened).
- **Post-conditions:** every unread signal **of this device** is written to standard error, oldest first, one line each: `signal (<local date and time>): <message>`, the time in `Clock::timezone()`. Then they are marked read in one local apply. Nothing of this appears in the JSON result document (outside frame INV-13).
- Not shown for help, usage errors or "module not enabled".

**RG-ST04-6.3: Once** `@Property` [AC-6.3]
- **Formal expression:** with sequential commands, each signal is shown exactly once. Under a race between two commands, at least once and at most twice `[HYP]`.
- **Simulator needed:** a test action `test.signal --source --message` that deposits from a separate process.

#### Case matrix — US-6

| # | Case | Type | Input | Expected | Rule |
|---|---|---|---|---|---|
| 1 | Shown once | Nominal | process 1: `test signal`; then `fake-a list` twice | first: signal on stderr; second: none | 6.2, 6.3 |
| 2 | JSON mode | Edge | row 1 with `--json` | stdout is one document without the signal; signal on stderr | 6.2 |
| 3 | Not on help | Edge | `test signal`, then `--help`, then an action | help shows nothing; the action shows it | 6.2 |
| 4 | Other device | Edge | signal record of device B applied to A | not shown on A | 6.2 |
| 5 | Order | Edge | two signals | oldest first | 6.2 |

### SPEC-US-7: Merged data is re-checked by its module
**Ref:** US-7; frame CONFLIT-1; archi "Validation après fusion"; plan R-7, F-7

**RG-ST04-7.1: The hook** `@Invariant` [AC-7.1, Q4]
- **Formal expression:** `revalidate(records, view) → Ok(problems) | Err(StoreError)`. `records` are records of the module's namespace (as `(category, id)`); the hook reads through the view and writes nothing. `problems` lists, for each invalid record, a `Problem`.
- **`Problem`** [Q4 (a)]: `{ record: RecordId, id: snake_case, message }`. Frozen.

**RG-ST04-7.2: First caller: enabling** `@Invariant` [archi, Q4 (a)]
- **Post-conditions:** `module enable` of a module that was disabled calls `revalidate` on **all** records of that module and returns the problems in its output (`problems`, a list of `{category, id, problem, message}`). Exit 0 even with problems: they are reported, not refused. ST-05 adds marking, ST-13 the question, ST-04b and ST-19 call the hook on the records a merge touched.

#### Case matrix — US-7

| # | Case | Type | Input | Expected | Rule |
|---|---|---|---|---|---|
| 1 | Invalid record found | Nominal | `fake-a put --key k --value invalid`, disable, enable | output `problems` = one entry `invalid_value` for `k` | 7.1, 7.2 |
| 2 | Valid data | Nominal | values `x`, `y` | `problems: []` | 7.1 |
| 3 | Enable when already enabled | Edge | enable twice | second: `changed: false`, no revalidation, `problems: []` | 1.2, 7.2 |

### SPEC-US-8: Day limits follow my time zone, everywhere
**Ref:** US-8; ST-01 archi P-2; S1-A criterion (7)

**RG-ST04-8.1: `Clock::timezone()`** `@Invariant` [AC-8.1]
- `Clock` gains `timezone() → jiff::tz::TimeZone`. `SimClock` gains a setter (default UTC). The native clock of the binary returns the system time zone, UTC when it cannot be read `[HYP]`.

**RG-ST04-8.2: Embedded database, browser included** `@Invariant` [AC-8.2]
- `arbitre-core` depends on `jiff` (shared, `default-features = false`) with the bundled time zone database. `20-st01-browser.sh` stays green. A named zone (`Europe/Paris`) resolves in a core unit test with no system database.

#### Case matrix — US-8

| # | Case | Type | Input | Expected | Rule |
|---|---|---|---|---|---|
| 1 | Named zone in core | Nominal | `Europe/Paris`, 2026-03-29 01:30 UTC | offset +02:00 (after the change) | 8.2 |
| 2 | Simulator | Nominal | `SimClock` set to `America/New_York` | `timezone()` returns it | 8.1 |
| 3 | Browser build | Nominal | `20-st01-browser.sh` | green | 8.2 |

## Architecture notes (phase 3 folded in)

**Placement.**

| Path | Content | Zone |
|---|---|---|
| `crates/core/src/module.rs` | `Module`, `Context`, `ModuleStore`, `Problem`, the boxed-future alias, registry (foundation + modules, enabled filter, RG-ST04-3.4), "first command word" helper for RG-ST04-2.2 | Std |
| `crates/core/src/ai.rs` | `Ai` port, `AiError` | Std |
| `crates/core/src/clock.rs` | `timezone()` added; `SimClock` setter | Std (exists) |
| `crates/core/src/catalog.rs` | `Catalog::build` takes the foundation apart | Std (exists) |
| `crates/cli/src/modules.rs` | `Wiring` removed; `all()` returns `Box<dyn Module<SqliteStore>>`; default enabled list | Std |
| `crates/cli/src/wiring/arbitre.rs` | foundation module: `module.enable/disable/list`, `test-foundation` under `test-actions` | Std |
| `crates/cli/src/signals.rs` | `deposit`, show and mark read | Std |
| `crates/cli/src/clock.rs` | native `Clock` (system time, system time zone) | Std |
| `crates/cli/src/main.rs` | enabled state read before parsing (existing store only), RG-ST04-2.2 check, signals before the action | Std |
| `crates/cli/src/wiring/fake.rs` | fake module | Gen |
| `crates/cli/src/wiring/test.rs` | `test.not-enabled` removed, `test.signal` added | Gen (exists) |
| `crates/cli/tests/support/{network.rs, conformance.rs, surfaces/{mod.rs, cli.rs}}` | observer, generic test helpers, surface registry | tests |
| `crates/cli/tests/conformance.rs`, `modules.rs`, `signals.rs` | acceptance tests | tests |

**Executor.** ST-03's single poll on a no-op waker stays. The foundation, fake and test modules never return a pending future; a pending one is still `internal_error`. `tokio` arrives with ST-07.

**Store opening.** Before parsing: open only an existing store, read the device and its enabled-state record. After parsing, for an action: open (create if absent) as in ST-03, one handle per process. `RG-ST03-5.2`'s eager opening for actions is kept (AMB-ST04-1).

**`jiff` in a `no_std` core.** Features `alloc` and the bundled database; the plan checks that this feature set builds without `std` for `wasm32-unknown-unknown` (risk for the planner). The binary adds `tz-system` for the native clock.

**Proposed `zones.toml`** (written by the plan): `Std` for the core and CLI files above; `Gen` for `crates/cli/src/wiring/fake.rs`.

## Global rules

**RG-ST04-G.1: Fully offline** `@Invariant` [frame RG-1.1] — every action of ST-04 except `fake-*.push` declares `∅` and needs no network.

**RG-ST04-G.2: Frozen contract** [§ Collisions, RG-ST03-G.2] — at the end of ST-04 the Rust types `ActionDescriptor`, `Schema`, `Property`, `Value`, `Output`, `Category`, `ActionError`, `Module`, `Context`, `ModuleStore`, `Problem`, `Ai`, `AiError`, `Store`, `StoreError`, `Clock` are frozen, as are the tooling of RG-ST04-5.6 and the record layouts of RG-ST04-1.5 and 6.1. Changing them is a foundation amendment. Error ids may only be added to.

## Acceptance tests

All in `crates/cli/tests/` (spawning the binary, `--all-features`) unless noted; run by `10-st01-tests.sh`, no CI change.

| # | Test | Checks |
|---|---|---|
| E-ST04-1 | **Enable, disable, list**: matrix US-1 rows 1–10. | exit codes, `changed`, store reads, D-2 flow |
| E-ST04-2 | **Invisible and not enabled**: matrix US-2 rows 1–8, each ± `--json`. | help text, exit 4 beats 2, store absent or identical, damaged store |
| E-ST04-3 | **Isolation**: matrix US-3 rows 1–2; catalogue refusal row 3 (core unit test). | per-namespace data; `catalog_invalid` |
| E-ST04-4 | **Observer self-test**: matrix US-4 rows 1–2 (std TCP client, no new dependency). | recorded / refused |
| E-ST04-5 | **Generic conformance**: RG-ST04-5.2 items 1–7 over every module of the build and every registered surface, network cut for item 6. | as listed; < 10 s per command |
| E-ST04-6 | **Signals**: matrix US-6 rows 1–5. | once, stderr only, order, device filter |
| E-ST04-7 | **Revalidate**: matrix US-7 rows 1–3. | `problems` in the enable output |
| E-ST04-8 | **Time zone** (core unit tests, feature `sim`): matrix US-8 rows 1–2. | offsets, simulator |
| E-ST04-9 | **Module store view** (core unit tests on `SimStore`): every operation targets namespace `m`; a write of `fake-a` is invisible to the view of `fake-b`. | by construction and by reads |
| E-ST04-10 | **ST-03 suite unchanged**: E-ST03-1 to 10 still green, `test.not-enabled` row moved to a disabled fake module. | no regression |

## Identified dependencies

| Dependency | Type | Description | Impact |
|---|---|---|---|
| ST-03 (PR #8) | Internal | Catalogue, CLI derivation, codes, JSON shape, test module | Blocking (merge first; ST-04 branches from `st-03/story`) |
| ST-02 (PR #6) | Internal | `Store`, `SqliteStore`, `SimStore` | Blocking (via ST-03) |
| `jiff` 0.2.39 (shared) | External | Time zones with bundled database, in core | Blocking for US-8 |
| ST-04b, ST-05, ST-07, ST-11, ST-13, ST-19, ST-21, ST-22, ST-30 | Downstream | Use the frozen contract and tooling unchanged | — |

## Conflicts and ambiguities

| # | Rules | Description | Resolution |
|---|---|---|---|
| [CONFLICT-ST04-1] | RG-ST03-1.3 ("usage, help ⇒ store never opened") vs RG-ST04-2.1/2.2 (help and precedence need the enabled state) | Reading the enabled state means opening the store before parsing. | As foreseen by AMB-ST03-2: RG-ST03-1.3 becomes "never created, never written"; an existing store is opened read only (RG-ST04-2.3). |
| [CONFLICT-ST04-2] | RG-ST03-1.4 ("help always exits 0") vs a damaged store | The help cannot be built without the enabled state. | Help on an unreadable store exits 3 with the store id; `--version` still exits 0. |
| [CONFLICT-ST04-3] | frame RG-1.1 ("every operation succeeds offline") vs a generic test with synthesized inputs | A synthesized input may be refused by a business rule (`not_found`). | Q6. Default: outcome ∈ {success, refusal} with zero network attempts, plus a full success scenario on the fake module. |
| [CONFLICT-ST04-4] | archi D-2 (`arbitre module enable diff`, positional) vs ST-03 derivation (options only, frozen subset) | A positional argument is not in the frozen subset. | Q7. Default: `arbitre module enable --module diff`. |
| [CONFLICT-ST04-5] | plan § ST-04 ("signals at the start of every command") vs ST-03 Q-DA2 (a) | Q-DA2 restricts to well-formed actions, on stderr. | Q-DA2 (a) is binding (RG-ST04-6.2). |
| [AMB-ST04-1] | RG-ST03-5.2 note (lazy opening possible) vs archi D-2 (`diff check` reads no store) | Should an action that needs no store skip opening it? | Kept eager: the D-2 CI flow creates a throwaway store anyway when it enables `diff`. Revisit only if ST-22 measures a cost. |
| [AMB-ST04-2] | plan "proxy used by every HTTP client" | Process-level proxy or in-process port? | Q5. |
| [GAP-ST04-1] | `test.not-enabled` (ST-03) | A hand-made "module not enabled" is obsolete. | Removed; E-ST03-3 row 5 uses a disabled fake module. |
| [GAP-ST04-2] | `arbitre.test-foundation` (ST-03 handover) | It lives in the test module under id `arbitre`, which RG-ST04-3.4 now refuses. | Moved into the foundation module, under `test-actions`. |
| [GAP-ST04-3] | ST-03 tests vs a default with nothing enabled | ST-03 tests call `arbitre test …` on an absent store and expect usage (2), not 4. | **v2 (DA F-1)**: ST-03's test actions become foundation actions `arbitre.test.*` (same command paths), always on, never seen by the conformance test. No default exception in any build. |

## Candidate invariants (C-TDD synthesis)

### @Invariant (stateless, blocking)
- `INV-ST04-1` (frame INV-23): disabled ⇒ absent from help and from every surface's action list — RG-ST04-2.1
- `INV-ST04-2` (frame INV-15): explicit call of a disabled module ⇒ exit 4, before parsing, store not created nor written — RG-ST04-2.2, 2.3
- `INV-ST04-3` (frame INV-22): the module store view cannot name another namespace — RG-ST04-3.3
- `INV-ST04-4`: catalogue refuses id `arbitre` outside the foundation — RG-ST04-3.4
- `INV-ST04-5`: enable/disable idempotent; foundation always on — RG-ST04-1.2, 1.7
- `INV-ST04-6`: `arbitre-core` builds for `wasm32-unknown-unknown` with `jiff` as its only normal dependency — RG-ST04-8.2
- `INV-ST04-7`: the minimal input of RG-ST04-5.3 conforms to its schema — RG-ST04-5.3

### @Property (stateful, simulator needed)
- `PROP-ST04-1` (frame INV-21): disable then enable keeps the module's data — store with data — RG-ST04-1.3
- `PROP-ST04-2` (frame INV-20): every subset of modules works alone — RG-ST04-1.4
- `PROP-ST04-3` (frame PROP-1): no-network actions, network cut ⇒ exit ∈ {0, 1}, zero attempts — observer in cut mode — RG-ST04-4.2
- `PROP-ST04-4` (F-37): network actions, network cut ⇒ explicit outcome, < 10 s, no panic — RG-ST04-4.3
- `PROP-ST04-5` (frame PROP-2, tooling): attempts only towards declared kinds — observer in open mode — RG-ST04-4.4
- `PROP-ST04-6`: a signal from another process is shown once — separate process — RG-ST04-6.3
- `PROP-ST04-7`: per-device enabled state ignores other devices' records — applied foreign writes — RG-ST04-1.5

## Defaults settled here (no question)

- Foundation module id `arbitre`, passed to the catalogue apart from `modules.rs`; always enabled (RG-ST04-1.7, 3.4).
- `module list` names every module of the build and its state, actions only for enabled ones (`[HYP]`).
- Unknown module id ⇒ usage (exit 2), through the derived enum.
- Fake module: one implementation, two instances `fake-a` and `fake-b`; one `relay` action; no `ai` action (one declared-network action is enough for F-37).
- Signals per device, stderr, local time, at least once and at most twice under a race.
- `revalidate` reads only; it never marks (ST-05) nor asks (ST-13).
- Executor and eager store opening unchanged from ST-03.
- UTC when the system time zone cannot be read.
- ST-03's hand-written date parser stays (P-3).
- No new dependency: `jiff` is already shared; no `tokio`, no `reqwest` in ST-04 (if Q5 (a)).

## Open questions

Questions marked **[FREEZES]** fix part of the contract frozen at the end of ST-04; reopening them later is a foundation amendment.

**Q1 — Where does device-local state live (enabled modules, signals)? [FREEZES]**
- (a) In the store, foundation namespace, keyed by device: `arbitre/module-state/<device>` and `arbitre/signal/<device>-…`. Sync carries them; each device reads only its own.
- (b) In a small settings file next to the store (for example `arbitre.toml`). Never synced by construction, but a second storage format with its own concurrency and its own place in export (ST-04b).
- (c) In the store, in a local table outside the journal. Never synced, but it changes the `Store` port that ST-02 froze.
- **Recommendation: (a).** No new storage, atomic writes and concurrent processes come free from ST-02, and AMB-12 holds because no device reads another's record. The cost is a few bytes of harmless, encrypted sync traffic.

**Q2 — What is enabled on a fresh device?**
- (a) Nothing. `arbitre --help` shows only `module`; the first call to a module says how to enable it. CI enables `diff` first, as archi D-2 already writes.
- (b) Everything. A user disables what they do not want.
- **Recommendation: (a).** It is the frame's nominal case ("enables this one module and never sees planning") and the D-2 flow as arbitrated. The cost is one command on first use, which the "module not enabled" message spells out.

**Q3 — Rust shape of the `Module` contract (and of the `Ai` port). [FREEZES]**
- (a) `trait Module<S: Store>`, dyn-compatible: `execute` and `revalidate` return a boxed local future. The registry is a `Vec<Box<dyn Module<S>>>`; `Context` carries a concrete `ModuleStore<'_, S>`. `Ai` uses the same future shape.
- (b) `async fn` in the trait, like `Store`. Not dyn-compatible: the registry becomes a hand-written dispatch (one `match` arm per module in `modules.rs`), which breaks "one line per module".
- (c) Synchronous `execute`, with a synchronous store facade. Simplest, but it gives up the asynchronous store of PL-8, which the browser worker needs (S1-A).
- **Recommendation: (a).** One trait, one registration line per module, works in the browser; the price is one small allocation per call, nothing at this scale.

**Q4 — `revalidate`: problem shape and first caller. [FREEZES `Problem`]**
- (a) `Problem { record: RecordId, id, message }`. `module enable` calls the hook on all of the module's records and lists the problems in its output (archi: "on a device where the module is disabled, revalidation happens at activation"). ST-04b and ST-19 call it on the records a merge touched.
- (b) Same shape, but no caller in ST-04: the hook is only tested directly on the fake module; ST-04b is its first caller.
- **Recommendation: (a).** It implements the archi rule now, gives the hook a real caller and an end-to-end test, and costs one loop in `enable`.

**Q5 — How does the network observer see traffic? [FREEZES the test tooling]**
- (a) A recording proxy in the test tooling. The binary under test gets `HTTP(S)_PROXY`/`ALL_PROXY`; the proxy records or, in cut mode, refuses. No product code. The fake `relay` action makes no real call (`relay_not_configured`). Each adapter story must show its traffic **is** recorded, which catches a client that ignores the proxy.
- (b) Same proxy, but the fake `relay` action makes a real HTTP call through `reqwest` (test builds only), proving the whole path now. It brings `reqwest` and an async runtime into the test build two stories early.
- (c) An "egress" port in the core: every adapter gets its connections from it, and tests inject a recorder. Strongest, but a new frozen type that every network adapter must thread through.
- **Recommendation: (a).** Black-box, zero product code, and it works for any future client that keeps the standard proxy settings; the bypass risk is closed at the first adapter instead of being paid for now.

**Q6 — How strict is "works offline" in the generic test (frame PROP-1)?**
- (a) Each no-network action runs once with a minimal conforming input; it must end in success **or refusal** (exit 0 or 1), with zero connection attempts and no hang. Plus a full create-read-update-delete scenario on the fake module that must fully succeed.
- (b) Strict success for every action: each module provides a setup scenario so every action succeeds. That adds a method to the frozen `Module` trait that every module must write.
- **Recommendation: (a).** The property is about the network, not business rules: a refusal proves the action ran to its own rules without the network. Each module's own story proves its success paths.

**Q7 — Command form of `module enable` (archi D-2 writes `arbitre module enable diff`).**
- (a) `arbitre module enable --module diff`: the ST-03 derivation, unchanged. D-2 is read with this form.
- (b) A positional argument, `arbitre module enable diff`: needs a "positional" marker in the schema subset frozen by ST-03, so a foundation amendment now, used by later modules too.
- **Recommendation: (a).** One rule for every command; the help shows the option and its accepted values. A positional form can come later, for all actions at once, if users ask for it.

## DA iteration 1 → specs v2 (2026-11-05)

These lines override the rules they name. Detail and reasons: `05_da_review.md`; tasks: `04_plan.md` v2.

| Flaw | Change to the specs |
|---|---|
| F-1 [B] | RG-ST04-1.6, GAP-ST04-3: test actions are foundation actions `arbitre.test.*`; nothing enabled by default in any build. Matrix US-5 row 1 reads "green on `fake-a`, `fake-b`". |
| F-2 | RG-ST04-5.2 item 4: compare only the writes after the setup position. RG-ST04-6.3: the simulator is the foundation action `arbitre.test.signal`. Each conformance check starts from a fresh store with no signal. |
| F-3 | RG-ST04-1.1, 7.2: the foundation is not a `Module` impl; it holds the whole store. Records of a module = `writes_since(START)` filtered on the module, deleted ones left out. Revalidation runs before the state write; a `StoreError` exits 3 and changes nothing. |
| F-4 | RG-ST04-3.3: `ModuleStore` fields are private; one constructor; no getter returning the store. |
| F-5 | RG-ST04-2.3: before parsing, the store is opened only if its header is SQLite with application id `ARBT`. Absent or 0-byte file ⇒ default state. Other content ⇒ exit 3 `storage_corrupted`, file untouched. A location error before parsing ⇒ default state (the action still reports `no_default_store`). |
| F-6 | RG-ST04-2.2, 2.3: `--store` is read anywhere before `--`; the location is resolved once, before parsing, and the action uses it. |
| F-7 | RG-ST04-1.1: `module.enable` and `module.disable` exist only when the build has at least one toggleable module; `module.list` always. |
| F-8 | RG-ST04-3.4: the whole build is validated at every start, then filtered. A toggleable module id must have the name-segment form and must not equal a foundation command word (`module`, `test`). |
| F-9 | RG-ST04-5.2, 5.4: `Env { store, vars }`, `Called { code, doc }` (code from `doc.error.code` when the surface has none); `actions()` reads what the user sees (CLI: recursive `--help`; MCP: `tools/list`). Per surface: checks 1, 2 (visibility) and 7. CLI only: explicit call ⇒ 4, checks 3 to 6. |
| F-10 | RG-ST04-5.2 items 5, 7: comparison runs use byte copies of one prepared store and a pinned time (`ARBITRE_TEST_NOW`, read only with `test-actions`). |
| F-11 | RG-ST04-4.1: frozen observer API `Observer::start(mode)`, `env()`, `records()`. Cut mode is a proxy refusal, not a network cut; a client that ignores the proxy variables needs its own support before its story passes. |
| F-13 | Matrix US-6: one case with `TZ=America/New_York` and a pinned time checks the local time in the signal line. |
| F-15 | Matrix US-5 row 4 becomes a unit test of the conformance helpers. `jiff` 0.2 is part of the frozen contract (through `Clock::timezone`). |

## Resolution (Quentin, 2026-11-05)

Binding. Q1 to Q7: **(a)**, the recommendation in every case.
- **Q1 (a)**: device-local state (enabled modules, signals) lives in the store, foundation namespace, one record per device; no device reads another's.
- **Q2 (a)**: nothing enabled on a fresh device (test builds add `test`, GAP-ST04-3).
- **Q3 (a)**: `trait Module<S: Store>`, dyn-compatible, boxed local futures; `Ai` uses the same shape.
- **Q4 (a)**: `Problem { record, id, message }`; `module enable` revalidates the whole module and lists the problems.
- **Q5 (a)**: recording proxy through the proxy environment variables; no product code; the fake `relay` action makes no real call.
- **Q6 (a)**: no-network actions end in exit 0 or 1 with zero attempts, plus a full CRUD scenario on the fake module.
- **Q7 (a)**: `arbitre module enable --module <id>`; archi D-2 is read with this form.
