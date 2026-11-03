"""Juge (T-11, A-8) : exécuté depuis `main` par `juge.yml` ; lit la PR comme données, n'en exécute rien.

Usage : `CI_LABELS=… python3 scripts/lib/juge.py <base> <tete>`. Python >= 3.13, bibliothèque standard seule.
"""

import sys

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
    raise NotImplementedError("T-11: juger — A-8, D-1, D-2")


def main(argv: list[str]) -> int:
    """Enveloppe `git` (`merge-base`, `diff`, `ls-tree`, `show`), lit `CI_LABELS`, appelle `juger`. 0 = vert."""
    raise NotImplementedError("T-11: main — A-8")


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
