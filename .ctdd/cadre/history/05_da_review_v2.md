# Revue Devil's Advocate — OpenArbitre, cadre global (découpe en stories), plan v2

- Inputs : `01_story.md`, `02_specs.md`, `04_plan.md` (v2) ; contexte `03_archi.md` ; revue précédente `history/05_da_review_v1.md`. Les sections « Arbitrages de Quentin » de 01, 02, 03 et de la revue v1 (F-10) font foi.
- Référence d'estimation fixée par Quentin, non contestée ici : une story S tient en un tiers de soirée, le cadre en 3 à 5 semaines. Cette revue ne vérifie que la cohérence interne du plan par rapport à cette référence.
- Date : 2026-11-03 · Itération DA : 2 / 3.

## Verdict global
**BLOQUÉ**

La v2 corrige l'essentiel de la v1 : il y a des voies, un tableau des collisions et un plafond, et le repli CalDAV ne dépend plus de Google. La logique du planning tourne sur simulateur et la table des codes est gelée. Sur 23 failles, 16 sont résolues, 7 partiellement, aucune ne reste ouverte. Mais le calendrier, qui était la deuxième exigence non négociable de la v1, n'est toujours pas cohérent avec les unités du plan lui-même. La taille est définie comme du temps machine, l'arbitrage comme du temps humain qui s'y ajoute, et l'ordonnancement remplit chaque voie avec le seul temps machine. Le mécanisme de recalibrage qui doit corriger cela ne peut pas mesurer la moitié de ses paramètres.

---

## Suivi des failles de la v1

