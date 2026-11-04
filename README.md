# OpenArbitre

A local-first toolkit, written in Rust, for the decisions you have to arbitrate yourself. It has two modules: **planning** and **decision diff**.

> **Status: early development.** Only the foundation's merge engine (`arbitre-core`) exists today. Nothing is usable yet.

## Principles

- **Independent of any AI.** Models plug in through MCP and through a plain API. A single adapter talks to Claude, GPT, Mistral or a local model.
- **Local-first.** Your data lives on your machine. A self-hosted sync server only copies it to your phone.
- **All logic in the foundation, never in the interface.** The PWA, CLI, MCP server, native app and sync server all reuse the same library.

## Architecture

| Part | Contents |
|---|---|
| Foundation | local SQLite storage, sync, CLI, MCP server, multi-model AI adapter, module mechanism |
| Planning module | sorted stack of tasks, load per day, focus blocks in the calendar |
| Decision diff module | invariant compiler (no LLM), extraction of the decisions in a PR, delta between iterations |

Modules depend on the foundation only, never on each other. Build order: foundation, then planning, then decision diff.

### Planning

- Holds your tasks, pending decisions and dated commitments.
- Shows the load per day in hours against a capacity (7.5 h minus the meetings in your calendar). An overloaded day is visible.
- Two views: the sorted stack, and blocks in your calendar. The calendar is the authority on *when*; the stack is the authority on order, durations and what is not yet scheduled.
- Calendar blocks are titled `Focus — <subject>`, with no sensitive detail.
- The tool proposes durations and remembers your corrections.
- It plans on Monday and recomputes every morning. A non-urgent item goes to the first free day without moving anything else. An urgent item or an overloaded day raises a question to you: the tool never settles it alone.

### Decision diff

Review the decisions an agent made rather than its code. Declarative invariants are checked in CI without any LLM, and an onboarding skill proposes invariants with evidence and counterexamples. Rust is supported first, Python right after.

## Stack

- Rust everywhere: foundation, CLI (`arbitre plan`, `arbitre diff`), MCP server.
- Front end in Rust compiled to WebAssembly with Leptos, shipped as a PWA for desktop and phone. Tauri may come later for an installable app with the same interface.
- Code analysis (decision diff): tree-sitter and Ruff's parser.

## Building

The toolchain is pinned in `rust-toolchain.toml`; `rustup` picks it up.

```sh
cargo test --workspace
scripts/ci.sh   # the full verification, same as CI
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md), including the policy on AI-assisted contributions. This project was itself developed with generative AI. To report a vulnerability, see [SECURITY.md](SECURITY.md).

## License

[Apache 2.0](LICENSE).
