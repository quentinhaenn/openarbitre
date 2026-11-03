# Cahier des Charges — ST-01 : Fusion pure (horloge, écritures, versions écartées) et fondations du dépôt

## Contexte
OpenArbitre fait vivre les mêmes données sur plusieurs appareils de l'utilisateur, qui peuvent les modifier chacun hors ligne. Avant tout stockage, toute synchronisation et toute surface, il faut une règle unique, prouvée, qui combine les modifications de plusieurs appareils en un même état partout, sans jamais perdre une valeur en silence (US-2 / AC-2.6 du cadre, RG-G7). Cette story livre ce moteur de convergence seul, sans stockage. Comme c'est la première story du projet, elle pose aussi les fondations que toutes les autres réutiliseront : un dépôt assaini, une chaîne de vérification automatique, une mesure de la robustesse des tests et le choix des dépendances communes.

## Utilisateurs cibles
- **Utilisateur solo** (bénéficiaire indirect) : ne voit rien de cette story directement. Il en tirera la garantie que ses appareils finissent d'accord entre eux et qu'aucune modification ne disparaît sans trace (affichage livré par des stories ultérieures).
- **Mainteneur du projet** (Quentin) : arbitre les choix de cette story, relit les vérifications automatiques et s'appuie sur elles pour accepter une modification.
- **Agents de développement** (acteurs non humains) : mènent les cycles des stories suivantes, souvent en parallèle, et doivent trouver des règles de vérification et des dépendances déjà fixées, sans avoir à les rediscuter.

## User Stories

### US-1 : Converger vers le même état, quel que soit l'ordre d'arrivée
**En tant qu'** utilisateur solo, **je veux** que deux appareils qui ont reçu les mêmes modifications aient exactement le même état, quel que soit l'ordre dans lequel elles leur sont parvenues, **afin de** ne jamais avoir deux versions différentes de mes données selon l'appareil que je regarde.

**Acceptance Criteria :**
- [ ] AC-1.1 : Pour un même champ modifié plusieurs fois, la valeur retenue est celle de la modification la plus récente selon l'horloge du projet (US-4) ; en cas d'égalité parfaite des marques de temps, le départage se fait par l'identité de l'appareil auteur, et donne le même gagnant sur tous les appareils (RG-2.1, INV-4).
- [ ] AC-1.2 : Deux modifications portant sur des champs différents d'une même donnée sont toutes deux conservées (matrice US-2, cas 4).
- [ ] AC-1.3 : Combiner A puis B donne le même résultat que combiner B puis A (commutativité, INV-5).
- [ ] AC-1.4 : Combiner A et (B et C) donne le même résultat que combiner (A et B) et C (associativité, INV-5).
- [ ] AC-1.5 : Recevoir une seconde fois une modification déjà reçue ne change rien : ni l'état, ni les versions écartées, ni les conflits signalés (idempotence, INV-5).
- [ ] AC-1.6 : Les propriétés AC-1.3 à AC-1.5 sont vérifiées automatiquement sur des ensembles de modifications générés au hasard en grand nombre (au moins plusieurs centaines de jeux par propriété), et pas seulement sur des exemples écrits à la main.
- [ ] AC-1.7 : Le moteur ne lit et n'écrit aucun stockage, ne consulte aucune horloge système et ne contacte aucun réseau : à entrées égales, il rend toujours le même résultat.

**Cas nominaux :**
- L'ordinateur modifie le titre d'une donnée, le téléphone modifie son échéance, tous deux hors ligne. Une fois leurs modifications échangées, les deux appareils affichent le nouveau titre et la nouvelle échéance.

**Edge cases identifiés :**
- Mêmes modifications reçues dans des ordres différents par deux appareils : états identiques (matrice US-2, cas 8).
- Marques de temps strictement égales venant de deux appareils différents : même gagnant partout, désigné par l'identité d'appareil (matrice US-2, cas 7).
- Ensemble de modifications vide : l'état est inchangé, rien n'est écarté ni signalé.
- Très grand nombre de modifications sur le même champ : une seule valeur retenue, toutes les autres écartées et récupérables (US-2).

