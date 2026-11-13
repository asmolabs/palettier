//! The eyedropper: reading a reference photo, sampling it, and building a measured
//! project from groups of samples. The HUD and the assistant share the photo helpers.

use crate::state::AppServices;
use base64::Engine;
use palettier_domain::color::{Rgb, neutralise};
use palettier_domain::error::DomainResult;
use palettier_domain::i18n::Locale;
use palettier_domain::image::{PixelMap, StudyMode};
use palettier_domain::palette::Palette;
use palettier_domain::plan::{ColourGroup, unique_name};
use palettier_domain::port::ProjectRepository;
use palettier_domain::project::{DominantColour, Project};

/// A reference photo ready to show and to sample.
#[derive(Debug, Clone)]
pub struct ReferencePhoto {
    pub pixels: PixelMap,
    /// The photo scaled down to 1 400 px, as a JPEG `data:` URI.
    pub data_uri: String,
}

/// Decodes a reference photo. `None` when the bytes are not an image.
pub fn open_reference(services: &AppServices, bytes: &[u8]) -> Option<ReferencePhoto> {
    let pixels = services.decoder.decode(bytes).ok()?;
    let scaled = services
        .decoder
        .scale_to(bytes, 1400)
        .unwrap_or_else(|_| bytes.to_vec());
    Some(ReferencePhoto {
        pixels,
        data_uri: jpeg_data_uri(&scaled),
    })
}

/// A photo scaled down to `max_edge` px, as a JPEG `data:` URI.
pub fn scaled_data_uri(services: &AppServices, bytes: Vec<u8>, max_edge: u32) -> String {
    let scaled = services.decoder.scale_to(&bytes, max_edge).unwrap_or(bytes);
    jpeg_data_uri(&scaled)
}

fn jpeg_data_uri(bytes: &[u8]) -> String {
    let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
    format!("data:image/jpeg;base64,{b64}")
}

/// The photo seen through a value study (grisaille, notan…), as a PNG `data:` URI.
pub fn study_data_uri(services: &AppServices, pixels: &PixelMap, mode: StudyMode) -> Option<String> {
    let png = services.decoder.encode_png(&pixels.to_study_map(mode)).ok()?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&png);
    Some(format!("data:image/png;base64,{b64}"))
}

/// The eight dominant colours of a photo.
pub fn dominants(services: &AppServices, bytes: &[u8]) -> Option<Vec<DominantColour>> {
    services.decoder.dominant_colours(bytes, 8).ok()
}

/// A click on the photo shown at `zoom_pct` %: the pixel hit and the colour averaged
/// over `radius` px around it.
pub fn sample_click(
    pixels: &PixelMap,
    x: f64,
    y: f64,
    zoom_pct: u32,
    radius: isize,
) -> Option<((isize, isize), Rgb)> {
    let scale = (zoom_pct as f64 / 100.0).max(0.05);
    let cx = (x / scale).floor() as isize;
    let cy = (y / scale).floor() as isize;
    pixels.sample_around(cx, cy, radius).map(|rgb| ((cx, cy), rgb))
}

/// A colour corrected by the neutral grey point (unchanged when the grey is not a colour).
pub fn neutralised(raw: Rgb, grey_hex: &str) -> Rgb {
    Rgb::of_hex(grey_hex).map(|g| neutralise(raw, g)).unwrap_or(raw)
}

/// The groups with a new, empty one at the end. `None` when the name is blank.
pub fn add_group(groups: &[ColourGroup], name: &str, material: &str) -> Option<Vec<ColourGroup>> {
    let name = name.trim();
    if name.is_empty() {
        return None;
    }
    let mut list = groups.to_vec();
    list.push(ColourGroup {
        name: name.to_string(),
        material: material.trim().to_string(),
        samples: Vec::new(),
    });
    Some(list)
}

/// The groups with a sample added to one. `None` when the colour is not a hex colour.
pub fn add_sample(groups: &[ColourGroup], idx: usize, hex: &str) -> Option<Vec<ColourGroup>> {
    let rgb = Rgb::of_hex(hex).ok()?;
    let mut list = groups.to_vec();
    if let Some(g) = list.get_mut(idx) {
        g.samples.push(rgb);
    }
    Some(list)
}

