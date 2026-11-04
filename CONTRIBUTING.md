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

## `Cargo.lock`

On a `Cargo.lock` conflict, never merge it by hand: take `main`'s (`git checkout origin/main -- Cargo.lock`), then run `cargo check --workspace` to regenerate it.

## Dependencies

- Shared dependencies and their declaration are in `docs/decisions/ST-01-dependencies.md`. A component inherits them with `workspace = true`, without repeating the version.
- A new shared dependency, or a tool version bump (`scripts/tools.env`, `rust-toolchain.toml`), is a foundation amendment.

## Verification: `scripts/ci.sh`

- Single entry point, locally and in CI. It runs every `scripts/ci.d/*.sh`, sorted bytewise (`LC_COLLATE=C`). A failing check makes the verdict `red`.
- **Adding a check**: a file `scripts/ci.d/NN-stxx-<subject>.sh`, with `NN ≥ 60`. Its helper code goes in `scripts/lib/`, never in `ci.d/`. The `*-st01-*` files and `ci.sh` do not change.
- Such an addition goes through **a dedicated PR with the `ci-change` label** (accepted deviation from AC-6.6, see the judge).

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
