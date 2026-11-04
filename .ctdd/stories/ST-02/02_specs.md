# Functional Specifications — ST-02: Persistence, async `Store` port and SQLite adapter

## Traceability
- Source: `.ctdd/stories/ST-02/01_story.md` (StoryWriter requirements). Its section "Quentin's arbitrations (2026-11-04)" is binding and overrides the open questions Q-1 to Q-5 above it.
- Frame: `.ctdd/cadre/02_specs.md` (US-1, US-3, RG-1.1, RG-1.6, RG-1.7, RG-3.6, PROP-3, PROP-4, PROP-7, AMB-21); `.ctdd/cadre/03_archi.md` (port `Store`, `arbitre-sqlite`, "Write paths"); `.ctdd/cadre/04_plan.md` § ST-02 (deliverable, completion criteria, zone), PL-8, F-20, F-26, R-2.
- Spike: `.ctdd/spikes/S1-A.md` (GO with reservations: no `Send` bound, `&self`, immediate native futures).
- ST-01 deliverable: `crates/core/src/{hlc,write,merge,clock}.rs`, `CONTRIBUTING.md` (glossary, deserialization contract, zones), `.ctdd/stories/ST-01/02_specs.md` (identifiers `RG-ST01-*`, `INV-ST01-*`, `MANQUE-ST01-*`), `scripts/lib/graph.py` (allowed-edge table, forbidden crates of the browser list).
- Date: 2026-11-04
- Status: DRAFT
- Codebase context (read 2026-11-04): workspace with `crates/core` (`arbitre-core`, `#![no_std]` + `alloc`, no normal dependency, `proptest` dev-only, feature `sim` for `SimClock`) and `crates/cli`. No `arbitre-sqlite` yet; `graph.py` already allows `arbitre-sqlite → arbitre-core` and forbids `rusqlite`, `libsqlite3-sys`, `sqlite-wasm-rs`, `tokio` in the browser list. CI runs `cargo test --workspace --all-features --locked` (`10-st01-tests.sh`), so tests of a new crate run without a CI change.

### Conventions of this document
- `US-n` and `AC-n.m` are the user stories and criteria **of ST-02**. Frame references are prefixed: `frame RG-1.6`. ST-01 identifiers are quoted as they are (`RG-ST01-4.2`, `INV-ST01-5`, `MANQUE-ST01-2`).
- Identifiers created here carry the `ST02` prefix: `RG-ST02-x.y`, `INV-ST02-n`, `PROP-ST02-n`, `E-ST02-n` (verification requirement), `CONFLICT-ST02-n`, `AMB-ST02-n`, `GAP-ST02-n`.
- `[ARB]`: derives from an arbitration (frame or 2026-11-04). `[HYP]`: rests on a hypothesis of the requirements. `[DERIVED]`: not written in the requirements but needed for an AC to hold; to be validated at plan review. `[TO DEFINE]`: missing value, proposal attached.
- Operation names: the story names `apply`, `writes_since`, `StorageFull`, `Corrupted`, `Unavailable`. The other operations are described by what they do ("read the writes of a record", "local apply", …); their code names are the architect's choice.

### Notation
The persisted state of a store is `Σ = (D, W, pos, K, M, V)`:
- `D`: the device identifier (`DeviceId`), or absent before first opening.
- `W`: the set of stored writes; a write is identified by its identity `(target, stamp)` **and** its content `(value?, seen, author)` (ST-01 A-3: one identity may carry several contents, an `Anomaly`).
- `pos : W → P`: the storage position of each stored write, `P` a totally ordered set with a least element `⊥P` ("from the start").
- `K = (l, c)`: the persisted state of the device's `HybridClock` (Q-2).
- `M : DeviceId ⇀ Watermark`: the watermark of each peer device (Q-5).
- `V`: the layout version.

`Log(W)` is the ST-01 `Log` of device `D` obtained by merging `W` into an empty log. `R(W) = (projection, discarded, conflicts, anomalies)` of `Log(W)`. `max_lex` is the lexicographic maximum on `(l, c)`.

### Completeness analysis of the requirements
The requirements are complete enough to specify. Points that need a decision or a correction are listed at the end:
1. **Local writes cannot be stamped by the calling shell any more** (Q-2 arbitration) while AC-6.5 says "the calling shell loads, merges, then applies": `CONFLICT-ST02-1`.
2. **Write rejections that are not storage errors** (`WriteError`: `ClockExhausted`, `Invalid`) versus "storage errors are exactly three" (AC-6.6): `CONFLICT-ST02-2`.
3. **AC-7.2, third item** ("two contents for one identity introduced by storage itself"): the store cannot tell such a pair from a legitimate `Anomaly` received from a peer: `CONFLICT-ST02-3`, open question Q-A.
4. **The core cannot hand a persisted clock state to `Log::write`**: `GAP-ST02-1`.
5. **The meaning of a watermark** differs between the ST-01 glossary (largest stamp received from a peer) and the use AC-5.3 suggests (a storage position): `AMB-ST02-1`.

## Domain glossary

Terms of the `CONTRIBUTING.md` glossary (`Log`, `Write`, `RecordId`, `Change`, `TargetKind`, `Target`, `Seen`, `Stamp`, `HybridClock`, `ClockExhausted`, `DeviceId`, `Author`, batch, `BatchError`, `validate`, `Log::write`, `Log::merge`, `Report`, projection, `RecordState`, `Status`, retained, discarded, `Conflict`, `Anomaly`) keep their meaning. New terms:

| Term | Definition |
|-------|-----------|
| Store | Durable holder of one device's known log, clock state, device identifier and watermarks. Accessed only through the `Store` port. |
| `Store` port | The storage access contract defined in `arbitre-core`, implemented by every back end. Frozen after ST-04. |
| Adapter | An implementation of the `Store` port. ST-02 delivers the SQLite adapter (`arbitre-sqlite`) and the simulated store. |
| Simulated store | In-memory adapter for tests, with injectable failures (full, unavailable, crash). |
| Store handle | One opened access to a store, held by one process. Several handles, in one or several processes, may target the same store. |
| Local change | `(record, change, author)`: what a surface wants to write, before it has a stamp and a `seen`. |
| Local apply | Store operation that turns an ordered list of local changes into stamped writes and stores them, in one write transaction (Q-2). |
| Received apply | Store operation that stores an already-stamped batch (received from a peer, an import, or a test). Named `apply` in the story. |
| Write transaction | The unit of atomicity of the store: everything it changes in `Σ` becomes visible and durable together, or not at all. |
| Confirmed | A write transaction is confirmed when it is committed **and** has reached durable storage (AMB-21, Q-1). The success result of a write operation is the confirmation. |
| Clock state | `K = (l, c)` of the device's `HybridClock`, persisted in the store; one per device, shared by every process (Q-2). |
| Physical time | `pt`, UTC milliseconds from the `Clock` port, given by the caller to a local apply. The store never reads system time itself `[DERIVED]` (keeps local apply testable with `SimClock`). |
| Storage position | `pos(w)`: local order in which `w` was first stored. Not a stamp. Opaque to callers except for comparison. |
| Watermark | Value recorded per peer device, read and moved forward by synchronization (ST-19). See `AMB-ST02-1`. |
| Layout version | Integer stored with the data that names the stored layout. `1` for the first release. |
| Busy | A write transaction cannot start because another handle holds the store for writing. |
| Wait delay | Maximum time a writer waits for a busy store. A setting, default 5 s (Q-3). |
| Forced stop | The writing process is killed by the operating system at an arbitrary instant (SIGKILL on Unix); no code of the process runs after it. |
| Damage | Stored content that cannot be read back into a valid `Σ` (see RG-ST02-7.2). |
| Contract suite | The single set of tests of the `Store` port, run against every adapter. |

## Specifications per User Story

### SPEC-US-1: Never lose a confirmed write
**Ref:** US-1; frame US-1/AC-1.4, RG-1.6, PROP-3, AMB-21; Q-1, Q-2 arbitrations

#### Formalized rules

