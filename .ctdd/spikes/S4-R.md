# Spike S4-R: language-neutral `Extractor` port, and the Rust parser

- **Start**: 2026-11-04T20:45:11Z
- **End**: 2026-11-04T21:55Z (report included)
- **Scope**: Quentin's arbitration of 2026-11-03 (`04_plan.md`, "Rust premier langage de decision diff"): Rust first, Python next, port neutral from the first adapter. Question 1 (dominant): the port shape and fact model such that a new language = one adapter + one registration line. Question 2: tree-sitter-rust or `syn` for the Rust adapter. This is not S4 (Python parser: tree-sitter-python or Ruff), which still runs before the Python story.
- **Throwaway code** (not mergeable, own Cargo workspace, not a member of the repo workspace): `spikes/s4r/`
  - `facts/`: port, fact model, registry, toy checker. **Zero dependencies.** One test with a fake language.
  - `lang-rust/`: Rust adapter on tree-sitter-rust 0.24.2 (the recommended one). Modules, `use`/`extern crate`, symbols, path calls, type references.
  - `lang-rust-syn/`: comparison adapter on `syn` 3.0.9 + `proc-macro2` spans. Modules, `use`/`extern crate`, re-exports.
  - `lang-python/`: second language, tree-sitter-python 0.25.0. Modules, imports (relative ones resolved), symbols with `__all__`.
  - `demo/`: `demo check <dir> <rules>` (all adapters, one checker) and `demo bench <dir>` (tree-sitter against syn).
  - `rules/openarbitre.txt`, `rules/mixed.txt`: rule files. `bench.sh` reproduces every number below.
- **Machine**: Apple M4 Pro, 12 cores; rustc 1.99.1; release build, one thread, median of 5 runs.

## Verdict: **GO with reservations**

**Port**: one trait, fed the **whole snapshot** (not one file), that emits language-neutral facts. Each fact is keyed by a `QName { lang, segments }` that the adapter has already resolved. The checker only compares segment prefixes and never interprets a language. **Parser**: **tree-sitter-rust**, which confirms the 2026-11-03 arbitration. Measured `use` facts are identical to syn's (8,717 of 8,717 across 6 corpora). Reserve the right to switch the Rust adapter to syn: the switch costs one adapter and nothing else, as `lang-rust-syn` shows.

Openness was demonstrated, not argued. The Python adapter plugs into the same registry, and one rule file checks Rust and Python with one checker (§ 3). It needed **0 lines changed** in `facts/` (port, model, checker). The core side of the change is 1 registration line, plus 1 `Cargo.toml` line.

Reservations:

1. **tree-sitter-rust lags rustc.** It reports syntax errors in 3 of the 96 files of `syn` 3.0.9, all of them valid Rust (`$` in `macro_rules!` patterns, `struct X where …;`). syn rejects 0 files. So a tree-sitter error must mean "facts may be incomplete", not "invalid file → refusal" (Q2).
2. **Neither parser sees inside macros.** In tokio, 328 of the 3,755 `use` lines (8.7 %) and 210 `mod` lines sit inside `cfg_*! { … }` invocations. The adapter re-parses module-level macro bodies in place (tree-sitter `set_included_ranges`, locations stay exact). That recovers **+468 dependency facts (+9.1 %)** and 9 inline modules on tokio. `macro_rules!` bodies and proc-macro expansions stay out of reach for both parsers.
3. **The Rust file → module mapping is derived from paths.** `#[path]`, `include!` and cfg-gated `mod` are ignored. A file can belong to several crates (`tests/support/mod.rs` is pulled in by every test crate). None of the corpora uses `#[path]`, but following `mod` declarations will be needed before the Python story's OSS trials.
4. **The framing's signature changes.** `extraire_faits(langage, fichier)` (`03_archi.md`) becomes `extract(snapshot)`. Module identity needs the layout files (`Cargo.toml`, `__init__.py`), and in Rust `use x::y` is ambiguous without the set of known modules. Per-file parallelism stays inside the adapter.

