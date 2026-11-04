# Requirements — ST-03: Action catalogue, derived CLI, exit codes

Sources: `.ctdd/cadre/04_plan.md` § ST-03, F-6, F-11, F-20, F-37, F-38, § Collisions ("contract types frozen", "exit code table: frozen"); `.ctdd/cadre/01_story.md` US-3 (AC-3.3, 3.4, 3.5); `.ctdd/cadre/02_specs.md` RG-3.3, 3.4, 3.5, 3.7, RG-15.3, INV-12, 13, 14, AMB-16, CONFLIT-6; `.ctdd/cadre/03_archi.md` (pattern "Action catalogue", `DescripteurAction`, `Sortie`, "error category → exit code", constraint "Exit codes"); ST-02 deliverable (public `Store` trait and `StoreError` only).

Names follow the `CONTRIBUTING.md` glossary. New terms introduced by this story (proposed code names, confirmed by the specs):

| Spec term (French) | Code name |
|---|---|
| catalogue d'actions | action catalogue / `Catalog` |
| `DescripteurAction` | `ActionDescriptor` |
| mutante | `mutating` |
| `journalisée_comme` (`aucune`, `suppression`, `déplacement`) | `logged_as` (`None`, `Delete`, `Move`) |
| `requiert_réseau` (`aucun`, `agenda`, `relais`, `ia`) | `requires_network` (`None`, `Calendar`, `Relay`, `Ai`) |
| `Sortie` | `Output` |
| rendu humain | human rendering |
| sortie structurée | structured output (`--json`) |
| catégorie d'erreur | `Category` (`Success`, `Refusal`, `Usage`, `ToolError`, `ModuleNotEnabled`) |
| refus ou violation | refusal (a decision diff violation is a refusal) |
| erreur d'outil | tool error |
| module non activé | module not enabled |
| code de retour | exit code |

## Context
Every later lane (planning, decision diff, mobile, sync) exposes its features as actions, and every surface (command line now, assistant connector in ST-07, graphical interface later) is derived from those actions rather than written by hand. ST-03 fixes, once, what an action declares about itself, what it returns, how the command line is derived from it, and which exit code each kind of outcome gets. After ST-03 these are frozen: ST-04, ST-22 and ST-24 use the exit code table without changing it (F-6), and the contract types are frozen after ST-04 (§ Collisions). The story has no user-visible value on its own (F-20): ST-04 adds the module mechanism, ST-05 the first real actions.

