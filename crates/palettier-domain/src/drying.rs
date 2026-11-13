use crate::error::{DomainError, DomainResult};
use crate::i18n::Locale;
use crate::paint::{DryingClass, LayerThickness, Medium, PaintSystem, Technique, Ventilation};
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
    pub fn new(
        name: impl Into<String>,
        oil: f64,
        solvent: f64,
        resin: f64,
        drier_drops: u32,
    ) -> Self {
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
    /// Oil or acrylic: the two systems do not share a drying model.
    pub system: PaintSystem,
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
            system: medium.system().unwrap_or_default(),
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
        Self::for_technique_in(
            technique,
            technique.system_within(PaintSystem::Oil),
            drying_class,
            workshop,
        )
    }

    /// Default settings of `technique` when done in `system`.
    pub fn for_technique_in(
        technique: Technique,
        system: PaintSystem,
        drying_class: DryingClass,
        workshop: Workshop,
    ) -> Self {
        let system = technique.system_within(system);
        Self {
            system,
            drying_class,
            medium: technique.default_medium_in(system),
            medium_ratio: technique.default_ratio_in(system),
            thickness: technique.typical_thickness(),
            workshop,
        }
    }

    /// Same context, painted in another system.
    pub fn with_system(mut self, system: PaintSystem) -> Self {
        self.system = system;
        self
    }
}

