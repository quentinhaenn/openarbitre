# Contribuer à arbitre

Les règles ci-dessous viennent de ST-01 (`.ctdd/stories/ST-01/`). Les changer est un **amendement du socle**.

## Branches et PR

- `main` est protégée : toute modification passe par une PR, `.ctdd/` compris.
- Une PR est vérifiée par deux checks : `ci` (`scripts/ci.sh`) et `juge` (`scripts/lib/juge.py`).

## `Cargo.lock`

En cas de conflit sur `Cargo.lock`, ne jamais le fusionner à la main : reprendre celui de `main` (`git checkout origin/main -- Cargo.lock`), puis `cargo check --workspace` pour le régénérer.

## Dépendances

- Les dépendances communes et leur déclaration sont dans `docs/decisions/ST-01-dependances.md`. Un composant les reprend par `workspace = true`, sans répéter la version.
- Une nouvelle dépendance commune, ou la montée de version d'un outil (`scripts/outils.env`, `rust-toolchain.toml`), est un amendement du socle.

## Vérification : `scripts/ci.sh`

- Point d'entrée unique, en local comme en CI. Il exécute chaque `scripts/ci.d/*.sh`, triés dans l'ordre des octets (`LC_COLLATE=C`). Un contrôle en échec rend le verdict `rouge`.
- **Ajouter un contrôle** : un fichier `scripts/ci.d/NN-stxx-<sujet>.sh`, avec `NN ≥ 60`. Son code d'aide va dans `scripts/lib/`, jamais dans `ci.d/`. Les fichiers `*-st01-*` et `ci.sh` ne changent pas.
- Cet ajout passe **par une PR dédiée, avec le label `ci-change`** (écart accepté à AC-6.6, voir le juge).

## Le juge (`juge.yml`)

- Il s'exécute sur `pull_request_target`, depuis le code de `main`. Il lit la PR comme des données (`git diff`, `git show`) et n'en exécute rien. **`juge.yml` n'extrait jamais la tête de la PR** dans l'arbre de travail : sinon du code de PR s'exécuterait avec les droits de la base.
- Il rend `rouge` si la PR :
  - touche un chemin sous `.github/` ou `scripts/`, ajout, modification ou suppression ;
  - abaisse la zone effective d'un `.rs` existant (voir « Zones »).
- Avec le label `ci-change`, ces listes sont seulement affichées, mais la PR ne doit toucher aucun chemin sous `crates/`.
- **Le label `ci-change` est posé par Quentin seul, jamais par un agent.**

## Zones : `.ctdd/stories/ST-xx/zones.toml`

```toml
Full = ["crates/core/src/hlc.rs"]   # seuil de mutation 85 %
Std  = ["crates/core/src/clock.rs"] # seuil 70 %
Gen  = ["crates/cli/src/**"]        # score informatif, sans seuil
```

- Clés de premier niveau : `Full`, `Std`, `Gen`, chacune une liste de motifs. Une autre clé, un autre type ou un TOML illisible ⇒ `rouge`.
- Motifs relatifs à la racine, sémantique de `PurePosixPath.full_match` (Python ≥ 3.13) : `*` reste dans un segment, `**` couvre zéro ou plusieurs segments.
- Toutes les déclarations de toutes les stories sont lues. La zone d'un fichier est la plus exigeante qui le couvre (`Full > Std > Gen`) ; un fichier non couvert prend `Full`.
- Un fichier de `crates/core/` ou `crates/plan/` en `Gen` ⇒ `rouge`.

## Mutants équivalents : `.ctdd/stories/ST-xx/mutants_equivalents.toml`

```toml
[[equivalent]]
mutant = "crates/core/src/hlc.rs:42:9: replace < with <= in Horloge::emettre"
justification = "pourquoi aucun comportement observable ne change"
```

- `mutant` est le nom exact donné par cargo-mutants, copié depuis `mutants.out`. Si le mutant change de ligne, l'entrée ne correspond plus et la CI passe au rouge : on la met à jour.
- Justification vide ou absente, ou clé inconnue ⇒ `rouge`.
- **C'est le seul canal d'exclusion.** Sont interdits, et rendent `50-st01-mutation.sh` rouge : l'attribut `mutants::skip` dans `crates/**/*.rs`, tout fichier `mutants.toml` du dépôt (donc `exclude_re` et `exclude_globs`), et `--exclude` hors de la ligne de commande de `50-st01-mutation.sh`.

## Faux positifs de secrets : `.gitleaksignore`

Format natif de gitleaks, une empreinte par ligne, **chacune précédée d'une ligne `# justification`**. Une empreinte sans justification ⇒ `rouge`.

## Moteur de fusion (`arbitre-core`)

- `arbitre-core` est `#![no_std]` + `alloc` : ni heure système, ni E/S.
- Une écriture locale passe **uniquement** par `Journal::ecrire`, qui calcule `vu` et date l'écriture. `Journal::fusionner` est réservé aux lots reçus.
- **Contrat de désérialisation (ST-02 et suivantes)** : l'aller-retour d'une `Ecriture` est exact, `vu` compris. Sinon une copie réémise est une identité à deux contenus, donc une anomalie.
