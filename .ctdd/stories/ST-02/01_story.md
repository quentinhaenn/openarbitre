# Requirements — ST-02: Persistence, async `Store` port and SQLite adapter

Sources: `.ctdd/cadre/04_plan.md` § ST-02, PL-8, R-2, "Under-engineering detected"; `.ctdd/cadre/02_specs.md` RG-1.1, RG-1.6, RG-1.7, RG-3.6, PROP-3, PROP-4, PROP-7, AMB-21; `.ctdd/cadre/03_archi.md` (port `Store`, `arbitre-sqlite`, data flows); `.ctdd/spikes/S1-A.md` (verdict GO with reservations); ST-01 deliverable (`crates/core/src/`, `CONTRIBUTING.md`, `.ctdd/stories/ST-01/03_archi.md` § "Contract given to ST-02").

Names follow the `CONTRIBUTING.md` glossary. New terms introduced by this story:

| Spec term (French) | Code name |
|---|---|
| `Store` (port de stockage) | `Store` |
| `StockagePlein` | `StorageFull` |
| `Corrompu` | `Corrupted` |
| `Indisponible` | `Unavailable` |
| `appliquer` | `apply` |
| `ecritures_depuis` | `writes_since` |
| filigrane (point de dernière réception) | watermark |
| confirmée | confirmed (= the storage transaction is committed, AMB-21) |

## Context
ST-01 delivered the merge engine, but everything it knows lives in memory and disappears when the program stops. ST-02 makes the writes durable: a confirmed write survives a crash or power cut, a full disk refuses the write without damaging anything, and several programs on the same computer (command line, assistant connector, background recalculation) share the same data without losing a write. It also fixes, once, the shape of the storage access that every later story and every future storage back end (browser, phone) must follow. The story has no user-visible value on its own (F-20): ST-03 is the first one that puts it in a user's hands.

## Target users
- **Solo user** (indirect beneficiary): never sees this story directly. Gets the guarantee that what the tool told them is saved is really saved, whatever happens to the machine.
- **Maintainer** (Quentin): arbitrates the durability and concurrency choices; the shape of the storage access is frozen after ST-04 (plan § Collisions), so changing it later is a foundation amendment.
- **Development agents** (non-human actors): build ST-03 onward on top of this storage access, and the browser storage back end later (ST-20, after spike S1-B), without changing the core.

## User Stories

### US-1: Never lose a confirmed write
**As a** solo user, **I want** every write the tool has confirmed to be present when I reopen it, whatever the way it stopped, **so that** "saved" always means saved (US-1/AC-1.4 of the frame, RG-1.6, PROP-3).

**Acceptance Criteria:**
- [ ] AC-1.1: A write is **confirmed** exactly when the storage has committed it; nothing is reported as done before that point (AMB-21).
- [ ] AC-1.2: After a forced stop of the process at an arbitrary moment (before, during or after confirmation), every write confirmed before the stop is present at reopening, with all its parts identical.
- [ ] AC-1.3: A batch of writes that was not confirmed when the stop happened is found at reopening either entirely or not at all; no data that existed before is altered.
- [ ] AC-1.4: AC-1.2 and AC-1.3 are checked automatically by killing a real writing process at randomly chosen moments, over many runs (the exact number is set by the specs), not only on hand-picked moments.
- [ ] AC-1.5: Reading a stored write gives back exactly the write that was stored, **`seen` included** (deserialization contract of `CONTRIBUTING.md`). Checked by a property over randomly generated writes.
- [ ] AC-1.6: After reopening, the stored writes rebuild a `Log` whose projection, discarded writes, conflicts and anomalies are identical to those of the `Log` before the stop (no loss, no spurious anomaly).
- [ ] AC-1.7: After reopening, a new local write gets a stamp strictly greater than every stamp already stored, even if the computer's clock has gone back in the meantime (INV-ST01-5 survives a restart).

**Nominal cases:**
- The user records a change from the command line; the tool says it is done; the laptop's battery dies a second later. On restart, the change is there.

