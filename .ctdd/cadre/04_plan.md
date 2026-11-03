# Plan d'implémentation — OpenArbitre, cadre global (découpe en stories) — v3

## Traçabilité
- Sources : `.ctdd/cadre/02_specs.md`, `.ctdd/cadre/03_archi.md`, et les sections « Arbitrages de Quentin » de `01_story.md`, `02_specs.md`, `03_archi.md` et de la revue v2 (`history/05_da_review_v2.md`, qui reprend F-10), qui font foi. `.ctdd/cadre/inputs/plan-decision-diff-2026-10-02.md` sert de contexte pour la partie C.
- Versions précédentes : `history/04_plan_v1.md` et `history/04_plan_v2.md`, bloquées par les revues 1/3 et 2/3.
- Date : 2026-11-03 · Statut : DRAFT v3, dernière itération avant escalade · Cycle sans code.
- Ce plan **est** la découpe en stories. Chaque story `ST-xx` aura son propre cycle C-TDD complet (story → specs → archi locale → plan → DA → skeleton → implémentation → review). Les `[À DÉFINIR]`, `AMB-*` et `RENVOI-*` d'une story se tranchent pendant son cycle.
- **Identifiants stables** : numéros de la v2 conservés. Ajouts v3 : S3 coupé en S3-L (lancement) et S3-V (verdict).
- **Consigne de Quentin pour la v3** : aucune date au-delà de ST-04. Les jalons sont datés au recalibrage, à partir des mesures de ST-01 à ST-04.

## Résolution des failles de la revue v2

| Faille | Résolution en une ligne |
|---|---|
| **F-24** (B) | La durée d'une story est définie comme temps machine **plus** temps d'arbitrage (§ Légende) ; plus aucune date au-delà de ST-04, les jalons se calculent au recalibrage par cette somme (§ Calendrier). |
| **F-25** | Le recalibrage ne mesure que deux facteurs d'échelle (machine, arbitrage) et une part d'arbitrage sur ST-01 à ST-04, relevés par des horodatages posés dès G-4 ; le plafond en découle par une formule écrite (§ Recalibrage). |
| **F-26** | Le port `Store` est **asynchrone** d'office (ST-02, PL-8) : il couvre OPFS par worker comme IndexedDB, donc un échec iOS en S1-B ne change qu'un adaptateur, jamais le socle. |
| **F-27** | Les surfaces s'enregistrent dans `tests/support/surfaces/` (une ligne par surface), le signal « à l'ouverture » est posé en ST-04, un amendement du socle a une procédure, et ST-30 passe en voie D avec `wiring/ai.rs`. |
| **F-28** | ST-11 crée `arbitre tache-de-fond`, qui exécute un registre de tâches ; ST-19 y inscrit la synchro (arête ST-11 → ST-19) et ST-19/ST-20 vérifient la chaîne recalcul → synchro → téléphone. |
| **F-29** | Spikes et gestes comptent dans le plafond comme les stories ; G-3 est un cycle fort de la voie D, pas une attente ; S3 est coupé pour ne plus chevaucher une story forte. |
| **F-30** | Borne de S3 : sans verdict quand ST-16 est fusionnée, ST-17 (CalDAV) s'engage à la place de ST-10 et ST-15b. |
| **F-31** | En repli, ST-17 reprend explicitement tous les critères de ST-10 et ST-15b, dont PROP-14 côté adaptateur et un essai réel de visibilité (R-4). |
| **F-32** | Le consentement IA est une commande hors catalogue (`arbitre ia consentir`), exigeant un terminal interactif, donc absente du MCP ; sans lui, tout fournisseur distant est « non configuré ». |
| **F-33** | Arêtes ajoutées : ST-12, ST-16, ST-08 → ST-20 ; ST-06, ST-08, ST-13 → ST-21 ; ST-31 → ST-32 ; ST-11 → ST-19. |
| F-34 (m) | Le relais crée le sel à son initialisation, atomiquement ; les appareils le lisent (ST-19). |
| F-35 (m) | Les jalons sur simulateur sont nommés comme tels ; la valeur sur agenda réel est rattachée à ST-10 ou ST-17. |
| F-36 (m) | Chaque cycle déclare ses fichiers par zone dans `.ctdd/stories/ST-xx/zones.toml` ; un fichier non listé prend le seuil Full. |
| F-37 (m) | La conformance de ST-04 coupe aussi le réseau sous les actions déclarées réseau : issue explicite exigée, jamais blocage ni panique. |
| F-38 (m) | AMB-16 est retiré de ST-22 : tranché en ST-03, rouvrir est un amendement du socle. |
| **F-39** | Aucune date absolue : la soirée 1 est la première soirée après le verdict VALIDÉ. |

## Résolution des failles de la revue v1 (v2, inchangé sauf mention)

