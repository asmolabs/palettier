use super::archive::BackupWriter;
use super::model::{
    BackupDocument, BackupLayer, BackupPaint, BackupPaintRef, BackupPalette, BackupPhoto,
    BackupProject, BackupRecipe, BackupStep, BackupZone,
};
use super::{BackupImporter, STUDIO_SETTING_KEYS, format_iso_instant};
use indexmap::{IndexMap, IndexSet};
use palettier_domain::error::{DomainError, DomainResult};
use palettier_domain::port::{
    PaintCatalogRepository, PaletteRepository, ProjectRepository, RecipeRepository,
    SettingsRepository,
};
use palettier_domain::project::Project;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportReport {
    pub paints: usize,
    pub palettes: usize,
    pub projects: usize,
    pub recipes: usize,
    pub photos: usize,
}

pub struct BackupExporter {
    catalog: Arc<dyn PaintCatalogRepository>,
    palettes: Arc<dyn PaletteRepository>,
    projects: Arc<dyn ProjectRepository>,
    recipes: Arc<dyn RecipeRepository>,
    settings: Option<Arc<dyn SettingsRepository>>,
}

impl BackupExporter {
    pub const DOCUMENT: &'static str = "palettier.json";
    pub const IMAGES: &'static str = "images";
    const APPLICATION: &'static str = "Palettier";

    pub fn new(
        catalog: Arc<dyn PaintCatalogRepository>,
        palettes: Arc<dyn PaletteRepository>,
        projects: Arc<dyn ProjectRepository>,
        recipes: Arc<dyn RecipeRepository>,
    ) -> Self {
        Self {
            catalog,
            palettes,
            projects,
            recipes,
            settings: None,
        }
    }

    pub fn with_settings(mut self, settings: Arc<dyn SettingsRepository>) -> Self {
        self.settings = Some(settings);
        self
    }

    pub async fn export<W: BackupWriter>(&self, writer: &mut W) -> DomainResult<ExportReport> {
        let paints = self.catalog.all().await?;
        let all_palettes = self.palettes.all().await?;
        let all_recipes = self.recipes.all().await?;
        let mut settings = IndexMap::new();
        if let Some(ref settings_repo) = self.settings {
            for key in STUDIO_SETTING_KEYS {
                if let Some(val) = settings_repo.get(key).await? {
                    settings.insert(key.to_string(), val);
                }
            }
        }

        let mut pieces = Vec::new();
        for p in self.projects.all().await? {
            if let Some(id) = p.id
                && let Some(full) = self.projects.find_with_photos(id).await?
            {
                pieces.push(full);
            }
        }

        let mut photos = 0;
        let mut proj_docs = Vec::with_capacity(pieces.len());
        let mut used_slugs = IndexSet::new();
        for project in &pieces {
            let base_slug = Self::slug(&project.name);
            let mut unique_slug = base_slug.clone();
            let mut suffix = 2usize;
            while !used_slugs.insert(unique_slug.clone()) {
                unique_slug = format!("{base_slug}-{suffix}");
                suffix += 1;
            }

            let mut entries = Vec::with_capacity(project.photos.len());
            for (index, photo) in project.photos.iter().enumerate() {
                let path = format!("{}/{}-{:02}.jpg", Self::IMAGES, unique_slug, index + 1);
                writer.write_file(path.clone(), photo.data.clone());
                photos += 1;
                entries.push(BackupPhoto {
                    file: path,
                    role: photo.role.as_db_str().to_string(),
                    caption: photo.caption.clone(),
                    added_at: Some(format_iso_instant(photo.added_at)),
                });
            }
            proj_docs.push(Self::describe(project, entries));
        }

        let document = BackupDocument {
            format_version: BackupImporter::SUPPORTED_FORMAT,
            exported_at: Some(format_iso_instant(chrono::Utc::now())),
            application: Some(Self::APPLICATION.to_string()),
            paints: paints
                .iter()
                .map(|p| BackupPaint {
                    brand: p.brand.clone(),
                    name: p.name.clone(),
                    code: p.code.clone(),
                    pigments: p.pigments.iter().cloned().collect(),
                    hex: p.hex_color.clone(),
                    tint_hex: p.tint_hex.clone(),
                    opacity: p.opacity.as_db_str().to_string(),
                    drying_class: p.drying_class.as_db_str().to_string(),
                    tinting_strength: p.tinting_strength,
                    owned: p.in_stock,
                    color_derived: p.color_derived,
                    notes: p.notes.clone(),
                    system: p.system.as_db_str().to_string(),
                    user_added: p.user_added,
                })
                .collect(),
            palettes: all_palettes
                .iter()
                .map(|pal| BackupPalette {
                    name: pal.name.clone(),
                    purpose: pal.purpose.clone(),
                    notes: pal.notes.clone(),
                    paints: pal
                        .paints
                        .iter()
                        .map(|p| BackupPaintRef {
                            brand: p.brand.clone(),
                            name: p.name.clone(),
                        })
                        .collect(),
                })
                .collect(),
            projects: proj_docs,
            recipes: all_recipes
                .iter()
                .map(|rec| BackupRecipe {
                    name: rec.name.clone(),
                    subject: rec.subject.clone(),
                    notes: rec.notes.clone(),
                    steps: rec
                        .steps
                        .iter()
                        .map(|s| BackupStep {
                            technique: s.technique.as_db_str().to_string(),
                            paint_mix: s.paint_mix.clone(),
                            medium: s.medium.as_db_str().to_string(),
                            medium_ratio: s.medium_ratio,
                            thickness: s.thickness.as_db_str().to_string(),
                            note: s.notes.clone(),
                        })
                        .collect(),
                })
                .collect(),
            settings,
        };

        let json = serde_json::to_string_pretty(&document)
            .map_err(|e| DomainError::Persistence(format!("Erreur de sérialisation JSON: {e}")))?;
        writer.write_document(json);

        Ok(ExportReport {
            paints: paints.len(),
            palettes: all_palettes.len(),
            projects: pieces.len(),
            recipes: all_recipes.len(),
            photos,
        })
    }

