//! Pieces: creating them, editing their zones and coats, their tubes and photo journal.

use crate::state::AppServices;
use chrono::{DateTime, Utc};
use palettier_domain::drying::Workshop;
use palettier_domain::error::DomainResult;
use palettier_domain::i18n::Locale;
use palettier_domain::paint::{DryingClass, Paint, PaintSystem, Technique};
use palettier_domain::palette::Palette;
use palettier_domain::port::ProjectRepository;
use palettier_domain::project::{
    LayerKind, ObservedColour, PhotoRole, Project, ProjectLayer, ProjectPhoto, ProjectZone,
};

/// The value ladder a new piece starts with on its first zone (the face).
pub const FACE_LADDER: [&str; 5] = ["#4A2E24", "#7D5140", "#B87E63", "#D9A68B", "#F2CFB8"];
/// The value ladder a zone added by hand starts with.
pub const ZONE_LADDER: [&str; 5] = ["#2B303A", "#434A59", "#5E687D", "#8591A8", "#B2BDD1"];

/// A piece after a change: the piece itself, read back with its photos, and every piece
/// for the lists.
#[derive(Debug, Clone)]
pub struct ProjectChange {
    pub project: Option<Project>,
    pub projects: Vec<Project>,
}

/// What the painter typed in the « Nouveau projet » card.
#[derive(Debug, Clone)]
pub struct NewProject {
    pub name: String,
    pub subject: String,
    pub system: PaintSystem,
    pub palette: Option<Palette>,
}

/// The fields of the inline coat editor.
#[derive(Debug, Clone, PartialEq)]
pub struct LayerEdit {
    pub role: String,
    pub target_hex: String,
    pub technique: Technique,
    pub note: String,
}

/// A piece with its photos.
pub async fn load_project(services: &AppServices, id: i64) -> DomainResult<Option<Project>> {
    services.projects.find_with_photos(id).await
}

/// Builds a new piece with a face zone and its starter ladder, then saves it.
/// Returns `None` when the name is blank (nothing is saved).
pub async fn create_project(
    services: &AppServices,
    draft: NewProject,
    locale: Locale,
) -> DomainResult<Option<ProjectChange>> {
    let name = draft.name.trim().to_string();
    if name.is_empty() {
        return Ok(None);
    }
    let paints = draft
        .palette
        .as_ref()
        .map(|p| p.paints.clone())
        .unwrap_or_default();
    let project = Project {
        id: None,
        name,
        subject: draft.subject.trim().to_string(),
        approach: String::new(),
        notes: String::new(),
        system: draft.system,
        palette: draft.palette,
        paints,
        zones: vec![ProjectZone {
            id: None,
            name: locale.tr("Visage", "Face").to_string(),
            material: locale.tr("Peau", "Skin").to_string(),
            note: String::new(),
            layers: starter_ladder(locale, FACE_LADDER, draft.system),
        }],
        photos: Vec::new(),
        created_at: Utc::now(),
    };
    let saved = services.projects.save(&project).await?;
    Ok(Some(ProjectChange {
        projects: services.projects.all().await?,
        project: Some(saved),
    }))
}

/// Saves a piece, then reads it back with its photos.
pub async fn save_project(services: &AppServices, project: &Project) -> DomainResult<ProjectChange> {
    let saved = services.projects.save(project).await?;
    let projects = services.projects.all().await?;
    let project = match saved.id {
        Some(id) => services.projects.find_with_photos(id).await?,
        None => None,
    };
    Ok(ProjectChange { project, projects })
}

/// Deletes a piece and returns the pieces left.
pub async fn delete_project(services: &AppServices, id: i64) -> DomainResult<Vec<Project>> {
    services.projects.delete(id).await?;
    services.projects.all().await
}

/// Adds an accent coat at the end of a zone, in the piece's default technique.
pub async fn add_accent_layer(
    services: &AppServices,
    project: &Project,
    zone_idx: usize,
) -> DomainResult<ProjectChange> {
    let mut next = project.clone();
    let system = next.system;
    if let Some(zone) = next.zones.get_mut(zone_idx) {
        let mut layer = ProjectLayer::new("Accent", "#C89B6E");
        layer.kind = LayerKind::Accent;
        layer.technique = Technique::default_for(system);
        zone.layers.push(layer);
    }
    save_project(services, &next).await
}

