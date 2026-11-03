# Spécifications Fonctionnelles — ST-01 : Fusion pure (horloge, écritures, versions écartées) et fondations du dépôt

## Traçabilité
- Source : `.ctdd/stories/ST-01/01_story.md` (cahier des charges StoryWriter). Sa section « Arbitrages de Quentin (2026-11-03) » fait foi.
- Cadre : `.ctdd/cadre/02_specs.md` (RG-2.1 à 2.4, INV-4 à 7, CONFLIT-3 à 5), les sections « Arbitrages de Quentin » de `.ctdd/cadre/01_story.md`, `02_specs.md` et `03_archi.md`, et `.ctdd/cadre/04_plan.md` § ST-01 (PL-6, F-16, F-17, F-36).
- Date : 2026-11-03
- Statut : DRAFT
- Contexte codebase : workspace Cargo (`resolver = "3"`, édition 2024). `crates/core` (`arbitre-core`) contient la fonction `add` de démonstration et dépend de `chrono` (feature `clock`) et de `rusqlite` (`bundled`). `crates/cli` (`arbitre`) affiche « Hello, world! » et dépend de `chrono` (`clock`) et de `clap`. Aucune `[workspace.dependencies]`, aucune CI.

### Conventions de ce document
- `US-n` et `AC-n.m` désignent les user stories et les critères **de ST-01**. Une référence au cadre est préfixée : `cadre US-2/AC-2.6`.
- Les identifiants du cadre sont repris tels quels : `RG-2.1` à `RG-2.4`, `INV-4` à `INV-7`, `CONFLIT-3` à `CONFLIT-5`, `RG-G2`, `RG-G3`, `RG-G7`, `RG-G9`, `INV-22`.
- Les identifiants créés ici portent le préfixe `ST01` pour ne pas entrer en collision avec ceux du cadre : `RG-ST01-x.y`, `INV-ST01-n`, `PROP-ST01-n`, `CONFLIT-ST01-n`, `AMB-ST01-n`, `MANQUE-ST01-n`.
- `[ARB]` : la règle découle d'un arbitrage (cadre ou 2026-11-03). `[HYP]` : elle repose sur une hypothèse du cahier. `[DÉRIVÉE]` : absente du cahier en toutes lettres mais nécessaire pour qu'une AC soit tenable ; à valider. `[À DÉFINIR]` : valeur manquante, proposition jointe.
- Notation : `S` est un ensemble d'écritures (le journal connu d'un appareil) ; `⊥` est une marque plus petite que toute autre ; `<` sur les marques est l'ordre de RG-ST01-4.1.

