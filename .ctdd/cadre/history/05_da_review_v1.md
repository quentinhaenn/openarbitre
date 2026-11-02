# Revue Devil's Advocate — OpenArbitre, cadre global (découpe en 32 stories)

- Inputs : `01_story.md`, `02_specs.md`, `04_plan.md` ; contexte `03_archi.md`, `CADRAGE.md`, `inputs/plan-decision-diff-2026-10-02.md`. Les sections « Arbitrages de Quentin » font foi.
- Date : 2026-11-02 · Itération DA : 1 / 3.

## Verdict global
**BLOQUÉ**

Toutes les règles de 02_specs.md ont une story : INV-1 à 84, PROP-1 à 24 et toutes les RG-x.y. Mais le plan n'est pas exécutable dans le mode prévu. Il est écrit pour un seul développeur qui avance en séquence. Il ne donne ni voies parallèles, ni points de collision, ni plafond de voies actives. Ses tailles sont en semaines, sans jalon daté, et elles contredisent le chiffre de PL-1. Enfin, le repli prévu si Google refuse l'accès (R-1) dépend de la réussite de Google.

---

## Failles identifiées

### [BLOQUANT] F-1 : Aucune voie parallèle, aucun point de collision, aucun plafond
- **Axe** : Cohérence
- **Source** : exigence de la revue (stories menées en parallèle, en worktrees pilotés par un orchestrateur, dès que ST-01 à ST-04 sont posées) ; `04_plan.md` § Ordre d'exécution, l. 434 : « Séquence de travail retenue, pour un développeur seul » ; `03_archi.md` l. 318 : « S-5 et S-6 (téléphone) sont parallélisables avec le planning dès S-3 ».
- **Tâche(s) concernée(s)** : tout le plan après ST-04
- **Description** : le plan donne une seule séquence linéaire de 31 stories. Le graphe ASCII montre quatre branches après ST-04 (planning, mobile, diff, IA), mais le plan n'en tire aucune voie. Il ne nomme aucune voie, ne dit pas laquelle avance pendant qu'une autre attend, ne fixe aucun plafond de voies actives et ne relève aucune collision. Les collisions qu'un orchestrateur rencontrera sont pourtant lisibles dans les champs « Crates » du plan :
  - **crate `arbitre` (cli)** : 13 stories l'écrivent (ST-03, 04, 05, 06, 07, 10, 11, 13, 15, 19, 22, 27, 32), réparties sur les voies planning, mobile et diff. Le point d'entrée clap, le registre des modules et le serveur MCP y vivent.
  - **crate `arbitre-core` après le socle** : ST-07 (auteur, journal de l'assistant), ST-18 (`envelope`), ST-19 (port `Transport`), ST-30 (port `Ia`). Trois voies différentes modifient la crate dont toutes les autres dépendent.
  - **catalogue d'actions** : chaque story de module y ajoute des actions. Le test de parité de ST-07 (« aucune action CLI absente du MCP ») et celui de ST-21 (GUI) sont dérivés du catalogue. Une voie qui ajoute une action casse ou modifie la parité d'une autre.
  - **table des codes de retour** : voir F-6.
  - **`Cargo.toml` du workspace et `Cargo.lock`** : `[workspace.dependencies]` est créé en ST-01. Il reçoit ensuite des dépendances dans presque toutes les voies : rmcp en ST-07, client Google, OAuth et trousseau en ST-10, RustCrypto en ST-18, HTTP en ST-19, Leptos en ST-20, tree-sitter ou Ruff en ST-22, gix en ST-24, HTTP en ST-30. Avec `members = ["crates/*"]`, chaque crate qui naît change le lockfile : `arbitre-plan` en ST-05, `arbitre-calendar` en ST-10, `arbitre-web` en ST-20, `arbitre-diff` en ST-22, `arbitre-ai` en ST-30. Le lockfile entre donc en conflit à chaque fusion de voies.
  - **configuration CI** : ST-01 fixe la liste wasm32 et les arêtes interdites. ST-05 (`arbitre-plan` en wasm32), ST-20 et ST-22 (`arbitre-diff` natif, arête plan ↛ diff) les modifient.
  - **infrastructure de test partagée** : l'observateur réseau de PROP-2 est « posé [en ST-10] puis réutilisé » par ST-15, 19, 30 et 32, qui sont dans trois voies.
  - **`action.yml`** : ST-24 et ST-29.
- **Preuve** : le plan ne contient ni « voie », ni « plafond », ni « collision ». Rien ne dit à un orchestrateur quelles stories peuvent tourner ensemble sans toucher les mêmes fichiers, et rien ne borne le nombre de voies par la capacité d'arbitrage de Quentin. Chaque cycle C-TDD demande des arbitrages humains (story, specs, archi locale, DA, review). Rien ne relie le nombre de voies à ce débit.

### [BLOQUANT] F-2 : Tailles non exploitables, aucun jalon daté, PL-1 contredit par le plan lui-même
- **Axe** : Cohérence
- **Source** : `04_plan.md` l. 66 (légende : « S ≤ 1 semaine, M ≈ 2 semaines, L ≈ 3 semaines et plus, au budget de 8 à 10 h par semaine ») ; PL-1 (« la valeur arrive environ 3 à 4 mois plus tôt ») ; l. 436 (jalons sans date) ; exigence de la revue (taille S/M/L en soirées, dates de jalon pour 2 à 3 soirées par semaine en cycles C-TDD menés par agents).
- **Tâche(s) concernée(s)** : toutes ; PL-1 ; § Jalons de valeur
- **Description** :
  1. **Mauvaise unité.** Les tailles sont en semaines de travail manuel, à 8 à 10 h par semaine. Ce budget vient de `plan-decision-diff-2026-10-02.md` l. 152 (« 2 soirs + un bloc le week-end »). Le budget fixé ici est de 2 à 3 soirées, sans bloc de week-end. Le plan ne dit pas non plus ce que coûte une story menée par agents. Le temps de frappe disparaît. Restent le temps d'arbitrage et de relecture de Quentin sur les 8 phases du cycle, et le temps machine des tests de mutation et de propriétés. Aucune taille ne le mesure.
  2. **PL-1 n'est pas cohérent avec les tailles du plan.** Le bloc mobile compte ST-18 (S), ST-19 (L), ST-20 (L) et ST-21 (M), soit 1 + 3 + 3 + 2 = 9 semaines. S1 et S2 ajoutent 3 à 4 soirs. Le total est d'environ 2,5 mois, pas de 3 à 4.
  3. **Aucun jalon n'est daté.** En sommant les tailles du plan depuis le 2026-11-02, hors spikes et en prenant les L à leur plancher de 3 semaines, on obtient :

     | Jalon | Fin de story | Semaines cumulées | Date au plus tôt |
     |---|---|---|---|
     | Planning sur CLI utile au quotidien | ST-09 | 18 | 2027-03-08 |
     | Planning complet (blocs, déplacements) | ST-16 | 34 | 2027-06-28 |
     | Téléphone | ST-21 | 43 | 2027-08-30 |
     | Démo decision diff (« 3 violations ») | ST-22 | 46 | 2027-09-20 |
     | Decision diff complet | ST-32 | 66 | 2028-02-07 |

     4 stories sont S, 19 sont M et 8 sont L (ST-17 exclue). Comme L veut dire « 3 semaines et plus », sans plafond, ces dates sont des minimums.
- **Preuve** : le seul chiffre de calendrier du plan, PL-1, ne découle pas de ses propres tailles. Le plan ne donne aucune date qui permettrait de vérifier R-5 (« glissement, abandon »).

### [BLOQUANT] F-3 : Le repli CalDAV de R-1 dépend du succès de Google
- **Axe** : Cas d'erreur
- **Source** : `04_plan.md` R-1 (« Repli : CalDAV avancé (ST-17) ») ; ST-17 (« Dépendances : ST-16 ») ; PL-4 (« ou si S3 conclut au repli ») ; ST-16 → ST-15 → ST-10, qui demande le **verdict S3**.
- **Tâche(s) concernée(s)** : ST-10, ST-15, ST-16, ST-17, S3
- **Description** : si S3 conclut que le client OAuth personnel est refusé par Workspace (critère D-4), ST-10 ne peut pas démarrer, donc ST-15 et ST-16 non plus. ST-17 dépend de ST-16. Le repli exige donc que le chemin Google soit fini.
- **Preuve** : la chaîne est écrite noir sur blanc dans les champs « Dépendances ». R-1 est jugé de probabilité moyenne et « bloque le cœur de la valeur planning ». Sa seule parade ne peut pas s'exécuter dans le cas qu'elle doit couvrir.

### [MAJEUR] F-4 : La logique pure du planning attend un délai externe
- **Axe** : Cohérence
- **Source** : ST-12 (« Dépendances : ST-11 »), ST-11 (« ST-10 »), ST-10 (« verdict S3 ») ; PL-2 (le verdict de S3 peut attendre un admin Workspace, « délai externe incompressible ») ; `03_archi.md` l. 302-305 (P-4, le placement, après P-3).
- **Tâche(s) concernée(s)** : ST-12, ST-13, ST-14
- **Description** : le placement hebdomadaire (ST-12), les questions (ST-13) et le plan de blocs en aperçu (ST-14) sont des fonctions pures en zone Core / Full. ST-09 fournit déjà un simulateur d'`Agenda`. Ces trois stories dépendent pourtant, par ST-11 puis ST-10, du verdict de S3 et de l'accord d'un admin. ST-12 ne cite aucun besoin réel de ST-11 (scheduler) ou de ST-10 (Google) dans son livrable ni dans son critère de complétion.
- **Preuve** : le critère de ST-12 (« propriété sur pile et capacités générées ») n'utilise que des données simulées. La dépendance relève de l'ordre de la séquence, pas d'un besoin technique. Elle bloque la plus grosse part de la voie planning derrière R-1.

### [MAJEUR] F-5 : Dépendances manquantes vers ST-07 (MCP) dans la voie diff
- **Axe** : Cohérence
- **Source** : ST-27 (« Crates : `arbitre` (cli et MCP) » ; critère « acceptation par la CLI et par le MCP identique » ; « Dépendances : ST-26 ») ; ST-32 (« `arbitre` (cli et MCP) » ; « Dépendances : ST-30, ST-23 ») ; ST-07 (naissance de `mcp/`, « Dépendances : ST-06, spike S5 »).
- **Tâche(s) concernée(s)** : ST-07, ST-27, ST-32
- **Description** : ST-27 et ST-32 ont besoin du serveur MCP, qui naît en ST-07 dans la voie planning. Aucune arête ST-07 → ST-27 ou ST-07 → ST-32 n'est déclarée. Par ailleurs, ST-07 dépend de ST-06 (pile ordonnée), alors que l'archi fait dépendre le MCP du seul catalogue (`03_archi.md` l. 300 : « [S-4] MCP dérivé du catalogue » sous S-3). Une brique du socle se retrouve derrière deux stories du planning.
- **Preuve** : en séquence, l'oubli ne se voit pas, puisque ST-07 passe avant ST-27. Dès que les voies tournent en parallèle, la voie diff atteint ST-27 sans le prérequis que le graphe aurait dû signaler.

### [MAJEUR] F-6 : Table des codes de retour « fixée une fois pour toutes » en ST-03, puis étendue
- **Axe** : Cohérence
- **Source** : ST-03 (« table des codes de retour fixée une fois pour toutes ») ; ST-04 (« module non activé ⇒ code dédié ») ; ST-22 (AMB-16, « code "violation" distinct de "erreur d'outil" ») ; ST-24 (« codes distincts "violation" et "outil cassé" lisibles par la CI ») ; `03_archi.md` l. 368.
- **Tâche(s) concernée(s)** : ST-03, ST-04, ST-22, ST-24
- **Description** : ST-03 promet une table définitive, mais trois stories plus tardives la complètent. Le plan ne dit pas si ST-03 contient déjà les codes de ST-04, ST-22 et ST-24, ni qui a le droit de modifier la table ensuite.
- **Preuve** : soit la promesse de ST-03 est fausse, soit les critères de ST-04, ST-22 et ST-24 décrivent un travail déjà fait. Avec des voies parallèles, cette table partagée devient un point de collision (F-1), et c'est un contrat public lu par les automatisations (AC-3.5).

### [MAJEUR] F-7 : ST-19, une story du socle, dépend d'une story du module planning
- **Axe** : Cohérence
- **Source** : ST-19 (« Dépendances : ST-18, ST-04 (revalidation), ST-13 (questions sur élément invalide) ») ; RG-2 du cahier (« un module ne dépend que du socle ») ; RG-1 (« aucune story d'un module ne démarre avant que les stories du socle dont elle dépend soient livrées ») ; AC-5.1 (decision diff utilisable sans planning).
- **Tâche(s) concernée(s)** : ST-13, ST-19, ST-20, ST-21
- **Description** : la synchronisation appartient au socle. Le plan la fait pourtant dépendre de ST-13, une story du planning. La voie mobile se bloque alors après ST-18 jusqu'à la 27e semaine de la voie planning (fin de ST-13 dans le tableau de F-2). De plus, la convergence est vérifiée à travers une fonction de module, alors que le crochet générique `revalider` existe depuis ST-04.
- **Preuve** : un utilisateur qui n'active que `diff` (US-5, cas nominal) dépend d'une synchro dont la complétion a été prouvée par le planning. Le graphe présente la voie mobile comme autonome dès ST-02 (« ST-18 (dès ST-02) »), et ce n'est pas le cas.

### [MAJEUR] F-8 : Spikes placés « juste à temps » par rapport à une séquence qui n'existera pas
- **Axe** : Cohérence
- **Source** : `04_plan.md` § Spikes (S4 « Après ST-21 », S1 et S2 « Après ST-19 », S5 « Après ST-06 ») ; `03_archi.md` l. 404 (« S3 et S4 peuvent tourner en parallèle de S-1 à S-3 ») ; exigence de la revue (parallélisme dès ST-04).
- **Tâche(s) concernée(s)** : S1, S2, S4, S5 ; ST-07, ST-20, ST-22
- **Description** : les spikes sont placés selon la séquence linéaire. ST-22 peut partir dès ST-04 d'après le graphe (« ST-22 dès ST-04 »), mais S4 n'est placé qu'après ST-21. S1 et S2 portent le risque le plus élevé du plan (R-2, « Haute »). Ils sont placés aussi tard que possible, alors qu'ils peuvent remettre en cause le port `Store` posé en ST-02. Leur critère (7) mesure aussi le poids des fuseaux horaires, dont le plan dit qu'il ne faut pas le « découvrir en ST-20 ».
- **Preuve** : le plan se contredit. Il veut mesurer le poids des fuseaux avant ST-20, mais il place S1 juste avant ST-20. Avec des voies parallèles, aucun spike n'a de position définie.

### [MAJEUR] F-9 : Le calendrier de decision diff n'est pas confronté à la fenêtre de marché du plan du 2026-10-02
- **Axe** : Complétude (métier)
- **Source** : `inputs/plan-decision-diff-2026-10-02.md` l. 69 (« Le tier Team se ferme vers mi-2027 »), l. 72 (« Arriver dans 8 mois = trop tard sur ce segment »), l. 37 (quickwin montrable en call) ; `04_plan.md` R-5 et l. 291 (ST-22 « montrable en entretien ») ; G-2 (entretiens « dès maintenant ») ; arbitrage : l'ordre du cadrage l'emporte.
- **Tâche(s) concernée(s)** : ST-22, G-2, garde-fou RG-G8
- **Description** : l'ordre socle → planning → décision diff est arbitré, et la revue ne le conteste pas. Mais avec les tailles du plan, la démo ST-22 tombe au plus tôt le 2027-09-20 (F-2), après la fermeture estimée de la fenêtre Team. Pendant ce temps, les entretiens G-2 ont lieu sans rien à montrer. Le plan ne signale ni ce décalage ni son effet sur le garde-fou des 10 entretiens.
- **Preuve** : R-5 cite ST-22 comme démo d'entretien sans dire quand elle arrive. Quentin ne peut pas arbitrer un décalage que le plan ne montre pas.

### [MAJEUR] F-10 : Le parallélisme de la voie diff entre en tension avec l'ordre des modules, sans arbitrage demandé
- **Axe** : Cohérence
- **Source** : `CADRAGE.md` (« Ordre de développement : socle → planning → decision diff ») ; RG-1 du cahier ; `04_plan.md` l. 434 (« la séquence suit l'ordre des modules du cadrage ») ; exigence de la revue (parallélisme dès ST-04).
- **Tâche(s) concernée(s)** : ST-22 à ST-32
- **Description** : faire tourner la voie diff pendant le planning s'écarte de la lecture littérale de « puis ». La seconde phrase de RG-1 ne lie un module qu'aux stories du socle dont il dépend, et le plan s'en sert déjà pour PL-1. Le plan ne dit pas si cette lecture vaut aussi pour lancer diff avant la fin du planning. Aucun point PL ne pose la question.
- **Preuve** : PL-1 justifie d'avancer le planning avant le mobile avec cet argument. Le même argument permettrait de lancer la voie diff en parallèle, mais le plan ne le formule pas et ne le soumet pas à Quentin.

### [MAJEUR] F-11 : ST-03 est trop grosse et fige le contrat que toutes les voies partagent
- **Axe** : Over/Under-engineering
- **Source** : ST-03 (L ; zone « Core / Full pour l'export et l'import ; Supporting / Std pour le catalogue et les codes » ; livrable : descripteur, sortie, taxonomie d'erreurs, dérivation clap, `--json`, export, import) ; ST-04 (vue `Store` restreinte à l'espace de noms) ; INV-24.
- **Tâche(s) concernée(s)** : ST-03, ST-04
- **Description** : ST-03 regroupe deux sujets indépendants, de deux niveaux de rigueur : le catalogue et la CLI d'un côté, l'export et l'import par propriétés de l'autre. Elle fixe aussi le format d'export versionné avant que les espaces de noms des modules existent (ST-04). ST-04 doit pourtant garantir qu'un import conserve les données d'un module désactivé (INV-24).
- **Preuve** : le format d'export est figé avant de connaître la structure en espaces de noms qu'il devra transporter. La plus grosse story du socle est aussi celle dont sortent le catalogue et la CLI que toutes les voies vont étendre (F-1).

### [MAJEUR] F-12 : AC-1.1 (tout faire hors ligne) n'est vérifié que là où il n'y a rien à vérifier
- **Axe** : Complétude
- **Source** : AC-1.1 ; PROP-1 (« toute opération sur toute donnée réussit hors ligne ») ; `04_plan.md` § Invariants (PROP-1 → ST-02, ST-10, ST-20) ; ST-02 (« aucune surface utilisateur »).
- **Tâche(s) concernée(s)** : ST-02, ST-05 à ST-16
- **Description** : ST-02 n'a aucune opération utilisateur, donc PROP-1 y est vide. Les stories du planning, qui créent les opérations, ne rejouent pas PROP-1. ST-10 ne le vérifie que pour la lecture d'agenda. Le plan rejoue PROP-2 à chaque nouvelle sortie réseau, mais pas PROP-1 à chaque nouvelle opération.
- **Preuve** : AC-1.1 ne figure dans aucun champ « Couvre ». Sur toute la voie planning, rien ne vérifie que créer, modifier, faire, réordonner, répondre à une question ou planifier un jour fonctionne réseau coupé.

### [MAJEUR] F-13 : Le partage du secret de chiffrement entre appareils n'a pas de story
- **Axe** : Complétude
- **Source** : `02_specs.md` l. 1086 (« Secret de chiffrement de l'utilisateur… partage entre appareils à définir — Bloquant pour US-2 ») ; ST-18 (phrase secrète, Argon2id) ; ST-19 ; ST-20 ; arbitrage CONFLIT-7 (« le téléphone reçoit le résultat par la synchronisation »).
- **Tâche(s) concernée(s)** : ST-18, ST-19, ST-20, ST-11
- **Description** : aucune story ne couvre la façon dont un deuxième appareil, puis le téléphone, obtient le secret, ni ce que fait une synchro qui tourne sans personne. Le recalcul planifié de ST-11 doit pouvoir pousser son résultat vers le téléphone sans que l'utilisateur saisisse sa phrase secrète. Le critère de ST-19 ne couvre pas ce cas. Le plan ne marque pas non plus ce point comme `[À DÉFINIR]` dans une story.
- **Preuve** : 02_specs.md classe cette dépendance comme bloquante pour US-2 et la laisse à définir. Le plan ne la reprend nulle part. C'est une spec orpheline.

### [MAJEUR] F-14 : Jeton d'agenda dans le trousseau, lu par un recalcul sans session
- **Axe** : Cas d'erreur
- **Source** : ST-10 (« jeton dans le trousseau ») ; ST-11 (recalcul par launchd ou systemd, « outil fermé ») ; arbitrage CONFLIT-7 et 8 ; AC-13.7.
- **Tâche(s) concernée(s)** : ST-10, ST-11
- **Description** : le recalcul planifié lit l'agenda avec le jeton rangé dans le trousseau en ST-10. Ni ST-10, ni ST-11, ni aucun spike ne vérifie que ce trousseau est accessible depuis un agent launchd, ou depuis un timer systemd sans session D-Bus. Le plan ne dit pas non plus ce que fait le recalcul si le jeton est inaccessible ou expiré, ni comment l'utilisateur l'apprend à l'ouverture.
- **Preuve** : le critère de ST-11 vérifie l'idempotence et le rattrapage, pas l'accès aux secrets en tâche de fond. Sans traitement explicite, AC-13.7 (« son résultat est prêt quand il l'ouvre ») échoue sans bruit, contre RG-G7.

### [MAJEUR] F-15 : Les décisions IA de ST-31 n'ont ni déclencheur défini, ni PROP-2
- **Axe** : Sécurité & intégrité
- **Source** : ST-31 ; arbitrage CONFLIT-7 et 8 (« un fournisseur d'IA n'est jamais sollicité automatiquement ») ; D-2 et ST-24 (CI sans clé) ; `04_plan.md` § Invariants (PROP-2 → ST-10, 15, 19, 30, 32, sans ST-31) ; CONFLIT-16.
- **Tâche(s) concernée(s)** : ST-24, ST-29, ST-31
- **Description** : ST-31 envoie du code de PR à un fournisseur d'IA, ce qui est une sortie réseau. Le plan ne dit pas qui déclenche l'appel. Si c'est l'Action CI, il faut une clé dans la CI, ce que D-2 exclut, et l'appel devient automatique, ce que l'arbitrage exclut aussi. Si c'est un appel local, il manque l'avertissement de CONFLIT-16, que seule ST-32 porte. PROP-2 n'est pas rejoué en ST-31.
- **Preuve** : la liste des stories qui rejouent PROP-2 omet ST-31, alors que c'est une nouvelle sortie vers l'IA, le cas même que PROP-2 contrôle.

### [MAJEUR] F-16 : Les seuils de mutation sont exigés, mais aucune story ne pose l'outil
- **Axe** : Complétude
- **Source** : `04_plan.md` l. 66 (Full : mutation ≥ 85 % ; Std : ≥ 70 %) ; ST-01, livrable CI (« `cargo test`, wasm32 sur core, arêtes interdites, gitleaks »).
- **Tâche(s) concernée(s)** : ST-01 et toutes les stories Full et Std
- **Description** : 21 stories ont un seuil de mutation. Aucune story ne pose l'outil de mutation, ne le met en CI ou ne fixe comment le seuil est mesuré et appliqué. Le coût en temps machine d'une campagne de mutation n'entre pas non plus dans les tailles (F-2).
- **Preuve** : le livrable CI de ST-01 est exhaustif et ne contient pas la mutation. Dans l'état du plan, le premier critère Full (ST-01) ne peut pas être constaté.

### [MAJEUR] F-17 : Aucune story ne choisit les dépendances partagées entre voies
- **Axe** : Cohérence
- **Source** : ST-10 (client HTTP Google), ST-19 (transport relais), ST-24 (« git en sous-processus ou `gix` »), ST-30 (client HTTP), ST-18 (`getrandom` avec feature wasm) ; ST-01 (`[workspace.dependencies]`).
- **Tâche(s) concernée(s)** : ST-10, ST-19, ST-30
- **Description** : trois stories de trois voies différentes ont besoin d'un client HTTP (et d'un runtime asynchrone, avec rmcp en ST-07). Aucune ne porte le choix. Menées en parallèle, elles peuvent en choisir trois différents, qui finiront tous dans le même binaire `arbitre`.
- **Preuve** : `[workspace.dependencies]` existe justement pour unifier les versions (ST-01). Le plan crée le mécanisme mais ne dit pas qui choisit les dépendances communes.

### [MINEUR] F-18 : L'observateur réseau PROP-2 appartient à ST-10
- **Axe** : Cohérence
- **Source** : ST-10 (« PROP-2 (observateur réseau, posé ici puis réutilisé à chaque nouvelle sortie) »).
- **Tâche(s) concernée(s)** : ST-10, ST-19, ST-30
- **Description** : en parallèle, ST-19 ou ST-30 peuvent ouvrir la première sortie réseau avant ST-10. L'outillage qu'elles doivent réutiliser n'existe pas encore, et aucune arête du graphe ne le dit.
- **Preuve** : ST-19 et ST-30 ne dépendent pas de ST-10.

### [MINEUR] F-19 : ST-25 dépend de ST-24 sans raison donnée
- **Axe** : Cohérence
- **Source** : ST-25 (« vérité terrain = la vérification » ; « Dépendances : ST-24 »).
- **Tâche(s) concernée(s)** : ST-24, ST-25
- **Description** : le harnais utilise les verdicts de vérification comme labels, ce que livrent ST-22 et ST-23. Il ne se sert ni de la GitHub Action ni de l'union base ∪ tête de ST-24.
- **Preuve** : ni le livrable ni le critère de ST-25 ne citent ST-24. La voie diff est allongée pour rien.

### [MINEUR] F-20 : Stories sans valeur seules, non comptées comme telles
- **Axe** : Over/Under-engineering
- **Source** : ST-02 (« aucune surface utilisateur ») ; ST-18 (enveloppe sans transport) ; ST-30 (« placée juste avant ST-31, sa première utilisatrice ») ; R-5 (« stories petites et livrables seules »).
- **Tâche(s) concernée(s)** : ST-02, ST-18, ST-30
- **Description** : trois stories n'apportent rien à l'utilisateur avant la suivante. Le plan le reconnaît pour ST-02 seulement. ST-30 est justifiée par sa place dans la séquence, et cette justification disparaît dès que les voies tournent en parallèle.
- **Preuve** : la parade de R-5 (« livrables seules ») ne vaut pas pour ces trois stories.

### [MINEUR] F-21 : PROP-7 côté GUI attribué à ST-21 sans critère
- **Axe** : Edge cases
- **Source** : ST-21 (« PROP-7 (côté GUI) ») ; D-1 (la PWA est toujours une réplique, avec son propre stockage) ; edge US-3 (« commande lancée pendant que l'interface graphique modifie la même donnée »).
- **Tâche(s) concernée(s)** : ST-21
- **Description** : avec D-1, la CLI et la GUI n'écrivent jamais dans le même stockage. Leur concurrence passe par la synchro. Le critère de ST-21 ne contient aucun cas d'écritures concurrentes entre la CLI et la GUI.
- **Preuve** : PROP-7 est listé dans « Couvre » mais ne figure pas dans le critère de complétion.

### [MINEUR] F-22 : PL-3 ne dit pas comment le relais intégré à la CLI sert la PWA
- **Axe** : Over/Under-engineering
- **Source** : PL-3 (relais dans la CLI) ; D-6 (le relais sert les fichiers de la PWA) ; ST-20 (« relais (fichiers statiques) »).
- **Tâche(s) concernée(s)** : ST-19, ST-20
- **Description** : le plan ne dit pas si les fichiers de la PWA sont embarqués dans le binaire `arbitre` ou servis depuis un répertoire. S'ils sont embarqués, la compilation de la CLI dépend du build wasm d'`arbitre-web` dans toutes les voies. Dans les deux cas, le serveur qui héberge le relais reçoit tout le binaire, avec les modules, le client Google et le MCP.
- **Preuve** : PL-3 évalue la frontière de type (`Enveloppe`), pas le couplage de build ni le contenu déployé sur le relais.

### [MINEUR] F-23 : PL-5 est sans objet au niveau plan, mais ses effets aval ne sont pas notés
- **Axe** : Cohérence
- **Source** : PL-5 ; arbitrages des specs (« pour les autres CONFLIT… la résolution proposée vaut par défaut ») ; ST-24 ; ST-32 et l'edge US-18 (« adopter et doit alors corriger »).
- **Tâche(s) concernée(s)** : ST-22, ST-24, ST-32
- **Description** : la résolution par défaut de CONFLIT-17 s'applique déjà selon les arbitrages, donc PL-5 n'est pas une question ouverte. En revanche, si ST-22 tranche autrement, les critères de ST-24 et ST-32 changent : avec un verdict sur tout l'état, une PR sur un dépôt qui a déjà des violations échoue. Le plan ne signale pas ce lien.
- **Preuve** : ST-24 et ST-32 ne mentionnent pas CONFLIT-17.

---

## Points PL-1 à PL-5

| # | Constat DA |
|---|---|
| PL-1 | Le principe est défendable (RG-1, seconde phrase). Le chiffre de « 3 à 4 mois » n'est pas sourcé et contredit les tailles du plan, qui donnent environ 2,5 mois (F-2). La même lecture de RG-1 devrait être appliquée à la voie diff, mais elle n'est pas soumise à Quentin (F-10). |
| PL-2 | Juste : S3 a le seul délai externe du plan. Mais il ne protège rien tant que le repli dépend de Google (F-3) et que la logique pure attend S3 (F-4). |
| PL-3 | Défendable, avec deux points non traités : couplage de build et contenu déployé sur le relais (F-22) ; il ajoute aussi une collision dans `arbitre` (F-1). |
| PL-4 | Le principe « à la demande » est juste, mais la clause « ou si S3 conclut au repli » ne peut pas s'exécuter (F-3). |
| PL-5 | Déjà tranché par défaut par les arbitrages des specs. Il manque le lien avec ST-24 et ST-32 (F-23). |

## Axes sans faille détectée
- **Complétude des ID** : INV-1 à 84 et PROP-1 à 24 sont tous attribués à une story, ainsi que toutes les RG-x.y et tous les AC sauf AC-1.1 (F-12). Les AMB, CONFLIT, MANQUE et RENVOI sont tous rattachés à une story. Les seules spécifications orphelines sont le partage du secret (F-13) et le hors-ligne effectif (F-12).
- **Edge cases** : les edge cases des matrices US-1 à US-18 sont couverts par au moins une story, ou explicitement renvoyés à un cycle (RENVOI-1 à 9, AMB à définir). Seul le cas F-21 n'a pas de critère.
- **Régressions** : la codebase est vide (`add`, « Hello, world! ») et le plan corrige ses anti-patterns avant tout commit (G-0, ST-01, ST-02). Le schéma générique de ST-02 évite les migrations par module. Entre stories, les tests de conformité (ST-04) et de parité (ST-07, ST-21) encadrent les régressions. Restent les collisions entre voies (F-1).

## Recommandation
Retour au Planner. Il doit corriger les 3 BLOQUANTS (F-1, F-2, F-3) et les 14 MAJEURS (F-4 à F-17) avant de reproposer la découpe. Les deux exigences de cette revue sont non négociables :
1. une taille S/M/L **en soirées** pour chaque story et chaque spike, avec des dates de jalon cohérentes avec 2 à 3 soirées par semaine en cycles C-TDD menés par agents ;
2. des voies parallèles explicites après ST-04, la liste des points de collision (fichiers et crates partagés, catalogue d'actions, table des codes, `Cargo.toml` et `Cargo.lock` du workspace, configuration CI, outillage de test partagé) et un plafond de voies actives fixé par la capacité d'arbitrage de Quentin, pas par le graphe.

La décision F-10 (diff en parallèle du planning, malgré l'ordre du cadrage) revient à Quentin, pas au Planner. Itération 1 sur 3.
