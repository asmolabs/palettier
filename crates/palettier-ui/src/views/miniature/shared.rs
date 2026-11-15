//! What several miniature tools share: picking the candidate tubes, the closest mix of a
//! target, the link to the Mixer and the clipboard.

use crate::state::{AppServices, AppState, Section};
use crate::theme::ColorSwatch;
use dioxus::prelude::*;
use palettier_domain::color::Rgb;
use palettier_domain::mix::MixSuggestion;
use palettier_domain::paint::{Paint, PaintSystem};
use palettier_domain::palette::Palette;

/// The palettes the mixes are computed from: those of the working system (and the empty
/// ones, which belong to no system yet).
pub(super) fn working_palettes(all: &[Palette], system: PaintSystem) -> Vec<Palette> {
    all.iter()
        .filter(|p| p.paints.is_empty() || p.system() == system)
        .cloned()
        .collect()
}

/// The tubes of the chosen palette; past the last palette, the in-stock tubes of the
/// working system (`system_paints`).
pub(super) fn active_candidates(palettes: &[Palette], idx: usize, system_paints: &[Paint]) -> Vec<Paint> {
    match palettes.get(idx) {
        Some(palette) => palette.paints.clone(),
        None => system_paints.iter().filter(|p| p.in_stock).cloned().collect(),
    }
}

/// The tubes an acrylic is matched with: always oil. In oil, the active candidates; in
/// acrylic, every in-stock oil tube of the catalog.
pub(super) fn oil_candidates(system: PaintSystem, active: &[Paint], catalog: &[Paint]) -> Vec<Paint> {
    if system == PaintSystem::Oil {
        active.to_vec()
    } else {
        catalog
            .iter()
            .filter(|p| p.system == PaintSystem::Oil && p.in_stock)
            .cloned()
            .collect()
    }
}

/// The acrylic pots of the catalog, sorted by brand then name.
pub(super) fn acrylic_catalog(catalog: &[Paint]) -> Vec<Paint> {
    let mut all: Vec<Paint> = catalog
        .iter()
        .filter(|p| p.system == PaintSystem::Acrylic)
        .cloned()
        .collect();
    all.sort_by(|a, b| a.brand.cmp(&b.brand).then(a.name.cmp(&b.name)));
    all
}

/// Stable key of a catalog paint in a select: brand and name are unique together.
pub(super) fn paint_key(p: &Paint) -> String {
    format!("{}|{}", p.brand, p.name)
}

/// The paint whose key is `key`, or the first one when nothing (valid) is selected.
pub(super) fn pick(paints: &[Paint], key: Option<&str>) -> Option<Paint> {
    key.and_then(|k| paints.iter().find(|p| paint_key(p) == k))
        .or_else(|| paints.first())
        .cloned()
}

/// The colour of `hex`, or a grey of value `grey` when it does not parse.
pub(super) fn rgb_or_grey(hex: &str, grey: f64) -> Rgb {
    Rgb::of_hex(hex).unwrap_or_else(|_| Rgb::new(grey, grey, grey))
}

/// The closest mix of `target` from `candidates`, with at most three tubes.
pub(super) fn best_mix(services: &AppServices, target: Rgb, candidates: &[Paint]) -> Option<MixSuggestion> {
    services
        .mix_service
        .suggest_mixes(target, candidates, 1, Some(3))
        .into_iter()
        .next()
}

/// Opens the Mixer on `hex`.
pub(super) fn open_in_mixer(mut state: AppState, hex: String) {
    state.mixer_target_hex.set(hex);
    state.section.set(Section::Mixer);
}

