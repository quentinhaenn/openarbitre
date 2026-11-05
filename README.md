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

## Exit codes

Every command ends with one of five exit codes. The table never changes: altering it is a foundation amendment.

| Category | JSON `category` | Exit code | Meaning |
|---|---|---|---|
| `Success` | — | 0 | The action ran. |
| `Refusal` | `refusal` | 1 | The action understood the command and declined (unknown item, invalid transition, business rule). |
| `Usage` | `usage` | 2 | The command is malformed or an option does not fit its schema. Nothing ran, the store is untouched. |
| `ToolError` | `tool_error` | 3 | The tool failed (store unavailable, crash, unwritable output). |
| `ModuleNotEnabled` | `module_not_enabled` | 4 | The action belongs to a module that is not enabled. |

With `--json`, standard output carries exactly one document, whatever the outcome. Its shape never changes:

- success: `{"data": <output>, "format": 1, "ok": true}`;
- failure: `{"error": {"category": "<JSON category>", "code": <exit code>, "id": "<error id>", "message": "<text>"}, "format": 1, "ok": false}`, plus a top-level `"data"` key only when the failure carries structured details.

Keys are sorted and no whitespace is written. Messages are not stable. These error ids never disappear: `storage_full`, `storage_corrupted`, `storage_busy`, `storage_incompatible_version`, `storage_unavailable`, `write_rejected`, `no_default_store`, `catalog_invalid`, `output_mismatch`, `internal_error`, `output_failed`. Other ids (usage ids in particular) may change.

An existing store that cannot be read makes every command exit 3 (`storage_corrupted` or another store id), help and usage errors included: without its state, `arbitre` cannot tell a malformed command from a module that is not enabled. Only `--version` still exits 0. The file is never written.

A failed command changes nothing, with one exception: if a mutating command commits and then cannot write its result (`output_failed`, exit 3), the change has been applied.

Store location: `--store <PATH>`, else a non-empty `ARBITRE_STORE`, else the platform default (`$XDG_DATA_HOME/arbitre/arbitre.db` when absolute, else `$HOME/.local/share/arbitre/arbitre.db` on Linux and other Unix; `$HOME/Library/Application Support/arbitre/arbitre.db` on macOS). With none of these usable, the command fails with `no_default_store` (exit 3).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md), including the policy on AI-assisted contributions. This project was itself developed with generative AI. To report a vulnerability, see [SECURITY.md](SECURITY.md).

## License

[Apache 2.0](LICENSE).