**Identified edge cases:**
- Stop in the middle of a multi-write batch (for example an action and its trace in the assistant log): all of it or none of it at reopening.
- Power cut rather than process kill: see Open questions (Q-1).
- A process that stops while another one keeps writing to the same data: the survivor's writes are unaffected.

### US-2: Refuse cleanly when storage is full
**As a** solo user, **I want** a write that cannot be stored for lack of space to be refused with an explicit message and to leave everything as it was, **so that** a full disk never damages what I already had (US-1 edge "disk full", RG-1.7, PROP-4).

**Acceptance Criteria:**
- [ ] AC-2.1: When storage is full, applying a batch returns `StorageFull`; the batch is not confirmed.
- [ ] AC-2.2: After a `StorageFull` refusal, the stored state (writes, device identifier, watermarks) is identical to the state before the attempt.
- [ ] AC-2.3: Once space is freed, the same batch can be applied again and succeeds, with no leftover from the failed attempt.
- [ ] AC-2.4: The refusal is checked automatically with a simulated full storage, at several fill levels (including "full in the middle of the batch").

**Nominal cases:**
- The disk is full; the user records a change; the tool refuses it, says storage is full, and everything recorded earlier is still there and readable.

**Identified edge cases:**
- Storage becomes full in the middle of a batch: whole batch refused, nothing partial.
- Storage is full at opening, before any write: reading keeps working `[HYPOTHESIS]`.

### US-3: Share the same data between several programs on the computer
**As a** solo user, **I want** the command line, the assistant connector and the background recalculation to write to the same data at the same time without any write being lost in silence, **so that** I never have to think about which program is running (US-3 edge "command while the interface edits the same record", RG-3.6, PROP-7, plan "one SQLite file for CLI, MCP and recalculation").

**Acceptance Criteria:**
- [ ] AC-3.1: Two **separate processes** writing to the same storage with interleaved writes, including on the same field of the same record, lose no write: every write confirmed to either process is stored, and for each field one write is retained and the others are among the discarded writes (RG-3.6).
- [ ] AC-3.2: Two writes issued by the same device never carry the same stamp, even when they come from two processes (otherwise they would form an anomaly, MANQUE-ST01-2).
- [ ] AC-3.3: A writer that finds the storage busy waits for it up to a bounded delay (value: Q-3), then gets `Unavailable` with an explicit cause; its batch is not applied.
- [ ] AC-3.4: A process reading while another writes sees either the state before or the state after the other's batch, never a part of it.
- [ ] AC-3.5: AC-3.1 is checked automatically with two real processes and a controllable interleaving.

**Nominal cases:**
- The morning recalculation runs in the background while the user records a change from the command line. Both writes are stored; the next read shows both.

**Identified edge cases:**
- Both processes change the same field at the same moment: whether this is reported to the user as a conflict or treated as a sequential overwrite is Q-2; in both cases the other value stays recoverable.
- A process holds the storage for longer than the wait delay: the other gets `Unavailable`, nothing is applied, and it may retry.
- The browser app on the same computer is a separate replica (archi D-1): its concurrency with the command line goes through synchronization, out of scope here (ST-21).

### US-4: Identify the device durably
**As a** solo user, **I want** each installation of the tool to get its own identifier once and keep it, **so that** the tie-break between devices in the merge (RG-2.1, ST-01) is always well defined.

**Acceptance Criteria:**
- [ ] AC-4.1: At first opening of an empty storage, a device identifier (`DeviceId`, 16 bytes) is drawn at random and stored; it never changes afterwards.
- [ ] AC-4.2: Every later opening, by any process, returns the same identifier.
- [ ] AC-4.3: Two storages created independently get different identifiers (random draw, 128 bits).

**Nominal cases:**
- The user installs the tool, runs a first command, then the MCP connector starts: both use the same device identifier.

