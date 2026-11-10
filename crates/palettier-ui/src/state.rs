//! Shared reactive state, dependency container, theme mode, and navigation sections for the desktop UI.

use dioxus::prelude::*;
use palettier_ai::{
    AiSettings, AiSettingsStore, ChatEngines, ModelCatalog, PaintingPlanService, SampledZone,
    TubeRecognitionService,
};
use palettier_data::{
    BackupExporter, BackupImporter, CatalogSeeder, Database, SeaOrmPaintCatalogRepository,
    SeaOrmPaletteMixRepository, SeaOrmPaletteRepository, SeaOrmProjectRepository,
    SeaOrmRecipeRepository, SeaOrmSettingsRepository,
};
use palettier_domain::drying::{DryingTimeService, Workshop};
use palettier_domain::error::DomainResult;
use palettier_domain::guide::MixingGuideBuilder;
use palettier_domain::i18n::Locale;
use palettier_domain::mix::ColorMixService;
use palettier_domain::paint::Paint;
use palettier_domain::palette::{Palette, PaletteMix, PaletteMixService};
use palettier_domain::plan::{ProjectPlanner, SampledPlanService};
use palettier_domain::port::{
    PaintCatalogRepository, PaletteMixRepository, PaletteRepository, ProjectRepository,
    RecipeRepository, SettingsRepository,
};
use palettier_domain::project::{
    FatOverLeanService, ProgressCheckService, Project, SubstituteService,
};
use palettier_domain::recipe::{Recipe, RecipeTimelineService};
use palettier_domain::workbench::WorkbenchService;
use palettier_image::ImageDecoder;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

/// Visual theme mode (Night / Dark studio vs Day / Light studio).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeMode {
    #[default]
    Dark,
    Light,
}

impl ThemeMode {
    pub fn css_class(&self) -> &'static str {
        match self {
            Self::Dark => "theme-dark",
            Self::Light => "theme-light",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Dark => "dark",
            Self::Light => "light",
        }
    }

    pub fn toggle(&self) -> Self {
        match self {
            Self::Dark => Self::Light,
            Self::Light => Self::Dark,
        }
    }

    pub fn button_label(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::Dark, Locale::Fr) => "☀️ Mode Jour",
            (Self::Dark, Locale::En) => "☀️ Day Mode",
            (Self::Light, Locale::Fr) => "🌙 Mode Nuit",
            (Self::Light, Locale::En) => "🌙 Night Mode",
        }
    }

    pub fn status_label(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::Dark, Locale::Fr) => "Nuit",
            (Self::Dark, Locale::En) => "Night",
            (Self::Light, Locale::Fr) => "Jour",
            (Self::Light, Locale::En) => "Day",
        }
    }
}

impl FromStr for ThemeMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "light" | "day" | "jour" => Ok(Self::Light),
            "dark" | "night" | "nuit" => Ok(Self::Dark),
            _ => Err(()),
        }
    }
}

/// Top-level navigation sections in the sidebar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    Today,
    Projects,
    Palettes,
    Miniature,
    Eyedropper,
    Assistant,
    Mixer,
    Catalog,
    Drying,
    Recipes,
    Settings,
}