/// A choice in a list of colours (metal, effect, light source): its swatch, label and
/// hex; clicking it selects `value`.
pub(super) fn swatch_option<T: Copy + PartialEq + 'static>(
    mut selected: Signal<T>,
    value: T,
    hex: &str,
    label: &str,
) -> Element {
    let is_sel = *selected.read() == value;
    rsx! {
        div {
            class: if is_sel { "card-subtle active" } else { "card-subtle" },
            style: if is_sel { "border-color: var(--accent); cursor: pointer; padding: 10px;" } else { "cursor: pointer; padding: 10px;" },
            onclick: move |_| selected.set(value),
            div { class: "row u-gap-10 u-ai-center",
                ColorSwatch { hex: hex.to_string(), size: 28 }
                div {
                    div { class: "u-fw-700 u-fs-13", "{label}" }
                    div { class: "mono u-fs-11 u-op-0p7", "{hex}" }
                }
            }
        }
    }
}

pub(super) fn copy_to_clipboard(text: &str) {
    use std::io::Write;
    if let Ok(mut child) = std::process::Command::new("pbcopy")
        .stdin(std::process::Stdio::piped())
        .spawn()
    {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(text.as_bytes());
        }
        let _ = child.wait();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paint(brand: &str, name: &str, system: PaintSystem, in_stock: bool) -> Paint {
        let mut p = Paint::new(brand, name, "#808080").in_system(system);
        p.in_stock = in_stock;
        p
    }

    #[test]
    fn past_the_last_palette_the_in_stock_drawer_is_used() {
        let mut palette = Palette::new("Portrait");
        palette.paints = vec![paint("W&N", "Ivory Black", PaintSystem::Oil, false)];
        let drawer = vec![
            paint("W&N", "Titanium White", PaintSystem::Oil, true),
            paint("W&N", "Raw Umber", PaintSystem::Oil, false),
        ];
        let palettes = vec![palette];
        assert_eq!("Ivory Black", active_candidates(&palettes, 0, &drawer)[0].name);
        let fallback = active_candidates(&palettes, 1, &drawer);
        assert_eq!(1, fallback.len());
        assert_eq!("Titanium White", fallback[0].name);
    }

    #[test]
    fn acrylics_are_matched_with_in_stock_oils_only() {
        let catalog = vec![
            paint("Citadel", "Mephiston Red", PaintSystem::Acrylic, true),
            paint("W&N", "Cadmium Red", PaintSystem::Oil, true),
            paint("W&N", "Alizarin", PaintSystem::Oil, false),
        ];
        let active = vec![catalog[0].clone()];
        let oils = oil_candidates(PaintSystem::Acrylic, &active, &catalog);
        assert_eq!(vec!["Cadmium Red"], oils.iter().map(|p| p.name.as_str()).collect::<Vec<_>>());
        assert_eq!(active, oil_candidates(PaintSystem::Oil, &active, &catalog));
    }

    #[test]
    fn acrylic_catalog_is_sorted_by_brand_then_name() {
        let catalog = vec![
            paint("Vallejo", "Flat Red", PaintSystem::Acrylic, true),
            paint("Citadel", "Mephiston Red", PaintSystem::Acrylic, true),
            paint("Citadel", "Abaddon Black", PaintSystem::Acrylic, true),
            paint("W&N", "Cadmium Red", PaintSystem::Oil, true),
        ];
        let names: Vec<String> = acrylic_catalog(&catalog).into_iter().map(|p| p.name).collect();
        assert_eq!(vec!["Abaddon Black", "Mephiston Red", "Flat Red"], names);
    }

    #[test]
    fn pick_falls_back_to_the_first_paint() {
        let paints = vec![
            paint("W&N", "Ivory Black", PaintSystem::Oil, true),
            paint("W&N", "Raw Umber", PaintSystem::Oil, true),
        ];
        assert_eq!("Raw Umber", pick(&paints, Some("W&N|Raw Umber")).unwrap().name);
        assert_eq!("Ivory Black", pick(&paints, Some("nope")).unwrap().name);
        assert_eq!("Ivory Black", pick(&paints, None).unwrap().name);
        assert!(pick(&[], None).is_none());
    }
}
