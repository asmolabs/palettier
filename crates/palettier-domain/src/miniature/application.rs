//! How paint goes on: consistency of a technique, drops of thinner, wet-on-wet compatibility.

use super::*;

/// How runny the paint of a technique is, from paste to wash.
///
/// Derived from [`Technique`], not a second list of techniques: the consistency follows
/// from what the technique does (a pin wash is a wash, OPR is a cream).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Consistency {
    Paste,
    Cream,
    Glaze,
    Wash,
}

impl Consistency {
    pub fn of(technique: Technique) -> Self {
        match technique {
            Technique::Highlight
            | Technique::DotFading
            | Technique::Drybrush
            | Technique::ContrastPaint => Self::Paste,
            Technique::Blending
            | Technique::OilRendering
            | Technique::BaseLayer
            | Technique::Layering
            | Technique::WetBlending
            | Technique::EdgeHighlight => Self::Cream,
            Technique::Glaze | Technique::Filter | Technique::StreakingGrime => Self::Glaze,
            Technique::OilWash | Technique::PinWash | Technique::ShadeWash => Self::Wash,
        }
    }

    pub fn label(&self, locale: Locale) -> &'static str {
        match self {
            Self::Paste => locale.tr("Pâte pure (empâtement)", "Pure paste (impasto)"),
            Self::Cream => locale.tr("Crème onctueuse (fondus)", "Creamy (blending)"),
            Self::Glaze => locale.tr("Glacis translucide", "Translucent glaze"),
            Self::Wash => locale.tr("Jus liquide (creux)", "Runny wash (recesses)"),
        }
    }

    pub fn description_i18n(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::Paste, Locale::Fr) => "Viscosité 100%. Idéal pour les arêtes vives, pointes de lumière spéculaire et textures en relief au cure-dent.",
            (Self::Paste, Locale::En) => "100% viscosity. Ideal for sharp edge highlights, specular catchlights, and thick textured relief.",
            (Self::Cream, Locale::Fr) => "Viscosité mayonnaise. Permet de poser des dégradés frais sur frais ultra-doux (Oil Paint Rendering) sans traces de pinceau.",
            (Self::Cream, Locale::En) => "Mayonnaise viscosity. Allows ultra-smooth wet-on-wet blending (Oil Paint Rendering) with zero brush marks.",
            (Self::Glaze, Locale::Fr) => "Viscosité lait. Teinte les surfaces en transparence tout en laissant transparaître les volumes et ombres de la sous-couche.",
            (Self::Glaze, Locale::En) => "Milk viscosity. Tints surfaces with rich translucency while preserving underlying zenithal shading.",
            (Self::Wash, Locale::Fr) => "Viscosité eau / thé fort. Tension superficielle quasi nulle : glisse instantanément dans les lignes de structure.",
            (Self::Wash, Locale::En) => "Water viscosity. Near-zero surface tension: wicks instantly into panel lines.",
        }
    }
}

/// Drops of thinner to add per part of paint (a part being a drop-sized dab), from the
/// technique's default medium ratio in `system`: a ratio r of medium means r / (1 - r)
/// parts of medium per part of paint. Zero when the technique uses paint as it comes.
pub fn thinner_drops_per_part(technique: Technique, system: PaintSystem) -> u32 {
    if technique.default_medium_in(system) == Medium::None {
        return 0;
    }
    let r = technique.default_ratio_in(system).clamp(0.0, 0.95);
    (r / (1.0 - r)).round() as u32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlendingRating {
    Excellent,
    Good,
    Caution,
    Difficult,
}

impl BlendingRating {
    pub fn badge_class(&self) -> &'static str {
        match self {
            Self::Excellent => "badge-success",
            Self::Good => "badge-info",
            Self::Caution => "badge-warning",
            Self::Difficult => "badge-danger",
        }
    }

    pub fn label(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::Excellent, Locale::Fr) => "🟢 Fondu Idéal (Friction Parfaite)",
            (Self::Excellent, Locale::En) => "🟢 Ideal Blend (Zero Resistance)",
            (Self::Good, Locale::Fr) => "🔵 Très Bon Fondu",
            (Self::Good, Locale::En) => "🔵 Very Good Blend",
            (Self::Caution, Locale::Fr) => "🟡 Attention : Risque d'Arrachement (Tearing)",
            (Self::Caution, Locale::En) => "🟡 Caution: Tearing / Drag Risk",
            (Self::Difficult, Locale::Fr) => "🔴 Incompatible dans le Frais",
            (Self::Difficult, Locale::En) => "🔴 Incompatible Wet-on-Wet",
        }
    }
}

pub struct BlendingEvaluation {
    pub rating: BlendingRating,
    pub traction_analysis: String,
    pub ideal_window_minutes: String,
    pub recommended_brush: String,
    pub medium_tweak_advice: String,
}

pub struct WetOnWetBlendingAdvisor;