/// Applies the inline editor's fields to one coat.
pub async fn edit_layer(
    services: &AppServices,
    project: &Project,
    zone_idx: usize,
    layer_idx: usize,
    edit: LayerEdit,
) -> DomainResult<ProjectChange> {
    let mut next = project.clone();
    if let Some(layer) = layer_mut(&mut next, zone_idx, layer_idx) {
        layer.role = edit.role.trim().to_string();
        layer.target_hex = edit.target_hex.trim().to_string();
        layer.technique = edit.technique;
        layer.note = edit.note.trim().to_string();
    }
    save_project(services, &next).await
}

/// The piece with one coat marked as laid (at `now`, in the studio's climate) or, if it
/// already was, back to pending. The drying pace is that of the piece's slowest tube.
pub fn toggle_layer_applied(
    project: &Project,
    zone_idx: usize,
    layer_idx: usize,
    workshop: Workshop,
    now: DateTime<Utc>,
) -> Project {
    let mut next = project.clone();
    let slowest = next
        .effective_paints()
        .iter()
        .fold(DryingClass::Medium, |acc, pt| acc.slowest(pt.drying_class));
    if let Some(layer) = layer_mut(&mut next, zone_idx, layer_idx) {
        *layer = if layer.is_applied() {
            layer.clear_applied()
        } else {
            layer.mark_applied(now, workshop, slowest)
        };
    }
    next
}

/// Marks a coat as laid now (or pending again) and saves the piece.
pub async fn mark_layer_applied(
    services: &AppServices,
    project: &Project,
    zone_idx: usize,
    layer_idx: usize,
    workshop: Workshop,
) -> DomainResult<ProjectChange> {
    let next = toggle_layer_applied(project, zone_idx, layer_idx, workshop, Utc::now());
    save_project(services, &next).await
}

/// Adds a zone with its starter ladder. Returns `None` when the name is blank.
pub async fn add_zone(
    services: &AppServices,
    project: &Project,
    name: &str,
    material: &str,
    locale: Locale,
) -> DomainResult<Option<ProjectChange>> {
    let name = name.trim();
    if name.is_empty() {
        return Ok(None);
    }
    let mut next = project.clone();
    next.zones.push(ProjectZone {
        id: None,
        name: name.to_string(),
        material: material.trim().to_string(),
        note: String::new(),
        layers: starter_ladder(locale, ZONE_LADDER, next.system),
    });
    save_project(services, &next).await.map(Some)
}

/// Takes the tubes of the piece's palette as it is now.
pub async fn sync_project_paints(
    services: &AppServices,
    project: &Project,
) -> DomainResult<ProjectChange> {
    let mut next = project.clone();
    if let Some(pal) = &next.palette {
        next.paints = pal.paints.clone();
    }
    save_project(services, &next).await
}

/// Replaces an out-of-stock tube of the piece by one on the shelf.
pub async fn substitute_project_paint(
    services: &AppServices,
    project: &Project,
    absent_id: Option<i64>,
    replacement: Paint,
) -> DomainResult<ProjectChange> {
    let mut next = project.clone();
    if next.paints.is_empty()
        && let Some(pal) = &next.palette
    {
        next.paints = pal.paints.clone();
    }
    for pt in &mut next.paints {
        if pt.id == absent_id {
            *pt = replacement.clone();
        }
    }
    save_project(services, &next).await
}

/// Adds a photo to the piece's journal and returns the piece with its photos.
pub async fn add_photo(
    services: &AppServices,
    project_id: i64,
    data: Vec<u8>,
    role: PhotoRole,
    caption: String,
) -> DomainResult<Project> {
    let photo = ProjectPhoto {
        id: None,
        data,
        role,
        caption,
        added_at: Utc::now(),
    };
    services.projects.add_photo(project_id, &photo).await
}

