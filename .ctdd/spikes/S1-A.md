# Spike S1-A : port `Store` asynchrone et worker de stockage côté wasm

- **Début** : 2026-11-03T00:32:44Z
- **Fin** : 2026-11-03T00:38:14Z (rédaction comprise, environ 10 min de plus)
- **Périmètre** : critères (1), (6) et (7) de S1 (`03_archi.md`, § Spikes). Les critères (2) à (5) relèvent de S1-B.
- **Code d'essai** (non fusionnable) : `/Users/quentin-pro/Perso/openarbitre/.claude/worktrees/agent-ac5860c27fb69a1ae/spikes/s1a/`
  - `port/` : crate `s1a-port`, qui joue le rôle de `arbitre-core::store`. Elle n'a aucune dépendance et contient le trait, une fonction cœur `scenario<S: Store>`, un adaptateur mémoire et un test natif.
  - `web/` : crate `s1a-web` (cdylib), avec l'adaptateur `WorkerStore` et `src/worker.js`, un worker dédié qui tient le journal dans IndexedDB.
  - `build.sh` construit les variantes `base`, `jiff` et `chrono` et mesure leur poids. `run-headless.sh` les charge dans Chrome headless.

## Verdict : **GO avec réserves**

Le port asynchrone se branche sur un worker de stockage. Le même code cœur, générique sur `Store`, tourne sans modification en natif et dans le navigateur. Il fait l'aller-retour thread principal → worker → IndexedDB → thread principal. Le cœur ne dépend d'aucune crate, ni native ni web.

Les réserves :

1. **Pas de borne `Send` sur le port.** Les futures côté wasm ne sont pas `Send` (preuve plus bas).
2. **Le HLC ne doit pas traverser `postMessage` comme un `Number`.** Au-delà de 2^53, la valeur perd de la précision. Il faut un `BigInt` ou une chaîne.
3. **L'`arbitre-core` actuel ne compile pas en wasm32** à cause de `rusqlite`. Ce retrait est déjà prévu en ST-01.

### Forme recommandée du port pour ST-02

```rust
#[allow(async_fn_in_trait)]          // voulu : on ne promet pas Send
pub trait Store {
    async fn appliquer(&self, lot: &[Ecriture]) -> Result<Seq, ErreurStore>;   // atomique
    async fn ecritures_depuis(&self, filigrane: Seq) -> Result<Vec<(Seq, Ecriture)>, ErreurStore>;
    async fn appareil(&self) -> Result<IdAppareil, ErreurStore>;
    // + etat / parcourir / journal / filigrane / avancer_filigrane, sur le même modèle
}
pub enum ErreurStore { StockagePlein, Corrompu(String), Indisponible(String) }
```

- **`async fn` natif dans le trait**, stable depuis Rust 1.75. On n'utilise ni `async-trait` ni `Box<dyn Future>`.
- **`?Send` de fait.** Le trait n'impose aucune borne `Send`. Un adaptateur natif peut rendre des futures `Send`, mais le code générique ne doit pas l'exiger. Si la CLI ou le MCP ont besoin de `Send` (tokio multi-thread), on ajoute côté natif seulement une variante `SendStore` avec `trait-variant`, ou on exécute sur un runtime `current_thread`. Le cœur ne change pas.
- **`&self` partout.** L'adaptateur gère sa mutabilité interne (`RefCell` en wasm, connexion ou `Mutex` en natif). Avec `&mut self`, l'UI ne pourrait pas lancer deux requêtes en parallèle sur le worker.
- **Côté natif, des futures immédiates.** Un seul `poll` suffit et aucun exécuteur n'est requis (test `scenario_natif_futures_immediates`, avec `Waker::noop`).
- **Erreurs.** Le worker répond `{ok}` ou `{err: name, msg}`. L'adaptateur traduit `QuotaExceededError` en `StockagePlein` et toute autre erreur en `Indisponible`. Une réponse illisible devient `Corrompu`. La traduction reste dans l'adaptateur : le cœur ne voit jamais un `JsValue`.
- **Transport.** Chaque appel crée son propre `MessageChannel` et transfère `port2` au worker, qui répond sur ce port. Il n'y a donc ni table de corrélation ni identifiant de requête, et plusieurs appels peuvent être en vol en même temps.

## Critères

