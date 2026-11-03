# Plan d'implémentation — ST-01 : Fusion pure et fondations du dépôt (v2)

## Traçabilité
- Specs : `.ctdd/stories/ST-01/02_specs.md`. Ses « Arbitrages de Quentin (2026-11-03) » font foi, dont **F-1 : `vu` = table par cible**.
- Archi : `.ctdd/stories/ST-01/03_archi.md`. Ses arbitrages font foi : P-1, P-2, P-3, F-1, passe de simplification avant la review. Les écarts de ce plan à l'archi sont dans « Amendements à l'archi ». `03_archi.md` n'est pas réécrit.
- Plan v1 : `history/04_plan_v1.md`. Revue DA v1 (BLOQUÉ, F-1 à F-20) : `05_da_review.md`. Arbitrages du plan v1 (Q-1, Q-2, F-9) : repris en fin de fichier.
- Cadre : `.ctdd/cadre/04_plan.md` § ST-01 ; `.ctdd/cadre/03_archi.md` § Graphe de dépendances (pour F-14).
- Livré avec ce plan : `.ctdd/stories/ST-01/zones.toml` (inchangé depuis la v1).
- Date : 2026-11-03 · Statut : DRAFT v2. Numérotation des tâches refaite : la v1 ne sert plus de référence.

## Contexte technique
- **Codebase** (relevée le 2026-11-03) :
  - `crates/core` (`arbitre-core`) contient `add` et dépend de `chrono/clock` et de `rusqlite` ;
  - `crates/cli` (`arbitre`) affiche « Hello, world! » et dépend de `chrono` et de `clap` ;
  - `Cargo.lock` est versionné ; ni CI, ni `scripts/`, ni `docs/` ;
  - `origin` = `github.com/quentinhaenn/openarbitre`, **public**. `main` locale est **en avance de 4 commits**, et `main` n'a **aucune protection** (API : 404).
- **Stack** :
  - Rust 1.99.1, édition 2024, cible `wasm32-unknown-unknown` ;
  - `proptest` 1.12.0 ;
  - **Python ≥ 3.13**, bibliothèque standard seule (`tomllib`, `json`, `PurePosixPath.full_match`). Le Python local est en 3.15 ;
  - bash, gitleaks 8.30.1, cargo-mutants 27.1.0 (**non installé**), GitHub Actions.
- **Patterns à respecter** :
  - cœur fonctionnel : l'heure entre en valeur `u64` ;
  - aucun `HashMap` dans le cœur ;
  - états impossibles non représentables ;
  - aucune dépendance normale dans le cœur ;
  - `ci.sh` est un exécuteur gelé, les contrôles vivent dans `scripts/ci.d/`.
  - Les ordres contractuels sont ceux de l'amendement A-4, pas « tous les `Ord` dérivés ».

## Correspondance failles → corrections

| Faille | Correction retenue (forme la plus simple qui ferme la faille) | Où | Statut |
|---|---|---|---|
| F-1 | `vu` devient une table `Visee → Marque` (arbitrage F-1), clé sans la donnée | A-1 ; T-4, T-5, T-6 | Levée. Contre-exemple F-1 = test nommé (T-6) |
| F-2 | Propriété d'oracle : les conflits calculés sont égaux aux conflits déduits de ce que chaque appareil simulé connaissait vraiment au moment d'écrire | T-7 | Levée |
| F-3 | Graine proptest fixée dans le code ; matrice US-6, cas 12 dans le critère de T-14 | T-7, T-14 | Levée, sauf le délai d'attente des mutants (R-10, accepté) |
| F-4 | Le journal garde **toutes** les écritures d'une identité. L'anomalie devient une fonction de `S` | A-3 ; T-5 | Levée. Effet de bord à confirmer : Q-3 |
| F-5 | Un seul ordre écrit par sortie. Les `Ord` dérivés ne sont contractuels que sur les clés | A-4 ; T-5, T-6 | Levée |
| F-6 | Features écrites dépendance par dépendance. Vérification ponctuelle par une crate jetable qui hérite des huit | A-5 ; T-1 | Levée |
| F-7 | (c) sans exemption de `clock.rs`, avec les motifs `UNIX_EPOCH` et `env::` en plus | A-6 ; T-8 | Levée (reste : obfuscation volontaire, hors périmètre) |
| F-8 | Périmètre fixé : `mutants::skip` dans `crates/**/*.rs`, et tout fichier `mutants.toml` du dépôt | A-7 ; T-10 | Levée |
| F-9 | Contrôle `05-st01-juge.sh` (arbitrage F-9) | A-8 ; T-9, T-10, T-11 | Levée (restes : Q-8) |
| F-10 | `l < 2^53` vérifié à la validation et à l'émission | A-11 ; T-3, T-4 | (2) levée. (1), compteur saturé reçu : **acceptation proposée**, Q-4 |
| F-11 | `Journal::ecrire` est le seul chemin d'une écriture locale, et `vu_pour` devient privée | A-2 ; T-5 | Levée pour les surfaces. Un lot forgé à la main reste possible : Q-6 |
| F-12 | Comportement fixé et testé : `présente`, plus un conflit. Pas de validation ajoutée | T-6 | **Acceptation proposée**, Q-5 |
| F-13 | Python 3.13 et `full_match` partout ; plus de traducteur | tout le plan | Levée |
| F-14 | Table d'autorisations par crate, tirée du graphe du cadre | A-9 ; T-8 | Levée |
| F-15 | `clock.rs` rejoint T-3 | T-3 | Levée |
| F-16 | Le `Journal` porte l'horloge : `fusionner` valide, insère, puis adopte la marque maximale du lot, ou ne touche à rien | A-2 ; T-5 | Levée |
| F-17 | Levier de durée = `--jobs` de cargo-mutants, jamais moins de jeux | R-2 ; T-14 | Levée |
| F-18 | T-0 envoie `main` et pose la protection. T-14 ajoute le check requis | T-0, T-14 | Levée |
| F-19 | Démonstration « test en échec » | T-12 | Levée |
| F-20 | Conflits par requête d'intervalle sur le journal trié : `O(n log n + conflits)` | A-10 ; T-6 | Levée |

## Amendements à l'archi

Ces amendements remplacent, dans `03_archi.md`, les passages cités. L'orchestrateur les y reporte après arbitrage. Rien d'autre ne change dans l'archi.

### A-1 : `vu` par visée (F-1)
Remplace : « `vu : Option<Marque>` », « Portée de `vu` », et la ligne `Cible` du tableau des types.
- `Visee = Champ(Champ) | Suppression`, `Ord` dérivé (les champs avant la suppression). `Cible = { donnee: IdDonnee, visee: Visee }`.
- `Vu = BTreeMap<Visee, Marque>`. La clé omet la donnée : toutes les entrées portent sur la donnée de l'écriture, et une entrée sur une autre donnée n'est pas représentable.
- `Ecriture = { donnee, quoi, marque, vu: Vu, auteur }`.
- Contenu de `vu`, calculé par le journal (A-2) :
  - pour une écriture de champ `f`, les entrées `Champ(f)` et `Suppression` ;
  - pour une suppression, une entrée par visée connue de la donnée, champs et suppression.
  - Chaque entrée vaut la plus grande marque connue sur cette visée. Une visée inconnue n'a pas d'entrée.
- `a_vu(w, x) ⇔ vu(w)[visee(x)] ≥ marque(x)`. Une entrée absente vaut `⊥`.
- Validation : chaque entrée doit être `< marque`. Des entrées en trop (par exemple une écriture de champ qui a une entrée pour un autre champ) sont inoffensives, puisque ces paires ne sont jamais exclusives. Elles ne sont pas refusées.