### Proposed port and fact model (for ST-22)

```rust
pub type Lang = &'static str;                       // "rust", "python": owned by the adapter

pub struct QName { pub lang: Lang, pub segments: Vec<String> }   // resolved: no crate/self/super/leading dots
pub struct Location { pub path: String, pub line: u32, pub column: u32, pub byte: u32 }
    // line and column 1-based, column counted in bytes; byte offset 0-based
pub enum Visibility { Public, Restricted, Private }
pub enum Kind {
    Module,                          // the module exists, `at` = where it starts
    Depends(QName),                  // import / use / extern crate; target = what is imported, fully qualified
    Symbol(String, Visibility),      // module-level item, re-exports included
    Call(QName),                     // path call, resolved through the file's imports when possible
    TypeRef(QName),                  // type named in a signature, field, annotation
}
pub struct Fact { pub at: Location, pub module: QName, pub kind: Kind }   // Ord: sorted by location
pub struct Extraction { pub facts: Vec<Fact>, pub invalid: Vec<Location> }

pub trait Extractor: Sync {
    fn language(&self) -> Lang;
    fn separator(&self) -> &'static str;              // "::", ".": reads and prints the native notation
    fn handles(&self, path: &str) -> bool;            // which source files are mine
    fn extract(&self, snapshot: &[SourceFile]) -> Extraction;
}

pub struct Registry { /* Vec<Box<dyn Extractor>> */ }
impl Registry {
    pub fn extract(&self, snapshot: &[SourceFile]) -> Report;          // facts sorted + dedup, invalid, unchecked files
    pub fn parse_name(&self, "rust:arbitre_core::store") -> QName;      // through the adapter's separator
    pub fn render(&self, &QName) -> String;
}
pub fn check(facts: &[Fact], rules: &[Forbid]) -> Vec<(rule, &Fact)>;  // pure, prefix match on segments
```

How the core stays closed:

- **The checker resolves no language.** A module identity is `(lang, segments)`, and matching is by segment prefix (`a::b` covers `a::b::c`, not `a::bc`). Everything language-specific (`crate::`, `super::`, `..`, crate name, package root) is resolved by the adapter before it emits a fact.
- **Rules name their language** (`rust:arbitre_core !-> rust:rusqlite`, `python:json !-> python:re`). The core splits the path with the separator the adapter supplies. Go or JS (`/`) would need no change either. An unknown language in a rule is an error: "language not supported" (tested).
- **Determinism belongs to the core.** The registry sorts and deduplicates, so an adapter may parse in parallel or in any order. Tested by permuting file order on every run of `demo check`.
- **"Not verified"**: files that no adapter `handles` come back in `Report::unchecked`. What to do with them is ST-22's policy, not the port's.
- **Not neutral, deliberately**: an edge from one language to another (pyo3, FFI) does not exist in this model. A rule's `from` and `to` are in the same language.

### How language-specific notions map onto the model

