# Cadrage Architectural — ST-01 : Fusion pure et fondations du dépôt

## Traçabilité
- Sources : `.ctdd/stories/ST-01/02_specs.md` (ses « Arbitrages de Quentin (2026-11-03) » font foi), `.ctdd/stories/ST-01/01_story.md`, `.ctdd/cadre/03_archi.md` (décliné, pas rediscuté), `.ctdd/cadre/04_plan.md` § ST-01, PL-6, PL-8, § Collisions, `.ctdd/spikes/S1-A.md`.
- Date : 2026-11-03 · Statut : DRAFT · Zone : Core / Full.
- Codebase relevée le 2026-11-03 (voir « Périmètre codebase »). Versions relevées sur crates.io le même jour.

## Synthèse

1. **Le moteur est une fonction pure de l'ensemble des écritures.** Grâce au champ `vu` (CONFLIT-ST01-1, option a), projection, écartées **et** conflits se calculent à partir du seul journal connu `S`. Les points de dernière réception (filigranes) sortent de la signature de la fusion : ils restent l'outil de la synchronisation (ST-02, ST-19).
2. **Le cœur de ST-01 n'a aucune dépendance normale.** `proptest` est sa seule dépendance de développement. La frontière wasm est donc tenue par construction, et la CI la garde.
3. **Les états impossibles ne sont pas représentables.** L'appareil se lit dans la marque, l'auteur est une énumération fermée, une suppression n'a pas de valeur et le seuil d'avance est non nul. Ce qui reste à valider à l'exécution tient en deux règles : `vu < marque` et des identifiants non vides.
4. **La fusion est locale à une donnée.** Une écriture n'entre en relation (gagnante, exclusivité, conflit) qu'avec les écritures de sa propre donnée. ST-02 peut donc ne charger que les données touchées par un lot.
5. **La CI se réduit à un exécuteur gelé et à des fichiers de contrôle.** `scripts/ci.sh` ne fait qu'exécuter `scripts/ci.d/*.sh`. Les cinq contrôles de ST-01 en sont les premiers fichiers. Le code d'aide est en Python 3.13, bibliothèque standard seulement.

---

## Périmètre codebase

### État de l'existant

| Élément | Contenu | Constat |
|---|---|---|
| `Cargo.toml` | `resolver = "3"`, `members = ["crates/*"]`, `workspace.package` (édition 2024, Apache-2.0) | Sain. Il manque `[workspace.dependencies]`. |
| `crates/core` (`arbitre-core`) | `lib.rs` = `add` ; dépend de `chrono` 0.4.45 (`clock`) et `rusqlite` 0.41.1 (`bundled`) | Le `.git` imbriqué a disparu (G-0 fait). Les deux dépendances restent à retirer. |
| `crates/cli` (`arbitre`) | `main.rs` = « Hello, world! » ; dépend de `arbitre-core`, `chrono` (`clock`), `clap` 4.6.7 (`derive`) | `chrono` n'y sert à rien. |
| `Cargo.lock` | versionné | À garder : toute la CI tourne en `--locked`. |
| `.pre-commit-config.yaml` | gitleaks local `--staged` | À garder. Sa version est épinglée par le même fichier que la CI (voir « Épinglage »). |
| CI, `scripts/`, `docs/`, `CONTRIBUTING.md`, `rust-toolchain.toml` | absents | Tous créés par ST-01. |
| Outillage local | rustc 1.99.1, cible `wasm32-unknown-unknown` installée, gitleaks 8.30.1, Python 3.15, jq | `cargo-mutants` n'est **pas installé**. |

### Modules impactés

| Module | Rôle actuel | Impact | Action |
|---|---|---|---|
| `crates/core/src/` | démonstration | Direct : marques, horloge, écritures, fusion, port `Clock` | Réécriture |
| `crates/core/Cargo.toml` | `chrono`, `rusqlite` | Les deux sortent. Ajout de `proptest` en dev et d'une feature `sim` | Modification |
| `crates/cli/` | démonstration | Retrait de `chrono`. `main.rs` minimal (clap, `--version`), en attendant ST-03 | Modification |
| `Cargo.toml` racine | workspace | `[workspace.dependencies]` complète (PL-6) | Extension |
| `scripts/`, `.github/workflows/` | — | Point d'entrée CI, contrôles, aides | Création |
| `docs/decisions/`, `CONTRIBUTING.md`, `rust-toolchain.toml` | — | Décision de dépendances, règles de contribution, toolchain | Création |
| `.ctdd/stories/ST-01/` | cycle | `zones.toml`, `mutants_equivalents.toml` (si besoin) | Création |

### Arborescence ciblée