| Critère | Statut | Preuve |
|---|---|---|
| (1) Compile en `wasm32-unknown-unknown`, port sans dépendance native | **Prouvé** pour un port et un adaptateur jumeau. **Échec** pour l'`arbitre-core` actuel, comme attendu | voir 1a à 1c |
| (6) Architecture UI ↔ worker de stockage acceptable | **Prouvé** dans Chrome 156 headless, avec IndexedDB dans le worker. **Plausible** pour OPFS et SQLite-wasm dans le même worker (même transport, seul le moteur change, à faire en S1-B) | voir 6a et 6b |
| (7) Poids du bundle, fuseaux compris | **Prouvé, mesuré** | voir 7 |

### 1a. Le port n'a aucune dépendance en wasm32

```
$ cargo tree -p s1a-port --target wasm32-unknown-unknown -e normal
s1a-port v0.0.0 (.../spikes/s1a/port)
$ cargo build -p s1a-port --target wasm32-unknown-unknown
✓ cargo build (1 crates compiled)
$ cargo test -p s1a-port          # natif
✓ cargo test: 1 passed
```

### 1b. L'adaptateur web : son graphe ne contient que des crates wasm-bindgen

```
$ cargo tree -p s1a-web --target wasm32-unknown-unknown -e normal --depth 2
s1a-web
├── js-sys v0.3.107 (cfg-if, futures-util, wasm-bindgen)
├── s1a-port
├── wasm-bindgen v0.2.130
├── wasm-bindgen-futures v0.4.80
└── web-sys v0.3.107
$ cargo build -p s1a-web --target wasm32-unknown-unknown --release
Finished `release` profile [optimized] target(s) in 8.79s
```

### 1c. Contre-exemple : l'`arbitre-core` actuel

```
$ cargo tree -p arbitre-core --target wasm32-unknown-unknown -e normal --depth 2
arbitre-core ├── chrono v0.4.45 └── rusqlite v0.41.1 ... └── sqlite-wasm-rs v0.6.2
$ cargo build -p arbitre-core --target wasm32-unknown-unknown
warning: sqlite-wasm-rs@0.6.2: error: unable to create target: 'No available targets are compatible with triple "wasm32-unknown-unknown"'
error: failed to run custom build command for `sqlite-wasm-rs v0.6.2`
```

On en tire deux choses. D'abord, cela confirme le retrait de `rusqlite` du cœur prévu en ST-01. Ensuite, c'est une information pour S1-B : `rusqlite` 0.41 passe déjà par `sqlite-wasm-rs` en wasm32, mais la compilation demande un clang qui gère la cible wasm, ce que l'Apple clang n'est pas.

### 6a. Aller-retour dans le navigateur

Le serveur statique tourne avec `python3 -m http.server 8731`, puis `sh run-headless.sh` lance Chrome headless. La page renvoie son résultat par un `GET /done?...`, relevé dans le journal d'accès du serveur :

```
Google Chrome 156.0.8078.80
GET /done?pkg=base&r=RESULT OK appareil=worker-idb seqs=2,3 journal=3 | 32.0 ms | crossOriginIsolated=false
GET /done?pkg=jiff&r=RESULT OK appareil=worker-idb seqs=2,3 journal=3 | 39.1 ms | crossOriginIsolated=false | TZ jiff 2023-11-14T23:13:20+01:00[Europe/Paris]
GET /done?pkg=chrono&r=RESULT OK appareil=worker-idb seqs=2,3 journal=3 | 24.1 ms | crossOriginIsolated=false | TZ chrono-tz 2023-11-14 23:13:20 CET
```

`scenario` est le code cœur. Il applique un lot de 2 écritures, puis un lot de 1, et relit le journal depuis 0 puis depuis le premier filigrane. Il vérifie le contenu et lève `Corrompu` en cas d'écart. Les 5 appels au port prennent 24 à 39 ms au total, création du worker et ouverture d'IndexedDB comprises. La page n'a pas besoin de `crossOriginIsolated` : sans `SharedArrayBuffer`, il n'y a pas d'en-têtes COOP/COEP à poser.

### 6b. Preuve que la contrainte `?Send` est nécessaire

```
$ cargo build -p s1a-web --target wasm32-unknown-unknown --features send-probe
error: future cannot be sent between threads safely
   future returned by `appareil` is not `Send`
note: ... await occurs here on type `JsFuture`, which is not `Send`
```

### 7. Poids du bundle (release, `opt-level="z"`, LTO, `codegen-units=1`, sans wasm-opt)