| Faille | Résolution en une ligne |
|---|---|
| **F-1** (B) | Trois voies nommées après ST-04 (planning, mobile et socle étendu, diff). Les collisions sont listées dans un tableau qui désigne un propriétaire et une règle de fusion pour chacune. Le plafond est de 2 voies en cycle et de 8 points d'arbitrage par soirée (§ Parallélisme). |
| **F-2** (B) | Les tailles sont exprimées en soirées (S = ⅓, M = ⅔, L = 1), avec une charge d'arbitrage en points par story. Le chiffre de PL-1 est retiré. *v3 : les jalons ne sont plus datés au-delà de ST-04 (F-24).* |
| **F-3** (B) | ST-17 (CalDAV) ne dépend plus que de ST-09 et de ST-15a, qui sont des fonctions pures sur simulateur. Le repli s'exécute sans rien de Google. |
| **F-4** | ST-11, ST-12, ST-13, ST-14, ST-15a et ST-16 tournent sur le simulateur d'`Agenda`. Google n'apparaît que dans deux feuilles du graphe, ST-10 (lecture) et ST-15b (écriture). |
| **F-5** | ST-07 (MCP) dépend de ST-04 et de S5, sans attendre la pile. ST-27 et ST-32 déclarent l'arête ST-07. |
| **F-6** | ST-03 fixe la table **complète** des 5 catégories et de leurs codes (succès, refus ou violation, usage, erreur d'outil, module non activé). ST-04, ST-22 et ST-24 l'utilisent sans la modifier. La table est gelée : la changer est un amendement du socle. |
| **F-7** | ST-19 dépend de ST-18, ST-04 (crochet `revalider`) et S6. La convergence est prouvée sur le module factice. La question sur un élément invalide reste dans ST-13. |
| **F-8** | Les spikes sont placés par voie. S4 passe avant la voie diff, S5 dans le socle. *v3 : S1-A valide le port asynchrone au lieu de choisir sa forme (F-26).* |
| **F-9** | ST-22 est la première story ouverte après le recalibrage, quel que soit le plafond (PL-9), très en amont de la fenêtre Team (mi-2027). Le garde-fou des 10 entretiens n'est pas touché. |
| **F-10** | L'arbitrage de Quentin est appliqué : la voie diff tourne en parallèle dès ST-04, et le planning passe d'abord en cas de conflit de capacité (règle de priorité, § Parallélisme). |
| **F-11** | ST-03 est réduite au catalogue, à la CLI et aux codes de retour. L'export et l'import passent en ST-04b, après les espaces de noms ; INV-24 y est rattaché. |
| **F-12** | Le test de conformité générique de ST-04 exécute toute action non réseau de tout module réseau coupé (PROP-1). Il est rejoué automatiquement à chaque action ajoutée. AC-1.1 est couvert. |
| **F-13** | Modèle par défaut fixé en ST-18 : la même phrase secrète est saisie une fois par appareil, avec un sel public sur le relais et une clé dérivée gardée au trousseau. Le cas « sans session » est validé par S6 puis vérifié en ST-19. Le cas de la PWA est marqué `[À DÉFINIR]` en ST-20. |
| **F-14** | Le spike S6 (trousseau depuis launchd et systemd) passe avant ST-11. ST-11 reçoit un critère : jeton inaccessible ou expiré ⇒ recalcul sans lecture, capacité « non à jour », signal visible à l'ouverture. |
| **F-15** | ST-31 ne se déclenche que par une commande locale explicite et jamais dans l'Action. L'avertissement de CONFLIT-16 et la vérification PROP-2 y sont ajoutés. *v3 : consentement hors MCP (F-32).* |
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
- **CalDAV** (ST-17) : la story est différable. Elle s'engage à la demande d'un utilisateur réel, ou en repli (S3 refusé ou sans verdict à la fusion de ST-16, F-30), et elle ne dépend plus de Google (F-3).
- **Crate `arbitre-relay`** : le relais devient un module de `arbitre` (cli), extrait le jour où une autre surface en a besoin (PL-3). La frontière tient par le type `Enveloppe`.
- **Simulation fine du calendrier** : seul le socle est ordonnancé soirée par soirée. Au-delà, le plan donne l'ordre, les durées estimées et une règle d'ouverture ; les dates sortent du recalibrage (F-24).

### Under-engineering détecté
- **`.git` imbriqué dans `crates/core`** : à supprimer avant tout commit (G-0).
- **`rusqlite` et `chrono/clock` dans le cœur** : retirés en ST-01, puis déplacés vers `arbitre-sqlite` en ST-02.
- **Port `Store` synchrone contre stockage navigateur** : OPFS passe par un worker, IndexedDB est asynchrone, et l'échange UI ↔ worker l'est toujours. Un port synchrone gelé en ST-02 pourrait ne pas être implémentable côté wasm. **v3 : le port est asynchrone d'office (PL-8).** C'est la seule forme qui couvre les trois montages (OPFS par worker, IndexedDB, cœur dans le worker) ; l'adaptateur natif rend des futures immédiates. Le verdict iOS de S1-B ne choisit plus qu'un adaptateur (F-26).
- **Un seul fichier SQLite pour la CLI, le MCP et le recalcul** : WAL et délai d'attente sur verrou. ST-02 le vérifie avec deux processus réels.
- **Artefacts C-TDD uniques** (nouveau) : `.ctdd/cadre/01_story.md` à `08_*.md` et `state.json` entreraient en collision entre deux cycles parallèles. Chaque story mène donc son cycle dans `.ctdd/stories/ST-xx/` (G-4).
- **Numérotation des décisions** (nouveau) : `docs/decisions/0001-…` entrerait en collision entre voies. Les décisions sont nommées par story (`ST-xx-<sujet>.md`).
- **Protection contre l'envoi de secrets côté GitHub** : à activer avant le premier push (G-1).

### Risques identifiés

| # | Risque | Probabilité | Impact | Mitigation |
|---|--------|-------------|--------|------------|
| R-1 | Google refuse, bride ou ne répond pas (admin Workspace, utilisateurs non vérifiés) | Moyenne | ST-10 et ST-15b bloquées | S3-L à la 1re soirée. Repli ST-17 (CalDAV), qui ne dépend que de ST-09 et de ST-15a (F-3), engagé sur refus **ou** sans verdict quand ST-16 est fusionnée (F-30). Toute la logique planning avance sur simulateur pendant l'attente (F-4) |
| R-2 | SQLite-WASM + OPFS instable sur Safari iOS | Haute | ST-20 | Port `Store` asynchrone d'office (PL-8) : un échec iOS ne touche que l'adaptateur. S1-B valide sur appareils avant ST-20. Repli IndexedDB derrière le port |
| R-3 | Parseur de Ruff seulement en dépendance git | Moyenne | Maintenance de la voie diff | S4 avant ST-22 ; repli tree-sitter-python |
| R-4 | Le titre « Focus — X » ou le marqueur fuit vers les collègues | Moyenne | Viole AC-11.1/11.2 | Critères (5) et (6) de S3 obligatoires avant ST-15b, essai équivalent en ST-17 en repli (F-31) ; PROP-14 bloquant en ST-15a **et** dans l'adaptateur (ST-15b ou ST-17) |
| R-5 | **Saturation d'arbitrage** : trois voies qui posent des questions en même temps, perte de fil entre cycles | Haute | Arbitrages bâclés, décisions incohérentes entre voies | Plafond de 2 voies en cycle, budget de 8 points par soirée, une seule story à charge forte à la fois ; recalibrage après ST-04 |
| R-6 | Glissement vers un troisième module avant validation marché | Moyenne | Temps perdu | Garde-fou RG-G8 en fin de plan |
| R-7 | Fusion champ par champ qui produit un élément invalide (CONFLIT-1) | Haute dès la synchro | Données incohérentes | Crochet `revalider` en ST-04, marquage en ST-05, question en ST-13 |
| R-8 | Recalcul en parallèle sur deux ordinateurs | Moyenne | Agenda pollué | Identifiants déterministes (ST-13, ST-14), PROP-16 (ST-15a) |
| R-9 | `rmcp` jeune | Moyenne | ST-07 | S5 en 1re soirée ; repli JSON-RPC stdio |
| R-10 | **Conflits de fusion entre voies** (`Cargo.lock`, `arbitre`, `lib.rs`) | Haute | Temps d'intégration, régressions | Propriétaires et règles de fusion (§ Collisions), une seule story fusionnée à la fois, `Cargo.lock` régénéré et jamais fusionné à la main |
| R-11 | **Trousseau inaccessible en tâche de fond** (launchd hors session, systemd sans D-Bus) | Moyenne | AC-13.7 et synchro planifiée échouent en silence | S6 avant ST-11 ; repli fichier à droits restreints (archi) ; échec signalé à l'ouverture |
| R-12 | **Annotation du harnais** (30 à 50 cas à la main) sous-estimée | Moyenne | Voie diff à l'arrêt après ST-25 | G-3 compté comme un cycle fort de la voie D (6 points), dans le plafond (F-29). Les agents pré-étiquettent, Quentin valide |
| R-13 | **Contournement du consentement IA** par un assistant connecté au MCP | Moyenne | Code envoyé à un fournisseur distant sans accord humain (CONFLIT-16) | Consentement hors catalogue, terminal interactif exigé, absent du MCP (ST-30, F-32) |

## Gestes hors story (sans cycle C-TDD)

| # | Geste | Quand | Charge |
|---|---|---|---|
| G-0 | Supprimer `crates/core/.git` et `crates/core/.gitignore` | Avant le premier commit (soirée 1) | faible |
| G-1 | Activer la protection contre l'envoi de secrets sur GitHub | Avant le premier push (soirée 1) | faible |
| G-2 | Mener les entretiens design partners (`/user-interview`) | En continu. La démo ST-22 est disponible dès sa fusion | hors plan |
| G-3 | Valider l'annotation de 30 à 50 cas du harnais (pré-étiquetés par agent) | Entre ST-25 et ST-26, **dans la voie D, compté comme un cycle fort** (F-29) | forte + : 6 points |
| G-4 | Ranger les cycles par story : `.ctdd/stories/ST-xx/` avec son propre `state.json`, **qui porte les mesures** (ci-dessous) | Avant ST-01 | faible |

**Mesures posées par G-4 (F-25).** Dans chaque `state.json` de story, pour chaque phase du cycle : `debut`, `fin`, et une liste `portes` de paires `{ouverte, fermee}`. Une porte s'ouvre quand l'agent rend la main à Quentin (question, relecture, décision, test à la main) et se ferme quand Quentin répond. L'orchestrateur écrit ces horodatages à chaque passage de main, sans saisie de Quentin. On en tire, par story et par phase :
- **minutes d'arbitrage** `a` = somme des durées des portes ;
- **minutes machine** `m` = (`fin` − `debut`) − `a`, mutation comprise.

L'orchestrateur note aussi le début et la fin de chaque soirée dans `.ctdd/soirees.log` ; tout temps hors soirée (nuit, pause) est retiré de `a` et de `m`. Un `jq` sur les quatre fichiers du socle suffit au recalibrage.

## Spikes

Chaque spike rend un verdict écrit dans `.ctdd/spikes/Sn.md` et ne laisse aucun code en production. Il tourne dans son propre worktree. **Il compte dans le plafond comme une story de même charge** (F-29) : s'il est moyen ou fort, il occupe une place de cycle.

| Spike | Question (critères dans `03_archi.md`) | Place | Verdict requis avant | Taille | Arbitrage |
|---|---|---|---|---|---|
| **S3-L** | Lancement de S3 : créer le client OAuth personnel, solliciter l'admin Workspace (D-4), critères (1) à (4) | Socle, soirée 1 | — | S | faible (1) |
| **S3-V** | Verdict de S3 : réponse de l'admin, critères (5) et (6) vérifiés chez un collègue | Voie P, dès que l'admin répond, au premier début de story libre | ST-10 ; (5) et (6) avant ST-15b | S | forte (3) |
| **S5** | SDK MCP Rust | Socle, soirée 2 | ST-07 | S | faible (1) |
| **S1-A** | Le port `Store` **asynchrone** (PL-8) se branche-t-il sur un worker de stockage côté wasm ? Critères (1) et (6), plus (7) : poids du bundle avec la base des fuseaux | Socle, soirée 1 | **ST-02** | S | moyenne (2) |
| **S4** | tree-sitter-python ou parseur de Ruff | Socle, soirée 2 | ST-22 | S | moyenne (2) |
| **S6** | Secrets en tâche de fond : trousseau lisible depuis un agent launchd et depuis un timer systemd utilisateur sans session D-Bus ; repli fichier `0600` | Voie P, avant ST-11 | ST-11, ST-19 | S | faible (1) |
| **S1-B + S2** | Critères (2) à (5) de S1 sur appareils ; PWA Leptos installable et hors ligne, servie par le relais (D-6). Un échec iOS choisit l'adaptateur IndexedDB, sans toucher au port | Voie M, avant ST-20 | ST-20 | M | forte (4) : installation sur iOS et Android, à la main |

**Borne de S3 (F-30).** Trois issues :
- accepté : S3-V, puis ST-10 et ST-15b en fin de voie P ;
- refusé : ST-17 s'engage à la place de ST-10 et ST-15b ;
- **pas de verdict au moment où ST-16 est fusionnée** : ST-17 s'engage de même. Si l'admin répond plus tard, ST-10 et ST-15b redeviennent des stories différables, engagées à la demande.

## Stories

**Légende.**
- **Zone** : classification DDD de `03_archi.md`.
  - **Full** = Full C-TDD (mutation ≥ 85 %, CI > 0,95, propriétés massives).
  - **Std** = C-TDD standard (mutation ≥ 70 %).
  - **Gen** = Classic AI-Assisted, avec les tests de contrat ou de propriétés indiqués.
- **Taille** : temps machine du cycle complet mené par agents. S = 50 min, M = 100 min, L = 150 min (⅓, ⅔ et 1 soirée de 150 min). Ce temps comprend la mutation `--in-diff`.
- **Arbitrage** : temps de Quentin sur tout le cycle (questions, relectures, décisions, tests à la main). faible = 1 point, moyenne = 2, forte = 4. 1 point vaut 15 minutes.
- **Durée** (F-24) : un cycle attend Quentin à chaque porte, et aucun agent n'avance cette story pendant ce temps. La durée d'une story dans sa voie est donc **taille + arbitrage**. Exemples : S faible = 65 min ; M moyenne = 130 min ; M forte = 160 min ; L forte = 210 min. Une voie avance de la somme des durées de ses stories. C'est la seule unité de calendrier du plan.
- **Voie** : Socle, P (planning), M (mobile et socle étendu), D (diff).
- **« Sans valeur seule »** : la story ne sert l'utilisateur qu'avec la suivante. Aucun jalon n'est posé sur elle (F-20).

---

### Bloc A — Socle local (voie unique, soirées 1 à 4)

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
    - script de seuil par zone (85 / 70), qui lit la zone de chaque fichier dans `.ctdd/stories/ST-xx/zones.toml` (motifs de chemins → zone), écrit par le plan de la story ; **un fichier non listé prend le seuil Full** (F-36) ;
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
- **Dépendances** : ST-01, **verdict S1-A** (le port asynchrone se branche sur un worker).
- **Livrable** :
  - **port asynchrone** (`async fn` dans le trait, PL-8) ; l'adaptateur SQLite rend des futures immédiates ;
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
  - **test de conformité générique** : activation, invisibilité, espace de noms, parité, et **toute action `requiert_réseau = aucun` réussit réseau coupé** (PROP-1) ; **toute action déclarée réseau, réseau coupé, rend une issue explicite (code de la table de ST-03 ou mode dégradé nommé), sans blocage ni panique** (F-37) ;
  - **registre des surfaces** (F-27) : la parité itère sur `tests/support/surfaces/`, un fichier par surface et une ligne d'enregistrement par surface. ST-04 y pose `cli.rs`. Une surface nouvelle (MCP, GUI) s'ajoute sans modifier le test générique ;
  - **signaux à l'ouverture** (F-27) : un espace de noms `arbitre.signaux` où une tâche de fond dépose un signal daté ; `main.rs` affiche les signaux non lus au début de toute commande, puis les marque lus. ST-11 et ST-19 s'en servent sans toucher `main.rs` ;
  - assertion PROP-2 : aucune sortie hors des destinations déclarées ;
  - **`Clock::fuseau()`** (arbitrage P-2 de l'archi ST-01, 2026-11-03) : `jiff` avec la base des fuseaux embarquée, wasm compris ; adaptateur natif dans la CLI, simulateur commandable.
- **Critère de complétion** :
  - un module désactivé est absent de l'aide et de la complétion ; son appel explicite ⇒ code « module non activé », état inchangé ;
  - une lecture hors espace de noms est impossible ;
  - le module factice passe la conformité, réseau coupé compris, y compris une action factice déclarée `relais` ;
  - un signal déposé par un processus séparé s'affiche à la commande suivante, une seule fois ;
  - après ST-04, `arbitre-core` n'est plus modifié que par la voie M (§ Collisions).

> **Recalibrage (fin de la soirée 4, environ 30 min, 2 points).** Voir § Recalibrage.

---

### Bloc A' — Socle étendu (voie M, sauf ST-07 en voie P et ST-30 en voie D)

#### ST-07 : L'assistant agit par MCP
- **Objectif** : Claude Code, ou un autre client MCP, appelle toute action du catalogue. Ses suppressions et déplacements sont journalisés.
- **Zone** : Supporting / Std ; Gen pour le transport. **Voie** : P (sa première utilisatrice, et elle n'écrit pas `arbitre-core`). **Taille** : M. **Arbitrage** : moyenne (essai à la main avec deux clients).
- **Couvre** : US-4 (AC-4.1, 4.5), US-6 (AC-6.5, partie MCP) · RG-4.1, 4.6, 4.7, 6.4 · INV-10 (CLI ⊆ MCP), 18, 19, 23, 28 · AMB-11.
- **Crates** : `arbitre` (`mcp/` ; la requête « journal de l'assistant » filtre le journal par auteur côté CLI, sans toucher `arbitre-core`).
- **Dépendances** : ST-04, **S5** (F-5 : plus de dépendance à la pile).
- **Critère de complétion** :
  - la parité MCP entre dans le test de conformité générique par **`tests/support/surfaces/mcp.rs`** et une ligne d'enregistrement (registre de ST-04), sans modifier le test : tout module passé et futur est couvert (F-27) ;
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
- **Dépendances** : ST-18, ST-04 (crochet `revalider`, signaux), **S6**, **ST-11** (`arbitre tache-de-fond`, F-28).
- **Livrable** (en plus du relais et de la synchro) :
  - la synchro s'inscrit au registre des tâches de fond de ST-11 (`arbitre/taches.rs`, une ligne), **avant et après** le recalcul : récupérer les écritures du téléphone, recalculer, publier le résultat ;
  - **sel public créé par le relais** à sa première initialisation, atomiquement ; les appareils le lisent et ne le créent jamais (F-34).
- **Critère de complétion** :
  - simulateur à deux appareils, un relais, connectivité commandable et horloge ⇒ propagation < 60 s, reprise seule après 3 jours, date de dernière synchro affichée ;
  - une fusion qui rend une donnée du module factice invalide ⇒ `revalider` appelé, donnée marquée ;
  - **synchro lancée sans session interactive** (clé lue au trousseau selon S6) ; clé inaccessible ⇒ signal déposé (mécanisme de ST-04), affiché à l'ouverture suivante ;
  - **chaîne de bout en bout** (F-28) : un `arbitre tache-de-fond` lancé hors session sur l'appareil A produit un recalcul visible sur l'appareil B simulé, sans commande interactive sur A ;
  - deux appareils qui initialisent la synchro en même temps ⇒ un seul sel, même clé ;
  - sans relais configuré ⇒ outil complet ;
  - le relais ne reçoit que des enveloppes.

#### ST-20 : Le téléphone consulte, hors ligne
- **Objectif** : la PWA, installée sans store et servie par le relais (D-6), affiche la pile, les jours, la charge et les questions, hors ligne et synchronisés.
- **Zone** : Gen. **Voie** : M. **Taille** : L. **Arbitrage** : forte (acceptation manuelle sur iOS et Android).
- **Couvre** : US-2 (AC-2.2, 2.5), C-2.9 · RG-1.1 (téléphone), 2.5 · D-1, D-6 · PROP-1 (PWA).
- **Crates** : **`arbitre-web` (naît)**, adaptateur `Store` navigateur (selon S1), `arbitre` (`relay serve --pwa <dir>`, PL-3) ; job CI `web` dans son propre fichier de workflow.
- **Dépendances** : ST-19, **S1-B + S2**, **ST-16 et ST-08** (F-33). `arbitre-web` compile `arbitre-plan` en wasm : c'est une dépendance de compilation, pas seulement de format. ST-16 ferme la chaîne ST-05 → ST-06 → ST-09 → ST-11 → ST-12 (les jours) → ST-13 → ST-14 → ST-15a → ST-16 ; ST-08 est hors de cette chaîne.
- **`[À DÉFINIR]`** : où la PWA garde la clé dérivée (stockage de l'origine, risque documenté, ou saisie à chaque ouverture).
- **Critère de complétion** :
  - installation sur iOS et Android ; ouverture hors ligne ;
  - écriture distante visible en moins de 60 s ;
  - **un recalcul en tâche de fond sur l'ordinateur apparaît sur le téléphone réel sans que la CLI soit ouverte** (F-28, CONFLIT-7) ;
  - aucune sortie vers l'agenda ni vers une IA depuis la PWA.

#### ST-21 : Le téléphone agit
- **Objectif** : créer, modifier, faire, réordonner et répondre aux questions depuis le téléphone, avec le même résultat que la CLI.
- **Zone** : Gen (UI) ; parité en Supporting / Std. **Voie** : M. **Taille** : M. **Arbitrage** : moyenne.
- **Couvre** : US-3 (AC-3.1), US-6 (AC-6.5 complet) · RG-3.1, 6.4 · INV-10 (GUI ⊆ CLI), 28 · PROP-7 (côté GUI).
- **Crates** : `arbitre-web`.
- **Dépendances** : ST-20, **ST-06** (réordonner), **ST-08** (durée proposée), **ST-13** (questions) (F-33 ; déjà couvertes par ST-20, déclarées pour que l'arête survive à tout réordonnancement).
- **Critère de complétion** :
  - parité du catalogue étendue à la GUI par **`tests/support/surfaces/gui.rs`** et une ligne d'enregistrement, sans modifier le test générique (GUI ⊆ CLI : une action ajoutée côté CLI ne casse pas le test) (F-27) ;
  - même entrée ⇒ même élément, même durée proposée, mêmes signalements ;
  - **écritures concurrentes de la CLI et de la PWA sur un même élément ⇒ convergence par la synchro et conflit signalé selon RG-2.4** (F-21).

#### ST-30 : Adaptateur IA multi-fournisseurs
- **Objectif** : un réglage unique choisit Claude, GPT, Mistral ou un modèle local, et l'absence de fournisseur ne casse rien.
- **Zone** : Gen (contrat) ; Supporting / Std (dégradation) ; **consentement : Supporting (sécurité) / Full**. **Voie** : **D** (story du socle `[S-7]`, placée juste avant sa première utilisatrice ST-31 ; F-27). **Taille** : M. **Arbitrage** : **moyenne** (le consentement est un sujet de sécurité). **Sans valeur seule** (première utilisatrice : ST-31).
- **Couvre** : US-4 (AC-4.2, 4.3, 4.4) · RG-4.2 à 4.5 ; RG-1.2 (sur sollicitation seulement) · INV-16, 17 · PROP-8, 9 ; PROP-2 rejoué · **CONFLIT-16 (consentement, F-32)**.
- **Crates** : **`arbitre-ai` (naît)**, `arbitre` (**`wiring/ai.rs`**, sous-commande `ia consentir` dans `main.rs`). Le port `Ia` existe depuis ST-04 : `arbitre-core` n'est pas modifié.
- **Dépendances** : ST-04.
- **Consentement (F-32)** :
  - `arbitre ia consentir <fournisseur>` est une sous-commande **hors catalogue**, comme `relay serve` : elle n'existe ni au MCP ni dans la GUI, et aucune action du catalogue ne l'écrit ;
  - elle refuse de tourner si l'entrée standard n'est pas un terminal interactif, affiche l'avertissement de CONFLIT-16 (du code part chez ce fournisseur) et attend une saisie explicite ;
  - le consentement est propre à l'appareil, gardé hors du `Store` (donc jamais synchronisé ni écrit par l'assistant), et nomme le fournisseur : en changer exige un nouveau consentement ;
  - sans consentement, tout fournisseur **distant** est vu comme `NonConfiguré` (erreur gelée du port `Ia`, aucun amendement du socle), avec un message qui renvoie à la commande. Un modèle local n'en a pas besoin ;
  - limite assumée : un agent qui a un shell sur la machine peut déjà lire et envoyer le code lui-même. La barrière vise la surface MCP, pas un agent qui a les droits de l'utilisateur.
- **À trancher au cycle** : AMB-13.
- **Critère de complétion** :
  - changement de fournisseur ⇒ données et catalogue identiques ;
  - injoignable, clé refusée ou délai dépassé ⇒ erreur explicite pour les seules fonctions dépendantes ;
  - aucun appel automatique ;
  - **appel par MCP sans consentement ⇒ `NonConfiguré`, zéro sortie réseau (observateur PROP-2)** ;
  - `ia consentir` sans terminal ⇒ refus, rien d'écrit ; absente de la liste des outils MCP.

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
- **Crates** : `arbitre-plan` (`capacity`, **port `Agenda` lecture et écriture + simulateur**, propriété de la voie P ; erreurs du port : injoignable, identifiants indisponibles ou expirés, refus).
- **Dépendances** : ST-06.
- **À trancher au cycle** : AMB-2, RENVOI-8.
- **Critère de complétion** :
  - capacité = 450 − union des réunions comptées, plancher 0 ; week-end et absence à 0 ;
  - événement marqué par l'outil non compté ;
  - débordement et excès présents en humain et en JSON.

#### ST-11 : Recalcul chaque matin, outil fermé
- **Zone** : Core / Full (idempotence) ; Gen (launchd et systemd). **Taille** : M. **Arbitrage** : moyenne.
- **Couvre** : US-13 (AC-13.2 : relecture et capacités ; **AC-13.7**) · RG-13.4 (lecture), 13.6, 13.7 · PROP-19 (partie), 21 · INV-56 (idempotence) · CONFLIT-7 · RG-G7.
- **Crates** : `arbitre-plan` (orchestration `recalcul` sur le port `Agenda`), `arbitre` (`scheduler/`, `arbitre plan schedule install`, **`taches.rs`**).
- **Dépendances** : ST-09, **S6** (F-4 : plus de dépendance à ST-10).
- **Livrable** (F-28) : le job système lance une seule commande, **`arbitre tache-de-fond`**, qui exécute dans l'ordre les tâches inscrites dans `arbitre/taches.rs` (une ligne par tâche, ajout en fin, comme `modules.rs`). ST-11 y inscrit `plan.recalcul` ; ST-19 y ajoute la synchro. Un échec de tâche dépose un signal (mécanisme de ST-04) et n'arrête pas les suivantes.
- **À trancher au cycle** : RENVOI-7.
- **Critère de complétion** :
  - deux recalculs successifs ⇒ le second est sans effet ;
  - horloge simulée avec cycles marche/arrêt ⇒ rattrapage au démarrage ;
  - **jeton inaccessible ou expiré ⇒ recalcul sans lecture d'agenda, capacité « non à jour », signal visible à l'ouverture suivante, jamais d'échec silencieux** (F-14). Le port `Agenda` posé en ST-09 porte l'erreur « identifiants indisponibles ou expirés » ;
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
- **Dépendances** : ST-09, **S3-V accepté**. Peut s'engager **dès** ce verdict, au premier début de story libre de la voie P. Sans verdict à la fusion de ST-16 : voir la borne de S3 (F-30).
- **Critère de complétion** :
  - tests de contrat sur réponses enregistrées : occurrences dépliées, journée entière, `outOfOffice`, statut de réponse, `transparency`, **événement de l'outil déplacé** (pour ST-16) ;
  - agenda injoignable ⇒ dernière capacité connue « non à jour », avec sa date ;
  - observateur PROP-2 : seule sortie vers l'agenda connecté ;
  - jeton au trousseau, aucun secret dans le dépôt ;
  - **le job système installé en ST-11 lit l'agenda réel hors session**, jeton lu au trousseau (reste de F-14 v1).

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
- **Dépendances** : **ST-09, ST-15a** (F-3 : aucune dépendance à Google). S'engage à la demande d'un utilisateur réel, **ou en repli** : S3 refusé, ou sans verdict à la fusion de ST-16 (F-30). En repli, elle prend dans la voie P la place de ST-10 et de ST-15b, **taille L, arbitrage forte**.
- **Critère de complétion** :
  - la suite de contrat du port `Agenda` passe à l'identique sur le simulateur, sur CalDAV et, si elle existe, sur Google ;
  - **en repli, tous les critères de ST-10 et de ST-15b, transposés à CalDAV** (F-31) :
    - contrat sur réponses enregistrées : occurrences dépliées, journée entière, indisponibilité, statut de réponse, transparence, événement de l'outil déplacé ; PROP-12 (agendas multiples, changements, pannes) ; AC-8.5 ;
    - serveur injoignable ⇒ dernière capacité connue « non à jour », avec sa date ;
    - identifiants CalDAV au trousseau, aucun secret dans le dépôt ; le job système de ST-11 lit l'agenda hors session ;
    - **l'adaptateur refuse toute opération sur un événement sans marqueur** (PROP-14 en défense en profondeur) ;
    - **essai réel de visibilité (R-4)** : sur un agenda partagé, un autre compte voit le titre « Focus — X » selon le partage, et ne voit ni le marqueur ni rien d'autre (AC-11.1, 11.2 ; équivalent des critères (5) et (6) de S3) ;
    - essai réel : blocs posés, puis retirés quand l'élément est fait ; observateur PROP-2.

---

### Bloc D — Decision diff (voie D)

> La voie démarre dès ST-04 (arbitrage F-10). ST-22 est le quickwin du plan du 2026-10-02 (« 3 violations dans ton repo »), montrable en entretien (G-2).

#### ST-22 : `arbitre diff check` — frontières d'import
- **Zone** : Core / Full (vérification, verdict) ; Supporting / Std (extracteur Python). **Taille** : L. **Arbitrage** : forte (MANQUE-2, CONFLIT-17, essais OSS, démo).
- **Couvre** : US-14 (AC-14.1 à 14.4), US-15 (AC-15.1 en local, 15.2 à 15.5) · RG-14.1 à 14.6 (frontière d'import), 15.1 à 15.6 ; RG-3.2 (`arbitre diff`) · INV-11, 58 à 62, 63 (partie), 64 à 69 · AMB-14, 15 · CONFLIT-17 · D-2. AMB-16 est tranché et gelé en ST-03 : le rouvrir est un amendement du socle (F-38).
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
- **Déclencheur** : **commande locale explicite uniquement** (`arbitre diff decisions --ia`, ou l'action MCP correspondante, déclarée `requiert_réseau: ia`). L'Action CI ne l'appelle jamais : il n'y a pas de clé en CI (D-2) ni d'appel automatique (CONFLIT-7/8). **Un fournisseur distant n'est sollicité qu'après le consentement humain donné par `arbitre ia consentir` (ST-30)** ; un appel MCP ne peut ni le donner ni l'acquitter (F-32).
- **Critère de complétion** :
  - fournisseur simulé absent ou défaillant ⇒ décisions structurelles intactes, verdict inchangé ;
  - l'avertissement de CONFLIT-16 est celui de `ia consentir` : il est vu et accepté par un humain au terminal avant le **premier** envoi de code à un fournisseur distant ;
  - **appel MCP sans consentement ⇒ décisions structurelles seules, message renvoyant à `arbitre ia consentir`, zéro sortie vers le fournisseur** ;
  - observateur PROP-2 : sortie uniquement vers le fournisseur consenti, uniquement sur la commande.

#### ST-32 : Invariants candidats, preuves à l'appui
- **Zone** : Supporting / Std ; Core / Full pour INV-81 et 82. **Taille** : L. **Arbitrage** : moyenne.
- **Couvre** : US-18 (AC-18.1 à 18.5) · RG-18.1 à 18.6 · INV-81 à 84 · PROP-23, 24 ; PROP-2 (rejoué) · CONFLIT-16 (réutilise l'avertissement de ST-31), MANQUE-3.
- **Crates** : `arbitre-diff` (`candidates`).
- **Dépendances** : ST-30, ST-23, **ST-07** (F-5), **ST-31** (avertissement et consentement réutilisés, F-33).
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
| **Socle** (soirées 1–4) | ST-01 → ST-02 → ST-03 → ST-04 | tout, puis plus rien après ST-04, sauf amendement du socle (ci-dessous) |
| **P — planning** | ST-05 → ST-06 → ST-07 (MCP) → ST-08 → ST-09 → S6 → ST-11 → ST-12 → ST-13 → ST-14 → ST-15a → ST-16 → S3-V → ST-10 → ST-15b (ST-17 à la place de S3-V, ST-10 et ST-15b si S3 est refusé ou sans verdict, F-30) | `arbitre-plan`, `arbitre-calendar`, `arbitre/{wiring/plan.rs, mcp/, scheduler/, taches.rs}`, `tests/support/surfaces/mcp.rs` |
| **D — decision diff** | ST-22 → ST-23 → ST-24 → ST-25 → G-3 → ST-26 → ST-27 → ST-28 → ST-29 → ST-30 → ST-31 → ST-32 | `arbitre-diff`, **`arbitre-ai`**, `arbitre/{wiring/diff.rs, wiring/ai.rs}`, `action.yml`, workflow d'autotest |
| **M — mobile et socle étendu** | ST-04b → ST-18 → ST-19 → S1-B + S2 → ST-20 → ST-21 | **`arbitre-core` après ST-04**, `arbitre-web`, `arbitre/{relay/, sync}`, `tests/support/surfaces/gui.rs`, workflow `web` |

Arêtes entre voies, **toutes déclarées dans les stories** (F-33) : l'ordre ne repose que sur elles, jamais sur un ordonnancement.
- ST-07 (P) → ST-27, ST-32 (D) ;
- ST-11 (P) → ST-19 (M) ;
- ST-08, ST-16 (P) → ST-20 (M) ; ST-06, ST-08, ST-13 (P) → ST-21 (M).

### Plafond et règles d'ouverture (F-29)

Le plafond vaut avant le recalibrage (valeurs par défaut) et après (valeurs recalculées). L'orchestrateur n'ouvre un élément que si toutes les règles tiennent :
1. **Un cycle** est tout élément d'arbitrage moyen ou fort — story, spike **ou geste** —, de sa première phase à sa fusion ou à son verdict. G-3, S3-V et S1-B + S2 sont des cycles comme les autres. Un élément faible n'est pas un cycle, mais compte dans les points.
2. **Au plus N cycles en même temps.** N = 2 par défaut ; recalculé au recalibrage.
3. **Au plus un cycle fort en cours**, tous types confondus.
4. **Points de la soirée ≤ budget** : 8 par défaut ; recalculé au recalibrage.
5. **Une attente purement externe n'occupe aucune place** : seul S3 entre S3-L et la réponse de l'admin est dans ce cas. Rien d'autre n'est une attente.
6. **Priorité quand une place se libère** : P, puis D, puis M (arbitrage F-10), avec une seule exception, ST-22 ouverte en premier après le recalibrage (PL-9). Une voie dont l'élément suivant attend une arête d'une autre voie cède sa place à la suivante dans l'ordre de priorité.
7. Une voie qui attend un arbitrage n'en prend pas un autre en parallèle : elle attend.

Conséquence pour N = 2 : P et D tournent ensemble après le recalibrage ; M s'ouvre quand l'une des deux se termine. Pour N = 1 : ST-22, puis toute la voie P, puis le reste de D, puis M. Pour N = 3 : les trois voies ensemble, toujours sous les règles 3 et 4.

### Amendement du socle (F-27)

Tout changement d'un contrat gelé (types de contrat d'`arbitre-core`, table des codes, test de conformité générique hors registre des surfaces, `Cargo.toml` racine) est un **amendement du socle** :
- c'est un mini-cycle `ST-Ax`, mené par la voie qui en a besoin, dans son propre worktree, avec un arbitrage **fort** ;
- dès qu'il est demandé, plus aucun nouvel élément ne s'ouvre dans aucune voie. Les cycles en cours finissent, puis l'amendement s'ouvre (règle 3 : il attend le cycle fort en cours) ;
- une fois fusionné, chaque voie se rebase au début de sa story suivante.

### Points de collision et règles

| Point de collision | Voies qui y touchent | Règle |
|---|---|---|
| `arbitre-core` | M seule après ST-04 (ST-04b `export`, ST-18 `envelope`, ST-19 `Transport`) | Ajouts dans de nouveaux fichiers. Dans `lib.rs`, une ligne `mod` ajoutée en fin. Les types de contrat (`DescripteurAction`, `Sortie`, catégories d'erreur, `Module`, `Contexte`, port `Ia`, port `Store`) sont **gelés**. Les changer est un amendement du socle |
| Crate `arbitre` (cli) | P, D, M | Un fichier par voie (`wiring/<module>.rs`, `wiring/ai.rs`, `mcp/`, `scheduler/`, `relay/`). Fichiers chauds, ajouts en fin de liste uniquement : `modules.rs` (une ligne en ST-05 et en ST-22), `main.rs` (une ligne par sous-commande hors catalogue : `mcp`, `schedule`, `tache-de-fond`, `relay serve`, `sync`, `ia consentir`), **`taches.rs`** (créé par P en ST-11 ; M y ajoute une ligne en ST-19) |
| Catalogue d'actions | P, D | Les noms sont préfixés par le module (`plan.*`, `diff.*`). Les tests de parité sont **génériques** : ajouter une action ne les modifie pas. GUI ⊆ CLI : une action CLI nouvelle ne casse pas la GUI |
| Table des codes de retour | aucune après ST-03 | Gelée (F-6) ; un changement est un amendement du socle |
| `Cargo.toml` racine | aucune en principe | Toutes les dépendances partagées sont déclarées en ST-01 (PL-6). Une nouvelle dépendance partagée est un amendement du socle. Chaque crate ajoute ses dépendances dans son propre `Cargo.toml` |
| `Cargo.lock` | toutes | Jamais fusionné à la main : sur conflit, prendre celui de `main` puis lancer `cargo check --workspace`, qui le régénère. Une seule story fusionnée à la fois |
| Configuration CI | M (workflow `web`), D (autotest de l'Action) | Les règles wasm32 et d'arêtes, écrites en ST-01, couvrent toutes les crates prévues. Les ajouts se font dans de **nouveaux** fichiers de workflow |
| Outillage de test partagé | toutes | Observateur réseau, mode coupé, module factice et test de conformité sont posés au socle (ST-04), dans `arbitre/tests/support/`, et gelés. **Seul point d'extension : `tests/support/surfaces/`**, un fichier par surface (P : `mcp.rs` en ST-07 ; M : `gui.rs` en ST-21) et une ligne d'enregistrement ajoutée en fin (F-27). Le simulateur d'`Agenda` appartient à P |
| `action.yml` | D seule (ST-24, ST-29) | Séquentiel dans la voie |
| `.ctdd/` et `docs/decisions/` | toutes | Un répertoire par story (G-4) ; décisions nommées par story |

**Protocole d'intégration** :
- chaque story vit dans son worktree et sa branche ;
- elle se rebase sur `main` juste avant de fusionner ;
- une seule fusion à la fois ;
- les autres voies se rebasent à leur prochain début de story, pas en cours de cycle.

## Calendrier

Unité : la **durée** d'un élément = taille + arbitrage, en minutes (§ Légende). Une soirée = 150 min. La soirée 1 est la **première soirée après le verdict VALIDÉ** de ce plan (F-39) ; aucune date absolue n'est posée.

### Socle (seule partie ordonnancée soirée par soirée)

Lecture : `élément (points)`. Les spikes et gestes comptent dans le plafond (N = 2, une seule forte, 8 points).

| Soirée | Voie Socle | Hors voie | Points | Cycles en cours |
|---|---|---|---|---|
| 1 | G-0, G-1, G-4 (1) · ST-01 (4), 150 de ses 160 min | S1-A (2) · S3-L (1) | 8 | ST-01 (forte), S1-A |
| 2 | ST-01 fin (10 min) · ST-02 (2) · ST-03 démarre à 140 min | S4 (2) · S5 (1) | 5 | ST-02, S4, puis ST-03 |
| 3 | ST-03 (4), fin à 150 min | — | 4 | ST-03 (forte) |
| 4 | ST-04 (2), fin à 130 min · **recalibrage** (2), 30 min | — | 4 | ST-04 |

Voie Socle : 160 + 130 + 160 + 130 = 580 min, soit 3,9 soirées. Les points de ST-03 sont comptés en soirée 3, où se tiennent ses portes. Si S3-V devient possible pendant le socle, il attend la fin du socle (règle 3 aux soirées 1 et 3, ouverture des voies ensuite).

### Après le socle : ordre et durées estimées, sans date

Durées estimées (minutes) et cumul dans la voie, **avant** recalibrage. Ce ne sont pas des dates : une voie n'avance que pendant qu'elle a une place de cycle, et la règle 3 peut la faire attendre.

| Voie P | T | A | Durée | Cumul | Voie D | T | A | Durée | Cumul | Voie M | T | A | Durée | Cumul |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| ST-05 | M | moy. | 130 | 130 | ST-22 | L | forte | 210 | 210 | ST-04b | M | moy. | 130 | 130 |
| ST-06 | S | faible | 65 | 195 | ST-23 | M | moy. | 130 | 340 | ST-18 | S | moy. | 80 | 210 |
| ST-07 | M | moy. | 130 | 325 | ST-24 | M | moy. | 130 | 470 | ST-19 | L | forte | 210 | 420 |
| ST-08 | S | faible | 65 | 390 | ST-25 | M | forte | 160 | 630 | S1-B + S2 | M | forte | 160 | 580 |
| ST-09 | M | moy. | 130 | 520 | G-3 | — | forte + (6) | 90 | 720 | ST-20 | L | forte | 210 | 790 |
| S6 | S | faible | 65 | 585 | ST-26 | L | moy. | 180 | 900 | ST-21 | M | moy. | 130 | 920 |
| ST-11 | M | moy. | 130 | 715 | ST-27 | M | moy. | 130 | 1030 | | | | | |
| ST-12 | M | faible | 115 | 830 | ST-28 | M | moy. | 130 | 1160 | | | | | |
| ST-13 | M | moy. | 130 | 960 | ST-29 | S | faible | 65 | 1225 | | | | | |
| ST-14 | M | forte | 160 | 1120 | ST-30 | M | moy. | 130 | 1355 | | | | | |
| ST-15a | M | moy. | 130 | 1250 | ST-31 | S | faible | 65 | 1420 | | | | | |
| ST-16 | M | moy. | 130 | 1380 | ST-32 | L | moy. | 180 | 1600 | | | | | |
| S3-V | S | forte (3) | 95 | 1475 | | | | | | | | | | |
| ST-10 | M | moy. | 130 | 1605 | | | | | | | | | | |
| ST-15b | M | moy. | 130 | 1735 | | | | | | | | | | |
| *repli : ST-17 au lieu de S3-V, ST-10, ST-15b* | L | forte | 210 | *1590* | | | | | | | | | | |

### Jalons (sans date ; datés au recalibrage)

| Jalon | Fin de | Voie, cumul estimé |
|---|---|---|
| Socle posé, **recalibrage** | ST-04 | Socle, soirée 4 |
| **Démo decision diff** (« 3 violations ») | ST-22 | D, 210 |
| Decision diff en CI | ST-24 | D, 470 |
| Planning sur CLI et assistant, **sur agenda simulé** (F-35) | ST-09 (avec ST-07) | P, 520 |
| Recalcul du matin outil fermé, **sur agenda simulé ; ne place encore rien** (F-35) | ST-11 | P, 715 |
| Planning complet sur simulateur (semaine, questions, blocs, déplacements) | ST-16 | P, 1380 |
| **Planning sur agenda réel** — la valeur quotidienne annoncée (F-35) | ST-15b (Google) ou ST-17 (CalDAV) | P, 1735 ou 1590 |
| **Decision diff complet** | ST-32 | D, 1600 |
| Synchro entre deux ordinateurs | ST-19 | M, 420 après l'ouverture de M |
| **Téléphone** (consultation puis action) | ST-20, ST-21 | M, 790 et 920 après l'ouverture de M |

### Recalibrage après ST-04 (F-24, F-25)

En fin de soirée 4, avant d'ouvrir les voies, environ 30 min. Il ne mesure **que** ST-01 à ST-04, avec les horodatages de G-4.

**1. Mesurer**, pour chaque story i de ST-01 à ST-04 :
- `m_i` = minutes machine ; `a_i` = minutes d'arbitrage (somme des portes), par phase et au total.

**2. Calculer trois nombres**, sur les sommes des quatre stories (pas de médiane : quatre valeurs, toutes M) :
- facteur machine `k_m = Σ m_i / Σ taille_i` (estimé : Σ taille = 400 min) ;
- facteur arbitrage `k_a = Σ a_i / Σ arbitrage_i` (estimé : Σ arbitrage = 180 min) ;
- part d'arbitrage `f = Σ a_i / Σ (m_i + a_i)` (estimé : 180 / 580 = 0,31).

L'échantillon ne contient ni S, ni L, ni « faible ». Le recalibrage ne leur invente donc pas de valeur propre : il applique **le même facteur** à toutes les tailles et à toutes les charges. C'est une hypothèse, écrite comme telle. Le biais des premiers cycles (plus lents) gonfle `k_m` et `k_a` : il va dans le sens prudent, et il est accepté.

**3. Déduire le plafond.**
- `B` = minutes d'arbitrage disponibles par soirée. Ce n'est pas une mesure : c'est une donnée déclarée par Quentin, 120 par défaut.
- Une voie en cycle demande en moyenne `150 × f` minutes de Quentin par soirée.
- **N = min(3, max(1, ⌊B / (150 × f)⌋))**. Avec les estimations : ⌊120 / 46,5⌋ = 2, le plafond de la v2.
- **Budget de points par soirée = B / (15 × k_a)**, en points estimés. Avec les estimations : 8.
- La règle 3 (une seule forte) ne change pas.

**4. Recalculer les durées** : durée recalibrée = `k_m × taille + k_a × arbitrage`. Les éléments sans équivalent mesuré suivent la même formule : G-3 et S1-B + S2 par `k_a` sur leur part humaine. Le délai de l'admin pour S3 n'est pas estimé : il est borné par la règle de la borne de S3.

**5. Dater les jalons.** On déroule les règles d'ouverture 1 à 7 sur les tableaux ci-dessus, avec N, le budget et les durées recalibrées. À chaque fin d'élément, on ouvre l'élément suivant autorisé ; un élément fort qui attend démarre à la fin du fort en cours. Une quarantaine de lignes, à la main ou par un petit script. La soirée d'un jalon est la soirée où se termine sa story. On convertit en dates au rythme choisi (2 ou 3 soirées par semaine) et on écrit les dates dans ce fichier. **Le déroulé ne peut pas dépasser le plafond, puisqu'il applique les règles qui le définissent.**

**6. S3** : si l'admin a refusé, ST-17 remplace S3-V, ST-10 et ST-15b dans la voie P dès maintenant. S'il n'a pas répondu, rien ne change : la borne joue à la fusion de ST-16.

Les mêmes horodatages continuent d'être relevés à chaque story. Le même calcul peut être refait plus tard sans rien ajouter ; ce n'est pas prévu par le plan.

## Points à trancher (niveau plan)

| # | Question | Recommandation (retenue par défaut) |
|---|---|---|
| PL-1 | Place de la voie mobile ? | **Troisième en priorité**, derrière P et D (arbitrage F-10 pour D). Avec N = 2, elle s'ouvre quand P ou D se termine ; avec N = 3, dès le recalibrage. Aucune date avant le recalibrage. |
| PL-2 | S3 lancé tôt ? | **Oui : S3-L à la 1re soirée**, en charge faible. C'est le seul délai externe du plan. Il ne bloque que ST-10 et ST-15b, deux feuilles du graphe, et il est borné par la fusion de ST-16 (F-30). |
| PL-3 | Relais en crate dédiée ou en module de la CLI ? | **Module de la CLI.** La PWA est servie depuis un répertoire (`relay serve --pwa <dir>`) : pas de build wasm dans le binaire, pas de couplage entre voies. Le serveur du relais reçoit tout le binaire `arbitre`, ce qui est accepté en v1. On extraira `arbitre-relay` si le poids ou la surface exposée deviennent un problème. |
| PL-4 | CalDAV (ST-17) : engagé d'office ou à la demande ? | **À la demande, ou en repli** (S3 refusé, ou sans verdict à la fusion de ST-16). Exécutable, puisque ST-17 ne dépend que de ST-09 et de ST-15a. |
| PL-6 | Dépendances partagées (F-17) | Retenu par défaut et consigné en ST-01 : **tokio** (exigé par rmcp, réutilisé partout), **reqwest + rustls** (Google, relais, IA ; pas d'OpenSSL), **keyring** (trousseau, avec repli fichier `0600`), **serde / serde_json**, **proptest** (propriétés), **cargo-mutants** (outil, hors `Cargo.toml`). Côté wasm : `fetch` du navigateur derrière le port `Transport`, sans reqwest. |
| PL-7 | Plafond : 2 cycles, 8 points par soirée, un seul cycle fort à la fois ; spikes et gestes compris | Retenu par défaut ; N et le budget sont recalculés après ST-04 par la formule du § Recalibrage. |
| PL-8 | Forme du port `Store` (F-26) | **Asynchrone d'office.** Seule forme qui couvre OPFS par worker, IndexedDB et le cœur dans un worker. Coût : `async` dans le trait et dans `Module::executer`, futures immédiates côté SQLite. S1-A vérifie le branchement sur un worker avant ST-02 ; S1-B ne choisit plus qu'un adaptateur. |
| PL-9 | Place de la démo ST-22 si le plafond tombe à 1 | **ST-22 est ouverte en premier après le recalibrage, quel que soit N.** Seule exception à P > D > M, pour tenir la conséquence de l'arbitrage F-10 sur F-9 (démo tôt pour les entretiens). |

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
| **PROP-1** (AC-1.1) | ST-04 (outillage + conformance), **rejoué automatiquement pour chaque action** de ST-05 à ST-16 et de ST-22 à ST-32 ; ST-20 (PWA) | Mode réseau coupé du test de conformité ; actions déclarées réseau : issue explicite exigée (F-37) |
| **PROP-2** | ST-04 (observateur), rejoué en ST-10, 15b, 17, 19, 30, **31**, 32 | Observateur réseau : destination et déclencheur ; **appel MCP sans consentement ⇒ zéro sortie IA** (ST-30, ST-31, F-32) |
| PROP-3, 4, 7 | ST-02 ; PROP-7 côté GUI en ST-21 (concurrence par la synchro) | Arrêt brutal, stockage plein, deux processus ; CLI contre PWA |
| PROP-10, 14, 15, 16 | ST-15a (pure + simulateur) ; PROP-14 aussi en ST-15b ou, en repli, ST-17 (adaptateur, F-31) | Origines mêlées, panne/reprise, deux appareils ; **PROP-14 bloquant** |
| PROP-12 | ST-10 ou, en repli, ST-17 | Agendas multiples avec changements et pannes |
| PROP-19, 21 | ST-11, complétées en ST-13 et ST-16 | Horloge, cycles marche/arrêt, jeton inaccessible ; chaîne recalcul → synchro → téléphone en ST-19 et ST-20 (F-28) |
| PROP-22 | ST-29 | Hébergeur simulé |
| C-2.9 | ST-20 | Acceptation manuelle iOS et Android |
| Seuils de mutation | ST-01 (outil), chaque review de story | `cargo mutants --in-diff`, seuil de la zone lue dans `zones.toml` de la story, Full par défaut (F-36) |

## Arbitrage de Quentin : Rust premier langage de decision diff (2026-11-03)

- **Rust en premier, Python juste après.** Le premier extracteur de decision diff est tree-sitter-rust, appliqué au développement d'OpenArbitre lui-même (dogfooding dès la première story de la voie D). L'extracteur Python (S4 : parseur de Ruff ou tree-sitter-python) devient la story suivante.
- **Conséquences, à découper au cycle de ST-22** : ST-22 = extracteur Rust + fichier d'invariants d'OpenArbitre (migration des règles de source de `graphe.py` ; les règles du graphe Cargo restent hors AST) vérifié par le juge avec le binaire de `main`, jamais celui de la PR. S4 passe juste avant l'extracteur Python. Essais OSS et démo glissent d'une story.
- **Condition** : le port `Extracteur` est neutre en langage dès le premier ; à vérifier au cycle de ST-22.
- Raison : decision diff est d'abord un besoin personnel ; l'outil qui juge ses propres PR est son meilleur test, au prix d'une story de retard sur la validation externe.