```
rust-toolchain.toml              ← toolchain épinglée (créé)
Cargo.toml                       ← + [workspace.dependencies], + rust-version
CONTRIBUTING.md                  ← Cargo.lock, ajout d'un contrôle, zones, équivalents (créé)
docs/decisions/ST-01-dependances.md   ← AC-8.1 (créé)
crates/
├── core/
│   ├── Cargo.toml               ← plus de dépendance normale ; dev : proptest ; feature `sim`
│   ├── src/
│   │   ├── lib.rs               ← `mod` + réexports (une ligne par module, ajouts en fin)
│   │   ├── hlc.rs               ← IdAppareil, Marque, Horloge, Seuil, avances()
│   │   ├── write.rs             ← IdDonnee, Champ, Valeur, Auteur, Quoi, Cible, Ecriture, vu_pour(), validation
│   │   ├── merge.rs             ← Journal, fusion, Projection, Conflit, Anomalie, Rapport, ErreurLot
│   │   └── clock.rs             ← trait Clock ; HorlogeSimulee sous `cfg(feature = "sim")`
│   ├── tests/
│   │   ├── support/             ← générateurs proptest, appareils et réseau simulés (PROP-ST01-2)
│   │   └── *.rs                 ← propriétés et cas des matrices
│   └── proptest-regressions/    ← graines d'échec versionnées (E-1.1 « rejouable »)
└── cli/
    ├── Cargo.toml               ← - chrono
    └── src/main.rs              ← clap minimal
scripts/
├── ci.sh                        ← exécuteur, gelé après ST-01
├── outils.env                   ← versions épinglées des outils hors Cargo
├── ci.d/
│   ├── 10-st01-tests.sh
│   ├── 20-st01-navigateur.sh
│   ├── 30-st01-graphe.sh
│   ├── 40-st01-secrets.sh
│   └── 50-st01-mutation.sh
└── lib/
    ├── graphe.py                ← arêtes, dépendances interdites, dépendances communes, pureté
    ├── mutation.py              ← zones, équivalents, scores, verdict, rapport
    └── test_ci.py               ← unittest des deux aides (matrices US-5 à US-7)
.github/workflows/ci.yml         ← un job, appelle scripts/ci.sh (gelé après ST-01)
.gitleaksignore                  ← faux positifs, une justification par entrée (vide au départ)
.ctdd/stories/ST-01/zones.toml
```

Les noms de fichiers Rust suivent `03_archi.md`. Les autres sont structurants, parce que les stories suivantes s'y branchent.

---

## Patterns existants identifiés

| Pattern | Localisation | Respecter ? | Notes |
|---|---|---|---|
| `workspace.package` hérité | `Cargo.toml` | Oui | On l'étend avec `rust-version` et `[workspace.dependencies]`. |
| Cœur fonctionnel, coquille impérative | archi globale | Oui, on l'instaure | `merge` et `hlc` ne reçoivent que des valeurs. L'heure physique entre en `u64`, jamais par un appel. |
| Ports et adaptateurs, un simulateur par port | archi globale | Oui, on l'instaure | `Clock` + `HorlogeSimulee`. Pas d'adaptateur système en ST-01 : rien ne le consomme avant ST-03. |
| Rapports déterministes | archi globale | Oui, on l'instaure | `BTreeMap`/`BTreeSet` partout. Les tris de RG-ST01-2.2 découlent des `Ord` dérivés (voir « Ordres »). Aucun `HashMap` dans `arbitre-core`. |
| gitleaks en pre-commit | `.pre-commit-config.yaml` | Oui | La CI ajoute le contrôle par commit (RG-ST01-6.4). |

---

## Modèle de données

### Tuple d'écriture : place de `vu` (point à fixer n° 1)

L'écriture de l'archi globale `(donnée, champ, valeur, hlc, appareil, auteur)` devient :

```
Ecriture = (donnée, quoi, marque, vu, auteur)
    quoi   = Champ(nom, valeur) | Suppression
    marque = (l, c, a)            // a = appareil auteur
    vu     = ⊥ | marque
```

| Choix | Raison |
|---|---|
| **`vu` vient juste après `marque`** | Ce sont les deux métadonnées causales, du même type. L'export texte (ST-04b) et tout format de sérialisation suivent cet ordre de colonnes. |
| `vu` est **hors de l'identité** `(cible, marque)` et hors des clés de tri | RG-2.1 et RG-ST01-2.2 ne le citent pas. Deux écritures de même identité et de `vu` différent relèvent de MANQUE-ST01-2 (anomalie), comme une valeur différente. |
| `vu : Option<Marque>`, avec `None` pour `⊥` | L'ordre dérivé d'`Option` place `None` sous tout `Some` : `⊥` est le plus petit, sans code. |
| **`appareil` n'est plus un champ** : c'est `marque.a` | L'invariant `w.appareil = w.marque.a` (RG-ST01-4.5) devient vrai par construction. Le cas 13 de la matrice US-4 n'est plus représentable : voir « Écarts aux specs ». |
| `quoi` est une énumération | Une suppression sans valeur, ou un champ sans valeur, ne se construit pas. |

