//! Working palettes: creating them and choosing their tubes.

use crate::state::AppServices;
use palettier_domain::error::DomainResult;
use palettier_domain::i18n::Locale;
use palettier_domain::paint::Paint;
use palettier_domain::palette::Palette;
use palettier_domain::port::PaletteRepository;

/// Catalog codes of the eight tubes of the Schmincke Norma Blue set 71200097.
pub const NORMA_BLUE_SET: [&str; 8] = [
    "21114", "21236", "21311", "21318", "21418", "21502", "21600", "21705",
];

/// Creates an empty palette and returns every palette. Returns `None` when the name is
/// blank (nothing is saved).
pub async fn create_palette(
    services: &AppServices,
    name: &str,
    purpose: &str,
    notes: &str,
) -> DomainResult<Option<Vec<Palette>>> {
    let name = name.trim();
    if name.is_empty() {
        return Ok(None);
    }
    let palette = Palette {
        id: None,
        name: name.to_string(),
        purpose: purpose.trim().to_string(),
        notes: notes.trim().to_string(),
        paints: Vec::new(),
    };
    save_palette(services, &palette).await.map(Some)
}

/// The Norma Blue set, built from the tubes of `paints` that belong to it.
pub fn norma_blue_set(paints: &[Paint], locale: Locale) -> Palette {
    let set: Vec<Paint> = NORMA_BLUE_SET
        .iter()
        .filter_map(|code| {
            paints
                .iter()
                .find(|p| p.brand.contains("Norma Blue") && p.code == *code)
                .cloned()
        })
        .collect();
    Palette {
        id: None,
        name: locale
            .tr("Schmincke Norma Blue (Coffret 71200097)", "Schmincke Norma Blue (Set 71200097)")
            .to_string(),
        purpose: locale
            .tr("Coffret 8 tubes — Bustes & figurines", "8-tube set — Busts & miniatures")
            .to_string(),
        notes: locale
            .tr(
                "Palette de 8 tubes avec mélanges-mères Si (Sienne), BF (Brun foncé) et NC (Noir coloré).",
                "8-tube palette with mother mixes Si (Burnt Sienna), BF (Dark Brown), and NC (Chromatic Black).",
            )
            .to_string(),
        paints: set,
    }
}

/// Saves the Norma Blue set, built from `paints`, and returns every palette.
pub async fn add_norma_blue_set(
    services: &AppServices,
    paints: &[Paint],
    locale: Locale,
) -> DomainResult<Vec<Palette>> {
    save_palette(services, &norma_blue_set(paints, locale)).await
}

/// Saves a palette and returns every palette.
pub async fn save_palette(services: &AppServices, palette: &Palette) -> DomainResult<Vec<Palette>> {
    services.palettes.save(palette).await?;
    services.palettes.all().await
}

/// Adds a tube to a palette.
pub async fn add_paint(
    services: &AppServices,
    palette: &Palette,
    paint: Paint,
) -> DomainResult<Vec<Palette>> {
    save_palette(services, &palette.with_paint(paint)).await
}

/// Takes a tube out of a palette.
pub async fn remove_paint(
    services: &AppServices,
    palette: &Palette,
    paint: &Paint,
) -> DomainResult<Vec<Palette>> {
    save_palette(services, &palette.without_paint(paint)).await
}

/// Replaces an out-of-stock tube of a palette by one on the shelf.
pub async fn substitute_paint(
    services: &AppServices,
    palette: &Palette,
    absent: &Paint,
    replacement: Paint,
) -> DomainResult<Vec<Palette>> {
    save_palette(services, &palette.without_paint(absent).with_paint(replacement)).await
}

/// Deletes a palette and returns the palettes left.
pub async fn delete_palette(services: &AppServices, id: i64) -> DomainResult<Vec<Palette>> {
    services.palettes.delete(id).await?;
    services.palettes.all().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use palettier_domain::port::PaintCatalogRepository;

    #[tokio::test]
    async fn creates_fills_and_deletes_a_palette() {
        let svc = AppServices::in_memory().await.unwrap();
        assert!(create_palette(&svc, " ", "", "").await.unwrap().is_none());

        let all = create_palette(&svc, " Drap bleu ", " 75mm ", "").await.unwrap().unwrap();
        let pal = all.iter().find(|p| p.name == "Drap bleu").unwrap().clone();
        assert_eq!(pal.purpose, "75mm");
        assert!(pal.paints.is_empty());

        let paint = svc.catalog.all().await.unwrap().remove(0);
        let all = add_paint(&svc, &pal, paint.clone()).await.unwrap();
        let pal = all.into_iter().find(|p| p.id == pal.id).unwrap();
        assert_eq!(pal.paints.len(), 1);

        let all = remove_paint(&svc, &pal, &paint).await.unwrap();
        assert!(all.iter().find(|p| p.id == pal.id).unwrap().paints.is_empty());

        let left = delete_palette(&svc, pal.id.unwrap()).await.unwrap();
        assert!(left.iter().all(|p| p.id != pal.id));
    }

    #[tokio::test]
    async fn the_norma_blue_set_takes_its_eight_tubes_from_the_catalog() {
        let svc = AppServices::in_memory().await.unwrap();
        let paints = svc.catalog.all().await.unwrap();
        let set = norma_blue_set(&paints, Locale::Fr);
        assert!(set.paints.iter().all(|p| p.brand.contains("Norma Blue")));
        assert!(set.paints.len() <= NORMA_BLUE_SET.len());
        let all = add_norma_blue_set(&svc, &paints, Locale::Fr).await.unwrap();
        assert!(all.iter().any(|p| p.name.contains("71200097")));
    }
}
