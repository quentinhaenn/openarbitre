# Code Review — ST-01 : Fusion pure et fondations du dépôt

- Date : 2026-11-03 · Branche : `st-01/fondations` @ `11a7425` · Diff : `git diff d40e577...HEAD`
- Entrées : `02_specs.md`, `03_archi.md`, `04_plan.md` v3 + `zones.toml`, `05_da_review.md` (v2) et `history/05_da_review_v1.md`, `06_skeleton.md`, `07_demonstrations.md`, `ci-local.log`, `target/ci/mutants.out/` (sorties brutes de la CI locale).
- Les sections « Arbitrages de Quentin » font foi. Revue en lecture seule : je n'ai rien lancé.

## Verdict : CHANGES REQUESTED

## Résumé
Le moteur est conforme aux specs arbitrées. Chaque invariant @Invariant du moteur a au moins un test qui vérifie la règle elle-même. L'oracle causal (T-7) compare, après chaque action et sur chaque appareil, les conflits à la règle arbitrée appliquée à ce que l'appareil savait vraiment. C'est la bonne preuve d'INV-ST01-2. Le juge tient l'arbitrage : aucun code de la PR n'est exécuté sous `pull_request_target`. Il n'y a aucun défaut bloquant. Trois points MAJEURS restent à corriger avant T-14 :
- la mesure de mutation du cœur a un angle mort, dû à `no_std` ;
- les trois survivants de `merge.rs` ne sont pas traités, alors que T-14 l'exige ;
- plusieurs preuves exigées par le plan ne sont consignées nulle part.

## Les trois mutants survivants de `merge.rs`

| Mutant | Verdict | Détail |
|---|---|---|
| `348:63` `ks.len() > 1` → `>= 1` (`anomalies_de`) | **Cas limite non testé** | Aucun test ne vérifie qu'une identité à **un seul** contenu n'est pas une anomalie (A-3, MANQUE-ST01-2). `us1_cas10` ne connaît qu'une identité, qui a deux contenus : sous le mutant, `anomalies().len()` vaut toujours 1. `us1_cas7`, `seconde_reception_*` et `inv_5_algebre` comparent les anomalies avant et après, ou ne regardent que la seconde réception : elles sont identiques sous le mutant. **Test à ajouter** : dans `us1_cas10`, après `recu(&[w1])`, `assert!(x.anomalies().is_empty())`. Ajouter aussi `assert!(rapport.nouvelles_anomalies.is_empty())` dans `us2_cas1`. |
| `361:51` `&&` → `\|\|` (`conflits_de`) | **Équivalent** | Sous le mutant, `visees_champ` reçoit (1) la cible `Suppression` de la donnée, une fois, et (2) des doublons de chaque cible de champ. Pour (1), cette liste ne sert que quand `w` est une suppression, et l'intervalle ne rend alors que des suppressions `x`. Deux gardes indépendantes les rejettent : `kx.valeur == kw.valeur` (`None == None`, l. 396), puis le `let … else` des l. 399-403, qui exige qu'un des deux côtés soit un `Champ`. Pour (2), les doublons produisent des `Conflit` identiques. Le tri par `cle_conflit` les rend adjacents, puisque la clé contient `rang` des deux écritures, injectif (voir ligne suivante). `dedup()` les retire. Les sorties sont donc identiques, seul le coût change. |
| `405:66` `rang(&w) > rang(&x)` → `>=` (`conflits_de`) | **Équivalent** | Les deux versions ne diffèrent que si `rang(w) == rang(x)`. `rang` = `(marque, cible, valeur, vu, auteur)` détermine l'`Ecriture` en entier : `donnee` est dans `cible`, et `quoi` se déduit de `visee` et de `valeur` (c'est exactement `en_ecriture`). Donc rangs égaux ⇒ `x == w`. Or ce cas sort à la l. 396 (`x == w`) avant d'atteindre la l. 405. Branche inatteignable. |

