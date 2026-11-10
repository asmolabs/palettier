use crate::error::{DomainError, DomainResult};
use crate::i18n::Locale;
use crate::paint::{DryingClass, LayerThickness, Medium, Technique, Ventilation};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// A customized studio medium formulation blending oils, solvents, and resins.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomMedium {
    pub name: String,
    /// Percentage of oil (e.g. Linseed, Walnut, Stand oil) from 0.0 to 1.0.
    pub oil_ratio: f64,
    /// Percentage of solvent (e.g. Odorless mineral spirits, Turpentine) from 0.0 to 1.0.
    pub solvent_ratio: f64,
    /// Percentage of resin/varnish (e.g. Dammar, Alkyd, Retouch) from 0.0 to 1.0.
    pub resin_ratio: f64,
    /// Drops of siccative/drier per 50ml (e.g. 0 to 10).
    pub drier_drops: u32,
}

impl CustomMedium {
    pub fn new(name: impl Into<String>, oil: f64, solvent: f64, resin: f64, drier_drops: u32) -> Self {
        let total = oil + solvent + resin;
        let (oil_ratio, solvent_ratio, resin_ratio) = if total > 0.0 {
            (oil / total, solvent / total, resin / total)
        } else {
            (0.33, 0.33, 0.34)
        };
        Self {
            name: name.into(),
            oil_ratio,
            solvent_ratio,
            resin_ratio,
            drier_drops,
        }
    }

    /// Classic Flemish medium (1 part stand oil, 1 part dammar, 1 part turpentine).
    pub fn flemish() -> Self {
        Self::new("Flamand 1/3-1/3-1/3", 0.333, 0.333, 0.334, 0)
    }

    /// Lean wash medium (1 part oil, 4 parts odorless mineral spirits).
    pub fn lean_wash() -> Self {
        Self::new("Lavis maigre", 0.20, 0.80, 0.0, 0)
    }

    /// Fat glaze medium (2 parts stand oil, 1 part dammar, 1 part turpentine).
    pub fn fat_glaze() -> Self {
        Self::new("Glacis gras", 0.50, 0.25, 0.25, 0)
    }

    /// Fat index on a scale from 0.0 (very lean) to 1.0 (very fat).
    /// Used by the Fat-Over-Lean validator.
    pub fn fat_index(&self) -> f64 {
        let fat = self.oil_ratio * 1.0 + self.resin_ratio * 0.45;
        fat.clamp(0.05, 1.0)
    }

    /// Multiplier on touch-dry time.
    pub fn drying_multiplier(&self) -> f64 {
        let base = 0.4 * self.solvent_ratio + 1.6 * self.oil_ratio + 0.9 * self.resin_ratio;
        let drier_discount = (1.0 - (self.drier_drops.min(10) as f64 * 0.06)).max(0.4);
        (base * drier_discount).max(0.2)
    }

    /// Description of the medium's behavior in the studio.
    pub fn description_i18n(&self, locale: Locale) -> String {
        let fat_pct = (self.fat_index() * 100.0).round() as u32;
        let speed = if self.drying_multiplier() < 0.7 {
            locale.tr("Séchage rapide", "Fast drying")
        } else if self.drying_multiplier() > 1.3 {
            locale.tr("Séchage lent / ouvert", "Slow / long open time")
        } else {
            locale.tr("Séchage moyen", "Medium drying")
        };
        format!(
            "{} — {}: {}% · {}",
            self.name,
            locale.tr("Indice gras", "Fat index"),
            fat_pct,
            speed
        )
    }
}

/// Workshop environmental conditions.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Workshop {
    pub temperature_celsius: f64,
    pub relative_humidity: f64,
    pub ventilation: Ventilation,
}

impl Workshop {
    pub fn new(
        temperature_celsius: f64,
        relative_humidity: f64,
        ventilation: Ventilation,
    ) -> DomainResult<Self> {
        if !(-20.0..=60.0).contains(&temperature_celsius) {
            return Err(DomainError::InvalidArgument(format!(
                "Température hors plage (-20..60 °C): {temperature_celsius}"
            )));
        }
        if !(0.0..=100.0).contains(&relative_humidity) {
            return Err(DomainError::InvalidArgument(format!(
                "Humidité hors plage (0..100 %): {relative_humidity}"
            )));
        }
        Ok(Self {
            temperature_celsius,
            relative_humidity,
            ventilation,
        })
    }

    pub fn standard() -> Self {
        Self {
            temperature_celsius: 20.0,
            relative_humidity: 50.0,
            ventilation: Ventilation::Normal,
        }
    }
}

impl Default for Workshop {
    fn default() -> Self {
        Self::standard()
    }
}

/// All parameters that govern the drying of a coat.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DryingContext {
    pub drying_class: DryingClass,
    pub medium: Medium,
    pub medium_ratio: f64,
    pub thickness: LayerThickness,
    pub workshop: Workshop,
}

