# Revue Devil's Advocate — OpenArbitre, cadre global (découpe en stories), plan v3

- Inputs : `01_story.md`, `02_specs.md`, `04_plan.md` (v3) ; contexte `03_archi.md`, `~/.claude/commands/orchestrateur.md` ; revue précédente `history/05_da_review_v2.md`. Les sections « Arbitrages de Quentin » de 01, 02, 03 et de la revue v2 (F-10) font foi.
- Consigne de Quentin, appliquée ici : le plan ne date aucun jalon au-delà de ST-04. Cette revue ne le reproche pas. Elle vérifie que le recalibrage est exécutable et que l'ordonnancement respecte le plafond.
- Seuil BLOQUANT : ce qui empêche de lancer le cycle de ST-01, ou crée un trou de sécurité.
- Date : 2026-11-03 · Itération DA : 3 / 3.
- Biais à signaler : le plan v3 et cette revue sont produits dans la même session. La revue s'appuie donc sur des vérifications externes au plan (skill orchestrateur, specs, archi) plutôt que sur une relecture de son raisonnement.

## Verdict global
**VALIDÉ**, avec deux conditions et des reprises au cycle des stories concernées.

Sur les 16 failles de la v2, 13 sont résolues et 3 partiellement. Aucune ne reste ouverte. Le calendrier est désormais cohérent : la durée vaut taille + arbitrage, le socle est ordonnancé avec cette unité, et le plafond découle d'une formule qui ne prend que des mesures de ST-01 à ST-04. Rien n'empêche de lancer ST-01. Deux points doivent toutefois être réglés à un moment précis, sinon ils ne pourront plus l'être :
- **avant la soirée 1** : écrire les horodatages de mesure (F-40). Sans eux, le recalibrage n'a aucune donnée, et ce manque ne se rattrape pas après coup ;
- **dans la story de ST-30, avant toute story qui envoie du code** : fermer la définition du fournisseur « local » (F-41).

---

## Suivi des failles de la v2

| Faille v2 | Statut | Preuve |
|---|---|---|
| **F-24** (B) jalons hors des unités | **Résolue** | La légende définit la durée comme taille + arbitrage (« S faible = 65 min ; … L forte = 210 min »). Le socle est recompté avec cette unité : 580 min, 3,9 soirées, ST-01 sur 160 min à cheval sur les soirées 1 et 2. Au-delà de ST-04, il n'y a plus de dates, seulement des cumuls en minutes. La phrase « 5 semaines » et la marge fictive ont disparu. |
| **F-25** recalibrage non mesurable | **Partiellement** | Le recalibrage ne mesure plus que deux facteurs d'échelle (`k_m`, `k_a`) et la part d'arbitrage `f`, sur les sommes de ST-01 à ST-04. Il applique le même facteur à S, L et « faible », et le dit comme une hypothèse. Le biais des premiers cycles est assumé comme prudent. Les seuils ambigus sont remplacés par des formules (N, budget). La branche N = 1 dit ce que devient la démo (PL-9). Reste un manque : l'outil censé produire les mesures ne sait pas le faire (F-40). |
| **F-26** port `Store` gelé sans iOS | **Résolue** | PL-8 : le port est asynchrone d'office, ce qui couvre OPFS par worker, IndexedDB et le cœur dans un worker. S1-B ne choisit plus qu'un adaptateur. L'archi (`03_archi.md`, « Port Store ») ne fixe ni synchrone ni asynchrone, il n'y a donc pas de contradiction. |
| **F-27** propriété et amendement | **Résolue** | Le registre `tests/support/surfaces/` permet à ST-07 et ST-21 d'ajouter un fichier sans toucher le test générique. Les signaux à l'ouverture sont posés en ST-04, et ST-11 comme ST-19 n'ont plus besoin de `main.rs`. La procédure d'amendement est écrite : mini-cycle fort, gel des ouvertures, rebase. ST-30 passe en voie D avec `wiring/ai.rs`. |
| **F-28** recalcul → synchro → téléphone | **Partiellement** | `arbitre tache-de-fond` et le registre `taches.rs` existent. La chaîne est vérifiée en ST-19 (simulateur) et en ST-20 (téléphone réel), et l'arête ST-11 → ST-19 est déclarée. Mais l'ordre « synchro avant et après le recalcul » est incompatible avec la règle « ajout en fin » du registre (F-42). |
| **F-29** plafond violé | **Résolue** | Règle 1 : spikes et gestes moyens ou forts sont des cycles, G-3 compris. Règle 5 : seule l'attente de l'admin pour S3 n'occupe aucune place. S3 est coupé en S3-L (faible) et S3-V (fort). Socle vérifié soirée par soirée : jamais plus de 2 cycles, jamais deux forts, points ≤ 8. |
| **F-30** S3 sans borne | **Résolue** | Trois issues sont écrites. « Pas de verdict à la fusion de ST-16 » ⇒ ST-17. Reste un cas limite mineur (F-48). |
| **F-31** couverture perdue en repli | **Résolue** | ST-17 reprend un par un les critères de ST-10 et ST-15b : PROP-14 côté adaptateur, PROP-12, AC-8.5, « non à jour », trousseau, essai réel de visibilité. R-4 et la table des invariants sont mis à jour. |
| **F-32** consentement IA par MCP | **Partiellement** | La sous-commande `ia consentir` est hors catalogue, exige un terminal et n'apparaît pas au MCP. Le consentement est hors `Store`, nommé par fournisseur, et le critère « appel MCP sans consentement ⇒ zéro sortie » est posé en ST-30 et ST-31. Restent l'exemption « modèle local », non définie (F-41), et le contournement par pseudo-terminal, que le plan n'assume qu'en partie (F-49). |
| **F-33** arêtes manquantes | **Résolue** | ST-20 : ST-16, ST-08, avec la dépendance de compilation dite. ST-21 : ST-06, ST-08, ST-13. ST-32 : ST-31. ST-19 : ST-11. La section Voies dit que l'ordre ne repose plus que sur ces arêtes. |
| F-34 sel concurrent | **Résolue** | Le relais crée le sel atomiquement ; critère « un seul sel, même clé » en ST-19. |
| F-35 jalons sur simulateur | **Résolue** | Libellés « sur agenda simulé » ; jalon « Planning sur agenda réel » rattaché à ST-15b ou ST-17. |
| F-36 zones mixtes | **Résolue** | `zones.toml` par story, Full par défaut. |
| F-37 PROP-1 autodéclaré | **Résolue** | Conformance : une action déclarée réseau, réseau coupé, rend une issue explicite ; critère sur une action factice `relais`. |
| F-38 AMB-16 | **Résolue** | Retiré de ST-22 ; le rouvrir est un amendement du socle. |
| F-39 dates dès aujourd'hui | **Résolue** | La soirée 1 est la première soirée après le verdict VALIDÉ ; aucune date absolue. |

