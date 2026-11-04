# Devil's Advocate Review — ST-02: Persistence, async `Store` port and SQLite adapter

- Reviewed: `.ctdd/stories/ST-02/04_plan.md` (DRAFT v2), against `01_story.md`, `02_specs.md`, `03_archi.md` and their "Quentin's arbitration(s) (2026-11-04)" sections, which are binding. Previous review: `history/05_da_review_v1.md` (BLOCKED, F-1 to F-17).
- Code read: `crates/core/src/write.rs` (`TargetKind`, `Seen`, `Write`), `crates/core/Cargo.toml`, `crates/core/tests/support/mod.rs`, `crates/core/tests/convergence.rs` (include pattern); archi layout v1, open sequence, error mapping, `rebuild`/`receive`.
- Iteration 2 of 3 · Date: 2026-11-04.
- Pending questions taken at their recommendations: Q-P1 (b), Q-P2 (a), Q-P3 (a), Q-P4 (a). None changes the verdict.

## Overall verdict
**PASS WITH RESERVATIONS.**
The v1 blocker is fixed for real: the `kind` column makes the `TargetKind → (kind, field)` map injective, the PK holds every `Seen`, and the generator now draws `Field("")` keys. F-2 to F-16 are resolved with named cases and red steps that bite, with one exception: the free-running reader red step targets an equivalent mutant (F-18). F-17's resolution is cosmetic: a deviation from a binding arbitration is recorded "for information" instead of being put to Quentin (F-19). Q-P5 (F-19) should be answered **before the Skeleton**, because it changes the red tests the Skeleton writes for T-7 and T-13.

## Check of the v1 resolutions

