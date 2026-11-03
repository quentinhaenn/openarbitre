"""Contrôle du graphe de dépendances (T-8) : règles (a) à (e) de l'archi ST-01, amendées par A-6 et A-9.

Python >= 3.13, bibliothèque standard seule. Chaque règle rend une liste de lignes d'échec (vide = vert).
"""

import json
import re
import subprocess
import sys
import tomllib
from collections import deque
from pathlib import Path

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

SECTIONS = ("dependencies", "dev-dependencies", "build-dependencies")
# `nom vX.Y.Z (chemin) features` d'une ligne de `cargo tree -f '{p} {f}'`, préfixe d'arbre retiré.
LIGNE_ARBRE = re.compile(r"(\S+) v\S+(?: \([^)]*\))?\s*(.*)$")


def regle_aretes(metadata: dict) -> list[str]:
    """(a) A-9 : un composant atteint hors de sa table ⇒ ligne avec la chaîne ; crate du workspace absente de la table ⇒ ligne.

    `metadata` : JSON de `cargo metadata --format-version 1`.
    """
    noms = {p["id"]: p["name"] for p in metadata["packages"]}
    membres = {i: noms[i] for i in metadata["workspace_members"]}
    deps = {n["id"]: [d["pkg"] for d in n["deps"]] for n in metadata["resolve"]["nodes"]}
    lignes = [
        f"{nom} : crate du workspace absente de la table d'autorisations (amendement du socle requis)"
        for nom in sorted(membres.values())
        if nom not in AUTORISATIONS
    ]
    for depart, nom in sorted(membres.items(), key=lambda t: t[1]):
        if nom not in AUTORISATIONS:
            continue
        parent = {depart: None}
        file = deque([depart])
        while file:
            cur = file.popleft()
            for suivant in deps.get(cur, []):
                if suivant in parent:
                    continue
                parent[suivant] = cur
                file.append(suivant)
                if suivant in membres and membres[suivant] not in AUTORISATIONS[nom]:
                    chaine, i = [], suivant
                    while i is not None:
                        chaine.append(noms[i])
                        i = parent[i]
                    lignes.append(f"{nom} atteint {membres[suivant]}, interdit : {' -> '.join(reversed(chaine))}")
    return lignes


def regle_interdites(arbres: dict[str, str]) -> list[str]:
    """(b) Dépendances interdites dans le graphe normal d'une crate de la liste navigateur.

    `arbres` : {crate: sortie de `cargo tree -p <crate> -e normal,features --target wasm32-unknown-unknown -f '{p} {f}'`}.
    Le message nomme la crate et la dépendance (RG-ST01-5.1).
    """
    lignes = []
    for crate, arbre in sorted(arbres.items()):
        trouvees = set()
        for brut in arbre.splitlines():
            m = LIGNE_ARBRE.match(re.sub(r"^[\s│├└─]+", "", brut))
            if not m:
                continue
            nom, features = m.group(1), set(m.group(2).replace("(*)", "").strip().split(","))
            if nom in INTERDITES:
                trouvees.add(nom)
            elif nom in INTERDITES_PAR_FEATURE and INTERDITES_PAR_FEATURE[nom] in features:
                trouvees.add(f"{nom} (feature {INTERDITES_PAR_FEATURE[nom]})")
        lignes += [f"{crate} : dépendance interdite dans le graphe navigateur : {t}" for t in sorted(trouvees)]
    return lignes


def regle_purete(sources: dict[str, dict[str, str]]) -> list[str]:
    """(c) A-6 : pour chaque crate présente de {arbitre-core, arbitre-plan}, `src/lib.rs` contient `#![no_std]`
    et aucune source ne contient `extern crate std`.

    `sources` : {crate: {chemin: contenu}}.
    """
    lignes = []
    for crate in LISTE_NAVIGATEUR:
        fichiers = sources.get(crate)
        if fichiers is None:
            continue
        libs = [t for c, t in fichiers.items() if c.endswith("src/lib.rs")]
        if not any(re.search(r"^#!\[no_std\]", t, re.M) for t in libs):
            lignes.append(f"{crate} : src/lib.rs sans #![no_std]")
        for chemin, texte in sorted(fichiers.items()):
            if re.search(r"^\s*extern\s+crate\s+std\b", texte, re.M):
                lignes.append(f"{crate} : {chemin} contient extern crate std")
    return lignes


