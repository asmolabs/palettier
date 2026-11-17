//! Core paint physical and studio properties: drying speed, opacity, thickness, ventilation, and paint system.

use crate::error::DomainError;
use crate::i18n::Locale;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use std::time::Duration;

/// Drying speed of a pigment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum DryingClass {
    Fast,
    Medium,
    Slow,
    VerySlow,
}

impl DryingClass {
    pub const ALL: [DryingClass; 4] = [
        DryingClass::Fast,
        DryingClass::Medium,
        DryingClass::Slow,
        DryingClass::VerySlow,
    ];

    pub fn label(&self, locale: Locale) -> &'static str {
        match locale {
            Locale::Fr => match self {
                DryingClass::Fast => "Rapide",
                DryingClass::Medium => "Moyen",
                DryingClass::Slow => "Lent",
                DryingClass::VerySlow => "Très lent",
            },
            Locale::En => match self {
                DryingClass::Fast => "Fast",
                DryingClass::Medium => "Medium",
                DryingClass::Slow => "Slow",
                DryingClass::VerySlow => "Very slow",
            },
        }
    }

    pub fn reference_touch_dry(&self) -> Duration {
        match self {
            DryingClass::Fast => Duration::from_secs(10 * 3600),
            DryingClass::Medium => Duration::from_secs(20 * 3600),
            DryingClass::Slow => Duration::from_secs(38 * 3600),
            DryingClass::VerySlow => Duration::from_secs(64 * 3600),
        }
    }

    pub fn typical_pigments(&self, locale: Locale) -> &'static str {
        match locale {
            Locale::Fr => match self {
                DryingClass::Fast => "Terres d'ombre, bleu de Prusse, siccatifs naturels.",
                DryingClass::Medium => "Terres de Sienne, ocres, bleus et verts courants.",
                DryingClass::Slow => "Noirs d'ivoire, blancs de titane, la plupart des rouges.",
                DryingClass::VerySlow => "Cadmiums, blanc de zinc, laques organiques.",
            },
            Locale::En => match self {
                DryingClass::Fast => "Umbers, Prussian blue, natural driers.",
                DryingClass::Medium => "Siennas, ochres, common blues and greens.",
                DryingClass::Slow => "Ivory blacks, titanium whites, most reds.",
                DryingClass::VerySlow => "Cadmiums, zinc white, organic lakes.",
            },
        }
    }

    pub fn slowest(self, other: DryingClass) -> DryingClass {
        if self >= other { self } else { other }
    }

    pub fn as_db_str(&self) -> &'static str {
        match self {
            DryingClass::Fast => "FAST",
            DryingClass::Medium => "MEDIUM",
            DryingClass::Slow => "SLOW",
            DryingClass::VerySlow => "VERY_SLOW",
        }
    }
}

impl FromStr for DryingClass {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_uppercase().as_str() {
            "FAST" => Ok(DryingClass::Fast),
            "MEDIUM" => Ok(DryingClass::Medium),
            "SLOW" => Ok(DryingClass::Slow),
            "VERY_SLOW" => Ok(DryingClass::VerySlow),
            other => Err(DomainError::InvalidArgument(format!(
                "DryingClass inconnue : {other}"
            ))),
        }
    }
}

impl fmt::Display for DryingClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label(Locale::Fr))
    }
}

/// Covering power (opacity) of an oil paint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Opacity {
    Transparent,
    SemiTransparent,
    SemiOpaque,
    Opaque,
}

impl Opacity {
    pub const ALL: [Opacity; 4] = [
        Opacity::Transparent,
        Opacity::SemiTransparent,
        Opacity::SemiOpaque,
        Opacity::Opaque,
    ];

