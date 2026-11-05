# Contributing to arbitre

The rules below come from ST-01 (`.ctdd/stories/ST-01/`). Changing them is a **foundation amendment**.

## AI-assisted contributions

This project was developed with generative AI, and AI-assisted contributions are welcome on three conditions:

1. The change passes every quality gate: `ci` (tests, mutation testing) and `judge`.
2. You can explain the code you submit and the main trade-offs it makes, as if you had written it yourself.
3. For critical components (zone `Full`: the merge engine, cryptography, security), you can also explain the implementation details.

You remain responsible for the code you submit, whoever or whatever wrote it.

## Branches and PRs

- `main` is protected: every change goes through a PR, `.ctdd/` included.
- A PR is verified by two checks: `ci` (`scripts/ci.sh`) and `judge` (`scripts/lib/judge.py`).

## Language

Everything public in this repository is written in English: code, comments, messages, docs. The ST-01 cycle documents in `.ctdd/` are kept in French as history; they are not translated. The glossary below maps their terms to the names used in the code.

| Spec term (French, `.ctdd/stories/ST-01/`) | Code name |
|---|---|
| journal | `Log` |
| écriture | `Write` (fields `record`, `change`, `stamp`, `seen`, `author`) |
| donnée / `IdDonnee` | record / `RecordId` (`module`, `category`, `id`) |
| quoi | `Change` (`Field { name, value }` or `Delete`) |
| visée | `TargetKind` (`Delete` or `Field(Field)`) |
| cible | `Target` (`record`, `kind`) |
| vu | `Seen` (table), field `seen` |
| marque | `Stamp` `(l, c, a)` |
| horloge (hybride) | `HybridClock` (`new`, `issue`, `receive`) |
| horloge épuisée | `ClockExhausted` |
| horloge simulée | `SimClock` (feature `sim`; `set`, `advance`, `rewind`, `freeze`) |
| appareil / `IdAppareil` | device / `DeviceId` |
| auteur | `Author` (`CliUser`, `UiUser`, `Assistant`, `System`) |
| lot | batch; `BatchError`, `validate` |
| fusion / fusionner | merge / `Log::merge`, returns a `Report` |
| écrire (écriture locale) | `Log::write` |
| projection, état projeté | `Log::projection`, `RecordState`, `Status` (`Present`, `Deleted`) |
| retenue / gagnante | retained / winner |
| écartée (version écartée) | discarded (superseded version), `Log::discarded` |
| conflit | `Conflict` (`retained`, `discarded`), `Log::conflicts` |
| anomalie | `Anomaly`, `Log::anomalies` |
| rapport | `Report` (`new_discarded`, `new_conflicts`, `new_anomalies`) |
| avance, seuil σ | `Lead`, `Threshold`, `leads()`, `DEFAULT_THRESHOLD` |
| liste navigateur | browser list |
| juge | judge |
| rouge / vert | red / green |
| magasin, port `Store` | store, `Store` |
| appliquer (écriture locale) | `Store::apply_local` |
| `Store::appliquer` (lot reçu) | `Store::apply` |
| `ecritures_depuis`, position | `Store::writes_since`, `Position` |
| filigrane (par pair) | `Watermark`, `Store::watermark`, `Store::advance_watermark` |
| état d'horloge | `ClockState` |
| `StockagePlein` / `Corrompu` / `Indisponible` | `StoreError::StorageFull` / `Corrupted` / `Unavailable` |
| action | `ActionDescriptor` (name, summary, input and output schema, `mutating`, `logged_as`, `requires_network`); one operation of a module, exposed by every surface |
| action name, command path | dot-separated name, first segment = module id; command path = its words after `arbitre` (the reserved `arbitre` segment dropped) |
| group | a command path that is a strict prefix of an action's path |
| catalogue | `Catalog`: the validated descriptors of every wired module, built once per process |
| schema, property | `Schema` (JSON Schema subset, RG-ST03-4.3), `Property` (one command-line option in an input) |
| valeur structurée | `Value` (`Bool`, `Int`, `Str`, `List`, `Object`; no null, no float), `arbitre_core::Value` (re-exported from `value`); the ST-01 raw field value is `FieldValue` |
| JSON canonique | `encode`: sorted keys, no whitespace, fixed escaping |
| sortie | `Output` (= `Value`), human text from `render` |
| catégorie | `Category` (`Success`, `Refusal`, `Usage`, `ToolError`, `ModuleNotEnabled`), `exit_code`, `json_name` |
| erreur d'action | `ActionError` (`category`, `id`, `message`, optional `data`) |
| identifiant d'erreur | error id: stable `snake_case` string (`unknown_option`, `storage_busy`) |
| document résultat | result document: the one JSON object of `--json`, built by `document` |
| mode JSON | JSON mode: a command run with `--json` |
| emplacement du magasin | store location: `--store`, `ARBITRE_STORE`, platform default |
| module de test | test actions: foundation actions `arbitre.test.*` (command `arbitre test …`), feature `test-actions`, never shipped |
| module (ST-04) | `Module<S: Store>`: `id`, `actions`, `execute`, `revalidate`; registered in `crates/cli/src/modules.rs` |
| foundation module | the actions under id `arbitre` (`arbitre.module.*`, signals); always on, not a `Module` impl, not listed in `modules.rs` |
| toggleable module | every module of the build other than the foundation |
| enabled state | the toggleable modules enabled on this device: record `arbitre/module-state/<device hex>`, one field per module id, `1` on, `0` off |
| namespace | the `module` part of a `RecordId`; a module's namespace is its id |
| module store view | `ModuleStore`: the store as one module sees it, `(category, id)` only |
| context | `Context` (`author`, `clock`, `store`: the view, `ai`) |
| problem | `Problem` (`record`, `id`, `message`): one finding of `revalidate` |
| signal | dated note for the next command on this device: record `arbitre/signal/<device hex>-<ms>-<source>`, fields `at`, `source`, `message`, `read`; `signals::deposit` |
| fake module | `wiring/fake.rs`, instances `fake-a` and `fake-b`, feature `test-actions`, never shipped |
| surface | a way to reach actions (CLI now, MCP, GUI); `Surface` in `crates/cli/tests/support/surfaces/` |
| network observer, cut mode | `Observer`, `Mode::{Open, Cut}` (see "Network clients") |
| `Ai` port | `Ai::complete`, `AiError` (`NotConfigured`, `Unreachable`, `KeyRefused`, `Timeout`) |
| conformance test | `crates/cli/tests/conformance.rs`: replayed on every toggleable module and every surface |
| time zone | `Clock::timezone` (`jiff::tz::TimeZone`) |