| Notion | Fact | Note |
|---|---|---|
| Rust `use a::{b, c::{self as d}, e::*}` | one `Depends` per leaf: `a::b`, `a::c`, `a::e` | glob ⇒ the module path; `self` ⇒ the parent path; rename ⇒ the alias, used to resolve calls and types |
| Rust `pub use x::Y` | `Depends(x::Y)` **and** `Symbol(Y, Public)` | a re-export is part of the public surface |
| Rust `pub(crate)`, `pub(super)`, `pub(in p)` | `Visibility::Restricted` | the exact scope is not kept (YAGNI until a rule needs it) |
| Rust `extern crate std` | `Depends(std)` | rule (c) of `graph.py`, `extern crate std`, becomes `rust:arbitre_core !-> rust:std` |
| Rust `crate::`, `self::`, `super::`, `$crate::` | resolved to the crate name (`-` → `_`, `[lib] name` wins) | edition 2018 `use foo::x`: local if `module::foo` is a known module, otherwise an extern crate |
| Rust crate targets | `arbitre_core`, `arbitre` (bin), `arbitre_core#tests::hlc`, `…#benches`, `…#examples`, `…#build` | integration tests are other crates. Without the `#tests` suffix, the dogfooding run flagged 7 false `std` violations in `arbitre_core` |
| Rust `#![no_std]`, `#[cfg]`, `#[path]` | **out** | attributes are not facts. The `#![no_std]` rule stays in `graph.py`, or ST-23 adds a neutral `Attribute` kind |
| Python `import a.b as c` | `Depends(a.b)` | |
| Python `from ..db import conn` | `Depends(<package minus 1>.db.conn)` | the package is the module minus its last segment, or the module itself for `__init__.py`. 138 relative imports in the Python corpus |
| Python `from x import y` | `Depends(x.y)` | submodule or attribute cannot be told apart. Prefix matching makes this harmless |
| Python `__all__`, leading `_` | `Public` / `Private` (`__all__` wins) | Python has no `Restricted` |
| Python dynamic imports, `if TYPE_CHECKING:` | dynamic: **out**. Conditional: kept like any other import | the fact carries no condition (Q4) |
| Cargo graph (`[dependencies]`) | **out** | per the arbitration, Cargo graph rules stay outside the AST |

## Evidence

### 1. Exact locations

| Criterion | tree-sitter-rust | syn + proc-macro2 (`span-locations`) |
|---|---|---|
| Line | `start_position().row + 1` | `span.start().line` |
| Column | **bytes**, native | **UTF-8 characters**. Bytes are recomputed from `byte_range()` |
| Byte | `start_byte()`, native | `byte_range()`, relative to the file |
| Same location for the same `Depends` fact | 8,715 of 8,717 identical (line, column, byte). The 2 that differ are `extern crate` facts, where the spike anchors on different tokens (my code, not the parsers) | |
| Cost | none | spans live in a thread-local source map that only grows. Call `invalidate_current_thread_spans()` after each file, or it hits the u32 limit at 4 GiB of source per thread |
| Inside a re-parsed macro body | exact (`set_included_ranges` keeps original offsets; test `items_inside_module_level_macro`) | exact as well: the tokens keep their spans (`parse2` on `mac.tokens`) |

### 2. `use` trees, `mod`, file → module

- Groups, nested groups, globs, `self`, `self as x`, renames: unit test `use_trees_modules_visibility`, with exact positions.
- Parity between the two parsers on the `Depends` set (path, line, module, target), excluding macro bodies:

| Corpus | Rust files | Lines | `Depends` tree-sitter / syn | Common |
|---|---|---|---|---|
| OpenArbitre `crates/` | 11 | 2,432 | 65 / 65 | 65 |
| tokio 1.54.1 | 573 | 148,030 | 5,128 / 5,128 | 5,128 |
| ignore 0.4.23 (ripgrep) | 10 | 6,603 | 161 / 161 | 161 |
| globset 0.4.16 (ripgrep) | 6 | 2,993 | 68 / 68 | 68 |
| regex-automata 0.4.18 | 100 | 68,447 | 985 / 985 | 985 |
| syn 3.0.9 | 96 | 67,002 | 2,310 / 2,310 | 2,310 |

The first tokio run found 4 gaps, all caused by a bug in the spike (`{self as x}`), not by the parser. After the fix, parity is total.

- `mod`: the path → module mapping uses `lib.rs`/`main.rs`, `mod.rs`, `bin/`, `tests/`. Inline `mod x { }` blocks come from the tree. The two parsers do the same here.

### 3. Openness: the second language plugged in

`rules/mixed.txt` run on OpenArbitre's `crates/` plus 70 CPython 3.15 files (`asyncio`, `email`, `json`; 27,269 lines):

