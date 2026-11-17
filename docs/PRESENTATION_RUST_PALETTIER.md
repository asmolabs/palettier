---
marp: true
theme: default
paginate: true
header: "Rust par la pratique — Étude de cas : Palettier"
footer: "Asmolabs • Rust 2024 Edition"
---

# Apprendre Rust par la pratique
## Des fondamentaux aux grandes librairies à travers le projet **Palettier**

* Édition **Rust 2024**
* Architecture Hexagonale multi-crates
* `#![forbid(unsafe_code)]`

---

## Plan de la présentation

1. **Pourquoi Rust sur un tel projet ?**
2. **Les bases de Rust** : Ownership, Borrowing, Types forts
3. **Architecture & Traits** : Workspace Cargo & Ports/Adapters
4. **Erreurs & Sérialisation** : `thiserror`, `anyhow`, `serde`
5. **Concurrence & Parallélisme** : `tokio`, `async-trait`, `rayon`
6. **Données, Sécurité & Média** : `sea-orm`, `secrecy`, `image`
7. **Interfaces & Outillage** : `clap`, `dioxus`, `clippy`, `rustdoc`

---

## 1. Vue d'ensemble de l'architecture Palettier

Un workspace Cargo modulaire avec des responsabilités isolées :

```
palettier-ui (Dioxus)     palettier-cli (Clap)
         │                         │
         ├─────────┬───────────────┤
         ▼         ▼               ▼
palettier-ai   palettier-data   palettier-image
   (Reqwest)      (SeaORM)         (image)
         │         │               │
         └─────────┴───────┬───────┘
                           ▼
                palettier-domain
       (Zéro I/O, Kubelka-Munk, Ports)
```

---

## 2. Les bases : Ownership & Borrowing

* Chaque donnée a un **propriétaire unique** (libération déterministe, pas de GC).
* On **emprunte** par référence (`&T` ou `&mut T`) pour éviter les copies inutiles.

```rust
// crates/palettier-domain/src/paint/tube.rs
pub fn find_closest(
    target: Rgb,
    candidates: &[Paint], // Emprunt d'une tranche (slice) en lecture seule
    limit: usize,
) -> Vec<PaintMatchByColour>
```

> **Règle d'or** : Préférez `&[T]` à `&Vec<T>` et `&str` à `&String` dans vos API.

---

## 3. Les bases : Types forts & Enums exhaustifs

Rendre les états invalides impossibles à compiler (*Type-Driven Development*) :

```rust
// crates/palettier-domain/src/paint/properties.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PaintSystem {
    Oil,
    Acrylic,
}
```

* Le `match` est **exhaustif** : ajouter un nouveau système force le compilateur à signaler chaque endroit à mettre à jour.
* `Rgb` encapsule ses champs pour garantir des valeurs dans `[0.0, 1.0]`.

---

## 4. Pas de `null` ni d'exceptions : `Option` & `Result`

* `Option<T>` : `Some(valeur)` ou `None` (ex. `tint: Option<Rgb>`).
* `Result<T, E>` : `Ok(valeur)` ou `Err(erreur)`.
* Propagation concise avec l'opérateur **`?`** :

```rust
// crates/palettier-domain/src/catalog.rs
let color = Rgb::of_hex(row.get("hex")).map_err(|e| csv_err(&row, e))?;
```

---

## 5. Les 4 visages du mot-clé `impl`