| Variante | wasm brut | wasm gzip -9 | colle JS |
|---|---|---|---|
| base (port + adaptateur worker) | 134 879 o | 35 139 o | 22 917 o |
| + `jiff` 0.2.39, `tzdb-bundle-always` | 592 860 o | 169 735 o | 23 248 o |
| + `chrono` 0.4.45 et `chrono-tz` 0.10.4 | 1 034 543 o | 156 327 o | 23 247 o |

La base complète des fuseaux coûte environ **+120 à +135 Ko gzip**, soit +460 Ko (jiff) à +900 Ko (chrono-tz) en brut. Une PWA peut l'absorber. Pour réduire, deux pistes : ne garder que quelques fuseaux (`chrono-tz` avec `filter-by-regex`), ou lire la base de fuseaux de l'OS via `Intl`, ce qui casse la promesse « base embarquée, wasm compris » de `Clock::fuseau`. Le choix revient à ST-01 ou au port `Clock`. Pour l'arbitrer, il faut savoir que jiff fonctionne en wasm32 sans `now()` (la feature `js` n'est pas nécessaire tant que l'heure vient de `Clock`). Les deux crates ont rendu la même heure pour Paris. Le `worker.js` (env. 1,5 Ko) n'est pas compté dans le tableau.

## Crates retenues (versions exactes)

- `wasm-bindgen` 0.2.130, avec `wasm-bindgen-cli` 0.2.130, installé par `cargo install --locked` (les deux versions doivent être identiques)
- `wasm-bindgen-futures` 0.4.80
- `js-sys` 0.3.107
- `web-sys` 0.3.107, features `Worker`, `Blob`, `BlobPropertyBag`, `Url`, `MessageChannel`, `MessagePort`, `MessageEvent`
- Fuseaux, pour la mesure seulement : `jiff` 0.2.39 (`std`, `tzdb-bundle-always`) ; `chrono` 0.4.45 (`alloc`) et `chrono-tz` 0.10.4
- Toolchain : rustc et cargo 1.99.1, cible `wasm32-unknown-unknown`
- Aucune dépendance dans le port.

## Pièges rencontrés

- **`--dump-dom` de Chrome headless** prend la capture au chargement, avant la fin des tâches asynchrones. Ni `--virtual-time-budget` ni `--timeout` n'y changent rien, la page reste « pending ». Le contournement retenu : la page envoie une balise `GET /done?…`, relevée dans le journal du serveur. Ce montage évite chromedriver et wasm-bindgen-test. Pour la CI, il faudra un vrai harnais (`wasm-bindgen-test` avec chromedriver, ou un script CDP).
- **u64 vers JS** : `postMessage` sérialise un `Number` (f64). Un HLC au-delà de 2^53 serait tronqué sans erreur. Il faut le passer en `BigInt` (`js_sys::BigInt`) ou en chaîne.
- **Le worker est servi depuis une Blob URL** (`include_str!` du JS) : pas de fichier à déployer à part, et le chemin ne casse pas. Un vrai worker Rust (le cœur dans le worker, ou sqlite-wasm) aura besoin de son propre point d'entrée wasm et d'un bundler, par exemple Trunk avec `data-type="worker"`. Ce montage n'est pas testé ici.
- **`rusqlite` en wasm32** passe par `sqlite-wasm-rs` et compile du C : l'Apple clang ne sait pas produire du wasm32, il faut un LLVM ou clang upstream.
- **Clippy** demande `#[allow(async_fn_in_trait)]`. Le lint avertit justement qu'on ne promet pas `Send`, ce qui est voulu ici.

## Reste pour S1-B

- Le moteur réel dans le worker : SQLite-wasm et OPFS (`createSyncAccessHandle`, réservé aux workers), ou IndexedDB en repli. Il faut trouver la chaîne de compilation C vers wasm32 pour `sqlite-wasm-rs`.
- Les critères (2) rechargement et fermeture d'onglet, (3) fermeture brutale après confirmation, (4) quota (vérifier en conditions réelles la traduction `QuotaExceededError` → `StockagePlein`) et (5) PWA installée sur Safari iOS et Chrome Android.
- L'atomicité de `appliquer` côté navigateur. Elle n'est pas testée ici, même si l'adaptateur passe déjà par une seule transaction IndexedDB par lot.
- Un harnais de test navigateur pour la CI, et le passage du HLC en `BigInt`.
- La mesure du bundle avec wasm-opt et avec le vrai `arbitre-core` (serde, AEAD).
