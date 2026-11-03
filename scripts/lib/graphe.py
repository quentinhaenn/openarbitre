"""Contrôle du graphe de dépendances (T-8) : règles (a) à (e) de l'archi ST-01, amendées par A-6 et A-9.

Python >= 3.13, bibliothèque standard seule. Chaque règle rend une liste de lignes d'échec (vide = vert).
"""

import sys

# A-9 : composants atteignables, transitivement, toutes sortes de dépendances.
AUTORISATIONS: dict[str, frozenset[str]] = {
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

# RG-ST01-6.2 : liste navigateur.
LISTE_NAVIGATEUR = ("arbitre-core", "arbitre-plan")

# Règle (b) : interdites dans le graphe normal de la liste navigateur ; `chrono` seulement avec `clock`.
INTERDITES = ("rusqlite", "libsqlite3-sys", "sqlite-wasm-rs", "tokio", "reqwest", "keyring")
INTERDITES_PAR_FEATURE = {"chrono": "clock"}

# Règle (d).
TLS_INTERDITS = ("openssl-sys", "native-tls")


def regle_aretes(metadata: dict) -> list[str]:
    """(a) A-9 : un composant atteint hors de sa table ⇒ ligne avec la chaîne ; crate du workspace absente de la table ⇒ ligne.

    `metadata` : JSON de `cargo metadata --format-version 1`.
    """
    raise NotImplementedError("T-8: regle_aretes — A-9, RG-ST01-6.3")


def regle_interdites(arbres: dict[str, str]) -> list[str]:
    """(b) Dépendances interdites dans le graphe normal d'une crate de la liste navigateur.

    `arbres` : {crate: sortie de `cargo tree -p <crate> -e normal,features --target wasm32-unknown-unknown -f '{p} {f}'`}.
    Le message nomme la crate et la dépendance (RG-ST01-5.1).
    """
    raise NotImplementedError("T-8: regle_interdites — RG-ST01-5.1, INV-ST01-9")


def regle_purete(sources: dict[str, dict[str, str]]) -> list[str]:
    """(c) A-6 : pour chaque crate présente de {arbitre-core, arbitre-plan}, `src/lib.rs` contient `#![no_std]`
    et aucune source ne contient `extern crate std`.

    `sources` : {crate: {chemin: contenu}}.
    """
    raise NotImplementedError("T-8: regle_purete — A-6, INV-ST01-9")


def regle_tls(metadata: dict) -> list[str]:
    """(d) Ni `openssl-sys` ni `native-tls` dans tout le workspace."""
    raise NotImplementedError("T-8: regle_tls — PL-6")


def regle_communes(racine: dict, manifestes: dict[str, dict]) -> list[str]:
    """(e) RG-ST01-5.2 : une dépendance du `[workspace.dependencies]` est reprise par `workspace = true`,
    et une dépendance utilisée par au moins deux composants est déclarée au workspace.

    `racine` : `Cargo.toml` racine lu par `tomllib` ; `manifestes` : {crate: son `Cargo.toml` lu par `tomllib`}.
    """
    raise NotImplementedError("T-8: regle_communes — RG-ST01-5.2, INV-ST01-10")


def main(argv: list[str]) -> int:
    """Lance `cargo metadata` et `cargo tree`, lit les manifestes et les sources, applique (a) à (e). 0 = vert."""
    raise NotImplementedError("T-8: main")


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
