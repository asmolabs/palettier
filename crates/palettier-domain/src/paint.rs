use crate::color::{Colorant, Rgb, delta_e_2000};
use crate::error::DomainError;
use crate::i18n::Locale;
use indexmap::IndexSet;
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

    pub fn label(&self) -> &'static str {
        match self {
            DryingClass::Fast => "Rapide",
            DryingClass::Medium => "Moyen",
            DryingClass::Slow => "Lent",
            DryingClass::VerySlow => "Tres lent",
        }
    }

    pub fn label_i18n(&self, locale: Locale) -> &'static str {
        match locale {
            Locale::Fr => self.label(),
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

    pub fn typical_pigments(&self) -> &'static str {
        match self {
            DryingClass::Fast => "Terres d'ombre, bleu de Prusse, siccatifs naturels.",
            DryingClass::Medium => "Terres de Sienne, ocres, bleus et verts courants.",
            DryingClass::Slow => "Noirs d'ivoire, blancs de titane, la plupart des rouges.",
            DryingClass::VerySlow => "Cadmiums, blanc de zinc, laques organiques.",
        }
    }

    pub fn typical_pigments_i18n(&self, locale: Locale) -> &'static str {
        match locale {
            Locale::Fr => self.typical_pigments(),
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
        f.write_str(self.label())
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

    pub fn label(&self) -> &'static str {
        match self {
            Opacity::Transparent => "Transparent",
            Opacity::SemiTransparent => "Semi-transparent",
            Opacity::SemiOpaque => "Semi-opaque",
            Opacity::Opaque => "Opaque",
        }
    }

    pub fn label_i18n(&self, locale: Locale) -> &'static str {
        match locale {
            Locale::Fr => self.label(),
            Locale::En => match self {
                Opacity::Transparent => "Transparent",
                Opacity::SemiTransparent => "Semi-transparent",
                Opacity::SemiOpaque => "Semi-opaque",
                Opacity::Opaque => "Opaque",
            },
        }
    }

    pub fn advice(&self) -> &'static str {
        match self {
            Opacity::Transparent => {
                "Ideal pour les glacis et les filtres : la couche du dessous reste visible."
            }
            Opacity::SemiTransparent => "Bon compromis pour un jus ou un fondu leger.",
            Opacity::SemiOpaque => "Couvre en deux passes, garde un peu de profondeur.",
            Opacity::Opaque => "Couvre en une passe : pour les aplats et les points lumineux.",
        }
    }

    pub fn advice_i18n(&self, locale: Locale) -> &'static str {
        match locale {
            Locale::Fr => self.advice(),
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
        f.write_str(self.label())
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

    pub fn label(&self) -> &'static str {
        match self {
            LayerThickness::Glaze => "Glacis / voile",
            LayerThickness::Thin => "Couche fine",
            LayerThickness::Normal => "Couche normale",
            LayerThickness::Thick => "Couche chargee",
            LayerThickness::Impasto => "Empatement",
        }
    }

    pub fn label_i18n(&self, locale: Locale) -> &'static str {
        match locale {
            Locale::Fr => self.label(),
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
        f.write_str(self.label())
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
    pub const ALL: [Ventilation; 3] = [Ventilation::Confined, Ventilation::Normal, Ventilation::Good];

    pub fn label(&self) -> &'static str {
        match self {
            Ventilation::Confined => "Boite fermee / vitrine",
            Ventilation::Normal => "Piece normale",
            Ventilation::Good => "Bien aere ou courant d'air",
        }
    }

    pub fn label_i18n(&self, locale: Locale) -> &'static str {
        match locale {
            Locale::Fr => self.label(),
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
        f.write_str(self.label())
    }
}

/// Thinner or medium added to oil paint.
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
}

