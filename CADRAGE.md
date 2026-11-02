# OpenArbitre — cadrage

Décisions du 2026-11-02. Un socle local-first en Rust, deux modules : **planning** puis **decision diff**.

## Principes

- **Indépendant de toute IA.** Les modèles s'y branchent par MCP (standard ouvert) et par une API classique ; un adaptateur unique parle à Claude, GPT, Mistral ou un modèle local.
- **Local-first.** Les données vivent sur la machine de l'utilisateur ; un serveur de synchronisation auto-hébergé ne sert qu'à recopier vers le téléphone.
- **Toute la logique dans le socle**, jamais dans l'interface : PWA, CLI, serveur MCP, app native et serveur de synchro réutilisent la même bibliothèque, pour ne jamais réécrire.

## Architecture

| Brique | Contenu |
|---|---|
| Socle | stockage SQLite local, synchronisation, CLI, serveur MCP, adaptateur d'IA multi-modèles, mécanisme de modules |
| Module planning | pile triée, charge par jour, blocs dans l'agenda |
| Module decision diff | compilateur d'invariants (sans LLM), extraction des décisions d'une PR, delta entre itérations |

Les modules dépendent du socle seul, jamais l'un de l'autre. **Ordre de développement : socle → planning → decision diff.**

## Stack

- **Rust** partout (back, socle, CLI, MCP).
- **Front Rust compilé en WebAssembly avec Leptos**, livré en **PWA** (ordinateur et téléphone, sans store). Tauri plus tard pour une app installable, même interface.
- Analyse de code (decision diff) : tree-sitter et le parseur de Ruff, tous deux en Rust.

## Module planning — besoin

1. Contient les gestes, décisions en attente et engagements datés de l'utilisateur.
2. Charge par jour en heures face à une capacité ; un jour qui déborde se voit.
3. Capacité = 7 h 30 par jour moins les réunions de l'agenda.
4. Deux vues : la pile triée (consultable entière, pour reprioriser) et des blocs dans l'agenda.
5. L'agenda fait foi pour le « quand » (bloc déplacé à la main → la pile suit) ; la pile fait foi pour l'ordre, les durées et le non planifié.
6. Blocs visibles des collègues, titrés « Focus — <sujet> », sans détail sensible.
7. Durées proposées par l'outil, corrigées par l'utilisateur (corrections mémorisées) ; bloc de 30 min minimum, petits gestes regroupés dans un bloc quotidien.
8. Planification le lundi, recalcul chaque matin. Un élément non urgent se place au premier jour libre sans bouger l'existant ; un élément urgent ou un jour qui déborde → question posée à l'utilisateur, jamais réglé seul.

## Module decision diff

Reprend le plan « Startup decision diff » du 2026-10-02 : relire les arbitrages de l'agent plutôt que son code, invariants déclaratifs vérifiés sans LLM en CI, skill d'onboarding qui propose des invariants avec preuves et contre-exemples. Garde-fou : aucun nouveau module avant 10 entretiens de design partners.

## Dépôt

- GitHub, **public dès le premier commit**, licence **Apache 2.0**. Fonctions Enterprise (auto-hébergement, graphe multi-dépôts, export de preuves) hors du dépôt public.
- Détection de secrets dès le premier commit (pre-commit + push protection GitHub).
- Commande courte : `arbitre plan`, `arbitre diff`.

## Ouvert

- DCO ou CLA au premier contributeur externe.
- Domaine (openarbitre.com / .fr / .io sans NS au 2026-11-02, à confirmer à l'achat).
