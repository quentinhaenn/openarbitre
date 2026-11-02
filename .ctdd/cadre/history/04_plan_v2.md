# Plan d'implémentation — OpenArbitre, cadre global (découpe en stories) — v2

## Traçabilité
- Sources : `.ctdd/cadre/02_specs.md`, `.ctdd/cadre/03_archi.md`, et les sections « Arbitrages de Quentin » de `01_story.md`, `02_specs.md`, `03_archi.md` et `05_da_review.md` (F-10), qui font foi. `.ctdd/cadre/inputs/plan-decision-diff-2026-10-02.md` sert de contexte pour la partie C.
- Version précédente : `.ctdd/cadre/history/04_plan_v1.md`, bloquée par `05_da_review.md` (itération 1/3).
- Date : 2026-11-02 · Statut : DRAFT v2 · Cycle sans code.
- Ce plan **est** la découpe en stories. Chaque story `ST-xx` aura son propre cycle C-TDD complet (story → specs → archi locale → plan → DA → skeleton → implémentation → review). Les `[À DÉFINIR]`, `AMB-*` et `RENVOI-*` d'une story se tranchent pendant son cycle.
- **Identifiants stables** : les numéros de la v1 sont conservés pour que la revue reste lisible. Ajouts : ST-04b (export et import, sortis de ST-03), ST-15a et ST-15b (ST-15 coupée en deux), S1-A et S1-B (S1 coupé en deux), S6 (nouveau spike), G-3 (nouveau geste).

## Résolution des failles de la revue

