# Decision ST-01: shared dependencies

- Date: 2026-11-03 · Story: ST-01 · Sources: `.ctdd/stories/ST-01/03_archi.md` (Shared dependencies, arbitration P-1; French), amendment A-5 of `.ctdd/stories/ST-01/04_plan.md`, PL-6 of the framework.
- Status: adopted.

## Rule

A shared dependency is declared once, in `[workspace.dependencies]` of the root `Cargo.toml`, even if no component uses it yet. Each component inherits it with `x = { workspace = true }` and adds its own features (they are additive). The workspace fixes the version and turns default features off, unless stated otherwise.

**Adding, removing or changing a shared dependency is a foundation amendment**, outside any story. The same goes for a tool version bump (`scripts/tools.env`, `rust-toolchain.toml`).

## Choices by topic

| Topic | Choice | Version | Workspace declaration | Reason | Rejected option |
|---|---|---|---|---|---|
| Async runtime | `tokio` | 1.54.1 | `default-features = false` | De facto standard; required by rmcp (AI agent) | async-std (discontinued), smol (rmcp requires tokio) |
| Outgoing network calls | `reqwest` + rustls | 0.13.5 | `default-features = false, features = ["rustls"]` | Full HTTP client, TLS without OpenSSL (PL-6). In the browser, `fetch` behind the `Transport` port | ureq (blocking), bare hyper (too low-level), native-tls / OpenSSL (excluded by PL-6, checked by `graph.py`) |
| System keychain | `keyring`, `0600` file fallback | 4.2.0 | `default-features = false` | One access path to the macOS, Windows and Linux keychains | direct secret-service (Linux only) |
| Serialization | `serde` (`derive`) + `serde_json` | 1.0.229 / 1.0.154 | `serde`: `default-features = false, features = ["derive"]`; `serde_json`: default features (`std`) | De facto standard | — |
| Property testing | `proptest` | 1.12.0 | default features (`std`: failure persistence, `RngSeed`); in `[dev-dependencies]` only | Fine-grained shrinking of counter-examples, replayable failures | quickcheck (coarser shrinking, no persistence) |
| Mutation testing | `cargo-mutants` | 27.1.0 | **outside `Cargo.toml`**: tool, pinned in `scripts/tools.env` | Only maintained tool, `--in-diff` mode | mutagen (abandoned) |
| Dates and time zones (addition) | `jiff` | 0.2.39 | `default-features = false` | See "Additions" | chrono + chrono-tz |
| Randomness (addition) | `getrandom` | 0.4.3 | `default-features = false` | See "Additions" | rand |

Removed in ST-01: `chrono` (no longer used) and `rusqlite`, which returns in ST-02 in `arbitre-sqlite` alone, as a local dependency. Still local: `clap` (CLI only) and the wasm-bindgen crates (`arbitre-web` only).

## Additions to PL-6 (arbitration P-1)

- **`jiff`** will serve `Clock::timezone` (core, ST-04), planning and the calendar. A single crate for dates and time zones, working on wasm32 without `now()` (spike S1-A). chrono + chrono-tz would need two crates, and a time-zone database twice as heavy raw.
- **`getrandom`** will serve the device identifier (ST-02) and the envelope nonces (ST-18). `rand` is unnecessary for drawing raw bytes. The wasm backend is enabled by the web crate.
- Each will serve at least two components: declaring them now avoids two foundation amendments.

## Points of attention

- **`keyring` 4.2.0 does not compile without a feature**: `default-features = false` alone triggers `compile_error!` ("At least one of the features 'v1' or 'cli' must be enabled"). The crate using it adds `features = ["v1"]` (found by the one-off check of T-1).
- **TLS**: `reqwest` 0.13 with `rustls` pulls in `aws-lc-rs`, which compiles C. No effect on wasm (`reqwest` is forbidden in the browser list), acceptable natively.
- **`aws-lc-rs` → `ring` reversibility**: no data or format depends on the TLS provider. Switching is one workspace feature, plus a `rustls::crypto::CryptoProvider::install_default()` at native binary startup. **Pitfall**: if two rustls providers are compiled in, an explicit `install_default()` becomes mandatory, otherwise rustls panics on the first call.
- **Saturated counter received from a peer (Q-4)**: a corrupted or hostile peer sending `(future l, c = u32::MAX)` stops the device from writing (`ClockExhausted`) until its time passes that `l`. Accepted: beyond `σ`, the stamp is reported by `leads`.
- `Cargo.lock` is authoritative only for dependencies inherited by a component (A-5).