**Portée de `vu`** (déclinaison de l'arbitrage, utile au calcul) :
- pour une écriture de champ `(d, f)`, c'est le maximum des marques connues sur `(d, f)` et sur les suppressions de `d` ;
- pour une suppression de `d`, c'est le maximum des marques connues sur **toutes** les écritures de `d`, champs et suppressions.

C'est exactement l'ensemble des écritures avec lesquelles elle peut être `exclusive` ou de même cible (RG-ST01-2.1). Le cœur fournit `vu_pour(journal, donnée, quoi) -> Option<Marque>`, pour que les surfaces (ST-03 et suivantes) ne réimplémentent pas cette règle.

**Contrainte `vu < marque`.** Une émission est strictement supérieure à tout ce que l'appareil a émis ou reçu (RG-ST01-4.2). Cela suppose que toute écriture entrée dans le journal local est passée par `Horloge::recevoir`. C'est un contrat de la coquille (ST-02, ST-19). La validation d'un lot le vérifie : `vu ≥ marque` ⇒ lot refusé.

### Types du cœur

| Type | Forme | Ordre / remarques |
|---|---|---|
| `IdAppareil` | `[u8; 16]` | Ordre octet par octet. Générée par ST-02 (aléa). Constructeur depuis `u128` pour les tests. |
| `Marque` | `{ l: u64, c: u32, a: IdAppareil }` | `Ord` **dérivé, champs dans cet ordre** : c'est RG-ST01-4.1. `l` = millisecondes UTC depuis l'époque Unix. Ne jamais tasser la marque dans un seul `u64` : `l` reste sous 2^53 (S1-A, réserve 2), ce qui n'est plus vrai d'une marque tassée. |
| `Horloge` | état `(l, c)` + `IdAppareil` | `emettre(pt: u64) -> Result<Marque, CompteurEpuise>` ; `recevoir(&Marque)`. Saturation de `c` : erreur, jamais de retour à zéro (MANQUE-ST01-3). |
| `IdDonnee` | `{ module, genre, id }`, trois `String` | Ordre lexicographique sur les octets UTF-8, sans locale. |
| `Champ` | `String` | idem |
| `Valeur` | `Vec<u8>` (sérialisation canonique produite par le module) | Le cœur ne fait que tester l'égalité et l'ordre des octets (départage de MANQUE-ST01-2). |
| `Auteur` | enum `UtilisateurCli`, `UtilisateurInterface`, `Assistant`, `Systeme` | Liste fermée. |
| `Quoi` | enum `Champ { nom, valeur }`, `Suppression` | Dans `Cible`, `Suppression` se trie après tous les champs (ordre des variantes). |
| `Cible` | `(IdDonnee, Champ \| Suppression)` | Clé de tri des écartées : `(donnée, cible, marque)`. |
| `Ecriture` | voir ci-dessus | |
| `Journal` | `BTreeMap<(Cible, Marque), Contenu>`, avec `Contenu = (valeur?, vu, auteur)` | Croît par union, rien n'en sort. L'itération suit déjà l'ordre des écartées. |
| `Projection` | `BTreeMap<IdDonnee, { statut, champs: BTreeMap<Champ, Ecriture> }>` | Les gagnantes d'une donnée supprimée restent présentes, masquées (AMB-ST01-1). |
| `Conflit` | `{ donnée, champ, retenue: Ecriture, ecartee: Ecriture }` | Tri `(donnée, champ, marque retenue, marque écartée)`. Paire suppression/champ : le champ est celui de l'écriture de champ. |
| `Anomalie` | `{ identité, contenus: [Contenu; 2] }` | MANQUE-ST01-2. On retient le plus grand contenu selon l'ordre `(valeur, vu, auteur)`. |
| `Seuil` | `NonZeroU64` en ms, 3 600 000 par défaut | `σ ≤ 0` n'est pas représentable (RG-ST01-4.6). |
| `Avance` | `{ appareil, avance_ms: u64 }` | Tri par appareil. |
| `ErreurLot` | `{ index, raison }` | Seules raisons en ST-01 : `vu ≥ marque`, identifiant vide. ST-02 et ST-04b réutilisent ce type pour leurs erreurs de désérialisation. |

---

## Interfaces à définir

Pseudo-notation, sans code. Le Skeleton la traduira.

```
// hlc.rs
Horloge::nouvelle(appareil) -> Horloge                     // état (0, 0)
Horloge::emettre(&mut self, pt: u64) -> Result<Marque, CompteurEpuise>   // RG-ST01-4.2
Horloge::recevoir(&mut self, m: &Marque)                   // RG-ST01-4.3, max_lex
avances(lot: &[Ecriture], maintenant: u64, σ: Seuil) -> Vec<Avance>       // RG-ST01-4.6, pure, triée

// write.rs
vu_pour(journal: &Journal, donnée: &IdDonnee, quoi: &Quoi) -> Option<Marque>
Ecriture::cible(), Ecriture::appareil()                    // appareil() = marque.a

// merge.rs
Journal::vide() -> Journal
Journal::fusionner(&mut self, lot: &[Ecriture]) -> Result<Rapport, ErreurLot>
    // tout ou rien : le lot est validé en entier avant toute insertion (RG-ST01-4.5)
    Rapport = { nouvelles_ecartees, nouveaux_conflits, anomalies }   // différences, triées
Journal::projection(&self) -> Projection
Journal::ecartees(&self) -> Vec<Ecriture>                  // R(S), pour les propriétés
Journal::conflits(&self) -> Vec<Conflit>

// clock.rs
trait Clock { fn now(&self) -> u64 }                       // ms UTC ; synchrone (Date.now côté wasm)
HorlogeSimulee (feature `sim`) : fixer(t), avancer(Δ), reculer(Δ), figer() ; impl Clock
```

**Déclinaisons de l'archi globale** (à reporter dans `.ctdd/cadre/03_archi.md` par l'orchestrateur, sans les rediscuter) :
- `fusion(projection, écritures, filigranes)` devient `Journal::fusionner(lot)`. Les filigranes ne servent plus à la détection : c'est la conséquence directe de l'option (a).
- `avances` est une fonction à part, et non une sortie de la fusion. `R(S)` reste ainsi une fonction de `S` seul, et RG-2.2 se teste sans heure.
- `Clock::fuseau()` n'est **pas** livrée en ST-01 : aucune règle de la story ne s'en sert. Voir « Points ouverts », P-2.

