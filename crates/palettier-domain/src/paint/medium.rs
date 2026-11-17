//! Thinners, oils, and acrylic mediums used to modify paint flow, transparency, and drying.

use super::properties::PaintSystem;
use crate::error::DomainError;
use crate::i18n::Locale;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Thinner or medium added to the paint: solvents and oils for oil paint, water and
/// acrylic mediums for acrylics. `None` (straight from the pot or tube) belongs to both.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Medium {
    None,
    OdorlessThinner,
    WhiteSpirit,
    Turpentine,
    AlkydMedium,
    LinseedOil,
    WalnutOil,
    StandOil,
    CobaltDrier,
    Water,
    AcrylicGlazeMedium,
    FlowImprover,
    Retarder,
}

impl Medium {
    pub const ALL: [Medium; 13] = [
        Medium::None,
        Medium::OdorlessThinner,
        Medium::WhiteSpirit,
        Medium::Turpentine,
        Medium::AlkydMedium,
        Medium::LinseedOil,
        Medium::WalnutOil,
        Medium::StandOil,
        Medium::CobaltDrier,
        Medium::Water,
        Medium::AcrylicGlazeMedium,
        Medium::FlowImprover,
        Medium::Retarder,
    ];

    /// The system this medium belongs to, or `None` when it fits both (pure paint).
    pub fn system(&self) -> Option<PaintSystem> {
        match self {
            Medium::None => None,
            Medium::Water
            | Medium::AcrylicGlazeMedium
            | Medium::FlowImprover
            | Medium::Retarder => Some(PaintSystem::Acrylic),
            _ => Some(PaintSystem::Oil),
        }
    }

    /// Whether this medium can be mixed into paint of the given system.
    pub fn fits(&self, system: PaintSystem) -> bool {
        self.system().is_none_or(|s| s == system)
    }

    /// The mediums offered when painting in the given system.
    pub fn for_system(system: PaintSystem) -> Vec<Medium> {
        Self::ALL.into_iter().filter(|m| m.fits(system)).collect()
    }

    /// The usual thinner of the system: what a wash or a glaze is diluted with.
    pub fn thinner_of(system: PaintSystem) -> Medium {
        match system {
            PaintSystem::Oil => Medium::OdorlessThinner,
            PaintSystem::Acrylic => Medium::Water,
        }
    }

