//! The painting plan assistant: which tubes it mixes from, the plan, and the piece
//! made from it.

use crate::state::AppServices;
use chrono::Utc;
use palettier_ai::SampledZone;
use palettier_domain::error::DomainResult;
use palettier_domain::i18n::Locale;
use palettier_domain::paint::Paint;
use palettier_domain::palette::Palette;
use palettier_domain::plan::{PaintingPlan, unique_name};
use palettier_domain::port::ProjectRepository;
use palettier_domain::project::{PhotoRole, Project, ProjectPhoto};

/// The tubes in stock, or every tube when none is.
pub fn stock_or_all(paints: &[Paint]) -> Vec<Paint> {
    let owned: Vec<_> = paints.iter().filter(|p| p.in_stock).cloned().collect();
    if owned.is_empty() { paints.to_vec() } else { owned }
}

/// The name and the tubes the plan is mixed from: the chosen palette, else the studio
/// stock (« Tous mes tubes en stock »).
pub fn plan_paints(
    palettes: &[Palette],
    palette_idx: usize,
    paints: &[Paint],
    locale: Locale,
) -> (String, Vec<Paint>) {
    match palettes.get(palette_idx) {
        Some(pal) => (pal.name.clone(), pal.paints.clone()),
        None => (
            locale.tr("Stock atelier", "Studio Stock").to_string(),
            stock_or_all(paints),
        ),
    }
}

/// What the painter gave the assistant.
#[derive(Debug, Clone)]
pub struct PlanRequest {
    pub image: Option<Vec<u8>>,
    pub description: String,
    pub palette_name: String,
    pub paints: Vec<Paint>,
    pub sampled: Vec<SampledZone>,
}

/// Asks the AI engine for a plan; the Kubelka-Munk solver doses each step.
pub async fn generate_plan(
    services: &AppServices,
    request: PlanRequest,
    locale: Locale,
) -> DomainResult<PaintingPlan> {
    services
        .plan_service
        .plan_i18n(
            request.image.as_deref(),
            &request.description,
            &request.palette_name,
            &request.paints,
            &request.sampled,
            locale,
        )
        .await
}

/// Saves a plan as a piece under a name no other piece uses. Without a palette the piece
/// takes `fallback_paints`; an attached photo joins its journal (a failure there is
/// ignored). Returns the saved piece and, when they could be read, every piece.
pub async fn create_project_from_plan(
    services: &AppServices,
    plan: &PaintingPlan,
    palette: Option<&Palette>,
    fallback_paints: Vec<Paint>,
    taken_names: &[String],
    photo: Option<Vec<u8>>,
    locale: Locale,
) -> DomainResult<(Project, Option<Vec<Project>>)> {
    let name = unique_name(&plan.subject, taken_names.iter().map(String::as_str));
    let mut project = plan.as_project(name, palette);
    if project.paints.is_empty() {
        project.paints = fallback_paints;
    }
    let saved = services.projects.save(&project).await?;
    if let (Some(pid), Some(bytes)) = (saved.id, photo) {
        let photo = ProjectPhoto {
            id: None,
            data: bytes,
            role: PhotoRole::Piece,
            caption: locale.tr("Photo de référence", "Reference photo").to_string(),
            added_at: Utc::now(),
        };
        let _ = services.projects.add_photo(pid, &photo).await;
    }
    let projects = services.projects.all().await.ok();
    Ok((saved, projects))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::eyedropper::{add_group, add_sample};
    use crate::actions::mixer::{palettes_of_system, paints_of_system};
    use palettier_domain::image::PixelMap;
    use palettier_domain::paint::PaintSystem;
    use palettier_domain::port::{PaintCatalogRepository, PaletteRepository};

    /// A plan made without the AI engine, from sampled groups.
    fn plan(svc: &AppServices, palette: Option<&Palette>) -> PaintingPlan {
        let groups = add_group(&[], "Manteau", "Drap").unwrap();
        let groups = add_sample(&groups, 0, "#2B303A").unwrap();
        let groups = add_sample(&groups, 0, "#5E687D").unwrap();
        svc.sampled_plan_service.plan("Grognard", palette, &groups, 3, Locale::Fr).unwrap()
    }

    #[tokio::test]
    async fn the_plan_mixes_from_the_palette_else_the_stock() {
        let svc = AppServices::in_memory().await.unwrap();
        let paints = paints_of_system(&svc.catalog.all().await.unwrap(), PaintSystem::Acrylic);
        let palettes = palettes_of_system(&svc.palettes.all().await.unwrap(), PaintSystem::Acrylic);

        let (name, from_palette) = plan_paints(&palettes, 0, &paints, Locale::Fr);
        assert_eq!(name, palettes[0].name);
        assert_eq!(from_palette.len(), palettes[0].paints.len());

        let (name, stock) = plan_paints(&palettes, 9999, &paints, Locale::En);
        assert_eq!(name, "Studio Stock");
        assert_eq!(stock, stock_or_all(&paints));
        assert!(stock.iter().all(|p| p.system == PaintSystem::Acrylic));

        let none_in_stock: Vec<Paint> = paints.iter().cloned().map(|mut p| {
            p.in_stock = false;
            p
        }).collect();
        assert_eq!(stock_or_all(&none_in_stock).len(), paints.len());
    }

    #[tokio::test]
    async fn an_empty_request_asks_for_something_to_plan() {
        let svc = AppServices::in_memory().await.unwrap();
        let request = PlanRequest {
            image: None,
            description: "  ".into(),
            palette_name: "Stock".into(),
            paints: Vec::new(),
            sampled: Vec::new(),
        };
        assert!(generate_plan(&svc, request, Locale::Fr).await.is_err());
    }

    #[tokio::test]
    async fn saves_a_plan_as_a_project_with_its_photo() {
        let svc = AppServices::in_memory().await.unwrap();
        let palette = svc.palettes.all().await.unwrap().into_iter().find(|p| !p.paints.is_empty());
        let plan = plan(&svc, palette.as_ref());
        let pixels = PixelMap::new(2, 2, vec![0xFF8C_6248; 4]).unwrap();
        let photo = Some(svc.decoder.encode_png(&pixels).unwrap());

        let (saved, projects) =
            create_project_from_plan(&svc, &plan, palette.as_ref(), Vec::new(), &[], photo, Locale::Fr)
                .await
                .unwrap();
        assert_eq!(saved.name, "Grognard");
        assert_eq!(saved.zones[0].name, "Manteau");
        assert_eq!(saved.paints.len(), palette.unwrap().paints.len());
        assert!(projects.unwrap().iter().any(|p| p.id == saved.id));
        let with_photos = svc.projects.find_with_photos(saved.id.unwrap()).await.unwrap().unwrap();
        assert_eq!(with_photos.photos.len(), 1);
        assert_eq!(with_photos.photos[0].caption, "Photo de référence");
        assert_eq!(with_photos.photos[0].role, PhotoRole::Piece);
    }

    #[tokio::test]
    async fn without_a_palette_the_project_takes_the_fallback_tubes_and_a_free_name() {
        let svc = AppServices::in_memory().await.unwrap();
        let paints = svc.catalog.all().await.unwrap();
        let plan = plan(&svc, None);
        let fallback = paints[..3].to_vec();

        let (saved, _) = create_project_from_plan(
            &svc,
            &plan,
            None,
            fallback.clone(),
            &["grognard".to_string()],
            None,
            Locale::En,
        )
        .await
        .unwrap();
        assert_ne!(saved.name.to_lowercase(), "grognard");
        assert_eq!(saved.paints.len(), fallback.len());
    }
}