**RG-ST02-1.1: Confirmed means committed and durable** `@Property` [ARB AMB-21, Q-1]
- **AC ref:** [US-1/AC-1.1]
- **Pre-conditions:** an opened store handle.
- **Post-conditions:** a write operation (local apply, received apply, watermark advance) returns success only after its write transaction is committed and its effects have reached durable storage (survive an operating-system crash or power cut, not only a process kill). Before that point, no success is returned and no partial effect is visible to any reader.
- **External state required:** the instant of a forced stop; the durability setting in effect.
- **Simulator needed:** forced-stop harness (E-ST02-1). A power cut is not reproducible in CI `[HYP]`; it is covered by checking the durability setting.
- **Formal expression:** `success(op) ⇒ committed(op) ∧ durable(op)`; on the SQLite adapter, every opened handle runs with the setting that synchronizes each commit to the physical disk — checked by a test that opens a handle and reads the setting in effect.
- **Error cases:**
  - Durable synchronization fails (I/O error at commit) → `Unavailable` with cause, or `StorageFull` if the cause is lack of space; the transaction is not confirmed (RG-ST02-1.3 applies).

**RG-ST02-1.2: A confirmed write survives a forced stop** `@Property` [frame RG-1.6, PROP-3]
- **AC ref:** [US-1/AC-1.2], [US-1/AC-1.4]
- **Pre-conditions:** a process applies a sequence of batches to a store and is stopped by force at an arbitrary instant: before, during or after a confirmation.
- **Post-conditions:** at reopening, every write of every batch confirmed before the stop is in `W`, with identity and content identical (RG-ST02-1.4).
- **External state required:** the instant of the stop.
- **Simulator needed:** forced-stop harness (E-ST02-1).
- **Formal expression:** `∀ batch b, confirmed(b) before stop ⇒ b ⊆ W_after_reopen`.
- **Error cases:**
  - A batch confirmed but whose confirmation the harness did not record before the stop → it may be present or absent; the property only binds recorded confirmations (it can only under-count).
  - A second process keeps writing while the first is stopped → its own confirmed writes obey the same rule; none of them is altered by the stop of the other (edge case of US-1).

**RG-ST02-1.3: A batch is all or nothing** `@Property` [frame RG-1.6 derived, RG-1.7; global rule "All or nothing"]
- **AC ref:** [US-1/AC-1.3], [US-2/AC-2.2], [US-3/AC-3.3]
- **Pre-conditions:** a write transaction interrupted or refused for any reason: forced stop, `StorageFull`, `Unavailable` (busy or I/O), `Corrupted`, write rejection (RG-ST02-6.5).
- **Post-conditions:** `Σ` after the attempt (or at reopening after a stop) is either `Σ_before` or `Σ_before` plus **all** the effects of the batch: all its writes in `W`, their positions, the updated `K`, the moved watermarks. Never a subset. `D`, `V` and every write of `W_before` are unchanged in both cases.
- **External state required:** the failure and its instant.
- **Simulator needed:** forced-stop harness (E-ST02-1); full-storage simulator (E-ST02-2); busy store (E-ST02-3).
- **Formal expression:** `Σ_after ∈ { Σ_before, Σ_before ⊕ effects(b) }`; `∀ w ∈ W_before, w ∈ W_after` with identical content and position.
- **Error cases:** this rule is the error case of every write operation.

**RG-ST02-1.4: Exact round trip of a write** `@Property` [CLASSIFICATION TO VALIDATE] [ARB `CONTRIBUTING.md` deserialization contract]
- **AC ref:** [US-1/AC-1.5]
- **Pre-conditions:** any write `w` that passes `validate` (ST-01), including: values of any bytes (empty, non-UTF-8, large), identifiers of any non-empty UTF-8 (including NUL, non-ASCII, very long), `seen` with zero, one or several entries, including extra entries allowed by A-1, `l` up to `L_MAX − 1`, `c` up to `u32::MAX`, any `DeviceId` bytes, every `Author`.
- **Post-conditions:** reading `w` back from the store, through any read operation and after a reopening, gives a write equal to `w` (`==` on `Write`, `seen` included).
- **External state required:** a store (the encoding lives in an adapter).
- **Simulator needed:** none beyond a fresh store per case; generator of valid writes.
- **Formal expression:** `∀ w, validate([w]) = Ok ⇒ read(store(w)) = w`.
- **Error cases:**
  - Stored bytes that do not decode into a write → `Corrupted` naming the damaged write (RG-ST02-7.2), never a "close" write.
- **Classification note:** `@Invariant` if the architect isolates a pure encode/decode pair; then the property holds on the pair and the store test checks it is the one used.

**RG-ST02-1.5: The stored writes rebuild the same log** `@Property`
- **AC ref:** [US-1/AC-1.6]; global rule "The log is the truth"
- **Pre-conditions:** a store fed by any sequence of local and received applies, possibly interrupted by forced stops; an in-memory reference `Log` fed with the confirmed batches only.
- **Post-conditions:** after reopening, `R(W)` equals the reference's projection, discarded writes, conflicts and anomalies. No anomaly appears that the reference does not have.
- **External state required:** history of operations and stops.
- **Simulator needed:** model-based harness (PROP-ST02-9) plus forced-stop harness.
- **Formal expression:** `R(W_after_reopen) = R(Log_ref)`, compared field by field.
- **Error cases:**
  - A difference → test failure; in production there is no reference, so this rule is a test property only.

**RG-ST02-1.6: The device clock is persisted and only moves forward** `@Property` [ARB Q-2; INV-ST01-5, RG-ST01-4.2, RG-ST01-4.3]
- **AC ref:** [US-1/AC-1.7], [US-3/AC-3.2]
- **Pre-conditions:** a store with clock state `K`.
- **Post-conditions:**
  - after a local apply that stores stamps `s₁ < … < sₙ`: `K' = (sₙ.l, sₙ.c)`;
  - after a received apply of batch `b`: `K' = max_lex(K, max_lex{(w.l, w.c) | w ∈ b})` (RG-ST01-4.3);
  - `K` never decreases, across transactions, handles, processes and reopenings;
  - consistency: `K ≥ max_lex{(w.l, w.c) | w ∈ W}` at all times `[DERIVED]`; in a store written only through the port, equality holds.
- **External state required:** physical time `pt` given by the caller, which may go back between two runs.
- **Simulator needed:** `SimClock` (ST-01) with `rewind`; reopening.
- **Formal expression:** `∀ local stamp s issued after reopening, ∀ w ∈ W_before, s > w.stamp`, whatever `pt` (including `pt` far below every stored `l`).
- **Error cases:**
  - Persisted `K` lower than a stored stamp (damage, external edit) → `Corrupted` naming the clock state, store refused (RG-ST02-7.2); never silently raised, since that would hide damage.
  - Issue impossible (`ClockExhausted`, `c` saturated or `l' ≥ L_MAX`) → local apply rejected (RG-ST02-6.5), `Σ` unchanged.

