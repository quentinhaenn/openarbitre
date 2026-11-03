# Démonstrations rouges — ST-01, T-12 (voie outillage)

Branches locales jetables, jamais poussées, supprimées. Base du diff : le commit d'outillage `eedd421`+ (HEAD de `st-01/outillage` avant les démonstrations). `scripts/ci.sh` est lancé avec `CI_BASE=<base>` ; les lignes ci-dessous sont extraites de sa sortie (le moteur n'est pas fini : `10-st01-tests` est rouge partout, par les `todo!` de `tests/support/mod.rs`, sans rapport avec la démonstration).

| # | Modification | Résultat | Message |
|---|---|---|---|
| 0 | `#[test] fn rouge_demo() { assert!(false) }` dans `crates/core/src/lib.rs` | rouge, `10` | `test zz::rouge_demo ... FAILED` ; `ci : rouge : 10-st01-tests : code 1` |
| 1 | `rusqlite` ajouté au cœur | rouge, `30` (et `20` : `sqlite-wasm-rs` ne compile pas en wasm) | `graphe : arbitre-core : dépendance interdite dans le graphe navigateur : rusqlite` (et `sqlite-wasm-rs`) |
| 2 | `chrono` avec `clock` ajouté au cœur | rouge, `30` | `arbitre-core : dépendance interdite … : chrono (feature clock)` |
| 3a | crates factices `arbitre-plan` → `arbitre-diff` | rouge, `30` | `arbitre-plan atteint arbitre-diff, interdit : arbitre-plan -> arbitre-diff` |
| 3b | `arbitre-plan` → `arbitre-sqlite` → `arbitre-diff` | rouge, `30` | `arbitre-plan atteint arbitre-diff, interdit : arbitre-plan -> arbitre-sqlite -> arbitre-diff` (+ 2 autres arêtes) |
| 4 | `reqwest` ajouté au cœur | rouge, `30` | `arbitre-core : dépendance interdite … : reqwest` |
| 5 | faux jeton `ghp_…` commité, retiré au commit suivant | rouge, `40` | `RuleID: github-pat`, `File: demo-jeton.txt`, `Commit: 941e9e5…`, `leaks found: 1` (local seulement) |
| 6 | **non obtenue sur `hlc.rs`** (moteur à `todo!`, base rouge) | voir ci-dessous | |
| 7 | seuil `Full` modifié dans `mutation.py` | rouge | `juge : chemin protégé touché : scripts/lib/mutation.py` |
| 8 | `merge.rs` de `Full` à `Std` dans `ST-01/zones.toml` | rouge | `juge : zone abaissée : crates/core/src/merge.rs (Full -> Std)` |
| 9 | `ST-05/zones.toml` : `Std = ["crates/core/src/**"]` | rouge | `juge : zone abaissée : crates/core/src/lib.rs (Full -> Std)` |
| 10 | `continue-on-error: true` dans `ci.yml` | rouge | `juge : chemin protégé touché : .github/workflows/ci.yml` |
| 11 | `scripts/ci.sh` terminé par `exit 0` | rouge | `juge : chemin protégé touché : scripts/ci.sh` |
| 12 | ajout de `scripts/ci.d/45-st05-prep.sh` | rouge | `juge : chemin protégé touché : scripts/ci.d/45-st05-prep.sh` |
| 13a | 7 avec `CI_LABELS=ci-change` | vert (code 0), liste affichée | `juge : chemin protégé touché : scripts/lib/mutation.py` puis `juge : vert` |
| 13b | 7 + une ligne dans `crates/core/src/hlc.rs`, avec le label | rouge | `juge : avec ci-change, la PR ne doit pas toucher crates/ : crates/core/src/hlc.rs` |

Compléments de T-10 (critères US-6) :

| Cas | Résultat |
|---|---|
| PR documentaire (`docs/demo-note.md`) | `50-st01-mutation` vert, mutation neutre : cargo-mutants rend « Diff changes no Rust source files », code 0, aucun `mutants.out`, rapport vide. `10` reste rouge (moteur) |
| `scripts/ci.d/60-demo.sh` avec `exit 1` | exécuté après les autres, `ci : rouge : 60-demo : code 1` |
| cargo-mutants hors du `PATH` | `ci : rouge : cargo-mutants absent du PATH. Installer : cargo install --locked cargo-mutants@27.1.0` (gitleaks et Python 3.9 d'Xcode nommés de même) |

## Démonstration 6 : non obtenue, substitut

Faute de moteur, le score de `hlc.rs` n'est pas mesurable. Substitut sur les **vraies sorties** de cargo-mutants 27.1.0 d'un crate jetable (hors dépôt), `src/lib.rs` déclaré `Full` : `mutation.py` rend

```
| src/lib.rs | Full | 63.6 % (7/11) | 85 % | rouge |   (code 1)
mutation : rouge : src/lib.rs sous le seuil
```

et, sur un `outcomes.json` réel dont la base est rouge, `mutation : rouge : tests en échec sans mutation (base rouge) : Failure` (code 1). À refaire sur `hlc.rs` quand le moteur sera complet.

## Constat hors démonstration

Un cache `.pyc` périmé (modification de même taille dans la même seconde) a masqué un instant un changement de `mutation.py` ; `ci.sh` exporte maintenant `PYTHONDONTWRITEBYTECODE=1`.
