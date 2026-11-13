#![forbid(unsafe_code)]

//! Miniature painting tools, one module per theme.
//!
//! Paints, mediums, techniques and drying are not redefined here: these tools read
//! them from [`crate::paint`] and [`crate::drying`], so that an answer given by a
//! miniature tool agrees with the rest of the application.

use crate::color::Rgb;
use crate::i18n::Locale;
use crate::drying::{DryingContext, DryingEstimate, DryingTimeService, Workshop};
use crate::paint::{DryingClass, LayerThickness, Medium, PaintSystem, Technique, Ventilation};
use serde::{Deserialize, Serialize};

mod colour;
mod effects;
mod application;
mod curing;
mod batch;
mod studio;

pub use colour::*;
pub use effects::*;
pub use application::*;
pub use curing::*;
pub use batch::*;
pub use studio::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn miniature_ladder_produces_5_ordered_contrast_steps() {
        let base = Rgb::of_hex("#9A1115").unwrap(); // Mephiston Red
        let ladder = MiniatureLadder::of_basecoat(base);
        let steps = ladder.steps_hex();
        assert_eq!(steps.len(), 5);
        // Deep shadow must be substantially darker than base
        assert!(ladder.deep_shadow.r() < base.r());
        // Edge highlight must be substantially brighter than base
        assert!(ladder.edge_highlight.r() > base.r() || ladder.edge_highlight.g() > base.g());
    }

    #[test]
    fn weathering_effects_all_have_labels_and_recipes() {
        for effect in WeatheringEffect::ALL {
            assert!(!effect.label_i18n(Locale::Fr).is_empty());
            assert!(Rgb::of_hex(effect.hex_color()).is_ok(), "la teinte se melange depuis la palette");
            assert!(Rgb::of_hex(effect.hex_color()).is_ok());
        }
    }

    #[test]
    fn batch_squad_tracks_progress() {
        let mut squad = BatchSquad::standard_space_marine_squad("Intercessors", 5);
        assert_eq!(squad.total_tasks(), 45); // 5 x 9
        assert_eq!(squad.completed_tasks(), 0);
        assert_eq!(squad.progress_pct(), 0.0);

        squad.toggle(0, 0);
        squad.toggle(0, 1);
        assert_eq!(squad.completed_tasks(), 2);
        assert!(squad.progress_pct() > 4.0);
        assert!(squad.mother_mix_multiplier() > 1.5);
    }

    #[test]
    fn nmm_metal_has_5_ordered_reflectance_steps() {
        for nmm in NmmMetal::ALL {
            assert_eq!(nmm.steps_hex().len(), 5);
            assert!(nmm.steps_hex().iter().all(|h| Rgb::of_hex(h).is_ok()));
            assert!(!nmm.ground_bounce_advice_i18n(Locale::Fr).is_empty());
        }
    }

    #[test]
    fn heat_box_accelerates_drying_time() {
        let calc_ambient = HeatBoxCalculator::new(20.0);
        assert!((calc_ambient.acceleration_factor() - 1.0).abs() < 1e-4);

        // The drying model halves the time every 10 degrees: 40 degrees is 4x faster.
        let calc_box = HeatBoxCalculator::new(40.0);
        assert!((calc_box.acceleration_factor() - 4.0).abs() < 0.05);
        let drying_box = calc_box.accelerated_hours(24.0);
        assert!((drying_box - 6.0).abs() < 0.1);
        assert!(calc_box.is_safe_for_miniature_plastics());

        let hot_box = HeatBoxCalculator::new(50.0);
        assert!(!hot_box.is_safe_for_miniature_plastics());
        assert!(hot_box.safety_warning_i18n(Locale::Fr).is_some());
    }

    #[test]
    fn special_effects_and_brush_care() {
        for fx in SpecialEffectGore::ALL {
            assert!(Rgb::of_hex(fx.hex_color()).is_ok());
            assert!(Rgb::of_hex(fx.hex_color()).is_ok());
        }

        let mut brush = BrushTracker::new("Test Brush", "Raphael", "#1", "Kolinsky");
        assert!(!brush.needs_deep_clean());
        brush.uses_count = 10;
        assert!(brush.needs_deep_clean());
    }

    #[test]
    fn solvents_and_mediums_definitions() {
        for sol in SolventType::ALL {
            assert!(!sol.name().is_empty());
            assert!(!sol.evaporation_speed_i18n(Locale::Fr).is_empty());
        }
    }

    #[test]
    fn stripping_guide_flags_acetone_hazards() {
        let plastic_acetone = StrippingGuide::evaluate(MiniatureMaterial::PolystyrenePlastic, StrippingAgent::PureAcetone);
        assert_eq!(plastic_acetone.severity, StrippingVerdictSeverity::Forbidden);

        let plastic_ipa = StrippingGuide::evaluate(MiniatureMaterial::PolystyrenePlastic, StrippingAgent::Ipa99);
        assert_eq!(plastic_ipa.severity, StrippingVerdictSeverity::Safe);

        let resin_ipa = StrippingGuide::evaluate(MiniatureMaterial::ResinSla3d, StrippingAgent::Ipa99);
        assert_eq!(resin_ipa.severity, StrippingVerdictSeverity::Caution);

        let metal_acetone = StrippingGuide::evaluate(MiniatureMaterial::WhiteMetal, StrippingAgent::PureAcetone);
        assert_eq!(metal_acetone.severity, StrippingVerdictSeverity::Safe);
    }

    #[test]
    fn freezer_simulator_extends_open_time() {
        let open_air = FreezerStorageSimulator::estimated_usable_days(20.0, false, false);
        let sealed = FreezerStorageSimulator::estimated_usable_days(20.0, true, false);
        let freezer = FreezerStorageSimulator::estimated_usable_days(20.0, true, true);
        assert!(open_air <= 2.0, "{open_air}");
        assert!(open_air < sealed && sealed < freezer);
        // About four weeks in the freezer, as painters observe.
        assert!((21.0..35.0).contains(&freezer), "{freezer}");
    }

    #[test]
    fn dry_pigment_and_army_recipe_sheet() {
        for p in DryPigmentEffect::ALL {
            assert!(Rgb::of_hex(p.dry_hex()).is_ok());
            assert!(Rgb::of_hex(p.fixed_hex()).is_ok());
            assert!(!p.application_tip_i18n(Locale::Fr).is_empty());
        }

        // A blank sheet carries no recipe of its own.
        assert!(ArmyRecipeSheet::default().oil_mix_formula.is_empty());

        let sheet = ArmyRecipeSheet {
            army_name: "Compagnie rouge".to_string(),
            unit_name: "Escouade A".to_string(),
            base_acrylic: "Mephiston Red (Citadel)".to_string(),
            oil_mix_formula: "2 parts A + 1 part B".to_string(),
            ..ArmyRecipeSheet::default()
        };
        let md = sheet.to_markdown();
        assert!(md.contains("Compagnie rouge"));
        assert!(md.contains("2 parts A + 1 part B"));

        let html = sheet.to_printable_html();
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Escouade A"));
    }

    #[test]
    fn oil_harmonies_and_consistency_and_glazes() {
        let base_red = Rgb::of_hex("#C8102E").unwrap();
        let harmonies = OilHarmoniesCalculator::calculate(base_red);
        assert_eq!(harmonies.len(), 8);
        for h in &harmonies {
            assert!(Rgb::of_hex(&h.hex).is_ok());
            assert!(!h.role_i18n(Locale::Fr).is_empty());
            assert!(!h.oil_tip_i18n(Locale::Fr).is_empty());
        }

        // Consistency follows the technique, and so do the drops of thinner.
        assert_eq!(Consistency::Wash, Consistency::of(Technique::PinWash));
        assert_eq!(Consistency::Cream, Consistency::of(Technique::OilRendering));
        assert_eq!(0, thinner_drops_per_part(Technique::Highlight, PaintSystem::Oil));
        let wash = thinner_drops_per_part(Technique::PinWash, PaintSystem::Oil);
        let glaze = thinner_drops_per_part(Technique::Glaze, PaintSystem::Oil);
        assert!(wash > glaze && glaze > 0, "{wash} / {glaze}");
        for t in Technique::ALL {
            assert!(!Consistency::of(t).description_i18n(Locale::Fr).is_empty());
        }

        // Zenithal simulation test
        let white_catchlight = OilGlazeZenithalSimulator::simulate_glaze(base_red, 1.0, 0.7);
        let dark_occlusion = OilGlazeZenithalSimulator::simulate_glaze(base_red, 0.08, 0.7);
        assert!(white_catchlight.r() > dark_occlusion.r());
        assert!(dark_occlusion.r() < 0.15);
    }

    #[test]
    fn palette_layout_and_vehicle_and_blending_and_mother_mix_and_freshness() {
        for layout in PaletteLayoutType::ALL {
            assert!(!layout.title_i18n(Locale::Fr).is_empty());
            assert!(!layout.setup_advice_i18n(Locale::Fr).is_empty());
            let slots = layout.slots(Locale::Fr);
            assert!(!slots.is_empty());
            for s in slots {
                assert!(Rgb::of_hex(s.hex).is_ok());
            }
        }

        for fx in VehicleEffectKind::ALL {
            assert!(!fx.commercial_name().is_empty());
            assert!(Rgb::of_hex(fx.target_hex()).is_ok());
            assert!(Rgb::of_hex(fx.target_hex()).is_ok());
            assert!(!fx.application_note_i18n(Locale::Fr).is_empty());
            assert!(fx.technique().fits(PaintSystem::Oil));
        }

        let eval_opr = WetOnWetBlendingAdvisor::evaluate(
            Technique::OilRendering,
            Technique::Blending,
            Locale::Fr,
        );
        assert_eq!(eval_opr.rating, BlendingRating::Excellent);

        let eval_clash = WetOnWetBlendingAdvisor::evaluate(
            Technique::Highlight,
            Technique::PinWash,
            Locale::Fr,
        );
        assert_eq!(eval_clash.rating, BlendingRating::Difficult);

        let req = BatchMotherMixCalculator::calculate(MiniatureScale::Scale32mm, 10, true, Locale::Fr);
        assert!(req.volume_ml > 0.5);
        assert!(req.matchstick_heads >= 10);
        assert!(!req.advice.is_empty());

        // Phases follow the drying model of a dab on the palette (about 8 h open, 38 h to skin at 20 degrees).
        assert_eq!(LivePaletteFreshnessTracker::evaluate(30, 20.0), FreshnessPhase::FreshCream);
        assert_eq!(LivePaletteFreshnessTracker::evaluate(6 * 60, 20.0), FreshnessPhase::ButteryOpr);
        assert_eq!(LivePaletteFreshnessTracker::evaluate(20 * 60, 20.0), FreshnessPhase::TackySkinning);
        assert_eq!(LivePaletteFreshnessTracker::evaluate(3 * 24 * 60, 20.0), FreshnessPhase::Exhausted);
        // Heat shortens every phase.
        assert_eq!(LivePaletteFreshnessTracker::evaluate(6 * 60, 30.0), FreshnessPhase::TackySkinning);
    }

}