impl Medium {
    pub const ALL: [Medium; 9] = [
        Medium::None,
        Medium::OdorlessThinner,
        Medium::WhiteSpirit,
        Medium::Turpentine,
        Medium::AlkydMedium,
        Medium::LinseedOil,
        Medium::WalnutOil,
        Medium::StandOil,
        Medium::CobaltDrier,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Medium::None => "Pur (sortie de tube)",
            Medium::OdorlessThinner => "Diluant inodore",
            Medium::WhiteSpirit => "White spirit",
            Medium::Turpentine => "Essence de terebenthine",
            Medium::AlkydMedium => "Medium alkyde (type Liquin)",
            Medium::LinseedOil => "Huile de lin",
            Medium::WalnutOil => "Huile de noix",
            Medium::StandOil => "Huile stand",
            Medium::CobaltDrier => "Siccatif au cobalt",
        }
    }

    pub fn label_i18n(&self, locale: Locale) -> &'static str {
        match locale {
            Locale::Fr => self.label(),
            Locale::En => match self {
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
        }
    }

    fn fat_direction(&self) -> i32 {
        match self {
            Medium::None | Medium::CobaltDrier => 0,
            Medium::OdorlessThinner | Medium::WhiteSpirit | Medium::Turpentine => -1,
            Medium::AlkydMedium | Medium::LinseedOil | Medium::WalnutOil | Medium::StandOil => 1,
        }
    }

    pub fn advice(&self) -> &'static str {
        match self {
            Medium::None => "Pate epaisse : reserve aux empatements et aux textures.",
            Medium::OdorlessThinner => "Le standard pour les jus et les fondus sur figurine.",
            Medium::WhiteSpirit => "Plus agressif : verifier la tenue du vernis en dessous.",
            Medium::Turpentine => "Evapore vite, odeur forte, bonne accroche.",
            Medium::AlkydMedium => "Accelere fortement et lisse le fondu.",
            Medium::LinseedOil => {
                "Rallonge le temps ouvert, jaunit legerement en couche epaisse."
            }
            Medium::WalnutOil => "Temps ouvert tres long, ne jaunit presque pas.",
            Medium::StandOil => "Tres lent, tres lissant : pour les fondus longs.",
            Medium::CobaltDrier => {
                "Quelques gouttes suffisent ; au-dela, la couche craquelle."
            }
        }
    }

    pub fn advice_i18n(&self, locale: Locale) -> &'static str {
        match locale {
            Locale::Fr => self.advice(),
            Locale::En => match self {
                Medium::None => "Thick paste: best saved for impasto and texture work.",
                Medium::OdorlessThinner => "The standard choice for miniature washes and blending.",
                Medium::WhiteSpirit => "More aggressive: check that the underlying varnish holds up.",
                Medium::Turpentine => "Evaporates quickly, strong odor, good grip.",
                Medium::AlkydMedium => "Speeds up drying significantly and smooths out blends.",
                Medium::LinseedOil => {
                    "Extends open time; yellows slightly in thick coats."
                }
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
            other => Err(DomainError::InvalidArgument(format!(
                "Medium inconnu : {other}"
            ))),
        }
    }
}

impl fmt::Display for Medium {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// Oil painting technique, along with its typical settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Technique {
    OilWash,
    PinWash,
    Filter,
    Glaze,
    Blending,
    DotFading,
    StreakingGrime,
    OilRendering,
    BaseLayer,
    Highlight,
}

