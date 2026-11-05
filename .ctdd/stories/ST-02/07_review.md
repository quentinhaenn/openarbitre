# Code Review — ST-02: Persistence, async `Store` port and SQLite adapter

- Branch `st-02/store` (PR #6), HEAD `ed8a846`. Diff read: `git diff origin/main...HEAD -- crates/ CONTRIBUTING.md Cargo.toml`.
- Inputs: `01_story.md`, `02_specs.md`, `03_archi.md`, `04_plan.md` v3 (with arbitrations), `05_da_review.md`, `06_skeleton.md`, `zones.toml`, `CONTRIBUTING.md`.
- Date: 2026-11-04 · Reviewer phase 8.

## Verdict: APPROVED WITH CHANGES

The implementation follows plan v3 closely and every acceptance criterion has a test. Each blocking property runs where the plan says it does. The adapter is clean: one read transaction per read, `K` never cached, one decoder shared by reads and the open-time check, a closed error mapping with a unit test per row. The required changes are small:
- the five `sim_store.rs` survivors must each get a test or a deletion (T-16 requires it);
- the SQLite handle does not apply its poison on two paths (R-1).

Neither change touches the design. One arbitration is needed (Q-R1).

## Test results (run by the reviewer, macOS)

| Command | Result |
|---|---|
| `cargo test -p arbitre-core --all-features --locked` | green: convergence 5, hlc 20, merge 44, **store 31**, write 12 (store suite 50 s) |
| `cargo test -p arbitre-sqlite --all-features --locked` | green: **49 tests, 8 binaries, 309 s** |
| `cargo check -p arbitre-core --target wasm32-unknown-unknown --features sim` | green |
| `cargo tree -p arbitre-core -e normal` | no normal dependency (INV-ST02-1) |
| Mutation (implementer's local run, not re-run) | `store.rs` 100% (Full, ≥ 85%); `sim_store.rs` 90% (Std, ≥ 70%), 5 survivors |

## Findings

| Id | Sev. | File:line | Finding | Proposed fix |
|---|---|---|---|---|
| R-1 | **Major (must fix)** | `crates/sqlite/src/lib.rs:622-624`, `:632-634` | `SqliteStore::device()` and `apply_local(&[])` skip the poison slot: once poisoned, a handle still returns `Ok`. RG-ST02-7.2 [DERIVED] says "every later operation on that handle returns `Corrupted`", and `SimStore` does that (`gate` on `device`, before the empty-list short-circuit). The two adapters of a port that freezes after ST-04 disagree, and ST-20 will copy one of them. | Check `self.poisoned` in `device()` and before the empty-list return (or route both through `exec`). Add `device` and `apply_local(&[])` to the poison assertions in `damage.rs :: damage_after_open_is_found_by_the_operation`. |
| R-2 | **Major (must fix)** | `crates/core/src/sim_store.rs:62, 145, 179, 180, 230` | 5 surviving mutants; T-16 requires a killing test or a justified equivalent for each. None is equivalent (see point 3). | 3 tests and either 2 tests or 2 deletions, depending on Q-R1. |
| R-3 | Minor | `crates/sqlite/src/lib.rs:102-114` | The open retry is bounded by `deadline`, but each `open_once` also arms `busy_timeout(options.wait)`. One attempt that starts just before the deadline can block for another full `wait`, so the worst case is about `2 × wait`. No test covers the bound. | Use `busy_timeout(deadline - now)` per attempt, or document "≤ 2 × wait". Add one test: a raw connection holds `BEGIN EXCLUSIVE` on a fresh path; `open` with `wait = 200 ms` → `Unavailable(Busy)` within `[200 ms, 400 ms + ε]`; after release, `open` succeeds. |
| R-4 | Minor | `crates/sqlite/tests/damage.rs:179-209` | After Q-P6 the truncation test still accepts `Ok(store)` at open if a later read finds the damage. The comment "a cut past every page the check reads" is stale now that `quick_check` reads every b-tree page. The plan sold Q-P6 as making this case deterministic: "at open". | Assert `Err(Corrupted(_))` at open and drop the `Ok` arm. If a seed shows a cut that loses only unallocated bytes, skip it the way all-zero tails are skipped. |
| R-5 | Minor | `crates/core/src/sim_store.rs:179`, `:230` vs `crates/sqlite/src/lib.rs:657`, `:707-711` | Behaviour differs on a **no-op write under full storage**: `SimStore` skips the charge and returns `Ok`. SQLite always runs `UPDATE device` / `INSERT OR REPLACE`, which dirties a page, so it most likely returns `StorageFull`. The contract suite never sees it because it runs without a budget. | See Q-R1. |
| R-6 | Minor | `crates/core/tests/support/contract.rs:134-160` | RG-ST02-1.4 is `@Property` and plan T-9 asks for a property over the edge domain. Generated stamps stay small (`l < 12`, `c < 3`). `l = 2^53 − 1` and `c = u32::MAX` appear only in named cases (`settings.rs :: round_trip_edge_values`, contract US-1 case 6). Names, values, `seen` keys (`Field("")` included) and authors are generated as specified. | Optional: draw `l` from `prop_oneof![0..12, L_MAX-12..L_MAX]` and `c` from `{0..3, u32::MAX-2..=u32::MAX}` in `Item::Fresh`. |
| R-7 | Minor | `crates/sqlite/tests/crash.rs:122-128` | The RG-ST02-1.5 check after a forced stop compares `state()` with a `Log` built from the **stored** writes. That is self-consistency, not equality with an independent reference. It also checks the projection only; discarded writes, conflicts and anomalies are not compared. The contract suite covers the full `R(W)` with reopen, so this is a weaker duplicate, not a hole. | Optional: build the reference from the children's confirmed batches plus the one complete unconfirmed batch, and also compare `discarded`/`conflicts`/`anomalies`. |
| R-8 | Minor | `crates/sqlite/src/lib.rs:502` | `AUTHORS.iter().position(..).unwrap_or(0)` would silently store a future `Author` variant as `CliUser`, which is silent data change. | Exhaustive `match w.author { … }`, so that a new variant breaks compilation. |
| R-9 | Nit | `crates/sqlite/tests/support/child.rs:1-15` | A 15-line module doc describing the child protocol. Under the comment rule (short what + pointer) this belongs in a doc. It is a test-protocol contract, though, not a "why". | Keep it, or move it to `crates/sqlite/tests/README.md` and leave one line plus a pointer. |

No other comment-style problem was found: the docstrings in `store.rs` (5-6 lines) state contracts of public functions, and the code comments are one line with a pointer.

## The five points

### 1. T-12, substituted red step ("ignore the commit result"): **acceptable**
- The planned red step (skip `ROLLBACK` after an error) cannot bite. `rusqlite::Transaction` rolls back on drop, and in WAL mode frames without a commit marker are invisible. The adapter has no means to leave a partial state under `SQLITE_FULL`, because once the budget is spent the commit fails too.
- What the adapter can get wrong is the **report**: a confirmation after a failed commit, or `SQLITE_FULL` mapped to something else. "Ignore the commit result" is exactly that defect at level (c). `assert!(is_full(first))` catches it, and so does the retry's "one application, no doubled write". The harness therefore proves what E-ST02-2 asks of the adapter: `StorageFull`, no confirmation, `Σ` unchanged on the live handle and after reopen, a clean retry, and a budget crossed at every level. The `Σ`-unchanged assertions mostly guard SQLite itself, which is still useful as a regression check of the layout and pragmas.
- To do: record the substitution and its reason in the PR body (it deviates from T-12's text).

### 2. T-10 / R-9 retry on `Busy`: **acceptable**, with R-3
- **Correct**: `open_once` is idempotent. A store created by the other process is seen by `inspect`, `wal()` is a no-op once WAL is set, and a store left out of WAL by a kill is switched at the next open, which answers DA F-21. Only `Unavailable(Busy)` is retried. Damage, version and I/O errors return immediately. With `wait = 0`, there is one attempt.
- **Bounded**: by `wait`, plus up to one extra `wait` for the last attempt (R-3).
- **Covered**: `crash.rs :: concurrent_first_opening_50_runs` is the test that showed R-9, so it fails without the retry. `kill_during_first_opening_50_runs` checks `journal_mode = wal` after reopen (F-21). The bound itself is not tested (R-3).
- Beyond the plan: R-9 planned "set WAL after creation, at every open, retried until `wait` elapses". The implementation retries the whole open sequence rather than only the WAL switch. That is a superset of the plan, confined to `src/lib.rs` as the collision rule requires. Fine.

### 3. `sim_store.rs` survivors: all five are **real gaps**, none equivalent
Mutant names below are inferred from line:column. Copy the exact names from `mutants.out` if an entry is ever needed.

| Line:col | Likely mutant | Why it survives | Kill |
|---|---|---|---|
| 62:17 | `>` → `>=` in `State::charge` | No test sets the budget to exactly the cost. Level (c) uses `n`, below the cost `n + 1` | Test: `full_after(n + 1)` → `apply_local` of `n` changes succeeds (and `full_after(n)` fails, as today) |
| 145:9 | `SimStore::clock_state` → `Default::default()` | Every `Sigma.clock` comparison is before/after on the same hook, so both sides read `(0, 0)`. **All "K unchanged" assertions on `SimStore` are currently vacuous** | One assertion: after `apply_local`, `s.clock_state() == ClockState { l: w.stamp.l, c: w.stamp.c }` of the last returned write |
| 180:37 | `+` → `*` (or `-`) in `apply`'s `charge(got.new.len() + 1)` | The received path is tested only at budget 0 (`us2_case8`), never at "all but the commit" | Received variant of level (c): `full_after(batch.len())` → `StorageFull`, `Σ` unchanged; `full_after(batch.len() + 1)` → `Ok`. Mirrors `full.rs`'s `received` flag |
| 179:45 | `!=` → `==` in `!got.new.is_empty() \|\| got.clock != k` | `got.clock != k` with `new` empty cannot happen: known writes are ≤ `K`, which `rebuild` checks. The only observable difference is a charge on a no-op batch under a full budget | Depends on Q-R1: (a) delete the condition and charge always → mutant gone; (b) test "duplicate or empty batch on `full_after(0)` → `Ok`" |
| 230:55 | `>` → `>=` in `advance_watermark` | An equal advance only differs by charging under a full budget | Q-R1 (a): delete the condition, `charge(1)` then insert `max` → mutant gone; (b): test "equal advance on `full_after(0)` → `Ok`" |

No entry is needed in `mutants_equivalents.toml`. After these changes `sim_store.rs` should reach 100%.

### 4. Deferred F-13 (`Cause::RandomSource` untested): **acceptable**
- It is one `map_err` on `getrandom::fill` (`lib.rs:317`), reached only at creation. On the supported targets (Linux `getrandom(2)`, macOS `getentropy`) it practically never fails.
- A seam would mean a new parameter or hook on `open` for one branch. The plan (DA F-13) arbitrated this as not worth it, and the DA accepted it.
- Condition: the PR body must state it, as T-16 requires. `Cause::RandomSource` stays in the port enum for ST-20.

### 5. Q-P6 (`PRAGMA quick_check` at open, `23fe4bf`): **correct**, with R-4
- **Placement**: it is last in `inspect`, inside the same read transaction as the `K` check and `check_stored`, so row damage keeps its precise variant (`StoredWrite`, `ClockState`, `LocalIdentityPair`). Damage that only the engine sees gives `Corrupted(Engine(report))`. `report != ["ok"]` handles multi-row output.
- **No write on refusal**: `wal()` runs only after `inspect` succeeds, and `damage.rs` asserts the file is byte-identical for every case at rest.
- **Not on creation**: the `create` path skips `inspect`, as intended (empty store).
- **Cost**: 63 ms open check including 12.5 ms `quick_check` at 10,000 writes on macOS, inside the 100 ms budget. The Linux CI figure is still owed by T-15.
- **Test**: the new truncation test cuts anywhere and skips all-zero tails. It is red without `quick_check`, as stated. It still tolerates "open OK, later read fails" (R-4).

## Spec coverage (AC → test)

| AC / rule | Status | Test |
|---|---|---|
| AC-1.1 RG-ST02-1.1 | ✅ | `settings.rs :: settings_on_fresh_and_reopened_handles` (WAL, FULL, fullfsync, checkpoint_fullfsync) |
| AC-1.2/1.3/1.4 PROP-ST02-2 | ✅ | `crash.rs` 200 + 50 runs, R-6 kill bias, torn-batch and lost-write checks |
| AC-1.5 PROP-ST02-3 | ✅ (R-6) | contract suite (generated names/values/`seen` keys), US-1 cases 6, 6b; `settings.rs :: round_trip_edge_values` |
| AC-1.6 PROP-ST02-4 | ✅ (R-7) | contract suite with `Reopen` op and `Report` compared; `crash.rs` projection |
| AC-1.7 PROP-ST02-5 | ✅ | `store.rs` US-1 cases 8, 9, `(l, c)` boundaries; contract write at new `K`; `damage.rs` clock below stamp; `crash.rs` next stamp |
| AC-2.1–2.4 PROP-ST02-6 | ✅ | `full.rs` levels (a)–(d) × 100, local and received, same handle and reopen, `xSync` failure, first opening, reads when full; `store.rs` `full_after` levels |
| AC-3.1/3.2/3.5 PROP-ST02-7 | ✅ | `interleave.rs :: stepped_100_schedules` (`pt` from a small domain with rewinds, sequential reference, distinct increasing stamps) |
| AC-3.3 PROP-ST02-8 | ✅ | `settings.rs` busy 2δ / δ/2 / retry / `wait = 0`; `interleave.rs` held 2δ / δ/2 |
| AC-3.4 RG-ST02-3.5 | ✅ | `interleave.rs :: free_running_10_seeds_2s`; structural rule holds (one `Deferred` per read, `lib.rs:613-618`) |
| AC-4.1–4.3 PROP-ST02-10 | ✅ | `settings.rs` create/reopen, 1,000 distinct; `crash.rs` concurrent × 50, kill during creation × 50; `full.rs :: full_at_first_opening` |
| AC-5.1–5.4 PROP-ST02-9 | ✅ | contract suite on both adapters + named US-5 cases, `Position(u64::MAX)` |
| AC-5.5 RG-ST02-5.5 | ✅ | `store.rs :: unavailable_n_operations_then_back`; `damage.rs` directory at path, per-operation damage |
| AC-6.1 INV-ST02-1 | ✅ | wasm32 check with and without `sim`; no normal dependency |
| AC-6.2/6.3 INV-ST02-2 | ✅ | `store.rs` US-6 cases 2, 3 (negative `Send`/`Sync` check) |
| AC-6.4 PROP-ST02-11 | ✅ | `settings.rs :: every_operation_is_ready_on_first_poll`; busy tests use single-poll `ready` |
| AC-6.5 INV-ST02-3 | ✅ | `merge.rs` diff = `with_clock` only; `hlc/write/clock.rs` untouched |
| AC-6.6 INV-ST02-4 | ✅ | P-8 unit tests per row; rejections on core, `SimStore`, SQLite; Q-A guard on both adapters |
| AC-6.7 INV-ST02-5 | ✅ | `rusqlite` local to `arbitre-sqlite`; graph check (PR 1) |
| AC-6.8 RG-ST02-6.7 | ✅ (R-2) | `store.rs` contract on `SimStore`, `full_after`, `unavailable`, `crash_next_write`, `corrupt` + `reopen` |
| AC-7.1–7.4 PROP-ST02-12, INV-ST02-6 | ✅ (R-1, R-4) | `settings.rs` `V = 1`, schema, newer version untouched; `damage.rs` at rest / after open / two-process / constraint rollback; random names in contract suite |
| RG-ST02-4.1 "random source unavailable" | ⚠️ deferred | F-13, point 4 |

## Plan adherence

| Task | Status | Note |
|---|---|---|
| T-3 to T-9 | ✅ | As planned. Skeleton deviations (boxed `LocalIdentityPair`, `corrupt`/`clock_state`, `PhantomData` marker) recorded in `06_skeleton.md` |
| T-10 | ✅ | R-9 triggered and fixed as a separate `src/lib.rs` commit (`3d2c1c9`), following the collision rule |
| T-11 | ✅ | Stepped + free-running phases at spec counts |
| T-12 | ✅ | Red step substituted (point 1); record it in the PR body |
| T-13 | ✅ | P-10, P-11 (orphan check is broader than planned: any `pos` not in `writes`, which is better), two-process test, timing probe |
| T-14 | ✅ | Glossary rows only |
| T-15 / T-16 | pending | CI timings (Q-P1), Linux open-check figure (Q-P5 budget), PR body items: T-12 substitution, P-7, P-10/P-11, F-13 |

## Error model and `no_std`
- `StoreError` is three variants; `Rejection` stays separate (CONFLICT-ST02-2). `map` is exhaustive with a catch-all to `Unavailable(Io)`. `exec` poisons only on `Corrupted` and rolls back on every error.
- `store.rs` and `sim_store.rs` use `alloc`/`core` only (`RefCell` from `core::cell`); the core builds for wasm32.

## DA findings addressed

| Finding | Status |
|---|---|
| F-18 (free-running red step) | ✅ Retargeted red steps; tag and `==` checks present |
| F-19 (Q-P5) | ✅ Open-time check in every process; `SimStore` damage survives `reopen` |
| F-20 (orphan `seen`) | ✅ Detected at open (broader query) + colliding case rolled back |
| F-21 (WAL at every open) | ✅ `wal()` at every open; checked after kill during creation |
| F-22 (`Harness` shape) | ✅ Exactly as pinned |
| F-13 (`RandomSource`) | ⚠️ Deferred, acceptable (point 4) |

## Arbitration questions for Quentin

**Q-R1 — A no-op write on full storage (`apply` of a batch already stored or empty; `advance_watermark` to a lower or equal mark): `Ok` or `StorageFull`?** The spec does not say: RG-ST02-5.3/5.4 say "no change", and RG-ST02-2.1 says "full ⇒ `StorageFull`, `Σ` unchanged". `SimStore` answers `Ok`; SQLite always writes, so it most likely answers `StorageFull`. This decides 2 of the 5 survivors (point 3) and the port's semantics before the freeze.
- (a) `StorageFull` on both. In `SimStore`, delete the two conditions (always `charge`) to match SQLite. This removes mutants 179:45 and 230:55, with no new test and less code.
- (b) `Ok` on both. SQLite skips `set_clock` when `K` is unchanged and skips the watermark write when the mark does not grow; one fault-VFS case is added per path, plus the two `SimStore` tests.
- **Recommendation: (a).** Both results satisfy the spec, and (a) deletes code instead of adding a branch to the adapter that ST-20 will copy. A caller that gets `StorageFull` on a re-sent batch loses nothing and may retry.

## Resolution (branch `st-02/review-fixes`)

| Finding | Resolution |
|---|---|
| R-1 | Fixed: `check_poison()` in `device()` and before the empty `apply_local` return; both added to `damage.rs :: damage_after_open_is_found_by_the_operation`, observed red without the fix. Commit "R-1 ... R-8" |
| R-2 | Fixed: 62:17, 145:9, 180:37 killed (tests in `store.rs`, mutants run: 6/6 caught). 179:45 and 230:55 killed by the Q-R1 (b) test. Commits "R-2 ..." and "Q-R1 b" |
| R-3 | Fixed: per-attempt `busy_timeout(deadline - now)`. `settings.rs :: open_busy_is_bounded_by_wait` guards the bound but does not prove the fix (the old worst case, about 2 x wait, is inside its margin). Commit "R-1 ... R-8" |
| R-4 | Fixed: truncation must be refused at open. Commit "R-1 ... R-8" |
| R-5 / Q-R1 | Arbitrated **(b)**: a no-op write on full storage returns `Ok` on both adapters. SQLite writes no page when row contents are unchanged, so `xWrite` is never reached and the fault never fires (observed with the fault VFS, `set_budget(0)`); forcing a write only to fail would be artificial. `SimStore` conditions kept; contract line in the `Store` trait doc; tests on both adapters |
| R-6, R-7 | Not done (optional) |
| R-8 | Fixed: exhaustive `match` on `Author` in `insert`. Commit "R-1 ... R-8" |
| R-9 | Fixed: 3-line header in `child.rs`, protocol moved to `06_skeleton.md`. Commit "R-9 ..." |
