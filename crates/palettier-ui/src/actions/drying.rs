//! The drying calculator: what one coat, painted in the workshop of the day, takes to dry.

use crate::state::AppServices;
use palettier_domain::drying::{CustomMedium, DryingContext, DryingEstimate, Workshop};
use palettier_domain::paint::{DryingClass, LayerThickness, Medium, PaintSystem};

/// The coat set in « Paramètres de la couche ».
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CoatSettings {
    pub system: PaintSystem,
    pub drying_class: DryingClass,
    pub medium: Medium,
    /// Share of medium, in percent.
    pub medium_ratio_pct: f64,
    pub thickness: LayerThickness,
    pub workshop: Workshop,
}

/// A coat's estimate and what it is computed from.
#[derive(Debug, Clone)]
pub struct CoatDrying {
    /// The medium actually used: one of the other system (left over from a switch)
    /// falls back to pure paint.
    pub medium: Medium,
    /// Fatness index of the medium at its ratio.
    pub fatness: f64,
    pub estimate: DryingEstimate,
}

/// Estimates the drying of a coat. The ratio is clamped to 0..100 %.
pub fn estimate_coat(services: &AppServices, coat: CoatSettings) -> CoatDrying {
    let medium = if coat.medium.fits(coat.system) {
        coat.medium
    } else {
        Medium::None
    };
    let ratio = (coat.medium_ratio_pct / 100.0).clamp(0.0, 1.0);
    let context = DryingContext {
        system: coat.system,
        drying_class: coat.drying_class,
        medium,
        medium_ratio: ratio,
        thickness: coat.thickness,
        workshop: coat.workshop,
    };
    CoatDrying {
        medium,
        fatness: medium.fatness(ratio),
        estimate: services.drying_service.estimate(&context),
    }
}

/// The painter's own medium, from its oil / thinner / resin shares and drops of driers.
pub fn formulate_medium(oil: u32, solvent: u32, resin: u32, driers: u32) -> CustomMedium {
    CustomMedium::new("Medium Atelier", oil as f64, solvent as f64, resin as f64, driers)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn coat(system: PaintSystem, medium: Medium, pct: f64) -> CoatSettings {
        CoatSettings {
            system,
            drying_class: DryingClass::Medium,
            medium,
            medium_ratio_pct: pct,
            thickness: LayerThickness::Normal,
            workshop: Workshop::default(),
        }
    }

    #[tokio::test]
    async fn a_medium_of_the_other_system_falls_back_to_pure_paint() {
        let svc = AppServices::in_memory().await.unwrap();
        let oil_medium = Medium::ALL
            .into_iter()
            .find(|m| *m != Medium::None && !m.fits(PaintSystem::Acrylic))
            .unwrap();
        let drying = estimate_coat(&svc, coat(PaintSystem::Acrylic, oil_medium, 40.0));
        assert_eq!(drying.medium, Medium::None);

        let kept = estimate_coat(&svc, coat(PaintSystem::Oil, oil_medium, 40.0));
        assert_eq!(kept.medium, oil_medium);
        assert_eq!(kept.fatness, oil_medium.fatness(0.4));
    }

    #[tokio::test]
    async fn the_ratio_is_clamped_and_acrylic_dries_faster_than_oil() {
        let svc = AppServices::in_memory().await.unwrap();
        let over = estimate_coat(&svc, coat(PaintSystem::Oil, Medium::None, 250.0));
        let full = estimate_coat(&svc, coat(PaintSystem::Oil, Medium::None, 100.0));
        assert_eq!(over.fatness, full.fatness);

        let oil = estimate_coat(&svc, coat(PaintSystem::Oil, Medium::None, 0.0));
        let acrylic = estimate_coat(&svc, coat(PaintSystem::Acrylic, Medium::None, 0.0));
        assert!(acrylic.estimate.touch_dry < oil.estimate.touch_dry);
    }

    #[test]
    fn a_fatter_medium_has_a_higher_fat_index() {
        let lean = formulate_medium(20, 80, 0, 0);
        let fat = formulate_medium(50, 25, 25, 1);
        assert!(fat.fat_index() > lean.fat_index());
    }
}
