//! Painting techniques across oil and acrylic systems.

use super::medium::Medium;
use super::properties::{LayerThickness, PaintSystem};
use super::{fold, strip_accents};
use crate::error::DomainError;
use crate::i18n::Locale;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Miniature painting technique, along with its typical settings.
///
/// Some techniques belong to one system only (oil rendering, wet-on-wet blending and
/// streaking are oil work; layering, drybrush and edge highlighting are acrylic work),
/// others exist in both and change their thinner and their advice with the system.
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
    Layering,
    WetBlending,
    Drybrush,
    EdgeHighlight,
    ShadeWash,
    ContrastPaint,
}

impl Technique {
    pub const ALL: [Technique; 16] = [
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
        Technique::Layering,
        Technique::WetBlending,
        Technique::Drybrush,
        Technique::EdgeHighlight,
        Technique::ShadeWash,
        Technique::ContrastPaint,
    ];

    /// The only system this technique belongs to, or `None` when it exists in both.
    pub fn native_system(&self) -> Option<PaintSystem> {
        match self {
            Technique::OilWash
            | Technique::Blending
            | Technique::DotFading
            | Technique::StreakingGrime
            | Technique::OilRendering => Some(PaintSystem::Oil),
            Technique::Layering
            | Technique::WetBlending
            | Technique::Drybrush
            | Technique::EdgeHighlight
            | Technique::ShadeWash
            | Technique::ContrastPaint => Some(PaintSystem::Acrylic),
            Technique::PinWash
            | Technique::Filter
            | Technique::Glaze
            | Technique::BaseLayer
            | Technique::Highlight => None,
        }
    }

    /// Whether this technique can be done in the given system.
    pub fn fits(&self, system: PaintSystem) -> bool {
        self.native_system().is_none_or(|s| s == system)
    }

    /// The system a coat done with this technique is painted in, within a piece painted
    /// in `project`: a technique that exists in one system only imposes it (an oil wash
    /// on an acrylic piece is still oil), a shared one follows the piece.
    pub fn system_within(&self, project: PaintSystem) -> PaintSystem {
        self.native_system().unwrap_or(project)
    }

    /// The techniques offered when painting in the given system.
    pub fn for_system(system: PaintSystem) -> Vec<Technique> {
        Self::ALL.into_iter().filter(|t| t.fits(system)).collect()
    }

