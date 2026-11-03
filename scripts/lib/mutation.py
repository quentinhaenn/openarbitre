"""Mesure de mutation (T-9) : zones, équivalents, scores, verdict, rapport.

Formats : `CONTRIBUTING.md` et archi ST-01 § Formats. Python >= 3.13, bibliothèque standard seule.
`juge.py` réutilise `lire_zones` et `zone_de` (A-8).
"""

import sys
from dataclasses import dataclass

SEUILS = {"Full": 0.85, "Std": 0.70}
# Ordre d'exigence, du plus au moins exigeant (RG-ST01-7.2).
ZONES = ("Full", "Std", "Gen")
# MANQUE-ST01-1 : `Gen` interdit sous ces préfixes.
SANS_GEN = ("crates/core/", "crates/plan/")


class Rouge(Exception):
    """Entrée illisible ou interdite : le contrôle est rouge, le message nomme le fichier."""


@dataclass(frozen=True)
class Issue:
    """Résultat d'un mutant. `issue` ∈ {caught, timeout, missed, unviable} (format relevé en T-9, R-3)."""

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
    raise NotImplementedError("T-9: couvre — Q-2")


def lire_zones(contenus: dict[str, str]) -> dict[str, list[str]]:
    """Lit des `zones.toml` donnés par contenu ({chemin: texte}) ; rend {zone: motifs} fusionnés.

    TOML invalide, clé hors `ZONES` ou type inattendu ⇒ `Rouge` qui nomme le fichier (RG-ST01-7.2).
    """
    raise NotImplementedError("T-9: lire_zones — RG-ST01-7.2, AMB-ST01-4")


def zone_de(chemin: str, zones: dict[str, list[str]]) -> str:
    """Zone la plus exigeante qui couvre `chemin`, `Full` par défaut (INV-ST01-15)."""
    raise NotImplementedError("T-9: zone_de — INV-ST01-15")


def garde_gen(chemins: list[str], zones: dict[str, list[str]]) -> list[str]:
    """Fichiers de `SANS_GEN` résolus en `Gen` (MANQUE-ST01-1) ; liste vide = vert."""
    raise NotImplementedError("T-9: garde_gen — MANQUE-ST01-1")


def lire_equivalents(contenus: dict[str, str]) -> set[str]:
    """Noms exacts des mutants équivalents de tous les `mutants_equivalents.toml`.

    Justification vide ou absente, clé inconnue, TOML invalide ⇒ `Rouge` (US-7 cas 8).
    """
    raise NotImplementedError("T-9: lire_equivalents — RG-ST01-7.3, MANQUE-ST01-5")


def lire_issues(outcomes: dict) -> list[Issue]:
    """Issues de `mutants.out/outcomes.json` (cargo-mutants 27.1.0). Base rouge ou format illisible ⇒ `Rouge`."""
    raise NotImplementedError("T-9: lire_issues — R-3, MANQUE-ST01-8")


def scores(issues: list[Issue], equivalents: set[str], zones: dict[str, list[str]]) -> list[Score]:
    """Score par fichier : `caught`/`timeout` détectés, `unviable` et équivalents exclus. Trié par fichier."""
    raise NotImplementedError("T-9: scores — RG-ST01-7.3")


def verdict(s: Score) -> str:
    """`neutre` si rien n'est compté ; `info` en `Gen` ; sinon `vert` si score ≥ seuil, `rouge` sinon."""
    raise NotImplementedError("T-9: verdict — RG-ST01-7.3, MANQUE-ST01-1")


def rapport(liste: list[Score]) -> str:
    """Tableau Markdown (fichier, zone, score, seuil, verdict), trié par fichier, sans durée ni horodatage (C-7.4)."""
    raise NotImplementedError("T-9: rapport — C-7.4")


def main(argv: list[str]) -> int:
    """Lit `mutants.out`, les zones et les équivalents du dépôt, écrit `$CI_OUT/mutation.md`. 0 = vert."""
    raise NotImplementedError("T-9: main")


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
