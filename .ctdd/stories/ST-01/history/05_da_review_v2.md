# Revue Devil's Advocate — ST-01 : Fusion pure et fondations du dépôt (itération 2)

## Verdict global
**PASSE AVEC RÉSERVES.**
Le moteur tient : `vu` par visée réfute le contre-exemple F-1, l'oracle causal ferme F-2, et A-3, A-4 et A-10 ferment F-4, F-5 et F-20 sans laisser de trou que j'aie pu construire. Les réserves portent sur l'outillage. Le contrôle « juge touché » s'exécute depuis la tête de la PR qu'il juge, donc F-9 n'est levée qu'en partie. Un motif de zone ajouté peut abaisser le `Full` par défaut. Enfin, `zones.toml` laisse la CLI en `Full`, ce qui rend inatteignables les critères de T-10 et de T-14.

## Vérification des failles de la v1

| Faille v1 | Statut | Constat |
|---|---|---|
| F-1 | **Levée** | Contre-exemple rejoué avec A-1 : la suppression (10,0,A) a `vu = {Champ(échéance): (9,0,A)}`, donc `vu[Champ(titre)] = ⊥` et la paire est concurrente, d'où un conflit. Le cas 3 de la matrice US-3 passe aussi, avec `vu = {f: (10,0,A)}`. Le test est nommé en T-6. |
| F-2 | **Levée** | `a_vu*` suit la règle arbitrée appliquée au registre réel. L'égalité est vérifiée après chaque action, et la vérification ponctuelle (i) prouve que la propriété voit la règle v1. Le sens « conflit ⇒ ni l'un ni l'autre ne connaissait » est bien impliqué. |
| F-3 | **Levée** (hors R-10, accepté en Q-10) | Graine fixée dans `config()`. Cas 12 de la matrice US-6 en T-14. |
| F-4 | **Levée** | Un `BTreeSet<Contenu>` par identité : rien n'est remplacé, la partition tient, la seconde réception est vide, et `conflits(S)` redevient monotone. |
| F-5 | **Levée** | A-4 sépare les clés contractuelles des `Ord` de commodité. Le tri des conflits est explicite. |
| F-6 | **Levée** | Les features sont écrites par dépendance, `proptest` et `serde_json` gardent `std`, et une crate jetable vérifie une fois. Ce qui reste (le `Cargo.lock` ne fige pas les six crates non héritées) est assumé par A-5. |
| F-7 | **Levée sur l'heure** | Plus d'exemption, `UNIX_EPOCH` et `env::` ajoutés. Effets de bord en F-7 ci-dessous. |
| F-8 | **Levée** | Périmètre fixé par `git ls-files`. La documentation n'est plus fouillée. |
| F-9 | **Partiellement levée** | Le juge attrape une modification de `mutation.py`, `graphe.py`, `outils.env` et des contrôles 10 à 50. Il ne peut rien contre une modification de son propre exécuteur : voir F-1 ci-dessous. |
| F-10 | **Levée** (2) ; (1) acceptée en Q-4 | `L_MAX` est contrôlé à la validation et à l'émission. |
| F-11 | **Levée** ; reste accepté en Q-6 | `vu_pour` est privée, et `ecrire` est le seul chemin local. |
| F-12 | **Acceptée** (Q-5) | Le comportement est fixé et testé. Un détail du conflit reste non spécifié : F-4 ci-dessous. |
| F-13 | **Levée** | 3.13 et `full_match` partout dans le plan. Le corps de l'archi dit encore 3.11 (P-3), mais il est amendé. |
| F-14 | **Levée** | La table A-9 suit le graphe du cadre (`.ctdd/cadre/03_archi.md` l. 52-66), Q-7 compris. Échec fermé pour une crate inconnue. |
| F-15 | **Levée** | `clock.rs` est passé en T-3. |
| F-16 | **Levée** | Le journal porte l'horloge, avec un tout ou rien qui inclut l'horloge (testé en T-5). |
| F-17 | **Levée dans le plan** | Le corps de l'archi n'est pas amendé : F-6 ci-dessous. |
| F-18 | **Levée** | T-0 envoie et protège, T-14 rend le check requis. La commande de T-14 est fausse : F-5 ci-dessous. |
| F-19 | **Levée** | Démonstration 0 de T-12. |
| F-20 | **Levée pour un gros lot unique** | La complétude de l'intervalle d'A-10 est vérifiée : `x` de rang inférieur et `¬a_vu(w, x)` placent `marque(x)` dans `]vu(w)[v], marque(w)]`. La borne haute inclusive couvre les collisions et le cas F-12. Reste à voir : F-8 ci-dessous. |