Le score de `merge.rs` passe déjà (91,2 %). Il passera à 32/32 une fois le test ajouté et les deux équivalents listés.

## Conformité specs

| Règle / invariant | Statut | Notes |
|---|---|---|
| RG-2.1 / INV-4 | ✅ | US-1 cas 1 à 3 et 8 (en séquentiel et en un lot), départage par appareil dans les deux ordres |
| RG-2.2 / INV-5 | ✅ | `inv_5_algebre` : commutativité (3 ordres), associativité (`a, b∪c` contre `a, b, c`), idempotence (rapport vide), neutre. Lots tirés de scénarios avec suppressions (AC-3.5) |
| RG-ST01-1.1 / INV-ST01-1 | ✅ | `#![no_std]` (A-6, le compilateur ferme l'heure et les E/S), `determinisme_permutation`, graine fixe |
| RG-2.4 / INV-7 | ✅ | `inv_7_partition` (`r ∩ e = ∅`, `r ∪ e = S`), `inv_7_monotonie`, US-2 cas 9 (écartée complète) |
| RG-ST01-2.1 / INV-ST01-2 | ✅ | Oracle causal par force brute, égalité dans les deux sens, après chaque action (F-2). Contre-exemple F-1, F-12 (sens de la paire, D-4), collision, même valeur, double suppression, pair jamais synchronisé, 3 concurrentes |
| RG-ST01-2.2 / INV-ST01-3 | ✅ | `inv_st01_3_tris` : ordre strict des clés A-4, sur `Rapport` et sur `conflits()`/`ecartees()`. Avances triées : `avances_maximum_par_appareil_tri_par_appareil` |
| RG-2.3 / INV-6 | ✅ | US-3 cas 1 à 9, état et conflits. Résurrection tous champs |
| RG-ST01-4.1 / INV-ST01-4 | ✅ | `inv_st01_4_ordre_total` |
| RG-ST01-4.2 / INV-ST01-5 | ✅ | Propriété sur suites de `pt` quelconques. Bornes `u32::MAX` et `L_MAX ± 1` (MANQUE-ST01-3, A-11) |
| RG-ST01-4.3 / INV-ST01-6 | ✅ | `inv_st01_6_reception`. Le journal adopte le max du lot (`fusionner_lot_valide_l_horloge_adopte_le_maximum`) |
| RG-ST01-4.4 / PROP-ST01-1 | ✅ | `HorlogeSimulee` (fixer, avancer, reculer, figer), US-4 cas 1 avec deux sources décalées de 5 min |
| RG-ST01-4.5 / INV-ST01-7 | ✅ | `inv_st01_7_lot_invalide` (position quelconque). Journal **et** horloge inchangés. Cas 12 et 13 non représentables (E-1, arbitré) |
| RG-ST01-4.6 / INV-ST01-8 | ✅ | Seuil exact non signalé, seuil réglé, retard non signalé, propriété. Sans effet sur la fusion par construction : `avances` est une fonction séparée |
| PROP-ST01-2 | ✅ | 3 à 5 appareils, livraisons partielles, réordonnées et rejouées. `couverture_des_familles` vérifie les 9 familles d'E-1.1 (R-7) |
| MANQUE-ST01-2 (anomalies) | ⚠️ Partiel | Le cas positif est testé, le cas négatif ne l'est pas (mutant 348, R-2) |
| RG-ST01-5.1 / INV-ST01-9 | ✅ | `graphe.py` (b), (c) ; démonstrations 1 et 2 |
| RG-ST01-5.2, 8.2 / INV-ST01-10 | ✅ | Règle (e) ; les 8 dépendances sont au workspace (A-5) ; vérification ponctuelle consignée dans `06_skeleton.md` |
| RG-ST01-6.1 / INV-ST01-11 | ✅ local | Échec fermé (outil absent, mauvaise version, `ci.d/` vide). US-6 cas 9, 10, 11 démontrés. Cas 12 : sur GitHub seulement |
| RG-ST01-6.2 / INV-ST01-12 | ✅ | `arbitre-core` toujours passé (AMB-ST01-5), `arbitre-plan` ignoré s'il est absent |
| RG-ST01-6.3 / INV-ST01-13 | ✅ | Table A-9, chaîne nommée, crate inconnue ⇒ rouge ; démonstrations 3a et 3b |
| RG-ST01-6.4 / INV-ST01-14 | ✅ local | Plage `base..tête`, démonstration 5 ; exceptions justifiées (MANQUE-ST01-4) |
| RG-ST01-6.5 | ✅ (écart arbitré) | `ci.d/` + label `ci-change` |
| RG-ST01-7.1 à 7.3 / INV-ST01-15, 16 | ✅ | US-7 cas 1 à 8, 10 et 11 en `unittest` ; cas 9 par `--in-diff`. Zone effective protégée par le juge (D-2) |
| C-5.3, C-8.1, C-8.3 | ✅ | `add` et « Hello, world! » retirés ; 6 sujets + 2 ajouts ; règle `Cargo.lock` dans `CONTRIBUTING.md` |
| K-7.5 | ⚠️ | Seuils tenus (hlc 100 %, write 100 %, merge 91,2 %, clock 100 %), mais la mesure est amputée (R-1) |

## Conformité plan

| Tâche | Statut | Notes |
|---|---|---|
| T-0 | ⏳ GitHub | `main` locale a 7 commits d'avance sur `origin/main`. Protection, G-1 et label à constater |
| T-1, T-2 | ✅ | |
| T-3 à T-6 | ✅ | `ponytail:` de D-8 présent dans `fusionner` |
| T-7 | ⚠️ | Code conforme. Les vérifications ponctuelles (i) « `vu` scalaire ⇒ la propriété échoue » et (ii) « départage inversé » ne sont consignées nulle part (R-3) |
| T-8 | ⚠️ | Code conforme. La vérification ponctuelle « `std::fs::read` dans `merge.rs` ⇒ `cargo check` échoue » n'est pas consignée (R-3) |
| T-9, T-10, T-11 | ✅ | |
| T-12 | ⚠️ | La démonstration 6 n'est pas refaite sur `hlc.rs` alors que le moteur est fini (`07_demonstrations.md` : « À refaire ») (R-3) |
| T-13 | ⚠️ | Le commit `04b0c8a` liste les propositions retenues, pas les écartées avec leur raison (R-3) |
| T-14 | ⏳ | Le traitement des survivants peut se faire dès maintenant (R-2). Le reste est sur GitHub |

## Issues détaillées

### [MAJEUR] R-1 : avec `no_std`, les mutants « fonction vide » du cœur sont non construisibles, donc pas mesurés
- **Réf** : K-7.5, RG-ST01-7.3 (`comptés = mesurés \ non_construisibles`), A-6 / D-7
- **Fichier** : `crates/core/src/lib.rs:6` ; `target/ci/mutants.out/unviable.txt`
- **Description** : 44 mutants sur 107 sont `unviable`. Environ 30 d'entre eux le sont parce que `vec!` n'existe pas dans une crate `no_std`. Le log de `hlc.rs:92` dit : « error: cannot find macro `vec` in this scope ». Ce sont les mutants les plus élémentaires : `avances`, `conflits`, `conflits_de`, `ecartees`, `ecartees_de`, `anomalies`, `anomalies_de`, `nouveaux`, `ecritures`, `donnees`, `sorties` → `vec![]`. Le score les exclut, comme la spec le permet, mais aucun n'a été testé. C'est pour cette raison que `hlc.rs` n'a que 6 mutants comptés. L'angle mort vaudra pour toute crate `no_std` à venir (`arbitre-plan`). La plupart seraient probablement attrapés, mais ce n'est pas mesuré.
- **Action requise** : `#[macro_use] extern crate alloc;` dans `lib.rs` (une ligne, sans effet sur le code), puis relancer `50-st01-mutation`. Les 3 mutants `::std::iter::*` de `Journal::de` et les `Default::default()` sur des types sans `Default` restent non construisibles à bon droit.

### [MAJEUR] R-2 : survivants de `merge.rs` non traités (critère de T-14)
- **Réf** : T-14 (« pour chaque mutant survivant : le test qui le tue, ou une entrée justifiée ») ; A-3 ; MANQUE-ST01-5
- **Fichiers** : `crates/core/tests/merge.rs` ; `.ctdd/stories/ST-01/mutants_equivalents.toml`, à créer
- **Action requise** :
  - `348:63` : ajouter les deux assertions de la section « Les trois mutants survivants » ;
  - `405:66` : l'inscrire comme équivalent, avec la preuve ci-dessus ;
  - `361:51` : l'inscrire comme équivalent, ou plutôt supprimer le site du mutant (moins de code) : `let visees_champ: BTreeSet<Cible> = self.de(d).map(|((c, _), _)| c).filter(|c| matches!(c.visee, Visee::Champ(_))).cloned().collect();`. Voir la question 2 à Quentin.

### [MAJEUR] R-3 : preuves exigées par le plan non consignées
- **Réf** : T-7 (critère : vérifications (i) et (ii) « consignées au rapport ») ; T-8 (vérification `std::fs`) ; T-12 (démonstration 6) ; T-13 (« chaque proposition écartée a une raison d'une ligne »)
- **Description** : `state.json` indique `7_implementation.file = null`, et aucun rapport d'implémentation n'existe. La vérification (i) est la seule preuve que la propriété « concurrentes ⇔ conflit » voit la règle de la v1 (F-2 du DA v1). La mutation ne la remplace pas : aucun mutant ne remplace `vu` par une marque scalaire.
- **Action requise** : refaire (i), (ii), la vérification `std::fs` et la démonstration 6 sur une branche jetable, puis les consigner avec la liste des écartées de T-13. Le plus simple est une section dans `07_demonstrations.md`.

### [MINEUR] R-4 : `.gitleaksignore` et `.cargo/` hors du périmètre du juge
- **Réf** : A-8 (règle 1) ; MANQUE-ST01-4 ; arbitrage F-9 (« dérive d'un agent »)
- **Fichier** : `scripts/lib/juge.py:12`
- **Description** : un agent qui voit `40-st01-secrets` rouge aura pour réflexe d'ajouter une empreinte justifiée à `.gitleaksignore`. Rien ne l'en empêche : la justification exigée par `40` ne demande qu'une ligne. De même, un `.cargo/config.toml` (`[env]`, `rustflags`) agit sur `cargo test` et `cargo mutants` sans toucher `scripts/`. Q-8 n'a accepté ce reste que pour `mutants_equivalents.toml`.
- **Suggestion** : `PROTEGES = (".github/", "scripts/", ".gitleaksignore", ".cargo/")`, plus une ligne dans `test_ci.py`. La PR fondatrice n'est pas jugée, donc pas besoin du label.

### [MINEUR] R-5 : temps mesuré à l'horloge dans un test
- **Réf** : T-6 (« en moins de 2 s en debug »)
- **Fichier** : `crates/core/tests/merge.rs`, `us1_cas8_dix_mille_ecritures_en_un_lot_recu`
- **Description** : `Instant::now()` < 2 s peut échouer sur un runner chargé, ou pendant la mutation avec `--jobs` (R-2 du plan). Un faux `caught` gonflerait le score, un faux rouge sur la base bloquerait la CI. Aujourd'hui 8,7 s pour 44 tests : la marge est inconnue.
- **Suggestion** : garder, et relever la durée réelle en T-14. Si elle dépasse 0,5 s, passer la borne à 10 s : elle garde le contrôle du quadratique (A-10), qui coûterait des minutes.

### [MINEUR] R-6 : US-4 cas 1 ne vérifie pas que « la correction gagne » à la fusion
- **Réf** : matrice US-4 cas 1 (« `m' > m` ; la correction gagne », RG-2.1)
- **Fichier** : `crates/core/tests/hlc.rs:23`
- **Description** : le test s'arrête à `m2 > m`. La suite (gagnante = la correction) découle de RG-2.1, testé par ailleurs, et la propriété `convergence` la couvre de bout en bout. Le cas nominal n'est pas écrit tel quel.
- **Suggestion** : trois lignes avec deux `Journal` au lieu de deux `Horloge`. Optionnel.

## Juge (`juge.yml`, `juge.py`) contre l'arbitrage DA v2 F-1

| Exigence | Constat |
|---|---|
| `pull_request_target`, types avec `labeled`/`unlabeled` | ✅ |
| `permissions: contents: read`, aucun secret | ✅ |
| `checkout` sans `ref:` (code de `main`), `persist-credentials: false`, `fetch-depth: 0` | ✅ |
| Tête dans la base d'objets seulement (`git fetch origin refs/pull/N/head`, aucun checkout) | ✅ |
| Aucun `${{ }}` dans un `run:` (numéro, SHA et labels passés par `env:`) | ✅ |
| Rien de la PR n'est exécuté ni importé : `juge.py` et `mutation.py` viennent de l'arbre de `main` (`sys.path[0]`). De la tête, il ne lit que `git diff --name-only` (pas de pilote de diff), `git ls-tree` et `git show` des `zones.toml`, analysés par `tomllib` | ✅ |
| Règle 1 : tout `.github/` et `scripts/`, ajouts et suppressions compris, `--no-renames` | ✅ (périmètre : R-4) |
| Règle 2 : zone effective des `.rs` de la base, `Full` par défaut, toutes les `zones.toml` ; tête illisible ⇒ rouge | ✅ ; démonstrations 8 et 9 |
| Avec le label : listes affichées, rouge si `crates/` est touché | ✅ ; démonstrations 13a et 13b |
| Ligne R-11 dans `juge.yml` et `CONTRIBUTING.md` | ✅ |

**Conforme** : aucun code de la PR n'est exécuté sous `pull_request_target`.

## Métriques C-TDD (zone Core)

| Métrique | Valeur | Seuil | Statut |
|---|---|---|---|
| Mutation `hlc.rs` | 100 % (6/6) | ≥ 85 % | ✅ mais assiette réduite (R-1) |
| Mutation `write.rs` | 100 % (13/13) | ≥ 85 % | ✅ |
| Mutation `merge.rs` | 91,2 % (31/34) | ≥ 85 % | ✅ ; 1 cas manqué, 2 équivalents (R-2) |
| Mutation `clock.rs` | 100 % (9/9) | ≥ 70 % | ✅ |
| `main.rs` (Gen) | 0/1 | informatif | ✅ (D-3) |
| Tests Rust | 81/81 | 100 % | ✅ |
| Tests Python | 50/50 | 100 % | ✅ |
| Propriétés | 1 000 jeux, graine fixe | `N` = 1 000 | ✅ |

## Failles DA adressées

| Faille | Statut | Notes |
|---|---|---|
| v1 F-1 (`vu` scalaire) | ✅ | `vu` par visée, test `contre_exemple_f1`. Famille couverte par le générateur |
| v1 F-2 (oracle) | ✅ code / ⚠️ preuve | Oracle sur le registre de vérité ; vérification (i) non consignée (R-3) |
| v1 F-3, F-4, F-5, F-10, F-11, F-16, F-20 | ✅ | Graine fixe ; `BTreeSet<Contenu>` ; clés A-4 ; `L_MAX` ; `vu_pour` privée ; horloge dans le journal ; intervalle |
| v1 F-7 / v3 D-7 | ✅ | `no_std`. Effet de bord non prévu sur la mutation : R-1 |
| v1 F-8 | ✅ | `git grep mutants::skip`, `mutants.toml` interdit |
| v2 F-1 (juge depuis la tête) | ✅ | `juge.yml` depuis `main` ; démonstrations 10 à 12. Contre-exemple 1 sur GitHub (T-14) |
| v2 F-2 (motif qui abaisse `Full`) | ✅ | Démonstration 9 |
| v2 F-3 (`main.rs` en `Full`) | ✅ | `Gen`, informatif |
| v2 F-4 (sens de la paire F-12) | ✅ | `cas_f12_meme_marque_suppression_et_champ` : retenue = le champ |
| v2 F-5 (commande PUT) | ⏳ | T-14 |
| v2 F-8, F-9, F-10 | ✅ | `ponytail:` ; contrat ST-02 dans `CONTRIBUTING.md` ; égalité ensembliste sans doublon |

## Vérifiable seulement sur GitHub (hors défauts)

- **T-0** : G-1 (push protection) active ; `main` envoyée (7 commits d'avance aujourd'hui) ; protection `enforce_admins` ; label `ci-change`.
- **T-14** :
  - `ci` vert sur la PR (US-6 cas 1) ;
  - les SHA épinglés correspondent bien aux tags commentés ;
  - `taiki-e/install-action` fournit `cargo-mutants@27.1.0` et `gitleaks@8.30.1` ;
  - durée de la mutation (R-2 du plan) et écarts de délai (R-10) ;
  - `mutation.md` identique à l'octet entre local et CI (US-6 cas 12) ;
  - démonstration du contre-exemple 1 (PR qui neutralise `juge.py` ⇒ `juge` rouge) ;
  - checks requis `ci` et `juge` (`PUT` de D-5) ;
  - PR du rapport verte sur les deux checks.
- La PR de ST-01 n'est pas jugée par `juge` (limite d'A-8) : c'est la relecture de Quentin qui la couvre.

## Points à arbitrer (Quentin)

| # | Question | Recommandation |
|---|---|---|
| 1 | R-1 : rendre `vec!` disponible dans le cœur (`#[macro_use] extern crate alloc;`) pour que les mutants « fonction vide » soient mesurés ? | **Oui.** Une ligne, et environ 7 min de mutation en plus. Sinon, l'angle mort vaut pour tout le cœur et pour `arbitre-plan`. |
| 2 | Mutant `361:51` : l'inscrire comme équivalent, ou réécrire `visees_champ` en `BTreeSet` pour supprimer le site ? | **Réécrire.** Moins de code, et l'équivalence ne repose plus sur deux gardes situées 40 lignes plus bas. `405:66` va dans `mutants_equivalents.toml`. |
| 3 | R-4 : ajouter `.gitleaksignore` et `.cargo/` aux chemins protégés par le juge ? | **Oui**, tant que la PR fondatrice n'est pas jugée : après, il faudra le label. |
| 4 | R-3 : refaire les preuves manquantes ou les accepter par substitution ? | **Refaire** (i) de T-7 et la démonstration 6 (30 min d'agent environ). Accepter les autres si Quentin le juge suffisant : (ii) est couvert par les mutants `Ord`, et `std::fs` par le compilateur. |

## Arbitrages de Quentin (2026-11-03)

- Validé en bloc : R-1 (`#[macro_use] extern crate alloc;` puis mutation relancée), R-2 (348 : assertions ajoutées ; 361 : `visees_champ` en `BTreeSet` ; 405 : `mutants_equivalents.toml` avec preuve), R-3 (vérification (i) de T-7, démonstration 6 sur `hlc.rs`, vérification `std::fs` de T-8, consignées), R-4 (`.gitleaksignore` et `.cargo/` protégés par le juge dès la PR fondatrice).
