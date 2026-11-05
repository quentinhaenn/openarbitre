# Skeleton — ST-04: Modules you can enable, kept apart, with one generic conformance test

- Date: 2026-11-05 · Branch: `st-04/story` (on `st-03/story`, PR #8 not merged; rebase on `main` once it lands) · Plan: `04_plan.md` v2, task T-1.

## Approach

Unlike ST-03, the running pipeline is kept: `main.rs` now dispatches to the foundation (`wiring/arbitre.rs`) or to a `Module` through `Context`/`ModuleStore`, so the ST-03 suite stays green. Only the new behaviour (pre-parse reading, enabled state, `module.*`, fake actions, signals, time zone, observer, conformance helpers) is stubbed with `todo!("T-n …")`. T-4 wires the pre-parse steps into `main.rs`.

## Files created or modified

| File | Action | Task(s) | State |
|---|---|---|---|
| `crates/core/src/module.rs` | Created: `LocalBoxFuture`, `Problem`, `Module<S>`, `Context`, `ModuleStore` (private fields, `new`, `module`, `compile_fail` doctest), `PreScan`, `pre_scan`, `state_record`, `enabled_from`, `state_change` | T-2 | Types complete; view operations and helpers `todo!` |
| `crates/core/src/ai.rs` | Created: `Ai`, `AiError`, `From<AiError> for ActionError` | T-2 | Mapping `todo!` |
| `crates/core/src/signal.rs` | Created: `Signal`, `signal_record`, `signal_changes`, `read_change`, `unread`, `line` | T-6 | `todo!` |
| `crates/core/src/clock.rs` | `Clock::timezone`; `SimClock` field `tz`, `set_timezone` | T-3 | `todo!` |
| `crates/core/src/catalog.rs` | `Catalog::build(foundation, modules)` | T-2 | Old checks run on foundation + modules; reserved and ill-formed ids are T-2 |
| `crates/core/src/lib.rs`, `Cargo.toml` | `ai`, `module`, `signal`; `jiff` (`alloc`, `tzdb-bundle-always`); tests `module`, `timezone`, `signal` (feature `sim`) | T-1 | Complete |
| `crates/cli/src/modules.rs` | Rewritten: `Modules`, `all()` (one line per module), `ids`, `catalog`; `foundation` = `wiring/arbitre.rs` | T-1, T-4 | Complete |
| `crates/cli/src/wiring/arbitre.rs` | Created: `Env`, descriptor helpers, `descriptors(toggleable)` (`enable`/`disable` only with a toggleable module), `execute` | T-4, T-5 | Descriptors written; `switch`, `list_modules` `todo!` |
| `crates/cli/src/wiring/arbitre_test.rs` | From `wiring/test.rs`: actions renamed `arbitre.test.*`, `test.not-enabled` removed, `test.signal` added, records in namespace `arbitre` | T-1, T-6 | Bodies kept; `test.signal` `todo!` |
| `crates/cli/src/wiring/test.rs` | Deleted | T-1 | — |
| `crates/cli/src/wiring/fake.rs` | Created: `Fake`, `modules()` (`fake-a`, `fake-b`), descriptors | T-5 | `execute`, `revalidate` `todo!` |
| `crates/cli/src/clock.rs` | Created: `SystemClock` | T-3 | `now` written (system time); `ARBITRE_TEST_NOW` and `timezone` are T-3 |
| `crates/cli/src/signals.rs` | Created: `deposit`, `show_and_mark` | T-6 | `todo!` |
| `crates/cli/src/store.rs` | `open_existing` | T-4 | `todo!` |
| `crates/cli/src/main.rs` | `module_of`, dispatch foundation / module, unit test | T-1 | Working; pre-parse steps are T-4 |
| `crates/cli/Cargo.toml` | `jiff` (`std`, `tz-system`); tests `modules`, `signals`, `network`, `conformance` | T-1 | Complete |
| `crates/cli/tests/support/{network.rs, conformance.rs, surfaces/{mod.rs, cli.rs}}` | Created: `Observer` (`start`, `env`, `take`), `Env`, `Called`, `Surface`, `all()`, `Cli`, conformance helpers | T-7 | `todo!` |
| `crates/core/tests/catalog.rs` | Helper splits the foundation for the new `build` | T-1 | Green |
| `crates/cli/tests/codes.rs`, `output.rs` | "Module not enabled" row → `fake-a list`; help-order list without `not-enabled`, with `signal` | T-1 | Row red until T-5 |
| `.ctdd/stories/ST-04/zones.toml` | Created (plan v2 § Zones) | T-1 | Complete |
| `Cargo.lock` | Regenerated: `jiff`, `jiff-core`, `jiff-tzdb`, `jiff-static`, plus optional entries the lock lists for `jiff` (`defmt`, `log`, `portable-atomic`, a second `syn`); none is built for `arbitre-core` | T-1 | Complete |

## Red tests

| Task | File | Red | Red on |
|---|---|---|---|
| T-2 | `core/tests/module.rs` | 8: view namespace and stamping (E-ST04-9), pre-scan table, state record, `enabled_from`, `state_change`, reserved / ill-formed ids, `AiError` ids | `todo!` in `module.rs`, `ai.rs`; missing refusals in `catalog.rs` |
| T-3 | `core/tests/timezone.rs` | 1: `SimClock` UTC then set | `todo!` |
| T-6 | `core/tests/signal.rs` | 3: record id, unread order and filter, local-time line | `todo!` |
| T-4, T-5 | `cli/tests/modules.rs` | 11: matrices US-1, US-2 (non-store files, `--store` after the word), US-3, US-7, relay action | `todo!` in `switch`; fake actions; no filter yet |
| T-6 | `cli/tests/signals.rs` | 4: once on stderr, not on help/usage, order, `TZ` case | `todo!` in `test.signal` |
| T-7 | `cli/tests/network.rs` | 3: variables, cut, open | `todo!` in `Observer` |
| T-7 | `cli/tests/conformance.rs` | 7: checks 1–7, fake CRUD offline, minimal input | `todo!` in helpers / `module.enable` |
| T-5 | `cli/tests/codes.rs` | 1: five categories (row `fake-a list` → 4) | fake actions `todo!` |

**Total: 38 red** (37 new, 1 moved ST-03 row). **New and already green: 3** (unknown module ⇒ usage through the derived enum; the named zone resolves from the bundled database; the no-action helper case), plus the `compile_fail` doctest. ST-01, ST-02 and the rest of ST-03: green.

## Verification

- `cargo build --workspace --all-features --all-targets` and without features: green.
- `cargo fmt --all`; `cargo clippy --workspace --all-targets -- -D warnings`, with and without `--all-features`: green.
- `20-st01-browser.sh`, `30-st01-graph.sh`: green. `cargo tree -p arbitre-core -e normal`: `jiff`, `jiff-core`, `jiff-tzdb` only.
- `cargo test --workspace --all-features --no-fail-fast`: every red test fails on a `todo!` of its task, or on an assertion the task makes true (help still lists `fake-a` before T-4).

## Decisions taken here

1. **`module list` gives schemas as canonical JSON text** (`input`, `output` strings): a JSON Schema has no fixed shape, so it cannot be described in the frozen schema subset.
2. **`Observer::take()` instead of `records()`**: it drains, so each command's attempts are read right after it runs. This replaces the plan's `argv` tag (the proxy cannot see which process connects). The rest of the API is unchanged.
3. **`PreScan` has no `json` field**: `main.rs`'s existing `json_mode` stays the only JSON pre-scan.
4. **`revalidate` takes `(category, id)` pairs**, not `RecordId`s: the caller cannot name another namespace (same rule as the view).
5. **`SystemClock::now` written now** (it was in ST-03's `main.rs`); only `ARBITRE_TEST_NOW` and `timezone` are T-3.
6. **Stub parameters are used in the `todo!` messages**, so no `allow(unused)` was added. Two `expect(dead_code, reason = …)` mark `open_existing` (T-4) and `signals::*` (T-6).