impl Technique {
    pub const ALL: [Technique; 10] = [
        Technique::OilWash,
        Technique::PinWash,
        Technique::Filter,
        Technique::Glaze,
        Technique::Blending,
        Technique::DotFading,
        Technique::StreakingGrime,
        Technique::OilRendering,
        Technique::BaseLayer,
        Technique::Highlight,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Technique::OilWash => "Jus a l'huile",
            Technique::PinWash => "Jus capillaire (pin wash)",
            Technique::Filter => "Filtre",
            Technique::Glaze => "Glacis",
            Technique::Blending => "Fondu / degrade",
            Technique::DotFading => "Dot fading",
            Technique::StreakingGrime => "Coulures et salissures",
            Technique::OilRendering => "Oil Paint Rendering (OPR)",
            Technique::BaseLayer => "Aplat de base",
            Technique::Highlight => "Eclaircis et points lumineux",
        }
    }

    pub fn label_i18n(&self, locale: Locale) -> &'static str {
        match locale {
            Locale::Fr => self.label(),
            Locale::En => match self {
                Technique::OilWash => "Oil wash",
                Technique::PinWash => "Pin wash",
                Technique::Filter => "Filter",
                Technique::Glaze => "Glaze",
                Technique::Blending => "Blending",
                Technique::DotFading => "Dot fading",
                Technique::StreakingGrime => "Streaking & grime",
                Technique::OilRendering => "Oil Paint Rendering (OPR)",
                Technique::BaseLayer => "Base coat",
                Technique::Highlight => "Highlights & catchlights",
            },
        }
    }

    pub fn default_medium(&self) -> Medium {
        match self {
            Technique::Blending | Technique::DotFading | Technique::Highlight => Medium::None,
            _ => Medium::OdorlessThinner,
        }
    }

    pub fn min_ratio(&self) -> f64 {
        match self {
            Technique::OilWash => 0.80,
            Technique::PinWash => 0.85,
            Technique::Filter => 0.88,
            Technique::Glaze => 0.70,
            Technique::Blending => 0.00,
            Technique::DotFading => 0.00,
            Technique::StreakingGrime => 0.60,
            Technique::OilRendering => 0.50,
            Technique::BaseLayer => 0.10,
            Technique::Highlight => 0.00,
        }
    }

    pub fn max_ratio(&self) -> f64 {
        match self {
            Technique::OilWash => 0.92,
            Technique::PinWash => 0.94,
            Technique::Filter => 0.96,
            Technique::Glaze => 0.88,
            Technique::Blending => 0.25,
            Technique::DotFading => 0.20,
            Technique::StreakingGrime => 0.85,
            Technique::OilRendering => 0.80,
            Technique::BaseLayer => 0.35,
            Technique::Highlight => 0.20,
        }
    }

    pub fn default_ratio(&self) -> f64 {
        (self.min_ratio() + self.max_ratio()) / 2.0
    }

    pub fn typical_thickness(&self) -> LayerThickness {
        match self {
            Technique::OilWash | Technique::PinWash | Technique::Filter | Technique::Glaze => {
                LayerThickness::Glaze
            }
            Technique::BaseLayer => LayerThickness::Normal,
            _ => LayerThickness::Thin,
        }
    }

    pub fn requires_cured_base(&self) -> bool {
        !matches!(self, Technique::BaseLayer)
    }

    pub fn tips(&self) -> &'static [&'static str] {
        match self {
            Technique::OilWash => &[
                "Le vernis brillant en dessous doit etre sec depuis 24 h au minimum.",
                "Charger le pinceau puis l'essuyer : le jus doit couler, pas s'etaler.",
                "Retirer l'exces au coton-tige a peine humide de diluant avant la prise.",
            ],
            Technique::PinWash => &[
                "Poser la pointe du pinceau dans le creux et laisser la capillarite faire le trajet.",
                "Un seul passage par ligne : repasser dessus casse le depot.",
            ],
            Technique::Filter => &[
                "Tres dilue : le filtre unifie une teinte, il ne la remplace pas.",
                "Deux filtres legers valent mieux qu'un seul charge.",
            ],
            Technique::Glaze => &[
                "Privilegier une huile transparente : une opaque tue la profondeur.",
                "Laisser secher entre deux glacis, sinon la couche precedente se releve.",
            ],
            Technique::Blending => &[
                "Travailler dans le temps ouvert : une fois la prise commencee, arreter.",
                "Pinceau propre et sec pour tirer la transition, essuye a chaque passage.",
            ],
            Technique::DotFading => &[
                "Poser des points minuscules de plusieurs teintes, puis les tirer vers le bas.",
                "Moins de peinture que ce que l'on croit : le pinceau doit etre presque sec.",
            ],
            Technique::StreakingGrime => &[
                "Tracer le trait, laisser mordre une a deux minutes, puis tirer vers le bas.",
                "Suivre le sens de l'eau : verticales sur les flancs, en eventail sous les rivets.",
            ],
            Technique::OilRendering => &[
                "Poser clair et fonce cote a cote, puis fondre la frontiere.",
                "Construire en plusieurs seances plutot qu'en une couche chargee.",
            ],
            Technique::BaseLayer => &[
                "A l'huile, un aplat de base reste rare : il seche lentement et bloque la suite.",
                "Utile surtout sur les grandes surfaces lisses (bustes, 1/10).",
            ],
            Technique::Highlight => &[
                "Une huile opaque tient mieux le point lumineux qu'une transparente.",
                "Le blanc de titane seche tres lentement : prevoir la couche suivante a distance.",
            ],
        }
    }

    pub fn tips_i18n(&self, locale: Locale) -> &'static [&'static str] {
        match locale {
            Locale::Fr => self.tips(),
            Locale::En => match self {
                Technique::OilWash => &[
                    "The underlying gloss varnish must cure for at least 24 hours.",
                    "Load the brush then blot it: the wash should flow, not flood.",
                    "Wipe away excess with a cotton swab barely dampened with thinner before it sets.",
                ],
                Technique::PinWash => &[
                    "Touch the tip of the brush into the recess and let capillary action pull the wash along.",
                    "One pass per panel line: going back over it disturbs the deposit.",
                ],
                Technique::Filter => &[
                    "Heavily thinned: a filter unifies a hue without replacing it.",
                    "Two light filters are better than one heavy pass.",
                ],
                Technique::Glaze => &[
                    "Favor transparent oils: an opaque pigment kills depth.",
                    "Let each glaze dry before applying the next, otherwise you will lift the previous coat.",
                ],
                Technique::Blending => &[
                    "Work within the open time: once the paint starts setting, stop.",
                    "Use a clean, dry brush to feather the transition, wiping it between strokes.",
                ],
                Technique::DotFading => &[
                    "Apply tiny dots of several hues, then streak them downward.",
                    "Use less paint than you think: the brush should be almost dry.",
                ],
                Technique::StreakingGrime => &[
                    "Paint the streak, let it bite for a minute or two, then feather downward.",
                    "Follow gravity and water flow: vertical on flat sides, fanned out under rivets.",
                ],
                Technique::OilRendering => &[
                    "Lay light and dark side by side, then blend the boundary.",
                    "Build up over several sessions rather than in one heavy coat.",
                ],
                Technique::BaseLayer => &[
                    "In oils, a full base coat is rare: it dries slowly and holds up subsequent steps.",
                    "Mostly useful on large, smooth surfaces (busts, 1/10 scale).",
                ],
                Technique::Highlight => &[
                    "An opaque oil holds a sharp catchlight better than a transparent one.",
                    "Titanium white dries very slowly: plan the next coat accordingly.",
                ],
            },
        }
    }

    pub fn by_label(label: Option<&str>, fallback: Technique) -> Technique {
        let Some(raw) = label else { return fallback };
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return fallback;
        }
        Self::ALL
            .into_iter()
            .find(|t| {
                t.label().eq_ignore_ascii_case(trimmed)
                    || t.label_i18n(Locale::En).eq_ignore_ascii_case(trimmed)
            })
            .unwrap_or(fallback)
    }

    pub fn as_db_str(&self) -> &'static str {
        match self {
            Technique::OilWash => "OIL_WASH",
            Technique::PinWash => "PIN_WASH",
            Technique::Filter => "FILTER",
            Technique::Glaze => "GLAZE",
            Technique::Blending => "BLENDING",
            Technique::DotFading => "DOT_FADING",
            Technique::StreakingGrime => "STREAKING_GRIME",
            Technique::OilRendering => "OIL_RENDERING",
            Technique::BaseLayer => "BASE_LAYER",
            Technique::Highlight => "HIGHLIGHT",
        }
    }
}