/// The steps of a group, from its darkest sample to its lightest: (« #1 », hex).
pub fn group_ladder(group: &ColourGroup) -> Vec<(String, String)> {
    let mut sorted = group.samples.clone();
    sorted.sort_by(|a, b| a.relative_luminance().total_cmp(&b.relative_luminance()));
    sorted
        .iter()
        .enumerate()
        .map(|(i, s)| (format!("#{}", i + 1), s.to_hex()))
        .collect()
}

/// Plans a measured project from the sampled groups (3 tubes at most per step), then
/// saves it under a name no other piece uses. A blank name becomes « Projet relevé ».
///
/// `Err` when there is nothing to plan; `Ok(None)` when the piece could not be saved.
pub async fn create_project_from_groups(
    services: &AppServices,
    name: &str,
    palette: Option<&Palette>,
    groups: &[ColourGroup],
    taken_names: &[String],
    locale: Locale,
) -> DomainResult<Option<Project>> {
    let name = name.trim();
    let subject = if name.is_empty() {
        locale.tr("Projet relevé", "Sampled Project")
    } else {
        name
    };
    let plan = services.sampled_plan_service.plan(subject, palette, groups, 3, locale)?;
    let unique = unique_name(&plan.subject, taken_names.iter().map(String::as_str));
    let project = plan.as_project(unique, palette);
    Ok(services.projects.save(&project).await.ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use palettier_domain::port::PaletteRepository;

    fn sampled_groups() -> Vec<ColourGroup> {
        let groups = add_group(&[], " Cape ", " Drap ").unwrap();
        let groups = add_sample(&groups, 0, "#D9A68B").unwrap();
        let groups = add_sample(&groups, 0, "#4A2E24").unwrap();
        add_sample(&groups, 0, "#8C6248").unwrap()
    }

    #[test]
    fn groups_take_samples_and_sort_them_by_value() {
        assert!(add_group(&[], "  ", "Drap").is_none());
        let groups = sampled_groups();
        assert_eq!(groups[0].name, "Cape");
        assert_eq!(groups[0].material, "Drap");
        assert!(add_sample(&groups, 0, "pas une couleur").is_none());

        let ladder = group_ladder(&groups[0]);
        let hexes: Vec<_> = ladder.iter().map(|(_, h)| h.as_str()).collect();
        assert_eq!(hexes, ["#4A2E24", "#8C6248", "#D9A68B"]);
        assert_eq!(ladder[0].0, "#1");
    }

    #[test]
    fn a_neutral_grey_leaves_the_colour_alone() {
        let raw = Rgb::of_hex("#8C6248").unwrap();
        assert_eq!(neutralised(raw, "pas une couleur"), raw);
        assert_ne!(neutralised(raw, "#A08060").to_hex(), raw.to_hex());
    }

    #[tokio::test]
    async fn creates_a_measured_project_from_sampled_groups() {
        let svc = AppServices::in_memory().await.unwrap();
        let palette = svc.palettes.all().await.unwrap().into_iter().find(|p| !p.paints.is_empty());
        let groups = sampled_groups();

        let saved = create_project_from_groups(&svc, "  ", palette.as_ref(), &groups, &[], Locale::Fr)
            .await
            .unwrap()
            .expect("the project is saved");
        assert!(saved.id.is_some());
        assert_eq!(saved.name, "Projet relevé");
        assert_eq!(saved.zones.len(), 1);
        assert_eq!(saved.zones[0].name, "Cape");
        assert!(!saved.zones[0].layers.is_empty());

        let taken = vec![saved.name.clone()];
        let again = create_project_from_groups(&svc, "Projet relevé", palette.as_ref(), &groups, &taken, Locale::Fr)
            .await
            .unwrap()
            .unwrap();
        assert_ne!(again.name, saved.name);
    }

    #[tokio::test]
    async fn groups_without_samples_cannot_make_a_project() {
        let svc = AppServices::in_memory().await.unwrap();
        let empty = add_group(&[], "Cape", "").unwrap();
        assert!(create_project_from_groups(&svc, "X", None, &empty, &[], Locale::En).await.is_err());
    }
}