impl DryingContext {
    pub fn new(
        drying_class: DryingClass,
        medium: Medium,
        medium_ratio: f64,
        thickness: LayerThickness,
        workshop: Workshop,
    ) -> DomainResult<Self> {
        if !(0.0..=1.0).contains(&medium_ratio) {
            return Err(DomainError::InvalidArgument(format!(
                "Proportion de medium hors de [0, 1] : {medium_ratio}"
            )));
        }
        Ok(Self {
            drying_class,
            medium,
            medium_ratio,
            thickness,
            workshop,
        })
    }

    pub fn for_technique(
        technique: Technique,
        drying_class: DryingClass,
        workshop: Workshop,
    ) -> Self {
        Self {
            drying_class,
            medium: technique.default_medium(),
            medium_ratio: technique.default_ratio(),
            thickness: technique.typical_thickness(),
            workshop,
        }
    }
}

/// Drying milestones for a coat.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DryingEstimate {
    pub open_time: Duration,
    pub touch_dry: Duration,
    pub recoat: Duration,
    pub through_dry: Duration,
    pub full_cure: Duration,
    pub advice: Vec<String>,
}

/// Parameters of the drying model.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DryingProperties {
    pub reference_temperature: f64,
    pub temperature_halving_step: f64,
    pub reference_humidity: f64,
    pub humidity_sensitivity: f64,
    pub open_time_share: f64,
    pub recoat_factor: f64,
    pub through_dry_factor: f64,
    pub full_cure_factor: f64,
}

impl Default for DryingProperties {
    fn default() -> Self {
        Self {
            reference_temperature: 20.0,
            temperature_halving_step: 10.0,
            reference_humidity: 50.0,
            humidity_sensitivity: 0.6,
            open_time_share: 0.22,
            recoat_factor: 1.8,
            through_dry_factor: 5.0,
            full_cure_factor: 22.0,
        }
    }
}

/// Estimates drying milestones for an oil paint coat.
#[derive(Debug, Clone, Default)]
pub struct DryingTimeService {
    properties: DryingProperties,
}

impl DryingTimeService {
    pub fn new(properties: DryingProperties) -> Self {
        Self { properties }
    }

    pub fn estimate(&self, context: &DryingContext) -> DryingEstimate {
        let base_minutes = (context.drying_class.reference_touch_dry().as_secs() / 60) as f64;
        let hours = (base_minutes / 60.0)
            * context.medium.drying_factor(context.medium_ratio)
            * context.thickness.factor()
            * self.temperature_factor(context.workshop.temperature_celsius)
            * self.humidity_factor(context.workshop.relative_humidity)
            * context.workshop.ventilation.factor();

        DryingEstimate {
            open_time: Self::hours_to_duration(hours * self.properties.open_time_share),
            touch_dry: Self::hours_to_duration(hours),
            recoat: Self::hours_to_duration(hours * self.properties.recoat_factor),
            through_dry: Self::hours_to_duration(hours * self.properties.through_dry_factor),
            full_cure: Self::hours_to_duration(hours * self.properties.full_cure_factor),
            advice: Self::advice(context),
        }
    }

    fn temperature_factor(&self, celsius: f64) -> f64 {
        let delta = self.properties.reference_temperature - celsius;
        2.0_f64.powf(delta / self.properties.temperature_halving_step)
    }

    fn humidity_factor(&self, relative_humidity: f64) -> f64 {
        let delta = relative_humidity - self.properties.reference_humidity;
        (1.0 + delta / 100.0 * self.properties.humidity_sensitivity).max(0.5)
    }

    fn advice(context: &DryingContext) -> Vec<String> {
        let mut out = Vec::new();
        let workshop = &context.workshop;

        if context.medium_ratio > context.medium.max_useful_ratio()
            && context.medium != Medium::None
        {
            let max_pct = (context.medium.max_useful_ratio() * 100.0).round() as i64;
            out.push(format!(
                "Proportion de {} au-dela de la limite utile ({} %) : le liant ne tient plus le pigment.",
                context.medium.label().to_lowercase(),
                max_pct
            ));
        }
        if workshop.temperature_celsius < 15.0 {
            out.push(
                "Sous 15 degres, l'huile prend tres lentement et peut rester collante plusieurs jours."
                    .to_string(),
            );
        }
        if workshop.temperature_celsius > 28.0 {
            out.push(
                "Au-dela de 28 degres, le temps ouvert se reduit fortement : travaillez par petites zones."
                    .to_string(),
            );
        }
        if workshop.relative_humidity > 70.0 {
            out.push(
                "Humidite elevee : le sechage tire en longueur, evitez de vernir juste apres."
                    .to_string(),
            );
        }
        if context.thickness >= LayerThickness::Thick {
            out.push(
                "Couche epaisse : la peau se forme avant le coeur. Attendez le sechage a coeur avant toute manipulation."
                    .to_string(),
            );
        }
        if context.medium == Medium::CobaltDrier && context.medium_ratio > 0.03 {
            out.push(
                "Le siccatif au cobalt s'emploie a la goutte : au-dela, la couche craquelle en vieillissant."
                    .to_string(),
            );
        }
        if context.drying_class == DryingClass::VerySlow {
            out.push(format!(
                "{} Ces pigments restent ouverts longtemps : parfait pour les fondus, penible pour les couches suivantes.",
                context.drying_class.typical_pigments()
            ));
        }
        out.push(context.medium.advice().to_string());
        out
    }