impl Section {
    pub const FIGURINE: [Self; 1] = [Self::Miniature];
    pub const WORKSHOP: [Self; 3] = [Self::Today, Self::Projects, Self::Palettes];
    pub const COLOR: [Self; 3] = [Self::Mixer, Self::Eyedropper, Self::Assistant];
    pub const REFERENCE: [Self; 3] = [Self::Catalog, Self::Drying, Self::Recipes];

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Miniature => "⚔️",
            Self::Today => "⏱️",
            Self::Projects => "📋",
            Self::Palettes => "🎨",
            Self::Mixer => "🧪",
            Self::Eyedropper => "💧",
            Self::Assistant => "🤖",
            Self::Catalog => "📚",
            Self::Drying => "⏳",
            Self::Recipes => "📖",
            Self::Settings => "⚙️",
        }
    }

    pub fn label_i18n(&self, locale: Locale) -> &'static str {
        match self {
            Self::Miniature => locale.tr("Figurine & Weathering", "Miniatures & Weathering"),
            Self::Today => locale.tr("Aujourd'hui", "Today"),
            Self::Projects => locale.tr("Projets", "Projects"),
            Self::Palettes => locale.tr("Palettes", "Palettes"),
            Self::Mixer => locale.tr("Mélangeur", "Mixer"),
            Self::Eyedropper => locale.tr("Pipette", "Eyedropper"),
            Self::Assistant => locale.tr("Assistant IA", "AI Assistant"),
            Self::Catalog => locale.tr("Catalogue", "Catalog"),
            Self::Drying => locale.tr("Séchage", "Drying"),
            Self::Recipes => locale.tr("Recettes", "Recipes"),
            Self::Settings => locale.tr("Paramètres", "Settings"),
        }
    }

    pub fn shortcut(&self) -> &'static str {
        match self {
            Self::Miniature => "⌘M",
            Self::Today => "⌘1",
            Self::Projects => "⌘2",
            Self::Palettes => "⌘3",
            Self::Mixer => "⌘4",
            Self::Eyedropper => "⌘5",
            Self::Assistant => "⌘6",
            Self::Catalog => "⌘7",
            Self::Drying => "⌘8",
            Self::Recipes => "⌘9",
            Self::Settings => "⌘0",
        }
    }
}

/// Application services and repositories shared across views via Dioxus context.
#[derive(Clone)]
pub struct AppServices {
    pub db_dir: PathBuf,
    pub _database: Database,
    pub catalog: Arc<SeaOrmPaintCatalogRepository>,
    pub palettes: Arc<SeaOrmPaletteRepository>,
    pub projects: Arc<SeaOrmProjectRepository>,
    pub recipes: Arc<SeaOrmRecipeRepository>,
    pub mixes: Arc<SeaOrmPaletteMixRepository>,
    pub settings: Arc<SeaOrmSettingsRepository>,
    pub ai_store: Arc<AiSettingsStore>,
    pub chat_engines: Arc<ChatEngines>,
    pub plan_service: Arc<PaintingPlanService>,
    pub recognition_service: Arc<TubeRecognitionService>,
    pub model_catalog: Arc<ModelCatalog>,
    pub mix_service: ColorMixService,
    pub drying_service: DryingTimeService,
    pub fat_over_lean: FatOverLeanService,
    pub substitute_service: SubstituteService,
    pub progress_service: ProgressCheckService,
    pub palette_mix_service: PaletteMixService,
    pub recipe_timeline: RecipeTimelineService,
    pub workbench: WorkbenchService,
    pub planner: ProjectPlanner,
    pub sampled_plan_service: SampledPlanService,
    pub guide_builder: MixingGuideBuilder,
    pub decoder: ImageDecoder,
}

impl PartialEq for AppServices {
    fn eq(&self, other: &Self) -> bool {
        self.db_dir == other.db_dir && Arc::ptr_eq(&self.catalog, &other.catalog)
    }
}