    pub fn label(&self, locale: Locale) -> &'static str {
        match locale {
            Locale::Fr => match self {
                Opacity::Transparent => "Transparent",
                Opacity::SemiTransparent => "Semi-transparent",
                Opacity::SemiOpaque => "Semi-opaque",
                Opacity::Opaque => "Opaque",
            },
            Locale::En => match self {
                Opacity::Transparent => "Transparent",
                Opacity::SemiTransparent => "Semi-transparent",
                Opacity::SemiOpaque => "Semi-opaque",
                Opacity::Opaque => "Opaque",
            },
        }
    }

    pub fn advice(&self, locale: Locale) -> &'static str {
        match locale {
            Locale::Fr => match self {
                Opacity::Transparent => {
                    "Idéal pour les glacis et les filtres : la couche du dessous reste visible."
                }
                Opacity::SemiTransparent => "Bon compromis pour un jus ou un fondu leger.",
                Opacity::SemiOpaque => "Couvre en deux passes, garde un peu de profondeur.",
                Opacity::Opaque => "Couvre en une passe : pour les aplats et les points lumineux.",
            },
            Locale::En => match self {
                Opacity::Transparent => {
                    "Ideal for glazes and filters: the underlying layer remains visible."
                }
                Opacity::SemiTransparent => "Good compromise for a wash or a gentle blend.",
                Opacity::SemiOpaque => "Covers in two passes while keeping some depth.",
                Opacity::Opaque => "Covers in a single pass: for base coats and catchlights.",
            },
        }
    }

    pub fn as_db_str(&self) -> &'static str {
        match self {
            Opacity::Transparent => "TRANSPARENT",
            Opacity::SemiTransparent => "SEMI_TRANSPARENT",
            Opacity::SemiOpaque => "SEMI_OPAQUE",
            Opacity::Opaque => "OPAQUE",
        }
    }
}

impl FromStr for Opacity {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_uppercase().as_str() {
            "TRANSPARENT" => Ok(Opacity::Transparent),
            "SEMI_TRANSPARENT" => Ok(Opacity::SemiTransparent),
            "SEMI_OPAQUE" => Ok(Opacity::SemiOpaque),
            "OPAQUE" => Ok(Opacity::Opaque),
            other => Err(DomainError::InvalidArgument(format!(
                "Opacity inconnue : {other}"
            ))),
        }
    }
}

impl fmt::Display for Opacity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label(Locale::Fr))
    }
}

/// Thickness of the applied coat.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LayerThickness {
    Glaze,
    Thin,
    Normal,
    Thick,
    Impasto,
}

impl LayerThickness {
    pub const ALL: [LayerThickness; 5] = [
        LayerThickness::Glaze,
        LayerThickness::Thin,
        LayerThickness::Normal,
        LayerThickness::Thick,
        LayerThickness::Impasto,
    ];

    pub fn label(&self, locale: Locale) -> &'static str {
        match locale {
            Locale::Fr => match self {
                LayerThickness::Glaze => "Glacis / voile",
                LayerThickness::Thin => "Couche fine",
                LayerThickness::Normal => "Couche normale",
                LayerThickness::Thick => "Couche chargée",
                LayerThickness::Impasto => "Empâtement",
            },
            Locale::En => match self {
                LayerThickness::Glaze => "Glaze / veil",
                LayerThickness::Thin => "Thin coat",
                LayerThickness::Normal => "Normal coat",
                LayerThickness::Thick => "Thick coat",
                LayerThickness::Impasto => "Impasto",
            },
        }
    }

    pub fn factor(&self) -> f64 {
        match self {
            LayerThickness::Glaze => 0.30,
            LayerThickness::Thin => 0.60,
            LayerThickness::Normal => 1.00,
            LayerThickness::Thick => 1.90,
            LayerThickness::Impasto => 3.40,
        }
    }

    pub fn as_db_str(&self) -> &'static str {
        match self {
            LayerThickness::Glaze => "GLAZE",
            LayerThickness::Thin => "THIN",
            LayerThickness::Normal => "NORMAL",
            LayerThickness::Thick => "THICK",
            LayerThickness::Impasto => "IMPASTO",
        }
    }
}

impl FromStr for LayerThickness {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_uppercase().as_str() {
            "GLAZE" => Ok(LayerThickness::Glaze),
            "THIN" => Ok(LayerThickness::Thin),
            "NORMAL" => Ok(LayerThickness::Normal),
            "THICK" => Ok(LayerThickness::Thick),
            "IMPASTO" => Ok(LayerThickness::Impasto),
            other => Err(DomainError::InvalidArgument(format!(
                "LayerThickness inconnue : {other}"
            ))),
        }
    }
}