impl FromStr for Technique {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_uppercase().as_str() {
            "OIL_WASH" => Ok(Technique::OilWash),
            "PIN_WASH" => Ok(Technique::PinWash),
            "FILTER" => Ok(Technique::Filter),
            "GLAZE" => Ok(Technique::Glaze),
            "BLENDING" => Ok(Technique::Blending),
            "DOT_FADING" => Ok(Technique::DotFading),
            "STREAKING_GRIME" => Ok(Technique::StreakingGrime),
            "OIL_RENDERING" => Ok(Technique::OilRendering),
            "BASE_LAYER" => Ok(Technique::BaseLayer),
            "HIGHLIGHT" => Ok(Technique::Highlight),
            other => {
                if let Some(found) = Self::ALL.into_iter().find(|t| {
                    t.label().eq_ignore_ascii_case(s.trim())
                        || t.label_i18n(Locale::En).eq_ignore_ascii_case(s.trim())
                }) {
                    Ok(found)
                } else {
                    Err(DomainError::InvalidArgument(format!(
                        "Technique inconnue : {other}"
                    )))
                }
            }
        }
    }
}

impl fmt::Display for Technique {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// A Colour Index pigment and the properties derived from it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Pigment {
    pub code: String,
    pub name: String,
    pub color: Rgb,
    pub opacity: Opacity,
    pub drying_class: DryingClass,
    pub tinting_strength: f64,
    pub tint: Option<Rgb>,
}

