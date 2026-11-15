//! The mixer: what it offers for the working system, the inverse solver, the bench.

use crate::actions::mixes::{NewMix, place_mix};
use crate::state::AppServices;
use palettier_domain::color::Rgb;
use palettier_domain::drying::Workshop;
use palettier_domain::error::DomainResult;
use palettier_domain::i18n::Locale;
use palettier_domain::mix::{MixSuggestion, PaintPart};
use palettier_domain::paint::{DryingClass, Paint, PaintSystem};
use palettier_domain::palette::{Palette, PaletteMix};
use std::collections::BTreeSet;

/// Search scope value for « Mes tubes en stock ».
pub const SCOPE_IN_STOCK: usize = 9998;
/// Search scope value for « Tout le catalogue ».
pub const SCOPE_CATALOG: usize = 9999;

/// The tubes of one system: oil and acrylic never mix.
pub fn paints_of_system(paints: &[Paint], system: PaintSystem) -> Vec<Paint> {
    paints.iter().filter(|p| p.system == system).cloned().collect()
}

/// The palettes offered for one system: those of that system, and the empty ones.
pub fn palettes_of_system(palettes: &[Palette], system: PaintSystem) -> Vec<Palette> {
    palettes
        .iter()
        .filter(|p| p.paints.is_empty() || p.system() == system)
        .cloned()
        .collect()
}

