# Devil's Advocate Review — ST-02: Persistence, async `Store` port and SQLite adapter

- Reviewed: `.ctdd/stories/ST-02/04_plan.md` (DRAFT v1), against `01_story.md`, `02_specs.md`, `03_archi.md` and their "Quentin's arbitration(s) (2026-11-04)" sections, which are binding.
- Code read: `crates/core/src/{merge,hlc,write,lib}.rs`, `crates/core/Cargo.toml`, `crates/core/tests/support/mod.rs`, `Cargo.toml`, `CONTRIBUTING.md`, `scripts/ci.sh`, `scripts/ci.d/*.sh`, `scripts/lib/{mutation,judge,graph}.py`, `.github/workflows/{ci,judge}.yml`, `.ctdd/stories/ST-01/zones.toml`.
- Iteration 1 of 3 · Date: 2026-11-04.
- Open questions Q-P1 and Q-P2 are taken at their recommendations ((b) and (a)). Neither changes the verdict; see "Effect of Q-P1 / Q-P2".

## Overall verdict
**BLOCKED.**
The plan is careful and follows the arbitrations closely: two PRs, Q-M → (e), Q-A guard, one persisted clock in `BEGIN IMMEDIATE`. One defect in the persisted layout v1 makes RG-ST02-1.4 false for writes that pass `validate`, and the planned generator is not stated to cover them (F-1). Several of the planned "red first" steps would not fail for the reason the plan gives, or would be hidden by handle poisoning (F-4, F-5). Fix F-1 before the Skeleton: v1 is a frozen on-disk format, so fixing it later needs a migration.

## Findings