    fn describe(project: &Project, photos: Vec<BackupPhoto>) -> BackupProject {
        BackupProject {
            name: project.name.clone(),
            subject: project.subject.clone(),
            approach: project.approach.clone(),
            notes: project.notes.clone(),
            palette_name: project.palette.as_ref().map(|p| p.name.clone()),
            system: project.system.as_db_str().to_string(),
            created_at: Some(format_iso_instant(project.created_at)),
            paints: project
                .paints
                .iter()
                .map(|p| BackupPaintRef {
                    brand: p.brand.clone(),
                    name: p.name.clone(),
                })
                .collect(),
            zones: project
                .zones
                .iter()
                .map(|z| BackupZone {
                    name: z.name.clone(),
                    material: z.material.clone(),
                    note: z.note.clone(),
                    layers: z
                        .layers
                        .iter()
                        .map(|l| BackupLayer {
                            role: l.role.clone(),
                            target_hex: l.target_hex.clone(),
                            // The unaccented French label: the form the Java version reads.
                            technique: l.technique.legacy_label(),
                            note: l.note.clone(),
                            kind: l.kind.as_db_str().to_string(),
                            applied_at: l.applied.as_ref().map(|a| format_iso_instant(a.at)),
                            applied_temperature: l
                                .applied
                                .as_ref()
                                .map(|a| a.workshop.temperature_celsius),
                            applied_humidity: l
                                .applied
                                .as_ref()
                                .map(|a| a.workshop.relative_humidity),
                            applied_ventilation: l
                                .applied
                                .as_ref()
                                .map(|a| a.workshop.ventilation.as_db_str().to_string()),
                            applied_drying_class: l
                                .applied
                                .as_ref()
                                .map(|a| a.drying_class.as_db_str().to_string()),
                        })
                        .collect(),
                })
                .collect(),
            photos,
        }
    }

    fn slug(name: &str) -> String {
        let mapped: String = name
            .to_lowercase()
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
            .collect();
        let mut collapsed = String::new();
        let mut prev_dash = false;
        for c in mapped.trim_matches('-').chars() {
            if c == '-' {
                if !prev_dash {
                    collapsed.push('-');
                }
                prev_dash = true;
            } else {
                collapsed.push(c);
                prev_dash = false;
            }
        }
        if collapsed.is_empty() {
            "projet".to_string()
        } else {
            collapsed
        }
    }
}