Reste de F-14 (v1) : **résolu**. ST-10 et ST-17 vérifient la lecture réelle hors session par le job de ST-11, et ST-09 nomme l'erreur « identifiants indisponibles ou expirés » du port `Agenda`.

---

## Contrôle demandé : recalibrage exécutable, plafond respecté

- **Exécutable**, à une condition (F-40). Les trois nombres viennent de sommes sur quatre fichiers. N et le budget sortent de formules fermées. Avec les estimations, on retombe sur N = 2 et sur 8 points, ce qui est cohérent avec la v2. Le déroulé de l'étape 5 est déterministe : règles 1 à 7, et un fort qui attend démarre à la fin du fort en cours. Deux imprécisions n'empêchent pas de l'exécuter (F-44, F-45).
- **Le plafond est respecté par construction** : l'étape 5 date les jalons en appliquant les règles qui définissent le plafond. Le seul ordonnancement explicite, celui du socle, a été vérifié ligne par ligne (cycles, forts, points).

---

## Failles nouvelles (v3)

### [MAJEUR] F-40 : Personne n'écrit les horodatages dont dépend le recalibrage
- **Axe** : Complétude
- **Source** : `04_plan.md` § Mesures posées par G-4 (« L'orchestrateur écrit ces horodatages à chaque passage de main, sans saisie de Quentin ») ; `~/.claude/commands/orchestrateur.md` § « Format de state.json » (statut par phase seulement, aucun champ `debut`, `fin` ni `portes`) ; § Recalibrage, étape 1.
- **Tâche(s) concernée(s)** : G-4, recalibrage, ST-01 à ST-04
- **Description** : le skill orchestrateur ne connaît ni les portes ni les horodatages, ni `.ctdd/soirees.log`. G-4 est noté « faible » et ne prévoit pas de modifier le skill. Si ST-01 démarre sans ce changement, ses mesures sont perdues. Or ST-01 pèse 160 des 580 min de l'échantillon, et ses deux portes fortes en font la story la plus chargée en arbitrage.
- **Preuve** : le recalibrage ne mesure que ST-01 à ST-04, et une mesure ne se reconstitue pas après coup. Cela n'empêche pas de lancer ST-01, mais c'est le dernier moment où le manque peut être évité. **Condition du verdict : à régler avant la soirée 1.**

### [MAJEUR] F-41 : L'exemption « modèle local » n'est pas définie, et le réglage du fournisseur n'est pas placé hors MCP
- **Axe** : Sécurité & intégrité
- **Source** : ST-30 (« Un modèle local n'en a pas besoin » ; « le consentement… nomme le fournisseur ») ; RG-4.2 et RG-4.3 (« un seul réglage » choisit le fournisseur, dont « modèle local ») ; CONFLIT-16 (« Le plan reste vrai avec un modèle local ») ; ST-31 (observateur PROP-2 : « sortie uniquement vers le fournisseur consenti »).
- **Tâche(s) concernée(s)** : ST-30, ST-31, ST-32
- **Description** : le plan ne dit pas ce qui fait qu'un fournisseur est « local » : son type, ou l'adresse à laquelle il répond. Il ne dit pas non plus si le réglage du fournisseur (RG-4.3) est une action du catalogue, donc exposée au MCP. Si « local » est un type de fournisseur dont l'adresse est configurable, et que ce réglage passe par une action, un assistant peut faire pointer le fournisseur « local » vers un hôte distant puis lancer `--ia`. Le code part alors sans consentement. Le critère de ST-31 ne voit rien : la sortie va bien vers le fournisseur configuré.
- **Preuve** : la barrière de F-32 repose sur l'identité du fournisseur. Le plan laisse à l'implémentation l'attribut qui exempte de cette barrière. Le trou n'existe pas encore : aucun code ne part avant ST-31, et le consentement de ST-30 est en zone Full. Mais rien dans les critères n'empêche de l'ouvrir. **Condition du verdict : fermer ce point dans les critères de la story de ST-30, avant d'ouvrir ST-31.**

### [MAJEUR] F-42 : « Synchro avant et après le recalcul » contredit le registre « ajout en fin »
- **Axe** : Cohérence
- **Source** : ST-11 (« tâches inscrites dans `arbitre/taches.rs` (une ligne par tâche, ajout en fin, comme `modules.rs`) ») ; ST-19 (« s'inscrit… **avant et après** le recalcul ») ; § Collisions (`taches.rs` : « M y ajoute une ligne en ST-19 »).
- **Tâche(s) concernée(s)** : ST-11, ST-19
- **Description** : une ligne ajoutée en fin passe après `plan.recalcul`. La synchro « avant » exige d'insérer en tête d'un fichier qui appartient à P, ou d'ajouter deux lignes alors que la collision n'en prévoit qu'une. Sans synchro avant, le recalcul du matin ignore les écritures faites sur le téléphone depuis la veille. Elles ne seront prises en compte qu'au recalcul suivant.
- **Preuve** : la règle de collision et le livrable de ST-19 ne peuvent pas être vrais en même temps. Cela peut être repris au cycle de ST-19.

### [MINEUR] F-43 : PL-9 déroge à la lettre de l'arbitrage F-10 dans la branche N = 1
- **Axe** : Cohérence
- **Source** : arbitrage F-10 (« en cas de conflit d'arbitrage ou de capacité, le planning passe d'abord ») ; PL-9 (« ST-22 est ouverte en premier… quel que soit N »).
- **Tâche(s) concernée(s)** : PL-9, ST-22, ST-05
- **Description** : avec N ≥ 2, il n'y a pas de conflit. Avec N = 1, le plan tranche lui-même entre deux phrases d'un même arbitrage, la priorité au planning et la démo tôt. La v2 avait laissé ce point ouvert (F-25.5) ; la v3 le tranche dans un sens.
- **Preuve** : ce choix revient à Quentin. Il est marqué « retenu par défaut », ce qui suffit, mais il faut le confirmer au recalibrage si N = 1.

### [MINEUR] F-44 : Les durées ignorent l'attente entre deux voies qui arbitrent en même temps
- **Axe** : Cohérence
- **Source** : § Légende (durée = taille + arbitrage) ; § Recalibrage, étape 5 ; revue v2, F-24.3 (« sans même compter l'attente quand Quentin arbitre l'autre voie »).
- **Tâche(s) concernée(s)** : Recalibrage
- **Description** : quand deux portes s'ouvrent en même temps sur deux voies, l'une attend. N borne la demande moyenne, pas ces collisions ponctuelles. Les jalons datés à l'étape 5 seront donc optimistes. Les portes mesurées au socle, où des spikes tournaient en parallèle, en captent une partie.
- **Preuve** : le déroulé de l'étape 5 ne modélise l'attente que pour la règle 3 (forts).

### [MINEUR] F-45 : Aucune règle ne répartit les points d'une story sur les soirées
- **Axe** : Complétude
- **Source** : règle 4 (« Points de la soirée ≤ budget ») ; tableau du socle (« Les points de ST-03 sont comptés en soirée 3, où se tiennent ses portes »).
- **Tâche(s) concernée(s)** : Recalibrage, étape 5
- **Description** : pour le socle, le plan répartit les points au cas par cas. Le déroulé de l'étape 5 doit vérifier la règle 4 sur environ 40 éléments, sans règle pour répartir les points d'une story à cheval sur deux soirées.
- **Preuve** : deux personnes qui appliquent l'étape 5 peuvent obtenir des dates différentes.

### [MINEUR] F-46 : Les signaux vivent dans le `Store`, donc ils se synchronisent
- **Axe** : Edge cases
- **Source** : ST-04 (« un espace de noms `arbitre.signaux` » ; « marque lus ») ; arbitrage AMB-12 (« les données de tous les modules se synchronisent toujours »).
- **Tâche(s) concernée(s)** : ST-04, ST-11, ST-19
- **Description** : un signal « jeton expiré » déposé par l'ordinateur A est lu, puis marqué lu, sur l'ordinateur B. Il n'apparaît plus jamais sur A, alors que c'est A qui doit agir (son trousseau).
- **Preuve** : ST-04 ne dit pas si `arbitre.signaux` est propre à l'appareil, et AMB-12 synchronise tout par défaut.

### [MINEUR] F-47 : Deux stories ont grossi sans être réestimées, et une charge change contre la consigne
- **Axe** : Cohérence
- **Source** : ST-04 (registre des surfaces, signaux, F-37 ajoutés ; toujours M moyenne) ; ST-30 (arbitrage passé de faible à moyenne) ; consigne de Quentin (« Garde… les charges d'arbitrage »).
- **Tâche(s) concernée(s)** : ST-04, ST-30
- **Description** : ST-04 était déjà au plafond d'une M. Le changement de ST-30 est justifié par la sécurité, mais il n'apparaît pas dans le tableau de résolution.
- **Preuve** : effet faible, puisque le recalibrage mesure ST-04 telle qu'elle sera réellement faite.

### [MINEUR] F-48 : S3-V « accepté par l'admin » mais critères (5) ou (6) en échec
- **Axe** : Cas d'erreur
- **Source** : § Borne de S3 (trois issues) ; ST-15b (« S3 critères (5) et (6) ») ; R-4.
- **Tâche(s) concernée(s)** : S3-V, ST-10, ST-15b, ST-17
- **Description** : l'admin accepte, mais le marqueur ou le titre fuit chez un collègue. ST-10 peut se faire, ST-15b non. Le plan ne dit pas si ST-17 prend le relais pour l'écriture, ni si les blocs sont abandonnés.
- **Preuve** : c'est une quatrième issue, qui n'entre dans aucune des trois écrites. Elle peut être reprise au cycle de S3-V.

### [MINEUR] F-49 : Le contrôle « terminal interactif » se contourne avec un pseudo-terminal
- **Axe** : Sécurité & intégrité
- **Source** : ST-30 (« refuse de tourner si l'entrée standard n'est pas un terminal » ; « limite assumée : un agent qui a un shell… »).
- **Tâche(s) concernée(s)** : ST-30
- **Description** : un assistant qui a un shell (c'est le cas du client MCP principal, Claude Code) peut lancer `ia consentir` sous un pseudo-terminal et répondre lui-même. Le plan assume la limite pour l'exfiltration directe, mais pas pour ce cas. Or c'est précisément le geste de consentement que CONFLIT-16 réserve au mainteneur.
- **Preuve** : la limite est écrite, ce qui rend le risque connu et accepté. Il reste à le formuler comme tel dans la doc de ST-30.

---

## Axes sans faille détectée
- **Complétude des identifiants** : RAS. Tous les INV, PROP, RG et AC restent rattachés, et PROP-12 et PROP-14 ont maintenant un porteur dans la branche de repli.
- **Régressions** : RAS. La codebase est vide, et un échec iOS ne touche plus le socle (PL-8). La procédure d'amendement borne le seul risque de rebase large.
- **Over-engineering** : RAS. Le port asynchrone coûte peu (futures immédiates côté SQLite) et retire le seul amendement tardif probable du socle.

## Recommandation
**VALIDÉ.** Le cycle de ST-01 peut s'ouvrir, sous deux conditions :
1. **F-40, avant la soirée 1** : faire écrire `debut`, `fin`, `portes` et `soirees.log` par l'orchestrateur, ou tenir ces relevés à la main. Ce travail entre dans G-4.
2. **F-41, au cycle de ST-30** : définir « local » et placer le réglage du fournisseur, avec un critère qui le vérifie, avant d'ouvrir ST-31.

À reprendre au cycle de la story concernée : F-42 (ST-19), F-46 (ST-04), F-48 (S3-V), F-49 (ST-30). À relire au recalibrage : F-43, F-44, F-45, F-47.

---

## Arbitrage de Quentin (2026-11-02) — repris des revues v1 et v2, fait foi

- **F-10** : la voie decision diff tourne en parallèle du planning dès que le socle (ST-01 à ST-04) est posé. « Socle → planning → decision diff » devient un ordre de priorité : en cas de conflit d'arbitrage ou de capacité, le planning passe d'abord. Conséquence pour F-9 : la démo ST-22 arrive tôt, au service des entretiens.