### [B] F-1: The `seen` table cannot store an entry keyed by an empty field name, which `validate` accepts
- **Axis**: Completeness / Integrity
- **Source**: specs RG-ST02-1.4 (pre-condition "any write `w` that passes `validate`… including extra entries allowed by A-1"); archi "SQLite layout v1", `seen.field BLOB (x'' = TargetKind::Delete)` and "empty field names are invalid, so `''` is unambiguous"; code `crates/core/src/write.rs::validate` (checks `seen` **values** only, never `seen` keys), `identifiers_non_empty` (checks the change's field name only).
- **Tasks**: T-8 (DDL), T-9 (round trip), T-6 (generators).
- **Description**: `Seen = BTreeMap<TargetKind, Stamp>`. A received write may carry `seen[Field(Field(""))]` and still pass `validate`. Layout v1 encodes `Delete` as `field = x''`, so such an entry: (i) reads back as `Delete`, which breaks the `==` round trip and changes conflict detection (`conflicts_of` reads `seen` by kind); or (ii) if the same write also has a `Delete` entry, it collides with `PRIMARY KEY (pos, field)`. The collision gives `SQLITE_CONSTRAINT`, which is missing from the error-mapping table, so a valid peer batch can never be applied. The archi's "unambiguous" claim holds for `writes.field` but not for `seen.field`. T-9's generator says only "0–3 `seen` entries", without saying which keys, so the property can pass with the defect present.
- **Proof**: `validate` in `write.rs` lines `w.seen.values().any(|m| *m >= w.stamp)`; no key check exists anywhere in the core.
- **Fix**: make the `seen` key unambiguous in layout v1. For example, add a `kind INTEGER` column (0 = delete, 1 = field) with `field` meaningful only for kind 1, or store `Delete` as `NULL` with `field BLOB NULL` in the PK through a `kind` column. Then require the T-9/T-6 write generator to draw `seen` keys from {`Delete`, the write's own field, another field, `Field("")`}. Name US-1 case 6b, "seen key = empty field name, plus a `Delete` entry", as a fixed case. Add `SQLITE_CONSTRAINT` to the mapping table (`Unavailable(Io)` or a named damage), so that no rusqlite error escapes the closed set.

### [M] F-2: The "known stamp above `K`" check is not pinned to `(l, c)`; a full-`Stamp` comparison would mark healthy stores as damaged after a sync
- **Axis**: Edge cases
- **Source**: specs RG-ST02-1.6 (`K ≥ max_lex{(w.l, w.c)}`); archi `rebuild` ("A known stamp above `K` → `Damage::ClockState`"); plan T-4 ("boundary: equal to `K` passes").
- **Tasks**: T-4, T-6, T-13.
- **Description**: after a received write `(l, c, a_peer)` with `(l, c) > K`, `K' = (l, c)`. If `rebuild` compares `Stamp`s, for example `w.stamp > Stamp { l: K.l, c: K.c, a: D }`, then every peer with `a_peer > D` makes every later read of that record return `Corrupted(ClockState)` and poisons the handle. That is about half of all syncs. T-4's boundary "equal to `K`" does not say which device. If it uses `D` or a smaller id, the bug passes. The contract generator draws fresh peer stamps and would rarely hit equal `(l, c)`.
- **Fix**: in T-4, name the boundary case "known peer stamp with `(l, c) = K` and `a > D` → no damage", plus the converse "`(l, c) = K` with `c + 1` → damage". Add the same named case to the contract suite (received write at exactly the new `K`, then read the record).

### [M] F-3: The Q-A guard is tested only on the pure function, not through either adapter
- **Axis**: Completeness
- **Source**: specs "Quentin's arbitration", guard on the received path ("refused as a whole… The store stays openable"); RG-ST02-6.5 (`Rejected ⇒ Σ unchanged`).
- **Tasks**: T-5, T-6, T-9, T-13.
- **Description**: T-5 tests `receive` in memory. T-9's "rejections on SQLite" list only US-3 case 10 and US-6 case 6. T-6's generator lists fresh peer writes, duplicates, peer anomalies and late writes, but not received writes with `stamp.a = D`. So no test checks, on `SimStore` or `SqliteStore`, any of the following: a re-issued local identity gives `Rejected(LocalIdentityReused)`; `Σ` is unchanged (positions, `K`); the handle is **not** poisoned; the store reopens. Also, the reference model (ST-01 `Log`) would accept such a batch as an anomaly. If the generator is extended, the reference has to implement the guard, and the plan does not say so.
- **Fix**: add to T-9 (SQLite) and T-7 (SimStore) a named case "received batch re-using a stored local identity with another content → `LocalIdentityReused`, `Σ` byte-for-byte unchanged, next op on the same handle succeeds, reopen succeeds", plus "local-device write with unknown identity → accepted, `K` raised". In T-6, either add local-device received writes to the generator, with the guard in the reference, or state why they are excluded.

### [M] F-4: The interleaving harness, as described, does not exercise the stamp race, and its planned red step fails for a different reason than the plan states
- **Axis**: Completeness / Edge cases
- **Source**: story Q-2 arbitration ("no process keeps its own clock state between writes"); specs RG-ST02-3.1, 3.2, 3.5, E-ST02-3; plan T-11 "broken step: `BEGIN DEFERRED`… the harness must catch duplicate stamps or a conflict"; archi "Every `async fn` body has no `.await`… no hook".
- **Tasks**: T-11 (and T-9's busy tests).
- **Description**:
  1. In WAL mode a `DEFERRED` transaction that read `K` cannot commit over a newer snapshot. SQLite returns `SQLITE_BUSY` or `SQLITE_BUSY_SNAPSHOT` without calling the busy handler on a read-to-write upgrade. So the mutant gives early `Unavailable(Busy)`, **never duplicate stamps**. It would be caught by "held δ/2 → success", not by the stamp checks. The plan names the wrong mechanism.
  2. The mutant that actually breaks AC-3.2 is "the handle caches `K`, or reads it outside the write transaction". It is not used as a red step. It only bites if both children stamp with the **same or rewound `pt`**, and T-11 does not say that the coordinator controls `pt`.
  3. "Driven step by step, go per operation": each operation completes inside one poll, so two adapter transactions overlap only by chance. RG-ST02-3.5's realistic defect is a read doing two `SELECT`s (writes, then `seen`) outside one snapshot. Biting it needs a commit between those two statements, which no step-by-step protocol produces without a hook. US-3 case 9 ("C reads during A's 5-write batch") is therefore not reliably reached.
- **Fix**: in T-11, (a) make the coordinator send `pt` to each writer and draw it from a small domain with equal values and rewinds; (b) replace the red step with "cache `K` in `SqliteStore` after the first transaction", and keep `DEFERRED` as a second red step for the busy checks; (c) for RG-ST02-3.5, state the mutant the reader check targets. Either (i) the reader child loops reads freely while a writer child commits large batches in a loop (free-running phase, many iterations, check every read is a whole-batch prefix), or (ii) accept that the snapshot property rests on the code shape (one `BEGIN DEFERRED` per read) and say so in the plan.

### [M] F-5: Poisoning hides which read operations detect damage; "open and read everything" can pass with `writes_since` not validating
- **Axis**: Error cases
- **Source**: specs RG-ST02-5.5, RG-ST02-7.2 (damage (b) "a stored write that fails `validate`"), AMB-ST02-3; archi "`writes_since` returns raw writes" (it does not go through `rebuild`, which is where `validate` runs); plan T-13 "open and read everything", US-7 cases 4 and 5.
- **Tasks**: T-13.
- **Description**: after the first `Corrupted`, every later operation on the handle returns the remembered damage. If T-13 reads `writes_of` first, the later `writes_since` returns `Corrupted` because of the poison, even if `writes_since` itself would have returned the stored `seen ≥ stamp` write as valid. RG-ST02-5.5 forbids that ("never… a write that was not stored" or a damaged one). The archi does not say that `writes_since` validates. Only `rebuild` does.
- **Fix**: in T-13, run each damage case with **one fresh handle per read operation** (`writes_of`, `state`, `records`, `writes_since`, `apply`, `apply_local`, `advance_watermark`), and assert the expected result per operation. Write down in T-9 that `writes_since` decodes **and** validates each row. `LocalIdentityPair` is the only documented exception (archi "Q-A detection scope").

### [M] F-6: Including core's `support/mod.rs` in `arbitre-sqlite` contradicts P-4's "no Cargo feature" decision
- **Axis**: Consistency
- **Source**: code `crates/core/tests/support/mod.rs` (`#[cfg(feature = "sim")] mod sim { … scenario(), oracle … }`); plan "Under-engineering: `arbitre-sqlite` declares no Cargo feature", T-9 "includes `../../core/tests/support/{mod,contract}.rs` by `#[path]`"; archi "along with core's `support/mod.rs` generators".
- **Tasks**: T-3, T-6, T-9.
- **Description**: `cfg(feature = "sim")` is evaluated against the **including** crate. In `arbitre-sqlite`, which declares no feature, the whole `sim` module (generators, oracle) disappears, and rustc emits `unexpected_cfgs` warnings. If `contract.rs` uses anything from `sim`, T-9 does not compile. If it does not, T-6's "reuses its generators" is false. This also forces T-6 to pick a placement before T-9 runs in another worktree.
- **Fix**: either constrain T-6 so that `contract.rs` uses only items of `support/mod.rs` outside `cfg(feature = "sim")` (and say so), or declare `[features] sim = ["arbitre-core/sim"]` in `arbitre-sqlite`. That feature is harmless under `--all-features` because it only enables the dev path. If you take the second option, delete the contrary under-engineering bullet.

### [M] F-7: The fault-VFS byte budget is described as "shared"; parallel tests in `full.rs` would interfere
- **Axis**: Over/Under-engineering (feasibility of test-only `unsafe`)
- **Source**: plan T-12 ("shared byte budget"), Under-engineering ("`cargo test` runs tests on several threads"); archi R-2.
- **Tasks**: T-12.
- **Description**: `sqlite3_vfs_register` is process-global and `full.rs` has several `#[test]` functions run in parallel threads (levels (a)–(d), cases 5, 6, `xSync`). A budget held in a static would be spent by whichever test writes first, so a `StorageFull` could land in the wrong test. That gives flaky reds, or worse, a green level that never reached its intended point. Feasibility is otherwise fine (about 150 lines; nothing outside `crates/sqlite/tests`). One more point: `pMethods` is per file and the unix VFS uses different `io_methods` tables depending on the locking style, so "override only `xWrite`/`xSync`" means copying the table **per opened file**, not once.
- **Fix**: specify a per-store budget: one VFS name per test, with the budget reached through `pAppData`, or a budget keyed by the main file path and inherited by its `-wal`. Specify per-file `io_methods` copies, kept alive as long as the file. Require each level's test to assert that the budget was actually crossed (it already asserts this for (b)).

### [M] F-8: The values returned by `apply` (`Report`) and `apply_local` (stamped writes) are not checked by the contract suite
- **Axis**: Completeness
- **Source**: archi "Port `Store`" (`apply → Report` is in the port now so that it does not need an amendment, GAP-ST02-5); story "frozen after ST-04"; plan T-6 "after each step every **read** is compared".
- **Tasks**: T-6, T-7, T-9.
- **Description**: both return values are part of a frozen port, and SQLite computes `Report` on a **partial** `Log` (touched records only). Nothing compares them with the reference. A `Report` that is wrong on SQLite only (for example, `before` computed after insert, or records missing from the load) would ship and be found in ST-19, after the freeze.
- **Fix**: in T-6, also compare after each write step: the `Report` of `apply` with `reference.merge(batch)`'s `Report`, and the writes returned by `apply_local` with the reference's `Log::write` results and with what `writes_since` returns next.

### [M] F-9: Concurrent first opening (US-4 case 3) is a single run with no red step
- **Axis**: Edge cases
- **Source**: specs RG-ST02-4.1 error case ("exactly one `D`… both handles report it"); archi "Open sequence" step 2 (`journal_mode = WAL` is set **outside** the `BEGIN IMMEDIATE` that re-checks); plan T-10 ("two children open the same empty path at once"; only case 4 is "repeated, seeded").
- **Tasks**: T-10.
- **Description**: this is a race. A single run passes or fails depending on timing. Before the first commit, a 0-byte database has no WAL marker, so the two processes can briefly disagree on journal mode, and the journal-mode switch itself needs a lock. These are exactly the interleavings where SQLite's handling of an empty file matters, and one run will not reach them. No broken step shows that the test can fail.
- **Fix**: repeat US-4 case 3 N times (seeded, same order of magnitude as case 4), with both children released by the same "go". Add a red step, "skip the re-check inside `BEGIN IMMEDIATE`", that must produce two different `D`s or a store refused as damaged. Accept `Unavailable(Busy)` for the loser only if the spec says so. Today it says "both handles report it".

### [m] F-10: Planned assertion after an `xSync` failure is stronger than SQLite guarantees after a crash
- **Axis**: Error cases
- **Source**: plan T-12 ("`xSync` failure → `Unavailable(Io)`, `Σ` unchanged"), R-7; specs RG-ST02-1.3 (`Σ ∈ {Σ_before, Σ_before ⊕ b}` at reopening).
- **Description**: when the sync fails, the commit frame, and with `synchronous = FULL` its sector padding, can already be in the WAL with valid checksums. The live view is unchanged, but recovery after a crash can replay the frame. The same applies to a budget that runs out during padding at level (c). The spec allows both outcomes. The plan states only one.
- **Fix**: assert `Σ_live = Σ_before` on the same handle, and `Σ ∈ {before, before ⊕ b}` after a reopen. Compute level (c)'s budget from frame boundaries without padding.

### [m] F-11: A failure of the Gen listing does not turn `50-st01-mutation.sh` red with the obvious implementation
- **Axis**: Consistency (CI)
- **Source**: plan T-1 ("read that list (`mapfile`)… A failure of the listing makes the check red"); `50-st01-mutation.sh` runs under `set -uo pipefail` without `-e`.
- **Description**: `mapfile -t x < <(python3 …)` drops the exit status of the process substitution. The failure is fail-open towards **more** mutation, so it costs time but does not weaken the verdict. Still, the completion criterion as written would be false.
- **Fix**: write the listing to a file (`python3 … --gen-files > "$CI_OUT/gen.txt" || exit 1`), then use `mapfile < file`. Add a shell-level demonstration to T-1's completion criterion.

### [m] F-12: `settings()` test omits `checkpoint_fullfsync`, which the archi sets
- **Axis**: Completeness
- **Source**: archi "Per connection at open" (`checkpoint_fullfsync = ON`); plan T-8 (WAL, `synchronous`, `fullfsync`).
- **Fix**: assert all four values in US-1 case 11. On Q-1 itself: no finding. WAL with `synchronous = FULL` syncs the WAL on every commit, the unix VFS syncs the directory when it creates the WAL, and `fullfsync` turns that sync into `F_FULLFSYNC` on macOS (a no-op on Linux, where `fsync`/`fdatasync` is enough). The settings test is the only check of this, as the specs accept (process kill does not exercise fsync).

### [m] F-13: Spec error cases with no planned test
- **Axis**: Completeness
- **Source**: specs RG-ST02-4.1 (random source unavailable → `Unavailable(RandomSource)`), RG-ST02-3.4 (`δ = 0` → immediate `Unavailable`), RG-ST02-5.3 (`P` beyond every position → empty list), T-13's `LocalIdentityPair` detection by `apply`/`apply_local` (archi says both applies detect it; T-13 lists only reads).
- **Fix**: add `δ = 0` to T-9's busy schedule, `P = Position(u64::MAX)` as a named contract case, and `apply`/`apply_local` on the damaged record in T-13. For `RandomSource`, state that it is untested (no injection seam in Gen code) or put the draw behind a function argument.

### [m] F-14: `sim_store.rs` zone is `Std`, while specs RG-ST02-G.2 say core files added by ST-02 are `Full`
- **Axis**: Consistency
- **Source**: specs RG-ST02-G.2 ("simulated store if placed there… `Full`"); archi zones (`Std`, like `clock.rs`); plan T-3.
- **Description**: the deviation is reasonable and the judge accepts it (new file). But it is not arbitrated, and the specs' `[HYP]` says the opposite.
- **Fix**: have Quentin confirm `Std` for `sim_store.rs` in the plan's open questions, or align the specs.

### [m] F-15: Who writes `contract.rs` and the T-7 tests is ambiguous between the Skeleton and the tasks
- **Axis**: Consistency (parallel lanes)
- **Source**: plan "Convention" (the Skeleton writes the red tests of T-4 to T-9); T-6 "Red: the suite itself"; collision table (T-6 adds a call site at the end of `tests/store.rs`).
- **Description**: if the Skeleton writes them, the T-4/T-5 ∥ T-6 collision on `tests/store.rs` disappears and T-6 only makes the suite green. If T-6 writes them, the convention is wrong. The parallel lanes are otherwise collision-free. `src/lib.rs` in T-10–T-13 is handled by the "separate commit, rebase one at a time" rule, and `support/child.rs` by the T-10 → T-11 sequencing.
- **Fix**: state which phase authors `contract.rs` and T-7's red tests.

### [m] F-16: Run-time measurement covers neither CI's platform nor the mutation step
- **Axis**: Over/Under-engineering
- **Source**: plan T-15 ("run `scripts/ci.sh` locally"), Q-P1 ("if `10-st01-tests` exceeds 10 min **on CI**"); R-2.
- **Description**: locally (macOS) every commit pays `F_FULLFSYNC`, which costs several milliseconds, while CI is Linux. The 1,000-sequence SQLite contract suite and the 250 crash runs will therefore not take the same time on the two platforms. Separately, `50-st01-mutation.sh` re-runs the core tests, including the 1,000-sequence `SimStore` contract suite, once per mutant of `store.rs` and `sim_store.rs`. Q-P1 does not cover this.
- **Fix**: T-15 takes the CI timings from the PR 2 CI run (the step summary already reports per-check durations) and applies Q-P1 to `10` measured there. It also records `50`'s duration and the mutant count.

### [m] F-17: Q-4 (a) holds only for damage visible at open
- **Axis**: Consistency (fidelity to an arbitration, for Quentin's information)
- **Source**: story Q-4 → (a) "refuses to open"; specs RG-ST02-7.2 (no full scan; per-handle poison, `[DERIVED]`); archi "No full integrity scan at open".
- **Description**: damage found by a read (cases 4, 5, `LocalIdentityPair`) poisons only that handle. The next process opens normally and can write to other records. This is consistent with the specs, but it is weaker than the literal wording of the arbitration, and the plan does not point that out.
- **Fix**: one line in the PR 2 body, or a plan note for Quentin. No code change.

## Effect of Q-P1 / Q-P2
- **Q-P1**: (b) is accepted by the specs' floor, and (a) or (c) would not change any finding except the measurement point of F-16. No change to the verdict.
- **Q-P2**: whether (a) or (b), PR 1 passes the judge's rules. Its paths are `scripts/**`, `CONTRIBUTING.md` and `test_ci.py`, with no `crates/` path: red without `ci-change`, green with it (`judge.py`, rule "with label, red only if a path under `crates/`"). Its Rust diff is empty, so `50` is neutral. PR 2 touches no protected path, and its `zones.toml` lowers no existing `.rs` file (`crates/sqlite/src` is new; `NO_GEN` still guards `crates/core`). A separate `ST-A1` cycle would change bookkeeping only. No change to the verdict.

## Axes with no finding
- **Regressions**: none. The `merge.rs` change is one `pub(crate)` constructor. `Log` and `HybridClock` have no `Default`, so cargo-mutants' only candidate (`Default::default()`) is unviable, which `mutation.py` excludes, and the file stays neutral. No public ST-01 signature changes. `hlc`, `write` and `clock` are untouched, with a diff check in T-4.
- **Security (CI)**: the Gen exclusion reuses `zone_of` (strictest wins), `gen_guard` still makes Gen under `crates/core`/`crates/plan` red after exclusion, and no new exclusion channel is open to a PR without the label.
- **Q-1 durability**: delivered on macOS and Linux by WAL with `synchronous = FULL` and `fullfsync`, apart from F-12.

## Recommendation
Return to the Planner. F-1 must be fixed before the Skeleton (layout v1 is a frozen on-disk format). F-2 to F-9 should be fixed in plan v2. The minors can be folded in or noted. A second DA iteration will be needed.