/// The brands of the tubes, sorted, without repeats.
pub fn brands(paints: &[Paint]) -> Vec<String> {
    paints
        .iter()
        .map(|p| p.brand.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// The tubes of one brand (every tube when the brand is blank).
pub fn paints_of_brand(paints: &[Paint], brand: &str) -> Vec<Paint> {
    paints
        .iter()
        .filter(|p| brand.is_empty() || p.brand == brand)
        .cloned()
        .collect()
}

/// The tubes the solver may use: a palette (by index), the tubes in stock, or the
/// whole catalog of the system.
pub fn search_pool(palettes: &[Palette], paints: &[Paint], scope: usize) -> Vec<Paint> {
    if let Some(pal) = palettes.get(scope) {
        pal.paints.clone()
    } else if scope == SCOPE_IN_STOCK {
        paints.iter().filter(|p| p.in_stock).cloned().collect()
    } else {
        paints.to_vec()
    }
}

/// The six best recipes for a colour, from the given tubes, using at most `max_paints`.
/// `None` when the colour is not a hex colour (the search is not run).
pub fn suggest_recipes(
    services: &AppServices,
    target_hex: &str,
    pool: &[Paint],
    max_paints: usize,
) -> Option<Vec<MixSuggestion>> {
    let target = Rgb::of_hex(target_hex).ok()?;
    Some(services.mix_service.suggest_mixes(target, pool, 6, Some(max_paints)))
}

/// The bench with a dose of one tube added (to its existing dose if already there).
pub fn add_to_bench(bench: &[PaintPart], paint: Paint, dose: f64) -> Vec<PaintPart> {
    let dose = dose.max(0.5);
    let mut list = bench.to_vec();
    if let Some(existing) = list
        .iter_mut()
        .find(|p| p.paint.brand == paint.brand && p.paint.name == paint.name)
    {
        existing.parts += dose;
    } else if let Ok(pp) = PaintPart::new(paint, dose) {
        list.push(pp);
    }
    list
}

/// The bench with one tube's dose changed by `delta` parts, never below half a part.
pub fn nudge_part(bench: &[PaintPart], idx: usize, delta: f64) -> Vec<PaintPart> {
    let mut list = bench.to_vec();
    if let Some(item) = list.get_mut(idx) {
        item.parts = if delta < 0.0 {
            (item.parts + delta).max(0.5)
        } else {
            item.parts + delta
        };
    }
    list
}

/// The bench written as a recipe: « 2.0× Ocre + 1.0× Blanc ».
pub fn bench_recipe(bench: &[PaintPart]) -> String {
    bench
        .iter()
        .map(|p| format!("{:.1}× {}", p.parts, p.paint.display_name()))
        .collect::<Vec<_>>()
        .join(" + ")
}

/// The tubes drawn on the gamut map: the chosen palette, else the bench, else the first
/// twelve tubes in stock.
pub fn gamut_paints(
    palettes: &[Palette],
    scope: usize,
    bench: &[PaintPart],
    paints: &[Paint],
) -> Vec<Paint> {
    if let Some(pal) = palettes.get(scope) {
        pal.paints.clone()
    } else if !bench.is_empty() {
        bench.iter().map(|p| p.paint.clone()).collect()
    } else {
        paints.iter().filter(|p| p.in_stock).take(12).cloned().collect()
    }
}

/// The mix simulated on the bench, about to be laid on the palette.
#[derive(Debug, Clone)]
pub struct BenchMix {
    pub name: String,
    pub hex_color: String,
    pub recipe: String,
    pub drying_class: DryingClass,
    pub system: PaintSystem,
    pub workshop: Workshop,
}

/// Lays the bench's mix on the palette and returns every mix followed. A blank name
/// becomes « Mélange #RRGGBB ».
pub async fn place_bench_mix(
    services: &AppServices,
    mix: BenchMix,
    locale: Locale,
) -> DomainResult<Vec<PaletteMix>> {
    let name = match mix.name.trim() {
        "" => format!("{} {}", locale.tr("Mélange", "Mix"), mix.hex_color),
        n => n.to_string(),
    };
    let placed = place_mix(
        services,
        NewMix {
            name,
            hex_color: mix.hex_color,
            recipe: mix.recipe,
            drying_class: mix.drying_class,
            system: mix.system,
            workshop: mix.workshop,
        },
    )
    .await?;
    Ok(placed.unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use palettier_domain::port::{PaintCatalogRepository, PaletteRepository};

    async fn catalog(svc: &AppServices) -> (Vec<Paint>, Vec<Palette>) {
        (
            svc.catalog.all().await.unwrap(),
            svc.palettes.all().await.unwrap(),
        )
    }

    #[tokio::test]
    async fn an_acrylic_system_offers_only_acrylic_paints_and_palettes() {
        let svc = AppServices::in_memory().await.unwrap();
        let (paints, palettes) = catalog(&svc).await;
        assert!(paints.iter().any(|p| p.system == PaintSystem::Oil));
        assert!(paints.iter().any(|p| p.system == PaintSystem::Acrylic));

        let acrylics = paints_of_system(&paints, PaintSystem::Acrylic);
        assert!(!acrylics.is_empty());
        assert!(acrylics.iter().all(|p| p.system == PaintSystem::Acrylic));

        let pals = palettes_of_system(&palettes, PaintSystem::Acrylic);
        assert!(pals.iter().any(|p| !p.paints.is_empty()));
        for pal in &pals {
            assert!(pal.paints.is_empty() || pal.system() == PaintSystem::Acrylic, "{}", pal.name);
        }
        // Whatever the scope, the solver only ever draws from acrylics.
        for scope in (0..pals.len()).chain([SCOPE_IN_STOCK, SCOPE_CATALOG]) {
            let pool = search_pool(&pals, &acrylics, scope);
            if scope >= pals.len() {
                assert!(pool.iter().all(|p| p.system == PaintSystem::Acrylic));
            }
        }
    }

    #[tokio::test]
    async fn an_oil_system_offers_only_oil_paints_and_palettes() {
        let svc = AppServices::in_memory().await.unwrap();
        let (paints, palettes) = catalog(&svc).await;

        let oils = paints_of_system(&paints, PaintSystem::Oil);
        assert!(!oils.is_empty());
        assert!(oils.iter().all(|p| p.system == PaintSystem::Oil));

        let pals = palettes_of_system(&palettes, PaintSystem::Oil);
        assert!(pals.iter().any(|p| !p.paints.is_empty()));
        assert!(pals.iter().all(|p| p.paints.is_empty() || p.system() == PaintSystem::Oil));
        let acrylic_pals = palettes_of_system(&palettes, PaintSystem::Acrylic);
        assert!(
            pals.iter()
                .filter(|p| !p.paints.is_empty())
                .all(|p| !acrylic_pals.iter().any(|a| a.name == p.name)),
            "a palette is offered in both systems"
        );

        let catalog_pool = search_pool(&pals, &oils, SCOPE_CATALOG);
        assert_eq!(catalog_pool.len(), oils.len());
        let stock = search_pool(&pals, &oils, SCOPE_IN_STOCK);
        assert!(stock.iter().all(|p| p.in_stock && p.system == PaintSystem::Oil));
    }

    #[tokio::test]
    async fn the_solver_finds_a_recipe_within_the_system() {
        let svc = AppServices::in_memory().await.unwrap();
        let (paints, _) = catalog(&svc).await;
        let oils = paints_of_system(&paints, PaintSystem::Oil);

        assert!(suggest_recipes(&svc, "pas une couleur", &oils, 3).is_none());
        let found = suggest_recipes(&svc, "#8C6248", &oils, 2).unwrap();
        assert!(!found.is_empty() && found.len() <= 6);
        for sug in &found {
            assert!(sug.parts.len() <= 2);
            assert!(sug.parts.iter().all(|p| p.paint.system == PaintSystem::Oil));
        }
    }

    #[tokio::test]
    async fn the_bench_adds_doses_and_never_goes_below_half_a_part() {
        let svc = AppServices::in_memory().await.unwrap();
        let (paints, _) = catalog(&svc).await;
        let a = paints[0].clone();
        let b = paints[1].clone();

        let bench = add_to_bench(&[], a.clone(), 2.0);
        let bench = add_to_bench(&bench, b, 0.1);
        let bench = add_to_bench(&bench, a, 1.0);
        assert_eq!(bench.len(), 2);
        assert_eq!(bench[0].parts, 3.0);
        assert_eq!(bench[1].parts, 0.5);

        let bench = nudge_part(&bench, 1, -0.5);
        assert_eq!(bench[1].parts, 0.5);
        let bench = nudge_part(&bench, 1, 0.5);
        assert_eq!(bench[1].parts, 1.0);

        let recipe = bench_recipe(&bench);
        assert!(recipe.starts_with("3.0× ") && recipe.contains(" + 1.0× "), "{recipe}");
    }

    #[tokio::test]
    async fn the_gamut_map_shows_the_palette_else_the_bench_else_the_stock() {
        let svc = AppServices::in_memory().await.unwrap();
        let (paints, palettes) = catalog(&svc).await;
        let pals = palettes_of_system(&palettes, PaintSystem::Oil);
        let oils = paints_of_system(&paints, PaintSystem::Oil);

        assert_eq!(gamut_paints(&pals, 0, &[], &oils).len(), pals[0].paints.len());
        let bench = add_to_bench(&[], oils[0].clone(), 1.0);
        assert_eq!(gamut_paints(&pals, SCOPE_IN_STOCK, &bench, &oils).len(), 1);
        let stock = gamut_paints(&pals, SCOPE_IN_STOCK, &[], &oils);
        assert!(stock.len() <= 12 && stock.iter().all(|p| p.in_stock));
    }

    #[tokio::test]
    async fn places_the_bench_mix_on_the_palette() {
        let svc = AppServices::in_memory().await.unwrap();
        let (paints, _) = catalog(&svc).await;
        let bench = add_to_bench(&[], paints_of_system(&paints, PaintSystem::Acrylic)[0].clone(), 1.0);
        let result = svc.mix_service.mix(&bench).unwrap();
        let mix = BenchMix {
            name: "  ".into(),
            hex_color: result.hex(),
            recipe: bench_recipe(&bench),
            drying_class: result.drying_class,
            system: PaintSystem::Acrylic,
            workshop: Workshop::default(),
        };

        let all = place_bench_mix(&svc, mix.clone(), Locale::Fr).await.unwrap();
        let placed = all.iter().find(|m| m.hex_color == result.hex()).unwrap();
        assert_eq!(placed.name, format!("Mélange {}", result.hex()));
        assert_eq!(placed.system, PaintSystem::Acrylic);
        assert_eq!(placed.recipe, bench_recipe(&bench));

        let named = BenchMix { name: " Ombre chaude ".into(), ..mix };
        let all = place_bench_mix(&svc, named, Locale::En).await.unwrap();
        assert!(all.iter().any(|m| m.name == "Ombre chaude"));
    }
}