### US-2 : Ne jamais perdre une valeur en silence
**En tant qu'** utilisateur solo, **je veux** que toute valeur remplacée lors de la combinaison des modifications reste récupérable, et que seuls les vrais conflits me soient signalés, **afin de** retrouver une version que j'ai perdue sans être dérangé à chaque modification ordinaire.

**Acceptance Criteria :**
- [ ] AC-2.1 : Toute valeur qui perd lors d'une combinaison figure dans la liste des versions écartées rendue par le moteur, avec son champ, sa valeur, sa marque de temps, son appareil et son auteur (RG-2.4, INV-7).
- [ ] AC-2.2 : Un **conflit** est un champ modifié des deux côtés depuis la dernière synchronisation entre ces deux appareils, c'est-à-dire deux modifications dont aucune n'a été faite en connaissance de l'autre. Chaque conflit est signalé avec le champ, la valeur retenue et la valeur écartée (arbitrage CONFLIT-4/5).
- [ ] AC-2.3 : Un **écrasement séquentiel** (une modification faite après avoir reçu la précédente) n'est pas signalé comme conflit ; la valeur remplacée va tout de même aux versions écartées (arbitrage CONFLIT-5).
- [ ] AC-2.4 : Pour déterminer « depuis la dernière synchronisation », le moteur reçoit en entrée, pour chaque appareil pair, le point jusqu'où ses modifications ont déjà été reçues ; il ne compare pas des vecteurs de versions (arbitrage CONFLIT-4/5).
- [ ] AC-2.5 : Rejouer les mêmes entrées signale exactement les mêmes conflits et écarte exactement les mêmes valeurs, dans le même ordre (rapports triés).

**Cas nominaux :**
- Le même champ est modifié hors ligne sur l'ordinateur puis, plus tard, sur le téléphone. À la reconnexion, la valeur du téléphone est retenue, celle de l'ordinateur est écartée et récupérable, et un conflit est signalé (matrice US-2, cas 3).
- L'utilisateur modifie un champ sur l'ordinateur, le téléphone reçoit la modification, puis l'utilisateur la corrige depuis le téléphone. Aucun conflit n'est signalé ; l'ancienne valeur est dans les versions écartées.

**Edge cases identifiés :**
- Trois appareils : A et B modifient le même champ sans s'être vus, C reçoit les deux. Le conflit est signalé et le résultat est le même sur les trois appareils.
- Modification concurrente qui arrive à la même valeur des deux côtés : voir Questions ouvertes.
- Point de dernière réception absent pour un appareil pair jamais synchronisé : toute modification de ce pair sur un champ aussi modifié localement est traitée comme concurrente `[HYPOTHÈSE]`.

### US-3 : Traiter une suppression comme une modification datée
**En tant qu'** utilisateur solo, **je veux** qu'une suppression faite sur un appareil et une modification faite sur un autre soient départagées par leur date, comme deux modifications ordinaires, **afin de** ne jamais perdre sans trace ni ce que j'ai supprimé ni ce que j'ai modifié.

**Acceptance Criteria :**
- [ ] AC-3.1 : Une suppression porte une marque de temps comme toute modification (RG-2.3, arbitrage « Conflits »).
- [ ] AC-3.2 : Une donnée est supprimée si et seulement si sa suppression est plus récente que la plus récente des modifications de ses champs (INV-6, résolution par défaut de CONFLIT-3).
- [ ] AC-3.3 : Supprimée sur A, puis modifiée plus tard sur B : la donnée est présente après combinaison, et la suppression reste récupérable (matrice US-2, cas 5).
- [ ] AC-3.4 : Modifiée sur A, puis supprimée plus tard sur B : la donnée est supprimée après combinaison, et la version modifiée reste récupérable (matrice US-2, cas 6).
- [ ] AC-3.5 : Les propriétés de convergence de US-1 (AC-1.3 à AC-1.6) restent vraies quand les ensembles générés contiennent des suppressions.