| Faille v1 | Statut | Preuve |
|---|---|---|
| **F-1** (B) voies, collisions, plafond | **Partiellement** | Voies P, D, M nommées, tableau des collisions avec règles, plafond de 2 voies et 8 points (§ Parallélisme). Restent : la règle de propriété de l'outillage de test est contredite par ST-07 et ST-21 (F-27). L'enchaînement recalcul → synchro tombe entre deux voies (F-28). L'ordonnancement viole le plafond qu'il fixe (F-29). |
| **F-2** (B) tailles, jalons, PL-1 | **Partiellement** | Tailles en soirées, charge en points, jalons datés aux deux rythmes, PL-1 chiffré retiré. Mais les jalons ne découlent pas des définitions de taille du plan (F-24), et le recalibrage ne peut pas remplacer S, L ni « faible » (F-25). |
| **F-3** (B) repli dépendant de Google | **Résolue** | ST-17 : « Dépendances : ST-09, ST-15a ». Le repli s'exécute sans Google. Deux failles nouvelles touchent son déclenchement et sa couverture (F-30, F-31). |
| F-4 logique pure derrière S3 | **Résolue** | ST-11 : « ST-09, S6 » ; ST-12 → ST-11 sans Google ; Google ne reste que dans les feuilles ST-10 et ST-15b. |
| F-5 arêtes vers ST-07 | **Résolue** | ST-07 : « ST-04, S5 » ; ST-27 et ST-32 déclarent ST-07. |
| F-6 table des codes | **Résolue** | ST-03 fixe la table complète des 5 catégories, gelée. Les codes de ST-22 et ST-24 sont « pris dans la table de ST-03 ». Reste un point de forme (F-38). |
| F-7 ST-19 dépend du planning | **Résolue** | ST-19 : « ST-18, ST-04, S6 ». La convergence est prouvée sur le module factice. |
| F-8 spikes juste à temps | **Partiellement** | S4 passe avant D, S5 au socle, S1-A avant ST-02. Mais S1-A gèle la forme du port sans le critère iOS qui porte R-2 (F-26). |
| F-9 fenêtre de marché | **Résolue** (par l'arbitrage F-10) | Démo ST-22 à la soirée 4. Cela suppose 2 voies ; la branche « une voie » du recalibrage la renvoie derrière tout le planning sans le dire (F-25, point 5). |
| F-10 diff en parallèle | **Résolue** | Arbitrage appliqué : D démarre dès ST-04, avec la priorité P > D > M. |
| F-11 ST-03 trop grosse | **Résolue** | ST-03 se limite au catalogue, à la CLI et aux codes. ST-04b, qui dépend de ST-04, porte l'export et INV-24. |
| F-12 AC-1.1 hors ligne | **Résolue** | Conformance générique de ST-04 : « toute action `requiert_réseau = aucun` réussit réseau coupé », rejouée par module. Reste un point mineur (F-37). |
| F-13 partage du secret | **Résolue** | Modèle par défaut en ST-18 (phrase par appareil, sel public, clé au trousseau). Le cas sans session est couvert en ST-19, et la PWA est marquée `[À DÉFINIR]` en ST-20 comme demandé. Reste un point mineur (F-34). |
| F-14 trousseau sans session | **Partiellement** | S6 avant ST-11, et ST-11 a un critère « jeton inaccessible ou expiré ». Mais aucune story ne teste la chaîne réelle launchd ou systemd → adaptateur Google → jeton au trousseau. S6 teste un secret générique en lecture. ST-11 tourne sur simulateur, donc sur une erreur simulée. ST-10 n'a aucun critère d'exécution en tâche de fond (« jeton au trousseau, aucun secret dans le dépôt »). Enfin, ST-11 suppose que le port `Agenda` sait signaler « identifiants indisponibles ou expirés », alors que le livrable de ST-09, qui pose ce port, ne cite aucune erreur. |
| F-15 déclencheur IA | **Partiellement** | Commande locale uniquement, jamais dans l'Action, avec l'avertissement CONFLIT-16 et PROP-2 en ST-31. Mais l'action est aussi exposée au MCP, ce qui permet de contourner le consentement (F-32). |
| F-16 outil de mutation | **Résolue** | ST-01 pose `cargo-mutants --in-diff`, le script de seuil, et un critère sur sa propre PR. Reste un point mineur (F-36). |
| F-17 dépendances partagées | **Résolue** | `ST-01-dependances.md` et PL-6 : tokio, reqwest + rustls, keyring, proptest. |
| F-18 observateur réseau | **Résolue** | Posé en ST-04, avant toute sortie réseau. |
| F-19 ST-25 → ST-24 | **Résolue** | ST-25 : « ST-23 ». |
| F-20 stories sans valeur seule | **Résolue** | ST-02, 03, 18 et 30 sont marquées ; aucun jalon n'est posé sur elles. |
| F-21 PROP-7 côté GUI | **Résolue** | ST-21 : critère d'écritures concurrentes CLI et PWA. |
| F-22 PWA et relais | **Résolue** | `relay serve --pwa <dir>`, sans embarquer la PWA dans le binaire ; le binaire entier sur le relais est accepté en v1. |
| F-23 CONFLIT-17 aval | **Résolue** | ST-22, ST-24 et ST-32 citent CONFLIT-17 et son effet. |

---

## Failles nouvelles ou résiduelles (v2)

### [BLOQUANT] F-24 : Les jalons ne découlent pas des unités du plan, puisque le temps d'arbitrage n'est jamais compté dans le chemin critique
- **Axe** : Cohérence
- **Source** : `04_plan.md` § Légende (« **Taille** : temps machine du cycle complet… S = ⅓, M = ⅔, L = 1 » ; « **Arbitrage** : temps de Quentin sur tout le cycle… 1 point vaut environ 15 minutes ») ; § Plafond (« 8 points… environ 2 h… dans une soirée de 2 h 30 ») ; § Ordonnancement (« une story L occupe une voie toute la soirée ») ; § Jalons datés ; texte sous le tableau (« Tout le cadre tient en 5 semaines à 3 soirées par semaine. C'est la borne haute ») ; exigence v1 F-2 (jalons cohérents avec les tailles).
- **Tâche(s) concernée(s)** : § Calendrier ; ST-01, ST-03, ST-22, ST-19, ST-20, ST-26, ST-32 ; tous les jalons
- **Description** :
  1. **Le temps machine et le temps humain d'une story s'additionnent.** Un cycle C-TDD avance en phases séparées par des portes humaines : story, specs, archi locale, plan, DA, review. Quand une story attend Quentin, aucun agent ne travaille sur elle. Sa durée réelle est donc au moins la somme de son temps machine et de son temps d'arbitrage. L'ordonnancement ne compte que le temps machine. Dans chaque voie et chaque soirée, de la soirée 2 à la 13, la somme des tailles vaut **exactement 1 soirée**. Exemples : soirée 5 en P, ST-06 (⅓) + ST-07 (⅔) ; soirée 12 en D, ST-28 (⅓ restant) + ST-29 (⅓) + ST-31 (⅓). Le temps d'arbitrage de ces stories ne tient donc nulle part.
  2. **Conséquence directe sur les jalons**, avec les seules unités du plan :
     - ST-22, L + forte : 150 + 60 = **210 min au minimum**, seule dans sa voie à la soirée 4. Le jalon « Démo decision diff » à la soirée 4 ne tient pas.
     - ST-01, M + forte : 100 + 60 = 160 min, plus que la soirée de 150 min. Même chose pour ST-03.
     - ST-19, ST-20 (L + forte) et ST-26, ST-32 (L + moyenne) dépassent chacune leur soirée.
  3. **Ordre de grandeur.** On recompte chaque voie en additionnant temps machine et temps d'arbitrage, sans même compter l'attente quand Quentin arbitre l'autre voie. Le socle passe de 3 à environ 3,9 soirées. La voie P passe de 9 à environ 10,9 soirées après le socle. La fin de M (ST-19 → S1-B + S2 → ST-20 → ST-21) passe de 4 à environ 4,7 soirées. Le cadre se termine alors vers la soirée 19 ou 20, en **7e semaine** à 3 soirées par semaine. Cela dépasse la borne haute de la référence, alors que le plan affirme la respecter.
  4. **Même sans ce point, la phrase « 5 semaines » est fausse dans le tableau du plan** : la soirée 16 tombe le 2026-12-08, un mardi de 6e semaine.
  5. **La marge annoncée n'existe pas là où elle est placée.** Elle est comptée en points (« les soirées 12 à 16 ont de la marge »), alors que les soirées 13 à 16 sont le chemin critique **machine** de la voie M, pleine à chaque soirée. Un dépassement du socle décale tout, puisque les trois voies dépendent de ST-04 : il ne peut pas être « absorbé » en fin de plan. « Se tiennent à 2 soirées par semaine sans glisser » contredit la colonne à 2 soirées par semaine du même tableau, qui finit en janvier.
  6. Autre erreur de lecture : « La voie M ne peut pas aller plus vite, parce que ses trois stories fortes passent l'une après l'autre ». Aux soirées 10 à 12, M est arrêtée par le plafond de 2 voies, pas par ses stories fortes.
- **Preuve** : les deux lectures possibles de la légende sont toutes deux incohérentes.
  - Si la taille ne compte que le temps machine, comme l'écrit la légende, les jalons sont faux (points 1 à 3).
  - Si elle compte toute la durée de la story, arbitrage compris, alors la légende, l'étape 1 du recalibrage (« les soirées machine réelles ») et le budget de 8 points, compté à côté, sont faux.

  Dans les deux cas, les jalons ne peuvent pas être recalculés à partir des définitions du plan. C'était l'objet de F-2 (BLOQUANT en v1).

### [MAJEUR] F-25 : Le recalibrage après ST-04 ne peut pas mesurer ce qu'il doit remplacer
- **Axe** : Cohérence
- **Source** : § Recalibrage, étapes 1 à 4 ; tailles et arbitrages de ST-01 à ST-04 ; § Gestes G-0 à G-4 ; texte sous l'ordonnancement (« dépassement probable des premiers cycles ») ; § Plafond (priorité P, puis D, puis M) ; arbitrage F-10 (« la démo ST-22 arrive tôt »).
- **Tâche(s) concernée(s)** : Recalibrage ; ST-01 à ST-04 ; G-x ; ST-22
- **Description** :
  1. **Échantillon dégénéré.** ST-01 à ST-04 sont **toutes de taille M**. Leurs arbitrages sont forte, moyenne, forte, moyenne. L'étape 2 (« remplacer les valeurs S/M/L… et faible/moyenne/forte… par les médianes mesurées ») n'a aucune mesure pour S, L ou « faible ». Les médianes de « forte » et de « moyenne » portent chacune sur 2 valeurs.
  2. **Biais connu, non corrigé.** Le plan s'attend lui-même à ce que les premiers cycles débordent. Il en fait pourtant la seule base pour fixer les valeurs de tout le reste du plan.
  3. **Rien ne mesure ces durées.** L'étape 1 demande « les minutes d'arbitrage réelles par phase du cycle » dès ST-01. Aucun geste ni aucune story ne met en place cette mesure : ni journal, ni champ dans `state.json` par story (G-4), ni consigne à l'orchestrateur. La soirée 1 est datée au 2026-11-03, c'est-à-dire aujourd'hui.
  4. **Seuils ambigus.** « Si l'arbitrage mesuré dépasse 1,5 fois l'estimation » : le plan ne dit pas s'il s'agit du total du socle, d'une médiane par story ou d'une valeur par phase. Deux lectures donnent deux plafonds différents.
  5. **Branche « une seule voie », effet non dit.** Avec la priorité P > D > M, une seule voie en cycle place ST-22 derrière toute la voie P. La démo, que l'arbitrage F-10 voulait tôt pour les entretiens, passe alors de la soirée 4 à la fin du planning. Le plan ne dit pas ce que devient la démo dans cette branche.
  6. **Ce qui n'a aucun équivalent dans l'échantillon reste au jugé** : S1-B + S2 (installation à la main sur appareils), G-3 (annotation), le délai de S3. Le recalibrage ne les touche pas, et il n'y a pas de second point de recalibrage.
- **Preuve** : l'étape 4 abandonne l'ordonnancement indicatif et le remplace par des mesures. Pour S, L, « faible » et les arbitrages par phase, ces mesures ne peuvent pas exister telles que le plan est écrit.

### [MAJEUR] F-26 : S1-A fige la forme du port `Store` sans le critère iOS qui porte R-2
- **Axe** : Edge cases / Régressions
- **Source** : § Spikes (S1-A : « critères (1) et (6), plus (7) », verdict avant ST-02, « gel du port » ; S1-B : critères « (2) à (5) », soirée 14) ; `03_archi.md` S1, critère (5) « Fonctionne en PWA installée sur Safari iOS et Chrome Android » ; R-2 (« Haute ») ; § Under-engineering (« le repli IndexedDB de l'archi tomberait avec lui ») ; § Collisions, `arbitre-core` (« Les changer est un amendement du socle »).
- **Tâche(s) concernée(s)** : S1-A, ST-02, ST-04b, ST-18, ST-19, S1-B, ST-20
- **Description** : le choix entre port synchrone et port asynchrone dépend de ce que Safari iOS supporte dans une PWA installée : accès synchrone OPFS depuis un worker, ou repli IndexedDB, qui est asynchrone. C'est exactement le critère (5). Or ce critère est renvoyé à S1-B, soirée 14. À cette date, ST-02, ST-04b, ST-18 et ST-19 auront été construites et fusionnées sur la forme gelée.
- **Preuve** : le plan reconnaît qu'un port synchrone ferait tomber le repli IndexedDB, et il classe R-2 en probabilité Haute. Un échec de S1-B déclencherait un amendement du socle à la soirée 14. Toutes les voies devraient alors se rebaser sur un contrat du socle modifié, alors que les trois quarts du plan sont déjà faits.

### [MAJEUR] F-27 : Règles de propriété contredites par les stories, et plus aucune voie pour amender le socle
- **Axe** : Cohérence
- **Source** : § Collisions, ligne « Outillage de test partagé » (« posés au socle (ST-04), dans `arbitre/tests/support/` ; seul le socle les modifie ») ; ST-07 (« la parité MCP est ajoutée **au test de conformité générique** ») ; ST-21 (« parité du catalogue étendue à la GUI ») ; ST-20 (PROP-1 côté PWA) ; § Voies (Socle : « tout, puis plus rien après ST-04 ») ; ST-30 (« `arbitre` (`wiring/`) ») ; ST-11 et ST-19 (« signalé à l'ouverture suivante »).
- **Tâche(s) concernée(s)** : ST-07, ST-11, ST-19, ST-20, ST-21, ST-30
- **Description** :
  1. ST-07 (voie P) et ST-21 (voie M) modifient par construction le test de conformité générique, que seul le socle a le droit de modifier. Or la voie Socle n'existe plus après ST-04. Le plan ne dit pas qui exécute un « amendement du socle », ni dans quelle voie ou quel worktree. Il ne dit pas non plus comment cette exception passe le plafond et la priorité.
  2. ST-11 (P) et ST-19 (M) ont toutes deux besoin d'un signal affiché « à l'ouverture suivante ». Aucune story ne pose de mécanisme commun. Le point d'entrée de la CLI (`main.rs`) est un fichier chaud qui ne figure dans le tableau que pour des sous-commandes.
  3. Le fichier de câblage de ST-30 n'est attribué à aucune voie : M possède `relay/` et `sync`, pas `wiring/ai.rs` ni son équivalent.
- **Preuve** : la règle du tableau et les critères de complétion de ST-07 et ST-21 ne peuvent pas être vrais en même temps. Un orchestrateur qui applique le tableau refusera ST-07. S'il applique ST-07, il viole le tableau.

### [MAJEUR] F-28 : Recalcul planifié → synchro → téléphone : aucune story ne fait l'enchaînement
- **Axe** : Complétude
- **Source** : arbitrage CONFLIT-7 et 8 dans `02_specs.md` (« Le téléphone reçoit le résultat par la synchronisation ») ; AC-13.7 ; ST-11 (voie P, `scheduler/`, avant toute synchro) ; ST-19 (voie M, « synchro lancée sans session interactive ») ; § Voies (P possède `arbitre/scheduler/`).
- **Tâche(s) concernée(s)** : ST-11, ST-19, ST-20
- **Description** : ST-11 met en place le recalcul planifié alors que la synchro n'existe pas encore. ST-19 ajoute une synchro lancée sans session, mais ne dit pas ce qui la déclenche. `scheduler/` appartient à P, donc M ne peut pas y ajouter « puis synchroniser », et aucune collision P/M sur `scheduler/` n'est listée. Aucun critère ne vérifie qu'un recalcul fait ordinateur fermé à l'écran apparaît sur le téléphone sans que l'utilisateur ouvre la CLI.
- **Preuve** : l'arbitrage de CONFLIT-7 décrit le chemin recalcul → synchro → téléphone. Chaque morceau a sa story, mais le chemin complet n'en a aucune, et le fichier où il devrait vivre appartient à une autre voie.

### [MAJEUR] F-29 : L'ordonnancement viole le plafond qu'il fixe
- **Axe** : Cohérence
- **Source** : § Plafond (« Une 3e voie ne s'ouvre que pour une story faible, ou quand l'une des deux attend sans cycle en cours (délai externe, annotation G-3) » ; « Au plus une story à charge forte en cours ») ; G-3 (« forte + : 6 points ») ; S3 (« forte (4) ») ; § Ordonnancement, soirées 1, 3, 7, 8 et 9.
- **Tâche(s) concernée(s)** : G-3, S3, ST-01, ST-03, ST-04b, ST-14, ST-18
- **Description** :
  1. G-3, l'activité la plus chargée du plan en arbitrage (6 points, la seule notée « forte + »), est traitée comme une **attente** qui libère une place de voie. Aux soirées 7 et 8, cela ouvre M sur ST-04b et ST-18, deux stories moyennes.
  2. Soirée 9 : ST-14 (forte) et G-3 (forte +) sont en cours en même temps. Soirées 1 et 3 : ST-01, puis ST-03 (fortes), avec S3 (forte).
  3. La règle ne dit pas si les spikes et les gestes comptent dans le plafond. Le plan les place « hors des voies », mais leur charge est mesurée dans les mêmes points.
- **Preuve** : le plafond est justifié par la capacité d'arbitrage (R-5 « Haute »). L'exception « attente » appliquée à G-3 laisse passer précisément la charge d'arbitrage que le plafond devait bloquer.

### [MAJEUR] F-30 : S3 n'a pas de borne, donc le repli ne se déclenche jamais si l'admin ne répond pas
- **Axe** : Cas d'erreur
- **Source** : § Spikes (« Chaque spike a un budget fixe » ; S3 : « S + délai externe », « verdict dès que l'admin répond ») ; ST-17 (« Ne s'engage qu'à la demande d'un utilisateur réel, ou si S3 conclut au repli ») ; Recalibrage, étape 5 (« si l'admin Workspace n'a pas répondu, garder ST-10 en fin de voie P ») ; R-1 ; jalon « Blocs dans Google Calendar (si S3 a répondu à la soirée 11) ».
- **Tâche(s) concernée(s)** : S3, ST-10, ST-15b, ST-17
- **Description** : il y a trois issues possibles : accepté, refusé, pas de réponse. Le plan ne traite que les deux premières. Sans réponse, ST-10 reste « en fin de voie P » sans échéance, et ST-17 ne remplit aucune de ses deux conditions. La voie P se termine (ST-16, soirée 11) sans aucun agenda réel, et rien ne déclenche le repli.
- **Preuve** : la règle « budget fixe » des spikes et le « délai externe » de S3 se contredisent. Dans le cas « pas de réponse », qui est le délai externe que PL-2 juge incompressible, la parade de R-1 ne s'exécute pas.

### [MAJEUR] F-31 : En repli, ST-17 perd ce que ST-10 et ST-15b couvraient
- **Axe** : Complétude / Sécurité
- **Source** : ST-17 (« Couvre : RG-8.\*, 10.\* rejouées… ; PROP-2 » ; critère : suite de contrat du port) ; ST-10 (AC-8.5, AC-1.2, PROP-12, CONFLIT-8, « agenda injoignable ⇒ non à jour », secret au trousseau) ; ST-15b (PROP-14 côté adaptateur, « essai réel ») ; R-4 (« Critères (5) et (6) de S3 obligatoires avant ST-15b ») ; `03_archi.md` S3 (5) et (6), propres à Google.
- **Tâche(s) concernée(s)** : ST-17
- **Description** : si ST-17 « prend la place de ST-10 et de ST-15b », plus aucune story ne porte les éléments suivants :
  - le refus d'un événement sans marqueur, côté adaptateur : PROP-14 en défense en profondeur, que le plan dit « bloquant » ;
  - la vérification réelle que le titre du bloc et le marqueur ne fuient pas vers les collègues (R-4, AC-11.1 et 11.2). Les critères (5) et (6) de S3 ne valent que pour Google ;
  - PROP-12 (agendas multiples, pannes), AC-8.5, la capacité « non à jour » quand l'agenda est injoignable, et le stockage des identifiants CalDAV au trousseau.
- **Preuve** : R-4 est noté Moyenne, avec un impact qui « viole AC-11.1/11.2 ». Sa seule parade est un essai sur Google. Dans le cas où le repli s'exécute, cette parade n'existe plus et ST-17 ne la remplace pas.

### [MAJEUR] F-32 : Le consentement à l'envoi de code vers l'IA peut être contourné par le MCP
- **Axe** : Sécurité & intégrité
- **Source** : ST-31 (« commande locale explicite uniquement (`arbitre diff decisions --ia`, **ou l'action MCP correspondante**) » ; « avertissement avant le premier envoi ») ; ST-32 (réutilise cet avertissement) ; INV-10 et RG-4.1 (parité CLI ⊆ MCP, rendue générique en ST-07) ; CONFLIT-16 (« prévenir le mainteneur ») ; arbitrage CONFLIT-7 et 8 (« un fournisseur d'IA n'est jamais sollicité automatiquement »).
- **Tâche(s) concernée(s)** : ST-07, ST-31, ST-32
- **Description** : la parité générique expose automatiquement au MCP toute action du catalogue, y compris celles qui sont déclarées `requiert_réseau: ia`. Si un assistant appelle l'outil MCP, l'avertissement de CONFLIT-16 est rendu à l'assistant, pas au mainteneur, et l'assistant peut l'acquitter seul. Le plan ne dit pas si un appel venu d'un agent compte comme une « sollicitation » au sens de l'arbitrage, ni qui doit voir et accepter l'avertissement.
- **Preuve** : le critère de ST-31 vérifie que l'envoi suit la commande (« uniquement sur la commande »). Il ne vérifie ni l'origine de cette commande ni la personne qui a donné son accord. Du code peut donc partir chez un fournisseur distant sans qu'un humain l'ait vu, ce que CONFLIT-16 voulait empêcher.

### [MAJEUR] F-33 : Arêtes entre voies manquantes vers ST-20, ST-21 et ST-32
- **Axe** : Cohérence
- **Source** : ST-20 (« affiche la pile, **les jours**, la charge et les questions » ; lecture de ST-05, ST-09, ST-13) ; ST-21 (« créer, modifier, faire, **réordonner** et **répondre aux questions**… même durée proposée » ; « Dépendances : ST-20 ») ; ST-32 (« CONFLIT-16 (**réutilise l'avertissement de ST-31**) » ; « Dépendances : ST-30, ST-23, ST-07 ») ; Recalibrage, étape 4 (« L'ordonnancement indicatif… est alors abandonné ») ; `03_archi.md` (`arbitre-web` dépend de `plan`).
- **Tâche(s) concernée(s)** : ST-20, ST-21, ST-32
- **Description** :
  - « Les jours » planifiés sont le résultat de ST-12, qui ne figure pas dans les arêtes déclarées vers ST-20.
  - ST-21 a besoin de ST-06 (réordonner), de ST-13 (répondre aux questions) et de ST-08 (même durée proposée), mais ne dépend que de ST-20.
  - `arbitre-web` compile `arbitre-plan` en wasm : la voie M dépend donc à la compilation de l'état de la voie P. Ce n'est pas qu'une lecture du format des données.
  - ST-32 réutilise l'avertissement de ST-31 sans déclarer l'arête.
- **Preuve** : c'est le même mécanisme que F-5 en v1. Le plan dit ces arêtes « toutes en avance sur leur besoin dans l'ordonnancement », mais il abandonne cet ordonnancement dès la soirée 4. À partir de là, seules les arêtes déclarées protègent l'ordre.

### [MINEUR] F-34 : Création du sel public sur le relais en cas d'initialisation concurrente
- **Axe** : Edge cases
- **Source** : ST-18 (« sel public stocké sur le relais » ; ST-18 précède le relais) ; ST-19 (aucun critère sur le sel).
- **Tâche(s) concernée(s)** : ST-18, ST-19
- **Description** : aucune story ne dit qui crée le sel ni quand. Si deux appareils initialisent la synchro en même temps, ils peuvent créer deux sels. Ils dérivent alors deux clés différentes et ne peuvent pas lire les enveloppes l'un de l'autre.
- **Preuve** : le critère de ST-18 suppose « la même phrase et le même sel », et aucun critère ne garantit ce sel commun.

### [MINEUR] F-35 : Jalons « utile au quotidien » et « recalcul du matin » posés sur un agenda simulé
- **Axe** : Complétude (métier)
- **Source** : jalons « Planning sur CLI et assistant, utile au quotidien » (ST-09, soirée 6) et « Recalcul du matin, outil fermé » (ST-11, soirée 7) ; ST-09 (« agenda simulé ») ; bloc B (« sur le simulateur d'`Agenda` »).
- **Tâche(s) concernée(s)** : ST-09, ST-11
- **Description** : à ces jalons, la charge et la capacité sont calculées sur un agenda simulé, sans les vraies réunions de l'utilisateur. Le recalcul ne place encore rien, puisque ST-12 vient après lui.
- **Preuve** : la valeur annoncée par ces jalons n'arrive qu'avec ST-10 ou ST-17, au mieux à la soirée 11, et sous réserve de S3.

### [MINEUR] F-36 : Seuil de mutation par zone sur les stories à zones mixtes
- **Axe** : Complétude
- **Source** : ST-01 (« script de seuil par zone (85 / 70) ») ; F-16 (« chaque review vérifie le seuil de sa zone sur les fichiers qu'elle modifie ») ; zones mixtes de ST-10, 19, 22, 24, 26 et 32.
- **Tâche(s) concernée(s)** : ST-01 et les stories à zones mixtes
- **Description** : rien ne dit à quelle zone appartient chaque fichier. Sur une story à zones mixtes, le script ne sait pas quel seuil appliquer.
- **Preuve** : par exemple, ST-19 mélange la zone Gen (relais) et la zone Full (convergence) dans la même PR.

### [MINEUR] F-37 : PROP-1 repose sur une déclaration faite par le module lui-même
- **Axe** : Sécurité & intégrité
- **Source** : ST-03 (`requiert_réseau`) ; ST-04 (« toute action `requiert_réseau = aucun` réussit réseau coupé ») ; PROP-1 (« toute opération… réussit hors ligne »).
- **Tâche(s) concernée(s)** : ST-04
- **Description** : une action déclarée `agenda` ou `relais` sort du contrôle hors ligne de la conformance. Son comportement réseau coupé (mode dégradé) n'est alors vérifié que si sa propre story le prévoit, comme ST-10 et ST-11.
- **Preuve** : la conformance ne vérifie pas qu'une action déclarée « réseau » ait bien un comportement hors ligne. PROP-2 ne détecte que la déclaration inverse, une action déclarée « aucun » qui sort sur le réseau.

### [MINEUR] F-38 : AMB-16 est tranché en ST-03 et encore listé dans ST-22
- **Axe** : Cohérence
- **Source** : ST-03 (« Couvre : … AMB-16 », table gelée) ; ST-22 (« Couvre : … AMB-14, 15, 16 »).
- **Tâche(s) concernée(s)** : ST-03, ST-22
- **Description** : le plan ne dit pas si le cycle de ST-22 peut rouvrir AMB-16. S'il le peut, toucher à la table gelée devient un amendement du socle.
- **Preuve** : la même ambiguïté apparaît dans deux stories, dont l'une a gelé la réponse.

### [MINEUR] F-39 : Jalons datés à partir d'aujourd'hui, alors que le plan est encore en revue
- **Axe** : Cohérence
- **Source** : § Jalons datés (« à partir du mardi 2026-11-03 ») ; `state.json` (phase 5 en cours, DA itération 2 sur 3).
- **Tâche(s) concernée(s)** : § Calendrier
- **Description** : la soirée 1, avec G-0, G-1, G-4, ST-01, S5 et le lancement de S3, est datée ce soir. Le plan n'est pas encore validé, et une 3e itération de revue reste possible.
- **Preuve** : chaque itération décale toutes les dates d'au moins une soirée, et le plan ne le dit pas.

---

## Cohérence interne vérifiée sans faille
- **Dates** : le passage soirée → date est exact aux deux rythmes (mardi, mercredi et jeudi, ou mardi et jeudi, à partir du 2026-11-03), décalage de Noël compris.
- **Points** : les totaux par soirée sont exacts et font bien 97. Chaque soirée reste à 8 points ou moins.
- **Tailles par voie** : prises comme du temps machine seul, elles tiennent à 1 soirée par voie et par soirée. Le problème vient de ce qui n'est pas compté (F-24), pas du calcul lui-même.

## Axes sans faille détectée
- **Complétude des identifiants** : INV-1 à 84, PROP-1 à 24, toutes les RG et tous les AC sont rattachés à une story, AC-1.1 compris. Les seuls manques sont des enchaînements entre stories (F-28) et des couvertures perdues en cas de repli (F-31).
- **Régressions** : la codebase est vide, G-0 vient avant le premier commit, le schéma est générique et les tests de parité et de conformité sont génériques. Le seul risque de régression grave est un amendement tardif du socle (F-26).

## Recommandation
Retour au Planner pour l'itération 3, qui est la dernière avant escalade. Points à corriger :
- **BLOQUANT** : F-24, qui demande une définition de taille cohérente avec l'ordonnancement et des jalons recalculés à partir de cette définition.
- **MAJEURS** : F-25 à F-33. Trois d'entre eux sont à régler **avant la soirée 1 ou la soirée 2**, faute de quoi ils ne pourront plus l'être :
  - F-25, point 3 : mettre en place la mesure des temps dès le premier cycle ;
  - F-26 : critère iOS manquant avant le gel du port en ST-02 ;
  - F-27 : savoir qui amende le socle.

Si l'itération 3 bloque encore sur le calendrier, l'escalade portera sur la définition de la taille. C'est une question de modèle d'estimation, à trancher par Quentin, et non un défaut de découpe.

---

## Arbitrage de Quentin (2026-11-02) — repris de la revue v1, fait foi

- **F-10** : la voie decision diff tourne en parallèle du planning dès que le socle (ST-01 à ST-04) est posé. « Socle → planning → decision diff » devient un ordre de priorité : en cas de conflit d'arbitrage ou de capacité, le planning passe d'abord. Conséquence pour F-9 : la démo ST-22 arrive tôt, au service des entretiens.
