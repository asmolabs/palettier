use super::archive::BackupArchive;
use super::model::{
    BackupDocument, BackupLayer, BackupPaint, BackupPaintRef, BackupPalette, BackupProject,
    BackupRecipe,
};
use super::{STUDIO_SETTING_KEYS, parse_iso_instant};
use crate::db::{Database, DbExec, map_db_err};
use crate::repository::{
    SeaOrmPaintCatalogRepository, SeaOrmPaletteRepository, SeaOrmProjectRepository,
    SeaOrmRecipeRepository, SeaOrmSettingsRepository,
};
use chrono::Utc;
use indexmap::{IndexMap, IndexSet};
use palettier_domain::color::Rgb;
use palettier_domain::drying::Workshop;
use palettier_domain::error::{DomainError, DomainResult};
use palettier_domain::i18n::Locale;
use palettier_domain::paint::{
    DryingClass, LayerThickness, Medium, Opacity, Paint, PaintSystem, Technique, Ventilation,
};
use palettier_domain::palette::Palette;
use palettier_domain::port::{
    PaintCatalogRepository, PaletteRepository, ProjectRepository, RecipeRepository,
    SettingsRepository,
};
use palettier_domain::project::{
    AppliedCoat, LayerKind, PhotoRole, Project, ProjectLayer, ProjectPhoto, ProjectZone,
};
use palettier_domain::recipe::{Recipe, RecipeStep};
use sea_orm::{DatabaseConnection, TransactionTrait};
use std::str::FromStr;
use std::sync::Arc;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImportReport {
    pub paints_updated: usize,
    pub paints_created: usize,
    pub palettes_added: usize,
    pub palettes_skipped: usize,
    pub projects_added: usize,
    pub projects_skipped: usize,
    pub recipes_added: usize,
    pub recipes_skipped: usize,
    pub photos: usize,
    pub warnings: Vec<String>,
}

impl ImportReport {
    pub fn skipped(&self) -> usize {
        self.palettes_skipped + self.projects_skipped + self.recipes_skipped
    }
}

pub struct BackupImporter {
    catalog: Arc<dyn PaintCatalogRepository>,
    palettes: Arc<dyn PaletteRepository>,
    projects: Arc<dyn ProjectRepository>,
    recipes: Arc<dyn RecipeRepository>,
    settings: Option<Arc<dyn SettingsRepository>>,
    db: Option<DatabaseConnection>,
}

