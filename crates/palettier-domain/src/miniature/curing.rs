//! Drying on the palette and in the box (heat box, freezer, freshness), read from the shared drying model.

use super::*;

/// Heat Box / Curing Chamber / Dehydrator temperature calculator.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HeatBoxCalculator {
    pub temperature_celsius: f64,
}

impl HeatBoxCalculator {
    pub fn new(celsius: f64) -> Self {
        Self {
            temperature_celsius: celsius.clamp(18.0, 60.0),
        }
    }

    /// How much faster an oil coat dries in the box than at 20 degrees, read from the
    /// application's drying model (the same rule as the Drying screen and the workbench).
    pub fn acceleration_factor(&self) -> f64 {
        let at = |celsius: f64| oil_dab(celsius, Ventilation::Normal).touch_dry.as_secs_f64();
        at(20.0) / at(self.temperature_celsius).max(1.0)
    }

    /// Converts standard 20°C ambient hours into accelerated chamber drying time.
    pub fn accelerated_hours(&self, standard_hours: f64) -> f64 {
        let factor = self.acceleration_factor();
        if factor <= 0.0 { standard_hours } else { standard_hours / factor }
    }

    /// Safety threshold for miniature plastics (GW High Impact Polystyrene & Finecast resin).
    pub fn is_safe_for_miniature_plastics(&self) -> bool {
        self.temperature_celsius <= 45.0
    }

    pub fn safety_warning_i18n(&self, locale: Locale) -> Option<&'static str> {
        if self.temperature_celsius > 48.0 {
            Some(match locale {
                Locale::Fr => "⚠️ DANGER PLASTIQUE/RÉSINE : À plus de 48°C, le plastique polystyrène et la résine Finecast ramollissent et se déforment irrémédiablement !",
                Locale::En => "⚠️ PLASTIC/RESIN DANGER: Above 48°C, polystyrene and Finecast resin soften and permanently warp!",
            })
        } else if self.temperature_celsius > 45.0 {
            Some(match locale {
                Locale::Fr => "⚠️ Prudence : Température limite pour les pièces très fines (épées, lances). Restez sous 42°C par précaution.",
                Locale::En => "⚠️ Caution: Borderline heat for thin parts (swords, spears). Maintain below 42°C as safety buffer.",
            })
        } else {
            None
        }
    }
}

/// Preservation of mixed oil palettes using airtight boxes and freezer temperatures.
pub struct FreezerStorageSimulator;

