use crate::color::Rgb;
use crate::error::DomainResult;
use crate::paint::Paint;
use crate::palette::{Palette, PaletteMix};
use crate::project::{Project, ProjectPhoto};
use crate::recipe::Recipe;
use async_trait::async_trait;
use indexmap::IndexSet;

/// Persistence port for the oil paint catalog.
#[async_trait]
pub trait PaintCatalogRepository: Send + Sync {
    async fn all(&self) -> DomainResult<Vec<Paint>>;
    async fn in_stock(&self) -> DomainResult<Vec<Paint>>;
    async fn find_by_id(&self, id: i64) -> DomainResult<Option<Paint>>;
    async fn find_by_natural_key(&self, brand: &str, name: &str) -> DomainResult<Option<Paint>>;
    async fn save(&self, paint: &Paint) -> DomainResult<Paint>;
    async fn save_all(&self, paints: &[Paint]) -> DomainResult<()>;
    async fn set_owned(&self, paint_id: i64, owned: bool) -> DomainResult<()>;
    async fn record_tint(&self, paint_id: i64, tint: Rgb) -> DomainResult<()>;
    async fn record_masstone(&self, paint_id: i64, measured: Rgb) -> DomainResult<()>;
    async fn count_owned(&self) -> DomainResult<u64>;
    async fn count(&self) -> DomainResult<u64>;
    async fn refresh_identity(&self, paint: &Paint) -> DomainResult<()>;
    async fn ids_in_use(&self) -> DomainResult<IndexSet<i64>>;
    async fn delete(&self, paint_id: i64) -> DomainResult<()>;
}

/// Persistence port for projects (structural protection of photos: `save` never touches photos).
#[async_trait]
pub trait ProjectRepository: Send + Sync {
    async fn all(&self) -> DomainResult<Vec<Project>>;
    async fn find_with_photos(&self, id: i64) -> DomainResult<Option<Project>>;
    async fn save(&self, project: &Project) -> DomainResult<Project>;
    async fn add_photo(&self, project_id: i64, photo: &ProjectPhoto) -> DomainResult<Project>;
    async fn remove_photo(&self, project_id: i64, photo_id: i64) -> DomainResult<Project>;
    async fn delete(&self, project_id: i64) -> DomainResult<()>;
}

/// Persistence port for mixtures laid out on the palette.
#[async_trait]
pub trait PaletteMixRepository: Send + Sync {
    async fn all(&self) -> DomainResult<Vec<PaletteMix>>;
    async fn save(&self, mix: &PaletteMix) -> DomainResult<PaletteMix>;
    async fn delete(&self, id: i64) -> DomainResult<()>;
}

/// Persistence port for recipes.
#[async_trait]
pub trait RecipeRepository: Send + Sync {
    async fn all(&self) -> DomainResult<Vec<Recipe>>;
    async fn exists(&self, name: &str) -> DomainResult<bool>;
    async fn save(&self, recipe: &Recipe) -> DomainResult<Recipe>;
    async fn delete(&self, id: i64) -> DomainResult<()>;
}

/// Persistence port for palettes.
#[async_trait]
pub trait PaletteRepository: Send + Sync {
    async fn all(&self) -> DomainResult<Vec<Palette>>;
    async fn find_by_id(&self, id: i64) -> DomainResult<Option<Palette>>;
    async fn save(&self, palette: &Palette) -> DomainResult<Palette>;
    async fn delete(&self, id: i64) -> DomainResult<()>;
}

/// Persistence port for user settings.
#[async_trait]
pub trait SettingsRepository: Send + Sync {
    async fn get(&self, key: &str) -> DomainResult<Option<String>>;
    async fn put(&self, key: &str, value: &str) -> DomainResult<()>;
    async fn remove(&self, key: &str) -> DomainResult<()>;
}
