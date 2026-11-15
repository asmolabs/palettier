//! The tube catalog: searching it, adding the painter's own tubes, the drawer, the
//! calibration of a tube and the recognition of tubes on a photo.

use crate::state::AppServices;
use palettier_ai::RecognizedTube;
use palettier_domain::color::{Rgb, delta_e_2000};
use palettier_domain::error::DomainResult;
use palettier_domain::paint::{DryingClass, Opacity, Paint, PaintSystem};
use palettier_domain::port::PaintCatalogRepository;
use std::collections::BTreeSet;

/// The brand filter value that keeps every brand.
pub const ALL_BRANDS: &str = "ALL";

/// What the painter typed in the « Ajouter un tube personnalisé » card.
#[derive(Debug, Clone)]
pub struct NewTube {
    pub brand: String,
    pub name: String,
    pub code: String,
    /// C.I. pigment codes, comma-separated.
    pub pigments: String,
    pub hex_color: String,
    pub opacity: Opacity,
    pub drying_class: DryingClass,
    pub system: PaintSystem,
}

/// The pigment codes of a tube, comma-separated.
pub fn pigment_codes(paint: &Paint) -> String {
    paint.pigments.iter().cloned().collect::<Vec<_>>().join(", ")
}

/// The brands of the given tubes, sorted, once each.
pub fn brands_of(paints: &[Paint]) -> Vec<String> {
    paints
        .iter()
        .map(|p| p.brand.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// The tubes matching the search: the drawer only if asked, one brand (or
/// [`ALL_BRANDS`]), and a query looked up in brand, name, code and pigments.
pub fn filter_paints(paints: &[Paint], query: &str, brand: &str, stock_only: bool) -> Vec<Paint> {
    let q = query.trim().to_lowercase();
    paints
        .iter()
        .filter(|p| {
            if stock_only && !p.in_stock {
                return false;
            }
            if brand != ALL_BRANDS && p.brand != brand {
                return false;
            }
            if !q.is_empty() {
                let hay = format!("{} {} {} {}", p.brand, p.name, p.code, pigment_codes(p))
                    .to_lowercase();
                if !hay.contains(&q) {
                    return false;
                }
            }
            true
        })
        .cloned()
        .collect()
}

/// The `count` tubes closest in colour to `target` (ΔE 2000), closest first.
pub fn closest_substitutes(paints: &[Paint], target: &Paint, count: usize) -> Vec<(Paint, f64)> {
    let mut list: Vec<(Paint, f64)> = paints
        .iter()
        .filter(|p| p.id != target.id)
        .map(|p| (p.clone(), delta_e_2000(target.color(), p.color())))
        .collect();
    list.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    list.into_iter().take(count).collect()
}

/// Adds the painter's own tube to the catalog and to the drawer. Pigment codes are
/// upper-cased. Returns `None` when the brand or the name is blank (nothing is saved).
pub async fn add_custom_tube(services: &AppServices, tube: NewTube) -> DomainResult<Option<Paint>> {
    let brand = tube.brand.trim().to_string();
    let name = tube.name.trim().to_string();
    if brand.is_empty() || name.is_empty() {
        return Ok(None);
    }
    let mut paint = Paint::new(brand, name, tube.hex_color.trim());
    paint.code = tube.code.trim().to_string();
    paint.pigments = tube
        .pigments
        .split(',')
        .map(|s| s.trim().to_uppercase())
        .filter(|s| !s.is_empty())
        .collect();
    paint.opacity = tube.opacity;
    paint.drying_class = tube.drying_class;
    paint.user_added = true;
    paint.in_stock = true;
    paint.system = tube.system;
    services.catalog.save(&paint).await.map(Some)
}

/// Puts a tube in the drawer, or takes it out.
pub async fn set_in_stock(services: &AppServices, paint_id: i64, owned: bool) -> DomainResult<()> {
    services.catalog.set_owned(paint_id, owned).await
}

/// Removes a tube from the catalog.
pub async fn delete_tube(services: &AppServices, paint_id: i64) -> DomainResult<()> {
    services.catalog.delete(paint_id).await
}

/// Records the measured masstone and 1:10 white tint of a tube. A colour that does not
/// read as hex is skipped; both are attempted even if the first fails, and the first
/// error is returned.
pub async fn calibrate_tube(
    services: &AppServices,
    paint_id: i64,
    masstone_hex: &str,
    tint_hex: &str,
) -> DomainResult<()> {
    let mut masstone = Ok(());
    if let Ok(rgb) = Rgb::of_hex(masstone_hex) {
        masstone = services.catalog.record_masstone(paint_id, rgb).await;
    }
    let mut tint = Ok(());
    if let Ok(rgb) = Rgb::of_hex(tint_hex) {
        tint = services.catalog.record_tint(paint_id, rgb).await;
    }
    masstone.and(tint)
}

/// Reads the tubes on a photo and matches them against the catalog.
pub async fn recognize_tubes(
    services: &AppServices,
    image: &[u8],
    catalog: &[Paint],
) -> DomainResult<Vec<RecognizedTube>> {
    services.recognition_service.recognize(image, catalog).await.map(|r| r.tubes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tube(brand: &str, name: &str) -> NewTube {
        NewTube {
            brand: brand.into(),
            name: name.into(),
            code: " 042 ".into(),
            pigments: " py42, pr101 ,, ".into(),
            hex_color: " #A85638 ".into(),
            opacity: Opacity::Opaque,
            drying_class: DryingClass::Slow,
            system: PaintSystem::Acrylic,
        }
    }

    async fn find(svc: &AppServices, id: i64) -> Paint {
        svc.catalog.find_by_id(id).await.unwrap().unwrap()
    }

    #[tokio::test]
    async fn adds_a_custom_tube_to_the_catalog_and_the_drawer() {
        let svc = AppServices::in_memory().await.unwrap();
        let saved = add_custom_tube(&svc, tube(" Studio ", " Ocre maison "))
            .await
            .unwrap()
            .expect("a named tube is saved");

        let paint = find(&svc, saved.id.unwrap()).await;
        assert_eq!(paint.brand, "Studio");
        assert_eq!(paint.name, "Ocre maison");
        assert_eq!(paint.code, "042");
        assert_eq!(pigment_codes(&paint), "PY42, PR101");
        assert_eq!(paint.hex_color.to_uppercase(), "#A85638");
        assert_eq!(paint.opacity, Opacity::Opaque);
        assert_eq!(paint.drying_class, DryingClass::Slow);
        assert_eq!(paint.system, PaintSystem::Acrylic);
        assert!(paint.user_added);
        assert!(paint.in_stock);
    }

    #[tokio::test]
    async fn a_tube_without_brand_or_name_is_not_added() {
        let svc = AppServices::in_memory().await.unwrap();
        let before = svc.catalog.count().await.unwrap();
        assert!(add_custom_tube(&svc, tube(" ", "Ocre")).await.unwrap().is_none());
        assert!(add_custom_tube(&svc, tube("Studio", "  ")).await.unwrap().is_none());
        assert_eq!(svc.catalog.count().await.unwrap(), before);
    }

    #[tokio::test]
    async fn puts_a_tube_in_the_drawer_and_takes_it_out() {
        let svc = AppServices::in_memory().await.unwrap();
        let id = svc.catalog.all().await.unwrap()[0].id.unwrap();

        set_in_stock(&svc, id, true).await.unwrap();
        assert!(find(&svc, id).await.in_stock);
        set_in_stock(&svc, id, false).await.unwrap();
        assert!(!find(&svc, id).await.in_stock);
    }

    #[tokio::test]
    async fn deletes_a_custom_tube() {
        let svc = AppServices::in_memory().await.unwrap();
        let saved = add_custom_tube(&svc, tube("Studio", "Ocre")).await.unwrap().unwrap();
        let id = saved.id.unwrap();

        delete_tube(&svc, id).await.unwrap();
        assert!(svc.catalog.find_by_id(id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn calibrates_masstone_and_tint_and_skips_what_is_not_a_colour() {
        let svc = AppServices::in_memory().await.unwrap();
        let id = svc.catalog.all().await.unwrap()[0].id.unwrap();

        calibrate_tube(&svc, id, "#123456", "#E8DFD5").await.unwrap();
        let paint = find(&svc, id).await;
        assert_eq!(paint.hex_color.to_uppercase(), "#123456");
        assert_eq!(paint.tint_hex.map(|h| h.to_uppercase()).as_deref(), Some("#E8DFD5"));

        calibrate_tube(&svc, id, "pas une couleur", "#AABBCC").await.unwrap();
        let paint = find(&svc, id).await;
        assert_eq!(paint.hex_color.to_uppercase(), "#123456");
        assert_eq!(paint.tint_hex.map(|h| h.to_uppercase()).as_deref(), Some("#AABBCC"));
    }

    #[tokio::test]
    async fn filters_the_catalog_and_finds_substitutes() {
        let svc = AppServices::in_memory().await.unwrap();
        let mut paints = svc.catalog.all().await.unwrap();
        paints[0].in_stock = true;
        for p in paints.iter_mut().skip(1) {
            p.in_stock = false;
        }

        let brands = brands_of(&paints);
        assert!(brands.windows(2).all(|w| w[0] < w[1]));
        assert_eq!(filter_paints(&paints, "", ALL_BRANDS, false).len(), paints.len());
        assert_eq!(filter_paints(&paints, "", ALL_BRANDS, true), vec![paints[0].clone()]);

        let brand = &paints[0].brand;
        assert!(filter_paints(&paints, "", brand, false).iter().all(|p| &p.brand == brand));
        let by_name = filter_paints(&paints, &paints[0].name.to_uppercase(), ALL_BRANDS, false);
        assert!(by_name.contains(&paints[0]));

        let subs = closest_substitutes(&paints, &paints[0], 6);
        assert_eq!(subs.len(), 6.min(paints.len() - 1));
        assert!(subs.iter().all(|(p, _)| p.id != paints[0].id));
        assert!(subs.windows(2).all(|w| w[0].1 <= w[1].1));
    }
}