**Cas nominaux :**
- L'utilisateur supprime une donnée sur son ordinateur ; plus tard, sans le savoir, il la modifie sur son téléphone hors ligne. À la reconnexion, elle réapparaît partout, et la suppression reste visible dans ce qui a été écarté.

**Edge cases identifiés :**
- Suppression et modification de champ de marques de temps strictement égales : départage par l'identité d'appareil, même résultat partout.
- Donnée supprimée deux fois, sur deux appareils : supprimée une seule fois ; signalement : voir Questions ouvertes.
- Donnée supprimée puis « ressuscitée » par une modification plus récente : elle réapparaît avec toutes les valeurs de champs les plus récentes connues. La remise en place de ses effets dérivés (bloc d'agenda) est hors scope.

### US-4 : Dater les modifications dans un ordre fidèle à ce que l'utilisateur a vu
**En tant qu'** utilisateur solo, **je veux** qu'une modification faite après avoir vu une autre soit toujours considérée comme plus récente qu'elle, même si l'horloge de mon appareil retarde, **afin que** ma dernière correction gagne vraiment.

**Acceptance Criteria :**
- [ ] AC-4.1 : Les modifications sont datées par une horloge qui combine l'heure de l'appareil, un compteur et l'identité de l'appareil (horloge logique hybride, arbitrage CONFLIT-4/5). Deux marques de temps sont toujours comparables, et l'ordre obtenu est le même sur tous les appareils.
- [ ] AC-4.2 : Une modification faite sur un appareil après qu'il a reçu une autre modification porte une marque strictement supérieure à celle-ci, même si l'heure de cet appareil est en retard sur celle de l'appareil d'origine (CONFLIT-4).
- [ ] AC-4.3 : Deux modifications successives sur un même appareil portent des marques strictement croissantes, même si elles tombent au même instant ou si l'heure de l'appareil recule.
- [ ] AC-4.4 : L'heure de l'appareil est fournie au moteur par une source remplaçable : on peut la simuler (avancer, reculer, figer) pour vérifier AC-4.2 et AC-4.3 sans attendre.
- [ ] AC-4.5 : Une modification porte, outre sa marque de temps, la donnée et le champ visés, la valeur, l'appareil auteur et l'auteur (utilisateur par la ligne de commande ou l'interface, assistant, système).

**Cas nominaux :**
- L'ordinateur, dont l'heure avance de 5 minutes, modifie une échéance. Le téléphone reçoit cette modification puis l'utilisateur la corrige depuis le téléphone, une minute plus tard en temps réel. La correction du téléphone gagne.

**Edge cases identifiés :**
- Heure de l'appareil qui recule (changement d'heure manuel, resynchronisation) : les marques continuent de croître.
- Grand nombre de modifications au même instant sur un appareil : ordre strict conservé.
- Modification reçue d'un appareil dont l'heure est très en avance : voir Questions ouvertes.

### US-5 : Partir d'un dépôt sain
**En tant que** mainteneur, **je veux** que le cœur du projet ne contienne ni accès au stockage ni lecture de l'horloge système, et qu'il puisse s'exécuter dans le navigateur comme sur l'ordinateur, **afin que** la même logique serve plus tard au téléphone sans réécriture (RG-G3).

**Acceptance Criteria :**
- [ ] AC-5.1 : Le cœur ne dépend plus d'aucune bibliothèque de stockage ni de lecture de l'heure système (anti-patterns « stockage natif dans le cœur » et « horloge système implicite » de l'archi).
- [ ] AC-5.2 : Le cœur se construit pour la cible navigateur ; la vérification est automatique (US-6).
- [ ] AC-5.3 : Les versions de dépendances ne sont plus dupliquées entre les composants du projet : chaque dépendance commune est déclarée une fois, au niveau du projet (US-8).
- [ ] AC-5.4 : Le code de démonstration laissé par la création du projet est remplacé.

**Cas nominaux :**
- Un agent ouvre la story suivante : le cœur se construit pour l'ordinateur et pour le navigateur, sans avertissement de dépendance interdite.

**Edge cases identifiés :**
- La ligne de commande a encore besoin de l'heure système : elle peut la garder, puisque la règle ne vise que le cœur et le futur module planning.

### US-6 : Faire vérifier chaque modification proposée automatiquement
**En tant que** mainteneur, **je veux** que chaque modification proposée au dépôt soit vérifiée automatiquement sur des règles fixées une fois pour toutes, **afin de** ne plus relire à la main ce qu'une machine peut garantir, et que les voies parallèles ne divergent pas.

**Acceptance Criteria :**
- [ ] AC-6.1 : Sur chaque modification proposée, les tests du projet sont exécutés ; un test qui échoue rend la vérification rouge.
- [ ] AC-6.2 : La construction pour la cible navigateur est vérifiée sur une liste nommée de composants (le cœur et le futur module planning). Un composant de la liste qui n'existe pas encore est ignoré, sans échec.
- [ ] AC-6.3 : Les dépendances interdites entre composants sont vérifiées, avec des règles écrites par nom pour **tous** les composants prévus par l'architecture, y compris ceux qui n'existent pas encore. Au minimum : le planning ne dépend jamais de decision diff ni l'inverse (RG-G2) ; le cœur et le planning ne dépendent de rien qui ne se construise pas pour le navigateur.
- [ ] AC-6.4 : Introduire volontairement une dépendance interdite dans une modification de test rend la vérification rouge, avec un message qui nomme les deux composants.
- [ ] AC-6.5 : Les secrets sont recherchés dans chaque modification proposée ; un secret détecté rend la vérification rouge (RG-G9).
- [ ] AC-6.6 : Les stories suivantes ajoutent leurs propres vérifications dans de nouveaux fichiers, sans modifier les règles posées ici (règle de collision « Configuration CI » du plan).

**Cas nominaux :**
- Un agent propose une modification qui ajoute une bibliothèque réseau au cœur. La vérification échoue sur la règle « construction pour le navigateur » ou « dépendance interdite », avant toute relecture humaine.

**Edge cases identifiés :**
- Modification qui ne touche que de la documentation : les vérifications tournent et passent.
- Composant prévu par l'architecture qui naît dans une story ultérieure : ses règles d'arêtes s'appliquent dès sa naissance, sans retoucher la vérification.

### US-7 : Mesurer si les tests attrapent vraiment les erreurs
**En tant que** mainteneur, **je veux** qu'une mesure automatique dise, pour les fichiers modifiés, quelle part des erreurs introduites volontairement dans le code est détectée par les tests (score de mutation), et la compare au seuil de leur zone, **afin de** savoir si les tests protègent vraiment les règles du cœur.

**Acceptance Criteria :**
- [ ] AC-7.1 : Sur chaque modification proposée, la mesure porte sur les seules parties du code modifiées par cette modification.
- [ ] AC-7.2 : Le seuil dépend de la zone du fichier : 85 % en zone Full, 70 % en zone standard. Une story déclare la zone de ses fichiers dans `.ctdd/stories/ST-xx/zones.toml`, par motifs de chemins.
- [ ] AC-7.3 : Un fichier qui ne figure dans aucune déclaration de zone prend le seuil Full (85 %) (F-36).
- [ ] AC-7.4 : Un score sous le seuil de sa zone est signalé, avec le fichier, son score et le seuil attendu.
- [ ] AC-7.5 : La mesure tourne sur la modification proposée de ST-01 elle-même et rend au moins 85 % sur le moteur de combinaison (US-1 à US-3) et sur l'horloge (US-4).
- [ ] AC-7.6 : La relecture de chaque story ultérieure peut lire ce résultat pour vérifier le seuil de sa zone, sans outillage supplémentaire.

**Cas nominaux :**
- Une story modifie un fichier du cœur sans tests suffisants : la mesure rend 60 % pour ce fichier, sous le seuil de 85 %, et le signale.

**Edge cases identifiés :**
- Modification proposée sans code modifié : aucune mesure à faire, résultat neutre.
- Aucun fichier `zones.toml` pour la story : tous les fichiers prennent le seuil Full.
- Fichier couvert par deux motifs de zones différentes : voir Questions ouvertes.

### US-8 : Choisir une fois les dépendances communes, et les consigner
**En tant que** mainteneur, **je veux** que les dépendances partagées par plusieurs composants soient choisies et déclarées une seule fois dès cette story, avec leur justification écrite, **afin que** les voies parallèles ne se contredisent pas et qu'une nouvelle dépendance commune soit une décision explicite (amendement du socle).

**Acceptance Criteria :**
- [ ] AC-8.1 : Un document de décision `docs/decisions/ST-01-dependances.md` existe. Il couvre au moins : l'exécution asynchrone, les appels réseau sortants, l'accès au trousseau du système, la sérialisation, les tests par propriétés et la mesure de mutation. Pour chacune : le choix, la raison et l'option écartée.
- [ ] AC-8.2 : Les choix retenus par défaut au plan (PL-6) sont repris, sauf justification écrite contraire dans le document.
- [ ] AC-8.3 : Toutes les dépendances communes sont déclarées au niveau du projet, y compris celles qu'aucun composant n'utilise encore (PL-6).
- [ ] AC-8.4 : Le guide de contribution écrit la règle de résolution des conflits sur le fichier de verrouillage des versions : ne jamais le fusionner à la main ; reprendre celui de la branche principale, puis le régénérer (règle de collision du plan).

**Cas nominaux :**
- Un agent de la voie planning a besoin d'appels réseau : la dépendance est déjà déclarée au niveau du projet ; il l'utilise sans ouvrir de décision.

**Edge cases identifiés :**
- Une story ultérieure a besoin d'une dépendance commune absente du document : ce n'est pas réglé dans la story, c'est un amendement du socle (procédure du plan).

## Règles métier globales
- **Pas de perte silencieuse** (RG-G7) : aucune combinaison ne fait disparaître une valeur sans qu'elle figure dans les versions écartées.
- **Une seule logique** (RG-G3) : le moteur de combinaison est le même pour la synchronisation et, plus tard, l'import (arbitrage « Import sur une machine non vide »).
- **Déterminisme** : à entrées égales, mêmes sorties, rapports triés ; aucune dépendance à l'ordre d'arrivée, au fuseau ou à l'heure de la machine qui exécute.
- **Le moteur est aveugle au métier** : il ne connaît ni éléments, ni pile, ni agenda. Il combine des valeurs de champs sans les interpréter.
- **Ouvert dès le départ** (RG-G9) : aucun secret dans le dépôt ; la vérification automatique le contrôle.
- **Zone Full par défaut** : ce qui n'est pas déclaré est vérifié au niveau le plus exigeant.

## Hors-scope
- Le stockage des modifications, de l'état courant, des versions écartées et des points de dernière réception (ST-02). Ici, « récupérable » veut dire « rendu par le moteur » ; la conservation durable vient avec le stockage.
- La forme de l'accès au stockage, notamment son caractère asynchrone (ST-02, PL-8).
- L'export et l'import (ST-04b), même s'ils réutiliseront ce moteur.
- La synchronisation, le relais, le chiffrement et le secret de l'utilisateur (ST-18, ST-19).
- L'affichage à l'utilisateur du résultat retenu, des conflits et des versions écartées (stories de surfaces).
- La revalidation métier d'une donnée après combinaison, quand la combinaison champ par champ produit une donnée invalide (CONFLIT-1 : crochet en ST-04, marquage en ST-05, question en ST-13).
- L'ordre de la pile après combinaison de deux réordonnancements concurrents (CONFLIT-2, ST-06).
- La remise en place des effets dérivés d'une donnée « ressuscitée », comme un bloc d'agenda (RG-10.1, stories de blocs).
- La compaction du journal et la signature des modifications par appareil (plafond assumé de l'archi ; arbitrage MANQUE-6).
- Les gestes sans cycle : retrait du dépôt imbriqué (G-0), protection contre l'envoi de secrets chez l'hébergeur (G-1), rangement des cycles par story et horodatages de mesure (G-4, condition F-40).
- Les vérifications propres aux voies suivantes (interface web, action decision diff) : elles s'ajoutent dans leurs propres fichiers.

## Hypothèses
- [HYPOTHÈSE] La résolution par défaut de CONFLIT-3 est retenue : la suppression se compare à la plus récente des modifications des champs de la donnée. Les arbitrages des specs ne la tranchent pas explicitement ; ils disent que la résolution proposée vaut par défaut et se revoit au cycle de la story. C'est ce cycle.
- [HYPOTHÈSE] Une suppression et une modification de champ concurrentes (aucune n'a vu l'autre) forment un conflit signalé, au même titre que deux modifications du même champ : c'est ce qui rend AC-2.6 du cadre tenable pour les cas 5 et 6 de la matrice US-2.
- [HYPOTHÈSE] Un appareil pair dont on n'a encore rien reçu a un point de dernière réception « vide » : tout ce qu'il envoie est postérieur à ce point.
- [HYPOTHÈSE] La vérification automatique tourne chez l'hébergeur public du dépôt, sur chaque modification proposée, et la protection contre l'envoi de secrets (G-1) est active avant le premier envoi.
- [HYPOTHÈSE] « Plusieurs centaines de jeux par propriété » (AC-1.6) est un plancher pour l'exécution courante ; un nombre plus élevé peut être fixé pour la zone Full par la spécification (exigence CI > 0,95 de l'archi).
- [HYPOTHÈSE] Le score de 85 % d'AC-7.5 se mesure fichier par fichier sur le moteur de combinaison et sur l'horloge, pas en moyenne sur toute la modification proposée.

## Questions ouvertes
- **Dérive d'horloge vers le futur** : une modification qui arrive d'un appareil dont l'heure est très en avance (des heures, des jours) est-elle acceptée telle quelle, ce qui fait gagner cet appareil sur tout ce qui suit jusqu'à ce que l'heure réelle le rattrape, ou faut-il une limite au-delà de laquelle elle est signalée ou refusée ? Si limite, laquelle ?
- **Conflit de même valeur** : deux modifications concurrentes du même champ qui arrivent à la même valeur doivent-elles être signalées comme conflit, ou seulement rangées dans les versions écartées ?
- **Double suppression** : deux suppressions concurrentes de la même donnée sont-elles signalées, alors que rien n'est perdu pour l'utilisateur ?
- **Zones qui se recouvrent** : si un fichier correspond à deux motifs de zones différentes dans `zones.toml`, quel seuil s'applique (le plus exigeant, le premier motif, refus du fichier de zones) ?
- **Score sous le seuil** : un score de mutation sous le seuil bloque-t-il la fusion de la modification proposée, ou est-il seulement signalé pour la relecture de la story ?

## Arbitrages de Quentin (2026-11-03)

Ils répondent aux questions ouvertes et font foi.

- **Horloge en avance** : une modification datée dans le futur est acceptée, sans perte. Au-delà d'une avance d'1 h sur l'heure locale (seuil réglable), l'outil signale l'appareil concerné et son avance.
- **Même valeur** : deux modifications concurrentes du même champ qui donnent la même valeur ne sont pas un conflit. L'écriture non retenue va aux versions écartées, sans signalement.
- **Double suppression** : même règle. Deux suppressions concurrentes ne sont pas un conflit.
- **Zones qui se recouvrent** : le seuil le plus exigeant s'applique, quel que soit l'ordre des motifs dans `zones.toml`.
- **Score de mutation sous le seuil** : bloquant, la PR ne fusionne pas. Seule exception, explicite : un mutant équivalent listé avec sa justification.
- **CI** : sur GitHub dès ST-01, avec un dépôt public sous le compte `quentinhaenn`, la protection contre l'envoi de secrets activée (G-1) et GitHub Actions. Les vérifications vivent dans un seul script (`scripts/ci.sh`), que la CI appelle et qu'on peut lancer en local.