**Identified edge cases:**
- Two processes open an empty storage at the same time: only one identifier is created and both see it.
- The storage file is copied to another computer: both computers then share one identifier, which breaks the tie-break. See Hypotheses.

### US-5: Read back what is stored, for later stories
**As a** development agent, **I want** to read from storage everything the next stories need (the writes of one record, the records of a module and category, the writes added since a given point, the watermark of each peer device), **so that** ST-03 onward and the synchronization (ST-19) do not need to change the storage access, which is frozen after ST-04.

**Acceptance Criteria:**
- [ ] AC-5.1: Reading the writes of a record returns **all** stored writes of that record, every field and every deletion, not only the retained ones (ST-01 contract: exclusivity covers the whole record).
- [ ] AC-5.2: The state of a record, and the list of records of a given `(module, category)`, are readable, and are identical to the `Log` projection of the stored writes.
- [ ] AC-5.3: "Writes added since point P" returns every write stored after P, in storage order, including a write received late with an old stamp; reading from the start returns everything. This point is a local order of storage, not a stamp `[HYPOTHESIS]` (S1-A used a local sequence number).
- [ ] AC-5.4: A watermark per peer device can be read and moved forward; reading the watermark of a never-seen peer returns "nothing received" (ST-01 hypothesis, empty watermark). Scope: Q-5.
- [ ] AC-5.5: Every read on a storage that cannot be read returns `Unavailable` or `Corrupted`, never a partial or invented result.

**Nominal cases:**
- ST-03 lists the user's items: it reads the records of `(plan, item)` and gets their projected state.

**Identified edge cases:**
- Record with only a deletion: it is listed with status deleted (ST-01: winners kept, hidden).
- 10,000 writes on one field (ST-01, case 8): reading the record's writes returns all of them.

### US-6: One storage access shape for every back end
**As a** maintainer, **I want** the storage access to be defined once in the core, in a shape that both the native database and a browser storage worker can implement, **so that** a failure of the browser storage on iOS (R-2) only changes a back end, never the core (PL-8, F-26).

**Acceptance Criteria:**
- [ ] AC-6.1: The `Store` port lives in the core (`arbitre-core`), which still builds for the browser target with no new dependency (ST-01 check stays green).
- [ ] AC-6.2: Every operation of the port is asynchronous (PL-8). The port does **not** require the operations to be movable between threads (S1-A reservation 1: browser futures are not `Send`); the native adapter may offer that, generic code must not demand it.
- [ ] AC-6.3: Every operation takes shared access to the store (`&self`), so that several requests can be in flight at once (S1-A).
- [ ] AC-6.4: The native SQLite adapter completes each operation on its first poll, without needing an async runtime (S1-A, "immediate futures"); checked by a test.
- [ ] AC-6.5: The merge engine stays synchronous and never sees the port: the calling shell loads, merges, then applies (ST-01 contract).
- [ ] AC-6.6: Storage errors are exactly three: `StorageFull` (French `StockagePlein`), `Corrupted` (`Corrompu`) and `Unavailable` (`Indisponible`), each with an explicit cause where relevant. A batch rejected by `validate` (ST-01 `BatchError`) is refused before reaching storage.
- [ ] AC-6.7: The SQLite-specific code (database library included) is only in the new component `arbitre-sqlite`, which is native only; the core and the planning module never depend on it (forbidden-edge check of ST-01).
- [ ] AC-6.8: A simulated `Store` (in memory, with controllable failures: full, unavailable, crash) is provided for the tests of later stories (archi: "every port has a real adapter and a simulator").

**Nominal cases:**
- Later, ST-20 writes a browser adapter that forwards each operation to a storage worker; it implements the same port, and no core file changes.

**Identified edge cases:**
- Stamps cross to JavaScript as numbers in the future browser adapter: precision is guaranteed by `l < 2^53` (ST-01, A-11); the encoding itself (S1-A reservation 2) belongs to the browser adapter, out of scope.