/// Whether a coat of `over` must wait for the coat below to be dry through, rather
/// than simply recoatable.
///
/// - oil over oil: the technique decides (a glaze or a wash needs a cured base);
/// - oil over acrylic: yes, the acrylic must be dry through (and ideally varnished),
///   or the solvent lifts it;
/// - acrylic over anything: an acrylic coat only waits for the recoat time. Acrylic
///   over oil is a mistake of its own, reported by the fat-over-lean inspection.
pub fn waits_for_through_dry(
    under: PaintSystem,
    over: PaintSystem,
    over_technique: Technique,
) -> bool {
    match (under, over) {
        (PaintSystem::Oil, PaintSystem::Oil) => over_technique.requires_cured_base(),
        (PaintSystem::Acrylic, PaintSystem::Oil) => true,
        (_, PaintSystem::Acrylic) => false,
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

/// Parameters of the acrylic drying model.
///
/// Acrylic dries by evaporation of water, not by oxidation: the pigment barely matters,
/// the thickness, the humidity and the air flow do. Times are for a normal coat at
/// 20 degrees and 50 % humidity.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AcrylicDryingProperties {
    pub reference_touch_dry_minutes: f64,
    pub humidity_sensitivity: f64,
    pub open_time_share: f64,
    pub recoat_factor: f64,
    pub through_dry_factor: f64,
    pub full_cure_factor: f64,
}

impl Default for AcrylicDryingProperties {
    fn default() -> Self {
        Self {
            reference_touch_dry_minutes: 15.0,
            humidity_sensitivity: 1.5,
            open_time_share: 0.30,
            recoat_factor: 2.0,
            // 15 min -> 12 h through dry, about 3 days before a final varnish.
            through_dry_factor: 48.0,
            full_cure_factor: 300.0,
        }
    }
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

/// Estimates drying milestones for a coat, with the model of its system (oil or acrylic).
#[derive(Debug, Clone, Default)]
pub struct DryingTimeService {
    properties: DryingProperties,
    acrylic: AcrylicDryingProperties,
}

impl DryingTimeService {
    pub fn new(properties: DryingProperties) -> Self {
        Self {
            properties,
            acrylic: AcrylicDryingProperties::default(),
        }
    }

    pub fn estimate(&self, context: &DryingContext) -> DryingEstimate {
        match context.system {
            PaintSystem::Oil => self.estimate_oil(context),
            PaintSystem::Acrylic => self.estimate_acrylic(context),
        }
    }

    fn estimate_acrylic(&self, context: &DryingContext) -> DryingEstimate {
        let a = &self.acrylic;
        // An oil medium has no business in acrylic: ignore it rather than slow the coat down.
        let medium_factor = if context.medium.fits(PaintSystem::Acrylic) {
            context.medium.drying_factor(context.medium_ratio)
        } else {
            1.0
        };
        let humidity = {
            let delta = context.workshop.relative_humidity - self.properties.reference_humidity;
            (1.0 + delta / 100.0 * a.humidity_sensitivity).max(0.4)
        };
        let hours = (a.reference_touch_dry_minutes / 60.0)
            * medium_factor
            * context.thickness.factor()
            * self.temperature_factor(context.workshop.temperature_celsius)
            * humidity
            * context.workshop.ventilation.factor();

        DryingEstimate {
            open_time: Self::hours_to_duration(hours * a.open_time_share),
            touch_dry: Self::hours_to_duration(hours),
            recoat: Self::hours_to_duration(hours * a.recoat_factor),
            through_dry: Self::hours_to_duration(hours * a.through_dry_factor),
            full_cure: Self::hours_to_duration(hours * a.full_cure_factor),
            advice: Self::acrylic_advice(context),
        }
    }

    fn acrylic_advice(context: &DryingContext) -> Vec<String> {
        let mut out = Vec::new();
        let workshop = &context.workshop;
        if !context.medium.fits(PaintSystem::Acrylic) {
            out.push(format!(
                "{} ne se melange pas a l'acrylique : diluez a l'eau ou avec un medium acrylique.",
                context.medium.label()
            ));
        } else if context.medium != Medium::None
            && context.medium_ratio > context.medium.max_useful_ratio()
        {
            let max_pct = (context.medium.max_useful_ratio() * 100.0).round() as i64;
            out.push(format!(
                "Proportion de {} au-dela de la limite utile ({} %) : le liant acrylique ne tient plus le pigment.",
                context.medium.label().to_lowercase(),
                max_pct
            ));
        }
        if workshop.temperature_celsius < 10.0 {
            out.push(
                "Sous 10 degres, l'acrylique filme mal : la couche peut rester laiteuse ou craqueler."
                    .to_string(),
            );
        }
        if workshop.temperature_celsius > 28.0 || workshop.relative_humidity < 35.0 {
            out.push(
                "Air chaud ou sec : la peinture seche sur la palette et au bout du pinceau. Palette humide conseillee."
                    .to_string(),
            );
        }
        if workshop.relative_humidity > 70.0 {
            out.push(
                "Humidite elevee : la couche reste molle plus longtemps, attendez avant le lavis suivant."
                    .to_string(),
            );
        }
        if context.thickness >= LayerThickness::Thick {
            out.push(
                "Couche epaisse : l'acrylique se retracte en sechant et perd les details. Preferez deux couches fines."
                    .to_string(),
            );
        }
        if context.medium.fits(PaintSystem::Acrylic) {
            out.push(context.medium.advice().to_string());
        }
        out
    }

    fn estimate_oil(&self, context: &DryingContext) -> DryingEstimate {
        let base_minutes = (context.drying_class.reference_touch_dry().as_secs() / 60) as f64;
        let medium_factor = if context.medium.fits(PaintSystem::Oil) {
            context.medium.drying_factor(context.medium_ratio)
        } else {
            1.0
        };
        let hours = (base_minutes / 60.0)
            * medium_factor
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

        if !context.medium.fits(PaintSystem::Oil) {
            out.push(format!(
                "{} ne se melange pas a l'huile : diluez au diluant inodore ou avec un medium a l'huile.",
                context.medium.label()
            ));
            return out;
        }
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

#[cfg(test)]
mod acrylic_tests {
    use super::*;

    fn acrylic(technique: Technique) -> DryingEstimate {
        DryingTimeService::default().estimate(&DryingContext::for_technique_in(
            technique,
            PaintSystem::Acrylic,
            DryingClass::VerySlow,
            Workshop::standard(),
        ))
    }

    #[test]
    fn acrylic_layer_is_touch_dry_in_minutes_whatever_the_pigment() {
        let layer = acrylic(Technique::Layering);
        assert!(
            layer.touch_dry < Duration::from_secs(30 * 60),
            "{:?}",
            layer.touch_dry
        );
        let fast = DryingTimeService::default().estimate(&DryingContext::for_technique_in(
            Technique::Layering,
            PaintSystem::Acrylic,
            DryingClass::Fast,
            Workshop::standard(),
        ));
        assert_eq!(
            layer.touch_dry, fast.touch_dry,
            "le pigment ne compte pas en acrylique"
        );
    }

    #[test]
    fn same_glaze_dries_far_faster_in_acrylic_than_in_oil() {
        let oil = DryingTimeService::default().estimate(&DryingContext::for_technique_in(
            Technique::Glaze,
            PaintSystem::Oil,
            DryingClass::Medium,
            Workshop::standard(),
        ));
        let acr = acrylic(Technique::Glaze);
        assert!(
            acr.recoat * 20 < oil.recoat,
            "{:?} vs {:?}",
            acr.recoat,
            oil.recoat
        );
    }

    #[test]
    fn retarder_extends_acrylic_open_time() {
        let service = DryingTimeService::default();
        let base = DryingContext::new(
            DryingClass::Medium,
            Medium::Water,
            0.3,
            LayerThickness::Thin,
            Workshop::standard(),
        )
        .unwrap();
        let slowed = DryingContext {
            medium: Medium::Retarder,
            medium_ratio: 0.15,
            ..base
        };
        assert_eq!(PaintSystem::Acrylic, base.system);
        assert!(service.estimate(&slowed).open_time > service.estimate(&base).open_time);
    }

    #[test]
    fn oil_medium_in_acrylic_is_reported_not_applied() {
        let ctx = DryingContext::for_technique_in(
            Technique::Layering,
            PaintSystem::Acrylic,
            DryingClass::Medium,
            Workshop::standard(),
        );
        let wrong = DryingContext {
            medium: Medium::LinseedOil,
            medium_ratio: 0.3,
            ..ctx
        };
        let est = DryingTimeService::default().estimate(&wrong);
        assert!(est.advice.iter().any(|a| a.contains("ne se melange pas")));
    }

    #[test]
    fn waiting_rules_between_systems() {
        // acrylic over acrylic: recoat only, even for a glaze
        assert!(!waits_for_through_dry(
            PaintSystem::Acrylic,
            PaintSystem::Acrylic,
            Technique::Glaze
        ));
        // oil over acrylic: the base must be dry through
        assert!(waits_for_through_dry(
            PaintSystem::Acrylic,
            PaintSystem::Oil,
            Technique::Blending
        ));
        // oil over oil: as before, the technique decides
        assert!(waits_for_through_dry(
            PaintSystem::Oil,
            PaintSystem::Oil,
            Technique::Glaze
        ));
        assert!(!waits_for_through_dry(
            PaintSystem::Oil,
            PaintSystem::Oil,
            Technique::BaseLayer
        ));
    }
}