**Contrat donné à ST-02** (non gelé ici, PL-8) :
- `Store::journal` rend toutes les écritures d'une **donnée**, et pas seulement celles d'un champ, puisque l'exclusivité couvre la donnée ;
- le moteur reste synchrone et ne voit jamais le port `Store` asynchrone. La coquille charge, puis fusionne, puis applique ;
- les types du cœur ne portent ni `Rc` ni `RefCell` : ils restent `Send` sans que rien ne l'exige (S1-A, réserve 1).

---

## Classification DDD

| Composant | Zone | Rigueur C-TDD |
|---|---|---|
| `hlc.rs` : marque, horloge, avances | Core Domain | Full : mutation ≥ 85 %, propriétés à 1 000 jeux |
| `write.rs` : écriture, `vu_pour`, validation | Core Domain | Full |
| `merge.rs` : journal, fusion, écartées, conflits, anomalies | Core Domain | Full ; PROP-ST01-2 avec ≥ 3 appareils simulés |
| `clock.rs` : port `Clock`, `HorlogeSimulee` | Supporting | Std (simulateur, sans règle métier) |
| `crates/cli/src/main.rs` | Generic | Gen (provisoire, réécrit en ST-03) |
| `scripts/lib/*.py` : zones, scores, graphe | Supporting (outillage) | Exemples des matrices US-5 à US-7 dans `test_ci.py`. **Mutation non mesurée** : cargo-mutants ne mute que du Rust (limite assumée) |
| `scripts/ci.sh`, `ci.d/*.sh`, workflow | Generic | Constat : PR de démonstration rouge (AC-6.4), PR documentaire verte |

