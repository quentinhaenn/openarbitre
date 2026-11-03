# Plan d'implémentation — ST-01 : Fusion pure et fondations du dépôt

## Traçabilité
- Specs : `.ctdd/stories/ST-01/02_specs.md`. Ses « Arbitrages de Quentin (2026-11-03) » font foi : `vu` par écriture, `ci.d/`, `Gen` informatif, reste validé en bloc.
- Archi : `.ctdd/stories/ST-01/03_archi.md`. Ses « Arbitrages de Quentin (2026-11-03) » font foi : P-1 `jiff` et `getrandom` communs, P-2 `Clock::fuseau` en ST-04, P-3 E-1 à E-3 acceptés et **Python ≥ 3.11**.
- Cadre : `.ctdd/cadre/04_plan.md` § ST-01 (livrable, critère de complétion), § Collisions.
- Livré avec ce plan : `.ctdd/stories/ST-01/zones.toml`.
- Date : 2026-11-03 · Statut : DRAFT.

## Contexte technique
- **Codebase** :
  - `crates/core` (`arbitre-core`) contient `add` et dépend de `chrono/clock` et de `rusqlite` ;
  - `crates/cli` (`arbitre`) affiche « Hello, world! » et dépend de `chrono` et de `clap` ;
  - `Cargo.lock` est versionné ;
  - le remote `origin` est `github.com/quentinhaenn/openarbitre`, **public** ;
  - il n'y a ni CI, ni `scripts/`, ni `docs/`.
- **Stack** :
  - Rust 1.99.1, édition 2024, cible `wasm32-unknown-unknown` installée ;
  - `proptest` 1.12.0 ;
  - Python ≥ 3.11 (bibliothèque standard seule) pour les aides de CI ;
  - bash ;
  - gitleaks 8.30.1 (installé) ;
  - cargo-mutants 27.1.0 (**non installé**) ;
  - GitHub Actions.
- **Patterns à respecter** (archi ST-01) :
  - cœur fonctionnel : l'heure entre en valeur `u64` ;
  - aucun `HashMap` dans le cœur ;
  - l'ordre de déclaration des champs de `Marque`, `IdDonnee`, `Cible` et `Conflit` fait partie du contrat, parce que les `Ord` dérivés en découlent ;
  - les états impossibles ne sont pas représentables ;
  - le cœur n'a aucune dépendance normale ;
  - `ci.sh` est un exécuteur gelé ; les contrôles vivent dans `scripts/ci.d/`.

## Analyse pré-plan

### Over-engineering détecté
- **Simulateur réseau de PROP-ST01-2.** On ne crée pas d'objet réseau. Le scénario est une suite d'actions générée par proptest : `émettre(i, quoi)`, `livrer(i → j, sous-ensemble)`, `relivrer`. Les appareils sont des couples `(Horloge, Journal)`. Livrer un sous-ensemble dans un ordre quelconque couvre à lui seul le retard, la duplication et le réordonnancement. *Trade-off* : aucun.
- **Matrice US-1, cas 9 (fuseau, locale, heure machine).** On ne lance pas de test multi-processus avec `TZ` et `LANG` variés. Le cœur n'a aucune dépendance et ne lit aucun environnement. Le contrôle de pureté de `graphe.py` (`SystemTime`, `::now(`) et la propriété de déterminisme dans le processus (mêmes entrées, permutées ⇒ sorties identiques) le prouvent par construction. *Trade-off* : un `std::env::var` ajouté au cœur ne serait pas détecté. Il n'existe aucune raison d'en ajouter un, et la relecture le voit.
- **Réduction des jeux en profil de mutation.** On ne la construit pas d'avance. On mesure d'abord, en T-13.