impl fmt::Display for LayerThickness {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label(Locale::Fr))
    }
}

/// Workshop ventilation level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Ventilation {
    Confined,
    Normal,
    Good,
}

impl Ventilation {
    pub const ALL: [Ventilation; 3] = [
        Ventilation::Confined,
        Ventilation::Normal,
        Ventilation::Good,
    ];

    pub fn label(&self, locale: Locale) -> &'static str {
        match locale {
            Locale::Fr => match self {
                Ventilation::Confined => "Boîte fermée / vitrine",
                Ventilation::Normal => "Pièce normale",
                Ventilation::Good => "Bien aere ou courant d'air",
            },
            Locale::En => match self {
                Ventilation::Confined => "Closed box / cabinet",
                Ventilation::Normal => "Normal room",
                Ventilation::Good => "Well-ventilated / draft",
            },
        }
    }

    pub fn factor(&self) -> f64 {
        match self {
            Ventilation::Confined => 1.30,
            Ventilation::Normal => 1.00,
            Ventilation::Good => 0.82,
        }
    }

    pub fn as_db_str(&self) -> &'static str {
        match self {
            Ventilation::Confined => "CONFINED",
            Ventilation::Normal => "NORMAL",
            Ventilation::Good => "GOOD",
        }
    }
}

impl FromStr for Ventilation {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_uppercase().as_str() {
            "CONFINED" => Ok(Ventilation::Confined),
            "NORMAL" => Ok(Ventilation::Normal),
            "GOOD" => Ok(Ventilation::Good),
            other => Err(DomainError::InvalidArgument(format!(
                "Ventilation inconnue : {other}"
            ))),
        }
    }
}

impl fmt::Display for Ventilation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label(Locale::Fr))
    }
}

/// The paint family a piece, a coat or a tube belongs to.
///
/// Oil and acrylic do not obey the same rules: an acrylic coat is touch dry in minutes
/// and thinned with water, an oil coat takes days, is thinned with solvent and must
/// respect fat over lean. Every rule of the application reads this value instead of
/// assuming oil.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Default,
)]
pub enum PaintSystem {
    #[default]
    Oil,
    Acrylic,
}

impl PaintSystem {
    pub const ALL: [PaintSystem; 2] = [PaintSystem::Oil, PaintSystem::Acrylic];

    pub fn label(&self, locale: Locale) -> &'static str {
        match self {
            PaintSystem::Oil => locale.tr("Huile", "Oil"),
            PaintSystem::Acrylic => locale.tr("Acrylique", "Acrylic"),
        }
    }

    /// One-line reminder of what changes when working in this system.
    pub fn summary_i18n(&self, locale: Locale) -> &'static str {
        match self {
            PaintSystem::Oil => locale.tr(
                "Diluant ou huile, séchage en jours, gras sur maigre à respecter.",
                "Thinner or oil, drying in days, fat over lean applies.",
            ),
            PaintSystem::Acrylic => locale.tr(
                "Eau et médiums acryliques, sec au toucher en minutes, couches fines et superposées.",
                "Water and acrylic mediums, touch dry in minutes, thin stacked layers.",
            ),
        }
    }

    pub fn as_db_str(&self) -> &'static str {
        match self {
            PaintSystem::Oil => "OIL",
            PaintSystem::Acrylic => "ACRYLIC",
        }
    }
}

impl FromStr for PaintSystem {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_uppercase().as_str() {
            "OIL" | "HUILE" | "" => Ok(PaintSystem::Oil),
            "ACRYLIC" | "ACRYLIQUE" => Ok(PaintSystem::Acrylic),
            other => Err(DomainError::InvalidArgument(format!(
                "Systeme de peinture inconnu : {other}"
            ))),
        }
    }
}

impl fmt::Display for PaintSystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label(Locale::Fr))
    }
}