    pub fn label(&self, locale: Locale) -> &'static str {
        match locale {
            Locale::Fr => match self {
                Medium::Water => "Eau",
                Medium::AcrylicGlazeMedium => "Médium à glacis acrylique",
                Medium::FlowImprover => "Fluidifiant (flow improver)",
                Medium::Retarder => "Retardateur",
                Medium::None => "Pur (sortie de tube)",
                Medium::OdorlessThinner => "Diluant inodore",
                Medium::WhiteSpirit => "White spirit",
                Medium::Turpentine => "Essence de térébenthine",
                Medium::AlkydMedium => "Médium alkyde (type Liquin)",
                Medium::LinseedOil => "Huile de lin",
                Medium::WalnutOil => "Huile de noix",
                Medium::StandOil => "Huile stand",
                Medium::CobaltDrier => "Siccatif au cobalt",
            },
            Locale::En => match self {
                Medium::Water => "Water",
                Medium::AcrylicGlazeMedium => "Acrylic glaze medium",
                Medium::FlowImprover => "Flow improver",
                Medium::Retarder => "Retarder",
                Medium::None => "Pure (from tube)",
                Medium::OdorlessThinner => "Odorless thinner",
                Medium::WhiteSpirit => "White spirit",
                Medium::Turpentine => "Turpentine",
                Medium::AlkydMedium => "Alkyd medium (Liquin)",
                Medium::LinseedOil => "Linseed oil",
                Medium::WalnutOil => "Walnut oil",
                Medium::StandOil => "Stand oil",
                Medium::CobaltDrier => "Cobalt drier",
            },
        }
    }

    fn drying_factor_at_full_ratio(&self) -> f64 {
        match self {
            Medium::None => 1.00,
            Medium::OdorlessThinner => 0.55,
            Medium::WhiteSpirit => 0.48,
            Medium::Turpentine => 0.50,
            Medium::AlkydMedium => 0.35,
            Medium::LinseedOil => 1.90,
            Medium::WalnutOil => 2.20,
            Medium::StandOil => 2.60,
            Medium::CobaltDrier => 0.28,
            // Acrylic factors apply to the acrylic model, whose base is in minutes.
            Medium::Water => 0.90,
            Medium::AcrylicGlazeMedium => 1.50,
            Medium::FlowImprover => 1.30,
            Medium::Retarder => 3.00,
        }
    }

    pub fn max_useful_ratio(&self) -> f64 {
        match self {
            Medium::None => 0.00,
            Medium::OdorlessThinner => 0.85,
            Medium::WhiteSpirit => 0.85,
            Medium::Turpentine => 0.85,
            Medium::AlkydMedium => 0.50,
            Medium::LinseedOil => 0.40,
            Medium::WalnutOil => 0.40,
            Medium::StandOil => 0.30,
            Medium::CobaltDrier => 0.05,
            Medium::Water => 0.90,
            Medium::AcrylicGlazeMedium => 0.80,
            Medium::FlowImprover => 0.20,
            Medium::Retarder => 0.20,
        }
    }

    fn fat_direction(&self) -> i32 {
        match self {
            // Fat over lean is an oil rule: acrylic mediums carry no oil.
            Medium::None
            | Medium::CobaltDrier
            | Medium::Water
            | Medium::AcrylicGlazeMedium
            | Medium::FlowImprover
            | Medium::Retarder => 0,
            Medium::OdorlessThinner | Medium::WhiteSpirit | Medium::Turpentine => -1,
            Medium::AlkydMedium | Medium::LinseedOil | Medium::WalnutOil | Medium::StandOil => 1,
        }
    }

    pub fn advice(&self, locale: Locale) -> &'static str {
        match locale {
            Locale::Fr => match self {
                Medium::Water => {
                    "Le diluant de l'acrylique : au-delà de moitié d'eau, le liant ne tient plus et la couche poudre."
                }
                Medium::AcrylicGlazeMedium => {
                    "Dilue sans casser le liant : glacis et lavis acryliques transparents qui tiennent."
                }
                Medium::FlowImprover => {
                    "Quelques gouttes : la peinture file mieux et marque moins le pinceau."
                }
                Medium::Retarder => {
                    "Allonge le temps ouvert pour les fondus humides ; au-delà de 20 %, la couche reste poisseuse."
                }
                Medium::None => "Pâte épaisse : réservée aux empâtements et aux textures.",
                Medium::OdorlessThinner => "Le standard pour les jus et les fondus sur figurine.",
                Medium::WhiteSpirit => "Plus agressif : vérifier la tenue du vernis en dessous.",
                Medium::Turpentine => "Évapore vite, odeur forte, bonne accroche.",
                Medium::AlkydMedium => "Accélère fortement et lisse le fondu.",
                Medium::LinseedOil => "Rallonge le temps ouvert, jaunit légèrement en couche épaisse.",
                Medium::WalnutOil => "Temps ouvert très long, ne jaunit presque pas.",
                Medium::StandOil => "Très lent, très lissant : pour les fondus longs.",
                Medium::CobaltDrier => "Quelques gouttes suffisent ; au-delà, la couche craquelle.",
            },
            Locale::En => match self {
                Medium::Water => {
                    "The acrylic thinner: beyond half water the binder gives up and the coat turns chalky."
                }
                Medium::AcrylicGlazeMedium => {
                    "Thins without breaking the binder: durable transparent acrylic glazes and washes."
                }
                Medium::FlowImprover => {
                    "A few drops: paint flows better and shows fewer brush marks."
                }
                Medium::Retarder => {
                    "Extends open time for wet blending; beyond 20% the coat stays tacky."
                }
                Medium::None => "Thick paste: best saved for impasto and texture work.",
                Medium::OdorlessThinner => "The standard choice for miniature washes and blending.",
                Medium::WhiteSpirit => {
                    "More aggressive: check that the underlying varnish holds up."
                }
                Medium::Turpentine => "Evaporates quickly, strong odor, good grip.",
                Medium::AlkydMedium => "Speeds up drying significantly and smooths out blends.",
                Medium::LinseedOil => "Extends open time; yellows slightly in thick coats.",
                Medium::WalnutOil => "Very long open time, barely yellows.",
                Medium::StandOil => "Very slow, self-leveling: great for long blends.",
                Medium::CobaltDrier => {
                    "A few drops are enough; beyond that, the paint film will crack."
                }
            },
        }
    }

    pub fn drying_factor(&self, ratio: f64) -> f64 {
        let r = ratio.clamp(0.0, 1.0);
        let reference = self.max_useful_ratio().max(1e-6);
        let progress = (r / reference).min(1.0);
        1.0 + (self.drying_factor_at_full_ratio() - 1.0) * progress
    }

    pub fn fatness(&self, ratio: f64) -> f64 {
        1.0 + (self.fat_direction() as f64) * ratio.clamp(0.0, 1.0)
    }

    pub fn as_db_str(&self) -> &'static str {
        match self {
            Medium::None => "NONE",
            Medium::OdorlessThinner => "ODORLESS_THINNER",
            Medium::WhiteSpirit => "WHITE_SPIRIT",
            Medium::Turpentine => "TURPENTINE",
            Medium::AlkydMedium => "ALKYD_MEDIUM",
            Medium::LinseedOil => "LINSEED_OIL",
            Medium::WalnutOil => "WALNUT_OIL",
            Medium::StandOil => "STAND_OIL",
            Medium::CobaltDrier => "COBALT_DRIER",
            Medium::Water => "WATER",
            Medium::AcrylicGlazeMedium => "ACRYLIC_GLAZE_MEDIUM",
            Medium::FlowImprover => "FLOW_IMPROVER",
            Medium::Retarder => "RETARDER",
        }
    }
}

impl FromStr for Medium {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_uppercase().as_str() {
            "NONE" => Ok(Medium::None),
            "ODORLESS_THINNER" => Ok(Medium::OdorlessThinner),
            "WHITE_SPIRIT" => Ok(Medium::WhiteSpirit),
            "TURPENTINE" => Ok(Medium::Turpentine),
            "ALKYD_MEDIUM" => Ok(Medium::AlkydMedium),
            "LINSEED_OIL" => Ok(Medium::LinseedOil),
            "WALNUT_OIL" => Ok(Medium::WalnutOil),
            "STAND_OIL" => Ok(Medium::StandOil),
            "COBALT_DRIER" => Ok(Medium::CobaltDrier),
            "WATER" => Ok(Medium::Water),
            "ACRYLIC_GLAZE_MEDIUM" => Ok(Medium::AcrylicGlazeMedium),
            "FLOW_IMPROVER" => Ok(Medium::FlowImprover),
            "RETARDER" => Ok(Medium::Retarder),
            other => Err(DomainError::InvalidArgument(format!(
                "Medium inconnu : {other}"
            ))),
        }
    }
}

impl fmt::Display for Medium {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label(Locale::Fr))
    }
}