### A-2 : le journal porte l'horloge de l'appareil (F-11, F-16)
Remplace, dans « Interfaces à définir » : `vu_pour` public, `Journal::vide`, `Journal::fusionner(lot)`, et le « contrat de la coquille » (« toute écriture entrée dans le journal est passée par `recevoir` »).
- `Journal::nouveau(appareil)` contient une `Horloge` et les écritures connues.
- `Journal::ecrire(&mut self, pt, donnee, quoi, auteur) -> Result<Ecriture, ErreurEcriture>`. Elle valide les identifiants **avant** d'émettre (un refus ne consomme pas de marque). Puis elle émet, calcule `vu` par `vu_pour` (privée), insère l'écriture et la rend pour l'envoi.
- `Journal::fusionner(&mut self, lot) -> Result<Rapport, ErreurLot>`. Elle valide tout le lot. Si une écriture est invalide, le journal **et l'horloge** restent inchangés. Sinon elle insère, puis l'horloge adopte la plus grande marque du lot (RG-ST01-4.3).
- Une surface ne construit donc jamais `vu` elle-même, et l'ordre fusion / réception n'est plus à réécrire. `R(S)` reste une fonction de `S` seul, et l'appareil du journal n'y entre pas.
- `ErreurEcriture = HorlogeEpuisee | Invalide(raison)`.

### A-3 : identités en collision (F-4, MANQUE-ST01-2)
Remplace : `Journal = BTreeMap<(Cible, Marque), Contenu>` et « on retient le plus grand contenu ».
- `Journal = BTreeMap<(Cible, Marque), BTreeSet<Contenu>>`, avec `Contenu = (valeur?, vu, auteur)`. Il y a un seul élément, sauf en cas de collision. Rien n'est jamais remplacé, donc `S` croît par union au sens strict.
- Rang total d'une écriture : `(marque, cible, contenu)`. La gagnante d'un champ est celle de rang maximal, ce qui reprend le départage de l'archi par `(valeur, vu, auteur)`. Les autres contenus sont écartés : la partition est tenue.
- `Journal::anomalies()` renvoie les identités qui ont au moins deux contenus, en fonction de `S`. `Rapport.nouvelles_anomalies` est la différence avant/après. Une seconde réception ne donne aucune nouvelle anomalie.
- Effet de la règle, sans exception ajoutée : deux contenus de valeurs différentes sur la même identité sont aussi un **conflit** (`vu < marque` des deux côtés, donc concurrents). Voir Q-3.

### A-4 : ordres (F-5)
Remplace : « les tris de RG-ST01-2.2 découlent des `Ord` dérivés » et « l'ordre de déclaration des champs de […] `Conflit` est contractuel ».
- **Contractuels** : les `Ord` dérivés de `IdAppareil`, `Marque` (`l, c, a`), `IdDonnee` (`module, genre, id`), `Visee` et `Cible` (`donnee, visee`).
- **Écartées** : ordre d'itération du journal, `(cible, marque, contenu)`. C'est `(donnée, cible, marque)` des specs, avec le contenu comme départage des collisions.
- **Conflits** : tri explicite par la clé `(donnée, champ, retenue.marque, écartée.marque, rang(retenue), rang(écartée))`, puis dédoublonnage. Les deux derniers termes ne servent qu'aux collisions.
- **Avances** : par appareil.
- Les `Ord` dérivés d'`Ecriture`, de `Contenu` et de `Conflit` ne servent qu'aux ensembles et aux départages. Leur ordre de champs n'est pas contractuel.

### A-5 : dépendances communes (F-6)
Remplace : la colonne « Déclaration au workspace » et la phrase « le `Cargo.lock` fait foi ».

| Dépendance | Déclaration au workspace |
|---|---|
| `tokio` 1.54.1, `keyring` 4.2.0, `jiff` 0.2.39, `getrandom` 0.4.3 | `default-features = false` |
| `reqwest` 0.13.5 | `default-features = false, features = ["rustls"]` (nom de feature vérifié en T-1) |
| `serde` 1.0.229 | `default-features = false, features = ["derive"]` |
| `serde_json` 1.0.154 | **features par défaut gardées** (`std`) |
| `proptest` 1.12.0 | **features par défaut gardées** (`std` : persistance des échecs, `RngSeed`) |

- Le `Cargo.lock` ne fait foi que pour les dépendances héritées.
- Les huit déclarations sont vérifiées **une fois** en T-1, par une crate jetable non versionnée qui les hérite toutes, puis à leur premier héritage réel. Aucun contrôle permanent n'est ajouté : une déclaration ne change que par amendement du socle.

### A-6 : pureté du cœur (F-7)
Remplace la règle (c) de `graphe.py`.
- Elle s'applique à toutes les sources de `crates/{core,plan}/src`, **sans exemption**.
- Motifs recherchés : `SystemTime`, `Instant`, `UNIX_EPOCH`, `::now(`, `env::`.
- `HorlogeSimulee` ne contient aucun de ces motifs : `fn now(` n'est pas `::now(`. L'adaptateur système vivra hors du cœur (ST-03).
- `env::` couvre la lecture de `TZ` et de `LANG`, c'est-à-dire la matrice US-1, cas 9.

### A-7 : canaux d'exclusion de mutation (F-8)
Remplace : « `50-st01-mutation.sh` les recherche (dépôt et `.cargo/mutants.toml`) ». Le contrôle est `rouge` si l'une de ces conditions est vraie :
- `mutants::skip` apparaît dans un fichier de `git ls-files 'crates/**/*.rs'` ;
- `git ls-files` contient un fichier dont le nom de base est `mutants.toml`, à n'importe quelle profondeur.

`--exclude` et `exclude_re` ne peuvent apparaître que dans la ligne de commande de `50-st01-mutation.sh`, que protège A-8. La documentation n'est pas fouillée.

### A-8 : contrôle « juge touché » (F-9, arbitrage)
Complète « CI : arborescence et contrat ».
- **Fichier** : `scripts/ci.d/05-st01-juge.sh`. ST-01 prend les numéros 05 à 50, et les stories suivantes ≥ 60, comme avant.
- **Fichiers protégés** : `.github/workflows/*`, `scripts/ci.sh`, `scripts/outils.env`, `scripts/ci.d/*-st01-*`, `scripts/lib/graphe.py`, `scripts/lib/mutation.py`.
- **Règle** :
  - une modification, une suppression ou un renommage d'un fichier protégé dans `CI_BASE...CI_HEAD` ⇒ `rouge`, avec la liste des fichiers. Un ajout reste libre : un nouveau contrôle ou une nouvelle aide dans `scripts/lib/`, et ST-01 elle-même.
  - pour chaque `.ctdd/stories/*/zones.toml` présent à la base, chaque couple `(motif, zone)` doit rester dans le même fichier à la tête, avec une zone au moins aussi exigeante. Sinon ⇒ `rouge`. Ajouter une zone ou un motif reste libre.
- **Label `ci-change`** :
  - avec le label, les deux règles précédentes ne rendent plus `rouge`. Le contrôle liste quand même les fichiers ;
  - en échange, la PR ne doit toucher aucun fichier sous `crates/`, sinon ⇒ `rouge`. C'est ce qui la rend « dédiée ».
  - Le label arrive par `CI_LABELS` : variable d'environnement, liste séparée par des virgules. Le workflow la remplit, et on la pose à la main en local. `ci.sh` n'est pas modifié.
- **Workflow** : `pull_request` seulement (arbitrage Q-1), avec les types `opened, synchronize, reopened, labeled, unlabeled`, pour qu'un label relance le check.
- La logique sur les zones va dans `mutation.py`, qui détient déjà la lecture du format.

### A-9 : arêtes entre composants (F-14)
Remplace la règle (a) de `graphe.py`. Chaque crate a une table d'autorisations : l'ensemble des composants atteignables, transitivement, sur toutes les sortes de dépendances.

| Crate | Composants autorisés (transitif) |
|---|---|
| `arbitre-core` | aucun |
| `arbitre-plan`, `arbitre-diff`, `arbitre-sqlite`, `arbitre-relay`, `arbitre-ai` | `arbitre-core` |
| `arbitre-calendar` | `arbitre-plan`, `arbitre-core` |
| `arbitre-web` | `arbitre-core`, `arbitre-plan`, `arbitre-sqlite` (stockage navigateur, si S1-B retient sqlite-wasm : Q-7) |
| `arbitre` | toutes, sauf `arbitre-web` |

- Un composant atteint hors de la table ⇒ `rouge`, avec la chaîne. `plan ↛* diff` en découle.
- Une crate du workspace absente de la table ⇒ `rouge` : il faut un amendement du socle pour l'y ajouter (échec fermé).

