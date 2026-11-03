"""Dependency graph check (T-8): rules (a) to (e) of the ST-01 design, amended by A-6 and A-9.

Python >= 3.13, standard library only. Each rule returns a list of failure lines (empty = green).
"""

import json
import re
import subprocess
import sys
import tomllib
from collections import deque
from pathlib import Path

# A-9: components reachable, transitively, through any kind of dependency.
ALLOWED: dict[str, frozenset[str]] = {
    "arbitre-core": frozenset(),
    "arbitre-plan": frozenset({"arbitre-core"}),
    "arbitre-diff": frozenset({"arbitre-core"}),
    "arbitre-sqlite": frozenset({"arbitre-core"}),
    "arbitre-relay": frozenset({"arbitre-core"}),
    "arbitre-ai": frozenset({"arbitre-core"}),
    "arbitre-calendar": frozenset({"arbitre-plan", "arbitre-core"}),
    "arbitre-web": frozenset({"arbitre-core", "arbitre-plan", "arbitre-sqlite"}),
    "arbitre": frozenset(
        {"arbitre-core", "arbitre-plan", "arbitre-diff", "arbitre-sqlite",
         "arbitre-relay", "arbitre-ai", "arbitre-calendar"}
    ),
}

# RG-ST01-6.2: browser list.
BROWSER_LIST = ("arbitre-core", "arbitre-plan")

# Rule (b): forbidden in the normal graph of the browser list; `chrono` only with `clock`.
FORBIDDEN = ("rusqlite", "libsqlite3-sys", "sqlite-wasm-rs", "tokio", "reqwest", "keyring")
FORBIDDEN_BY_FEATURE = {"chrono": "clock"}

# Rule (d).
FORBIDDEN_TLS = ("openssl-sys", "native-tls")

SECTIONS = ("dependencies", "dev-dependencies", "build-dependencies")
# `name vX.Y.Z (path) features` of a `cargo tree -f '{p} {f}'` line, tree prefix removed.
TREE_LINE = re.compile(r"(\S+) v\S+(?: \([^)]*\))?\s*(.*)$")


def rule_edges(metadata: dict) -> list[str]:
    """(a) A-9: a component reached outside its table ⇒ a line with the chain; workspace crate missing from the table ⇒ a line.

    `metadata`: JSON of `cargo metadata --format-version 1`.
    """
    names = {p["id"]: p["name"] for p in metadata["packages"]}
    members = {i: names[i] for i in metadata["workspace_members"]}
    deps = {n["id"]: [d["pkg"] for d in n["deps"]] for n in metadata["resolve"]["nodes"]}
    lines = [
        f"{name}: workspace crate missing from the allowed table (foundation amendment required)"
        for name in sorted(members.values())
        if name not in ALLOWED
    ]
    for start, name in sorted(members.items(), key=lambda t: t[1]):
        if name not in ALLOWED:
            continue
        parent = {start: None}
        queue = deque([start])
        while queue:
            cur = queue.popleft()
            for nxt in deps.get(cur, []):
                if nxt in parent:
                    continue
                parent[nxt] = cur
                queue.append(nxt)
                if nxt in members and members[nxt] not in ALLOWED[name]:
                    chain, i = [], nxt
                    while i is not None:
                        chain.append(names[i])
                        i = parent[i]
                    lines.append(f"{name} reaches {members[nxt]}, forbidden: {' -> '.join(reversed(chain))}")
    return lines


def rule_forbidden(trees: dict[str, str]) -> list[str]:
    """(b) Forbidden dependencies in the normal graph of a browser-list crate.

    `trees`: {crate: output of `cargo tree -p <crate> -e normal,features --target wasm32-unknown-unknown -f '{p} {f}'`}.
    The message names the crate and the dependency (RG-ST01-5.1).
    """
    lines = []
    for crate, tree in sorted(trees.items()):
        found = set()
        for raw in tree.splitlines():
            m = TREE_LINE.match(re.sub(r"^[\s│├└─]+", "", raw))
            if not m:
                continue
            name, features = m.group(1), set(m.group(2).replace("(*)", "").strip().split(","))
            if name in FORBIDDEN:
                found.add(name)
            elif name in FORBIDDEN_BY_FEATURE and FORBIDDEN_BY_FEATURE[name] in features:
                found.add(f"{name} (feature {FORBIDDEN_BY_FEATURE[name]})")
        lines += [f"{crate}: forbidden dependency in the browser graph: {t}" for t in sorted(found)]
    return lines


