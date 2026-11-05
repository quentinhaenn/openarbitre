# Skeleton — ST-02: Persistence, async `Store` port and SQLite adapter

- Date: 2026-11-04 · Branch: `st-02/store` (no commit) · Plan: `04_plan.md` v3 (DA v2 reservations integrated, Quentin's arbitrations of 2026-11-04).
- Scope: PR 2 only. PR 1 (T-1, T-2: `scripts/`, `CONTRIBUTING.md` zones) is not touched.

## Files created or modified

| File | Action | Task(s) | State |
|---|---|---|---|
| `crates/core/src/store.rs` | Created: `LocalChange`, `ClockState`, `Position` (`START`), `Watermark`, `StoreError`, `Damage`, `Cause`, `Rejection`, `ApplyError` (+ `From<StoreError>`), `Received`, trait `Store`, `rebuild`, `stamp_local`, `receive` | T-3, T-4, T-5 | Types complete; the three functions `todo!` |
| `crates/core/src/sim_store.rs` | Created (feature `sim`): `SimStore` (`new`, `full_after`, `unavailable`, `crash_next_write`, `corrupt`, `reopen`, `clock_state`), private `State`, `impl Store` | T-3, T-7 | Skeleton (`todo!`) |
| `crates/core/src/merge.rs` | Modified: `pub(crate) fn Log::with_clock` (only change) | T-4 | Complete (3-line constructor), `#[expect(dead_code)]` until T-4 calls it |
| `crates/core/src/lib.rs` | Modified: `mod store`, `#[cfg(feature = "sim")] mod sim_store`, re-exports | T-3 | Complete |
| `crates/core/Cargo.toml` | Modified: `[[test]] store`, `required-features = ["sim"]` | T-3 | Complete |
| `crates/core/tests/support/contract.rs` | Created: `Harness` trait (DA F-22 shape), `ready()` (single poll, `Waker::noop()`), `run::<H>()` | T-6 (stub) | `run` is `todo!`; T-6 owns its body |
| `crates/core/tests/store.rs` | Created: red tests of T-4, T-5, T-7, `SimHarness`, contract call site on `SimStore` | T-4, T-5, T-7 | 30 tests |
| `crates/sqlite/Cargo.toml` | Created: `rusqlite` 0.41.1 `bundled` (local), `getrandom` (workspace); dev `arbitre-core` with `sim`, `proptest`; feature `sim`; `[[test]] contract` `required-features = ["sim"]` (P-9) | T-3 | Complete |
| `crates/sqlite/src/lib.rs` | Created: `Options` (+ `Default`, `wait = 5 s`), `Settings` (four fields), `SqliteStore` (`open`, `settings`), `map` (P-8), `impl Store` | T-3, T-8, T-9 | Skeleton (`todo!`), crate-level `#![expect(dead_code)]` |
| `crates/sqlite/tests/contract.rs` | Created: includes core's `support/{mod,contract}.rs` by `#[path]`, `SqliteHarness` (temp dir per store, `reopen` = drop + open, dir removed on drop), call site | T-6, T-9 | 1 test |
| `.ctdd/stories/ST-02/zones.toml` | Created: `Full` store.rs, `Std` sim_store.rs (Q-P4), `Gen` `crates/sqlite/src/**` | T-3 | Complete |
| `Cargo.lock` | Regenerated (`rusqlite`, `libsqlite3-sys`, `getrandom` resolved) | T-3 | Complete |

## Red tests (`crates/core/tests/store.rs`)

- **T-4**: restored clock issues like the original (property, 1,000 cases); `stamp_local` list order, `a = D`, `K'` (US-4 case 6); `seen` within a list (same field, deletion after field, field after deletion); US-1 case 8; US-3 case 10; `ClockExhausted` (`c = u32::MAX`, `l ≥ L_MAX`, and on the 2nd change) with `Log` unchanged; `rebuild` damage (`StoredWrite`, `LocalIdentityPair`, peer pair = anomaly); clock check on `(l, c)` only, four DA F-2 boundaries.
- **T-5**: US-1 case 9; US-5 case 7 with batch max `<`, `=`, `>` `K`; `new` deduplicated in batch order; US-6 case 6; Q-A guard (stored local identity, two contents in the batch, unknown local identity accepted, identical re-send = duplicate); US-5 case 4.
- **T-7**: contract suite on `SimStore`; E-ST02-2 levels (a)–(d), 100 cases each, batches 1–20, `Σ` unchanged then retry = one success; US-2 case 8; `unavailable(n)`; US-6 case 8 (crash); damage poisons and survives `reopen`; Q-A rejection does not poison; US-6 case 2 (non-`Send`/non-`Sync` static check + generic `use_store`); US-6 case 3.
- `crates/sqlite/tests/contract.rs`: contract suite on `SqliteStore`.

## State

- `cargo check --workspace --all-features [--locked]`, `cargo clippy --workspace --all-features --all-targets`: green, no warning (no `unexpected_cfgs`).
- `cargo test --workspace --all-features --no-run`: green. ST-01 suites unchanged and green (hlc 20, write 12, merge 44, convergence 5).
- Red: `store` 30/30 and `contract` (SQLite) 1/1 fail, each on a `todo!` only (`T-4: rebuild` reached first by T-4/T-5 tests, `T-7: SimStore::new`, `T-6: contract suite`). The `store.proptest-regressions` file produced by these failures was deleted.
- `cargo check -p arbitre-core --target wasm32-unknown-unknown` with and without `--features sim`: green. `20-st01-browser.sh`, `30-st01-graph.sh`: green.
- `cargo tree -p arbitre-sqlite -e normal`: `arbitre-core` (without `sim`), `getrandom`, `rusqlite`/`libsqlite3-sys`; no network crate (INV-ST02-7).
- R-4: bundled SQLite of `libsqlite3-sys` 0.39.1 is **3.54.1** (≥ 3.37, `STRICT` available).

## Deviations from the plan and points for Quentin

1. **`Damage::LocalIdentityPair { target: Box<Target>, stamp }`** instead of `Target` inline: clippy `result_large_err` flagged every `Result<_, StoreError>` (≥ 128 bytes). Same fields, one allocation on a damage path.
2. **`SimStore` API additions** not named by the archi: `corrupt(damage)` (T-7 damage injection), `clock_state()` (test hook to compare `K` in `Σ`). `full_after(units)`: one unit per stored write plus one per commit, `usize::MAX` lifts the limit; this makes levels (b) "inside" and (c) "at commit" distinct budgets (`1..n` and `n`).
3. **`SimStore` is `!Send` through a `PhantomData<*const ()>` marker**: a `RefCell` alone is `Send`; the archi requires a non-`Send` adapter for the compile test.
4. **`Log::with_clock` written in full** (a 3-line constructor, not logic); it carries `#[expect(dead_code)]`, which turns into a warning to remove once T-4's `rebuild` calls it. Same mechanism (`#![expect(dead_code)]`) on `sim_store.rs` and `crates/sqlite/src/lib.rs`.
5. **Not created by the Skeleton**, per the plan's Convention: SQLite test files `settings.rs` (T-8, T-9), `crash.rs`, `interleave.rs`, `full.rs`, `damage.rs`, `support/{child,full_vfs}.rs` (harness-first tasks T-10 to T-13), the P-8 unit tests (T-8), layout v1 DDL and `application_id` constant (T-8).
6. Not run: the local judge (`judge.py main HEAD` needs a commit); to run at T-15/T-16.

## Child process protocol (`crates/sqlite/tests/support/child.rs`)

A test target declares `#[test] fn child_entry() { child::child_main() }`. The parent re-runs its own binary with `--exact child_entry` and `ARBITRE_CHILD_ROLE`; the child talks over stdin (command lines, EOF = stop) and stdout (lines prefixed `@ `; others are libtest chatter). Env: `ARBITRE_CHILD_PATH` (store), `ARBITRE_CHILD_WAIT_MS` (busy wait, default 5000), `ARBITRE_CHILD_ID`, `ARBITRE_CHILD_SEED`, `ARBITRE_CHILD_K`.

Roles and their lines (child -> parent):
- `open`: `ready`; waits `go`; `opened <hex D>` | `open_failed <dbg>`.
- `writer`: `opened`; per batch `start <b> <n>` then `confirmed <b> <dbg Vec<Write>>` (or `failed <b> <dbg>`); stops on any stdin line or EOF. Record ids are `<id>.<b>.<i>`.
- `steps`: `opened`; commands `local <pt> <id=field=value|id=-;...>` -> `ok <dbg>` | `err <dbg>`; `hold <ms>` -> `held`, `released`; `since` -> `since <dbg>`.
- `free_writer` / `free_reader`: free-running phase (RG-ST02-3.5), see `interleave.rs`.
