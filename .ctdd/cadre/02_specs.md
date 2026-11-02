# Spécifications Fonctionnelles — OpenArbitre, cadre global (socle, planning, decision diff)

## Traçabilité
- Source : `.ctdd/cadre/01_story.md` (cahier des charges StoryWriter), section « Arbitrages de Quentin (2026-11-02) » incluse et faisant foi ; `.ctdd/cadre/inputs/plan-decision-diff-2026-10-02.md` pour la partie C ; `CADRAGE.md` l'emporte en cas de contradiction.
- Date : 2026-11-02
- Statut : DRAFT
- Portée : cadre global, cycle sans code. Les questions « renvoyées au cycle de chaque story » restent ouvertes, sans option (listées dans `Conflits et ambiguïtés`, lignes `RENVOI-*`).
- Contexte codebase : workspace Rust vide (`crates/core`, `crates/cli`, squelettes). Aucune interface existante à respecter.

### Conventions de ce document
- `RG-Gn` : règle métier globale n° n du cahier (RG-1 à RG-9 du cahier, renommées pour ne pas se confondre avec les règles `RG-x.y` ci-dessous).
- `[ARB]` : la règle découle d'un arbitrage du 2026-11-02.
- `[DÉRIVÉE]` : règle absente du cahier en toutes lettres mais nécessaire pour qu'une AC soit tenable ; à valider.
- `[À DÉFINIR]` : valeur ou critère manquant, à fixer au cycle de la story concernée.
- `[RENVOI]` : question renvoyée au cycle de la story ; aucune option proposée.
- Durées exprimées en minutes. 7 h 30 = 450 min.

## Glossaire du domaine