def regle_tls(metadata: dict) -> list[str]:
    """(d) Ni `openssl-sys` ni `native-tls` dans tout le workspace."""
    return [
        f"workspace : {nom} présent (rustls seul, PL-6)"
        for nom in sorted({p["name"] for p in metadata["packages"]})
        if nom in TLS_INTERDITS
    ]


def regle_communes(racine: dict, manifestes: dict[str, dict]) -> list[str]:
    """(e) RG-ST01-5.2 : une dépendance du `[workspace.dependencies]` est reprise par `workspace = true`,
    et une dépendance utilisée par au moins deux composants est déclarée au workspace.

    `racine` : `Cargo.toml` racine lu par `tomllib` ; `manifestes` : {crate: son `Cargo.toml` lu par `tomllib`}.
    """
    communes = set(racine.get("workspace", {}).get("dependencies", {}))
    lignes, usage = [], {}
    for crate, manifeste in sorted(manifestes.items()):
        tables = [manifeste.get(k, {}) for k in SECTIONS]
        for cible in manifeste.get("target", {}).values():
            tables += [cible.get(k, {}) for k in SECTIONS]
        for table in tables:
            for dep, spec in table.items():
                spec = spec if isinstance(spec, dict) else {}
                usage.setdefault(dep, set()).add(crate)
                if dep in communes and spec.get("workspace") is not True:
                    lignes.append(f"{crate} : {dep} est au workspace, à reprendre par workspace = true")
    for dep, crates in sorted(usage.items()):
        if len(crates) >= 2 and dep not in communes and not dep.startswith("arbitre"):
            lignes.append(f"{dep} utilisée par {', '.join(sorted(crates))} : à déclarer dans [workspace.dependencies]")
    return lignes


def sortie(*cmd: str) -> str:
    r = subprocess.run(cmd, capture_output=True, text=True)  # noqa: S603, S607 -- commandes cargo fixes
    if r.returncode:
        raise SystemExit(f"graphe : {' '.join(cmd)} a échoué : {r.stderr.strip()}")
    return r.stdout


def main(argv: list[str]) -> int:
    """Lance `cargo metadata` et `cargo tree`, lit les manifestes et les sources, applique (a) à (e). 0 = vert."""
    metadata = json.loads(sortie("cargo", "metadata", "--format-version", "1", "--locked"))
    racine_ws = Path(metadata["workspace_root"])
    membres = {
        p["name"]: Path(p["manifest_path"]) for p in metadata["packages"] if p["id"] in metadata["workspace_members"]
    }
    presents = [c for c in LISTE_NAVIGATEUR if c in membres]
    arbres = {
        c: sortie("cargo", "tree", "--locked", "-p", c, "-e", "normal,features",
                  "--target", "wasm32-unknown-unknown", "-f", "{p} {f}")
        for c in presents
    }
    sources = {
        c: {str(f.relative_to(racine_ws)): f.read_text() for f in sorted(membres[c].parent.rglob("src/**/*.rs"))}
        for c in presents
    }
    manifestes = {c: tomllib.loads(m.read_text()) for c, m in membres.items()}
    racine = tomllib.loads((racine_ws / "Cargo.toml").read_text())
    lignes = (
        regle_aretes(metadata) + regle_interdites(arbres) + regle_purete(sources)
        + regle_tls(metadata) + regle_communes(racine, manifestes)
    )
    for ligne in lignes:
        print(f"graphe : {ligne}", file=sys.stderr)
    return 1 if lignes else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