| Syntaxe | Rôle | Exemple dans Palettier |
| :--- | :--- | :--- |
| `impl Rgb { ... }` | Méthodes (`&self`, `&mut self`) & constructeurs | `Rgb::of_hex`, `rgb.to_hex()` |
| `impl Trait for Type` | Implémenter un contrat (`Display`, `FromStr`, port) | `impl Display for PaintSystem` |
| `arg: impl Trait` | Générique statique (ergonomie d'API, zéro coût) | `Database::open(path: impl AsRef<Path>)` |
| `-> impl Trait` | Type de retour concret masqué (vs `dyn Trait`) | Itérateurs, closures vs `Arc<dyn ChatEngine>` |

---

## 6. Architecture Hexagonale via les `trait`

Le domaine définit **ce dont il a besoin** (`port.rs`), sans connaître SQLite :

```rust
// crates/palettier-domain/src/port.rs
#[async_trait]
pub trait PaintCatalogRepository: Send + Sync {
    async fn all(&self) -> DomainResult<Vec<Paint>>;
    async fn find_by_id(&self, id: i64) -> DomainResult<Option<Paint>>;
    async fn save(&self, paint: &Paint) -> DomainResult<Paint>;
}
```

* Production : `SeaOrmPaintCatalogRepository` (SQLite sur disque)
* Tests : `Database::in_memory()` (SQLite en RAM, 100 % isolé)

---

## 6. Erreurs : `thiserror` vs `anyhow`

| Contexte | Crate | Usage dans Palettier |
| :--- | :--- | :--- |
| **Bibliothèques** | `thiserror` | `DomainError` typé (`InvalidHex`, `CsvError`, `Service`, `Persistence`) |
| **Exécutables** | `anyhow` | `palettier-cli` avec `.context("...")` pour l'affichage terminal |

```rust
#[derive(Debug, Error, Clone, PartialEq)]
pub enum DomainError {
    #[error("Code hexadécimal invalide : {0}")]
    InvalidHex(String),
    // ...
}
```

---

## 7. Sérialisation universelle avec `serde`

`serde` génère le code de sérialisation à la compilation (zéro coût de réflexion) :

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AiProvider {
    Ollama,
    OpenAi,
    Gemini,
}
```

Utilisé dans Palettier pour :
* Les sauvegardes ZIP (`serde_json`)
* Les réponses structurées IA (`schemars` + `jsonschema`)

---

## 8. Concurrence (`tokio`) vs Parallélisme (`rayon`)

* **I/O-bound (`tokio` + `async-trait`)** :
  * Requêtes HTTP vers Gemini / OpenAI / Ollama (`reqwest`)
  * Accès SQLite non bloquants (`sea-orm`)
* **CPU-bound (`rayon`)** :
  * Recherche de mélanges Kubelka-Munk sur des centaines de tubes (`mix.rs`) :

```rust
use rayon::prelude::*;

candidates
    .into_par_iter() // Parallélisme multi-cœurs garanti sans data-race !
    .map(|combo| evaluate(combo))
```

---

## 9. Persistance SQL typée avec `sea-orm`

* Entités et relations vérifiées par le compilateur.
* Migrations SQLite automatiques au démarrage (`Migrator::up`).
* Transactions atomiques pour les mises à jour par lot (`set_owned_batch`).

```rust
let conn = sea_orm::Database::connect(opt).await.map_err(map_db_err)?;
conn.execute_unprepared("PRAGMA foreign_keys = ON;").await?;
Migrator::up(&conn, None).await?;
```

---

## 10. Sécurité par conception (`secrecy`, `zeroize`, `image`)

1. **Zéro `unsafe`** : `[workspace.lints.rust] unsafe_code = "forbid"`.
2. **Clés API protégées** (`palettier-ai`) :
   * `SecretString` empêche l'affichage des secrets dans les logs.
   * `zeroize` efface la mémoire vive à la libération.
3. **Anti-OOM sur les images** (`palettier-image`) :
   * `Limits` borne le décodeur à `8192×8192` px et `256 Mo` max.

---

## 11. CLI (`clap`) et UI Desktop (`dioxus`)

* **`clap` (`palettier-cli`)** : `#[derive(Parser)]` + `#[derive(Subcommand)]` génère le parser et le `--help` depuis les types Rust.
* **`dioxus` 0.6 (`palettier-ui`)** :
  * Composants déclaratifs `rsx!` + signaux réactifs (`use_signal`).
  * Séparation stricte : les vues appellent des fonctions pures dans `actions/`.

---

## 12. Qualité, Tests et Documentation

Une chaîne d'outillage unifiée avec `cargo` :

* **Lints stricts** : `unwrap_used = "warn"`, `expect_used = "warn"`.
* **Vérification statique** : `cargo clippy --workspace --all-targets -- -D warnings`.
* **Documentation vivante** : `cargo doc --workspace --no-deps` + **doc-tests** exécutés par `cargo test --workspace`.

---

## 13. Itérateurs & Idiomes clés (Zero-Cost Abstractions)

* **Paresseux (*Lazy*)** : `.iter()`, `.filter()`, `.map()`, `.filter_map()`, `.zip()` ne font rien avant un consommateur (`.collect()`, `.max_by_key()`, `.find()`).

```rust
// crates/palettier-domain/src/workbench.rs
let last = zone.layers.iter()
    .enumerate()
    .filter_map(|(idx, l)| l.applied.as_ref().map(|a| (idx, a)))
    .max_by_key(|(_, a)| a.at);
```

* **Closures `move`** : transfèrent la propriété dans `tokio::spawn` et les callbacks Dioxus.
* **Smart Pointers** : `Arc<dyn Trait>` pour le partage multi-threads, `Box<T>` sur le tas.

---

## Pour aller plus loin

* Lire le cours complet détaillé : `docs/COURS_RUST_PALETTIER.md`
* Explorer l'architecture : `ARCHITECTURE.md`
* Générer la doc interactive du projet :
  ```bash
  cargo doc --workspace --no-deps --open
  ```
