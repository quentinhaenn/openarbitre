# Skeleton — ST-01 : fusion pure et fondations du dépôt

- Date : 2026-11-03 · Branche : `st-01/fondations` (aucun commit) · Plan : `04_plan.md` v3, amendements A-1 à A-12.
- T-0 non touchée (gestes de l'orchestrateur). T-12 à T-14 ne produisent pas de fichier à ce stade.

## Fichiers créés ou modifiés

| Fichier | Action | Tâche(s) | État |
|---|---|---|---|
| `rust-toolchain.toml` | Créé | T-1 | Complet |
| `Cargo.toml` | Modifié : `rust-version`, `[workspace.dependencies]` selon A-5 | T-1 | Complet |
| `crates/core/Cargo.toml` | Modifié : plus de `[dependencies]`, `proptest` en dev, feature `sim`, `[[test]] required-features` | T-1, T-3, T-7 | Complet |
| `crates/cli/Cargo.toml`, `crates/cli/src/main.rs` | Modifiés : `chrono` retiré, parseur clap (`--version`, `--help`) | T-1 | Complet |
| `Cargo.lock` | Régénéré (`chrono`, `rusqlite` sortis, `proptest` entré) | T-1 | Complet |
| `crates/core/src/lib.rs` | Réécrit : `#![no_std]`, `extern crate alloc`, `mod` + réexports | T-1, T-5 | Complet |
| `crates/core/src/hlc.rs` | Créé : `IdAppareil`, `Marque`, `L_MAX`, `Horloge`, `HorlogeEpuisee`, `Seuil`, `SEUIL_DEFAUT`, `Avance`, `avances` | T-3 | Squelette (`todo!`) |
| `crates/core/src/clock.rs` | Créé : `trait Clock`, `HorlogeSimulee` (feature `sim`) | T-3 | Squelette |
| `crates/core/src/write.rs` | Créé : `IdDonnee`, `Champ`, `Valeur`, `Auteur`, `Quoi`, `Visee`, `Cible`, `Vu`, `Ecriture`, `Raison`, `ErreurLot`, `valider` | T-4 | Squelette |
| `crates/core/src/merge.rs` | Créé : `Contenu`, `ErreurEcriture`, `Statut`, `EtatDonnee`, `Projection`, `Conflit`, `Anomalie`, `Rapport`, `Journal` (+ `vu_pour` privée) | T-5, T-6 | Squelette |
| `crates/core/tests/support/mod.rs` | Créé : `config()` (1 000 jeux, `RngSeed::Fixed(GRAINE)`), constructeurs ; appareils simulés, `Scenario`, `Famille`, oracle `attendus` | T-3 (config), T-7 | Config et constructeurs complets ; simulateur et oracle en `todo!` |
| `crates/core/tests/hlc.rs` | Créé : US-4 cas 1 à 10, bornes, `HorlogeSimulee`, propriétés INV-ST01-4, 5, 6, 8 | T-3 | 20 tests |
| `crates/core/tests/write.rs` | Créé : US-4 cas 11, `VuNonAnterieur`, `IdentifiantVide`, `MarqueHorsBornes`, `index` | T-4 | 12 tests |
| `crates/core/tests/merge.rs` | Créé : US-1 cas 1-3, 7, 8, 10, 11 ; US-3 cas 1-9 ; US-2 cas 1-4, 6-10 ; F-1, F-12, collision, 10 000 en un lot ; propriétés INV-7, INV-ST01-7, INV-ST01-3 | T-5, T-6 | 44 tests |
| `crates/core/tests/convergence.rs` | Créé : concurrentes ⇔ conflit (oracle, D-10), convergence, INV-5, permutation, couverture R-7 | T-7 | 5 tests |
| `docs/decisions/ST-01-dependances.md` | Créé | T-2 | Complet |
| `CONTRIBUTING.md` | Créé | T-2 | Complet |
| `scripts/lib/graphe.py` | Créé : tables A-9, interdites, règles (a) à (e), `main` | T-8 | Squelette (`NotImplementedError`) |
| `scripts/lib/mutation.py` | Créé : `SEUILS`, `Issue`, `Score`, `Rouge`, motifs, zones, équivalents, issues, scores, verdict, rapport | T-9 | Squelette |
| `scripts/lib/juge.py` | Créé : `juger`, `main` | T-11 | Squelette |
| `scripts/lib/test_ci.py` | Créé : 18 tests graphe, 21 mutation, 9 juge | T-8, T-9, T-11 | 48 tests |
| `scripts/outils.env`, `.gitleaksignore` (vide) | Créés | T-10 | Complets |
| `scripts/ci.sh`, `scripts/ci.d/{10,20,30,40,50}-st01-*.sh` | Créés : en-tête et contrat, corps `exit 1` « non implémenté (T-10) » | T-10 | Squelette |
| `.github/workflows/ci.yml`, `.github/workflows/juge.yml` | Créés : déclencheurs, permissions, étapes ; SHA en `SHA_A_EPINGLER_T11` | T-11 | Squelette |

## État

- `cargo build --workspace --locked` : vert, sans avertissement.
- `cargo test --workspace --no-run` et `--all-features --no-run` : verts, sans avertissement.
- `cargo check -p arbitre-core --target wasm32-unknown-unknown` (avec et sans `--features sim`) : vert. `cargo tree -p arbitre-core -e normal` n'affiche que `arbitre-core`. `grep -rn 'chrono\|rusqlite' crates/` est vide. `arbitre --version` rend `arbitre 0.1.0`.
- Tests en échec attendus : **81 Rust** (hlc 20, write 12, merge 44, convergence 5) et **48 Python**. Tous échouent sur un `todo!("T-x: …")` ou un `NotImplementedError("T-x: …")`, aucun sur une autre cause (relevé sur `cargo test --all-features --no-fail-fast` et `python3 -m unittest`).
- Les builds ont été lancés avec `RUSTUP_TOOLCHAIN=stable` (= 1.99.1) : voir « Outils système manquants ».

## Vérification ponctuelle de T-1 (A-5)

Crate jetable `crates/zz-verif-deps` héritant des huit dépendances (`serde` + `std`), `cargo check -p zz-verif-deps` :
- **premier essai rouge** : `keyring` 4.2.0 avec `default-features = false` seul ⇒ `compile_error!("At least one of the features 'v1' or 'cli' must be enabled")` ;
- avec `features = ["v1"]` côté consommateur : **vert** (aws-lc-sys compilé sans souci, R-9 ne s'est pas produit). Arbre direct : `getrandom v0.4.3`, `jiff v0.2.39`, `keyring v4.2.0 (v1)`, `proptest v1.12.0`, `reqwest v0.13.5 (__rustls, __rustls-aws-lc-rs, rustls)`, `serde v1.0.229 (derive, std)`, `serde_json v1.0.154 (std)`, `tokio v1.54.1`.
- Crate supprimée, `Cargo.lock` restauré. Le point `keyring` est consigné dans la décision de dépendances.

## Outils système manquants (non installés)

- **Toolchain `1.99.1` nommée, `clippy`, `rustfmt`** : seul `stable` (= 1.99.1, cible wasm32 comprise) est installé, sans clippy ni rustfmt. Sans `RUSTUP_TOOLCHAIN=stable`, rustup installerait automatiquement la toolchain de `rust-toolchain.toml` au premier `cargo`. À accepter par Quentin (ou lancer `rustup toolchain install 1.99.1 -c clippy rustfmt -t wasm32-unknown-unknown`).
- **cargo-mutants 27.1.0** : non installé (T-0). Bloque le relevé de format de T-9 (R-3) et `50-st01-mutation.sh`.

## Écarts au plan et points à arbitrer

1. **`keyring`** : la déclaration d'A-5 est gardée telle quelle ; la crate consommatrice devra ajouter `features = ["v1"]`. Recommandation : garder (le backend est choisi par la crate qui l'utilise, comme le prévoit l'archi).
2. **Ajouts d'API nécessaires aux tests**, non nommés par le plan : `Journal::ecritures()` (rend `S`, pour INV-7 et l'oracle) ; `Statut::Supprimee(Ecriture)` porte la suppression retenue (sinon la partition n'est pas vérifiable) ; `Journal`, `Horloge` en `Clone + PartialEq` (contrôle « inchangé ») ; `HorlogeSimulee::nouvelle(t)` ; énumération des raisons nommée `Raison` ; `Anomalie { cible, marque, contenus }`.
3. **Sémantique de `HorlogeSimulee::figer`** (non fixée par les specs) : `now()` rend la dernière heure posée ; après `figer()`, `avancer`/`reculer` sont sans effet jusqu'au prochain `fixer`. À confirmer.
4. **Placement des tests** : T-4 à T-6 en tests d'intégration (`tests/{write,merge}.rs`), le plan ne listant que les sources. `hlc` et `convergence` exigent la feature `sim` (`required-features`) : un `cargo test` sans `--all-features` les saute ; la CI utilise `--all-features`.
5. **Graines d'échec** : proptest 1.12 écrit `tests/<cible>.proptest-regressions` à côté du fichier, pas dans un dossier `proptest-regressions/`. Un fichier créé par les échecs `todo!` a été supprimé ; l'implémenteur doit faire de même avant de versionner.
6. **R-8 levé** : `RngSeed::Fixed` existe dans proptest 1.12.0, utilisé dans `config()`.
7. **US-3 cas 5** : le test attend un conflit (règle RG-ST01-2.1 : concurrentes, exclusives, valeurs différentes), la colonne de la matrice étant muette.
8. **Données figées** : les tests de `graphe.py` et `juge.py` construisent leurs entrées en mémoire, sans `scripts/lib/fixtures/`. Les données de `mutation.py` issues de cargo-mutants (format d'`outcomes.json`) restent à relever en T-9. US-7 cas 9 relève de `--in-diff` : couvert par une démonstration, pas par `unittest`.
9. **`juge.yml`** : étape `setup-python` 3.13 ajoutée (le Python d'`ubuntu-latest` est antérieur à 3.13, or `full_match` l'exige), non listée par A-8. SHA des Actions à épingler en T-11 (aucune API GitHub appelée ici).

## Arbitrages de Quentin (2026-11-03)

- Toolchain `1.99.1` installée (clippy, rustfmt, wasm32).
- Validés en bloc : (1) déclaration `keyring` d'A-5 gardée, la crate utilisatrice ajoute `v1` ; (2) ajouts d'API `Journal::ecritures()`, `Statut::Supprimee(Ecriture)`, `Clone + PartialEq` sur `Journal`/`Horloge`, `HorlogeSimulee::nouvelle` ; (3) `figer` : `avancer`/`reculer` sans effet jusqu'au prochain `fixer` ; (4) US-3 cas 5 : conflit attendu (RG-ST01-2.1).
