# Requirements — ST-04: Modules you can enable, kept apart, with one generic conformance test

Sources: `.ctdd/cadre/04_plan.md` § ST-04, F-6, F-7, F-12, F-18, F-27, F-37, R-7, § Collisions ("contract types frozen", "shared test tooling"), § Invariants (INV-11, 15, 20–23, PROP-1, PROP-2); `.ctdd/cadre/01_story.md` US-1 (AC-1.1), US-3 (AC-3.2, edge "module not enabled"), US-5 (AC-5.1 to 5.4); `.ctdd/cadre/02_specs.md` RG-1.1, 1.2, 3.2, 3.7, 5.1 to 5.4, CONFLIT-1, CONFLIT-6, AMB-12 and its arbitration; `.ctdd/cadre/03_archi.md` ("Trait Module", "Contexte", "Port Ia", "Port Clock: fuseau", "Activation des modules", "Validation après fusion", D-2); ST-01 archi arbitration P-2 (`Clock::fuseau`, `jiff`); ST-03 documents (`02_specs.md` RG-ST03-1.5, 3.6, 4.6, 5.2, AMB-ST03-2 with Q-DA2 (a); `04_plan.md` P-3, P-4, P-11 and "ST-04 handover").

Names follow the `CONTRIBUTING.md` glossary. New terms (proposed code names, confirmed by the specs):