### Under-engineering détecté
- **Python ≥ 3.11 (arbitrage P-3) contre `PurePosixPath.full_match` (3.13), retenu par l'archi pour les motifs de `zones.toml`.** `full_match` n'existe pas en 3.11. Correction : `mutation.py` traduit lui-même le motif en expression régulière (`**/` → `(?:.*/)?`, `**` → `.*`, `*` → `[^/]*`, `?` → `[^/]`, le reste échappé, ancré aux deux bouts). La sémantique documentée reste celle de l'archi, et cinq exemples de `test_ci.py` la fixent. La CI garde `setup-python` 3.13, qui satisfait le minimum.
- **`github.event.before` (archi, CI sur `push` vers `main`).** Au premier envoi d'une branche, il vaut `0000…`, et la mutation ou gitleaks sur `0000..HEAD` casserait le contrôle. Correction : le workflow ne se déclenche **que sur `pull_request`**. C'est suffisant, puisque toute fusion passe par une PR rebasée, une seule à la fois (protocole d'intégration du cadre). Si Quentin veut aussi `push`, il faut traiter la base nulle comme « neutre ».
- **Démonstration du contrôle des secrets (matrice US-6, cas 8).** Envoyer un faux jeton sur le dépôt public est exclu. La démonstration se fait **en local seulement**, sur une branche jetable jamais envoyée, et la protection contre l'envoi (G-1) la bloquerait de toute façon.
- **Démonstration d'arête interdite (AC-6.4).** `arbitre-plan` et `arbitre-diff` n'existent pas. Il faut deux crates factices sur une branche jetable (T-12), en plus des tests sur données figées.
- **Format réel de cargo-mutants 27.** L'archi suppose `mutants.out/outcomes.json`, les issues `caught`/`missed`/`timeout`/`unviable` et le nom « `fichier:ligne:col: description` ». Rien de cela n'a été vérifié sur la version 27.1.0. T-9b commence par une vraie exécution sur un mini-crate, et ses données figées en sont tirées.
- **Calcul des conflits.** Une évaluation naïve sur toutes les paires de `S` à chaque fusion est quadratique, et le cas 8 de la matrice US-1 (10 000 écritures) la rendrait lente en debug. `fusionner` n'évalue que les paires (écriture nouvelle × écritures de sa portée). `Journal::conflits()`, qui recalcule `R(S)` en entier, n'est appelé que par les propriétés, sur de petits jeux.

### Risques identifiés

| # | Risque | Probabilité | Impact | Mitigation |
|---|--------|-------------|--------|------------|
| R-1 | Score < 85 % sur `merge.rs` ou `hlc.rs`, à cause de mutants équivalents (comparaisons dans les tris, `max` sur des égaux impossibles) | Moyenne | K-7.5 non tenu, story bloquée (arbitrage « bloquant ») | Tests ciblés sur les bornes (`=` contre `<`, `σ` exact, compteur saturé). Les équivalents restants vont dans `mutants_equivalents.toml`, chacun justifié et relu par Quentin en T-13 |
| R-2 | La mutation est longue en CI : chaque mutant relance des propriétés à 1 000 jeux | Moyenne | PR lente, délais d'attente | On mesure en T-13. Au-delà de ~20 min, on ajoute un profil de mutation qui réduit les jeux (variable lue par les tests), sans toucher l'exécution courante (E-1.1) |
| R-3 | Le format de sortie de cargo-mutants 27 diffère de l'hypothèse de l'archi | Moyenne | `mutation.py` faux | T-9b part d'une sortie réelle. Une sortie illisible ⇒ `rouge` (échec fermé) |
| R-4 | La sortie de `cargo tree -e features` est fragile à analyser | Moyenne | Faux vert sur `chrono/clock` | Format imposé `-f '{p} {f}'`. Test en T-12 : réintroduire `chrono/clock` dans le cœur ⇒ `rouge` |
| R-5 | G-1 (protection contre l'envoi de secrets) est encore inactive au premier envoi | Basse | Risque RG-G9 | T-0 la vérifie avant tout `git push` |
| R-6 | `cargo install cargo-mutants` est lent en CI | Haute | +3 à 5 min par exécution | Binaire précompilé à la version de `outils.env` (`taiki-e/install-action`, épinglée par SHA) et cache Cargo |
| R-7 | Les générateurs ne produisent pas assez de cas durs (même `(l, c)`, concurrentes de même valeur) | Moyenne | Propriétés vertes sans rien prouver | Générateurs biaisés : petit domaine pour `l`, `c` et les valeurs, au moins 3 appareils. Une assertion de couverture (`prop_assume` exclu) vérifie qu'un échantillon contient chaque famille d'E-1.1 |

## Tâches

Convention C-TDD pour les tâches Core / Full :
- le Skeleton pose d'abord les types et les signatures de l'archi, puis les tests issus des matrices et des invariants, en échec ;
- la tâche les fait passer.

Chaque tâche cite les cas de matrice qu'elle doit rendre verts.

### T-0 : Gestes préalables (hors code)
- **Zone DDD** : Generic
- **Réf specs** : C-6.6, MANQUE-ST01-6 ; cadre G-1
- **Dépendances** : aucune
- **Description** : Quentin confirme que la protection contre l'envoi de secrets est active sur `quentinhaenn/openarbitre` (Settings → Code security). Installer en local `cargo install --locked cargo-mutants@27.1.0`.
- **Livrable** : aucun fichier.
- **Critère de complétion** : `cargo mutants --version` affiche `27.1.0` ; Quentin a confirmé G-1.
- **Effort estimé** : agent 5 min (installation, en fond) · arbitrage 10 min

### T-1 : Assainir le workspace et déclarer les dépendances communes
- **Zone DDD** : Generic
- **Réf specs** : AC-5.1, AC-5.3, AC-5.4, AC-8.3 ; RG-ST01-5.1, 5.2, 8.2 ; C-5.3
- **Dépendances** : aucune
- **Description** :
  - `rust-toolchain.toml` : `channel = "1.99.1"`, composants `clippy`, `rustfmt`, cible `wasm32-unknown-unknown` ;
  - `Cargo.toml` racine : `rust-version = "1.99"` dans `workspace.package` ; `[workspace.dependencies]` avec les huit communes de l'archi, `default-features = false` sauf mention : `tokio` 1.54.1, `reqwest` 0.13.5 (`features = ["rustls"]`), `keyring` 4.2.0, `serde` 1.0.229 (`derive`), `serde_json` 1.0.154, `proptest` 1.12.0, `jiff` 0.2.39, `getrandom` 0.4.3 ;
  - `crates/core/Cargo.toml` : aucune `[dependencies]`, `[dev-dependencies] proptest = { workspace = true }`, `[features] sim = []` ;
  - `crates/cli/Cargo.toml` : retirer `chrono`, garder `clap` en local ;
  - `core/src/lib.rs` vidé de `add` (déclarations `mod` posées en T-3 à T-6) ;
  - `cli/src/main.rs` : un parseur clap dérivé, sans sous-commande (`--version`, `--help`) ;
  - `Cargo.lock` régénéré par `cargo check --workspace`.
- **Livrable** : `rust-toolchain.toml`, `Cargo.toml`, `crates/core/Cargo.toml`, `crates/cli/Cargo.toml`, `crates/core/src/lib.rs`, `crates/cli/src/main.rs`, `Cargo.lock`.
- **Critère de complétion** :
  - `cargo build --workspace --locked` vert ;
  - `cargo tree -p arbitre-core -e normal` n'affiche que `arbitre-core` ;
  - `cargo check -p arbitre-core --target wasm32-unknown-unknown` vert (matrice US-5, cas 1) ;
  - `grep -rn 'chrono\|rusqlite' crates/` vide ;
  - `arbitre --version` répond.
- **Effort estimé** : agent 5 min · arbitrage 0

### T-2 : Décision de dépendances et guide de contribution
- **Zone DDD** : Generic
- **Réf specs** : AC-8.1, AC-8.2, AC-8.4 ; C-8.1, C-8.3 ; MANQUE-ST01-4, MANQUE-ST01-5 ; arbitrage P-1 (réversibilité d'`aws-lc-rs`)
- **Dépendances** : T-1
- **Description** :
  - **`docs/decisions/ST-01-dependances.md`** : les huit sujets du tableau de l'archi, avec pour chacun le choix, la raison et l'option écartée. Il justifie les deux ajouts à PL-6 (`jiff`, `getrandom`). Il note la réversibilité rustls `aws-lc-rs` → `ring` et son piège (deux fournisseurs compilés ⇒ `install_default()` explicite au démarrage des binaires natifs). Il rappelle qu'une nouvelle dépendance commune est un amendement du socle.
  - **`CONTRIBUTING.md`** :
    - règle `Cargo.lock` : ne jamais le fusionner à la main ; prendre celui de `main`, puis `cargo check --workspace` ;
    - ajout d'un contrôle : `scripts/ci.d/NN-stxx-<sujet>.sh`, NN ≥ 60, aides dans `scripts/lib/`, fichiers `*-st01-*` gelés ;
    - formats de `zones.toml` (sémantique des motifs comprise), de `mutants_equivalents.toml` et de `.gitleaksignore` (une ligne `# justification` au-dessus de chaque empreinte) ;
    - interdiction de `#[mutants::skip]` et des options d'exclusion ;
    - montée de version d'un outil = amendement du socle.
- **Livrable** : `docs/decisions/ST-01-dependances.md`, `CONTRIBUTING.md`.
- **Critère de complétion** : relecture. Les six sujets d'AC-8.1 y figurent (matrice US-8, cas 4). Chaque version citée est égale à celle de `Cargo.toml`.
- **Effort estimé** : agent 10 min · arbitrage 10 min (relecture de Quentin)

### T-3 : Marques, horloge hybride et signalement d'avance (`hlc.rs`)
- **Zone DDD** : Core (Full)
- **Réf specs** : RG-ST01-4.1, 4.2, 4.3, 4.6 ; INV-ST01-4, 5, 6, 8 ; PROP-ST01-1 (partie pure) ; MANQUE-ST01-3 ; CONFLIT-ST01-3
- **Dépendances** : T-1
- **Description** :
  - `IdAppareil([u8; 16])`, avec un constructeur depuis `u128` ;
  - `Marque { l: u64, c: u32, a }`, dont l'`Ord` est dérivé dans cet ordre de champs ;
  - `Horloge::nouvelle(a)` ;
  - `emettre(pt) -> Result<Marque, CompteurEpuise>` : `l' = max(l, pt)`, `c' = c + 1` si `l' = l`, sinon `0` ; saturation de `c` ⇒ erreur, état inchangé ;
  - `recevoir(&Marque)` : `(l, c) ← max_lex` ;
  - `Seuil(NonZeroU64)`, avec la constante `SEUIL_DEFAUT` de 3 600 000 ms ;
  - `avances(lot, maintenant, σ) -> Vec<Avance>` : par appareil auteur, l'avance maximale strictement supérieure à `σ`, triée par appareil.
- **Livrable** : `crates/core/src/hlc.rs` ; tests unitaires dans le fichier ; propriétés dans `crates/core/tests/hlc.rs`.
- **Critère de complétion** :
  - matrice US-4, cas 2 à 10, verts ;
  - propriétés à 1 000 jeux : ordre total et antisymétrique (INV-ST01-4) ; pour toute suite d'heures `pt` (croissante, figée, décroissante, mélangée), émissions strictement croissantes et `l ≥ pt` (INV-ST01-5) ; après `recevoir(m)`, toute émission > `m` (INV-ST01-6) ; `avances` ne signale que `> σ` (INV-ST01-8) ;
  - test de saturation : `c = u32::MAX` et `pt` figé ⇒ `CompteurEpuise`, puis émission suivante identique.
- **Effort estimé** : agent 15 min · arbitrage 0

### T-4 : Port `Clock` et horloge simulée (`clock.rs`)
- **Zone DDD** : Supporting (Std)
- **Réf specs** : RG-ST01-4.4 ; AC-4.4
- **Dépendances** : T-1
- **Description** :
  - `trait Clock { fn now(&self) -> u64 }`, en ms UTC ;
  - `HorlogeSimulee`, sous `cfg(feature = "sim")` : `fixer(t)`, `avancer(Δ)`, `reculer(Δ)`, `figer()`, avec une mutabilité interne (`Cell`) pour que `now(&self)` suffise ;
  - pas d'adaptateur système (archi : ST-03).
- **Livrable** : `crates/core/src/clock.rs`.
- **Critère de complétion** : tests de `fixer`, `avancer`, `reculer` et `figer`. Matrice US-4, cas 1 vert : deux `HorlogeSimulee` décalées de 5 min, plus deux `Horloge` (T-3), donnent `m' > m`. `cargo check --target wasm32-unknown-unknown -p arbitre-core --features sim` vert.
- **Effort estimé** : agent 5 min · arbitrage 0

### T-5 : Écritures et validation de lot (`write.rs`, sans `vu_pour`)
- **Zone DDD** : Core (Full)
- **Réf specs** : RG-ST01-4.5 (avec E-1, E-2) ; AC-4.5 ; INV-ST01-7 (partie validation)
- **Dépendances** : T-3
- **Description** :
  - types `IdDonnee { module, genre, id }`, `Champ`, `Valeur(Vec<u8>)`, `Auteur` (4 variantes), `Quoi { Champ { nom, valeur }, Suppression }`, `Cible`, `Ecriture { donnee, quoi, marque, vu: Option<Marque>, auteur }` ;
  - `cible()`, `appareil()` (= `marque.a`) ;
  - `ErreurLot { index, raison }`, avec les raisons `VuNonAnterieur` et `IdentifiantVide` ;
  - `valider(lot) -> Result<(), ErreurLot>`, qui renvoie la première écriture fautive dans l'ordre du lot.
- **Livrable** : `crates/core/src/write.rs`.
- **Critère de complétion** :
  - matrice US-4, cas 11 vert ;
  - `vu = marque` et `vu > marque` ⇒ `VuNonAnterieur` ;
  - module, genre, id ou champ vide ⇒ `IdentifiantVide` ;
  - l'`index` désigne bien l'écriture fautive ;
  - les cas 12 et 13 de la matrice US-4 sont consignés comme garanties de type (E-1), dans un commentaire d'une ligne qui renvoie à l'archi.
- **Effort estimé** : agent 10 min · arbitrage 0

### T-6 : Journal, projection, écartées, anomalies (`merge.rs`, partie 1, et `vu_pour`)
- **Zone DDD** : Core (Full)
- **Réf specs** : RG-2.1, RG-2.3, RG-2.4 ; INV-4, INV-6, INV-7, INV-ST01-7 ; AMB-ST01-1 ; MANQUE-ST01-2 ; CONFLIT-ST01-1 (calcul de `vu`)
- **Dépendances** : T-5
- **Description** :
  - **Journal et fusion** : `Journal` est un `BTreeMap<(Cible, Marque), Contenu>`. `Journal::fusionner(&mut self, lot)` valide le lot en entier, puis insère, ou ne touche à rien. Une identité déjà connue avec un contenu différent garde le plus grand contenu selon `(valeur, vu, auteur)` et rend une `Anomalie`.
  - **Projection** : `projection()` suit RG-2.1 (gagnante par champ) et RG-2.3 (`t_s > t_f`). Les gagnantes d'une donnée supprimée restent présentes, masquées.
  - **Écartées** : `ecartees()` rend `S \ retenues`, dans l'ordre `(donnée, cible, marque)`.
  - **`vu_pour`** : `vu_pour(journal, donnée, quoi)` dans `write.rs`. Sa portée est celle de l'archi : pour un champ, la même cible plus les suppressions de la donnée ; pour une suppression, toute la donnée.
  - **`Rapport`** : `Rapport.nouvelles_ecartees` est la différence avant/après, triée. `nouveaux_conflits` reste vide jusqu'à T-7.
- **Livrable** : `crates/core/src/merge.rs`, `vu_pour` dans `crates/core/src/write.rs`, `lib.rs` (`mod` et réexports).
- **Critère de complétion** :
  - matrice US-1, cas 1, 2, 3, 7, 8 (10 000 écritures séquentielles : une gagnante, 9 999 écartées), 10 et 11 ;
  - matrice US-3, cas 1 à 9, sur l'état et les écartées seulement : les colonnes « conflit » passent en T-7 ;
  - matrice US-2, cas 9 ;
  - propriétés à 1 000 jeux : partition `retenues ⊎ écartées = S` et monotonie (INV-7) ; lot invalide ⇒ journal inchangé (INV-ST01-7) ; `vu_pour` rend le maximum de sa portée, ou `None`.
- **Effort estimé** : agent 20 min · arbitrage 0

### T-7 : Conflits et rapports triés (`merge.rs`, partie 2)
- **Zone DDD** : Core (Full)
- **Réf specs** : RG-ST01-2.1, RG-ST01-2.2 ; INV-ST01-2, INV-ST01-3 ; AMB-ST01-2, AMB-ST01-3 ; arbitrages « Même valeur » et « Double suppression »
- **Dépendances** : T-6
- **Description** :
  - **Règle de conflit** : `a_vu(w, x) ⇔ vu(w) ≥ marque(x)`. `exclusives` vaut pour la même cible de champ, ou pour une suppression contre un champ de la même donnée. Un conflit exige `concurrentes ∧ exclusives ∧ valeurs différentes`.
  - **`Journal::conflits()`** : calcul complet, pour `R(S)`. Chaque conflit est rendu sous la forme `(donnée, champ, retenue = max, écartée = min)`, trié par `(donnée, champ, marque retenue, marque écartée)`.
  - **`fusionner`** : il remplit `nouveaux_conflits` en n'évaluant que les paires (nouvelle × portée), puis retire ceux déjà connus.
- **Livrable** : `crates/core/src/merge.rs`.
- **Critère de complétion** :
  - matrice US-2, cas 1 à 8 et 10 ;
  - colonnes « conflit » de la matrice US-3, cas 1 à 6 ;
  - la seconde réception d'un lot rend des listes vides ;
  - propriété : `nouveaux_conflits` incrémentaux = `conflits(S ∪ L) \ conflits(S)` sur 1 000 jeux (garde le raccourci de performance honnête) ;
  - propriété : les trois listes sont triées (INV-ST01-3).
- **Effort estimé** : agent 20 min · arbitrage 5 min (si un cas de matrice se révèle ambigu à l'exécution)

### T-8 : Générateurs, appareils simulés et propriétés de convergence
- **Zone DDD** : Core (Full)
- **Réf specs** : E-1.1 ; RG-2.2, RG-ST01-1.1 ; INV-5, INV-ST01-1 ; PROP-ST01-1, PROP-ST01-2 ; MANQUE-ST01-7 ; AC-1.6, AC-3.5
- **Dépendances** : T-3, T-4, T-7
- **Description** :
  - **Support** (`crates/core/tests/support/`) :
    - générateurs biaisés selon E-1.1 : lots vides, doublons, au moins 3 appareils, même `l` et même `(l, c)`, suppressions, concurrentes de même valeur, écrasements séquentiels ;
    - écritures produites en faisant émettre des appareils simulés (`Horloge` + `Journal` + `HorlogeSimulee`), pour que `vu` soit cohérent ;
    - scénario = suite d'actions `émettre`, `livrer(i → j, sous-ensemble, ordre)`, `relivrer` ; pas d'objet réseau.
  - **Propriétés**, à `ProptestConfig { cases: 1000 }` fixé dans le code :
    - commutativité, associativité, idempotence, neutre, avec suppressions (INV-5, AC-3.5) ;
    - déterminisme sous permutation de l'ordre interne d'un lot (INV-ST01-1) ;
    - PROP-ST01-2 : N ≥ 3 appareils, livraisons aléatoires, puis tout livré à tous ⇒ même `R(S)` partout ; un écrasement fait après réception n'est jamais un conflit ;
    - assertion de couverture des familles d'E-1.1 sur un échantillon (R-7).
  - `proptest-regressions/` est versionné.
- **Livrable** : `crates/core/tests/support/mod.rs`, `crates/core/tests/convergence.rs`.
- **Critère de complétion** :
  - `cargo test -p arbitre-core --all-features` vert ;
  - matrice US-1, cas 4, 5, 6 et 9 couverts par propriété ;
  - matrice US-2, cas 5, et US-3, cas 8, couverts par PROP-ST01-2 ;
  - un bogue injecté à la main (départage par appareil inversé) fait échouer au moins une propriété en moins de 1 000 jeux. C'est une vérification ponctuelle, non versionnée.
- **Effort estimé** : agent 25 min · arbitrage 0

### T-9a : Aide `graphe.py` (arêtes, interdites, communes, pureté)
- **Zone DDD** : Supporting (outillage ; mutation non mesurée, limite assumée par l'archi)
- **Réf specs** : RG-ST01-5.1, 5.2, 6.3 ; INV-ST01-9, 10, 13 ; AMB-ST01-6 ; AC-6.3, AC-6.4
- **Dépendances** : T-1. **Parallélisable avec T-3 à T-8** : ne touche aucun fichier Rust.
- **Description** : `scripts/lib/graphe.py`, en Python ≥ 3.11 et bibliothèque standard seule. Il reçoit en entrée le JSON de `cargo metadata`, la sortie `cargo tree -f '{p} {f}'` par crate de la liste navigateur et les manifestes lus par `tomllib`. Il applique les règles (a) à (e) de l'archi :
  - (a) règles par nom pour les **neuf** crates de l'archi ; la chaîne de dépendances est nommée dans le message ;
  - (b) dépendances interdites dans le graphe normal de la liste navigateur ;
  - (c) recherche textuelle de `SystemTime`, `Instant::now` et `::now(` dans `crates/{core,plan}/src`, hors `clock.rs` ;
  - (d) ni `openssl-sys` ni `native-tls` dans tout le workspace ;
  - (e) dépendances communes reprises par `workspace = true`.

  Chaque échec rend une ligne qui nomme les composants et la dépendance.
- **Livrable** : `scripts/lib/graphe.py`, sa partie de `scripts/lib/test_ci.py`, données figées sous `scripts/lib/fixtures/`.
- **Critère de complétion** : `python3 -m unittest scripts/lib/test_ci.py` vert, avec les matrices US-5, cas 2 à 5, et US-6, cas 5 à 7 (arête directe, arête transitive, crate qui naît plus tard) sur données figées. Le message du cas 5 nomme `arbitre-plan` et `arbitre-diff`.
- **Effort estimé** : agent 15 min · arbitrage 0

### T-9b : Aide `mutation.py` (zones, équivalents, scores, rapport)
- **Zone DDD** : Supporting (outillage)
- **Réf specs** : RG-ST01-7.1, 7.2, 7.3 ; INV-ST01-15, 16 ; AMB-ST01-4 ; MANQUE-ST01-1 (arbitrage), MANQUE-ST01-5, MANQUE-ST01-8 ; C-7.4 ; AC-7.1 à AC-7.4, AC-7.6
- **Dépendances** : T-0 (cargo-mutants installé). **Parallélisable avec T-3 à T-8 et avec T-9a.**
- **Description** :
  - **Relevé du format réel** (R-3) : lancer `cargo mutants` 27.1.0 sur un mini-crate jetable sous `$TMPDIR` et en tirer les données figées : structure de `outcomes.json`, nom des issues, forme du nom de mutant, issue en cas d'échec de la base.
  - **Fonctions de `mutation.py`** :
    - traduction motif → regex (sous-section Under-engineering) ;
    - lecture de tous les `.ctdd/stories/*/zones.toml` ; clé inconnue ou TOML invalide ⇒ erreur qui nomme le fichier ;
    - zone la plus exigeante (`Full > Std > Gen`), `Full` par défaut ;
    - `Gen` dans `crates/core/` ou `crates/plan/` ⇒ `rouge` ;
    - lecture des `mutants_equivalents.toml`, à l'identique ; une justification vide ⇒ `rouge` ;
    - correspondance des issues (`caught`/`timeout` = détecté, `missed`, `unviable` = exclu, base rouge ⇒ `rouge`) ;
    - score par fichier, neutre si rien n'est compté, `≥` seuil pour passer ; `Gen` est affiché à titre informatif ;
    - rapport Markdown trié par fichier.
- **Livrable** : `scripts/lib/mutation.py`, sa partie de `scripts/lib/test_ci.py`, `scripts/lib/fixtures/mutants/`.
- **Critère de complétion** : `test_ci.py` vert, avec la matrice US-7, cas 1 à 11, plus cinq exemples de motifs (`*` ne traverse pas `/` ; `**` couvre 0 et n segments ; motif exact ; motif sans correspondance). Une sortie cargo-mutants illisible ⇒ `rouge`.
- **Effort estimé** : agent 20 min · arbitrage 0

### T-10 : Exécuteur `ci.sh`, versions épinglées et les cinq contrôles
- **Zone DDD** : Generic
- **Réf specs** : RG-ST01-6.1, 6.2, 6.4, 6.5 ; INV-ST01-11, 12, 14 ; CONFLIT-ST01-2 (arbitrage, avec E-3) ; MANQUE-ST01-4, MANQUE-ST01-6 ; AMB-ST01-5
- **Dépendances** : T-9a, T-9b
- **Description** :
  - **`scripts/outils.env`** : `CARGO_MUTANTS_VERSION=27.1.0`, `GITLEAKS_VERSION=8.30.1`, `PYTHON_MIN=3.11`.
  - **`scripts/ci.sh`** :
    - versions vérifiées à l'égalité exacte (minimum pour Python), avec la commande d'installation dans le message ; le cas du Python 3.9 d'Xcode est nommé (arbitrage P-3) ;
    - base : `$CI_BASE`, sinon `git merge-base HEAD origin/main` ;
    - exports `CI_BASE`, `CI_HEAD`, `CI_OUT=target/ci` ;
    - liste affichée, puis `bash` de chaque `ci.d/*.sh` trié avec `LC_ALL=C` ; tous sont exécutés, même après un échec ;
    - `ci.d/` vide ⇒ `rouge` ;
    - sous Actions : `::group::`, `::error::` et une ligne par contrôle dans `$GITHUB_STEP_SUMMARY`.
  - **Les cinq contrôles `ci.d/`**, tels que l'archi les décrit :
    - `10-st01-tests.sh` : `cargo test --workspace --all-features --locked`, plus `unittest` ;
    - `20-st01-navigateur.sh` : `arbitre-core` absent ⇒ `rouge` ;
    - `30-st01-graphe.sh` ;
    - `40-st01-secrets.sh` : gitleaks sur `CI_BASE..CI_HEAD`, plus la règle de justification de `.gitleaksignore` ;
    - `50-st01-mutation.sh` : diff vide ⇒ neutre, puis `cargo mutants --in-diff … --all-features --output $CI_OUT` et `mutation.py`, plus la recherche de `mutants::skip`, `exclude_re`, `exclude_globs` et `--exclude`.
  - `.gitleaksignore` vide.
- **Livrable** : `scripts/ci.sh`, `scripts/outils.env`, `scripts/ci.d/{10,20,30,40,50}-st01-*.sh`, `.gitleaksignore`.
- **Critère de complétion** :
  - en local, `scripts/ci.sh` est vert sur la branche de ST-01 une fois T-8 fait ; avant T-8, seul `50` peut être rouge ;
  - matrice US-6, cas 4, 9 (PR documentaire : vert, mutation neutre) et 11 (cargo-mutants retiré du `PATH` ⇒ `rouge` explicite) ;
  - un fichier `ci.d/60-demo.sh` qui fait `exit 1` est exécuté, nommé et rend `rouge`, sans modifier `ci.sh` (cas 10, fichier retiré ensuite).
- **Effort estimé** : agent 15 min · arbitrage 0

### T-11 : Workflow GitHub Actions
- **Zone DDD** : Generic
- **Réf specs** : C-6.6 ; RG-ST01-6.1 (local = CI) ; C-7.4
- **Dépendances** : T-10, T-0 (G-1)
- **Description** : `.github/workflows/ci.yml`, un job `ubuntu-latest`, **sur `pull_request` seulement** (sous-section Under-engineering). Étapes : `checkout` `fetch-depth: 0` ; toolchain depuis `rust-toolchain.toml` ; `setup-python` 3.13 ; cargo-mutants et gitleaks installés aux versions de `outils.env` (binaires précompilés) ; cache Cargo ; `CI_BASE=${{ github.event.pull_request.base.sha }} scripts/ci.sh` ; `target/ci/` publié en artefact `if: always()`. Toutes les Actions sont épinglées par SHA de commit. `permissions: contents: read`.
- **Livrable** : `.github/workflows/ci.yml`.
- **Critère de complétion** : la PR de ST-01 déclenche le job. Le résumé affiche une ligne par contrôle et l'artefact contient `mutation.md` (C-7.4). Aucune Action n'est référencée par un tag.
- **Effort estimé** : agent 10 min · arbitrage 5 min (Quentin confirme les réglages du dépôt : Actions autorisées, check requis sur `main`)

### T-12 : Démonstrations rouges (branches jetables, jamais envoyées)
- **Zone DDD** : Generic
- **Réf specs** : AC-6.4, AC-6.5 ; matrice US-5, cas 2 et 3 ; US-6, cas 3, 5, 6 et 8 ; US-7, cas 1
- **Dépendances** : T-10
- **Description** : sur des branches locales jetables, lancer `scripts/ci.sh` (ou le contrôle seul) après chacune de ces modifications, puis copier la sortie dans le rapport d'implémentation :
  1. `rusqlite` ajouté au cœur ;
  2. `chrono` avec `clock` ajouté au cœur ;
  3. crates factices `arbitre-plan` → `arbitre-diff`, en direct puis par une crate intermédiaire ;
  4. `reqwest` ajouté au cœur ;
  5. un faux jeton commité puis retiré au commit suivant (**jamais envoyé**) ;
  6. un test de `hlc.rs` supprimé, qui fait passer le score sous 85 %.
- **Livrable** : section « Démonstrations » du rapport de la phase 7 (sorties brutes, contrôle, message). Aucun fichier dans le dépôt.
- **Critère de complétion** : les six démonstrations sont rouges, avec un message qui nomme ce que la spec exige (composants, chaîne, commit, fichier/score/seuil). Les branches sont supprimées.
- **Effort estimé** : agent 10 min · arbitrage 0

### T-13 : PR de ST-01 et mutation sur elle-même
- **Zone DDD** : Core (critère de complétion de la story)
- **Réf specs** : K-7.5, AC-7.5 ; RG-ST01-7.3 ; plan du cadre § ST-01, critère de complétion
- **Dépendances** : T-2, T-8, T-11, T-12
- **Description** :
  - Rebaser sur `main`, envoyer la branche, ouvrir la PR. Relever la durée de la mutation (R-2).
  - Pour chaque mutant survivant de `hlc.rs`, `write.rs` ou `merge.rs` : écrire le test qui le tue ou, s'il est réellement équivalent, une entrée dans `.ctdd/stories/ST-01/mutants_equivalents.toml`, avec sa justification.
  - Au-delà de ~20 min de mutation : profil réduit (R-2), à arbitrer par Quentin.
- **Livrable** : PR verte ; `.ctdd/stories/ST-01/mutants_equivalents.toml` si besoin.
- **Critère de complétion** :
  - CI verte sur la PR ;
  - `mutation.md` affiche ≥ 85 % pour `hlc.rs`, `write.rs` et `merge.rs` (K-7.5) et ≥ 70 % pour `clock.rs` ;
  - chaque équivalent listé a été relu par Quentin.
- **Effort estimé** : agent 30 min (mutation comprise) · arbitrage 15 min (équivalents, durée de CI)

## Ordre d'exécution

```
T-0 ──────────────────────────────┐
T-1 ─┬─► T-2 ──────────────────────────────────────────────┐
     ├─► T-3 ─► T-5 ─► T-6 ─► T-7 ─┐                       │
     ├─► T-4 ──────────────────────┴─► T-8 ────────────────┤
     │                                                     │
     ├─► T-9a ─────────┐                                   │
     └── (T-0) ─► T-9b ┴─► T-10 ─┬─► T-11 ─────────────────┤
                                 └─► T-12 ─────────────────┴─► T-13
```

- **Deux voies explicites après T-1** : le moteur (T-3 à T-8) et l'outillage (T-9a, T-9b, T-10). Elles ne partagent aucun fichier. T-4 est parallélisable avec T-3.
- **Jonction en T-10** : son critère « vert complet » exige T-8.
- **T-13 est la seule tâche qui envoie** quelque chose sur GitHub, et elle exige T-0 (G-1).

### Estimation (temps d'agent et temps d'arbitrage)

| Voie | Tâches | Agent (min) | Arbitrage (min) |
|---|---|---|---|
| Préalable | T-0, T-1, T-2 | 20 | 20 |
| Moteur | T-3 → T-8 | 95 (T-4 en parallèle de T-3 : 90 en durée) | 5 |
| Outillage | T-9a ∥ T-9b → T-10 → T-11, T-12 | 70 (T-9a ∥ T-9b : 50 en durée) | 5 |
| Clôture | T-13 | 30 | 15 |
| **Total** | | **215 cumulés, ≈ 140 en durée avec les deux voies** | **≈ 45** |

- **Périmètre** : ce sont les phases 6 à 8 (Skeleton, implémentation, PR). Les cycles du Devil's Advocate et la relecture finale ne sont pas comptés.
- **Écart au cadre** : le cadre estime ST-01 à M (100 min machine) + forte (60 min d'arbitrage). La durée machine estimée ici, ≈ 140 min, dépasse M d'environ 40 %, avant même le DA et la review. L'arbitrage tient dans « forte ». Ce sont les données attendues par le recalibrage. On ne les corrige pas : on les relève (`state.json`).

## Invariants C-TDD à vérifier post-implémentation

| Invariant / Property | Tâche(s) | Vérification |
|---|---|---|
| INV-4 (gagnante par champ, champs indépendants) | T-6 | Matrice US-1, cas 1 à 3 et 8 ; propriété d'indépendance |
| INV-5 (convergence) | T-8 | Commutativité, associativité, idempotence, neutre ; 1 000 jeux, suppressions comprises |
| INV-6 (suppression datée, résurrection) | T-6 | Matrice US-3, cas 1 à 9 |
| INV-7 (partition, monotonie des écartées) | T-6 | Propriété à 1 000 jeux |
| INV-ST01-1 (déterminisme) | T-8, T-9a (pureté) | Permutation des entrées ; recherche de `SystemTime` et `::now(` |
| INV-ST01-2 (définition du conflit) | T-7 | Matrice US-2, cas 1 à 8 et 10 ; US-3, cas 1 à 6 |
| INV-ST01-3 (rapports triés) | T-7 | Propriété de tri sur les trois listes |
| INV-ST01-4, 5, 6 (ordre, émission, réception) | T-3 | Propriétés sur des suites d'heures quelconques |
| INV-ST01-7 (lot invalide refusé en entier) | T-5, T-6 | Journal inchangé après `ErreurLot` |
| INV-ST01-8 (avances) | T-3 | Seuil exact non signalé ; sans effet sur `R(S)` |
| INV-ST01-9 (cœur sans stockage ni heure) | T-1, T-9a, T-12 | `cargo tree` ; démonstrations 1 et 2 |
| INV-ST01-10 (dépendances communes au workspace) | T-1, T-9a | Règle (e) de `graphe.py` |
| INV-ST01-11 (verdict unique) | T-10, T-11 | Matrice US-6, cas 9, 10, 11 et 12 |
| INV-ST01-12 (liste navigateur) | T-10 | `20-st01-navigateur.sh` ; matrice US-6, cas 4 |
| INV-ST01-13 (arêtes interdites) | T-9a, T-12 | Données figées ; démonstration 3 |
| INV-ST01-14 (secrets par commit) | T-10, T-12 | Démonstration 5, en local |
| INV-ST01-15 (zone la plus exigeante, `Full` par défaut) | T-9b | Matrice US-7, cas 3, 4 et 10 |
| INV-ST01-16 (seuil bloquant, équivalents justifiés) | T-9b, T-12, T-13 | Matrice US-7, cas 1, 2 et 5 à 9 ; démonstration 6 ; PR de ST-01 |
| PROP-ST01-1 (horloge simulée) | T-3, T-4 | Matrice US-4, cas 1 à 4 |
| PROP-ST01-2 (N appareils, réseau simulé) | T-8 | Scénarios générés, N ≥ 3 |
| K-7.5 (≥ 85 % sur moteur et horloge) | T-13 | `mutation.md` de la PR |

## Points à arbitrer (Quentin)

| # | Question | Recommandation (retenue par défaut) |
|---|---|---|
| Q-1 | Workflow sur `pull_request` seulement, et non sur `push` vers `main` comme le disait l'archi ? | **Oui.** Toute fusion passe par une PR rebasée, et la base nulle d'un premier `push` cassait gitleaks et la mutation. |
| Q-2 | Les motifs de `zones.toml` sont traduits à la main en regex, puisque Python ≥ 3.11 n'a pas `full_match` : sémantique de l'archi gardée ? | **Oui.** Sémantique inchangée, figée par des exemples. |

## Arbitrages de Quentin (2026-11-03)

- **Q-1 → PR seulement, `main` protégée sans exception** (check CI requis, pas de contournement admin). Toute modification, y compris la doc `.ctdd/`, passe par une PR. Raison : travail parallèle sur plusieurs branches, relecture dans l'interface GitHub, discipline voulue même en solo. Les commits locaux antérieurs à la protection sont envoyés avant qu'elle soit activée.
- **Q-2 → Python ≥ 3.13, `PurePosixPath.full_match`.** Pas de traducteur glob → regex maison. Remplace le minimum 3.11 de l'arbitrage P-3 de l'archi : `outils.env` déclare 3.13 et `ci.sh` le vérifie avec un message explicite.
- **DA itération 1, F-9 → contrôle « juge touché ».** Un fichier `ci.d/` rend `rouge` toute PR qui modifie `ci.yml`, `ci.sh`, `ci.d/*-st01-*`, `outils.env`, les scripts de seuil, ou qui retire ou abaisse une entrée d'un `zones.toml` existant ; il liste les fichiers en cause. Ajouter une zone plus exigeante reste libre. Une modification légitime de la CI passe par une PR dédiée portant le label `ci-change`, posé par Quentin seul. Garde-fou contre la dérive d'un agent, pas contre la malveillance (la relecture de Quentin couvre ce cas).