| Faille | Résolution en une ligne |
|---|---|
| **F-1** (B) | Trois voies nommées après ST-04 (planning, mobile et socle étendu, diff). Les collisions sont listées dans un tableau qui désigne un propriétaire et une règle de fusion pour chacune. Le plafond est de 2 voies en cycle et de 8 points d'arbitrage par soirée (§ Parallélisme). |
| **F-2** (B) | Les tailles sont exprimées en soirées (S = ⅓, M = ⅔, L = 1), avec une charge d'arbitrage en points par story. Un ordonnancement soirée par soirée donne des jalons datés à 3 et à 2 soirées par semaine. Le chiffre de PL-1 est retiré. |
| **F-3** (B) | ST-17 (CalDAV) ne dépend plus que de ST-09 et de ST-15a, qui sont des fonctions pures sur simulateur. Le repli s'exécute sans rien de Google. |
| **F-4** | ST-11, ST-12, ST-13, ST-14, ST-15a et ST-16 tournent sur le simulateur d'`Agenda`. Google n'apparaît que dans deux feuilles du graphe, ST-10 (lecture) et ST-15b (écriture). |
| **F-5** | ST-07 (MCP) dépend de ST-04 et de S5, sans attendre la pile. ST-27 et ST-32 déclarent l'arête ST-07. |
| **F-6** | ST-03 fixe la table **complète** des 5 catégories et de leurs codes (succès, refus ou violation, usage, erreur d'outil, module non activé). ST-04, ST-22 et ST-24 l'utilisent sans la modifier. La table est gelée : la changer est un amendement du socle. |
| **F-7** | ST-19 dépend de ST-18, ST-04 (crochet `revalider`) et S6. La convergence est prouvée sur le module factice. La question sur un élément invalide reste dans ST-13. |
| **F-8** | Les spikes sont placés par voie et par soirée. S1-A rend le verdict synchrone ou asynchrone avant le gel du port `Store` en ST-02, et mesure le poids des fuseaux dès la 2e soirée. S4 passe avant la voie diff, S5 dans le socle. |
| **F-9** | La démo ST-22 tombe au plus tôt le 2026-11-10, au plus tard le 2026-11-12, très en amont de la fenêtre Team (mi-2027). Le garde-fou des 10 entretiens n'est pas touché. |
| **F-10** | L'arbitrage de Quentin est appliqué : la voie diff tourne en parallèle dès ST-04, et le planning passe d'abord en cas de conflit de capacité (règle de priorité, § Parallélisme). |
| **F-11** | ST-03 est réduite au catalogue, à la CLI et aux codes de retour. L'export et l'import passent en ST-04b, après les espaces de noms ; INV-24 y est rattaché. |
| **F-12** | Le test de conformité générique de ST-04 exécute toute action non réseau de tout module réseau coupé (PROP-1). Il est rejoué automatiquement à chaque action ajoutée. AC-1.1 est couvert. |
| **F-13** | Modèle par défaut fixé en ST-18 : la même phrase secrète est saisie une fois par appareil, avec un sel public sur le relais et une clé dérivée gardée au trousseau. Le cas « sans session » est validé par S6 puis vérifié en ST-19. Le cas de la PWA est marqué `[À DÉFINIR]` en ST-20. |
| **F-14** | Le spike S6 (trousseau depuis launchd et systemd) passe avant ST-11. ST-11 reçoit un critère : jeton inaccessible ou expiré ⇒ recalcul sans lecture, capacité « non à jour », signal visible à l'ouverture. |
| **F-15** | ST-31 ne se déclenche que par une commande locale explicite et jamais dans l'Action. L'avertissement de CONFLIT-16 et la vérification PROP-2 y sont ajoutés. |
| **F-16** | ST-01 pose cargo-mutants : un job CI `--in-diff` et un script de seuil. Chaque review de story vérifie le seuil de sa zone sur les fichiers qu'elle modifie. |
| **F-17** | ST-01 livre la décision `docs/decisions/ST-01-dependances.md`, qui choisit runtime, client HTTP, trousseau, PBT et mutation, et les déclare dans `[workspace.dependencies]` (PL-6). |
| F-18 (m) | L'observateur réseau passe au socle (ST-04), avant toute sortie réseau. |
| F-19 (m) | ST-25 dépend de ST-23, et non plus de ST-24. |
| F-20 (m) | ST-02, ST-03, ST-18 et ST-30 sont marquées « sans valeur seule ». Aucun jalon n'est posé sur elles. |
| F-21 (m) | ST-21 reçoit un critère : écritures concurrentes de la CLI et de la PWA sur un même élément ⇒ convergence par la synchro et conflit signalé. |
| F-22 (m) | PL-3 : la PWA est servie depuis un répertoire (`--pwa <dir>`), pas embarquée dans le binaire, donc sans couplage de build. Le relais reçoit tout le binaire `arbitre` : c'est accepté en v1. |
| F-23 (m) | PL-5 est retiré. Les critères de ST-22, ST-24 et ST-32 citent CONFLIT-17 et son effet sur un dépôt qui a déjà des violations. |

## Contexte technique
- **Codebase** : workspace Cargo vide, sans aucun commit. On y trouve `crates/core` (`arbitre-core`, fonction `add`) et `crates/cli` (`arbitre`, « Hello, world! »). Les anti-patterns relevés dans `03_archi.md` sont toujours là : `.git` imbriqué, `rusqlite` et `chrono/clock` dans le cœur.
- **Stack** : Rust (édition 2024), SQLite via rusqlite, Leptos CSR en PWA (wasm32), clap, MCP (`rmcp` selon S5), tree-sitter-python ou le parseur de Ruff (selon S4), RustCrypto (XChaCha20-Poly1305, Argon2id). Dépendances partagées : voir PL-6.
- **Patterns à respecter** (instaurés par l'archi) : journal d'écritures et projection, cœur fonctionnel dans une coquille impérative, ports et adaptateurs avec un simulateur par port, catalogue d'actions comme source unique des surfaces, identifiants déterministes, rang par indexation fractionnaire, réconciliation déclarative, rapports triés.
- **Garde-fous mécaniques** posés dès ST-01 :
  - `cargo check --target wasm32-unknown-unknown` sur `arbitre-core` et `arbitre-plan` ;
  - contrôle des arêtes interdites sur `cargo metadata` ;
  - gitleaks ;
  - cargo-mutants `--in-diff`.

## Analyse pré-plan

### Over-engineering détecté
- **Durées améliorées par l'IA** (RG-4.4 la cite comme option) : aucune story. La proposition déterministe (ST-05, ST-08) suffit à AC-7.1 à 7.5.
- **CalDAV** (ST-17) : la story est différable. Elle s'engage à la demande d'un utilisateur réel ou si S3 conclut au repli, et elle ne dépend plus de Google (F-3).
- **Crate `arbitre-relay`** : le relais devient un module de `arbitre` (cli), extrait le jour où une autre surface en a besoin (PL-3). La frontière tient par le type `Enveloppe`.
- **Simulation fine du calendrier** : l'ordonnancement soirée par soirée est indicatif. Il sera **remplacé** par les durées mesurées au recalibrage qui suit ST-04. Il ne faut pas le maintenir à la main au-delà.

### Under-engineering détecté
- **`.git` imbriqué dans `crates/core`** : à supprimer avant tout commit (G-0).
- **`rusqlite` et `chrono/clock` dans le cœur** : retirés en ST-01, puis déplacés vers `arbitre-sqlite` en ST-02.
- **Port `Store` synchrone contre stockage navigateur** (nouveau) : OPFS passe par un worker, et IndexedDB est asynchrone. Un port `Store` synchrone gelé en ST-02 pourrait ne pas être implémentable côté wasm, et le repli IndexedDB de l'archi tomberait avec lui. **S1-A tranche ce point avant que ST-02 ne gèle le port.**
- **Un seul fichier SQLite pour la CLI, le MCP et le recalcul** : WAL et délai d'attente sur verrou. ST-02 le vérifie avec deux processus réels.
- **Artefacts C-TDD uniques** (nouveau) : `.ctdd/cadre/01_story.md` à `08_*.md` et `state.json` entreraient en collision entre deux cycles parallèles. Chaque story mène donc son cycle dans `.ctdd/stories/ST-xx/` (G-4).
- **Numérotation des décisions** (nouveau) : `docs/decisions/0001-…` entrerait en collision entre voies. Les décisions sont nommées par story (`ST-xx-<sujet>.md`).
- **Protection contre l'envoi de secrets côté GitHub** : à activer avant le premier push (G-1).

### Risques identifiés

| # | Risque | Probabilité | Impact | Mitigation |
|---|--------|-------------|--------|------------|
| R-1 | Google refuse ou bride le client OAuth personnel (admin Workspace, utilisateurs non vérifiés) | Moyenne | ST-10 et ST-15b bloquées | S3 est lancé à la 1re soirée. Repli : ST-17 (CalDAV), qui ne dépend que de ST-09 et de ST-15a (F-3). Toute la logique planning avance sur simulateur pendant l'attente (F-4) |
| R-2 | SQLite-WASM + OPFS instable sur Safari iOS | Haute | ST-20 | S1-A tranche dès la 2e soirée la forme du port (synchrone ou asynchrone). S1-B le valide sur appareils avant ST-20. Repli IndexedDB derrière le port |
| R-3 | Parseur de Ruff seulement en dépendance git | Moyenne | Maintenance de la voie diff | S4 avant ST-22 ; repli tree-sitter-python |
| R-4 | Le titre « Focus — X » ou le marqueur fuit vers les collègues | Moyenne | Viole AC-11.1/11.2 | Critères (5) et (6) de S3 obligatoires avant ST-15b ; PROP-14 bloquant en ST-15a **et** en ST-15b |
| R-5 | **Saturation d'arbitrage** : trois voies qui posent des questions en même temps, perte de fil entre cycles | Haute | Arbitrages bâclés, décisions incohérentes entre voies | Plafond de 2 voies en cycle, budget de 8 points par soirée, une seule story à charge forte à la fois ; recalibrage après ST-04 |
| R-6 | Glissement vers un troisième module avant validation marché | Moyenne | Temps perdu | Garde-fou RG-G8 en fin de plan |
| R-7 | Fusion champ par champ qui produit un élément invalide (CONFLIT-1) | Haute dès la synchro | Données incohérentes | Crochet `revalider` en ST-04, marquage en ST-05, question en ST-13 |
| R-8 | Recalcul en parallèle sur deux ordinateurs | Moyenne | Agenda pollué | Identifiants déterministes (ST-13, ST-14), PROP-16 (ST-15a) |
| R-9 | `rmcp` jeune | Moyenne | ST-07 | S5 en 1re soirée ; repli JSON-RPC stdio |
| R-10 | **Conflits de fusion entre voies** (`Cargo.lock`, `arbitre`, `lib.rs`) | Haute | Temps d'intégration, régressions | Propriétaires et règles de fusion (§ Collisions), une seule story fusionnée à la fois, `Cargo.lock` régénéré et jamais fusionné à la main |
| R-11 | **Trousseau inaccessible en tâche de fond** (launchd hors session, systemd sans D-Bus) | Moyenne | AC-13.7 et synchro planifiée échouent en silence | S6 avant ST-11 ; repli fichier à droits restreints (archi) ; échec signalé à l'ouverture |
| R-12 | **Annotation du harnais** (30 à 50 cas à la main) sous-estimée | Moyenne | Voie diff à l'arrêt après ST-25 | G-3 compté comme charge d'arbitrage (6 points). Les agents pré-étiquettent, Quentin valide |

## Gestes hors story (sans cycle C-TDD)

| # | Geste | Quand | Charge |
|---|---|---|---|
| G-0 | Supprimer `crates/core/.git` et `crates/core/.gitignore` | Avant le premier commit (soirée 1) | faible |
| G-1 | Activer la protection contre l'envoi de secrets sur GitHub | Avant le premier push (soirée 1) | faible |
| G-2 | Mener les entretiens design partners (`/user-interview`) | En continu. La démo ST-22 est disponible dès la soirée 4 | hors plan |
| G-3 | Valider l'annotation de 30 à 50 cas du harnais (pré-étiquetés par agent) | Entre ST-25 et ST-26 | forte + : 6 points |
| G-4 | Ranger les cycles par story : `.ctdd/stories/ST-xx/` avec son propre `state.json` | Avant ST-01 | faible |

## Spikes

Chaque spike a un budget fixe, rend un verdict écrit dans `.ctdd/spikes/Sn.md` et ne laisse aucun code en production. Il tourne dans son propre worktree, hors des voies.

| Spike | Question (critères dans `03_archi.md`) | Soirée | Verdict requis avant | Taille | Arbitrage |
|---|---|---|---|---|---|
| **S3** | Google Calendar depuis une CLI open source, client OAuth personnel accepté par Workspace (D-4) | 1 (lancement), verdict dès que l'admin répond | ST-10 ; critères (5) et (6) avant ST-15b | S + délai externe | forte (4) : créer le client, solliciter l'admin, vérifier la visibilité chez un collègue |
| **S5** | SDK MCP Rust | 1 | ST-07 | S | faible (1) |
| **S1-A** | Port `Store` côté wasm : synchrone (worker + OPFS « sync access handle ») ou asynchrone ? Critères (1) et (6), plus (7) : poids du bundle avec la base des fuseaux | 1–2 | **ST-02** (gel du port) | S | moyenne (2) |
| **S4** | tree-sitter-python ou parseur de Ruff | 2 | ST-22 | S | moyenne (2) |
| **S6** | Secrets en tâche de fond : trousseau lisible depuis un agent launchd et depuis un timer systemd utilisateur sans session D-Bus ; repli fichier `0600` | 7 (voie planning) | ST-11, ST-19 | S | faible (1) |
| **S1-B + S2** | Critères (2) à (5) de S1 sur appareils ; PWA Leptos installable et hors ligne, servie par le relais (D-6) | 14 (voie mobile) | ST-20 | M | forte (4) : installation sur iOS et Android, à la main |

## Stories

**Légende.**
- **Zone** : classification DDD de `03_archi.md`.
  - **Full** = Full C-TDD (mutation ≥ 85 %, CI > 0,95, propriétés massives).
  - **Std** = C-TDD standard (mutation ≥ 70 %).
  - **Gen** = Classic AI-Assisted, avec les tests de contrat ou de propriétés indiqués.
- **Taille** : temps machine du cycle complet mené par agents, en soirées. S = ⅓, M = ⅔, L = 1. Ce temps comprend la mutation `--in-diff`.
- **Arbitrage** : temps de Quentin sur tout le cycle (questions, relectures, décisions, tests à la main). faible = 1 point, moyenne = 2, forte = 4. 1 point vaut environ 15 minutes.
- **Voie** : Socle, P (planning), M (mobile et socle étendu), D (diff).
- **« Sans valeur seule »** : la story ne sert l'utilisateur qu'avec la suivante. Aucun jalon n'est posé sur elle (F-20).

---

### Bloc A — Socle local (voie unique, soirées 1 à 3)

#### ST-01 : Fusion pure — HLC, écritures, versions écartées
- **Objectif** : le moteur de convergence, sans stockage, prouvé par propriétés. La story assainit aussi le workspace et pose la CI, la mutation et les dépendances partagées.
- **Zone** : Core / Full. **Voie** : Socle. **Taille** : M. **Arbitrage** : forte (premier cycle, choix des dépendances partagées, conventions de CI).
- **Couvre** : US-2 (AC-2.6, logique) · RG-2.1, 2.2, 2.3, 2.4 · INV-4, 5, 6, 7 · CONFLIT-3, CONFLIT-4/5.
- **Crates** : `arbitre-core` (`hlc`, `write`, `merge`, `clock`), `Cargo.toml` (workspace).
- **Dépendances** : G-0, G-4.
- **Livrable** :
  - port `Clock` ; HLC avec départage par appareil ; écriture `(donnée, champ, valeur, hlc, appareil, auteur)` ; `fusion(projection, écritures, filigranes) -> (projection', écartées, conflits)` ;
  - retrait de `rusqlite` et de `chrono/clock` du cœur ;
  - **`docs/decisions/ST-01-dependances.md`**, avec toutes les dépendances partagées déclarées dans `[workspace.dependencies]`, même celles qui ne servent pas encore (PL-6) ;
  - **CI** :
    - `cargo test` ;
    - wasm32 sur la liste nommée `arbitre-core`, `arbitre-plan`, sans échec si une crate n'existe pas encore ;
    - arêtes interdites, avec règles par nom de crate pour toutes les crates prévues par l'archi ;
    - gitleaks ;
    - **cargo-mutants `--in-diff`** sur PR ;
    - script de seuil par zone (85 / 70) ;
  - règle de fusion du `Cargo.lock` écrite dans le README de contribution.
- **Critère de complétion** :
  - propriétés vertes sur INV-5 (commutativité, associativité, idempotence), INV-6 et INV-7 ;
  - écrasement séquentiel non signalé, modification concurrente signalée ;
  - CI wasm32 verte ;
  - le job de mutation tourne sur la PR de ST-01 elle-même et rend un score ≥ 85 % sur `merge` et `hlc`.

#### ST-02 : Persistance — port Store et adaptateur SQLite
- **Objectif** : les écritures survivent à l'arrêt brutal et au disque plein, et plusieurs processus partagent la même base.
- **Zone** : Gen (adaptateur), avec propriétés bloquantes. **Voie** : Socle. **Taille** : M. **Arbitrage** : moyenne. **Sans valeur seule.**
- **Couvre** : US-1 (AC-1.4, edge disque plein), US-3 (edge concurrence) · RG-1.6, 1.7, 3.6 ; RG-1.1 (stockage local) · PROP-3, 4, 7.
- **Crates** : `arbitre-core` (`store` : port, erreurs `StockagePlein`, `Corrompu`, `Indisponible`), **`arbitre-sqlite` (naît)**.
- **Dépendances** : ST-01, **verdict S1-A** (forme synchrone ou asynchrone du port).
- **Livrable** :
  - `Store::appliquer` atomique, projection, journal par champ, `ecritures_depuis`, filigranes par pair, identifiant d'appareil ;
  - schéma générique (journal + projection), versionné ;
  - WAL et délai d'attente sur verrou.
- **Critère de complétion** :
  - arrêt brutal à un instant aléatoire : une écriture confirmée est présente, une écriture non confirmée est appliquée en entier ou pas du tout ;
  - stockage plein simulé ⇒ refus et état intact ;
  - deux **processus** écrivains entrelacés ⇒ aucune perte silencieuse.

#### ST-03 : Catalogue d'actions, CLI dérivée, codes de retour
- **Objectif** : le contrat que toutes les voies partagent, figé une fois.
- **Zone** : Supporting / Std. **Voie** : Socle. **Taille** : M. **Arbitrage** : forte (ce contrat est gelé pour toutes les voies). **Sans valeur seule.**
- **Couvre** : US-3 (AC-3.3, 3.4, 3.5) · RG-3.3, 3.4, 3.5 · INV-12, 13, 14 · AMB-16.
- **Crates** : `arbitre-core` (`catalog`, `error`), `arbitre` (cli).
- **Dépendances** : ST-02.
- **Livrable** :
  - `DescripteurAction` : nom préfixé par le module, schémas, `mutante`, `journalisée_comme`, et **`requiert_réseau: aucun | agenda | relais | ia`** (sert à PROP-1 et PROP-2) ;
  - `Sortie` : structurée et canonique, avec un rendu humain dérivé ;
  - **table complète** catégorie → code : succès, refus ou violation, usage, erreur d'outil, module non activé (F-6) ;
  - dérivation clap, `--json` ;
  - structure de `arbitre` : `main.rs`, `modules.rs` (une ligne par module), `wiring/<module>.rs`, un répertoire par surface.
- **Critère de complétion** :
  - commande mal formée ⇒ usage, état inchangé, code ≠ 0 ;
  - les sorties humaine et JSON portent la même information ;
  - chaque catégorie a son code, testé sur une action factice ;
  - la table est publiée dans la doc comme contrat (AC-3.5).

#### ST-04 : Modules activables et cloisonnés, conformance générique
- **Objectif** : le mécanisme de modules, et un test de conformité que chaque module rejoue sans le modifier.
- **Zone** : Supporting / Std. **Voie** : Socle. **Taille** : M. **Arbitrage** : moyenne.
- **Couvre** : US-1 (**AC-1.1**), US-3 (AC-3.2 ; edge module non activé), US-5 (AC-5.1 à 5.4) · RG-1.1, 1.2 (forme), 3.2 (forme), 3.7, 5.1 à 5.4 · INV-11 (forme), 15, 20, 21, 22, 23 · **PROP-1, PROP-2 (outillage)** · CONFLIT-1 (crochet), CONFLIT-6, AMB-12.
- **Crates** : `arbitre-core` (`module` ; `Contexte` avec `ia: Option<Port Ia>`, et **port `Ia` déclaré** : signature et erreurs seulement), `arbitre` (cli, `tests/support/`).
- **Dépendances** : ST-03.
- **Livrable** :
  - trait `Module` (`id`, `actions`, `executer`, `revalider`), registre, `arbitre module enable|disable|list` ;
  - vue `Store` restreinte à l'espace de noms ;
  - module factice de test ;
  - **observateur réseau** (proxy d'enregistrement utilisé par tous les clients HTTP, mode « coupé ») ;
  - **test de conformité générique** : activation, invisibilité, espace de noms, parité, et **toute action `requiert_réseau = aucun` réussit réseau coupé** (PROP-1) ;
  - assertion PROP-2 : aucune sortie hors des destinations déclarées.
- **Critère de complétion** :
  - un module désactivé est absent de l'aide et de la complétion ; son appel explicite ⇒ code « module non activé », état inchangé ;
  - une lecture hors espace de noms est impossible ;
  - le module factice passe la conformité, réseau coupé compris ;
  - après ST-04, `arbitre-core` n'est plus modifié que par la voie M (§ Collisions).

> **Recalibrage (début de la soirée 4, environ 30 min, 2 points).** Voir § Recalibrage.

---

### Bloc A' — Socle étendu (voie M, sauf ST-07 en voie P)

#### ST-07 : L'assistant agit par MCP
- **Objectif** : Claude Code, ou un autre client MCP, appelle toute action du catalogue. Ses suppressions et déplacements sont journalisés.
- **Zone** : Supporting / Std ; Gen pour le transport. **Voie** : P (sa première utilisatrice, et elle n'écrit pas `arbitre-core`). **Taille** : M. **Arbitrage** : moyenne (essai à la main avec deux clients).
- **Couvre** : US-4 (AC-4.1, 4.5), US-6 (AC-6.5, partie MCP) · RG-4.1, 4.6, 4.7, 6.4 · INV-10 (CLI ⊆ MCP), 18, 19, 23, 28 · AMB-11.
- **Crates** : `arbitre` (`mcp/` ; la requête « journal de l'assistant » filtre le journal par auteur côté CLI, sans toucher `arbitre-core`).
- **Dépendances** : ST-04, **S5** (F-5 : plus de dépendance à la pile).
- **Critère de complétion** :
  - la parité MCP est ajoutée **au test de conformité générique** : tout module passé et futur est couvert sans modification ;
  - module désactivé ⇒ aucun outil listé ;
  - suppression par MCP ⇒ entrée d'historique (date, « assistant ») dans la même transaction, prouvée sur le module factice ;
  - testé avec deux clients MCP.

#### ST-04b : Export et import
- **Objectif** : l'utilisateur exporte et réimporte toutes ses données, tous modules confondus.
- **Zone** : Core / Full. **Voie** : M. **Taille** : M. **Arbitrage** : moyenne (format versionné).
- **Couvre** : US-1 (AC-1.3), US-5 (edge import) · RG-1.3, 1.4, 1.5 · INV-1, 2, 3, **24**.
- **Crates** : `arbitre-core` (`export`), `arbitre` (actions socle `arbitre export`, `arbitre import`).
- **Dépendances** : ST-04 (espaces de noms, F-11).
- **Critère de complétion** :
  - `import(∅, export(S)) ≡ S` et `import(T, export(S)) = fusion(T, S)` par propriétés ;
  - fichier tronqué, illisible ou incohérent ⇒ état inchangé et cause nommée ;
  - données d'un module désactivé conservées à l'import, puis visibles à l'activation (module factice).

#### ST-18 : Enveloppe chiffrée et modèle du secret
- **Objectif** : ce qui part vers un relais est illisible et inaltérable sans la phrase secrète.
- **Zone** : Supporting (sécurité) / **Full malgré la zone**. **Voie** : M. **Taille** : S. **Arbitrage** : moyenne. **Sans valeur seule.**
- **Couvre** : US-2 (edge tiers non autorisé ; edge appareil sans le bon secret) · RG-2.8 · INV-9 · MANQUE-6 · **dépendance « secret de chiffrement » de `02_specs.md` (F-13)**.
- **Crates** : `arbitre-core` (`envelope` ; wasm32, `getrandom` avec sa feature wasm).
- **Dépendances** : ST-02.
- **Modèle par défaut** (`[À DÉFINIR]` à confirmer au cycle) :
  - la même phrase secrète est saisie une fois par appareil ;
  - la clé est dérivée par Argon2id avec un **sel public stocké sur le relais** ;
  - la clé dérivée est gardée au trousseau (natif) ;
  - aucune clé ne transite par le relais.
- **Critère de complétion** :
  - vecteurs de test XChaCha20-Poly1305 et Argon2id ;
  - tout octet altéré ⇒ rejet, données locales intactes ;
  - mauvais secret ⇒ message explicite ;
  - deux appareils qui partent de la même phrase et du même sel ⇒ même clé ;
  - numéro de format présent.

#### ST-19 : Relais et synchro entre deux ordinateurs
- **Objectif** : deux appareils natifs convergent en moins d'une minute via un relais que l'utilisateur héberge, et l'utilisateur voit les conflits.
- **Zone** : Gen (relais, transport) ; Core / Full pour la convergence de bout en bout. **Voie** : M. **Taille** : L. **Arbitrage** : forte.
- **Couvre** : US-2 (AC-2.1, 2.3, 2.4, 2.6) · RG-2.4, 2.5, 2.6, 2.7 ; RG-1.2 (relais) · INV-8 · PROP-5, 6 ; PROP-2 rejoué.
- **Crates** : `arbitre-core` (port `Transport`), `arbitre` (`relay/` : `arbitre relay serve` ; `arbitre sync`, `arbitre conflits`).
- **Dépendances** : ST-18, ST-04 (crochet `revalider`), **S6** (F-7 : plus de dépendance à ST-13).
- **Critère de complétion** :
  - simulateur à deux appareils, un relais, connectivité commandable et horloge ⇒ propagation < 60 s, reprise seule après 3 jours, date de dernière synchro affichée ;
  - une fusion qui rend une donnée du module factice invalide ⇒ `revalider` appelé, donnée marquée ;
  - **synchro lancée sans session interactive** (clé lue au trousseau selon S6) ; clé inaccessible ⇒ échec signalé à l'ouverture suivante ;
  - sans relais configuré ⇒ outil complet ;
  - le relais ne reçoit que des enveloppes.

#### ST-20 : Le téléphone consulte, hors ligne
- **Objectif** : la PWA, installée sans store et servie par le relais (D-6), affiche la pile, les jours, la charge et les questions, hors ligne et synchronisés.
- **Zone** : Gen. **Voie** : M. **Taille** : L. **Arbitrage** : forte (acceptation manuelle sur iOS et Android).
- **Couvre** : US-2 (AC-2.2, 2.5), C-2.9 · RG-1.1 (téléphone), 2.5 · D-1, D-6 · PROP-1 (PWA).
- **Crates** : **`arbitre-web` (naît)**, adaptateur `Store` navigateur (selon S1), `arbitre` (`relay serve --pwa <dir>`, PL-3) ; job CI `web` dans son propre fichier de workflow.
- **Dépendances** : ST-19, **S1-B + S2**. Lit le format des données du planning en lecture seule (ST-05, ST-09, ST-13 fusionnées : c'est le cas à la soirée 15).
- **`[À DÉFINIR]`** : où la PWA garde la clé dérivée (stockage de l'origine, risque documenté, ou saisie à chaque ouverture).
- **Critère de complétion** :
  - installation sur iOS et Android ; ouverture hors ligne ;
  - écriture distante visible en moins de 60 s ;
  - aucune sortie vers l'agenda ni vers une IA depuis la PWA.

#### ST-21 : Le téléphone agit
- **Objectif** : créer, modifier, faire, réordonner et répondre aux questions depuis le téléphone, avec le même résultat que la CLI.
- **Zone** : Gen (UI) ; parité en Supporting / Std. **Voie** : M. **Taille** : M. **Arbitrage** : moyenne.
- **Couvre** : US-3 (AC-3.1), US-6 (AC-6.5 complet) · RG-3.1, 6.4 · INV-10 (GUI ⊆ CLI), 28 · PROP-7 (côté GUI).
- **Crates** : `arbitre-web`.
- **Dépendances** : ST-20.
- **Critère de complétion** :
  - parité du catalogue étendue à la GUI (GUI ⊆ CLI : une action ajoutée côté CLI ne casse pas le test) ;
  - même entrée ⇒ même élément, même durée proposée, mêmes signalements ;
  - **écritures concurrentes de la CLI et de la PWA sur un même élément ⇒ convergence par la synchro et conflit signalé selon RG-2.4** (F-21).

#### ST-30 : Adaptateur IA multi-fournisseurs
- **Objectif** : un réglage unique choisit Claude, GPT, Mistral ou un modèle local, et l'absence de fournisseur ne casse rien.
- **Zone** : Gen (contrat) ; Supporting / Std (dégradation). **Voie** : M (story du socle, `[S-7]`). **Taille** : M. **Arbitrage** : faible. **Sans valeur seule** (première utilisatrice : ST-31).
- **Couvre** : US-4 (AC-4.2, 4.3, 4.4) · RG-4.2 à 4.5 ; RG-1.2 (sur sollicitation seulement) · INV-16, 17 · PROP-8, 9 ; PROP-2 rejoué.
- **Crates** : **`arbitre-ai` (naît)**, `arbitre` (`wiring/`). Le port `Ia` existe depuis ST-04 : `arbitre-core` n'est pas modifié.
- **Dépendances** : ST-04.
- **À trancher au cycle** : AMB-13.
- **Critère de complétion** :
  - changement de fournisseur ⇒ données et catalogue identiques ;
  - injoignable, clé refusée ou délai dépassé ⇒ erreur explicite pour les seules fonctions dépendantes ;
  - aucun appel automatique.

---

### Bloc B — Module planning (voie P)

Toutes les stories de ce bloc, sauf ST-10 et ST-15b, tournent sur le **simulateur d'`Agenda`** posé en ST-09. Aucune n'attend Google (F-4).

#### ST-05 : Éléments — créer, modifier, faire, supprimer
- **Zone** : Core / Full. **Taille** : M. **Arbitrage** : moyenne.
- **Couvre** : US-6 (AC-6.1 à 6.4), US-7 (AC-7.1, 7.4, 7.5) · RG-6.1, 6.2, 6.3, 7.1, 7.2, 7.4 ; RG-3.2 (`arbitre plan`) · INV-11, 25, 26, 27, 29, 30, 32 · CONFLIT-1 (marquage).
- **Crates** : **`arbitre-plan` (naît)**, `arbitre` (`modules.rs` : une ligne ; `wiring/plan.rs`).
- **Dépendances** : ST-04.
- **À trancher au cycle** : AMB-10, MANQUE-5, RENVOI-1 (côté saisie).
- **Critère de complétion** :
  - conformité générique verte pour `plan`, réseau coupé compris ;
  - engagement daté sans échéance refusé (création, changement de type, retrait d'échéance) ;
  - un élément fait sort de la pile et reste dans l'historique ;
  - `en_retard` conforme à INV-27 ;
  - `supprimer` est déclarée `journalisée_comme: suppression`.

#### ST-06 : Pile ordonnée et repriorisation
- **Zone** : Core / Full. **Taille** : S. **Arbitrage** : faible.
- **Couvre** : US-9 (AC-9.1 à 9.4) · RG-9.1, 9.2, 9.3, 9.5 · INV-38 à 41 · CONFLIT-2, CONFLIT-10.
- **Crates** : `arbitre-plan` (`item`), `arbitre` (`wiring/plan.rs`).
- **Dépendances** : ST-05.
- **Critère de complétion** :
  - « déplacer e n'écrit que le rang de e » ;
  - deux réordonnancements concurrents fusionnés ⇒ ordre total strict ;
  - pile vide ⇒ message ; aucune troncature.

#### ST-08 : Corriger les durées, et que l'outil apprenne
- **Zone** : Supporting / Std. **Taille** : S. **Arbitrage** : faible.
- **Couvre** : US-7 (AC-7.2, 7.3) · RG-7.3, 7.5, 7.6 · INV-31, 33 · PROP-11 · AMB-9.
- **Crates** : `arbitre-plan` (`duration`).
- **Dépendances** : ST-05.
- **Critère de complétion** :
  - une durée fixée n'est jamais réécrite ;
  - « semblable » est déterministe et sans IA ;
  - un générateur d'historiques de corrections montre le rapprochement.

#### ST-09 : Charge et capacité par jour (agenda simulé)
- **Zone** : Core / Full. **Taille** : M. **Arbitrage** : moyenne.
- **Couvre** : US-8 (AC-8.1 à 8.4, 8.6 sur simulateur) · RG-8.1 à 8.4 · INV-34 à 37 · AMB-1, 3, 4, 8 · « planifier à un jour ».
- **Crates** : `arbitre-plan` (`capacity`, **port `Agenda` lecture et écriture + simulateur**, propriété de la voie P).
- **Dépendances** : ST-06.
- **À trancher au cycle** : AMB-2, RENVOI-8.
- **Critère de complétion** :
  - capacité = 450 − union des réunions comptées, plancher 0 ; week-end et absence à 0 ;
  - événement marqué par l'outil non compté ;
  - débordement et excès présents en humain et en JSON.

#### ST-11 : Recalcul chaque matin, outil fermé
- **Zone** : Core / Full (idempotence) ; Gen (launchd et systemd). **Taille** : M. **Arbitrage** : moyenne.
- **Couvre** : US-13 (AC-13.2 : relecture et capacités ; **AC-13.7**) · RG-13.4 (lecture), 13.6, 13.7 · PROP-19 (partie), 21 · INV-56 (idempotence) · CONFLIT-7 · RG-G7.
- **Crates** : `arbitre-plan` (orchestration `recalcul` sur le port `Agenda`), `arbitre` (`scheduler/`, `arbitre plan schedule install`).
- **Dépendances** : ST-09, **S6** (F-4 : plus de dépendance à ST-10).
- **À trancher au cycle** : RENVOI-7.
- **Critère de complétion** :
  - deux recalculs successifs ⇒ le second est sans effet ;
  - horloge simulée avec cycles marche/arrêt ⇒ rattrapage au démarrage ;
  - **jeton inaccessible ou expiré ⇒ recalcul sans lecture d'agenda, capacité « non à jour », signal visible à l'ouverture suivante, jamais d'échec silencieux** (F-14) ;
  - installation testée sur macOS, documentée pour Linux.

#### ST-12 : Planifier la semaine
- **Zone** : Core / Full. **Taille** : M. **Arbitrage** : faible (défauts des specs).
- **Couvre** : US-13 (AC-13.1, 13.3) · RG-13.1, 13.2, 13.8 · INV-53, 54, 57 · AMB-5, 6, 7.
- **Crates** : `arbitre-plan` (`placement`, branché dans `recalcul`).
- **Dépendances** : ST-11 (F-4 : sur simulateur, sans Google).
- **Critère de complétion** :
  - « aucun jour déjà planifié ne change » sur pile et capacités générées ;
  - un élément qui ne tient nulle part reste non planifié, avec un avertissement ;
  - résultat indépendant de l'appareil.

#### ST-13 : Questions — urgence, débordement, élément invalide
- **Zone** : Core / Full. **Taille** : M. **Arbitrage** : moyenne.
- **Couvre** : US-13 (AC-13.2 : signalement ; AC-13.4 à 13.6) · RG-13.3, 13.4, 13.5 ; RG-G6 · INV-55, 56 · PROP-20 · CONFLIT-1 (question, via `revalider`) · AMB-22.
- **Crates** : `arbitre-plan` (`questions`), `arbitre` (`wiring/plan.rs`).
- **Dépendances** : ST-12.
- **À trancher au cycle** : RENVOI-6.
- **Critère de complétion** :
  - question ouverte ⇒ aucun déplacement lié ; réponse ⇒ appliquée, question fermée ;
  - deux appareils qui recalculent ⇒ même identifiant, une seule question après fusion ;
  - une fusion simulée rend un élément invalide ⇒ une question (sans la synchro réelle).

#### ST-14 : Plan de blocs et confidentialité (aperçu)
- **Zone** : Core / Full. **Taille** : M. **Arbitrage** : forte (cinq renvois et la confidentialité).
- **Couvre** : US-10 (AC-10.1 à 10.4), US-11 (AC-11.1 à 11.4) · RG-10.1 à 10.3, 11.1 à 11.5 · INV-42 à 49 · CONFLIT-9, CONFLIT-11.
- **Crates** : `arbitre-plan` (`blocks`).
- **Dépendances** : ST-12.
- **À trancher au cycle** : RENVOI-1, 2, 4, 5, 8.
- **Critère de complétion** :
  - contenu visible = titre, début, fin et statut occupé, rien d'autre ;
  - aucun titre « Focus — » suivi de l'intitulé brut sans sujet public ;
  - bloc quotidien ≥ 30 min, au titre constant ;
  - identifiants de blocs déterministes.

#### ST-15a : Réconciliation des blocs (pure, sur simulateur)
- **Zone** : Core / Full, **PROP-14 bloquant**. **Taille** : M. **Arbitrage** : moyenne.
- **Couvre** : US-10 (AC-10.5, 10.6, logique), US-6 (edge élément supprimé avec bloc) · RG-10.4 à 10.7, 6.5 · PROP-10, 14 (fonction pure + simulateur), 15, 16 · AMB-22.
- **Crates** : `arbitre-plan` (`reconcile`, branché dans `recalcul`).
- **Dépendances** : ST-14, ST-13.
- **Critère de complétion** :
  - simulateur d'agenda à origines mêlées ⇒ aucune opération sur un événement sans marqueur ;
  - panne puis reprise ⇒ opérations sans doublon ;
  - deux appareils ⇒ au plus un bloc par élément et un bloc quotidien par jour.

#### ST-16 : Bloc déplacé à la main, la pile suit
- **Zone** : Core / Full. **Taille** : M. **Arbitrage** : moyenne.
- **Couvre** : US-12 (AC-12.1 à 12.4), US-9 (AC-9.5), US-13 (AC-13.2 : blocs déplacés) · RG-12.1 à 12.5, 9.4 · INV-50, 51, 52 · PROP-13, 17, 18 · CONFLIT-9.
- **Crates** : `arbitre-plan` (`reconcile` : déplacements lus).
- **Dépendances** : ST-15a.
- **À trancher au cycle** : RENVOI-3.
- **Critère de complétion** :
  - aucun événement d'agenda ne change un rang ni une durée ;
  - déplacement qui fait déborder ⇒ question, rien d'autre ne bouge ;
  - jour à capacité 0 ⇒ « hors capacité » ;
  - déplacement après l'échéance ⇒ élément en retard.

#### ST-10 : Adaptateur Google — lecture
- **Zone** : Gen (contrat) ; Core / Full pour la fraîcheur. **Taille** : M. **Arbitrage** : moyenne.
- **Couvre** : US-8 (AC-8.5), US-1 (AC-1.2 côté agenda ; matrice US-1 cas 3) · RG-8.5, 1.2 · PROP-2 (rejoué), 12 · CONFLIT-8 · D-3, D-4.
- **Crates** : **`arbitre-calendar` (naît)**, `arbitre` (connexion, `arbitre plan recalcul` branché sur Google).
- **Dépendances** : ST-09, **verdict S3**. Peut s'engager **dès** ce verdict : sa place à la soirée 11 n'est qu'une hypothèse prudente sur le délai de l'admin.
- **Critère de complétion** :
  - tests de contrat sur réponses enregistrées : occurrences dépliées, journée entière, `outOfOffice`, statut de réponse, `transparency`, **événement de l'outil déplacé** (pour ST-16) ;
  - agenda injoignable ⇒ dernière capacité connue « non à jour », avec sa date ;
  - observateur PROP-2 : seule sortie vers l'agenda connecté ;
  - jeton au trousseau, aucun secret dans le dépôt.

#### ST-15b : Adaptateur Google — écriture des blocs
- **Zone** : Gen (écriture) ; PROP-14 en défense en profondeur. **Taille** : M. **Arbitrage** : moyenne.
- **Couvre** : US-10 (AC-10.5, 10.6 en réel) · RG-1.2 (écriture agenda) · PROP-14 (adaptateur), PROP-2 (rejoué).
- **Crates** : `arbitre-calendar` (écriture ; refus de tout événement sans marqueur).
- **Dépendances** : ST-15a, ST-10, **S3 critères (5) et (6)**.
- **Critère de complétion** :
  - l'adaptateur refuse toute opération sur un événement sans marqueur, même si la fonction pure en émettait une ;
  - essai réel : blocs posés, puis retirés quand l'élément est fait.

#### ST-17 : Agendas CalDAV — *différable, repli de R-1*
- **Zone** : Gen (contrat). **Taille** : M. **Arbitrage** : moyenne.
- **Couvre** : arbitrage « Agendas » (CalDAV ensuite) ; RG-8.*, 10.* rejouées sur un second adaptateur ; PROP-2 (rejoué).
- **Crates** : `arbitre-calendar` (`caldav/`) ; crée la crate si ST-10 n'a pas eu lieu.
- **Dépendances** : **ST-09, ST-15a** (F-3 : aucune dépendance à Google). Ne s'engage qu'à la demande d'un utilisateur réel, ou si S3 conclut au repli. Dans ce cas, elle prend dans la voie P la place de ST-10 et de ST-15b.
- **Critère de complétion** : la suite de contrat du port `Agenda` passe à l'identique sur le simulateur, sur CalDAV et, si elle existe, sur Google.

---

### Bloc D — Decision diff (voie D)

> La voie démarre dès ST-04 (arbitrage F-10). ST-22 est le quickwin du plan du 2026-10-02 (« 3 violations dans ton repo »), montrable en entretien (G-2).

#### ST-22 : `arbitre diff check` — frontières d'import
- **Zone** : Core / Full (vérification, verdict) ; Supporting / Std (extracteur Python). **Taille** : L. **Arbitrage** : forte (MANQUE-2, CONFLIT-17, essais OSS, démo).
- **Couvre** : US-14 (AC-14.1 à 14.4), US-15 (AC-15.1 en local, 15.2 à 15.5) · RG-14.1 à 14.6 (frontière d'import), 15.1 à 15.6 ; RG-3.2 (`arbitre diff`) · INV-11, 58 à 62, 63 (partie), 64 à 69 · AMB-14, 15, 16 · CONFLIT-17 · D-2.
- **Crates** : **`arbitre-diff` (naît ; natif)**, `arbitre` (`modules.rs` : une ligne ; `wiring/diff.rs`).
- **Dépendances** : ST-04, **S4**.
- **À trancher au cycle** : MANQUE-2 ; **reconfirmer CONFLIT-17** (défaut : verdict sur tout l'état résultant).
- **Critère de complétion** :
  - conformité générique verte pour `diff`, réseau coupé compris ;
  - déterminisme par permutation de l'ordre des fichiers ;
  - fichier mal formé ⇒ refus total, invariant fautif nommé ;
  - langage non pris en charge ⇒ « non vérifié » ; pas de fichier d'invariants ⇒ succès avec mention ;
  - essayé sur 2 ou 3 dépôts OSS ;
  - codes « violation » et « erreur d'outil » **pris dans la table de ST-03**.

#### ST-23 : Familles « types du domaine » et « appels interdits »
- **Zone** : Core / Full. **Taille** : M. **Arbitrage** : moyenne.
- **Couvre** : US-14 (AC-14.2, 14.4), US-15 (AC-15.4, 15.5) · RG-14.6 · INV-63.
- **Crates** : `arbitre-diff`.
- **Dépendances** : ST-22.
- **À trancher au cycle** : AMB-14 (type du domaine en Python non annoté).
- **Critère de complétion** :
  - cas qui respectent et cas qui violent pour chaque famille, avec fichier et ligne exacts ;
  - extension sans toucher au modèle de faits.

#### ST-24 : Vérification de PR en CI
- **Zone** : Core / Full (union base ∪ tête, affaiblissement) ; Gen (Action). **Taille** : M. **Arbitrage** : moyenne.
- **Couvre** : US-15 (AC-15.1 en CI) · RG-15.1, 15.3 · CONFLIT-12 · D-2.
- **Crates** : `arbitre-diff` (port `Dépôt`), `action.yml` et son workflow d'autotest (fichiers propres à la voie D).
- **Dépendances** : ST-23.
- **Critère de complétion** :
  - une PR qui retire l'invariant qu'elle viole ⇒ échec et alerte distincte ;
  - codes de ST-03 lisibles par la CI ;
  - aucune sortie réseau ;
  - **effet de CONFLIT-17 explicité dans la doc de l'Action** : avec le défaut, une PR sur un dépôt qui a déjà des violations échoue (F-23).

#### ST-25 : Harnais d'évals de l'extracteur
- **Zone** : Supporting / Std (outil de dev, hors produit). **Taille** : M. **Arbitrage** : forte, plus G-3 (6 points).
- **Couvre** : MANQUE-1, MANQUE-4 ; plan du 2026-10-02, brique 2.
- **Crates** : `arbitre-diff` (tests ou binaire de dev, hors `arbitre`).
- **Dépendances** : **ST-23** (F-19 : les labels viennent de la vérification, pas de l'Action).
- **Critère de complétion** :
  - jeu de cas versionné, pré-étiqueté par les violations de `verifier`, prêt pour G-3 ;
  - une commande rend rappel et précision par catégorie de décision.

#### ST-26 : Décisions structurelles d'une PR, avec leur couverture
- **Zone** : Supporting / Std (extraction) ; Core / Full (couverture). **Taille** : L. **Arbitrage** : moyenne.
- **Couvre** : US-16 (AC-16.1, 16.2) · RG-16.1, 16.2, 16.5, 16.6 · INV-70, 71, 74, 75 · CONFLIT-12, 14, 15, AMB-18.
- **Crates** : `arbitre-diff` (`extract`).
- **Dépendances** : ST-25, **G-3**.
- **À trancher au cycle** : RENVOI-9 ; objectifs de rappel et de précision.
- **Critère de complétion** :
  - objectifs atteints sur le harnais ;
  - modification cosmétique ⇒ aucune décision, avec un message ;
  - le fichier de décisions n'en produit pas.

#### ST-27 : Accepter ou rejeter chaque décision
- **Zone** : Core / Full. **Taille** : M. **Arbitrage** : moyenne.
- **Couvre** : US-16 (AC-16.3) · RG-16.4 · INV-73 · D-5.
- **Crates** : `arbitre-diff` (`decisions`). Les surfaces CLI et MCP sont dérivées du catalogue, sans code propre à `diff` dans `mcp/`.
- **Dépendances** : ST-26, **ST-07** (F-5).
- **À trancher au cycle** : AMB-19, format exact des champs.
- **Critère de complétion** :
  - aller-retour TOML sans perte ;
  - acceptation identique par la CLI et par le MCP.

#### ST-28 : Ce qui a changé entre deux itérations
- **Zone** : Core / Full. **Taille** : M. **Arbitrage** : moyenne.
- **Couvre** : US-17 (AC-17.1 à 17.3) · RG-17.1 à 17.5 · INV-76 à 80 · CONFLIT-13, AMB-20.
- **Crates** : `arbitre-diff` (`delta`).
- **Dépendances** : ST-27.
- **À trancher au cycle** : AMB-17.
- **Critère de complétion** :
  - un décalage de lignes seul ne change pas l'identité ;
  - première itération ⇒ toutes les décisions sont nouvelles ;
  - push forcé ⇒ « N acceptations perdues ».

#### ST-29 : Décisions affichées sur la PR
- **Zone** : Gen. **Taille** : S. **Arbitrage** : faible.
- **Couvre** : US-16 (AC-16.4) · RG-16.7 · PROP-22.
- **Crates** : `arbitre-diff` (rendu Markdown), `action.yml` (publication).
- **Dépendances** : ST-28, ST-24.
- **Critère de complétion** : hébergeur simulé en échec ⇒ erreur explicite, liste disponible en local.

#### ST-31 : Décisions proposées par l'IA
- **Zone** : Supporting / Std. **Taille** : S. **Arbitrage** : faible.
- **Couvre** : US-16 · RG-16.3 · INV-72 · RG-4.4 ; RG-1.2 · **CONFLIT-16** · **PROP-2 (rejoué)** (F-15).
- **Crates** : `arbitre-diff`.
- **Dépendances** : ST-30, ST-28.
- **Déclencheur** : **commande locale explicite uniquement** (`arbitre diff decisions --ia`, ou l'action MCP correspondante, déclarée `requiert_réseau: ia`). L'Action CI ne l'appelle jamais : il n'y a pas de clé en CI (D-2) ni d'appel automatique (CONFLIT-7/8).
- **Critère de complétion** :
  - fournisseur simulé absent ou défaillant ⇒ décisions structurelles intactes, verdict inchangé ;
  - avertissement avant le **premier** envoi de code à un fournisseur distant (CONFLIT-16) ;
  - observateur PROP-2 : sortie uniquement vers le fournisseur configuré, uniquement sur la commande.

#### ST-32 : Invariants candidats, preuves à l'appui
- **Zone** : Supporting / Std ; Core / Full pour INV-81 et 82. **Taille** : L. **Arbitrage** : moyenne.
- **Couvre** : US-18 (AC-18.1 à 18.5) · RG-18.1 à 18.6 · INV-81 à 84 · PROP-23, 24 ; PROP-2 (rejoué) · CONFLIT-16 (réutilise l'avertissement de ST-31), MANQUE-3.
- **Crates** : `arbitre-diff` (`candidates`).
- **Dépendances** : ST-30, ST-23, **ST-07** (F-5).
- **Critère de complétion** :
  - candidats hors famille v1 ou mal formés écartés ;
  - contre-exemples = violations de `verifier` ;
  - dépôt trop petit ⇒ message, aucun candidat ;
  - sans fournisseur ⇒ fonction indisponible, reste du module nominal ;
  - **la preuve d'un candidat suit CONFLIT-17** : un candidat déjà violé dans le dépôt est présenté avec ses violations, et l'adopter fait échouer les PR suivantes tant qu'elles ne sont pas corrigées (edge US-18, F-23).

---

## ⛔ GARDE-FOU — fin de plan

**Aucun module au-delà de planning et decision diff ne démarre avant 10 entretiens de design partners** (RG-G8). Planning et decision diff sont des besoins personnels : le garde-fou ne coupe aucune story de ce plan.

- **Critère de levée** : 10 entretiens menés et consignés (`/user-interview`, vault `01-work/meetings/`). L'orchestrateur vérifie le compte avant d'ouvrir le cycle d'un troisième module.

---

## Parallélisme

### Voies

| Voie | Contenu, dans l'ordre | Propriétaire exclusif de |
|---|---|---|
| **Socle** (soirées 1–3) | ST-01 → ST-02 → ST-03 → ST-04 | tout, puis plus rien après ST-04 |
| **P — planning** | ST-05 → ST-06 → ST-07 (MCP) → ST-08 → ST-09 → S6 → ST-11 → ST-12 → ST-13 → ST-14 → ST-15a → ST-16 → ST-10 → ST-15b (ST-17 à la place de ST-10 et ST-15b si S3 échoue) | `arbitre-plan`, `arbitre-calendar`, `arbitre/{wiring/plan.rs, mcp/, scheduler/}` |
| **D — decision diff** | ST-22 → ST-23 → ST-24 → ST-25 → G-3 → ST-26 → ST-27 → ST-28 → ST-29 → ST-31 → ST-32 | `arbitre-diff`, `arbitre/wiring/diff.rs`, `action.yml`, workflow d'autotest |
| **M — mobile et socle étendu** | ST-04b → ST-18 → ST-30 → ST-19 → S1-B + S2 → ST-20 → ST-21 | **`arbitre-core` après ST-04**, `arbitre-ai`, `arbitre-web`, `arbitre/{relay/, sync}`, workflow `web` |

Arêtes entre voies (toutes en avance sur leur besoin dans l'ordonnancement ci-dessous) :
- ST-07 (P) → ST-27, ST-32 (D) ;
- ST-30 (M) → ST-31, ST-32 (D) ;
- ST-05, ST-09, ST-13 (P) → ST-20 (M), en lecture seule.

### Plafond de voies actives (fixé par l'arbitrage)
- **Budget** : 8 points d'arbitrage par soirée. Cela fait environ 2 h de décisions et de relectures dans une soirée de 2 h 30, le reste partant en changements de contexte.
- **Au plus 2 voies en cycle C-TDD en même temps.** Une 3e voie ne s'ouvre que pour une story **faible**, ou quand l'une des deux attend sans cycle en cours (délai externe, annotation G-3).
- **Au plus une story à charge forte en cours.** Deux fortes ensemble, c'est deux contrats à tenir en tête.
- **Priorité en cas de conflit** (arbitrage F-10) : P, puis D, puis M. Une voie qui attend un arbitrage n'en prend pas un autre en parallèle : elle attend.
- Le graphe autoriserait 3 voies dès la soirée 4. Le budget n'en tient que 2.

### Points de collision et règles

| Point de collision | Voies qui y touchent | Règle |
|---|---|---|
| `arbitre-core` | M seule après ST-04 (ST-04b `export`, ST-18 `envelope`, ST-19 `Transport`) | Ajouts dans de nouveaux fichiers. Dans `lib.rs`, une ligne `mod` ajoutée en fin. Les types de contrat (`DescripteurAction`, `Sortie`, catégories d'erreur, `Module`, `Contexte`, port `Ia`) sont **gelés**. Les changer est un *amendement du socle* : arbitrage fort, les autres voies se rebasent avant de reprendre |
| Crate `arbitre` (cli) | P, D, M | Un fichier par voie (`wiring/<module>.rs`, `mcp/`, `scheduler/`, `relay/`). Fichiers chauds : `modules.rs` (une ligne en ST-05 et en ST-22) et `main.rs` (une ligne par sous-commande hors catalogue : `mcp`, `schedule`, `relay serve`, `sync`). Ajouts en fin de liste uniquement |
| Catalogue d'actions | P, D | Les noms sont préfixés par le module (`plan.*`, `diff.*`). Les tests de parité sont **génériques** (ST-04, ST-07, ST-21) : ajouter une action ne les modifie pas. GUI ⊆ CLI : une action CLI nouvelle ne casse pas la GUI |
| Table des codes de retour | aucune après ST-03 | Gelée (F-6) ; un changement est un amendement du socle |
| `Cargo.toml` racine | aucune en principe | Toutes les dépendances partagées sont déclarées en ST-01 (PL-6). Une nouvelle dépendance partagée passe par un amendement de `ST-01-dependances.md`. Chaque crate ajoute ses dépendances dans son propre `Cargo.toml` |
| `Cargo.lock` | toutes | Jamais fusionné à la main : sur conflit, prendre celui de `main` puis lancer `cargo check --workspace`, qui le régénère. Une seule story fusionnée à la fois |
| Configuration CI | M (workflow `web`), D (autotest de l'Action) | Les règles wasm32 et d'arêtes, écrites en ST-01, couvrent toutes les crates prévues. Les ajouts se font dans de **nouveaux** fichiers de workflow |
| Outillage de test partagé | toutes | Observateur réseau, mode coupé et module factice sont posés au socle (ST-04), dans `arbitre/tests/support/` ; seul le socle les modifie. Le simulateur d'`Agenda` appartient à P |
| `action.yml` | D seule (ST-24, ST-29) | Séquentiel dans la voie |
| `.ctdd/` et `docs/decisions/` | toutes | Un répertoire par story (G-4) ; décisions nommées par story |

**Protocole d'intégration** :
- chaque story vit dans son worktree et sa branche ;
- elle se rebase sur `main` juste avant de fusionner ;
- une seule fusion à la fois ;
- les autres voies se rebasent à leur prochain début de story, pas en cours de cycle.

## Calendrier

### Ordonnancement indicatif (soirée par soirée)

Lecture : `ST-xx (points)`. Le total de la soirée est en points. Une story apparaît la soirée où son cycle tourne ; une story L occupe une voie toute la soirée.

| Soirée | Socle / P | D | M | Points |
|---|---|---|---|---|
| 1 | G-0, G-1, G-4 (1) · ST-01 (4) · S5 (1) · S3 lancé (2) | — | — | 8 |
| 2 | S1-A (2) · ST-02 (2) · ST-03 entamée (2) · S4 (2) | — | — | 8 |
| 3 | ST-03 finie (2) · ST-04 (2) · S3 suite (2) | — | — | 6 |
| 4 | **Recalibrage (2)** · ST-05 (2) | ST-22 (4) | — | 8 |
| 5 | ST-06 (1) · ST-07 (2) | ST-23 (2) · ST-24 entamée (2) | — | 7 |
| 6 | ST-08 (1) · ST-09 (2) | ST-24 finie · ST-25 (4) | — | 7 |
| 7 | S6 (1) · ST-11 (2) | G-3 (2) | ST-04b (2) | 7 |
| 8 | ST-12 (1) · ST-13 entamée (2) | G-3 (2) | ST-18 (2) | 7 |
| 9 | ST-13 finie · ST-14 (4) | G-3 (2) | ST-30 (1) | 7 |
| 10 | ST-15a (2) · ST-16 entamée (2) | ST-26 (2) | — | 6 |
| 11 | ST-16 finie · ST-10 (2) | ST-27 (2) · ST-28 entamée (2) | — | 6 |
| 12 | ST-15b (2) | ST-28 finie · ST-29 (1) · ST-31 (1) | — | 4 |
| 13 | — | ST-32 (2) | ST-19 (4) | 6 |
| 14 | — | — | S1-B + S2 (4) | 4 |
| 15 | — | — | ST-20 (4) | 4 |
| 16 | — | — | ST-21 (2) | 2 |

Total : environ 97 points sur 16 soirées. Les soirées 12 à 16 ont de la marge. Elles absorbent le dépassement probable des premiers cycles, ST-17 si S3 échoue, ou se tiennent à 2 soirées par semaine sans glisser. La voie M ne peut pas aller plus vite, parce que ses trois stories fortes (ST-19, S1-B + S2, ST-20) passent l'une après l'autre.

### Jalons datés

Hypothèses : 3 soirées par semaine (mardi, mercredi, jeudi) ou 2 (mardi, jeudi), à partir du mardi 2026-11-03, sans congés. À 2 soirées par semaine, les soirées 15 et 16 tombent sur la semaine de Noël et sont décalées à janvier.

| Jalon | Fin de | Soirée | 3 soirées/sem. | 2 soirées/sem. |
|---|---|---|---|---|
| Socle posé, **recalibrage** | ST-04 | 3 | 2026-11-05 | 2026-11-10 |
| **Démo decision diff** (« 3 violations ») | ST-22 | 4 | 2026-11-10 | 2026-11-12 |
| Planning sur CLI et assistant, utile au quotidien | ST-09 (avec ST-07) | 6 | 2026-11-12 | 2026-11-19 |
| Decision diff en CI | ST-24 | 6 | 2026-11-12 | 2026-11-19 |
| Recalcul du matin, outil fermé | ST-11 | 7 | 2026-11-17 | 2026-11-24 |
| Planning complet sur simulateur (semaine, questions, blocs, déplacements) | ST-16 | 11 | 2026-11-25 | 2026-12-08 |
| **Blocs dans Google Calendar** (si S3 a répondu à la soirée 11) | ST-15b | 12 | 2026-11-26 | 2026-12-10 |
| **Decision diff complet** | ST-32 | 13 | 2026-12-01 | 2026-12-15 |
| Synchro entre deux ordinateurs | ST-19 | 13 | 2026-12-01 | 2026-12-15 |
| **Téléphone** (consultation puis action) | ST-20, ST-21 | 15–16 | 2026-12-03 – 12-08 | 2027-01-05 – 01-07 |

Tout le cadre tient en **5 semaines à 3 soirées par semaine**. C'est la borne haute de la référence. À 2 soirées par semaine, il en faut environ 8, congés compris. La fenêtre du tier Team (mi-2027, plan du 2026-10-02) n'est pas menacée : la démo arrive dès la 2e semaine (F-9).

### Recalibrage après ST-04

Au début de la soirée 4, avant d'ouvrir les voies :
1. **Mesurer** sur ST-01 à ST-04 et sur les spikes, pour chaque story : les soirées machine réelles, les minutes d'arbitrage réelles par phase du cycle, et le temps machine de la mutation.
2. **Remplacer** les valeurs S/M/L (⅓, ⅔, 1 soirée) et faible/moyenne/forte (1, 2, 4 points) par les médianes mesurées, et le budget de 8 points par le temps d'arbitrage réellement disponible dans une soirée.
3. **Recalculer** le plafond. Si l'arbitrage mesuré dépasse 1,5 fois l'estimation, une seule voie tourne en cycle, et la seconde n'ouvre que des stories faibles. S'il est inférieur à 0,7 fois l'estimation, une 3e voie peut s'ouvrir sur des stories moyennes.
4. **Re-dater** les jalons dans ce fichier, à partir des mesures. L'ordonnancement indicatif ci-dessus est alors abandonné.
5. **Vérifier S3** : si l'admin Workspace n'a pas répondu, garder ST-10 en fin de voie P. S'il a refusé, engager ST-17.

## Points à trancher (niveau plan)

| # | Question | Recommandation (retenue par défaut) |
|---|---|---|
| PL-1 | Place de la voie mobile ? | **Troisième en priorité**, derrière P et D (arbitrage F-10 pour D). Elle démarre à la soirée 7, quand D attend l'annotation. Le téléphone arrive entre le 2026-12-03 et le 2027-01-05. Le chiffre « 3 à 4 mois » de la v1 est retiré. |
| PL-2 | S3 lancé tôt ? | **Oui, à la 1re soirée.** C'est le seul délai externe du plan. Il ne bloque plus que ST-10 et ST-15b, deux feuilles du graphe. |
| PL-3 | Relais en crate dédiée ou en module de la CLI ? | **Module de la CLI.** La PWA est servie depuis un répertoire (`relay serve --pwa <dir>`) : pas de build wasm dans le binaire, pas de couplage entre voies. Le serveur du relais reçoit tout le binaire `arbitre`, ce qui est accepté en v1. On extraira `arbitre-relay` si le poids ou la surface exposée deviennent un problème. |
| PL-4 | CalDAV (ST-17) : engagé d'office ou à la demande ? | **À la demande, ou si S3 échoue.** C'est désormais exécutable, puisque ST-17 ne dépend que de ST-09 et de ST-15a. |
| PL-6 | Dépendances partagées (F-17) | Retenu par défaut et consigné en ST-01 : **tokio** (exigé par rmcp, réutilisé partout), **reqwest + rustls** (Google, relais, IA ; pas d'OpenSSL), **keyring** (trousseau, avec repli fichier `0600`), **serde / serde_json**, **proptest** (propriétés), **cargo-mutants** (outil, hors `Cargo.toml`). Côté wasm : `fetch` du navigateur derrière le port `Transport`, sans reqwest. |
| PL-7 | Plafond : 2 voies en cycle, 8 points par soirée, une seule story forte à la fois | Retenu par défaut ; recalibré après ST-04. |

PL-5 est retiré : la résolution par défaut de CONFLIT-17 s'applique, et elle est reconfirmée au cycle de ST-22. Ses effets sont notés dans ST-24 et ST-32.

## Invariants C-TDD à vérifier post-implémentation

| Invariant / Property | Story | Vérification |
|---|---|---|
| INV-4, 5, 6, 7 | ST-01 | Propriétés sur la fusion |
| INV-1, 2, 3, 24 | ST-04b | Aller-retour export/import, fichiers abîmés générés, module factice désactivé |
| INV-12, 13, 14 | ST-03 | CLI dérivée ; équivalence humain/JSON ; table des codes |
| INV-11 | ST-04 (forme), ST-05 (`plan`), ST-22 (`diff`) | Routage des points d'entrée |
| INV-15, 20, 21, 22, 23 | ST-04, rejoués par la conformance en ST-05 et ST-22 | Test de conformité générique |
| INV-10 | ST-07 (CLI ⊆ MCP, générique), ST-21 (GUI ⊆ CLI) | Parité dérivée du catalogue |
| INV-18, 19, 28 | ST-07 (module factice), ST-05 (classes déclarées), ST-21 | Journal de l'assistant dans la même transaction ; même entrée ⇒ même résultat |
| INV-25, 26, 27, 29, 30, 32 | ST-05 | Validité et cycle de vie |
| INV-31, 33 · PROP-11 | ST-08 | Générateur d'historiques de corrections |
| INV-34 à 37 | ST-09 | Simulateur `Agenda` |
| INV-38 à 41 | ST-06 | Rang fractionnaire, y compris après fusion |
| INV-42 à 49 | ST-14 | Fonctions pures, contenu visible exhaustif |
| INV-50, 51, 52 · PROP-13, 17, 18 | ST-16 | Simulateur avec déplacements manuels ; contrat Google en ST-10 |
| INV-53, 54, 57 | ST-12 | « L'existant ne bouge pas » |
| INV-55, 56 · PROP-20 | ST-13 (idempotence en ST-11) | Utilisateur simulé, horloge, deux appareils |
| INV-8 · PROP-5, 6 | ST-19 | Deux appareils, relais, connectivité, horloge |
| INV-9 | ST-18 | Vecteurs de test, altération, mauvais secret, même clé sur deux appareils |
| INV-16, 17 · PROP-8, 9 | ST-30 | Bouchons de fournisseurs |
| INV-58 à 69 | ST-22, ST-23, ST-24 | Permutation ; dépôts OSS |
| INV-70, 71, 74, 75 | ST-26 | Harnais (ST-25 + G-3) |
| INV-72 | ST-31 | Fournisseur simulé défaillant |
| INV-73 | ST-27 | Aller-retour TOML |
| INV-76 à 80 | ST-28 | Itérations générées, push forcé simulé |
| INV-81 à 84 · PROP-23, 24 | ST-32 | Fournisseur simulé (valides, mal formés, hors famille, aucun) |
| **PROP-1** (AC-1.1) | ST-04 (outillage + conformance), **rejoué automatiquement pour chaque action** de ST-05 à ST-16 et de ST-22 à ST-32 ; ST-20 (PWA) | Mode réseau coupé du test de conformité |
| **PROP-2** | ST-04 (observateur), rejoué en ST-10, 15b, 17, 19, 30, **31**, 32 | Observateur réseau : destination et déclencheur |
| PROP-3, 4, 7 | ST-02 ; PROP-7 côté GUI en ST-21 (concurrence par la synchro) | Arrêt brutal, stockage plein, deux processus ; CLI contre PWA |
| PROP-10, 14, 15, 16 | ST-15a (pure + simulateur) ; PROP-14 aussi en ST-15b (adaptateur) | Origines mêlées, panne/reprise, deux appareils ; **PROP-14 bloquant** |
| PROP-12 | ST-10 | Agendas multiples avec changements et pannes |
| PROP-19, 21 | ST-11, complétées en ST-13 et ST-16 | Horloge, cycles marche/arrêt, jeton inaccessible |
| PROP-22 | ST-29 | Hébergeur simulé |
| C-2.9 | ST-20 | Acceptation manuelle iOS et Android |
| Seuils de mutation | ST-01 (outil), chaque review de story | `cargo mutants --in-diff`, seuil de la zone |