### A-10 : calcul des conflits (F-20)
Remplace la phrase de « Performance » : « nouvelle × portée ».
- Pour chaque écriture `w` d'une donnée, et pour chaque visée `v` exclusive avec la sienne :
  - on parcourt `journal.range((d, v), marque ∈ ]vu(w)[v], marque(w)])` ;
  - on garde les `x ≠ w` concurrentes, de valeur différente ;
  - on normalise la paire (retenue = rang max), puis on dédoublonne.
- **Complétude** : soit `x` de rang inférieur à `w`. `¬a_vu(w, x)` place `marque(x)` dans l'intervalle. Toute paire en conflit est donc vue depuis sa retenue.
- Coût : `O(log n + conflits)` par couple (écriture, visée). Le cas 8 de la matrice US-1 (10 000 écritures dans un seul lot) devient linéaire.
- **Sorties incrémentales** :
  - `nouveaux_conflits = conflits_de(d) après \ avant`, pour chaque donnée touchée par le lot ;
  - même chose pour les écartées et les anomalies.
  - C'est la définition de RG-2.2, sans raisonnement de monotonie, et ce même calcul sert à `Journal::conflits()`. Il n'y a qu'un seul algorithme, et la propriété « incrémental = complet » de la v1 disparaît.
- La projection se recalcule en entier (`ponytail:` déjà prévu par l'archi).

### A-11 : borne de `l` (F-10)
Complète `Marque` et `valider`.
- `L_MAX = 2^53`.
- `valider` refuse `marque.l ≥ L_MAX`, avec la raison `MarqueHorsBornes`. Les entrées de `vu` sont couvertes par `vu < marque`.
- `emettre` rend `HorlogeEpuisee` si `c` sature, **ou** si `l' ≥ L_MAX` : une seule erreur (renommage de `CompteurEpuise`).

## Analyse pré-plan

### Over-engineering détecté (et écarté)
- **Type `Anomalie` stocké, remplacement de contenu.** C'était la cause de F-4. Remplacé par un ensemble, plus une fonction de `S` (A-3).
- **Deux calculs de conflits (complet et incrémental) et une propriété pour les garder égaux.** Un seul calcul suffit (A-10).
- **Validation des clés de `vu`** et **refus d'une même marque sur deux cibles d'une donnée** (F-12) : écartés, inoffensifs ou acceptés (Q-5).
- **Simulateur réseau.** Il n'y a pas d'objet réseau. Un scénario est une suite d'actions `ecrire(i, …)`, `livrer(i → j, sous-ensemble, ordre)`, `relivrer`, puis tout est livré à tous.
- **Matrice US-1, cas 9** : il n'y a pas de test multi-processus avec `TZ` et `LANG`. Le cas est couvert par A-6 (motif `env::`) et par la propriété de déterminisme.
- **Profil de mutation à jeux réduits** : il n'est pas construit (F-17).

### Under-engineering détecté
- **Démonstration des secrets** : elle se fait en local seulement, sur une branche jetable jamais envoyée.
- **Démonstration d'arêtes** : `arbitre-plan` et `arbitre-diff` n'existent pas encore. On utilise des crates factices sur une branche jetable.
- **Format de cargo-mutants 27** : non vérifié. T-9 part d'une exécution réelle.
- **Check requis avant que le workflow existe** : une protection qui exige `ci` dès T-0 bloquerait toute PR de documentation jusqu'à la fusion de ST-01, puisque le check ne serait jamais rapporté. La protection se pose donc en deux temps : T-0 puis T-14 (Q-9).
- **Graine fixée** : les 1 000 jeux sont les mêmes à chaque exécution. L'exploration ne s'étend que si l'on change la graine, par une PR délibérée. C'est le prix de F-3.

### Risques identifiés

| # | Risque | Probabilité | Impact | Mitigation |
|---|---|---|---|---|
| R-1 | Score < 85 % sur `hlc.rs`, `write.rs` ou `merge.rs` (mutants équivalents dans les tris et les bornes) | Moyenne | K-7.5 non tenu, story bloquée | Tests ciblés sur les bornes (`<` contre `≤` dans les intervalles d'A-10, `σ` exact, saturation, `L_MAX`). Équivalents justifiés et relus par Quentin (T-14) |
| R-2 | Mutation longue en CI (propriétés à 1 000 jeux par mutant) | Moyenne | PR lente | Mesure en T-14. Premier levier `--jobs` (4 cœurs sur `ubuntu-latest`), puis `--shard` en matrice. Jamais moins de jeux (K-7.5, F-17) |
| R-3 | Le format de cargo-mutants 27 diffère de l'hypothèse | Moyenne | `mutation.py` faux | T-9 part d'une sortie réelle. Une sortie illisible ⇒ `rouge` |
| R-4 | La sortie de `cargo tree -e features` est fragile | Moyenne | Faux vert sur `chrono/clock` | Format `-f '{p} {f}'`. Démonstration 2 de T-12 |
| R-5 | G-1 inactive au premier envoi | Basse | RG-G9 | T-0 la fait confirmer avant tout `git push` |
| R-6 | Installation de cargo-mutants lente en CI | Haute | +3 à 5 min | Binaire précompilé, épinglé (`taiki-e/install-action` par SHA), cache Cargo |
| R-7 | Les générateurs produisent peu de cas durs | Moyenne | Propriétés vertes sans preuve | Petits domaines (`pt`, valeurs, 2 données, 2 champs), 3 à 5 appareils, test de couverture des familles d'E-1.1 |
| R-8 | `ProptestConfig::rng_seed` (`RngSeed::Fixed`) absent de proptest 1.12 | Basse | Pas de graine fixe | Vérifié au Skeleton. Repli : `TestRunner::new_with_rng(config, TestRng::from_seed(ChaCha, …))` dans `support/` |
| R-9 | `aws-lc-sys` (via `reqwest`/rustls) ne compile pas en local pendant la vérification ponctuelle de T-1 | Moyenne | T-1 bloquée sur un outil C | Relevé dans le rapport. C'est sans effet sur ST-01, puisqu'aucune crate n'hérite de `reqwest`. Bascule `ring` possible (arbitrage P-1) |
| R-10 | Délai d'attente des mutants dépendant de la machine (MANQUE-ST01-8) : un mutant lent peut être « détecté » en CI et « manqué » en local | Basse | Score différent entre local et CI | **Accepté** (reste de F-3). En pratique, seuls les mutants qui bouclent atteignent le délai. Écart constaté ⇒ `--timeout` explicite dans `50-st01-mutation.sh`, par PR `ci-change` |

## Tâches

Convention C-TDD pour les tâches Core / Full : le Skeleton pose d'abord les types et signatures (archi + amendements), puis les tests issus des matrices et des invariants, en échec. La tâche les fait passer. Chaque tâche cite les cas de matrice qu'elle rend verts.

### T-0 : Gestes préalables, envoi initial, protection de `main`
- **Zone DDD** : Generic
- **Réf specs** : C-6.6, MANQUE-ST01-6 ; cadre G-1 ; arbitrage Q-1 ; F-18
- **Dépendances** : aucune
- **Description** :
  1. Quentin confirme que G-1 est active (Settings → Code security → Push protection).
  2. `cargo install --locked cargo-mutants@27.1.0`, en fond.
  3. Sur le go de Quentin : `git push origin main`, pour envoyer les commits locaux antérieurs à la protection.
  4. Sur le go de Quentin, poser la protection : PR obligatoire (0 approbation), `enforce_admins: true`, ni force-push ni suppression, **sans check requis** pour l'instant (voir Under-engineering) :
     `gh api -X PUT repos/quentinhaenn/openarbitre/branches/main/protection --input -` avec `{"required_status_checks":null,"enforce_admins":true,"required_pull_request_reviews":{"required_approving_review_count":0},"restrictions":null,"allow_force_pushes":false,"allow_deletions":false}`.
  5. Créer le label `ci-change` (`gh label create ci-change`).
- **Livrable** : aucun fichier.
- **Critère de complétion** :
  - `cargo mutants --version` = `27.1.0` ;
  - `git status -sb` ne montre plus d'avance sur `origin/main` ;
  - `gh api …/branches/main/protection` répond avec `enforce_admins.enabled = true` ;
  - le label existe.
- **Effort estimé** : agent 5 min · arbitrage 15 min (G-1, deux go)

### T-1 : Assainir le workspace et déclarer les dépendances communes
- **Zone DDD** : Generic
- **Réf specs** : AC-5.1, AC-5.3, AC-5.4, AC-8.3 ; RG-ST01-5.1, 5.2, 8.2 ; C-5.3 ; F-6
- **Dépendances** : aucune
- **Description** :
  - `rust-toolchain.toml` : `channel = "1.99.1"`, composants `clippy` et `rustfmt`, cible `wasm32-unknown-unknown` ;
  - `Cargo.toml` racine : `rust-version = "1.99"` ; `[workspace.dependencies]` **selon le tableau d'A-5** ;
  - `crates/core/Cargo.toml` : aucune `[dependencies]` ; `[dev-dependencies] proptest = { workspace = true }` ; `[features] sim = []` ;
  - `crates/cli/Cargo.toml` : retrait de `chrono`, `clap` garde sa version locale ; `main.rs` = un parseur clap dérivé (`--version`, `--help`) ;
  - `core/src/lib.rs` vidé de `add` ;
  - `Cargo.lock` régénéré.
  - **Vérification ponctuelle (A-5)** :
    - créer `crates/zz-verif-deps` (bibliothèque vide), qui hérite des huit dépendances avec `workspace = true`, plus `std` pour `serde` ;
    - lancer `cargo check -p zz-verif-deps` ;
    - copier la sortie dans le rapport d'implémentation, puis supprimer la crate et restaurer `Cargo.lock`.
- **Livrable** : `rust-toolchain.toml`, `Cargo.toml`, `crates/{core,cli}/Cargo.toml`, `crates/core/src/lib.rs`, `crates/cli/src/main.rs`, `Cargo.lock`.
- **Critère de complétion** :
  - `cargo build --workspace --locked` vert ;
  - `cargo tree -p arbitre-core -e normal` n'affiche que `arbitre-core` ;
  - `cargo check -p arbitre-core --target wasm32-unknown-unknown` vert (matrice US-5, cas 1) ;
  - `grep -rn 'chrono\|rusqlite' crates/` vide ;
  - `arbitre --version` répond ;
  - la vérification ponctuelle est verte et consignée (matrice US-8, cas 1 et 2) ; si R-9 se produit, il est consigné et soumis à Quentin ;
  - `crates/zz-verif-deps` n'existe plus.
- **Effort estimé** : agent 10 min · arbitrage 0

### T-2 : Décision de dépendances et guide de contribution
- **Zone DDD** : Generic
- **Réf specs** : AC-8.1, AC-8.2, AC-8.4 ; C-8.1, C-8.3 ; MANQUE-ST01-4, 5 ; arbitrages P-1, F-9
- **Dépendances** : T-1
- **Description** :
  - **`docs/decisions/ST-01-dependances.md`** :
    - les huit sujets du tableau de l'archi, avec pour chacun le choix, la raison, l'option écartée et la déclaration d'A-5 ;
    - la justification des ajouts `jiff` et `getrandom` ;
    - la réversibilité `aws-lc-rs` → `ring`, et son piège (`install_default()`) ;
    - le rappel : une nouvelle dépendance commune est un amendement du socle.
  - **`CONTRIBUTING.md`** :
    - règle du `Cargo.lock` : prendre celui de `main`, puis `cargo check --workspace` ;
    - ajout d'un contrôle : `ci.d/NN-stxx-<sujet>.sh` avec NN ≥ 60, aides dans `scripts/lib/` ;
    - fichiers protégés et label `ci-change` (A-8) : posé par Quentin seul, PR sans `crates/` ;
    - formats de `zones.toml` (motifs `full_match`), de `mutants_equivalents.toml` et de `.gitleaksignore` ;
    - canaux d'exclusion interdits (A-7) ;
    - écriture locale par `Journal::ecrire`, `fusionner` réservé aux lots reçus (A-2) ;
    - montée de version d'un outil = amendement du socle.
- **Livrable** : `docs/decisions/ST-01-dependances.md`, `CONTRIBUTING.md`.
- **Critère de complétion** : relecture. Les six sujets d'AC-8.1 sont présents (matrice US-8, cas 3 et 4). Chaque version citée est égale à celle de `Cargo.toml`.
- **Effort estimé** : agent 10 min · arbitrage 10 min (relecture)

### T-3 : Marques, horloge hybride, avances, port `Clock` (`hlc.rs`, `clock.rs`)
- **Zone DDD** : Core (Full) pour `hlc.rs` · Supporting (Std) pour `clock.rs`
- **Réf specs** : RG-ST01-4.1, 4.2, 4.3, 4.4, 4.6 ; INV-ST01-4, 5, 6, 8 ; PROP-ST01-1 ; MANQUE-ST01-3 ; CONFLIT-ST01-3 ; F-10, F-15
- **Dépendances** : T-1
- **Description** :
  - **`hlc.rs`** :
    - `IdAppareil([u8; 16])`, avec un constructeur depuis `u128` ;
    - `Marque { l: u64, c: u32, a }` et `L_MAX` ;
    - `Horloge::nouvelle(a)` ;
    - `emettre(pt) -> Result<Marque, HorlogeEpuisee>`, selon RG-ST01-4.2. Il échoue si `c` sature ou si `l' ≥ L_MAX`, et laisse alors l'état inchangé (A-11) ;
    - `recevoir(&Marque)`, par `max_lex` ;
    - `Seuil(NonZeroU64)` et `SEUIL_DEFAUT` = 3 600 000 ms ;
    - `avances(lot, maintenant, σ)`.
  - **`clock.rs`** :
    - `trait Clock { fn now(&self) -> u64 }` ;
    - `HorlogeSimulee`, sous `cfg(feature = "sim")` : `fixer`, `avancer`, `reculer`, `figer`, avec un `Cell` interne.
- **Livrable** : `crates/core/src/{hlc,clock}.rs` ; propriétés dans `crates/core/tests/hlc.rs`.
- **Critère de complétion** :
  - matrice US-4, cas 1 à 10, verts. Le cas 1 utilise deux `HorlogeSimulee` décalées de 5 min et deux `Horloge` ;
  - propriétés à 1 000 jeux, avec la configuration à graine fixe de T-7 (posée ici dans `tests/support/`) :
    - INV-ST01-4 : ordre total et antisymétrique ;
    - INV-ST01-5 : pour des suites de `pt` croissantes, figées, décroissantes et mélangées, les émissions sont strictement croissantes et `l ≥ pt` ;
    - INV-ST01-6 : après `recevoir(m)`, toute émission est `> m` ;
    - INV-ST01-8 : seul ce qui dépasse `σ` est signalé ;
  - tests de bornes : `c = u32::MAX` et `pt` figé ⇒ `HorlogeEpuisee`, état inchangé ; `pt = L_MAX` ⇒ `HorlogeEpuisee` ; `pt = L_MAX − 1` ⇒ émission ;
  - `HorlogeSimulee` : `fixer`, `avancer`, `reculer` et `figer` testés ;
  - `cargo check --target wasm32-unknown-unknown -p arbitre-core --features sim` vert.
- **Effort estimé** : agent 20 min · arbitrage 0

### T-4 : Écritures et validation de lot (`write.rs`)
- **Zone DDD** : Core (Full)
- **Réf specs** : RG-ST01-4.5 (avec E-1, E-2) ; AC-4.5 ; INV-ST01-7 (validation) ; F-1, F-10
- **Dépendances** : T-3
- **Description** :
  - types `IdDonnee { module, genre, id }`, `Champ`, `Valeur(Vec<u8>)`, `Auteur` (4 variantes), `Quoi { Champ { nom, valeur }, Suppression }`, `Visee`, `Cible`, `Vu` et `Ecriture` (A-1) ;
  - méthodes `cible()` et `appareil()` (= `marque.a`) ;
  - `ErreurLot { index, raison }`, avec les raisons `VuNonAnterieur`, `IdentifiantVide` et `MarqueHorsBornes` ;
  - `valider(lot)` rend la première écriture fautive, dans l'ordre du lot.
- **Livrable** : `crates/core/src/write.rs`.
- **Critère de complétion** :
  - matrice US-4, cas 11 ;
  - une entrée de `vu` `= marque` ou `> marque` ⇒ `VuNonAnterieur`, y compris quand seule la deuxième entrée de la table fautive ;
  - module, genre, id ou champ vide ⇒ `IdentifiantVide` ;
  - `l = L_MAX` ⇒ `MarqueHorsBornes`, et `l = L_MAX − 1` passe ;
  - `index` exact ;
  - matrice US-4, cas 12 et 13 : garanties de type (E-1), avec un commentaire d'une ligne qui renvoie à l'archi.
- **Effort estimé** : agent 10 min · arbitrage 0

### T-5 : Journal, écriture locale, fusion, projection, écartées, anomalies (`merge.rs`, partie 1)
- **Zone DDD** : Core (Full)
- **Réf specs** : RG-2.1, RG-2.3, RG-2.4, RG-ST01-4.3 ; INV-4, 6, 7, INV-ST01-7 ; AMB-ST01-1 ; MANQUE-ST01-2 ; F-1, F-4, F-11, F-16
- **Dépendances** : T-4
- **Description** : A-2, A-3 et A-4, plus `vu_pour` (privée, A-1).
  - `Journal::nouveau`, `ecrire`, `fusionner` (tout ou rien, journal et horloge compris) ;
  - `projection()` suit RG-2.1 et RG-2.3 : `supprimée ⇔ t_s > t_f`, les gagnantes masquées restent présentes ;
  - `ecartees()` et `anomalies()` ;
  - le `Rapport` de `fusionner` rend `nouvelles_ecartees` et `nouvelles_anomalies` ; `nouveaux_conflits` reste vide jusqu'à T-6.
- **Livrable** : `crates/core/src/merge.rs`, `lib.rs` (`mod` et réexports).
- **Critère de complétion** :
  - matrice US-1, cas 1, 2, 3, 7, 8 (10 000 écritures séquentielles produites par `ecrire` : une gagnante, 9 999 écartées), 10 et 11 ;
  - matrice US-3, cas 1 à 9 (état et écartées seulement) ; matrice US-2, cas 9 ;
  - collision (US-1, cas 10) :
    - les deux contenus restent dans `S` ; le plus grand est retenu, l'autre écarté ;
    - une anomalie est rendue ;
    - une **seconde réception** de l'écriture perdante ne rend aucune nouvelle anomalie et laisse `R(S)` inchangé ;
  - `ecrire` :
    - `vu` = table attendue d'A-1, contrôlée sur une donnée à deux champs et une suppression ;
    - identifiant vide ⇒ refus, sans marque consommée (l'émission suivante a le `c` attendu) ;
  - `fusionner` d'un lot invalide ⇒ journal **et** horloge inchangés (INV-ST01-7). Après un lot valide, l'émission suivante est `>` la marque maximale du lot ;
  - propriétés à 1 000 jeux : partition `retenues ⊎ écartées = S` et monotonie (INV-7) ; lot invalide ⇒ journal inchangé.
- **Effort estimé** : agent 25 min · arbitrage 0

### T-6 : Conflits et rapports triés (`merge.rs`, partie 2)
- **Zone DDD** : Core (Full)
- **Réf specs** : RG-ST01-2.1, 2.2 ; INV-ST01-2, 3 ; AMB-ST01-2, 3 ; arbitrages « Même valeur », « Double suppression », F-1 ; F-5, F-12, F-20
- **Dépendances** : T-5
- **Description** :
  - `a_vu`, `exclusives` et `conflit` selon les specs, avec `a_vu` par visée (A-1) ;
  - calcul par intervalle et sorties incrémentales (A-10) ;
  - tri (A-4) ;
  - `Journal::conflits()`.
- **Livrable** : `crates/core/src/merge.rs`.
- **Critère de complétion** :
  - matrice US-2, cas 1 à 4, 6 à 8 et 10 ; colonnes « conflit » de la matrice US-3, cas 1 à 6 ;
  - **contre-exemple F-1**, en test nommé : B écrit `titre` ; A écrit `échéance`, puis supprime ⇒ un conflit `(d, titre, retenue = suppression, écartée = titre)` ;
  - **cas F-12**, en test nommé : un même appareil pose la même marque sur une suppression et sur un champ de `d` ⇒ `présente`, et un conflit (comportement de Q-5) ;
  - collision de valeurs différentes ⇒ un conflit (Q-3) ;
  - la seconde réception d'un lot rend trois listes vides ;
  - cas 8 de la matrice US-1 **en un seul lot reçu** : 10 000 écritures séquentielles, en moins de 2 s en debug ;
  - propriété : les trois listes de `Rapport` et `conflits()` sont triées selon A-4 (INV-ST01-3).
- **Effort estimé** : agent 20 min · arbitrage 5 min (cas de matrice ambigu à l'exécution)

### T-7 : Appareils simulés, oracle causal et propriétés de convergence
- **Zone DDD** : Core (Full)
- **Réf specs** : E-1.1 ; RG-2.2, RG-ST01-1.1, RG-ST01-2.1 ; INV-5, INV-ST01-1, INV-ST01-2 ; PROP-ST01-2 ; MANQUE-ST01-7 ; AC-1.6, AC-3.5 ; F-2, F-3
- **Dépendances** : T-6
- **Description** :
  - **Configuration** (`tests/support/mod.rs`) : une seule fonction `config()` rend `ProptestConfig { cases: 1000, rng_seed: RngSeed::Fixed(GRAINE), .. }` (R-8 pour le repli). Tous les tests de propriétés l'utilisent. `proptest-regressions/` est versionné. Changer `GRAINE` passe par une PR.
  - **Appareils simulés** :
    - un appareil = un `Journal`, une `HorlogeSimulee`, et un **registre de vérité** `connu: BTreeMap<identité, BTreeSet<identité>>`. Pour chaque écriture émise, il garde les identités présentes dans le journal de son auteur juste avant `ecrire` ;
    - un scénario = 3 à 5 appareils et une suite d'actions générées : `ecrire(i, donnée ∈ 2, visée ∈ {f1, f2, suppression}, valeur ∈ {a, b}, Δpt ∈ {−2…+2})`, `livrer(i → j, sous-ensemble, ordre)`, `relivrer`. À la fin, tout est livré à tous.
  - **Oracle (F-2)** : il calcule par force brute, sur toutes les paires de `S`, à partir du seul registre de vérité :
    - `a_vu*(w, x) ⇔ ∃ y ∈ connu(w), cible(y) = cible(x) ∧ marque(y) ≥ marque(x)` ;
    - `attendus(S) = { {w, x} | ¬a_vu*(w, x) ∧ ¬a_vu*(x, w) ∧ exclusives ∧ valeurs différentes }`, normalisés selon A-4.
    - C'est la règle arbitrée (« plus grande marque connue de l'auteur, cible par cible »), évaluée sur ce que l'appareil savait réellement, et non sur le champ `vu` transporté. L'égalité implique aussi la correction au sens direct : un conflit rendu ⇒ aucun des deux auteurs ne connaissait l'autre écriture.
  - **Propriétés** (1 000 jeux chacune) :
    - **concurrentes ⇔ conflit** : après **chaque** action, sur **chaque** appareil, `conflits()` = `attendus(S_appareil)` ; la concaténation des `nouveaux_conflits` d'un appareil = `conflits()` ;
    - convergence : en fin de scénario, `R(S)` est identique sur tous les appareils (PROP-ST01-2) ;
    - commutativité, associativité, idempotence et neutre sur des lots tirés des scénarios, avec suppressions (INV-5, AC-3.5) ;
    - déterminisme sous permutation de l'ordre interne d'un lot (INV-ST01-1) ;
    - couverture (R-7) : sur 1 000 scénarios de la graine, chaque famille d'E-1.1 apparaît au moins une fois (lot vide, doublon, ≥ 3 appareils, même `l`, même `(l, c)` sur deux appareils, suppression, concurrentes de même valeur, écrasement séquentiel, contre-exemple F-1).
- **Livrable** : `crates/core/tests/support/mod.rs`, `crates/core/tests/convergence.rs`.
- **Critère de complétion** :
  - `cargo test -p arbitre-core --all-features` vert, **deux fois de suite avec la même sortie** (graine fixe) ;
  - matrice US-1, cas 4, 5, 6 et 9 ; matrice US-2, cas 5 ; matrice US-3, cas 8 ;
  - vérifications ponctuelles, non versionnées, consignées au rapport : (i) `vu` remplacé par une marque scalaire (règle de la v1) ⇒ la propriété « concurrentes ⇔ conflit » échoue ; (ii) départage par appareil inversé ⇒ au moins une propriété échoue.
- **Effort estimé** : agent 30 min · arbitrage 0

### T-8 : Aide `graphe.py` (arêtes, interdites, communes, pureté)
- **Zone DDD** : Supporting (outillage ; mutation non mesurée, limite assumée par l'archi)
- **Réf specs** : RG-ST01-5.1, 5.2, 6.3 ; INV-ST01-9, 10, 13 ; AMB-ST01-6 ; AC-6.3, AC-6.4 ; F-7, F-14
- **Dépendances** : T-1. **Parallélisable avec T-3 à T-7** (aucun fichier Rust touché).
- **Description** : `scripts/lib/graphe.py`, Python ≥ 3.13, bibliothèque standard seule. Entrées : JSON de `cargo metadata`, sortie `cargo tree -f '{p} {f}'` par crate de la liste navigateur, manifestes lus par `tomllib`. Règles :
  - (a) table d'autorisations (A-9) ;
  - (b) dépendances interdites dans le graphe normal de la liste navigateur (archi, inchangé) ;
  - (c) pureté (A-6) ;
  - (d) ni `openssl-sys` ni `native-tls` dans le workspace ;
  - (e) dépendances communes reprises par `workspace = true`.

  Chaque échec rend une ligne qui nomme les composants, la dépendance ou la chaîne.
- **Livrable** : `scripts/lib/graphe.py`, sa partie de `scripts/lib/test_ci.py`, données figées sous `scripts/lib/fixtures/`.
- **Critère de complétion** : `python3 -m unittest scripts/lib/test_ci.py` vert, avec, sur données figées :
  - matrice US-5, cas 2 à 5, et matrice US-6, cas 5 à 7 ;
  - `calendar → plan` accepté, `relay → plan` refusé ;
  - crate inconnue ⇒ `rouge` ;
  - `UNIX_EPOCH.elapsed()` et `std::env::var` dans `clock.rs` ⇒ `rouge`, mais `fn now(&self)` accepté.
- **Effort estimé** : agent 15 min · arbitrage 0

### T-9 : Aide `mutation.py` (zones, équivalents, scores, rapport, zones abaissées)
- **Zone DDD** : Supporting (outillage)
- **Réf specs** : RG-ST01-7.1, 7.2, 7.3 ; INV-ST01-15, 16 ; AMB-ST01-4 ; MANQUE-ST01-1, 5, 8 ; C-7.4 ; AC-7.1 à 7.4, AC-7.6 ; F-9
- **Dépendances** : T-0 (cargo-mutants installé). **Parallélisable avec T-3 à T-8.**
- **Description** :
  - **Relevé du format réel** (R-3) : `cargo mutants` 27.1.0 sur un mini-crate jetable sous `$TMPDIR`. On en tire les données figées : structure de `outcomes.json`, noms des issues, forme du nom de mutant, issue quand la base est rouge.
  - **Fonctions** :
    - motifs par `PurePosixPath.full_match` ;
    - lecture de tous les `.ctdd/stories/*/zones.toml` (clé inconnue ou TOML invalide ⇒ erreur qui nomme le fichier) ;
    - zone la plus exigeante, `Full` par défaut ; `Gen` dans `crates/core/` ou `crates/plan/` ⇒ `rouge` ;
    - équivalents lus à l'identique, justification vide ⇒ `rouge` ;
    - correspondance des issues (`caught`/`timeout` = détecté, `unviable` exclu, base rouge ⇒ `rouge`) ;
    - score par fichier, neutre si rien n'est compté, `≥` seuil pour passer ; `Gen` informatif ;
    - rapport Markdown trié par fichier, **sans durée ni horodatage** (comparé à l'octet près en T-14) ;
    - `zones_abaissees(base)` (A-8) : lit les `zones.toml` de la base par `git show`.
- **Livrable** : `scripts/lib/mutation.py`, sa partie de `test_ci.py`, `scripts/lib/fixtures/mutants/`.
- **Critère de complétion** : `test_ci.py` vert, avec :
  - matrice US-7, cas 1 à 11 ;
  - cinq exemples de motifs : `*` ne traverse pas `/` ; `**` couvre 0 et n segments ; motif exact ; motif sans correspondance ;
  - `zones_abaissees` : motif retiré ⇒ signalé ; `Full` → `Std` ⇒ signalé ; `Std` → `Full` et motif ajouté ⇒ rien ; fichier supprimé ⇒ signalé ;
  - une sortie illisible ⇒ `rouge`.
- **Effort estimé** : agent 20 min · arbitrage 0

### T-10 : Exécuteur `ci.sh`, versions épinglées et les six contrôles
- **Zone DDD** : Generic
- **Réf specs** : RG-ST01-6.1, 6.2, 6.4, 6.5 ; INV-ST01-11, 12, 14 ; CONFLIT-ST01-2 (E-3) ; MANQUE-ST01-4, 6 ; AMB-ST01-5 ; F-8, F-9
- **Dépendances** : T-8, T-9
- **Description** :
  - **`scripts/outils.env`** : `CARGO_MUTANTS_VERSION=27.1.0`, `GITLEAKS_VERSION=8.30.1`, `PYTHON_MIN=3.13`.
  - **`scripts/ci.sh`**, conforme à l'archi :
    - versions comparées à l'égalité exacte, sauf Python (minimum), avec la commande d'installation dans le message et le cas du Python 3.9 d'Xcode nommé ;
    - base = `$CI_BASE`, sinon `git merge-base HEAD origin/main` ;
    - exports `CI_BASE`, `CI_HEAD` et `CI_OUT=target/ci` ;
    - chaque `ci.d/*.sh` exécuté par `bash`, triés avec `LC_ALL=C` ; tous sont exécutés ; `ci.d/` vide ⇒ `rouge` ;
    - sous Actions : `::group::`, `::error::` et `$GITHUB_STEP_SUMMARY`.
  - **`ci.d/`** :
    - `05-st01-juge.sh` (A-8) ;
    - `10-st01-tests.sh` (`cargo test --workspace --all-features --locked`, plus `unittest`) ;
    - `20-st01-navigateur.sh` ;
    - `30-st01-graphe.sh` ;
    - `40-st01-secrets.sh` ;
    - `50-st01-mutation.sh` (A-7 pour les canaux d'exclusion).
  - `.gitleaksignore` vide.
- **Livrable** : `scripts/ci.sh`, `scripts/outils.env`, `scripts/ci.d/{05,10,20,30,40,50}-st01-*.sh`, `.gitleaksignore`.
- **Critère de complétion** :
  - `scripts/ci.sh` vert en local sur la branche de ST-01 une fois T-7 fait (avant, seul `50` peut être rouge). `05` est vert, puisque la base n'a aucun fichier protégé ;
  - matrice US-6, cas 4, 9 (PR documentaire : vert, mutation neutre), 10 (`ci.d/60-demo.sh` avec `exit 1` : exécuté, nommé, `rouge`, puis retiré) et 11 (cargo-mutants hors du `PATH` ⇒ `rouge` explicite) ;
  - `CONTRIBUTING.md` contient `mutants::skip` et `exclude_re` sans faire rougir `50` (F-8).
- **Effort estimé** : agent 20 min · arbitrage 0

### T-11 : Workflow GitHub Actions
- **Zone DDD** : Generic
- **Réf specs** : C-6.6 ; RG-ST01-6.1 ; C-7.4 ; arbitrages Q-1, F-9
- **Dépendances** : T-10, T-0
- **Description** : `.github/workflows/ci.yml`.
  - Un job nommé **`ci`** (nom du check requis en T-14), sur `ubuntu-latest`.
  - Déclencheur : `pull_request` (types d'A-8).
  - `permissions: contents: read`.
  - Étapes :
    - `checkout` avec `fetch-depth: 0` ;
    - toolchain lue dans `rust-toolchain.toml` ;
    - `setup-python` 3.13 ;
    - cargo-mutants et gitleaks en binaires précompilés, aux versions d'`outils.env` ;
    - cache Cargo ;
    - `CI_BASE=${{ github.event.pull_request.base.sha }} CI_LABELS=${{ join(github.event.pull_request.labels.*.name, ',') }} scripts/ci.sh` ;
    - `target/ci/` publié en artefact, `if: always()`.
  - Actions épinglées par SHA.
- **Livrable** : `.github/workflows/ci.yml`.
- **Critère de complétion** : vérifié en T-14. Pas de tag dans les `uses:` (`grep -E 'uses: .*@v[0-9]'` vide).
- **Effort estimé** : agent 10 min · arbitrage 0

### T-12 : Démonstrations rouges (branches jetables, jamais envoyées)
- **Zone DDD** : Generic
- **Réf specs** : AC-6.4, AC-6.5 ; matrice US-5, cas 2 et 3 ; matrice US-6, cas 2, 3, 5, 6, 8 ; matrice US-7, cas 1 ; arbitrage F-9 ; F-19
- **Dépendances** : T-10
- **Description** : sur des branches locales jetables, lancer `scripts/ci.sh` (ou le contrôle seul) après chaque modification ci-dessous, puis copier la sortie dans le rapport d'implémentation.
  0. un test `assert!(false)` dans le cœur (US-6, cas 2) ;
  1. `rusqlite` ajouté au cœur ;
  2. `chrono` avec `clock` ajouté au cœur ;
  3. crates factices `arbitre-plan` → `arbitre-diff`, en direct puis par `arbitre-sqlite` factice ;
  4. `reqwest` ajouté au cœur ;
  5. un faux jeton commité puis retiré au commit suivant ;
  6. un test de `hlc.rs` supprimé, qui fait passer le score sous 85 % ;
  7. **juge** : seuil modifié dans `mutation.py` ⇒ `rouge` qui nomme le fichier ;
  8. **juge** : `merge.rs` passé de `Full` à `Std` dans `ST-01/zones.toml` ⇒ `rouge` ;
  9. **juge** : 7 avec `CI_LABELS=ci-change` ⇒ `05` vert. Puis 7, plus une ligne modifiée dans `crates/`, avec le label ⇒ `rouge`.

  Pour 7 à 9, la base est un commit local qui contient déjà les fichiers de ST-01 (`CI_BASE=<ce commit>`).
- **Livrable** : section « Démonstrations » du rapport de la phase 7. Aucun fichier dans le dépôt.
- **Critère de complétion** : les démonstrations 0 à 8 et le second cas de 9 sont rouges, et le premier cas de 9 est vert. Chaque message nomme ce que la spec exige (composants, chaîne, commit, fichier/score/seuil, fichiers protégés). Les branches sont supprimées.
- **Effort estimé** : agent 15 min · arbitrage 0

### T-13 : Passe de simplification (arbitrage de l'archi)
- **Zone DDD** : transverse
- **Réf specs** : arbitrage « Passe de simplification » de `03_archi.md`
- **Dépendances** : T-2, T-7, T-11, T-12
- **Description** :
  - Lancer `/ponytail-review` (ou `/simplify`) sur le diff de la story, `git diff main...HEAD`.
  - Appliquer ce qui retire du code sans toucher à un contrat (types publics, formats, noms de fichiers de `ci.d/`). Les propositions qui touchent un contrat vont à Quentin.
- **Livrable** : commit « simplification » sur la branche de ST-01 ; liste des propositions retenues et écartées dans le rapport.
- **Critère de complétion** : `scripts/ci.sh` vert en local après la passe ; chaque proposition écartée a une raison d'une ligne.
- **Effort estimé** : agent 10 min · arbitrage 5 min

### T-14 : PR de ST-01, mutation sur elle-même, check requis
- **Zone DDD** : Core (critère de complétion de la story)
- **Réf specs** : K-7.5, AC-7.5 ; RG-ST01-6.1 ; matrice US-6, cas 1 et 12 ; matrice US-7, cas 12 ; F-3, F-18
- **Dépendances** : T-13
- **Description** :
  - Rebaser sur `main`, envoyer la branche et ouvrir la PR. Relever la durée de la mutation (R-2).
  - Pour chaque mutant survivant de `hlc.rs`, `write.rs` ou `merge.rs` : écrire le test qui le tue ou, s'il est équivalent, une entrée justifiée dans `.ctdd/stories/ST-01/mutants_equivalents.toml`.
  - **Comparaison local / CI** (US-6, cas 12) : `scripts/ci.sh` en local sur la même tête, avec `CI_BASE` = la base de la PR. On compare le verdict de chaque contrôle et `target/ci/mutation.md` à l'artefact de la CI, à l'octet près.
  - Une fois le check `ci` rapporté sur la PR, et sur le go de Quentin : `gh api -X PATCH …/branches/main/protection/required_status_checks -f strict=true -f 'contexts[]=ci'`.
- **Livrable** : PR verte ; `mutants_equivalents.toml` si besoin ; check `ci` requis sur `main`.
- **Critère de complétion** :
  - CI verte sur la PR (US-6, cas 1) ;
  - `mutation.md` ≥ 85 % pour `hlc.rs`, `write.rs` et `merge.rs`, et ≥ 70 % pour `clock.rs` (K-7.5) ;
  - chaque équivalent relu par Quentin ;
  - verdicts et `mutation.md` identiques entre local et CI. Un écart dû à un délai d'attente relève de R-10 et va à Quentin ;
  - la protection liste `ci` en check requis, avec `strict: true`.
- **Effort estimé** : agent 30 min (mutation comprise) · arbitrage 20 min (équivalents, durée de CI, go protection)

## Ordre d'exécution

```
T-0 ──────────────────────────────────────────────────────────┐
T-1 ─┬─► T-2 ─────────────────────────────────────────────────┤
     ├─► T-3 ─► T-4 ─► T-5 ─► T-6 ─► T-7 ─────────────────────┤
     ├─► T-8 ──────────┐                                      ├─► T-13 ─► T-14
     └── (T-0) ─► T-9 ─┴─► T-10 ─┬─► T-11 ────────────────────┤
                                 └─► T-12 ────────────────────┘
```

- **Deux voies après T-1** : le moteur (T-3 à T-7) et l'outillage (T-8 à T-12). Elles ne partagent aucun fichier. T-8 et T-9 sont parallélisables entre elles.
- T-13 attend les deux voies : la passe porte sur le diff complet.
- Les envois sur GitHub n'ont lieu qu'en T-0 (`main` + protection) et en T-14 (branche + PR + check requis), toujours sur le go de Quentin.

### Estimation (temps d'agent et temps d'arbitrage)

| Voie | Tâches | Agent (min) | Arbitrage (min) |
|---|---|---|---|
| Préalable | T-0, T-1, T-2 | 25 | 25 |
| Moteur | T-3 → T-7 | 105 | 5 |
| Outillage | T-8 ∥ T-9 → T-10 → T-11, T-12 | 80 (≈ 65 en durée) | 0 |
| Clôture | T-13, T-14 | 40 | 25 |
| **Total** | | **250 cumulés, ≈ 180 en durée** (le moteur est le chemin critique) | **≈ 55** |

- Périmètre : phases 6 à 8. Les itérations DA ne sont pas comptées.
- Par rapport à la v1 : +35 min d'agent, pour l'oracle, le juge, la passe de simplification et la vérification des dépendances.
- Écart au cadre : M (100 min machine) + forte (60 min d'arbitrage). La durée machine dépasse M d'environ 80 %. L'arbitrage tient dans « forte ». On relève ces données dans `state.json`, sans les corriger.

## Invariants C-TDD à vérifier post-implémentation

| Invariant / Property | Tâche(s) | Vérification |
|---|---|---|
| INV-4 (gagnante par champ, champs indépendants) | T-5 | Matrice US-1, cas 1 à 3 et 8 |
| INV-5 (convergence) | T-7 | Commutativité, associativité, idempotence, neutre, avec suppressions ; scénarios multi-appareils |
| INV-6 (suppression datée, résurrection) | T-5, T-6 | Matrice US-3, cas 1 à 9 ; cas F-12 |
| INV-7 (partition, monotonie) | T-5 | Propriété ; collision gardée dans `S` (F-4) |
| INV-ST01-1 (déterminisme) | T-7, T-8 | Permutation des lots ; graine fixe ; pureté A-6 |
| INV-ST01-2 (définition du conflit) | T-6, T-7 | Matrices US-2 et US-3 ; contre-exemple F-1 ; **oracle causal, égalité dans les deux sens** |
| INV-ST01-3 (rapports triés) | T-6 | Propriété de tri (A-4) |
| INV-ST01-4, 5, 6 (ordre, émission, réception) | T-3 | Propriétés sur des suites d'heures quelconques ; bornes `L_MAX` |
| INV-ST01-7 (lot invalide refusé en entier) | T-4, T-5 | Journal et horloge inchangés |
| INV-ST01-8 (avances) | T-3 | Seuil exact non signalé |
| INV-ST01-9 (cœur sans stockage ni heure) | T-1, T-8, T-12 | `cargo tree` ; A-6 ; démonstrations 1 et 2 |
| INV-ST01-10 (dépendances communes) | T-1, T-8 | Règle (e) ; vérification ponctuelle (A-5) |
| INV-ST01-11 (verdict unique) | T-10, T-14 | Matrice US-6, cas 9, 10, 11 et 12 (comparaison local / CI) |
| INV-ST01-12 (liste navigateur) | T-10 | Matrice US-6, cas 4 |
| INV-ST01-13 (arêtes interdites) | T-8, T-12 | Table A-9 ; démonstration 3 |
| INV-ST01-14 (secrets par commit) | T-10, T-12 | Démonstration 5 |
| INV-ST01-15 (zone la plus exigeante) | T-9, T-10 | Matrice US-7, cas 3, 4, 10 ; zones abaissées (A-8) |
| INV-ST01-16 (seuil bloquant, équivalents justifiés) | T-9, T-12, T-14 | Matrice US-7 ; démonstrations 6 à 8 ; PR de ST-01 |
| PROP-ST01-1 (horloge simulée) | T-3 | Matrice US-4, cas 1 à 4 |
| PROP-ST01-2 (N appareils) | T-7 | Scénarios générés, 3 à 5 appareils, oracle causal |
| K-7.5 (≥ 85 % sur moteur et horloge) | T-14 | `mutation.md` de la PR |

## Arbitrages hérités du plan v1 (2026-11-03, font foi)

- **Q-1 → PR seulement, `main` protégée sans exception** (check CI requis, pas de contournement admin). Toute modification, y compris la doc `.ctdd/`, passe par une PR. Raison : travail parallèle sur plusieurs branches, relecture dans l'interface GitHub, discipline voulue même en solo. Les commits locaux antérieurs à la protection sont envoyés avant qu'elle soit activée.
- **Q-2 → Python ≥ 3.13, `PurePosixPath.full_match`.** Pas de traducteur glob → regex maison. Remplace le minimum 3.11 de l'arbitrage P-3 de l'archi : `outils.env` déclare 3.13 et `ci.sh` le vérifie avec un message explicite.
- **DA itération 1, F-9 → contrôle « juge touché ».** Un fichier `ci.d/` rend `rouge` toute PR qui modifie `ci.yml`, `ci.sh`, `ci.d/*-st01-*`, `outils.env`, les scripts de seuil, ou qui retire ou abaisse une entrée d'un `zones.toml` existant ; il liste les fichiers en cause. Ajouter une zone plus exigeante reste libre. Une modification légitime de la CI passe par une PR dédiée portant le label `ci-change`, posé par Quentin seul. Garde-fou contre la dérive d'un agent, pas contre la malveillance (la relecture de Quentin couvre ce cas).

## Reste à arbitrer (Quentin)

Chaque recommandation est retenue par défaut si Quentin ne dit rien.

| # | Question | Recommandation |
|---|---|---|
| Q-3 | **A-2 et A-3, changements de forme.** (1) Le `Journal` porte l'horloge de l'appareil, avec `ecrire` et `fusionner` comme seuls chemins, ce qui ferme F-11 et F-16. (2) Une collision d'identité garde les deux contenus dans `S`, et l'anomalie devient une fonction de `S`. Conséquence de la règle : si les deux valeurs diffèrent, la collision est **aussi** un conflit. D'accord ? | **Oui aux deux**, conflit compris. Il n'y a aucune exception à coder, et l'utilisateur voit les deux valeurs. |
| Q-4 | **F-10 (1)** : un pair corrompu ou hostile envoie `(l futur, c = u32::MAX)`. L'appareil ne peut plus écrire (`HorlogeEpuisee`) tant que son heure n'a pas dépassé ce `l`. Au-delà de `σ`, la marque est signalée par `avances`. Accepter ? | **Accepter**, et le documenter dans la décision. L'alternative, « à saturation, `(l + 1, 0)` », modifie RG-ST01-4.2 et l'arbitrage MANQUE-ST01-3, et bute de toute façon sur `L_MAX`. |
| Q-5 | **F-12** : même marque sur une suppression et sur un champ de la même donnée (impossible par `ecrire`, possible seulement dans un lot forgé). Le comportement est fixé et testé : `présente`, plus un conflit. Accepter plutôt que valider ? | **Accepter.** La valeur reste visible, rien n'est perdu, et une validation inter-écritures (lot × journal) coûte plus que la faille. |
| Q-6 | **F-11, reste** : `fusionner` accepte un lot construit à la main, avec des `vu` maximaux. Il le faut pour la désérialisation de ST-02. Accepter, avec la règle dans `CONTRIBUTING.md` et la doc de `fusionner` ? | **Accepter.** Un `vu` reçu n'est de toute façon pas vérifiable. |
| Q-7 | **A-9** : `arbitre-web` est autorisée vers `arbitre-sqlite`, pour le stockage navigateur si S1-B retient sqlite-wasm. Le graphe du cadre dit seulement « stockage navigateur ». | **Autoriser maintenant.** Sinon, ce sera un amendement d'un fichier protégé si S1-B retient cette voie. |
| Q-8 | **Restes du juge (F-9)** : (1) `mutants_equivalents.toml` n'est pas protégé, puisque ses entrées accompagnent forcément le code de la PR. Il reste couvert par la relecture de Quentin. (2) Un agent qui agit avec le jeton de Quentin peut poser `ci-change` lui-même. Accepter ces deux restes, dans le périmètre de l'arbitrage (dérive, pas malveillance) ? | **Accepter** les deux. Les agents reçoivent la consigne écrite de ne jamais poser ce label. |
| Q-9 | **Protection en deux temps** : en T-0, PR obligatoire et pas de contournement admin ; en T-14, le check `ci` devient requis, une fois qu'il a été rapporté. Sinon, les PR de documentation des phases 5 à 8 resteraient bloquées jusqu'à la fusion de ST-01. | **Oui.** |
| Q-10 | **R-10 (reste de F-3)** : le délai d'attente des mutants dépend de la machine. Accepter, avec un `--timeout` explicite seulement si un écart est constaté en T-14 ? | **Accepter.** |

## Arbitrages de Quentin sur le plan v2 (2026-11-03)

- **Q-3 → oui aux deux** : le `Journal` porte l'horloge (`ecrire` et `fusionner` seuls chemins) ; une collision d'identité garde tous les contenus, l'anomalie est une fonction de `S`, et deux contenus de valeurs différentes forment aussi un conflit.
- **Q-4 à Q-10 → validés en bloc**, selon les recommandations du tableau ci-dessus.

## Arbitrages de Quentin sur le DA v2 (2026-11-03)

- **DA v2 F-1 → juge séparé, exécuté depuis `main`.** Workflow `juge.yml` sur `pull_request_target` (types incluant `labeled`/`unlabeled`) : workflow et scripts pris sur la base, **aucun code de la PR exécuté** ; il lit la liste des fichiers modifiés et les `zones.toml` de la tête comme données. Protège tout `.github/` et tout `scripts/`, ajouts compris, et la zone effective de chaque fichier existant (rouge si elle baisse entre base et tête, ce qui ferme aussi DA v2 F-2). Toute modification protégée exige le label `ci-change`, posé par Quentin seul. `juge` est un check requis à côté de `ci`. Remplace `05-st01-juge.sh` d'A-8. Écart accepté à AC-6.6 : ajouter une vérification dans `ci.d/` passe par le label.
- **Suite** : plan v3 léger (DA v2 F-1 à F-3 et mineurs F-4 à F-10, sans toucher au moteur), puis Skeleton sans itération DA 3, conformément à la recommandation du DA v2.
