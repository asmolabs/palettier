use chrono::{DateTime, Utc};
use indexmap::{IndexMap, IndexSet};
use palettier_domain::color::Rgb;
use palettier_domain::error::{DomainError, DomainResult};
use palettier_domain::paint::{
    DryingClass, LayerThickness, Medium, Opacity, Paint, Technique, Ventilation,
};
use palettier_domain::palette::Palette;
use palettier_domain::port::{
    PaintCatalogRepository, PaletteRepository, ProjectRepository, RecipeRepository,
};
use palettier_domain::project::{
    AppliedCoat, LayerKind, PhotoRole, Project, ProjectLayer, ProjectPhoto, ProjectZone,
};
use palettier_domain::recipe::{Recipe, RecipeStep};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;
use zip::ZipArchive;
use zip::write::SimpleFileOptions;

/// Maximum accepted size for `palettier.json` (32 MB) to guard against Zip Bombs.
const MAX_JSON_BYTES: u64 = 32 * 1024 * 1024;
/// Maximum accepted size per image in an archive (50 MB).
const MAX_ENTRY_BYTES: u64 = 50 * 1024 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupDocument {
    #[serde(default = "default_format_version")]
    pub format_version: i32,
    #[serde(default)]
    pub exported_at: Option<String>,
    #[serde(default)]
    pub application: Option<String>,
    #[serde(default)]
    pub paints: Vec<BackupPaint>,
    #[serde(default)]
    pub palettes: Vec<BackupPalette>,
    #[serde(default)]
    pub projects: Vec<BackupProject>,
    #[serde(default)]
    pub recipes: Vec<BackupRecipe>,
}

fn default_format_version() -> i32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupPaint {
    pub brand: String,
    pub name: String,
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub pigments: Vec<String>,
    pub hex: String,
    #[serde(default)]
    pub tint_hex: Option<String>,
    pub opacity: String,
    pub drying_class: String,
    #[serde(default = "default_tinting")]
    pub tinting_strength: f64,
    #[serde(default)]
    pub owned: bool,
    #[serde(default)]
    pub color_derived: bool,
    #[serde(default)]
    pub notes: String,
}

