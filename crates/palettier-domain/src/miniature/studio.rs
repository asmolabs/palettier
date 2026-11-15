//! Studio care: brushes, varnishes, solvents and stripping.

use super::*;

/// Kolinsky sable brush care and maintenance tracker.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BrushTracker {
    pub name: String,
    pub brand: String,
    pub size: String,
    pub hair_type: String,
    pub uses_count: u32,
}

impl BrushTracker {
    pub fn new(name: impl Into<String>, brand: impl Into<String>, size: impl Into<String>, hair_type: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            brand: brand.into(),
            size: size.into(),
            hair_type: hair_type.into(),
            uses_count: 0,
        }
    }

    pub fn default_sable_kit() -> Vec<Self> {
        vec![
            Self::new("Pinceau de Précision", "Winsor & Newton Series 7", "#00", "Martre Kolinsky"),
            Self::new("Pinceau Maître Établi", "Raphael 8404", "#1", "Martre Kolinsky"),
            Self::new("Brosse Fondus & Lavis", "Artis Opus Series D", "Small", "Brosse estompe"),
            Self::new("Pinceau Biseauté Travail Réductif", "Da Vinci Maestro", "#2", "Martre synthétique"),
        ]
    }

    pub fn needs_deep_clean(&self) -> bool {
        self.uses_count >= 8
    }

    pub fn safety_advice_i18n(&self, locale: Locale) -> &'static str {
        match locale {
            Locale::Fr => "Ne JAMAIS immerger la virole métallique dans l'essence minérale ou le solvant. Le solvant dissout la colle tenant les poils.",
            Locale::En => "NEVER immerse the metal ferrule into mineral spirits. Solvent dissolves the internal glue binding the bristles.",
        }
    }
}

/// Varnish finish simulator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VarnishFinish {
    RawStudio,
    UltraMatte,
    Satin,
    Gloss,
}

impl VarnishFinish {
    pub const ALL: [VarnishFinish; 4] = [
        VarnishFinish::RawStudio,
        VarnishFinish::UltraMatte,
        VarnishFinish::Satin,
        VarnishFinish::Gloss,
    ];

    pub fn label(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::RawStudio, Locale::Fr) => "Fini Brut d'Atelier (Hétérogène)",
            (Self::RawStudio, Locale::En) => "Raw Studio Finish (Uneven)",
            (Self::UltraMatte, Locale::Fr) => "Vernis Ultra-Mat (AK Ultra Matte)",
            (Self::UltraMatte, Locale::En) => "Ultra-Matte Varnish (AK Ultra Matte)",
            (Self::Satin, Locale::Fr) => "Vernis Satiné d'Exposition",
            (Self::Satin, Locale::En) => "Exhibition Satin Varnish",
            (Self::Gloss, Locale::Fr) => "Vernis Brillant Spéculaire",
            (Self::Gloss, Locale::En) => "Specular Gloss Varnish",
        }
    }

    pub fn description_i18n(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::RawStudio, Locale::Fr) => "Contraste de brillance indésirable : acrylique mat, lavis d'huile satiné dans les creux.",
            (Self::RawStudio, Locale::En) => "Patchy studio finish: chalky matte acrylic base with semi-gloss oil traces in recesses.",
            (Self::UltraMatte, Locale::Fr) => "Supprime tout reflet parasite. Unifie les fondus et donne un aspect musée sans aucune brillance sous la lumière.",
            (Self::UltraMatte, Locale::En) => "Eliminates all stray reflections. Harmonizes all blends with a museum-grade dead flat finish.",
            (Self::Satin, Locale::Fr) => "Conserve l'intensité chromatique tout en protégeant la figurine des manipulations de jeu.",
            (Self::Satin, Locale::En) => "Preserves high color saturation and richness while safeguarding the piece during gaming.",
            (Self::Gloss, Locale::Fr) => "Maximise la profondeur des noirs (rehaussage des ombres) et la réflexion spéculaire (eau, sang, yeux).",
            (Self::Gloss, Locale::En) => "Maximizes deep black depth and intense specular reflections (water, fresh blood, lenses).",
        }
    }
}