    fn hours_to_duration(value: f64) -> Duration {
        let mins = (value * 60.0).round().max(0.0) as u64;
        Duration::from_secs(mins * 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_context(
        medium: Medium,
        ratio: f64,
        thickness: LayerThickness,
        workshop: Workshop,
    ) -> DryingContext {
        DryingContext::new(DryingClass::Medium, medium, ratio, thickness, workshop).unwrap()
    }

    #[test]
    fn drying_milestones_are_ordered() {
        let service = DryingTimeService::default();
        let estimate = service.estimate(&make_context(
            Medium::OdorlessThinner,
            0.8,
            LayerThickness::Glaze,
            Workshop::standard(),
        ));
        assert!(estimate.open_time < estimate.touch_dry);
        assert!(estimate.touch_dry < estimate.recoat);
        assert!(estimate.recoat < estimate.through_dry);
        assert!(estimate.through_dry < estimate.full_cure);
    }

    #[test]
    fn thinner_accelerates_and_linseed_oil_slows_drying() {
        let service = DryingTimeService::default();
        let pure = service
            .estimate(&make_context(
                Medium::None,
                0.0,
                LayerThickness::Normal,
                Workshop::standard(),
            ))
            .touch_dry;
        let thinned = service
            .estimate(&make_context(
                Medium::OdorlessThinner,
                0.8,
                LayerThickness::Normal,
                Workshop::standard(),
            ))
            .touch_dry;
        let oiled = service
            .estimate(&make_context(
                Medium::LinseedOil,
                0.4,
                LayerThickness::Normal,
                Workshop::standard(),
            ))
            .touch_dry;

        assert!(thinned < pure);
        assert!(oiled > pure);
    }

    #[test]
    fn thick_coat_dries_much_slower_than_glaze() {
        let service = DryingTimeService::default();
        let glaze = service
            .estimate(&make_context(
                Medium::None,
                0.0,
                LayerThickness::Glaze,
                Workshop::standard(),
            ))
            .touch_dry;
        let impasto = service
            .estimate(&make_context(
                Medium::None,
                0.0,
                LayerThickness::Impasto,
                Workshop::standard(),
            ))
            .touch_dry;

        assert!(impasto > glaze * 5);
    }

    #[test]
    fn ten_degrees_colder_doubles_drying_time() {
        let service = DryingTimeService::default();
        let warm = service
            .estimate(&make_context(
                Medium::None,
                0.0,
                LayerThickness::Normal,
                Workshop {
                    temperature_celsius: 20.0,
                    relative_humidity: 50.0,
                    ventilation: Ventilation::Normal,
                },
            ))
            .touch_dry;
        let cold = service
            .estimate(&make_context(
                Medium::None,
                0.0,
                LayerThickness::Normal,
                Workshop {
                    temperature_celsius: 10.0,
                    relative_humidity: 50.0,
                    ventilation: Ventilation::Normal,
                },
            ))
            .touch_dry;

        let warm_mins = (warm.as_secs() / 60) as i64;
        let cold_mins = (cold.as_secs() / 60) as i64;
        assert!((cold_mins - warm_mins * 2).abs() <= 2);
    }

    #[test]
    fn excessive_dilution_triggers_warning() {
        let service = DryingTimeService::default();
        let estimate = service.estimate(&make_context(
            Medium::OdorlessThinner,
            0.98,
            LayerThickness::Glaze,
            Workshop::standard(),
        ));
        assert!(estimate.advice.iter().any(|a| a.contains("limite utile")));
    }

    #[test]
    fn cold_workshop_is_reported() {
        let service = DryingTimeService::default();
        let estimate = service.estimate(&make_context(
            Medium::None,
            0.0,
            LayerThickness::Normal,
            Workshop {
                temperature_celsius: 12.0,
                relative_humidity: 50.0,
                ventilation: Ventilation::Normal,
            },
        ));
        assert!(estimate.advice.iter().any(|a| a.contains("15 degres")));
    }

    #[test]
    fn fat_over_lean_reads_medium_ratio() {
        assert!(Medium::OdorlessThinner.fatness(0.8) < Medium::None.fatness(0.0));
        assert!(Medium::LinseedOil.fatness(0.4) > Medium::None.fatness(0.0));
    }
}
