# Plan d'implémentation — OpenArbitre, cadre global (découpe en stories)

## Traçabilité
- Sources : `.ctdd/cadre/02_specs.md`, `.ctdd/cadre/03_archi.md`, et les sections « Arbitrages de Quentin » de `.ctdd/cadre/01_story.md`, `02_specs.md` et `03_archi.md`, qui font foi. `.ctdd/cadre/inputs/plan-decision-diff-2026-10-02.md` sert de contexte pour la partie C.
- Date : 2026-11-02 · Statut : DRAFT · Cycle sans code.
- Ce plan **est** la découpe en stories. Chaque story `ST-nn` aura son propre cycle C-TDD complet (story → specs → archi locale → plan → DA → skeleton → implémentation → review). Les `[À DÉFINIR]`, `AMB-*` et `RENVOI-*` listés dans une story se tranchent au cycle de cette story.

## Contexte technique
- **Codebase** : workspace Cargo vide, sans aucun commit. `crates/core` (`arbitre-core`, fonction `add`) et `crates/cli` (`arbitre`, « Hello, world! »). Les anti-patterns relevés dans `03_archi.md` sont encore présents : `.git` imbriqué, `rusqlite` et `chrono/clock` dans le cœur.
- **Stack** : Rust (édition 2024), SQLite via rusqlite, Leptos CSR en PWA (wasm32), clap, MCP (`rmcp` selon S5), tree-sitter-python ou le parseur de Ruff (selon S4), RustCrypto (XChaCha20-Poly1305, Argon2id).
- **Patterns à respecter** (instaurés par l'archi) : journal d'écritures et projection, cœur fonctionnel dans une coquille impérative, ports et adaptateurs avec un simulateur par port, catalogue d'actions comme source unique des surfaces, identifiants déterministes, rang par indexation fractionnaire, réconciliation déclarative, rapports triés.
- **Garde-fous mécaniques** posés dès ST-01 : `cargo check --target wasm32-unknown-unknown` sur `arbitre-core` et `arbitre-plan`, contrôle des arêtes interdites sur `cargo metadata` (plan ↛ diff, diff ↛ plan, core/plan ↛ crate non wasm), gitleaks.

## Analyse pré-plan

### Over-engineering détecté
- **Téléphone (synchro + PWA) avant le planning.** L'ordre littéral « tout le socle d'abord » ferait passer ST-18 à ST-21 (chiffrement, relais, Leptos, OPFS, avec deux spikes à haut risque) avant la moindre valeur du planning. RG-G1 n'exige que les briques du socle **dont une story dépend**, et aucune story du planning ne dépend de la synchro. **Retenu** : le socle local (ST-01 à ST-04), puis le planning sur CLI et MCP, puis le socle mobile, puis decision diff. L'ordre de développement des **modules** reste socle → planning → decision diff. Trade-off : le téléphone arrive après le planning complet. Pendant ce temps, le planning s'utilise depuis l'ordinateur, en CLI et par l'assistant.
- **Durées améliorées par l'IA** (RG-4.4 la cite comme option) : aucune story. La proposition déterministe (ST-05, ST-08) suffit à AC-7.1 à 7.5. On l'ajoutera si les propositions se révèlent mauvaises à l'usage.
- **CalDAV** : l'arbitrage le place « ensuite ». ST-17 est en séquence mais **différable** : elle ne s'engage qu'à la demande d'un utilisateur réel. Le port `Agenda` garantit qu'elle ne coûtera alors qu'un adaptateur.
- **Crate `arbitre-relay`** : le relais ne sert aucune autre surface. Recommandation : un module de `arbitre` (cli), extrait en crate le jour où une autre surface en a besoin, selon la même règle que le serveur MCP dans `03_archi.md`. La frontière « le relais ne voit que des enveloppes » tient par le type d'entrée (`Enveloppe` opaque), pas par la crate. Point **PL-3**.
- **Adaptateur IA** : seules ST-31 et ST-32 s'en servent (RG-4.4). Il est donc placé juste avant elles (ST-30), en fin de séquence, et pas dans le socle initial.

### Under-engineering détecté
- **`.git` imbriqué dans `crates/core`** : au premier `git add`, `crates/core` entrerait comme gitlink vide, et le dépôt public serait cassé. Il faut supprimer `crates/core/.git` et `crates/core/.gitignore` **avant tout commit**, même avant ST-01. Geste G-0.
- **`rusqlite` et `chrono/clock` dans le cœur** : ils interdisent le wasm32 et l'injection du temps, que PROP-5, 19 et 21 exigent. Corrigé dans ST-01 (retrait) et ST-02 (déplacement vers `arbitre-sqlite`).
- **Un seul fichier SQLite partagé par la CLI, le MCP et le recalcul planifié** : sans WAL ni délai d'attente sur verrou, deux processus concurrents se renvoient `SQLITE_BUSY`, et l'erreur surfacerait comme un échec utilisateur. Couvert par ST-02 (PROP-7 avec deux processus réels, pas seulement deux fils).
- **Fuseaux horaires en wasm** (AMB-8) : la base des fuseaux embarquée pèse sur le bundle PWA. Il faut la mesurer dans S1 (critère 7, poids du bundle) et ne pas la découvrir en ST-20.
- **Protection contre l'envoi de secrets côté GitHub** : elle n'est pas activée, et le dépôt est public dès le premier commit. Geste G-1, avant le premier push.

### Risques identifiés

| # | Risque | Probabilité | Impact | Mitigation |
|---|--------|-------------|--------|------------|
| R-1 | Google refuse ou bride le client OAuth personnel (admin Workspace, utilisateurs non vérifiés) | Moyenne | Bloque ST-10, ST-15, donc le cœur de la valeur planning | S3 démarre **tôt** (après ST-01) : son verdict peut demander un geste d'admin Workspace au délai externe. Repli : CalDAV avancé (ST-17) |
| R-2 | SQLite-WASM + OPFS instable sur Safari iOS | Haute | Bloque ST-20 | S1 juste avant ST-20, repli IndexedDB derrière le port `Store` (archi) ; n'affecte rien d'autre |
| R-3 | Parseur de Ruff seulement en dépendance git, sans semver | Moyenne | Coût de maintenance de ST-22 et suivantes | S4 avant ST-22 ; repli tree-sitter-python (publié) |
| R-4 | Le titre « Focus — X » ou le marqueur fuit vers les collègues selon le partage de l'agenda | Moyenne | Viole AC-11.1/11.2, perte de confiance | S3 critères (5) et (6) obligatoires avant ST-15 ; PROP-14 bloquant |
| R-5 | Budget de 8 à 10 h par semaine : 32 stories étalées, perte de fil entre les cycles | Haute | Glissement, abandon | Stories petites et livrables seules ; la valeur du planning sur CLI arrive à ST-09 ; ST-22 est montrable en entretien |
| R-6 | Glissement vers un troisième module avant validation marché | Moyenne | Temps perdu (risque n° 1 du plan du 2026-10-02) | Garde-fou RG-G8 inscrit en fin de séquence, après ST-32 |
| R-7 | Fusion champ par champ qui produit un élément invalide (CONFLIT-1) | Haute dès la synchro | Données incohérentes | Crochet de revalidation posé dès ST-04 ; marquage en ST-05 ; question en ST-13, avant que la synchro (ST-19) ne rende le cas fréquent |
| R-8 | Recalcul lancé en parallèle sur deux ordinateurs : blocs ou questions en double | Moyenne | Agenda pollué | Identifiants déterministes (ST-13, ST-14) et PROP-16 (ST-15) |
| R-9 | `rmcp` jeune, liste d'outils dynamique mal gérée par les clients | Moyenne | Bloque ST-07 | S5 juste avant ; repli JSON-RPC stdio à la main |

## Gestes hors story (sans cycle C-TDD)

| # | Geste | Quand |
|---|---|---|
| G-0 | Supprimer `crates/core/.git` et `crates/core/.gitignore` | Avant le premier commit |
| G-1 | Activer la protection contre l'envoi de secrets sur le dépôt GitHub | Avant le premier push |
| G-2 | Mener les entretiens design partners (`/user-interview`) | En continu, dès maintenant ; le compteur conditionne tout module au-delà de planning et decision diff (garde-fou en fin de séquence) |

## Spikes

Chaque spike a un budget fixe, rend un verdict écrit (`.ctdd/spikes/Sn.md`) et ne laisse aucun code en production. Placement : **juste à temps**, sauf S3, à cause de son délai externe.

| Spike | Question (critères dans `03_archi.md`) | Placé | Doit rendre son verdict avant | Budget indicatif |
|---|---|---|---|---|
| **S3** | Google Calendar depuis une CLI open source, client OAuth personnel accepté par Workspace (D-4) | **Après ST-01**, en tâche de fond | ST-10 (lecture) ; critères (5) et (6) avant ST-15 | 1 à 2 soirs, plus l'attente de l'admin |
| **S5** | SDK MCP Rust | Après ST-06 | ST-07 | 1 soir |
| **S1** | SQLite WASM + OPFS (y compris le poids des fuseaux) | Après ST-19 | ST-20 | 2 soirs |
| **S2** | PWA Leptos hors ligne, installable, servie par le relais (D-6) | Après ST-19, avec S1 | ST-20 | 1 à 2 soirs |
| **S4** | tree-sitter-python ou parseur de Ruff | Après ST-21 | ST-22 | 1 à 2 soirs |

## Stories

Légende : **Zone** = classification DDD de `03_archi.md`. **Full** = Full C-TDD (mutation ≥ 85 %, CI > 0,95, propriétés massives) ; **Std** = C-TDD standard (mutation ≥ 70 %) ; **Gen** = Classic AI-Assisted (+ tests de contrat ou de propriétés indiqués). Effort : S ≤ 1 semaine, M ≈ 2 semaines, L ≈ 3 semaines et plus, au budget de 8 à 10 h par semaine.

---

### Bloc A — Socle local

#### ST-01 : Fusion pure — HLC, écritures, versions écartées
- **Objectif** : le moteur de convergence, sans stockage, prouvé par propriétés. Il assainit aussi le workspace et pose la CI.
- **Zone** : Core / Full.
- **Couvre** : US-2 (AC-2.6, logique) · RG-2.1, 2.2, 2.3, 2.4 · INV-4, 5, 6, 7 · arbitrages CONFLIT-3 (comparaison à l'estampille la plus récente des champs), CONFLIT-4/5 (HLC, conflit = modifié des deux côtés depuis la dernière synchro).
- **Crates** : `arbitre-core` (`hlc`, `write`, `merge`, `clock`), `Cargo.toml` (workspace).
- **Dépendances** : G-0.
- **Livrable** : port `Clock` ; HLC avec départage par appareil ; écriture `(donnée, champ, valeur, hlc, appareil, auteur)` ; `fusion(projection, écritures, filigranes) -> (projection', écartées, conflits)` ; retrait de `rusqlite` et de `chrono/clock` du cœur ; `[workspace.dependencies]` ; CI (`cargo test`, wasm32 sur core, arêtes interdites, gitleaks).
- **Critère de complétion** : propriétés vertes sur commutativité, associativité et idempotence (INV-5), sur la suppression qui gagne ou perd (INV-6) et sur la récupérabilité de toute valeur écartée (INV-7) ; écrasement séquentiel non signalé, modification concurrente signalée ; `cargo check --target wasm32-unknown-unknown -p arbitre-core` vert en CI.
- **Effort** : M.

#### ST-02 : Persistance — port Store et adaptateur SQLite
- **Objectif** : les écritures survivent à l'arrêt brutal et au disque plein, et plusieurs processus partagent la même base.
- **Zone** : Generic (adaptateur) avec propriétés bloquantes.
- **Couvre** : US-1 (AC-1.4, edge disque plein), US-3 (edge concurrence) · RG-1.6, 1.7, 3.6 ; RG-1.1 (côté stockage local) · PROP-3, 4, 7.
- **Crates** : `arbitre-core` (`store` : port, erreurs `StockagePlein`, `Corrompu`, `Indisponible`) et **`arbitre-sqlite` (naît)**.
- **Dépendances** : ST-01.
- **Livrable** : `Store::appliquer` atomique, projection, journal par champ, `ecritures_depuis`, filigranes par pair, identifiant d'appareil ; schéma générique (journal + projection), versionné ; WAL et délai d'attente sur verrou.
- **Critère de complétion** : simulateur d'arrêt brutal (processus tué à un instant aléatoire) : confirmée ⇒ présente, non confirmée ⇒ tout ou rien ; stockage plein simulé ⇒ refus et état intact ; deux **processus** écrivains entrelacés ⇒ aucune perte silencieuse.
- **Note** : aucune surface utilisateur. La première valeur visible arrive à ST-03.
- **Effort** : M.

#### ST-03 : CLI dérivée du catalogue — export et import
- **Objectif** : première surface. L'utilisateur exporte et réimporte toutes ses données, et la CLI est générée depuis le catalogue d'actions.
- **Zone** : Core / Full pour l'export et l'import ; Supporting / Std pour le catalogue et les codes.
- **Couvre** : US-1 (AC-1.3), US-3 (AC-3.3, 3.4, 3.5) · RG-1.3, 1.4, 1.5, 3.3, 3.4, 3.5 · INV-1, 2, 3, 12, 13, 14 · AMB-16 (codes distincts).
- **Crates** : `arbitre-core` (`catalog`, `error`, `export`), `arbitre` (cli).
- **Dépendances** : ST-02.
- **Livrable** : `DescripteurAction` (nom, schémas, mutante, classe de journalisation), `Sortie` (structurée canonique + rendu humain dérivé), catégories d'erreur → **table des codes de retour fixée une fois pour toutes** ; dérivation clap ; `--json` ; actions socle `arbitre export` et `arbitre import` (format texte versionné, une écriture par ligne).
- **Critère de complétion** : `import(∅, export(S)) ≡ S` et `import(T, export(S)) = fusion(T, S)` par propriétés ; fichier tronqué, illisible ou incohérent ⇒ état inchangé et cause nommée ; commande mal formée ⇒ usage, état inchangé, code ≠ 0 ; humain et JSON portent la même information.
- **Effort** : L.

#### ST-04 : Modules activables et cloisonnés
- **Objectif** : le mécanisme de modules, prêt à accueillir planning et decision diff sans qu'ils se voient.
- **Zone** : Supporting / Std.
- **Couvre** : US-3 (AC-3.2, routage ; edge module non activé), US-5 (AC-5.1 à 5.4, edge import) · RG-3.2 (forme), 3.7, 5.1, 5.2, 5.3, 5.4, 5.5 · INV-11 (forme), 15, 20, 21, 22, 23, 24 · CONFLIT-1 (crochet de revalidation), CONFLIT-6, AMB-12 (activation propre à l'appareil, non synchronisée).
- **Crates** : `arbitre-core` (`module`), `arbitre` (cli).
- **Dépendances** : ST-03.
- **Livrable** : trait `Module` (`id`, `actions`, `executer`, `revalider`), registre, `arbitre module enable|disable|list`, vue `Store` restreinte à l'espace de noms du module, **test de conformité générique** applicable à tout module (activation, invisibilité, espace de noms, parité), éprouvé sur un module factice de test.
- **Critère de complétion** : un module désactivé est absent de l'aide et de la complétion ; son appel explicite ⇒ « module non activé », code dédié, état inchangé ; ses données importées sont conservées, puis visibles à l'activation ; une lecture hors espace de noms est impossible.
- **Effort** : M.

---

### Bloc B — Module planning (CLI et assistant)

#### ST-05 : Éléments — créer, modifier, faire, supprimer
- **Objectif** : `arbitre plan` existe, et l'utilisateur tient ses éléments, chacun avec une durée proposée sans IA.
- **Zone** : Core / Full.
- **Couvre** : US-6 (AC-6.1 à 6.4), US-7 (AC-7.1, 7.4, 7.5) · RG-6.1, 6.2, 6.3, 7.1, 7.2, 7.4 ; RG-3.2 (`arbitre plan`) · INV-11, 25, 26, 27, 29, 30, 32 · CONFLIT-1 (un élément fusionné invalide est marqué, pas rejeté).
- **Crates** : **`arbitre-plan` (naît ; wasm32 en CI)**, `arbitre` (cli).
- **Dépendances** : ST-04.
- **À trancher au cycle** : AMB-10 (retour de fait à actif, récupération d'une suppression), MANQUE-5 (durées par défaut par type), RENVOI-1 (élément plus long qu'une journée, côté saisie).
- **Critère de complétion** : test de conformité de ST-04 vert pour `plan` ; un engagement daté sans échéance est refusé à la création, au changement de type et au retrait d'échéance ; un élément fait sort de la pile et reste dans l'historique ; `en_retard` conforme à INV-27.
- **Effort** : M.

#### ST-06 : Pile ordonnée et repriorisation
- **Objectif** : voir la pile entière dans un ordre unique, et la réordonner.
- **Zone** : Core / Full.
- **Couvre** : US-9 (AC-9.1 à 9.4) · RG-9.1, 9.2, 9.3, 9.5 · INV-38, 39, 40, 41 · CONFLIT-2 (ordre total après fusion), CONFLIT-10 (un réordonnancement ne déplace aucun élément planifié).
- **Crates** : `arbitre-plan` (`item` : rang fractionnaire), `arbitre` (cli).
- **Dépendances** : ST-05.
- **Critère de complétion** : propriété « déplacer e n'écrit que le rang de e » ; deux réordonnancements concurrents fusionnés ⇒ ordre total strict, départagé par identifiant ; pile vide ⇒ message ; aucune troncature.
- **Note** : RG-9.4 / AC-9.5 (l'agenda ne change jamais un rang) se vérifie en ST-16, quand l'agenda peut déplacer quelque chose.
- **Effort** : M.

#### ST-07 : L'assistant agit par MCP
- **Objectif** : Claude Code (ou un autre client MCP) gère la pile, et ses suppressions et déplacements sont journalisés.
- **Zone** : Supporting / Std ; Generic pour le transport.
- **Couvre** : US-4 (AC-4.1, 4.5), US-6 (AC-6.5, partie CLI et MCP) · RG-4.1, 4.6, 4.7, 6.4 · INV-10 (CLI ⊆ MCP), 18, 19, 23 (catalogue MCP), 28 · AMB-11 (déplacer = jour planifié ou rang ; auteur « assistant » = reçu par MCP).
- **Crates** : `arbitre` (cli : `mcp/`), `arbitre-core` (auteur, requête « journal de l'assistant »).
- **Dépendances** : ST-06, **spike S5**.
- **Critère de complétion** : test de parité dérivé du catalogue (aucune action CLI absente du MCP) ; module désactivé ⇒ aucun outil listé ; suppression par MCP ⇒ entrée d'historique (date, « assistant ») dans la même transaction ; testé avec deux clients MCP.
- **Effort** : M.

#### ST-08 : Corriger les durées, et que l'outil apprenne
- **Objectif** : les corrections de l'utilisateur rapprochent les durées proposées des éléments suivants.
- **Zone** : Supporting / Std.
- **Couvre** : US-7 (AC-7.2, 7.3) · RG-7.3, 7.5, 7.6 · INV-31, 33 · PROP-11 · AMB-9 (une durée déjà proposée ne change pas en silence).
- **Crates** : `arbitre-plan` (`duration`).
- **Dépendances** : ST-05.
- **Critère de complétion** : une durée fixée n'est jamais réécrite par une proposition ; « semblable » est déterministe et sans IA ; un générateur d'historiques de corrections montre le rapprochement.
- **Effort** : S.

#### ST-09 : Charge et capacité par jour (agenda simulé)
- **Objectif** : `arbitre plan` montre, jour par jour, la charge face à la capacité, et signale le débordement. Utile sans agenda (capacité de 450 min les jours travaillés).
- **Zone** : Core / Full.
- **Couvre** : US-8 (AC-8.1 à 8.4, 8.6 sur simulateur) · RG-8.1, 8.2, 8.3, 8.4 · INV-34, 35, 36, 37 · AMB-1, AMB-3, AMB-4, AMB-8 (défauts des specs) · action « planifier à un jour » (jour planifié choisi à la main).
- **Crates** : `arbitre-plan` (`capacity`, port `Agenda` en lecture + simulateur).
- **Dépendances** : ST-06.
- **À trancher au cycle** : AMB-2 (peut-être, sans réponse, disponible), RENVOI-8 (plage de travail).
- **Critère de complétion** : capacité = 450 − union des intervalles des réunions comptées, plancher 0, week-end et absence à 0 ; un événement porteur du marqueur de l'outil n'est pas compté ; débordement et excès présents en sortie humaine et en JSON.
- **Effort** : M.

> **S3 doit avoir rendu son verdict ici** (lancé après ST-01).

#### ST-10 : Lire l'agenda Google
- **Objectif** : la capacité reflète les vraies réunions de plusieurs agendas, et l'utilisateur voit la date de dernière lecture.
- **Zone** : Generic (adaptateur, tests de contrat) ; Core / Full pour la fraîcheur.
- **Couvre** : US-8 (AC-8.5), US-1 (AC-1.2 côté agenda ; matrice US-1 cas 3) · RG-8.5, 1.1, 1.2 · PROP-1, 2 (observateur réseau, posé ici puis réutilisé à chaque nouvelle sortie), 12 · CONFLIT-8 (la connexion vaut accord durable) · D-3, D-4 (client OAuth fourni par l'utilisateur, guide documenté).
- **Crates** : **`arbitre-calendar` (naît)**, `arbitre-plan`, `arbitre` (cli : connexion, recalcul manuel `arbitre plan recalcul`).
- **Dépendances** : ST-09, **verdict S3**.
- **Critère de complétion** : tests de contrat sur des réponses enregistrées (occurrences dépliées, journée entière, `outOfOffice`, statut de réponse, `transparency`) ; agenda injoignable ⇒ dernière capacité connue « non à jour » et sa date ; aucune sortie réseau hors de l'agenda connecté ; jeton dans le trousseau, aucun secret dans le dépôt.
- **Effort** : L.

#### ST-11 : Recalcul chaque matin, outil fermé
- **Objectif** : le recalcul tourne seul via le planificateur système, rattrape une exécution manquée, et son résultat est prêt à l'ouverture.
- **Zone** : Core / Full (idempotence) ; Generic (installation launchd et systemd).
- **Couvre** : US-13 (AC-13.2 : relecture et capacités ; AC-13.7) · RG-13.4 (partie lecture), 13.6, 13.7 · PROP-19 (partie), 21 · INV-56 (idempotence) · CONFLIT-7.
- **Crates** : `arbitre-plan` (orchestration `recalcul`), `arbitre` (cli : `scheduler/`, `arbitre plan schedule install`).
- **Dépendances** : ST-10.
- **À trancher au cycle** : RENVOI-7 (heure du recalcul et de la planification).
- **Critère de complétion** : deux recalculs successifs ⇒ second sans effet ; horloge simulée avec cycles marche/arrêt ⇒ rattrapage au démarrage ; installation testée sur macOS (launchd) et documentée pour Linux (systemd).
- **Effort** : M.

#### ST-12 : Planifier la semaine
- **Objectif** : le lundi (ou le premier jour travaillé), les éléments non planifiés se placent au premier jour qui suffit, dans l'ordre de la pile, sans rien bouger.
- **Zone** : Core / Full.
- **Couvre** : US-13 (AC-13.1, 13.3) · RG-13.1, 13.2, 13.8 · INV-53, 54, 57 · AMB-5, AMB-6 (placement des éléments créés en cours de semaine), AMB-7 (jour férié lu dans l'agenda).
- **Crates** : `arbitre-plan` (`placement`).
- **Dépendances** : ST-11.
- **Critère de complétion** : propriété « aucun jour déjà planifié ne change » sur pile et capacités générées ; un élément qui ne tient nulle part reste non planifié avec un avertissement ; résultat indépendant de l'appareil qui exécute.
- **Effort** : M.

#### ST-13 : Questions — urgence et débordement
- **Objectif** : l'outil ne tranche jamais seul. Urgence, débordement et élément fusionné invalide deviennent des questions à choix concrets, et rien ne bouge avant la réponse.
- **Zone** : Core / Full.
- **Couvre** : US-13 (AC-13.2 : signalement ; AC-13.4, 13.5, 13.6) · RG-13.3, 13.4 (une question par jour débordé), 13.5 ; RG-G6 · INV-55, 56 (une question par situation, id déterministe) · PROP-20 · CONFLIT-1 (question sur l'élément invalide) · AMB-22.
- **Crates** : `arbitre-plan` (`questions`), `arbitre` (cli : lister, répondre).
- **Dépendances** : ST-12.
- **À trancher au cycle** : RENVOI-6 (ordre et expiration des questions).
- **Critère de complétion** : question ouverte ⇒ aucun déplacement lié ; réponse ⇒ appliquée, question fermée ; deux appareils qui recalculent ⇒ même identifiant de question, une seule après fusion.
- **Effort** : M.

#### ST-14 : Plan de blocs et confidentialité (aperçu)
- **Objectif** : calculer, sans toucher à l'agenda, les blocs attendus et leur contenu visible. `arbitre plan blocs --apercu` les montre.
- **Zone** : Core / Full.
- **Couvre** : US-10 (AC-10.1 à 10.4, règles), US-11 (AC-11.1 à 11.4) · RG-10.1, 10.2, 10.3, 11.1 à 11.5 · INV-42 à 49 · CONFLIT-9 (non-chevauchement réservé aux blocs placés par l'outil), CONFLIT-11 (marqueur opaque).
- **Crates** : `arbitre-plan` (`blocks`).
- **Dépendances** : ST-12.
- **À trancher au cycle** : RENVOI-1, 2, 4, 5, 8 (élément plus long qu'une journée, créneau insuffisant, sujet public par défaut, bloc des petits gestes, plage de travail).
- **Critère de complétion** : contenu visible = titre, début, fin et statut occupé, rien d'autre ; aucun titre « Focus — » + intitulé brut sans sujet public ; bloc quotidien ≥ 30 min, au titre constant ; identifiants de blocs déterministes.
- **Effort** : M.

> **S3, critères (5) et (6), doivent être verts ici.**

#### ST-15 : Poser les blocs dans Google Calendar
- **Objectif** : les blocs apparaissent dans l'agenda d'écriture et disparaissent quand l'élément est fait ou supprimé. Les événements tiers restent intouchés.
- **Zone** : Core / Full (réconciliation, **PROP-14 bloquant**) ; Generic (écriture Google).
- **Couvre** : US-10 (AC-10.5, 10.6), US-6 (edge élément supprimé avec bloc) · RG-10.4, 10.5, 10.6, 10.7, 6.5 ; RG-1.2 (écriture agenda) · PROP-10, 14, 15, 16 · AMB-22.
- **Crates** : `arbitre-plan` (`reconcile`), `arbitre-calendar` (écriture, refus de tout événement sans marqueur), `arbitre` (cli).
- **Dépendances** : ST-14, ST-13 (recalcul complet), S3.
- **Critère de complétion** : simulateur d'agenda à origines mêlées ⇒ aucune modification ni suppression d'un événement sans marqueur, vérifiée dans la fonction pure **et** dans l'adaptateur ; panne puis reprise ⇒ opérations appliquées sans doublon ; deux appareils ⇒ au plus un bloc par élément et un bloc quotidien par jour.
- **Effort** : L.

#### ST-16 : Bloc déplacé à la main, la pile suit
- **Objectif** : l'agenda fait foi pour le « quand ». Un bloc glissé à un autre jour change le jour planifié de l'élément, et rien d'autre.
- **Zone** : Core / Full.
- **Couvre** : US-12 (AC-12.1 à 12.4), US-9 (AC-9.5), US-13 (AC-13.2 : blocs déplacés) · RG-12.1 à 12.5, 9.4 · INV-50, 51, 52 · PROP-13, 17, 18 · CONFLIT-9.
- **Crates** : `arbitre-plan` (`reconcile` : déplacements lus).
- **Dépendances** : ST-15.
- **À trancher au cycle** : RENVOI-3 (bloc supprimé ou redimensionné à la main).
- **Critère de complétion** : aucun événement d'agenda ne change un rang ni une durée ; un déplacement qui fait déborder ⇒ question, rien d'autre ne bouge ; déplacement vers un jour à capacité 0 ⇒ « hors capacité » ; après l'échéance ⇒ en retard.
- **Effort** : M.

#### ST-17 : Agendas CalDAV — *différable*
- **Objectif** : un utilisateur hors Google obtient le même planning.
- **Zone** : Generic (tests de contrat).
- **Couvre** : arbitrage « Agendas » (CalDAV ensuite) ; RG-8.*, 10.* rejouées sur un second adaptateur.
- **Crates** : `arbitre-calendar` (`caldav/`).
- **Dépendances** : ST-16. Ne s'engage qu'à la demande d'un utilisateur réel, ou si S3 conclut au repli.
- **Critère de complétion** : la suite de contrat du port `Agenda` passe à l'identique sur Google et sur CalDAV.
- **Effort** : M.

---

### Bloc C — Socle mobile (synchro et téléphone)

#### ST-18 : Enveloppe chiffrée
- **Objectif** : ce qui part vers un relais est illisible et inaltérable sans la phrase secrète de l'utilisateur.
- **Zone** : Supporting (sécurité) / **Full malgré la zone**.
- **Couvre** : US-2 (edge tiers non autorisé) · RG-2.8 · INV-9 · MANQUE-6 (AEAD, rejet de l'altération, format versionné, appareil auteur dans chaque écriture).
- **Crates** : `arbitre-core` (`envelope` ; wasm32 en CI, `getrandom` avec sa feature wasm).
- **Dépendances** : ST-02.
- **Critère de complétion** : vecteurs de test XChaCha20-Poly1305 et Argon2id ; tout octet altéré ⇒ rejet sans altérer les données locales ; mauvais secret ⇒ message explicite ; numéro de format présent.
- **Effort** : S.

#### ST-19 : Relais et synchro entre deux ordinateurs
- **Objectif** : deux appareils natifs convergent en moins d'une minute via un relais que l'utilisateur héberge, et l'utilisateur voit les conflits.
- **Zone** : Generic (relais, transport) ; Core / Full pour la convergence de bout en bout.
- **Couvre** : US-2 (AC-2.1, 2.3, 2.4, 2.6) · RG-2.4 (signalement visible), 2.5, 2.6, 2.7 ; RG-1.2 (sortie vers le relais) · INV-8 · PROP-5, 6.
- **Crates** : `arbitre-core` (port `Transport`), relais (crate `arbitre-relay` ou module de la CLI, voir PL-3), `arbitre` (cli : `arbitre relay serve`, `arbitre sync`, `arbitre conflits`).
- **Dépendances** : ST-18, ST-04 (revalidation), ST-13 (questions sur élément invalide).
- **Critère de complétion** : simulateur à deux appareils, un relais, une connectivité commandable et une horloge ⇒ propagation < 60 s, reprise seule après 3 jours d'arrêt, date de dernière synchro réussie affichée ; sans relais configuré ⇒ outil complet ; le relais ne reçoit que des enveloppes.
- **Effort** : L.

> **S1 et S2 ici.**

#### ST-20 : Le téléphone consulte, hors ligne
- **Objectif** : la PWA, installée sans store et servie par le relais (D-6), affiche la pile, les jours, la charge et les questions, hors ligne, synchronisés.
- **Zone** : Generic (UI, stockage navigateur).
- **Couvre** : US-2 (AC-2.2, 2.5), C-2.9 · RG-1.1 (téléphone), 2.5 · D-1 (la PWA est toujours une réplique), D-6.
- **Crates** : **`arbitre-web` (naît)**, adaptateur `Store` navigateur (selon S1 : SQLite-WASM ou repli IndexedDB), relais (fichiers statiques).
- **Dépendances** : ST-19, **S1**, **S2**.
- **Critère de complétion** : installation sur iOS et Android ; ouverture hors ligne ; écriture distante visible en moins de 60 s ; aucune sortie vers l'agenda ni vers une IA depuis la PWA.
- **Effort** : L.

#### ST-21 : Le téléphone agit
- **Objectif** : créer, modifier, faire, réordonner et répondre aux questions depuis le téléphone, avec le même résultat que la CLI.
- **Zone** : Generic (UI) ; parité en Supporting / Std.
- **Couvre** : US-3 (AC-3.1), US-6 (AC-6.5 complet) · RG-3.1, 6.4 · INV-10 (GUI ⊆ CLI), 28 · PROP-7 (côté GUI).
- **Crates** : `arbitre-web`.
- **Dépendances** : ST-20.
- **Critère de complétion** : test de parité du catalogue étendu à la GUI ; même entrée ⇒ même élément, même durée proposée, mêmes signalements sur toutes les surfaces.
- **Effort** : M.

---

### Bloc D — Decision diff : vérification d'invariants

> **S4 ici.** Les entretiens (G-2) se poursuivent en parallèle. ST-22 est le « quickwin » du plan du 2026-10-02 (« 3 violations dans ton repo »), montrable en entretien.

#### ST-22 : `arbitre diff check` — frontières d'import
- **Objectif** : un mainteneur déclare des frontières d'import dans un fichier versionné et obtient un verdict déterministe, sans IA ni réseau, en local.
- **Zone** : Core / Full (vérification, verdict) ; Supporting / Std (extracteur de faits Python).
- **Couvre** : US-14 (AC-14.1 à 14.4), US-15 (AC-15.1 en local, 15.2 à 15.5) · RG-14.1 à 14.5, 14.6 (famille frontière d'import), 15.1 à 15.6 ; RG-3.2 (`arbitre diff`) · INV-11, 58 à 62, 63 (partie), 64 à 69 · AMB-14 (frontière directe), AMB-15, AMB-16 (code « violation » distinct de « erreur d'outil »), CONFLIT-17 (défaut des specs : verdict sur tout l'état résultant) · D-2 (`diff check` ne lit pas le stockage local).
- **Crates** : **`arbitre-diff` (naît ; natif)** : `facts/`, `python/`, `invariants`, `check` ; `arbitre` (cli).
- **Dépendances** : ST-04, **S4**. Aucune dépendance au planning (contrôle d'arêtes en CI).
- **À trancher au cycle** : MANQUE-2 (héritage d'un dépôt de règles : la forme du fichier d'invariants doit-elle le prévoir dès v1 ?).
- **Critère de complétion** : test de conformité de ST-04 vert pour `diff` ; déterminisme par permutation de l'ordre des fichiers ; fichier mal formé ⇒ refus total, invariant fautif nommé ; langage non pris en charge ⇒ « non vérifié » ; pas de fichier d'invariants ⇒ succès avec mention ; essayé sur 2 ou 3 dépôts OSS.
- **Effort** : L.

#### ST-23 : Familles « types du domaine » et « appels interdits »
- **Objectif** : les trois familles v1 sont exprimables et vérifiées.
- **Zone** : Core / Full.
- **Couvre** : US-14 (AC-14.2, 14.4), US-15 (AC-15.4, 15.5) · RG-14.6 · INV-63.
- **Crates** : `arbitre-diff`.
- **Dépendances** : ST-22.
- **À trancher au cycle** : AMB-14 (critère d'un « type du domaine » en Python non annoté).
- **Critère de complétion** : pour chaque famille, cas qui respectent et cas qui violent, avec fichier et ligne exacts ; extension sans toucher au modèle de faits.
- **Effort** : M.

#### ST-24 : Vérification de PR en CI
- **Objectif** : la GitHub Action fait échouer une PR qui viole un invariant, sans clé ni stockage, et une PR qui affaiblit ou retire un invariant déclenche une alerte distincte.
- **Zone** : Core / Full (union base ∪ tête, détection d'affaiblissement) ; Generic (Action).
- **Couvre** : US-15 (AC-15.1 en CI) · RG-15.1, 15.3 · arbitrage CONFLIT-12 (union des invariants de base et de tête, alerte explicite et distincte) · D-2 (activation dans un répertoire de configuration éphémère).
- **Crates** : `arbitre-diff` (port `Dépôt` : git en sous-processus ou `gix`), `action.yml`.
- **Dépendances** : ST-23.
- **Critère de complétion** : PR qui retire l'invariant qu'elle viole ⇒ échec et alerte ; codes distincts « violation » et « outil cassé » lisibles par la CI ; aucune sortie réseau pendant la vérification.
- **Effort** : M.

---

### Bloc E — Decision diff : décisions, itérations, candidats

#### ST-25 : Harnais d'évals de l'extracteur
- **Objectif** : mesurer le rappel et la précision de l'extracteur de décisions avant de l'écrire, avec la vérification comme vérité terrain.
- **Zone** : Supporting / Std (outil de développement, hors produit livré).
- **Couvre** : MANQUE-1, MANQUE-4 ; plan du 2026-10-02, brique 2 (PR d'agents sur des dépôts OSS, violations = labels, 30 à 50 cas annotés à la main).
- **Crates** : `arbitre-diff` (tests ou binaire de dev, hors `arbitre`).
- **Dépendances** : ST-24.
- **Critère de complétion** : jeu de cas versionné ; une commande rend rappel et précision par catégorie de décision.
- **Effort** : M.

#### ST-26 : Décisions structurelles d'une PR, avec leur couverture
- **Objectif** : pour une PR, une liste de décisions en une phrase chacune, rattachées au code, avec leur statut respecte, viole ou non couverte. Sans IA.
- **Zone** : Supporting / Std (extraction) ; Core / Full (couverture).
- **Couvre** : US-16 (AC-16.1, 16.2) · RG-16.1, 16.2, 16.5, 16.6 · INV-70, 71, 74, 75 · CONFLIT-12 (toute modification du fichier d'invariants = décision), CONFLIT-14, CONFLIT-15, AMB-18.
- **Crates** : `arbitre-diff` (`extract`).
- **Dépendances** : ST-25.
- **À trancher au cycle** : RENVOI-9 (taille maximale d'une modification).
- **Critère de complétion** : objectifs de rappel et de précision fixés au cycle, atteints sur le harnais ; modification cosmétique ⇒ aucune décision et un message ; le fichier de décisions n'en produit pas.
- **Effort** : L.

#### ST-27 : Accepter ou rejeter chaque décision
- **Objectif** : le relecteur tranche décision par décision, et ses états de revue sont commités avec la PR, lisibles à la main.
- **Zone** : Core / Full.
- **Couvre** : US-16 (AC-16.3) · RG-16.4 · INV-73 · D-5 (`.arbitre/decisions/<branche>.toml` : en-tête, puis un bloc par décision : quoi, où, origine, statut, par, le, note).
- **Crates** : `arbitre-diff` (`decisions`), `arbitre` (cli et MCP).
- **Dépendances** : ST-26.
- **À trancher au cycle** : AMB-19 (effet d'un rejet), format exact des champs.
- **Critère de complétion** : aller-retour lecture/écriture du TOML sans perte ; acceptation par la CLI et par le MCP identique.
- **Effort** : M.

#### ST-28 : Ce qui a changé entre deux itérations
- **Objectif** : à chaque nouveau push, le relecteur ne relit que les décisions ajoutées, retirées ou modifiées, et il est averti si des acceptations ont été perdues.
- **Zone** : Core / Full.
- **Couvre** : US-17 (AC-17.1 à 17.3) · RG-17.1 à 17.5 · INV-76 à 80 · arbitrage CONFLIT-13 (« N acceptations perdues », les décisions concernées repassent « à relire »), AMB-20.
- **Crates** : `arbitre-diff` (`delta`).
- **Dépendances** : ST-27.
- **À trancher au cycle** : AMB-17 (composition de l'identité de décision).
- **Critère de complétion** : un décalage de lignes seul ne modifie pas l'identité ; première itération ⇒ toutes nouvelles ; un push forcé qui efface des acceptations ⇒ signalement explicite.
- **Effort** : M.

#### ST-29 : Décisions affichées sur la PR
- **Objectif** : la liste des décisions apparaît sur la page de la PR GitHub.
- **Zone** : Generic.
- **Couvre** : US-16 (AC-16.4) · RG-16.7 · PROP-22.
- **Crates** : `arbitre-diff` (rendu Markdown), `action.yml` (publication). Aucun client GitHub dans `arbitre-diff`.
- **Dépendances** : ST-28, ST-24.
- **Critère de complétion** : hébergeur simulé en échec ⇒ erreur explicite et liste disponible en local.
- **Effort** : S.

#### ST-30 : Adaptateur IA multi-fournisseurs (socle)
- **Objectif** : un réglage unique choisit Claude, GPT, Mistral ou un modèle local, et l'absence de fournisseur ne casse rien.
- **Zone** : Generic (tests de contrat) ; Supporting / Std (dégradation).
- **Couvre** : US-4 (AC-4.2, 4.3, 4.4) · RG-4.2, 4.3, 4.4, 4.5 ; RG-1.2 (sortie IA sur sollicitation seulement) · INV-16, 17 · PROP-8, 9.
- **Crates** : **`arbitre-ai` (naît)** : protocole Anthropic et protocole compatible OpenAI ; `arbitre-core` (port `Ia`).
- **Dépendances** : ST-04 (placée en séquence juste avant ST-31, sa première utilisatrice).
- **À trancher au cycle** : AMB-13 (délai « trop lent »).
- **Critère de complétion** : changement de fournisseur ⇒ données et catalogue identiques ; injoignable, clé refusée, délai dépassé ⇒ erreur explicite pour les seules fonctions dépendantes ; aucun appel automatique.
- **Effort** : M.

#### ST-31 : Décisions proposées par l'IA
- **Objectif** : l'IA ajoute des décisions marquées « proposé par IA », sans effet sur le verdict ni sur le code de retour.
- **Zone** : Supporting / Std.
- **Couvre** : US-16 · RG-16.3 · INV-72 · RG-4.4.
- **Crates** : `arbitre-diff`.
- **Dépendances** : ST-30, ST-28.
- **Critère de complétion** : fournisseur simulé absent ou défaillant ⇒ décisions structurelles intactes, verdict inchangé.
- **Effort** : S.

#### ST-32 : Invariants candidats, preuves à l'appui
- **Objectif** : l'outil propose des invariants avec preuves et contre-exemples calculés sans IA, et rien n'entre dans le fichier d'invariants sans adoption explicite.
- **Zone** : Supporting / Std ; Core / Full pour INV-81 et 82.
- **Couvre** : US-18 (AC-18.1 à 18.5) · RG-18.1 à 18.6 · INV-81 à 84 · PROP-23, 24 · CONFLIT-16 (avertir avant le premier envoi de code à un fournisseur distant), MANQUE-3.
- **Crates** : `arbitre-diff` (`candidates`), `arbitre` (cli et MCP).
- **Dépendances** : ST-30, ST-23.
- **Critère de complétion** : candidats hors famille v1 ou mal formés écartés ; contre-exemples = violations de `verifier` ; dépôt trop petit ⇒ message et aucun candidat ; sans fournisseur ⇒ fonction indisponible, reste du module nominal.
- **Effort** : L.

---

## ⛔ GARDE-FOU — fin de séquence

**Aucun module au-delà de planning et decision diff ne démarre avant 10 entretiens de design partners** (RG-G8, `CADRAGE.md`). Planning et decision diff sont des besoins personnels : le garde-fou ne coupe aucune des stories ST-01 à ST-32.

- **Critère de levée** : 10 entretiens menés et consignés (`/user-interview`, vault `01-work/meetings/`). L'orchestrateur vérifie le compte avant d'ouvrir le cycle d'un troisième module.

---

## Ordre d'exécution

```
G-0 ─► ST-01 ─► ST-02 ─► ST-03 ─► ST-04
         │                           │
         └─► S3 (tâche de fond) ─────┼───────────────┐
                                     ▼               ▼
        PLANNING   ST-05 ─► ST-06 ─► [S5] ST-07      │
                     │        ├─► ST-08              │
                     │        └─► ST-09 ─► ST-10 ◄───┘ (verdict S3)
                     │                       └─► ST-11 ─► ST-12 ─► ST-13
                     │                                       └─► ST-14 ─► ST-15 ─► ST-16 ─► (ST-17 différable)
                     │
        MOBILE     ST-18 (dès ST-02) ─► ST-19 ─► [S1+S2] ST-20 ─► ST-21
                     (ST-19 attend aussi ST-13)
                     │
        DIFF       [S4] ST-22 ─► ST-23 ─► ST-24 ─► ST-25 ─► ST-26 ─► ST-27 ─► ST-28 ─► ST-29
                   ST-30 (dès ST-04) ─► ST-31 (+ ST-28) ; ST-30 + ST-23 ─► ST-32

        ════════ ⛔ GARDE-FOU : aucun 3e module avant 10 entretiens ════════
```

Séquence de travail retenue, pour un développeur seul : **ST-01 → 04 → 05 → 06 → 07 → 08 → 09 → 10 → 11 → 12 → 13 → 14 → 15 → 16 → 18 → 19 → 20 → 21 → 22 → 23 → 24 → 25 → 26 → 27 → 28 → 29 → 30 → 31 → 32 → ⛔ garde-fou (3e module)**. ST-17 s'insère à la demande. Le graphe autorise d'autres ordres (ST-18 dès ST-02, ST-22 dès ST-04), mais la séquence suit l'ordre des modules du cadrage.

Jalons de valeur : **ST-03** export (première commande), **ST-09** planning utile au quotidien sur CLI, **ST-07** planning piloté par l'assistant, **ST-15** blocs dans l'agenda, **ST-20** téléphone, **ST-22** démo decision diff en entretien.

## Points à trancher (niveau plan)

| # | Question | Recommandation (retenue par défaut) |
|---|---|---|
| PL-1 | Socle mobile (ST-18 à ST-21) après le planning sur CLI, ou avant, dans l'ordre littéral « socle → planning » ? | **Après le planning.** RG-G1 le permet, puisque aucune story du planning n'en dépend ; la valeur arrive environ 3 à 4 mois plus tôt au budget de 8 à 10 h par semaine. |
| PL-2 | S3 lancé tôt, en tâche de fond, plutôt que juste à temps ? | **Tôt, après ST-01** : son verdict peut demander l'accord d'un admin Workspace, avec un délai externe incompressible. |
| PL-3 | Relais en crate dédiée ou en module de la CLI ? | **Module de la CLI** jusqu'à ce qu'une autre surface en ait besoin (même règle que le MCP). La frontière tient par le type `Enveloppe`. |
| PL-4 | CalDAV (ST-17) : engagé d'office ou à la demande ? | **À la demande**, ou si S3 conclut au repli. |
| PL-5 | CONFLIT-17 (verdict sur tout l'état ou sur les seules violations introduites) n'est pas arbitré. | La résolution par défaut des specs vaut : **tout l'état résultant**. À reconfirmer au cycle de ST-22. |

## Invariants C-TDD à vérifier post-implémentation

| Invariant / Property | Story | Vérification |
|---|---|---|
| INV-4, 5, 6, 7 | ST-01 | Propriétés sur la fusion (algèbre, tombstone, versions écartées, conflits) |
| INV-1, 2, 3 | ST-03 | Propriétés aller-retour export/import ; fichiers abîmés générés |
| INV-12, 13, 14 | ST-03 | Tests sur la CLI dérivée ; équivalence humain/JSON |
| INV-11 | ST-04 (forme), ST-05 (`plan`), ST-22 (`diff`) | Routage des points d'entrée |
| INV-15, 20, 21, 22, 23, 24 | ST-04, rejoués par module en ST-05 et ST-22 | Test de conformité générique de `Module` |
| INV-10 | ST-07 (CLI ⊆ MCP), ST-21 (GUI ⊆ CLI) | Test de parité dérivé du catalogue |
| INV-18, 19 | ST-07 | Journal de l'assistant dans la même transaction |
| INV-25, 26, 27, 29, 30, 32 | ST-05 | Propriétés sur la validité et le cycle de vie |
| INV-28 | ST-07, ST-21 | Même entrée sur chaque surface ⇒ même résultat |
| INV-31, 33 · PROP-11 | ST-08 | Générateur d'historiques de corrections |
| INV-34, 35, 36, 37 | ST-09 | Propriétés sur occurrences générées (simulateur `Agenda`) |
| INV-38, 39, 40, 41 | ST-06 | Propriétés sur le rang fractionnaire, y compris après fusion |
| INV-42 à 49 | ST-14 | Fonctions pures, contenu visible exhaustif |
| INV-50, 51, 52 · PROP-13, 17, 18 | ST-16 | Simulateur d'agenda avec déplacements manuels |
| INV-53, 54, 57 | ST-12 | Propriété « l'existant ne bouge pas » |
| INV-55, 56 · PROP-20 | ST-13 (et idempotence en ST-11) | Utilisateur simulé + horloge ; deux appareils |
| INV-8 · PROP-5, 6 | ST-19 | Simulateur deux appareils + relais + connectivité + horloge |
| INV-9 | ST-18 | Vecteurs de test, altération, mauvais secret |
| INV-16, 17 · PROP-8, 9 | ST-30 | Bouchons de fournisseurs |
| INV-58 à 69 | ST-22, ST-23, ST-24 | Déterminisme par permutation ; dépôts OSS réels |
| INV-70, 71, 74, 75 | ST-26 | Harnais d'évals (ST-25) + propriétés |
| INV-72 | ST-31 | Fournisseur simulé défaillant |
| INV-73 | ST-27 | Aller-retour du TOML |
| INV-76 à 80 | ST-28 | Itérations générées, push forcé simulé |
| INV-81 à 84 · PROP-23, 24 | ST-32 | Fournisseur simulé (valides, mal formés, hors famille, aucun) |
| PROP-1 | ST-02, ST-10, ST-20 | Simulateur réseau coupé |
| PROP-2 | ST-10, puis à chaque nouvelle sortie : ST-15, ST-19, ST-30, ST-32 | Observateur réseau (destination, déclencheur) |
| PROP-3, 4, 7 | ST-02 (PROP-7 aussi côté GUI en ST-21) | Arrêt brutal, stockage plein, deux processus |
| PROP-10, 14, 15, 16 | ST-15 | Agenda à origines mêlées, panne/reprise, deux appareils ; **PROP-14 bloquant** |
| PROP-12 | ST-10 | Agendas multiples avec changements et pannes |
| PROP-19, 21 | ST-11 (lecture, rattrapage), complétée en ST-13 et ST-16 | Horloge + cycles marche/arrêt |
| PROP-22 | ST-29 | Hébergeur simulé |
| C-2.9 | ST-20 | Acceptation manuelle iOS et Android |