**`zones.toml` recommandé pour ST-01** (le plan l'écrit) :

```toml
Full = ["crates/core/src/hlc.rs", "crates/core/src/write.rs", "crates/core/src/merge.rs"]
Std  = ["crates/core/src/clock.rs"]
Gen  = ["crates/cli/src/**"]
```

`lib.rs` n'est pas listé : il prend `Full` par défaut et ne contient aucun mutant. Pas de `crates/core/src/**` en `Full` : il couvrirait `clock.rs`, et la règle du plus exigeant l'emporterait.

---

## Formats (point à fixer n° 2)

### `.ctdd/stories/ST-xx/zones.toml`

- Les clés de premier niveau sont les noms de zone, parmi `Full`, `Std` et `Gen`. Chaque valeur est une liste de motifs. Une autre clé, un type inattendu ou un TOML illisible ⇒ `rouge`, avec un message qui nomme le fichier (RG-ST01-7.2).
- Un motif est relatif à la racine du dépôt, avec la sémantique de `PurePosixPath.full_match` (Python ≥ 3.13) : `*` reste dans un segment, `**` couvre zéro ou plusieurs segments.
- Sont lues **toutes** les déclarations `.ctdd/stories/*/zones.toml` de la tête (AMB-ST01-4). La zone d'un fichier est la plus exigeante qui le couvre, dans l'ordre `Full > Std > Gen`. Un fichier qu'aucun motif ne couvre prend `Full`.
- Garde-fou de l'arbitrage MANQUE-ST01-1 : un fichier de `crates/core/` ou `crates/plan/` qui résout en `Gen` ⇒ `rouge`.

### `.ctdd/stories/ST-xx/mutants_equivalents.toml`

```toml
[[equivalent]]
mutant = "crates/core/src/hlc.rs:42:9: replace < with <= in Horloge::emettre"
justification = "…pourquoi aucun comportement observable ne change…"
```

- `mutant` est le **nom exact** que cargo-mutants donne au mutant (`fichier:ligne:colonne: description`), copié depuis `mutants.out`. La comparaison se fait à l'identique.
- **Ça échoue fermé.** Si le code bouge et que le mutant change de ligne tout en restant dans le diff, l'entrée ne correspond plus : le mutant est compté, la CI passe au rouge, et on met l'entrée à jour. Un appariement sans ligne pourrait exclure en silence un autre mutant de même description dans la même fonction. Ce choix l'interdit.
- `justification` vide ou absente, ou clé inconnue ⇒ `rouge` (matrice US-7, cas 8). Toutes les listes de toutes les stories sont lues, comme les zones.
- **C'est le seul canal d'exclusion.** `#[mutants::skip]`, `exclude_re`, `exclude_globs` et `--exclude` contournent la justification exigée : ils sont interdits, et `50-st01-mutation.sh` les recherche (dépôt et `.cargo/mutants.toml`).

### Résultats de cargo-mutants (lus dans `mutants.out/outcomes.json`)

| Issue cargo-mutants | Compte comme |
|---|---|
| `caught` | détecté |
| `timeout` | détecté (MANQUE-ST01-8) |
| `missed` | non détecté |
| `unviable` | exclu (non construisible) |
| échec de la base (tests rouges sans mutation) | `rouge` |

Le score est calculé fichier par fichier, sur les mutants du diff, hors équivalents listés. Il est neutre si aucun mutant n'est compté. Un score égal au seuil passe. Le rapport (fichier, zone, score, seuil, verdict) est trié par fichier et écrit dans `target/ci/mutation.md`. Sous Actions, il est aussi ajouté à `$GITHUB_STEP_SUMMARY` et publié en artefact (C-7.4 / AC-7.6).

### `.gitleaksignore` (MANQUE-ST01-4)

Le format est le natif de gitleaks (une empreinte par ligne). Règle : chaque empreinte est précédée d'une ligne `# justification`. Une empreinte sans justification ⇒ `rouge`, vérifié par `40-st01-secrets.sh`.

---

## CI : arborescence et contrat (point à fixer n° 3)

### `scripts/ci.sh` (gelé après ST-01)

- Il charge `scripts/outils.env`, vérifie les versions (voir « Épinglage ») et calcule la base : `$CI_BASE` si elle est fournie, sinon `git merge-base HEAD origin/main`.
- Il exporte aux contrôles `CI_BASE`, `CI_HEAD` et `CI_OUT=target/ci`.
- Il liste puis exécute chaque `scripts/ci.d/*.sh`, trié avec `LC_ALL=C`, par `bash <fichier>`. Il les **exécute tous**, même après un échec, pour un rapport complet.
- Verdict : `rouge` si un contrôle sort avec un code non nul, si une version d'outil diffère ou si `ci.d/` est vide. Sinon `vert`.
- Quand `GITHUB_ACTIONS=true` : `::group::`/`::endgroup::` par contrôle, `::error::` sur échec, une ligne verdict/durée par contrôle dans `$GITHUB_STEP_SUMMARY`.

**Déclinaison de l'arbitrage CONFLIT-ST01-2.** L'arbitrage dit « en plus de ses contrôles ». Ici, les contrôles de ST-01 sont eux-mêmes les premiers fichiers de `ci.d/`, et `ci.sh` ne contient que l'exécuteur. Le comportement observable est le même : un point d'entrée, un contenu qui ne change plus, une liste affichée. Une seule boucle gère les groupes, les erreurs et le récapitulatif. Si Quentin tient à la lettre de l'arbitrage, les cinq contrôles redeviennent des fonctions de `ci.sh`, sans autre changement.

### `scripts/ci.d/` : convention

- Nom : `NN-stxx-<sujet>.sh`. Les nombres 10 à 50 sont pris par ST-01. Une story suivante prend un numéro libre ≥ 60. Le préfixe `stxx` évite les collisions de noms entre voies parallèles, et l'ordre entre deux numéros égaux n'a pas d'importance.
- Un contrôle code son verdict dans son code de sortie, écrit ses rapports sous `$CI_OUT` et n'appelle jamais `ci.sh`.
- Le code d'aide va dans `scripts/lib/`, jamais dans `ci.d/`, qui exécuterait tout ce qu'il contient.
- Les fichiers `*-st01-*` sont gelés. Les modifier est un amendement du socle.

### Les cinq contrôles de ST-01

| Fichier | Ce qu'il exécute | Règles |
|---|---|---|
| `10-st01-tests.sh` | `cargo test --workspace --all-features --locked` ; `python3 -m unittest scripts/lib/test_ci.py` | RG-ST01-6.1 |
| `20-st01-navigateur.sh` | Liste nommée `arbitre-core`, `arbitre-plan`, confrontée aux membres de `cargo metadata`. `arbitre-core` absent ⇒ `rouge` (AMB-ST01-5) ; les autres absents sont ignorés. Puis `cargo check --locked --target wasm32-unknown-unknown -p <présents>` | RG-ST01-6.2 |
| `30-st01-graphe.sh` → `graphe.py` | (a) Arêtes entre composants, par nom, pour **toutes** les crates de l'archi, sur `cargo metadata` (toutes sortes de dépendances, transitif) : `arbitre-core` → aucun composant ; `arbitre-plan`, `arbitre-diff` → `arbitre-core` seulement ; `plan ↛* diff` et `diff ↛* plan`, avec la chaîne dans le message. (b) Interdites dans le graphe normal de la liste navigateur (`cargo tree -p <k> -e normal,features --target wasm32-unknown-unknown`, par crate, pour éviter l'unification de features avec la CLI) : `rusqlite`, `libsqlite3-sys`, `sqlite-wasm-rs`, `chrono` avec la feature `clock`, `tokio`, `reqwest`, `keyring`. (c) Pureté des sources de `crates/{core,plan}/src` : aucune occurrence de `SystemTime`, `Instant::now`, `::now(` hors `clock.rs`. (d) `openssl-sys` et `native-tls` absents de tout le workspace (choix rustls de PL-6). (e) RG-ST01-5.2 : une dépendance déclarée dans `[workspace.dependencies]` est reprise par `workspace = true` dans chaque composant, et une dépendance utilisée par au moins deux composants est déclarée au workspace (lecture des manifestes par `tomllib`) | RG-ST01-5.1, 5.2, 6.3 |
| `40-st01-secrets.sh` | `gitleaks git --redact --log-opts="$CI_BASE..$CI_HEAD"` (chaque commit), puis la règle de justification de `.gitleaksignore` | RG-ST01-6.4 |
| `50-st01-mutation.sh` → `mutation.py` | `git diff $CI_BASE...$CI_HEAD > $CI_OUT/diff.patch` ; diff vide ⇒ neutre. Sinon `cargo mutants --in-diff $CI_OUT/diff.patch --all-features --output $CI_OUT`, puis zones, équivalents, scores et verdict. Plus le contrôle des canaux d'exclusion interdits | RG-ST01-7.1 à 7.3 |