## `Cargo.lock`

On a `Cargo.lock` conflict, never merge it by hand: take `main`'s (`git checkout origin/main -- Cargo.lock`), then run `cargo check --workspace` to regenerate it.

## Dependencies

- Shared dependencies and their declaration are in `docs/decisions/ST-01-dependencies.md`. A component inherits them with `workspace = true`, without repeating the version.
- A new shared dependency, or a tool version bump (`scripts/tools.env`, `rust-toolchain.toml`), is a foundation amendment.

## Verification: `scripts/ci.sh`

- Single entry point, locally and in CI. It runs every `scripts/ci.d/*.sh`, sorted bytewise (`LC_COLLATE=C`). A failing check makes the verdict `red`.
- **Adding a check**: a file `scripts/ci.d/NN-stxx-<subject>.sh`, with `NN ≥ 60`. Its helper code goes in `scripts/lib/`, never in `ci.d/`. The `*-st01-*` files and `ci.sh` do not change.
- Such an addition goes through **a dedicated PR with the `ci-change` label** (accepted deviation from AC-6.6, see the judge).
- **Lint (`60-st02-lint.sh`)**: `cargo fmt --check`, `clippy -D warnings` on every target, and a cognitive-complexity gate (threshold in `clippy.toml`) on production code only. There is deliberately no function-length limit: a clean 80-line function beats three unreadable 20-line ones.
- **Mutation** runs 3 mutants in parallel (`--jobs 3`, the CI runner has 4 cores).

## The judge (`judge.yml`)

- It runs on `pull_request_target`, from `main`'s code. It reads the PR as data (`git diff`, `git show`) and executes none of it. **`judge.yml` never checks out the PR head** into the working tree: otherwise PR code would run with the base's privileges.
- It returns `red` if the PR:
  - touches a path under `.github/` or `scripts/`, whether added, modified or deleted;
  - lowers the effective zone of an existing `.rs` file (see "Zones").
- With the `ci-change` label, these lists are only displayed, but the PR must not touch any path under `crates/`.
- **The `ci-change` label is set by Quentin alone, never by an agent.**

## Zones: `.ctdd/stories/ST-xx/zones.toml`

```toml
Full = ["crates/core/src/hlc.rs"]   # mutation threshold 85%
Std  = ["crates/core/src/clock.rs"] # threshold 70%
Gen  = ["crates/cli/src/**"]        # not mutated in CI, no threshold
```

- Top-level keys: `Full`, `Std`, `Gen`, each a list of patterns. Any other key, another type or unreadable TOML ⇒ `red`.
- Patterns are relative to the root, with `PurePosixPath.full_match` semantics (Python ≥ 3.13): `*` stays within a segment, `**` covers zero or more segments.
- All declarations of all stories are read. A file's zone is the strictest that covers it (`Full > Std > Gen`); an uncovered file gets `Full`.
- A file of `crates/core/` or `crates/plan/` in `Gen` ⇒ `red`.
- `Gen` files are not mutated in CI: `50-st01-mutation.sh` excludes them (same resolution as the judge). Local run: `cargo mutants --all-features -f 'crates/sqlite/src/**'`.

## Equivalent mutants: `.ctdd/stories/ST-xx/mutants_equivalents.toml`

```toml
[[equivalent]]
mutant = "crates/core/src/hlc.rs:42:9: replace < with <= in HybridClock::issue"
justification = "why no observable behaviour changes"
```