## Target users
- **Automation author** (the solo user's scripts, a CI job running `arbitre diff check`): reads the exit code and the structured output, never the human text.
- **Solo user at the terminal**: types a command, reads the human output, gets told what was expected when the command is wrong.
- **Maintainer** (Quentin): arbitrates the frozen contract; changing it later is a foundation amendment.
- **Development agents** (non-human actors): declare actions in every later story and get the command line, the structured output and the exit codes without writing them.

## User Stories

### US-1: A malformed command explains itself and changes nothing
**As a** solo user, **I want** a malformed command to show what was expected, change no data and fail with a non-success code, **so that** a typo never damages my data or passes silently in a script (AC-3.3, RG-3.3, INV-12).

**Acceptance Criteria:**
- [ ] AC-1.1: A command with a missing argument, an unknown argument or option, or a value of the wrong type shows the expected usage of that command and exits with the **usage** code.
- [ ] AC-1.2: After any malformed command, the stored state is identical to the state before (checked with a **mutating** test action, so that "nothing changed" is meaningful).
- [ ] AC-1.3: A malformed command run on a machine with no storage yet creates no storage `[HYPOTHESIS]`: the storage is opened only once the command is well formed.
- [ ] AC-1.4: An unknown action name is a malformed command (usage code), not a tool error.
- [ ] AC-1.5: `--help` and `--version` succeed (success code); they are not usage errors.
- [ ] AC-1.6: Where a value is well formed but refused by the action's own rules (for example a non-existent date), see Q-3 for which category applies; in both cases nothing is changed and the code is not success.

**Nominal cases:**
- The user types `arbitre test-action --colour red` instead of `--color`: the tool names the unknown option, shows the usage, exits with the usage code, and the data is untouched.

**Identified edge cases:**
- Malformed command with `--json`: the usage error is also given in the structured format (Q-4).
- Malformed command of a module that is not enabled: "module not enabled" takes precedence over usage `[HYPOTHESIS]`, since the user cannot fix the arguments of a module they cannot see (CONFLIT-6).

### US-2: Every command has a machine output equivalent to the human one
**As an** automation author, **I want** every command to produce, on request, a structured output carrying the same information as the human output, **so that** my scripts never parse human text (AC-3.4, RG-3.4, INV-13).

**Acceptance Criteria:**
- [ ] AC-2.1: Every command accepts `--json`; with it, the command writes one structured document and no human text on the standard output.
- [ ] AC-2.2: The human output is derived from the structured value only: it shows nothing the structured value does not carry, and everything the structured value carries is shown (same state, same signals). Checked by a property over the outputs of the test actions.
- [ ] AC-2.3: A failure also has a structured form in a stable format: category, exit code, stable error identifier and message (RG-3.4 "error case"). Shape: Q-4.
- [ ] AC-2.4: The structured output is canonical: the same result always gives the same bytes (key order, number formats), whatever the machine.
- [ ] AC-2.5: The structured output carries the result only; diagnostics and progress messages never mix into it.

**Nominal cases:**
- A script runs `arbitre test-action --json`, reads the document, and gets exactly the values the human output shows.

**Identified edge cases:**
- Empty result (nothing to list): a valid structured document with an empty list, success code.
- An action whose output contains free text (titles with line breaks, quotes, non-Latin characters): the structured document stays valid and the text is identical in both outputs.

### US-3: Exit codes tell automation what happened
**As an** automation author (in particular a CI job running decision diff), **I want** each kind of outcome to have its own fixed exit code, **so that** I can tell "the answer is no" from "I called it wrong" from "the tool is broken" (AC-3.5, RG-3.5, INV-14, AMB-16, F-6).

**Acceptance Criteria:**
- [ ] AC-3.1: The table has exactly five categories, each with one distinct code: **success**, **refusal or violation**, **usage**, **tool error**, **module not enabled** (F-6). Values: Q-1.
- [ ] AC-3.2: Only success gives the success code; every failure gives a non-success code (INV-14).
- [ ] AC-3.3: A decision diff violation and a tool error (for example a malformed invariants file) have distinct codes (AMB-16, frozen here per F-38).
- [ ] AC-3.4: Each category is produced by a test action and its exit code checked (plan completion criterion).
- [ ] AC-3.5: An unexpected internal failure of the tool (a crash) never exits with the success code, nor with the refusal code `[HYPOTHESIS]`: a CI must not read a crash as "the code has violations".
- [ ] AC-3.6: Failures from outside the action's own rules (storage full, damaged or busy; external service unreachable) map to a category fixed by Q-2.
- [ ] AC-3.7: The table is published in the user documentation as a contract, with the statement that it does not change (plan completion criterion).
- [ ] AC-3.8: The code is the same whether or not `--json` is given.

**Nominal cases:**
- `arbitre diff check` in CI exits with the refusal code when the change violates an invariant, and with the tool error code when the invariants file is malformed; the CI shows "non-compliant code" or "broken tool" accordingly (ST-22, ST-24 use these codes).

**Identified edge cases:**
- Explicit call of a module that is not enabled: "module not enabled" message, its own code, nothing changed (RG-3.7, CONFLIT-6). The mechanism comes with ST-04; ST-03 only fixes the code and tests it on a test action.
- Success with a remark (for example "no invariants file", RG-15.6): success code; the remark is part of the output.
- Interruption by the user (Ctrl-C): out of scope.

### US-4: An action declares itself once; the command line follows
**As a** development agent, **I want** to declare an action once (name, inputs, outputs, whether it changes data, how it is logged, which network it needs) and get the command, its help, its option parsing and its structured output without writing them, **so that** surfaces never drift from each other and every later lane uses the same frozen contract (archi "Action catalogue", plan deliverable).

**Acceptance Criteria:**
- [ ] AC-4.1: An action descriptor carries: a name prefixed by its module (form: Q-6), a one-line summary, an input description, an output description (form: Q-7), `mutating` (yes or no), `logged_as` (none, delete, move; RG-4.6), and `requires_network` (none, calendar, relay, AI; form: Q-8).
- [ ] AC-4.2: The command, its help text, its options and their types are derived from the descriptor; no command is written by hand. Checked by a test action declared with each kind of input.
- [ ] AC-4.3: The action result (`Output`) is a structured, canonical value; the human rendering is derived from it (US-2).
- [ ] AC-4.4: The descriptor, `Output` and the error categories live in the core (`arbitre-core`), which still builds for the browser target with no new dependency (ST-01 check stays green); command-line code lives only in `arbitre`.
- [ ] AC-4.5: In `arbitre`, adding a module later means one line in a module list and one wiring file for that module; each surface has its own directory (plan deliverable). Adding a module touches no shared file beyond that line.
- [ ] AC-4.6: Two actions with the same name are refused when the catalogue is built, naming both.

**Nominal cases:**
- ST-05 declares `plan.add` with a title, a type and an optional due date; the user immediately gets `arbitre plan add --title ... --due ...`, its help, its JSON output and its codes.

**Identified edge cases:**
- An action with no input and no output (a pure command): still has a structured output (empty success document).
- An action input that is a list or an optional value: representable on the command line (repeated option, absent option).

### US-5: Find the data without configuring anything
**As a** solo user, **I want** the tool to know where my data lives without configuration, and to let me point it elsewhere, **so that** the command line, the assistant connector and the background recalculation use the same data by default (ST-02 left the storage location to ST-03).

**Acceptance Criteria:**
- [ ] AC-5.1: With no setting, every command uses one default storage location per user account (value: Q-5).
- [ ] AC-5.2: The user can point a command at another storage location; the way is identical for every command (Q-5).
- [ ] AC-5.3: A storage that cannot be opened (missing directory not creatable, `Corrupted`, newer version) is reported with an explicit message and the category of Q-2, and nothing is written.

**Nominal cases:**
- The user runs a first command; the storage is created in the default location; the MCP connector started later finds the same data.

**Identified edge cases:**
- Two different storage locations given in the same session by two programs: they are two separate data sets, not merged (no warning) `[HYPOTHESIS]`.

## Global business rules
- **Frozen once** (F-6, F-38, § Collisions): the exit code table, the meaning of each category, the descriptor fields and the structured output shape do not change after this story (types: after ST-04); a change is a foundation amendment.
- **Nothing changes on failure**: whatever the category other than success, the stored state is unchanged (RG-3.3, RG-3.7, all-or-nothing of ST-02).
- **One source**: every surface is derived from the catalogue; nothing exposed by a surface is declared elsewhere.
- **Same information on both outputs** (INV-13): human and structured outputs differ in form only.
- **Codes do not depend on the output mode or the surface** that runs the action.
- **Fully offline** (RG-1.1): parsing, help and codes need no network.

## Out of scope
- The module mechanism: `Module` trait, registry, enabling and disabling, hiding a disabled module from help and completion, namespace-restricted store view (ST-04). ST-03 fixes the "module not enabled" code only.
- The generic conformance test, the network observer and the PROP-1 / PROP-2 checks that use `requires_network` (ST-04).
- Parity GUI ⊆ CLI ⊆ MCP (INV-10, RG-3.1, ST-04 / ST-07) and the MCP surface itself (ST-07).
- `arbitre plan` and `arbitre diff` entry points and any real module action (ST-05, ST-22).
- Export and import (ST-04b, F-11).
- Shell completion scripts, colours, paging, localisation of messages; user-facing messages are in English `[HYPOTHESIS]`.
- Signals displayed at the start of a command (ST-04, F-27).
- Interruption handling (Ctrl-C) and its exit code.
- Wording of individual messages: only categories, codes and the structured shape are frozen, not message text.

## Hypotheses
- [HYPOTHESIS] ST-03 checks everything on test actions (one per category, one mutating, one per input kind) that are not shipped to users; the real catalogue is empty until ST-04 / ST-05.
- [HYPOTHESIS] Storage is opened only after the command is parsed (AC-1.3), so a usage error never touches or creates storage.
- [HYPOTHESIS] "Module not enabled" beats "usage" when both apply (US-1 edge).
- [HYPOTHESIS] A crash maps to the tool error code (AC-3.5); the specs decide how it is guaranteed.
- [HYPOTHESIS] Two storage locations are two independent data sets (US-5 edge).
- [HYPOTHESIS] Zones: the files in `arbitre-core` (`catalog`, `error`) are Std (Supporting zone of the plan; `CONTRIBUTING.md` forbids `Gen` under `crates/core/`); `arbitre` CLI derivation is Std. The specs/plan write `zones.toml`.
- [HYPOTHESIS] ST-03 needs no new shared dependency beyond those of `[workspace.dependencies]` (clap is already in `arbitre`; serde / serde_json are declared shared, PL-6).
- [HYPOTHESIS] ST-03 uses only the public `Store` trait of ST-02 (opening the SQLite store at a path, `apply_local`, reads); nothing else of ST-02.

## Questions for Quentin

Questions marked **[FREEZES]** fix a part of the shared contract; reopening them later is a foundation amendment.

**Q-1 — Exit code values. [FREEZES]**
- (a) 0 success · 1 refusal or violation · 2 usage · 3 tool error · 4 module not enabled.
- (b) BSD `sysexits` style: 0 success · 1 refusal · 64 usage · 70 tool error · 69 module not enabled.
- (c) 0 success · 1 tool error · 2 usage · 3 refusal or violation · 4 module not enabled.
- **Recommendation: (a).** Same convention as `grep` and `diff` (1 = "the answer is no", ≥ 2 = trouble), so a CI can test `code == 1` for "non-compliant" and `code ≥ 2` for "fix the setup"; 2 is also the default usage code of the argument parser. (b) is little known; (c) makes a crash-like 1 look like a verdict.

**Q-2 — Category of failures outside the action's rules** (storage full, damaged, busy; external service unreachable; AI provider not configured). **[FREEZES]**
- (a) All are **tool error**; the structured error carries a stable identifier (`storage_full`, `storage_unavailable`, …) so a script can retry on "busy".
- (b) A sixth code "environment" for them, kept apart from a tool bug.
- (c) Storage errors are tool error; unreachable service or unconfigured provider is a refusal.
- **Recommendation: (a).** F-6 fixes five categories; the stable identifier gives automation the detail without widening the frozen table. (c) would make a CI read "relay down" as "the answer is no".

**Q-3 — Boundary between usage and refusal** (value of the right shape but refused, e.g. due date "31/02", or a title over the length limit). **[FREEZES]**
- (a) Anything the action's declared input description can reject (type, format, a date that does not exist, a range) is **usage**; anything that needs the stored state or a business rule (unknown item, invalid transition) is **refusal**.
- (b) Only what the parser rejects without knowing the action's types is usage; every value check is refusal.
- (c) Everything is usage except decision diff violations.
- **Recommendation: (a).** One rule an agent can apply without asking: "could the command line have known without the data?" → usage. It matches the frame's matrix (invalid date under RG-3.3, "malformed command").

**Q-4 — Shape of the structured output. [FREEZES]**
- (a) One JSON document on standard output for every outcome, usage errors included: `{"format": 1, "ok": true, "data": …}` or `{"format": 1, "ok": false, "error": {"category", "code", "id", "message"}}`; human mode writes errors on standard error.
- (b) Same, without the `format` version number.
- (c) Success document on standard output, error document on standard error.
- **Recommendation: (a).** A single place to read is the simplest for scripts and for the MCP surface; the version number costs one field now and lets the shape evolve later without a foundation amendment.

**Q-5 — Default storage location and how to override it.** (Not frozen across lanes, but the MCP connector and recalculation must agree on it.)
- (a) The platform's per-user data directory (e.g. `~/.local/share/arbitre/` on Linux, `~/Library/Application Support/arbitre/` on macOS); override by a global `--store <path>` option and an `ARBITRE_STORE` environment variable (option wins).
- (b) Same default, option only.
- (c) A file in the current directory.
- **Recommendation: (a).** The environment variable lets an MCP client configuration and a scheduled task point at the same data without repeating an option; (c) would give each directory its own data, contradicting "same data by default".

**Q-6 — Form of action names and their command path. [FREEZES]**
- (a) `<module>.<verb>` in lowercase kebab-case (`plan.add`, `plan.move-item`), mapped to `arbitre <module> <verb>`; foundation actions use the reserved module `arbitre` and map to `arbitre <verb>` (`arbitre.export` → `arbitre export`).
- (b) Dotted path of any depth (`plan.item.add` → `arbitre plan item add`, ST-04's `arbitre module enable` = `arbitre.module.enable`).
- (c) Underscore names (`plan_add`), one command level after the module.
- **Recommendation: (b).** Same mapping rule as (a) (each dot is a command word), but ST-04 already needs three words (`arbitre module enable|disable|list`), so freezing exactly two levels would force an amendment at the very next story. Agents keep two levels unless a group is real.

**Q-7 — How an action describes its inputs and outputs. [FREEZES]**
- (a) The format the assistant tool protocol already uses (JSON Schema, a small agreed subset: string, integer, boolean, enum, date, optional, list), from which the command line is derived.
- (b) A home-made list of typed fields, translated to the assistant protocol's format in ST-07.
- **Recommendation: (a).** ST-07 then exposes descriptors as tools without translation, and the subset keeps the command-line derivation finite; a type outside the subset is refused when the catalogue is built.

**Q-8 — `requires_network`: one value or several. [FREEZES]**
- (a) A set: `{}` (none), or any combination of calendar, relay, AI (e.g. a recalculation that reads the calendar and asks the AI).
- (b) One value, as written in the plan (`none | calendar | relay | ai`).
- **Recommendation: (a).** The morning recalculation (ST-11) needs calendar and possibly AI; with one value it would have to lie, and PROP-1 / PROP-2 (ST-04) check per declared destination.

**Q-9 — Checking "same information" between human and structured output** (not frozen).
- (a) Human rendering is a function of the structured value only, plus a property test that every value of the structured output appears in the human text.
- (b) Construction only (no property test).
- **Recommendation: (a).** Construction prevents adding information to the human side; only the test catches information dropped from it.