### US-7: Detect damage and incompatible versions instead of guessing
**As a** solo user, **I want** the tool to refuse to use a damaged storage, or one written by a newer version, with an explicit message, **so that** it never overwrites or misreads my data.

**Acceptance Criteria:**
- [ ] AC-7.1: The stored layout carries a version number from the first release (archi "Migrations").
- [ ] AC-7.2: A storage whose content cannot be read back into valid writes (unreadable bytes, write failing `validate`, two contents for one identity introduced by storage itself) gives `Corrupted`, naming what is damaged; behaviour after detection: Q-4.
- [ ] AC-7.3: A storage whose version is newer than the tool's is refused with an explicit message, and nothing is written to it.
- [ ] AC-7.4: The stored layout is generic (writes and their projection), independent of any module's business model: a new field or a new kind of record needs no change to the stored layout (archi).

**Nominal cases:**
- The user downgrades the tool after an upgrade that changed the layout: the old version refuses to open and says why.

**Identified edge cases:**
- Storage truncated by an external tool: `Corrupted`, nothing written.

## Global business rules
- **Confirmed means committed** (AMB-21): no surface may say "done" before the store has confirmed the batch.
- **All or nothing** (RG-1.6, RG-1.7): a batch is applied entirely or not at all, whatever the failure (crash, full storage, busy storage, invalid batch).
- **No silent loss** (RG-G7, RG-3.6): no write confirmed to any process disappears; a superseded value stays among the discarded writes.
- **Exact round trip**: a write read back equals the write stored, `seen` included; otherwise the engine would report false anomalies.
- **The log is the truth**: projection, discarded writes and conflicts are derived from the stored writes and must equal what the ST-01 `Log` computes from them.
- **Fully offline** (RG-1.1): no storage operation needs the network.
- **Core stays pure**: no storage library, no system time, no I/O in `arbitre-core` (ST-01, `no_std`).

## Out of scope
- The browser storage back end (OPFS, IndexedDB worker), the browser test harness and the `BigInt` encoding of stamps (S1-B, ST-20).
- Synchronization, envelopes, relay and the use of watermarks to send and receive (ST-18, ST-19); concurrency CLI ↔ browser app (ST-21, PROP-7 GUI side).
- Export and import (ST-04b), even though they will read the same writes.
- The action catalogue, the command line, user-facing messages and exit codes (ST-03): this story returns typed errors; their wording to the user comes later.
- The namespace-restricted view of the store per module (INV-22, ST-04).
- Where the storage file lives on disk by default, and how a user picks it: decided by ST-03 (CLI); the adapter is given a path `[HYPOTHESIS]`.
- Compaction or deletion of old writes (accepted ceiling of the archi; the log only grows).
- Encryption of the local storage at rest.
- Detecting a storage copied to another machine (shared device identifier).
- Showing conflicts, discarded writes or anomalies to the user (surface stories).
- Performance targets: none are set by the inputs; loading only the records touched by a batch is allowed by ST-01 (merge is local to a record) but not required.

## Hypotheses
- [HYPOTHESIS] A forced process kill is the automated crash test (AC-1.4); a real power cut is not reproduced in CI, it is covered by the durability setting chosen in Q-1.
- [HYPOTHESIS] Reads stay possible when storage is full (US-2 edge case).
- [HYPOTHESIS] The device identifier is created at the first opening of an empty storage, not by a separate "init" command; a copied storage file sharing its identifier with the original is out of scope.
- [HYPOTHESIS] "Writes since P" uses a local storage order, not a stamp, so that a write received late with an old stamp is not skipped by synchronization; the specs may confirm or replace it.
- [HYPOTHESIS] Zones: the port file in `arbitre-core` follows the core default (Full; `CONTRIBUTING.md` forbids `Gen` under `crates/core/`); `arbitre-sqlite` is `Gen` (plan: "Gen (adapter), with blocking properties"), so its mutation score is informational while the properties of AC-1.4, AC-1.5, AC-2.4 and AC-3.5 block.
- [HYPOTHESIS] `rusqlite` is a dependency local to `arbitre-sqlite` and not a shared one (ST-01 dependency decision), so adding it is not a foundation amendment.
- [HYPOTHESIS] The hybrid clock state does not need its own storage: at opening it is rebuilt by receiving the largest stored stamp, which is enough for AC-1.7 and AC-3.2 provided stamps are issued inside the writing transaction (see Q-2).