impl Pigment {
    pub fn colorant(&self) -> Colorant {
        match self.tint {
            None => Colorant::of_masstone(self.color),
            Some(tint) => Colorant::of_masstone_and_tint(
                self.color,
                tint,
                Colorant::REFERENCE_TINT_CONCENTRATION,
            ),
        }
    }

    pub fn unknown(code: &str) -> Self {
        Self {
            code: code.to_string(),
            name: "Pigment non reference".to_string(),
            color: Rgb::new(0.5, 0.5, 0.5),
            opacity: Opacity::SemiOpaque,
            drying_class: DryingClass::Medium,
            tinting_strength: 0.5,
            tint: None,
        }
    }
}

/// An oil paint tube from the catalog or the painter's shelf.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Paint {
    pub id: Option<i64>,
    pub brand: String,
    pub name: String,
    pub code: String,
    pub legacy_code: String,
    pub pigments: IndexSet<String>,
    pub hex_color: String,
    pub tint_hex: Option<String>,
    pub opacity: Opacity,
    pub drying_class: DryingClass,
    pub tinting_strength: f64,
    pub color_derived: bool,
    pub pigments_verified: bool,
    pub user_added: bool,
    pub in_stock: bool,
    pub notes: String,
}

impl Paint {
    pub fn new(brand: impl Into<String>, name: impl Into<String>, hex_color: impl Into<String>) -> Self {
        Self {
            id: None,
            brand: brand.into(),
            name: name.into(),
            code: String::new(),
            legacy_code: String::new(),
            pigments: IndexSet::new(),
            hex_color: hex_color.into(),
            tint_hex: None,
            opacity: Opacity::SemiOpaque,
            drying_class: DryingClass::Medium,
            tinting_strength: 0.5,
            color_derived: false,
            pigments_verified: false,
            user_added: false,
            in_stock: true,
            notes: String::new(),
        }
    }

    pub fn color(&self) -> Rgb {
        Rgb::of_hex(&self.hex_color).unwrap_or_else(|_| Rgb::new(0.5, 0.5, 0.5))
    }

    pub fn display_name(&self) -> String {
        format!("{} - {}", self.brand, self.name)
    }

    pub fn colorant(&self) -> Colorant {
        let masstone = self.color();
        match self.tint_hex.as_deref() {
            Some(s) if !s.trim().is_empty() => match Rgb::of_hex(s) {
                Ok(tint) => Colorant::of_masstone_and_tint(
                    masstone,
                    tint,
                    Colorant::REFERENCE_TINT_CONCENTRATION,
                ),
                Err(_) => Colorant::of_masstone(masstone),
            },
            _ => Colorant::of_masstone(masstone),
        }
    }

    /// Evaluates pigment toxicity and returns studio handling safety advice.
    pub fn safety_info(&self) -> SafetyInfo {
        let mut hazards = Vec::new();
        let mut level = SafetyLevel::Safe;

        for pig in &self.pigments {
            let p = pig.to_uppercase();
            if p.starts_with("PR108") || p.starts_with("PY35") || p.starts_with("PY37") || p.starts_with("PO20") {
                hazards.push("Cadmium (sulfoséléniure)".to_string());
                level = SafetyLevel::Hazard;
            } else if p.starts_with("PB28") || p.starts_with("PB35") || p.starts_with("PB36") || p.starts_with("PB72") || p.starts_with("PB74") || p.starts_with("PG19") || p.starts_with("PG26") || p.starts_with("PV14") {
                hazards.push("Cobalt".to_string());
                level = SafetyLevel::Hazard;
            } else if p.starts_with("PW1") {
                hazards.push("Carbonate de plomb (Blanc de plomb)".to_string());
                level = SafetyLevel::Hazard;
            } else if p.starts_with("PW4") {
                hazards.push("Oxyde de zinc (éco-toxique aquatique)".to_string());
                if level < SafetyLevel::Caution {
                    level = SafetyLevel::Caution;
                }
            } else if p.starts_with("PBK26") || p.starts_with("PBR8") {
                hazards.push("Manganèse".to_string());
                if level < SafetyLevel::Caution {
                    level = SafetyLevel::Caution;
                }
            }
        }

        match level {
            SafetyLevel::Hazard => SafetyInfo {
                level,
                hazards,
                advice: "Contient des métaux lourds. Ne pas pulvériser à l'aérographe sans masque respiratoire. Éviter d'affiner le pinceau avec la bouche. Se laver les mains après la séance.",
                advice_en: "Contains heavy metals. Do not spray or aerosolize without a particle respirator. Never point brushes with lips. Wash hands thoroughly after painting.",
            },
            SafetyLevel::Caution => SafetyInfo {
                level,
                hazards,
                advice: "Éviter de jeter les solvants usagés dans l'évier. Laisser décanter et déposer les résidus en déchetterie.",
                advice_en: "Avoid pouring rinse thinners into drains. Let thinners settle and dispose of paint sludge safely.",
            },
            SafetyLevel::Safe => SafetyInfo {
                level,
                hazards,
                advice: "Pigments minéraux ou organiques sans toxicité identifiée en usage d'atelier normal.",
                advice_en: "Inert mineral or organic pigments with no known acute toxicity in normal studio conditions.",
            },
        }
    }