Note sur (c) : la construction wasm ne détecte pas `SystemTime::now()`, qui compile pour `wasm32-unknown-unknown` et panique à l'exécution. Une simple recherche textuelle comble ce trou.

Note sur `--all-features` : la feature `sim` doit être compilée pendant la mutation, sinon les tests qui l'utilisent ne tournent pas.

### `.github/workflows/ci.yml` (gelé après ST-01)

Un seul job `ubuntu-latest`, déclenché sur `pull_request` et sur `push` vers `main`. Étapes :
1. `checkout` avec `fetch-depth: 0` (gitleaks par commit, merge-base) ;
2. toolchain lue dans `rust-toolchain.toml` ;
3. `setup-python` 3.13 ;
4. installation des outils aux versions de `scripts/outils.env` ;
5. cache Cargo ;
6. `CI_BASE=${{ github.event.pull_request.base.sha || github.event.before }} scripts/ci.sh` ;
7. publication de `target/ci/` en artefact, `if: always()`.

Aucune étape par contrôle n'est écrite à la main dans le workflow (arbitrage). Les **Actions tierces sont épinglées par SHA de commit** : le dépôt est public, et un tag se déplace.

---

## Épinglage des versions d'outils (point à fixer n° 4)

| Outil | Où | Valeur |
|---|---|---|
| rustc, cargo, clippy, rustfmt, cible wasm32 | `rust-toolchain.toml` (`channel = "1.99.1"`, `targets = ["wasm32-unknown-unknown"]`) | 1.99.1. Lu par rustup en local comme en CI. `rust-version = "1.99"` dans `workspace.package` |
| Dépendances Rust | `Cargo.lock`, `--locked` partout | — |
| cargo-mutants | `scripts/outils.env` : `CARGO_MUTANTS_VERSION=27.1.0` | Installation : `cargo install --locked cargo-mutants@27.1.0`, ou binaire précompilé en CI à la même version. **Pas encore installé en local.** |
| gitleaks | `scripts/outils.env` : `GITLEAKS_VERSION=8.30.1` | La version locale actuelle. Le pre-commit utilise la même binaire. |
| Python | `scripts/outils.env` : `PYTHON_MIN=3.13` | Un minimum, pas une version exacte : bibliothèque standard seulement (`tomllib`, `json`, `pathlib.full_match`). |
| Actions GitHub | `ci.yml` | SHA de commit |

`ci.sh` compare la sortie `--version` de chaque outil à `outils.env`, **à l'égalité exacte** (un minimum pour Python). Un écart ⇒ `rouge`, avec la commande d'installation dans le message. C'est ce qui donne « même verdict en local et en CI » (MANQUE-ST01-6). Monter une version revient à modifier `outils.env`, ce qui est un amendement du socle.

---

## Dépendances

### Internes

| Dépendance | Type | Couplage |
|---|---|---|
| `arbitre` (cli) → `arbitre-core` | Cargo, normale | Faible. Rien n'est consommé en ST-01. |
| Tests du cœur → feature `sim` | dev, même crate | Aucun |
| `scripts/lib` → `cargo metadata`, `cargo tree`, `mutants.out` | Formats JSON et texte de Cargo et de cargo-mutants | Moyen : un changement de format d'outil casse le contrôle. Les versions épinglées le bornent. |

### Dépendances communes retenues (point à fixer n° 5)

