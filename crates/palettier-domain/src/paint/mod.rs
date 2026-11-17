//! Core paint tube, pigment, medium, and paint system specifications.

pub mod medium;
pub mod properties;
pub mod technique;
pub mod tube;

pub use medium::Medium;
pub use properties::{DryingClass, LayerThickness, Opacity, PaintSystem, Ventilation};
pub use technique::Technique;
pub use tube::{Lightfastness, Paint, PaintLabelMatch, PaintMatchByColour, PaintMatcher, Pigment, SafetyInfo, SafetyLevel, find_closest};

#[cfg(test)]
use std::str::FromStr;
#[cfg(test)]
use crate::color::Rgb;
#[cfg(test)]
use crate::i18n::Locale;

#[cfg(test)]
mod tests {
    use super::*;

    fn make_paint(id: i64, brand: &str, name: &str, hex: &str) -> Paint {
        let mut p = Paint::new(brand, name, hex);
        p.id = Some(id);
        p
    }

    fn sample_catalogue() -> Vec<Paint> {
        vec![
            make_paint(1, "Winsor & Newton", "Burnt Umber", "#4A3427"),
            make_paint(2, "Winsor & Newton", "Winsor Lemon", "#F5E04B"),
            make_paint(3, "Winsor & Newton", "Titanium White", "#F4F2EC"),
            make_paint(4, "Gamblin", "Burnt Umber", "#4C3628"),
            make_paint(5, "Schmincke Norma", "Terre d'ombre brulee", "#4B3529"),
        ]
    }

    #[test]
    fn full_label_matches_tube_including_noise() {
        let matcher = PaintMatcher::new();
        let cat = sample_catalogue();
        let m = matcher
            .match_label(Some("W&N Burnt Umber 37ml"), &cat)
            .expect("doit trouver Burnt Umber");
        assert_eq!("Burnt Umber", m.paint.name);
        assert!(m.is_reliable(), "confiance {}", m.confidence);
    }

    #[test]
    fn brand_alone_matches_nothing() {
        let matcher = PaintMatcher::new();
        let cat = sample_catalogue();
        assert!(matcher.match_label(Some("Winsor & Newton"), &cat).is_none());
    }

    #[test]
    fn accents_do_not_break_matching() {
        let matcher = PaintMatcher::new();
        let cat = sample_catalogue();
        let m = matcher
            .match_label(Some("Terre d'ombre brûlée"), &cat)
            .expect("doit trouver Terre d'ombre brulee");
        assert_eq!("Terre d'ombre brulee", m.paint.name);
    }

    #[test]
    fn unrelated_label_matches_nothing() {
        let matcher = PaintMatcher::new();
        let cat = sample_catalogue();
        assert!(
            matcher
                .match_label(Some("pinceau martre numero 2"), &cat)
                .is_none()
        );
        assert!(matcher.match_label(Some(""), &cat).is_none());
        assert!(matcher.match_label(None, &cat).is_none());
    }

    #[test]
    fn paints_are_ranked_by_distance_to_target_color() {
        let cat = sample_catalogue();
        let closest = find_closest(Rgb::of_hex("#4A3427").unwrap(), &cat, 2);
        assert_eq!(2, closest.len());
        assert!(closest[0].delta_e <= closest[1].delta_e);
        assert_eq!("Burnt Umber", closest[0].paint.name);
    }
}

#[cfg(test)]
mod system_tests {
    use super::*;

    #[test]
    fn mediums_belong_to_one_system_except_pure_paint() {
        assert!(Medium::None.fits(PaintSystem::Oil) && Medium::None.fits(PaintSystem::Acrylic));
        assert!(!Medium::Water.fits(PaintSystem::Oil));
        assert!(!Medium::LinseedOil.fits(PaintSystem::Acrylic));
        assert!(Medium::for_system(PaintSystem::Acrylic).contains(&Medium::Retarder));
        assert!(!Medium::for_system(PaintSystem::Acrylic).contains(&Medium::OdorlessThinner));
    }