    /// Evaluates ASTM D4302 lightfastness permanence.
    pub fn lightfastness(&self) -> Lightfastness {
        for pig in &self.pigments {
            let p = pig.to_uppercase();
            if p.starts_with("PR83") || p.starts_with("NR4") || p.starts_with("NY14") {
                return Lightfastness::ClassIII;
            }
        }
        for pig in &self.pigments {
            let p = pig.to_uppercase();
            if p.starts_with("PR177") || p.starts_with("PG8") || p.starts_with("PY1") {
                return Lightfastness::ClassII;
            }
        }
        Lightfastness::ClassI
    }
}

/// Studio toxicity and chemical hygiene level for an oil paint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SafetyLevel {
    Safe,
    Caution,
    Hazard,
}

impl SafetyLevel {
    pub fn badge_label(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::Safe, Locale::Fr) => "Non toxique",
            (Self::Safe, Locale::En) => "Non-toxic",
            (Self::Caution, Locale::Fr) => "Attention éco-toxique",
            (Self::Caution, Locale::En) => "Eco caution",
            (Self::Hazard, Locale::Fr) => "Métaux lourds (Cadmium/Cobalt/Plomb)",
            (Self::Hazard, Locale::En) => "Heavy metals (Cadmium/Cobalt/Lead)",
        }
    }

    pub fn css_class(&self) -> &'static str {
        match self {
            Self::Safe => "pill-ok",
            Self::Caution => "pill-warn",
            Self::Hazard => "pill-danger",
        }
    }
}

/// Detailed safety and handling advisory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafetyInfo {
    pub level: SafetyLevel,
    pub hazards: Vec<String>,
    pub advice: &'static str,
    pub advice_en: &'static str,
}

impl SafetyInfo {
    pub fn advice_i18n(&self, locale: Locale) -> &'static str {
        match locale {
            Locale::Fr => self.advice,
            Locale::En => self.advice_en,
        }
    }
}

/// ASTM D4302 Lightfastness rating for oil paints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Lightfastness {
    /// Excellent lightfastness (100+ years in museum conditions).
    ClassI,
    /// Very good lightfastness (50-100 years).
    ClassII,
    /// Fair / Fugitive (fades over time).
    ClassIII,
}

impl Lightfastness {
    pub fn label(&self) -> &'static str {
        match self {
            Self::ClassI => "ASTM I (Excellent)",
            Self::ClassII => "ASTM II (Très bon)",
            Self::ClassIII => "ASTM III (Fugitif)",
        }
    }

    pub fn label_i18n(&self, locale: Locale) -> &'static str {
        match locale {
            Locale::Fr => self.label(),
            Locale::En => match self {
                Self::ClassI => "ASTM I (Excellent)",
                Self::ClassII => "ASTM II (Very Good)",
                Self::ClassIII => "ASTM III (Fugitive)",
            },
        }
    }
}

/// Finds a catalog tube from a free-form label such as `"W&N Burnt Umber 37ml"`.
#[derive(Debug, Default, Clone)]
pub struct PaintMatcher;

#[derive(Debug, Clone, PartialEq)]
pub struct PaintLabelMatch {
    pub paint: Paint,
    pub confidence: f64,
    pub source: String,
}

impl PaintLabelMatch {
    pub fn is_reliable(&self) -> bool {
        self.confidence >= 0.7
    }
}

impl PaintMatcher {
    const MINIMUM_CONFIDENCE: f64 = 0.45;