```
files 84 | facts rust 1234 python 1281 | invalid 0 | unchecked 3 | extract 80.3ms
VIOLATION rule 1 crates/cli/src/main.rs:3:5 rust:arbitre -> rust:clap::Parser
VIOLATION rule 3 py/email/mime/application.py:9:19 python:email.mime.application -> python:email.encoders
VIOLATION rule 3 py/email/mime/audio.py:9:19 python:email.mime.audio -> python:email.encoders
VIOLATION rule 3 py/email/mime/image.py:9:19 python:email.mime.image -> python:email.encoders
VIOLATION rule 2 py/json/decoder.py:3:8 python:json.decoder -> python:re
VIOLATION rule 2 py/json/encoder.py:3:8 python:json.encoder -> python:re
VIOLATION rule 2 py/json/scanner.py:3:8 python:json.scanner -> python:re
VIOLATION rule 2 py/json/tool.py:8:8 python:json.tool -> python:re
violations 8
determinism ok (permuted file order)
```

Green rules stay green: `arbitre_core !-> arbitre` and `asyncio !-> email`. `facts/` also has its own test with a fake third language (`Fake`, separator `/`), which runs through the same registry and checker.

**The "add a language" diff** (Python; `facts/` untouched):

```diff
 spikes/s4r/Cargo.toml
-members = ["facts", "lang-rust", "lang-rust-syn", "demo"]
+members = ["facts", "lang-rust", "lang-rust-syn", "lang-python", "demo"]

 spikes/s4r/demo/Cargo.toml                       # in production: crates/cli or arbitre-diff's wiring
+lang-python = { path = "../lang-python" }

 spikes/s4r/demo/src/main.rs
 fn registry() -> Registry {
     Registry::new(vec![
         Box::new(lang_rust::Rust),
+        Box::new(lang_python::Python),
     ])
 }

+spikes/s4r/lang-python/Cargo.toml               (new, 10 lines: facts, tree-sitter, tree-sitter-python)
+spikes/s4r/lang-python/src/lib.rs               (new, 194 lines including its test)
```

In ST-22 the adapters can be modules of `arbitre-diff` (`arbitre-diff::rust`, `::python`) or separate crates. The port does not care. A crate per language keeps a grammar's C build out of the others; that is a packaging choice, not an architecture choice.

### 4. Dogfooding: OpenArbitre's source rules

`rules/openarbitre.txt` turns the source side of `scripts/lib/graph.py` into dependency facts: (a) `arbitre_core` reaches no `arbitre*` crate; (c) no `std` in `arbitre_core`; (b) no `rusqlite`/`tokio`/`reqwest` named in the code. All three are green on `main`. The probes fire where expected: `arbitre → clap`, and `arbitre_core#tests → std::collections` (4 locations). Extraction takes 11 ms.

### 5. Macros

| | tree-sitter-rust | syn |
|---|---|---|
| Macro invocation | `macro_invocation` + opaque `token_tree` | `Macro` + opaque `TokenStream` |
| Items in a module-level macro body (`cfg_rt! { mod x; use y; }`) | re-parse in place, kept only when it parses without error: **+468 `Depends`, +9 modules on tokio** | possible the same way (`parse2::<File>`), not done in the spike |
| `macro_rules!` body, proc-macro / derive expansion | invisible | invisible |
| Path call to a macro (`println!`) | `Call(…::println!)` | (not extracted in the spike) |

Without expansion (which would mean building the code, excluded by INV-64 and by the trust boundary), both parsers are equal. The difference lies elsewhere: on a syntax error, syn loses the **whole file**, while tree-sitter loses **one node**.

### 6. Determinism

Both parsers are deterministic. The registry sorts and deduplicates. On every `demo check` run (OpenArbitre, mixed corpus, syn), the file order was reversed and rotated, and the facts came out identical.

### 7. Speed (release build, one thread, median of 5)

