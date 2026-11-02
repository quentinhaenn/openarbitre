# Cadrage Architectural — OpenArbitre, cadre global (socle, planning, decision diff)

## Traçabilité
- Sources : `.ctdd/cadre/02_specs.md` (sections « Arbitrages » incluses, elles font foi), `.ctdd/cadre/01_story.md` (arbitrages de fin), `CADRAGE.md`, `.ctdd/cadre/inputs/plan-decision-diff-2026-10-02.md` (partie C ; le cadrage l'emporte).
- Date : 2026-11-02 · Statut : DRAFT · Cycle sans code.
- Codebase relevée : workspace Cargo vide (voir « Périmètre codebase »).

## Synthèse

1. **Un seul primitif de données : le journal d'écritures.** Chaque changement est une écriture `(donnée, champ, valeur, HLC, appareil, auteur)`. Fusion de synchro, import, concurrence CLI/GUI, historique, versions écartées et journal de l'assistant en découlent tous. La fusion est une fonction pure du cœur (`@Invariant`), testable sans stockage.
2. **Deux graphes de compilation.** Ce qui tourne dans la PWA (wasm32) : `arbitre-core` et `arbitre-plan`. Tout le reste est natif. Les dépendances lourdes (rusqlite natif, tree-sitter, HTTP agenda et IA) restent hors du graphe wasm. Une CI `--target wasm32-unknown-unknown` tient cette frontière dès la première story.
3. **Une crate par module, et le graphe Cargo comme garde-fou de RG-G2.** `arbitre-plan` et `arbitre-diff` ne dépendent que d'`arbitre-core`. Une dépendance entre modules ne compile pas.
4. **Catalogue d'actions comme source unique des surfaces.** La CLI, le serveur MCP et l'interface sont des adaptateurs du catalogue. La parité (INV-10) est vérifiée par un test sur le catalogue, sans être maintenue à la main.
5. **L'agenda et l'IA ne sont accessibles qu'en natif.** Le recalcul tourne sur l'ordinateur (arbitrage CONFLIT-7/8) et le téléphone reçoit le résultat par la synchro. La PWA ne détient ni jeton OAuth ni clé d'IA.
6. **Le stockage est derrière un port défini par le journal, pas par SQL.** Si le spike SQLite-WASM échoue, le repli (journal dans IndexedDB) ne coûte qu'un adaptateur.

---

## Périmètre codebase

### État de l'existant

| Élément | Contenu | Constat |
|---|---|---|
| `Cargo.toml` (workspace) | `resolver = "3"`, `members = ["crates/*"]`, `workspace.package` (edition 2024, Apache-2.0) | Sain. À garder. Pas de `workspace.dependencies` (chrono est dupliqué entre les deux crates). |
| `crates/core` (`arbitre-core`) | `lib.rs` = fonction `add` de `cargo new` ; dépend de `chrono` (feature `clock`) et `rusqlite` (`bundled`) | **Dépôt git imbriqué** (`crates/core/.git`) et deux dépendances incompatibles avec le rôle du cœur : voir Anti-patterns. |
| `crates/cli` (`arbitre`) | `main.rs` « Hello, world! » ; dépend de `arbitre-core`, `chrono`, `clap` (derive) | Sain. |
| `.pre-commit-config.yaml` | gitleaks local (`--staged`) | Conforme à RG-G9. Protection à l'envoi côté GitHub encore à activer (geste, pas une story). |
| `.gitignore` | `/target`, `.ctdd/cadre/inputs/` | OK : le plan du 2026-10-02 reste hors dépôt public. |
| Historique git | aucun commit | Les anti-patterns se corrigent avant le premier commit, sans réécrire d'historique. |

Aucun pattern applicatif n'est en place. Le présent cadrage instaure les patterns, et le Planner s'y conforme.

### Crates cibles

Chaque crate naît avec la **première story qui en a besoin**, pas avant. La colonne « Cible » est une contrainte de compilation, vérifiée en CI.

| Crate | Rôle | Cible | Dépend de | Naît avec |
|---|---|---|---|---|
| `arbitre-core` (existe) | Socle sans E/S : HLC, écriture, fusion, journal (modèle), port `Store`, port `Clock`, mécanisme de modules, catalogue d'actions, modèle d'erreur et codes de retour, format d'export, enveloppe chiffrée | **wasm32 + natif** | serde, crypto AEAD en Rust pur | 1re story du socle |
| `arbitre-sqlite` | Adaptateur `Store` sur rusqlite (WAL, transactions) | natif (wasm si S1 le permet) | core, rusqlite | 1re story de persistance |
| `arbitre-plan` | Module planning : règles pures (capacité, charge, placement, urgence, questions, plan de blocs, réconciliation) et port `Agenda` | **wasm32 + natif** | core | 1re story planning |
| `arbitre-calendar` | Adaptateurs du port `Agenda` : Google Calendar, puis CalDAV | natif | plan, client HTTP | 1re story agenda |
| `arbitre-diff` | Module decision diff : faits de code, fichier d'invariants, vérification, extraction, fichier de décisions, écart entre itérations, candidats | natif | core, tree-sitter ou parseur Ruff (S4) | 1re story decision diff |
| `arbitre-ai` | Adaptateur du port `Ia` : protocole Anthropic et protocole compatible OpenAI (GPT, Mistral, Ollama/local) | natif | core, client HTTP | story US-18 (ou 1re story qui sollicite l'IA) |
| `arbitre-relay` | Relais : stocke et resservit des enveloppes opaques, authentification par jeton | natif | core (format d'enveloppe seulement) | 1re story synchro |
| `arbitre` (cli, existe) | Binaire : CLI dérivée du catalogue, serveur MCP (`arbitre mcp`), `arbitre relay serve`, installation du planificateur système, câblage des adaptateurs natifs | natif | toutes les crates natives | existe |
| `arbitre-web` | PWA Leptos (CSR), worker de stockage, client de synchro | **wasm32** | core, plan, (adaptateur stockage navigateur) | après S1/S2 |

**Règle de placement** : tout ce qui servira à une autre surface (Tauri plus tard, relais) va dans une crate bibliothèque. `arbitre` (cli) ne contient que l'analyse d'arguments, le rendu et le câblage. Le serveur MCP vit dans `arbitre` (cli), parce que Tauri n'en a pas besoin. On l'extrait le jour où une autre surface le réclame.

### Graphe de dépendances

```
                    ┌────────────── arbitre-core ──────────────┐   (wasm32 + natif, aucune E/S)
                    │            ▲            ▲                │
            arbitre-plan    arbitre-diff   arbitre-sqlite   arbitre-relay   arbitre-ai
           (wasm + natif)    (natif)         (natif)          (natif)        (natif)
              ▲     ▲           ▲               ▲                ▲              ▲
   arbitre-calendar │           │               │                │              │
        (natif)     │           │               │                │              │
              ▲     │           │               │                │              │
              └─────┴───────────┴───── arbitre (cli + mcp) ──────┴──────────────┘
                    │
              arbitre-web (wasm32) ── core, plan, stockage navigateur
```

Arêtes interdites, vérifiées mécaniquement :
- `arbitre-plan` ↛ `arbitre-diff` et l'inverse (RG-G2, INV-22). Cargo refuse déjà un cycle. L'arête dans un seul sens est détectée par un contrôle CI sur `cargo metadata`.
- `arbitre-core`, `arbitre-plan` ↛ toute crate qui ne compile pas en wasm32. Contrôle CI : `cargo check --target wasm32-unknown-unknown -p arbitre-core -p arbitre-plan`.
- `arbitre-relay` ne dépend que du format d'enveloppe : il ne détient aucune clé. RG-2.8 tient par construction, et la dépendance ne fait que le refléter.

### Arborescence ciblée

```
Cargo.toml                       ← + [workspace.dependencies] (versions partagées)
crates/
├── core/                        ← arbitre-core (retirer .git imbriqué, rusqlite, chrono/clock)
│   └── src/
│       ├── hlc.rs               ← horloge logique hybride, départage par appareil
│       ├── write.rs             ← écriture (donnée, champ, valeur, HLC, appareil, auteur)
│       ├── merge.rs             ← fusion pure, versions écartées, conflits
│       ├── store.rs             ← port Store, erreurs de stockage
│       ├── clock.rs             ← port Clock
│       ├── module.rs            ← trait Module, registre, activation
│       ├── catalog.rs           ← descripteurs d'actions, contexte d'exécution
│       ├── error.rs             ← catégories d'erreur → codes de retour
│       ├── export.rs            ← format d'export (journal) et import = fusion
│       └── envelope.rs          ← enveloppe AEAD versionnée
├── sqlite/                      ← arbitre-sqlite
├── plan/                        ← arbitre-plan
│   └── src/ (item, capacity, placement, questions, blocks, reconcile, duration, agenda_port)
├── calendar/                    ← arbitre-calendar (google/, caldav/ plus tard)
├── diff/                        ← arbitre-diff
│   └── src/ (facts/, python/, invariants, check, extract, decisions, delta, candidates)
├── ai/                          ← arbitre-ai
├── relay/                       ← arbitre-relay
├── cli/                         ← arbitre (cli/, mcp/, scheduler/)
└── web/                         ← arbitre-web (Leptos CSR, manifest, service worker)
.github/workflows/               ← CI : tests, wasm32 check, frontières, gitleaks
action.yml (ou dépôt séparé)     ← GitHub Action de decision diff (AC-15.1)
```

Les noms de fichiers sont indicatifs. Seules les frontières de crate et les ports sont structurants.

---

## Patterns à instaurer

| Pattern | Localisation | Ce qu'il porte | Règles servies |
|---|---|---|---|
| **Journal d'écritures + projection** | `core::write`, `core::merge`, `Store` | Toute mutation est une écriture horodatée HLC, ajoutée au journal. L'état courant est la projection « dernière écriture gagne » par champ. Une suppression est une écriture `supprimé` (tombstone). | RG-1.3/1.4, RG-2.1–2.4, RG-3.6, RG-4.6, RG-6.2, INV-2, 4–7 |
| **Cœur fonctionnel, coquille impérative** | `arbitre-plan`, `arbitre-diff`, `core::merge` | Les règles sont des fonctions pures sur des valeurs. Les E/S (stockage, agenda, git, IA, réseau) restent aux bords, derrière des ports injectés. | Tous les `@Invariant` testables sans simulateur ; `@Property` testées avec simulateurs |
| **Ports et adaptateurs** | ports dans `core` / `plan`, adaptateurs dans les crates natives ou web | Les ports sont `Store`, `Clock`, `Transport`, `Ia`, `Agenda` et l'accès au dépôt. Chaque port a un adaptateur réel et un simulateur. | Tous les simulateurs listés dans les specs |
| **Catalogue d'actions** | `core::catalog`, implémenté par chaque module | Chaque action a un nom, un schéma d'entrée et un schéma de sortie, porte un drapeau « mutante » et une classe de journalisation (suppression/déplacement), et expose une exécution. Les surfaces sont dérivées du catalogue. | RG-3.1–3.4, RG-4.1, RG-5.4, RG-6.4, INV-10–15, 23, 28 |
| **Identifiants déterministes** | blocs, bloc quotidien, questions, décisions | L'identité dérive du contenu (élément, jour, situation, catégorie et objet), pas du hasard ni de l'appareil. Deux appareils qui produisent la même chose produisent la même clé, et la fusion déduplique. | RG-10.7, RG-13.7, AMB-22, RG-17.1, INV-56, 76 |
| **Rang par indexation fractionnaire** | `plan::item` | Le rang est une clé ordonnable, insérée entre deux voisins, départagée par l'identifiant. Déplacer un élément n'écrit que son propre rang, et l'ordre reste total après n'importe quelle fusion. | RG-9.1, RG-9.2, CONFLIT-2, INV-38, 39 |
| **Réconciliation déclarative** | `plan::reconcile` | Une fonction pure prend le plan de blocs attendu et les événements de l'outil effectivement lus, et rend des opérations d'agenda. Elle ne reçoit jamais d'événement tiers. | RG-10.1–10.7, RG-12.1, PROP-10, 14, 15, 16 |
| **Rapports déterministes** | `arbitre-diff` | Tout parcours est trié (fichiers, invariants, violations) dans des structures ordonnées, sans dépendre de l'ordre d'un `HashMap` ni du système de fichiers. | RG-15.2, INV-65 |
| `workspace.package` (existant) | `Cargo.toml` | Édition et licence héritées. À étendre avec `workspace.dependencies`. | — |

---

## Modèle de données du socle

Ce paragraphe est structurant : c'est le contrat entre le socle et les modules.

| Notion | Forme | Remarques |
|---|---|---|
| Donnée | `(module, genre, id)` + ensemble de champs | Le socle ignore le schéma métier. Les valeurs sont sérialisées et opaques pour lui. Le module valide. |
| Écriture | `(donnée, champ, valeur, hlc, appareil, auteur)` | `auteur` ∈ {utilisateur via CLI, GUI, assistant via MCP, système via recalcul}. Le journal de l'assistant (RG-4.6) est une requête sur le journal, et l'atomicité action+trace (« les deux ou aucune ») est gratuite. |
| HLC | (temps physique, compteur logique, appareil) | Arbitrage CONFLIT-4/5. Ordre total et causal. L'horloge physique vient du port `Clock`. |
| Projection | état courant par champ | Calculée par la fusion. Matérialisée par l'adaptateur de stockage ou tenue en mémoire (quelques centaines d'éléments). |
| Versions écartées | écritures perdantes, conservées dans le journal | Le journal n'est jamais compacté en v1. Voir Contraintes, plafond. |
| Conflit signalé | champ modifié des deux côtés depuis la dernière synchro | Le stockage tient un **filigrane par appareil pair** (dernier HLC reçu de lui). Un écrasement séquentiel n'est pas un conflit. |
| Activation des modules | réglage local de l'appareil, non synchronisé | Arbitrage AMB-12. Les données de tous les modules se synchronisent toujours. Le stockage ne filtre pas par activation : c'est le registre de modules qui filtre. |
| Export | fichier texte : en-tête (format, version) + écritures, une par ligne | Lisible (AC-1.3). `import(T, F) = fusion(T, écritures(F))` donne INV-2 par construction. INV-3 : tout le fichier est validé avant une transaction unique. |
| Enveloppe de synchro | `(n° de format, appareil, séquence, nonce, chiffré AEAD d'un lot d'écritures)` | Arbitrage MANQUE-6. Le numéro de format et l'appareil auteur sont présents dès v1, pour qu'on puisse signer plus tard sans migration. |

**Validation après fusion (CONFLIT-1).** La fusion du cœur est aveugle au métier. Chaque module expose un crochet de revalidation des données touchées par une fusion ou un import. Une donnée invalide est conservée et marquée, puis elle donne lieu à une question (RG-G6). Sur un appareil où le module est désactivé, la revalidation a lieu à l'activation.

**Données du module decision diff.** Leur source de vérité est **le dépôt de code** : fichier d'invariants et fichier de décisions, versionnés. Le stockage local ne porte au plus que des réglages. Conséquence : `arbitre diff` doit fonctionner sans stockage local initialisé (CI) — voir Points à trancher, D-2.

---

## Interfaces à définir

Ces signatures sont en pseudo-notation, sans code. Le Skeleton les traduira.

### Ports du socle (`arbitre-core`)

```
Port Clock:
    - now() -> Instant UTC
    - fuseau() -> Fuseau          // limites de jour (AMB-8) ; base des fuseaux embarquée, wasm compris

Port Store:
    - appliquer(lot d'écritures) -> Result   // atomique : tout ou rien (RG-1.6, 1.7, INV-3)
    - etat(donnée) / parcourir(module, genre) -> projection
    - journal(donnée, champ) -> écritures     // versions écartées, historique
    - ecritures_depuis(filigrane) -> écritures   // pour la synchro et l'export
    - filigrane(appareil pair) / avancer_filigrane(...)
    - appareil() -> IdAppareil
    Erreurs : StockagePlein, Corrompu, Indisponible

Fonction pure fusion(projection, écritures entrantes, filigranes) -> (projection', écartées, conflits)
    // commutative, associative, idempotente (INV-5)

Trait Module:
    - id()                                   // "plan" | "diff"
    - actions() -> [DescripteurAction]
    - executer(action, entrée, Contexte) -> Result<Sortie>
    - revalider(données fusionnées) -> [Problème]   // CONFLIT-1

DescripteurAction:
    - nom, résumé, schéma d'entrée, schéma de sortie
    - mutante: bool
    - journalisée_comme: aucune | suppression | déplacement   // RG-4.6

Contexte (fourni par la surface):
    - auteur (surface), clock, vue Store restreinte à l'espace du module (INV-22), ia: Option<Port Ia>

Sortie:
    - valeur structurée (canonique, sérialisable) + rendu humain dérivé   // INV-13

Catégorie d'erreur -> code de retour                               // INV-12, 14, AMB-16
    usage | refus métier | violation (diff) | erreur d'outil | module non activé

Port Transport (synchro):
    - pousser(enveloppes) / tirer(depuis curseur) -> enveloppes
    // natif : client HTTP ; web : fetch

Port Ia:
    - completer(requête structurée, délai) -> Result<réponse structurée>
    Erreurs : NonConfiguré, Injoignable, CléRefusée, DélaiDépassé   // RG-4.5
```

### Ports et fonctions du module planning (`arbitre-plan`)

```
Port Agenda (lecture, plusieurs agendas):
    - occurrences(agenda, intervalle) -> [Occurrence]    // récurrences déjà dépliées (arbitrage)
Occurrence: début, fin, journée_entière, absence, réponse (acceptée/déclinée/…), occupé/disponible, marqueur_outil?

Port Agenda (écriture, un seul agenda):
    - evenements_de_l_outil(intervalle) -> [ÉvénementOutil]   // filtrés par marqueur opaque
    - appliquer(OpérationAgenda) -> Result
    // l'adaptateur refuse toute opération sur un événement sans marqueur (défense en profondeur, PROP-14)

Fonctions pures:
    capacite(jour, occurrences comptées, règles) -> minutes            // RG-8.1, 8.2
    charge(jour, éléments) ; debordement(jour)                          // RG-8.3, 8.4
    placer(pile, capacités, jours candidats) -> affectations + avertissements   // RG-13.1–13.3
    questions(situation) -> [Question à id déterministe]               // RG-13.5, 13.7
    plan_de_blocs(éléments planifiés) -> blocs attendus                 // RG-10.1, 10.2, 11.*
    reconcilier(blocs attendus, événements de l'outil) -> [OpérationAgenda] + déplacements lus  // RG-10.*, 12.1
    duree_proposee(élément, corrections) ; semblable(e1, e2)            // RG-7.*

Orchestration recalcul(now, Store, Agenda) :
    lire occurrences -> capacités datées (« lu le », RG-8.5) -> déplacements entrants
    -> placement des nouveaux (AMB-6) -> questions -> réconciliation des blocs
    // idempotent (INV-56) ; rattrapage sûr au démarrage
```

Le planning stocke **les capacités calculées par jour, avec leur date de lecture**. Il ne stocke pas les occurrences d'agenda. Le téléphone affiche ainsi capacité et fraîcheur sans recevoir le contenu des réunions.

### Fonctions du module decision diff (`arbitre-diff`)

```
Faits de code (modèle neutre, indépendant du langage):
    imports, appels, annotations de type, symboles publics, chacun avec emplacement (fichier, ligne)
    // nouveau langage = nouvel extracteur de faits ; les familles d'invariants ne changent pas

extraire_faits(langage, fichier) -> Faits | NonPrisEnCharge | Invalide       // RG-15.4, 15.5
lire_invariants(fichier) -> Invariants | Erreur(invariant fautif, raison)      // RG-14.*
verifier(faits, invariants) -> Verdict + rapport trié                          // RG-15.*, fonction pure
    // invariants appliqués = union(base, tête) ; affaiblissement ou retrait -> alerte distincte (arbitrage CONFLIT-12)
decisions_structurelles(faits base, faits tête, changements) -> [Décision à id de contenu]   // RG-16.1, 16.5, 16.6
couverture(décision, verdict) -> respecte | viole | non couverte               // RG-16.2
ecart(décisions n−1, décisions n, fichier de décisions) -> ajoutées, retirées, modifiées, acceptations perdues   // RG-17.*, arbitrage CONFLIT-13
candidats(dépôt, Port Ia) -> [Candidat] ; preuves et contre-exemples = verifier(...)   // RG-18.*

Port Dépôt (accès git):
    - fichiers(réf) / contenu(réf, chemin) / fichiers_modifiés(base, tête)
```

Publication sur la PR (AC-16.4) : l'outil rend la liste en Markdown, et la GitHub Action la poste. `arbitre-diff` ne contient pas de client GitHub.

---

## Flux principaux

| Flux | Chemin | Points d'attention |
|---|---|---|
| Écriture locale (toute surface) | surface → catalogue → `Module::executer` → validation → lot d'écritures → `Store::appliquer` (transaction) → confirmation | « Confirmée » = transaction validée (AMB-21, RG-1.6). Sur l'ordinateur, la CLI, le MCP et le recalcul partagent le même fichier SQLite (WAL). |
| Synchro | `Store::ecritures_depuis` → enveloppe AEAD → `Transport` → relais → autre appareil → déchiffrement (rejet si altéré) → `fusion` → revalidation module | Le relais est facultatif (RG-2.6). La date de la dernière synchro réussie est stockée par appareil. |
| Recalcul du matin | planificateur système → `arbitre plan recalcul` (natif) → `recalcul(...)` → écritures + opérations d'agenda | Seul un appareil natif parle à l'agenda. Le téléphone reçoit capacités, questions et jours planifiés par la synchro. |
| Vérification decision diff | `arbitre diff check` (local ou Action) → `Port Dépôt` → faits base/tête → invariants base ∪ tête → verdict → code de retour | Sans réseau ni IA (INV-64) ; sans stockage local (D-2). |
| Revue de décisions | `arbitre diff` → décisions + écart → acceptation/rejet → fichier de décisions écrit dans le dépôt → commit avec la PR | Écrit par la CLI ou le MCP. Rien ne s'accepte depuis la page GitHub en v1. |

---

## Classification DDD

Trois bounded contexts : **Socle**, **Planning**, **Decision diff**. Les modules ne se touchent jamais. Le socle les touche par le trait `Module` et le catalogue.

| Composant | Zone | Rigueur C-TDD |
|---|---|---|
| HLC, fusion, tombstones, versions écartées, détection de conflit | Core Domain | Full C-TDD — Mutation ≥ 85 %, CI > 0,95. Tests par propriétés (commutativité, associativité, idempotence). |
| Import = fusion, import atomique, export intégral | Core Domain | Full C-TDD |
| Capacité, charge, débordement, placement, urgence | Core Domain | Full C-TDD |
| Questions (id déterministe, rien ne bouge sans réponse) | Core Domain | Full C-TDD |
| Plan de blocs, réconciliation, confidentialité des blocs (RG-11.*) | Core Domain | Full C-TDD ; PROP-14 bloquant |
| Rang fractionnaire, pile | Core Domain | Full C-TDD |
| Vérification des invariants de dépôt (3 familles), verdict | Core Domain | Full C-TDD ; déterminisme testé par permutation de l'ordre des fichiers |
| Identité de décision, écart entre itérations, acceptations perdues | Core Domain | Full C-TDD |
| Enveloppe AEAD, dérivation de clé | Supporting (sécurité) | Full C-TDD malgré la zone : vecteurs de test, rejet des altérations |
| Catalogue d'actions, registre de modules, parité, codes de retour | Supporting | C-TDD standard ; test de conformité générique appliqué à chaque module |
| Durée proposée, proximité d'intitulés | Supporting | C-TDD standard |
| Extracteur de faits Python, décisions structurelles | Supporting | C-TDD standard + harnais d'évals (MANQUE-1) pour la précision |
| Candidats d'invariants (IA) | Supporting | C-TDD standard, fournisseur simulé |
| Adaptateur SQLite, stockage navigateur | Generic | Classic AI-Assisted + propriétés d'arrêt brutal et de disque plein |
| Adaptateurs Google Calendar / CalDAV | Generic | Classic AI-Assisted + tests de contrat |
| Adaptateur IA multi-fournisseurs | Generic | Classic AI-Assisted + tests de contrat |
| Relais HTTP | Generic | Classic AI-Assisted |
| Rendu CLI, transport MCP, UI Leptos, planificateur système, GitHub Action | Generic | Classic AI-Assisted |

---

## Dépendances

### Internes : graphe des briques

Une flèche se lit « est requis par ». Le Planner découpe dans ce graphe. RG-G1 exige seulement que les briques du socle **dont une story dépend** soient livrées, pas tout le socle.

```
[S-1] HLC + écriture + fusion (pur)
   └─► [S-2] Store + arbitre-sqlite + export/import
          └─► [S-3] Module + catalogue + CLI dérivée + codes de retour + --json
                 ├─► [S-4] MCP dérivé du catalogue (+ journal assistant)
                 ├─► [P-1] Pile : élément, validité, cycle de vie, rang, durée proposée  (US-6, 7, 9)
                 │      └─► [P-2] Capacité/charge/débordement sur occurrences simulées  (US-8)
                 │             └─► [P-3] Agenda Google (lecture) + recalcul + planificateur  (US-8, 13)  ◄─ S3
                 │                    └─► [P-4] Placement hebdo + questions  (US-13)
                 │                           └─► [P-5] Blocs : plan, réconciliation, écriture, confidentialité  (US-10, 11, 12)
                 └─► [D-1] Faits Python + fichier d'invariants + vérification + `arbitre diff check`  (US-14, 15)  ◄─ S4
                        ├─► [D-2] GitHub Action (CI)  (AC-15.1)
                        └─► [D-3] Décisions structurelles + fichier de décisions + écart  (US-16, 17)
                               └─► [D-4] Décisions IA optionnelles + candidats  (US-16, 18)  ◄─ [S-7]
   [S-2] ─► [S-5] Enveloppe AEAD + relais + client de synchro natif  (US-2)
   [S-3] + [S-5] + S1 + S2 ─► [S-6] PWA Leptos + stockage navigateur + synchro web  (US-2, C-2.9)
   [S-3] ─► [S-7] Adaptateur IA (Anthropic + compatible OpenAI)  (US-4 ; requis seulement par D-4)
```

Ce que ce graphe autorise :
- Le planning sur CLI (P-1 à P-5) n'attend ni la synchro, ni la PWA, ni l'IA.
- L'adaptateur IA peut arriver juste avant D-4 : rien d'autre ne l'exige (RG-4.4).
- S-5 et S-6 (téléphone) sont parallélisables avec le planning dès S-3.
- P-2 se développe entièrement sur simulateur d'agenda, avant que S3 n'ait rendu son verdict.

### Externes

| Dépendance | Type | Risque | Isolée par |
|---|---|---|---|
| SQLite (rusqlite `bundled`) | Bibliothèque native | Faible | `arbitre-sqlite` |
| SQLite WASM + OPFS (ex. `sqlite-wasm-rs`, ou le support wasm de rusqlite s'il est mûr) | Bibliothèque wasm | **Élevé** : maturité, worker obligatoire pour OPFS synchrone, Safari iOS | port `Store` ; spike S1 ; repli IndexedDB |
| Leptos (CSR) + outil de build (trunk) | Framework UI | Moyen : API mouvante, service worker hors Leptos | `arbitre-web` ; spike S2 |
| AEAD XChaCha20-Poly1305, Argon2id (crates RustCrypto) | Bibliothèques pures Rust | Faible ; `getrandom` demande sa feature wasm | `core::envelope` |
| Google Calendar API v3 | API | **Élevé** : OAuth, vérification d'application, visibilité des titres, quotas | port `Agenda` ; spike S3 ; tests de contrat |
| CalDAV | Protocole | Moyen (hétérogénéité des serveurs) | port `Agenda` ; après Google |
| tree-sitter-python **ou** parseur de Ruff | Bibliothèque | **Moyen** : le parseur de Ruff n'est pas publié sur crates.io (dépendance git, sans semver) | `arbitre-diff::python` ; spike S4 |
| git (CLI en sous-processus, ou `gix`) | Outil / bibliothèque | Faible | port `Dépôt` |
| SDK MCP Rust (`rmcp`) | Bibliothèque | Moyen (jeune) | `arbitre` (cli) ; spike court S5 |
| API Anthropic, compatible OpenAI (OpenAI, Mistral, Ollama) | API | Moyen : indisponibilité, clé, délai | port `Ia` ; tests de contrat |
| launchd (macOS), timer systemd (Linux) | Système | Faible ; rattrapage différent entre les deux | sous-commande d'installation ; recalcul idempotent |
| GitHub Actions, `gh` | Hébergeur | Faible | Action ; `arbitre-diff` ne parle pas à GitHub |
| Trousseau du système (jetons OAuth, clés IA, secret de synchro) | Système | Faible | adaptateurs natifs ; repli fichier à droits restreints |

---

## Spikes à faire avant d'engager un choix

Chaque spike a un budget de temps fixe et rend un verdict écrit. Il ne laisse pas de code en production.

| # | Question | Critères de sortie (tous requis) | Repli si échec | Bloque |
|---|---|---|---|---|
| **S1** | SQLite en WASM avec OPFS dans la PWA | (1) Le même adaptateur, ou un adaptateur jumeau, compile en `wasm32-unknown-unknown`. (2) La persistance survit au rechargement et à la fermeture de l'onglet. (3) Une écriture confirmée survit à la fermeture brutale (RG-1.6). (4) Le quota dépassé remonte une erreur exploitable (RG-1.7). (5) Fonctionne en PWA installée sur Safari iOS et Chrome Android. (6) L'architecture worker est acceptable : UI ↔ worker de stockage. (7) Le poids du bundle est noté. | Journal d'écritures dans IndexedDB, projection en mémoire. Le port `Store` le permet sans toucher au cœur. | S-6 seulement |
| **S2** | PWA Leptos hors ligne, installable sans store | Manifest + service worker (cache de l'app), installation iOS et Android, ouverture hors ligne, origine HTTPS choisie (servie par le relais ou par un hébergement statique). Mesurer l'éviction du stockage par Safari pour une PWA non installée. | Le téléphone en consultation seule tant que S1/S2 ne sont pas levés (à décider) | S-6, C-2.9 |
| **S3** | Google Calendar depuis une CLI open source | (1) OAuth « application installée » (boucle locale + PKCE) **sans secret client dans le dépôt** (RG-G9). (2) Portée et vérification Google : plafond d'utilisateurs non vérifiés, avertissement. (3) `singleEvents=true` donne bien des occurrences seules. (4) Statut de réponse, journée entière, `outOfOffice`, `transparency` lisibles. (5) Le marqueur opaque (`extendedProperties.private`) est invisible des invités et des collègues. (6) Le titre « Focus — X » est visible d'un collègue selon le partage de l'agenda, et rien d'autre ne l'est (AC-11.1/11.2). | Client OAuth fourni par l'utilisateur (BYO) ; CalDAV avancé si Google bloque | P-3, P-5 |
| **S4** | Parseur Python : tree-sitter-python ou parseur de Ruff | Sur 2–3 dépôts OSS : imports absolus et relatifs résolus en modules, appels, annotations de type, symboles publics, emplacements exacts, déterminisme, temps sur un dépôt de taille moyenne, mode de dépendance (crates.io contre git épinglé). | tree-sitter-python (publié, versionné) | D-1 |
| **S5** | SDK MCP Rust | stdio, liste d'outils dynamique (activation des modules, INV-23), schémas d'entrée dérivés du catalogue, compatibilité avec au moins deux clients (Claude Code, un autre). Budget court. | Implémentation JSON-RPC stdio minimale à la main | S-4 |

Ce qui n'est **pas** un spike : HLC et fusion, qu'on construit en C-TDD par propriétés ; l'adaptateur IA (deux protocoles, bien documentés) ; le planificateur système, vérifié dans la story P-3 grâce à l'idempotence.

---

## Contraintes et limites

- **Frontière wasm** : `arbitre-core` et `arbitre-plan` n'utilisent ni système de fichiers, ni réseau, ni horloge système, ni thread. Le temps vient de `Clock`, et l'aléa (nonces) d'une source injectée ou de `getrandom` avec sa feature wasm.
- **Déterminisme** : les fonctions de `arbitre-plan` et `arbitre-diff` sont pures à entrées égales. Les sorties sont triées. Aucune dépendance à l'ordre d'itération d'un `HashMap` ni au fuseau de la machine de CI (RG-15.2, RG-13.7).
- **Isolation des modules** : à la compilation par le graphe Cargo, à l'exécution par une vue `Store` restreinte à l'espace de noms du module (INV-22). Un module désactivé n'enregistre aucune action : il est absent de l'aide, de la complétion et du catalogue MCP. Un appel explicite renvoie « module non activé » (CONFLIT-6).
- **Agenda et IA natifs seulement (v1)** : la PWA ne contacte ni l'agenda ni un fournisseur d'IA. Pas de jeton OAuth ni de clé dans le navigateur. RG-1.2 se vérifie sur un seul point de sortie par service.
- **Un seul agenda en écriture** et des événements de l'outil reconnus par un marqueur opaque dérivé de l'identifiant de l'élément. Deux ordinateurs qui recalculent produisent les mêmes marqueurs, et la réconciliation ne crée pas de doublon (RG-10.7).
- **Sécurité de la synchro** : la clé de chiffrement est dérivée d'une phrase secrète de l'utilisateur (Argon2id) et ne quitte jamais les appareils. Le jeton d'accès au relais est un secret distinct. Le relais voit les métadonnées (appareil, taille, moment) : accepté, à documenter.
- **Performance** : la pile tient en mémoire (quelques centaines d'éléments, RG-9.5). Le recalcul ne fait qu'une lecture d'agenda par agenda et par semaine. La propagation en moins de 60 s (RG-2.5) s'obtient par interrogation périodique du relais quand l'app est ouverte, sans connexion poussée en v1.
- **Plafond assumé** : le journal n'est jamais compacté, puisque les versions écartées doivent rester récupérables (RG-2.4). C'est acceptable pour un outil personnel. Une compaction avec archive viendra quand la taille du journal se mesurera.
- **Migrations** : le format d'écriture, le format d'export et l'enveloppe portent chacun un numéro de version dès v1. Le schéma SQLite reste générique (journal + projection) : un changement de modèle métier ne demande pas de migration SQL.
- **Codes de retour** : à fixer une fois, dans le socle. Proposition alignée sur AMB-16 : succès, refus/violation, usage (clap rend 2 par défaut), erreur d'outil, module non activé. Le detail est à fixer dans la story S-3.
- **Bounded contexts externes** : un test de contrat par adaptateur (Google Calendar, CalDAV, chaque protocole IA, MCP), rejoué contre des réponses enregistrées. Un test de conformité générique s'applique à tout `Module` : activation, invisibilité, espace de noms, parité du catalogue.
- **CI dès la première story** : `cargo test`, `cargo check --target wasm32-unknown-unknown` sur core et plan, contrôle des arêtes interdites, gitleaks.

---

## Anti-patterns détectés (dette existante impactante)

| Anti-pattern | Localisation | Impact | Recommandation |
|---|---|---|---|
| **Dépôt git imbriqué** (laissé par `cargo new`) | `crates/core/.git`, `crates/core/.gitignore` | Au premier `git add`, `crates/core` entre comme gitlink, sans son contenu. Le dépôt public serait cassé dès le premier commit. | **Bloquant, avant le premier commit** : supprimer `crates/core/.git`. Le `.gitignore` local est redondant. |
| **Stockage natif dans le cœur** | `crates/core/Cargo.toml` : `rusqlite` (`bundled`) | Empêche `arbitre-core` de compiler en wasm32, et donc toute la PWA. Le domaine est couplé à SQL. | **Bloquant pour S-6** : déplacer vers `arbitre-sqlite` dès la story S-2. |
| **Horloge système implicite dans le cœur** | `crates/core/Cargo.toml` : `chrono` feature `clock` | `now()` n'est pas injectable : il rend impossibles les simulateurs d'horloge exigés par les specs (PROP-5, 19, 21) et le déterminisme. En wasm, il réclame en plus `wasmbind`. | Retirer la feature `clock` du cœur ; temps via le port `Clock`. La CLI peut la garder. |
| Code de démonstration | `core/src/lib.rs` (`add`), `cli/src/main.rs` | Aucun | Remplacé par la première story. |
| Versions dupliquées | `chrono` dans deux `Cargo.toml` | Dérive de versions | `[workspace.dependencies]`. |

---

## Points à trancher (niveau architecture)

Ils sont nouveaux : aucun n'est couvert par les arbitrages du 2026-11-02. Chacun porte une recommandation, que le Planner peut retenir par défaut.

| # | Question | Pourquoi ça compte | Recommandation |
|---|---|---|---|
| **D-1** | Sur l'ordinateur, la PWA est-elle une réplique de plus (stockage navigateur propre) ou un client de la base native ? | Le navigateur n'accède pas au fichier SQLite. Si la PWA est une réplique, une modification CLI n'apparaît dans la PWA de l'ordinateur qu'en passant par un relais. | La PWA est **toujours une réplique**, sur l'ordinateur comme sur le téléphone : un seul modèle. Sur l'ordinateur, la CLI reste la surface principale, et `arbitre relay serve` peut tourner en local. Tauri, plus tard, pourra brancher l'UI sur la base native. |
| **D-2** | `arbitre diff` en CI, sans appareil initialisé : comment s'articule-t-il avec l'activation des modules (RG-3.7, RG-5.4) ? | Une CI n'a ni stockage ni réglages. | Aucun cas particulier : la GitHub Action exécute `arbitre module enable diff` dans un répertoire de configuration éphémère. `arbitre diff check` ne lit pas le stockage local. |
| **D-3** | Où vit la frontière d'agenda ? L'arbitrage dit « le socle ne voit jamais une règle de récurrence ». | Seul le planning consomme l'agenda. | Le port `Agenda` vit dans `arbitre-plan`, et les adaptateurs dans `arbitre-calendar` (natif). L'arbitrage est tenu : seules des occurrences franchissent la frontière. Le socle n'a aucune notion de réunion. |
| **D-4** | Client OAuth Google : fourni par le projet ou par l'utilisateur ? | Un secret client dans un dépôt public contredit RG-G9 et déclenche gitleaks. La vérification Google d'une application publique est lourde. | Dépend de S3. Par défaut en v1, l'utilisateur fournit son propre client OAuth (documenté). Un client du projet viendra plus tard, si la vérification Google est faite. |
| **D-5** | Fichier de décisions : un fichier unique ou un fichier par PR/décision sous `.arbitre/decisions/` ? | Un fichier unique produit des conflits de fusion entre PR concurrentes, et il s'accumule sur `main` après fusion. | Un fichier par décision (nom = identité de contenu), ou un par branche, à fixer au cycle de US-16. Ne pas retenir le fichier unique. |
| **D-6** | Qui sert la PWA (origine HTTPS) ? | Le stockage OPFS est propre à chaque origine. La PWA exige HTTPS, et donc un certificat chez l'utilisateur s'il l'auto-héberge. | Laisser S2 trancher entre deux options : le relais sert aussi les fichiers statiques de la PWA (une seule chose à héberger), ou un hébergement statique public (GitHub Pages) qui parle au relais de l'utilisateur. |

---

## Notes pour le Planner

- La **première story** est forcément S-1/S-2. Elle corrige d'abord les trois anti-patterns du cœur, puis pose la CI avec la vérification wasm32 et le contrôle des frontières.
- Les **spikes S3 et S4** peuvent tourner en parallèle de S-1 à S-3 : ils ne touchent pas au code de production. S1 et S2 n'ont besoin d'aboutir qu'avant S-6.
- Une crate n'existe pas tant qu'aucune story ne la remplit.
- Les `RENVOI-*` et `[À DÉFINIR]` des specs n'ont pas d'impact structurel. Aucun ne change une frontière de crate ni un port. Ils se tranchent au cycle de leur story.
- `MANQUE-1` (harnais d'évals) est un outil de développement de `arbitre-diff` : un binaire ou des tests dans cette crate, hors du produit livré.

## Arbitrages de Quentin sur l'architecture (2026-11-02)

Ils tranchent D-1 à D-6 et font foi.

- **D-1** : la PWA est toujours une réplique (stockage navigateur et synchronisation par le relais), sur l'ordinateur comme sur le téléphone. Un seul modèle. Sur l'ordinateur, la CLI reste la surface principale ; Tauri, plus tard, branchera l'interface sur la base native.
- **D-2** : en CI, l'Action active `diff` dans un répertoire de configuration éphémère, puis lance `arbitre diff check`, qui ne lit que le dépôt. Aucun cas particulier dans le code.
- **D-3** : le port `Agenda` vit dans `arbitre-plan` ; ses adaptateurs vivent dans `arbitre-calendar` (natif). Le socle ne connaît ni réunions ni agendas.
- **D-4** : en v1, chaque utilisateur fournit son propre client OAuth Google (guide documenté) ; aucun secret dans le dépôt. Un client du projet viendra plus tard, avec la vérification Google. **Critère ajouté au spike S3** : le client OAuth personnel est accepté par un agenda Google Workspace (règles d'accès des applications tierces fixées par l'admin).
- **D-5** : un fichier d'acceptations par branche, en TOML, sous `.arbitre/decisions/<branche>.toml`. Il s'ouvre sur un en-tête (titre de la PR, branche, date) et contient un bloc par décision aux champs explicites (quoi, où, origine, statut, par, le, note). Il doit rester lisible à la main par quelqu'un qui creuse l'historique. Le format exact des champs se fixe au cycle de US-16.
- **D-6** : `arbitre relay serve` sert aussi les fichiers de la PWA, à la même adresse que la synchronisation (pas de CORS, une seule chose à héberger). HTTPS par un proxy devant le relais (Caddy ou Let's Encrypt).
