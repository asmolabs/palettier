//! Storage abstraction traits (ports) defining repository contracts for persistence.
//!
//! These ports follow Hexagonal Architecture / Ports & Adapters: the domain defines
//! the traits it needs, and the infrastructure crate (`palettier-data`) provides
//! the concrete implementations backed by SeaORM and SQLite.

use crate::color::Rgb;
use crate::error::DomainResult;
use crate::paint::Paint;
use crate::palette::{Palette, PaletteMix};
use crate::project::{Project, ProjectPhoto};
use crate::recipe::Recipe;
use async_trait::async_trait;
use indexmap::IndexSet;

/// Persistence port for the paint catalog (both oils and acrylics).
#[async_trait]
pub trait PaintCatalogRepository: Send + Sync {
    /// Retrieves all paints available in the catalog.
    async fn all(&self) -> DomainResult<Vec<Paint>>;

    /// Retrieves only paints marked as currently in stock (owned by the user).
    async fn in_stock(&self) -> DomainResult<Vec<Paint>>;

    /// Finds a paint by its database identifier.
    async fn find_by_id(&self, id: i64) -> DomainResult<Option<Paint>>;

    /// Finds a paint by its natural key: brand and name.
    async fn find_by_natural_key(&self, brand: &str, name: &str) -> DomainResult<Option<Paint>>;

    /// Inserts or updates a paint in the catalog, returning the persisted entity.
    async fn save(&self, paint: &Paint) -> DomainResult<Paint>;

    /// Persists a batch of paints in a single operation.
    async fn save_all(&self, paints: &[Paint]) -> DomainResult<()>;

    /// Toggles the stock/owned status for a single paint tube.
    async fn set_owned(&self, paint_id: i64, owned: bool) -> DomainResult<()>;

    /// Updates ownership for multiple paint tubes in a single transaction.
    async fn set_owned_batch(&self, paint_ids: &[i64], owned: bool) -> DomainResult<()>;

    /// Records a calibrated tint color (1:10 with titanium white) for a paint.
    async fn record_tint(&self, paint_id: i64, tint: Rgb) -> DomainResult<()>;

    /// Records a calibrated pure masstone color for a paint.
    async fn record_masstone(&self, paint_id: i64, measured: Rgb) -> DomainResult<()>;

    /// Counts how many paints are currently owned.
    async fn count_owned(&self) -> DomainResult<u64>;

    /// Counts total paints in the catalog.
    async fn count(&self) -> DomainResult<u64>;

    /// Updates paint metadata (pigments, opacity, drying class) while preserving user calibrations.
    async fn refresh_identity(&self, paint: &Paint) -> DomainResult<()>;

    /// Returns the set of paint IDs referenced by active projects, palettes, or recipes.
    async fn ids_in_use(&self) -> DomainResult<IndexSet<i64>>;

    /// Deletes a custom paint tube from the catalog.
    async fn delete(&self, paint_id: i64) -> DomainResult<()>;
}

/// Persistence port for projects, miniature zones, and applied coats.
///
/// **Contract guarantee**: `save` modifies project metadata, zones, layers, and coats,
/// but never deletes or touches attached photos directly. Photo attachments are managed
/// via [`ProjectRepository::add_photo`] and [`ProjectRepository::remove_photo`].
#[async_trait]
pub trait ProjectRepository: Send + Sync {
    /// Retrieves all projects without binary photo payloads for fast listing.
    async fn all(&self) -> DomainResult<Vec<Project>>;

    /// Finds a project by its identifier, loading complete photo metadata and blobs.
    async fn find_with_photos(&self, id: i64) -> DomainResult<Option<Project>>;

    /// Saves a project and its hierarchical structure (zones, layers, applied coats).
    async fn save(&self, project: &Project) -> DomainResult<Project>;

    /// Attaches a new reference or progress photo to a project.
    async fn add_photo(&self, project_id: i64, photo: &ProjectPhoto) -> DomainResult<Project>;

    /// Removes a photo attachment from a project.
    async fn remove_photo(&self, project_id: i64, photo_id: i64) -> DomainResult<Project>;

    /// Deletes a project and cascades deletion to its zones, layers, coats, and photos.
    async fn delete(&self, project_id: i64) -> DomainResult<()>;
}

/// Persistence port for paint mixtures laid out on the user's active palette.
#[async_trait]
pub trait PaletteMixRepository: Send + Sync {
    /// Retrieves all current palette mixes.
    async fn all(&self) -> DomainResult<Vec<PaletteMix>>;

    /// Saves a new or modified palette mix.
    async fn save(&self, mix: &PaletteMix) -> DomainResult<PaletteMix>;

    /// Deletes a mix from the palette.
    async fn delete(&self, id: i64) -> DomainResult<()>;
}

/// Persistence port for step-by-step painting recipes.
#[async_trait]
pub trait RecipeRepository: Send + Sync {
    /// Retrieves all saved recipes.
    async fn all(&self) -> DomainResult<Vec<Recipe>>;

    /// Checks if a recipe with the given name already exists.
    async fn exists(&self, name: &str) -> DomainResult<bool>;

    /// Saves a recipe and overwrites its steps atomically.
    async fn save(&self, recipe: &Recipe) -> DomainResult<Recipe>;

    /// Deletes a recipe by its identifier.
    async fn delete(&self, id: i64) -> DomainResult<()>;
}

/// Persistence port for saved tube palettes.
#[async_trait]
pub trait PaletteRepository: Send + Sync {
    /// Retrieves all saved palettes.
    async fn all(&self) -> DomainResult<Vec<Palette>>;

    /// Finds a palette by its identifier.
    async fn find_by_id(&self, id: i64) -> DomainResult<Option<Palette>>;

    /// Saves a palette and its member paint references.
    async fn save(&self, palette: &Palette) -> DomainResult<Palette>;

    /// Deletes a palette.
    async fn delete(&self, id: i64) -> DomainResult<()>;
}

/// Persistence port for key-value user settings and studio preferences.
#[async_trait]
pub trait SettingsRepository: Send + Sync {
    /// Retrieves a setting value by key.
    async fn get(&self, key: &str) -> DomainResult<Option<String>>;

    /// Stores or updates a setting key-value pair.
    async fn put(&self, key: &str, value: &str) -> DomainResult<()>;

    /// Removes a setting by key.
    async fn remove(&self, key: &str) -> DomainResult<()>;
}