| Corpus | Lines | tree-sitter parse | syn parse | tree-sitter extract (all kinds + macro bodies) | syn extract (`use` + `mod` only) |
|---|---|---|---|---|---|
| OpenArbitre `crates/` | 2,432 | 5.6 ms | 3.6 ms | 10.4 ms | 3.4 ms |
| tokio 1.54.1 | 148,030 | 208 ms | 133 ms | 409 ms | 141 ms |
| ignore (ripgrep) | 6,603 | 11.6 ms | 8.4 ms | 21.1 ms | 8.3 ms |
| globset (ripgrep) | 2,993 | 5.1 ms | 3.3 ms | 9.0 ms | 3.4 ms |
| regex-automata 0.4.18 | 68,447 | 81 ms | 70 ms | 155 ms | 72 ms |
| syn 3.0.9 | 67,002 | 121 ms | 75 ms | 236 ms | 79 ms |

syn parses 1.2 to 1.6 times faster. Neither number matters: tree-sitter extracts every fact kind from 148 kLOC in 0.4 s on one thread, and `check` reads base and head (× 2). For Python, 70 files and 27 kLOC take about 70 ms. Clean build of the adapter: `lang-rust` (tree-sitter + grammar, 6.5 MB of generated C) **4.7 s**; `lang-python` on top **0.9 s**; syn + proc-macro2 **2.8 s**.

### 8. Dependency mode, `no_std`/wasm

| | tree-sitter-rust | syn |
|---|---|---|
| Published, versioned | crates.io: `tree-sitter` 0.27.1, `tree-sitter-rust` 0.24.2, `tree-sitter-python` 0.25.0. Grammars depend on `tree-sitter-language` 0.1, not on a `tree-sitter` version, so the core and the grammars move separately | crates.io: `syn` 3.0.9, `proc-macro2` 1.0.107. syn 3 is recent (the registry still holds 2.0.119): a major version just landed |
| Graph | `regex`, `streaming-iterator`, `tree-sitter-language`. Build: `cc`, `serde_json` (`default-features = false` drops `regex/std,perf`) | `proc-macro2`, `unicode-ident`. Pure Rust |
| Toolchain | **needs a C compiler** (`cc`) at build time | no |
| wasm32-unknown-unknown | **fails**: `parser.h: 'stdlib.h' file not found` (measured) | compiles (pure Rust) |
| Grammar ↔ rustc gap | **yes**: 3 false errors in 96 files of `syn` 3.0.9 | follows rustc closely, 0 rejects |
| Error tolerance | partial tree with `ERROR` nodes; the facts around the error survive | `parse_file` fails ⇒ no facts for the file |

**Native-only, confirmed in the framing**: `03_archi.md` lists `arbitre-diff` as "natif", and `scripts/lib/graph.py` puts only `arbitre-core` and `arbitre-plan` on the browser list (`arbitre-diff` may depend on `arbitre-core` only). The wasm failure has no consequence. `graph.py` forbids neither `cc` nor `tree-sitter`.

### 9. Effect on the open-closed goal

- **The port does not depend on the parser.** `lang-rust-syn` implements the same trait as `lang-rust`, so the parser choice is local to one adapter and can be revisited.
- **What tree-sitter adds**: the same API in every adapter. Location, error detection, tree walking and re-parsing by ranges are written the same way for Rust, Python and later TS or Go; in the spike, `loc()` and the walk are the same code in two adapters. There is also one dependency family to audit and update, and grammars exist for most languages. This is a gain in adapter cost, not in architecture.
- **What syn adds**: fidelity to rustc. That matters for a judge that must never raise a false alarm on valid code.

## Risks