impl FreezerStorageSimulator {
    /// Days before a dab of oil on the palette skins over, from the drying model: a
    /// sealed box cuts the air flow, the freezer (-18 degrees) slows oxidation further.
    pub fn estimated_usable_days(ambient_temp_c: f64, is_airtight: bool, is_in_freezer: bool) -> f64 {
        let temperature = if is_in_freezer { -18.0 } else { ambient_temp_c };
        let ventilation = if is_airtight || is_in_freezer {
            Ventilation::Confined
        } else {
            Ventilation::Normal
        };
        oil_dab(temperature, ventilation).touch_dry.as_secs_f64() / 86_400.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FreshnessPhase {
    FreshCream,
    ButteryOpr,
    TackySkinning,
    Exhausted,
}

impl FreshnessPhase {
    pub fn badge_class(&self) -> &'static str {
        match self {
            Self::FreshCream => "badge-success",
            Self::ButteryOpr => "badge-info",
            Self::TackySkinning => "badge-warning",
            Self::Exhausted => "badge-danger",
        }
    }

    pub fn title_i18n(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::FreshCream, Locale::Fr) => "🟢 Crème fraîche (phase fluide)",
            (Self::FreshCream, Locale::En) => "🟢 Fresh cream (fluid stage)",
            (Self::ButteryOpr, Locale::Fr) => "🟡 Beurre onctueux (phase OPR)",
            (Self::ButteryOpr, Locale::En) => "🟡 Buttery paste (OPR stage)",
            (Self::TackySkinning, Locale::Fr) => "🟠 Peau de polymérisation",
            (Self::TackySkinning, Locale::En) => "🟠 Tacky skinning stage",
            (Self::Exhausted, Locale::Fr) => "🔴 Pâte épuisée / filmée",
            (Self::Exhausted, Locale::En) => "🔴 Exhausted / cured film",
        }
    }

    pub fn physical_state_i18n(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::FreshCream, Locale::Fr) => "Le solvant est actif, les huiles sont fluides et malléables. Parfait pour lavis capillaires, pin-washes et étalement rapide.",
            (Self::FreshCream, Locale::En) => "Solvent is active, oils are fluid and smooth. Optimal for pin washes, capillary lining, and quick spreading.",
            (Self::ButteryOpr, Locale::Fr) => "Le solvant s'est évaporé en surface, laissant la pâte pure et onctueuse. C'est le moment d'or pour le fondu dans le frais (wet-on-wet) et les dégradés sans trace de pinceau.",
            (Self::ButteryOpr, Locale::En) => "Solvent has flashed off, leaving pure buttery paste. Prime time for wet-on-wet blending and velvety smooth transitions.",
            (Self::TackySkinning, Locale::Fr) => "L'oxygène a commencé à polymériser le liant d'huile. La pâte devient collante et offre une résistance au pinceau. Attention au risque d'arrachement.",
            (Self::TackySkinning, Locale::En) => "Oxygen polymerization has started. Paste turns tacky and drags on the miniature. High risk of brush tearing.",
            (Self::Exhausted, Locale::Fr) => "Une pellicule durcie s'est formée. Ne tentez PAS de diluer au solvant (risque de grumeaux gélifiés). Gratter la palette ou percer sous la peau pour récupérer le cœur frais.",
            (Self::Exhausted, Locale::En) => "A hardened skin has formed over the dollop. Do NOT stir in solvent (causes curdling & lumps). Scrape off or pierce beneath the skin.",
        }
    }

    pub fn action_advice_i18n(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::FreshCream, Locale::Fr) => "Travaillez vos ombrages d'occlusion et vos sous-couches sans attendre.",
            (Self::FreshCream, Locale::En) => "Apply your occlusion washes and initial block-ins now.",
            (Self::ButteryOpr, Locale::Fr) => "Idéal pour poser et fondre les lumières. Aucun ajout de solvant nécessaire.",
            (Self::ButteryOpr, Locale::En) => "Ideal for laying and blending highlights. No solvent addition needed.",
            (Self::TackySkinning, Locale::Fr) => "Ajoutez une micro-goutte de Sansodor à côté du plot (pas dedans) pour détendre la pointe du pinceau.",
            (Self::TackySkinning, Locale::En) => "Place a micro-drop of odorless spirit next to the paint dollop to soften the brush tip.",
            (Self::Exhausted, Locale::Fr) => "Placez votre palette hermétique au congélateur (-18°C) pour stopper la polymérisation jusqu'à demain.",
            (Self::Exhausted, Locale::En) => "Put your sealed palette into the freezer (-18°C) to freeze polymerization until tomorrow.",
        }
    }
}

pub struct LivePaletteFreshnessTracker;

impl LivePaletteFreshnessTracker {
    /// Phase of a dab of oil put on the palette `elapsed_minutes` ago, from the drying
    /// model: fluid during the first half of its open time, buttery until the end of it,
    /// tacky until touch dry, then skinned over.
    pub fn evaluate(elapsed_minutes: u32, temp_c: f64) -> FreshnessPhase {
        let estimate = oil_dab(temp_c, Ventilation::Normal);
        let elapsed = std::time::Duration::from_secs(u64::from(elapsed_minutes) * 60);
        if elapsed <= estimate.open_time / 2 {
            FreshnessPhase::FreshCream
        } else if elapsed <= estimate.open_time {
            FreshnessPhase::ButteryOpr
        } else if elapsed <= estimate.touch_dry {
            FreshnessPhase::TackySkinning
        } else {
            FreshnessPhase::Exhausted
        }
    }
}

/// Drying of a dab of oil paint left on the palette (thick, no medium, average pigment),
/// at `celsius` with the given air flow: the shared base of the heat box, freezer and
/// freshness tools, so they agree with the Drying screen and the palette mixes.
fn oil_dab(celsius: f64, ventilation: Ventilation) -> DryingEstimate {
    DryingTimeService::default().estimate(&DryingContext {
        system: PaintSystem::Oil,
        drying_class: DryingClass::Medium,
        medium: Medium::None,
        medium_ratio: 0.0,
        thickness: LayerThickness::Thick,
        workshop: Workshop {
            temperature_celsius: celsius,
            relative_humidity: Workshop::standard().relative_humidity,
            ventilation,
        },
    })
}