impl AppServices {
    pub async fn init(db_dir: PathBuf) -> DomainResult<Self> {
        let database = Database::open(&db_dir).await?;
        let catalog = Arc::new(SeaOrmPaintCatalogRepository::new(&database));
        let palettes = Arc::new(SeaOrmPaletteRepository::new(&database, catalog.clone()));
        let projects = Arc::new(SeaOrmProjectRepository::new(
            &database,
            catalog.clone(),
            palettes.clone(),
        ));
        let recipes = Arc::new(SeaOrmRecipeRepository::new(&database));
        let mixes = Arc::new(SeaOrmPaletteMixRepository::new(&database));
        let settings = Arc::new(SeaOrmSettingsRepository::with_keyring(&database));

        let seeder = CatalogSeeder::new(catalog.clone(), palettes.clone(), recipes.clone());
        let _ = seeder.seed().await?;

        let ai_store = Arc::new(AiSettingsStore::new(settings.clone()));
        let initial_ai = ai_store.load().await.unwrap_or_default();
        let chat_engines = Arc::new(ChatEngines::new(initial_ai)?);
        let plan_service = Arc::new(PaintingPlanService::new(chat_engines.clone()));
        let recognition_service = Arc::new(TubeRecognitionService::new(chat_engines.clone()));
        let model_catalog = Arc::new(ModelCatalog::new());

        Ok(Self {
            db_dir,
            _database: database,
            catalog,
            palettes,
            projects,
            recipes,
            mixes,
            settings,
            ai_store,
            chat_engines,
            plan_service,
            recognition_service,
            model_catalog,
            mix_service: ColorMixService,
            drying_service: DryingTimeService::default(),
            fat_over_lean: FatOverLeanService,
            substitute_service: SubstituteService,
            progress_service: ProgressCheckService,
            palette_mix_service: PaletteMixService::default(),
            recipe_timeline: RecipeTimelineService::default(),
            workbench: WorkbenchService::default(),
            planner: ProjectPlanner::default(),
            sampled_plan_service: SampledPlanService::default(),
            guide_builder: MixingGuideBuilder::default(),
            decoder: ImageDecoder,
        })
    }

    pub fn importer(&self) -> BackupImporter {
        BackupImporter::new(
            self.catalog.clone(),
            self.palettes.clone(),
            self.projects.clone(),
            self.recipes.clone(),
        )
    }

    pub fn exporter(&self) -> BackupExporter {
        BackupExporter::new(
            self.catalog.clone(),
            self.palettes.clone(),
            self.projects.clone(),
            self.recipes.clone(),
        )
    }
}

/// Reactive application state stored in Dioxus signals.
#[derive(Clone, Copy)]
pub struct AppState {
    pub section: Signal<Section>,
    pub locale: Signal<Locale>,
    pub theme: Signal<ThemeMode>,
    pub paints: Signal<Vec<Paint>>,
    pub palettes: Signal<Vec<Palette>>,
    pub projects: Signal<Vec<Project>>,
    pub recipes: Signal<Vec<Recipe>>,
    pub mixes: Signal<Vec<PaletteMix>>,
    pub workshop: Signal<Workshop>,
    pub ai_settings: Signal<AiSettings>,
    pub toast: Signal<Option<String>>,
    pub sampled_zones: Signal<Vec<SampledZone>>,
    pub mixer_target_hex: Signal<String>,
    pub last_sampled_hex: Signal<Option<String>>,
    pub command_palette_open: Signal<bool>,
    pub miniature_tab: Signal<usize>,
}

impl AppState {
    pub const KEY_LOCALE: &'static str = "ui.locale";
    pub const KEY_THEME: &'static str = "ui.theme";

    pub async fn reload_all(&mut self, services: &AppServices) {
        if let Ok(Some(loc_str)) = services.settings.get(Self::KEY_LOCALE).await
            && let Ok(loc) = Locale::from_str(&loc_str)
        {
            self.locale.set(loc);
        }
        if let Ok(Some(th_str)) = services.settings.get(Self::KEY_THEME).await
            && let Ok(th) = ThemeMode::from_str(&th_str)
        {
            self.theme.set(th);
        }
        if let Ok(p) = services.catalog.all().await {
            self.paints.set(p);
        }
        if let Ok(pal) = services.palettes.all().await {
            self.palettes.set(pal);
        }
        if let Ok(proj) = services.projects.all().await {
            self.projects.set(proj);
        }
        if let Ok(rec) = services.recipes.all().await {
            self.recipes.set(rec);
        }
        if let Ok(m) = services.mixes.all().await {
            self.mixes.set(m);
        }
        if let Ok(ai) = services.ai_store.load().await {
            services.chat_engines.configure(ai.clone());
            self.ai_settings.set(ai);
        }
    }

    pub fn notify(&mut self, msg: impl Into<String>) {
        self.toast.set(Some(msg.into()));
    }
}