impl BackupImporter {
    pub const SUPPORTED_FORMAT: i32 = 1;

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
            db: None,
        }
    }

    pub fn with_settings(mut self, settings: Arc<dyn SettingsRepository>) -> Self {
        self.settings = Some(settings);
        self
    }

    /// Runs the import inside a single overarching SQLite transaction when provided.
    pub fn with_database(mut self, db: &Database) -> Self {
        self.db = Some(db.connection().clone());
        self
    }

    pub async fn import<A: BackupArchive>(&self, archive: &mut A) -> DomainResult<ImportReport> {
        self.import_in(archive, Locale::Fr).await
    }

    pub async fn import_in<A: BackupArchive>(
        &self,
        archive: &mut A,
        locale: Locale,
    ) -> DomainResult<ImportReport> {
        let doc_bytes = archive.document()?;
        let document: BackupDocument = serde_json::from_slice(&doc_bytes).map_err(|e| {
            let msg = match locale {
                Locale::Fr => format!("Document de sauvegarde JSON invalide: {e}"),
                Locale::En => format!("Invalid JSON backup document: {e}"),
            };
            DomainError::InvalidArgument(msg)
        })?;

        if document.format_version > Self::SUPPORTED_FORMAT {
            let msg = match locale {
                Locale::Fr => format!(
                    "Sauvegarde écrite par une version plus récente (format {}, connu jusqu'à {}).",
                    document.format_version,
                    Self::SUPPORTED_FORMAT
                ),
                Locale::En => format!(
                    "Backup written by a newer version (format {}, supported up to {}).",
                    document.format_version,
                    Self::SUPPORTED_FORMAT
                ),
            };
            return Err(DomainError::InvalidArgument(msg));
        }

        let Some(ref conn) = self.db else {
            return self.apply_document(&document, archive, locale).await;
        };

        let txn = Arc::new(conn.begin().await.map_err(map_db_err)?);
        let result = {
            let exec = DbExec::Txn(txn.clone());
            let catalog = Arc::new(SeaOrmPaintCatalogRepository::from_exec(exec.clone()));
            let palettes = Arc::new(SeaOrmPaletteRepository::from_exec(
                exec.clone(),
                catalog.clone(),
            ));
            let projects = Arc::new(SeaOrmProjectRepository::from_exec(
                exec.clone(),
                catalog.clone(),
                palettes.clone(),
            ));
            let recipes = Arc::new(SeaOrmRecipeRepository::from_exec(exec.clone()));
            let settings: Option<Arc<dyn SettingsRepository>> = self
                .settings
                .as_ref()
                .map(|_| Arc::new(SeaOrmSettingsRepository::from_exec(exec, false)) as Arc<dyn SettingsRepository>);
            let scoped = Self {
                catalog,
                palettes,
                projects,
                recipes,
                settings,
                db: None,
            };
            scoped.apply_document(&document, archive, locale).await
        };

        let txn = Arc::try_unwrap(txn).map_err(|_| {
            DomainError::Persistence("Transaction reference leaked during backup import".to_string())
        })?;
        match result {
            Ok(report) => {
                txn.commit().await.map_err(map_db_err)?;
                Ok(report)
            }
            Err(err) => {
                let _ = txn.rollback().await;
                Err(err)
            }
        }
    }

    async fn apply_document<A: BackupArchive>(
        &self,
        document: &BackupDocument,
        archive: &mut A,
        locale: Locale,
    ) -> DomainResult<ImportReport> {
        let mut warnings = Vec::new();
        let (paints_updated, paints_created) = self.restore_paints(&document.paints).await?;
        let (palettes_added, palettes_skipped) = self
            .restore_palettes(&document.palettes, &mut warnings, locale)
            .await?;
        let (recipes_added, recipes_skipped) = self.restore_recipes(&document.recipes).await?;
        let (projects_added, projects_skipped, photos) = self
            .restore_projects(&document.projects, archive, &mut warnings, locale)
            .await?;
        if let Some(ref settings_repo) = self.settings {
            for key in STUDIO_SETTING_KEYS {
                if let Some(val) = document.settings.get(key) {
                    if let Err(reason) = validate_studio_setting(key, val, locale) {
                        let msg = match locale {
                            Locale::Fr => format!("Préférence {key} ignorée : {reason}"),
                            Locale::En => format!("Preference {key} ignored: {reason}"),
                        };
                        warnings.push(msg);
                    } else {
                        settings_repo.put(key, val).await?;
                    }
                }
            }
        }

        Ok(ImportReport {
            paints_updated,
            paints_created,
            palettes_added,
            palettes_skipped,
            projects_added,
            projects_skipped,
            recipes_added,
            recipes_skipped,
            photos,
            warnings,
        })
    }

    async fn restore_paints(&self, entries: &[BackupPaint]) -> DomainResult<(usize, usize)> {
        let mut updated = 0;
        let mut created = 0;
        for entry in entries {
            if let Some(existing) = self
                .catalog
                .find_by_natural_key(&entry.brand, &entry.name)
                .await?
            {
                if let Some(id) = existing.id {
                    self.catalog.set_owned(id, entry.owned).await?;
                    if let Ok(masstone) = Rgb::of_hex(&entry.hex)
                        && !masstone.to_hex().eq_ignore_ascii_case(&existing.hex_color)
                    {
                        self.catalog.record_masstone(id, masstone).await?;
                    }
                    if let Some(ref tint) = entry.tint_hex
                        && let Ok(rgb) = Rgb::of_hex(tint)
                    {
                        self.catalog.record_tint(id, rgb).await?;
                    }
                    updated += 1;
                }
            } else {
                let hex_color = Rgb::of_hex(&entry.hex)
                    .map(|c| c.to_hex())
                    .unwrap_or_else(|_| "#808080".to_string());
                let tint_hex = entry
                    .tint_hex
                    .as_deref()
                    .and_then(|t| Rgb::of_hex(t).ok())
                    .map(|c| c.to_hex());
                let tinting_strength = if entry.tinting_strength.is_finite() {
                    entry.tinting_strength.clamp(0.01, 1.0)
                } else {
                    0.5
                };
                let paint = Paint {
                    id: None,
                    brand: entry.brand.clone(),
                    name: entry.name.clone(),
                    code: entry.code.clone(),
                    legacy_code: String::new(),
                    pigments: entry.pigments.iter().cloned().collect(),
                    hex_color,
                    tint_hex,
                    opacity: Opacity::from_str(&entry.opacity).unwrap_or(Opacity::SemiOpaque),
                    drying_class: DryingClass::from_str(&entry.drying_class)
                        .unwrap_or(DryingClass::Medium),
                    tinting_strength,
                    color_derived: entry.color_derived,
                    pigments_verified: false,
                    user_added: true,
                    in_stock: entry.owned,
                    notes: entry.notes.clone(),
                    system: PaintSystem::from_str(&entry.system).unwrap_or_default(),
                };
                self.catalog.save(&paint).await?;
                created += 1;
            }
        }
        Ok((updated, created))
    }

    async fn restore_palettes(
        &self,
        entries: &[BackupPalette],
        warnings: &mut Vec<String>,
        locale: Locale,
    ) -> DomainResult<(usize, usize)> {
        let known: IndexSet<String> = self
            .palettes
            .all()
            .await?
            .into_iter()
            .map(|p| p.name)
            .collect();

        let mut added = 0;
        let mut skipped = 0;
        for entry in entries {
            if known.contains(&entry.name) {
                skipped += 1;
                continue;
            }
            let ctx = match locale {
                Locale::Fr => format!("palette {}", entry.name),
                Locale::En => format!("palette {}", entry.name),
            };
            let paints = self.resolve(&entry.paints, &ctx, warnings, locale).await?;
            let palette = Palette {
                id: None,
                name: entry.name.clone(),
                purpose: entry.purpose.clone(),
                notes: entry.notes.clone(),
                paints,
            };
            self.palettes.save(&palette).await?;
            added += 1;
        }
        Ok((added, skipped))
    }

    async fn restore_recipes(&self, entries: &[BackupRecipe]) -> DomainResult<(usize, usize)> {
        let mut added = 0;
        let mut skipped = 0;
        for entry in entries {
            if self.recipes.exists(&entry.name).await? {
                skipped += 1;
                continue;
            }
            let steps = entry
                .steps
                .iter()
                .map(|s| RecipeStep {
                    id: None,
                    technique: Technique::from_str(&s.technique).unwrap_or(Technique::BaseLayer),
                    paint_mix: s.paint_mix.clone(),
                    medium: Medium::from_str(&s.medium).unwrap_or(Medium::None),
                    medium_ratio: s.medium_ratio,
                    thickness: LayerThickness::from_str(&s.thickness)
                        .unwrap_or(LayerThickness::Normal),
                    notes: s.note.clone(),
                })
                .collect();
            let recipe = Recipe {
                id: None,
                name: entry.name.clone(),
                subject: entry.subject.clone(),
                notes: entry.notes.clone(),
                steps,
            };
            self.recipes.save(&recipe).await?;
            added += 1;
        }
        Ok((added, skipped))
    }

    async fn restore_projects<A: BackupArchive>(
        &self,
        entries: &[BackupProject],
        archive: &mut A,
        warnings: &mut Vec<String>,
        locale: Locale,
    ) -> DomainResult<(usize, usize, usize)> {
        let existing: IndexSet<String> = self
            .projects
            .all()
            .await?
            .into_iter()
            .map(|p| p.name)
            .collect();
        let palettes_by_name: IndexMap<String, Palette> = self
            .palettes
            .all()
            .await?
            .into_iter()
            .map(|p| (p.name.clone(), p))
            .collect();

        let mut added = 0;
        let mut skipped = 0;
        let mut photos_count = 0;

        for entry in entries {
            if existing.contains(&entry.name) {
                skipped += 1;
                continue;
            }
            let ctx = match locale {
                Locale::Fr => format!("projet {}", entry.name),
                Locale::En => format!("project {}", entry.name),
            };
            let paints = self.resolve(&entry.paints, &ctx, warnings, locale).await?;
            let system = PaintSystem::from_str(&entry.system).unwrap_or_default();
            let zones = entry
                .zones
                .iter()
                .map(|z| ProjectZone {
                    id: None,
                    name: z.name.clone(),
                    material: z.material.clone(),
                    note: z.note.clone(),
                    layers: z.layers.iter().map(|l| Self::to_layer(l, system)).collect(),
                })
                .collect();

            let created_at = entry
                .created_at
                .as_deref()
                .and_then(parse_iso_instant)
                .unwrap_or_else(Utc::now);

            let project = self
                .projects
                .save(&Project {
                    id: None,
                    name: entry.name.clone(),
                    subject: entry.subject.clone(),
                    approach: entry.approach.clone(),
                    notes: entry.notes.clone(),
                    system,
                    palette: entry
                        .palette_name
                        .as_ref()
                        .and_then(|n| palettes_by_name.get(n).cloned()),
                    paints,
                    zones,
                    photos: Vec::new(),
                    created_at,
                })
                .await?;

            if let Some(proj_id) = project.id {
                for photo in &entry.photos {
                    let Some(data) = archive.file(&photo.file)? else {
                        let msg = match locale {
                            Locale::Fr => {
                                format!("Image annoncée mais absente de l'archive : {}", photo.file)
                            }
                            Locale::En => {
                                format!("Image listed but missing from archive: {}", photo.file)
                            }
                        };
                        warnings.push(msg);
                        continue;
                    };
                    let added_at = photo
                        .added_at
                        .as_deref()
                        .and_then(parse_iso_instant)
                        .unwrap_or_else(Utc::now);
                    let role = PhotoRole::from_str(&photo.role).unwrap_or(PhotoRole::Piece);
                    self.projects
                        .add_photo(
                            proj_id,
                            &ProjectPhoto {
                                id: None,
                                data,
                                role,
                                caption: photo.caption.clone(),
                                added_at,
                            },
                        )
                        .await?;
                    photos_count += 1;
                }
            }
            added += 1;
        }
        Ok((added, skipped, photos_count))
    }

    fn to_layer(layer: &BackupLayer, system: PaintSystem) -> ProjectLayer {
        let applied = layer
            .applied_at
            .as_deref()
            .and_then(parse_iso_instant)
            .map(|at| {
                let vent = layer
                    .applied_ventilation
                    .as_deref()
                    .and_then(|s| Ventilation::from_str(s).ok());
                let dc = layer
                    .applied_drying_class
                    .as_deref()
                    .and_then(|s| DryingClass::from_str(s).ok());
                AppliedCoat::of(
                    at,
                    layer.applied_temperature,
                    layer.applied_humidity,
                    vent,
                    dc,
                )
            });

        let target_hex = Rgb::of_hex(&layer.target_hex)
            .map(|c| c.to_hex())
            .unwrap_or_else(|_| "#808080".to_string());
        let mut out = ProjectLayer {
            id: None,
            role: layer.role.clone(),
            target_hex,
            technique: Technique::default_for(system),
            note: layer.note.clone(),
            kind: LayerKind::from_str(&layer.kind).unwrap_or(LayerKind::Ladder),
            applied,
        };
        out.set_technique_label(&layer.technique, system);
        out
    }

    async fn resolve(
        &self,
        refs: &[BackupPaintRef],
        context: &str,
        warnings: &mut Vec<String>,
        locale: Locale,
    ) -> DomainResult<Vec<Paint>> {
        let mut out = Vec::new();
        for r in refs {
            match self.catalog.find_by_natural_key(&r.brand, &r.name).await? {
                Some(p) => out.push(p),
                None => {
                    let msg = match locale {
                        Locale::Fr => {
                            format!("Tube introuvable pour {context} : {} - {}", r.brand, r.name)
                        }
                        Locale::En => {
                            format!("Paint not found for {context}: {} - {}", r.brand, r.name)
                        }
                    };
                    warnings.push(msg);
                }
            }
        }
        Ok(out)
    }
}