/// Solvents used for miniature oil washes, reductive technique, and blending.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SolventType {
    Sansodor,
    Gamsol,
    OdorlessWhiteSpirit,
    PureGumTurpentine,
}

impl SolventType {
    pub const ALL: [Self; 4] = [
        Self::Sansodor,
        Self::Gamsol,
        Self::OdorlessWhiteSpirit,
        Self::PureGumTurpentine,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Self::Sansodor => "Sansodor (Winsor & Newton)",
            Self::Gamsol => "Gamsol OMS (Gamblin)",
            Self::OdorlessWhiteSpirit => "White Spirit Désaromatisé / Ecosolv",
            Self::PureGumTurpentine => "Essence de Térébenthine Pure Gomme",
        }
    }

    pub fn evaporation_speed_i18n(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::Sansodor, Locale::Fr) => "Lente (2 à 4 heures) - Temps de manipulation maximal",
            (Self::Sansodor, Locale::En) => "Slow (2 to 4 hours) - Maximum working open window",
            (Self::Gamsol, Locale::Fr) => "Modérée (45 à 90 minutes) - Évaporation propre et neutre",
            (Self::Gamsol, Locale::En) => "Moderate (45 to 90 min) - Clean, neutral evaporation",
            (Self::OdorlessWhiteSpirit, Locale::Fr) => "Rapide (20 à 40 minutes) - Établi rapide",
            (Self::OdorlessWhiteSpirit, Locale::En) => "Fast (20 to 40 min) - Quick studio sessions",
            (Self::PureGumTurpentine, Locale::Fr) => "Moyenne - Forte réactivité chimique",
            (Self::PureGumTurpentine, Locale::En) => "Medium - Highly reactive solvent",
        }
    }

    pub fn acrylic_safety_level_i18n(&self, locale: Locale) -> (&'static str, &'static str) {
        match (self, locale) {
            (Self::Sansodor, Locale::Fr) => ("Excellente (Inoffensif sur acrylique sec)", "ok"),
            (Self::Sansodor, Locale::En) => ("Excellent (Safe on cured acrylics)", "ok"),
            (Self::Gamsol, Locale::Fr) => ("Excellente (Pureté 100% OMS)", "ok"),
            (Self::Gamsol, Locale::En) => ("Excellent (100% Pure OMS)", "ok"),
            (Self::OdorlessWhiteSpirit, Locale::Fr) => ("Bonne (Éviter de frotter agressivement)", "warn"),
            (Self::OdorlessWhiteSpirit, Locale::En) => ("Good (Avoid aggressive mechanical scrubbing)", "warn"),
            (Self::PureGumTurpentine, Locale::Fr) => ("DANGEREUX (Attaque le liant acrylique)", "danger"),
            (Self::PureGumTurpentine, Locale::En) => ("DANGEROUS (Attacks cured acrylic binder)", "danger"),
        }
    }

    pub fn odor_and_toxicity_i18n(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::Sansodor, Locale::Fr) => "Inodore, très faible taux d'aromatiques (<0.1%)",
            (Self::Sansodor, Locale::En) => "Odorless, minimal aromatics (<0.1%)",
            (Self::Gamsol, Locale::Fr) => "100% sans odeur, solvant le plus sûr en intérieur",
            (Self::Gamsol, Locale::En) => "100% odorless, safest studio solvent for indoors",
            (Self::OdorlessWhiteSpirit, Locale::Fr) => "Odeur très discrète, aération standard conseillée",
            (Self::OdorlessWhiteSpirit, Locale::En) => "Mild scent, standard ventilation recommended",
            (Self::PureGumTurpentine, Locale::Fr) => "Odeur de pin intense, vapeurs nocives, aération forte obligatoire",
            (Self::PureGumTurpentine, Locale::En) => "Pungent pine fumes, toxic vapours, strong ventilation required",
        }
    }
}