| v1 | Status | Note |
|---|---|---|
| F-1 [B] | **Resolved** | `kind` 0/1 + `CHECK (kind = 1 OR field = x'')` + PK `(pos, kind, field)`: injective, a `Field("")` key and a `Delete` key coexist. `writes.field = x''` stays unambiguous (`identifiers_non_empty` covers the change's field). The P-8 `CONSTRAINT` row closes the set. No valid input can violate a constraint: `author`, `length(a)`, `value IS NULL ⇔ field = x''` are guaranteed by the encoding, `l < L_MAX < i64::MAX`. Side effect: F-20 |
| F-2 | Resolved | T-4 boundaries on `(l, c)` with `a > D` and `a < D`. Contract case at the new `K` |
| F-3 | Resolved | Guard on both adapters, reference implements it, no poison. The reference must merge on a **clone** to stay "unchanged" on rejection. Implied, not stated, and caught by T-6's "guard removed" check |
| F-4 | Resolved, except RG-ST02-3.5 | Cached-`K` mutant bites. Even on one record, `rebuild` raises the clock above known stamps, so it surfaces as `Damage::ClockState` or as duplicate stamps across records. Both make the run red. `DEFERRED` is caught by the busy checks. Reader part: F-18 |
| F-5 | Resolved | One fresh handle per operation; `writes_since` validates |
| F-6 | Resolved | `[features] sim` in `arbitre-sqlite` makes `#[cfg(feature = "sim")]` in the included `support/mod.rs` compile the `sim` module (`SimClock` comes through the dev-dependency's `sim`). `required-features` copies `hlc`/`convergence`. Without `--features sim`, `cargo test -p arbitre-sqlite` silently skips `contract`. CI uses `--all-features`, so this is fine; the T-9 local loop must pass the feature |
| F-7 | Resolved | Budget per VFS in `pAppData`, `io_methods` per file, crossing asserted. The 100 cases inside one test share the VFS sequentially, so the budget must be reset per case. This is obvious enough |
| F-8 | Resolved | `Report` and `apply_local` returns compared, with the next `writes_since` tail |
| F-9 | Resolved | ×50, one "go", red step, `Busy` = failure. Side effect of R-9's fix: F-21 |
| F-10 to F-13, F-15, F-16 | Resolved | `RandomSource` untested and stated: acceptable |
| F-14 | Resolved as Q-P4 | |
| F-17 | **Not resolved** | See F-19 |

## Findings

### [M] F-18: The free-running reader phase's red step targets an equivalent mutant
- **Axis**: Edge cases / Completeness (proof that the harness bites)
- **Source**: specs RG-ST02-3.5, US-3 case 9; archi layout v1 ("rows are never deleted"), "Reads: one `BEGIN DEFERRED`"; plan T-11 free-running phase, completion criterion "the two-`SELECT` mutant observed red in at least one of 10 local runs".
- **Tasks**: T-11, T-16.
- **Description**: the named mutant is "`writes_since` issues its two `SELECT`s (writes, then `seen`) outside one read transaction". `writes` and `seen` rows are append-only and inserted in the same transaction. So the second `SELECT`, run on a later snapshot, returns the same `seen` rows for every write of the first result, plus rows for newer positions. If the adapter joins by `pos`, it drops those newer rows. The result is exactly the first snapshot's. The mutant gives no observable difference, and the batch-atomicity check can never see it. The criterion "observed red in 1 of 10 runs" then fails for a structural reason, not because the race is narrow. The fallback "note it and rely on the structural rule" turns it into a non-event. The phase as specified therefore has no demonstrated mutant it catches. Also, the plan does not say how "all of a batch or none" is read from `records` output. `records` returns projections, where later batches overwrite earlier values.
- **Proof**: a split read is observable only if a single logical result is assembled from several statements over **the same growing table**: `seen` read before `writes`, so a write appears with missing `seen` entries; or `records` loading writes with one `SELECT` per record, so record 1 comes from before a batch and record 2 from after it.
- **Fix**: retarget the red step to an observable mutant. Either `records` with one `SELECT` per record outside a transaction, with the writer's batch spanning several records of one category; or `writes_since` reading `seen` **before** `writes`, with the check comparing each returned write `==` the written one. State how batch atomicity is checked on `records`. For example, each batch writes one distinct field per record, so a whole batch is visible as a set of `(record, field, tag)` in the states. Keep the T-16 structural rule.

### [M] F-19: F-17 is recorded "for information" although it narrows a binding arbitration and the spec's own formal rule
- **Axis**: Consistency (fidelity to an arbitration) / Integrity
- **Source**: story "Quentin's arbitrations", **Q-4 → (a) "A store detected as `Corrupted` refuses to open"**; specs RG-ST02-7.2 formal expression "`damaged(Σ) ∧ detected ⇒ result = Corrupted(cause) ∧ no write after detection`" and post-condition "so no write follows a detection"; archi "No full integrity scan at open"; plan T-7 ("`reopen()` clears… poison"), T-13 ("`advance_watermark` and operations on other records succeed"), T-16 PR note, Open questions "For information (DA F-17), no decision needed".
- **Tasks**: T-7, T-13, T-16; contract semantics (port frozen after ST-04).
- **Description**: damage found only by a read (US-7 cases 4 and 5, `LocalIdentityPair`, `seen ≥ stamp`, engine damage in a page that open did not read) poisons the **handle**. The CLI opens the store once per command (archi, specs RG-ST02-7.2 error case), so the poison lasts one command. The next command opens normally and writes to other records. So "no write after detection" holds per handle, not per store, and Q-4 (a)'s "refuses to open" holds only for damage visible at open. Data is not overwritten (writes are append-only) and the damaged record is never misread. In that sense the story's "never overwrites or misreads" is kept. But this is Q-4 (b)/(c) territory without the read-only guard, and it was never arbitrated. v1 F-14, a smaller deviation, was moved to a Q-P; F-17 was not. Choosing now also matters because the scope of the poison is port semantics: `SimStore::reopen()` clearing it is a contract the browser adapter (ST-20) will copy, and the port freezes after ST-04.
- **Proof**: in T-13, case 4, `writes_of(r)` on handle 1 → `Corrupted(StoredWrite)`. Drop it, open handle 2, `apply_local` on record `r'` → success. A write followed a detection on the same store.
- **Fix**: turn the note into **Q-P5**, answered before the Skeleton, with options:
  - (a) Accept the narrowing. Quentin amends Q-4's wording to "refuses to open when damage is visible at open; damage found later blocks the handle". No code change.
  - (b) Cheapest faithful option: a validation pass at open, reusing `writes_since`'s decode + `validate` path over all rows, plus one `GROUP BY (module, category, id, field, l, c) HAVING count(*) > 1` restricted to `a = D` for `LocalIdentityPair`. It is about 20 lines in `src/lib.rs` (Gen), adds no new artifact, and clears itself once a backup is restored. Cost O(n) per open: T-15 measures it at US-5 case 3 size (10,000 writes). T-13 moves to "refused at open". `SimStore`'s injected `Corrupted` then survives `reopen()`.
  - (c) A side file `<path>-damaged` holding the cause, written on first detection and re-checked at open. This is more code, and its lifecycle is harder: it must clear itself after a restore, and the write can fail on a full disk.
  - The DA view: (b) if the T-15 measurement stays small (tens of ms at 10⁴ writes), otherwise (a). (c) costs more than either.

### [m] F-20: An orphan `seen` row at `max + 1` that does not collide is silently grafted onto the next write
- **Axis**: Integrity / Edge cases
- **Source**: archi layout v1 (`pos INTEGER PRIMARY KEY` without `AUTOINCREMENT`, "`max + 1` is enough"); specs RG-ST02-7.2 ("in no case is anything written… on top of damage"), RG-ST02-1.4; plan P-8 `CONSTRAINT` note, T-13 "P-8 constraint" case.
- **Tasks**: T-8, T-13.
- **Description**: T-13 tests only the colliding half: an orphan `(kind = 0)` row plus an incoming `Delete` `seen` entry → `SQLITE_CONSTRAINT` → `Corrupted(Engine)`. If the incoming write has no entry with the same key, the insert succeeds. The orphan becomes part of the new write, which then reads back with an extra `seen` entry. If that entry's stamp is below the write's, the write passes `validate`, so it is returned as valid by every read, and by `writes_since` to peers. The plan now names this scenario. That gives confidence in a check that covers only half of it.
- **Fix**: either detect orphans (at open, `SELECT 1 FROM seen WHERE pos > (SELECT max(pos) FROM writes) LIMIT 1` → `Corrupted(StoredWrite)`, one indexed query, the same kind as the `K` check), or state in P-8 / T-13 that the non-colliding orphan is undetected damage of the "part no operation reads" class, and test nothing more.

### [m] F-21: R-9's fallback can leave a store permanently out of WAL
- **Axis**: Error cases
- **Source**: archi "`journal_mode = WAL` is persistent and set at creation only"; plan R-9 ("sets WAL **after** the creation commit and retries… (no-op once WAL is set)"), T-10 US-4 case 4 (kill during first open: checks "fresh creation or complete store" only), T-8 US-1 case 11 (WAL on every opened handle).
- **Tasks**: T-10, T-8.
- **Description**: if R-9 triggers, a kill between the creation commit and the WAL switch leaves a complete store in rollback-journal mode. Whether a later open re-applies the switch is ambiguous ("no-op once WAL is set" suggests every open, but the archi says creation only). If it does not, every later handle runs without WAL: readers block writers (RG-ST02-3.5) and `settings()` reports a non-WAL mode. The kill-during-creation check would not see it.
- **Fix**: state that the WAL switch, if moved by R-9, runs at **every** open (a no-op when already WAL). Add "`settings().journal_mode = WAL` after reopen" to US-4 case 4's check.

### [m] F-22: `Harness` signature is inconsistent
- **Axis**: Consistency
- **Source**: plan T-6 `Harness { fn fresh() -> Self::Store; fn reopen(&mut self) }`; T-9 "`reopen` = drop and open".
- **Description**: `fresh()` returns a store, not a harness, while `reopen` takes `&mut self` on the harness, which then holds no store. On SQLite, reopening needs the path, and the handle must be dropped before the store is reopened. The Skeleton writes this trait, and two crates code against it in parallel (T-6, T-9).
- **Fix**: pin the shape in the Convention section, for example `fn fresh() -> Self; fn store(&self) -> &Self::Store; fn reopen(&mut self)`.

## Axes with no finding
- **Completeness**: every RG/PROP/INV in the specs has a task and a test row; the matrices are consistent with the tasks.
- **Regressions**: unchanged from v1. `merge.rs` gains one `pub(crate)` constructor, no ST-01 signature changes, and the P-9 feature only enables the simulator in `arbitre-core`.
- **Security (CI)**: the T-1 exclusion reuses `zone_of` (strictest wins), the listing failure is now fail-closed (F-11), PR 1 has no `crates/` path, and PR 2 has no `scripts/`/`.github/` path.
- **Over-engineering**: none new. The added harness work (×50 creation, per-test VFS, free-running phase) is proportionate to blocking properties.

## Recommendation
Proceed to the Skeleton once Q-P5 (F-19) is answered, because it decides T-7's poison/`reopen` tests and T-13's expectations. Fold F-18's retargeted red step into T-11, and the minors into the plan text. No third DA iteration is needed if the changes stay within these findings.