- `mutant` is the exact name given by cargo-mutants, copied from `mutants.out`. If the mutant moves to another line, the entry no longer matches and CI goes red: update it.
- Empty or missing justification, or unknown key ⇒ `red`.
- **This is the only exclusion channel**, besides the automatic exclusion of `Gen` files (see "Zones"). Forbidden, and making `50-st01-mutation.sh` red: the `mutants::skip` attribute in `crates/**/*.rs`, any `mutants.toml` file in the repository (hence `exclude_re` and `exclude_globs`), and `--exclude` outside the command line of `50-st01-mutation.sh`.

## Secret false positives: `.gitleaksignore`

Native gitleaks format, one fingerprint per line, **each preceded by a `# justification` line**. A fingerprint without justification ⇒ `red`.

## Merge engine (`arbitre-core`)

- `arbitre-core` is `#![no_std]` + `alloc`: no system time, no I/O.
- A local write goes **only** through `Log::write`, which computes `seen` and stamps the write. `Log::merge` is reserved for received batches.
- **Deserialization contract (ST-02 onwards)**: the round trip of a `Write` is exact, `seen` included. Otherwise a re-emitted copy is an identity with two contents, hence an anomaly.

## Frozen contract (ST-04)

Since ST-04 (RG-ST04-G.2), changing any of the following is a **foundation amendment**:

- Rust types: `ActionDescriptor`, `Schema`, `Property`, `Value`, `Output`, `Category`, `ActionError`, `Module`, `Context`, `ModuleStore`, `Problem`, `Ai`, `AiError`, `Store`, `StoreError`, `Clock`. `Clock::timezone` returns a `jiff::tz::TimeZone`, so `jiff` 0.2 is part of the contract: a major `jiff` upgrade is an amendment.
- Record layouts: the enabled state `arbitre/module-state/<device hex>` and the signal `arbitre/signal/<device hex>-<ms>-<source>` (see the glossary).
- Test tooling under `crates/cli/tests/support/` and `crates/cli/tests/conformance.rs`: the observer API (`Observer::start(Mode::{Open, Cut})`, `env()`, `take()` returning the `Attempt { host, port }` seen since the last call), the surface contract (`Env`, `Called`, `Surface`), the conformance helpers and checks, and the fake module. They change only as described in "Adding a surface" and "Network clients".
- Error ids may be added, never removed.

Other public items of `arbitre-core` (`pre_scan`, `enabled_from`, `state_change`, `state_record`, `unread`, `line`) are not frozen.

## Adding a module

1. One file `crates/cli/src/wiring/<id>.rs` that implements `Module<SqliteStore>` (`wiring/fake.rs` is the example).
2. One line in `all()` of `crates/cli/src/modules.rs` that adds it, plus its `mod` declaration.

Nothing else: the help, `module enable`'s choices, `module list` and the conformance test pick it up. The id has the name-segment form and must not be `arbitre`, `module` or `test` (the foundation's id and command words); the whole build is checked at every start and a bad id exits 3 `catalog_invalid`. A module reaches only its own namespace, through `ctx.store`.

## Adding a surface

One file `crates/cli/tests/support/surfaces/<name>.rs` that implements `Surface`, and one line in `all()` of `surfaces/mod.rs`. `actions()` must read what the user sees (CLI: `--help`; MCP: `tools/list`), not the catalogue. No other test file changes.

The conformance test then runs, per surface, checks 1 and 2 (actions visible when enabled, gone when disabled) and 7 (same actions and same result document as the other surfaces). Checks 3 to 6 and the explicit call of a disabled module (exit 4) run on the CLI only.

## Pinned time: `ARBITRE_TEST_NOW`

In a build with `test-actions`, `ARBITRE_TEST_NOW=<ms since the epoch>` replaces the system time of the binary's clock; the time zone still comes from the system (`TZ`). The conformance test sets it in `Env.vars` so that comparisons between runs and surfaces are byte-exact. Shipped builds ignore it.

## Network clients

- Tests observe the network through `Observer` (`crates/cli/tests/support/network.rs`): a local proxy that every spawned process reaches through `HTTP_PROXY`, `HTTPS_PROXY` and `ALL_PROXY` (both cases), with `NO_PROXY` empty. It records each attempt (host, port); call `take()` after each command to get that command's attempts.
- **Cut mode refuses connections at the proxy; it does not cut the network.** A client that ignores the proxy variables (raw TCP, its own DNS, some WebSocket stacks) goes around the observer and is neither recorded nor refused. Such a client needs its own observation support before its story can pass.
- So no network client may disable the proxy settings, and each adapter story asserts in its own `tests/<adapter>.rs`, through `Observer`, that its traffic **is** recorded.
- In open mode, the conformance test accepts only hosts listed in `ALLOWED` (`crates/cli/tests/conformance.rs`) for a network kind the action declares. An adapter story adds its `(kind, host)` rows there; that is the only change it makes to the file.