/// Miniature material to strip.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MiniatureMaterial {
    PolystyrenePlastic,
    ResinSla3d,
    WhiteMetal,
}

impl MiniatureMaterial {
    pub const ALL: [Self; 3] = [
        Self::PolystyrenePlastic,
        Self::ResinSla3d,
        Self::WhiteMetal,
    ];

    pub fn label(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::PolystyrenePlastic, Locale::Fr) => "Plastique Polystyrène (Games Workshop, Gunpla)",
            (Self::PolystyrenePlastic, Locale::En) => "Polystyrene Plastic (Games Workshop, Gunpla)",
            (Self::ResinSla3d, Locale::Fr) => "Résine (Forge World, Impression 3D SLA / DLP)",
            (Self::ResinSla3d, Locale::En) => "Resin (Forge World, 3D Printed SLA / DLP Photopolymer)",
            (Self::WhiteMetal, Locale::Fr) => "Métal Blanc / Étain (Old-school, Infinity)",
            (Self::WhiteMetal, Locale::En) => "White Metal / Pewter (Old-school, Infinity)",
        }
    }
}

/// Chemical stripping agent for paint removal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrippingAgent {
    Ipa99,
    Biostrip20,
    Glanzer,
    PureAcetone,
}

impl StrippingAgent {
    pub const ALL: [Self; 4] = [
        Self::Ipa99,
        Self::Biostrip20,
        Self::Glanzer,
        Self::PureAcetone,
    ];