| Spec term (French) | Code name |
|---|---|
| module (activable) | `Module` |
| socle (module) | foundation module, id `arbitre` |
| activer / désactiver | enable / disable |
| état d'activation | enabled state |
| `Contexte` | `Context` |
| vue `Store` restreinte à l'espace de noms | module store view / `ModuleStore` |
| espace de noms | namespace (the `module` part of a `RecordId`) |
| `revalider`, `Problème` | `revalidate`, `Problem` |
| module factice | fake module |
| test de conformité générique | generic conformance test |
| surface | surface (CLI now; MCP in ST-07, GUI in ST-21) |
| registre des surfaces | surface registry (`tests/support/surfaces/`) |
| observateur réseau, mode « coupé » | network observer, cut mode |
| signal (à l'ouverture) | signal |
| port `Ia` | `Ai` port |
| `Clock::fuseau()` | `Clock::timezone()` |

## Context
ST-03 fixed what an action is, the derived command line and the exit codes. It wired actions through a provisional seam (`Wiring` in `crates/cli/src/modules.rs`). ST-04 replaces that seam with the real module mechanism in the core: a `Module` contract, the `Context` an action runs with, and a store view that only reaches the module's own data. It adds the enabled state, so a user sees only the modules they use. It lays the shared test tooling every later lane reuses without editing it: a fake module, a network observer with a cut mode, a registry of surfaces, and one generic conformance test. It also adds two things later lanes need but may not add themselves, since the core is frozen after ST-04: signals shown at the next command (used by ST-11 and ST-19), the `revalidate` hook (used by ST-04b and ST-19), the declared `Ai` port (used by ST-30) and `Clock::timezone()` (used by ST-09). After ST-04, only lane M changes `arbitre-core`, and only by adding files (§ Collisions). The recalibration that follows ST-04 is a separate step, not part of this story.

## Target users
- **Solo user**: turns on only the modules they need and never sees the others.
- **Automation author** (a CI job running `arbitre diff check`): enables one module in a throwaway store, then runs it (archi D-2).
- **Development agents** of lanes P, D and M: add a module, a surface or a network adapter and get it checked by the existing conformance test, without editing that test.
- **Maintainer** (Quentin): arbitrates the contract types frozen at the end of this story.

## User Stories

### US-1: Enable only the modules I use
**As a** solo user, **I want** to enable, disable and list modules on this device, **so that** I only have the features I need, and turning a module off never costs me its data (AC-5.1, AC-5.2, RG-5.1, RG-5.2, AMB-12).

**Acceptance Criteria:**
- [ ] AC-1.1: `arbitre module enable`, `arbitre module disable` and `arbitre module list` exist as foundation actions (`arbitre.module.enable`, `.disable`, `.list`). Their exact command form: Q7.
- [ ] AC-1.2: Enabling or disabling a module that is already in that state succeeds and changes nothing.
- [ ] AC-1.3: Disabling a module deletes none of its data; enabling it again shows the same data (RG-5.2, INV-21).
- [ ] AC-1.4: Any subset of modules works on its own: no module needs another to be enabled (RG-5.1, INV-20).
- [ ] AC-1.5: The enabled state belongs to this device. It is not synchronised; the module data always is (AMB-12 arbitration). Where it is kept: Q1.
- [ ] AC-1.6: On a device that never enabled anything, the enabled state has a fixed default (Q2).
- [ ] AC-1.7: The foundation module (`arbitre`) is always enabled and cannot be disabled.

**Nominal cases:**
- A developer who only needs decision diff enables that one module and never sees planning (frame US-5 nominal).
- A CI job points `ARBITRE_STORE` at a temporary file, enables `diff`, then runs `arbitre diff check` (archi D-2).

**Identified edge cases:**
- Enabling a module that does not exist in this build: a usage error (the list of modules is known without data, ST-03 Q-3 (a)) `[HYPOTHESIS]`.
- Disabling a module while another process uses it: the other process keeps its view until it ends; the next command sees the new state `[HYPOTHESIS]`.

### US-2: A disabled module stays out of sight, and says so when called
**As a** solo user, **I want** a disabled module to be absent from the help and from every surface, and an explicit call to it to tell me it is not enabled, **so that** I am not distracted by features I do not use, and a typo never looks like a missing feature (AC-3.2 edge, AC-5.4, RG-3.7, RG-5.4, CONFLIT-6, INV-15, INV-23).

**Acceptance Criteria:**
- [ ] AC-2.1: A disabled module has no command in the help, at any level.
- [ ] AC-2.2: A disabled module's actions are absent from every list of actions a surface exposes (the CLI now; the MCP in ST-07 through the surface registry, US-5).
- [ ] AC-2.3: An explicit call to a disabled module (`arbitre diff check` when `diff` is disabled) exits with the **module not enabled** code (4), names the command that enables it, and changes nothing.
- [ ] AC-2.4: "Module not enabled" wins over a malformed command of that module (RG-ST03-1.5, frozen in ST-03; the mechanism and its test land here).
- [ ] AC-2.5: Neither help, nor a malformed command, nor a "module not enabled" outcome ever creates the store or writes to it (AMB-ST03-2 resolution).
- [ ] AC-2.6: Each module is reached by its id as the first command word (`arbitre plan` → planning, `arbitre diff` → decision diff); checked here on the fake module (INV-11, form only).

**Nominal cases:**
- With only `diff` enabled, `arbitre --help` lists `module` and `diff`, nothing about planning.

**Identified edge cases:**
- `arbitre module list` still names every module of the build with its state, so the user can find what to enable. It shows the actions of enabled modules only `[HYPOTHESIS]`.
- Shell completion scripts do not exist yet (ST-03 out of scope). "Absent from completion" holds by construction once a generator derives from the same filtered catalogue `[HYPOTHESIS]`.

### US-3: A module plugs in through one frozen contract and only sees its own data
**As a** development agent of lanes P and D, **I want** one contract to write a module against (its id, its actions, how they run, how its data is re-checked) and a store view that only reaches my module's data, **so that** I never touch shared files beyond one line, and no module can read or write another's data (AC-5.3, RG-5.3, INV-22, archi "Trait Module", "Contexte").

**Acceptance Criteria:**
- [ ] AC-3.1: The core defines `Module` (`id`, `actions`, `execute`, `revalidate`) and `Context` (author, clock, module store view, optional `Ai` port). They replace ST-03's `Wiring` and its `Context` without changing `ActionDescriptor`, `Value`, `Output`, `Category`, `ActionError` or the JSON shape. Their shape: Q3.
- [ ] AC-3.2: An action reads and writes only records of its own module. A read or write of another module's records cannot be expressed through the view (INV-22).
- [ ] AC-3.3: The foundation's records (`arbitre` namespace) are reachable only by the foundation module.
- [ ] AC-3.4: The catalogue refuses the id `arbitre` from any module other than the foundation, with a test (ST-03 handover, RG-ST03-4.6 bullet 3).
- [ ] AC-3.5: The `Ai` port is declared in the core: its signature and its errors only (`NotConfigured`, `Unreachable`, `KeyRefused`, `Timeout`). No adapter. Every surface passes "no AI" until ST-30.
- [ ] AC-3.6: Adding a module is still one line in `modules.rs` plus one wiring file (ST-03 AC-4.5).
- [ ] AC-3.7: `arbitre-core` still builds for the browser target. The only new core dependency is the one US-8 needs.
- [ ] AC-3.8: At the end of ST-04, the contract types are frozen: `ActionDescriptor`, `Output`, `Category`, `ActionError`, `Module`, `Context`, the module store view, `Problem`, the `Ai` port, the `Store` port, `Clock` (§ Collisions). Changing one is a foundation amendment.

**Nominal cases:**
- ST-05 writes `arbitre-plan` against `Module`, adds `plan` to `modules.rs` and `wiring/plan.rs`, and touches nothing else shared.

**Identified edge cases:**
- Two modules use the same category and id for their records (`item/1`): they are two distinct records; neither sees the other's.

### US-4: Everything works offline, and nothing leaves for an undeclared destination
**As a** solo user, **I want** every function that needs no network to work with the network cut, and no data to leave towards a destination the action did not declare, **so that** my data stays on my machine unless I asked for a service (AC-1.1, RG-1.1, RG-1.2, PROP-1, PROP-2, F-18, F-37).

**Acceptance Criteria:**
- [ ] AC-4.1: The test tooling has a network observer that records every outgoing connection of the binary under test (destination, and the command that caused it). It is in place before any lane writes a network client (F-18). Its mechanism: Q5.
- [ ] AC-4.2: The observer has a cut mode: every outgoing connection fails.
- [ ] AC-4.3: Network cut, every action that declares no network (`requires_network = ∅`) runs to its own outcome without trying to connect. Strength of this check: Q6.
- [ ] AC-4.4: Network cut, every action that declares a network need ends with an explicit outcome: a code of the ST-03 table, or a named degraded mode. Never a hang, never a panic (F-37).
- [ ] AC-4.5: Network open, an action connects only to destinations matching what it declared (PROP-2 tooling). No action of ST-04 connects anywhere.

**Nominal cases:**
- The fake module's actions all run with the network cut; its relay action reports an explicit "relay not reachable/configured" tool error.

**Identified edge cases:**
- A later adapter that bypasses the observer: the observer must notice, at the latest when that adapter's own story replays PROP-2 (Q5).

### US-5: One conformance test that every module and every surface replays
**As a** development agent of any lane, **I want** one generic conformance test that runs on every module of the build and every registered surface, **so that** a new module or a new surface is checked without anyone editing the test (F-12, F-27, INV-15, 20–23, PROP-1).

**Acceptance Criteria:**
- [ ] AC-5.1: The test discovers modules and actions from the binary itself; adding a module adds it to the test with no edit.
- [ ] AC-5.2: It checks, for every module: enabling, invisibility when disabled, the "module not enabled" outcome, namespace isolation, data kept across disable and enable, any subset working alone, parity across surfaces, and AC-4.3 / AC-4.4.
- [ ] AC-5.3: Surfaces are listed in `tests/support/surfaces/`: one file per surface and one registration line. ST-04 adds `cli.rs`. A new surface (MCP in ST-07, GUI in ST-21) joins by adding its file and its line only (F-27).
- [ ] AC-5.4: A fake module, compiled only in test builds, has real data actions (create, read, update, delete, list), an action declared `relay`, and a `revalidate` that can report a problem. It passes the whole test, network cut included.
- [ ] AC-5.5: The observer, the cut mode, the fake module and the conformance test are frozen after ST-04. The only extension point is `tests/support/surfaces/` (§ Collisions).

**Nominal cases:**
- ST-05 adds `plan`; the conformance test runs every `plan.*` action network cut, on the next CI run, with no change to the test.

**Identified edge cases:**
- A module with no action at all: still enabled, disabled and listed; the per-action checks are empty.

### US-6: A problem met in the background shows up at my next command
**As a** solo user, **I want** a problem met by a background task (morning recalculation, scheduled sync) to be shown at the start of my next command, once, **so that** nothing fails silently while the tool is closed (F-27, F-14).

**Acceptance Criteria:**
- [ ] AC-6.1: A background process can leave a dated signal in the foundation namespace.
- [ ] AC-6.2: The next well-formed action shows every unread signal on standard error, in both modes, then marks them read (Q-DA2 (a) of ST-03: outside the JSON result, only for well-formed actions, after the store is opened).
- [ ] AC-6.3: A signal left by a separate process is shown at the next command, and only once.
- [ ] AC-6.4: ST-11 and ST-19 leave signals without touching `main.rs`.

**Nominal cases:**
- The morning recalculation could not read the calendar; at 9:00 `arbitre plan` starts with "signal (07:00): calendar not refreshed: token expired".

**Identified edge cases:**
- Two commands start at the same instant: a signal may show twice, but is never lost `[HYPOTHESIS]`.
- Signals of another device reach this one through sync: they are not shown here `[HYPOTHESIS]` (a signal is about the device that met the problem).

### US-7: Merged data is re-checked by its module
**As a** solo user, **I want** each module to re-check data that a merge or an import touched, **so that** a field-by-field merge that makes an item invalid is caught instead of silently kept (CONFLICT-1, R-7, F-7).

**Acceptance Criteria:**
- [ ] AC-7.1: `Module::revalidate` takes the records touched and returns the problems found. It changes nothing itself.
- [ ] AC-7.2: The hook is part of the frozen contract, with a problem shape later stories use unchanged (ST-05 marks, ST-13 asks, ST-04b and ST-19 call it). Shape and first caller: Q4.
- [ ] AC-7.3: On the fake module, a record made invalid on purpose is reported; a valid one is not.

**Identified edge cases:**
- Data merged into a disabled module is re-checked when the module is enabled (archi "Validation après fusion").

### US-8: Day limits follow my time zone, everywhere
**As a** development agent of lane P, **I want** the clock to give the user's time zone, with the time zone database built in, browser included, **so that** day limits and capacity per day (ST-09) are right on every device, without lane P changing the frozen core (ST-01 archi arbitration P-2).

**Acceptance Criteria:**
- [ ] AC-8.1: `Clock` gains `timezone()`. The native adapter returns the system time zone; the simulator's is settable.
- [ ] AC-8.2: The time zone database is embedded through `jiff`, and `arbitre-core` still builds for `wasm32-unknown-unknown` (S1-A measured +120 to +135 KB gzip).

**Identified edge cases:**
- A system time zone that cannot be read: the native adapter falls back to UTC `[HYPOTHESIS]`.

## Global business rules
- **Frozen at the end of ST-04**: the contract types of AC-3.8 and the shared test tooling of AC-5.5. Changing them is a foundation amendment.
- **Nothing changes on failure** (ST-03): "module not enabled", usage and help never create or write the store.
- **The store does not filter by enabled state**: the module registry does (archi "Activation des modules"). Data of a disabled module is kept, synchronised, exported.
- **Modules never touch each other**: at compile time by the crate graph, at run time by the store view.
- **Fully offline** (RG-1.1): enabling, disabling, listing, signals, revalidation and every fake action except the relay one need no network.

## Out of scope
- Export and import (ST-04b, F-11).
- Real modules `plan` and `diff` (ST-05, ST-22), the MCP surface (ST-07), the GUI surface (ST-21).
- Any real network client or adapter: AI (ST-30), calendar (ST-10), relay and sync (ST-19).
- Marking an invalid item and asking the user about it (ST-05, ST-13): ST-04 only returns problems.
- Background scheduling (`arbitre tache-de-fond`, ST-11): ST-04 only gives the way to leave and show signals.
- Shell completion scripts.
- The recalibration after ST-04 (a separate step of the frame plan).
- Replacing ST-03's hand-written date parser with `jiff` (ST-03 P-3 allows it; not needed).

## Hypotheses
- [HYPOTHESIS] Enabling an unknown module is a usage error: the module ids of the build are a closed list, known without data.
- [HYPOTHESIS] A process keeps the enabled state it read at start; changes apply from the next command.
- [HYPOTHESIS] `module list` names all modules of the build and their state; it is the one place a disabled module is named.
- [HYPOTHESIS] Signals are per device; a race between two commands may show a signal twice, never zero times.
- [HYPOTHESIS] An unreadable system time zone falls back to UTC.
- [HYPOTHESIS] Zones: core files of ST-04 are `Std` (Supporting zone of the plan; `CONTRIBUTING.md` forbids `Gen` under `crates/core/`); the fake module's wiring is `Gen`, like ST-03's test module. The plan writes `zones.toml`.
- [HYPOTHESIS] ST-04 needs no new shared dependency: `jiff` is already declared in `[workspace.dependencies]` (ST-01 decision).

Open questions for Quentin are at the end of `02_specs.md` (Q1 to Q7).
