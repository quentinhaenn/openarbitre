# Revue Devil's Advocate — ST-01 : Fusion pure et fondations du dépôt (itération 1)

## Verdict global
**BLOQUÉ.**
Le plan est complet sur les matrices et bien découpé. Mais la portée de `vu` pour une suppression, fixée par l'archi (`03_archi.md`, « Portée de `vu` ») et implémentée telle quelle par T-6/T-7, laisse passer une suppression concurrente d'une modification sans la signaler, avec deux appareils seulement. Aucun test du plan ne peut le voir. La faille est en amont du plan, dans la règle arbitrée : elle remonte au Spécificateur ou à l'Architecte, puis à Quentin, et non au Planner seul.

## Failles identifiées

### [BLOQUANT] F-1 : une suppression « voit » les modifications concurrentes des autres champs
- **Axe** : Edge cases (et Cohérence specs ↔ archi)
- **Source** : 02_specs « Arbitrages » CONFLIT-ST01-1 (a) ; RG-ST01-2.1 ; [HYP] du cahier « suppression et modification concurrentes = conflit » (validée en bloc) ; matrice US-3, cas 2 ; 03_archi « Portée de `vu` »
- **Tâche(s) concernée(s)** : T-6 (`vu_pour`), T-7 (règle de conflit), T-8
- **Description** : l'archi donne à une suppression `vu = max` des marques connues sur **toutes** les écritures de la donnée, tous champs confondus. Une seule marque scalaire ne dit pas *quelles* écritures ont été vues. Une modification locale d'un autre champ, plus récente, fait donc passer pour « vue » une modification distante plus ancienne que l'auteur n'a jamais reçue.
- **Preuve (contre-exemple, deux appareils)** :
  1. Le téléphone B, hors ligne, écrit `titre = "x"` à (7,0,B), avec `vu = ⊥`.
  2. L'ordinateur A, qui n'a rien reçu de B, écrit `échéance = "y"` à (9,0,A), puis supprime la donnée à (10,0,A). Le `vu` de la suppression vaut (9,0,A), sa propre écriture.
  3. Après synchronisation, `a_vu(suppr, titre) ⇔ (9,0,A) ≥ (7,0,B)` est **vrai**. La paire n'est donc pas concurrente et aucun conflit n'est signalé. La paire `(échéance, titre)` n'est pas exclusive (champs différents), donc pas de conflit non plus.
  4. Résultat : donnée supprimée, modification du titre masquée, **aucun conflit nulle part**. C'est le cas 2 de la matrice US-3 (« conflit attendu »), avec un champ de plus.
  - Ce n'est pas la limite acceptée par l'arbitrage (« après avoir vu seulement la gagnante d'une paire concurrente ») : `échéance` et `titre` ne forment pas une paire, et rien n'est signalé.
  - La portée littérale de l'arbitrage (« même cible et suppressions de la donnée », soit pour une suppression : les seules suppressions) échoue dans l'autre sens. Au cas 3 de la matrice US-3 (A écrit f, B la reçoit puis supprime), le `vu` de la suppression vaut `⊥` et un **faux** conflit sort. Aucune portée d'une marque unique par suppression ne tient à la fois US-3 cas 3 et la [HYP]. L'archi a dévié de l'arbitrage sans le lister dans « Écarts aux specs » (E-1 à E-3).
  - Conséquence : cadre AC-2.6 et RG-G7 ne tiennent plus pour les cas 5 et 6 de la matrice US-2 du cadre, dès qu'un second champ a été modifié.

### [MAJEUR] F-2 : aucune propriété ne vérifie « concurrentes ⇒ conflit »
- **Axe** : Complétude
- **Source** : INV-ST01-2 (`conflit ⇔ concurrentes ∧ exclusives ∧ valeurs différentes`) ; PROP-ST01-2
- **Tâche(s) concernée(s)** : T-7, T-8
- **Description** : T-8 vérifie un seul sens (« un écrasement fait après réception n'est jamais un conflit »). Le sens inverse, une paire réellement concurrente toujours signalée, n'est couvert que par les exemples à deux appareils et à un seul champ des matrices US-2 et US-3.
- **Preuve** : la faille F-1 passe tous les critères de T-6, T-7 et T-8. Pourtant, les appareils simulés de T-8 savent exactement ce que chacun avait reçu au moment d'écrire : la vérité causale est disponible et le plan ne s'en sert pas.

### [MAJEUR] F-3 : tests par propriétés non déterministes, alors que le verdict de mutation est bloquant
- **Axe** : Cohérence / Cas d'erreur
- **Source** : RG-ST01-6.1 (`verdict_local = verdict_CI`), matrice US-6, cas 12 ; arbitrage « Score de mutation sous le seuil : bloquant » ; E-1.1 « rejouable à partir de sa graine »
- **Tâche(s) concernée(s)** : T-8, T-10, T-13
- **Description** : proptest tire une graine aléatoire à chaque exécution. Rien dans le plan ne fixe ni ne transmet la graine.
- **Preuve** :
  - Un mutant tué par 1 000 jeux dans une exécution peut survivre dans la suivante. Un score proche de 85 % (risque R-1 du plan) bascule alors d'une exécution à l'autre, sur la même tête. Le délai d'attente compté comme « détecté » (MANQUE-ST01-8) dépend en plus de la vitesse de la machine (macOS en local, `ubuntu-latest` en CI).
  - Un contre-exemple rare peut aussi rendre `10-st01-tests` rouge sur une PR purement documentaire, et `main` est « protégée sans exception » (arbitrage Q-1).
  - Le cas 12 de la matrice US-6 n'apparaît dans aucun critère de complétion. Le tableau des invariants l'attribue à T-10 et T-11, dont les critères ne le citent pas.

### [MAJEUR] F-4 : l'anomalie (MANQUE-ST01-2) casse l'idempotence et la partition
- **Axe** : Edge cases
- **Source** : AC-1.5 ; RG-2.2 (seconde réception : listes vides) ; RG-2.4 (`retenues ⊎ écartées = S`) ; 03_archi, type `Anomalie` ; T-6 (« garde le plus grand contenu »)
- **Tâche(s) concernée(s)** : T-6, T-7, T-8
- **Description** : le `Journal` ne garde qu'un `Contenu` par identité. Le contenu perdant d'une anomalie sort donc du journal.
- **Preuve** :
  - (1) Il n'est ni retenu ni écarté, ce qui contredit la partition de RG-2.4. Seul le `Rapport` transitoire le rend.
  - (2) Recevoir de nouveau l'écriture perdante produit une nouvelle `Anomalie`, puisque le journal ne sait pas qu'il l'a déjà vue. La seconde réception ne rend donc pas un rapport vide.
  - (3) Remplacer un contenu change son `vu`, donc les conflits des paires où il figure. `conflits(S)` cesse d'être monotone, et le calcul incrémental « nouvelle × portée » de T-7 ne réévalue pas les paires du contenu remplacé.
  - Les générateurs de T-8 produisent des doublons (même identité, même contenu), pas des identités en collision avec des contenus différents. Les propriétés INV-5 ne couvrent donc pas ce chemin.

### [MAJEUR] F-5 : les `Ord` dérivés ne donnent pas les ordres de tri des specs
- **Axe** : Cohérence
- **Source** : RG-ST01-2.2 (écartées par `(donnée, cible, marque)`, conflits par `(donnée, champ, marque retenue, marque écartée)`) ; 03_archi (« les tris découlent des `Ord` dérivés », « l'ordre de déclaration des champs […] est contractuel ») ; `Ecriture = (donnée, quoi, marque, vu, auteur)` avec `quoi = Champ { nom, valeur }`
- **Tâche(s) concernée(s)** : T-5, T-7
- **Description** :
  - Un `Ord` dérivé sur `Ecriture` compare `quoi`, donc `valeur`, **avant** `marque`. Trier des `Ecriture` revient à trier par valeur, pas par marque.
  - Un `Ord` dérivé sur `Conflit { donnée, champ, retenue: Ecriture, ecartee: Ecriture }` trie par `retenue.donnée`, puis `retenue.quoi` (valeur comprise), puis seulement `retenue.marque`.
  - Seul le parcours du `BTreeMap<(Cible, Marque), _>` donne l'ordre des écartées.
- **Preuve** : la consigne de l'archi (« ordre de déclaration contractuel, tris dérivés ») et celle de T-7 (« trié par (donnée, champ, marque retenue, marque écartée) ») mènent à deux ordres différents dès que deux conflits du même champ ont des valeurs retenues dans un autre ordre que leurs marques. Le Skeleton devra trancher entre deux sources.

### [MAJEUR] F-6 : les dépendances communes déclarées ne sont ni résolues ni vérifiées, et T-1 retire les features par défaut de `proptest`
- **Axe** : Cohérence / Cas d'erreur
- **Source** : AC-8.3 ; 03_archi, tableau des dépendances communes (`proptest` : « normale ») et « le `Cargo.lock` fait foi » ; T-1 (« `default-features = false` sauf mention », sans mention pour `proptest` ni `serde_json`)
- **Tâche(s) concernée(s)** : T-1, T-2, T-8
- **Description** :
  - (1) Cargo ne résout pas une entrée de `[workspace.dependencies]` qu'aucun membre n'hérite. Six des huit (`tokio`, `reqwest`, `keyring`, `serde`, `serde_json`, `jiff`, `getrandom`, moins celles utilisées) n'entrent pas dans `Cargo.lock`. Une version inexistante ou un nom de feature faux (par exemple celui de rustls dans `reqwest` 0.13) passe donc le critère `cargo build --workspace --locked` de T-1. « Le `Cargo.lock` fait foi » est faux pour elles.
  - (2) La lettre de T-1 contredit l'archi pour `proptest` (et pour `serde_json`, qui exige `std` ou `alloc`).
- **Preuve** : `proptest` sans ses features par défaut perd `std`. La persistance des échecs (`FileFailurePersistence`) en dépend, et E-1.1 (« rejouable », `proptest-regressions/` versionné) tombe en silence, s'il compile encore. `serde_json` sans `std` ni `alloc` ne compile pas, mais l'erreur n'apparaîtra qu'à la première story qui l'hérite. La décision « prise une fois » d'US-8 n'est donc jamais vérifiée en ST-01.

### [MAJEUR] F-7 : la garde de pureté textuelle exempte `clock.rs` et ne voit pas toutes les lectures d'heure
- **Axe** : Sécurité & intégrité (contournement d'invariant)
- **Source** : AC-5.1 ; RG-ST01-4.4 (« lecture de l'heure système par le cœur → interdite ; détectée par la vérification ») ; 03_archi (c) ; plan, « Over-engineering » (cas 9 de la matrice US-1 écarté **grâce à** (c))
- **Tâche(s) concernée(s)** : T-9a, T-8
- **Description** : (c) recherche `SystemTime`, `Instant::now` et `::now(` hors de `clock.rs`.
- **Preuve** :
  - `clock.rs` est le fichier où un adaptateur système posé « à côté du trait » atterrirait le plus naturellement. `SystemTime::now()` compile en `wasm32-unknown-unknown` et panique à l'exécution (note de l'archi elle-même). Or `HorlogeSimulee::now` est une définition (`fn now(`), pas un appel `::now(` : l'exemption ne protège aucun usage légitime.
  - Hors de `clock.rs`, `std::time::UNIX_EPOCH.elapsed()` lit l'heure système sans contenir aucun des trois motifs.
  - Le plan s'appuie sur cette garde pour ne pas tester le cas 9 de la matrice US-1 : c'est la seule défense de RG-ST01-1.1 sur ce point.

### [MAJEUR] F-8 : la recherche des canaux d'exclusion de mutation n'a pas de périmètre et se trouve elle-même
- **Axe** : Régressions
- **Source** : 03_archi, `mutants_equivalents.toml` (« `50-st01-mutation.sh` les recherche (dépôt et `.cargo/mutants.toml`) ») ; T-10 ; T-2 (CONTRIBUTING documente l'interdiction)
- **Tâche(s) concernée(s)** : T-2, T-10
- **Description** : « le dépôt » contient `mutants::skip`, `exclude_re` et `--exclude` dans `CONTRIBUTING.md` (T-2), dans `.ctdd/stories/ST-01/03_archi.md` et `04_plan.md`, et dans `50-st01-mutation.sh` lui-même.
- **Preuve** : lue à la lettre, la règle rend `rouge` en permanence dès la PR de ST-01. Restreinte au hasard par l'implémenteur, elle peut à l'inverse manquer un vrai canal. Le plan ne fixe ni les chemins fouillés ni la forme des motifs.

### [MAJEUR] F-9 : la PR jugée peut modifier son juge
- **Axe** : Sécurité & intégrité
- **Source** : AC-6.6 (« sans modifier les règles posées ici ») ; US-6, objectif « ne plus relire à la main ce qu'une machine peut garantir » ; 03_archi, « fichiers `*-st01-*` gelés » ; arbitrage Q-1 (check requis)
- **Tâche(s) concernée(s)** : T-10, T-11
- **Description** : sur `pull_request`, GitHub exécute le `ci.yml`, le `ci.sh`, les `ci.d/*`, les `outils.env` et les `zones.toml` **de la PR**. « Gelé » n'est qu'une convention écrite. Rien ne rend rouge une PR qui touche ces fichiers.
- **Preuve** : un agent bloqué par le seuil de 85 % peut, dans la même PR :
  - retirer `merge.rs` de `Full` et l'ajouter en `Std` : la règle du plus exigeant donne alors `Std`, et le garde-fou ne vise que `Gen` ;
  - ou abaisser le seuil dans `mutation.py` ;
  - ou supprimer `50-st01-mutation.sh`.

  Le check requis reste vert. Seule la relecture humaine l'arrête, alors qu'US-6 a justement pour but de s'en passer, et le cadre prévoit des agents qui travaillent en parallèle.

### [MAJEUR] F-10 : les marques reçues ne sont pas bornées
- **Axe** : Cas d'erreur / Sécurité
- **Source** : RG-ST01-4.3 (marque en avance acceptée, arbitrage « Horloge en avance ») ; MANQUE-ST01-3 ; 03_archi (`l` < 2^53, contrainte de S1-A) ; RG-G3 (même moteur pour l'import, ST-04b)
- **Tâche(s) concernée(s)** : T-3, T-5
- **Description** : `recevoir` adopte `(lm, cm)` sans borne, et `valider` ne contrôle ni `l` ni `c`.
- **Preuve** :
  - (1) Une marque reçue `(l_futur, u32::MAX, X)` met le compteur local à `u32::MAX`. Tant que l'heure physique n'a pas dépassé `l_futur`, ce qui peut prendre des jours puisque l'avance est acceptée, chaque `emettre` rend `CompteurEpuise` : l'appareil ne peut plus écrire. Le test de saturation de T-3 ne couvre que les émissions propres.
  - (2) Un `l ≥ 2^53` est accepté puis réémis par tous les appareils (RG-ST01-4.3). Il casse la contrainte que l'archi pose elle-même pour wasm, et `l ≥ 2^63` ne tient pas dans l'`INTEGER` signé de SQLite (ST-02).
  - L'entrée peut venir d'un fichier d'import corrompu (ST-04b), pas seulement d'un pair.

### [MINEUR] F-11 : `vu` est déclaré librement par l'émetteur
- **Axe** : Sécurité & intégrité
- **Source** : 03_archi, Synthèse 3 (« les états impossibles ne sont pas représentables ») ; `vu_pour` (« pour que les surfaces ne réimplémentent pas cette règle ») ; E-2
- **Tâche(s) concernée(s)** : T-5, T-6
- **Description** : `Ecriture` se construit avec n'importe quel `vu < marque`. Rien ne lie `vu` à `vu_pour`.
- **Preuve** : une surface de ST-03 ou plus tard qui prend l'état de l'`Horloge` au lieu de `vu_pour` produit des `vu` maximaux. Toutes ses écritures passent alors pour séquentielles, et plus aucun conflit n'est signalé, sans qu'aucun contrôle ne le voie.

### [MINEUR] F-12 : égalité `t_s = t_f` représentable
- **Axe** : Edge cases
- **Source** : RG-2.3 (« l'égalité `t_s = t_f` est impossible entre deux écritures distinctes ») ; glossaire, identité `(cible, marque)`
- **Tâche(s) concernée(s)** : T-5, T-6
- **Description** : l'identité inclut la cible. Un même appareil peut donc poser la même marque sur une suppression et sur un champ de la même donnée, et rien ne l'interdit.
- **Preuve** : dans ce cas, `supprimée` est faux (égalité). Comme `vu < marque` des deux côtés, les deux écritures du même auteur, au même instant, sont « concurrentes » : un conflit sort entre elles. Le comportement n'est ni spécifié ni testé.

### [MINEUR] F-13 : texte du plan contraire à l'arbitrage Q-2
- **Axe** : Cohérence
- **Source** : 04_plan, « Arbitrages » Q-2 (Python ≥ 3.13, `full_match`, pas de traducteur)
- **Tâche(s) concernée(s)** : T-9a, T-9b, T-10
- **Description** : le plan dit encore « Python ≥ 3.11 » à cinq endroits : Traçabilité, Stack, Under-engineering, T-9a, et T-10 (`PYTHON_MIN=3.11`). T-9b décrit toujours la « traduction motif → regex ».
- **Preuve** : un agent qui exécute T-10 tel qu'il est rédigé écrit `PYTHON_MIN=3.11`. Le contrôle de version passe alors en 3.11 et 3.12, puis `full_match` lève `AttributeError`.

### [MINEUR] F-14 : règles d'arêtes non définies pour six crates
- **Axe** : Complétude
- **Source** : AC-6.3 (« règles écrites par nom pour **tous** les composants prévus ») ; T-9a (« les **neuf** crates ») ; 03_archi (a) (règles pour `core`, `plan` et `diff` seulement) ; cadre 03_archi, graphe de dépendances
- **Tâche(s) concernée(s)** : T-9a
- **Description** : aucune source ne donne les règles de `arbitre-sqlite`, `-relay`, `-ai`, `-calendar`, `-web` et `arbitre`, alors que le graphe du cadre en implique (par exemple : `relay` ne dépend que de `core`, `calendar` de `plan`).
- **Preuve** : l'implémenteur les inventera ou les omettra. Comme `ci.d/*-st01-*` est gelé, les ajouter plus tard sera un amendement du socle.

### [MINEUR] F-15 : T-4 dépend de T-3 sans le déclarer
- **Axe** : Cohérence
- **Source** : T-4, critère (« plus deux `Horloge` (T-3) »)
- **Tâche(s) concernée(s)** : T-4, ordre d'exécution
- **Description** : T-4 est annoncée « parallélisable avec T-3 », mais son critère (matrice US-4, cas 1) exige l'`Horloge` de T-3.
- **Preuve** : lancée en parallèle, T-4 ne peut pas atteindre son critère.

### [MINEUR] F-16 : ordre `fusionner` / `recevoir` non spécifié
- **Axe** : Cas d'erreur
- **Source** : RG-ST01-4.3 ; 03_archi (« toute écriture entrée dans le journal est passée par `recevoir` », contrat de la coquille)
- **Tâche(s) concernée(s)** : T-3, T-6, T-8
- **Description** : rien ne dit si l'horloge adopte la marque d'un lot que `fusionner` refuse (`ErreurLot`), ni qui calcule le maximum du lot pour `recevoir(&Marque)`.
- **Preuve** : T-8 doit écrire ce protocole pour ses appareils simulés. Sans règle, chaque surface le réécrira, ce qu'`vu_pour` voulait justement éviter.

### [MINEUR] F-17 : profil de mutation par variable d'environnement
- **Axe** : Cohérence
- **Source** : 03_archi, « Propriétés » (« 1 000 jeux fixés dans le code, et non par variable d'environnement ») ; 04_plan R-2 (« variable lue par les tests »)
- **Tâche(s) concernée(s)** : T-13
- **Description** : la mitigation de R-2 contredit l'archi.
- **Preuve** : `ProptestConfig { cases: 1000 }` explicite ignore `PROPTEST_CASES`. Le jour où il faut réduire les jeux, la mitigation ne s'applique pas telle qu'elle est écrite. Moins de jeux fait aussi baisser le score, en contradiction avec K-7.5.

### [MINEUR] F-18 : envoi initial et protection de `main` absents du plan
- **Axe** : Complétude
- **Source** : 04_plan, arbitrage Q-1 (« les commits locaux antérieurs à la protection sont envoyés avant qu'elle soit activée ») ; T-13 (« seule tâche qui envoie ») ; état relevé : `main` locale **en avance de 3 commits** sur `origin/main`
- **Tâche(s) concernée(s)** : T-0, T-11, T-13
- **Description** : aucune tâche n'envoie ces trois commits après G-1, n'active la protection, ni ne nomme le check requis.
- **Preuve** : T-13 est présentée comme le seul envoi, ce que contredit l'arbitrage.

### [MINEUR] F-19 : matrice US-6, cas 2, sans tâche
- **Axe** : Complétude
- **Source** : 02_specs, matrice US-6, cas 2 (« test en échec → rouge »)
- **Tâche(s) concernée(s)** : T-10, T-12
- **Description** : aucun critère ni aucune démonstration ne cite ce cas. Le cas 12 est traité en F-3.
- **Preuve** : les critères de T-10 (cas 4, 9, 10, 11) et de T-12 (cas 3, 5, 6, 8) ne le contiennent pas.

### [MINEUR] F-20 : coût quadratique sur un gros lot unique
- **Axe** : Over/Under-engineering
- **Source** : 04_plan, Under-engineering (« nouvelle × portée ») ; matrice US-1, cas 8 ; RG-G3 (import par le même moteur)
- **Tâche(s) concernée(s)** : T-7
- **Description** : quand tout l'historique arrive dans un seul lot (première synchronisation, import ST-04b), « nouvelle × portée » vaut n². Avec les 10 000 écritures du cas 8 sur un champ, cela fait 10^8 paires, en debug, à chaque mutant.
- **Preuve** : le raccourci ne gagne que sur des lots petits devant le journal.

## Axes sans faille détectée
- **Régressions** : RAS. Rien n'est publié ni persisté. Le retrait de `chrono` dans la CLI et le remplacement du code de démonstration n'ont aucun consommateur.
- **Complétude des matrices** : hors F-2 et F-19, chaque cas des matrices US-1 à US-5, US-7 et US-8 a une tâche qui le nomme. Le tableau des invariants est complet.
- **Over-engineering** : RAS hors F-20. Les trois simplifications de l'analyse pré-plan (pas d'objet réseau, pas de test multiprocessus, pas de profil anticipé) sont justifiées. La seconde dépend toutefois de F-7.

## Recommandation
- **F-1 → Spécificateur, Architecte, puis Quentin.** La règle arbitrée (portée d'une marque unique `vu`) est réfutée par le contre-exemple. Le Planner ne peut pas la corriger seul.
- **F-2 à F-10 → Planner**, avant le Skeleton. F-5 et F-6 touchent aussi le texte de l'archi.
- **MINEURS** : à traiter au fil de l'eau.
- Itération DA suivante après correction.