fn default_tinting() -> f64 {
    0.5
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupPaintRef {
    pub brand: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupPalette {
    pub name: String,
    #[serde(default)]
    pub purpose: String,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub paints: Vec<BackupPaintRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupProject {
    pub name: String,
    #[serde(default)]
    pub subject: String,
    #[serde(default)]
    pub approach: String,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub palette_name: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub paints: Vec<BackupPaintRef>,
    #[serde(default)]
    pub zones: Vec<BackupZone>,
    #[serde(default)]
    pub photos: Vec<BackupPhoto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupZone {
    pub name: String,
    #[serde(default)]
    pub material: String,
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    pub layers: Vec<BackupLayer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupLayer {
    pub role: String,
    pub target_hex: String,
    #[serde(default)]
    pub technique: String,
    #[serde(default)]
    pub note: String,
    #[serde(default = "default_layer_kind")]
    pub kind: String,
    #[serde(default)]
    pub applied_at: Option<String>,
    #[serde(default)]
    pub applied_temperature: Option<f64>,
    #[serde(default)]
    pub applied_humidity: Option<f64>,
    #[serde(default)]
    pub applied_ventilation: Option<String>,
    #[serde(default)]
    pub applied_drying_class: Option<String>,
}

fn default_layer_kind() -> String {
    "LADDER".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupPhoto {
    pub file: String,
    pub role: String,
    #[serde(default)]
    pub caption: String,
    #[serde(default)]
    pub added_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupRecipe {
    pub name: String,
    #[serde(default)]
    pub subject: String,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub steps: Vec<BackupStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupStep {
    pub technique: String,
    #[serde(default)]
    pub paint_mix: String,
    pub medium: String,
    #[serde(default)]
    pub medium_ratio: f64,
    pub thickness: String,
    #[serde(default)]
    pub note: String,
}

pub trait BackupArchive {
    fn document(&mut self) -> DomainResult<Vec<u8>>;
    fn file(&mut self, path: &str) -> DomainResult<Option<Vec<u8>>>;
}

pub trait BackupWriter {
    fn write_document(&mut self, json: String);
    fn write_file(&mut self, path: String, bytes: Vec<u8>);
    fn finish(self) -> DomainResult<()>;
}

pub struct ZipBackupArchive {
    zip: ZipArchive<File>,
}

impl ZipBackupArchive {
    pub const DOCUMENT: &'static str = "palettier.json";

    pub fn open(path: impl AsRef<Path>) -> DomainResult<Self> {
        let file = File::open(path.as_ref()).map_err(|e| {
            DomainError::Persistence(format!("Impossible d'ouvrir l'archive {:?}: {e}", path.as_ref()))
        })?;
        let zip = ZipArchive::new(file).map_err(|e| {
            DomainError::Persistence(format!("Archive ZIP invalide {:?}: {e}", path.as_ref()))
        })?;
        Ok(Self { zip })
    }

    fn read_bounded(mut entry: zip::read::ZipFile<'_, File>, max_bytes: u64) -> DomainResult<Vec<u8>> {
        // Zip-Slip protection: reject paths containing `..` or absolute paths
        if entry.enclosed_name().is_none() {
            return Err(DomainError::InvalidArgument(format!(
                "Chemin d'entrée ZIP dangereux rejeté : {:?}",
                entry.name()
            )));
        }
        if entry.size() > max_bytes {
            return Err(DomainError::InvalidArgument(format!(
                "Entrée ZIP trop volumineuse ({} octets > limite {})",
                entry.size(),
                max_bytes
            )));
        }
        let mut buf = Vec::with_capacity(entry.size().min(1024 * 1024) as usize);
        let mut limited = (&mut entry).take(max_bytes + 1);
        limited
            .read_to_end(&mut buf)
            .map_err(|e| DomainError::Persistence(format!("Erreur de lecture ZIP: {e}")))?;
        if (buf.len() as u64) > max_bytes {
            return Err(DomainError::InvalidArgument(format!(
                "Entrée ZIP dépassant la taille maximale autorisée ({max_bytes} octets)"
            )));
        }
        Ok(buf)
    }
}

impl BackupArchive for ZipBackupArchive {
    fn document(&mut self) -> DomainResult<Vec<u8>> {
        let entry = self.zip.by_name(Self::DOCUMENT).map_err(|_| {
            DomainError::InvalidArgument(format!(
                "Cette archive ne contient pas de {} : ce n'est pas une sauvegarde Palettier.",
                Self::DOCUMENT
            ))
        })?;
        Self::read_bounded(entry, MAX_JSON_BYTES)
    }

    fn file(&mut self, path: &str) -> DomainResult<Option<Vec<u8>>> {
        // Pre-validate the requested path
        if Path::new(path)
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir | std::path::Component::RootDir))
        {
            return Err(DomainError::InvalidArgument(format!(
                "Chemin d'archive non sécurisé : {path}"
            )));
        }
        match self.zip.by_name(path) {
            Ok(entry) => Ok(Some(Self::read_bounded(entry, MAX_ENTRY_BYTES)?)),
            Err(zip::result::ZipError::FileNotFound) => Ok(None),
            Err(e) => Err(DomainError::Persistence(format!(
                "Erreur d'accès à l'entrée ZIP {path}: {e}"
            ))),
        }
    }
}

pub struct ZipBackupWriter {
    archive_path: PathBuf,
    document: Option<String>,
    files: IndexMap<String, Vec<u8>>,
}

impl ZipBackupWriter {
    pub fn new(archive_path: impl AsRef<Path>) -> Self {
        Self {
            archive_path: archive_path.as_ref().to_path_buf(),
            document: None,
            files: IndexMap::new(),
        }
    }
}

impl BackupWriter for ZipBackupWriter {
    fn write_document(&mut self, json: String) {
        self.document = Some(json);
    }

    fn write_file(&mut self, path: String, bytes: Vec<u8>) {
        self.files.insert(path, bytes);
    }

    fn finish(self) -> DomainResult<()> {
        let Some(json) = self.document else {
            return Ok(());
        };
        if let Some(parent) = self.archive_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                DomainError::Persistence(format!("Impossible de créer le dossier {:?}: {e}", parent))
            })?;
        }
        let file = File::create(&self.archive_path).map_err(|e| {
            DomainError::Persistence(format!(
                "Impossible de créer l'archive {:?}: {e}",
                self.archive_path
            ))
        })?;
        let mut zip = zip::ZipWriter::new(file);
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        zip.start_file(BackupExporter::DOCUMENT, options)
            .map_err(|e| DomainError::Persistence(format!("Erreur ZIP: {e}")))?;
        zip.write_all(json.as_bytes())
            .map_err(|e| DomainError::Persistence(format!("Erreur d'écriture ZIP: {e}")))?;

        for (path, bytes) in self.files {
            zip.start_file(path, options)
                .map_err(|e| DomainError::Persistence(format!("Erreur ZIP: {e}")))?;
            zip.write_all(&bytes)
                .map_err(|e| DomainError::Persistence(format!("Erreur d'écriture ZIP: {e}")))?;
        }

        zip.finish()
            .map_err(|e| DomainError::Persistence(format!("Erreur de clôture ZIP: {e}")))?;
        Ok(())
    }
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportReport {
    pub paints: usize,
    pub palettes: usize,
    pub projects: usize,
    pub recipes: usize,
    pub photos: usize,
}

pub struct BackupImporter {
    catalog: Arc<dyn PaintCatalogRepository>,
    palettes: Arc<dyn PaletteRepository>,
    projects: Arc<dyn ProjectRepository>,
    recipes: Arc<dyn RecipeRepository>,
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
        }
    }

    pub async fn import<A: BackupArchive>(&self, archive: &mut A) -> DomainResult<ImportReport> {
        let doc_bytes = archive.document()?;
        let document: BackupDocument = serde_json::from_slice(&doc_bytes).map_err(|e| {
            DomainError::InvalidArgument(format!("Document de sauvegarde JSON invalide: {e}"))
        })?;

        if document.format_version > Self::SUPPORTED_FORMAT {
            return Err(DomainError::InvalidArgument(format!(
                "Sauvegarde écrite par une version plus récente (format {}, connu jusqu'à {}).",
                document.format_version,
                Self::SUPPORTED_FORMAT
            )));
        }

        let mut warnings = Vec::new();
        let (paints_updated, paints_created) = self.restore_paints(&document.paints).await?;
        let (palettes_added, palettes_skipped) =
            self.restore_palettes(&document.palettes, &mut warnings).await?;
        let (recipes_added, recipes_skipped) = self.restore_recipes(&document.recipes).await?;
        let (projects_added, projects_skipped, photos) = self
            .restore_projects(&document.projects, archive, &mut warnings)
            .await?;

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
                    if let Some(ref tint) = entry.tint_hex
                        && let Ok(rgb) = Rgb::of_hex(tint)
                    {
                        self.catalog.record_tint(id, rgb).await?;
                    }
                    updated += 1;
                }
            } else {
                let paint = Paint {
                    id: None,
                    brand: entry.brand.clone(),
                    name: entry.name.clone(),
                    code: entry.code.clone(),
                    legacy_code: String::new(),
                    pigments: entry.pigments.iter().cloned().collect(),
                    hex_color: entry.hex.clone(),
                    tint_hex: entry.tint_hex.clone(),
                    opacity: Opacity::from_str(&entry.opacity).unwrap_or(Opacity::SemiOpaque),
                    drying_class: DryingClass::from_str(&entry.drying_class)
                        .unwrap_or(DryingClass::Medium),
                    tinting_strength: entry.tinting_strength,
                    color_derived: entry.color_derived,
                    pigments_verified: false,
                    user_added: false,
                    in_stock: entry.owned,
                    notes: entry.notes.clone(),
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
            let paints = self
                .resolve(&entry.paints, &format!("palette {}", entry.name), warnings)
                .await?;
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
            let paints = self
                .resolve(&entry.paints, &format!("projet {}", entry.name), warnings)
                .await?;
            let zones = entry
                .zones
                .iter()
                .map(|z| ProjectZone {
                    id: None,
                    name: z.name.clone(),
                    material: z.material.clone(),
                    note: z.note.clone(),
                    layers: z.layers.iter().map(Self::to_layer).collect(),
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
                        warnings.push(format!(
                            "Image annoncee mais absente de l'archive : {}",
                            photo.file
                        ));
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

    fn to_layer(layer: &BackupLayer) -> ProjectLayer {
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

        ProjectLayer {
            id: None,
            role: layer.role.clone(),
            target_hex: layer.target_hex.clone(),
            technique: layer.technique.clone(),
            note: layer.note.clone(),
            kind: LayerKind::from_str(&layer.kind).unwrap_or(LayerKind::Ladder),
            applied,
        }
    }

    async fn resolve(
        &self,
        refs: &[BackupPaintRef],
        context: &str,
        warnings: &mut Vec<String>,
    ) -> DomainResult<Vec<Paint>> {
        let mut out = Vec::new();
        for r in refs {
            match self.catalog.find_by_natural_key(&r.brand, &r.name).await? {
                Some(p) => out.push(p),
                None => warnings.push(format!(
                    "Tube introuvable pour {context} : {} - {}",
                    r.brand, r.name
                )),
            }
        }
        Ok(out)
    }
}

fn parse_iso_instant(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

fn format_iso_instant(dt: DateTime<Utc>) -> String {
    dt.to_rfc3339_opts(chrono::SecondsFormat::AutoSi, true)
}

pub struct BackupExporter {
    catalog: Arc<dyn PaintCatalogRepository>,
    palettes: Arc<dyn PaletteRepository>,
    projects: Arc<dyn ProjectRepository>,
    recipes: Arc<dyn RecipeRepository>,
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
        }
    }

    pub async fn export<W: BackupWriter>(&self, writer: &mut W) -> DomainResult<ExportReport> {
        let paints = self.catalog.all().await?;
        let all_palettes = self.palettes.all().await?;
        let all_recipes = self.recipes.all().await?;

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
        for project in &pieces {
            let mut entries = Vec::with_capacity(project.photos.len());
            for (index, photo) in project.photos.iter().enumerate() {
                let path = format!(
                    "{}/{}-{:02}.jpg",
                    Self::IMAGES,
                    Self::slug(&project.name),
                    index + 1
                );
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
            exported_at: Some(format_iso_instant(Utc::now())),
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
                            technique: l.technique.clone(),
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
            .map(|c| {
                if c.is_ascii_alphanumeric() {
                    c
                } else {
                    '-'
                }
            })
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
