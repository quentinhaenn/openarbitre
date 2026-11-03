"""Mesure de mutation (T-9) : zones, équivalents, scores, verdict, rapport.

Formats : `CONTRIBUTING.md` et archi ST-01 § Formats. Python >= 3.13, bibliothèque standard seule.
`juge.py` réutilise `lire_zones` et `zone_de` (A-8).
"""

import json
import os
import sys
import tomllib
from dataclasses import dataclass
from fractions import Fraction
from pathlib import Path, PurePosixPath

SEUILS = {"Full": 0.85, "Std": 0.70}
# Ordre d'exigence, du plus au moins exigeant (RG-ST01-7.2).
ZONES = ("Full", "Std", "Gen")
# MANQUE-ST01-1 : `Gen` interdit sous ces préfixes.
SANS_GEN = ("crates/core/", "crates/plan/")
# `summary` de cargo-mutants 27.1.0 (relevé réel, T-9) -> issue.
SUMMARY = {"CaughtMutant": "caught", "Timeout": "timeout", "MissedMutant": "missed", "Unviable": "unviable"}


class Rouge(Exception):
    """Entrée illisible ou interdite : le contrôle est rouge, le message nomme le fichier."""


@dataclass(frozen=True)
class Issue:
    """Résultat d'un mutant. `issue` ∈ {caught, timeout, missed, unviable}."""

    nom: str
    fichier: str
    issue: str


@dataclass(frozen=True)
class Score:
    fichier: str
    zone: str
    detectes: int
    comptes: int


def couvre(motif: str, chemin: str) -> bool:
    """`PurePosixPath(chemin).full_match(motif)` : `*` reste dans un segment, `**` couvre 0 à n segments."""
    return PurePosixPath(chemin).full_match(motif)


def _toml(chemin: str, texte: str) -> dict:
    try:
        return tomllib.loads(texte)
    except tomllib.TOMLDecodeError as e:
        raise Rouge(f"{chemin} : TOML illisible ({e})") from e


def lire_zones(contenus: dict[str, str]) -> dict[str, list[str]]:
    """Lit des `zones.toml` donnés par contenu ({chemin: texte}) ; rend {zone: motifs} fusionnés.

    TOML invalide, clé hors `ZONES` ou type inattendu ⇒ `Rouge` qui nomme le fichier (RG-ST01-7.2).
    """
    zones: dict[str, list[str]] = {z: [] for z in ZONES}
    for chemin, texte in sorted(contenus.items()):
        for cle, motifs in _toml(chemin, texte).items():
            if cle not in ZONES:
                raise Rouge(f"{chemin} : clé inconnue {cle!r} (attendu : {', '.join(ZONES)})")
            if not isinstance(motifs, list) or not all(isinstance(m, str) for m in motifs):
                raise Rouge(f"{chemin} : {cle} doit être une liste de motifs")
            zones[cle] += motifs
    return zones


def zone_de(chemin: str, zones: dict[str, list[str]]) -> str:
    """Zone la plus exigeante qui couvre `chemin`, `Full` par défaut (INV-ST01-15)."""
    for z in ZONES:
        if any(couvre(m, chemin) for m in zones.get(z, [])):
            return z
    return "Full"


def garde_gen(chemins: list[str], zones: dict[str, list[str]]) -> list[str]:
    """Fichiers de `SANS_GEN` résolus en `Gen` (MANQUE-ST01-1) ; liste vide = vert."""
    return [c for c in chemins if c.startswith(SANS_GEN) and zone_de(c, zones) == "Gen"]


def lire_equivalents(contenus: dict[str, str]) -> set[str]:
    """Noms exacts des mutants équivalents de tous les `mutants_equivalents.toml`.

    Justification vide ou absente, clé inconnue, TOML invalide ⇒ `Rouge` (US-7 cas 8).
    """
    noms: set[str] = set()
    for chemin, texte in sorted(contenus.items()):
        donnees = _toml(chemin, texte)
        if set(donnees) - {"equivalent"}:
            raise Rouge(f"{chemin} : clé inconnue {sorted(set(donnees) - {'equivalent'})}")
        for entree in donnees.get("equivalent", []):
            if set(entree) - {"mutant", "justification"}:
                raise Rouge(f"{chemin} : clé inconnue dans {entree}")
            mutant, justification = entree.get("mutant"), entree.get("justification")
            if not isinstance(mutant, str) or not mutant:
                raise Rouge(f"{chemin} : mutant absent")
            if not isinstance(justification, str) or not justification.strip():
                raise Rouge(f"{chemin} : justification vide ou absente pour {mutant!r}")
            noms.add(mutant)
    return noms


