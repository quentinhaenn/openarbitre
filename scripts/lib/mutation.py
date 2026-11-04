"""Mutation measurement (T-9): zones, equivalents, scores, verdict, report.

Formats: `CONTRIBUTING.md` and ST-01 design, Formats. Python >= 3.13, standard library only.
`judge.py` reuses `read_zones` and `zone_of` (A-8).
"""

import json
import os
import sys
import tomllib
from dataclasses import dataclass
from fractions import Fraction
from pathlib import Path, PurePosixPath

THRESHOLDS = {"Full": 0.85, "Std": 0.70}
# Strictest to laxest (RG-ST01-7.2).
ZONES = ("Full", "Std", "Gen")
# MANQUE-ST01-1: `Gen` forbidden under these prefixes.
NO_GEN = ("crates/core/", "crates/plan/")
# `summary` of cargo-mutants 27.1.0 (observed, T-9) -> result.
SUMMARY = {"CaughtMutant": "caught", "Timeout": "timeout", "MissedMutant": "missed", "Unviable": "unviable"}


class RedError(Exception):
    """Unreadable or forbidden input: the check is red, the message names the file."""


@dataclass(frozen=True)
class Outcome:
    """Result of one mutant. `result` is one of caught, timeout, missed, unviable."""

    name: str
    file: str
    result: str


@dataclass(frozen=True)
class Score:
    file: str
    zone: str
    detected: int
    counted: int


def covers(pattern: str, path: str) -> bool:
    """`PurePosixPath(path).full_match(pattern)`: `*` stays within a segment, `**` covers 0 to n segments."""
    return PurePosixPath(path).full_match(pattern)


def _toml(path: str, text: str) -> dict:
    try:
        return tomllib.loads(text)
    except tomllib.TOMLDecodeError as e:
        raise RedError(f"{path}: unreadable TOML ({e})") from e


def read_zones(contents: dict[str, str]) -> dict[str, list[str]]:
    """Reads `zones.toml` files given by content ({path: text}); returns the merged {zone: patterns}.

    Invalid TOML, key outside `ZONES` or unexpected type ⇒ `RedError` naming the file (RG-ST01-7.2).
    """
    zones: dict[str, list[str]] = {z: [] for z in ZONES}
    for path, text in sorted(contents.items()):
        for key, patterns in _toml(path, text).items():
            if key not in ZONES:
                raise RedError(f"{path}: unknown key {key!r} (expected: {', '.join(ZONES)})")
            if not isinstance(patterns, list) or not all(isinstance(p, str) for p in patterns):
                raise RedError(f"{path}: {key} must be a list of patterns")
            zones[key] += patterns
    return zones


def zone_of(path: str, zones: dict[str, list[str]]) -> str:
    """Strictest zone covering `path`, `Full` by default (INV-ST01-15)."""
    for z in ZONES:
        if any(covers(p, path) for p in zones.get(z, [])):
            return z
    return "Full"


def gen_guard(paths: list[str], zones: dict[str, list[str]]) -> list[str]:
    """Files under `NO_GEN` resolved to `Gen` (MANQUE-ST01-1); empty list = green."""
    return [p for p in paths if p.startswith(NO_GEN) and zone_of(p, zones) == "Gen"]


def gen_files(paths: list[str], zones: dict[str, list[str]]) -> list[str]:
    """Files whose effective zone is `Gen`: excluded from CI mutation by `50-st01-mutation.sh`."""
    return [p for p in paths if zone_of(p, zones) == "Gen"]

def read_equivalents(contents: dict[str, str]) -> set[str]:
    """Exact names of the equivalent mutants from all `mutants_equivalents.toml` files.

    Empty or missing justification, unknown key, invalid TOML ⇒ `RedError` (US-7 case 8).
    """
    names: set[str] = set()
    for path, text in sorted(contents.items()):
        data = _toml(path, text)
        if set(data) - {"equivalent"}:
            raise RedError(f"{path}: unknown key {sorted(set(data) - {'equivalent'})}")
        for entry in data.get("equivalent", []):
            if set(entry) - {"mutant", "justification"}:
                raise RedError(f"{path}: unknown key in {entry}")
            mutant, justification = entry.get("mutant"), entry.get("justification")
            if not isinstance(mutant, str) or not mutant:
                raise RedError(f"{path}: missing mutant")
            if not isinstance(justification, str) or not justification.strip():
                raise RedError(f"{path}: empty or missing justification for {mutant!r}")
            names.add(mutant)
    return names