Elles sont déclarées dans `[workspace.dependencies]` (AC-8.3), même inutilisées. Le workspace fixe la version et coupe les features par défaut. Chaque composant ajoute ses features : `x = { workspace = true, features = [...] }`, puisque les features sont additives. Le document `docs/decisions/ST-01-dependances.md` porte, pour chaque sujet, le choix, la raison et l'option écartée.

| Sujet (AC-8.1) | Choix | Version | Déclaration au workspace | Option écartée |
|---|---|---|---|---|
| Exécution asynchrone | `tokio` | 1.54.1 | `default-features = false` | async-std (arrêté), smol (rmcp exige tokio) |
| Appels réseau sortants | `reqwest` + rustls | 0.13.5 | `default-features = false, features = ["rustls"]` | ureq (bloquant), hyper seul (trop bas niveau), native-tls/OpenSSL (exclu par PL-6, contrôlé) |
| Trousseau | `keyring`, repli fichier `0600` | 4.2.0 | `default-features = false` (le backend par plateforme est choisi par la crate qui l'utilise, après S6) | secret-service direct (Linux seul) |
| Sérialisation | `serde` (`derive`) + `serde_json` | 1.0.229 / 1.0.154 | `serde = { …, default-features = false, features = ["derive"] }` | — |
| Tests par propriétés | `proptest` | 1.12.0 | normale (n'apparaît qu'en `[dev-dependencies]`) | quickcheck (shrinking moins fin, pas de persistance des échecs) |
| Mesure de mutation | `cargo-mutants` | 27.1.0 | **hors `Cargo.toml`** (outil, `outils.env`) | mutagen (abandonné) |
| *Dates et fuseaux* (ajout) | `jiff` | 0.2.39 | `default-features = false` | chrono + chrono-tz (deux crates, base des fuseaux ×2 en brut d'après S1-A) |
| *Aléa* (ajout) | `getrandom` | 0.4.3 | `default-features = false` (backend wasm activé par la crate web) | rand (inutile pour des octets bruts) |

- **Deux ajouts à PL-6, justifiés dans le document** (AC-8.2). Les dates et fuseaux serviront à `Clock::fuseau` (cœur), au planning et à l'agenda. L'aléa servira à l'identifiant d'appareil (ST-02) et aux nonces de l'enveloppe (ST-18). Les deux concernent au moins deux composants. Ne pas les déclarer maintenant forcerait un amendement du socle plus tard. **Choix à confirmer par Quentin** (P-1).
- **Retirés** : `chrono`, que plus personne n'utilise, et `rusqlite`, qui revient en ST-02 dans `arbitre-sqlite` seul. `rusqlite` n'est pas commune et reste locale à son composant.
- **Non communes, donc locales** : `clap` (cli seule) et les crates wasm-bindgen de S1-A (`arbitre-web` seule, épinglées par S1-B).
- Les versions sont celles de crates.io au 2026-11-03. Le Skeleton les fixe par `cargo add` et le `Cargo.lock` fait foi.
- Le cœur de ST-01 n'utilise **aucune** dépendance commune en normale. `serde` y entrera avec le premier format sérialisé (ST-02 ou ST-04b).
- `reqwest` 0.13 avec `rustls` tire `aws-lc-rs`, qui compile du C (cmake). C'est sans effet sur wasm (`reqwest` est interdit dans la liste navigateur) et acceptable en natif. À noter dans le document.

### Externes

| Dépendance | Type | Risque |
|---|---|---|
| GitHub (dépôt public, Actions, protection contre l'envoi de secrets) | Hébergeur | Faible. G-1 doit être actif avant le premier envoi. |
| cargo-mutants (format `outcomes.json`, nommage des mutants) | Outil | Moyen. Épinglé. Un changement de version peut invalider les noms de `mutants_equivalents.toml`, ce qui fait passer la CI au rouge (échec fermé). |
| gitleaks | Outil | Faible |

---

## Contraintes et limites

- **Frontière wasm** : `arbitre-core` n'a aucune dépendance normale en ST-01. Aucune entrée-sortie, aucun thread, aucune lecture d'heure.
- **Déterminisme** : aucun `HashMap`/`HashSet` dans le cœur, même interne, parce que l'ordre d'itération fuit vite dans une sortie. Les ordres de tri sont les `Ord` dérivés : l'**ordre de déclaration des champs** de `Marque`, `IdDonnee`, `Cible` et `Conflit` est contractuel.
- **Propriétés** : 1 000 jeux par propriété, fixés dans le code (`ProptestConfig`) et non par variable d'environnement. Les graines d'échec sont versionnées dans `proptest-regressions/`. Les générateurs suivent E-1.1 : lots vides, doublons, ≥ 3 appareils, même `(l, c)`, suppressions, concurrentes de même valeur, écrasements séquentiels. Pour produire des `vu` cohérents, le plus simple est de passer par les appareils simulés de PROP-ST01-2 (`tests/support/`) plutôt que de générer des écritures isolées.
- **Performance** : aucune exigence chiffrée. Le cas 8 (10 000 écritures sur un champ) borne le calcul des conflits : il faut l'évaluer par écriture nouvelle contre les écritures de sa portée, pas sur toutes les paires de `S` à chaque fusion. La projection peut se recalculer en entier en ST-01 (quelques centaines de données), avec une note `ponytail:` qui indique la piste « recalcul par donnée touchée » pour ST-02.
- **Temps de mutation** : chaque mutant relance les tests du paquet, propriétés à 1 000 jeux comprises. Le délai par mutant est celui que cargo-mutants déduit de la base. Si la PR de ST-01 dépasse un temps de CI acceptable, on réduit les jeux **dans un profil de mutation**, pas en zone Full courante. Le Planner mesure d'abord.
- **Compatibilité** : rien n'est publié et rien n'est persisté. Aucune migration. Le tuple d'écriture devient contractuel dès ST-02 (stockage) : `vu` doit y être dès le premier schéma.
- **Bounded context** : la story ne touche à aucun autre contexte. Le seul point de contact futur est le port `Store` (ST-02), dont le contrat est décrit plus haut, sans test de contrat à poser en ST-01.

---

## Écarts aux specs (à valider)

| # | Spec | Écart | Raison |
|---|---|---|---|
| E-1 | RG-ST01-4.5, matrice US-4, cas 12 et 13 | « Auteur hors liste » et « appareil ≠ appareil de la marque » ne sont plus représentables dans le cœur. Ils deviennent des garanties de type, et non des tests d'exécution. Le refus du lot entier s'appliquera à la désérialisation (ST-02, ST-04b), avec `ErreurLot`. | Un état impossible à construire ne peut pas être mal validé. |
| E-2 | RG-ST01-4.5 | Une règle de validation est ajoutée : `vu < marque`, sinon lot refusé. | Elle découle de RG-ST01-4.2 et protège `a_vu`. Une écriture qui prétendrait avoir vu sa propre marque fausserait la concurrence. |
| E-3 | Arbitrage CONFLIT-ST01-2 | Les contrôles de ST-01 sont des fichiers de `ci.d/`, pas du code de `ci.sh`. | Voir « CI ». Le comportement est inchangé. |

---

## Anti-patterns détectés (dette existante impactante)

| Anti-pattern | Localisation | Impact | Recommandation |
|---|---|---|---|
| Stockage natif dans le cœur | `crates/core/Cargo.toml` : `rusqlite` | Le cœur ne compile pas en wasm32 (S1-A, 1c) | Retirer en ST-01 (prévu). |
| Horloge système implicite | `chrono/clock`, dans `core` et `cli` | Pas de simulateur possible, pas de déterminisme | Retirer des deux : la CLI n'en a pas l'usage avant ST-03, qui implémentera `Clock` avec `std::time::SystemTime`. |
| Versions dupliquées | `chrono` dans deux manifestes | Dérive | Disparaît avec le retrait. La règle (e) de `graphe.py` empêche son retour. |
| Code de démonstration | `lib.rs`, `main.rs` | Aucun | Remplacé (C-5.3). |

---

## Points ouverts pour Quentin

| # | Question | Recommandation (retenue par défaut) |
|---|---|---|
| P-1 | Ajouter `jiff` et `getrandom` aux dépendances communes de PL-6 ? | **Oui.** Chacun servira à au moins deux composants. Le faire maintenant évite deux amendements du socle. `jiff` plutôt que chrono : une seule crate pour les fuseaux, fonctionne en wasm sans `now()` (S1-A). |
| P-2 | `Clock::fuseau()` : qui l'ajoute ? Le port vit dans `arbitre-core`, qui appartient à la voie M après ST-04, alors que c'est le planning (voie P) qui en aura besoin. | L'ajouter en **ST-03 ou ST-04** (socle), avec la base de fuseaux embarquée. Sinon, ce sera un amendement du socle dans la voie P. |
| P-3 | E-1 à E-3 ci-dessus | Les accepter. |

## Arbitrages de Quentin (2026-11-03)

- **P-1 → oui.** `jiff` et `getrandom` rejoignent les dépendances communes. `reqwest` garde rustls avec `aws-lc-rs` : choix réversible (aucune donnée ni format ne dépend du fournisseur TLS ; bascule vers `ring` = une feature au workspace plus un `install_default()` au démarrage des binaires natifs). `docs/decisions/ST-01-dependances.md` note la réversibilité et le piège : deux fournisseurs rustls compilés ⇒ `install_default()` explicite obligatoire.
- **P-2 → ST-04.** `Clock::fuseau()` (avec `jiff` et la base des fuseaux embarquée) est ajouté en ST-04, qui construit le contexte de module portant `clock`, avant la séparation des voies P et M. Noté dans `.ctdd/cadre/04_plan.md` § ST-04.
- **P-3 → E-1, E-2, E-3 acceptés.** `graphe.py` reste en Python (stdlib seule) ; `outils.env` déclare Python ≥ 3.11 et `ci.sh` le vérifie avec un message explicite (le `python3` d'Xcode est en 3.9, sans `tomllib`).
- **Amendé par le plan (Q-2) :** minimum Python porté à 3.13 pour `full_match`.