## Failles identifiées

### [MAJEUR] F-1 : le juge s'exécute depuis la tête de la PR qu'il juge
- **Axe** : Sécurité & intégrité
- **Source** : A-8 (fichiers protégés : `.github/workflows/*`, `scripts/ci.sh`, `scripts/ci.d/*-st01-*`) ; arbitrage F-9 du plan v1 (« garde-fou contre la dérive d'un agent ») ; F-9 de la v1 (« GitHub exécute le `ci.yml`, le `ci.sh`, les `ci.d/*` […] de la PR »)
- **Tâche(s) concernée(s)** : T-10 (`05-st01-juge.sh`), T-11
- **Description** : `ci.sh` exécute les `ci.d/*.sh` de l'arbre de travail, donc de la tête. Le workflow `pull_request` exécute le `ci.yml` de la PR. Trois des fichiers protégés sont donc précisément le code qui fait le contrôle. Les modifier ne déclenche jamais le contrôle.
- **Preuve (contre-exemples, chacun suffit, aucun label)** :
  1. La PR modifie `mutation.py` (seuil) et remplace le corps de `05-st01-juge.sh` par `exit 0`. Le `05` de la tête sort à 0, et le `50` de la tête lit le seuil abaissé. Le check `ci` est vert.
  2. La PR ajoute `continue-on-error: true` à l'étape `scripts/ci.sh` de `ci.yml`, ou écrit `scripts/ci.sh || true`. Le `05` s'exécute, rend `rouge`, et le job reste vert. C'est la dérive la plus probable d'un agent qui veut « faire passer la CI », et justement le cas que l'arbitrage F-9 vise.
  3. La PR **ajoute** `ci.d/45-st05-prep.sh`, ce qui est libre selon A-8 (« un ajout reste libre »). Ce script écrit un `.cargo/mutants.toml` non suivi, avec `exclude_re`, ou réécrit le seuil sur disque avant que `50` tourne. A-7 ne fouille que `git ls-files` : un fichier créé à l'exécution lui échappe. Rien n'impose non plus le numéro ≥ 60 aux ajouts.
  - Le critère de T-12 ne teste que des modifications de `mutation.py` et de `zones.toml` (démonstrations 7 à 9). Aucune démonstration ne touche `05`, `ci.sh` ou `ci.yml`. Le plan déclare F-9 « Levée (restes : Q-8) », et Q-8 ne couvre aucun de ces trois chemins.

### [MAJEUR] F-2 : ajouter un motif peut abaisser le `Full` par défaut
- **Axe** : Sécurité & intégrité / Cohérence
- **Source** : AC-7.3 et RG-ST01-7.2 (fichier non couvert ⇒ `Full`) ; règle de la story « Zone Full par défaut » ; A-8 (« chaque couple `(motif, zone)` présent à la base doit rester […] ; ajouter une zone ou un motif reste libre »)
- **Tâche(s) concernée(s)** : T-9 (`zones_abaissees`), T-10
- **Description** : le juge protège les couples **explicites** de la base. Il ne protège pas la zone **implicite** `Full` des fichiers qu'aucun motif ne couvre. Le garde-fou ne vise que `Gen`.
- **Preuve** : la PR de ST-05 crée `.ctdd/stories/ST-05/zones.toml` avec `Std = ["crates/core/src/**"]`, pour déclarer en une ligne son nouveau fichier de contexte. Ce fichier est une addition, donc libre. `hlc.rs`, `write.rs` et `merge.rs` restent en `Full` grâce à leurs couples explicites. En revanche, `crates/core/src/lib.rs` et tout fichier du cœur créé plus tard sans motif propre passent de `Full` à `Std` (la règle du plus exigeant ne joue qu'entre motifs qui couvrent le fichier). Aucune ligne ne devient `rouge`. Ce glob large est un raccourci naturel pour un agent, pas un acte malveillant.

### [MAJEUR] F-3 : `zones.toml` laisse `crates/cli/src/main.rs` en `Full`, donc les critères de T-10 et T-14 sont inatteignables
- **Axe** : Cohérence / Complétude
- **Source** : 03_archi, « `zones.toml` recommandé pour ST-01 » (`Gen = ["crates/cli/src/**"]`) et Classification DDD (`main.rs` : Generic, Gen) ; `zones.toml` livré (Full et Std seulement, « inchangé depuis la v1 ») ; RG-ST01-7.2 (fichier non couvert ⇒ `Full`) ; RG-ST01-7.3
- **Tâche(s) concernée(s)** : T-1 (réécrit `main.rs`), T-10 (critère « `ci.sh` vert en local une fois T-7 fait »), T-14 (CI verte)
- **Description** : T-1 réécrit `main.rs` en parseur clap. Le fichier entre donc dans le diff de la PR de ST-01, en `Full` par défaut. Aucune tâche ne lui écrit de test : T-1 vérifie `arbitre --version` à la main.
- **Preuve** : cargo-mutants génère pour ce fichier au moins `replace main with ()`. Aucun test du paquet `arbitre` n'échoue sur ce mutant, qui est donc `missed`. Le score vaut 0/1, sous les 85 %, et `50-st01-mutation.sh` rend `rouge`. Le critère de T-10 et la « CI verte » de T-14 ne peuvent donc pas être tenus, et le plan s'écarte de l'archi sans amendement (A-1 à A-11 ne citent pas `zones.toml`). Le fichier est déjà sur la base (`main`). Le juge (A-8) s'applique donc à ses couples dès la PR de ST-01.

### [MINEUR] F-4 : cas F-12, la « retenue » du conflit est la suppression, alors que la donnée est `présente`
- **Axe** : Edge cases
- **Source** : A-3 (rang `(marque, cible, contenu)`), A-1 (`Visee` : champs avant suppression), A-10 (« retenue = rang max ») ; RG-ST01-2.1 (`retenue = max_<`) ; RG-2.3 ; Q-5
- **Tâche(s) concernée(s)** : T-6 (test nommé F-12)
- **Description** : à marque égale, le rang départage par la cible, et `Suppression` se classe au-dessus de `Champ`. Le conflit rend donc `retenue = suppression`, `écartée = champ`. Pourtant `t_s = t_f` donne `présente`, avec la suppression dans les écartées.
- **Preuve** : la paire nomme comme « retenue » l'écriture que la projection écarte. Q-5 fixe « `présente`, plus un conflit » sans fixer le sens de la paire, et le critère de T-6 ne le vérifie pas. Un mutant qui inverse ce départage survit.

### [MINEUR] F-5 : la commande de T-14 qui rend le check requis échoue
- **Axe** : Cas d'erreur
- **Source** : T-0 (`"required_status_checks": null`) ; T-14 (`gh api -X PATCH …/protection/required_status_checks -f strict=true -f 'contexts[]=ci'`)
- **Tâche(s) concernée(s)** : T-14
- **Description et preuve** :
  - Le sous-objet n'existe pas après T-0. Le `PATCH` vise donc une ressource absente, et l'API répond « Required status checks not enabled ».
  - `-f` envoie une chaîne. `strict` part donc en `"true"`, pas en booléen, et l'API rend une erreur de validation. Il faut `-F`.
  - Le dernier critère de T-14 n'est pas atteignable avec cette commande telle qu'elle est écrite.

### [MINEUR] F-6 : le corps de l'archi garde le « profil de mutation à jeux réduits »
- **Axe** : Cohérence
- **Source** : 03_archi, « Temps de mutation » (« on réduit les jeux dans un profil de mutation ») ; plan, R-2 et Over-engineering (« jamais moins de jeux ») ; arbitrage de l'archi (« A-1 à A-11 font foi », et le corps fait foi sur le reste)
- **Tâche(s) concernée(s)** : T-14
- **Description** : la correction de F-17 vit dans R-2, pas dans un amendement. Le corps de l'archi, qui fait foi hors A-1 à A-11, prescrit toujours le contraire.
- **Preuve** : si la durée de la mutation dépasse le temps acceptable en T-14, deux sources qui font foi prescrivent deux leviers opposés.

### [MINEUR] F-7 : les motifs d'A-6 sont à la fois trop larges et incomplets
- **Axe** : Sécurité & intégrité / Régressions
- **Source** : A-6 ; AC-1.7 et RG-ST01-1.1 (« ne lit ni n'écrit aucun stockage […] aucun réseau ») ; archi, Contraintes (« aucune entrée-sortie, aucun thread ») ; A-8 (`graphe.py` protégé)
- **Tâche(s) concernée(s)** : T-8
- **Description et preuve** :
  - **Trop larges.** La recherche est textuelle et inclut les commentaires. `Instant` se déclenche sur le mot français en début de phrase (« /// Instant courant, en ms UTC », doc naturelle de `Clock::now`) et sur tout identifiant de planning qui contient ce mot (`crates/plan/src`, `InstantDebut`). `SystemTime` se déclenche sur une doc qui renvoie vers l'adaptateur de ST-03. Comme `graphe.py` est gelé, chaque faux positif exige une PR `ci-change`.
  - **Incomplets.** `std::fs`, `std::net`, `std::process` et `std::thread` compilent pour `wasm32-unknown-unknown` (ce sont des souches qui échouent à l'exécution). Ils échappent donc à la construction navigateur, à la règle (b), qui ne vise que des crates, et aux motifs d'A-6. Aucun contrôle ne voit une lecture de fichier ajoutée au cœur, alors qu'AC-1.7 l'interdit au même titre que l'heure.

### [MINEUR] F-8 : sorties incrémentales quadratiques sur une donnée chargée, reçue lot par lot
- **Axe** : Over/Under-engineering
- **Source** : A-10 (« `nouveaux_conflits = conflits_de(d) après \ avant`, pour chaque donnée touchée ») ; matrice US-1, cas 8 ; RG-G3
- **Tâche(s) concernée(s)** : T-6
- **Description** : chaque `fusionner` recalcule deux fois les conflits de toute donnée touchée, en `O(n_d log n)`.
- **Preuve** : prenons les 10 000 écritures du cas 8, reçues en 10 000 lots d'une écriture (synchronisation au fil de l'eau, ST-19). Cela donne environ 2·10^8 parcours d'intervalle. F-20 n'est levée que pour le lot unique : c'est le seul cas que mesure T-6 (« en un seul lot reçu »).

### [MINEUR] F-9 : la même écriture sous deux encodages de `vu` devient une collision
- **Axe** : Edge cases
- **Source** : A-1 (« des entrées en trop […] ne sont pas refusées ») ; A-3 (`Contenu = (valeur?, vu, auteur)`, anomalie = identité à plusieurs contenus) ; Q-6 (lots désérialisés en ST-02 et ST-04b)
- **Tâche(s) concernée(s)** : T-4, T-5
- **Description** : `vu` fait partie du contenu, et sa forme n'est pas canonique, puisque les entrées en trop sont admises.
- **Preuve** : prenons un sérialiseur (ST-02, ST-04b) qui ajoute ou retire une entrée inoffensive, par exemple une entrée `Champ(g)` sur une écriture de `f`. Il réémet une écriture connue avec un `vu` différent. Le journal y voit deux contenus de même identité : une **anomalie** sort, et une copie de la même écriture part aux écartées. Les valeurs sont égales, donc il n'y a pas de conflit, mais le signalement est faux. Ce cas ne se produit qu'aux frontières de désérialisation, qui sont précisément le contrat donné à ST-02.

### [MINEUR] F-10 : « concaténation des `nouveaux_conflits` = `conflits()` » ne précise pas l'égalité
- **Axe** : Cohérence
- **Source** : T-7, propriété « concurrentes ⇔ conflit » ; A-4 (chaque `Rapport` est trié pour lui-même)
- **Tâche(s) concernée(s)** : T-7
- **Description** : une concaténation de listes triées séparément n'est pas triée. Comparée en `Vec`, la propriété échoue dès que deux fusions produisent des conflits dans le désordre global. L'égalité attendue est ensembliste, mais le critère ne le dit pas.
- **Preuve** : la fusion 1 rend un conflit sur `d2`, puis la fusion 2 un conflit sur `d1`. La concaténation vaut `[d2, d1]`, alors que `conflits()` rend `[d1, d2]`.

## Axes sans faille détectée
- **Complétude** : RAS. Chaque cas des matrices US-1 à US-8 a une tâche qui le nomme (seuls les cas 5 et 6 de la matrice US-8 sont couverts par le contenu de T-2 sans être cités). Le tableau des invariants est complet.
- **Cohérence du moteur** : RAS. Avec `vu` par visée, le journal porteur d'horloge, les collisions gardées et l'intervalle d'A-10, aucun contre-exemple de perte silencieuse hors de la limite arbitrée. `vu < marque` est tenu par construction, puisque l'horloge adopte le maximum de tout lot accepté et que toute émission locale passe par `ecrire`.
- **Régressions** : RAS. Rien n'est publié ni persisté. La CLI n'a aucun consommateur.

## Recommandation
- **F-1 à F-3 → Planner**, avant le Skeleton. F-1 touche le périmètre de l'arbitrage F-9 et peut remonter à Quentin, si la fermer demande de sortir du « garde-fou contre la dérive ». F-3 est un écart à l'archi.
- **MINEURS** : à traiter au fil de l'eau. F-5 se corrige à l'écriture de T-14, et F-6 par un amendement d'une ligne.
- C'était l'itération 2 sur 3. Aucun BLOQUANT : le plan peut passer au Skeleton une fois les MAJEURS corrigés, sans nouvelle itération DA si la correction ne touche pas le moteur.