def read_outcomes(outcomes: dict) -> list[Outcome]:
    """Outcomes of `mutants.out/outcomes.json` (cargo-mutants 27.1.0). Red baseline or unreadable format ⇒ `RedError`."""
    try:
        results = []
        for o in outcomes["outcomes"]:
            scenario, summary = o["scenario"], o["summary"]
            if scenario == "Baseline":
                if summary != "Success":
                    raise RedError(f"tests failing without mutation (red baseline): {summary}")
                continue
            m = scenario["Mutant"]
            if summary not in SUMMARY:
                raise RedError(f"unknown outcome {summary!r} for {m['name']}")
            results.append(Outcome(m["name"], m["file"], SUMMARY[summary]))
        return results
    except (KeyError, TypeError) as e:
        raise RedError(f"unreadable outcomes.json ({e!r})") from e


def scores(outcomes: list[Outcome], equivalents: set[str], zones: dict[str, list[str]]) -> list[Score]:
    """Score per file: `caught`/`timeout` detected, `unviable` and equivalents excluded. Sorted by file."""
    per_file: dict[str, list[int]] = {}
    for o in outcomes:
        d, c = per_file.setdefault(o.file, [0, 0])
        if o.name in equivalents or o.result == "unviable":
            continue
        per_file[o.file] = [d + (o.result in ("caught", "timeout")), c + 1]
    return [Score(f, zone_of(f, zones), d, c) for f, (d, c) in sorted(per_file.items())]


def verdict(s: Score) -> str:
    """`neutral` if nothing is counted; `info` in `Gen`; otherwise `green` if score ≥ threshold, `red` if not."""
    if s.counted == 0:
        return "neutral"
    if s.zone == "Gen":
        return "info"
    return "green" if s.detected >= Fraction(str(THRESHOLDS[s.zone])) * s.counted else "red"


def report(scored: list[Score]) -> str:
    """Markdown table (file, zone, score, threshold, verdict), sorted by file, no duration or timestamp (C-7.4)."""
    lines = ["| file | zone | score | threshold | verdict |", "|---|---|---|---|---|"]
    for s in sorted(scored, key=lambda s: s.file):
        score = f"{100 * s.detected / s.counted:.1f}% ({s.detected}/{s.counted})" if s.counted else "—"
        threshold = f"{round(100 * THRESHOLDS[s.zone])}%" if s.zone in THRESHOLDS else "—"
        lines.append(f"| {s.file} | {s.zone} | {score} | {threshold} | {verdict(s)} |")
    return "\n".join(lines) + "\n"


def _read(pattern: str) -> dict[str, str]:
    return {p.as_posix(): p.read_text() for p in sorted(Path().glob(pattern))}


def main(argv: list[str]) -> int:
    """Reads `mutants.out`, the repository's zones and equivalents, writes `$CI_OUT/mutation.md`. 0 = green."""
    out = Path(os.environ.get("CI_OUT", "target/ci"))
    try:
        zones = read_zones(_read(".ctdd/stories/*/zones.toml"))
        if "--gen-files" in argv:
            print("\n".join(gen_files(sorted(p.as_posix() for p in Path("crates").rglob("*.rs")), zones)))
            return 0
        equivalents = read_equivalents(_read(".ctdd/stories/*/mutants_equivalents.toml"))
        forbidden = gen_guard([p.as_posix() for d in NO_GEN for p in Path(d).rglob("*.rs")], zones)
        outcomes_file = out / "mutants.out" / "outcomes.json"
        outcomes = read_outcomes(json.loads(outcomes_file.read_text())) if outcomes_file.exists() else []
    except (RedError, json.JSONDecodeError) as e:
        print(f"mutation: red: {e}", file=sys.stderr)
        return 1
    scored = scores(outcomes, equivalents, zones)
    text = report(scored)
    out.mkdir(parents=True, exist_ok=True)
    (out / "mutation.md").write_text(text)
    if summary := os.environ.get("GITHUB_STEP_SUMMARY"):
        with open(summary, "a") as f:
            f.write("### Mutation\n\n" + text)
    print(text, end="")
    for path in forbidden:
        print(f"mutation: red: {path} is in zone Gen (forbidden, MANQUE-ST01-1)", file=sys.stderr)
    reds = [s.file for s in scored if verdict(s) == "red"]
    for f in reds:
        print(f"mutation: red: {f} is below the threshold", file=sys.stderr)
    return 1 if forbidden or reds else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