fn validate_studio_setting(key: &str, value: &str, locale: Locale) -> Result<(), String> {
    match key {
        "ui.locale" => Locale::from_str(value).map(|_| ()).map_err(|_| match locale {
            Locale::Fr => format!("langue inconnue '{value}'"),
            Locale::En => format!("unknown language '{value}'"),
        }),
        "ui.theme" => {
            if matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "dark" | "light" | "neutral"
            ) {
                Ok(())
            } else {
                Err(match locale {
                    Locale::Fr => format!("thème inconnu '{value}'"),
                    Locale::En => format!("unknown theme '{value}'"),
                })
            }
        }
        "ui.paint_system" => {
            PaintSystem::from_str(value)
                .map(|_| ())
                .map_err(|_| match locale {
                    Locale::Fr => format!("système inconnu '{value}'"),
                    Locale::En => format!("unknown paint system '{value}'"),
                })
        }
        "ui.workshop" => {
            let mut parts = value.split(';');
            let temp = parts
                .next()
                .and_then(|s| s.trim().parse::<f64>().ok())
                .ok_or_else(|| match locale {
                    Locale::Fr => "température invalide".to_string(),
                    Locale::En => "invalid temperature".to_string(),
                })?;
            let humidity = parts
                .next()
                .and_then(|s| s.trim().parse::<f64>().ok())
                .ok_or_else(|| match locale {
                    Locale::Fr => "humidité invalide".to_string(),
                    Locale::En => "invalid humidity".to_string(),
                })?;
            let vent = parts
                .next()
                .and_then(|s| Ventilation::from_str(s.trim()).ok())
                .ok_or_else(|| match locale {
                    Locale::Fr => "ventilation invalide".to_string(),
                    Locale::En => "invalid ventilation".to_string(),
                })?;
            if parts.next().is_some() {
                return Err(match locale {
                    Locale::Fr => "format atelier inattendu".to_string(),
                    Locale::En => "unexpected workshop format".to_string(),
                });
            }
            Workshop::new(temp, humidity, vent)
                .map(|_| ())
                .map_err(|e| e.message(locale))
        }
        _ => Err(match locale {
            Locale::Fr => "clé non gérée".to_string(),
            Locale::En => "unsupported key".to_string(),
        }),
    }
}