    #[test]
    fn acrylic_mediums_never_make_a_coat_fatter_or_leaner() {
        for m in Medium::for_system(PaintSystem::Acrylic) {
            assert_eq!(1.0, m.fatness(0.5), "{m:?}");
        }
    }

    #[test]
    fn shared_technique_changes_thinner_with_the_system() {
        assert_eq!(
            Medium::OdorlessThinner,
            Technique::PinWash.default_medium_in(PaintSystem::Oil)
        );
        assert_eq!(
            Medium::Water,
            Technique::PinWash.default_medium_in(PaintSystem::Acrylic)
        );
        assert_eq!(
            Medium::AcrylicGlazeMedium,
            Technique::Glaze.default_medium_in(PaintSystem::Acrylic)
        );
        assert_ne!(
            Technique::BaseLayer.tips_in(PaintSystem::Oil, Locale::Fr),
            Technique::BaseLayer.tips_in(PaintSystem::Acrylic, Locale::Fr)
        );
    }

    #[test]
    fn native_technique_imposes_its_system_on_the_piece() {
        // An oil wash on an acrylic piece stays oil; layering on an oil piece stays acrylic.
        assert_eq!(
            PaintSystem::Oil,
            Technique::OilWash.system_within(PaintSystem::Acrylic)
        );
        assert_eq!(
            PaintSystem::Acrylic,
            Technique::Layering.system_within(PaintSystem::Oil)
        );
        assert_eq!(
            PaintSystem::Acrylic,
            Technique::Glaze.system_within(PaintSystem::Acrylic)
        );
        assert!(!Technique::for_system(PaintSystem::Oil).contains(&Technique::Drybrush));
        assert!(!Technique::for_system(PaintSystem::Acrylic).contains(&Technique::OilRendering));
    }

    #[test]
    fn technique_labels_match_with_or_without_accents() {
        // Saved by the Java version, without accents.
        assert_eq!(Some(Technique::OilWash), Technique::parse("Jus a l'huile"));
        assert_eq!(Some(Technique::Highlight), Technique::parse("eclaircis et points lumineux"));
        // Current label, English label, code.
        assert_eq!(Some(Technique::EdgeHighlight), Technique::parse("Liserés d'arêtes (edge highlight)"));
        assert_eq!(Some(Technique::Drybrush), Technique::parse("Drybrush"));
        assert_eq!(Some(Technique::ShadeWash), Technique::parse("SHADE_WASH"));
        assert_eq!(None, Technique::parse("Pointillé"));
        assert_eq!("Jus a l'huile", Technique::OilWash.legacy_label());
    }

    #[test]
    fn every_technique_survives_its_db_name() {
        for t in Technique::ALL {
            assert_eq!(t, Technique::from_str(t.as_db_str()).unwrap());
        }
        for m in Medium::ALL {
            assert_eq!(m, Medium::from_str(m.as_db_str()).unwrap());
        }
    }
}

/// Lowercased, trimmed, accents removed: the form two labels are compared in.
fn fold(s: &str) -> String {
    strip_accents(s.trim()).to_lowercase()
}

/// The text with French accents replaced by their base letter.
fn strip_accents(s: &str) -> String {
    s.chars()
        .flat_map(|c| {
            let base: &str = match c {
                'à' | 'â' | 'ä' => "a",
                'À' | 'Â' | 'Ä' => "A",
                'é' | 'è' | 'ê' | 'ë' => "e",
                'É' | 'È' | 'Ê' | 'Ë' => "E",
                'î' | 'ï' => "i",
                'Î' | 'Ï' => "I",
                'ô' | 'ö' => "o",
                'Ô' | 'Ö' => "O",
                'ù' | 'û' | 'ü' => "u",
                'Ù' | 'Û' | 'Ü' => "U",
                'ç' => "c",
                'Ç' => "C",
                'œ' => "oe",
                'Œ' => "OE",
                _ => return vec![c],
            };
            base.chars().collect()
        })
        .collect()
}