    const NOISE: &'static [&'static str] = &[
        "oil",
        "color",
        "colour",
        "couleur",
        "huile",
        "paint",
        "peinture",
        "artists",
        "artist",
        "professional",
        "fine",
        "ml",
        "tube",
        "series",
        "serie",
    ];

    pub fn new() -> Self {
        Self
    }

    pub fn match_label(&self, label: Option<&str>, catalogue: &[Paint]) -> Option<PaintLabelMatch> {
        let wanted = Self::words(label);
        if wanted.is_empty() {
            return None;
        }
        let source = label.unwrap_or_default().to_string();

        let mut best: Option<PaintLabelMatch> = None;
        for paint in catalogue {
            let conf = Self::score(&wanted, paint);
            if conf >= Self::MINIMUM_CONFIDENCE {
                let is_better = match &best {
                    None => true,
                    Some(prev) => conf > prev.confidence,
                };
                if is_better {
                    best = Some(PaintLabelMatch {
                        paint: paint.clone(),
                        confidence: conf,
                        source: source.clone(),
                    });
                }
            }
        }
        best
    }

    fn score(wanted: &IndexSet<String>, paint: &Paint) -> f64 {
        let brand_words = Self::words(Some(&paint.brand));
        let raw_name_words = Self::words(Some(&paint.name));
        let name_words = Self::distinctive(&raw_name_words, &brand_words);

        let asked_for: IndexSet<String> = wanted
            .iter()
            .filter(|w| !brand_words.contains(*w))
            .cloned()
            .collect();
        if asked_for.is_empty() || name_words.is_empty() {
            return 0.0;
        }

        let name_hits = asked_for.iter().filter(|w| name_words.contains(*w)).count();
        if name_hits == 0 {
            return 0.0;
        }

        let name_score = name_hits as f64 / name_words.len() as f64;
        let brand_named = wanted.iter().any(|w| brand_words.contains(w));
        let noise = 1.0 - (0.3_f64).min((asked_for.len() - name_hits) as f64 * 0.1);

        (name_score * noise + if brand_named { 0.2 } else { 0.0 }).min(1.0)
    }

    fn distinctive(words: &IndexSet<String>, brand_words: &IndexSet<String>) -> IndexSet<String> {
        let remaining: IndexSet<String> = words
            .iter()
            .filter(|w| !brand_words.contains(*w))
            .cloned()
            .collect();
        if remaining.is_empty() {
            words.clone()
        } else {
            remaining
        }
    }

    fn words(text: Option<&str>) -> IndexSet<String> {
        let Some(raw) = text else {
            return IndexSet::new();
        };
        if raw.trim().is_empty() {
            return IndexSet::new();
        }
        let mut plain = String::with_capacity(raw.len());
        for c in raw.to_lowercase().chars() {
            let folded = Self::fold_accent(c);
            if folded.is_ascii_lowercase() || folded.is_ascii_digit() {
                plain.push(folded);
            } else {
                plain.push(' ');
            }
        }
        plain
            .split(' ')
            .filter(|w| w.len() > 1 && !Self::NOISE.contains(w))
            .map(ToString::to_string)
            .collect()
    }

    fn fold_accent(c: char) -> char {
        match c {
            'à' | 'á' | 'â' | 'ä' | 'ã' | 'å' => 'a',
            'è' | 'é' | 'ê' | 'ë' => 'e',
            'ì' | 'í' | 'î' | 'ï' => 'i',
            'ò' | 'ó' | 'ô' | 'ö' | 'õ' => 'o',
            'ù' | 'ú' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            'ñ' => 'n',
            'ý' | 'ÿ' => 'y',
            other => other,
        }
    }
}

/// A catalog paint and its perceptual distance to a target color.
#[derive(Debug, Clone, PartialEq)]
pub struct PaintMatchByColour {
    pub paint: Paint,
    pub delta_e: f64,
}

/// Ranks paints by perceptual distance to a target color.
pub fn find_closest(target: Rgb, candidates: &[Paint], limit: usize) -> Vec<PaintMatchByColour> {
    let mut matches: Vec<PaintMatchByColour> = candidates
        .iter()
        .map(|p| PaintMatchByColour {
            paint: p.clone(),
            delta_e: delta_e_2000(target, p.color()),
        })
        .collect();
    matches.sort_by(|a, b| a.delta_e.total_cmp(&b.delta_e));
    matches.truncate(limit);
    matches
}

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