def rule_purity(sources: dict[str, dict[str, str]]) -> list[str]:
    """(c) A-6: for each present crate of {arbitre-core, arbitre-plan}, `src/lib.rs` contains `#![no_std]`
    and no source contains `extern crate std`.

    `sources`: {crate: {path: content}}.
    """
    lines = []
    for crate in BROWSER_LIST:
        files = sources.get(crate)
        if files is None:
            continue
        libs = [t for p, t in files.items() if p.endswith("src/lib.rs")]
        if not any(re.search(r"^#!\[no_std\]", t, re.M) for t in libs):
            lines.append(f"{crate}: src/lib.rs without #![no_std]")
        for path, text in sorted(files.items()):
            if re.search(r"^\s*extern\s+crate\s+std\b", text, re.M):
                lines.append(f"{crate}: {path} contains extern crate std")
    return lines


def rule_tls(metadata: dict) -> list[str]:
    """(d) Neither `openssl-sys` nor `native-tls` anywhere in the workspace."""
    return [
        f"workspace: {name} present (rustls only, PL-6)"
        for name in sorted({p["name"] for p in metadata["packages"]})
        if name in FORBIDDEN_TLS
    ]


def rule_shared(root: dict, manifests: dict[str, dict]) -> list[str]:
    """(e) RG-ST01-5.2: a `[workspace.dependencies]` dependency is inherited with `workspace = true`,
    and a dependency used by at least two components is declared at workspace level.

    `root`: root `Cargo.toml` read by `tomllib`; `manifests`: {crate: its `Cargo.toml` read by `tomllib`}.
    """
    shared = set(root.get("workspace", {}).get("dependencies", {}))
    lines, usage = [], {}
    for crate, manifest in sorted(manifests.items()):
        tables = [manifest.get(k, {}) for k in SECTIONS]
        for target in manifest.get("target", {}).values():
            tables += [target.get(k, {}) for k in SECTIONS]
        for table in tables:
            for dep, spec in table.items():
                spec = spec if isinstance(spec, dict) else {}
                usage.setdefault(dep, set()).add(crate)
                if dep in shared and spec.get("workspace") is not True:
                    lines.append(f"{crate}: {dep} is in the workspace, to be inherited with workspace = true")
    for dep, crates in sorted(usage.items()):
        if len(crates) >= 2 and dep not in shared and not dep.startswith("arbitre"):
            lines.append(f"{dep} used by {', '.join(sorted(crates))}: to be declared in [workspace.dependencies]")
    return lines


def run(*cmd: str) -> str:
    r = subprocess.run(cmd, capture_output=True, text=True)  # noqa: S603, S607 -- fixed cargo commands
    if r.returncode:
        raise SystemExit(f"graph: {' '.join(cmd)} failed: {r.stderr.strip()}")
    return r.stdout


def main(argv: list[str]) -> int:
    """Runs `cargo metadata` and `cargo tree`, reads manifests and sources, applies (a) to (e). 0 = green."""
    metadata = json.loads(run("cargo", "metadata", "--format-version", "1", "--locked"))
    workspace_root = Path(metadata["workspace_root"])
    members = {
        p["name"]: Path(p["manifest_path"]) for p in metadata["packages"] if p["id"] in metadata["workspace_members"]
    }
    present = [c for c in BROWSER_LIST if c in members]
    trees = {
        c: run("cargo", "tree", "--locked", "-p", c, "-e", "normal,features",
               "--target", "wasm32-unknown-unknown", "-f", "{p} {f}")
        for c in present
    }
    sources = {
        c: {str(f.relative_to(workspace_root)): f.read_text() for f in sorted(members[c].parent.rglob("src/**/*.rs"))}
        for c in present
    }
    manifests = {c: tomllib.loads(m.read_text()) for c, m in members.items()}
    root = tomllib.loads((workspace_root / "Cargo.toml").read_text())
    lines = (
        rule_edges(metadata) + rule_forbidden(trees) + rule_purity(sources)
        + rule_tls(metadata) + rule_shared(root, manifests)
    )
    for line in lines:
        print(f"graph: {line}", file=sys.stderr)
    return 1 if lines else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