    pub fn label(&self, locale: Locale) -> &'static str {
        match locale {
            Locale::Fr => match self {
                Technique::OilWash => "Jus à l'huile",
                Technique::PinWash => "Jus capillaire (pin wash)",
                Technique::Filter => "Filtre",
                Technique::Glaze => "Glacis",
                Technique::Blending => "Fondu / dégradé",
                Technique::DotFading => "Dot fading",
                Technique::StreakingGrime => "Coulures et salissures",
                Technique::OilRendering => "Oil Paint Rendering (OPR)",
                Technique::BaseLayer => "Aplat de base",
                Technique::Highlight => "Éclaircis et points lumineux",
                Technique::Layering => "Couches superposées (layering)",
                Technique::WetBlending => "Fondu humide (wet blending)",
                Technique::Drybrush => "Brossage à sec (drybrush)",
                Technique::EdgeHighlight => "Liserés d'arêtes (edge highlight)",
                Technique::ShadeWash => "Lavis d'ombrage",
                Technique::ContrastPaint => "Peinture contraste (speedpaint)",
            },
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
                Technique::Layering => "Layering",
                Technique::WetBlending => "Wet blending",
                Technique::Drybrush => "Drybrush",
                Technique::EdgeHighlight => "Edge highlight",
                Technique::ShadeWash => "Shade wash",
                Technique::ContrastPaint => "Contrast paint (speedpaint)",
            },
        }
    }

    /// Medium of the technique in its native system (oil for shared techniques).
    pub fn default_medium(&self) -> Medium {
        self.default_medium_in(self.system_within(PaintSystem::Oil))
    }

    /// Medium of the technique when done in `system`.
    pub fn default_medium_in(&self, system: PaintSystem) -> Medium {
        match self.system_within(system) {
            PaintSystem::Oil => match self {
                Technique::Blending | Technique::DotFading | Technique::Highlight => Medium::None,
                _ => Medium::OdorlessThinner,
            },
            PaintSystem::Acrylic => match self {
                Technique::Drybrush | Technique::Highlight | Technique::ContrastPaint => {
                    Medium::None
                }
                Technique::Glaze | Technique::ShadeWash => Medium::AcrylicGlazeMedium,
                Technique::WetBlending => Medium::Retarder,
                _ => Medium::Water,
            },
        }
    }

    /// Lowest useful medium ratio, in the technique's native system.
    pub fn min_ratio(&self) -> f64 {
        self.ratio_range_in(self.system_within(PaintSystem::Oil)).0
    }

    /// Highest useful medium ratio, in the technique's native system.
    pub fn max_ratio(&self) -> f64 {
        self.ratio_range_in(self.system_within(PaintSystem::Oil)).1
    }

    pub fn default_ratio(&self) -> f64 {
        self.default_ratio_in(self.system_within(PaintSystem::Oil))
    }

    pub fn default_ratio_in(&self, system: PaintSystem) -> f64 {
        let (min, max) = self.ratio_range_in(system);
        (min + max) / 2.0
    }

    /// Useful range of the default medium's ratio when the technique is done in `system`.
    pub fn ratio_range_in(&self, system: PaintSystem) -> (f64, f64) {
        match (self.system_within(system), self) {
            (PaintSystem::Oil, Technique::OilWash) => (0.80, 0.92),
            (PaintSystem::Oil, Technique::PinWash) => (0.85, 0.94),
            (PaintSystem::Oil, Technique::Filter) => (0.88, 0.96),
            (PaintSystem::Oil, Technique::Glaze) => (0.70, 0.88),
            (PaintSystem::Oil, Technique::Blending) => (0.00, 0.25),
            (PaintSystem::Oil, Technique::DotFading) => (0.00, 0.20),
            (PaintSystem::Oil, Technique::StreakingGrime) => (0.60, 0.85),
            (PaintSystem::Oil, Technique::OilRendering) => (0.50, 0.80),
            (PaintSystem::Oil, Technique::BaseLayer) => (0.10, 0.35),
            (PaintSystem::Oil, Technique::Highlight) => (0.00, 0.20),
            (PaintSystem::Acrylic, Technique::PinWash) => (0.60, 0.80),
            (PaintSystem::Acrylic, Technique::Filter) => (0.80, 0.90),
            (PaintSystem::Acrylic, Technique::Glaze) => (0.50, 0.75),
            (PaintSystem::Acrylic, Technique::BaseLayer) => (0.20, 0.35),
            (PaintSystem::Acrylic, Technique::Highlight) => (0.00, 0.00),
            (PaintSystem::Acrylic, Technique::Layering) => (0.30, 0.50),
            (PaintSystem::Acrylic, Technique::WetBlending) => (0.10, 0.20),
            (PaintSystem::Acrylic, Technique::Drybrush) => (0.00, 0.00),
            (PaintSystem::Acrylic, Technique::EdgeHighlight) => (0.20, 0.40),
            (PaintSystem::Acrylic, Technique::ShadeWash) => (0.50, 0.70),
            (PaintSystem::Acrylic, Technique::ContrastPaint) => (0.00, 0.00),
            // Unreachable: system_within never pairs a native technique with the other system.
            _ => (0.0, 0.0),
        }
    }

    pub fn typical_thickness(&self) -> LayerThickness {
        match self {
            Technique::OilWash
            | Technique::PinWash
            | Technique::Filter
            | Technique::Glaze
            | Technique::ShadeWash => LayerThickness::Glaze,
            Technique::BaseLayer => LayerThickness::Normal,
            _ => LayerThickness::Thin,
        }
    }

    /// Whether, in oil, the previous coat must be fully dry before this technique.
    ///
    /// Only meaningful between two oil coats: acrylic coats are sealed once touch dry
    /// and only need their recoat time. See [`crate::drying::waits_for_through_dry`].
    pub fn requires_cured_base(&self) -> bool {
        !matches!(self, Technique::BaseLayer) && self.native_system() != Some(PaintSystem::Acrylic)
    }

    /// Tips for the technique when done in `system`.
    pub fn tips_in(&self, system: PaintSystem, locale: Locale) -> &'static [&'static str] {
        let system = self.system_within(system);
        match (locale, system) {
            (Locale::Fr, PaintSystem::Oil) => self.oil_tips_fr(),
            (Locale::En, PaintSystem::Oil) => self.oil_tips_en(),
            (Locale::Fr, PaintSystem::Acrylic) => self.acrylic_tips_fr(),
            (Locale::En, PaintSystem::Acrylic) => self.acrylic_tips_en(),
        }
    }

    fn oil_tips_fr(&self) -> &'static [&'static str] {
        match self {
            Technique::OilWash => &[
                "Le vernis brillant en dessous doit être sec depuis 24 h au minimum.",
                "Charger le pinceau puis l'essuyer : le jus doit couler, pas s'etaler.",
                "Retirer l'excès au coton-tige à peine humide de diluant avant la prise.",
            ],
            Technique::PinWash => &[
                "Poser la pointe du pinceau dans le creux et laisser la capillarite faire le trajet.",
                "Un seul passage par ligne : repasser dessus casse le depot.",
            ],
            Technique::Filter => &[
                "Très dilué : le filtre unifie une teinte, il ne la remplace pas.",
                "Deux filtres legers valent mieux qu'un seul charge.",
            ],
            Technique::Glaze => &[
                "Privilegier une huile transparente : une opaque tue la profondeur.",
                "Laisser sécher entre deux glacis, sinon la couche précédente se relève.",
            ],
            Technique::Blending => &[
                "Travailler dans le temps ouvert : une fois la prise commencee, arreter.",
                "Pinceau propre et sec pour tirer la transition, essuyé à chaque passage.",
            ],
            Technique::DotFading => &[
                "Poser des points minuscules de plusieurs teintes, puis les tirer vers le bas.",
                "Moins de peinture que ce que l'on croit : le pinceau doit être presque sec.",
            ],
            Technique::StreakingGrime => &[
                "Tracer le trait, laisser mordre une a deux minutes, puis tirer vers le bas.",
                "Suivre le sens de l'eau : verticales sur les flancs, en eventail sous les rivets.",
            ],
            Technique::OilRendering => &[
                "Poser clair et foncé côte à côte, puis fondre la frontière.",
                "Construire en plusieurs séances plutôt qu'en une couche chargée.",
            ],
            Technique::BaseLayer => &[
                "À l'huile, un aplat de base reste rare : il sèche lentement et bloque la suite.",
                "Utile surtout sur les grandes surfaces lisses (bustes, 1/10).",
            ],
            Technique::Highlight => &[
                "Une huile opaque tient mieux le point lumineux qu'une transparente.",
                "Le blanc de titane sèche très lentement : prévoir la couche suivante à distance.",
            ],
            _ => &[],
        }
    }

    fn oil_tips_en(&self) -> &'static [&'static str] {
        match self {
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
            _ => &[],
        }
    }

    fn acrylic_tips_fr(&self) -> &'static [&'static str] {
        match self {
            Technique::PinWash => &[
                "Une goutte de medium ou de mouillant casse la tension de surface : le lavis file dans le creux au lieu de perler.",
                "L'acrylique ne se reprend pas une fois sec : nettoyer les debordements tout de suite au pinceau humide.",
            ],
            Technique::Filter => &[
                "Très dilué, au médium plutôt qu'à l'eau seule, pour ne pas laisser d'auréoles.",
                "Sur une surface satinee ou brillante, le filtre s'etale sans taches.",
            ],
            Technique::Glaze => &[
                "Diluer au médium à glacis plutôt qu'à l'eau : la couche reste transparente sans poudrer.",
                "Pinceau presque sec, plusieurs passages fins : l'acrylique sèche entre deux et se superpose.",
            ],
            Technique::BaseLayer => &[
                "Deux couches fines valent mieux qu'une épaisse : consistance de lait entier.",
                "Laisser sécher quelques minutes entre les deux : la seconde couvre sans arracher la première.",
            ],
            Technique::Highlight => &[
                "Peinture opaque, à peine diluée, sur la pointe du pinceau.",
                "Pas de blanc pur d'emblee : monter par paliers, le point le plus clair en dernier.",
            ],
            Technique::Layering => &[
                "Chaque couche un peu plus claire et un peu plus petite que la précédente.",
                "Consistance de lait : si la couche marque le pinceau, ajouter une goutte d'eau.",
            ],
            Technique::WetBlending => &[
                "Quelques gouttes de retardateur : l'acrylique sèche sinon avant d'avoir fondu.",
                "Poser les deux teintes côte à côte encore humides et tirer la frontière en un passage.",
            ],
            Technique::Drybrush => &[
                "Charger puis essuyer le pinceau presque entierement sur un papier absorbant.",
                "Un vieux pinceau plat et dur : le brossage use les poils.",
            ],
            Technique::EdgeHighlight => &[
                "Le plat du pinceau sur l'arete, pas la pointe : le trait reste fin et regulier.",
                "Peinture légèrement diluée, sinon elle fait bourrelet.",
            ],
            Technique::ShadeWash => &[
                "Laisser couler dans les creux sans repasser : le lavis se fixe en séchant.",
                "Absorber les flaques sur les surfaces planes avec un pinceau essore.",
            ],
            Technique::ContrastPaint => &[
                "Sur une sous-couche claire et lisse : la teinte dépend de ce qu'il y a dessous.",
                "Une seule couche généreuse, sans y revenir pendant qu'elle sèche.",
            ],
            _ => &[],
        }
    }

    fn acrylic_tips_en(&self) -> &'static [&'static str] {
        match self {
            Technique::PinWash => &[
                "A drop of medium or flow aid breaks surface tension: the wash runs into the recess instead of beading.",
                "Acrylic cannot be reworked once dry: clean overflow right away with a damp brush.",
            ],
            Technique::Filter => &[
                "Heavily thinned, with medium rather than water alone, to avoid tide marks.",
                "On a satin or gloss surface, the filter spreads without staining.",
            ],
            Technique::Glaze => &[
                "Thin with glaze medium rather than water: the coat stays transparent without turning chalky.",
                "Nearly dry brush, several thin passes: acrylic dries between them and stacks up.",
            ],
            Technique::BaseLayer => &[
                "Two thin coats beat one thick one: whole-milk consistency.",
                "Let it dry a few minutes in between: the second coat covers without lifting the first.",
            ],
            Technique::Highlight => &[
                "Opaque paint, barely thinned, on the tip of the brush.",
                "No pure white straight away: build up in steps, brightest point last.",
            ],
            Technique::Layering => &[
                "Each layer slightly lighter and slightly smaller than the one below.",
                "Milk consistency: if the coat shows brush marks, add a drop of water.",
            ],
            Technique::WetBlending => &[
                "A few drops of retarder: otherwise acrylic dries before it blends.",
                "Lay both hues side by side while wet and pull the boundary in one stroke.",
            ],
            Technique::Drybrush => &[
                "Load then wipe the brush almost completely on a paper towel.",
                "Use an old, stiff flat brush: drybrushing wears the bristles.",
            ],
            Technique::EdgeHighlight => &[
                "The flat of the brush on the edge, not the tip: the line stays thin and even.",
                "Slightly thinned paint, otherwise it leaves a ridge.",
            ],
            Technique::ShadeWash => &[
                "Let it flow into recesses without going back: the wash sets as it dries.",
                "Soak up pools on flat areas with a squeezed-out brush.",
            ],
            Technique::ContrastPaint => &[
                "Over a light, smooth undercoat: the hue depends on what lies beneath.",
                "One generous coat, without going back over it while it dries.",
            ],
            _ => &[],
        }
    }

    /// The technique named by `raw`: its code, or its French or English label, ignoring
    /// case and accents (so labels saved before accents were restored still match).
    pub fn parse(raw: &str) -> Option<Technique> {
        let wanted = fold(raw);
        if wanted.is_empty() {
            return None;
        }
        Self::ALL.into_iter().find(|t| {
            fold(t.as_db_str()) == wanted
                || fold(t.label(Locale::Fr)) == wanted
                || fold(t.label(Locale::En)) == wanted
        })
    }

    pub fn by_label(label: Option<&str>, fallback: Technique) -> Technique {
        label.and_then(Self::parse).unwrap_or(fallback)
    }

    /// The technique a new coat starts with in a piece painted in `system`.
    pub fn default_for(system: PaintSystem) -> Technique {
        match system {
            PaintSystem::Oil => Technique::Glaze,
            PaintSystem::Acrylic => Technique::Layering,
        }
    }

    /// Like [`Technique::by_label`], with a fallback that matches the system of the piece.
    pub fn by_label_in(label: Option<&str>, system: PaintSystem) -> Technique {
        Self::by_label(label, Self::default_for(system))
    }

    /// The French label without accents, as the Java version of the application wrote it.
    /// Used in backups so that the older application still recognises the technique.
    pub fn legacy_label(&self) -> String {
        strip_accents(self.label(Locale::Fr))
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
            Technique::Layering => "LAYERING",
            Technique::WetBlending => "WET_BLENDING",
            Technique::Drybrush => "DRYBRUSH",
            Technique::EdgeHighlight => "EDGE_HIGHLIGHT",
            Technique::ShadeWash => "SHADE_WASH",
            Technique::ContrastPaint => "CONTRAST_PAINT",
        }
    }
}

impl FromStr for Technique {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
            .ok_or_else(|| DomainError::InvalidArgument(format!("Technique inconnue : {}", s.trim())))
    }
}

impl fmt::Display for Technique {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label(Locale::Fr))
    }
}