def lire_issues(outcomes: dict) -> list[Issue]:
    """Issues de `mutants.out/outcomes.json` (cargo-mutants 27.1.0). Base rouge ou format illisible ⇒ `Rouge`."""
    try:
        issues = []
        for o in outcomes["outcomes"]:
            scenario, summary = o["scenario"], o["summary"]
            if scenario == "Baseline":
                if summary != "Success":
                    raise Rouge(f"tests en échec sans mutation (base rouge) : {summary}")
                continue
            m = scenario["Mutant"]
            if summary not in SUMMARY:
                raise Rouge(f"issue inconnue {summary!r} pour {m['name']}")
            issues.append(Issue(m["name"], m["file"], SUMMARY[summary]))
        return issues
    except (KeyError, TypeError) as e:
        raise Rouge(f"outcomes.json illisible ({e!r})") from e


def scores(issues: list[Issue], equivalents: set[str], zones: dict[str, list[str]]) -> list[Score]:
    """Score par fichier : `caught`/`timeout` détectés, `unviable` et équivalents exclus. Trié par fichier."""
    par_fichier: dict[str, list[int]] = {}
    for i in issues:
        d, c = par_fichier.setdefault(i.fichier, [0, 0])
        if i.nom in equivalents or i.issue == "unviable":
            continue
        par_fichier[i.fichier] = [d + (i.issue in ("caught", "timeout")), c + 1]
    return [Score(f, zone_de(f, zones), d, c) for f, (d, c) in sorted(par_fichier.items())]


def verdict(s: Score) -> str:
    """`neutre` si rien n'est compté ; `info` en `Gen` ; sinon `vert` si score ≥ seuil, `rouge` sinon."""
    if s.comptes == 0:
        return "neutre"
    if s.zone == "Gen":
        return "info"
    return "vert" if s.detectes >= Fraction(str(SEUILS[s.zone])) * s.comptes else "rouge"


def rapport(liste: list[Score]) -> str:
    """Tableau Markdown (fichier, zone, score, seuil, verdict), trié par fichier, sans durée ni horodatage (C-7.4)."""
    lignes = ["| fichier | zone | score | seuil | verdict |", "|---|---|---|---|---|"]
    for s in sorted(liste, key=lambda s: s.fichier):
        score = f"{100 * s.detectes / s.comptes:.1f} % ({s.detectes}/{s.comptes})" if s.comptes else "—"
        seuil = f"{round(100 * SEUILS[s.zone])} %" if s.zone in SEUILS else "—"
        lignes.append(f"| {s.fichier} | {s.zone} | {score} | {seuil} | {verdict(s)} |")
    return "\n".join(lignes) + "\n"


def _lire(motif: str) -> dict[str, str]:
    return {p.as_posix(): p.read_text() for p in sorted(Path().glob(motif))}


def main(argv: list[str]) -> int:
    """Lit `mutants.out`, les zones et les équivalents du dépôt, écrit `$CI_OUT/mutation.md`. 0 = vert."""
    sortie = Path(os.environ.get("CI_OUT", "target/ci"))
    try:
        zones = lire_zones(_lire(".ctdd/stories/*/zones.toml"))
        equivalents = lire_equivalents(_lire(".ctdd/stories/*/mutants_equivalents.toml"))
        interdits = garde_gen([p.as_posix() for d in SANS_GEN for p in Path(d).rglob("*.rs")], zones)
        fichier = sortie / "mutants.out" / "outcomes.json"
        issues = lire_issues(json.loads(fichier.read_text())) if fichier.exists() else []
    except (Rouge, json.JSONDecodeError) as e:
        print(f"mutation : rouge : {e}", file=sys.stderr)
        return 1
    liste = scores(issues, equivalents, zones)
    texte = rapport(liste)
    sortie.mkdir(parents=True, exist_ok=True)
    (sortie / "mutation.md").write_text(texte)
    if resume := os.environ.get("GITHUB_STEP_SUMMARY"):
        with open(resume, "a") as f:
            f.write("### Mutation\n\n" + texte)
    print(texte, end="")
    for chemin in interdits:
        print(f"mutation : rouge : {chemin} est en zone Gen (interdit, MANQUE-ST01-1)", file=sys.stderr)
    rouges = [s.fichier for s in liste if verdict(s) == "rouge"]
    for f in rouges:
        print(f"mutation : rouge : {f} sous le seuil", file=sys.stderr)
    return 1 if interdits or rouges else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
