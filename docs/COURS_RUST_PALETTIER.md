# Cours Rust par la pratique : à travers le projet Palettier

Ce cours vous guide des **fondamentaux du langage Rust** jusqu'à l'**utilisation des grandes bibliothèques de l'écosystème** (`serde`, `thiserror`, `tokio`, `rayon`, `sea-orm`, `reqwest`, `clap`, `dioxus`), en s'appuyant exclusivement sur des exemples réels tirés du projet **Palettier**.

---

## Table des matières

1. [Module 1 — Les fondamentaux de Rust dans `palettier-domain`](#module-1--les-fondamentaux-de-rust-dans-palettier-domain)
2. [Module 2 — Architecture modulaire, Traits et Inversion de dépendance](#module-2--architecture-modulaire-traits-et-inversion-de-dépendance)
3. [Module 3 — Gestion des erreurs et Sérialisation (`thiserror`, `anyhow`, `serde`)](#module-3--gestion-des-erreurs-et-sérialisation-thiserror-anyhow-serde)
4. [Module 4 — Asynchronisme et Parallélisme (`tokio`, `async-trait`, `rayon`)](#module-4--asynchronisme-et-parallélisme-tokio-async-trait-rayon)
5. [Module 5 — Persistance, Sécurité et Traitement d'image (`sea-orm`, `secrecy`, `image`)](#module-5--persistance-sécurité-et-traitement-dimage-sea-orm-secrecy-image)
6. [Module 6 — CLI et Interface Graphique (`clap`, `dioxus`)](#module-6--cli-et-interface-graphique-clap-dioxus)
7. [Module 7 — Qualité, Tests et Outils (`clippy`, `rustdoc`, `cargo test`)](#module-7--qualité-tests-et-outils-clippy-rustdoc-cargo-test)
8. [Module 8 — Itérateurs, Closures, Pattern Matching et Pointeurs Intelligents](#module-8--itérateurs-closures-pattern-matching-et-pointeurs-intelligents)

---

## Module 1 — Les fondamentaux de Rust dans `palettier-domain`

### 1.1 Ownership (Propriété) et Borrowing (Emprunt)
En Rust, chaque valeur a un **propriétaire unique**. Quand le propriétaire sort du scope, la mémoire est libérée automatiquement (sans ramasse-miettes). Pour lire ou modifier une donnée sans en prendre possession, on utilise des **références** (`&T` pour un emprunt immuable, `&mut T` pour un emprunt mutable).

Dans [`crates/palettier-domain/src/paint/tube.rs`](../crates/palettier-domain/src/paint/tube.rs), la fonction `find_closest` emprunte la tranche de peintures `&[Paint]` plutôt que de consommer un `Vec<Paint>` :

```rust
pub fn find_closest(target: Rgb, candidates: &[Paint], limit: usize) -> Vec<PaintMatchByColour> {
    // `candidates` est emprunté en lecture seule : l'appelant garde sa liste intacte
    let mut matches: Vec<PaintMatchByColour> = candidates
        .iter()
        .map(|p| PaintMatchByColour {
            paint: p.clone(),
            delta_e: delta_e_2000(target, p.color()),
        })
        .collect();
    matches.sort_by(|a, b| a.delta_e.total_cmp(&b.delta_e));
    matches.truncate(limit);
    matches
}
```

> **Bonne pratique** : Préférez `&[T]` à `&Vec<T>`, et `&str` à `&String` dans les paramètres de fonctions. Cela rend l'API plus souple et évite des allocations inutiles.

### 1.2 Modélisation par les types : `struct` et `enum`
Rust encourage le **Type-Driven Development** : rendre les états invalides impossibles à représenter.

* **`struct` avec invariants** : Dans [`crates/palettier-domain/src/color.rs`](../crates/palettier-domain/src/color.rs), `Rgb` garde ses champs `r`, `g`, `b` privés pour garantir qu'ils sont toujours bornés entre `0.0` et `1.0` via `Rgb::new(...)`.
* **`enum` exhaustifs** : Le compilateur vérifie que chaque `match` traite tous les cas possibles. Exemple avec `PaintSystem` dans [`crates/palettier-domain/src/paint/properties.rs`](../crates/palettier-domain/src/paint/properties.rs) :

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PaintSystem {
    Oil,
    Acrylic,
}
```

Contrairement à une chaîne `"oil"` / `"acrylic"`, aucune faute de frappe n'est possible à la compilation.

### 1.3 Pas de `null` ni d'exceptions : `Option<T>` et `Result<T, E>`
* `Option<T>` représente une valeur qui peut être absente (`Some(val)` ou `None`), par exemple `tint: Option<Rgb>` sur un pigment.
* `Result<T, E>` représente une opération qui peut échouer (`Ok(val)` ou `Err(err)`).
* L'opérateur **`?`** propage automatiquement l'erreur à l'appelant si la valeur est `Err(...)`, comme dans `PigmentIndex::try_new` ([`crates/palettier-domain/src/catalog.rs`](../crates/palettier-domain/src/catalog.rs)) :

```rust
let color = Rgb::of_hex(row.get("hex")).map_err(|e| csv_err(&row, e))?;
```

---

## Module 2 — Architecture modulaire, Traits, `impl` et Inversion de dépendance

### 2.1 Workspace Cargo multi-crates
Un workspace Cargo permet de découper une application en bibliothèques indépendantes avec des frontières de compilation strictes (`Cargo.toml` racine) :
* `palettier-domain` : logique métier pure (aucune dépendance vers SQLite ou Dioxus).
* `palettier-data` : implémentation de la persistance (dépend de `palettier-domain`).
* `palettier-image` : décodage sécurisé d'images.
* `palettier-ai` : clients LLM multimodaux.
* `palettier-ui` & `palettier-cli` : interfaces finales.

### 2.2 Maîtriser le mot-clé `impl` sous toutes ses formes

En Rust, le mot-clé `impl` apparaît dans **quatre contextes distincts** qu'il est essentiel de bien différencier.

#### A. Le bloc inhérent : `impl MonType { ... }`
Contrairement aux langages orientés objet classiques où les méthodes sont écrites dans le bloc `class`, Rust sépare les **données** (`struct` / `enum`) du **comportement** (`impl`).

Un bloc `impl MonType` peut contenir :
1. **Des constantes associées** : rattachées au type lui-même (ex. `Rgb::REFERENCE_WHITE` ou `PaintSystem::ALL`).
2. **Des fonctions associées (constructeurs)** : elles ne prennent pas `self` et s'appellent avec `::` (ex. `Rgb::new(...)`, `Rgb::of_hex(...)`).
3. **Des méthodes d'instance** : elles prennent un récepteur qui exprime l'usage mémoire et les droits d'accès :
   * `&self` : lecture seule (emprunt immuable), ex. `rgb.to_hex()` ou `system.label(locale)`.
   * `&mut self` : modification sur place (emprunt mutable), ex. modifier un état interne.
   * `self` : **consommation** de la valeur (transfert d'ownership), utile pour les types `Copy` comme `DryingClass::slowest(self, other)` ou les *builders*.

```rust
// crates/palettier-domain/src/color.rs
impl Rgb {
    pub const REFERENCE_WHITE: Rgb = Rgb { /* ... */ }; // Constante associée

    pub fn new(r: f64, g: f64, b: f64) -> Self { /* ... */ } // Constructeur (fonction associée)

    pub fn to_hex(&self) -> String { /* ... */ } // Méthode en lecture seule (&self)
}
```

> **Astuce** : Un même type peut avoir **plusieurs blocs `impl`**, même répartis dans différents fichiers d'un même crate (ou conditionnés par `#[cfg(test)]`), ce qui facilite l'organisation du code.

#### B. L'implémentation de contrat : `impl MonTrait for MonType { ... }`
C'est ainsi qu'on rattache un comportement standard ou métier à un type existant.

* **Traits de la bibliothèque standard** (`Display`, `FromStr`, `Default`, `From`) :
  Dans [`crates/palettier-domain/src/paint/properties.rs`](../crates/palettier-domain/src/paint/properties.rs), `PaintSystem` implémente `FromStr` (pour parser une chaîne venant de la base ou d'un CSV via `.parse()`) et `fmt::Display` (pour l'affichage).

```rust
impl fmt::Display for PaintSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label(Locale::Fr))
    }
}
```

* **Traits métier (Ports & Adapters)** :
  Dans `palettier-data`, `SeaOrmPaintCatalogRepository` implémente le trait `PaintCatalogRepository` défini dans `palettier-domain` :

```rust
#[async_trait]
impl PaintCatalogRepository for SeaOrmPaintCatalogRepository {
    async fn all(&self) -> DomainResult<Vec<Paint>> { /* requête SeaORM */ }
    // ...
}
```

> **Règle de cohérence (Orphan Rule)** : On ne peut écrire `impl Trait for Type` que si **au moins l'un des deux** (`Trait` ou `Type`) appartient au crate courant. On ne peut donc pas implémenter `Display` pour `Vec<T>` dans `palettier-domain`, car les deux viennent de la `std`.

#### C. `impl Trait` en paramètre de fonction (Dispatch Statique)
Utiliser `impl Trait` dans la signature d'une fonction est un sucre syntaxique pour un paramètre générique `<T: Trait>`. Le compilateur génère une version spécialisée pour chaque type concret utilisé (**monomorphisation**, zéro coût à l'exécution).

Palettier s'en sert pour offrir des API ergonomiques qui acceptent plusieurs types sans forcer de conversion préalable chez l'appelant :

```rust
// crates/palettier-data/src/db.rs
// Accepte &str, String, PathBuf, &Path... tout ce qui se voit comme un Path !
pub async fn open(path: impl AsRef<Path>) -> DomainResult<Self> {
    let path = path.as_ref();
    // ...
}

// crates/palettier-domain/src/miniature/batch.rs
// Accepte &str (sans allouer si inutile) ou String (sans re-cloner)
pub fn new(name: impl Into<String>, miniature_count: usize, steps: Vec<String>) -> Self {
    Self { name: name.into(), /* ... */ }
}
```

#### D. `impl Trait` en type de retour vs `dyn Trait`
* **`-> impl Trait` (Dispatch statique)** : la fonction retourne **un seul type concret** (souvent complexe ou anonyme, comme un itérateur enchaîné ou une *closure*), mais masque ce type à l'appelant. Aucune allocation sur le tas (`Box`) n'est nécessaire.
* **`dyn Trait` (Dispatch dynamique)** : indispensable quand une collection ou une structure doit contenir **plusieurs implémentations différentes** interchangeables à l'exécution (ex. choisir entre `GeminiEngine`, `OpenAiEngine` ou `OllamaEngine` selon les réglages de l'utilisateur). On l'utilise alors derrière un pointeur intelligent (`Box<dyn Trait>` ou `Arc<dyn Trait>`).

```rust
// 1. -> impl Trait : un seul type concret masqué (ici un itérateur filtré), zéro allocation
fn oil_paints<'a>(paints: &'a [Paint]) -> impl Iterator<Item = &'a Paint> {
    paints.iter().filter(|p| p.system == PaintSystem::Oil)
}

// 2. dyn Trait : plusieurs types concrets possibles choisis à l'exécution (Gemini, OpenAI, Ollama)
fn build_engine(provider: AiProvider) -> Arc<dyn ChatEngine> {
    match provider {
        AiProvider::Gemini => Arc::new(GeminiEngine::new(/* ... */)),
        AiProvider::OpenAi => Arc::new(OpenAiEngine::new(/* ... */)),
        AiProvider::Ollama => Arc::new(OllamaEngine::new(/* ... */)),
    }
}
```

> **En résumé** : Si une fonction peut retourner **plusieurs types différents** selon un `if` ou un `match`, `-> impl Trait` ne compilera pas (car les types des branches diffèrent) : il faut utiliser `Box<dyn Trait>` ou `Arc<dyn Trait>`. Si elle retourne toujours le **même type** (comme une chaîne `.iter().filter()`), `-> impl Trait` est idéal.

### 2.3 Les `trait` comme contrats (Architecture Hexagonale)
Un `trait` définit un comportement partagé (équivalent d'une interface). Dans [`crates/palettier-domain/src/port.rs`](../crates/palettier-domain/src/port.rs), le domaine déclare les contrats de persistance sans savoir comment ils seront stockés :

```rust
#[async_trait]
pub trait PaintCatalogRepository: Send + Sync {
    async fn all(&self) -> DomainResult<Vec<Paint>>;
    async fn find_by_id(&self, id: i64) -> DomainResult<Option<Paint>>;
    async fn save(&self, paint: &Paint) -> DomainResult<Paint>;
    // ...
}
```

L'interface utilisateur et les tests manipulent ces traits via `Arc<dyn PaintCatalogRepository>`, ce qui permet d'utiliser SQLite sur disque en production et une base SQLite en mémoire (`Database::in_memory()`) dans les tests.

---

## Module 3 — Gestion des erreurs et Sérialisation (`thiserror`, `anyhow`, `serde`)

### 3.1 `thiserror` (pour les bibliothèques) vs `anyhow` (pour les exécutables)
* **`thiserror`** génère automatiquement l'implémentation de `std::error::Error` et `Display` sur un `enum` typé. Dans [`crates/palettier-domain/src/error.rs`](../crates/palettier-domain/src/error.rs) :

```rust
#[derive(Debug, Error, Clone, PartialEq)]
pub enum DomainError {
    #[error("Code hexadécimal invalide : {0}")]
    InvalidHex(String),
    #[error("Argument invalide : {0}")]
    InvalidArgument(String),
    #[error("Erreur de format CSV dans {location} : {message}")]
    CsvError { location: String, message: String },
    #[error("Erreur de service externe : {0}")]
    Service(String),
    #[error("Erreur de persistance : {0}")]
    Persistence(String),
}
```

* **`anyhow`** est utilisé dans [`crates/palettier-cli/src/main.rs`](../crates/palettier-cli/src/main.rs) où l'on souhaite simplement remonter n'importe quelle erreur avec du contexte (`.context("...")`) jusqu'à `main()`.

### 3.2 `serde` et `serde_json`
`serde` est la référence absolue pour sérialiser/désérialiser des structures Rust (JSON, ZIP backup, requêtes API). Grâce aux macros `#[derive(Serialize, Deserialize)]` et aux attributs comme `#[serde(rename_all = "SCREAMING_SNAKE_CASE")]` (dans `AiProvider`), le code généré est rapide et sans réflexion à l'exécution.

---

## Module 4 — Asynchronisme et Parallélisme (`tokio`, `async-trait`, `rayon`)

En Rust, on distingue nettement **concurrence asynchrone (I/O)** et **parallélisme de données (CPU)**.

### 4.1 `tokio` et `async-trait` pour les entrées/sorties
* **`tokio`** fournit le moteur d'exécution asynchrone pour la base SQLite (`sea-orm`) et les requêtes réseau HTTP (`reqwest`).
* **`async-trait`** permet d'écrire des méthodes `async fn` dans des traits utilisés en dispatch dynamique (`dyn Trait`), comme `ChatEngine` ou `ProjectRepository`.

### 4.2 `rayon` pour le calcul intensif (CPU-bound)
Dans [`crates/palettier-domain/src/mix.rs`](../crates/palettier-domain/src/mix.rs), le solveur colorimétrique Kubelka-Munk doit tester des milliers de combinaisons de tubes et de proportions. En remplaçant simplement `.into_iter()` par `.into_par_iter()` grâce à `use rayon::prelude::*;`, `rayon` répartit automatiquement le calcul sur tous les cœurs du processeur grâce à du *work-stealing*, tout en garantissant l'absence de *data races* à la compilation.

---

## Module 5 — Persistance, Sécurité et Traitement d'image

### 5.1 `sea-orm` et `sea-orm-migration`
`palettier-data` utilise **SeaORM** pour interagir avec SQLite de manière typée et sécurisée :
* Les migrations (`Migrator::up`) s'exécutent au démarrage dans [`crates/palettier-data/src/db.rs`](../crates/palettier-data/src/db.rs).
* Les mises à jour groupées (comme `set_owned_batch`) utilisent des transactions SQLite atomiques.

### 5.2 `secrecy` et `zeroize`
Pour les clés d'API IA (OpenAI, Gemini) dans [`crates/palettier-ai/src/settings.rs`](../crates/palettier-ai/src/settings.rs) :
* `SecretString` empêche toute fuite accidentelle dans les logs (`Debug` affiche `[REDACTED]`).
* `zeroize` efface physiquement les octets sensibles en RAM lorsque la valeur est libérée.

### 5.3 `image` avec limites de ressources
Dans [`crates/palettier-image/src/lib.rs`](../crates/palettier-image/src/lib.rs), `ImageDecoder` configure `image::Limits` (`MAX_IMAGE_DIMENSION = 8192`, `MAX_ALLOC_BYTES = 256 Mo`) avant tout décodage, protégeant l'application contre les bombes de décompression.

---

## Module 6 — CLI et Interface Graphique (`clap`, `dioxus`)

### 6.1 `clap` (Command Line Argument Parser)
Dans [`crates/palettier-cli/src/main.rs`](../crates/palettier-cli/src/main.rs), `#[derive(Parser)]` et `#[derive(Subcommand)]` transforment directement un `enum Commands` en interface en ligne de commande complète avec aide générée et validation des arguments.

### 6.2 `dioxus` 0.6 Desktop
Dans `palettier-ui`, l'interface desktop native utilise **Dioxus** :
* Syntaxe déclarative `rsx! { ... }` vérifiée à la compilation.
* État réactif géré par des signaux (`use_signal`).
* **Bonne pratique appliquée** : la logique métier est extraite hors des vues dans `crates/palettier-ui/src/actions/`, sous forme de fonctions `async` pures testables sans fenêtre graphique.

---

## Module 7 — Qualité, Tests et Outils (`clippy`, `rustdoc`, `cargo test`)

Le projet applique une discipline stricte dans `Cargo.toml` :
```toml
[workspace.lints.rust]
unsafe_code = "forbid"

[workspace.lints.clippy]
unwrap_used = "warn"
expect_used = "warn"
dbg_macro = "warn"
todo = "warn"
```

* **`cargo clippy --workspace --all-targets -- -D warnings`** : analyse statique traitant tout avertissement comme une erreur bloquante.
* **`cargo doc --workspace --no-deps`** : génère la documentation HTML à partir des commentaires `//!` et `///`.
* **`cargo test --workspace`** : exécute à la fois les tests unitaires, les tests d'intégration SQLite en mémoire, les tests de l'UI `VirtualDom` et les **doc-tests** intégrés dans la documentation.

---

## Module 8 — Itérateurs, Closures, Pattern Matching et Pointeurs Intelligents

### 8.1 Les Itérateurs (`Iterator`) : puissance et coût zéro (*Zero-Cost Abstractions*)

En Rust, les boucles fonctionnelles avec itérateurs sont **aussi rapides (et souvent plus rapides)** que des boucles `for` impératives avec index, car le compilateur élimine les vérifications de bornes (*bounds checks*) et fusionne les étapes en une seule passe mémoire.

#### A. Les 3 façons d'itérer sur une collection
| Méthode | Élément produit | Ce qu'il advient de la collection |
| :--- | :--- | :--- |
| `.iter()` | `&T` (référence immuable) | Conservée intacte, lecture seule |
| `.iter_mut()` | `&mut T` (référence mutable) | Conservée, éléments modifiables sur place |
| `.into_iter()` | `T` (propriété transférée) | **Consommée** (déplacée) |

#### B. Paresse (*Lazy Evaluation*) : Adaptateurs vs Consommateurs
Un itérateur Rust ne fait **rien** tant qu'un **consommateur** ne le déclenche pas :
* **Adaptateurs** (paresseux, retournent un nouvel itérateur) : `.map()`, `.filter()`, `.filter_map()`, `.enumerate()`, `.zip()`, `.flat_map()`, `.take()`.
* **Consommateurs** (exécutent la chaîne et produisent un résultat) : `.collect()`, `.find()`, `.any()`, `.all()`, `.count()`, `.sum()`, `.max_by_key()`.

Exemples réels dans Palettier :

```rust
// 1. Chaînage de parsing dans crates/palettier-domain/src/catalog.rs
cell.split([' ', '\t'])
    .map(str::trim)
    .filter(|s| !s.is_empty())
    .map(ToString::to_string)
    .collect() // Déclenche l'exécution et construit un IndexSet<String>

// 2. Trouver la couche posée la plus récente dans crates/palettier-domain/src/workbench.rs
let last = zone
    .layers
    .iter()
    .enumerate() // Produit (index, &ProjectLayer)
    .filter_map(|(idx, l)| l.applied.as_ref().map(|a| (idx, a))) // Filtre + extrait en 1 étape
    .max_by_key(|(_, a)| a.at); // Consomme et retourne Option<(usize, &AppliedCoat)>

// 3. Parcourir deux tableaux en parallèle dans crates/palettier-domain/src/mix.rs
for (i, (paint, &part)) in paints.iter().zip(parts.iter()).enumerate() {
    weights[i] = (part as f64) * paint.tinting;
}
```

> **Passage au multi-cœurs en 1 ligne** : Grâce au trait `ParallelIterator` de `rayon`, il suffit de remplacer `.into_iter()` par `.into_par_iter()` (comme dans `mix.rs`) pour paralléliser toute la chaîne d'itérateurs sur tous les cœurs du CPU !

---

### 8.2 Les Closures (`|x| ...`) et le mot-clé `move`

Une closure est une fonction anonyme capable de capturer des variables de son environnement :
* Par défaut, elle capture par **emprunt** (`&T` ou `&mut T`).
* Avec **`move`**, elle **prend possession** des variables capturées. C'est indispensable dès qu'une closure survit à la fonction qui l'a créée (ex. tâches `tokio::spawn` ou callbacks Dioxus `onclick: move |_| { ... }`).

```rust
// Exemple typique en UI Dioxus : `move` transfère la copie/le Handle du signal dans le callback
onclick: move |_| {
    spawn(async move {
        // Code asynchrone exécuté plus tard au clic
    });
}
```

---

### 8.3 Pattern Matching avancé

Le `match` en Rust ne se limite pas à des constantes : il permet de **déstructurer** des tuples, des `struct` et des `enum` tout en garantissant l'exhaustivité.

```rust
// crates/palettier-domain/src/error.rs
// Match simultané sur un tuple (Locale, &DomainError)
match (locale, self) {
    (Locale::Fr, _) => self.to_string(),
    (Locale::En, Self::InvalidHex(hex)) => format!("Invalid hexadecimal code: {hex}"),
    (Locale::En, Self::CsvError { location, message }) => {
        format!("CSV format error in {location}: {message}")
    }
    // ...
}
```

* **`if let`** : pratique quand on ne s'intéresse qu'à un seul cas (ex. `if let Some(parent) = path.parent() { ... }` dans `db.rs`).
* **`let ... else`** : permet des retours anticipés (*early returns*) très lisibles lorsqu'une valeur doit correspondre à un motif, sinon on sort immédiatement.

---

### 8.4 Durées de vie (*Lifetimes* `'a`)

Le compilateur garantit qu'aucune référence ne survit à la donnée qu'elle pointe (*dangling pointer* impossible). Dans 95 % des cas, Rust devine les lifetimes automatiquement (*élision*). On ne les écrit explicitement que lorsqu'il y a ambiguïté sur la provenance d'une référence retournée :

```rust
// crates/palettier-domain/src/catalog.rs
// La chaîne retournée vit aussi longtemps que `self` ET `fallback`
pub fn get_or_default<'a>(&'a self, column: &str, fallback: &'a str) -> &'a str {
    self.values.get(column).map(String::as_str).unwrap_or(fallback)
}
```

---

### 8.5 Pointeurs intelligents (*Smart Pointers*) : `Box`, `Arc` et `Mutex`

* **`Box<T>`** : alloue une valeur sur le tas (*heap*) avec un propriétaire unique (utile pour les types récursifs ou `Box<dyn Error>`).
* **`Arc<T>` (*Atomic Reference Counting*)** : permet à **plusieurs propriétaires** de partager une même ressource en lecture seule entre plusieurs threads ou tâches asynchrones (ex. `Arc<dyn PaintCatalogRepository>` partagé dans `AppServices`).
* **`Mutex<T>`** : permet la mutabilité partagée sécurisée entre threads (un seul thread accède à `T` à la fois), comme `Mutex<HashMap<String, String>>` dans les tests de `palettier-ai`.

