## ST-04: Modules you can enable, kept apart, with one generic conformance test

Modules are enabled per device, nothing is enabled on a fresh device, and a disabled module stays out of sight. A module plugs in through one trait and sees only its own namespace. One generic conformance test, which every module and every surface replays, runs with the network cut and with it open. This PR also freezes the contract that later stories build on.

Cycle documents: `.ctdd/stories/ST-04/` (`02_specs.md` v2 with § Resolution, `04_plan.md` v2, `05_da_review.md`, `07_review.md` with § Resolution).

### Frozen by this PR (RG-ST04-G.2)

Changing any of these is now a foundation amendment. The list is in `CONTRIBUTING.md` § Frozen contract.

- **Types**: `ActionDescriptor`, `Schema`, `Property`, `Value`, `Output`, `Category`, `ActionError`, `Module`, `Context`, `ModuleStore`, `Problem`, `Ai`, `AiError`, `Store`, `StoreError`, `Clock`.
- **`jiff` 0.2**: `Clock::timezone` returns `jiff::tz::TimeZone`, so a major `jiff` upgrade is an amendment.
- **Record layouts**: `arbitre/module-state/<device hex>` (one field per module, `1`/`0`) and `arbitre/signal/<device hex>-<ms>-<source>` (`at`, `source`, `message`, `read`).
- **Tooling**: the observer `Observer::start(Mode::{Open, Cut})`, `env()`, and `take()` returning the `Attempt { host, port }` since the last call; the surfaces contract `Env`, `Called`, `Surface`; the conformance helpers and checks; the fake module. Extension points: a file plus one line in `tests/support/surfaces/`, and `ALLOWED` rows for adapters.
- Error ids may only be added to. Helpers such as `pre_scan`, `enabled_from`, `state_record`, `unread` and `line` are public but **not** frozen.

### Arbitrations

- **Q1 to Q7: (a)**, Quentin, 2026-11-05.
  - Q1: device-local state lives in the store, under the foundation namespace, keyed by device.
  - Q2: nothing is enabled on a fresh device.
  - Q3: `trait Module<S: Store>` is dyn-compatible and returns boxed local futures.
  - Q4: `Problem { record, id, message }`. `module enable` revalidates the module and lists the problems.
  - Q5: a recording proxy in the tests, with no product code.
  - Q6: no-network actions end with exit 0 or 1 and make zero attempts, plus a full CRUD scenario.
  - Q7: `arbitre module enable --module <id>`.
- **Q-DA1: (a)**, settled by default in the plan. ST-03's test actions become the foundation actions `arbitre.test.*`, with the same command paths. No build enables anything by default. This only moves test-only code.
- **Q-R1: (a)**, Quentin, 2026-11-05. With an unreadable store, every command exits 3, help and usage errors included. Only `--version` exits 0. The README § Exit codes says so.

### Checks

- **`jiff` no_std/wasm**: a scratch crate with `#![no_std]` and `jiff = "=0.2.39"`, `default-features = false`, features `alloc` and `tzdb-bundle-always`, builds for `wasm32-unknown-unknown`. `cargo tree -e features` shows no `std`. `Europe/Paris` resolves from the bundled database. `arbitre-core`'s normal dependencies are `jiff`, `jiff-core` and `jiff-tzdb`. `20-st01-browser.sh` passes.
- **Mutation**: 100 % on every ST-04 file. The run used `-f` on the 11 Std files, plus `core/src/write.rs` (Full) and `cli/src/cli/mod.rs`: 317 mutants, 250 caught, 67 unviable, **0 missed**. `core/src/merge.rs`, `RecordState::text`: 3/3 caught. No new equivalent mutant.
- **Conformance time**: `tests/conformance.rs` takes 1.75 s for 9 tests. The ceiling is 3 min.
- **Tests**: `cargo test --workspace --all-features --locked` gives 318 passed, 0 failed, in 30 suites (doctests included). `fmt` and both clippy runs pass. `30-st01-graph.sh` passes.

### Also in this PR

- **ST-03 bug R-3**: derived option names kept `_` (`--due_date`). RG-ST03-4.3 and both frozen encoders say `--due-date`. The derivation now replaces `_` with `-`, and the unit test `option_names_use_dashes` checks it. This brings the code in line with ST-03's frozen rule, so it is not an amendment.
- **Docs**: README § Modules, with the D-2 CI flow checked against the test build's `fake-a`. CONTRIBUTING gets glossary rows, § Frozen contract, § Adding a module, § Adding a surface, § Pinned time and the full § Network clients.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