impl WetOnWetBlendingAdvisor {
    /// How well a coat of `accent` blends into a still-wet coat of `base`.
    pub fn evaluate(base: Technique, accent: Technique, locale: Locale) -> BlendingEvaluation {
        match (Consistency::of(base), Consistency::of(accent)) {
            (Consistency::Cream, Consistency::Cream) => BlendingEvaluation {
                rating: BlendingRating::Excellent,
                traction_analysis: locale.tr(
                    "Les deux couches ont exactement la même viscosité beurrée. Le pinceau tire les pigments sans marquer de sillon.",
                    "Both layers have matching buttery viscosity. The brush glides seamlessly without furrowing."
                ).to_string(),
                ideal_window_minutes: locale.tr("Immédiat jusqu'à 2 heures après la pose", "Immediate up to 2 hours after laydown").to_string(),
                recommended_brush: locale.tr("Pinceau langue de chat Kolinsky #1 ou estompe mousse silicone", "Kolinsky filbert #1 or silicone colour shaper").to_string(),
                medium_tweak_advice: locale.tr("Aucun additif requis : travaillez directement dans le frais.", "No additives needed: blend directly into the wet layer.").to_string(),
            },
            (Consistency::Paste, Consistency::Paste) => BlendingEvaluation {
                rating: BlendingRating::Good,
                traction_analysis: locale.tr(
                    "Pâte dense contre pâte dense : fondu très couvrant mais demande un pinceau ferme pour adoucir la frontière.",
                    "Dense paste against dense paste: high coverage but requires a firmer brush to soften the seam."
                ).to_string(),
                ideal_window_minutes: locale.tr("Dans les 30 à 45 minutes", "Within 30 to 45 minutes").to_string(),
                recommended_brush: locale.tr("Pinceau plat biseauté en martre ou poils synthétiques fermes", "Flat angled sable or firm synthetic brush").to_string(),
                medium_tweak_advice: locale.tr("Effleurer le pinceau sur une micro-goutte de Sansodor pour lubrifier la traction.", "Touch brush tip to a micro-drop of odorless spirit to lubricate stroke.").to_string(),
            },
            (Consistency::Cream, Consistency::Paste) => BlendingEvaluation {
                rating: BlendingRating::Excellent,
                traction_analysis: locale.tr(
                    "Règle d'or de l'OPR : poser la lumière en pâte plus dense sur la crème de base. L'accroche est immédiate et ne bave pas.",
                    "Golden rule of OPR: lay firmer highlight paste over softer base cream. Grip is immediate with zero bleeding."
                ).to_string(),
                ideal_window_minutes: locale.tr("Attendre 10 min après la base, puis tirer pendant 90 min", "Wait 10 min after base, then blend for 90 min").to_string(),
                recommended_brush: locale.tr("Pinceau rond pointu #00 pour poser, puis langue de chat propre pour fondre", "Pointed round #00 to dab, clean filbert to feather").to_string(),
                medium_tweak_advice: locale.tr("La pâte pure en rehaut doit rester non diluée.", "Keep highlight paste undiluted for maximum opacity.").to_string(),
            },
            (Consistency::Paste, Consistency::Glaze)
            | (Consistency::Cream, Consistency::Glaze) => BlendingEvaluation {
                rating: BlendingRating::Caution,
                traction_analysis: locale.tr(
                    "⚠️ Risque de dilution inversée : le solvant du glacis peut dissoudre et creuser la couche de base fraîche.",
                    "⚠️ Washout hazard: glaze solvent may dissolve and pit the fresh base coat."
                ).to_string(),
                ideal_window_minutes: locale.tr("Déconseillé dans le frais : attendre le séchage au toucher de la base (24h) !", "Avoid wet-on-wet: wait for base to be touch-dry (24h)!").to_string(),
                recommended_brush: locale.tr("Pinceau lavis très doux sans frotter", "Ultra-soft mop brush with feather touch").to_string(),
                medium_tweak_advice: locale.tr("Si urgent, utiliser du Liquin pur sans solvant pour le glacis pour éviter d'attaquer la base.", "If urgent, use pure Liquin without spirits to glaze over without dissolving base.").to_string(),
            },
            (Consistency::Glaze, Consistency::Glaze) => BlendingEvaluation {
                rating: BlendingRating::Good,
                traction_analysis: locale.tr(
                    "Fusion de deux filtres optiques : transition par capillarité et transparence.",
                    "Two optical glazes merging: transition happens via capillarity and translucency."
                ).to_string(),
                ideal_window_minutes: locale.tr("Dans les 15 minutes (avant évaporation du solvant)", "Within 15 minutes (before spirit flash-off)").to_string(),
                recommended_brush: locale.tr("Pinceau rond souple #1", "Soft round #1 brush").to_string(),
                medium_tweak_advice: locale.tr("Ajouter 1 goutte de Liquin pour retarder l'évaporation et lustrer le glacis.", "Add 1 drop Liquin to extend working time and boost gloss transparency.").to_string(),
            },
            _ => BlendingEvaluation {
                rating: BlendingRating::Difficult,
                traction_analysis: locale.tr(
                    "Incompatibilité de viscosité : un jus très liquide posé sur une pâte fraîche arrache les pigments et fait des auréoles.",
                    "Viscosity clash: a watery wash over wet paste tears pigment apart and leaves tide marks."
                ).to_string(),
                ideal_window_minutes: locale.tr("Attendre que la première couche soit polymérisée (24h à 48h)", "Wait for first layer to polymerise (24h to 48h)").to_string(),
                recommended_brush: locale.tr("Pinceau doux de tapotement", "Stippling soft brush").to_string(),
                medium_tweak_advice: locale.tr("Respectez la règle Gras sur Maigre et laissez sécher entre les étapes.", "Respect Fat-over-Lean and let cure between layers.").to_string(),
            },
        }
    }
}