/// Removes a photo from the piece's journal.
pub async fn remove_photo(
    services: &AppServices,
    project_id: i64,
    photo_id: i64,
) -> DomainResult<Project> {
    services.projects.remove_photo(project_id, photo_id).await
}

/// Compares the six dominant colours of a photo of the piece with its planned coats.
pub fn compare_to_plan(
    services: &AppServices,
    project: &Project,
    photo: &[u8],
) -> DomainResult<Vec<ObservedColour>> {
    let dominant = services.decoder.dominant_colours(photo, 6)?;
    Ok(services.progress_service.compare(project, &dominant))
}

/// The photo that best shows the piece: the piece itself, else the reference, else the first.
pub fn showcase_photo(project: &Project) -> Option<&ProjectPhoto> {
    project
        .photos
        .iter()
        .find(|ph| ph.role == PhotoRole::Piece || ph.role == PhotoRole::Reference)
        .or_else(|| project.photos.first())
}

fn layer_mut(project: &mut Project, zone_idx: usize, layer_idx: usize) -> Option<&mut ProjectLayer> {
    project.zones.get_mut(zone_idx)?.layers.get_mut(layer_idx)
}

/// The five coats a new zone starts with, from the deepest shadow to the brightest light,
/// each with the technique that suits its place in the ladder and the piece's system.
pub fn starter_ladder(locale: Locale, hexes: [&str; 5], system: PaintSystem) -> Vec<ProjectLayer> {
    let roles = [
        locale.tr("Ombre 2", "Shadow 2"),
        locale.tr("Ombre 1", "Shadow 1"),
        "Base",
        locale.tr("Lumière 1", "Highlight 1"),
        locale.tr("Lumière 2", "Highlight 2"),
    ];
    let techniques = match system {
        PaintSystem::Oil => [
            Technique::Glaze,
            Technique::Glaze,
            Technique::BaseLayer,
            Technique::Highlight,
            Technique::Highlight,
        ],
        PaintSystem::Acrylic => [
            Technique::ShadeWash,
            Technique::Layering,
            Technique::BaseLayer,
            Technique::Layering,
            Technique::EdgeHighlight,
        ],
    };
    roles
        .into_iter()
        .zip(hexes)
        .zip(techniques)
        .map(|((role, hex), technique)| ProjectLayer::new(role, hex).with_technique(technique))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use palettier_domain::port::PaletteRepository;

    fn draft(name: &str, system: PaintSystem, palette: Option<Palette>) -> NewProject {
        NewProject {
            name: name.into(),
            subject: " Buste 1/10 ".into(),
            system,
            palette,
        }
    }

    #[tokio::test]
    async fn creates_an_acrylic_project_with_acrylic_techniques() {
        let svc = AppServices::in_memory().await.unwrap();
        let palette = svc
            .palettes
            .all()
            .await
            .unwrap()
            .into_iter()
            .find(|p| !p.paints.is_empty() && p.system() == PaintSystem::Acrylic);

        let change = create_project(&svc, draft("  Grognard  ", PaintSystem::Acrylic, palette.clone()), Locale::Fr)
            .await
            .unwrap()
            .expect("a named project is saved");

        let saved = change.project.unwrap();
        assert!(saved.id.is_some());
        assert_eq!(saved.name, "Grognard");
        assert_eq!(saved.subject, "Buste 1/10");
        assert_eq!(saved.system, PaintSystem::Acrylic);
        assert_eq!(saved.zones.len(), 1);
        assert_eq!(saved.zones[0].name, "Visage");
        let techniques: Vec<_> = saved.zones[0].layers.iter().map(|l| l.technique).collect();
        assert!(techniques.iter().all(|t| t.native_system() != Some(PaintSystem::Oil)));
        assert_eq!(techniques[0], Technique::ShadeWash);
        if let Some(pal) = palette {
            assert_eq!(saved.paints.len(), pal.paints.len());
        }
        assert!(change.projects.iter().any(|p| p.id == saved.id));
    }

    #[tokio::test]
    async fn a_blank_name_saves_nothing() {
        let svc = AppServices::in_memory().await.unwrap();
        let before = svc.projects.all().await.unwrap().len();
        let change = create_project(&svc, draft("   ", PaintSystem::Oil, None), Locale::Fr)
            .await
            .unwrap();
        assert!(change.is_none());
        assert_eq!(svc.projects.all().await.unwrap().len(), before);
    }

    #[tokio::test]
    async fn laying_a_coat_records_the_climate_and_can_be_undone() {
        let svc = AppServices::in_memory().await.unwrap();
        let project = create_project(&svc, draft("Hussard", PaintSystem::Oil, None), Locale::Fr)
            .await
            .unwrap()
            .unwrap()
            .project
            .unwrap();
        let warm = Workshop::new(26.0, 40.0, Workshop::default().ventilation).unwrap();

        let laid = mark_layer_applied(&svc, &project, 0, 2, warm).await.unwrap().project.unwrap();
        let applied = laid.zones[0].layers[2].applied.as_ref().expect("coat laid");
        assert_eq!(applied.workshop.temperature_celsius, 26.0);
        assert!(!laid.zones[0].layers[1].is_applied());

        let undone = mark_layer_applied(&svc, &laid, 0, 2, warm).await.unwrap().project.unwrap();
        assert!(!undone.zones[0].layers[2].is_applied());
    }

    #[tokio::test]
    async fn an_acrylic_coat_over_an_oil_coat_is_flagged() {
        let svc = AppServices::in_memory().await.unwrap();
        let project = create_project(&svc, draft("Cuirassier", PaintSystem::Acrylic, None), Locale::Fr)
            .await
            .unwrap()
            .unwrap()
            .project
            .unwrap();
        assert!(svc.fat_over_lean.inspect_project(&project).is_empty());

        // An oil glaze on the base, then an acrylic layer above it.
        let oil = Technique::ALL
            .into_iter()
            .find(|t| t.native_system() == Some(PaintSystem::Oil))
            .unwrap();
        let edit = LayerEdit {
            role: "Base".into(),
            target_hex: "#B87E63".into(),
            technique: oil,
            note: String::new(),
        };
        let edited = edit_layer(&svc, &project, 0, 2, edit).await.unwrap().project.unwrap();
        assert_eq!(edited.zones[0].layers[2].technique, oil);

        let risks = svc.fat_over_lean.inspect_project(&edited);
        assert_eq!(risks.len(), 1, "{risks:?}");
        assert!(risks[0].explanation.contains("acrylique"));
        assert_eq!(risks[0].over, edited.zones[0].layers[3].role);
    }

    #[tokio::test]
    async fn adds_a_zone_and_an_accent() {
        let svc = AppServices::in_memory().await.unwrap();
        let project = create_project(&svc, draft("Dragon", PaintSystem::Oil, None), Locale::En)
            .await
            .unwrap()
            .unwrap()
            .project
            .unwrap();
        assert!(add_zone(&svc, &project, "  ", "", Locale::En).await.unwrap().is_none());

        let with_zone = add_zone(&svc, &project, " Wings ", " Leather ", Locale::En)
            .await
            .unwrap()
            .unwrap()
            .project
            .unwrap();
        assert_eq!(with_zone.zones.len(), 2);
        assert_eq!(with_zone.zones[1].name, "Wings");
        assert_eq!(with_zone.zones[1].material, "Leather");
        assert_eq!(with_zone.zones[1].layers[0].role, "Shadow 2");

        let with_accent = add_accent_layer(&svc, &with_zone, 1).await.unwrap().project.unwrap();
        let accent = with_accent.zones[1].layers.last().unwrap();
        assert_eq!(accent.kind, LayerKind::Accent);
        assert_eq!(accent.technique, Technique::default_for(PaintSystem::Oil));
    }

    #[tokio::test]
    async fn deleting_a_project_returns_the_others() {
        let svc = AppServices::in_memory().await.unwrap();
        let a = create_project(&svc, draft("A", PaintSystem::Oil, None), Locale::Fr)
            .await
            .unwrap()
            .unwrap()
            .project
            .unwrap();
        let left = delete_project(&svc, a.id.unwrap()).await.unwrap();
        assert!(left.iter().all(|p| p.id != a.id));
    }
}