| Terme | Définition |
|-------|-----------|
| Socle | Partie commune à tous les modules : stockage, synchronisation, surfaces, adaptateur d'IA, mécanisme de modules. |
| Module | Ensemble de fonctions activable indépendamment : `planning` ou `decision diff`. |
| Surface | Point d'accès à une même logique : interface graphique (navigateur, ordinateur ou téléphone), ligne de commande, serveur d'outils pour assistant (MCP). |
| Action | Opération offerte par un module ou le socle, identique sur toutes les surfaces (RG-G3). |
| Catalogue d'actions | Liste des actions exposées par une surface. |
| Appareil | Installation de l'outil sur une machine (ordinateur ou téléphone), avec son propre stockage local. |
| Relais | Service hébergé par l'utilisateur qui recopie les écritures entre appareils, sans pouvoir les lire. |
| Écriture | Changement de la valeur d'un champ d'une donnée, porteur d'une estampille. |
| Estampille | Marque d'ordre associée à chaque écriture d'un champ (« horloge par champ » de l'arbitrage). |
| Suppression datée | Écriture qui marque une donnée comme supprimée, porteuse d'une estampille comme les autres. |
| Fusion | Combinaison de deux états de données en un état unique selon la règle « dernière écriture gagne, champ par champ ». |
| Version écartée | Valeur d'un champ perdue lors d'une fusion ; reste récupérable. |
| Conflit de synchronisation | Deux écritures sur le même champ dont aucune n'a été faite en connaissance de l'autre. |
| Export | Fichier lisible contenant l'intégralité des données de l'utilisateur, tous modules confondus. |
| Fournisseur d'IA | Service ou modèle local que l'outil sollicite pour les fonctions qui exigent de l'IA (Claude, GPT, Mistral, modèle local). |
| Assistant | Agent d'IA de l'utilisateur qui agit via la surface MCP ; auteur « assistant » dans l'historique. |
| Historique | Journal consultable des actions passées (éléments faits, actions de l'assistant, versions écartées). |
| Élément | Entrée de la pile : intitulé, type, échéance éventuelle, durée et son origine, rang, jour planifié éventuel, marque d'urgence, sujet public éventuel, statut. |
| Type d'élément | `geste`, `décision en attente` ou `engagement daté`. |
| Statut d'élément | `actif`, `fait` ou `supprimé`. |
| Pile | Ensemble des éléments actifs, ordonnés par rang en un ordre total. |
| Rang | Position d'un élément dans la pile. |
| Durée proposée / fixée | Durée d'un élément ; « proposée » si calculée par l'outil, « fixée » si saisie ou corrigée par l'utilisateur. |
| Correction | Fixation par l'utilisateur d'une durée différente de la durée proposée. |
| Éléments semblables | Deux éléments de même type dont les intitulés sont proches (RG-7.6). |
| Jour planifié | Jour auquel un élément est affecté. Un élément sans jour planifié est « non planifié ». |
| Jour travaillé | Lundi à vendredi. |
| Agenda en lecture | Agenda dont les occurrences comptent dans la capacité ; il peut y en avoir plusieurs [ARB]. |
| Agenda en écriture | L'unique agenda où l'outil pose ses blocs [ARB]. |
| Occurrence | Événement d'agenda daté (début, fin), récurrences déjà dépliées [ARB]. |
| Réunion comptée | Occurrence qui réduit la capacité (RG-8.1). |
| Événement de l'outil | Occurrence créée par l'outil dans l'agenda en écriture (bloc ou bloc quotidien). |
| Capacité d'un jour | Temps disponible ce jour-là (RG-8.2). |
| Charge d'un jour | Somme des durées des éléments actifs planifiés ce jour-là (RG-8.3). |
| Débordement | Charge > capacité ; l'excès est la différence. |
| Petit geste | Élément de durée < 30 min. |
| Bloc | Événement de l'outil qui réserve le temps d'un élément de durée ≥ 30 min. |
| Bloc quotidien | Événement de l'outil unique par jour qui regroupe les petits gestes. |
| Sujet public | Texte choisi par l'utilisateur pour le titre visible d'un bloc, distinct de l'intitulé. |
| En retard | Élément actif dont l'échéance est passée ou dont le jour planifié tombe après l'échéance (RG-6.3). |
| Urgent | Élément marqué urgent par l'utilisateur, ou dont l'échéance tombe avant le premier jour où il pourrait être placé (RG-13.3). |
| Question | Situation soumise à l'utilisateur (urgence, débordement) avec des choix concrets ; rien n'est déplacé pour la résoudre avant sa réponse. |
| Planification hebdomadaire | Placement des éléments non planifiés sur les jours de la semaine, au premier jour travaillé. |
| Recalcul | Relecture de l'agenda, mise à jour des capacités, prise en compte des blocs déplacés, détection des débordements ; chaque matin. |
| Dépôt | Dépôt de code versionné analysé par le module decision diff. |
| Invariant de dépôt | Règle déclarée par le mainteneur dans le fichier d'invariants du dépôt (nom, description, règle). À ne pas confondre avec le tag `@Invariant` de C-TDD. |
| Fichier d'invariants | Fichier versionné du dépôt qui contient les invariants de dépôt. |
| Famille d'invariant | Forme de règle exprimable en v1 : frontière d'import, usage des types du domaine, appel interdit [ARB]. |
| Portée | Ensemble des fichiers ou modules du dépôt auxquels une règle s'applique. |
| Vérification | Évaluation déterministe, sans IA, des invariants de dépôt sur un état du code. |
| Verdict | Résultat de la vérification : `succès` ou `échec`, accompagné d'un rapport. |
| Violation | Endroit du code (fichier, ligne) qui contredit un invariant de dépôt. |
| Modification proposée | Écart entre une base et une tête de code (une PR). |
| Itération | Version successive de la tête d'une même modification proposée. |
| Décision | Arbitrage porté par une modification proposée, résumé en une phrase et rattaché à des emplacements du code. |
| Origine d'une décision | `structurelle` (extraite sans IA, déterministe) ou `proposée par IA` (optionnelle, jamais bloquante) [ARB]. |
| Statut de couverture | `respecte`, `viole` ou `non couverte` par les invariants de dépôt. |
| État de revue | `à relire`, `acceptée` ou `rejetée`. |
| Fichier de décisions | Fichier versionné du dépôt (par ex. sous `.arbitre/decisions/`) qui conserve les états de revue [ARB]. |
| Identité de décision | Clé stable d'une décision d'une itération à l'autre, calculée sur le contenu (RG-17.1). |
| Invariant candidat | Invariant de dépôt proposé par l'outil, non encore adopté. |
| Preuve | Emplacement où un candidat est respecté. |
| Contre-exemple | Emplacement où un candidat est déjà violé. |

## Spécifications par User Story

### SPEC-US-1 : Garder ses données sur sa machine
**Réf :** US-1, RG-G4, RG-G7 ; arbitrages « Stockage », « Import sur une machine non vide »

#### Règles métier formalisées

**RG-1.1 : Usage complet hors ligne** `@Property`
- **Réf AC :** [US-1/AC-1.1]
- **Pré-conditions :** appareil sans aucun accès réseau.
- **Post-conditions :** l'opération est appliquée et visible à toute lecture ultérieure sur cet appareil.
- **État externe requis :** disponibilité du réseau.
- **Simulateur nécessaire :** simulateur de réseau coupé (toute sortie réseau échoue) sur un appareil.
- **Expression formelle :** `∀ type de donnée D d'un module activé ou du socle, ∀ op ∈ {lire, créer, modifier, supprimer}, réseau = coupé → op(D) réussit ∧ lire(D) reflète op`.
- **Cas d'erreur :**
  - Fonction dont la description exige un service externe (lecture d'agenda, fournisseur d'IA, relais) → elle se dégrade selon sa propre règle (RG-2.6, RG-4.5, RG-8.5, RG-10.6) ; aucune autre fonction n'est affectée.

**RG-1.2 : Aucun envoi à un tiers non choisi** `@Property`
- **Réf AC :** [US-1/AC-1.2], [US-2/AC-2.3]
- **Pré-conditions :** aucune.
- **Post-conditions :** aucune donnée n'a quitté l'appareil vers une destination non autorisée.
- **État externe requis :** agendas connectés, fournisseur d'IA choisi, relais configuré ; déclencheur de chaque envoi.
- **Simulateur nécessaire :** observateur réseau qui enregistre chaque envoi (destination, contenu, action déclenchante).
- **Expression formelle :** `∀ envoi s, destination(s) ∈ {agendas connectés, fournisseur d'IA choisi, relais de l'utilisateur} ∧ (destination(s) ∈ {agenda, fournisseur d'IA} → s est causé par une sollicitation)`. Le relais n'est pas un tiers (AC-2.3) mais ne reçoit que des données illisibles (RG-2.8). Ce qui est écrit dans l'agenda est limité par RG-11.2.
- **Cas d'erreur :**
  - Envoi vers une destination hors liste → violation bloquante.
  - Lecture d'agenda ou pose de bloc lors du recalcul automatique → voir [CONFLIT-8].

**RG-1.3 : Export intégral et réimport sans perte** `@Invariant` `[CLASSIFICATION À VALIDER]`
- **Réf AC :** [US-1/AC-1.3]
- **Pré-conditions :** état de données S quelconque.
- **Post-conditions :** un fichier lisible (format textuel documenté) contient tout S.
- **Expression formelle :** `∀ S, import(∅, export(S)) ≡ S`, où S inclut les données de tous les modules (activés ou non), l'historique, les versions écartées et les estampilles (nécessaires pour que l'import soit une fusion, RG-1.4).
- **Cas d'erreur :**
  - Échec d'écriture du fichier d'export → erreur explicite, aucune donnée modifiée.
- *Classification à valider : la règle porte sur une fonction pure de S, mais sa vérification traverse le stockage.*

**RG-1.4 : Import = fusion** `@Invariant` [ARB]
- **Réf AC :** [US-1/AC-1.3], arbitrage « Import sur une machine non vide »
- **Pré-conditions :** fichier d'export valide ; état local T quelconque (vide ou non).
- **Post-conditions :** l'état local est la fusion de T et du contenu importé.
- **Expression formelle :** `∀ T, S : import(T, export(S)) = fusion(T, S)` (fusion définie en RG-2.1 à RG-2.4). Aucun mode « remplacement ».
- **Cas d'erreur :** voir RG-1.5.

**RG-1.5 : Import atomique** `@Invariant`
- **Réf AC :** [US-1/AC-1.3] edge « fichier abîmé »
- **Pré-conditions :** fichier F présenté à l'import.
- **Post-conditions :** soit F est entièrement fusionné, soit l'état est strictement inchangé.
- **Expression formelle :** `invalide(F) → état_après = état_avant ∧ message(cause)` ; `invalide(F) ⇔ illisible ∨ tronqué ∨ incohérent ∨ format inconnu`.
- **Cas d'erreur :**
  - Fichier illisible, tronqué, incohérent, format inconnu → refus total, message qui nomme la cause.

**RG-1.6 : Durabilité des modifications confirmées** `@Property`
- **Réf AC :** [US-1/AC-1.4]
- **Pré-conditions :** une modification m a été confirmée à l'utilisateur.
- **Post-conditions :** m est présente à la réouverture, quel que soit le mode d'arrêt.
- **État externe requis :** moment de l'arrêt brutal.
- **Simulateur nécessaire :** simulateur d'arrêt brutal à un instant arbitraire (avant, pendant, après confirmation).
- **Expression formelle :** `∀ m, confirmée(m) avant arrêt → présente(m) après réouverture` ; `[DÉRIVÉE] ¬confirmée(m) → m appliquée entièrement ou pas du tout`.
- **Cas d'erreur :**
  - Arrêt pendant une modification non confirmée → aucune donnée existante n'est altérée.

**RG-1.7 : Disque plein** `@Property`
- **Réf AC :** [US-1] edge « disque plein »
- **Pré-conditions :** espace de stockage insuffisant.
- **Post-conditions :** modification refusée, non confirmée, état antérieur intact.
- **État externe requis :** espace disque disponible.
- **Simulateur nécessaire :** simulateur de stockage plein.
- **Expression formelle :** `stockage_plein → op refusée ∧ message explicite ∧ état_après = état_avant`.
- **Cas d'erreur :** c'est le cas d'erreur.

#### Matrice de cas — US-1

| # | Cas | Type | Input | État | Résultat attendu | Réf règle |
|---|-----|------|-------|------|-------------------|-----------|
| 1 | Créer un élément hors ligne | Nominal | création | réseau coupé | élément créé et relu | RG-1.1 |
| 2 | Supprimer hors ligne | Nominal | suppression | réseau coupé | suppression appliquée | RG-1.1 |
| 3 | Rafraîchir l'agenda hors ligne | Edge | recalcul | réseau coupé | capacité « non à jour », reste utilisable | RG-1.1, RG-8.5 |
| 4 | Export puis import sur machine vide | Nominal | export(S) | machine vide | état ≡ S, versions écartées et estampilles comprises | RG-1.3 |
| 5 | Import sur machine non vide | Nominal | export(S) | état T | état = fusion(T, S) | RG-1.4 |
| 6 | Import de fichier tronqué | Erreur | F tronqué | état T | refus, message, état = T | RG-1.5 |
| 7 | Import d'un fichier d'un module désactivé | Edge | export avec données decision diff | decision diff désactivé | données conservées, invisibles | RG-1.4, RG-5.5 |
| 8 | Arrêt brutal après confirmation | Edge | modification confirmée | arrêt forcé | modification présente | RG-1.6 |
| 9 | Arrêt brutal pendant écriture | Edge | modification en cours | arrêt forcé | appliquée entièrement ou pas du tout | RG-1.6 |
| 10 | Disque plein | Erreur | modification | stockage plein | refus explicite, rien d'altéré | RG-1.7 |
| 11 | Envoi vers un service non choisi | Erreur | toute action | observateur réseau | aucun envoi hors liste | RG-1.2 |

### SPEC-US-2 : Retrouver ses données sur son téléphone
**Réf :** US-2, RG-G4, RG-G7 ; arbitrage « Conflits de synchronisation »

#### Règles métier formalisées

**RG-2.1 : Dernière écriture gagne, champ par champ** `@Invariant` [ARB]
- **Réf AC :** [US-2/AC-2.6], arbitrage « Conflits »
- **Pré-conditions :** deux valeurs v1, v2 d'un même champ, d'estampilles t1, t2.
- **Post-conditions :** la valeur retenue est celle de plus grande estampille ; égalité départagée par un ordre total déterministe (par ex. identité d'appareil).
- **Expression formelle :** `fusion_champ((v1,t1),(v2,t2)) = (t1,a1) > (t2,a2) ? v1 : v2`, où `a` est l'identité de l'appareil auteur et `>` un ordre total.
- **Cas d'erreur :**
  - Horloges d'appareils décalées → voir [CONFLIT-4].

**RG-2.2 : Convergence de la fusion** `@Invariant`
- **Réf AC :** [US-2/AC-2.1], [US-2/AC-2.2], RG-G4
- **Pré-conditions :** ensembles d'écritures quelconques.
- **Post-conditions :** deux appareils ayant reçu les mêmes écritures, dans n'importe quel ordre, ont le même état.
- **Expression formelle :** `fusion` est commutative, associative et idempotente : `fusion(A,B)=fusion(B,A)`, `fusion(A,fusion(B,C))=fusion(fusion(A,B),C)`, `fusion(A,A)=A`.
- **Cas d'erreur :** aucun (propriété algébrique).

**RG-2.3 : Suppression = écriture datée** `@Invariant` [ARB]
- **Réf AC :** [US-2/AC-2.6], edge « supprimée d'un côté, modifiée de l'autre »
- **Pré-conditions :** une donnée porte une suppression datée t_s et des écritures de champs.
- **Post-conditions :** la plus récente des deux l'emporte, suppression ou modification.
- **Expression formelle :** `supprimée(d) ⇔ t_s(d) > max_f t(d, f)` — comparaison à valider, voir [CONFLIT-3].
- **Cas d'erreur :**
  - Donnée supprimée puis modifiée plus tard ailleurs → elle réapparaît ; ses effets dérivés (bloc d'agenda) sont rétablis (RG-10.1).

**RG-2.4 : Pas de perte silencieuse à la fusion** `@Invariant`
- **Réf AC :** [US-2/AC-2.6], RG-G7
- **Pré-conditions :** une fusion écarte une valeur.
- **Post-conditions :** la valeur écartée est récupérable ; si elle provient d'un conflit de synchronisation, l'utilisateur voit le résultat retenu et l'existence de l'autre version.
- **Expression formelle :** `∀ fusion, ∀ valeur écartée v → v ∈ versions_écartées(champ)` ; `conflit(v, v') → signalé(champ, retenue, écartée)`. Définition de « conflit » : voir [CONFLIT-5].
- **Cas d'erreur :**
  - Fusion produisant un élément invalide (règle inter-champs violée) → voir [CONFLIT-1].

**RG-2.5 : Délai de propagation** `@Property`
- **Réf AC :** [US-2/AC-2.1], [US-2/AC-2.2]
- **Pré-conditions :** deux appareils et un relais.
- **Post-conditions :** l'écriture est visible sur l'autre appareil dans le délai.
- **État externe requis :** connectivité de chaque appareil et du relais, temps écoulé.
- **Simulateur nécessaire :** simulateur à deux appareils + relais, connectivité commandable, horloge simulée.
- **Expression formelle :** `connectés(A,B,relais) à t → écriture(A,t) visible sur B avant t+60 s` ; `écriture(B) hors ligne, B reconnecté à t_r → visible sur A avant t_r+60 s`.
- **Cas d'erreur :**
  - Relais inaccessible → RG-2.6.

**RG-2.6 : Relais facultatif et reprise** `@Property`
- **Réf AC :** [US-2/AC-2.4], edge « relais inaccessible plusieurs jours »
- **Pré-conditions :** relais arrêté ou injoignable, durée quelconque.
- **Post-conditions :** chaque appareil reste entièrement utilisable ; la synchronisation reprend seule au retour du relais ; chaque appareil affiche la date de sa dernière synchronisation réussie.
- **État externe requis :** disponibilité du relais.
- **Simulateur nécessaire :** relais commandable (arrêt, reprise) + horloge simulée.
- **Expression formelle :** `relais_down → ∀ action a ne dépendant pas du relais, a réussit` ; `relais_up après down → convergence (RG-2.2) sans action utilisateur` ; `∀ appareil, dernière_synchro_réussie affichée`.
- **Cas d'erreur :** aucun blocage ; seule l'indication de fraîcheur change.

**RG-2.7 : Relais hébergé par l'utilisateur** `@Invariant`
- **Réf AC :** [US-2/AC-2.3]
- **Pré-conditions :** aucune.
- **Post-conditions :** l'outil ne synchronise qu'avec un relais dont l'utilisateur a fourni l'adresse.
- **Expression formelle :** `relais_configuré = ∅ → aucune synchronisation, outil pleinement utilisable sur l'appareil` ; aucune adresse de relais par défaut opérée par un tiers.
- **Cas d'erreur :**
  - Adresse de relais invalide ou injoignable → message explicite, RG-2.6.

**RG-2.8 : Données illisibles sur le relais** `@Invariant` `[CLASSIFICATION À VALIDER]`
- **Réf AC :** [US-2] edge « tiers non autorisé », Hypothèse « données illisibles y compris pour l'hébergeur »
- **Pré-conditions :** donnée transmise au relais.
- **Post-conditions :** le contenu n'est lisible qu'avec un secret détenu par l'utilisateur seul.
- **Expression formelle :** `∀ envoi s vers relais, contenu(s) indéchiffrable sans secret_utilisateur`.
- **Cas d'erreur :**
  - Appareil sans le bon secret → données reçues non lisibles, message explicite, aucune donnée locale altérée.
  - Altération des données par un tiers sur le relais → non couvert par le cahier, voir [MANQUE-6].
- *Classification à valider : propriété de la transformation appliquée avant envoi (sans état), mais sa preuve demande un relais observé.*

**Contrainte C-2.9 : Téléphone sans boutique** — non classée (contrainte de distribution)
- **Réf AC :** [US-2/AC-2.5]
- **Expression :** l'outil s'installe et s'utilise sur téléphone par le navigateur, sans passage par une boutique. Vérifiable par test d'acceptation manuel, pas par une règle. Dépend du spike stockage navigateur (voir Dépendances).

#### Matrice de cas — US-2

| # | Cas | Type | Input | État | Résultat attendu | Réf règle |
|---|-----|------|-------|------|-------------------|-----------|
| 1 | Modification sur l'ordinateur, tous connectés | Nominal | écriture A | A, B, relais up | visible sur B < 60 s | RG-2.5 |
| 2 | Modification hors ligne sur le téléphone | Nominal | écriture B hors ligne | B reconnecté | visible sur A < 60 s après reconnexion | RG-2.5 |
| 3 | Même champ modifié des deux côtés hors ligne | Edge | A: v1 à t1, B: v2 à t2>t1 | les deux reconnectés | v2 retenue, v1 récupérable, conflit signalé | RG-2.1, RG-2.4 |
| 4 | Champs différents modifiés des deux côtés | Nominal | A: titre, B: échéance | reconnectés | les deux modifications conservées | RG-2.1 |
| 5 | Supprimé sur A (t1), modifié sur B (t2>t1) | Edge | suppression + écriture | reconnectés | donnée présente, suppression récupérable | RG-2.3, RG-2.4 |
| 6 | Modifié sur A (t1), supprimé sur B (t2>t1) | Edge | écriture + suppression | reconnectés | donnée supprimée, version modifiée récupérable | RG-2.3, RG-2.4 |
| 7 | Estampilles égales | Edge | t1 = t2 | appareils distincts | même gagnant sur tous les appareils | RG-2.1, RG-2.2 |
| 8 | Ordre de réception différent | Edge | mêmes écritures, ordres différents | A, B | états identiques | RG-2.2 |
| 9 | Fusion produisant un engagement sans échéance | Edge | A: type→engagement+échéance, B: efface l'échéance plus tard | reconnectés | voir CONFLIT-1 | RG-2.4 |
| 10 | Relais arrêté 3 jours | Edge | usage normal | relais down | tout fonctionne, date de dernière synchro affichée, reprise seule | RG-2.6 |
| 11 | Aucun relais configuré | Edge | — | relais = ∅ | usage mono-appareil complet | RG-2.7 |
| 12 | Lecture par l'hébergeur du relais | Erreur | accès au stockage du relais | — | contenu illisible | RG-2.8 |

### SPEC-US-3 : Piloter l'outil en ligne de commande
**Réf :** US-3, RG-G3

#### Règles métier formalisées

**RG-3.1 : Parité interface graphique → ligne de commande** `@Invariant`
- **Réf AC :** [US-3/AC-3.1], RG-G3
- **Pré-conditions :** aucune.
- **Post-conditions :** toute action de l'interface graphique a son équivalent en ligne de commande, aux effets identiques.
- **Expression formelle :** `catalogue(GUI) ⊆ catalogue(CLI) ∧ ∀ a, effet_CLI(a, x) = effet_GUI(a, x)`.
- **Cas d'erreur :** action GUI sans équivalent CLI → violation bloquante.

**RG-3.2 : Points d'entrée des modules** `@Invariant`
- **Réf AC :** [US-3/AC-3.2]
- **Expression formelle :** `arbitre plan` → module planning ; `arbitre diff` → module decision diff.
- **Cas d'erreur :** module non activé → RG-3.7.

**RG-3.3 : Commande mal formée** `@Invariant`
- **Réf AC :** [US-3/AC-3.3], [US-3/AC-3.5]
- **Pré-conditions :** commande invalide (argument manquant, inconnu, valeur mal typée).
- **Post-conditions :** affichage de l'usage attendu, aucune donnée modifiée, code de retour d'échec.
- **Expression formelle :** `mal_formée(c) → état_après = état_avant ∧ sortie ∋ usage ∧ code(c) ≠ code_succès`.
- **Cas d'erreur :** c'est le cas d'erreur.

**RG-3.4 : Double sortie** `@Invariant`
- **Réf AC :** [US-3/AC-3.4]
- **Expression formelle :** `∀ commande c, ∃ sortie_humaine(c) ∧ sortie_structurée(c)` ; les deux portent la même information (même état, mêmes signalements : débordement, retard, non planifié, non à jour).
- **Cas d'erreur :** erreur → la sortie structurée porte aussi l'erreur, dans un format stable.

**RG-3.5 : Code de retour** `@Invariant`
- **Réf AC :** [US-3/AC-3.5]
- **Expression formelle :** `∀ c, échec(c) → code(c) ≠ code_succès`. Distinction violation / erreur d'outil pour decision diff : voir [AMB-16].

**RG-3.6 : Écritures concurrentes sur un même appareil** `@Property`
- **Réf AC :** [US-3] edge « commande pendant que l'interface modifie la même donnée »
- **Pré-conditions :** deux surfaces écrivent la même donnée au même moment.
- **Post-conditions :** aucune des deux écritures n'est perdue silencieusement.
- **État externe requis :** ordonnancement des deux écrivains.
- **Simulateur nécessaire :** deux écrivains locaux concurrents à entrelacement commandable.
- **Expression formelle :** même mécanisme que la fusion (RG-2.1, RG-2.4) : la valeur écartée reste récupérable.
- **Cas d'erreur :** aucune perte silencieuse.

**RG-3.7 : Module non activé appelé** `@Invariant`
- **Réf AC :** [US-3] edge « module non activé », [US-5/AC-5.4]
- **Expression formelle :** `¬activé(m) ∧ appel(m) → message « module non activé » ∧ code ≠ succès ∧ aucune modification`. Voir [CONFLIT-6].

#### Matrice de cas — US-3

| # | Cas | Type | Input | État | Résultat attendu | Réf règle |
|---|-----|------|-------|------|-------------------|-----------|
| 1 | Vue du matin | Nominal | `arbitre plan` | planning activé | journée, charge, questions en attente | RG-3.2 |
| 2 | Sortie machine | Nominal | commande + option structurée | — | sortie structurée équivalente | RG-3.4 |
| 3 | Argument inconnu | Erreur | `arbitre plan --xyz` | — | usage, rien modifié, code ≠ 0 | RG-3.3 |
| 4 | Date invalide | Erreur | échéance « 31/02 » | — | refus, rien modifié, code ≠ 0 | RG-3.3 |
| 5 | Module désactivé appelé | Erreur | `arbitre diff` | decision diff désactivé | « module non activé », code ≠ 0 | RG-3.7 |
| 6 | CLI et GUI modifient le même champ | Edge | deux écritures | concurrence locale | une retenue, l'autre récupérable | RG-3.6 |
| 7 | Action GUI sans équivalent CLI | Erreur | catalogue | — | violation détectée | RG-3.1 |

### SPEC-US-4 : Laisser un assistant d'IA agir pour moi, quel que soit son fournisseur
**Réf :** US-4, RG-G3, RG-G5

#### Règles métier formalisées

**RG-4.1 : Parité ligne de commande → assistant** `@Invariant`
- **Réf AC :** [US-4/AC-4.1]
- **Expression formelle :** `catalogue(CLI) ⊆ catalogue(MCP) ∧ ∀ a, effet_MCP(a, x) = effet_CLI(a, x)`.
- **Cas d'erreur :** action CLI absente du catalogue MCP → violation bloquante.

**RG-4.2 : Fournisseurs sélectionnables** `@Invariant`
- **Réf AC :** [US-4/AC-4.2]
- **Expression formelle :** `{Claude, GPT, Mistral, modèle local} ⊆ fournisseurs_sélectionnables`.

**RG-4.3 : Changement de fournisseur sans perte** `@Property`
- **Réf AC :** [US-4/AC-4.3]
- **Pré-conditions :** fournisseur P1 configuré.
- **Post-conditions :** après passage à P2 par un seul réglage, données et catalogue de fonctions identiques.
- **État externe requis :** fournisseurs disponibles.
- **Simulateur nécessaire :** bouchons de fournisseurs (réponses commandables).
- **Expression formelle :** `données(après) = données(avant) ∧ catalogue(après) = catalogue(avant)`.
- **Cas d'erreur :** P2 indisponible → RG-4.5.

**RG-4.4 : Fonctions sans IA disponibles sans fournisseur** `@Invariant`
- **Réf AC :** [US-4/AC-4.4], [US-7/AC-7.5], RG-G5
- **Expression formelle :** `∀ f, ¬requiert_IA(f) → disponible(f) quand fournisseur = ∅`. Liste fermée des fonctions qui requièrent l'IA dans ce cadre : proposition d'invariants (US-18). Fonctions où l'IA est optionnelle et dont l'absence ne fait rien échouer : décisions proposées par IA (US-16), amélioration des durées proposées (US-7).
- **Cas d'erreur :** fonction requérant l'IA appelée sans fournisseur → message « fournisseur non configuré », rien d'autre affecté.

**RG-4.5 : Défaillance du fournisseur** `@Property`
- **Réf AC :** [US-4] edges « fournisseur indisponible », « clé invalide », « modèle local lent ou absent »
- **Pré-conditions :** fournisseur injoignable, clé invalide, ou réponse au-delà d'un délai [À DÉFINIR].
- **Post-conditions :** la fonction dépendante échoue avec une erreur explicite ; les autres fonctions marchent ; aucune donnée modifiée par la fonction en échec.
- **État externe requis :** état du fournisseur.
- **Simulateur nécessaire :** fournisseur simulé (injoignable, clé refusée, réponse lente, absent).
- **Expression formelle :** `défaillant(P) → ∀ f requérant P : erreur_explicite(f) ∧ état inchangé ; ∀ g ne requérant pas P : comportement nominal`.
- **Cas d'erreur :** c'est le cas d'erreur.

**RG-4.6 : Journal des suppressions et déplacements par l'assistant** `@Invariant`
- **Réf AC :** [US-4/AC-4.5]
- **Expression formelle :** `∀ action a via surface MCP, a ∈ {suppression, déplacement} → historique ∋ (date, auteur = « assistant », action, donnée)`. « Déplacement » et identification de l'auteur : voir [AMB-11].
- **Cas d'erreur :** l'action n'est pas appliquée si son entrée d'historique ne peut pas l'être (les deux ou aucune) [DÉRIVÉE].

**RG-4.7 : Action invalide de l'assistant** `@Invariant`
- **Réf AC :** [US-4] edge « action invalide »
- **Expression formelle :** même validation que sur toute surface (RG-G3) : `invalide(a) → refus explicite ∧ état inchangé`.
- **Cas d'erreur :** date inexistante, élément introuvable, type inconnu → refus explicite.

#### Matrice de cas — US-4

| # | Cas | Type | Input | État | Résultat attendu | Réf règle |
|---|-----|------|-------|------|-------------------|-----------|
| 1 | Ajout via assistant | Nominal | « ajoute la relecture du contrat pour jeudi » | planning activé | élément créé, durée proposée | RG-4.1, RG-7.2 |
| 2 | Suppression via assistant | Nominal | suppression d'un élément | — | supprimé + historique (date, « assistant ») | RG-4.6 |
| 3 | Élément introuvable | Erreur | id inconnu | — | refus, rien modifié | RG-4.7 |
| 4 | Changement Claude → modèle local | Nominal | réglage | données existantes | données et fonctions identiques | RG-4.3 |
| 5 | Aucun fournisseur | Edge | usage planning | fournisseur = ∅ | planning complet, durées proposées | RG-4.4 |
| 6 | Clé invalide | Erreur | proposition d'invariants | clé refusée | erreur explicite, reste du module OK | RG-4.5 |
| 7 | Modèle local trop lent | Erreur | fonction IA | délai dépassé | erreur explicite, rien modifié | RG-4.5 |

### SPEC-US-5 : N'activer que les modules utiles
**Réf :** US-5, RG-G2

#### Règles métier formalisées

**RG-5.1 : Activation indépendante** `@Invariant`
- **Réf AC :** [US-5/AC-5.1], RG-G2
- **Expression formelle :** `∀ M ⊆ {planning, decision diff}, activés = M → ∀ m ∈ M, m pleinement fonctionnel`. Aucun module ne requiert un autre module.

**RG-5.2 : Désactiver conserve les données** `@Invariant`
- **Réf AC :** [US-5/AC-5.2]
- **Expression formelle :** `données(m) après (désactiver ; activer) = données(m) avant`, modulo les écritures reçues entre-temps par synchronisation (voir [AMB-12]).

**RG-5.3 : Cloisonnement** `@Invariant`
- **Réf AC :** [US-5/AC-5.3], RG-G2
- **Expression formelle :** `∀ module m, ∀ vue ou action de m : elle ne lit ni n'écrit que des données de m ou du socle`.

**RG-5.4 : Invisibilité d'un module désactivé** `@Invariant`
- **Réf AC :** [US-5/AC-5.4]
- **Expression formelle :** `¬activé(m) → m ∉ interface ∧ m ∉ aide/catalogue CLI ∧ m ∉ catalogue MCP`. Articulation avec RG-3.7 : [CONFLIT-6].

**RG-5.5 : Données importées d'un module désactivé** `@Invariant`
- **Réf AC :** [US-5] edge
- **Expression formelle :** `import(données(m)) ∧ ¬activé(m) → conservées ∧ invisibles ; activer(m) → visibles`.

#### Matrice de cas — US-5

| # | Cas | Type | Input | État | Résultat attendu | Réf règle |
|---|-----|------|-------|------|-------------------|-----------|
| 1 | Decision diff seul | Nominal | activer decision diff | planning jamais activé | aucune trace du planning | RG-5.1, RG-5.4 |
| 2 | Désactiver puis réactiver | Nominal | désactiver/activer planning | pile non vide | pile intacte | RG-5.2 |
| 3 | Catalogue MCP | Nominal | lister les outils | planning désactivé | aucun outil planning | RG-5.4 |
| 4 | Import pendant désactivation | Edge | export avec pile | planning désactivé | pile conservée, visible à l'activation | RG-5.5 |
| 5 | Vue planning | Erreur | lecture d'une donnée decision diff depuis planning | les deux activés | impossible | RG-5.3 |

### SPEC-US-6 : Tenir ma pile d'éléments à faire
**Réf :** US-6

#### Règles métier formalisées

**RG-6.1 : Validité d'un élément** `@Invariant`
- **Réf AC :** [US-6/AC-6.1], [US-6/AC-6.2]
- **Pré-conditions :** création ou modification.
- **Post-conditions :** l'élément stocké est valide.
- **Expression formelle :** `valide(e) ⇔ intitulé(e) non vide ∧ type(e) ∈ {geste, décision en attente, engagement daté} ∧ (type(e) = engagement daté → échéance(e) définie)`.
- **Cas d'erreur :**
  - Intitulé vide → refus explicite.
  - Type inconnu → refus explicite.
  - Engagement daté sans échéance → refus explicite.
  - Changement de type vers engagement daté sans échéance → refus explicite.
  - Échéance retirée d'un engagement daté → refus explicite.

**RG-6.2 : Cycle de vie** `@Invariant`
- **Réf AC :** [US-6/AC-6.3], [US-6/AC-6.4]
- **Expression formelle :** transitions `actif → fait`, `actif → supprimé` ; `fait(e) → e ∉ pile ∧ e ∈ historique consultable`. Retour de `fait` à `actif` et récupération d'un élément supprimé par l'utilisateur : [AMB-10].
- **Cas d'erreur :** opération sur un élément introuvable → refus explicite.

**RG-6.3 : Retard** `@Invariant`
- **Réf AC :** [US-6] edge « échéance passée », [US-12] edge « bloc déplacé après l'échéance »
- **Expression formelle :** `en_retard(e, aujourd'hui) ⇔ statut(e) = actif ∧ échéance(e) définie ∧ (échéance(e) < aujourd'hui ∨ (jour_planifié(e) défini ∧ jour_planifié(e) > échéance(e)))`. Une échéance passée est acceptée à la création.

**RG-6.4 : Même résultat sur toutes les surfaces** `@Invariant`
- **Réf AC :** [US-6/AC-6.5], RG-G3
- **Expression formelle :** `∀ surfaces s1, s2, ∀ entrée x : créer_s1(x) ≡ créer_s2(x)` (même élément, même durée proposée, mêmes signalements).

**RG-6.5 : Fin d'un élément et agenda** `@Property`
- **Réf AC :** [US-6] edge « élément supprimé avec bloc », [US-10/AC-10.5]
- **Pré-conditions :** élément planifié qui a un bloc ou une part du bloc quotidien.
- **Post-conditions :** voir RG-10.4.
- **État externe requis :** agenda en écriture.
- **Simulateur nécessaire :** simulateur d'agenda (voir US-10).
- **Expression formelle :** `statut(e) ∈ {fait, supprimé} → bloc(e) retiré (ou bloc quotidien réduit)` au plus tard à la prochaine écriture possible dans l'agenda.

#### Matrice de cas — US-6

| # | Cas | Type | Input | État | Résultat attendu | Réf règle |
|---|-----|------|-------|------|-------------------|-----------|
| 1 | Engagement daté | Nominal | « envoyer le devis à X », engagement, vendredi | — | créé dans la pile | RG-6.1 |
| 2 | Geste sans échéance | Nominal | geste | — | créé | RG-6.1 |
| 3 | Engagement sans échéance | Erreur | engagement, échéance vide | — | refus explicite | RG-6.1 |
| 4 | Intitulé vide | Erreur | « » | — | refus | RG-6.1 |
| 5 | Échéance hier | Edge | engagement, échéance = hier | — | créé, en retard | RG-6.3 |
| 6 | Échéance aujourd'hui | Edge | échéance = aujourd'hui | — | pas en retard | RG-6.3 |
| 7 | Marquer fait | Nominal | fait | élément actif | hors pile, dans l'historique | RG-6.2 |
| 8 | Supprimer un élément bloqué | Edge | suppression | bloc dans l'agenda | bloc retiré | RG-6.5 |
| 9 | Création depuis le téléphone vs CLI | Nominal | même entrée | — | même élément | RG-6.4 |

### SPEC-US-7 : Obtenir une durée pour chaque élément, et la corriger
**Réf :** US-7, RG-G5

#### Règles métier formalisées

**RG-7.1 : Durée valide** `@Invariant`
- **Réf AC :** [US-7] edge « durée nulle ou négative »
- **Expression formelle :** `durée(e) > 0` (minutes).
- **Cas d'erreur :**
  - Durée ≤ 0 → refus explicite.
  - Durée supérieure à la capacité d'une journée → [RENVOI-1], aucune option ici.

**RG-7.2 : Durée proposée à la création** `@Invariant`
- **Réf AC :** [US-7/AC-7.1], [US-7/AC-7.4], [US-7/AC-7.5]
- **Expression formelle :** `∀ e nouveau sans durée saisie : durée(e) définie ∧ origine(e) = proposée`, calculée sans IA. `origine` est exposée sur toutes les surfaces.

**RG-7.3 : Correction** `@Invariant`
- **Réf AC :** [US-7/AC-7.2], [US-7/AC-7.4]
- **Expression formelle :** `corriger(e, d) → durée(e) = d ∧ origine(e) = fixée` ; `origine(e) = fixée → aucune proposition ultérieure ne change durée(e)`. Une durée réduite via la réponse à une question (AC-13.6) est une correction.

**RG-7.4 : Durée par défaut par type** `@Invariant`
- **Réf AC :** [US-7] edge « aucun historique »
- **Expression formelle :** `corrections_semblables(e) = ∅ → durée_proposée(e) = défaut(type(e))`. Valeurs : [MANQUE-5].

**RG-7.5 : Prise en compte des corrections** `@Property`
- **Réf AC :** [US-7/AC-7.3]
- **Pré-conditions :** historique de corrections H.
- **Post-conditions :** la proposition pour un nouvel élément se rapproche des corrections faites sur les éléments semblables.
- **État externe requis :** historique des corrections.
- **Simulateur nécessaire :** générateur d'historiques de corrections.
- **Expression formelle :** `∀ e' nouveau, ∀ correction c = (e, d) avec semblable(e, e') : proposition(e', H ∪ {c}) ∈ [min(proposition(e', H), d), max(proposition(e', H), d)]` ; sans IA, `proposition(e', H)` est déterministe.
- **Cas d'erreur :** aucun ; sans élément semblable, RG-7.4.
- *Stabilité des propositions déjà attribuées : [AMB-9].*

**RG-7.6 : Éléments semblables** `@Invariant` `[CLASSIFICATION À VALIDER]`
- **Réf AC :** [US-7/AC-7.3], Hypothèse « même type et intitulé proche, règle laissée à la spécification »
- **Expression formelle :** `semblable(e1, e2) ⇒ type(e1) = type(e2) ∧ proximité(intitulé(e1), intitulé(e2)) ≥ seuil`. Propriétés exigées de `proximité` : déterministe, sans IA, symétrique, insensible à la casse et aux accents. Mesure et seuil : [À DÉFINIR] au cycle de US-7.

#### Matrice de cas — US-7

| # | Cas | Type | Input | État | Résultat attendu | Réf règle |
|---|-----|------|-------|------|-------------------|-----------|
| 1 | Premier élément | Edge | geste | aucun historique | durée = défaut(geste), origine proposée | RG-7.4 |
| 2 | Correction | Nominal | « relire le contrat » 1 h → 2 h | — | durée 2 h, origine fixée | RG-7.3 |
| 3 | Élément semblable après correction | Nominal | « relire le contrat de Y » | correction 1 h → 2 h | proposition entre l'ancienne proposition et 2 h | RG-7.5 |
| 4 | Intitulé proche, type différent | Edge | décision « relire le contrat » | correction sur un geste | correction non prise en compte | RG-7.6 |
| 5 | Durée 0 | Erreur | 0 min | — | refus | RG-7.1 |
| 6 | Durée négative | Erreur | −30 min | — | refus | RG-7.1 |
| 7 | Durée 10 h | Edge | 600 min | — | [RENVOI-1] | RG-7.1 |
| 8 | Sans fournisseur d'IA | Edge | nouvel élément | fournisseur = ∅ | durée proposée disponible | RG-7.2 |
| 9 | Durée fixée puis nouvelles corrections ailleurs | Edge | — | origine fixée | durée inchangée | RG-7.3 |

### SPEC-US-8 : Voir ma charge par jour face à ma capacité
**Réf :** US-8 ; arbitrage « Agendas »

#### Règles métier formalisées

**RG-8.1 : Réunions comptées** `@Invariant`
- **Réf AC :** [US-8/AC-8.1], [US-8/AC-8.6], Hypothèses « réunion déclinée », arbitrage « plusieurs agendas en lecture »
- **Expression formelle :** `comptée(o) ⇔ o ∈ ⋃ agendas_en_lecture ∧ ¬déclinée(o) ∧ ¬événement_de_l_outil(o) ∧ ¬journée_entière(o)`. Le socle ne reçoit que des occurrences datées [ARB]. Statuts « peut-être », « sans réponse », « disponible » : [AMB-2]. Réunions hors des heures de travail : [RENVOI-8].

**RG-8.2 : Capacité** `@Invariant`
- **Réf AC :** [US-8/AC-8.1], [US-8/AC-8.2], edges « chevauchement », « journée entière », Hypothèses « lundi–vendredi », « 7 h 30 non réglable »
- **Expression formelle :**
  ```
  capacité(d) = 0                                   si d ∉ jours_travaillés
              = 0                                   si ∃ o journée entière ou absence le jour d
              = max(0, 450 − |⋃ { [début(o), fin(o)] ∩ d : comptée(o) }|)   sinon
  ```
  `|⋃ …|` est la durée de l'union des intervalles : un temps chevauché, y compris entre agendas différents, n'est compté qu'une fois. `∩ d` découpe une réunion qui déborde sur un autre jour. Fenêtre horaire de `d` : [RENVOI-8] ; fuseau : [AMB-8]. Journée entière ≠ absence : [AMB-1].
- **Cas d'erreur :** aucun (fonction totale).

**RG-8.3 : Charge** `@Invariant`
- **Réf AC :** [US-8/AC-8.3]
- **Expression formelle :** `charge(d) = Σ { durée(e) : statut(e) = actif ∧ jour_planifié(e) = d }`. Les petits gestes comptent pour leur durée propre, pas pour la taille du bloc quotidien : [AMB-3]. Éléments faits dans la journée : [AMB-4].

**RG-8.4 : Débordement** `@Invariant`
- **Réf AC :** [US-8/AC-8.4]
- **Expression formelle :** `déborde(d) ⇔ charge(d) > capacité(d)` ; `excès(d) = charge(d) − capacité(d)`. Le signalement et l'excès figurent sur toutes les surfaces, y compris en sortie structurée (RG-3.4).

**RG-8.5 : Fraîcheur de la capacité** `@Property`
- **Réf AC :** [US-8/AC-8.5], edge « agenda injoignable »
- **Pré-conditions :** changement dans un agenda en lecture, ou agenda injoignable.
- **Post-conditions :** la capacité reflète le changement au plus tard au recalcul suivant ; si l'agenda est injoignable, la dernière capacité connue est affichée, marquée « non à jour » avec la date de la dernière lecture réussie.
- **État externe requis :** contenu et disponibilité de chaque agenda en lecture.
- **Simulateur nécessaire :** simulateur d'agendas (occurrences ajoutées, déplacées, annulées, déclinées ; indisponibilité ; plusieurs agendas).
- **Expression formelle :** `changement(agenda) à t ∧ recalcul à t' > t → capacité affichée après t' = capacité(nouvel agenda)` ; `injoignable(agenda) → capacité affichée = dernière connue ∧ marque(non à jour, date)`.
- **Cas d'erreur :** un seul agenda en lecture injoignable sur plusieurs → [À DÉFINIR] au cycle de US-8 (capacité partielle ou toute marquée non à jour).

#### Matrice de cas — US-8

| # | Cas | Type | Input | État | Résultat attendu | Réf règle |
|---|-----|------|-------|------|-------------------|-----------|
| 1 | Mardi 3 h de réunions, 5 h d'éléments | Nominal | — | — | capacité 270, débordement 30 min | RG-8.2, RG-8.4 |
| 2 | Réunions > 7 h 30 | Edge | 8 h de réunions | — | capacité 0 | RG-8.2 |
| 3 | Chevauchement | Edge | 10h–12h et 11h–13h | — | 3 h comptées | RG-8.2 |
| 4 | Chevauchement entre deux agendas | Edge | même créneau pro et perso | 2 agendas en lecture | compté une fois | RG-8.2 |
| 5 | Réunion déclinée | Edge | occurrence déclinée | — | non comptée | RG-8.1 |
| 6 | Bloc de l'outil | Edge | bloc 2 h | — | non compté | RG-8.1 |
| 7 | Congé | Edge | absence toute la journée | — | capacité 0 | RG-8.2 |
| 8 | Samedi | Edge | — | — | capacité 0 | RG-8.2 |
| 9 | Réunion à cheval sur minuit | Edge | 23h–01h | — | découpée par jour | RG-8.2 |
| 10 | Réunion à 20 h | Edge | — | — | [RENVOI-8] | RG-8.1 |
| 11 | Réunion ajoutée la veille | Nominal | nouvelle occurrence | recalcul | capacité mise à jour | RG-8.5 |
| 12 | Agenda injoignable | Erreur | recalcul | agenda down | dernière capacité + « non à jour » + date | RG-8.5 |
| 13 | Charge = capacité | Edge | 270 / 270 | — | pas de débordement | RG-8.4 |

### SPEC-US-9 : Consulter et reprioriser la pile entière
**Réf :** US-9

#### Règles métier formalisées

**RG-9.1 : Ordre total unique** `@Invariant`
- **Réf AC :** [US-9/AC-9.1]
- **Expression formelle :** `pile = { e : statut(e) = actif }`, planifiés ou non, munie d'un ordre total strict par rang. Après fusion de synchronisation, l'ordre reste total : [CONFLIT-2].

**RG-9.2 : Réordonnancement** `@Invariant`
- **Réf AC :** [US-9/AC-9.2]
- **Expression formelle :** `déplacer(e, p) → position(e) = p ∧ ordre relatif des autres éléments inchangé ∧ persistant`.
- **Cas d'erreur :** position hors bornes → refus explicite ; élément inactif ou introuvable → refus.

**RG-9.3 : Informations par élément** `@Invariant`
- **Réf AC :** [US-9/AC-9.3], [US-9/AC-9.4]
- **Expression formelle :** chaque élément listé expose type, durée et origine, échéance si définie, jour planifié si défini, et une marque `non planifié` distincte quand il n'en a pas (y compris en sortie structurée).

**RG-9.4 : La pile fait foi pour l'ordre** `@Property`
- **Réf AC :** [US-9/AC-9.5], CADRAGE §planning 5
- **Pré-conditions :** pile ordonnée, agenda quelconque.
- **Post-conditions :** aucun événement d'agenda ne change un rang.
- **État externe requis :** évolution de l'agenda.
- **Simulateur nécessaire :** simulateur d'agenda (déplacements, ajouts, suppressions d'occurrences, y compris de blocs).
- **Expression formelle :** `∀ séquence d'événements d'agenda E, rangs(après E) = rangs(avant E)`. Effet de la repriorisation sur le planning : [CONFLIT-10]. Durées : la pile fait foi, sauf question renvoyée sur le redimensionnement ([RENVOI-3]).

**RG-9.5 : Pile entière et pile vide** `@Invariant`
- **Réf AC :** [US-9] edges
- **Expression formelle :** consultation sans troncature quel que soit le nombre d'éléments ; `pile = ∅ → message « pile vide », pas d'erreur, code de succès`.

#### Matrice de cas — US-9

| # | Cas | Type | Input | État | Résultat attendu | Réf règle |
|---|-----|------|-------|------|-------------------|-----------|
| 1 | Remonter un sujet | Nominal | déplacer(e, 1) | pile de 20 | e en tête, autres inchangés | RG-9.2 |
| 2 | Position hors bornes | Erreur | déplacer(e, 999) | pile de 20 | refus | RG-9.2 |
| 3 | Non planifiés visibles | Nominal | consulter | mix planifié/non | marque `non planifié` | RG-9.3 |
| 4 | 500 éléments | Edge | consulter | 500 actifs | tous listés | RG-9.5 |
| 5 | Pile vide | Edge | consulter | ∅ | message, pas d'erreur | RG-9.5 |
| 6 | Bloc déplacé dans l'agenda | Edge | déplacement de bloc | — | rangs inchangés | RG-9.4 |
| 7 | Réordonnancements concurrents hors ligne | Edge | A et B réordonnent | reconnectés | ordre total, voir CONFLIT-2 | RG-9.1 |

### SPEC-US-10 : Voir mes éléments comme des blocs dans mon agenda
**Réf :** US-10 ; arbitrage « un seul agenda en écriture »

#### Règles métier formalisées

**RG-10.1 : Plan de blocs attendu** `@Invariant`
- **Réf AC :** [US-10/AC-10.1], [US-10/AC-10.2]
- **Expression formelle :** `∀ e actif, jour_planifié(e) = d, durée(e) ≥ 30 → exactement un bloc(e) attendu le jour d, de durée durée(e)`, dans l'agenda en écriture. Heure de placement dans le jour, créneau insuffisant : [RENVOI-2], [RENVOI-8].

**RG-10.2 : Bloc quotidien des petits gestes** `@Invariant`
- **Réf AC :** [US-10/AC-10.3], [US-10/AC-10.5]
- **Expression formelle :** `P(d) = { e actif : jour_planifié(e) = d ∧ durée(e) < 30 }` ; `P(d) ≠ ∅ → exactement un bloc quotidien le jour d, de durée max(30, Σ_{e∈P(d)} durée(e))`. Un élément de 30 min exactement a son propre bloc. `P(d) = ∅` (bloc quotidien vide), heure, taille maximale : [RENVOI-5].

**RG-10.3 : Pas de chevauchement avec une réunion** `@Invariant`
- **Réf AC :** [US-10/AC-10.4]
- **Expression formelle :** `∀ bloc b placé par l'outil, ∀ o comptée : intervalle(b) ∩ intervalle(o) = ∅`. `[DÉRIVÉE]` deux événements de l'outil ne se chevauchent pas entre eux. Bloc placé par l'utilisateur sur une réunion : [CONFLIT-9].

**RG-10.4 : Retrait des blocs** `@Property`
- **Réf AC :** [US-10/AC-10.5], [US-6] edge
- **Pré-conditions :** e passe à `fait` ou `supprimé`.
- **Post-conditions :** bloc(e) retiré ; si e ∈ P(d), le bloc quotidien rétrécit selon RG-10.2.
- **État externe requis :** agenda en écriture.
- **Simulateur nécessaire :** simulateur d'agenda en écriture (création, modification, suppression d'occurrences, injoignabilité, événements créés par d'autres).
- **Expression formelle :** `agenda converge vers le plan de blocs attendu (RG-10.1, RG-10.2)` à la prochaine écriture possible.

**RG-10.5 : Événements tiers intouchables** `@Property` (bloquant)
- **Réf AC :** [US-10/AC-10.6]
- **Pré-conditions :** toute écriture de l'outil dans un agenda.
- **Post-conditions :** seuls des événements de l'outil ont été créés, modifiés ou supprimés.
- **État externe requis :** origine de chaque événement de l'agenda.
- **Simulateur nécessaire :** simulateur d'agenda avec événements d'origines mêlées.
- **Expression formelle :** `∀ opération op ∈ {modifier, supprimer}, cible(op) ∈ événements_de_l_outil` ; aucune écriture dans un agenda en lecture seule. Marqueur d'appartenance : [CONFLIT-11].

**RG-10.6 : Agenda injoignable** `@Property`
- **Réf AC :** [US-10] edge
- **Pré-conditions :** agenda en écriture injoignable au moment de poser, déplacer ou retirer un bloc.
- **Post-conditions :** l'élément reste planifié dans la pile ; l'opération est appliquée dès que l'agenda redevient joignable, sans doublon.
- **État externe requis :** disponibilité de l'agenda.
- **Simulateur nécessaire :** simulateur d'agenda avec indisponibilité et reprise.
- **Expression formelle :** `injoignable → état de la pile inchangé ; joignable → agenda = plan attendu`.

**RG-10.7 : Unicité des blocs** `@Property` [DÉRIVÉE]
- **Réf AC :** [US-10/AC-10.1], [US-10/AC-10.3], RG-G3 (plusieurs appareils)
- **Pré-conditions :** plusieurs appareils ou plusieurs passages peuvent poser des blocs.
- **Post-conditions :** au plus un bloc par élément, au plus un bloc quotidien par jour.
- **État externe requis :** agenda en écriture, appareils actifs.
- **Simulateur nécessaire :** deux appareils posant des blocs concurrents + agenda simulé.
- **Expression formelle :** `∀ e, |blocs(e)| ≤ 1 ; ∀ d, |blocs_quotidiens(d)| ≤ 1`. Voir [AMB-22].

#### Matrice de cas — US-10

| # | Cas | Type | Input | État | Résultat attendu | Réf règle |
|---|-----|------|-------|------|-------------------|-----------|
| 1 | « Préparer la démo » 2 h mercredi | Nominal | — | créneau libre entre deux réunions | bloc de 2 h, hors réunions | RG-10.1, RG-10.3 |
| 2 | Trois gestes de 10 min | Nominal | — | même jour | un bloc quotidien de 30 min | RG-10.2 |
| 3 | Un geste de 10 min | Edge | — | — | bloc quotidien de 30 min | RG-10.2 |
| 4 | Gestes totalisant 75 min | Edge | — | — | bloc quotidien de 75 min ; seuil [RENVOI-5] | RG-10.2 |
| 5 | Élément de 30 min | Edge | — | — | bloc propre de 30 min | RG-10.1 |
| 6 | Élément fait | Nominal | fait | bloc posé | bloc retiré | RG-10.4 |
| 7 | Un geste de 10 min fait sur 4 | Nominal | fait | bloc quotidien 40 min | bloc quotidien 30 min | RG-10.2, RG-10.4 |
| 8 | Dernier petit geste fait | Edge | fait | bloc quotidien 30 min | [RENVOI-5] | RG-10.2 |
| 9 | Pas de créneau assez long | Edge | 3 h | créneaux de 1 h | [RENVOI-2] | RG-10.1 |
| 10 | Agenda injoignable | Erreur | planification | agenda down | élément planifié, bloc posé à la reprise, sans doublon | RG-10.6 |
| 11 | Événement d'un collègue au même créneau | Erreur | — | événement tiers | jamais modifié | RG-10.5 |
| 12 | Deux appareils posent le bloc | Edge | — | A et B actifs | un seul bloc | RG-10.7 |

### SPEC-US-11 : Montrer à mes collègues que je suis occupé, sans rien révéler
**Réf :** US-11

#### Règles métier formalisées

**RG-11.1 : Titre et statut** `@Invariant`
- **Réf AC :** [US-11/AC-11.1]
- **Expression formelle :** `∀ bloc(e) : statut = occupé ∧ titre = « Focus — » + sujet_affiché(e)` ; `sujet_public(e) défini → sujet_affiché(e) = sujet_public(e)`.

**RG-11.2 : Minimalité du contenu visible** `@Invariant`
- **Réf AC :** [US-11/AC-11.2], RG-1.2
- **Expression formelle :** contenu d'un événement de l'outil visible des collègues = `{titre, début, fin, statut occupé}` ; aucune description, échéance, type, intitulé. Marqueur d'appartenance : [CONFLIT-11].

**RG-11.3 : Sujet public indépendant** `@Invariant`
- **Réf AC :** [US-11/AC-11.3]
- **Expression formelle :** `sujet_public(e)` est un champ distinct de `intitulé(e)`, modifiable seul ; le modifier change le titre du bloc et rien d'autre.

**RG-11.4 : Titre générique du bloc quotidien** `@Invariant`
- **Réf AC :** [US-11/AC-11.4]
- **Expression formelle :** `titre(bloc_quotidien)` est une constante qui ne contient aucun intitulé ni sujet public d'un petit geste.

**RG-11.5 : Pas d'intitulé brut par défaut** `@Invariant`
- **Réf AC :** [US-11] edge
- **Expression formelle :** `sujet_public(e) non défini → sujet_affiché(e) ≠ intitulé(e)`. Valeur par défaut : [RENVOI-4], aucune option ici.

#### Matrice de cas — US-11

| # | Cas | Type | Input | État | Résultat attendu | Réf règle |
|---|-----|------|-------|------|-------------------|-----------|
| 1 | Sujet public défini | Nominal | « négocier l'augmentation de Z », sujet « RH » | — | « Focus — RH », occupé | RG-11.1 |
| 2 | Échéance et type | Erreur | engagement daté avec échéance | — | absents de l'événement | RG-11.2 |
| 3 | Sujet public modifié | Nominal | « RH » → « Admin » | bloc posé | titre mis à jour, rien d'autre | RG-11.3 |
| 4 | Bloc quotidien | Nominal | gestes « appeler Z », « payer X » | — | titre générique sans ces mots | RG-11.4 |
| 5 | Pas de sujet public | Edge | intitulé sensible | — | titre ≠ « Focus — » + intitulé ; défaut [RENVOI-4] | RG-11.5 |

### SPEC-US-12 : Déplacer un bloc dans mon agenda et que la pile suive
**Réf :** US-12, CADRAGE §planning 5

#### Règles métier formalisées

**RG-12.1 : L'agenda fait foi pour le « quand »** `@Property`
- **Réf AC :** [US-12/AC-12.1], [US-12/AC-12.2]
- **Pré-conditions :** le bloc de e se trouve, à la relecture, au jour d' ≠ jour_planifié(e).
- **Post-conditions :** jour_planifié(e) = d' au plus tard au recalcul suivant.
- **État externe requis :** position des blocs dans l'agenda en écriture.
- **Simulateur nécessaire :** simulateur d'agenda avec déplacements manuels.
- **Expression formelle :** `jour(bloc(e)) = d' ≠ jour_planifié(e) ∧ recalcul → jour_planifié(e) := d'`. Déplacement à une autre heure du même jour : aucun effet sur la pile. Bloc supprimé, redimensionné, bloc quotidien déplacé : [RENVOI-3], [RENVOI-5].

**RG-12.2 : Effet limité au jour planifié** `@Invariant`
- **Réf AC :** [US-12/AC-12.3], [US-12/AC-12.4]
- **Expression formelle :** la prise en compte d'un déplacement ne change que `jour_planifié(e)` : `rang(e)`, `durée(e)` et tout champ de tout autre élément sont inchangés.

**RG-12.3 : Débordement causé par un déplacement** `@Property`
- **Réf AC :** [US-12/AC-12.4]
- **Pré-conditions :** après RG-12.1, `déborde(d')`.
- **Post-conditions :** débordement signalé, une question est posée (RG-13.5) ; rien d'autre n'est déplacé.
- **État externe requis :** capacité de d' (agenda).
- **Simulateur nécessaire :** simulateur d'agenda (déplacement + réunions).
- **Expression formelle :** `déborde(d') → question(d') ∧ ∀ e' ≠ e : jour_planifié(e') inchangé`.

**RG-12.4 : Déplacement hors capacité** `@Invariant`
- **Réf AC :** [US-12] edge « week-end ou congé »
- **Expression formelle :** `capacité(d') = 0 → déplacement accepté ∧ signalé « hors capacité »`.

**RG-12.5 : Déplacement après l'échéance** `@Invariant`
- **Réf AC :** [US-12] edge
- **Expression formelle :** `d' > échéance(e) → déplacement accepté ∧ en_retard(e)` (RG-6.3).

#### Matrice de cas — US-12

| # | Cas | Type | Input | État | Résultat attendu | Réf règle |
|---|-----|------|-------|------|-------------------|-----------|
| 1 | Mercredi → jeudi | Nominal | bloc déplacé | jeudi a la place | jour_planifié = jeudi, rang et durée inchangés | RG-12.1, RG-12.2 |
| 2 | Même jour, autre heure | Edge | bloc déplacé de 10h à 15h | — | pile inchangée | RG-12.1 |
| 3 | Jour d'arrivée débordé | Edge | bloc déplacé | jeudi plein | question posée, rien d'autre ne bouge | RG-12.3 |
| 4 | Vers un samedi | Edge | — | — | accepté, « hors capacité » | RG-12.4 |
| 5 | Après l'échéance | Edge | — | échéance mercredi | accepté, en retard | RG-12.5 |
| 6 | Bloc posé sur une réunion | Edge | — | réunion existante | voir CONFLIT-9 | RG-12.1 |
| 7 | Bloc supprimé à la main | Edge | — | — | [RENVOI-3] | — |
| 8 | Bloc redimensionné à la main | Edge | — | — | [RENVOI-3] | — |

### SPEC-US-13 : Planifier la semaine le lundi et recalculer chaque matin
**Réf :** US-13, RG-G6

#### Règles métier formalisées

**RG-13.1 : Placement au premier jour qui suffit** `@Invariant`
- **Réf AC :** [US-13/AC-13.1], [US-13/AC-13.3], edge « plus long que chaque jour »
- **Pré-conditions :** pile, capacités, date du jour ; jours candidats J = jours travaillés de la semaine courante à partir du jour d'exécution ([AMB-5]).
- **Post-conditions :** chaque élément non planifié non urgent est placé ou signalé non placé.
- **Expression formelle :**
  ```
  pour e dans pile, par rang croissant, avec jour_planifié(e) indéfini ∧ ¬urgent(e) :
    d* = min { d ∈ J : capacité(d) − charge(d) ≥ durée(e) }
    si d* existe : jour_planifié(e) := d*   (charge(d*) mise à jour pour les suivants)
    sinon        : e reste non planifié ∧ avertissement(e)
  ```
- **Cas d'erreur :** aucun ; l'absence de place est un avertissement.

**RG-13.2 : L'existant ne bouge pas** `@Invariant`
- **Réf AC :** [US-13/AC-13.3], [US-13/AC-13.5]
- **Expression formelle :** `∀ e, jour_planifié(e) défini avant l'exécution → jour_planifié(e) identique après`, sauf application d'une réponse de l'utilisateur (RG-13.5) ou déplacement de bloc dans l'agenda (RG-12.1).

**RG-13.3 : Urgence** `@Invariant`
- **Réf AC :** [US-13/AC-13.4], edge « échéance avant le premier jour disponible », Hypothèse « urgent »
- **Expression formelle :** `urgent(e) ⇔ marqué_urgent(e) ∨ (échéance(e) définie ∧ (d*(e) indéfini ∨ d*(e) > échéance(e)))`, avec d* tel que défini en RG-13.1. `urgent(e) ∧ jour_planifié(e) indéfini → e n'est pas placé automatiquement ∧ question(e)`.

**RG-13.4 : Recalcul du matin** `@Property`
- **Réf AC :** [US-13/AC-13.2]
- **Pré-conditions :** jour travaillé, heure de recalcul atteinte ([RENVOI-7]).
- **Post-conditions :** agendas relus, capacités à jour (RG-8.5), blocs déplacés pris en compte (RG-12.1), une question par jour en débordement.
- **État externe requis :** agendas, date et heure.
- **Simulateur nécessaire :** simulateur d'agendas + horloge simulée (jours, heures, jours fériés, appareil éteint).
- **Expression formelle :** `recalcul(t) → ∀ d ∈ J : déborde(d) → question(d)`. Placement des éléments créés en cours de semaine par le recalcul : [AMB-6].

**RG-13.5 : Questions** `@Property`
- **Réf AC :** [US-13/AC-13.4], [US-13/AC-13.5], [US-13/AC-13.6], RG-G6
- **Pré-conditions :** situation d'urgence (RG-13.3) ou de débordement (RG-8.4, RG-12.3).
- **Post-conditions :** une question visible, avec au moins deux choix concrets dont l'effet est décrit ; tant qu'elle est sans réponse, rien n'est déplacé pour la résoudre ; la réponse est appliquée immédiatement et ferme la question.
- **État externe requis :** questions ouvertes, réponses dans le temps.
- **Simulateur nécessaire :** simulateur d'utilisateur (répond, ne répond pas, répond tard) + horloge.
- **Expression formelle :** `question q ouverte → visible(q) ∧ ∀ e concerné par q : jour_planifié(e), durée(e) inchangés par l'outil` ; `répondre(q, c) → effet(c) appliqué ∧ q fermée ∧ aucun autre changement`. Ordre et expiration : [RENVOI-6].
- **Cas d'erreur :** réponse à une question déjà fermée → refus explicite, rien modifié [DÉRIVÉE].

**RG-13.6 : Exécution sans ouverture de l'outil** `@Property`
- **Réf AC :** [US-13/AC-13.7], edge « ordinateur éteint »
- **Pré-conditions :** heure de recalcul atteinte, outil non ouvert.
- **Post-conditions :** le recalcul a eu lieu ; si l'appareil était éteint, il a lieu au démarrage suivant ; le résultat est prêt à l'ouverture.
- **État externe requis :** état de l'appareil (allumé, éteint), horloge.
- **Simulateur nécessaire :** horloge simulée + cycles marche/arrêt de l'appareil.
- **Expression formelle :** `∀ jour travaillé j, ∃ recalcul r(j) avant la première ouverture de j, ou au premier démarrage après l'heure prévue`. Faisabilité : [CONFLIT-7], [CONFLIT-8].

**RG-13.7 : Idempotence** `@Invariant` [DÉRIVÉE]
- **Réf AC :** [US-13/AC-13.2], [US-13/AC-13.5]
- **Expression formelle :** `recalcul(recalcul(S)) = recalcul(S)` à entrées égales ; une même situation ne produit jamais deux questions ouvertes. Plusieurs appareils : [AMB-22].

**RG-13.8 : Jour de la planification hebdomadaire** `@Invariant`
- **Réf AC :** [US-13/AC-13.1], edge « lundi férié », Hypothèse « premier jour travaillé »
- **Expression formelle :** `planification_hebdo(semaine) a lieu le premier jour d de la semaine tel que d ∈ jours_travaillés ∧ ¬férié_ou_chômé(d)`. Source de `férié_ou_chômé` : [AMB-7].

#### Matrice de cas — US-13

| # | Cas | Type | Input | État | Résultat attendu | Réf règle |
|---|-----|------|-------|------|-------------------|-----------|
| 1 | Lundi, 3 éléments non planifiés | Nominal | planification hebdo | capacité libre | placés au premier jour qui suffit, par rang | RG-13.1 |
| 2 | Élément plus petit après un gros non placé | Edge | rang 1 : 6 h ; rang 2 : 1 h | lundi libre 2 h, reste plein | rang 1 non placé + avertissement, rang 2 lundi | RG-13.1 |
| 3 | Élément plus long que chaque jour | Edge | 8 h | — | non planifié + avertissement | RG-13.1 |
| 4 | Élément déjà planifié jeudi | Nominal | planification | jeudi plein | ne bouge pas | RG-13.2 |
| 5 | Engagement, échéance avant le premier jour possible | Edge | échéance mardi | premier jour possible mercredi | urgent, question, non placé | RG-13.3 |
| 6 | Marqué urgent | Edge | — | — | question, non placé | RG-13.3 |
| 7 | Réunion ajoutée la veille | Nominal | recalcul mardi | mardi déborde d'1 h | question « repousser X à mercredi, ou accepter ? » | RG-13.4, RG-13.5 |
| 8 | Question sans réponse | Edge | — | 2 jours sans réponse | rien déplacé, question visible ; expiration [RENVOI-6] | RG-13.5 |
| 9 | Réponse « repousser X » | Nominal | choix | question ouverte | X déplacé, question fermée, rien d'autre | RG-13.5 |
| 10 | Réponse « réduire la durée » | Nominal | choix | — | durée fixée (correction) | RG-13.5, RG-7.3 |
| 11 | Lundi férié | Edge | — | lundi chômé | planification mardi | RG-13.8 |
| 12 | Ordinateur éteint à l'heure | Edge | — | éteint | recalcul au démarrage | RG-13.6 |
| 13 | Recalcul lancé deux fois | Edge | — | — | même état, pas de question en double | RG-13.7 |
| 14 | Plusieurs questions | Edge | — | 3 ouvertes | toutes visibles ; ordre [RENVOI-6] | RG-13.5 |

### SPEC-US-14 : Déclarer les invariants de mon dépôt
**Réf :** US-14 ; arbitrage « Familles d'invariants v1 » ; plan §Produit

#### Règles métier formalisées

**RG-14.1 : Fichier versionné** `@Invariant`
- **Réf AC :** [US-14/AC-14.1]
- **Expression formelle :** les invariants de dépôt sont lus dans un fichier d'emplacement conventionnel du dépôt ([À DÉFINIR] au cycle de US-14), dans l'état du code analysé. Version (base ou tête) utilisée pour une modification proposée : [CONFLIT-12].

**RG-14.2 : Structure minimale** `@Invariant`
- **Réf AC :** [US-14/AC-14.2], [US-14/AC-14.4]
- **Expression formelle :** `∀ invariant i : nom(i) non vide ∧ description(i) non vide ∧ règle(i) ∈ familles_v1 ∧ paramètres(règle(i)) valides`. La description figure telle quelle dans chaque rapport de violation. La compréhension par un non-spécialiste (AC-14.4) est un test d'acceptation manuel, non mécanisable.

**RG-14.3 : Fichier mal formé** `@Invariant`
- **Réf AC :** [US-14/AC-14.3]
- **Expression formelle :** `mal_formé(F) → refus du fichier entier ∧ message(nom ou position de l'invariant fautif, raison)`.
- **Cas d'erreur :** syntaxe illisible, champ manquant, famille inconnue, paramètre invalide, nom en double (RG-14.4).

**RG-14.4 : Noms uniques** `@Invariant`
- **Réf AC :** [US-14] edge
- **Expression formelle :** `∀ i ≠ j : nom(i) ≠ nom(j)` ; sinon refus (RG-14.3) qui nomme le doublon.

**RG-14.5 : Invariant sans objet** `@Invariant`
- **Réf AC :** [US-14] edge
- **Expression formelle :** `portée(i) ∩ fichiers(dépôt) = ∅ → avertissement « suspect » ∧ verdict non affecté`.

**RG-14.6 : Familles v1** `@Invariant` [ARB]
- **Réf AC :** [US-14/AC-14.2], arbitrage, plan brique 1
- **Expression formelle :**
  - `frontière_import(S, C)` : `∀ f ∈ S, ∀ import x de f : module(x) ∉ C`. Directs seulement ou transitifs : [AMB-14].
  - `type_domaine(notion, T, P)` : dans P, toute valeur relevant de `notion` est de type T, jamais d'un type primitif substituable (ex. montant : `Money`, jamais `float`). Critère de détection : [AMB-14].
  - `appel_interdit(P, F)` : `∀ f ∈ P : aucun appel à F`.

#### Matrice de cas — US-14

| # | Cas | Type | Input | État | Résultat attendu | Réf règle |
|---|-----|------|-------|------|-------------------|-----------|
| 1 | Facturation ↛ e-mails | Nominal | frontière_import(facturation, emails) | — | accepté | RG-14.6 |
| 2 | Description absente | Erreur | invariant sans description | — | refus, nom + raison | RG-14.2, RG-14.3 |
| 3 | Famille inconnue | Erreur | `regle: nommage` | — | refus, nom + raison | RG-14.3 |
| 4 | Noms en double | Erreur | deux « no-mail » | — | refus, doublon nommé | RG-14.4 |
| 5 | Portée vide | Edge | portée `legacy/` inexistante | — | avertissement « suspect », non bloquant | RG-14.5 |
| 6 | Fichier illisible | Erreur | syntaxe cassée | — | refus avec position | RG-14.3 |

### SPEC-US-15 : Vérifier les invariants à chaque modification, sans IA
**Réf :** US-15 ; plan « @Invariant sans état, bloquant, sans LLM en CI »

#### Règles métier formalisées

**RG-15.1 : Sans IA, sans réseau** `@Invariant` `[CLASSIFICATION À VALIDER]`
- **Réf AC :** [US-15/AC-15.1], [US-15/AC-15.2]
- **Expression formelle :** `vérification(code, invariants)` ne sollicite aucun fournisseur d'IA et aucun service réseau ; même comportement en local (`arbitre diff`) et en chaîne de vérification automatique.
- *Classification à valider : propriété de construction (fonction pure) ; sa preuve passe par un observateur réseau.*

**RG-15.2 : Déterminisme** `@Invariant`
- **Réf AC :** [US-15/AC-15.3]
- **Expression formelle :** `∀ code, invariants : vérification(code, invariants)` donne le même verdict et le même rapport (ordre compris), quels que soient la machine, l'ordre de lecture des fichiers et l'environnement.

**RG-15.3 : Verdict** `@Invariant`
- **Réf AC :** [US-15/AC-15.4], [US-15/AC-15.5]
- **Expression formelle :** `échec ⇔ ∃ violation ∨ fichier d'invariants mal formé` ; `succès` sinon. Chaque violation est rapportée avec : nom de l'invariant, description, fichier, ligne. Périmètre (tout l'état résultant ou seulement les violations introduites) : [CONFLIT-17].

**RG-15.4 : Langage non pris en charge** `@Invariant`
- **Réf AC :** [US-15] edge, Hypothèse « Python d'abord »
- **Expression formelle :** `langage(f) ∉ pris_en_charge → f listé « non vérifié » ∧ verdict non affecté`.

**RG-15.5 : Fichier syntaxiquement invalide** `@Invariant`
- **Réf AC :** [US-15] edge
- **Expression formelle :** `invalide(f) → f listé « non vérifiable »`. Effet sur le verdict : [AMB-15].

**RG-15.6 : Pas de fichier d'invariants** `@Invariant`
- **Réf AC :** [US-15] edge
- **Expression formelle :** `fichier_invariants absent → succès ∧ mention « aucun invariant déclaré »`.

#### Matrice de cas — US-15

| # | Cas | Type | Input | État | Résultat attendu | Réf règle |
|---|-----|------|-------|------|-------------------|-----------|
| 1 | Facturation appelle les e-mails | Erreur | import ajouté | invariant frontière | échec, nom + fichier + ligne | RG-15.3 |
| 2 | Modification conforme | Nominal | — | invariants respectés | succès | RG-15.3 |
| 3 | Deux exécutions | Nominal | même code | — | verdict et rapport identiques | RG-15.2 |
| 4 | Hors ligne | Nominal | — | réseau coupé | même verdict | RG-15.1 |
| 5 | Fichier `.ts` | Edge | — | TypeScript non pris en charge | « non vérifié », verdict inchangé | RG-15.4 |
| 6 | Fichier Python invalide | Edge | — | — | « non vérifiable », verdict [AMB-15] | RG-15.5 |
| 7 | Aucun fichier d'invariants | Edge | — | — | succès + mention | RG-15.6 |
| 8 | Fichier d'invariants mal formé | Erreur | — | — | échec, invariant fautif nommé | RG-15.3, RG-14.3 |
| 9 | Violation préexistante | Edge | modification sans rapport | violation déjà là | voir CONFLIT-17 | RG-15.3 |
| 10 | Usage de `float` pour un montant | Erreur | — | type_domaine(montant, Money) | échec | RG-15.3, RG-14.6 |

### SPEC-US-16 : Lire les décisions prises dans une modification plutôt que son code
**Réf :** US-16 ; arbitrages « Extraction », « Persistance des acceptations »

#### Règles métier formalisées

**RG-16.1 : Liste des décisions** `@Invariant` [ARB]
- **Réf AC :** [US-16/AC-16.1]
- **Expression formelle :** `décisions(M) = décisions_structurelles(M) ∪ décisions_IA(M)`. `décisions_structurelles` est déterministe, sans IA, utilisable sans clé, et couvre les catégories : dépendances, API publique, migrations, modules, configuration [ARB] (déclencheurs précis : [À DÉFINIR] au cycle de US-16). Chaque décision porte : résumé en une phrase, catégorie, origine, ≥ 1 emplacement (fichier, lignes). Taille maximale : [RENVOI-9].

**RG-16.2 : Statut de couverture** `@Invariant`
- **Réf AC :** [US-16/AC-16.2]
- **Expression formelle :** `statut(δ) ∈ {respecte, viole, non couverte}`, calculé uniquement à partir de la vérification déterministe (US-15), y compris pour une décision d'origine IA. Définition exacte : [AMB-18].

**RG-16.3 : Décisions IA marquées et non bloquantes** `@Invariant` [ARB]
- **Réf AC :** [US-16/AC-16.1], arbitrage « Extraction »
- **Expression formelle :** `origine(δ) = IA → marquée « proposé par IA » ∧ n'affecte ni le verdict ni le code de retour` ; `fournisseur = ∅ → décisions_IA = ∅` sans erreur.

**RG-16.4 : Acceptation individuelle et persistance** `@Invariant` [ARB]
- **Réf AC :** [US-16/AC-16.3], arbitrage « Persistance »
- **Expression formelle :** `∀ δ : état_revue(δ) ∈ {à relire, acceptée, rejetée}`, modifiable décision par décision ; l'état est écrit dans le fichier de décisions du dépôt, versionné avec la modification. Effet d'un rejet : [AMB-19]. Réécriture d'historique : [CONFLIT-13].

**RG-16.5 : Fichiers propres à l'outil exclus** `@Invariant` [DÉRIVÉE]
- **Réf AC :** [US-16/AC-16.1], [US-17/AC-17.2]
- **Expression formelle :** les modifications du fichier de décisions ne produisent pas de décision. Voir [CONFLIT-14] ; le fichier d'invariants, lui, relève de [CONFLIT-12].

**RG-16.6 : Modification cosmétique** `@Invariant`
- **Réf AC :** [US-16] edge
- **Expression formelle :** `cosmétique(M) → décisions(M) = ∅ ∧ message « aucune décision »`. Définition de `cosmétique` : [CONFLIT-15].

**RG-16.7 : Affichage sur la page de la modification** `@Property`
- **Réf AC :** [US-16/AC-16.4]
- **Pré-conditions :** modification proposée hébergée (GitHub d'abord).
- **Post-conditions :** la liste des décisions est publiée sur la page de la modification.
- **État externe requis :** hébergeur du dépôt, droits de publication.
- **Simulateur nécessaire :** hébergeur simulé (page de PR, échec d'écriture, droits manquants).
- **Expression formelle :** `publier(M) → page(M) contient décisions(M) avec statut et état de revue`.
- **Cas d'erreur :** hébergeur injoignable ou droits manquants → erreur explicite, liste disponible en local, verdict de vérification inchangé [DÉRIVÉE].

#### Matrice de cas — US-16

| # | Cas | Type | Input | État | Résultat attendu | Réf règle |
|---|-----|------|-------|------|-------------------|-----------|
| 1 | 40 fichiers, 5 décisions | Nominal | M | — | 5 décisions, emplacements | RG-16.1 |
| 2 | Ajout d'une dépendance | Nominal | M | — | décision « ajoute une dépendance à X », structurelle | RG-16.1 |
| 3 | Rejet d'une décision | Nominal | rejeter δ2 | — | état écrit dans le fichier de décisions | RG-16.4 |
| 4 | Décision qui viole | Erreur | M | invariant violé | statut `viole` | RG-16.2 |
| 5 | Décision IA | Edge | M | fournisseur configuré | marquée « proposé par IA », verdict inchangé | RG-16.3 |
| 6 | Sans fournisseur | Edge | M | fournisseur = ∅ | décisions structurelles seules, pas d'erreur | RG-16.3 |
| 7 | Mise en forme seule | Edge | M | — | aucune décision + message | RG-16.6 |
| 8 | Renommage d'une fonction publique | Edge | M | — | voir CONFLIT-15 | RG-16.6 |
| 9 | M modifie le fichier de décisions | Edge | M | — | pas de décision pour ce fichier | RG-16.5 |
| 10 | GitHub injoignable | Erreur | publier | — | erreur explicite, liste locale | RG-16.7 |
| 11 | Modification très volumineuse | Edge | M | — | [RENVOI-9] | RG-16.1 |

### SPEC-US-17 : Voir ce qui a changé entre deux itérations d'une modification
**Réf :** US-17

#### Règles métier formalisées

**RG-17.1 : Identité de décision** `@Invariant`
- **Réf AC :** [US-17/AC-17.1], edge « historique réécrit »
- **Expression formelle :** `id(δ)` est calculée sur le contenu (catégorie + objet de la décision), indépendante des commits et des numéros de ligne. Composition exacte : [AMB-17].

**RG-17.2 : Écart entre itérations** `@Invariant`
- **Réf AC :** [US-17/AC-17.1]
- **Expression formelle :** pour les itérations n−1 et n :
  `ajoutées = ids(Dₙ) \ ids(Dₙ₋₁)` ; `retirées = ids(Dₙ₋₁) \ ids(Dₙ)` ; `modifiées = { id ∈ ids(Dₙ) ∩ ids(Dₙ₋₁) : substance(δₙ) ≠ substance(δₙ₋₁) }`. Un simple décalage d'emplacement n'est pas une modification [À VALIDER, AMB-17].

**RG-17.3 : Pas de re-relecture de l'acquis** `@Invariant`
- **Réf AC :** [US-17/AC-17.2]
- **Expression formelle :** `acceptée(id) ∧ id ∉ modifiées → état_revue = acceptée` (non présentée à relire) ; `[DÉRIVÉE] id ∈ modifiées → état_revue := à relire`.

**RG-17.4 : Rejet réapparu** `@Invariant`
- **Réf AC :** [US-17/AC-17.3]
- **Expression formelle :** `rejetée(id) à une itération k < n ∧ id ∉ ids(Dₙ₋₁) ∧ id ∈ ids(Dₙ) → signalement explicite`. Définition à confirmer : [AMB-20].

**RG-17.5 : Calcul sur le contenu** `@Invariant`
- **Réf AC :** [US-17] edges
- **Expression formelle :** `écart(n−1, n)` ne dépend que des états de code (base, tête) des deux itérations et des fichiers de décisions ; `n = 1 → toutes ajoutées`.

#### Matrice de cas — US-17

| # | Cas | Type | Input | État | Résultat attendu | Réf règle |
|---|-----|------|-------|------|-------------------|-----------|
| 1 | 1 retirée, 1 nouvelle | Nominal | itération 2 | itération 1 relue | « 1 retirée, 1 nouvelle », seules à relire | RG-17.2 |
| 2 | Acceptée inchangée | Nominal | itération 2 | δ acceptée | non présentée à relire | RG-17.3 |
| 3 | Acceptée puis modifiée | Edge | itération 2 | δ acceptée | à relire | RG-17.3 |
| 4 | Rejetée qui revient | Edge | itération 3 | δ rejetée en 1, absente en 2 | signalée | RG-17.4 |
| 5 | Lignes décalées seulement | Edge | itération 2 | — | non modifiée [À VALIDER] | RG-17.2 |
| 6 | Push forcé | Edge | itération 2 | historique réécrit | écart sur le contenu | RG-17.5 |
| 7 | Première itération | Edge | itération 1 | — | toutes nouvelles | RG-17.5 |
| 8 | Fichier de décisions perdu au push forcé | Erreur | itération 2 | acceptations absentes | voir CONFLIT-13 | RG-17.3 |

### SPEC-US-18 : Démarrer avec des invariants proposés, preuves à l'appui
**Réf :** US-18 ; plan « Skill d'onboarding », « Piège : inférer fige la dérive »

#### Règles métier formalisées

**RG-18.1 : Invariants candidats** `@Property`
- **Réf AC :** [US-18/AC-18.1]
- **Pré-conditions :** dépôt, fournisseur d'IA configuré.
- **Post-conditions :** liste de candidats, chacun avec nom, description et règle exprimée dans une famille v1 [DÉRIVÉE : un candidat hors famille v1 ne peut pas être vérifié sans IA (AC-18.4)].
- **État externe requis :** réponse du fournisseur d'IA.
- **Simulateur nécessaire :** fournisseur simulé (candidats valides, mal formés, hors famille, aucun).
- **Expression formelle :** `∀ c ∈ candidats : valide_selon_RG-14.2(c)` ; candidat invalide → écarté, non présenté.
- **Cas d'erreur :** réponse du fournisseur inexploitable → message explicite, aucun candidat, rien écrit.

**RG-18.2 : Preuves et contre-exemples calculés sans IA** `@Invariant` [DÉRIVÉE]
- **Réf AC :** [US-18/AC-18.2], [US-18/AC-18.4], edge « candidat déjà violé »
- **Expression formelle :** `∀ c : contre_exemples(c) = violations(vérification(code, {c}))` (US-15) et `preuves(c)` = emplacements conformes dans la portée, comptés de façon déterministe ; ensemble vide → mention « aucun ». L'IA propose, la vérification prouve.

**RG-18.3 : Adoption explicite** `@Invariant`
- **Réf AC :** [US-18/AC-18.3], RG-G6
- **Expression formelle :** `fichier_invariants` n'est modifié que par `adopter(c)` explicite du mainteneur, candidat par candidat ; aucune autre action ne l'écrit.

**RG-18.4 : Un invariant adopté est un invariant comme un autre** `@Invariant`
- **Réf AC :** [US-18/AC-18.4]
- **Expression formelle :** `adopter(c) → c ∈ fichier_invariants ∧ vérifié selon RG-15.*` sans IA.

**RG-18.5 : Fournisseur choisi, et son absence** `@Property`
- **Réf AC :** [US-18/AC-18.5], edge « aucun fournisseur »
- **Pré-conditions :** réglage de fournisseur (US-4).
- **Post-conditions :** la proposition utilise le fournisseur configuré ; sans fournisseur, elle est indisponible avec un message et le reste du module fonctionne.
- **État externe requis :** fournisseur configuré et sa disponibilité.
- **Simulateur nécessaire :** bouchons de fournisseurs (Claude, GPT, Mistral, local, absent).
- **Expression formelle :** `fournisseur = ∅ → proposer() = message « indisponible » ∧ US-14..17 nominales`. Envoi du code au fournisseur : [CONFLIT-16].

**RG-18.6 : Matière insuffisante** `@Invariant`
- **Réf AC :** [US-18] edge « dépôt vide ou minuscule »
- **Expression formelle :** `taille_analysable(dépôt) < seuil → message « pas assez de matière » ∧ aucun candidat`. Seuil : [À DÉFINIR] au cycle de US-18.

#### Matrice de cas — US-18

| # | Cas | Type | Input | État | Résultat attendu | Réf règle |
|---|-----|------|-------|------|-------------------|-----------|
| 1 | « Aucun appel réseau depuis domaine » | Nominal | proposer | 120 conformes, 0 violation | candidat avec 120 preuves, « aucun » contre-exemple | RG-18.1, RG-18.2 |
| 2 | Adoption | Nominal | adopter(c) | — | ajouté au fichier, vérifié sans IA | RG-18.3, RG-18.4 |
| 3 | Candidat déjà violé | Edge | proposer | Money ×47, float ×3 | 3 contre-exemples = violations de la vérification | RG-18.2 |
| 4 | Pas d'adoption | Edge | proposer, ignorer | — | fichier d'invariants inchangé | RG-18.3 |
| 5 | Candidat hors famille v1 | Erreur | réponse fournisseur | — | écarté | RG-18.1 |
| 6 | Dépôt minuscule | Edge | proposer | 2 fichiers | « pas assez de matière » | RG-18.6 |
| 7 | Sans fournisseur | Erreur | proposer | fournisseur = ∅ | indisponible + message, reste OK | RG-18.5 |
| 8 | Fournisseur en panne | Erreur | proposer | injoignable | erreur explicite, rien écrit | RG-18.5, RG-4.5 |

## Dépendances identifiées

| Dépendance | Type | Description | Impact |
|-----------|------|-------------|--------|
| Stockage local durable (SQLite natif) [ARB] | Interne | Stockage transactionnel, résistant à l'arrêt brutal (RG-1.6), refus propre si disque plein (RG-1.7) | Bloquant (tout le cadre) |
| Stockage dans le navigateur (SQLite WASM + OPFS) [ARB] | Externe | Nécessaire pour l'usage hors ligne sur téléphone en PWA (RG-1.1, C-2.9) ; à valider par spike | Bloquant pour US-2 côté téléphone |
| Horloge par champ [ARB] | Interne | Estampilles totalement ordonnées, tie-break par appareil, causalité (CONFLIT-4, CONFLIT-5) | Bloquant pour US-2, RG-1.4 |
| Relais auto-hébergé | Externe (opéré par l'utilisateur) | Recopie des écritures chiffrées ; aucune lecture (RG-2.7, RG-2.8) | Bloquant pour US-2 seulement |
| Secret de chiffrement de l'utilisateur | Interne | Rend les données illisibles sur le relais ; partage entre appareils à définir | Bloquant pour US-2 |
| Google Calendar puis CalDAV [ARB] | Externe | Lecture d'occurrences datées (plusieurs agendas) ; écriture des blocs (un agenda) ; statut de réponse ; marqueur d'appartenance | Bloquant pour US-8, US-10 à US-13 |
| Adaptateur d'agenda (occurrences seulement) [ARB] | Interne | Le socle ne voit jamais une règle de récurrence | Bloquant pour US-8 |
| Protocole MCP | Externe | Surface assistant (RG-4.1) | Bloquant pour US-4 |
| Adaptateur de fournisseurs d'IA | Interne | Claude, GPT, Mistral, modèle local, derrière un seul réglage (RG-4.2, RG-4.3) | Bloquant pour US-18 ; non bloquant ailleurs |
| Planificateur système sur l'ordinateur | Externe | Exécuter le recalcul sans ouverture de l'outil (RG-13.6) | Bloquant pour AC-13.7 (CONFLIT-7) |
| Source des jours fériés | Externe ou agenda | RG-13.8 (AMB-7) | Non bloquant |
| Analyseurs syntaxiques (tree-sitter, parseur de Ruff) | Externe | Analyse Python déterministe, hors ligne (RG-15.1, RG-16.1) | Bloquant pour US-15, US-16 |
| Hébergeur de dépôt (GitHub d'abord) | Externe | Chaîne de vérification automatique (AC-15.1), page de la modification (RG-16.7) | Bloquant pour AC-15.1 (CI) et AC-16.4 ; non bloquant pour l'usage local |
| Fichier de décisions dans le dépôt [ARB] | Interne | Persistance des états de revue partagée entre relecteurs (RG-16.4) | Bloquant pour US-17 |
| Moteur de vérification (US-15) | Interne | Vérité terrain pour statut de couverture (RG-16.2) et preuves (RG-18.2) | Bloquant pour US-16, US-18 |
| Détection de secrets (pre-commit + protection à l'envoi) | Externe | RG-G9 ; protection côté hébergeur encore à activer | Non bloquant (geste de configuration) |

## Conflits et ambiguïtés

### Conflits

| # | Règles en conflit | Description | Résolution proposée |
|---|-------------------|-------------|---------------------|
| [CONFLIT-1] | RG-2.1 vs RG-6.1 | La fusion champ par champ peut produire un élément invalide : A passe un geste en engagement daté avec échéance, B efface l'échéance plus tard → engagement sans échéance. | Après fusion, revalider l'élément ; s'il est invalide, conserver l'état fusionné, le marquer invalide et poser une question à l'utilisateur (RG-G6), les valeurs écartées restant récupérables (RG-G7). |
| [CONFLIT-2] | RG-2.1 vs RG-9.1 | Deux réordonnancements concurrents fusionnés champ par champ peuvent donner deux éléments au même rang, ou un ordre qui ne reflète aucune intention. | Exiger que l'ordre reste total après fusion (départage déterministe des égalités) et signaler le conflit d'ordre comme tout conflit de synchronisation. |
| [CONFLIT-3] | RG-2.3 vs RG-2.1 | « La plus récente l'emporte, suppression ou modification » : la suppression porte sur la donnée, les estampilles sur les champs. Il faut fixer à quoi on compare l'estampille de suppression. | Comparer à l'estampille la plus récente parmi les champs de la donnée (RG-2.3). Une donnée « ressuscitée » retrouve ses effets dérivés (bloc). À valider. |
| [CONFLIT-4] | RG-2.1 (horloge) | « Dernière écriture » selon des horloges d'appareils décalées : une modification faite après en avoir vu une autre peut perdre si l'horloge de l'appareil retarde. | Exiger de l'horloge qu'une écriture faite en connaissance d'une autre soit estampillée après elle (causalité), avec départage par appareil. Choix de l'horloge renvoyé à l'architecture. |
| [CONFLIT-5] | AC-2.6 vs RG-2.1 | « L'utilisateur voit le résultat retenu » suppose de distinguer un conflit d'un écrasement normal ; comparer des estampilles ne suffit pas. | Définir le conflit comme deux écritures dont aucune n'a vu l'autre ; seuls ces cas sont signalés. Les écrasements séquentiels vont aux versions écartées sans signalement. |
| [CONFLIT-6] | AC-5.4 vs US-3 edge | AC-5.4 : un module désactivé n'apparaît pas en ligne de commande ; edge US-3 : un message explicite « module non activé ». | Absent de l'aide, de la complétion et du catalogue MCP ; si on l'appelle explicitement en ligne de commande, message « module non activé » et code d'échec (RG-3.7). |
| [CONFLIT-7] | AC-13.7 vs Hors-scope (application installable) et AC-2.4 | Le cahier ne couvre que l'usage par navigateur. Une PWA fermée ne peut pas lancer de recalcul. Le relais ne peut pas recalculer : il ne fait que recopier et ne lit rien. | Faire tourner le recalcul par le socle natif sur l'ordinateur (commande planifiée par le système), et au démarrage suivant si l'ordinateur était éteint. **À trancher par Quentin.** |
| [CONFLIT-8] | AC-1.2 vs AC-13.7 | AC-1.2 : agenda et IA ne sont contactés « qu'au moment où l'utilisateur les sollicite » ; le recalcul automatique lit l'agenda et y pose des blocs sans sollicitation. | Considérer la connexion d'un agenda comme une sollicitation durable pour la lecture et la pose de blocs lors des recalculs. Ne jamais solliciter un fournisseur d'IA automatiquement. À valider. |
| [CONFLIT-9] | AC-10.4 vs AC-12.1 | L'utilisateur peut glisser un bloc sur une réunion ; l'agenda fait foi, mais AC-10.4 interdit tout chevauchement. | Appliquer AC-10.4 aux seuls blocs placés par l'outil ; une position choisie par l'utilisateur est respectée et le chevauchement signalé. |
| [CONFLIT-10] | US-9 (cas nominal) vs AC-13.3 | « Remonte un sujet… voit son planning de la semaine se mettre à jour » alors que rien de planifié ne doit bouger. | Un réordonnancement n'agit que sur le placement des éléments non planifiés (au prochain placement) ; il ne déplace jamais un élément planifié. |
| [CONFLIT-11] | AC-10.6, AC-8.6 vs AC-11.2 | Reconnaître ses propres événements demande un marqueur ; un marqueur visible des collègues serait une information de plus. | Marqueur opaque, qui ne porte aucune information sur l'élément, placé si possible dans un champ invisible des invités. |
| [CONFLIT-12] | US-14/US-15 vs RG-G6 | Si la vérification lit le fichier d'invariants de la tête, un agent peut retirer ou affaiblir un invariant dans sa propre PR et passer la vérification. | Toute modification du fichier d'invariants est remontée comme décision structurelle à relire, quelle que soit la version lue. Version lue (base ou tête) : **à trancher par Quentin.** |
| [CONFLIT-13] | Arbitrage « persistance dans le dépôt » vs US-17 edge « historique réécrit » et RG-G7 | Les acceptations vivent dans la branche de la PR ; un push forcé de l'agent peut les effacer sans bruit. | Si le fichier de décisions de la tête perd des états de revue présents à l'itération précédente, le signaler explicitement. **À trancher par Quentin** (signalement seul, ou reconstitution). |
| [CONFLIT-14] | Arbitrage « persistance dans le dépôt » vs AC-16.1 | Le fichier de décisions, commité avec la PR, fait partie du diff et deviendrait lui-même une décision. | Exclure ce fichier de l'extraction et de la vérification (RG-16.5). |
| [CONFLIT-15] | US-16 edge « renommage cosmétique » vs RG-16.1 (catégorie API publique) | Renommer une fonction publique est un changement d'API, pas un geste cosmétique. | Cosmétique = mise en forme, commentaires, renommage de symboles non exportés. Un renommage d'API publique est une décision. À valider. |
| [CONFLIT-16] | Plan (« le code ne sort jamais ») vs AC-18.5 / AC-1.2 | Avec un fournisseur distant, la proposition d'invariants envoie le code au fournisseur. Le cadrage ne dit rien ; AC-1.2 l'autorise sur sollicitation. | AC-1.2 s'applique (envoi permis sur sollicitation) ; prévenir le mainteneur avant le premier envoi du code à un fournisseur distant. Le plan reste vrai avec un modèle local. À valider. |
| [CONFLIT-17] | AC-15.4 (« une modification qui viole ») vs AC-15.3 et US-18 edge (« adopter et doit alors corriger ») | Le verdict porte-t-il sur tout l'état résultant (les violations préexistantes font échouer) ou seulement sur les violations introduites ? | Verdict sur tout l'état résultant, ce qui va avec AC-15.3 (même code, même verdict) et avec l'edge US-18. **À trancher par Quentin.** |

### Ambiguïtés

| # | Règles concernées | Description | Résolution proposée |
|---|-------------------|-------------|---------------------|
| [AMB-1] | RG-8.2, Hypothèse « journée entière » | Un événement « journée entière » qui n'est pas une absence (anniversaire, rappel) ramènerait la capacité à 0. | Ne mettre à 0 que les journées entières marquées occupé ou absence. À valider. |
| [AMB-2] | RG-8.1 | Statuts « peut-être », « sans réponse », événements marqués « disponible » : comptés ou non ? | [À DÉFINIR] au cycle de US-8. |
| [AMB-3] | RG-8.3 vs RG-10.2 | Un geste de 10 min compte 10 min de charge, mais réserve 30 min dans l'agenda. | Charge = durées des éléments (lecture littérale d'AC-8.3). |
| [AMB-4] | RG-8.3 | Un élément fait dans la journée sort-il de la charge du jour ? | Oui (seuls les actifs comptent). À valider. |
| [AMB-5] | RG-13.1 | Jours candidats : le jour d'exécution inclus ? jusqu'au vendredi seulement ? | Du jour d'exécution au vendredi inclus. |
| [AMB-6] | RG-13.4 | Le recalcul du matin place-t-il les éléments créés en cours de semaine ? AC-13.2 ne le dit pas. | Oui, selon RG-13.1, sur les jours restants de la semaine. À valider. |
| [AMB-7] | RG-13.8 | D'où vient la connaissance d'un lundi férié ? | Uniquement de l'agenda (absence, journée entière) dans ce cycle ; pas de calendrier national. À valider. |
| [AMB-8] | RG-8.2, RG-6.3 | Fuseau horaire et limites de jour (voyage, heure d'été). | Fuseau local de l'utilisateur au jour considéré. À valider. |
| [AMB-9] | RG-7.5 | Les durées proposées des éléments existants sont-elles recalculées après une correction ? (AC-7.3 dit « éléments suivants ».) | Non : une durée proposée, une fois attribuée, ne change pas en silence. |
| [AMB-10] | RG-6.2, RG-G7 | Un élément fait peut-il redevenir actif ? Une suppression faite par l'utilisateur est-elle récupérable ? | [À DÉFINIR] au cycle de US-6. |
| [AMB-11] | RG-4.6 | « Déplacer » = changer le jour planifié, le rang, ou les deux ? Comment reconnaître l'assistant quand il passe par la ligne de commande ? | Les deux ; l'auteur « assistant » = action reçue par la surface MCP. |
| [AMB-12] | RG-5.2 | L'activation d'un module est-elle propre à l'appareil ou synchronisée ? Les données d'un module désactivé continuent-elles à se synchroniser ? | Activation propre à l'appareil ; données toujours synchronisées. À valider. |
| [AMB-13] | RG-4.5 | Délai au-delà duquel un modèle est « trop lent ». | [À DÉFINIR] au cycle de US-4. |
| [AMB-14] | RG-14.6 | Frontière d'import directe ou transitive ? Comment reconnaître une « valeur qui relève d'une notion » en Python non annoté ? | Directe en v1 (cas nominal « directement ») ; critère de type du domaine [À DÉFINIR] au cycle de US-14. |
| [AMB-15] | RG-15.5 | Un fichier invalide qui tombe dans la portée d'un invariant pourrait cacher une violation. | Échec si le fichier est dans la portée d'au moins un invariant, simple mention sinon. À valider. |
| [AMB-16] | RG-3.5, RG-15.3 | Une violation et une erreur d'outil (fichier d'invariants mal formé) ont-elles le même code de retour ? | Codes distincts, pour que la CI distingue « code non conforme » de « outil cassé ». |
| [AMB-17] | RG-17.1, RG-17.2 | Composition de l'identité d'une décision ; un décalage de lignes compte-t-il comme une modification ? | [À DÉFINIR] au cycle de US-17 ; le décalage seul ne compte pas. |
| [AMB-18] | RG-16.2 | Définition de « respecte » / « viole » / « non couverte » pour une décision. | `viole` si une violation tombe sur ses emplacements ; `respecte` si un invariant couvre ses emplacements sans violation ; `non couverte` sinon. À valider. |
| [AMB-19] | RG-16.4 | Le rejet d'une décision a-t-il un effet (bloquer la fusion, faire échouer la CI) ou n'est-il qu'un signal ? | [À DÉFINIR] au cycle de US-16. |
| [AMB-20] | RG-17.4 | « Rejetée qui réapparaît » : après une absence, ou toujours présente ? | Après une absence (RG-17.4) ; une décision rejetée restée présente garde l'état « rejetée ». |
| [AMB-21] | RG-1.6 | « Confirmée » = l'outil a dit à l'utilisateur que la modification est faite, quelle que soit la surface. | Retenue telle quelle. |
| [AMB-22] | RG-10.7, RG-13.7 | Plusieurs appareils peuvent recalculer, poser des blocs et créer des questions en parallèle → doublons. | Au plus un bloc par élément, un bloc quotidien par jour, une question par situation, quel que soit l'appareil. |

### Manques (absents du cahier, présents dans le plan ou nécessaires)

| # | Source | Description | Résolution proposée |
|---|--------|-------------|---------------------|
| [MANQUE-1] | Plan, brique 2 | Harnais d'évals (PR d'agents, violations = labels, rappel et précision de l'extracteur, 30–50 cas annotés) : aucune US. | Outil de développement, pas fonction produit : au Planner de l'inscrire comme tâche de la partie C. À confirmer par Quentin. |
| [MANQUE-2] | Plan, « Multi-repo : extends dès le départ » | Héritage d'un dépôt de règles d'organisation : aucune US. | Hors v1 ; à confirmer : « dès le départ » vise-t-il seulement la forme du fichier d'invariants ? |
| [MANQUE-3] | Plan, « Skill d'onboarding » | Les 5 à 8 questions triées par impact absentes d'US-18. | À ajouter à US-18 ou à écarter, par Quentin. |
| [MANQUE-4] | Plan, Risques | Filtrage du bruit de l'extracteur (frontières, types du domaine) et mesure de précision : aucune AC. | Lié à MANQUE-1. |
| [MANQUE-5] | US-7 edge | Valeurs des durées par défaut par type. | [À DÉFINIR] au cycle de US-7. |
| [MANQUE-6] | US-2 edge | Le cahier protège la lecture sur le relais, pas l'intégrité (altération ou rejeu par un tiers). | Exiger qu'une donnée altérée sur le relais soit rejetée par les appareils. À valider. |

### Renvois au cycle de chaque story (aucune option proposée)

| # | Question | Story |
|---|----------|-------|
| [RENVOI-1] | Élément plus long qu'une journée | US-7, US-10, US-13 |
| [RENVOI-2] | Créneau insuffisant (capacité suffisante, aucun créneau continu) | US-10 |
| [RENVOI-3] | Bloc supprimé ou redimensionné à la main dans l'agenda | US-12 |
| [RENVOI-4] | Sujet public par défaut | US-11 |
| [RENVOI-5] | Bloc des petits gestes (heure, taille maximale ; bloc vide ; bloc déplacé à la main) | US-10, US-12 |
| [RENVOI-6] | Ordre et expiration des questions | US-13 |
| [RENVOI-7] | Heure du recalcul et de la planification hebdomadaire | US-13 |
| [RENVOI-8] | Plage de travail (réunions hors horaires, placement des blocs dans la journée) | US-8, US-10 |
| [RENVOI-9] | Taille maximale d'une modification pour decision diff | US-16 |

### Règles globales non formalisables en règle produit
- **RG-G1** (ordre de développement) : contrainte de découpe, à porter par le Planner.
- **RG-G8** (10 entretiens avant tout nouveau module) : règle de gouvernance, hors logiciel.
- **RG-G9** (public, Apache 2.0, aucun secret) : hygiène du dépôt, tenue par pre-commit et la protection côté hébergeur, pas par une règle du produit.

## Invariants candidats (synthèse C-TDD)

### @Invariant (stateless, bloquant)
- `INV-1` : `import(∅, export(S)) ≡ S` [CLASSIFICATION À VALIDER] — source : RG-1.3
- `INV-2` : `import(T, export(S)) = fusion(T, S)` — source : RG-1.4
- `INV-3` : fichier d'import invalide → état inchangé — source : RG-1.5
- `INV-4` : fusion d'un champ = plus grande estampille, départage déterministe — source : RG-2.1
- `INV-5` : fusion commutative, associative, idempotente — source : RG-2.2
- `INV-6` : `supprimée(d) ⇔ t_s(d) > max_f t(d,f)` — source : RG-2.3
- `INV-7` : toute valeur écartée par une fusion est récupérable ; un conflit est signalé — source : RG-2.4
- `INV-8` : aucun relais par défaut ; sans relais, outil complet — source : RG-2.7
- `INV-9` : contenu envoyé au relais indéchiffrable sans le secret de l'utilisateur [CLASSIFICATION À VALIDER] — source : RG-2.8
- `INV-10` : `catalogue(GUI) ⊆ catalogue(CLI) ⊆ catalogue(MCP)`, effets identiques — source : RG-3.1, RG-4.1
- `INV-11` : `arbitre plan` → planning, `arbitre diff` → decision diff — source : RG-3.2
- `INV-12` : commande mal formée → usage, état inchangé, code ≠ succès — source : RG-3.3
- `INV-13` : sortie humaine et sortie structurée équivalentes — source : RG-3.4
- `INV-14` : échec → code ≠ succès — source : RG-3.5
- `INV-15` : module non activé appelé → message, code ≠ succès, état inchangé — source : RG-3.7
- `INV-16` : `{Claude, GPT, Mistral, local} ⊆ fournisseurs_sélectionnables` — source : RG-4.2
- `INV-17` : `¬requiert_IA(f) → disponible(f)` sans fournisseur — source : RG-4.4
- `INV-18` : suppression ou déplacement via MCP → historique (date, « assistant ») — source : RG-4.6
- `INV-19` : action invalide → refus, état inchangé — source : RG-4.7
- `INV-20` : tout sous-ensemble de modules activés fonctionne seul — source : RG-5.1
- `INV-21` : désactiver puis réactiver conserve les données — source : RG-5.2
- `INV-22` : un module ne lit ni n'écrit les données d'un autre — source : RG-5.3
- `INV-23` : module désactivé absent de l'interface, de l'aide CLI et du catalogue MCP — source : RG-5.4
- `INV-24` : données importées d'un module désactivé conservées, visibles à l'activation — source : RG-5.5
- `INV-25` : `valide(e)` (intitulé, type, engagement → échéance) — source : RG-6.1
- `INV-26` : transitions de statut ; fait → hors pile, dans l'historique — source : RG-6.2
- `INV-27` : définition de `en_retard` — source : RG-6.3
- `INV-28` : même entrée, même résultat sur toutes les surfaces — source : RG-6.4
- `INV-29` : `durée > 0` — source : RG-7.1
- `INV-30` : tout nouvel élément a une durée, origine `proposée`, sans IA — source : RG-7.2
- `INV-31` : correction → origine `fixée` ; une durée fixée n'est jamais réécrite par une proposition — source : RG-7.3
- `INV-32` : sans correction semblable → `défaut(type)` — source : RG-7.4
- `INV-33` : `semblable ⇒ même type ∧ proximité ≥ seuil`, déterministe, sans IA [CLASSIFICATION À VALIDER] — source : RG-7.6
- `INV-34` : réunion comptée ⇔ en lecture ∧ non déclinée ∧ non de l'outil ∧ non journée entière — source : RG-8.1
- `INV-35` : formule de capacité (union des intervalles, plancher 0, week-end et absence à 0) — source : RG-8.2
- `INV-36` : `charge(d) = Σ durées des actifs planifiés en d` — source : RG-8.3
- `INV-37` : `déborde ⇔ charge > capacité`, excès exposé sur toutes surfaces — source : RG-8.4
- `INV-38` : pile = actifs, ordre total strict — source : RG-9.1
- `INV-39` : déplacer(e, p) ne change que la position de e — source : RG-9.2
- `INV-40` : chaque élément expose type, durée+origine, échéance, jour, marque non planifié — source : RG-9.3
- `INV-41` : pile entière sans troncature ; pile vide → message — source : RG-9.5
- `INV-42` : un bloc par élément ≥ 30 min, de durée = durée de l'élément — source : RG-10.1
- `INV-43` : `P(d) ≠ ∅ → un bloc quotidien de max(30, Σ)` — source : RG-10.2
- `INV-44` : bloc placé par l'outil ∩ réunion comptée = ∅ — source : RG-10.3
- `INV-45` : bloc : occupé, « Focus — » + sujet affiché — source : RG-11.1
- `INV-46` : contenu visible = titre, début, fin, occupé — source : RG-11.2
- `INV-47` : sujet public indépendant de l'intitulé — source : RG-11.3
- `INV-48` : titre du bloc quotidien constant, sans intitulé de geste — source : RG-11.4
- `INV-49` : sans sujet public, titre ≠ « Focus — » + intitulé — source : RG-11.5
- `INV-50` : prise en compte d'un déplacement → seul `jour_planifié(e)` change — source : RG-12.2
- `INV-51` : déplacement vers un jour à capacité 0 → accepté, « hors capacité » — source : RG-12.4
- `INV-52` : déplacement après l'échéance → accepté, en retard — source : RG-12.5
- `INV-53` : placement au premier jour qui suffit, par rang ; sinon avertissement — source : RG-13.1
- `INV-54` : planification et recalcul ne changent aucun jour déjà planifié — source : RG-13.2
- `INV-55` : définition de `urgent` ; urgent non planifié → question, pas de placement — source : RG-13.3
- `INV-56` : recalcul idempotent ; une question par situation — source : RG-13.7
- `INV-57` : planification hebdo au premier jour travaillé non chômé — source : RG-13.8
- `INV-58` : invariants de dépôt lus dans le fichier versionné du dépôt — source : RG-14.1
- `INV-59` : nom, description, règle d'une famille v1 aux paramètres valides — source : RG-14.2
- `INV-60` : fichier mal formé → refus total, invariant fautif et raison — source : RG-14.3
- `INV-61` : noms uniques — source : RG-14.4
- `INV-62` : portée vide → avertissement, verdict inchangé — source : RG-14.5
- `INV-63` : sémantique des trois familles v1 — source : RG-14.6
- `INV-64` : vérification sans IA ni réseau [CLASSIFICATION À VALIDER] — source : RG-15.1
- `INV-65` : vérification déterministe (verdict et rapport) — source : RG-15.2
- `INV-66` : `échec ⇔ violation ∨ fichier mal formé` ; violation → nom, description, fichier, ligne — source : RG-15.3
- `INV-67` : langage non pris en charge → « non vérifié », verdict inchangé — source : RG-15.4
- `INV-68` : fichier invalide → « non vérifiable » — source : RG-15.5
- `INV-69` : pas de fichier d'invariants → succès + mention — source : RG-15.6
- `INV-70` : décisions structurelles déterministes, sans IA ; chaque décision : résumé, catégorie, origine, emplacements — source : RG-16.1
- `INV-71` : statut de couverture issu de la seule vérification déterministe — source : RG-16.2
- `INV-72` : décision IA marquée, sans effet sur verdict ni code de retour — source : RG-16.3
- `INV-73` : état de revue par décision, persisté dans le fichier de décisions — source : RG-16.4
- `INV-74` : le fichier de décisions ne produit pas de décision — source : RG-16.5
- `INV-75` : modification cosmétique → aucune décision + message — source : RG-16.6
- `INV-76` : identité de décision calculée sur le contenu — source : RG-17.1
- `INV-77` : écart = ajoutées, retirées, modifiées par identité — source : RG-17.2
- `INV-78` : acceptée inchangée non re-présentée ; modifiée → à relire — source : RG-17.3
- `INV-79` : rejetée qui réapparaît → signalée — source : RG-17.4
- `INV-80` : écart indépendant de l'historique ; première itération → toutes nouvelles — source : RG-17.5
- `INV-81` : contre-exemples d'un candidat = violations de la vérification ; preuves comptées sans IA — source : RG-18.2
- `INV-82` : le fichier d'invariants n'est écrit que par une adoption explicite — source : RG-18.3
- `INV-83` : invariant adopté vérifié comme tout invariant, sans IA — source : RG-18.4
- `INV-84` : dépôt trop petit → message, aucun candidat — source : RG-18.6

### @Property (stateful, nécessite simulateur)
- `PROP-1` : toute opération sur toute donnée réussit hors ligne — simulateur : réseau coupé — source : RG-1.1
- `PROP-2` : aucun envoi hors {agendas connectés, fournisseur choisi, relais} ; agenda et IA sur sollicitation — simulateur : observateur réseau — source : RG-1.2
- `PROP-3` : modification confirmée présente après arrêt brutal ; non confirmée tout ou rien — simulateur : arrêt brutal à instant arbitraire — source : RG-1.6
- `PROP-4` : disque plein → refus, état intact — simulateur : stockage plein — source : RG-1.7
- `PROP-5` : propagation < 60 s (connectés, ou après reconnexion) — simulateur : deux appareils + relais + connectivité + horloge — source : RG-2.5
- `PROP-6` : relais arrêté → appareils complets, reprise seule, date de dernière synchro affichée — simulateur : relais commandable + horloge — source : RG-2.6
- `PROP-7` : écritures concurrentes CLI/GUI sans perte silencieuse — simulateur : deux écrivains locaux entrelacés — source : RG-3.6
- `PROP-8` : changement de fournisseur → données et catalogue identiques — simulateur : bouchons de fournisseurs — source : RG-4.3
- `PROP-9` : fournisseur défaillant → erreur explicite pour les seules fonctions dépendantes — simulateur : fournisseur injoignable / clé refusée / lent / absent — source : RG-4.5
- `PROP-10` : élément fait ou supprimé → bloc retiré — simulateur : agenda en écriture — source : RG-6.5, RG-10.4
- `PROP-11` : proposition de durée rapprochée des corrections semblables — simulateur : générateur d'historiques de corrections — source : RG-7.5
- `PROP-12` : capacité à jour au recalcul suivant ; agenda injoignable → dernière connue « non à jour » + date — simulateur : agendas multiples avec changements et pannes — source : RG-8.5
- `PROP-13` : aucun événement d'agenda ne change un rang — simulateur : agenda (déplacements, ajouts, suppressions) — source : RG-9.4
- `PROP-14` : l'outil ne modifie ni ne supprime jamais un événement qu'il n'a pas créé (bloquant) — simulateur : agenda à origines mêlées — source : RG-10.5
- `PROP-15` : agenda injoignable → pile inchangée ; opérations appliquées à la reprise sans doublon — simulateur : agenda avec panne et reprise — source : RG-10.6
- `PROP-16` : au plus un bloc par élément, un bloc quotidien par jour, multi-appareils — simulateur : deux appareils + agenda — source : RG-10.7
- `PROP-17` : bloc déplacé à un autre jour → jour planifié suivi au recalcul — simulateur : agenda avec déplacements manuels — source : RG-12.1
- `PROP-18` : déplacement qui fait déborder → question, rien d'autre ne bouge — simulateur : agenda (déplacement + réunions) — source : RG-12.3
- `PROP-19` : recalcul du matin : relecture, capacités, blocs déplacés, question par jour débordé — simulateur : agendas + horloge — source : RG-13.4
- `PROP-20` : question ouverte → rien déplacé ; réponse → appliquée, question fermée — simulateur : utilisateur simulé + horloge — source : RG-13.5
- `PROP-21` : recalcul sans ouverture ; au démarrage si éteint — simulateur : horloge + cycles marche/arrêt — source : RG-13.6
- `PROP-22` : publication des décisions sur la page de la modification ; échec → erreur explicite, liste locale — simulateur : hébergeur de dépôt simulé — source : RG-16.7
- `PROP-23` : candidats valides et exprimés dans une famille v1 — simulateur : fournisseur simulé (valides, mal formés, hors famille, aucun) — source : RG-18.1
- `PROP-24` : proposition par le fournisseur choisi ; sans fournisseur, indisponible et reste du module nominal — simulateur : bouchons de fournisseurs — source : RG-18.5

## Arbitrages de Quentin sur les specs (2026-11-02)

Ils tranchent les points structurants de « Conflits et ambiguïtés » et font foi. Pour les autres CONFLIT, AMB et MANQUE, la résolution proposée ci-dessus vaut par défaut, à revoir au cycle de chaque story.

- **CONFLIT-4 et 5 (horloge, signalement)** : horloge logique hybride (HLC), avec départage par identifiant d'appareil. Un conflit signalé est un champ modifié des deux côtés depuis la dernière synchronisation ; un écrasement séquentiel va aux versions écartées sans signalement. Pas de vecteurs de versions.
- **CONFLIT-7 et 8 (recalcul outil fermé)** : le recalcul tourne par la CLI native sur l'ordinateur, planifiée par le système (launchd sur macOS, timer systemd sur Linux). Une exécution manquée est rattrapée au démarrage suivant. Le téléphone reçoit le résultat par la synchronisation. Connecter un agenda vaut accord durable pour le lire et y poser des blocs pendant les recalculs ; un fournisseur d'IA n'est jamais sollicité automatiquement.
- **MANQUE-6 (intégrité sur le relais)** : chiffrement authentifié (AEAD, par ex. XChaCha20-Poly1305) ; une donnée altérée échoue au déchiffrement et l'appareil la rejette. Le rejeu est inoffensif par construction (HLC, la dernière écriture gagne). Deux conditions, pour pouvoir ajouter plus tard une signature par appareil sans rien refaire : l'enveloppe chiffrée porte un numéro de format, et chaque écriture porte l'identifiant de l'appareil auteur.
- **AMB-12 (activation des modules)** : l'activation est propre à chaque appareil ; les données de tous les modules se synchronisent toujours.
- **CONFLIT-12 (version des invariants lue)** : la vérification applique l'union des invariants de la base et de la tête. Toute modification du fichier d'invariants remonte comme décision à relire. En plus, **un invariant affaibli ou retiré dans la PR déclenche une alerte explicite et distincte**, jamais noyée parmi les autres décisions.
- **CONFLIT-13 (acceptations effacées par réécriture d'historique)** : signalement explicite (« N acceptations perdues depuis l'itération précédente ») ; les décisions concernées repassent « à relire ». Pas de reconstitution.