    pub fn label(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::Ipa99, Locale::Fr) => "Alcool Isopropylique 99.9% (IPA)",
            (Self::Ipa99, Locale::En) => "99.9% Isopropyl Alcohol (IPA)",
            (Self::Biostrip20, Locale::Fr) => "Biostrip 20 (Gel aqueux biodégradable)",
            (Self::Biostrip20, Locale::En) => "Biostrip 20 (Water-based stripping gel)",
            (Self::Glanzer, Locale::Fr) => "Glanzer Décapant Sols (Ammoniaqué)",
            (Self::Glanzer, Locale::En) => "Glanzer Floor Stripper",
            (Self::PureAcetone, Locale::Fr) => "Acétone Pure (Solvant cétonique puissant)",
            (Self::PureAcetone, Locale::En) => "Pure Acetone (Aggressive ketone solvent)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrippingVerdictSeverity {
    Safe,
    Caution,
    Forbidden,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StrippingVerdict {
    pub severity: StrippingVerdictSeverity,
    pub title: &'static str,
    pub advice_fr: &'static str,
    pub advice_en: &'static str,
    pub recommended_duration: &'static str,
}

pub struct StrippingGuide;

impl StrippingGuide {
    pub fn evaluate(material: MiniatureMaterial, agent: StrippingAgent) -> StrippingVerdict {
        match (material, agent) {
            (MiniatureMaterial::PolystyrenePlastic, StrippingAgent::PureAcetone) => StrippingVerdict {
                severity: StrippingVerdictSeverity::Forbidden,
                title: "DANGER ABSOLU : FONTE DU PLASTIQUE",
                advice_fr: "L'acétone dissout instantanément les polymères de styrène. Votre figurine sera transformée en bouillie irrécupérable en moins de 30 secondes !",
                advice_en: "Acetone dissolves polystyrene immediately into mush. Your miniature will be permanently destroyed within 30 seconds!",
                recommended_duration: "0 sec (STRICTEMENT INTERDIT)",
            },
            (MiniatureMaterial::ResinSla3d, StrippingAgent::PureAcetone) => StrippingVerdict {
                severity: StrippingVerdictSeverity::Forbidden,
                title: "INTERDIT : RAMOLLISSEMENT IRRÉVERSIBLE",
                advice_fr: "L'acétone gonfle la résine, détruit les liaisons monomères et fait fondre les détails fins.",
                advice_en: "Acetone rapidly degrades resin polymers, dissolving fine details and causing irreversible rubbery warping.",
                recommended_duration: "0 sec (STRICTEMENT INTERDIT)",
            },
            (MiniatureMaterial::PolystyrenePlastic, StrippingAgent::Ipa99) => StrippingVerdict {
                severity: StrippingVerdictSeverity::Safe,
                title: "PARFAIT & SANS DANGER",
                advice_fr: "Le plastique supporte des bains de plusieurs jours sans aucune altération. Brosser avec une brosse à dents souple sous l'eau tiède.",
                advice_en: "Polystyrene plastic is completely immune to IPA. Submerge for hours to overnight, then scrub with a soft toothbrush.",
                recommended_duration: "2h à 24h",
            },
            (MiniatureMaterial::ResinSla3d, StrippingAgent::Ipa99) => StrippingVerdict {
                severity: StrippingVerdictSeverity::Caution,
                title: "ATTENTION : BAINS COURTS UNIQUEMENT",
                advice_fr: "La résine absorbe l'alcool au-delà de 20 minutes, devenant caoutchouteuse et cassante. Tremper 10-15 min max, brosser délicatement et sécher immédiatement.",
                advice_en: "Resin absorbs alcohol past 20 minutes, turning rubbery and brittle. Limit soak to 10-15 min max, brush gently and dry immediately.",
                recommended_duration: "10 à 15 minutes max",
            },
            (MiniatureMaterial::WhiteMetal, StrippingAgent::PureAcetone) => StrippingVerdict {
                severity: StrippingVerdictSeverity::Safe,
                title: "LE PLUS RAPIDE & ULTRA-EFFICACE",
                advice_fr: "Le métal est inerte à l'acétone. La peinture et les sous-couches les plus dures se décollent en lambeaux en 5 à 10 minutes.",
                advice_en: "White metal is 100% inert to acetone. Acrylics, enamels, and tough primers fall off cleanly within 5 to 10 minutes.",
                recommended_duration: "5 à 15 minutes",
            },
            (_, StrippingAgent::Biostrip20) => StrippingVerdict {
                severity: StrippingVerdictSeverity::Safe,
                title: "DOUX & UNIVERSEL",
                advice_fr: "Gel aqueux inodore sans danger pour le plastique, la résine et le métal. Appliquer au pinceau, laisser agir sous film plastique.",
                advice_en: "Water-based biodegradable gel safe across plastic, resin, and metal. Coat thoroughly, leave under plastic wrap.",
                recommended_duration: "30 min à 2h",
            },
            (MiniatureMaterial::PolystyrenePlastic, StrippingAgent::Glanzer) => StrippingVerdict {
                severity: StrippingVerdictSeverity::Safe,
                title: "TRÈS BON DÉCAPANT ÉCO",
                advice_fr: "Dissout les acryliques sans attaquer le plastique. Odeur ammoniaquée (bien aérer).",
                advice_en: "Dissolves acrylics without affecting plastic. Strong ammonia odor (ventilate).",
                recommended_duration: "4h à 12h",
            },
            (MiniatureMaterial::ResinSla3d, StrippingAgent::Glanzer) => StrippingVerdict {
                severity: StrippingVerdictSeverity::Caution,
                title: "MODÉRÉ : SURVEILLER LA PIÈCE",
                advice_fr: "Laisser tremper 30 minutes maximum, vérifier la rigidité de la pièce avant brossage.",
                advice_en: "Limit soak to 30 minutes, inspect stiffness before gentle scrubbing.",
                recommended_duration: "20 à 30 minutes max",
            },
            (MiniatureMaterial::WhiteMetal, _) => StrippingVerdict {
                severity: StrippingVerdictSeverity::Safe,
                title: "TOTALEMENT SÛR",
                advice_fr: "Le métal résiste à tous les solvants ménagers et décapants du marché.",
                advice_en: "Metal is impervious to standard chemical strippers.",
                recommended_duration: "30 min à 4h",
            },
        }
    }
}