## Open questions for Quentin

**Q-1 — Durability against a power cut.** The frame's AC-1.4 names "power cut", the plan only requires a process kill. The storage can make each confirmation reach the physical disk (every confirmed write survives a power cut, each write is slower, of the order of milliseconds) or only the operating system (survives a process crash, but the last confirmed writes can be lost on a power cut or kernel crash).
- (a) Always reach the disk before confirming.
- (b) Reach the operating system only; accept the loss window on power cut.
- **Recommendation: (a).** It is what AC-1.4 literally promises, and the write rate of a personal planning tool makes the cost invisible.

**Q-2 — Two local programs change the same field at the same moment.** Locally, the second writer can be made to see the first one's write before stamping its own (it is then a sequential overwrite: no conflict reported, old value in the discarded writes), or each program stamps from what it had in memory (it is then a conflict, reported like a sync conflict).
- (a) Sequential: each write is stamped inside the storage transaction, after reading the latest state; never a conflict between two local programs.
- (b) Conflict: two local programs that did not see each other's write produce a reported conflict.
- **Recommendation: (a).** It also guarantees AC-3.2 (no duplicate stamp from one device), and RG-3.6 only requires "no silent loss", which (a) satisfies through the discarded writes. The matrix case "one retained, the other recoverable" holds in both.

**Q-3 — How long a writer waits for a busy storage before giving up.**
- (a) 5 seconds, then `Unavailable`.
- (b) 30 seconds.
- (c) No limit (wait forever).
- **Recommendation: (a), as a setting with that default.** A recalculation batch is short; 5 s covers it, and a stuck process must not freeze the command line forever.

**Q-4 — What to do once damage is detected (`Corrupted`).**
- (a) Refuse to open the storage at all, until the user acts (restore a backup, export what is readable later).
- (b) Open read-only, skipping the damaged writes, and refuse every new write.
- (c) Skip the damaged writes silently and carry on.
- **Recommendation: (a).** Matches the all-or-nothing rule of ST-01 batches; (c) is a silent loss (RG-G7) and is excluded; (b) can come later with export (ST-04b).

**Q-5 — Watermarks per peer in ST-02 or in ST-19.** ST-01 removed watermarks from conflict detection; they now only serve synchronization (ST-19). But the `Store` port is frozen after ST-04, so adding them later is a foundation amendment.
- (a) Add "read / move forward the watermark of a peer" to the port now, with its storage; no use before ST-19.
- (b) Leave them to ST-19 as a foundation amendment.
- **Recommendation: (a).** Two operations and one stored table now cost less than an amendment of a frozen contract later, and the plan lists them in ST-02's deliverable.

## Quentin's arbitrations (2026-11-04)

These answer the open questions above and take precedence over them.

- **Q-1 → (a)** A write reaches the disk before it is confirmed.
- **Q-2 → (a), reworded.** One hybrid clock per device, persisted in the store. A write loads the clock state, issues its stamp, writes and commits inside a single write transaction (`BEGIN IMMEDIATE` on SQLite); no process keeps its own clock state between writes. Two local programs writing the same field are therefore a sequential overwrite: the later write's `seen` covers the earlier one, and no conflict is reported. Application-level lost updates (read in one transaction, write in another) are out of scope here; optimistic concurrency is to be raised in the ST-05 cycle.
- **Q-3 → (a)** A writer waits up to 5 s for a busy store, then gets `Unavailable`. The delay is a setting.
- **Q-4 → (a)** A store detected as `Corrupted` refuses to open.
- **Q-5 → (a)** Per-peer watermarks are part of the port now.
