# Functional Specifications — ST-03: Action catalogue, derived CLI, exit codes

## Traceability
- Source: `.ctdd/stories/ST-03/01_story.md`. Its section "Quentin's arbitrations (2026-11-04)" is binding: Q-1 (a), Q-2 (a), Q-3 (a), Q-4 (a), Q-5 (a), Q-6 (b), Q-7 (a), Q-8 (a) (confirmed 2026-11-04: a set of what the action cannot complete without), Q-9 (a).
- Frame: `.ctdd/cadre/02_specs.md` RG-3.2 to 3.5, RG-3.7, RG-15.6, INV-12, 13, 14, AMB-16, CONFLIT-6; `.ctdd/cadre/03_archi.md` ("Action catalogue", `DescripteurAction`, `Sortie`, "error category → exit code", target tree); `.ctdd/cadre/04_plan.md` § ST-03, § ST-04, F-6, F-11, F-20, F-37, F-38, § Collisions.
- Spike S5 (accepted): MCP through `rmcp` 3.8.0 (`local` feature); tools built at runtime with `Tool::new(name, description, json_schema)`; MCP tool name = action path with `.` → `_`; calls serialised behind a mutex. MCP itself is ST-07.
- ST-02 deliverable (PR #6, `st-02/store`, open at the time of writing): `arbitre_core::Store`, `StoreError` (`StorageFull`, `Corrupted(Damage)`, `Unavailable(Cause)`), `ApplyError`, `arbitre_sqlite::SqliteStore::open(path, &Options)` (creates the store if absent).
- Date: 2026-11-04 · Status: DRAFT
- Codebase context (read 2026-11-04): `arbitre-core` is `#![no_std]` + `alloc` with **no normal dependency**; `arbitre` (`crates/cli`) has only `clap` 4.6.7 (`derive`) and a 3-line `main.rs`. `serde`, `serde_json`, `tokio` are declared shared in `[workspace.dependencies]` but used by no crate yet.

### Conventions
- `US-n`, `AC-n.m`: the stories and criteria **of ST-03**. Frame references are prefixed (`frame RG-3.3`). ST-02 identifiers are quoted as they are.
- Identifiers created here: `RG-ST03-x.y`, `INV-ST03-n`, `PROP-ST03-n`, `E-ST03-n` (acceptance test), `CONFLICT-ST03-n`, `AMB-ST03-n`, `GAP-ST03-n`.
- `[ARB]` derives from an arbitration; `[HYP]` from a hypothesis of the requirements; `[DERIVED]` is not written in the requirements but needed for an AC to hold.
- Phase 3 is skipped (supporting zone). The section **Architecture notes** carries what the planner needs; it is the only part that names code placement.

### Completeness analysis
The requirements are complete enough to specify. Points to settle are listed under "Conflicts and ambiguities": the browser-safe core versus a JSON library (`CONFLICT-ST03-1`), the frame's "invalid date = refusal" versus Q-3 (`CONFLICT-ST03-2`), a missing `object` type in the Q-7 subset (`GAP-ST03-1`), and an action path that is also a group (`AMB-ST03-1`, question Q-B).

## Domain glossary

Terms of `CONTRIBUTING.md` keep their meaning. New terms (to add to the glossary):

| Term | Definition |
|---|---|
| Action | One operation a module offers, identified by its name. Every surface exposes actions and nothing else. |
| `ActionDescriptor` | What an action declares about itself: name, summary, input schema, output schema, `mutating`, `logged_as`, `requires_network`. Frozen after ST-04. |
| Action name | Dot-separated path of one or more segments, the first being the module id (`plan.add`, `arbitre.module.enable`). |
| Command path | Words typed after `arbitre`: the name's segments, with the reserved first segment `arbitre` dropped (`arbitre.export` → `arbitre export`). |
| Group | A command path that is a strict prefix of at least one action's command path (`plan` for `plan.add`). |
| `Catalog` | The validated set of descriptors of every wired module. Built once per process; every surface is derived from it. |
| Schema | The description of an input or an output, in the subset of JSON Schema fixed by RG-ST03-4.3. |
| Property | A named field of an object schema; in an input, one command-line option. |
| `Value` | A structured value: boolean, integer, string, list, object. No null, no floating-point number. |
| Canonical JSON | The single byte encoding of a `Value` fixed by RG-ST03-2.4. |
| `Output` | The success result of an action: a `Value` conforming to the output schema, plus its derived human rendering. |
| `Category` | The kind of an outcome: `Success`, `Refusal`, `Usage`, `ToolError`, `ModuleNotEnabled`. Each has one exit code. |
| `ActionError` | A failure: a non-success category, a stable error id and an English message. |
| Error id | Stable `snake_case` identifier of a failure (`unknown_argument`, `storage_busy`). Scripts may rely on it; the message is not stable. |
| Result document | The single JSON document written on standard output in JSON mode, for every outcome. |
| JSON mode | A command run with `--json`. Human mode otherwise. |
| Store location | Path of the store file the command opens (RG-ST03-5.1). |
| Test module | Module `test`, compiled only with the `test-actions` feature, whose actions exercise this story's rules. Never shipped. |

## Specifications per User Story

### SPEC-US-1: A malformed command explains itself and changes nothing
**Ref:** US-1; frame RG-3.3, INV-12, CONFLIT-6; Q-3 (a)

**RG-ST03-1.1: Malformed command ⇒ usage** `@Invariant` [frame RG-3.3]
- **AC ref:** [US-1/AC-1.1], [US-1/AC-1.4]
- **Pre-conditions:** a valid catalogue.
- **Post-conditions:** the command exits with the usage code (2); in human mode, standard error shows what was wrong and the usage of the closest command reached (the whole tool's usage when no action word matched).
- **Formal expression:** `malformed(argv) ⇒ exit = 2`, where `malformed(argv)` holds iff one of: unknown command word; unknown option; missing required property; option given a value that is not of its type (RG-ST03-4.4); non-repeatable option repeated; a group or the bare `arbitre` invoked with no action (unless the group is itself an action, `AMB-ST03-1`); argument not valid UTF-8.
- **Error cases:** this rule is the error case.

**RG-ST03-1.2: Usage is decided without data** `@Invariant` [ARB Q-3 (a)]
- **AC ref:** [US-1/AC-1.6]
- **Post-conditions:** every check that the input schema can express (type, format, enum membership, existing calendar date, integer bounds, string length) is a usage error, made before the action runs. An action receives only inputs that conform to its schema; anything it refuses afterwards (unknown item, invalid transition, business rule) is a **refusal**.
- **Formal expression:** `usage(c) ⇔ ¬conforms(input(c), schema(action(c))) ∨ malformed(argv)`; an action never returns `Usage` (RG-ST03-3.4).
- **Error cases:** —

**RG-ST03-1.3: Malformed ⇒ store never opened** `@Invariant` [HYP AC-1.3, DERIVED]
- **AC ref:** [US-1/AC-1.2], [US-1/AC-1.3]
- **Post-conditions:** a usage error, `--help`, `--version` and a catalogue error happen before the store is opened. The store location is not created, not opened, not read; an existing store is byte-for-byte unchanged.
- **Formal expression:** `exit ∈ {usage} ∨ help ∨ version ⇒ ¬opened(store)`. Stronger than "state unchanged", and it implies it.
- **Error cases:** —

**RG-ST03-1.4: Help and version succeed** `@Invariant` [DERIVED AC-1.5]
- **AC ref:** [US-1/AC-1.5], [US-2/AC-2.1]
- **Post-conditions:** `--help` (or `-h`) at any level and `--version` (or `-V`) exit 0. Human mode writes the text on standard output. JSON mode writes a success document whose data is `{"help": <text>}` or `{"version": <text>}`.
- **Error cases:** —

**RG-ST03-1.5: "Module not enabled" beats "usage"** `@Invariant` [HYP, frame CONFLIT-6] — rule frozen here, mechanism and test in ST-04
- **Post-conditions:** when the first command word names a known module that is not enabled, the outcome is `ModuleNotEnabled` (4), whatever else is wrong with the command.
- **Note:** see `AMB-ST03-2` (where ST-04 reads the enabled state without breaking RG-ST03-1.3).

#### Case matrix — US-1

| # | Case | Type | Input | State | Expected result | Rule |
|---|---|---|---|---|---|---|
| 1 | Unknown option | Error | `arbitre test echo --colour red` | — | usage text names `--colour`; exit 2 | 1.1 |
| 2 | Missing required | Error | `arbitre test echo` (no `--text`) | — | exit 2, usage of `test echo` | 1.1 |
| 3 | Integer not a number | Error | `--count abc` | — | exit 2 | 1.1, 1.2 |
| 4 | Date of right shape, non-existent | Error | `--due 2026-02-31` | — | exit 2 (usage, not refusal) | 1.2 |
| 5 | Date of wrong shape | Error | `--due 31/02` | — | exit 2 | 1.2 |
| 6 | Enum value outside the list | Error | `--level huge` | — | exit 2; help lists accepted values | 1.2 |
| 7 | Integer out of declared bounds | Error | `--count 1000` (max 100) | — | exit 2 | 1.2 |
| 8 | Unknown action | Error | `arbitre test nope` | — | exit 2 (not 3) | 1.1 |
| 9 | Bare group / bare tool | Error | `arbitre test`, `arbitre` | — | exit 2, list of sub-commands | 1.1 |
| 10 | Malformed, no store yet | Edge | any of 1–9 | store path absent | path still absent | 1.3 |
| 11 | Malformed mutating command | Edge | `arbitre test write --key k --bogus` | store with data | store unchanged (writes, device, clock) | 1.3 |
| 12 | Malformed with `--json` | Edge | case 1 + `--json` | — | one error document on stdout, exit 2 | 1.1, 2.3 |
| 13 | Help and version | Nominal | `--help`, `test echo -h`, `--version` | — | exit 0, text on stdout; no store opened | 1.4, 1.3 |
| 14 | Help with `--json` | Edge | `--help --json` | — | `{"format":1,"ok":true,"data":{"help":…}}`, exit 0 | 1.4 |
| 15 | Non-UTF-8 argument | Error | `--text $'\xff'` | — | exit 2 | 1.1 |
| 16 | Action refuses a well-formed value | Error | `arbitre test refuse` | — | exit 1 (refusal), not 2 | 1.2 |

### SPEC-US-2: Every command has a machine output equivalent to the human one
**Ref:** US-2; frame RG-3.4, INV-13; Q-4 (a), Q-9 (a)

**RG-ST03-2.1: One result document in JSON mode** `@Invariant` [ARB Q-4 (a)]
- **AC ref:** [US-2/AC-2.1], [US-2/AC-2.3], [US-2/AC-2.5]
- **Pre-conditions:** a `--json` token among the arguments, before any `--` terminator `[DERIVED]` (decided on the raw arguments, so that a parse failure is still reported in JSON).
- **Post-conditions:** standard output receives exactly one result document followed by one `\n`, and nothing else, for **every** outcome: success, each failure category, help, version, a caught crash (RG-ST03-3.5). Diagnostics, progress and panic messages go to standard error only.
- **Formal expression (frozen shape):**
  - success: `{"data":<Value>,"format":1,"ok":true}`
  - failure: `{"error":{"category":<c>,"code":<n>,"id":<id>,"message":<text>},"format":1,"ok":false}`
  - `c ∈ {"refusal","usage","tool_error","module_not_enabled"}`, `n` = exit code of `c` (RG-ST03-3.1), `id` matches `[a-z][a-z0-9_]*`.
  - Keys appear in canonical order (shown above). `format` is `1` until a foundation amendment.
- **Error cases:** standard output cannot be written (closed pipe) → exit 3; see `CONFLICT-ST03-3`.

**RG-ST03-2.2: Human mode streams** `@Invariant` [ARB Q-4 (a)]
- **Post-conditions:** success → the human rendering on standard output; failure → `error: <message>` (plus usage for a usage error) on standard error, nothing on standard output.

**RG-ST03-2.3: The human rendering is a function of the structured value** `@Invariant` + `@Property` [ARB Q-9 (a), frame INV-13]
- **AC ref:** [US-2/AC-2.2], [US-4/AC-4.3]
- **Post-conditions:**
  - construction: the human rendering takes the `Value` and nothing else (no store, no clock, no descriptor beyond the value), so it cannot show what the value does not carry;
  - property: every leaf of the value (string verbatim, integer in decimal, boolean as `true`/`false`) appears in the human text, on a line that carries its key;
  - the rendering is deterministic and walks objects in canonical key order. Its exact layout is **not** frozen (only the JSON shape is).
- **Formal expression:** `∀ v conforming to the output schema, ∀ leaf (path, x) ∈ v, render(v) contains a line with key(path) and text(x)`.
- **Error cases:** —

**RG-ST03-2.4: Canonical JSON** `@Invariant` [DERIVED AC-2.4]
- **AC ref:** [US-2/AC-2.4]
- **Formal expression:** `encode : Value → bytes` is a function (same value ⇒ same bytes, on every machine) with:
  - object keys sorted by their UTF-8 bytes; no duplicate keys (impossible by construction);
  - no whitespace outside strings;
  - integers in shortest decimal (`-0` impossible, no `+`, no leading zero, no exponent); no floating-point numbers exist in `Value`;
  - strings in UTF-8; escaped: `"` → `\"`, `\` → `\\`, U+0008 `\b`, U+0009 `\t`, U+000A `\n`, U+000C `\f`, U+000D `\r`, other U+0000–U+001F as `\u00XX` (lowercase hex); everything else literal (non-ASCII and `/` unescaped);
  - an absent optional value is an absent key, never `null`.
  - round trip: any standard JSON parser reads the bytes back as the same value.
- **Error cases:** —

#### Case matrix — US-2

| # | Case | Type | Input | Expected result | Rule |
|---|---|---|---|---|---|
| 1 | Nominal JSON | Nominal | `arbitre test echo --text hi --json` | one document, `ok:true`, data = the echoed input; exit 0 | 2.1 |
| 2 | Same info both modes | Nominal | same without `--json` | every leaf of case 1's data appears in the human text | 2.3 |
| 3 | Empty list | Edge | `test echo` without `--tag` | `"tags":[]`, exit 0 | 2.1, 4.4 |
| 4 | Free text | Edge | `--text` with `\n`, `"`, `\`, U+0001, `é`, `日本` | valid JSON, escaping per 2.4; human text contains the string verbatim | 2.3, 2.4 |
| 5 | Pure command | Edge | `arbitre test noop --json` | `{"data":{},"format":1,"ok":true}`; human mode prints nothing; exit 0 | 2.1, 4.5 |
| 6 | Failure in JSON | Error | `arbitre test refuse --json` | `ok:false`, category `refusal`, code 1, id, message; nothing else on stdout | 2.1 |
| 7 | Diagnostics | Edge | `arbitre test noisy --json` (writes to stderr) | stdout parses as one document | 2.1 |
| 8 | Same bytes | Edge | case 1 twice, keys inserted in different orders by the action | identical stdout bytes | 2.4 |

### SPEC-US-3: Exit codes tell automation what happened
**Ref:** US-3; frame RG-3.5, RG-3.7, INV-14, AMB-16; F-6, F-38; Q-1 (a), Q-2 (a)

**RG-ST03-3.1: The exit code table (frozen)** `@Invariant` [ARB Q-1 (a), F-6]
- **AC ref:** [US-3/AC-3.1], [US-3/AC-3.2], [US-3/AC-3.3]

  | Category | JSON `category` | Exit code | Meaning |
  |---|---|---|---|
  | `Success` | — | 0 | Done (possibly with a remark in the output, frame RG-15.6) |
  | `Refusal` | `refusal` | 1 | The answer is no: business refusal, or a decision diff violation |
  | `Usage` | `usage` | 2 | The command is malformed (RG-ST03-1.1, 1.2) |
  | `ToolError` | `tool_error` | 3 | The tool or its environment failed: bug, crash, storage, unreachable service, malformed invariants file |
  | `ModuleNotEnabled` | `module_not_enabled` | 4 | The command belongs to a module that is not enabled |
- **Formal expression:** `code` is injective over the five categories; `code(c) = 0 ⇔ c = Success`; no other exit code is produced by a normal return (RG-ST03-3.5 covers aborts).
- **Error cases:** —

**RG-ST03-3.2: The code does not depend on mode or surface** `@Invariant`
- **AC ref:** [US-3/AC-3.8]
- **Formal expression:** `∀ c, exit(c) = exit(c + "--json")`; the category of an outcome is decided before rendering, by the same code path for every surface.

**RG-ST03-3.3: Failures outside the action's rules are tool errors** `@Invariant` [ARB Q-2 (a)]
- **AC ref:** [US-3/AC-3.6], [US-5/AC-5.3]
- **Post-conditions:** storage, network and provider failures are `ToolError`, told apart by a stable id. Ids fixed now:

  | Source | Error id |
  |---|---|
  | `StoreError::StorageFull` | `storage_full` |
  | `StoreError::Corrupted(_)` | `storage_corrupted` |
  | `Unavailable(Busy)` | `storage_busy` |
  | `Unavailable(IncompatibleVersion)` | `storage_incompatible_version` |
  | `Unavailable(Io)`, `Unavailable(RandomSource)`, store directory not creatable | `storage_unavailable` |
  | `ApplyError::Rejected(_)` from a local apply | `write_rejected` (an action bug: it built an invalid change) |
  | No default store location (RG-ST03-5.1) | `no_default_store` |
  | Invalid catalogue (RG-ST03-4.6) | `catalog_invalid` |
  | Output not conforming to its schema (RG-ST03-4.5) | `output_mismatch` |
  | Caught crash (RG-ST03-3.5) | `internal_error` |
  | Result not writable to standard output | `output_failed` |
- Later ids (`network_unreachable`, `ai_not_configured`, …) are added by the stories that need them; adding an id is not an amendment, changing a category is.

**RG-ST03-3.4: Action failures carry a non-success, non-usage category** `@Invariant` [DERIVED Q-3 (a)]
- **Formal expression:** an `ActionError` returned by an action has category `Refusal` or `ToolError` (and, until ST-04 owns it, `ModuleNotEnabled` from the test module). `Success` is unrepresentable in an error; `Usage` from an action is a contract breach, reported as `ToolError`/`internal_error`.

**RG-ST03-3.5: A crash is never success nor refusal** `@Invariant` [HYP AC-3.5]
- **Post-conditions:** a panic anywhere after argument reading is caught: exit 3, id `internal_error` (JSON document in JSON mode; the panic message goes to standard error). An abort that cannot be caught (stack overflow, out of memory) ends the process by signal: the shell sees `128 + n`, never 0 nor 1.
- **Formal expression:** `crash ⇒ exit ∉ {0, 1}`.
- **Error cases:** a panic during a mutating action: the ST-02 write transaction is all or nothing, so the state is either before or after the whole batch.

**RG-ST03-3.6: Nothing changes on failure** `@Property` [global rule, ST-02 all or nothing]
- **Post-conditions:** for every non-success outcome, the stored state equals the state before, except `output_failed` (`CONFLICT-ST03-3`).
- **External state required:** the store.
- **Simulator needed:** the test module's mutating action with a failure switch (E-ST03-4).
- **Note:** for real actions this is a duty of each action (apply its batch last, in one local apply); ST-04's conformance test checks it per module.

**RG-ST03-3.7: The table is published as a contract** [AC-3.7]
- The table of RG-ST03-3.1 is in the user documentation (`README.md`, section "Exit codes"), stating that it never changes. A test compares that table with the code values (E-ST03-6).

#### Case matrix — US-3

| # | Case | Type | Input | Expected | Rule |
|---|---|---|---|---|---|
| 1 | Success | Nominal | `arbitre test noop` | 0 | 3.1 |
| 2 | Refusal | Error | `arbitre test refuse` | 1 | 3.1 |
| 3 | Usage | Error | `arbitre test echo --bogus` | 2 | 3.1 |
| 4 | Tool error | Error | `arbitre test fail` | 3 | 3.1 |
| 5 | Module not enabled | Error | `arbitre test not-enabled` | 4 | 3.1 |
| 6 | Each of 1–5 with `--json` | — | — | same code; document's `code` = exit code | 3.2 |
| 7 | Crash | Error | `arbitre test panic` (± `--json`) | 3, id `internal_error` | 3.5 |
| 8 | Store busy | Error | `test write` while another process holds a write transaction past the wait | 3, id `storage_busy` | 3.3 |
| 9 | Store damaged / newer layout | Error | `test noop` on a text file / a newer layout | 3, `storage_corrupted` / `storage_incompatible_version`; file unchanged | 3.3, 5.3 |
| 10 | Mutating action fails | Error | `test write --key k --fail refusal` (then `tool-error`, `panic`) | 1 / 3 / 3; store unchanged | 3.6 |
| 11 | Success with remark | Edge | `test echo --text x` (data carries a `remark` field) | 0; remark in both outputs | 3.1, 2.3 |

### SPEC-US-4: An action declares itself once; the command line follows
**Ref:** US-4; archi "Action catalogue"; plan § ST-03 deliverable; Q-6 (b), Q-7 (a), Q-8 (a)

**RG-ST03-4.1: Descriptor fields (frozen after ST-04)** `@Invariant`
- **AC ref:** [US-4/AC-4.1]
- **Fields:** `name` (action name); `summary` (one line, non-empty, no line break); `input` (object schema); `output` (object schema); `mutating: bool`; `logged_as ∈ {None, Delete, Move}`; `requires_network ⊆ {Calendar, Relay, Ai}` [ARB Q-8 (a)] — the destinations the action **cannot complete without**; an optional improvement (for example AI on top of the deterministic proposal) is a degraded path and is not declared. `∅` = no network.
- **Consistency:** `logged_as ≠ None ⇒ mutating` `[DERIVED]`.

**RG-ST03-4.2: Name form and command path** `@Invariant` [ARB Q-6 (b), S5]
- **Formal expression:**
  - `name = seg ("." seg)*`, `seg = [a-z][a-z0-9]*(-[a-z0-9]+)*`; at least one segment; the first is the declaring module's id; total length ≤ 64 (MCP clients' tool-name limit `[DERIVED]`).
  - command path = segments, minus the first when it is `arbitre` (reserved for foundation actions); the name `arbitre` alone is invalid.
  - MCP tool name = name with `.` → `_`. Injective because segments contain no `_`.
- **Error cases:** violated at catalogue build → RG-ST03-4.6.

**RG-ST03-4.3: Schema subset (frozen after ST-04)** `@Invariant` [ARB Q-7 (a), GAP-ST03-1]

  | Subset type | JSON Schema form | Allowed in input | CLI form |
  |---|---|---|---|
  | string | `{"type":"string"}` + optional `minLength`, `maxLength` | yes | `--name <TEXT>` |
  | integer (i64) | `{"type":"integer"}` + optional `minimum`, `maximum` | yes | `--name <INTEGER>` |
  | boolean | `{"type":"boolean"}` | yes, never required, absent = `false` | flag `--name` |
  | enum | `{"type":"string","enum":[…]}`, values `[a-z0-9]+(-[a-z0-9]+)*`, ≥ 1, distinct | yes | `--name <VALUE>`, values listed in help |
  | date | `{"type":"string","format":"date"}`, `YYYY-MM-DD`, years 0001–9999, an existing calendar day | yes | `--name <DATE>` |
  | list of T | `{"type":"array","items":T}` | yes, T scalar and not boolean; absent = `[]` | `--name` repeatable, order kept |
  | object | `{"type":"object","properties":{…},"required":[…],"additionalProperties":false}` | top level only | — |
  | optional | property left out of `required` | yes, except boolean and list | option may be omitted; value absent |
- Every property has a `description` (help text). Property names `[a-z][a-z0-9]*(_[a-z0-9]+)*`; CLI option = `--` + name with `_` → `-`. Names `json`, `store`, `help`, `version` are reserved.
- Output schemas use the same subset with no restriction on nesting (object in object, list of objects).
- The descriptor exposes each schema in the JSON Schema form above, encoded as canonical JSON (RG-ST03-2.4). This is what ST-07 hands to `Tool::new`, with no translation.

**RG-ST03-4.4: Input conformance** `@Invariant`
- **AC ref:** [US-4/AC-4.2], RG-ST03-1.2
- **Formal expression:** `conforms(v, s)`: `v` is an object with exactly the declared properties present (required always; optional when given; boolean and list always, with their defaults), each value of its type and within its constraints. One function decides it for every surface; the CLI turns option text into a `Value` with a per-type text parser (decimal integer; date as above; enum by exact match) and reports the first failure as a usage error.

**RG-ST03-4.5: Output conformance** `@Invariant` [DERIVED]
- **AC ref:** [US-4/AC-4.3]
- **Post-conditions:** an action's success value is checked against its output schema before rendering; a mismatch is `ToolError`/`output_mismatch` (an action bug), never a partial output.
- An action with nothing to return declares `{"type":"object","properties":{}}` and returns `{}`.

**RG-ST03-4.6: Catalogue build refuses invalid declarations** `@Invariant` [AC-4.6, DERIVED]
- **AC ref:** [US-4/AC-4.6]
- **Refused, with a message naming the action(s) concerned:**
  - two actions with the same name (names both declaring modules);
  - two actions with the same command path (for example module `export` action `export` and `arbitre.export`);
  - a name breaking RG-ST03-4.2, a first segment differing from the declaring module's id, a module id `arbitre` outside the foundation;
  - a schema outside RG-ST03-4.3 (optional boolean, optional list, list of list, list of boolean, list of object or nested object in an input, empty or duplicate enum, `minimum > maximum`, `minLength > maxLength`, reserved or ill-formed property name, missing description);
  - `logged_as ≠ None` with `mutating = false`; an empty or multi-line summary.
- **Effect:** every command (help included) exits 3, `catalog_invalid`, without opening the store. A test builds the shipped catalogue (with and without `test-actions`), so this reaches CI, never a user.

**RG-ST03-4.7: The command line is derived, never written** `@Invariant`
- **AC ref:** [US-4/AC-4.2], [US-4/AC-4.5]
- **Post-conditions:** the command tree, help text (summary, property descriptions, value names, enum values), option parsing and types come from the catalogue only. Sub-commands are listed in name order. Global options `--json`, `--store <PATH>`, `--help`, `--version` exist at every level. No short options are derived.

#### Case matrix — US-4

| # | Case | Type | Input | Expected | Rule |
|---|---|---|---|---|---|
| 1 | Every input kind | Nominal | `test echo` with string, integer, boolean, enum, date, optional, list | echo returns exactly the conforming `Value`; help shows each option and type | 4.3, 4.4, 4.7 |
| 2 | Optional absent | Edge | `test echo` without `--note` | key `note` absent from data | 4.3, 2.4 |
| 3 | Repeated list option | Edge | `--tag a --tag b --tag a` | `["a","b","a"]` | 4.3 |
| 4 | Duplicate name | Error | catalogue with two `test.echo` | build refused, both modules named | 4.6 |
| 5 | Optional boolean declared | Error | catalogue | build refused | 4.6 |
| 6 | Output mismatch | Error | `test bad-output` | 3, `output_mismatch` | 4.5 |
| 7 | Free-depth path | Nominal | `test.group.child` | `arbitre test group child` works | 4.2 |
| 8 | MCP name | Nominal | `plan.move-item` | `plan_move-item`; descriptor's input schema is valid JSON Schema | 4.2, 4.3 |
| 9 | Foundation path | Nominal | `arbitre.test-foundation` (test only) | `arbitre test-foundation` | 4.2 |

### SPEC-US-5: Find the data without configuring anything
**Ref:** US-5; Q-5 (a)

**RG-ST03-5.1: Store location** `@Invariant` [ARB Q-5 (a)]
- **AC ref:** [US-5/AC-5.1], [US-5/AC-5.2]
- **Formal expression:** location = `--store <PATH>` if given, else `ARBITRE_STORE` if set and non-empty, else the default:
  - Linux and other Unix: `$XDG_DATA_HOME/arbitre/arbitre.db` when `XDG_DATA_HOME` is set and absolute, else `$HOME/.local/share/arbitre/arbitre.db`;
  - macOS: `$HOME/Library/Application Support/arbitre/arbitre.db`;
  - no usable `HOME` (or another platform) and no override → `ToolError`/`no_default_store`.
- A relative path is relative to the current directory. `--store ""` is a usage error. The location names the store file.

**RG-ST03-5.2: Opened once, after parsing, for actions only** `@Invariant` [HYP AC-1.3]
- **Post-conditions:** the store is opened (and created with its missing parent directories if absent, ST-02 semantics) only when a well-formed command resolves to an action, before the action runs. One handle per process, ST-02 default options (5 s wait).

**RG-ST03-5.3: An unopenable store changes nothing** `@Property` [AC-5.3]
- **Post-conditions:** an opening failure is reported with the ids of RG-ST03-3.3, exit 3, and nothing is written (ST-02 RG-ST02-7.2, 7.3).
- **Simulator needed:** a damaged file, a newer layout, an uncreatable directory (read-only parent).

**RG-ST03-5.4: Two locations are two data sets** [HYP] — no merge, no warning.

#### Case matrix — US-5

| # | Case | Type | Input / state | Expected | Rule |
|---|---|---|---|---|---|
| 1 | Default | Nominal | no option, no env, `HOME=<tmp>` | store created at the platform default | 5.1 |
| 2 | Env | Nominal | `ARBITRE_STORE=<tmp>/a.db` | store at that path | 5.1 |
| 3 | Option beats env | Nominal | env `a.db`, `--store b.db` | `b.db` used, `a.db` not created | 5.1 |
| 4 | Directory not creatable | Error | `--store /readonly/x/a.db` | 3, `storage_unavailable`; nothing created | 5.3 |
| 5 | No home | Error | `HOME` unset, no override | 3, `no_default_store` | 5.1 |
| 6 | Same data across programs | Nominal | `test write` then `test read` via env in two processes | read sees the write | 5.1 |

## Architecture notes (phase 3 folded in)

**Placement.**

| Path | Content | Zone |
|---|---|---|
| `crates/core/src/catalog.rs` | `ActionDescriptor`, schema types, `Value`, `Output`, catalogue build and its checks (RG-ST03-4.6), input/output conformance, text parsers per type, JSON Schema form, canonical encoding, generic human rendering | Std |
| `crates/core/src/error.rs` | `Category` with `exit_code()` and JSON name, `ActionError` | Std |
| `crates/cli/src/main.rs` | read raw args (JSON mode), build catalogue, derive, parse, resolve store location, open store, run, render, exit; panic catch | Std |
| `crates/cli/src/modules.rs` | the module list: **one line per module** | Std |
| `crates/cli/src/wiring/<module>.rs` | one file per module: its descriptors and its execution entry | Std (`wiring/test.rs`: Gen) |
| `crates/cli/src/cli/` | the command-line surface: derivation, rendering. ST-07 adds `mcp/` beside it | Std |
| `crates/cli/tests/` | acceptance tests, spawning the built binary (`CARGO_BIN_EXE_arbitre`) | — |

The split between `catalog.rs` and sibling files is the planner's choice, provided nothing lands under `crates/core/` as `Gen` and `zones.toml` lists them.

**No new dependency in `arbitre-core`** (AC-4.4, `CONFLICT-ST03-1`): `Value` is a core type (`BTreeMap` objects, `i64`, no float, no null), and canonical JSON is a small pure encoder in core. The browser check (`20-st01-browser.sh`) stays green unchanged. The CLI needs no JSON parser either: it only emits. ST-07 converts `rmcp`'s `serde_json` values to `Value` at its boundary.

**CLI derivation.** `clap` builder API at runtime (`Command`, `Arg`), since names have free depth: one sub-command per path segment, nested; a leaf carries one `Arg` per input property (`long` only; `required`; `ArgAction::SetTrue` for boolean, `Append` for list; enum through possible values; a `value_parser` closure that calls core's per-type text parser, so clap formats the error with the usage). `disable_help_subcommand(true)`; `subcommand_required` on groups that are not actions. Parse with `try_get_matches_from`; `DisplayHelp`/`DisplayVersion` → exit 0 (RG-ST03-1.4), other clap errors → `Usage` with an id from the error kind (`unknown_command`, `unknown_argument`, `missing_argument`, `invalid_value`, `usage` otherwise). `--json` and `--store` are global args; JSON mode is also pre-detected on raw args for parse failures. `ARBITRE_STORE` is read with `std::env` (clap's `env` feature not needed). `clap`'s `derive` feature becomes unused and may be dropped.

**Execution seam (provisional until ST-04).** A wiring file exposes `descriptors()` and an execution entry taking the action name, the conforming input `Value` and a context (store handle, physical time from a native clock, `Author::CliUser`), returning `Result<Value, ActionError>`. ST-04 replaces it with `Module::execute` and `Context` without changing descriptors, `Value`, `Output`, categories or the JSON shape. ST-02's `SqliteStore` futures are ready on first poll (spike S1-A): driving them with a single poll on a no-op waker (`std::task::Waker::noop`) is enough in ST-03; a pending future is `ToolError`/`internal_error`. `tokio` arrives with ST-07.

**Test module.** `wiring/test.rs` behind a cargo feature `test-actions` of `arbitre`; CI's `--all-features` builds the binary with it; a release build does not. Actions: `test.echo` (every input kind, echoes input, adds a `remark`), `test.noop`, `test.refuse`, `test.fail`, `test.not-enabled`, `test.panic`, `test.bad-output`, `test.noisy`, `test.write` (mutating; `--key`, optional `--fail refusal|tool-error|panic` decided before the local apply), `test.read`, `test.group` + `test.group.child` (if Q-B (a)), `arbitre.test-foundation`.

**Panic catch.** `std::panic::catch_unwind` around everything after raw-argument reading; the result document is built in memory and written once, so a panic never leaves a half document on stdout.

**Dependency on ST-02.** ST-03 implementation needs PR #6 merged (or a branch on `st-02/store`): `arbitre` gains `arbitre-sqlite` as a dependency (edge already allowed by `graph.py`).

## Global rules

**RG-ST03-G.1: Fully offline** `@Invariant` [frame RG-1.1] — parsing, help, codes and every test action need no network; `arbitre`'s new edges add no network crate.

**RG-ST03-G.2: Frozen contract** [F-6, F-38, § Collisions] — the table of RG-ST03-3.1, the JSON shape of RG-ST03-2.1, the encoding of RG-ST03-2.4, the name form of RG-ST03-4.2 and the schema subset of RG-ST03-4.3 are frozen at the end of ST-03; the Rust types at the end of ST-04. Changing them is a foundation amendment. Error ids and messages are not frozen, except the ids listed in RG-ST03-3.3, which may only be added to.

## Acceptance tests

All in `crates/cli/tests/` (spawning the binary) unless noted; run by `10-st01-tests.sh` (`--all-features`), no CI change.

| # | Test | Checks |
|---|---|---|
| E-ST03-1 | **Malformed commands** (property, 200 cases): from a well-formed `test write`/`test echo` command, apply one random defect (drop a required option, add an unknown option, unknown word, bad integer, out-of-range integer, non-existent or ill-shaped date, enum outside the list, repeated non-list option), ± `--json`, on (a) an absent store path and (b) a store with data. | exit 2; (a) path still absent; (b) store reads (`writes_since(START)`, `device`) identical; JSON mode: one document, `category` `usage` | 
| E-ST03-2 | **Human/JSON equivalence** (property, 200 cases): random conforming inputs to `test echo`, including free text with control, quote and non-Latin characters. | same exit code; stdout of JSON mode parses (dev-dependency `serde_json`, shared) to the echoed value; every leaf appears in the human text | 
| E-ST03-3 | **One code per category**: matrix US-3 rows 1–7, each in both modes. | exact exit code; document `code` = exit code; one document only | 
| E-ST03-4 | **Nothing changes on failure**: `test write --fail refusal|tool-error|panic` on a store with data. | exit 1/3/3; store reads identical | 
| E-ST03-5 | **Store location**: matrix US-5 rows 1–6 with a temporary `HOME`. | paths used/created as specified; errors and ids | 
| E-ST03-6 | **Published table**: parse the "Exit codes" table of `README.md`. | equals `Category` names and `exit_code()` for all five | 
| E-ST03-7 | **Catalogue build** (core unit tests): each refusal of RG-ST03-4.6; the shipped catalogue builds with and without `test-actions`. | refused with names; shipped catalogue valid | 
| E-ST03-8 | **Canonical JSON** (core, property): random `Value`s. | `serde_json` (dev) parses the bytes back to the same value; keys sorted; re-encoding is byte-identical; golden bytes for the escaping list of RG-ST03-2.4 | 
| E-ST03-9 | **JSON Schema form** (core): golden canonical JSON of `test.echo`'s input schema; each subset type. | matches RG-ST03-4.3 table | 
| E-ST03-10 | **Help and version**: `--help`, `test echo --help`, `--version`, each ± `--json`, store path absent. | exit 0; help lists every option, type and enum value from the descriptor; path still absent | 

## Identified dependencies

| Dependency | Type | Description | Impact |
|---|---|---|---|
| ST-02 (`Store`, `StoreError`, `SqliteStore::open`) | Internal | Opening the store, local apply, reads for tests | Blocking (PR #6 to merge first) |
| `clap` 4.6.7 (already in `arbitre`) | External | Runtime command tree, parsing, usage text | Blocking |
| `serde_json` (shared, dev-dependency only) | External | Independent JSON parser in tests (E-ST03-2, 8) | Non-blocking |
| `proptest` (shared, dev) | External | E-ST03-1, 2, 8 | Non-blocking |
| Native physical clock | Internal | `pt` for `test.write`'s local apply (`SystemTime` in `arbitre`) | Blocking for `test.write` |
| ST-04 | Downstream | `Module`, `Context`, enabling, RG-ST03-1.5 mechanism, per-module RG-ST03-3.6 check | — |
| ST-07 | Downstream | Uses descriptor JSON Schema form and the `.`→`_` name rule unchanged | — |

## Conflicts and ambiguities

| # | Rules | Description | Resolution |
|---|---|---|---|
| [CONFLICT-ST03-1] | archi (`arbitre-core` deps: "serde") vs AC-4.4 ("no new dependency") | The archi expects serde in core; the binding story keeps core dependency-free. A JSON value type is needed in core. | AC-4.4 wins: core defines `Value` and the canonical encoder itself (no float, no null, sorted keys by construction). This also removes the `serde_json` `preserve_order` feature-unification risk once `rmcp` joins the graph. |
| [CONFLICT-ST03-2] | frame matrix US-3 row 4 ("invalid date → refusal") vs Q-3 (a) | The frame classes "31/02" as a refusal. | Q-3 (a) is binding: a non-existent date is **usage** (RG-ST03-1.2). The frame matrix is superseded on this point. |
| [CONFLICT-ST03-3] | "nothing changes on failure" vs a closed standard output | A mutating action may commit, then the result cannot be written (`arbitre … --json | head -c1`). | Exit 3, `output_failed`; the state **has** changed. Documented exception; the result is not replayed. |
| [AMB-ST03-1] | frame RG-3.2 (`arbitre plan` → morning view) vs a group | A command path may need to be both an action and a group. | Question Q-B. Until answered, specs allow it (RG-ST03-4.2 permits a one-segment name equal to the module id). |
| [AMB-ST03-2] | RG-ST03-1.5 vs RG-ST03-1.3 | If ST-04 keeps the enabled state in the store, deciding "module not enabled" on a malformed command means reading the store. | For ST-04: read an **existing** store only, never create; no store ⇒ default enabled state. RG-ST03-1.3 is then amended to "never created or written". |
| [GAP-ST03-1] | Q-7 subset | The arbitrated subset lists no `object`, but every input and output is an object, and outputs need records. | `object` added (RG-ST03-4.3); restricted to the top level in inputs. |
| [GAP-ST03-2] | Q-3 "range", "length limit" | The subset had no constraint keywords. | `minimum`/`maximum` (integer), `minLength`/`maxLength` (string) added, so these refusals are usage as Q-3 (a) intends. |

## Candidate invariants (C-TDD synthesis)

### @Invariant (stateless, blocking)
- `INV-ST03-1` (frame INV-12): malformed command ⇒ exit 2, usage shown, store never opened — RG-ST03-1.1, 1.2, 1.3
- `INV-ST03-2` (frame INV-13): human rendering is a function of the `Value` only; every leaf appears in it — RG-ST03-2.3
- `INV-ST03-3` (frame INV-14): five categories, codes 0–4 injective, `0 ⇔ Success`; same code in both modes — RG-ST03-3.1, 3.2
- `INV-ST03-4`: JSON mode writes exactly one result document of the frozen shape on stdout, for every outcome — RG-ST03-2.1
- `INV-ST03-5`: canonical encoding is a function, sorted keys, fixed escaping, parseable back — RG-ST03-2.4
- `INV-ST03-6`: crash ⇒ exit ∉ {0, 1}; caught panic ⇒ 3 `internal_error` — RG-ST03-3.5
- `INV-ST03-7`: catalogue build refuses duplicates, path collisions, ill-formed names, out-of-subset schemas, `logged_as` without `mutating` — RG-ST03-4.2, 4.3, 4.6
- `INV-ST03-8`: an action receives only conforming input; its output is checked against its schema — RG-ST03-4.4, 4.5
- `INV-ST03-9`: `arbitre-core` has no normal dependency and builds for `wasm32-unknown-unknown` — AC-4.4
- `INV-ST03-10`: help/version exit 0 and open no store — RG-ST03-1.4

### @Property (stateful, simulator needed)
- `PROP-ST03-1`: non-success outcome ⇒ store unchanged (except `output_failed`) — simulator: `test.write` with failure switch on a populated store — RG-ST03-3.6
- `PROP-ST03-2`: store location resolution (option > env > default) and opening failures ⇒ exit 3 with the mapped id, nothing written — simulator: temporary `HOME`, damaged/newer/uncreatable stores, a process holding a write transaction — RG-ST03-3.3, 5.1, 5.3

## Questions for Quentin

**Q-B — May a command path be both an action and a group? [FREEZES the name rule]** Frame RG-3.2 wants `arbitre plan` to show the morning view while `arbitre plan add` exists.
- (a) Yes: a name may be the module id alone (`plan` → `arbitre plan`), and any path may be both an action and a group; the action's options and the sub-commands exclude each other.
- (b) No: names are prefix-free; ST-05 names the view `plan.today` and RG-3.2 is read as "the `plan` group".
- **Recommendation: (a).** It keeps RG-3.2 as written; `clap` supports it directly; refusing it now would force an amendment at ST-05. Cost: one more test action.
