"""Juge (T-11, A-8) : exécuté depuis `main` par `juge.yml` ; lit la PR comme données, n'en exécute rien.

Usage : `CI_LABELS=… python3 scripts/lib/juge.py <base> <tete>`. Python >= 3.13, bibliothèque standard seule.
"""

import os
import subprocess
import sys

from mutation import ZONES, Rouge, lire_zones, zone_de

PROTEGES = (".github/", "scripts/")
LABEL = "ci-change"


def juger(
    changes: list[str],
    zones_base: dict[str, str],
    zones_tete: dict[str, str],
    fichiers_rs: list[str],
    labels: set[str],
) -> tuple[bool, list[str]]:
    """Rend `(rouge, lignes)`.

    `changes` : `git diff --name-only --no-renames MB HEAD` ; `zones_*` : {chemin: texte} des `zones.toml` ;
    `fichiers_rs` : `*.rs` de `git ls-tree -r MB`.
    Règle 1 : chemin protégé touché. Règle 2 : zone effective d'un `.rs` de la base qui baisse ;
    `zones.toml` de la tête illisible ⇒ rouge. Sans label : liste non vide ⇒ rouge.
    Avec label : listes affichées, rouge seulement si un chemin sous `crates/` est touché.
    """
    lignes = [f"chemin protégé touché : {c}" for c in changes if c.startswith(PROTEGES)]
    try:
        base = lire_zones(zones_base)
        tete = lire_zones(zones_tete)
    except Rouge as e:
        return True, [f"zones illisibles : {e}"]
    for f in fichiers_rs:
        avant, apres = zone_de(f, base), zone_de(f, tete)
        if ZONES.index(apres) > ZONES.index(avant):
            lignes.append(f"zone abaissée : {f} ({avant} -> {apres})")
    if LABEL not in labels:
        return bool(lignes), lignes
    sous_crates = [c for c in changes if c.startswith("crates/")]
    return bool(sous_crates), lignes + [f"avec {LABEL}, la PR ne doit pas toucher crates/ : {c}" for c in sous_crates]


def git(*args: str) -> str:
    r = subprocess.run(("git", *args), capture_output=True, text=True, check=True)  # noqa: S603, S607 -- git fixe
    return r.stdout


def main(argv: list[str]) -> int:
    """Enveloppe `git` (`merge-base`, `diff`, `ls-tree`, `show`), lit `CI_LABELS`, appelle `juger`. 0 = vert."""
    base, tete = argv
    mb = git("merge-base", base, tete).strip()
    changes = git("diff", "--name-only", "--no-renames", "-z", mb, tete).split("\0")[:-1]
    arbre = git("ls-tree", "-r", "--name-only", "-z", mb).split("\0")[:-1]
    fichiers_rs = [f for f in arbre if f.endswith(".rs")]

    def zones(rev: str) -> dict[str, str]:
        chemins = git("ls-tree", "-r", "--name-only", "-z", rev).split("\0")[:-1]
        return {c: git("show", f"{rev}:{c}") for c in chemins if c.startswith(".ctdd/stories/") and c.endswith("/zones.toml")}

    labels = {x for x in os.environ.get("CI_LABELS", "").split(",") if x}
    rouge, lignes = juger(changes, zones(mb), zones(tete), fichiers_rs, labels)
    for ligne in lignes:
        print(f"juge : {ligne}", file=sys.stderr)
    print(f"juge : {'rouge' if rouge else 'vert'}")
    return 1 if rouge else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
