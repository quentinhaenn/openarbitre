# Décision ST-01 : dépendances communes

- Date : 2026-11-03 · Story : ST-01 · Sources : `.ctdd/stories/ST-01/03_archi.md` (§ Dépendances communes, arbitrage P-1), amendement A-5 de `.ctdd/stories/ST-01/04_plan.md`, PL-6 du cadre.
- Statut : retenue.

## Règle

Une dépendance commune est déclarée une seule fois, dans `[workspace.dependencies]` du `Cargo.toml` racine, même si aucun composant ne l'utilise encore. Chaque composant la reprend par `x = { workspace = true }` et ajoute ses propres features (elles sont additives). Le workspace fixe la version et coupe les features par défaut, sauf mention contraire.

**Ajouter, retirer ou changer une dépendance commune est un amendement du socle**, hors story. Il en va de même pour la montée de version d'un outil (`scripts/outils.env`, `rust-toolchain.toml`).

## Choix par sujet

| Sujet | Choix | Version | Déclaration au workspace | Raison | Option écartée |
|---|---|---|---|---|---|
| Exécution asynchrone | `tokio` | 1.54.1 | `default-features = false` | Standard de fait ; exigé par rmcp (agent IA) | async-std (arrêté), smol (rmcp exige tokio) |
| Appels réseau sortants | `reqwest` + rustls | 0.13.5 | `default-features = false, features = ["rustls"]` | Client HTTP complet, TLS sans OpenSSL (PL-6). Côté navigateur, `fetch` derrière le port `Transport` | ureq (bloquant), hyper seul (trop bas niveau), native-tls / OpenSSL (exclu par PL-6, contrôlé par `graphe.py`) |
| Trousseau du système | `keyring`, repli fichier `0600` | 4.2.0 | `default-features = false` | Un seul accès aux trousseaux macOS, Windows et Linux | secret-service direct (Linux seul) |
| Sérialisation | `serde` (`derive`) + `serde_json` | 1.0.229 / 1.0.154 | `serde` : `default-features = false, features = ["derive"]` ; `serde_json` : features par défaut (`std`) | Standard de fait | — |
| Tests par propriétés | `proptest` | 1.12.0 | features par défaut (`std` : persistance des échecs, `RngSeed`) ; en `[dev-dependencies]` seulement | Réduction fine des contre-exemples, échecs rejouables | quickcheck (réduction moins fine, pas de persistance) |
| Mesure de mutation | `cargo-mutants` | 27.1.0 | **hors `Cargo.toml`** : outil, épinglé dans `scripts/outils.env` | Seul outil maintenu, mode `--in-diff` | mutagen (abandonné) |
| Dates et fuseaux (ajout) | `jiff` | 0.2.39 | `default-features = false` | Voir « Ajouts » | chrono + chrono-tz |
| Aléa (ajout) | `getrandom` | 0.4.3 | `default-features = false` | Voir « Ajouts » | rand |

Retirés en ST-01 : `chrono` (plus utilisé) et `rusqlite`, qui revient en ST-02 dans `arbitre-sqlite` seul, comme dépendance locale. Restent locales : `clap` (CLI seule) et les crates wasm-bindgen (`arbitre-web` seule).

## Ajouts à PL-6 (arbitrage P-1)

- **`jiff`** servira à `Clock::fuseau` (cœur, ST-04), au planning et à l'agenda. Une seule crate pour dates et fuseaux, qui fonctionne en wasm32 sans `now()` (spike S1-A). chrono + chrono-tz demanderait deux crates, et une base de fuseaux deux fois plus lourde en brut.
- **`getrandom`** servira à l'identifiant d'appareil (ST-02) et aux nonces de l'enveloppe (ST-18). `rand` est inutile pour tirer des octets bruts. Le backend wasm est activé par la crate web.
- Chacune servira à au moins deux composants : les déclarer maintenant évite deux amendements du socle.

## Points d'attention

- **`keyring` 4.2.0 ne compile pas sans une feature** : `default-features = false` seul déclenche `compile_error!` (« At least one of the features 'v1' or 'cli' must be enabled »). La crate qui l'utilise ajoute `features = ["v1"]` (constaté par la vérification ponctuelle de T-1).
- **TLS** : `reqwest` 0.13 avec `rustls` tire `aws-lc-rs`, qui compile du C. Sans effet en wasm (`reqwest` est interdit dans la liste navigateur), acceptable en natif.
- **Réversibilité `aws-lc-rs` → `ring`** : aucune donnée ni aucun format ne dépend du fournisseur TLS. La bascule est une feature au workspace, plus un `rustls::crypto::CryptoProvider::install_default()` au démarrage des binaires natifs. **Piège** : si deux fournisseurs rustls sont compilés, `install_default()` explicite devient obligatoire, sinon rustls panique au premier appel.
- **Compteur saturé reçu d'un pair (Q-4)** : un pair corrompu ou hostile qui envoie `(l futur, c = u32::MAX)` empêche l'appareil d'écrire (`HorlogeEpuisee`) tant que son heure n'a pas dépassé ce `l`. Accepté : au-delà de `σ`, la marque est signalée par `avances`.
- Le `Cargo.lock` ne fait foi que pour les dépendances héritées par un composant (A-5).