### Analyse de complétude du cahier
Le cahier couvre les huit US avec des AC testables, et les arbitrages du 2026-11-03 ferment ses cinq questions ouvertes. Restent trois points bloquants pour la suite du cycle, détaillés dans `Conflits et ambiguïtés` :
1. **AC-2.4 ne permet pas de tenir AC-2.3** : le point de dernière réception tenu par l'appareil récepteur ne dit pas si l'auteur d'une écriture avait vu la précédente. Contre-exemple et options en `CONFLIT-ST01-1`. **À trancher par Quentin.**
2. **Un script unique (arbitrage CI) contre des vérifications ajoutées dans de nouveaux fichiers (AC-6.6)** : `CONFLIT-ST01-2`.
3. **Seuil de mutation de la zone `Gen`** (zone du plan, absente d'AC-7.2) : `MANQUE-ST01-1`. ST-02 en dépend.

## Glossaire du domaine

| Terme | Définition |
|-------|-----------|
| Appareil | Installation de l'outil, identifiée par un identifiant unique et totalement ordonné (unicité garantie hors ST-01, par ST-02). |
| Donnée | Objet identifié `(module, genre, id)`, porteur de champs. Le moteur n'en connaît que l'identifiant. |
| Champ | Attribut nommé d'une donnée. Sa valeur est opaque pour le moteur, qui ne sait que tester l'égalité de deux valeurs. |
| Marque | Marque de temps d'horloge logique hybride `(l, c, a)` : `l` temps physique, `c` compteur, `a` appareil. Synonyme du terme « estampille » du cadre. |
| Heure physique | Valeur rendue par la source d'heure de l'appareil. Peut avancer, reculer ou rester figée. |
| Source d'heure | Fournisseur remplaçable de l'heure physique (port `Clock` de l'archi). Seule sa lecture de l'heure courante sert à ST-01. |
| Auteur | Origine humaine ou logicielle d'une écriture : `utilisateur-cli`, `utilisateur-interface`, `assistant`, `système`. |
| Écriture de champ | `(donnée, champ, valeur, marque, appareil, auteur)` : affecte une valeur à un champ. |
| Suppression datée | `(donnée, marque, appareil, auteur)` : marque une donnée comme supprimée. Se compare aux écritures de champ comme une écriture ordinaire. |
| Écriture | Écriture de champ ou suppression datée. |
| Cible | Ce que vise une écriture : le couple `(donnée, champ)` pour une écriture de champ, `(donnée, suppression)` pour une suppression. |
| Identité d'une écriture | `(cible, marque)`. Deux écritures de même identité sont la même écriture. |
| Lot | Ensemble fini d'écritures reçues ensemble. Peut être vide. |
| Journal connu | Ensemble `S` de toutes les écritures connues d'un appareil. Croît par union ; rien n'en sort. |
| Gagnante d'un champ | Écriture de plus grande marque parmi celles du champ dans `S`. |
| État projeté | Pour chaque donnée connue : son statut (`présente` ou `supprimée`) et la gagnante de chacun de ses champs. |
| Version écartée | Écriture de `S` qui n'est pas retenue dans l'état projeté (RG-2.4). |
| Connaissance | `a_vu(w, x)` : l'auteur de `w`, au moment de l'écrire, connaissait `x`. Source d'information : voir `CONFLIT-ST01-1`. |
| Écritures concurrentes | Deux écritures dont aucune n'a vu l'autre : `¬a_vu(w, x) ∧ ¬a_vu(x, w)`. |
| Conflit | Paire d'écritures concurrentes sur la même donnée, de valeurs différentes, qui ne sont pas deux suppressions (RG-ST01-2.1). |
| Écrasement séquentiel | Remplacement d'une écriture par une autre faite en connaissance d'elle. Jamais un conflit (AC-2.3). |
| Point de dernière réception | Pour un appareil pair, plus grande marque reçue de lui (« filigrane » de l'archi). |
| Avance d'horloge | Écart entre la composante `l` d'une marque reçue et l'heure physique locale au moment de la réception. |
| Moteur de fusion | Fonction pure qui, à partir d'un journal connu et d'un lot, rend l'état projeté, les nouvelles versions écartées et les nouveaux conflits. |
| Cœur | Composant `arbitre-core`. |
| Composant | Crate du workspace. Composants prévus par l'archi : `arbitre-core`, `arbitre-plan`, `arbitre-diff`, `arbitre-sqlite`, `arbitre-relay`, `arbitre-ai`, `arbitre-calendar`, `arbitre-web`, `arbitre`. |
| Cible navigateur | `wasm32-unknown-unknown`. |
| Liste navigateur | Liste nommée des composants qui doivent se construire pour la cible navigateur : `arbitre-core`, `arbitre-plan`. |
| Arête interdite | Dépendance, directe ou transitive, d'un composant vers un autre que l'architecture proscrit. |
| Vérification | Exécution de l'ensemble des contrôles du dépôt par le point d'entrée unique `scripts/ci.sh`, en local comme sur l'hébergeur. Verdict `vert` ou `rouge`. |
| Modification proposée | Pull request sur le dépôt public GitHub. |
| Mutant | Variante du code obtenue par une altération volontaire. Il est **détecté** si au moins un test échoue sur lui. |
| Mutant équivalent | Mutant dont le comportement observable est identique au code d'origine. Exclu du score seulement s'il est listé avec une justification [ARB]. |
| Score de mutation | Par fichier : mutants détectés / mutants comptés (RG-ST01-7.3). |
| Zone | Niveau d'exigence d'un fichier : `Full` (seuil 85 %), `Std` (seuil 70 %). `Gen` : voir `MANQUE-ST01-1`. |
| Déclaration de zones | Fichier `.ctdd/stories/ST-xx/zones.toml` : motifs de chemins associés à une zone. |
| Dépendance commune | Bibliothèque déclarée au niveau du workspace, avec sa version, pour tous les composants. |
| Fichier de verrouillage | `Cargo.lock`. |

## Spécifications par User Story

### SPEC-US-1 : Converger vers le même état, quel que soit l'ordre d'arrivée
**Réf :** US-1 ; cadre US-2/AC-2.6, RG-G7 ; arbitrage cadre « Conflits de synchronisation », CONFLIT-4/5

#### Règles métier formalisées

**RG-2.1 : Dernière écriture gagne, champ par champ** `@Invariant` [ARB]
- **Réf AC :** [US-1/AC-1.1], [US-1/AC-1.2]
- **Pré-conditions :** un journal connu `S` ; une donnée `d` et un champ `f` portant au moins une écriture de champ dans `S`.
- **Post-conditions :** la valeur de `f` dans l'état projeté est celle de l'écriture de plus grande marque. Les champs sont indépendants : la gagnante de `(d, f)` ne dépend d'aucune écriture d'un autre champ.
- **Expression formelle :**
  - `gagnante(S, d, f) = max_< { w ∈ S | w.cible = (d, f) }`
  - `∀ f ≠ f', gagnante(S, d, f) = gagnante(S ∪ E_{f'}, d, f)` où `E_{f'}` est un ensemble quelconque d'écritures sur `(d, f')`.
  - L'ordre `<` contient l'appareil (RG-ST01-4.1) : deux écritures distinctes de même `(l, c)` sont départagées par l'appareil, avec le même résultat sur tous les appareils.
- **Cas d'erreur :**
  - Deux écritures distinctes de même identité (même cible, même marque, valeurs différentes) → hors du modèle, puisqu'un appareil n'émet jamais deux fois la même marque (RG-ST01-4.2). Traitement : voir `MANQUE-ST01-2`.

**RG-2.2 : Convergence de la fusion** `@Invariant`
- **Réf AC :** [US-1/AC-1.3], [US-1/AC-1.4], [US-1/AC-1.5], [US-3/AC-3.5]
- **Pré-conditions :** des lots quelconques, y compris vides, avec suppressions, doublons et marques de même `l`.
- **Post-conditions :** le résultat de la fusion ne dépend que de l'union des écritures reçues. Le résultat comprend l'état projeté, l'ensemble cumulé des versions écartées et l'ensemble cumulé des conflits.
- **Expression formelle :** soit `R(S) = (état_projeté(S), écartées(S), conflits(S))` et `fusion(S, L) = S ∪ L`.
  - commutativité : `R(fusion(fusion(S, A), B)) = R(fusion(fusion(S, B), A))`
  - associativité : `R(fusion(S, fusion(A, B))) = R(fusion(fusion(S, A), B))`
  - idempotence : `R(fusion(fusion(S, A), A)) = R(fusion(S, A))`, et la seconde réception de `A` rend des listes vides de nouvelles écartées et de nouveaux conflits.
  - élément neutre : `R(fusion(S, ∅)) = R(S)`, sans nouvelle écartée ni nouveau conflit.
  - Les sorties incrémentales d'une fusion sont les différences : `nouvelles_écartées = écartées(S ∪ L) \ écartées(S)`, `nouveaux_conflits = conflits(S ∪ L) \ conflits(S)`.
- **Cas d'erreur :** aucun (propriété algébrique). Un lot invalide est refusé avant fusion (RG-ST01-4.5).

**RG-ST01-1.1 : Moteur pur** `@Invariant`
- **Réf AC :** [US-1/AC-1.7] ; règle globale « Déterminisme » du cahier
- **Pré-conditions :** aucune.
- **Post-conditions :** à entrées égales, sorties identiques, octet pour octet et dans le même ordre. Le moteur ne lit ni n'écrit aucun stockage, ne lit aucune heure système et ne contacte aucun réseau.
- **Expression formelle :** `∀ S, L, env₁, env₂ : fusion_{env₁}(S, L) = fusion_{env₂}(S, L)`, où `env` couvre l'heure de la machine, son fuseau, sa locale, l'ordre de construction des lots et toute graine d'itération.
- **Cas d'erreur :**
  - Le moteur a besoin de l'heure courante → elle lui est passée comme valeur d'entrée (cas de RG-ST01-4.6), jamais lue.

**Exigence E-1.1 : Vérification par propriétés en grand nombre** — non classée (exigence de vérification)
- **Réf AC :** [US-1/AC-1.6], [US-3/AC-3.5]
- **Expression :** RG-2.1, RG-2.2, RG-2.3, RG-2.4 et RG-ST01-2.1 sont vérifiées sur des jeux générés au hasard, au moins `N` jeux par propriété à chaque exécution courante. Les générateurs produisent, avec une probabilité non négligeable : des lots vides, des doublons, au moins trois appareils, des marques de même `l` (et de même `(l, c)` sur appareils différents), des suppressions, des écritures concurrentes de même valeur, des écrasements séquentiels. Un échec est rejouable à partir de sa graine.
- **`[À DÉFINIR]` :** `N`. Proposition : 1 000 en zone Full (plancher du cahier : « plusieurs centaines »). Voir `MANQUE-ST01-7`.

#### Matrice de cas — US-1

| # | Cas | Type | Input | État | Résultat attendu | Réf règle |
|---|-----|------|-------|------|-------------------|-----------|
| 1 | Champs différents, deux appareils | Nominal | A : titre=v1 à t1 ; B : échéance=v2 à t2 | `S` vide | titre=v1, échéance=v2 ; aucune écartée, aucun conflit | RG-2.1 |
| 2 | Même champ, marques différentes | Nominal | A : f=v1 à (10,0,A) ; B : f=v2 à (12,0,B) | `S` vide | f=v2 ; v1 écartée | RG-2.1, RG-2.4 |
| 3 | Même `(l, c)`, appareils différents | Edge | A : f=v1 à (10,0,A) ; B : f=v2 à (10,0,B), A < B | `S` vide | f=v2 sur tous les appareils ; v1 écartée | RG-2.1, RG-ST01-4.1 |
| 4 | Ordres d'arrivée différents | Edge | lots A, B, C | appareil X reçoit A, B, C ; Y reçoit C, A, B | `R` identique sur X et Y | RG-2.2 |
| 5 | Regroupements différents | Edge | lots A, B, C | X : A puis (B ∪ C) ; Y : (A ∪ B) puis C | `R` identique | RG-2.2 |
| 6 | Lot reçu deux fois | Edge | lot A deux fois | — | même `R` ; 2e réception : aucune nouvelle écartée, aucun nouveau conflit | RG-2.2 |
| 7 | Lot vide | Edge | `∅` | `S` quelconque | `R(S)` inchangé ; listes vides | RG-2.2 |
| 8 | 10 000 écritures sur un champ | Edge | 10 000 marques distinctes sur (d, f) | `S` vide | une gagnante ; 9 999 écartées, toutes rendues | RG-2.1, RG-2.4 |
| 9 | Même entrée, environnements différents | Edge | `S`, `L` fixés | fuseau, locale, heure machine différents | sorties identiques, même ordre | RG-ST01-1.1 |
| 10 | Écriture d'une identité déjà connue, valeur différente | Erreur | (d, f, v2, m) avec (d, f, v1, m) ∈ `S` | — | voir `MANQUE-ST01-2` | RG-2.1 |
| 11 | Lot avec écriture mal formée | Erreur | écriture sans champ ou auteur hors liste | `S` quelconque | lot refusé en entier, `R(S)` inchangé | RG-ST01-4.5 |

### SPEC-US-2 : Ne jamais perdre une valeur en silence
**Réf :** US-2 ; RG-G7 ; arbitrages CONFLIT-4/5 et 2026-11-03 (« Même valeur », « Double suppression »)

#### Règles métier formalisées

**RG-2.4 : Pas de perte silencieuse à la fusion** `@Invariant`
- **Réf AC :** [US-2/AC-2.1], [US-2/AC-2.3]
- **Pré-conditions :** un journal connu `S`.
- **Post-conditions :** toute écriture de `S` est soit retenue dans l'état projeté, soit dans les versions écartées, jamais dans les deux, jamais dans aucun. Une version écartée rend son champ (ou la marque de suppression), sa valeur, sa marque, son appareil et son auteur.
- **Expression formelle :**
  - `retenues(S) = { gagnante(S, d, f) } ∪ { suppression_retenue(S, d) | supprimée(S, d) }` (RG-2.3)
  - `écartées(S) = S \ retenues(S)`
  - partition : `retenues(S) ∩ écartées(S) = ∅` et `retenues(S) ∪ écartées(S) = S`
  - monotonie : `S ⊆ S' ⇒ écartées(S) ⊆ écartées(S')`. Une écriture écartée ne revient jamais à l'état retenu.
  - Les écritures de champ d'une donnée supprimée restent retenues (masquées) dans l'état projeté : voir `AMB-ST01-1`.
- **Cas d'erreur :**
  - Écrasement séquentiel → l'écriture remplacée est écartée, sans conflit (RG-ST01-2.1).

**RG-ST01-2.1 : Définition du conflit** `@Invariant` [ARB]
- **Réf AC :** [US-2/AC-2.2], [US-2/AC-2.3], [US-2/AC-2.4] ; arbitrages 2026-11-03 « Même valeur » et « Double suppression » ; [HYP] suppression contre écriture de champ concurrentes
- **Pré-conditions :** un journal connu `S` ; la connaissance `a_vu` évaluable pour toute paire d'écritures de `S` (source : `CONFLIT-ST01-1`).
- **Post-conditions :** une paire est un conflit si et seulement si les deux écritures sont concurrentes, portent sur la même donnée, peuvent s'exclure (même champ, ou suppression contre écriture de champ), ont des valeurs différentes et ne sont pas deux suppressions. Chaque conflit est rendu avec la donnée, le champ, l'écriture retenue de la paire et l'écriture écartée de la paire.
- **Expression formelle :**
  - `concurrentes(w, x) ⇔ ¬a_vu(w, x) ∧ ¬a_vu(x, w)`
  - `exclusives(w, x) ⇔ (w.cible = x.cible ∧ w.cible est un champ) ∨ (une seule des deux est une suppression ∧ w.donnée = x.donnée)`
  - `conflit(w, x) ⇔ concurrentes(w, x) ∧ exclusives(w, x) ∧ w.valeur ≠ x.valeur`
    (une suppression n'a pas de valeur et diffère de toute valeur de champ ; deux suppressions ne sont jamais exclusives, donc jamais en conflit)
  - `conflits(S) = { {w, x} ⊆ S | conflit(w, x) }`, rendu sous la forme `(donnée, champ, retenue = max_<(w, x), écartée = min_<(w, x))`. Pour une paire suppression / écriture de champ, `champ` est celui de l'écriture de champ.
  - Proposition pour `a_vu`, sous réserve de `CONFLIT-ST01-1` : `a_vu(w, x) ⇔ vu(w) ≥ marque(x)`, où `vu(w)` est la plus grande marque connue de l'auteur de `w`, au moment d'écrire, parmi les écritures de la même cible et les suppressions de la même donnée (`⊥` s'il n'en connaissait aucune).
- **Cas d'erreur :**
  - Écritures concurrentes de même valeur → pas de conflit ; la non retenue est écartée sans signalement [ARB].
  - Deux suppressions concurrentes → pas de conflit ; la plus ancienne est écartée sans signalement [ARB].
  - Trois écritures concurrentes ou plus sur un champ → un conflit par paire : voir `AMB-ST01-2`.
  - Appareil pair dont on n'a jamais rien reçu → son auteur n'avait rien vu de local : toute écriture locale exclusive est concurrente de la sienne [HYP] (avec la proposition : `vu = ⊥`).

**RG-ST01-2.2 : Rapports triés** `@Invariant`
- **Réf AC :** [US-2/AC-2.5] ; règle globale « Déterminisme »
- **Pré-conditions :** une fusion qui rend des versions écartées, des conflits ou des avances (RG-ST01-4.6).
- **Post-conditions :** chaque liste rendue est triée selon un ordre total fixe, indépendant de l'ordre d'arrivée.
- **Expression formelle :**
  - versions écartées : tri par `(donnée, cible, marque)` ;
  - conflits : tri par `(donnée, champ, marque retenue, marque écartée)` ;
  - avances : tri par `appareil`.
  - L'ordre sur les identifiants de données et d'appareils est total, et ne dépend ni de la locale ni de la plateforme (forme fixée à l'archi).
- **Cas d'erreur :** aucun.

#### Matrice de cas — US-2

| # | Cas | Type | Input | État | Résultat attendu | Réf règle |
|---|-----|------|-------|------|-------------------|-----------|
| 1 | Même champ modifié hors ligne des deux côtés (cadre matrice US-2 n° 3) | Nominal | A : f=v1 (10,0,A), sans avoir vu B ; B : f=v2 (12,0,B), sans avoir vu A | — | f=v2 ; v1 écartée ; conflit (d, f, retenue v2, écartée v1) | RG-2.1, RG-2.4, RG-ST01-2.1 |
| 2 | Écrasement séquentiel | Nominal | A : f=v1 (10,0,A) ; B reçoit v1, puis écrit f=v2 (11,0,B) | — | f=v2 ; v1 écartée ; aucun conflit | RG-2.4, RG-ST01-2.1 |
| 3 | Écrasement séquentiel, récepteur en retard de synchro | Edge | comme n° 2 ; A n'a reçu de B que jusqu'à (5,0,B) | — | aucun conflit (contre-exemple de `CONFLIT-ST01-1`) | RG-ST01-2.1 |
| 4 | Concurrentes de même valeur | Edge | A : f=v (10,0,A) ; B : f=v (12,0,B), concurrentes | — | f=v (marque de B) ; écriture de A écartée ; aucun conflit | RG-ST01-2.1 [ARB] |
| 5 | Trois appareils | Edge | A : f=v1, B : f=v2, concurrentes ; C reçoit les deux | A, B, C | même état sur A, B, C ; même conflit dans `conflits(S)` sur les trois | RG-2.2, RG-ST01-2.1 |
| 6 | Trois concurrentes sur un champ | Edge | A, B, C écrivent f concurrentes, valeurs distinctes | — | une gagnante ; deux écartées ; trois conflits (un par paire) | RG-ST01-2.1, `AMB-ST01-2` |
| 7 | Pair jamais synchronisé | Edge | lot du pair P sur f ; f modifié localement | aucune réception de P | conflit signalé | RG-ST01-2.1 [HYP] |
| 8 | Rejeu des mêmes entrées | Edge | mêmes `S`, `L` deux fois | — | mêmes listes, même ordre | RG-ST01-2.2, RG-ST01-1.1 |
| 9 | Valeur écartée complète | Nominal | tout cas avec écartée | — | l'écartée rend champ, valeur, marque, appareil, auteur | RG-2.4 |
| 10 | Conflit déjà signalé, nouvelle écriture séquentielle | Edge | après n° 1, A écrit f=v3 en connaissant v2 | — | f=v3 ; v2 écartée ; aucun nouveau conflit ; le conflit v1/v2 reste dans `conflits(S)` | RG-2.2, RG-ST01-2.1 |

### SPEC-US-3 : Traiter une suppression comme une modification datée
**Réf :** US-3 ; arbitrage cadre « Conflits » ; CONFLIT-3 (résolution par défaut retenue [HYP])

#### Règles métier formalisées

**RG-2.3 : Suppression = écriture datée** `@Invariant` [ARB]
- **Réf AC :** [US-3/AC-3.1], [US-3/AC-3.2], [US-3/AC-3.3], [US-3/AC-3.4]
- **Pré-conditions :** une donnée `d` portant au moins une écriture (de champ ou de suppression) dans `S`.
- **Post-conditions :** la donnée est supprimée si et seulement si sa suppression la plus récente est plus récente que la plus récente de ses écritures de champ. Une donnée présente montre, pour chaque champ, la gagnante de RG-2.1, y compris les champs écrits avant une suppression écartée (« résurrection »).
- **Expression formelle :**
  - `t_s(S, d) = max_< ({ marque(s) | s ∈ S suppression de d } ∪ {⊥})`
  - `t_f(S, d) = max_< ({ marque(w) | w ∈ S écriture de champ de d } ∪ {⊥})`
  - `supprimée(S, d) ⇔ t_s(S, d) > t_f(S, d)` (INV-6 ; comparaison fixée par CONFLIT-3, résolution par défaut)
  - `suppression_retenue(S, d)` = la suppression de marque `t_s(S, d)`, si `supprimée(S, d)` ; sinon aucune.
  - Toute autre suppression de `d` est écartée (RG-2.4).
  - L'égalité `t_s = t_f` est impossible entre deux écritures distinctes (la marque contient l'appareil, RG-ST01-4.1).
- **Cas d'erreur :**
  - Suppression et écriture de champ concurrentes, de valeurs forcément différentes → conflit (RG-ST01-2.1) [HYP].
  - Donnée sans aucune écriture de champ, avec une suppression → supprimée (`t_f = ⊥`).
  - Remise en place des effets dérivés d'une donnée ressuscitée → hors scope (RG-10.1, stories de blocs).

#### Matrice de cas — US-3

| # | Cas | Type | Input | État | Résultat attendu | Réf règle |
|---|-----|------|-------|------|-------------------|-----------|
| 1 | Supprimée sur A puis modifiée plus tard sur B, sans s'être vus (cadre n° 5) | Edge | A : suppr (10,0,A) ; B : f=v (12,0,B) | — | `présente`, f=v ; suppression écartée ; conflit (d, f, retenue v, écartée suppression) | RG-2.3, RG-2.4, RG-ST01-2.1 |
| 2 | Modifiée sur A puis supprimée plus tard sur B, sans s'être vus (cadre n° 6) | Edge | A : f=v (10,0,A) ; B : suppr (12,0,B) | — | `supprimée` ; f=v reste dans l'état projeté (masquée) ; conflit (d, f, retenue suppression, écartée v) | RG-2.3, RG-ST01-2.1, `AMB-ST01-1` |
| 3 | Suppression séquentielle | Nominal | A : f=v (10,0,A) ; B la reçoit, puis supprime (11,0,B) | — | `supprimée` ; aucun conflit | RG-2.3, RG-ST01-2.1 |
| 4 | Résurrection séquentielle | Nominal | A : suppr (10,0,A) ; B la reçoit, puis écrit f=v (11,0,B) | — | `présente` ; suppression écartée ; aucun conflit | RG-2.3, RG-ST01-2.1 |
| 5 | Même `(l, c)`, suppression contre champ | Edge | A : suppr (10,0,A) ; B : f=v (10,0,B), A < B | — | `présente` (B l'emporte) sur tous les appareils | RG-2.3, RG-ST01-4.1 |
| 6 | Double suppression concurrente | Edge | A : suppr (10,0,A) ; B : suppr (12,0,B) | — | `supprimée` ; suppression de A écartée ; aucun conflit | RG-2.3, RG-ST01-2.1 [ARB] |
| 7 | Résurrection, tous champs | Edge | f1=a (5), f2=b (6), suppr (8), f1=c (9) | — | `présente`, f1=c, f2=b ; suppression écartée | RG-2.3 |
| 8 | Convergence avec suppressions | Edge | lots générés avec suppressions | ordres quelconques | RG-2.2 vérifiée | RG-2.2, E-1.1 |
| 9 | Suppression seule | Edge | suppr de d, aucune écriture de champ | — | `supprimée` | RG-2.3 |

### SPEC-US-4 : Dater les modifications dans un ordre fidèle à ce que l'utilisateur a vu
**Réf :** US-4 ; arbitrage cadre CONFLIT-4/5 (HLC) ; arbitrage 2026-11-03 « Horloge en avance »

#### Règles métier formalisées

**RG-ST01-4.1 : Ordre total des marques** `@Invariant` [ARB]
- **Réf AC :** [US-4/AC-4.1], [US-1/AC-1.1]
- **Pré-conditions :** deux marques `m₁ = (l₁, c₁, a₁)`, `m₂ = (l₂, c₂, a₂)`.
- **Post-conditions :** les marques sont toujours comparables, et l'ordre ne dépend d'aucun contexte local.
- **Expression formelle :** `m₁ < m₂ ⇔ (l₁, c₁, a₁) <_lex (l₂, c₂, a₂)` ; ordre total, strict, antisymétrique et transitif. `⊥` est inférieur à toute marque.
- **Cas d'erreur :** aucun. L'unité de `l` est fixée à l'archi ; l'ordre n'en dépend pas.

**RG-ST01-4.2 : Émission d'une marque** `@Invariant`
- **Réf AC :** [US-4/AC-4.3], [US-4/AC-4.1]
- **Pré-conditions :** état d'horloge de l'appareil `a` : `(l, c)` (initialement `(0, 0)` ou équivalent) ; heure physique lue `pt`.
- **Post-conditions :** la marque émise est strictement supérieure à toute marque déjà émise **ou reçue** par l'appareil, et sa composante `l` n'est jamais en retard sur l'heure physique lue.
- **Expression formelle :**
  - `l' = max(l, pt)` ; `c' = c + 1 si l' = l, sinon 0` ; marque émise `(l', c', a)` ; nouvel état `(l', c')`.
  - Conséquences vérifiables : `marque_émise > (l, c, a)` ; `l' ≥ pt` ; pour toute suite d'heures physiques (croissante, figée ou décroissante), les marques émises successives sont strictement croissantes.
- **Cas d'erreur :**
  - Heure physique figée ou qui recule → `l` inchangé, `c` croît ; la croissance stricte tient.
  - Dépassement de la borne du compteur → voir `MANQUE-ST01-3`.

**RG-ST01-4.3 : Réception d'une marque (causalité)** `@Invariant` [ARB]
- **Réf AC :** [US-4/AC-4.2] ; CONFLIT-4
- **Pré-conditions :** état `(l, c)` de l'appareil `a` ; lot reçu, de plus grande marque `m = (lm, cm, am)`.
- **Post-conditions :** toute marque émise ensuite par `a` est strictement supérieure à `m`, quelle que soit l'heure physique de `a`.
- **Expression formelle :** nouvel état `(l, c) ← max_lex((l, c), (lm, cm))`. Par RG-ST01-4.2, toute émission suivante rend `(l', c', a) > m`.
- **Cas d'erreur :**
  - Marque reçue très en avance → acceptée telle quelle, sans perte ; l'horloge locale adopte son `l` [ARB]. Signalement : RG-ST01-4.6.
  - Lot vide → état inchangé.

**RG-ST01-4.4 : Source d'heure remplaçable** `@Property`
- **Réf AC :** [US-4/AC-4.4], [US-4/AC-4.2], [US-4/AC-4.3] ; [US-5/AC-5.1]
- **Pré-conditions :** un ou plusieurs appareils simulés, chacun avec sa source d'heure.
- **Post-conditions :** l'appareil n'obtient l'heure physique que de sa source ; une source simulée suffit à vérifier AC-4.2 et AC-4.3 sans attendre.
- **État externe requis :** heure de chaque source, état d'horloge de chaque appareil, ordre des émissions et des réceptions.
- **Simulateur nécessaire :** source d'heure commandable (`avancer(Δ)`, `reculer(Δ)`, `figer`, `fixer(t)`), une par appareil simulé, sans lien avec l'heure de la machine.
- **Expression formelle :**
  - source figée, `n` émissions → `n` marques strictement croissantes ;
  - source qui recule entre deux émissions → seconde marque > première ;
  - A en avance de 5 min sur B ; A émet `m` ; B reçoit `m` puis émet `m'` une minute de temps simulé plus tard → `m' > m`.
- **Cas d'erreur :**
  - Lecture de l'heure système par le cœur → interdite (RG-ST01-5.1) ; détectée par la vérification.

**RG-ST01-4.5 : Contenu d'une écriture** `@Invariant`
- **Réf AC :** [US-4/AC-4.5] ; [US-2/AC-2.1]
- **Pré-conditions :** une écriture présentée au moteur.
- **Post-conditions :** une écriture de champ porte donnée, champ, valeur, marque, appareil, auteur ; une suppression porte donnée, marque, appareil, auteur. L'appareil de l'écriture est celui de sa marque. L'auteur appartient à la liste fermée. Selon `CONFLIT-ST01-1`, l'écriture porte aussi l'information de connaissance.
- **Expression formelle :** `valide(w) ⇔ champs_requis(w) ∧ w.appareil = w.marque.a ∧ w.auteur ∈ {utilisateur-cli, utilisateur-interface, assistant, système}` ; `∃ w ∈ L, ¬valide(w) ⇒ L refusé en entier, R(S) inchangé`. [DÉRIVÉE] pour le refus du lot entier, par cohérence avec INV-3 du cadre.
- **Cas d'erreur :**
  - Champ requis absent, auteur hors liste, appareil différent de celui de la marque → lot refusé, erreur qui nomme l'écriture fautive, état inchangé.

**RG-ST01-4.6 : Signalement d'une horloge en avance** `@Invariant` [ARB]
- **Réf AC :** arbitrage 2026-11-03 « Horloge en avance » ; [US-4] edge « appareil très en avance »
- **Pré-conditions :** un lot reçu ; l'heure physique locale `maintenant`, passée en valeur (jamais lue par le moteur, RG-ST01-1.1) ; un seuil `σ` (défaut : 60 min, réglable).
- **Post-conditions :** le lot est accepté sans perte, quel que soit l'écart. Le signalement nomme chaque appareil auteur dont une écriture du lot dépasse `maintenant` de plus de `σ`, avec son avance maximale dans le lot. Le signalement ne change ni l'état projeté, ni les écartées, ni les conflits.
- **Expression formelle :** `avances(L, maintenant, σ) = { (a, max{ w.marque.l − maintenant | w ∈ L, w.appareil = a }) | ∃ w ∈ L, w.appareil = a ∧ w.marque.l − maintenant > σ }`, trié par appareil (RG-ST01-2.2). L'avance exactement égale à `σ` n'est pas signalée.
- **Cas d'erreur :**
  - `σ ≤ 0` → réglage refusé, `σ` inchangé `[À DÉFINIR]` (proposition).
  - Un appareil honnête qui a reçu une marque en avance émet à son tour des marques en avance (RG-ST01-4.3) et peut être nommé : voir `CONFLIT-ST01-3`.
  - Où le seuil est conservé → hors scope (stockage, ST-02) ; ST-01 le reçoit en entrée.

#### Matrice de cas — US-4

| # | Cas | Type | Input | État | Résultat attendu | Réf règle |
|---|-----|------|-------|------|-------------------|-----------|
| 1 | Correction faite sur un appareil en retard (cas nominal US-4) | Nominal | ordinateur +5 min émet `m` ; téléphone reçoit `m`, émet `m'` 1 min réelle plus tard | sources simulées | `m' > m` ; la correction gagne | RG-ST01-4.3, RG-ST01-4.4, RG-2.1 |
| 2 | Deux émissions au même instant | Edge | 2 émissions, source figée | — | marques strictement croissantes (`c` croît) | RG-ST01-4.2 |
| 3 | 100 000 émissions au même instant | Edge | source figée | — | ordre strict conservé, ou `MANQUE-ST01-3` si la borne du compteur est atteinte | RG-ST01-4.2 |
| 4 | Heure qui recule | Edge | émission à t=100, source reculée à t=40, émission | — | 2e marque > 1re ; `l` reste 100 | RG-ST01-4.2 |
| 5 | Heure qui avance après un gel | Nominal | état (100, 7), `pt`=150 | — | marque (150, 0, a) | RG-ST01-4.2 |
| 6 | Marques comparables partout | Nominal | paires de marques générées | appareils quelconques | même comparaison sur tous les appareils | RG-ST01-4.1 |
| 7 | Écriture reçue en avance de 3 h | Edge | `m.l = maintenant + 3 h`, σ = 1 h | — | lot accepté ; avance signalée (appareil, 3 h) ; horloge locale adopte `m.l` | RG-ST01-4.3, RG-ST01-4.6 |
| 8 | Avance exactement égale au seuil | Edge | `m.l = maintenant + σ` | — | accepté, non signalé | RG-ST01-4.6 |
| 9 | Seuil réglé à 10 min | Edge | avance de 20 min, σ = 10 min | — | signalé | RG-ST01-4.6 |
| 10 | Écriture reçue en retard (appareil en retard) | Edge | `m.l = maintenant − 2 h` | — | acceptée, rien à signaler | RG-ST01-4.6 |
| 11 | Écriture complète | Nominal | écriture de chaque auteur | — | tous les champs requis présents | RG-ST01-4.5 |
| 12 | Auteur hors liste | Erreur | auteur = « inconnu » | `S` quelconque | lot refusé, état inchangé | RG-ST01-4.5 |
| 13 | Appareil ≠ appareil de la marque | Erreur | w.appareil = A, w.marque.a = B | — | lot refusé, état inchangé | RG-ST01-4.5 |

### SPEC-US-5 : Partir d'un dépôt sain
**Réf :** US-5 ; RG-G3 ; anti-patterns « stockage natif dans le cœur » et « horloge système implicite » de `03_archi.md`

#### Règles métier formalisées

**RG-ST01-5.1 : Cœur sans stockage ni heure système** `@Invariant` `[CLASSIFICATION À VALIDER]`
- **Réf AC :** [US-5/AC-5.1], [US-1/AC-1.7]
- **Pré-conditions :** un état du dépôt.
- **Post-conditions :** le graphe de dépendances du cœur, transitif, ne contient aucune bibliothèque de stockage et n'active aucune lecture de l'heure système.
- **Expression formelle :** `deps*(arbitre-core) ∩ Interdites_cœur = ∅`, avec au minimum `Interdites_cœur ⊇ { rusqlite, chrono avec la feature clock }`. La ligne de commande (`arbitre`) n'est pas visée.
- **Cas d'erreur :**
  - Dépendance interdite réintroduite → vérification rouge, message qui nomme le cœur et la dépendance.
- *Classification à valider : règle sans état sur un instantané du dépôt (`@Invariant` de dépôt), et non règle du domaine.*

**RG-ST01-5.2 : Dépendances communes déclarées une seule fois** `@Invariant` `[CLASSIFICATION À VALIDER]`
- **Réf AC :** [US-5/AC-5.3] ; [US-8/AC-8.3]
- **Pré-conditions :** un état du dépôt.
- **Post-conditions :** une dépendance utilisée par plusieurs composants a sa version au seul niveau du workspace ; les composants s'y réfèrent sans répéter la version.
- **Expression formelle :** `∀ d, |{ k composant | d ∈ deps_directes(k) }| ≥ 2 ⇒ d ∈ dépendances_workspace ∧ ∀ k, version(d, k) héritée du workspace`.
- **Cas d'erreur :**
  - Version répétée dans un composant → vérification rouge, message qui nomme le composant et la dépendance [DÉRIVÉE] (AC-5.3 ne dit pas qu'elle est vérifiée automatiquement).

**Contrainte C-5.3 : Code de démonstration remplacé** — non classée
- **Réf AC :** [US-5/AC-5.4]
- **Expression :** la fonction `add` du cœur et le « Hello, world! » de la ligne de commande n'existent plus. Vérifiable par relecture.

#### Matrice de cas — US-5

| # | Cas | Type | Input | État | Résultat attendu | Réf règle |
|---|-----|------|-------|------|-------------------|-----------|
| 1 | Story suivante ouverte (cas nominal US-5) | Nominal | dépôt après ST-01 | — | le cœur se construit pour l'ordinateur et pour le navigateur, aucune alerte | RG-ST01-5.1, RG-ST01-6.3 |
| 2 | `rusqlite` réintroduit dans le cœur | Erreur | modification proposée | — | vérification rouge, nomme `arbitre-core` et `rusqlite` | RG-ST01-5.1 |
| 3 | `chrono/clock` réintroduit par une dépendance transitive | Erreur | dépendance qui active `clock` | — | vérification rouge | RG-ST01-5.1 |
| 4 | La ligne de commande lit l'heure système | Nominal | `arbitre` dépend de `chrono/clock` | — | vérification verte | RG-ST01-5.1 |
| 5 | Version répétée dans un composant | Erreur | `serde = "1.0.x"` dans un `Cargo.toml` de composant | `serde` au workspace | vérification rouge | RG-ST01-5.2 |

### SPEC-US-6 : Faire vérifier chaque modification proposée automatiquement
**Réf :** US-6 ; RG-G2, RG-G9, INV-22 ; arbitrage 2026-11-03 « CI » ; règle de collision « Configuration CI » du plan

#### Règles métier formalisées

**RG-ST01-6.1 : Verdict unique, en local comme sur l'hébergeur** `@Invariant` [ARB]
- **Réf AC :** [US-6/AC-6.1] ; arbitrage « CI »
- **Pré-conditions :** un état du dépôt et une modification proposée (base, tête).
- **Post-conditions :** la vérification passe par le seul point d'entrée `scripts/ci.sh`. Le verdict est `rouge` si au moins un contrôle échoue, `vert` sinon. Il est le même en local et sur GitHub Actions pour les mêmes entrées et les mêmes versions d'outils.
- **Expression formelle :** `verdict = vert ⇔ ∀ contrôle k ∈ Contrôles, k(base, tête) = succès`, avec `Contrôles ⊇ {tests, navigateur, arêtes, secrets, mutation}` ; `verdict_local(e) = verdict_CI(e)`.
- **Cas d'erreur :**
  - Un test échoue → `rouge` (AC-6.1).
  - Outil de contrôle absent ou en échec technique → `rouge`, jamais `vert` par défaut [DÉRIVÉE].
  - Versions d'outils différentes entre local et CI → voir `MANQUE-ST01-6`.

**RG-ST01-6.2 : Construction pour le navigateur sur la liste nommée** `@Invariant`
- **Réf AC :** [US-6/AC-6.2], [US-5/AC-5.2]
- **Pré-conditions :** liste navigateur `{arbitre-core, arbitre-plan}`.
- **Post-conditions :** chaque composant présent de la liste se construit pour la cible navigateur. Un composant de la liste encore absent est ignoré, sans échec.
- **Expression formelle :** `∀ k ∈ liste ∩ composants_présents, construit(k, wasm32-unknown-unknown)` ; `k ∈ liste \ composants_présents ⇒ ignoré`.
- **Cas d'erreur :**
  - Composant présent qui ne se construit pas → `rouge`, message qui nomme le composant.
  - `arbitre-core` absent ou renommé → voir `AMB-ST01-5`.

**RG-ST01-6.3 : Arêtes interdites entre composants** `@Invariant`
- **Réf AC :** [US-6/AC-6.3], [US-6/AC-6.4] ; RG-G2, INV-22
- **Pré-conditions :** graphe des dépendances du workspace ; règles écrites par nom pour tous les composants prévus, présents ou non.
- **Post-conditions :** aucune arête interdite, directe ou transitive. Une règle sur un composant absent est sans effet tant qu'il n'existe pas, et s'applique dès sa naissance sans retoucher la vérification.
- **Expression formelle :**
  - `arbitre-plan ↛* arbitre-diff` et `arbitre-diff ↛* arbitre-plan` (RG-G2) ;
  - `∀ k ∈ {arbitre-core, arbitre-plan}, ∀ d ∈ deps*(k), construit(d, wasm32-unknown-unknown)` (tenu par RG-ST01-6.2) ;
  - [DÉRIVÉE, archi § Synthèse 3] `arbitre-core ↛ aucun autre composant` ; `arbitre-plan` et `arbitre-diff` ne dépendent d'aucun composant autre que `arbitre-core`.
  - `↛*` : ni directement, ni par une chaîne de dépendances (voir `AMB-ST01-6`).
- **Cas d'erreur :**
  - Arête interdite introduite → `rouge`, message qui nomme les deux composants (et la chaîne, si transitive).

**RG-ST01-6.4 : Détection des secrets** `@Invariant` [ARB]
- **Réf AC :** [US-6/AC-6.5] ; RG-G9
- **Pré-conditions :** une modification proposée.
- **Post-conditions :** chaque commit de la modification proposée est inspecté, pas seulement la tête : un secret ajouté puis retiré reste public dans l'historique.
- **Expression formelle :** `∃ commit ∈ modification, ∃ secret détecté ⇒ rouge`.
- **Cas d'erreur :**
  - Secret détecté → `rouge`, message qui nomme le fichier et le commit, sans afficher la valeur [DÉRIVÉE].
  - Faux positif → voir `MANQUE-ST01-4`.

**RG-ST01-6.5 : Vérifications ajoutées sans retoucher celles de ST-01** `@Invariant`
- **Réf AC :** [US-6/AC-6.6] ; règle de collision « Configuration CI »
- **Pré-conditions :** une story ultérieure ajoute un contrôle.
- **Post-conditions :** le contrôle s'ajoute dans un nouveau fichier ; les contrôles de ST-01 et le point d'entrée restent inchangés, et le nouveau contrôle compte dans le verdict.
- **Expression formelle :** `Contrôles(après) = Contrôles(ST-01) ∪ Nouveaux`, avec les fichiers de `Contrôles(ST-01)` inchangés. Articulation avec le script unique : `CONFLIT-ST01-2`.
- **Cas d'erreur :**
  - Un contrôle ajouté échoue → `rouge`, comme tout contrôle.

**Contrainte C-6.6 : Hébergement** — non classée [ARB]
- **Réf AC :** arbitrage « CI » ; [HYP] du cahier
- **Expression :** dépôt public sur GitHub sous le compte `quentinhaenn` ; protection contre l'envoi de secrets active avant le premier envoi (G-1) ; GitHub Actions appelle `scripts/ci.sh` sur chaque modification proposée. Vérifiable par constat.

#### Matrice de cas — US-6

| # | Cas | Type | Input | État | Résultat attendu | Réf règle |
|---|-----|------|-------|------|-------------------|-----------|
| 1 | Modification propre | Nominal | PR conforme | — | `vert` | RG-ST01-6.1 |
| 2 | Test en échec | Erreur | PR avec un test rouge | — | `rouge` | RG-ST01-6.1 |
| 3 | Bibliothèque réseau ajoutée au cœur (cas nominal US-6) | Erreur | PR | — | `rouge` sur la construction navigateur ou sur RG-ST01-5.1 | RG-ST01-6.2, RG-ST01-5.1 |
| 4 | `arbitre-plan` absent | Edge | PR ST-01 | seul `arbitre-core` existe | contrôle navigateur vert, `arbitre-plan` ignoré | RG-ST01-6.2 |
| 5 | Arête plan → diff (test volontaire) | Erreur | PR de test | les deux composants existent | `rouge`, message nommant `arbitre-plan` et `arbitre-diff` | RG-ST01-6.3 |
| 6 | Arête transitive plan → X → diff | Erreur | PR de test | — | `rouge`, chaîne nommée | RG-ST01-6.3 |
| 7 | `arbitre-diff` naît plus tard | Edge | PR qui crée `arbitre-diff` | règles déjà écrites | règles appliquées sans modifier la vérification | RG-ST01-6.3 |
| 8 | Secret dans un commit intermédiaire | Erreur | commit 1 ajoute un jeton, commit 2 le retire | — | `rouge` | RG-ST01-6.4 |
| 9 | Documentation seule | Edge | PR qui ne touche que `.md` | — | contrôles exécutés, `vert` ; mutation neutre | RG-ST01-6.1, RG-ST01-7.3 |
| 10 | Contrôle ajouté par une story ultérieure | Nominal | nouveau fichier de contrôle | — | exécuté par le point d'entrée, compté dans le verdict, fichiers ST-01 inchangés | RG-ST01-6.5 |
| 11 | Outil absent sur la machine | Erreur | `scripts/ci.sh` lancé sans l'outil de mutation | — | `rouge` explicite | RG-ST01-6.1 |
| 12 | Même tête, local puis CI | Edge | même PR | mêmes versions d'outils | même verdict | RG-ST01-6.1 |

### SPEC-US-7 : Mesurer si les tests attrapent vraiment les erreurs
**Réf :** US-7 ; F-16, F-36 ; arbitrages 2026-11-03 « Zones qui se recouvrent » et « Score de mutation sous le seuil »

#### Règles métier formalisées

**RG-ST01-7.1 : Périmètre de la mesure** `@Invariant`
- **Réf AC :** [US-7/AC-7.1]
- **Pré-conditions :** une modification proposée (base, tête).
- **Post-conditions :** seuls comptent les mutants situés dans le code modifié par la modification proposée.
- **Expression formelle :** `Mutants_mesurés = { μ | emplacement(μ) ∈ diff(base, tête) }`.
- **Cas d'erreur :**
  - Aucun code modifié (documentation, configuration) → aucune mesure, résultat neutre, pas d'échec.

**RG-ST01-7.2 : Zone d'un fichier** `@Invariant` [ARB]
- **Réf AC :** [US-7/AC-7.2], [US-7/AC-7.3] ; F-36 ; arbitrage « Zones qui se recouvrent »
- **Pré-conditions :** les déclarations de zones `.ctdd/stories/*/zones.toml` présentes dans la tête ; un fichier `p`.
- **Post-conditions :** la zone d'un fichier est la plus exigeante parmi toutes les zones dont un motif le couvre, quel que soit l'ordre des motifs ; un fichier qu'aucun motif ne couvre prend `Full`.
- **Expression formelle :** `zones(p) = { z | ∃ motif → z déclaré, p ∈ motif }` ; `zone(p) = plus_exigeante(zones(p))` si `zones(p) ≠ ∅`, sinon `Full` ; `seuil(Full) = 85 %`, `seuil(Std) = 70 %` ; `Full` plus exigeante que `Std`.
- **Cas d'erreur :**
  - Aucun fichier `zones.toml` → tous les fichiers en `Full`.
  - Fichier `zones.toml` illisible ou zone inconnue → `rouge`, message qui nomme le fichier `[À DÉFINIR]` (proposition ; seuil Full si Quentin préfère ne pas bloquer).
  - Zone `Gen` → voir `MANQUE-ST01-1`.
  - Déclarations lues (celles de la story seule, ou toutes) → voir `AMB-ST01-4` ; la proposition retenue ici est « toutes ».

**RG-ST01-7.3 : Score et verdict de mutation** `@Invariant` [ARB]
- **Réf AC :** [US-7/AC-7.4], [US-7/AC-7.5] ; arbitrage « Score de mutation sous le seuil » ; [HYP] mesure fichier par fichier
- **Pré-conditions :** les mutants mesurés d'un fichier `p` et leur résultat ; la liste des mutants équivalents justifiés.
- **Post-conditions :** un score sous le seuil de la zone rend la vérification rouge et bloque la fusion. Le rapport nomme le fichier, son score et le seuil attendu. Un mutant équivalent n'est exclu que s'il est listé avec une justification.
- **Expression formelle :**
  - `comptés(p) = mesurés(p) \ (équivalents_justifiés ∪ non_construisibles)`
  - `score(p) = |détectés(p) ∩ comptés(p)| / |comptés(p)|`, défini si `|comptés(p)| > 0` ; sinon neutre.
  - `∃ p, score(p) < seuil(zone(p)) ⇒ rouge`, rapport `(p, score(p), seuil(zone(p)))` trié par `p`.
  - Un score exactement égal au seuil passe.
- **Cas d'erreur :**
  - Entrée de la liste des équivalents sans justification → entrée invalide, `rouge` [DÉRIVÉE].
  - Mutant qui fait dépasser le temps imparti aux tests → `[À DÉFINIR]`, voir `MANQUE-ST01-8`.
  - Emplacement de la liste des équivalents → voir `MANQUE-ST01-5`.

**Contrainte C-7.4 : Résultat lisible par la relecture** — non classée
- **Réf AC :** [US-7/AC-7.6]
- **Expression :** le résultat (par fichier : zone, score, seuil, verdict) est conservé avec la vérification de la modification proposée, sous une forme lisible par la relecture d'une story sans outil supplémentaire. Vérifiable par constat.

**Critère de complétion K-7.5 : ST-01 se mesure elle-même** — non classé
- **Réf AC :** [US-7/AC-7.5] ; plan § ST-01
- **Expression :** la mesure tourne sur la modification proposée de ST-01 et rend `score(p) ≥ 85 %` pour chaque fichier du moteur de fusion (US-1 à US-3) et de l'horloge (US-4).

#### Matrice de cas — US-7

| # | Cas | Type | Input | État | Résultat attendu | Réf règle |
|---|-----|------|-------|------|-------------------|-----------|
| 1 | Fichier du cœur sous-testé (cas nominal US-7) | Erreur | score 60 % | fichier non déclaré | `rouge` ; rapport (fichier, 60 %, 85 %) | RG-ST01-7.2, RG-ST01-7.3 |
| 2 | Fichier `Std` à 72 % | Nominal | score 72 % | motif → `Std` | `vert` | RG-ST01-7.2, RG-ST01-7.3 |
| 3 | Fichier couvert par `Std` et `Full` | Edge | score 80 % | deux motifs, `Std` listé d'abord | zone `Full`, `rouge` | RG-ST01-7.2 [ARB] |
| 4 | Aucun `zones.toml` | Edge | score 80 % | — | `Full`, `rouge` | RG-ST01-7.2 |
| 5 | PR sans code modifié | Edge | documentation seule | — | neutre, pas d'échec | RG-ST01-7.1 |
| 6 | Score égal au seuil | Edge | score 85 % | `Full` | `vert` | RG-ST01-7.3 |
| 7 | Mutant équivalent listé avec justification | Edge | 17/20 détectés, 1 équivalent justifié | `Full` | 17/19 = 89,5 %, `vert` | RG-ST01-7.3 [ARB] |
| 8 | Mutant équivalent listé sans justification | Erreur | entrée sans justification | — | `rouge` | RG-ST01-7.3 |
| 9 | Mutants hors du code modifié | Edge | fichier modifié en une fonction | autres fonctions sous-testées | seuls les mutants de la fonction modifiée comptent | RG-ST01-7.1 |
| 10 | `zones.toml` mal formé | Erreur | syntaxe invalide | — | `rouge` (proposition) | RG-ST01-7.2 |
| 11 | Tous les mutants non construisibles | Edge | `comptés = ∅` | — | neutre | RG-ST01-7.3 |
| 12 | PR de ST-01 | Nominal | moteur et horloge | `Full` | ≥ 85 % par fichier | K-7.5 |

### SPEC-US-8 : Choisir une fois les dépendances communes, et les consigner
**Réf :** US-8 ; PL-6, F-17 ; règle de collision `Cargo.lock` du plan

#### Règles métier formalisées

**Contrainte C-8.1 : Document de décision** — non classée
- **Réf AC :** [US-8/AC-8.1], [US-8/AC-8.2]
- **Expression :** `docs/decisions/ST-01-dependances.md` existe et couvre au moins six sujets : exécution asynchrone, appels réseau sortants, trousseau du système, sérialisation, tests par propriétés, mesure de mutation. Pour chacun : choix, raison, option écartée. Par défaut, il reprend PL-6 : `tokio`, `reqwest` avec `rustls` (sans OpenSSL ; côté navigateur, `fetch` derrière le port `Transport`), `keyring` (repli fichier `0600`), `serde` / `serde_json`, `proptest`, `cargo-mutants` (outil, hors `Cargo.toml`). Tout écart à PL-6 porte une justification écrite. Vérifiable par relecture.

**RG-ST01-8.2 : Toutes les dépendances communes déclarées au workspace** `@Invariant` `[CLASSIFICATION À VALIDER]`
- **Réf AC :** [US-8/AC-8.3] ; PL-6
- **Pré-conditions :** la liste des dépendances communes du document C-8.1 (hors outils).
- **Post-conditions :** chacune est déclarée au niveau du workspace, y compris si aucun composant ne l'utilise encore.
- **Expression formelle :** `∀ d ∈ Communes(C-8.1), d ∈ dépendances_workspace`.
- **Cas d'erreur :**
  - Dépendance commune absente du workspace → écart détectable par comparaison avec le document ; contrôle automatique `[À DÉFINIR]` (proposition : relecture, la liste étant gelée par amendement du socle).
  - Story ultérieure qui a besoin d'une dépendance commune absente → amendement du socle, hors story.

**Contrainte C-8.3 : Règle du fichier de verrouillage** — non classée
- **Réf AC :** [US-8/AC-8.4]
- **Expression :** le guide de contribution écrit : en cas de conflit sur `Cargo.lock`, ne jamais le fusionner à la main ; reprendre celui de la branche principale, puis le régénérer. Vérifiable par relecture.

#### Matrice de cas — US-8

| # | Cas | Type | Input | État | Résultat attendu | Réf règle |
|---|-----|------|-------|------|-------------------|-----------|
| 1 | Voie planning qui a besoin du réseau (cas nominal US-8) | Nominal | composant qui utilise `reqwest` | déclaré au workspace | utilisation sans nouvelle décision ; version héritée | RG-ST01-8.2, RG-ST01-5.2 |
| 2 | Dépendance commune inutilisée | Edge | `keyring` | aucun composant ne l'utilise | déclarée au workspace | RG-ST01-8.2 |
| 3 | Choix différent de PL-6 | Edge | autre runtime | — | justification écrite dans le document | C-8.1 |
| 4 | Sujet manquant dans le document | Erreur | document sans « trousseau » | — | relecture refusée | C-8.1 |
| 5 | Conflit sur `Cargo.lock` entre deux voies | Edge | deux branches | — | règle du guide appliquée | C-8.3 |
| 6 | Nouvelle dépendance commune en story ultérieure | Edge | besoin non couvert | — | amendement du socle | RG-ST01-8.2 |

## Dépendances identifiées

| Dépendance | Type | Description | Impact |
|-----------|------|-------------|--------|
| G-0 : retrait de `crates/core/.git` | Interne (geste) | Dépôt imbriqué à supprimer avant le premier commit | Bloquant (US-5, US-6) |
| G-1 : protection contre l'envoi de secrets | Externe (GitHub) | À activer avant le premier envoi | Bloquant pour C-6.6 |
| Dépôt public GitHub + Actions | Externe | Exécute `scripts/ci.sh` sur chaque modification proposée [ARB] | Bloquant pour US-6, US-7 sur l'hébergeur ; non bloquant en local |
| Source d'heure (port `Clock`) | Interne | Seule la lecture de l'heure courante sert à ST-01 ; le fuseau du port de l'archi n'est pas requis ici | Bloquant pour US-4 |
| Unicité des identifiants d'appareil | Interne (ST-02) | Le départage par appareil suppose deux appareils jamais identiques | Non bloquant pour ST-01 (identifiants fournis en entrée) |
| Ordre total sur les identifiants de données et d'appareils | Interne (archi ST-01) | Requis par RG-ST01-2.2 et RG-ST01-4.1 | Bloquant |
| Information de connaissance par écriture | Interne | Requise pour évaluer `a_vu` (RG-ST01-2.1) ; forme selon `CONFLIT-ST01-1` | **Bloquant pour US-2** |
| Journal par champ (`Store::journal`) | Interne (ST-02) | La détection des conflits porte sur toutes les écritures connues d'une cible, pas seulement sur la gagnante. ST-01 les reçoit en entrée | Non bloquant pour ST-01 ; contrainte pour ST-02 |
| Bibliothèque de tests par propriétés | Externe | E-1.1 (PL-6 : `proptest`) | Bloquant |
| Outil de mutation | Externe | RG-ST01-7.x (PL-6 : `cargo-mutants`, mode « modification seule ») | Bloquant pour US-7 |
| Outil de détection de secrets | Externe | RG-ST01-6.4 (plan : gitleaks) | Bloquant pour US-6 |
| Cible `wasm32-unknown-unknown` | Externe | RG-ST01-6.2 | Bloquant pour US-5, US-6 |
| Lecture du graphe de dépendances | Externe (Cargo) | RG-ST01-5.1, 5.2, 6.3 | Bloquant |

## Conflits et ambiguïtés

### Conflits

| # | Règles en conflit | Description | Résolution proposée |
|---|-------------------|-------------|---------------------|
| [CONFLIT-ST01-1] | AC-2.4 vs AC-2.3 et AC-2.2 (RG-ST01-2.1) | Le point de dernière réception tenu par l'**appareil récepteur** dit ce que celui-ci a reçu du pair, pas ce que le pair avait vu au moment d'écrire. **Contre-exemple** : A écrit f=v1 (10,0,A) et l'envoie. B la reçoit et écrit f=v2 (11,0,B). A n'avait reçu de B que jusqu'à (5,0,B). À la réception de v2, A voit f modifié localement (10) après son point de réception de B (5) : la règle « modifié des deux côtés depuis la dernière synchronisation », évaluée avec ce point, signale un conflit, alors que c'est l'écrasement séquentiel d'AC-2.3 (cas nominal 2 d'US-2). La marque seule ne tranche pas non plus : `a_vu(w, x) ⇒ marque(w) > marque(x)`, mais la réciproque est fausse. | **À trancher par Quentin.** Options : **(a)** chaque écriture porte `vu`, une seule marque : la plus récente que son auteur connaissait sur la même cible (et sur les suppressions de la donnée). `a_vu(w, x) ⇔ vu(w) ≥ marque(x)`. Ce n'est pas un vecteur de versions : une marque par écriture, quel que soit le nombre d'appareils. Les conflits deviennent une fonction de l'ensemble des écritures, donc RG-2.2 tient aussi pour eux. Les points de réception restent l'outil de la synchronisation (ST-02, ST-19), plus celui de la détection. Cela ajoute un champ au tuple d'écriture de l'archi. **(b)** Chaque lot porte les points de réception **de son auteur** : sans champ par écriture, mais une écriture faite avant une réception et envoyée après passe pour séquentielle (conflit manqué côté récepteur ; l'auteur le signale de son côté). **(c)** Garder AC-2.4 tel quel : faux conflits sur les écrasements séquentiels, ce qui viole AC-2.3. Recommandation : (a). |
| [CONFLIT-ST01-2] | Arbitrage « CI » (un seul script `scripts/ci.sh`) vs AC-6.6 (vérifications ajoutées dans de nouveaux fichiers, sans modifier celles de ST-01) | Un script unique qu'on doit modifier pour chaque vérification nouvelle contredit AC-6.6 et recrée la collision « Configuration CI » entre voies. | `scripts/ci.sh` reste le point d'entrée unique ; il exécute aussi toute vérification déposée à un emplacement convenu (un fichier par vérification), sans que son propre contenu change. Les contrôles de ST-01 ne sont pas modifiés par les stories suivantes. |
| [CONFLIT-ST01-3] | Arbitrage « Horloge en avance » (« signale l'appareil concerné ») vs RG-ST01-4.3 (l'horloge adopte une marque reçue) | Un appareil à l'heure, qui a reçu une marque en avance, émet ensuite des marques en avance. La détection le nomme alors aussi, alors que la cause est ailleurs. La marque seule ne permet pas de distinguer l'appareil fautif. | Signaler l'appareil auteur de chaque écriture en avance au-delà du seuil (RG-ST01-4.6), et documenter qu'un appareil peut être nommé par héritage : le signalement reste informatif et ne change rien à la fusion. Alternative, si Quentin veut le fautif exact : chaque écriture porte aussi l'heure physique de son auteur, au prix d'un champ de plus. |

### Ambiguïtés

| # | Règles concernées | Description | Résolution proposée |
|---|-------------------|-------------|---------------------|
| [AMB-ST01-1] | AC-3.4 (« la version modifiée reste récupérable ») vs RG-2.4 (partition retenues / écartées) | Quand la suppression gagne, les gagnantes des champs sont-elles des versions écartées ? Les y mettre rendrait les écartées non monotones : une résurrection devrait les en retirer. | Les gagnantes des champs d'une donnée supprimée restent dans l'état projeté, masquées par le statut `supprimée` ; elles sont rendues par le moteur, donc récupérables, et réapparaissent à la résurrection (edge US-3). Elles ne vont pas aux écartées. |
| [AMB-ST01-2] | AC-2.2 (« chaque conflit ») | Avec trois écritures concurrentes ou plus sur un champ, faut-il un conflit par paire, ou un seul conflit par champ ? | Un conflit par paire concurrente (RG-ST01-2.1). Cela garde les conflits comme une fonction de l'ensemble des écritures. Le regroupement par champ relève de l'affichage (stories de surfaces). |
| [AMB-ST01-3] | AC-2.2 (« valeur retenue ») | « Retenue » = la plus récente de la paire, ou la valeur finale du champ, qui peut venir d'une troisième écriture plus tardive ? | Retenue et écartée **de la paire** ; la valeur finale est dans l'état projeté. |
| [AMB-ST01-4] | AC-7.2 vs AC-7.3 | Pour une PR de la story ST-xx, faut-il lire seulement `ST-xx/zones.toml`, ou toutes les déclarations ? Un fichier déclaré `Std` par ST-05 et modifié par ST-09 sans déclaration propre prendrait sinon `Full`. | Lire toutes les déclarations présentes dans la tête ; le plus exigeant gagne (même règle que l'arbitrage sur les recouvrements). La CI n'a pas à savoir à quelle story appartient la PR. |
| [AMB-ST01-5] | AC-6.2 (« un composant absent est ignoré ») | Un nom mal orthographié ou un composant renommé serait ignoré en silence, et la frontière navigateur ne serait plus vérifiée. | Les noms de la liste sont ceux de l'archi, à l'identique ; `arbitre-core` doit exister, sinon `rouge`. Seuls les composants pas encore nés (`arbitre-plan` en ST-01) sont ignorés. |
| [AMB-ST01-6] | AC-6.3 (« ne dépend jamais ») | Directement seulement, ou aussi par une chaîne de dépendances ? | Transitif : `plan → X → diff` est interdit. |

### Manques

| # | Concerne | Description | Résolution proposée |
|---|----------|-------------|---------------------|
| [MANQUE-ST01-1] | AC-7.2 vs plan (zone `Gen` de ST-02, ST-05…) | AC-7.2 ne donne que `Full` et `Std`. Le seuil d'un fichier déclaré `Gen` n'est pas défini. | **À trancher par Quentin.** Proposition : pas de seuil de mutation en `Gen` (le plan y exige des tests de contrat ou de propriétés, pas une mutation), ce qui doit être explicite dans le script de seuil. À défaut, `Gen` prend le seuil `Std`. |
| [MANQUE-ST01-2] | RG-2.1 | Deux écritures de même identité et de valeurs différentes (appareil réinstallé avec le même identifiant et une horloge remise à zéro, données altérées). | Départage déterministe par la valeur (ordre fixe sur les valeurs sérialisées) pour garder la convergence, et anomalie rendue par le moteur. À valider. |
| [MANQUE-ST01-3] | RG-ST01-4.2 | Le compteur `c` a une borne. Que faire s'il l'atteint (heure figée et très nombreuses émissions) ? | Borne assez large pour être hors d'atteinte en usage ; en cas de dépassement, erreur explicite et aucune marque émise, jamais de retour à zéro. |
| [MANQUE-ST01-4] | RG-ST01-6.4 | Procédure en cas de faux positif du détecteur de secrets. | Liste d'exceptions versionnée, une justification par entrée, visible dans la PR (même principe que les mutants équivalents). |
| [MANQUE-ST01-5] | RG-ST01-7.3 | Emplacement et format de la liste des mutants équivalents. | Versionnée dans le dépôt, à côté de `zones.toml` de la story, avec mutant, fichier et justification par entrée ; forme fixée à l'archi locale. |
| [MANQUE-ST01-6] | RG-ST01-6.1 | Le même verdict en local et en CI suppose les mêmes versions d'outils (outil de mutation, détecteur de secrets, chaîne de compilation). | Versions d'outils figées et lues par `scripts/ci.sh`. |
| [MANQUE-ST01-7] | E-1.1 | Nombre exact de jeux par propriété. | 1 000 par propriété en zone Full, à l'exécution courante. |
| [MANQUE-ST01-8] | RG-ST01-7.3 | Mutant qui fait boucler les tests (dépassement de temps). | Compté comme détecté (les tests ne passent pas sur lui), avec un temps limite fixé par l'outil. |

## Invariants candidats (synthèse C-TDD)

### @Invariant (stateless, bloquant)
- `INV-4` : `gagnante(S, d, f) = max_< { w ∈ S | w.cible = (d, f) }`, champs indépendants — source : RG-2.1
- `INV-5` : `R(S ∪ A ∪ B)` ne dépend ni de l'ordre, ni du regroupement, ni des doublons ; lot vide neutre ; seconde réception sans nouvelle écartée ni nouveau conflit — source : RG-2.2
- `INV-6` : `supprimée(S, d) ⇔ t_s(S, d) > t_f(S, d)`, résurrection avec les gagnantes de tous les champs — source : RG-2.3
- `INV-7` : `retenues(S) ⊎ écartées(S) = S`, `écartées` monotone — source : RG-2.4
- `INV-ST01-1` : `fusion_{env₁}(S, L) = fusion_{env₂}(S, L)` pour tout environnement — source : RG-ST01-1.1
- `INV-ST01-2` : `conflit(w, x) ⇔ concurrentes ∧ exclusives ∧ valeurs différentes`, jamais deux suppressions — source : RG-ST01-2.1 (évaluation de `a_vu` selon `CONFLIT-ST01-1`)
- `INV-ST01-3` : écartées, conflits et avances triés selon un ordre total fixe — source : RG-ST01-2.2
- `INV-ST01-4` : ordre lexicographique total sur `(l, c, a)` — source : RG-ST01-4.1
- `INV-ST01-5` : émission strictement supérieure à tout ce qui a été émis ou reçu, `l ≥ pt`, pour toute suite d'heures physiques — source : RG-ST01-4.2
- `INV-ST01-6` : après réception de `m`, toute émission > `m` — source : RG-ST01-4.3
- `INV-ST01-7` : lot avec une écriture invalide → refusé en entier, état inchangé — source : RG-ST01-4.5
- `INV-ST01-8` : `avances(L, maintenant, σ)` signale `> σ` seulement, sans effet sur la fusion — source : RG-ST01-4.6
- `INV-ST01-9` : `deps*(arbitre-core) ∩ Interdites_cœur = ∅` [CLASSIFICATION À VALIDER] — source : RG-ST01-5.1
- `INV-ST01-10` : version d'une dépendance partagée au seul workspace ; toutes les dépendances communes y sont déclarées [CLASSIFICATION À VALIDER] — source : RG-ST01-5.2, RG-ST01-8.2
- `INV-ST01-11` : `verdict = vert ⇔ tous les contrôles réussissent` ; outil en échec → `rouge` — source : RG-ST01-6.1
- `INV-ST01-12` : composants présents de la liste navigateur construits pour `wasm32-unknown-unknown` — source : RG-ST01-6.2
- `INV-ST01-13` : `arbitre-plan ↮* arbitre-diff`, règles par nom pour tous les composants prévus — source : RG-ST01-6.3 (RG-G2, INV-22)
- `INV-ST01-14` : secret dans un commit de la PR → `rouge` — source : RG-ST01-6.4
- `INV-ST01-15` : `zone(p)` = la plus exigeante des zones qui le couvrent, sinon `Full` — source : RG-ST01-7.2
- `INV-ST01-16` : `∃ p, score(p) < seuil(zone(p)) ⇒ rouge`, score sur les seuls mutants du code modifié, hors équivalents justifiés — source : RG-ST01-7.1, RG-ST01-7.3

### @Property (stateful, nécessite simulateur)
- `PROP-ST01-1` : marques strictement croissantes avec une source figée ou qui recule ; correction faite sur un appareil en retard, après réception, toujours gagnante — simulateur : source d'heure commandable par appareil (`avancer`, `reculer`, `figer`, `fixer`) — source : RG-ST01-4.4
- `PROP-ST01-2` : `N` appareils simulés (N ≥ 3), qui émettent et échangent des lots dans des ordres aléatoires avec des horloges décalées, finissent avec le même état projeté, les mêmes écartées et les mêmes conflits ; un écrasement fait après réception n'est jamais un conflit — simulateur : appareils simulés (source d'heure + état d'horloge + journal connu) et réseau simulé qui livre, retarde, duplique et réordonne les lots — source : RG-2.2, RG-ST01-2.1, RG-ST01-4.3 [DÉRIVÉE : rend vérifiables le cas « trois appareils » d'US-2 et le cas nominal d'US-4 de bout en bout]

## Arbitrages de Quentin (2026-11-03)

- **CONFLIT-ST01-1 → option (a).** Chaque écriture porte `vu`, la plus grande marque que son auteur connaissait sur la même cible et sur les suppressions de la donnée (`⊥` sinon). `a_vu(w, x) ⇔ vu(w) ≥ marque(x)`. Le tuple d'écriture de l'archi globale gagne ce champ. Limite acceptée : une écriture faite après avoir vu seulement la gagnante d'une paire concurrente passe pour avoir vu la perdante aussi ; le conflit de la paire reste signalé, donc aucune perte silencieuse.
- **CONFLIT-ST01-2 → option (a).** `scripts/ci.sh` reste le point d'entrée unique et exécute, en plus de ses contrôles, chaque fichier de `scripts/ci.d/` par ordre alphabétique (`bash <fichier>`, un fichier par vérification) ; un échec → `rouge`. Son contenu ne change plus d'une story à l'autre. Il affiche la liste des vérifications exécutées. Sous GitHub Actions, chaque vérification est une section repliable (`::group::`), une ligne verdict/durée dans `$GITHUB_STEP_SUMMARY`, et un échec émet `::error::`. Un job par vérification (matrice générée depuis `scripts/ci.d/`) seulement si les logs deviennent illisibles ; pas de steps écrits à la main dans le workflow.
- **MANQUE-ST01-1 → option (a).** Pas de seuil de mutation bloquant en `Gen` : score calculé et affiché dans le récap CI à titre informatif ; la review de story vérifie la présence des tests de contrat ou de propriétés prévus par le plan. Le script de seuil traite `Gen` explicitement. Garde-fou : un fichier déclaré `Gen` dans `arbitre-core` ou `arbitre-plan` → `rouge`.
- **Validées en bloc, telles que proposées :** CONFLIT-ST01-3 (signalement informatif, appareil nommé par héritage documenté, pas d'heure physique par écriture), AMB-ST01-1 à 6, MANQUE-ST01-2 à 8.
- **DA itération 1, F-1 → `vu` par cible (a'), amende CONFLIT-ST01-1.** Une marque scalaire mélange plusieurs cibles et laisse passer une suppression concurrente d'une modification d'un autre champ (contre-exemple F-1 de `05_da_review.md`). `vu` devient une table `cible → marque` : écriture de champ `f` → entrées `f` et `suppression` ; suppression → une entrée par champ connu de la donnée, plus `suppression`. `a_vu(w, x) ⇔ vu(w)[cible(x)] ≥ marque(x)`, entrée absente = `⊥`. Taille bornée par le nombre de champs de la donnée, pas par le nombre d'appareils. La limite acceptée s'applique cible par cible. Cas bénin accepté : deux suppressions concurrentes suivies d'une recréation par l'un des auteurs ne lèvent pas de conflit (deux suppressions ne sont jamais en conflit, aucune valeur perdue).
