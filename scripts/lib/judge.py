"""Judge (T-11, A-8): run from `main` by `judge.yml`; reads the PR as data, executes none of it.

Usage: `CI_LABELS=… python3 scripts/lib/judge.py <base> <head>`. Python >= 3.13, standard library only.
"""

import os
import subprocess
import sys

from mutation import ZONES, RedError, read_zones, zone_of

PROTECTED = (".github/", "scripts/", ".cargo/", ".gitleaksignore")
LABEL = "ci-change"


def judge(
    changes: list[str],
    zones_base: dict[str, str],
    zones_head: dict[str, str],
    rs_files: list[str],
    labels: set[str],
) -> tuple[bool, list[str]]:
    """Returns `(red, lines)`.

    `changes`: `git diff --name-only --no-renames MB HEAD`; `zones_*`: {path: text} of the `zones.toml` files;
    `rs_files`: `*.rs` of `git ls-tree -r MB`.
    Rule 1: protected path touched. Rule 2: effective zone of a base `.rs` file lowered;
    unreadable head `zones.toml` ⇒ red. Without label: non-empty list ⇒ red.
    With label: lists shown, red only if a path under `crates/` is touched.
    """
    lines = [f"protected path touched: {c}" for c in changes if c.startswith(PROTECTED)]
    try:
        base = read_zones(zones_base)
        head = read_zones(zones_head)
    except RedError as e:
        return True, [f"unreadable zones: {e}"]
    for f in rs_files:
        before, after = zone_of(f, base), zone_of(f, head)
        if ZONES.index(after) > ZONES.index(before):
            lines.append(f"zone lowered: {f} ({before} -> {after})")
    if LABEL not in labels:
        return bool(lines), lines
    under_crates = [c for c in changes if c.startswith("crates/")]
    return bool(under_crates), lines + [f"with {LABEL}, the PR must not touch crates/: {c}" for c in under_crates]


def git(*args: str) -> str:
    r = subprocess.run(("git", *args), capture_output=True, text=True, check=True)  # noqa: S603, S607 -- fixed git
    return r.stdout


def main(argv: list[str]) -> int:
    """Wraps `git` (`merge-base`, `diff`, `ls-tree`, `show`), reads `CI_LABELS`, calls `judge`. 0 = green."""
    base, head = argv
    mb = git("merge-base", base, head).strip()
    changes = git("diff", "--name-only", "--no-renames", "-z", mb, head).split("\0")[:-1]
    tree = git("ls-tree", "-r", "--name-only", "-z", mb).split("\0")[:-1]
    rs_files = [f for f in tree if f.endswith(".rs")]

    def zones(rev: str) -> dict[str, str]:
        paths = git("ls-tree", "-r", "--name-only", "-z", rev).split("\0")[:-1]
        return {p: git("show", f"{rev}:{p}") for p in paths if p.startswith(".ctdd/stories/") and p.endswith("/zones.toml")}

    labels = {x for x in os.environ.get("CI_LABELS", "").split(",") if x}
    red, lines = judge(changes, zones(mb), zones(head), rs_files, labels)
    for line in lines:
        print(f"judge: {line}", file=sys.stderr)
    print(f"judge: {'red' if red else 'green'}")
    return 1 if red else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