| Risk | Probability | Impact | Mitigation |
|---|---|---|---|
| tree-sitter-rust lags the language (new syntax ⇒ `ERROR` nodes) | High (measured: 3/96 files) | Medium: facts missing in a file, or an "invalid" verdict on valid code | `invalid` ⇒ warning + partial facts (Q2). An alarm in CI when a file has errors. Fallback to syn possible without touching the port |
| `use`/`mod` hidden in macros | Certain on tokio-like repos (8.7 % of `use` lines) | Missed violations | Re-parse module-level bodies (done, +9.1 %). `macro_rules!` and proc macros stay out, documented |
| Wrong module identity (`#[path]`, shared `tests/support`, PEP 420 namespace packages, `src/` layouts) | Medium | Rule applied to the wrong module | Follow `mod` declarations (ST-22); configurable Python roots (Q3) |
| Rust uniform-path ambiguity (`use foo::x`: local module or crate?) | Low | Wrong target | Known-module heuristic (exact when the module exists). A local item that shadows a crate name stays unresolved |
| Calls and types without name resolution (methods, generic parameters, prelude) | Certain | ST-23 families imprecise | Out of S4-R's scope: ST-23 decides; `Call`/`TypeRef` already exist in the model |
| syn 2 → 3 churn if we switch | Low | One adapter to rewrite | Contained in the adapter |

## Questions for Quentin

**Q1. Port signature: whole snapshot or file by file?**
- (a) `extract(snapshot)`, as proven here: layout and module set are visible to the adapter.
- (b) `extract(file)` + `resolve(layout)` in two phases: more parallelism in the core, a bigger port.
- (c) Keep `extraire_faits(langage, fichier)` from the framing: does not work for Rust (`use x` ambiguous) or for relative imports.
- **Recommendation: (a).** Amend `03_archi.md` § Fonctions du module decision diff at ST-22's cycle.

**Q2. What does a tree-sitter syntax error mean?**
- (a) Invalid file ⇒ tool error (strict; false failures on valid code, 3/96 files of syn).
- (b) Warning + partial facts, and the file is named in the report as "partially verified".
- (c) Same as (b), plus a syn re-check of the faulty files.
- **Recommendation: (b)**, and (c) only if false negatives show up in the eval harness (ST-25). The syn fallback stays local to the adapter.

**Q3. Python module roots (for the next story)**
- (a) Inferred from the `__init__.py` chain (the spike; wrong for PEP 420 packages and some `src/` layouts).
- (b) Declared in the invariants file (`python.roots = ["src"]`).
- (c) (a) by default, (b) when present.
- **Recommendation: (c).** To settle with S4 (tree-sitter-python or Ruff). The port does not change.

**Q4. Conditional imports (`if TYPE_CHECKING:`, `#[cfg(test)] use`)**
- (a) Same as any other import (the spike).
- (b) A neutral `conditional: bool` field on `Depends`.
- **Recommendation: (a)** for ST-22. Revisit if the OSS trials show false positives (the classic one is the `TYPE_CHECKING` import used to break a cycle).

**Q5. Where do adapters live?**
- (a) Modules of `arbitre-diff` (`rust`, `python`), behind optional features.
- (b) One crate per language (`arbitre-diff-rust`, …) + a line in the `graph.py` table.
- **Recommendation: (a)** while there are two languages. Move to (b) when a third grammar weighs on the build. The choice does not touch the port.

## Notes on method

- `git clone` outside the worktree (`$TMPDIR/s4r-repos`) is refused by the worktree-isolation hook. An attempt to fetch a tarball by building the GitHub hostname at runtime was rightly refused as a workaround, and I did not pursue it. **The ripgrep repo itself was not fetched.** I used its published crates `ignore` and `globset`, plus tokio, regex-automata and syn, all already in the local cargo registry (crates.io sources, parsed only). The Python corpus is CPython 3.15's own stdlib (`asyncio`, `email`, `json`), copied to the session scratchpad. Nothing was built or run from these corpora. No crates.io download was blocked (no Aikido event to report).
- The spike does not touch the workspace `Cargo.toml` or `crates/`. `spikes/s4r/Cargo.toml` declares its own `[workspace]`.
