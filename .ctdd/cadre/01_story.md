# Cahier des Charges — OpenArbitre, cadre global (socle, planning, decision diff)

## Contexte
Un travailleur du savoir qui délègue une partie de son travail à des agents d'IA perd la maîtrise de deux choses : son temps (ce qu'il a promis, quand il le fera, si sa semaine tient) et les arbitrages pris en son nom (ce qu'un agent a décidé dans une modification de code). OpenArbitre lui rend cette maîtrise avec un outil personnel dont les données restent chez lui, utilisable depuis l'ordinateur, le téléphone, la ligne de commande et n'importe quel assistant d'IA. Ce cahier couvre le cadre complet, dans l'ordre où il sera développé : le socle commun, puis le module planning, puis le module decision diff.

## Utilisateurs cibles
- **Utilisateur solo** (persona principal) : professionnel qui jongle entre réunions, engagements datés et décisions en attente, et qui veut savoir chaque matin si sa journée et sa semaine tiennent. Il utilise l'outil sur son ordinateur et le consulte sur son téléphone.
- **Collègue de l'utilisateur** (persona passif) : voit dans l'agenda partagé les créneaux que l'utilisateur s'est réservés, sans en connaître le détail.
- **Assistant d'IA de l'utilisateur** (acteur non humain) : lit et modifie les données de l'utilisateur pour son compte, quel que soit son fournisseur.
- **Développeur qui fait travailler des agents de code** (persona du module decision diff) : relit les modifications proposées par un agent et veut contrôler les arbitrages pris plutôt que relire chaque ligne.
- **Mainteneur d'un dépôt de code** (persona du module decision diff) : déclare les règles que son code doit respecter et veut qu'elles soient vérifiées à chaque modification, sans dépendre d'une IA.

---

## User Stories

### Partie A — Socle (à développer en premier)

### US-1 : Garder ses données sur sa machine
**En tant qu'** utilisateur solo, **je veux** que toutes mes données soient stockées sur mon ordinateur et utilisables sans connexion, **afin de** rester propriétaire de mes informations et de travailler même hors ligne.

**Acceptance Criteria :**
- [ ] AC-1.1 : Ordinateur déconnecté d'internet, l'utilisateur peut consulter, créer, modifier et supprimer chacune de ses données.
- [ ] AC-1.2 : Aucune donnée de l'utilisateur n'est envoyée à un service tiers, à l'exception de l'agenda qu'il a connecté et du fournisseur d'IA qu'il a choisi, et seulement au moment où il les sollicite.
- [ ] AC-1.3 : L'utilisateur peut exporter l'intégralité de ses données dans un fichier lisible, et les réimporter sur une autre machine sans perte.
- [ ] AC-1.4 : Après une fermeture brutale de l'outil (coupure de courant, arrêt forcé), toute modification confirmée à l'utilisateur est retrouvée à la réouverture.

**Cas nominaux :**
- L'utilisateur installe l'outil, crée ses premières données, coupe le wifi dans le train et continue à travailler ; tout est là au retour.

**Edge cases identifiés :**
- Disque plein au moment d'une modification : la modification est refusée avec un message explicite, aucune donnée existante n'est altérée.
- Import d'un fichier d'export abîmé ou incomplet : l'import est refusé en entier avec un message, les données existantes restent intactes.
- Import sur une machine qui contient déjà des données : comportement à définir (remplacement, fusion, refus) — voir Questions ouvertes.

### US-2 : Retrouver ses données sur son téléphone
**En tant qu'** utilisateur solo, **je veux** retrouver sur mon téléphone les données de mon ordinateur, et inversement, **afin de** consulter et ajuster mon planning loin de mon bureau.

**Acceptance Criteria :**
- [ ] AC-2.1 : Une modification faite sur l'ordinateur apparaît sur le téléphone en moins d'une minute lorsque les deux appareils sont connectés.
- [ ] AC-2.2 : Une modification faite sur le téléphone hors ligne apparaît sur l'ordinateur au plus tard une minute après le retour de la connexion.
- [ ] AC-2.3 : Le relais de synchronisation est hébergé par l'utilisateur lui-même ; aucun service opéré par un tiers n'est nécessaire.
- [ ] AC-2.4 : Le relais ne fait que recopier ; l'outil reste entièrement utilisable sur chaque appareil quand le relais est arrêté.
- [ ] AC-2.5 : Le téléphone s'utilise sans passer par une boutique d'applications.
- [ ] AC-2.6 : Aucune modification n'est perdue silencieusement : quand deux appareils ont modifié la même donnée hors ligne, l'utilisateur voit le résultat retenu et peut retrouver l'autre version.

**Cas nominaux :**
- L'utilisateur repousse une tâche depuis son téléphone dans le métro ; en arrivant au bureau, son ordinateur l'affiche déjà repoussée.

**Edge cases identifiés :**
- Même donnée modifiée différemment sur les deux appareils hors ligne : la règle de résolution est à définir (voir Questions ouvertes), mais l'autre version reste récupérable.
- Donnée supprimée sur un appareil et modifiée sur l'autre : à trancher (voir Questions ouvertes) ; jamais de perte silencieuse.
- Relais inaccessible pendant plusieurs jours : chaque appareil continue à fonctionner ; la synchronisation reprend d'elle-même au retour du relais, et l'utilisateur voit depuis quand ses appareils ne se sont pas synchronisés.
- Un tiers non autorisé tente de lire les données sur le relais : il ne doit pas pouvoir les lire (voir Hypothèses).

### US-3 : Piloter l'outil en ligne de commande
**En tant qu'** utilisateur solo, **je veux** accéder à chaque module par une commande courte (`arbitre plan`, `arbitre diff`), **afin de** l'utiliser depuis mon terminal et de l'intégrer à mes automatisations.

**Acceptance Criteria :**
- [ ] AC-3.1 : Toute action disponible dans l'interface graphique est disponible en ligne de commande.
- [ ] AC-3.2 : Le module planning s'appelle par `arbitre plan`, le module decision diff par `arbitre diff`.
- [ ] AC-3.3 : Une commande mal formée affiche ce qui est attendu et ne modifie aucune donnée.
- [ ] AC-3.4 : Chaque commande peut produire une sortie lisible par un autre programme, en plus de la sortie lisible par un humain.
- [ ] AC-3.5 : Une commande signale son échec par un code de retour distinct du succès, pour qu'une automatisation puisse le détecter.

**Cas nominaux :**
- L'utilisateur tape `arbitre plan` le matin et voit sa journée, sa charge et les questions en attente.

**Edge cases identifiés :**
- Commande lancée pendant que l'interface graphique modifie la même donnée : aucune des deux modifications n'est perdue silencieusement.
- Commande d'un module non activé : message explicite indiquant que le module n'est pas activé.

### US-4 : Laisser un assistant d'IA agir pour moi, quel que soit son fournisseur
**En tant qu'** utilisateur solo, **je veux** que mon assistant d'IA puisse lire et modifier mes données, et choisir librement le fournisseur de cet assistant (dont un modèle qui tourne sur ma machine), **afin de** ne dépendre d'aucun fournisseur et d'en changer sans perdre mes données ni mes habitudes.

**Acceptance Criteria :**
- [ ] AC-4.1 : Un assistant d'IA compatible avec le protocole ouvert d'outils pour assistants peut effectuer chaque action disponible en ligne de commande.
- [ ] AC-4.2 : Lorsque l'outil sollicite lui-même un modèle d'IA, l'utilisateur choisit le fournisseur parmi au moins : Claude, GPT, Mistral et un modèle local.
- [ ] AC-4.3 : Changer de fournisseur ne demande que de modifier un réglage ; aucune donnée n'est perdue et aucune fonction ne disparaît.
- [ ] AC-4.4 : Toutes les fonctions dont la description n'exige pas un modèle d'IA restent utilisables sans aucun fournisseur configuré.
- [ ] AC-4.5 : Une action d'un assistant qui supprime ou déplace des données apparaît dans un historique consultable, avec la date et l'auteur « assistant ».

**Cas nominaux :**
- L'utilisateur demande à son assistant « ajoute la relecture du contrat pour jeudi » ; l'élément apparaît dans sa pile, avec une durée proposée.

**Edge cases identifiés :**
- Fournisseur d'IA indisponible ou clé invalide : la fonction qui en dépend affiche une erreur explicite ; les autres fonctions continuent.
- Modèle local trop lent ou absent : même comportement.
- Assistant qui demande une action invalide (date inexistante, élément introuvable) : refus explicite, aucune donnée modifiée.

### US-5 : N'activer que les modules utiles
**En tant qu'** utilisateur solo, **je veux** activer ou désactiver chaque module indépendamment, **afin de** n'avoir que les fonctions dont j'ai besoin.

**Acceptance Criteria :**
- [ ] AC-5.1 : L'utilisateur peut utiliser le module planning sans le module decision diff, et inversement.
- [ ] AC-5.2 : Désactiver un module ne supprime pas ses données ; le réactiver les retrouve intactes.
- [ ] AC-5.3 : Un module n'affiche jamais de donnée ni de fonction d'un autre module.
- [ ] AC-5.4 : Un module désactivé n'apparaît ni dans l'interface, ni en ligne de commande, ni auprès de l'assistant d'IA.

**Cas nominaux :**
- Un développeur qui n'a besoin que du decision diff installe l'outil, active ce seul module et ne voit jamais le planning.

**Edge cases identifiés :**
- Données d'un module importées alors que ce module est désactivé : elles sont conservées et deviennent visibles à l'activation.

---

### Partie B — Module planning (à développer en second)

### US-6 : Tenir ma pile d'éléments à faire
**En tant qu'** utilisateur solo, **je veux** consigner dans une seule pile mes gestes à faire, mes décisions en attente et mes engagements datés, **afin de** ne rien oublier et de tout voir au même endroit.

**Acceptance Criteria :**
- [ ] AC-6.1 : Chaque élément a un intitulé et un type parmi : geste, décision en attente, engagement daté.
- [ ] AC-6.2 : Un engagement daté porte obligatoirement une échéance ; geste et décision en attente peuvent en porter une.
- [ ] AC-6.3 : L'utilisateur peut créer, modifier, marquer comme fait et supprimer un élément.
- [ ] AC-6.4 : Un élément marqué fait sort de la pile active et reste consultable dans un historique.
- [ ] AC-6.5 : Créer un élément depuis le téléphone, l'ordinateur, la ligne de commande ou l'assistant d'IA donne le même résultat.

**Cas nominaux :**
- En sortant d'une réunion, l'utilisateur note « envoyer le devis à X avant vendredi » ; l'élément entre dans la pile comme engagement daté.

**Edge cases identifiés :**
- Échéance déjà passée à la création : l'élément est accepté et signalé en retard.
- Engagement daté sans échéance : refusé avec un message explicite.
- Élément supprimé alors qu'un bloc existe dans l'agenda : le bloc est retiré de l'agenda.

### US-7 : Obtenir une durée pour chaque élément, et la corriger
**En tant qu'** utilisateur solo, **je veux** que l'outil me propose une durée pour chaque élément et que mes corrections soient retenues, **afin de** ne pas estimer tout à la main et d'avoir des estimations de plus en plus justes.

**Acceptance Criteria :**
- [ ] AC-7.1 : Chaque nouvel élément reçoit une durée proposée par l'outil, visible et modifiable.
- [ ] AC-7.2 : La durée corrigée par l'utilisateur remplace la durée proposée pour cet élément.
- [ ] AC-7.3 : Les corrections passées de l'utilisateur sont prises en compte dans les durées proposées pour les éléments suivants (voir Hypothèses sur ce que « semblable » veut dire).
- [ ] AC-7.4 : L'utilisateur peut voir, pour un élément, si sa durée est proposée par l'outil ou fixée par lui.
- [ ] AC-7.5 : Une durée proposée est disponible même sans aucun fournisseur d'IA configuré.

**Cas nominaux :**
- L'outil propose 1 h pour « relire le contrat » ; l'utilisateur corrige à 2 h. La semaine suivante, « relire le contrat de Y » est proposé plus près de 2 h.

**Edge cases identifiés :**
- Durée nulle ou négative saisie : refusée.
- Durée saisie supérieure à la capacité d'une journée entière : comportement à définir (voir Questions ouvertes : découpage sur plusieurs jours ?).
- Aucun historique de correction (premier usage) : une durée par défaut par type d'élément est proposée.

### US-8 : Voir ma charge par jour face à ma capacité
**En tant qu'** utilisateur solo, **je veux** voir pour chaque jour le nombre d'heures planifiées face aux heures dont je dispose réellement, **afin de** repérer d'un coup d'œil les jours qui débordent.

**Acceptance Criteria :**
- [ ] AC-8.1 : La capacité d'un jour vaut 7 h 30 moins la durée des réunions de ce jour lues dans l'agenda.
- [ ] AC-8.2 : La capacité d'un jour n'est jamais négative : un jour où les réunions dépassent 7 h 30 a une capacité de 0.
- [ ] AC-8.3 : La charge d'un jour vaut la somme des durées des éléments planifiés ce jour-là.
- [ ] AC-8.4 : Un jour dont la charge dépasse la capacité est signalé visuellement comme en débordement, avec le nombre d'heures en trop.
- [ ] AC-8.5 : Après un changement dans l'agenda (réunion ajoutée, déplacée, annulée), la capacité affichée reflète ce changement au plus tard au recalcul suivant.
- [ ] AC-8.6 : Les blocs posés par l'outil lui-même ne sont pas comptés comme des réunions dans le calcul de capacité.

**Cas nominaux :**
- Mardi compte 3 h de réunions : capacité 4 h 30. L'utilisateur y a 5 h d'éléments : mardi s'affiche en débordement de 30 min.

**Edge cases identifiés :**
- Réunions qui se chevauchent : le temps chevauché n'est compté qu'une fois.
- Réunion d'une journée entière ou absence (congé) : capacité 0 — voir Hypothèses.
- Réunion déclinée par l'utilisateur dans l'agenda : non comptée — voir Hypothèses.
- Réunion en dehors des heures de travail (soir) : à trancher (voir Questions ouvertes).
- Agenda injoignable : la dernière capacité connue est affichée, marquée comme non à jour avec sa date.

### US-9 : Consulter et reprioriser la pile entière
**En tant qu'** utilisateur solo, **je veux** consulter toute ma pile triée par ordre de priorité et la réordonner, **afin de** décider moi-même de ce qui passe en premier.

**Acceptance Criteria :**
- [ ] AC-9.1 : La pile affiche tous les éléments actifs, planifiés ou non, dans un ordre unique.
- [ ] AC-9.2 : L'utilisateur peut déplacer un élément dans l'ordre ; le nouvel ordre est conservé.
- [ ] AC-9.3 : Chaque élément affiche son type, sa durée, son échéance éventuelle et son jour planifié s'il en a un.
- [ ] AC-9.4 : Les éléments non planifiés sont identifiables d'un coup d'œil.
- [ ] AC-9.5 : La pile fait foi pour l'ordre, les durées et les éléments non planifiés : aucune modification de l'agenda ne change l'ordre de la pile.

**Cas nominaux :**
- Le lundi, l'utilisateur parcourt toute sa pile, remonte un sujet devenu prioritaire et voit son planning de la semaine se mettre à jour.

**Edge cases identifiés :**
- Pile de plusieurs centaines d'éléments : la pile reste consultable entière.
- Pile vide : un message l'indique, sans erreur.

### US-10 : Voir mes éléments comme des blocs dans mon agenda
**En tant qu'** utilisateur solo, **je veux** que les éléments planifiés apparaissent comme des blocs dans mon agenda, **afin de** protéger ce temps et de voir ma journée dans l'outil que j'utilise déjà.

**Acceptance Criteria :**
- [ ] AC-10.1 : Chaque élément planifié produit un bloc dans l'agenda de l'utilisateur, au jour planifié.
- [ ] AC-10.2 : Un bloc dure au minimum 30 minutes.
- [ ] AC-10.3 : Les éléments dont la durée est inférieure à 30 minutes ne produisent pas de bloc propre : ils sont regroupés dans un bloc quotidien unique.
- [ ] AC-10.4 : Les blocs ne chevauchent jamais une réunion existante.
- [ ] AC-10.5 : Le bloc d'un élément marqué fait ou supprimé disparaît de l'agenda ; le bloc quotidien rétrécit d'autant, sans descendre sous 30 minutes.
- [ ] AC-10.6 : L'outil ne modifie ni ne supprime jamais un événement d'agenda qu'il n'a pas créé.

**Cas nominaux :**
- Mercredi, « préparer la démo » (2 h) devient un bloc de 2 h entre deux réunions, et trois petits gestes de 10 min sont réunis dans un bloc quotidien de 30 min.

**Edge cases identifiés :**
- Aucun créneau libre assez long dans le jour planifié pour un élément : comportement à définir (découpage en plusieurs blocs ? question à l'utilisateur ?) — voir Questions ouvertes.
- Petits gestes dont la somme dépasse largement 30 min : le bloc quotidien grandit ; seuil éventuel à trancher (voir Questions ouvertes).
- Agenda injoignable au moment de poser un bloc : l'élément reste planifié dans la pile, le bloc est posé dès que l'agenda redevient joignable.

### US-11 : Montrer à mes collègues que je suis occupé, sans rien révéler
**En tant que** collègue de l'utilisateur, **je veux** voir les créneaux que l'utilisateur s'est réservés, **afin de** ne pas lui proposer de réunion par-dessus.
**En tant qu'** utilisateur solo, **je veux** que ces créneaux ne révèlent aucune information sensible, **afin de** protéger la confidentialité de mon travail.

**Acceptance Criteria :**
- [ ] AC-11.1 : Chaque bloc apparaît aux collègues comme occupé, avec le titre « Focus — <sujet> ».
- [ ] AC-11.2 : Le bloc visible des collègues ne contient ni description, ni échéance, ni type d'élément, ni aucune autre information que ce titre.
- [ ] AC-11.3 : L'utilisateur peut définir le sujet affiché publiquement indépendamment de l'intitulé de l'élément.
- [ ] AC-11.4 : Le bloc quotidien des petits gestes a un titre générique qui ne cite aucun des gestes.

**Cas nominaux :**
- L'élément « négocier l'augmentation de Z » s'affiche aux collègues « Focus — RH ».

**Edge cases identifiés :**
- Intitulé contenant une information sensible et aucun sujet public défini : le sujet affiché par défaut ne doit pas reprendre l'intitulé tel quel — règle à trancher (voir Questions ouvertes).

### US-12 : Déplacer un bloc dans mon agenda et que la pile suive
**En tant qu'** utilisateur solo, **je veux** pouvoir déplacer un bloc directement dans mon agenda et que l'outil en tienne compte, **afin de** ne pas avoir à faire la modification deux fois.

**Acceptance Criteria :**
- [ ] AC-12.1 : L'agenda fait foi pour le « quand » : un bloc déplacé à la main dans l'agenda change le jour planifié de l'élément dans la pile.
- [ ] AC-12.2 : La prise en compte a lieu au plus tard au recalcul suivant.
- [ ] AC-12.3 : Déplacer un bloc ne change ni l'ordre de la pile ni la durée de l'élément.
- [ ] AC-12.4 : Si le déplacement fait déborder le jour d'arrivée, le débordement est signalé et une question est posée à l'utilisateur ; l'outil ne déplace rien d'autre de lui-même.

**Cas nominaux :**
- L'utilisateur glisse le bloc « préparer la démo » de mercredi à jeudi dans son agenda ; la pile affiche l'élément planifié jeudi.

**Edge cases identifiés :**
- Bloc supprimé à la main dans l'agenda : comportement à définir (élément repassé en non planifié ? marqué fait ?) — voir Questions ouvertes.
- Bloc raccourci ou allongé à la main : à trancher (la durée de l'élément change-t-elle, ce qui contredirait « la pile fait foi pour les durées » ?) — voir Questions ouvertes.
- Bloc déplacé sur un week-end ou un jour de congé : accepté (l'agenda fait foi), signalé comme hors capacité.
- Bloc déplacé au-delà de l'échéance de l'élément : accepté et signalé en retard.

### US-13 : Planifier la semaine le lundi et recalculer chaque matin
**En tant qu'** utilisateur solo, **je veux** une planification de ma semaine le lundi puis un recalcul chaque matin, **afin de** commencer chaque journée avec un planning qui tient compte de ce qui a changé.

**Acceptance Criteria :**
- [ ] AC-13.1 : Le lundi, l'outil place les éléments non planifiés de la pile sur les jours de la semaine, dans l'ordre de la pile.
- [ ] AC-13.2 : Chaque matin, l'outil relit l'agenda, met à jour les capacités, prend en compte les blocs déplacés et signale les jours qui débordent.
- [ ] AC-13.3 : Un élément non urgent est placé au premier jour dont la capacité restante suffit, sans déplacer aucun élément déjà planifié.
- [ ] AC-13.4 : Un élément urgent, ou un jour qui déborde, provoque une question à l'utilisateur ; l'outil ne règle jamais seul ces situations.
- [ ] AC-13.5 : Tant que l'utilisateur n'a pas répondu à une question, rien n'est déplacé pour la résoudre, et la question reste visible.
- [ ] AC-13.6 : Une question propose des choix concrets (par exemple : repousser tel élément, accepter le débordement, réduire une durée) et la réponse de l'utilisateur est appliquée immédiatement.
- [ ] AC-13.7 : Le recalcul du matin a lieu même si l'utilisateur n'ouvre pas l'outil, et son résultat est prêt quand il l'ouvre.

**Cas nominaux :**
- Mardi matin, une réunion de 2 h a été ajoutée la veille au soir : mardi déborde d'1 h. L'outil pose la question « mardi déborde d'1 h : repousser X à mercredi, ou accepter ? » ; l'utilisateur choisit, le planning est mis à jour.

**Edge cases identifiés :**
- Élément plus long que la capacité restante de chaque jour de la semaine : il reste non planifié et l'utilisateur en est averti.
- Engagement daté dont l'échéance tombe avant le premier jour disponible : traité comme urgent, question posée.
- Lundi férié ou chômé : la planification hebdomadaire a lieu le premier jour travaillé — voir Hypothèses.
- Ordinateur éteint à l'heure du recalcul : le recalcul a lieu dès le prochain démarrage.
- Plusieurs questions en attente : elles sont toutes visibles, dans un ordre défini (voir Questions ouvertes).

---

### Partie C — Module decision diff (à développer en troisième)

### US-14 : Déclarer les invariants de mon dépôt
**En tant que** mainteneur d'un dépôt de code, **je veux** écrire dans mon dépôt, sous une forme lisible, les règles que mon code doit toujours respecter, **afin que** ces arbitrages soient explicites et partagés avec toute l'équipe, humains et agents.

**Acceptance Criteria :**
- [ ] AC-14.1 : Les invariants sont écrits dans un fichier versionné avec le code du dépôt.
- [ ] AC-14.2 : Un invariant comporte au minimum un nom, une description en langage naturel et la règle vérifiable.
- [ ] AC-14.3 : Un fichier d'invariants mal formé est refusé avec un message qui désigne l'invariant fautif et la raison.
- [ ] AC-14.4 : Un mainteneur non spécialiste comprend ce que vérifie chaque invariant en lisant sa description.

**Cas nominaux :**
- Le mainteneur déclare « le module de facturation n'appelle jamais directement le module d'envoi d'e-mails ».

**Edge cases identifiés :**
- Deux invariants portant le même nom : refusé.
- Invariant qui ne peut concerner aucun fichier du dépôt : signalé comme suspect, sans bloquer.

### US-15 : Vérifier les invariants à chaque modification, sans IA
**En tant que** mainteneur d'un dépôt de code, **je veux** que mes invariants soient vérifiés automatiquement à chaque modification proposée, sans faire appel à un modèle d'IA, **afin d'**obtenir un verdict reproductible, gratuit et fiable.

**Acceptance Criteria :**
- [ ] AC-15.1 : La vérification se lance en local (`arbitre diff`) et dans la chaîne de vérification automatique du dépôt.
- [ ] AC-15.2 : La vérification ne sollicite aucun modèle d'IA et fonctionne sans connexion internet.
- [ ] AC-15.3 : Sur le même code et les mêmes invariants, la vérification donne toujours le même verdict.
- [ ] AC-15.4 : Une modification qui viole un invariant fait échouer la vérification, avec le nom de l'invariant et l'endroit du code concerné.
- [ ] AC-15.5 : Une modification qui respecte tous les invariants fait réussir la vérification.

**Cas nominaux :**
- Un agent propose une modification qui fait appeler l'envoi d'e-mails depuis la facturation ; la vérification échoue et désigne la ligne en cause.

**Edge cases identifiés :**
- Fichier dans un langage non pris en charge : ignoré et listé comme non vérifié, sans faire échouer la vérification.
- Fichier syntaxiquement invalide : signalé comme non vérifiable.
- Dépôt sans fichier d'invariants : la vérification réussit et signale qu'aucun invariant n'est déclaré.

### US-16 : Lire les décisions prises dans une modification plutôt que son code
**En tant que** développeur qui fait travailler des agents de code, **je veux** voir la liste des décisions prises dans une modification proposée, **afin de** relire les arbitrages de l'agent plutôt que chaque ligne de code.

**Acceptance Criteria :**
- [ ] AC-16.1 : Pour une modification proposée, l'outil produit une liste de décisions, chacune résumée en une phrase et reliée aux endroits du code qui la portent.
- [ ] AC-16.2 : Chaque décision indique si elle respecte, viole ou n'est couverte par aucun invariant déclaré.
- [ ] AC-16.3 : Le relecteur peut accepter ou rejeter chaque décision individuellement.
- [ ] AC-16.4 : La liste des décisions peut être affichée sur la page de la modification proposée, dans l'outil d'hébergement du dépôt.

**Cas nominaux :**
- Une modification de 40 fichiers est présentée comme 5 décisions (« ajoute une dépendance à X », « change le format de date de l'export »…) ; le développeur en rejette une et accepte les autres.

**Edge cases identifiés :**
- Modification purement cosmétique (renommage, mise en forme) : aucune décision n'est remontée, et l'outil l'indique.
- Modification trop volumineuse : comportement et seuil à définir (voir Questions ouvertes).

### US-17 : Voir ce qui a changé entre deux itérations d'une modification
**En tant que** développeur qui fait travailler des agents de code, **je veux** voir l'écart de décisions entre deux versions successives d'une même modification proposée, **afin de** ne relire que ce que l'agent a changé depuis ma dernière relecture.

**Acceptance Criteria :**
- [ ] AC-17.1 : Entre deux itérations, l'outil liste les décisions ajoutées, retirées et modifiées.
- [ ] AC-17.2 : Une décision déjà acceptée et inchangée n'est pas présentée à nouveau comme à relire.
- [ ] AC-17.3 : Une décision déjà rejetée qui réapparaît est signalée explicitement.

**Cas nominaux :**
- Après une remarque, l'agent pousse une nouvelle version ; le développeur voit « 1 décision retirée, 1 nouvelle » et ne relit que celles-là.

**Edge cases identifiés :**
- Historique réécrit entre deux itérations : l'écart reste calculé sur le contenu, pas sur l'historique.
- Première itération : toutes les décisions sont nouvelles.

### US-18 : Démarrer avec des invariants proposés, preuves à l'appui
**En tant que** mainteneur d'un dépôt de code, **je veux** qu'on me propose des invariants à partir de mon code existant, avec des preuves et des contre-exemples, **afin de** démarrer sans écrire mes règles de zéro et de n'adopter que celles qui sont vraies.

**Acceptance Criteria :**
- [ ] AC-18.1 : L'outil propose une liste d'invariants candidats pour un dépôt.
- [ ] AC-18.2 : Chaque candidat est accompagné de preuves (endroits du code où il est respecté) et des contre-exemples existants (endroits où il est déjà violé), ou de la mention qu'il n'y en a aucun.
- [ ] AC-18.3 : Aucun candidat n'est ajouté au fichier d'invariants sans validation explicite du mainteneur.
- [ ] AC-18.4 : Un invariant adopté se vérifie ensuite sans IA, comme tout invariant déclaré (US-15).
- [ ] AC-18.5 : Cette proposition fonctionne avec le fournisseur d'IA choisi par l'utilisateur (US-4).

**Cas nominaux :**
- Sur un dépôt existant, l'outil propose « aucun appel réseau depuis le dossier domaine — 120 occurrences conformes, 0 contre-exemple » ; le mainteneur l'adopte.

**Edge cases identifiés :**
- Candidat déjà violé dans le code : proposé avec ses contre-exemples ; le mainteneur choisit de l'adopter (et doit alors corriger) ou de l'écarter.
- Dépôt vide ou minuscule : l'outil indique qu'il n'a pas assez de matière pour proposer.
- Aucun fournisseur d'IA configuré : la proposition est indisponible avec un message ; le reste du module fonctionne.

---

## Règles métier globales
- **RG-1 — Ordre de développement** : socle, puis planning, puis decision diff. Aucune story d'un module ne démarre avant que les stories du socle dont elle dépend soient livrées.
- **RG-2 — Indépendance des modules** : un module ne dépend que du socle, jamais d'un autre module. Aucune donnée ni fonction ne passe d'un module à l'autre.
- **RG-3 — Une seule logique, plusieurs surfaces** : chaque règle métier est définie une fois et se comporte à l'identique sur l'ordinateur, le téléphone, la ligne de commande, l'assistant d'IA et, plus tard, l'application installable.
- **RG-4 — Local-first** : les données appartiennent à l'utilisateur et vivent sur ses appareils ; le relais de synchronisation ne fait que recopier.
- **RG-5 — Indépendance vis-à-vis de l'IA** : aucune fonction n'est liée à un fournisseur d'IA particulier ; toute fonction qui n'exige pas d'IA marche sans IA.
- **RG-6 — L'utilisateur arbitre** : l'outil propose, l'utilisateur décide. Aucune situation de conflit (urgence, débordement, invariant candidat, décision d'agent) n'est tranchée sans lui.
- **RG-7 — Pas de perte silencieuse** : aucune action (synchronisation, recalcul, déplacement, import) ne fait disparaître une donnée sans que l'utilisateur puisse le voir et la retrouver.
- **RG-8 — Garde-fou produit** : aucun module au-delà de planning et decision diff n'est lancé avant 10 entretiens menés avec des design partners.
- **RG-9 — Ouvert dès le départ** : tout ce qui est décrit dans ce cahier est public, sous licence Apache 2.0. Aucun secret ne doit entrer dans l'historique du dépôt public.

## Hors-scope
- Les fonctions Enterprise : auto-hébergement géré, graphe de dépendances entre plusieurs dépôts, export de preuves. Elles vivent hors du dépôt public.
- L'application installable (ordinateur et téléphone) : prévue plus tard, avec la même interface ; ce cahier ne couvre que l'usage par navigateur.
- Tout nouveau module au-delà de planning et decision diff (RG-8).
- Le travail en équipe sur le planning : planning partagé, délégation d'éléments, vue d'un manager. Le planning est personnel ; les collègues ne voient que des créneaux occupés.
- La gestion des réunions elles-mêmes (création, invitation, acceptation) : l'outil lit les réunions, il ne les organise pas.
- Le suivi du temps réellement passé (chronométrage).
- Un service de synchronisation opéré par OpenArbitre pour le compte des utilisateurs.
- La correction automatique du code qui viole un invariant : l'outil signale, il ne corrige pas.
- La relecture du code ligne à ligne par une IA : le module decision diff présente des décisions, il ne remplace pas une revue de code.
- La politique de contribution externe (DCO ou CLA) et le choix du nom de domaine : décisions de gouvernance, pas des fonctions.
- La mise en place du dépôt déjà faite : licence, fichier d'exclusion, détection de secrets avant commit. Seule la protection contre l'envoi de secrets côté hébergeur reste à activer (geste de configuration, pas une story).

## Hypothèses
- [HYPOTHÈSE] L'utilisateur solo a un seul agenda professionnel, et l'agenda visé est celui de son employeur, partagé avec ses collègues.
- [HYPOTHÈSE] Les jours travaillés sont du lundi au vendredi ; samedi et dimanche ont une capacité de 0. Quand le lundi est férié, la planification hebdomadaire a lieu le premier jour travaillé.
- [HYPOTHÈSE] Les 7 h 30 de capacité sont une valeur unique pour tous les jours, non réglable dans ce cycle.
- [HYPOTHÈSE] Un événement « journée entière » ou un congé dans l'agenda ramène la capacité du jour à 0 ; une réunion déclinée n'est pas comptée.
- [HYPOTHÈSE] « Petit geste » = élément de moins de 30 minutes.
- [HYPOTHÈSE] « Élément semblable » pour la mémorisation des corrections de durée = même type d'élément et intitulé proche ; la règle exacte est laissée à la spécification.
- [HYPOTHÈSE] Le module planning fonctionne entièrement sans IA, y compris la proposition de durées ; l'IA peut améliorer ces propositions sans être requise.
- [HYPOTHÈSE] Un élément est urgent si l'utilisateur le marque comme tel, ou si son échéance tombe avant le premier jour où il pourrait être placé sans débordement.
- [HYPOTHÈSE] Les données recopiées sur le relais de synchronisation sont illisibles pour quiconque n'est pas l'utilisateur, y compris l'hébergeur du relais.
- [HYPOTHÈSE] Le module decision diff vise d'abord les dépôts hébergés sur GitHub et le langage Python, d'après les outils d'analyse cités dans le cadrage ; d'autres langages suivent.
- [HYPOTHÈSE] Le plan « Startup decision diff » du 2026-10-02, auquel renvoie le cadrage, n'a pas été relu pour ce cahier (introuvable dans le dépôt) ; les US-14 à US-18 sont tirées du seul résumé du cadrage.
- [HYPOTHÈSE] Le « skill d'onboarding » du cadrage est la fonction de proposition d'invariants (US-18), utilisable depuis l'assistant d'IA de l'utilisateur.

## Questions ouvertes
- **Agenda** : quel(s) fournisseur(s) d'agenda doivent être pris en charge en premier ? Un utilisateur peut-il avoir plusieurs agendas (pro et perso) à cumuler dans le calcul de capacité ?
- **Heures de travail** : une réunion hors des heures de travail (le soir) réduit-elle la capacité du jour ? Faut-il définir une plage horaire de travail pour placer les blocs ?
- **Élément plus long qu'une journée** : un élément de 10 h est-il découpé en plusieurs blocs sur plusieurs jours, ou refusé tant que l'utilisateur ne l'a pas découpé lui-même ?
- **Créneau insuffisant** : si un jour a assez de capacité au total mais aucun créneau continu assez long, l'élément est-il découpé en plusieurs blocs ?
- **Bloc supprimé à la main dans l'agenda** : l'élément repasse-t-il en non planifié, ou est-il considéré comme fait ?
- **Bloc redimensionné à la main** : la nouvelle durée devient-elle la durée de l'élément (et compte-t-elle comme une correction mémorisée) ? Cela entre en tension avec « la pile fait foi pour les durées ».
- **Sujet public par défaut** : quand l'utilisateur n'a pas défini de sujet public, que montre « Focus — <sujet> » ? Un sujet générique, une catégorie, ou l'intitulé (risque de fuite) ?
- **Bloc quotidien des petits gestes** : à quelle heure de la journée se place-t-il ? A-t-il une taille maximale au-delà de laquelle une question est posée ?
- **Questions en attente** : dans quel ordre sont-elles présentées ? Une question non répondue expire-t-elle ?
- **Recalcul** : à quelle heure a lieu le recalcul du matin et la planification du lundi ? Sont-elles réglables ?
- **Conflits de synchronisation** : quelle version est retenue quand deux appareils modifient la même donnée hors ligne (la plus récente ? question à l'utilisateur ?), et quand l'un supprime ce que l'autre modifie ?
- **Import sur une machine non vide** : remplacement, fusion ou refus ?
- **Decision diff — forme des invariants** : quelles familles de règles doivent être exprimables dès la première version (dépendances interdites entre dossiers, fonctions interdites, conventions de nommage…) ?
- **Decision diff — extraction des décisions** : l'extraction des décisions d'une modification (US-16) se fait-elle sans IA, avec IA, ou les deux ? Le cadrage n'exige l'absence d'IA que pour la vérification des invariants.
- **Decision diff — taille** : au-delà de quelle taille une modification est-elle jugée trop volumineuse pour en extraire les décisions, et que fait l'outil alors ?
- **Decision diff — persistance des acceptations** : où vivent les décisions acceptées ou rejetées (dans le dépôt, sur la page de la modification, sur la machine du relecteur) pour que l'écart entre itérations (US-17) soit le même pour tous les relecteurs ?
- **Plan du 2026-10-02** : peut-on joindre le plan « Startup decision diff » au dépôt pour confirmer ou corriger les US-14 à US-18 avant la spécification ?

## Arbitrages de Quentin (2026-11-02)

Réponses aux questions ouvertes qui touchent l'architecture ou la découpe. Elles font foi.

- **Stockage** : SQLite local (rusqlite côté natif). SQLite dans le navigateur (PWA en WASM, OPFS) est à valider par un spike avant d'en dépendre.
- **Conflits de synchronisation** : la dernière écriture gagne, champ par champ, selon une horloge par champ. Une suppression est une écriture datée comme une autre : la plus récente l'emporte, suppression ou modification.
- **Import sur une machine non vide** : fusion, par le même mécanisme que la synchronisation. Pas de mode remplacement.
- **Agendas** : plusieurs agendas en lecture (la capacité les cumule), un seul en écriture pour les blocs. Google Calendar d'abord, CalDAV ensuite, derrière la même frontière d'adaptateur. Le socle ne voit jamais une règle de récurrence, seulement des occurrences datées.
- **Decision diff — extraction** : une base structurelle sans IA (tree-sitter : dépendances, API publique, migrations, modules, config), déterministe et utilisable en CI sans clé. L'IA s'y ajoute en option par l'adaptateur du socle ; ses décisions sont marquées « proposé par IA » et ne bloquent jamais.
- **Decision diff — persistance des acceptations** : dans le dépôt, en fichiers versionnés (par ex. `.arbitre/decisions/`), commités avec la PR.
- **Plan du 2026-10-02** : copie de travail dans `.ctdd/cadre/inputs/plan-decision-diff-2026-10-02.md` (hors dépôt public). Il fait référence pour la partie C. Là où il contredit `CADRAGE.md` (Python, MIT, ordre de build), le cadrage l'emporte. Familles d'invariants v1, reprises de ce plan : frontières d'import, usage des types du domaine, appels interdits.

**Renvoyées au cycle de chaque story** (non tranchées ici, ne pas proposer d'option au niveau global) : élément plus long qu'une journée, créneau insuffisant, bloc supprimé ou redimensionné à la main, sujet public par défaut, bloc des petits gestes, ordre et expiration des questions, heure du recalcul, plage de travail, taille maximale d'une modification pour decision diff.