**E-ST02-1: Forced-stop verification** — verification requirement [ARB plan completion criterion "random crash"]
- **AC ref:** [US-1/AC-1.4]
- **Expression:** a test starts a real writing process on a SQLite store; the process applies batches of 1 to 5 writes (mixing local and received applies) and reports each confirmation to the test **after** the success result; the test kills it with `SIGKILL` at an instant drawn at random over its activity, so that stops fall before, during and after commits; the test reopens the store and checks RG-ST02-1.2, RG-ST02-1.3, RG-ST02-1.5 and RG-ST02-1.6. A variant runs two writing processes on the same store and kills only one (survivor's writes unaffected). A failing run is replayable from its seed.
- **`[TO DEFINE]`:** number of runs per CI execution. Proposal: **200** runs (single-writer) + **50** (two writers, one killed); kill instant uniform over `[0, run duration]`. Ceiling: CI time, to be measured; lower bound 100 + 25.

#### Case matrix — US-1

| # | Case | Type | Input | State | Expected result | Rule ref |
|---|-----|------|-------|------|-------------------|-----------|
| 1 | Confirmed then killed | Nominal | local apply of 1 change, success, then SIGKILL | store with history | write present at reopening, identical | RG-ST02-1.2 |
| 2 | Killed before commit | Edge | batch of 5, SIGKILL before commit | store with history | none of the 5 present; `Σ = Σ_before` | RG-ST02-1.3 |
| 3 | Killed during commit | Edge | batch of 5, SIGKILL during commit | — | all 5 or none; old writes intact | RG-ST02-1.3 |
| 4 | Killed after commit, before the result reached the caller | Edge | batch, SIGKILL | — | present or absent; if present, entire | RG-ST02-1.2, 1.3 |
| 5 | Action and its trace in one batch | Edge | local apply of 2 changes | — | both or neither at reopening | RG-ST02-1.3 |
| 6 | Round trip of an unusual write | Edge | value `[]`, id with NUL and emoji, 3 `seen` entries, `c = u32::MAX − 1` | — | read back `==` original | RG-ST02-1.4 |
| 7 | Rebuild after many stops | Edge | 200 runs of E-ST02-1 | — | `R(W)` = reference | RG-ST02-1.5 |
| 8 | Clock went back | Edge | reopen with `pt = 0` after writes at `l = 10⁹` | `K = (10⁹, 3)` | new stamp `(10⁹, 4, D)` | RG-ST02-1.6 |
| 9 | Received stamp far ahead | Edge | received apply with `l = 10¹²` | `K = (10⁹, 0)` | `K' = (10¹², c)`; next local stamp > it | RG-ST02-1.6 |
| 10 | Clock state below a stored stamp | Error | open | `K < max stored` (edited file) | `Corrupted`, store refused, nothing written | RG-ST02-1.6, 7.2 |
| 11 | Durability setting | Nominal | open a handle | — | setting in effect = synchronous to disk | RG-ST02-1.1 |
| 12 | One of two writers killed | Edge | two processes, kill one | — | survivor's confirmed writes all present | RG-ST02-1.2 |

### SPEC-US-2: Refuse cleanly when storage is full
**Ref:** US-2; frame RG-1.7, PROP-4

#### Formalized rules

**RG-ST02-2.1: A full storage refuses the batch and changes nothing** `@Property` [frame RG-1.7]
- **AC ref:** [US-2/AC-2.1], [US-2/AC-2.2], [US-2/AC-2.4]
- **Pre-conditions:** the storage has not enough space for the transaction, whether before it starts, in the middle of the batch, or at commit.
- **Post-conditions:** the operation returns `StorageFull`; the batch is not confirmed; `Σ_after = Σ_before` (writes, positions, `K`, `D`, watermarks, `V`).
- **External state required:** available space.
- **Simulator needed:** full-storage simulator able to make the store run out of space at a chosen point of a transaction, applicable to the **SQLite adapter** (not only to the simulated store) (E-ST02-2).
- **Formal expression:** `full_during(op) ⇒ result(op) = StorageFull ∧ Σ_after = Σ_before`.
- **Error cases:**
  - Lack of space reported by the system under another name (for example a write error during the durable synchronization) → still `StorageFull` when the cause is space; `Unavailable` otherwise.

**RG-ST02-2.2: Retrying after space is freed succeeds cleanly** `@Property`
- **AC ref:** [US-2/AC-2.3]
- **Pre-conditions:** a `StorageFull` refusal of batch `b`; space freed afterwards; same handle or a new one.
- **Post-conditions:** applying `b` again succeeds; `Σ` equals what a single successful application of `b` on `Σ_before` would give (no leftover position, no doubled write, no clock step consumed by the failed attempt that would differ from a first attempt `[DERIVED]`).
- **External state required:** available space over time.
- **Simulator needed:** full-storage simulator with release.
- **Formal expression:** `apply(b) = StorageFull; free(); apply(b) = Ok ⇒ Σ_after ≡ apply_once(Σ_before, b)`; for a local apply, equality is up to the stamps (which depend on `pt`), and the stamps are strictly greater than every stamp of `W_before`.
- **Error cases:** still full → `StorageFull` again, same post-conditions.

**RG-ST02-2.3: Reads keep working when storage is full** `@Property` [HYP]
- **AC ref:** [US-2] edge "full at opening"
- **Pre-conditions:** an existing store; storage full.
- **Post-conditions:** opening an existing store and every read operation succeed and return the stored data.
- **Simulator needed:** full-storage simulator.
- **Formal expression:** `full ∧ D present ⇒ open = Ok ∧ ∀ read r, r = r_when_not_full`.
- **Error cases:**
  - The adapter needs to write to open (for example to recover an interrupted transaction) and cannot → `StorageFull` or `Unavailable` with cause; never a read that returns a state other than the last confirmed one.

**RG-ST02-2.4: First opening on a full storage creates nothing** `@Property` [DERIVED]
- **AC ref:** [US-2/AC-2.2], [US-4/AC-4.1]
- **Pre-conditions:** an empty store location; storage full.
- **Post-conditions:** opening returns `StorageFull`; no device identifier is fixed; a later opening with space creates the store normally (RG-ST02-4.1).
- **Formal expression:** `full ∧ D absent ⇒ open = StorageFull ∧ D absent after`.
- **Error cases:** —

**E-ST02-2: Full-storage verification** — verification requirement [ARB plan completion criterion "simulated full disk"]
- **AC ref:** [US-2/AC-2.4]
- **Expression:** on the SQLite adapter and on the simulated store, a generated store state and batch are applied with the space limit set to at least these levels: (a) full before the batch, (b) full after the `k`-th write of the batch, `k` random in `[1, n−1]`, (c) full at commit, (d) full while advancing a watermark. Each case checks RG-ST02-2.1, then frees space and checks RG-ST02-2.2.
- **`[TO DEFINE]`:** **100** generated cases per level per CI execution, batches of 1 to 20 writes.

#### Case matrix — US-2

| # | Case | Type | Input | State | Expected result | Rule ref |
|---|-----|------|-------|------|-------------------|-----------|
| 1 | Full before the batch | Error | local apply | full | `StorageFull`; `Σ` unchanged; reads OK | RG-ST02-2.1, 2.3 |
| 2 | Full in the middle of the batch | Error | 10 writes, full after 4 | — | `StorageFull`; none of the 10 stored | RG-ST02-2.1 |
| 3 | Full at commit | Error | batch | — | `StorageFull`; `Σ` unchanged | RG-ST02-2.1 |
| 4 | Retry after freeing space | Nominal | same batch | space freed | success; no leftover, no duplicate | RG-ST02-2.2 |
| 5 | Full on an empty location | Edge | open | empty, full | `StorageFull`; no `D`; later open creates it | RG-ST02-2.4 |
| 6 | Full, existing store, open and read | Edge | open, read record | full | data returned | RG-ST02-2.3 |
| 7 | Full while advancing a watermark | Error | advance watermark | full | `StorageFull`; watermark unchanged | RG-ST02-2.1 |
| 8 | Received batch on full storage | Error | received apply | full | `StorageFull`; `K` unchanged | RG-ST02-2.1, 1.6 |

### SPEC-US-3: Share the same data between several programs on the computer
**Ref:** US-3; frame RG-3.6, PROP-7; Q-2, Q-3 arbitrations; plan "one SQLite file for CLI, MCP and recalculation"

#### Formalized rules

**RG-ST02-3.1: Local writes are stamped inside the write transaction** `@Property` [ARB Q-2]
- **AC ref:** [US-3/AC-3.1], [US-3/AC-3.2]; Q-2
- **Pre-conditions:** a local apply of an ordered list of local changes `x₁ … xₙ` (n ≥ 1) with physical time `pt`.
- **Post-conditions:** in one write transaction, the store: reads `K` and the stored writes of the records touched; issues `n` stamps from `K` in list order with `pt` (`HybridClock::issue`, RG-ST01-4.2); computes each `seen` from the stored writes of the record **as of this transaction**, including the writes issued earlier in the same list (ST-01 A-1, `Log::write`); stores the `n` writes; updates `K`; commits. No process keeps a clock state between transactions. The result is the list of stored writes, in list order.
- **External state required:** other handles writing concurrently.
- **Simulator needed:** two real processes with controllable interleaving (E-ST02-3).
- **Formal expression:** for two local writes `w₁`, `w₂` of device `D`, committed in that order (same or different transactions, same or different processes):
  - `w₁.stamp < w₂.stamp` (commit order = stamp order);
  - if `w₁.record = w₂.record` and `w₁.target.kind` is relevant to `w₂` under A-1 (same field, a deletion, or `w₂` a deletion), then `w₂.seen[w₁.target.kind] ≥ w₁.stamp` (sequential overwrite);
  - consequently `(w₁, w₂)` is never a `Conflict` (RG-ST01-2.1).
- **Error cases:**
  - A change with empty identifiers → write rejection `Invalid(EmptyIdentifier)` before any stamp is consumed, `Σ` unchanged (RG-ST02-6.5).
  - `ClockExhausted` on any of the `n` issues → whole list rejected, `Σ` unchanged, `K` unchanged.
  - Busy beyond the wait delay → `Unavailable` (RG-ST02-3.4).
- **Out of scope (Q-2):** lost updates at application level (a surface reads in one transaction and writes in another). Optimistic concurrency is raised in the ST-05 cycle.

**RG-ST02-3.2: One device never issues the same stamp twice** `@Property` [MANQUE-ST01-2]
- **AC ref:** [US-3/AC-3.2]
- **Pre-conditions:** any number of handles and processes on one store, any interleaving, any forced stops.
- **Post-conditions:** the local writes of `W` have pairwise distinct stamps; the rebuilt `Log` reports no `Anomaly` caused by local writes.
- **Formal expression:** `∀ w ≠ w' ∈ W, w.stamp.a = w'.stamp.a = D ∧ local(w) ∧ local(w') ⇒ w.stamp ≠ w'.stamp`.
- **Error cases:** a duplicate → test failure (it would mean two transactions read the same `K`).

**RG-ST02-3.3: No write is lost between interleaved processes** `@Property` [frame RG-3.6, PROP-7; global rule "No silent loss"]
- **AC ref:** [US-3/AC-3.1], [US-3/AC-3.5]
- **Pre-conditions:** two separate processes `A` and `B` write to the same store with interleaved local applies, including changes to the same field of the same record.
- **Post-conditions:** every write confirmed to `A` or to `B` is in `W`; for each field, exactly one write is retained and every other write of the field is in the discarded writes; no `Conflict` and no `Anomaly` between local writes (RG-ST02-3.1, 3.2); `R(W)` equals the `Log` obtained by applying the confirmed batches sequentially in commit order.
- **External state required:** interleaving of the two processes.
- **Simulator needed:** E-ST02-3.
- **Formal expression:** `confirmed_A ∪ confirmed_B ⊆ W` ∧ `∀ (d, f), |retained(d, f)| = 1 ∧ writes(d, f) \ retained(d, f) ⊆ discarded(W)` ∧ `conflicts(W) = ∅` ∧ `anomalies(W) = ∅`.
- **Error cases:**
  - One process gets `Unavailable` (busy) → its batch is not confirmed and not stored; the property counts only confirmed batches; nothing is lost silently since the caller got an explicit error.

**RG-ST02-3.4: A busy store is awaited for a bounded delay** `@Property` [ARB Q-3]
- **AC ref:** [US-3/AC-3.3]
- **Pre-conditions:** a handle `H₁` holds a write transaction; another handle `H₂` (same or other process) starts a write operation. Wait delay `δ` (setting; default 5 s).
- **Post-conditions:**
  - if `H₁` releases within `δ`, `H₂`'s operation proceeds and succeeds normally;
  - otherwise `H₂` gets `Unavailable` with a cause stating that the store is busy, after waiting at least `δ` and at most `δ + ε`; its batch is not applied (`Σ` as left by `H₁`); `H₂` may retry.
- **External state required:** duration of the other transaction.
- **Simulator needed:** a process that holds a write transaction for a controlled time; `δ` set short in tests.
- **Formal expression:** `busy for t > δ ⇒ result = Unavailable(Busy) ∧ δ ≤ wait ≤ δ + ε ∧ batch ∉ W`; `busy for t < δ ⇒ result = Ok`.
- **`[TO DEFINE]`:** tolerance `ε`. Proposal: 1 s in CI, with `δ` = 200 ms in the test.
- **Error cases:**
  - `δ = 0` → no wait, immediate `Unavailable` when busy `[DERIVED]`.
  - Reads: a read never waits longer than `δ` either; in practice it is not blocked by a writer (RG-ST02-3.5).

**RG-ST02-3.5: Readers see a whole batch or none of it** `@Property`
- **AC ref:** [US-3/AC-3.4]
- **Pre-conditions:** a handle reads while another commits a batch.
- **Post-conditions:** every read operation returns a result computed on `Σ_before` or on `Σ_after` of the other's batch, never on a mixture. A single read operation is consistent with itself (for example "writes since P" never returns half a batch).
- **Simulator needed:** E-ST02-3 with a reader process.
- **Formal expression:** `∀ read r concurrent with commit of b, r(Σ) ∈ { r(Σ_before), r(Σ_after) }`.
- **Error cases:** —

**E-ST02-3: Interleaved-processes verification** — verification requirement [ARB plan completion criterion "two interleaved writer processes"]
- **AC ref:** [US-3/AC-3.5]
- **Expression:** the test launches two real writer processes (and, for RG-ST02-3.5, a reader process) on one SQLite store. A coordinator controls the interleaving step by step (each process waits for a go before each operation, and may be told to hold a transaction open). Schedules are generated at random, with a high share of changes to the same field of the same record, and include the cases "held longer than `δ`" and "released within `δ`". Each schedule checks RG-ST02-3.1 to 3.5. A failing schedule is replayable from its seed.
- **`[TO DEFINE]`:** **100** schedules per CI execution, 2 to 20 operations per process.

#### Case matrix — US-3

| # | Case | Type | Input | State | Expected result | Rule ref |
|---|-----|------|-------|------|-------------------|-----------|
| 1 | Recalc and CLI, different records | Nominal | A writes (plan, item, 1), B writes (plan, item, 2) | — | both stored; next read shows both | RG-ST02-3.3 |
| 2 | Same field, interleaved | Edge | A then B set (d, title) | — | B's stamp > A's; B.seen covers A; B retained; A discarded; no conflict | RG-ST02-3.1, 3.3 |
| 3 | Same field, B's transaction starts first, A's commits first | Edge | controlled schedule | — | commit order decides; later commit retained | RG-ST02-3.1 |
| 4 | Deletion against a field write | Edge | A sets (d, f), B deletes d | — | B.seen covers A; d deleted; A kept hidden; no conflict | RG-ST02-3.1 |
| 5 | Many writes from two processes | Edge | 20 ops each | — | all stamps distinct; no anomaly | RG-ST02-3.2 |
| 6 | Held beyond the delay | Error | A holds 2δ; B writes | — | B: `Unavailable(Busy)` after ≈ δ; B's batch absent | RG-ST02-3.4 |
| 7 | Held within the delay | Edge | A holds δ/2; B writes | — | B succeeds after A | RG-ST02-3.4 |
| 8 | Retry after `Unavailable` | Nominal | B retries | A released | success | RG-ST02-3.4 |
| 9 | Reader during a commit | Edge | C reads during A's 5-write batch | — | C sees 0 or 5 of them | RG-ST02-3.5 |
| 10 | Invalid change in a local list | Error | 3 changes, 2nd with empty field name | — | rejected; no stamp consumed; `Σ` unchanged | RG-ST02-3.1, 6.5 |

### SPEC-US-4: Identify the device durably
**Ref:** US-4; RG-2.1 tie-break (ST-01)

#### Formalized rules

**RG-ST02-4.1: The device identifier is created once, at first opening** `@Property` [HYP]
- **AC ref:** [US-4/AC-4.1], [US-4/AC-4.2]
- **Pre-conditions:** a store location with no store yet (missing or empty file).
- **Post-conditions:** the first successful opening creates the store with `D` = 16 random bytes, `K = (0, 0)`, `W = ∅`, `M = ∅`, `V` = current layout version, all in one confirmed transaction. Every later opening, by any process, returns the same `D`. `D` never changes. Every local write is stamped with `a = D`.
- **External state required:** concurrent openings.
- **Simulator needed:** two processes opening the same empty location at the same time.
- **Formal expression:** `∀ openings o₁, o₂ of one store, D(o₁) = D(o₂)`; `∀ local w ∈ W, w.stamp.a = D`.
- **Error cases:**
  - Two processes open an empty location at the same time → exactly one `D` is created; both handles report it.
  - Opening fails during creation (stop, full, I/O) → no store, or the whole initial state; never a store without `D` (RG-ST02-1.3).
  - Random source unavailable → `Unavailable` with cause; no store created.
  - An existing store without `D` → `Corrupted` (RG-ST02-7.2).

**RG-ST02-4.2: Independent stores get different identifiers** `@Property`
- **AC ref:** [US-4/AC-4.3]
- **Pre-conditions:** stores created independently.
- **Post-conditions:** their identifiers differ (128-bit random draw from the operating system's secure random source `[DERIVED]`).
- **Formal expression:** test: 1,000 stores created in a row have 1,000 distinct identifiers, none all-zero.
- **Error cases:** —

#### Case matrix — US-4

| # | Case | Type | Input | State | Expected result | Rule ref |
|---|-----|------|-------|------|-------------------|-----------|
| 1 | First opening | Nominal | open | no file | store created; `D` random; `K = (0,0)` | RG-ST02-4.1 |
| 2 | Reopening by another process | Nominal | open | existing | same `D` | RG-ST02-4.1 |
| 3 | Two simultaneous first openings | Edge | 2 processes open | no file | one `D`, seen by both | RG-ST02-4.1 |
| 4 | Stop during creation | Edge | SIGKILL during first open | no file | next open: fresh creation or complete store, never without `D` | RG-ST02-4.1, 1.3 |
| 5 | Two independent stores | Nominal | create 1,000 | — | all `D` distinct | RG-ST02-4.2 |
| 6 | Local write's stamp | Nominal | local apply | `D` | `stamp.a = D` | RG-ST02-4.1 |
| 7 | Copied store file | Edge | copy to another machine | — | shared `D`: out of scope, not detected | — |

### SPEC-US-5: Read back what is stored, for later stories
**Ref:** US-5; frame archi port `Store` (`etat`, `parcourir`, `journal`, `ecritures_depuis`, `filigrane`, `avancer_filigrane`, `appareil`); Q-5

#### Formalized rules

**RG-ST02-5.1: The writes of a record are all returned** `@Property`
- **AC ref:** [US-5/AC-5.1]
- **Pre-conditions:** a record `d`.
- **Post-conditions:** "read the writes of a record" returns every write of `W` whose record is `d`: every field, every deletion, every content of an identity with several contents, retained and discarded alike; in the order of `Log::writes` (`(target, stamp, content)`) `[DERIVED]` so that results are deterministic.
- **Formal expression:** `writes_of(d) = sort({ w ∈ W | w.record = d })`.
- **Error cases:**
  - Unknown record → empty list, not an error.
  - 10,000 writes on one field → all 10,000 returned.

**RG-ST02-5.2: Record state and listing equal the projection** `@Property`
- **AC ref:** [US-5/AC-5.2]
- **Pre-conditions:** a record `d`; a pair `(module, category)`.
- **Post-conditions:** "read the state of a record" returns `Log(W).projection()[d]` (absent if the record has no write). "List the records of `(module, category)`" returns every record of that pair with its `RecordState`, deleted records included with `Status::Deleted`, sorted by `RecordId` (A-4) `[DERIVED]`.
- **Formal expression:** `state(d) = projection(W)[d]`; `list(m, k) = { (d, projection(W)[d]) | d.module = m ∧ d.category = k }`.
- **Error cases:**
  - Unknown pair → empty list.
  - Record with only a deletion → listed, `Status::Deleted`, no fields.

**RG-ST02-5.3: "Writes since P" follows storage order** `@Property` [HYP]
- **AC ref:** [US-5/AC-5.3]
- **Pre-conditions:** a position `P`, or `⊥P` ("from the start").
- **Post-conditions:**
  - each write receives its position when it is first stored; positions are strictly increasing in commit order, unique, never reused, stable across reopenings;
  - storing again a write already in `W` (same identity and content) is a no-op: no new position, no change to `Σ` other than `K` per RG-ST02-1.6 (idempotence of received apply);
  - `writes_since(P)` returns exactly the writes with `pos(w) > P`, in increasing position order, each with its position; `writes_since(⊥P)` returns all of `W`;
  - a write received late with an old stamp gets a new, larger position and is returned by `writes_since` of any earlier `P`.
- **Formal expression:** `writes_since(P) = [ (pos(w), w) | w ∈ W, pos(w) > P ] sorted by pos`.
- **Error cases:**
  - `P` greater than every position (for example from another store) → empty list, not an error `[DERIVED]`.

**RG-ST02-5.4: Watermarks per peer device** `@Property` [ARB Q-5]
- **AC ref:** [US-5/AC-5.4]
- **Pre-conditions:** a peer device `p`.
- **Post-conditions:** reading the watermark of a never-seen `p` returns "nothing received". Moving the watermark of `p` to `x` is a write transaction (confirmed, all or nothing, durable); after it, the watermark is `max(old, x)`: it never goes back `[DERIVED]` (moving to a lower or equal value succeeds and changes nothing). Watermarks of different peers are independent.
- **Formal expression:** `watermark(p) = M(p)` or "nothing received"; `advance(p, x) ⇒ M'(p) = max(M(p), x)`.
- **Error cases:** `StorageFull`, `Unavailable` → watermark unchanged (RG-ST02-1.3).
- **Note:** what a watermark value denotes is `AMB-ST02-1`.

**RG-ST02-5.5: A read never returns a partial or invented result** `@Property` [global rule "All or nothing"]
- **AC ref:** [US-5/AC-5.5]
- **Pre-conditions:** a store that cannot be read, wholly or partly.
- **Post-conditions:** the read returns `Unavailable` (cannot access) or `Corrupted` (content damaged), with cause; never a truncated list, a default value, or a write that was not stored.
- **Simulator needed:** simulated store with "unavailable" injection; damaged SQLite files (E-ST02-4).
- **Formal expression:** `read r ⇒ result ∈ { exact(r, Σ), Unavailable(cause), Corrupted(cause) }`.
- **Error cases:** this rule is the error case of every read.

#### Case matrix — US-5

| # | Case | Type | Input | State | Expected result | Rule ref |
|---|-----|------|-------|------|-------------------|-----------|
| 1 | List items of the plan | Nominal | list (plan, item) | 3 items, 1 deleted | 3 records, one `Deleted`, sorted | RG-ST02-5.2 |
| 2 | Record with only a deletion | Edge | state(d) | one deletion | `Deleted`, no fields | RG-ST02-5.2 |
| 3 | 10,000 writes on a field | Edge | writes_of(d) | — | 10,000 returned | RG-ST02-5.1 |
| 4 | Identity with two contents | Edge | writes_of(d) | anomaly received from a peer | both contents returned | RG-ST02-5.1 |
| 5 | Writes since start | Nominal | `writes_since(⊥P)` | — | all of `W`, position order | RG-ST02-5.3 |
| 6 | Late write with an old stamp | Edge | received apply of an old-stamped write, then `writes_since(P_before)` | — | the late write is returned | RG-ST02-5.3 |
| 7 | Same batch received twice | Edge | received apply twice | — | second: no new position, `W` unchanged | RG-ST02-5.3 |
| 8 | Positions after reopening | Edge | reopen, `writes_since(P)` | — | same positions as before | RG-ST02-5.3 |
| 9 | Watermark of an unknown peer | Edge | watermark(p) | — | "nothing received" | RG-ST02-5.4 |
| 10 | Watermark moved back | Edge | advance(p, lower) | `M(p) = x` | unchanged, success | RG-ST02-5.4 |
| 11 | Unreadable store | Error | any read | file unreadable | `Unavailable` with cause | RG-ST02-5.5 |
| 12 | Unknown record | Edge | state(d) | — | absent, no error | RG-ST02-5.2 |

### SPEC-US-6: One storage access shape for every back end
**Ref:** US-6; PL-8, F-26, R-2; S1-A reservations 1 and 3; archi "every port has a real adapter and a simulator"

#### Formalized rules

**RG-ST02-6.1: The port lives in a core that stays pure and browser-buildable** `@Invariant` [global rule "Core stays pure"]
- **AC ref:** [US-6/AC-6.1]
- **Post-conditions:** the `Store` port and its error type are in `arbitre-core`; `arbitre-core` has still no normal dependency; the browser-target build check (`20-st01-browser.sh`) stays green.
- **Formal expression:** `normal_deps(arbitre-core) = ∅ ∧ build(arbitre-core, wasm32-unknown-unknown) = Ok`.
- **Error cases:** a dependency added to the core → existing CI check red.

**RG-ST02-6.2: Asynchronous operations, no `Send` requirement, shared access** `@Invariant` [ARB PL-8; S1-A]
- **AC ref:** [US-6/AC-6.2], [US-6/AC-6.3]
- **Post-conditions:** every port operation is asynchronous and takes shared access to the store. Generic code over the port compiles with an adapter whose futures are not `Send` and whose handle is not `Sync`; no generic code of the core requires `Send` or `Sync` of the port.
- **Formal expression:** compile-time test: a test adapter built on non-`Send` interior mutability implements the port and is accepted by every generic function of the core that takes a store; two operations can be in flight at once on one handle (two futures created before either is polled).
- **Error cases:** a `Send` bound in generic code → that test does not compile.

**RG-ST02-6.3: The SQLite adapter completes on first poll** `@Property` [S1-A]
- **AC ref:** [US-6/AC-6.4]
- **Post-conditions:** every future returned by the SQLite adapter is ready on its first poll with a no-op waker; no async runtime is needed. This holds for successes and for every error, including `Unavailable` after waiting `δ` on a busy store (the wait happens inside the poll) `[DERIVED]`.
- **Formal expression:** `∀ op, poll_once(op(adapter), noop_waker) = Ready(_)`.
- **Error cases:** a pending first poll → test failure.

**RG-ST02-6.4: The merge engine stays synchronous and port-free** `@Invariant` [ST-01 contract]
- **AC ref:** [US-6/AC-6.5]
- **Post-conditions:** `Log`, `HybridClock`, `validate` and the merge stay synchronous and pure; nothing in them refers to the `Store` port or to an asynchronous operation. The store calls the core; the core never calls the store. See `CONFLICT-ST02-1` for the local write path.
- **Formal expression:** the core's merge, write and clock modules contain no reference to the port; their signatures are unchanged except for `GAP-ST02-1`.
- **Error cases:** —

**RG-ST02-6.5: Closed set of storage errors; invalid input refused before storage** `@Invariant` [ST-01 RG-ST01-4.5, INV-ST01-7]
- **AC ref:** [US-6/AC-6.6]
- **Post-conditions:**
  - storage errors are exactly `StorageFull`, `Corrupted` (with a cause naming what is damaged) and `Unavailable` (with a cause: busy, I/O, incompatible version, random source, …);
  - a received apply runs `validate` first; on `BatchError` it returns that error and touches nothing;
  - a local apply validates each change first; on `WriteError` (`Invalid`, `ClockExhausted`) it returns that error and touches nothing.
  - These two rejections are not storage errors (`CONFLICT-ST02-2`).
- **Formal expression:** `result(write op) ∈ { Ok, Rejected(BatchError | WriteError), StorageFull, Corrupted(cause), Unavailable(cause) }` and `Rejected ⇒ Σ_after = Σ_before`.
- **Error cases:** —

**RG-ST02-6.6: SQLite stays inside `arbitre-sqlite`** `@Invariant`
- **AC ref:** [US-6/AC-6.7]
- **Post-conditions:** the database library is a dependency of `arbitre-sqlite` only, local to it (not shared) `[HYP]`; `arbitre-sqlite` depends on `arbitre-core` only among workspace crates; neither `arbitre-core` nor `arbitre-plan` reaches it.
- **Formal expression:** existing graph check (`30-st01-graph.sh`, allowed-edge table and forbidden crates) stays green with `arbitre-sqlite` added.
- **Error cases:** —

**RG-ST02-6.7: A simulated store obeys the same contract** `@Property`
- **AC ref:** [US-6/AC-6.8]
- **Post-conditions:** a simulated store, in memory, implements the port and passes the whole contract suite; it can be told to fail: (a) full from a chosen point of a transaction (`StorageFull`), (b) unavailable for the next operations (`Unavailable`), (c) crash: forget every effect not confirmed, as a forced stop would. It is usable by the tests of later stories without the SQLite adapter, and it compiles for the browser target if it lives in the core `[DERIVED]`.
- **Formal expression:** `contract_suite(simulated) = contract_suite(sqlite) = pass`; with injections, RG-ST02-1.3, 2.1, 5.5 hold.
- **Error cases:** —

**PROP-ST02-9 (contract suite, model-based)** `@Property`
- **AC ref:** [US-5/AC-5.1 to 5.3], [US-6/AC-6.8], [US-1/AC-1.6]
- **Expression:** random sequences of operations (local apply, received apply including duplicates and anomalies, watermark advance, every read, reopen) run on the adapter and on a reference made of an ST-01 `Log` plus a position list and a watermark map. After each step, every read on the adapter equals the reference.
- **`[TO DEFINE]`:** **1,000** sequences of up to 50 operations per adapter per CI execution (ST-01 E-1.1 count).

#### Case matrix — US-6

| # | Case | Type | Input | State | Expected result | Rule ref |
|---|-----|------|-------|------|-------------------|-----------|
| 1 | Core built for the browser | Nominal | wasm32 build | port added | green | RG-ST02-6.1 |
| 2 | Non-`Send` adapter | Nominal | compile generic code with it | — | compiles | RG-ST02-6.2 |
| 3 | Two requests in flight | Edge | two futures, then poll | — | both complete | RG-ST02-6.2 |
| 4 | First poll | Nominal | each op, noop waker | — | `Ready` | RG-ST02-6.3 |
| 5 | First poll on a busy store | Edge | write, other handle holds | — | `Ready(Unavailable)` after δ | RG-ST02-6.3, 3.4 |
| 6 | Invalid received batch | Error | `seen` not before stamp | — | `BatchError`; `Σ` unchanged | RG-ST02-6.5 |
| 7 | `arbitre-plan` depending on `arbitre-sqlite` | Error | added edge | — | graph check red | RG-ST02-6.6 |
| 8 | Simulated store, crash injection | Edge | batch, crash before commit | — | batch absent after "reopen" | RG-ST02-6.7 |
| 9 | Contract suite on both adapters | Nominal | PROP-ST02-9 | — | green on both | RG-ST02-6.7 |

### SPEC-US-7: Detect damage and incompatible versions instead of guessing
**Ref:** US-7; archi "Migrations"; Q-4 arbitration

#### Formalized rules

**RG-ST02-7.1: The layout carries a version** `@Invariant`
- **AC ref:** [US-7/AC-7.1]
- **Post-conditions:** every store holds `V`; a store created by this release has `V = 1`.
- **Formal expression:** `create() ⇒ V = 1`.
- **Error cases:** missing `V` in an existing store → `Corrupted`.

**RG-ST02-7.2: A damaged store is refused** `@Property` [ARB Q-4]
- **AC ref:** [US-7/AC-7.2]
- **Pre-conditions:** a store whose content is damaged.
- **Post-conditions:** the damage is reported as `Corrupted` with a cause naming what is damaged (the write, the clock state, the device identifier, the layout). If detected at opening, the store does not open. If detected later by an operation, that operation returns `Corrupted` and every later operation on that handle returns `Corrupted` too `[DERIVED]`, so no write follows a detection. In no case is anything written, repaired or skipped.
- **Damage is at least:** (a) bytes that cannot be decoded into a write, a clock state or a device identifier; (b) a stored write that fails `validate`; (c) `D` or `V` missing in an existing store, or `V` not a known version; (d) `K` lower than a stored stamp (RG-ST02-1.6); (e) a file at the store's path that is not a store, or that the storage engine reports as damaged (truncated, malformed). For "two contents for one identity", see `CONFLICT-ST02-3` / Q-A.
- **External state required:** damaged files.
- **Simulator needed:** E-ST02-4.
- **Formal expression:** `damaged(Σ) ∧ detected ⇒ result = Corrupted(cause) ∧ no write after detection`.
- **Error cases:**
  - Damage in a part that no operation reads → not necessarily detected at opening; a full integrity scan at each opening is not required by the inputs `[DERIVED]` (no performance target, but every CLI command opens the store).

**RG-ST02-7.3: A newer layout is refused** `@Property`
- **AC ref:** [US-7/AC-7.3]
- **Pre-conditions:** a store whose `V` is greater than the versions the tool knows.
- **Post-conditions:** opening fails with `Unavailable` whose cause names the stored version and the supported one `[DERIVED]` (it is not damage); nothing is written: the stored content is unchanged, as read by the newer version.
- **Formal expression:** `V_stored > V_supported ⇒ open = Unavailable(IncompatibleVersion{stored, supported}) ∧ Σ unchanged`.
- **Error cases:** —

**RG-ST02-7.4: The layout is generic** `@Invariant`
- **AC ref:** [US-7/AC-7.4]
- **Post-conditions:** the stored layout knows writes, their positions, the clock state, the device identifier, watermarks and the version; nothing about a module's business model. A new module, category or field name needs no layout change and no version change.
- **Formal expression:** the round trip (RG-ST02-1.4) and the contract suite (PROP-ST02-9) pass with module, category and field names generated at random, on one store, without any layout change.
- **Error cases:** —

**E-ST02-4: Damage verification** — verification requirement
- **AC ref:** [US-7/AC-7.2], [US-5/AC-5.5]
- **Expression:** tests build stores, then damage them from outside the port in each way (a) to (e) of RG-ST02-7.2, plus truncation at a random length; opening (or the first read reaching the damage) returns `Corrupted`; the file is byte-identical after the attempt.

#### Case matrix — US-7

| # | Case | Type | Input | State | Expected result | Rule ref |
|---|-----|------|-------|------|-------------------|-----------|
| 1 | New store | Nominal | create | — | `V = 1` | RG-ST02-7.1 |
| 2 | Downgrade after a layout change | Error | open | `V = 2` | `Unavailable(IncompatibleVersion{2, 1})`; nothing written | RG-ST02-7.3 |
| 3 | Truncated by an external tool | Error | open | truncated file | `Corrupted`; file unchanged | RG-ST02-7.2 |
| 4 | Undecodable write bytes | Error | read the record | edited row | `Corrupted` naming the write; handle refuses later ops | RG-ST02-7.2 |
| 5 | Stored write failing `validate` | Error | open or read | `seen ≥ stamp` stored | `Corrupted` | RG-ST02-7.2 |
| 6 | Not a store | Error | open | text file at the path | `Corrupted` ("not a store"); file unchanged | RG-ST02-7.2 |
| 7 | New module's records | Nominal | writes with module "diff" | store created before | stored and read back, no layout change | RG-ST02-7.4 |

## Global rules (cross-cutting)

**RG-ST02-G.1: Fully offline** `@Invariant` [frame RG-1.1]
- **Post-conditions:** no store operation uses the network.
- **Formal expression:** `arbitre-sqlite`'s dependency graph contains no network crate `[DERIVED]`; the contract suite needs no network.

**RG-ST02-G.2: Zones** [HYP]
- `crates/core/` files added by ST-02 (port, errors, simulated store if placed there) are `Full` (default; `Gen` forbidden under `crates/core/`).
- `crates/sqlite/**` is `Gen`: informational mutation score (MANQUE-ST01-1 arbitration), while E-ST02-1, E-ST02-2, E-ST02-3, RG-ST02-1.4 and PROP-ST02-9 are blocking tests run by `10-st01-tests.sh`.

## Identified dependencies

| Dependency | Type | Description | Impact |
|-----------|------|-------------|--------|
| ST-01 core (`Log`, `HybridClock`, `validate`, `Write`) | Internal | Stamping, `seen`, validation, projection used by the store and by the reference model | Blocking |
| Rebuild of a `Log`/clock from a persisted clock state | Internal | Not offered by the current core API (`GAP-ST02-1`) | Blocking for RG-ST02-3.1 |
| Embedded transactional database (SQLite) | External | Atomic durable transactions, multi-process locking with a wait delay, snapshot reads | Blocking |
| Operating system secure random source | External | Device identifier (RG-ST02-4.2) | Blocking (first opening only) |
| `Clock` port (ST-01) | Internal | Physical time given to local apply | Blocking |
| Property-test library (`proptest`, shared) | External | RG-ST02-1.4, PROP-ST02-9, generated schedules | Blocking |
| Process spawning and kill in tests | External (OS) | E-ST02-1, E-ST02-3 (Unix `SIGKILL`) | Blocking for those tests; CI host is Unix |
| Means to make storage full under the SQLite adapter | External / test | E-ST02-2 | Blocking for AC-2.4 |
| Existing CI checks (`10`, `20`, `30` scripts) | Internal | Run the new tests and guard the edges without a `ci-change` PR | Non-blocking (no CI change expected) |

## Conflicts and ambiguities

| # | Rules in conflict | Description | Proposed resolution |
|---|-------------------|-------------|---------------------|
| [CONFLICT-ST02-1] | AC-6.5 vs Q-2 arbitration | AC-6.5: "the calling shell loads, merges, then applies". Q-2: the stamp and `seen` of a local write are fixed inside the single write transaction, so the shell cannot stamp beforehand. | Two write paths: **local apply** takes unstamped local changes and `pt`, and stamps them inside the transaction by calling the pure core (RG-ST02-3.1); **received apply** takes stamped writes and keeps the ST-01 flow (validate, store, clock receive). The core stays synchronous and port-free (RG-ST02-6.4). The browser adapter must offer the same atomicity (ST-20). Shape of the operations: architect. |
| [CONFLICT-ST02-2] | AC-6.6 "exactly three errors" vs `WriteError` | A local apply can be rejected for `Invalid(EmptyIdentifier)` or `ClockExhausted`, which are neither storage errors nor `BatchError`. | Treated like `BatchError`: an input rejection returned before any storage effect, distinct from the three storage errors (RG-ST02-6.5). The three-error set stays closed. |
| [CONFLICT-ST02-3] | AC-7.2 (iii) vs ST-01 A-3 / MANQUE-ST01-2 | "Two contents for one identity introduced by storage itself" must be `Corrupted`, but the store keeps legitimate anomalies received from peers and cannot tell where a pair came from. | Open question Q-A. |
| [CONFLICT-ST02-4] | Frame AC-1.4 "power cut" vs plan "process kill" | Already settled by Q-1 (a) and the story's hypothesis. | Process kill tested (E-ST02-1); power cut covered by the durability setting check (RG-ST02-1.1). |
| [AMB-ST02-1] | AC-5.4 vs ST-01 glossary | ST-01 defines a watermark as "the largest stamp received from a peer"; AC-5.3 makes synchronization read by storage position. ST-19 may track the peer's position, not a stamp. | ST-02 stores an opaque, totally ordered value per peer and only guarantees RG-ST02-5.4; its meaning is fixed by ST-19. The type must be able to hold either a stamp or a position; the architect chooses it. |
| [AMB-ST02-2] | AC-5.3 hypothesis | Is a position the same notion across a reopening and across processes? | Yes: one sequence per store, shared by all handles, persisted (RG-ST02-5.3). |
| [AMB-ST02-3] | Q-4 vs detection after opening | Q-4 says a `Corrupted` store refuses to open; damage can also be found by a later read. | The handle refuses every later operation after a detection (RG-ST02-7.2) `[DERIVED]`. |

### Gaps (absent from the requirements, needed)

| # | Source | Description | Proposed resolution |
|---|--------|-------------|---------------------|
| [GAP-ST02-1] | Q-2, `crates/core/src/{hlc,merge}.rs` | `HybridClock` **can** be rebuilt from a persisted `K = (l, c)` with the current API (`new(D)` then `receive(&Stamp { l, c, a: D })` gives exactly `(l, c)`), and its state after `issue` is `(stamp.l, stamp.c)`. But `Log` owns a private clock created at `(0, 0)`, and `Log::write` is the only path of a local write (`CONTRIBUTING.md`). A `Log` built from the writes of the touched records only would stamp from their largest stamp, not from `K`: a stamp could repeat (RG-ST02-3.2) or go below a stored one. | The core must let a local write be issued from a given clock state and give back the new state, without changing the semantics of `Log::write` (validation before issuing, `seen` per A-1). Exact API: architect (core change, `Full` zone, covered by the ST-01 tests plus a test "issue from a restored state = issue from the original"). |
| [GAP-ST02-2] | AC-1.4, AC-2.4, AC-3.5 | Run counts not in the inputs. | Set here: E-ST02-1 200 + 50, E-ST02-2 100 per level, E-ST02-3 100 schedules, PROP-ST02-9 1,000 sequences. To be lowered only if CI time is measured too high; floor: half these values. |
| [GAP-ST02-3] | AC-3.3 | Tolerance on the wait delay. | `ε` = 1 s in tests (RG-ST02-3.4). |
| [GAP-ST02-4] | AC-7.3 | Which of the three errors a newer layout gives. | `Unavailable` with an "incompatible version" cause (RG-ST02-7.3): it is not damage. |
| [GAP-ST02-5] | Received apply | Whether received apply returns the merge `Report` (new discarded writes, conflicts, anomalies). No AC requires it in ST-02; a surface or ST-19 can compute it with the pure engine. | Not required in ST-02. If ST-19 needs it, it is an amendment of a frozen port: the architect may add it now at low cost. |

## Candidate invariants (C-TDD synthesis)

### @Invariant (stateless, blocking)
- `INV-ST02-1`: `arbitre-core` has no normal dependency and builds for `wasm32-unknown-unknown` with the port added — source: RG-ST02-6.1
- `INV-ST02-2`: port operations are asynchronous, take `&self`, and no generic code requires `Send`/`Sync` (compile test with a non-`Send` adapter) — source: RG-ST02-6.2
- `INV-ST02-3`: the merge engine, `Log`, `HybridClock` and `validate` are synchronous and never refer to the port — source: RG-ST02-6.4
- `INV-ST02-4`: write results ∈ {Ok, Rejected(BatchError | WriteError), StorageFull, Corrupted(cause), Unavailable(cause)}; Rejected ⇒ `Σ` unchanged — source: RG-ST02-6.5
- `INV-ST02-5`: the database library is reachable only from `arbitre-sqlite`; core and plan never reach `arbitre-sqlite` — source: RG-ST02-6.6
- `INV-ST02-6`: a created store has `V = 1`; the layout is generic (no layout change for new modules, categories, fields) — source: RG-ST02-7.1, RG-ST02-7.4
- `INV-ST02-7`: no store operation uses the network — source: RG-ST02-G.1

### @Property (stateful, simulator needed)
- `PROP-ST02-1`: success ⇒ committed and durable; durability setting = synchronous to disk on every handle — simulator: forced stop; setting inspection — source: RG-ST02-1.1
- `PROP-ST02-2`: confirmed ⇒ present after forced stop; unconfirmed batch all or nothing; prior state intact (frame PROP-3) — simulator: real process killed at random instants, E-ST02-1 — source: RG-ST02-1.2, RG-ST02-1.3
- `PROP-ST02-3`: exact round trip of any valid write, `seen` included — simulator: generator of valid writes — source: RG-ST02-1.4 [CLASSIFICATION TO VALIDATE]
- `PROP-ST02-4`: rebuilt `Log` after reopening has the same projection, discarded writes, conflicts, anomalies as the reference — simulator: model + forced stops — source: RG-ST02-1.5
- `PROP-ST02-5`: `K` persisted, never decreases, `K ≥` every stored stamp; a new local stamp > every stored stamp whatever `pt` — simulator: `SimClock` rewind + reopen — source: RG-ST02-1.6
- `PROP-ST02-6`: full storage at any point ⇒ `StorageFull`, `Σ` unchanged; retry after freeing succeeds with no leftover; reads still work (frame PROP-4) — simulator: full-storage at levels (a)–(d), E-ST02-2 — source: RG-ST02-2.1 to 2.4
- `PROP-ST02-7`: two interleaved writer processes: commit order = stamp order, later write's `seen` covers the earlier, no conflict, no anomaly, no duplicate stamp, every confirmed write stored (frame PROP-7) — simulator: two real processes with a coordinator, E-ST02-3 — source: RG-ST02-3.1 to 3.3
- `PROP-ST02-8`: busy beyond `δ` ⇒ `Unavailable(Busy)` after `δ ≤ wait ≤ δ + ε`, batch absent; within `δ` ⇒ success; readers see whole batches only — simulator: process holding a transaction; reader process — source: RG-ST02-3.4, RG-ST02-3.5
- `PROP-ST02-9`: contract suite: any sequence of operations (including reopen) on an adapter gives the same reads as the reference `Log` + positions + watermarks; run on SQLite and simulated stores — simulator: model-based generator — source: RG-ST02-5.1 to 5.4, RG-ST02-6.7
- `PROP-ST02-10`: device identifier created once, identical for every opening and process, distinct between stores; first opening on full storage creates nothing — simulator: concurrent openers; full storage — source: RG-ST02-4.1, RG-ST02-4.2, RG-ST02-2.4
- `PROP-ST02-11`: SQLite adapter futures are `Ready` on first poll, errors included — simulator: none (noop waker) — source: RG-ST02-6.3
- `PROP-ST02-12`: damaged store ⇒ `Corrupted(cause)`, refused, nothing written; newer layout ⇒ `Unavailable(IncompatibleVersion)`, nothing written; unreadable ⇒ never a partial read — simulator: damaged files, E-ST02-4 — source: RG-ST02-5.5, RG-ST02-7.2, RG-ST02-7.3

## Open questions for Quentin

**Q-A — "Two contents for one identity" in a stored log (AC-7.2, third item; `CONFLICT-ST02-3`).** The store keeps every received write, so it legitimately holds `Anomaly` pairs from peers (ST-01: reported, not refused). It cannot tell those from a pair created by storage damage.
- (a) Never `Corrupted` for this alone: the pair is stored and read back, and the engine reports it as an `Anomaly`. Damage that produces it is caught by the other checks (undecodable bytes, `validate`, storage-engine integrity).
- (b) `Corrupted` when the pair carries the local device's own identifier (`stamp.a = D`), since RG-ST02-3.2 makes that impossible through the port; pairs from peers stay anomalies.
- **Recommendation: (b).** It catches the only case the store can actually attribute to itself, and it would also catch a store file copied and written on two machines. The cost is that one bad local pair blocks the whole store (Q-4 (a)).

## Quentin's arbitration (2026-11-04)

Binding; overrides the open question above.

- **Q-A → (b), with a guard.** A stored pair "two contents for one identity" whose `stamp.a` is the local device's identifier makes the store `Corrupted`; pairs from peers stay an `Anomaly`.
- **Guard on the received path (in ST-02 scope):** a received batch containing a write whose `stamp.a` is the local device's identifier and whose identity already exists locally with a different content is refused as a whole, with an explicit error, before anything is stored. The store stays openable. Typical cause: a device restored from a backup or cloned, re-issuing stamps it had already issued.
- **Deferred to ST-04b:** a restore or import gives the device a fresh `DeviceId`.
