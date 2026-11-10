#![forbid(unsafe_code)]

//! Miniature painting domain models: acrylic-to-oil equivalence, 5-stage miniature ladders,
//! pin washes & reductive cleaning windows, weathering effects, batch squad painting, and OSL.

use crate::color::{Rgb, delta_e_2000};
use crate::i18n::Locale;
use serde::{Deserialize, Serialize};

/// Popular miniature acrylic paint brands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MiniatureBrand {
    Citadel,
    VallejoModelColor,
    VallejoGameColor,
    ArmyPainter,
    AkInteractive,
}

impl MiniatureBrand {
    pub const ALL: [MiniatureBrand; 5] = [
        MiniatureBrand::Citadel,
        MiniatureBrand::VallejoModelColor,
        MiniatureBrand::VallejoGameColor,
        MiniatureBrand::ArmyPainter,
        MiniatureBrand::AkInteractive,
    ];

    pub fn display_name(&self) -> &'static str {
        match self {
            MiniatureBrand::Citadel => "Citadel / Games Workshop",
            MiniatureBrand::VallejoModelColor => "Vallejo Model Color",
            MiniatureBrand::VallejoGameColor => "Vallejo Game Color",
            MiniatureBrand::ArmyPainter => "The Army Painter (Warpaints)",
            MiniatureBrand::AkInteractive => "AK Interactive 3rd Gen",
        }
    }
}

/// A popular miniature acrylic reference tone.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MiniatureAcrylic {
    pub brand: MiniatureBrand,
    pub name: &'static str,
    pub hex: &'static str,
    pub category: &'static str,
}

impl MiniatureAcrylic {
    pub fn rgb(&self) -> Rgb {
        Rgb::of_hex(self.hex).unwrap_or_else(|_| Rgb::new(0.5, 0.5, 0.5))
    }
}

/// Comprehensive built-in catalog of iconic miniature paints across Citadel, Vallejo, Army Painter, and AK.
pub const MINIATURE_ACRYLICS: &[MiniatureAcrylic] = &[
    // Citadel Reds & Warm tones
    MiniatureAcrylic { brand: MiniatureBrand::Citadel, name: "Khorne Red", hex: "#6A0001", category: "Red / Armor" },
    MiniatureAcrylic { brand: MiniatureBrand::Citadel, name: "Mephiston Red", hex: "#9A1115", category: "Red / Armor" },
    MiniatureAcrylic { brand: MiniatureBrand::Citadel, name: "Evil Sunz Scarlet", hex: "#C21814", category: "Red / Armor" },
    MiniatureAcrylic { brand: MiniatureBrand::Citadel, name: "Wild Rider Red", hex: "#EA2F28", category: "Red / Armor" },
    MiniatureAcrylic { brand: MiniatureBrand::Citadel, name: "Averland Sunset", hex: "#FBB813", category: "Yellow / Heraldry" },
    MiniatureAcrylic { brand: MiniatureBrand::Citadel, name: "Yriel Yellow", hex: "#FFDC00", category: "Yellow / Heraldry" },
    MiniatureAcrylic { brand: MiniatureBrand::Citadel, name: "Flash Gitz Yellow", hex: "#FFF300", category: "Yellow / Heraldry" },

    // Citadel Blues & Purples
    MiniatureAcrylic { brand: MiniatureBrand::Citadel, name: "Kantor Blue", hex: "#002040", category: "Blue / Space Marine" },
    MiniatureAcrylic { brand: MiniatureBrand::Citadel, name: "Macragge Blue", hex: "#0E3E7C", category: "Blue / Space Marine" },
    MiniatureAcrylic { brand: MiniatureBrand::Citadel, name: "Altdorf Guard Blue", hex: "#265DB0", category: "Blue / Space Marine" },
    MiniatureAcrylic { brand: MiniatureBrand::Citadel, name: "Calgar Blue", hex: "#427EC1", category: "Blue / Space Marine" },
    MiniatureAcrylic { brand: MiniatureBrand::Citadel, name: "Fenrisian Grey", hex: "#6D94B3", category: "Blue / Space Marine" },
    MiniatureAcrylic { brand: MiniatureBrand::Citadel, name: "Naggaroth Night", hex: "#3B2B4F", category: "Purple / Magic" },
    MiniatureAcrylic { brand: MiniatureBrand::Citadel, name: "Xereus Purple", hex: "#471F5E", category: "Purple / Magic" },
    MiniatureAcrylic { brand: MiniatureBrand::Citadel, name: "Genestealer Purple", hex: "#7658A5", category: "Purple / Magic" },

    // Citadel Greens
    MiniatureAcrylic { brand: MiniatureBrand::Citadel, name: "Caliban Green", hex: "#00401F", category: "Green / Dark Angels" },
    MiniatureAcrylic { brand: MiniatureBrand::Citadel, name: "Warpstone Glow", hex: "#1E7332", category: "Green / Dark Angels" },
    MiniatureAcrylic { brand: MiniatureBrand::Citadel, name: "Moot Green", hex: "#52B734", category: "Green / Plasma" },
    MiniatureAcrylic { brand: MiniatureBrand::Citadel, name: "Death Guard Green", hex: "#6D774D", category: "Green / Nurgle" },
    MiniatureAcrylic { brand: MiniatureBrand::Citadel, name: "Nurgling Green", hex: "#849B63", category: "Green / Nurgle" },
    MiniatureAcrylic { brand: MiniatureBrand::Citadel, name: "Orruk Flesh", hex: "#81A75B", category: "Green / Orks" },

    // Citadel Flesh & Leathers
    MiniatureAcrylic { brand: MiniatureBrand::Citadel, name: "Bugman's Glow", hex: "#804C43", category: "Flesh / Skin" },
    MiniatureAcrylic { brand: MiniatureBrand::Citadel, name: "Cadian Fleshtone", hex: "#C48A6F", category: "Flesh / Skin" },
    MiniatureAcrylic { brand: MiniatureBrand::Citadel, name: "Kislev Flesh", hex: "#D1A582", category: "Flesh / Skin" },
    MiniatureAcrylic { brand: MiniatureBrand::Citadel, name: "Flayed One Flesh", hex: "#E4D1AC", category: "Flesh / Highlight" },
    MiniatureAcrylic { brand: MiniatureBrand::Citadel, name: "Rhinox Hide", hex: "#463836", category: "Brown / Leather" },
    MiniatureAcrylic { brand: MiniatureBrand::Citadel, name: "Mournfang Brown", hex: "#62391F", category: "Brown / Leather" },
    MiniatureAcrylic { brand: MiniatureBrand::Citadel, name: "Skrag Brown", hex: "#8B4E1A", category: "Brown / Leather" },
    MiniatureAcrylic { brand: MiniatureBrand::Citadel, name: "Zandri Dust", hex: "#9E9162", category: "Khaki / Cloth" },
    MiniatureAcrylic { brand: MiniatureBrand::Citadel, name: "Ushabti Bone", hex: "#BBAD7A", category: "Bone / Parchment" },
    MiniatureAcrylic { brand: MiniatureBrand::Citadel, name: "Screaming Skull", hex: "#D4CE9E", category: "Bone / Parchment" },

    // Citadel Neutrals & Metallics (represented by undertone)
    MiniatureAcrylic { brand: MiniatureBrand::Citadel, name: "Abaddon Black", hex: "#1C1C1C", category: "Black / Neutral" },
    MiniatureAcrylic { brand: MiniatureBrand::Citadel, name: "Corvus Black", hex: "#272A2E", category: "Black / Neutral" },
    MiniatureAcrylic { brand: MiniatureBrand::Citadel, name: "Mechanicus Standard Grey", hex: "#3D4447", category: "Grey / Neutral" },
    MiniatureAcrylic { brand: MiniatureBrand::Citadel, name: "Dawnstone", hex: "#6E7474", category: "Grey / Neutral" },
    MiniatureAcrylic { brand: MiniatureBrand::Citadel, name: "Celestra Grey", hex: "#8FA3A3", category: "Grey / Light" },
    MiniatureAcrylic { brand: MiniatureBrand::Citadel, name: "Grey Seer", hex: "#A2A5A7", category: "Grey / Undercoat" },
    MiniatureAcrylic { brand: MiniatureBrand::Citadel, name: "Corax White", hex: "#D6D8D5", category: "White / Highlight" },
    MiniatureAcrylic { brand: MiniatureBrand::Citadel, name: "White Scar", hex: "#FFFFFF", category: "White / Pure" },

    // Vallejo Model Color
    MiniatureAcrylic { brand: MiniatureBrand::VallejoModelColor, name: "70.950 Black", hex: "#121212", category: "Black" },
    MiniatureAcrylic { brand: MiniatureBrand::VallejoModelColor, name: "70.951 White", hex: "#FDFDFD", category: "White" },
    MiniatureAcrylic { brand: MiniatureBrand::VallejoModelColor, name: "70.908 Carmine Red", hex: "#9E1E2C", category: "Red" },
    MiniatureAcrylic { brand: MiniatureBrand::VallejoModelColor, name: "70.926 Red", hex: "#C72527", category: "Red" },
    MiniatureAcrylic { brand: MiniatureBrand::VallejoModelColor, name: "70.963 Medium Blue", hex: "#194A80", category: "Blue" },
    MiniatureAcrylic { brand: MiniatureBrand::VallejoModelColor, name: "70.965 Prussian Blue", hex: "#1C2E46", category: "Blue" },
    MiniatureAcrylic { brand: MiniatureBrand::VallejoModelColor, name: "70.890 Refractive Green", hex: "#4C563D", category: "Green / Camo" },
    MiniatureAcrylic { brand: MiniatureBrand::VallejoModelColor, name: "70.822 German Camo Black Brown", hex: "#322A26", category: "Dark Brown / Shade" },
    MiniatureAcrylic { brand: MiniatureBrand::VallejoModelColor, name: "70.921 English Uniform", hex: "#7E6746", category: "Khaki / Leather" },
    MiniatureAcrylic { brand: MiniatureBrand::VallejoModelColor, name: "70.845 Sunny Skin Tone", hex: "#E9B78D", category: "Flesh / Skin" },
    MiniatureAcrylic { brand: MiniatureBrand::VallejoModelColor, name: "70.858 Ice Yellow", hex: "#F2E8A0", category: "Highlight / Desaturated Yellow" },

    // Vallejo Game Color
    MiniatureAcrylic { brand: MiniatureBrand::VallejoGameColor, name: "72.008 Orange Fire", hex: "#EC631E", category: "Orange / Fire" },
    MiniatureAcrylic { brand: MiniatureBrand::VallejoGameColor, name: "72.012 Scarlet Blood", hex: "#A81D23", category: "Red / Blood" },
    MiniatureAcrylic { brand: MiniatureBrand::VallejoGameColor, name: "72.021 Magic Blue", hex: "#0067A3", category: "Blue / Magic" },
    MiniatureAcrylic { brand: MiniatureBrand::VallejoGameColor, name: "72.029 Sick Green", hex: "#2F6E3B", category: "Green" },
    MiniatureAcrylic { brand: MiniatureBrand::VallejoGameColor, name: "72.043 Beasty Brown", hex: "#5C3E28", category: "Brown / Fur" },
    MiniatureAcrylic { brand: MiniatureBrand::VallejoGameColor, name: "72.099 Cadmium Skin", hex: "#D99371", category: "Flesh" },

    // The Army Painter Warpaints
    MiniatureAcrylic { brand: MiniatureBrand::ArmyPainter, name: "Matt Black", hex: "#181818", category: "Black" },
    MiniatureAcrylic { brand: MiniatureBrand::ArmyPainter, name: "Matt White", hex: "#F7F7F7", category: "White" },
    MiniatureAcrylic { brand: MiniatureBrand::ArmyPainter, name: "Pure Red", hex: "#B81D1E", category: "Red" },
    MiniatureAcrylic { brand: MiniatureBrand::ArmyPainter, name: "Ultramarine Blue", hex: "#1C3B7A", category: "Blue" },
    MiniatureAcrylic { brand: MiniatureBrand::ArmyPainter, name: "Angel Green", hex: "#1B3B2B", category: "Green" },
    MiniatureAcrylic { brand: MiniatureBrand::ArmyPainter, name: "Barbarian Flesh", hex: "#C78C73", category: "Flesh" },
    MiniatureAcrylic { brand: MiniatureBrand::ArmyPainter, name: "Leather Brown", hex: "#543823", category: "Brown" },
    MiniatureAcrylic { brand: MiniatureBrand::ArmyPainter, name: "Skeleton Bone", hex: "#C8B991", category: "Bone" },

    // AK Interactive 3rd Gen
    MiniatureAcrylic { brand: MiniatureBrand::AkInteractive, name: "AK11001 White", hex: "#FAFAFA", category: "White" },
    MiniatureAcrylic { brand: MiniatureBrand::AkInteractive, name: "AK11029 Black", hex: "#191919", category: "Black" },
    MiniatureAcrylic { brand: MiniatureBrand::AkInteractive, name: "AK11097 Dirty Red", hex: "#7E2325", category: "Red" },
    MiniatureAcrylic { brand: MiniatureBrand::AkInteractive, name: "AK11184 Medium Blue", hex: "#224F88", category: "Blue" },
    MiniatureAcrylic { brand: MiniatureBrand::AkInteractive, name: "AK11142 Deep Green", hex: "#1B4729", category: "Green" },
    MiniatureAcrylic { brand: MiniatureBrand::AkInteractive, name: "AK11075 Deep Brown", hex: "#3B2C24", category: "Brown" },
    MiniatureAcrylic { brand: MiniatureBrand::AkInteractive, name: "AK11055 Pale Flesh", hex: "#E1BCA6", category: "Flesh" },
];

/// Finds miniature acrylics matching a search term or brand.
pub fn search_miniature_acrylics(
    query: &str,
    brand_filter: Option<MiniatureBrand>,
) -> Vec<&'static MiniatureAcrylic> {
    let q = query.trim().to_lowercase();
    MINIATURE_ACRYLICS
        .iter()
        .filter(|a| {
            if let Some(b) = brand_filter
                && a.brand != b {
                return false;
            }
            if q.is_empty() {
                true
            } else {
                a.name.to_lowercase().contains(&q)
                    || a.category.to_lowercase().contains(&q)
                    || a.hex.to_lowercase().contains(&q)
            }
        })
        .collect()
}

/// Finds the closest miniature acrylic tone by perceptual Delta-E 2000.
pub fn find_closest_acrylic(target: Rgb) -> Option<&'static MiniatureAcrylic> {
    MINIATURE_ACRYLICS
        .iter()
        .min_by(|a, b| {
            let d_a = delta_e_2000(target, a.rgb());
            let d_b = delta_e_2000(target, b.rgb());
            d_a.total_cmp(&d_b)
        })
}

/// Standard 5-step miniature painting ladder (Deep shadow -> Shadow -> Basecoat -> Volumetric Highlight -> Edge Highlight).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MiniatureLadder {
    pub deep_shadow: Rgb,
    pub shadow: Rgb,
    pub basecoat: Rgb,
    pub highlight: Rgb,
    pub edge_highlight: Rgb,
}

fn rgb_to_hsv(rgb: Rgb) -> (f64, f64, f64) {
    let r = rgb.r();
    let g = rgb.g();
    let b = rgb.b();
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let d = max - min;
    let s = if max <= 0.0 { 0.0 } else { d / max };
    let v = max;
    let h = if d.abs() < 1e-6 {
        0.0
    } else if (max - r).abs() < 1e-6 {
        60.0 * (((g - b) / d).rem_euclid(6.0))
    } else if (max - g).abs() < 1e-6 {
        60.0 * (((b - r) / d) + 2.0)
    } else {
        60.0 * (((r - g) / d) + 4.0)
    };
    (h, s, v)
}

fn hsv_to_rgb(h: f64, s: f64, v: f64) -> Rgb {
    let s = s.clamp(0.0, 1.0);
    let v = v.clamp(0.0, 1.0);
    let c = v * s;
    let h_prime = (h.rem_euclid(360.0)) / 60.0;
    let x = c * (1.0 - ((h_prime % 2.0) - 1.0).abs());
    let (r1, g1, b1) = if h_prime < 1.0 {
        (c, x, 0.0)
    } else if h_prime < 2.0 {
        (x, c, 0.0)
    } else if h_prime < 3.0 {
        (0.0, c, x)
    } else if h_prime < 4.0 {
        (0.0, x, c)
    } else if h_prime < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    let m = v - c;
    Rgb::new(r1 + m, g1 + m, b1 + m)
}

impl MiniatureLadder {
    /// Generates a coherent 5-step ladder from a basecoat color, respecting chromatic shift
    /// (cooler deep shadows via umber/indigo tones, warmer or brighter volumetric highlights).
    pub fn of_basecoat(base: Rgb) -> Self {
        let (h, s, v) = rgb_to_hsv(base);

        // 1. Deep Shadow: darker (v*0.28), slightly desaturated and pushed towards cool earth / blue
        let ds_h = if (30.0..=90.0).contains(&h) { (h - 20.0).rem_euclid(360.0) } else { h };
        let ds = hsv_to_rgb(ds_h, (s * 1.15).min(1.0), (v * 0.32).max(0.04));

        // 2. Shadow: form shadow (v*0.60)
        let s_tone = hsv_to_rgb(h, (s * 1.05).min(1.0), (v * 0.62).max(0.08));

        // 3. Basecoat: untouched
        let bc = base;

        // 4. Volumetric Highlight: higher value, slightly reduced saturation (v*1.28, s*0.82)
        let hl_h = if (0.0..=50.0).contains(&h) { (h + 8.0).min(60.0) } else { h };
        let hl = hsv_to_rgb(hl_h, (s * 0.78).max(0.08), (v * 1.25).min(0.96));

        // 5. Edge Highlight / Catchlight: crisp arris line, high value (v*1.42, s*0.45)
        let eh_h = if (30.0..=90.0).contains(&h) { (h + 5.0).min(65.0) } else { h };
        let eh = hsv_to_rgb(eh_h, (s * 0.42).max(0.04), (v * 1.45).min(0.99));

        Self {
            deep_shadow: ds,
            shadow: s_tone,
            basecoat: bc,
            highlight: hl,
            edge_highlight: eh,
        }
    }

    pub fn steps_hex(&self) -> [String; 5] {
        [
            self.deep_shadow.to_hex(),
            self.shadow.to_hex(),
            self.basecoat.to_hex(),
            self.highlight.to_hex(),
            self.edge_highlight.to_hex(),
        ]
    }

    pub fn role_labels_i18n(&self, locale: Locale) -> [&'static str; 5] {
        match locale {
            Locale::Fr => [
                "1. Ombre profonde / Creux d'occlusion",
                "2. Ombre de forme",
                "3. Couleur de base",
                "4. Éclaircissement volumétrique",
                "5. Rehaut d'arête / Spéculaire",
            ],
            Locale::En => [
                "1. Deep Occlusion Shadow",
                "2. Form Shadow",
                "3. Basecoat",
                "4. Volumetric Highlight",
                "5. Edge Highlight / Specular",
            ],
        }
    }
}

/// Weathering & Grimdark effects library for miniature modellers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WeatheringEffect {
    StreakingGrime,
    FreshRust,
    CrustyOldRust,
    VerdigrisPatina,
    DustDeposit,
    DryMud,
    FreshFuelOil,
}

impl WeatheringEffect {
    pub const ALL: [WeatheringEffect; 7] = [
        WeatheringEffect::StreakingGrime,
        WeatheringEffect::FreshRust,
        WeatheringEffect::CrustyOldRust,
        WeatheringEffect::VerdigrisPatina,
        WeatheringEffect::DustDeposit,
        WeatheringEffect::DryMud,
        WeatheringEffect::FreshFuelOil,
    ];

    pub fn label_i18n(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::StreakingGrime, Locale::Fr) => "Streaking Grime (Coulures de crasse & pluie)",
            (Self::StreakingGrime, Locale::En) => "Streaking Grime (Rain & Grim streaks)",
            (Self::FreshRust, Locale::Fr) => "Rouille vive / Fraîche",
            (Self::FreshRust, Locale::En) => "Fresh Bright Rust",
            (Self::CrustyOldRust, Locale::Fr) => "Rouille ancienne & encroûtée",
            (Self::CrustyOldRust, Locale::En) => "Old Crusty Rust",
            (Self::VerdigrisPatina, Locale::Fr) => "Vert-de-gris / Patine de bronze & cuivre",
            (Self::VerdigrisPatina, Locale::En) => "Verdigris / Bronze & Copper patina",
            (Self::DustDeposit, Locale::Fr) => "Dépôt de poussière sèche d'atelier",
            (Self::DustDeposit, Locale::En) => "Dry Dust Deposit",
            (Self::DryMud, Locale::Fr) => "Boue séchée / Éclaboussures",
            (Self::DryMud, Locale::En) => "Dry Mud & Splatter",
            (Self::FreshFuelOil, Locale::Fr) => "Fuite de carburant & graisse fraîche",
            (Self::FreshFuelOil, Locale::En) => "Fresh Fuel & Oil Stains",
        }
    }

    pub fn hex_color(&self) -> &'static str {
        match self {
            Self::StreakingGrime => "#4A3E2D",
            Self::FreshRust => "#C85A17",
            Self::CrustyOldRust => "#652C16",
            Self::VerdigrisPatina => "#40A892",
            Self::DustDeposit => "#C2B294",
            Self::DryMud => "#594A36",
            Self::FreshFuelOil => "#1B1713",
        }
    }

    pub fn oil_recipe_i18n(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::StreakingGrime, Locale::Fr) => "3 parts Terre d'Ombre Naturelle (PBr7) + 1 part Noir de Vigne/Ivoire (PBk9) + diluant minéral sans odeur",
            (Self::StreakingGrime, Locale::En) => "3 parts Raw Umber (PBr7) + 1 part Vine/Ivory Black (PBk9) + odorless mineral spirits",
            (Self::FreshRust, Locale::Fr) => "3 parts Terre de Sienne Brûlée (PBr7/PR101) + 1 part Orange de Cadmium (PO20/PY35)",
            (Self::FreshRust, Locale::En) => "3 parts Burnt Sienna (PBr7/PR101) + 1 part Cadmium Orange (PO20/PY35)",
            (Self::CrustyOldRust, Locale::Fr) => "4 parts Oxyde Rouge de Fer (PR101) + 1 part Terre d'Ombre Brûlée (PBr7)",
            (Self::CrustyOldRust, Locale::En) => "4 parts Red Iron Oxide (PR101) + 1 part Burnt Umber (PBr7)",
            (Self::VerdigrisPatina, Locale::Fr) => "3 parts Oxyde de Chrome Vert (PG17) + 1 part Bleu Phtalo (PB15) + 4 parts Blanc de Titane (PW6)",
            (Self::VerdigrisPatina, Locale::En) => "3 parts Chromium Oxide Green (PG17) + 1 part Phthalo Blue (PB15) + 4 parts Titanium White (PW6)",
            (Self::DustDeposit, Locale::Fr) => "3 parts Ocre Jaune (PY43) + 1 part Blanc de Titane (PW6) + 1 part Terre d'Ombre Naturelle",
            (Self::DustDeposit, Locale::En) => "3 parts Yellow Ochre (PY43) + 1 part Titanium White (PW6) + 1 part Raw Umber",
            (Self::DryMud, Locale::Fr) => "4 parts Terre d'Ombre Naturelle + 2 parts Ocre Jaune + 1 part Noir d'Ivoire",
            (Self::DryMud, Locale::En) => "4 parts Raw Umber + 2 parts Yellow Ochre + 1 part Ivory Black",
            (Self::FreshFuelOil, Locale::Fr) => "4 parts Noir d'Ivoire (PBk9) + 1 part Terre de Sienne Brûlée + vernis brillant pur",
            (Self::FreshFuelOil, Locale::En) => "4 parts Ivory Black (PBk9) + 1 part Burnt Sienna + pure gloss varnish",
        }
    }

    pub fn application_technique_i18n(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::StreakingGrime, Locale::Fr) => "Poser en micro-points sur les rivets, puis tirer verticalement au pinceau biseauté humecté d'essence sans odeur.",
            (Self::StreakingGrime, Locale::En) => "Apply dot on rivets, then feather vertically downward with an odorless spirit-dampened brush.",
            (Self::FreshRust, Locale::Fr) => "Appliquer au pinceau fin dans les creux des plaques métalliques ou tamponner à l'éponge blister.",
            (Self::FreshRust, Locale::En) => "Apply with fine brush into armor joints or stipple using blister sponge.",
            (Self::CrustyOldRust, Locale::Fr) => "Base texturée sous la rouille fraîche pour donner du relief aux plaques corrodées.",
            (Self::CrustyOldRust, Locale::En) => "Use as darker textured base beneath fresh bright rust spots.",
            (Self::VerdigrisPatina, Locale::Fr) => "Diluer en jus très liquide (1:8), laisser filer par capillarité dans les creux de cuivre/bronze.",
            (Self::VerdigrisPatina, Locale::En) => "Dilute to wash consistency (1:8), let flow by capillary action into bronze/copper recesses.",
            (Self::DustDeposit, Locale::Fr) => "Brosser à sec ou en voile ultra-mat sur le bas des manteaux, chenilles et socles.",
            (Self::DustDeposit, Locale::En) => "Drybrush or feather matte wash onto cloak hems, tank tracks, and bases.",
            (Self::DryMud, Locale::Fr) => "Projeter délicatement avec un cure-dent ou un pinceau à poils durs sur les bas de caisse.",
            (Self::DryMud, Locale::En) => "Flick using stiff brush or toothpick onto vehicle skirts and boots.",
            (Self::FreshFuelOil, Locale::Fr) => "Poser pur sur les bouchons de réservoir et vérins hydrauliques pour garder le brillant.",
            (Self::FreshFuelOil, Locale::En) => "Apply pure around fuel caps and pistons to preserve glossy wet appearance.",
        }
    }
}

/// Pin wash & reductive technique configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PinWashConfig {
    pub ratio_oil: u32,
    pub ratio_solvent: u32,
    pub min_cleaning_window_mins: u32,
    pub max_cleaning_window_mins: u32,
}

impl PinWashConfig {
    /// Armor panel pin wash (optimal capillary flow in panel lines and around rivets).
    pub fn armor_pin_wash() -> Self {
        Self {
            ratio_oil: 1,
            ratio_solvent: 6,
            min_cleaning_window_mins: 15,
            max_cleaning_window_mins: 90,
        }
    }

    /// Deep recess shadow wash.
    pub fn deep_recess_wash() -> Self {
        Self {
            ratio_oil: 1,
            ratio_solvent: 3,
            min_cleaning_window_mins: 20,
            max_cleaning_window_mins: 120,
        }
    }

    /// Overall armor tint filter.
    pub fn overall_filter() -> Self {
        Self {
            ratio_oil: 1,
            ratio_solvent: 12,
            min_cleaning_window_mins: 10,
            max_cleaning_window_mins: 60,
        }
    }

    pub fn capillary_flow_desc(&self, locale: Locale) -> &'static str {
        if self.ratio_solvent >= 10 {
            locale.tr("Capillarité ultra-haute (voile teinté sans taches)", "Ultra-high capillary flow (glaze filter)")
        } else if self.ratio_solvent >= 5 {
            locale.tr("Capillarité optimale (file instantanément le long des arêtes)", "Optimal capillary flow (snaps along panel lines)")
        } else {
            locale.tr("Consistance semi-dense (lavis concentré de creux)", "Semi-dense wash (concentrated recess shade)")
        }
    }
}

/// Batch painting tracker for an army squad / platoon.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchSquad {
    pub name: String,
    pub miniature_count: usize,
    pub steps: Vec<String>,
    /// Matrix of [miniature_idx][step_idx]
    pub progress: Vec<Vec<bool>>,
}

impl BatchSquad {
    pub fn new(name: impl Into<String>, miniature_count: usize, steps: Vec<String>) -> Self {
        let count = miniature_count.clamp(1, 100);
        let s_len = steps.len();
        let progress = vec![vec![false; s_len]; count];
        Self {
            name: name.into(),
            miniature_count: count,
            steps,
            progress,
        }
    }

    pub fn standard_space_marine_squad(name: &str, count: usize) -> Self {
        Self::new(
            name,
            count,
            vec![
                "1. Sous-couche zénithale".to_string(),
                "2. Base armure".to_string(),
                "3. Pin wash d'huile".to_string(),
                "4. Nettoyage réductif".to_string(),
                "5. Éclaircissement d'arêtes".to_string(),
                "6. Cuirs & Sacoches".to_string(),
                "7. Armes & Métaux".to_string(),
                "8. Lentilles / OSL".to_string(),
                "9. Weathering & Socle".to_string(),
            ],
        )
    }

    pub fn total_tasks(&self) -> usize {
        self.miniature_count * self.steps.len()
    }

    pub fn completed_tasks(&self) -> usize {
        self.progress
            .iter()
            .flat_map(|row| row.iter())
            .filter(|&&c| c)
            .count()
    }

    pub fn progress_pct(&self) -> f64 {
        let total = self.total_tasks();
        if total == 0 {
            return 100.0;
        }
        (self.completed_tasks() as f64 / total as f64) * 100.0
    }

    pub fn toggle(&mut self, miniature_idx: usize, step_idx: usize) {
        if let Some(cell) = self
            .progress
            .get_mut(miniature_idx)
            .and_then(|row| row.get_mut(step_idx))
        {
            *cell = !*cell;
        }
    }

    /// Recommended mother mix volume multiplier (e.g. 10 miniatures require ~2.5x volume of paint).
    pub fn mother_mix_multiplier(&self) -> f64 {
        1.0 + (self.miniature_count as f64 - 1.0) * 0.22
    }
}

/// Object Source Lighting (OSL) energy source preset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OslPreset {
    pub name_fr: &'static str,
    pub name_en: &'static str,
    pub core_hex: &'static str,
    pub glow_hex: &'static str,
    pub ambient_decay_hex: &'static str,
}

impl OslPreset {
    pub const ALL: [OslPreset; 5] = [
        OslPreset {
            name_fr: "Plasma Cyan / Épée Énergétique",
            name_en: "Plasma Cyan / Power Sword",
            core_hex: "#E0FFFF",
            glow_hex: "#00E5FF",
            ambient_decay_hex: "#00557A",
        },
        OslPreset {
            name_fr: "Flamme / Torche Ardente",
            name_en: "Flame / Burning Torch",
            core_hex: "#FFF7C2",
            glow_hex: "#FF6D00",
            ambient_decay_hex: "#8B1E00",
        },
        OslPreset {
            name_fr: "Rune Warp / Malepierre Verte",
            name_en: "Warpstone / Toxic Green",
            core_hex: "#E8FDE8",
            glow_hex: "#00E676",
            ambient_decay_hex: "#004D25",
        },
        OslPreset {
            name_fr: "Magie Arcane / Énergie Violette",
            name_en: "Arcane Magic / Purple Psi",
            core_hex: "#FCE4EC",
            glow_hex: "#D500F9",
            ambient_decay_hex: "#4A0072",
        },
        OslPreset {
            name_fr: "Optique Rouge / Viseur Tactique",
            name_en: "Red Sensor / Tactical Eye",
            core_hex: "#FFCDD2",
            glow_hex: "#FF1744",
            ambient_decay_hex: "#880E4F",
        },
    ];

    pub fn name_i18n(&self, locale: Locale) -> &'static str {
        match locale {
            Locale::Fr => self.name_fr,
            Locale::En => self.name_en,
        }
    }
}

/// Non-Metallic Metal (NMM) metal family simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NmmMetal {
    Gold,
    AncientGold,
    SteelChrome,
    CopperBronze,
}

impl NmmMetal {
    pub const ALL: [NmmMetal; 4] = [
        NmmMetal::Gold,
        NmmMetal::AncientGold,
        NmmMetal::SteelChrome,
        NmmMetal::CopperBronze,
    ];

    pub fn label_i18n(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::Gold, Locale::Fr) => "NMM Or Royal & Héraldique",
            (Self::Gold, Locale::En) => "NMM Royal Gold",
            (Self::AncientGold, Locale::Fr) => "NMM Or Ancien & Reliques",
            (Self::AncientGold, Locale::En) => "NMM Ancient Antique Gold",
            (Self::SteelChrome, Locale::Fr) => "NMM Acier & Lame Chromée",
            (Self::SteelChrome, Locale::En) => "NMM Polished Steel & Chrome",
            (Self::CopperBronze, Locale::Fr) => "NMM Cuivre & Bronze Chaud",
            (Self::CopperBronze, Locale::En) => "NMM Warm Copper & Bronze",
        }
    }

    pub fn steps_hex(&self) -> [&'static str; 5] {
        match self {
            Self::Gold => ["#301B04", "#7E541A", "#C28C24", "#F4DF76", "#FFFFFF"],
            Self::AncientGold => ["#24190F", "#5C4223", "#9B753B", "#D1BA85", "#FAFAFA"],
            Self::SteelChrome => ["#0F151B", "#2B3845", "#6B7C8E", "#B8C8D7", "#FFFFFF"],
            Self::CopperBronze => ["#36120B", "#772C19", "#BA5832", "#E89B74", "#FFF3EB"],
        }
    }

    pub fn oil_formula_i18n(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::Gold, Locale::Fr) => "Ombre : Terre d'Ombre Brûlée (PBr7) · Demi-teinte : Ocre Jaune (PY43) · Lumière : Jaune de Cadmium clair (PY35) · Rehaut : Blanc de Titane (PW6)",
            (Self::Gold, Locale::En) => "Shadow: Burnt Umber (PBr7) · Midtone: Yellow Ochre (PY43) · Light: Cadmium Yellow Light (PY35) · Catchlight: Titanium White (PW6)",
            (Self::AncientGold, Locale::Fr) => "Ombre : Terre d'Ombre Naturelle · Base : Terre de Sienne Naturelle · Rehauts : Ocre Jaune + Blanc de Titane en fine arête",
            (Self::AncientGold, Locale::En) => "Shadow: Raw Umber · Base: Raw Sienna · Catchlights: Yellow Ochre + Titanium White fine edge",
            (Self::SteelChrome, Locale::Fr) => "Ombre : Noir d'Ivoire (PBk9) + pointe Bleu de Prusse (PB27) · Demi-teinte : Gris neutre froid · Arête : Blanc de Titane pur",
            (Self::SteelChrome, Locale::En) => "Shadow: Ivory Black (PBk9) + hint of Prussian Blue (PB27) · Midtone: Neutral Cold Grey · Edge: Pure Titanium White",
            (Self::CopperBronze, Locale::Fr) => "Ombre : Terre de Sienne Brûlée + Noir · Base : Rouge d'Oxyde de Fer + Orange Cadmium · Rehauts : Rose chair clair",
            (Self::CopperBronze, Locale::En) => "Shadow: Burnt Sienna + Black · Base: Red Iron Oxide + Cadmium Orange · Catchlight: Light flesh tint",
        }
    }

    pub fn ground_bounce_advice_i18n(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::Gold, Locale::Fr) => "Placer un léger voile de vert olive ou de terre cuite sur la face inférieure de la lame pour simuler le reflet du sol.",
            (Self::Gold, Locale::En) => "Apply subtle olive-green or warm earth tint on underside surfaces to simulate ground bounce reflection.",
            (Self::AncientGold, Locale::Fr) => "Glacer un vert-de-gris translucide (PG17 + PB15) dans les creux les plus profonds des ornements.",
            (Self::AncientGold, Locale::En) => "Glaze transparent verdigris (PG17 + PB15) into the deepest ornament folds.",
            (Self::SteelChrome, Locale::Fr) => "Tracer une ligne d'horizon sombre et nette au milieu du biseau, avec le reflet du ciel au-dessus et le sol en dessous.",
            (Self::SteelChrome, Locale::En) => "Paint a sharp dark horizon boundary line along the bevel, with sky reflection above and earth below.",
            (Self::CopperBronze, Locale::Fr) => "Les reflets de sol sur le cuivre doivent rester chauds (ocres et bruns orangés).",
            (Self::CopperBronze, Locale::En) => "Ground reflections on copper must remain warm (warm ochres and terracotta browns).",
        }
    }
}

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

    /// Reaction acceleration factor based on Arrhenius rate law (Q10 ~ 2.1 for oil cross-linking).
    pub fn acceleration_factor(&self) -> f64 {
        let delta = self.temperature_celsius - 20.0;
        2.1f64.powf(delta / 10.0)
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

/// Special effects recipes & textured applications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpecialEffectGore {
    FreshGore,
    CoagulatedBlood,
    FrostIce,
    XenoSlime,
    GemLens,
}

impl SpecialEffectGore {
    pub const ALL: [SpecialEffectGore; 5] = [
        SpecialEffectGore::FreshGore,
        SpecialEffectGore::CoagulatedBlood,
        SpecialEffectGore::FrostIce,
        SpecialEffectGore::XenoSlime,
        SpecialEffectGore::GemLens,
    ];

    pub fn label_i18n(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::FreshGore, Locale::Fr) => "Sang Frais & Éclaboussures Récentes",
            (Self::FreshGore, Locale::En) => "Fresh Arterial Gore & Splatter",
            (Self::CoagulatedBlood, Locale::Fr) => "Sang Coagulé & Plaies Sèches",
            (Self::CoagulatedBlood, Locale::En) => "Coagulated Blood & Crusted Wounds",
            (Self::FrostIce, Locale::Fr) => "Givre & Glace de Socle",
            (Self::FrostIce, Locale::En) => "Frostbite & Tundra Ice",
            (Self::XenoSlime, Locale::Fr) => "Bave Xéno & Acide Toxique",
            (Self::XenoSlime, Locale::En) => "Xeno Slime & Toxic Acid",
            (Self::GemLens, Locale::Fr) => "Gemmes & Optiques Réfléchissantes",
            (Self::GemLens, Locale::En) => "Reflective Gemstones & Lenses",
        }
    }

    pub fn hex_color(&self) -> &'static str {
        match self {
            Self::FreshGore => "#8F0808",
            Self::CoagulatedBlood => "#3F0C0C",
            Self::FrostIce => "#D0F0FD",
            Self::XenoSlime => "#86D400",
            Self::GemLens => "#00BCD4",
        }
    }

    pub fn recipe_i18n(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::FreshGore, Locale::Fr) => "Rouge Alizarine/Phtalo (PR177) + pointe Terre d'Ombre Brûlée (PBr7) + vernis brillant pur",
            (Self::FreshGore, Locale::En) => "Alizarin/Phthalo Red (PR177) + hint of Burnt Umber (PBr7) + pure gloss varnish",
            (Self::CoagulatedBlood, Locale::Fr) => "Oxyde de Fer Rouge (PR101) + Noir d'Ivoire (PBk9) + fini satiné",
            (Self::CoagulatedBlood, Locale::En) => "Red Iron Oxide (PR101) + Ivory Black (PBk9) + satin finish",
            (Self::FrostIce, Locale::Fr) => "Blanc de Titane (PW6) + voile transparent Bleu Phtalo (PB15) + vernis satin mat",
            (Self::FrostIce, Locale::En) => "Titanium White (PW6) + transparent glaze Phthalo Blue (PB15) + satin-matte finish",
            (Self::XenoSlime, Locale::Fr) => "Jaune de Cadmium (PY35) + Bleu Phtalo (PB15) + vernis brillant pur étiré",
            (Self::XenoSlime, Locale::En) => "Cadmium Yellow (PY35) + Phthalo Blue (PB15) + pure high-gloss varnish",
            (Self::GemLens, Locale::Fr) => "Dégradé inverse : Noir en haut à 11h vers Cyan lumineux en bas à 5h + point spéculaire blanc pur à 11h",
            (Self::GemLens, Locale::En) => "Inverted gradient: Black at 11 o'clock to Bright Cyan at 5 o'clock + pure white dot at 11 o'clock",
        }
    }

    pub fn technique_i18n(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::FreshGore, Locale::Fr) => "Projeter avec une brosse à dents ou souffler à l'aérographe sur la pointe d'un pinceau chargé.",
            (Self::FreshGore, Locale::En) => "Flick using stiff toothbrush or airbrush pulse against loaded brush bristles.",
            (Self::CoagulatedBlood, Locale::Fr) => "Tamponner à la mousse de blister dans les creux des armures et sur les bords des lames.",
            (Self::CoagulatedBlood, Locale::En) => "Stipple with blister sponge inside armor recesses and along blade cutting edges.",
            (Self::FrostIce, Locale::Fr) => "Brosser à sec très léger sur le bord des manteaux et les pointes des touffes d'herbe.",
            (Self::FrostIce, Locale::En) => "Ultra-light drybrush on cloak hems and tips of tufts.",
            (Self::XenoSlime, Locale::Fr) => "Étendre des filaments de colle polymère (UHU) puis teinter au jus d'huile vert translucide.",
            (Self::XenoSlime, Locale::En) => "String out clear polymer glue strands (UHU), then glaze with translucent green oil tint.",
            (Self::GemLens, Locale::Fr) => "Poser une goutte de vernis brillant après séchage complet pour donner une courbure vitrée réelle.",
            (Self::GemLens, Locale::En) => "Place a domed droplet of gloss varnish after thorough curing for authentic 3D refraction.",
        }
    }
}

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

    pub fn label_i18n(&self, locale: Locale) -> &'static str {
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

/// Specialized oil mediums for miniature painting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OilMedium {
    LiquinFineDetail,
    RefinedLinseedOil,
    StandOil,
    CobaltDrier,
}

impl OilMedium {
    pub const ALL: [Self; 4] = [
        Self::LiquinFineDetail,
        Self::RefinedLinseedOil,
        Self::StandOil,
        Self::CobaltDrier,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Self::LiquinFineDetail => "Liquin Fine Detail (Winsor & Newton)",
            Self::RefinedLinseedOil => "Huile de Lin Raffinée Purifiée",
            Self::StandOil => "Standolie (Huile de Lin Épaissie sous Vide)",
            Self::CobaltDrier => "Siccatif de Cobalt / Courtrai",
        }
    }

    pub fn drying_effect_i18n(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::LiquinFineDetail, Locale::Fr) => "Accélérateur alkyd : sec au toucher en 6 à 12h",
            (Self::LiquinFineDetail, Locale::En) => "Alkyd accelerator: touch-dry in 6 to 12 hours",
            (Self::RefinedLinseedOil, Locale::Fr) => "Ralentisseur classique : séchage en 2 à 5 jours",
            (Self::RefinedLinseedOil, Locale::En) => "Classic retarder: dries in 2 to 5 days",
            (Self::StandOil, Locale::Fr) => "Ralentisseur lissant : séchage en 4 à 8 jours",
            (Self::StandOil, Locale::En) => "Leveling retarder: dries in 4 to 8 days",
            (Self::CobaltDrier, Locale::Fr) => "Hyper-accélérateur : sec en 3 à 6h (ultra-rapide)",
            (Self::CobaltDrier, Locale::En) => "Extreme drying agent: touch-dry in 3 to 6h",
        }
    }

    pub fn finish_i18n(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::LiquinFineDetail, Locale::Fr) => "Satiné lisse sans trace de pinceau, autonivelant parfait",
            (Self::LiquinFineDetail, Locale::En) => "Smooth satin, self-leveling without brush strokes",
            (Self::RefinedLinseedOil, Locale::Fr) => "Brillant soyeux, souple, rehausse les teintes chaudes",
            (Self::RefinedLinseedOil, Locale::En) => "Silky gloss, rich, enhances warm tones",
            (Self::StandOil, Locale::Fr) => "Fini laque émail vitreux, élimine toute texture",
            (Self::StandOil, Locale::En) => "Vitreous enamel glaze, eliminates all brush marks",
            (Self::CobaltDrier, Locale::Fr) => "Fini neutre mais risque de friperie si surdosé",
            (Self::CobaltDrier, Locale::En) => "Neutral sheen, high risk of wrinkling if overdosed",
        }
    }

    pub fn dosage_rule_i18n(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::LiquinFineDetail, Locale::Fr) => "10% à 30% du volume de pâte d'huile",
            (Self::LiquinFineDetail, Locale::En) => "10% to 30% of total oil paint volume",
            (Self::RefinedLinseedOil, Locale::Fr) => "Quelques gouttes pour allonger les glacis",
            (Self::RefinedLinseedOil, Locale::En) => "A few drops to loosen translucent glazes",
            (Self::StandOil, Locale::Fr) => "Diluer 50/50 avec de l'essence minérale sans odeur",
            (Self::StandOil, Locale::En) => "Thin 50/50 with odorless mineral spirits",
            (Self::CobaltDrier, Locale::Fr) => "MAXIMUM 1 goutte pour une noisette entière de peinture !",
            (Self::CobaltDrier, Locale::En) => "STRICT MAX 1 drop per whole hazelnut of paint!",
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

    pub fn label_i18n(&self, locale: Locale) -> &'static str {
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

    pub fn label_i18n(&self, locale: Locale) -> &'static str {
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

/// Preservation of mixed oil palettes using airtight boxes and freezer temperatures.
pub struct FreezerStorageSimulator;

impl FreezerStorageSimulator {
    /// Estimates days before skinning/curing occurs.
    pub fn estimated_usable_days(ambient_temp_c: f64, is_airtight: bool, is_in_freezer: bool) -> f64 {
        if is_in_freezer {
            // At -18°C, oxidation polymerisation reaction rate drops by ~16x
            28.0 // ~4 weeks of safe conservation
        } else if is_airtight {
            if ambient_temp_c > 25.0 {
                2.0
            } else {
                3.5
            }
        } else {
            // Open air
            if ambient_temp_c > 25.0 {
                0.75 // 18 hours
            } else {
                1.5 // 36 hours
            }
        }
    }
}

/// Dry weathering pigment reference with wet-shift simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DryPigmentEffect {
    BurntUmberDust,
    ElysiumDesertDust,
    SootBlack,
    FreshRustPowder,
    IndustrialAsh,
}

impl DryPigmentEffect {
    pub const ALL: [Self; 5] = [
        Self::BurntUmberDust,
        Self::ElysiumDesertDust,
        Self::SootBlack,
        Self::FreshRustPowder,
        Self::IndustrialAsh,
    ];

    pub fn label_i18n(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::BurntUmberDust, Locale::Fr) => "Poudre Terre d'Ombre Brûlée",
            (Self::BurntUmberDust, Locale::En) => "Burnt Umber Soil Dust",
            (Self::ElysiumDesertDust, Locale::Fr) => "Poussière Désertique / Sable",
            (Self::ElysiumDesertDust, Locale::En) => "Desert Dust / Dune Sand",
            (Self::SootBlack, Locale::Fr) => "Suie Noire d'Échappement & Canon",
            (Self::SootBlack, Locale::En) => "Exhaust & Barrel Soot Black",
            (Self::FreshRustPowder, Locale::Fr) => "Pigment Rouille Calcinée Vive",
            (Self::FreshRustPowder, Locale::En) => "Vivid Rust Powder",
            (Self::IndustrialAsh, Locale::Fr) => "Cendre & Gravats Urbains",
            (Self::IndustrialAsh, Locale::En) => "Urban Ash & Rubble",
        }
    }

    /// Dry powder appearance (lighter, chalky, matte).
    pub fn dry_hex(&self) -> &'static str {
        match self {
            Self::BurntUmberDust => "#7D583F",
            Self::ElysiumDesertDust => "#DEC6A5",
            Self::SootBlack => "#303030",
            Self::FreshRustPowder => "#E0682B",
            Self::IndustrialAsh => "#A8A5A0",
        }
    }

    /// Darkened appearance once wetted or fixed with solvent / binder.
    pub fn fixed_hex(&self) -> &'static str {
        match self {
            Self::BurntUmberDust => "#3E2718",
            Self::ElysiumDesertDust => "#9F7E54",
            Self::SootBlack => "#111111",
            Self::FreshRustPowder => "#8F3810",
            Self::IndustrialAsh => "#575450",
        }
    }

    pub fn application_tip_i18n(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::BurntUmberDust, Locale::Fr) => "Déposer à sec dans les chenilles, puis capillariser une goutte d'essence sans odeur pour fixer sans écraser le volume.",
            (Self::BurntUmberDust, Locale::En) => "Deposit dry onto tracks, then touch with a drop of odorless spirit to wick and fix without flattening texture.",
            (Self::ElysiumDesertDust, Locale::Fr) => "Brosser à sec sur le bas des capes et des bottes. Très volatile : ne pas vernir à l'aérographe de trop près pour ne pas souffler la poudre.",
            (Self::ElysiumDesertDust, Locale::En) => "Dust dry onto bottom of cloaks and boots. Highly volatile: do not airbrush varnish too close to avoid blowing powder away.",
            (Self::SootBlack, Locale::Fr) => "Appliquer pur au pinceau sec ébouriffé sur les sorties de réacteurs, silencieux et canons. Ne pas fixer pour conserver le noir velouté ultra-mat.",
            (Self::SootBlack, Locale::En) => "Apply pure with a frayed drybrush onto engine exhausts, muzzles, and cannons. Leave unfixed for ultra-matte velvety soot.",
            (Self::FreshRustPowder, Locale::Fr) => "Saupoudrer sur un jus d'huile encore frais pour créer une texture granuleuse en relief d'oxydation réelle.",
            (Self::FreshRustPowder, Locale::En) => "Stipple onto still-wet oil wash to instantly create real textured corrosion relief.",
            (Self::IndustrialAsh, Locale::Fr) => "Mélanger avec du plâtre ou du pigment sombre pour soclage urbain et décombres d'usines.",
            (Self::IndustrialAsh, Locale::En) => "Combine with fine plaster or dark pigments for urban warfare basing and rubble.",
        }
    }
}

/// Printable / exportable army or squad painting recipe card.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArmyRecipeSheet {
    pub army_name: String,
    pub unit_name: String,
    pub faction: String,
    pub primer: String,
    pub base_acrylic: String,
    pub base_hex: String,
    pub oil_mix_formula: String,
    pub mixed_hex: String,
    pub wash_technique: String,
    pub weathering: String,
    pub varnish: String,
    pub notes: String,
}

impl ArmyRecipeSheet {
    pub fn new_standard_marine() -> Self {
        Self {
            army_name: "Blood Angels 2nd Company".to_string(),
            unit_name: "Tactical Squad Alpha".to_string(),
            faction: "Space Marines (Warhammer 40,000)".to_string(),
            primer: "Sous-couche Zénithale (Noir mat + Blanc pur à 45°)".to_string(),
            base_acrylic: "Mephiston Red (Citadel)".to_string(),
            base_hex: "#9A1115".to_string(),
            oil_mix_formula: "2 parts Rouge Cadmium Moyen + 1 part Laque d'Alizarine".to_string(),
            mixed_hex: "#A21418".to_string(),
            wash_technique: "Pin wash capillaire Terre d'Ombre Brûlée (1:8 dans Sansodor)".to_string(),
            weathering: "Streaking Grime sur les jambières + Éraflures au pinceau 00".to_string(),
            varnish: "AK Ultra Matte (2 voiles fins)".to_string(),
            notes: "Bien attendre 48h de séchage avant le vernis ultra-mat. Garder la palette fermée au congélateur entre deux sessions.".to_string(),
        }
    }

    pub fn to_markdown(&self) -> String {
        format!(
            "# Fiche de Recette Figurine : {} - {}\n\n\
             **Faction / Système** : {}\n\n\
             ---\n\n\
             ### 1. Préparation & Sous-couche\n\
             - **Sous-couche (Primer)** : {}\n\n\
             ### 2. Formule de Couleur de Base\n\
             - **Référence Acrylique** : {} (`{}`)\n\
             - **Mélange Huile Calculé** : {} (`{}`)\n\n\
             ### 3. Ombres, Lavis & Contrastes\n\
             - **Lavis & Pin Wash** : {}\n\n\
             ### 4. Weathering & Effets Spéciaux\n\
             - **Altération & Usure** : {}\n\n\
             ### 5. Finition & Protection\n\
             - **Vernis** : {}\n\n\
             ### Notes d'Atelier\n\
             {}\n",
            self.army_name, self.unit_name, self.faction,
            self.primer,
            self.base_acrylic, self.base_hex,
            self.oil_mix_formula, self.mixed_hex,
            self.wash_technique,
            self.weathering,
            self.varnish,
            self.notes,
        )
    }

    pub fn to_printable_html(&self) -> String {
        format!(
            r#"<!DOCTYPE html>
<html lang="fr">
<head>
<meta charset="utf-8">
<title>{army} - {unit}</title>
<style>
  body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; color: #1e293b; background: #ffffff; padding: 32px; max-width: 800px; margin: 0 auto; line-height: 1.5; }}
  h1 {{ font-size: 24px; border-bottom: 2px solid #e2e8f0; padding-bottom: 8px; margin-bottom: 6px; }}
  .subtitle {{ color: #64748b; font-size: 14px; margin-bottom: 24px; }}
  .card {{ border: 1px solid #cbd5e1; border-radius: 8px; padding: 16px; margin-bottom: 16px; page-break-inside: avoid; }}
  .card-title {{ font-size: 14px; font-weight: 700; color: #0f172a; text-transform: uppercase; letter-spacing: 0.5px; margin-bottom: 8px; }}
  .swatch {{ display: inline-block; width: 18px; height: 18px; border-radius: 4px; vertical-align: middle; margin-right: 6px; border: 1px solid rgba(0,0,0,0.2); }}
  .pill {{ display: inline-block; padding: 2px 8px; border-radius: 999px; font-size: 11px; font-weight: 600; background: #f1f5f9; color: #475569; }}
  @media print {{ body {{ padding: 0; }} }}
</style>
</head>
<body>
  <h1>{army}</h1>
  <div class="subtitle">{unit} · <strong>{faction}</strong></div>

  <div class="card">
    <div class="card-title">1. Sous-couche & Préparation</div>
    <div>{primer}</div>
  </div>

  <div class="card">
    <div class="card-title">2. Couleur de Base & Formule d'Huile</div>
    <div style="margin-bottom: 8px;">
      <span class="swatch" style="background-color: {base_hex};"></span>
      <strong>Acrylique de référence :</strong> {base_acrylic} (<code>{base_hex}</code>)
    </div>
    <div>
      <span class="swatch" style="background-color: {mixed_hex};"></span>
      <strong>Mélange d'huiles :</strong> {oil_mix_formula} (<code>{mixed_hex}</code>)
    </div>
  </div>

  <div class="card">
    <div class="card-title">3. Lavis & Travail Réductif</div>
    <div>{wash_technique}</div>
  </div>

  <div class="card">
    <div class="card-title">4. Weathering & Altération</div>
    <div>{weathering}</div>
  </div>

  <div class="card">
    <div class="card-title">5. Finition & Protection</div>
    <div><span class="pill">Vernis</span> {varnish}</div>
  </div>

  <div class="card">
    <div class="card-title">Notes d'Atelier</div>
    <div style="font-style: italic; color: #475569;">{notes}</div>
  </div>
</body>
</html>"#,
            army = self.army_name,
            unit = self.unit_name,
            faction = self.faction,
            primer = self.primer,
            base_acrylic = self.base_acrylic,
            base_hex = self.base_hex,
            oil_mix_formula = self.oil_mix_formula,
            mixed_hex = self.mixed_hex,
            wash_technique = self.wash_technique,
            weathering = self.weathering,
            varnish = self.varnish,
            notes = self.notes,
        )
    }
}

/// A calculated color harmony item adapted for miniature oil painting.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OilHarmonyItem {
    pub name_fr: &'static str,
    pub name_en: &'static str,
    pub hex: String,
    pub role_fr: &'static str,
    pub role_en: &'static str,
    pub oil_tip_fr: &'static str,
    pub oil_tip_en: &'static str,
    pub is_translucent_glaze: bool,
}

impl OilHarmonyItem {
    pub fn name_i18n(&self, locale: Locale) -> &'static str {
        match locale {
            Locale::Fr => self.name_fr,
            Locale::En => self.name_en,
        }
    }

    pub fn role_i18n(&self, locale: Locale) -> &'static str {
        match locale {
            Locale::Fr => self.role_fr,
            Locale::En => self.role_en,
        }
    }

    pub fn oil_tip_i18n(&self, locale: Locale) -> &'static str {
        match locale {
            Locale::Fr => self.oil_tip_fr,
            Locale::En => self.oil_tip_en,
        }
    }
}

/// Color harmony calculator specifically tuned for oil paint transparency and miniature contrast.
pub struct OilHarmoniesCalculator;

impl OilHarmoniesCalculator {
    pub fn calculate(base_rgb: Rgb) -> Vec<OilHarmonyItem> {
        let (h, s, v) = rgb_to_hsv(base_rgb);

        // 1. Complementary (180°) - Focal impact (gems, lenses, eyes, or opposite cloak)
        let h_comp = (h + 180.0) % 360.0;
        let comp_rgb = hsv_to_rgb(h_comp, s.min(0.95), v.max(0.65));

        // 2. Shadow Extinction / Neutralizer (desaturate with complementary instead of deadening black!)
        let ext_rgb = hsv_to_rgb(h_comp, (s * 0.6).min(0.7), (v * 0.4).max(0.12));

        // 3. Split Complementary (+150°)
        let h_split1 = (h + 150.0) % 360.0;
        let split1_rgb = hsv_to_rgb(h_split1, s, v);

        // 4. Split Complementary (+210°)
        let h_split2 = (h + 210.0) % 360.0;
        let split2_rgb = hsv_to_rgb(h_split2, s, v);

        // 5. Triad 1 (+120°) - Secondary armor / cape
        let h_triad1 = (h + 120.0) % 360.0;
        let triad1_rgb = hsv_to_rgb(h_triad1, s, v);

        // 6. Triad 2 (+240°) - Tertiary ornaments / basing
        let h_triad2 = (h + 240.0) % 360.0;
        let triad2_rgb = hsv_to_rgb(h_triad2, s, v);

        // 7. Warm Analogous (+30°) - Wet-on-wet blend
        let h_ana_warm = (h + 30.0) % 360.0;
        let ana_warm_rgb = hsv_to_rgb(h_ana_warm, s, v);

        // 8. Cool Analogous (-30°) - Halftone shift
        let h_ana_cool = (h + 330.0) % 360.0;
        let ana_cool_rgb = hsv_to_rgb(h_ana_cool, s, v);

        vec![
            OilHarmonyItem {
                name_fr: "Complémentaire Pure (180°)",
                name_en: "Pure Complementary (180°)",
                hex: comp_rgb.to_hex(),
                role_fr: "Points focaux : lentilles, gemmes, sceaux de pureté ou cape opposée.",
                role_en: "Focal accents: lenses, gems, purity seals, or contrasting cloaks.",
                oil_tip_fr: "Contraste chromatique maximal. À poser en rehaut opaque saturé pour attirer immédiatement le regard.",
                oil_tip_en: "Maximum chromatic contrast. Lay down as an opaque saturated highlight to draw the eye.",
                is_translucent_glaze: false,
            },
            OilHarmonyItem {
                name_fr: "Ombre d'Extinction (Désaturation sans Noir)",
                name_en: "Extinction Shadow (No Black)",
                hex: ext_rgb.to_hex(),
                role_fr: "Creux profonds et ombres propres sous l'armure.",
                role_en: "Deep recesses and core shadows beneath armor plates.",
                oil_tip_fr: "Règle d'or de l'huile : éteindre la couleur avec sa complémentaire assombrie plutôt qu'avec du noir du tube (qui salit et verdit).",
                oil_tip_en: "Golden rule of oils: neutralize vivid colors with their darkened complementary rather than deadening tube black.",
                is_translucent_glaze: true,
            },
            OilHarmonyItem {
                name_fr: "Complémentaire Adjacente 1 (+150°)",
                name_en: "Split-Complementary 1 (+150°)",
                hex: split1_rgb.to_hex(),
                role_fr: "Éléments secondaires et marquages d'escouade.",
                role_en: "Secondary elements and squad heraldry.",
                oil_tip_fr: "Contraste dynamique plus riche et moins agressif que la complémentaire directe.",
                oil_tip_en: "Dynamic contrast, richer and softer than direct complementary.",
                is_translucent_glaze: false,
            },
            OilHarmonyItem {
                name_fr: "Complémentaire Adjacente 2 (+210°)",
                name_en: "Split-Complementary 2 (+210°)",
                hex: split2_rgb.to_hex(),
                role_fr: "Équipements secondaires, étuis en cuir ou câblages.",
                role_en: "Secondary gear, leather holsters, or cables.",
                oil_tip_fr: "Permet de balancer le schéma sans saturer excessivement la silhouette.",
                oil_tip_en: "Balances the scheme without overwhelming the silhouette.",
                is_translucent_glaze: false,
            },
            OilHarmonyItem {
                name_fr: "Triade Majeure (+120°)",
                name_en: "Major Triad (+120°)",
                hex: triad1_rgb.to_hex(),
                role_fr: "Couleur d'accentuation majeure (Cape, tabard, bannière).",
                role_en: "Major accent color (Cloak, tabard, banner).",
                oil_tip_fr: "Standard d'or du style héroïque. À travailler en fondu crémeux OPR.",
                oil_tip_en: "Golden standard of comic / heroic style. Blend smoothly via cream OPR.",
                is_translucent_glaze: false,
            },
            OilHarmonyItem {
                name_fr: "Triade Mineure (+240°)",
                name_en: "Minor Triad (+240°)",
                hex: triad2_rgb.to_hex(),
                role_fr: "Couleur de soclage, armes de poing ou insignes.",
                role_en: "Basing tone, sidearms, or chapter badges.",
                oil_tip_fr: "Ferme le triangle d'équilibre chromatique de la figurine.",
                oil_tip_en: "Closes the chromatic triad balance across the miniature.",
                is_translucent_glaze: false,
            },
            OilHarmonyItem {
                name_fr: "Analogue Chaude (+30°)",
                name_en: "Warm Analogous (+30°)",
                hex: ana_warm_rgb.to_hex(),
                role_fr: "Demi-teinte chaude pour transition vers la lumière.",
                role_en: "Warm halftone for highlight transition.",
                oil_tip_fr: "S'étire parfaitement frais sur frais avec la couleur de base pour une transition imperceptible.",
                oil_tip_en: "Blends seamlessly wet-on-wet into the basecoat for invisible transitions.",
                is_translucent_glaze: false,
            },
            OilHarmonyItem {
                name_fr: "Analogue Froide (-30°)",
                name_en: "Cool Analogous (-30°)",
                hex: ana_cool_rgb.to_hex(),
                role_fr: "Demi-teinte froide pour transition vers l'ombre.",
                role_en: "Cool halftone for shadow transition.",
                oil_tip_fr: "Apporte une vibration colorée dans les dégradés sans casser la valeur.",
                oil_tip_en: "Adds subtle color vibration within blends without disrupting value.",
                is_translucent_glaze: true,
            },
        ]
    }
}

/// Consistency techniques for miniature oil painting (viscosity, paste, drops of solvent and medium).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OilConsistencyTechnique {
    PurePaste,
    CreamOpr,
    TranslucentGlaze,
    CapillaryPinWash,
}

impl OilConsistencyTechnique {
    pub const ALL: [Self; 4] = [
        Self::PurePaste,
        Self::CreamOpr,
        Self::TranslucentGlaze,
        Self::CapillaryPinWash,
    ];

    pub fn label_i18n(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::PurePaste, Locale::Fr) => "Pâte Pure du Tube (Empâtement)",
            (Self::PurePaste, Locale::En) => "Pure Tube Paste (Impasto)",
            (Self::CreamOpr, Locale::Fr) => "Crème Onctueuse / OPR (Fondus)",
            (Self::CreamOpr, Locale::En) => "Creamy OPR (Wet Blending)",
            (Self::TranslucentGlaze, Locale::Fr) => "Glacis Translucide (Filtre)",
            (Self::TranslucentGlaze, Locale::En) => "Translucent Glaze (Filter)",
            (Self::CapillaryPinWash, Locale::Fr) => "Pin Wash Capillaire (Creux)",
            (Self::CapillaryPinWash, Locale::En) => "Capillary Pin Wash (Recesses)",
        }
    }

    pub fn solvent_drops_per_part(&self) -> u32 {
        match self {
            Self::PurePaste => 0,
            Self::CreamOpr => 1,
            Self::TranslucentGlaze => 5,
            Self::CapillaryPinWash => 12,
        }
    }

    pub fn medium_drops_per_part(&self) -> u32 {
        match self {
            Self::PurePaste => 0,
            Self::CreamOpr => 1,
            Self::TranslucentGlaze => 1,
            Self::CapillaryPinWash => 0,
        }
    }

    pub fn description_i18n(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::PurePaste, Locale::Fr) => "Viscosité 100%. Idéal pour les arêtes vives, pointes de lumière spéculaire et textures en relief au cure-dent.",
            (Self::PurePaste, Locale::En) => "100% viscosity. Ideal for sharp edge highlights, specular catchlights, and thick textured relief.",
            (Self::CreamOpr, Locale::Fr) => "Viscosité mayonnaise. Permet de poser des dégradés frais sur frais ultra-doux (Oil Paint Rendering) sans traces de pinceau.",
            (Self::CreamOpr, Locale::En) => "Mayonnaise viscosity. Allows ultra-smooth wet-on-wet blending (Oil Paint Rendering) with zero brush marks.",
            (Self::TranslucentGlaze, Locale::Fr) => "Viscosité lait. Teinte les surfaces en transparence tout en laissant transparaître les volumes et ombres de la sous-couche.",
            (Self::TranslucentGlaze, Locale::En) => "Milk viscosity. Tints surfaces with rich translucency while preserving underlying zenithal shading.",
            (Self::CapillaryPinWash, Locale::Fr) => "Viscosité eau / thé fort. Tension superficielle quasi nulle grâce à l'essence minérale : glisse instantanément dans les lignes de structure.",
            (Self::CapillaryPinWash, Locale::En) => "Water viscosity. Zero surface tension thanks to odorless spirits: wicks instantly into panel lines.",
        }
    }
}

/// Optical simulator of an oil glaze layer laid over a zenithal grayscale undercoat.
pub struct OilGlazeZenithalSimulator;

impl OilGlazeZenithalSimulator {
    /// Steps of zenithal grayscale values on the miniature undercoat.
    pub const ZENITHAL_LEVELS: [(&'static str, &'static str, f64); 5] = [
        ("1. Occlusion Profonde", "1. Deep Occlusion", 0.08),
        ("2. Ombre Portée", "2. Cast Shadow", 0.25),
        ("3. Demi-teinte Médiane", "3. Midtone Gray", 0.50),
        ("4. Lumière Zénithale 45°", "4. Zenithal 45° Highlight", 0.80),
        ("5. Arête Blanche Pure", "5. Pure White Catchlight", 1.00),
    ];

    /// Computes resulting simulated color of an oil glaze over a zenithal grayscale value.
    pub fn simulate_glaze(oil_rgb: Rgb, zenithal_luminance: f64, glaze_strength: f64) -> Rgb {
        // Oil glaze behaves as a subtractive optical filter over physical undercoat luminance:
        // L_out = L_zenith * (L_zenith * (1 - strength) + oil_component * strength)
        let s = glaze_strength.clamp(0.1, 0.95);
        let l = zenithal_luminance.clamp(0.0, 1.0);

        let r = l * (l * (1.0 - s) + oil_rgb.r() * s);
        let g = l * (l * (1.0 - s) + oil_rgb.g() * s);
        let b = l * (l * (1.0 - s) + oil_rgb.b() * s);

        Rgb::new(r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0))
    }
}


// =========================================================================
// 1. Palette Layout Guide (Physical Palette Setup)
// =========================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaletteLayoutType {
    NormaBlueStudio,
    ZornClassic,
    SplitPrimary,
    GlassSlabCustom,
}

impl PaletteLayoutType {
    pub const ALL: [PaletteLayoutType; 4] = [
        PaletteLayoutType::NormaBlueStudio,
        PaletteLayoutType::ZornClassic,
        PaletteLayoutType::SplitPrimary,
        PaletteLayoutType::GlassSlabCustom,
    ];

    pub fn title_i18n(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::NormaBlueStudio, Locale::Fr) => "Palette Atelier Norma Blue (11 tubes)",
            (Self::NormaBlueStudio, Locale::En) => "Norma Blue Studio Palette (11 tubes)",
            (Self::ZornClassic, Locale::Fr) => "Palette Maître de Zorn (4 tubes)",
            (Self::ZornClassic, Locale::En) => "Zorn Master Palette (4 tubes)",
            (Self::SplitPrimary, Locale::Fr) => "Palette Primaires Doubles (Chaud / Froid)",
            (Self::SplitPrimary, Locale::En) => "Split-Primary Palette (Warm / Cool)",
            (Self::GlassSlabCustom, Locale::Fr) => "Agencement Libre sur Dalle de Verre",
            (Self::GlassSlabCustom, Locale::En) => "Free Glass Slab Perimeter Layout",
        }
    }

    pub fn surface_recommendation_i18n(&self, locale: Locale) -> &'static str {
        match locale {
            Locale::Fr => "Dalle de verre trempé neutre (fond blanc ou gris neutre 50%) ou palette étanche hermétique. Nettoyage facile au grattoir de vitrier à lame plate.",
            Locale::En => "Neutral tempered glass slab (white or 50% neutral gray backing) or airtight stay-wet box. Easily cleaned with a razor scraper.",
        }
    }

    pub fn setup_advice_i18n(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::NormaBlueStudio, Locale::Fr) => "Déposez les noisettes en arc de cercle sur le pourtour. Laissez au moins 6 cm entre le Bleu de Prusse (pigment tachant) et les Blancs pour éviter toute pollution accidentelle lors des mélanges.",
            (Self::NormaBlueStudio, Locale::En) => "Squeeze paint dots in an arc around the perimeter. Keep at least 6 cm separation between Prussian Blue (high tinting power) and Whites to prevent contamination.",
            (Self::ZornClassic, Locale::Fr) => "La palette suprême pour les peaux et bustes réalistes : du Blanc pur au Noir d'ivoire (qui joue le rôle de bleu froid), avec l'Ocre Jaune et le Rouge Cadmium.",
            (Self::ZornClassic, Locale::En) => "The quintessential skin & portrait palette: Titanium White to Ivory Black (acting as cold blue), with Yellow Ochre and Cadmium Red.",
            (Self::SplitPrimary, Locale::Fr) => "Chaque couleur primaire est doublée (une chaude, une froide) pour obtenir tous les verts, violets et oranges vibrants sans neutralisation indésirable.",
            (Self::SplitPrimary, Locale::En) => "Each primary is paired (one warm, one cool) to mix pure vibrant greens, purples, and oranges without accidental muddy desaturation.",
            (Self::GlassSlabCustom, Locale::Fr) => "Gardez toujours les 2/3 centraux de la plaque totalement dégagés pour le travail des jus, dégradés et tests de dilution.",
            (Self::GlassSlabCustom, Locale::En) => "Always keep the central 2/3 of the slab entirely clear for wash pooling, gradients, and consistency testing.",
        }
    }

    pub fn slots(&self, locale: Locale) -> Vec<PalettePlacementSlot> {
        match self {
            Self::NormaBlueStudio => vec![
                PalettePlacementSlot {
                    name: locale.tr("1. Blanc de Titane", "1. Titanium White").to_string(),
                    role: locale.tr("Base opaque & points de lumière", "Opaque base & highlights").to_string(),
                    hex: "#FFFFFF",
                    position_desc: locale.tr("Coin Haut Gauche", "Top Left Corner").to_string(),
                    safety_warning: Some(locale.tr("Garder pur à l'écart des pigments sombres", "Keep pure away from dark stainers").to_string()),
                },
                PalettePlacementSlot {
                    name: locale.tr("2. Jaune Citron", "2. Lemon Yellow").to_string(),
                    role: locale.tr("Primaire jaune froide", "Cool yellow primary").to_string(),
                    hex: "#FFF44F",
                    position_desc: locale.tr("Haut - 2", "Top - 2").to_string(),
                    safety_warning: None,
                },
                PalettePlacementSlot {
                    name: locale.tr("3. Ocre Jaune", "3. Yellow Ochre").to_string(),
                    role: locale.tr("Le 'ciment' unificateur de la figurine", "Universal unifier & natural base").to_string(),
                    hex: "#CC7722",
                    position_desc: locale.tr("Haut - 3", "Top - 3").to_string(),
                    safety_warning: None,
                },
                PalettePlacementSlot {
                    name: locale.tr("4. Rouge Cadmium Moyen", "4. Cadmium Red Medium").to_string(),
                    role: locale.tr("Primaire rouge chaude & opaque", "Warm opaque red primary").to_string(),
                    hex: "#E30022",
                    position_desc: locale.tr("Haut - 4", "Top - 4").to_string(),
                    safety_warning: None,
                },
                PalettePlacementSlot {
                    name: locale.tr("5. Laque de Garance", "5. Madder Lake").to_string(),
                    role: locale.tr("Rouge froid transparent (glacis & ombres de peau)", "Cool transparent red (glazes & flesh shadows)").to_string(),
                    hex: "#A50021",
                    position_desc: locale.tr("Haut Droite", "Top Right").to_string(),
                    safety_warning: None,
                },
                PalettePlacementSlot {
                    name: locale.tr("6. Bleu de Prusse", "6. Prussian Blue").to_string(),
                    role: locale.tr("Pigment ultra-puissant d'ombrage", "Ultra-staining deep shadow pigment").to_string(),
                    hex: "#003153",
                    position_desc: locale.tr("Bord Droit Isolé", "Isolated Right Edge").to_string(),
                    safety_warning: Some(locale.tr("⚠️ Prélever à la pointe du cure-dent ! Risque de contaminer la palette.", "⚠️ Pick with toothpick tip! High risk of palette contamination.").to_string()),
                },
                PalettePlacementSlot {
                    name: locale.tr("7. Bleu Céruléen / Cyan", "7. Cerulean / Cyan Blue").to_string(),
                    role: locale.tr("Bleu froid & ciels / énergies", "Cool blue & plasma / skies").to_string(),
                    hex: "#007BA7",
                    position_desc: locale.tr("Bord Droit - 2", "Right Edge - 2").to_string(),
                    safety_warning: None,
                },
                PalettePlacementSlot {
                    name: locale.tr("8. Vert Émeraude (Viridian)", "8. Viridian Green").to_string(),
                    role: locale.tr("Glacis transparent & demi-teintes de barbe", "Transparent glaze & jaw halftones").to_string(),
                    hex: "#40826D",
                    position_desc: locale.tr("Bas Droit", "Bottom Right").to_string(),
                    safety_warning: None,
                },
                PalettePlacementSlot {
                    name: locale.tr("9. Terre de Sienne Brûlée", "9. Burnt Sienna").to_string(),
                    role: locale.tr("Terre chaude, cuirs & base d'ombrage", "Warm earth, leathers & shading base").to_string(),
                    hex: "#E97451",
                    position_desc: locale.tr("Bas Centre", "Bottom Center").to_string(),
                    safety_warning: None,
                },
                PalettePlacementSlot {
                    name: locale.tr("10. Terre d'Ombre Brûlée", "10. Burnt Umber").to_string(),
                    role: locale.tr("Ombres profondes & désaturation naturelle", "Deep shadows & natural desaturation").to_string(),
                    hex: "#8A3324",
                    position_desc: locale.tr("Bas Gauche", "Bottom Left").to_string(),
                    safety_warning: None,
                },
                PalettePlacementSlot {
                    name: locale.tr("11. Noir d'Ivoire", "11. Ivory Black").to_string(),
                    role: locale.tr("Socles & olives (ne pas utiliser pour assombrir les couleurs)", "Bases & olives only (do not use for pure shading)").to_string(),
                    hex: "#232023",
                    position_desc: locale.tr("Coin Bas Gauche Isolé", "Isolated Bottom Left").to_string(),
                    safety_warning: Some(locale.tr("Ne jamais mélanger avec les rouges pour faire des ombres", "Never mix with reds for shadows").to_string()),
                },
            ],
            Self::ZornClassic => vec![
                PalettePlacementSlot {
                    name: locale.tr("1. Blanc de Titane", "1. Titanium White").to_string(),
                    role: locale.tr("Lumière spéculaire & chair haute", "Specular highlight & light skin").to_string(),
                    hex: "#FFFFFF",
                    position_desc: locale.tr("Coin Haut Gauche", "Top Left").to_string(),
                    safety_warning: None,
                },
                PalettePlacementSlot {
                    name: locale.tr("2. Ocre Jaune", "2. Yellow Ochre").to_string(),
                    role: locale.tr("Teinte de base peau & tons dorés", "Base skin tone & golden warms").to_string(),
                    hex: "#CC7722",
                    position_desc: locale.tr("Haut Centre", "Top Center").to_string(),
                    safety_warning: None,
                },
                PalettePlacementSlot {
                    name: locale.tr("3. Rouge Cadmium Moyen", "3. Cadmium Red Medium").to_string(),
                    role: locale.tr("Rougeurs, lèvres, pommettes & sang", "Blush, lips, cheeks & warmth").to_string(),
                    hex: "#E30022",
                    position_desc: locale.tr("Haut Droite", "Top Right").to_string(),
                    safety_warning: None,
                },
                PalettePlacementSlot {
                    name: locale.tr("4. Noir d'Ivoire", "4. Ivory Black").to_string(),
                    role: locale.tr("Agit comme un bleu froid + ombre profonde", "Acts as cool blue substitute + deep shadow").to_string(),
                    hex: "#232023",
                    position_desc: locale.tr("Bas Droite", "Bottom Right").to_string(),
                    safety_warning: Some(locale.tr("Mélangé au Blanc, il donne un bleu ardoise froid magnifique pour les demi-teintes.", "Mixed with White, it creates a gorgeous slate blue for cool skin halftones.").to_string()),
                },
            ],
            Self::SplitPrimary => vec![
                PalettePlacementSlot {
                    name: locale.tr("Blanc de Titane", "Titanium White").to_string(),
                    role: locale.tr("Clarté & opacité", "Lightness & opacity").to_string(),
                    hex: "#FFFFFF",
                    position_desc: locale.tr("Coin Haut Gauche", "Top Left").to_string(),
                    safety_warning: None,
                },
                PalettePlacementSlot {
                    name: locale.tr("Jaune Citron (Froid)", "Lemon Yellow (Cool)").to_string(),
                    role: locale.tr("Verts éclatants", "Vibrant greens").to_string(),
                    hex: "#FFF44F",
                    position_desc: locale.tr("Haut 1", "Top 1").to_string(),
                    safety_warning: None,
                },
                PalettePlacementSlot {
                    name: locale.tr("Jaune Cadmium (Chaud)", "Cadmium Yellow Deep (Warm)").to_string(),
                    role: locale.tr("Oranges purs", "Pure oranges").to_string(),
                    hex: "#FFA600",
                    position_desc: locale.tr("Haut 2", "Top 2").to_string(),
                    safety_warning: None,
                },
                PalettePlacementSlot {
                    name: locale.tr("Rouge Cadmium (Chaud)", "Cadmium Red (Warm)").to_string(),
                    role: locale.tr("Rouges écarlates", "Scarlet reds").to_string(),
                    hex: "#E30022",
                    position_desc: locale.tr("Haut 3", "Top 3").to_string(),
                    safety_warning: None,
                },
                PalettePlacementSlot {
                    name: locale.tr("Laque de Garance / Magenta (Froid)", "Madder Lake / Magenta (Cool)").to_string(),
                    role: locale.tr("Violets profonds", "Deep violets").to_string(),
                    hex: "#A50021",
                    position_desc: locale.tr("Haut Droite", "Top Right").to_string(),
                    safety_warning: None,
                },
                PalettePlacementSlot {
                    name: locale.tr("Bleu Outremer (Chaud/Rougeâtre)", "Ultramarine Blue (Warm)").to_string(),
                    role: locale.tr("Violets & ombres chaudes", "Violets & warm darks").to_string(),
                    hex: "#120A8F",
                    position_desc: locale.tr("Bord Droit", "Right Edge").to_string(),
                    safety_warning: None,
                },
                PalettePlacementSlot {
                    name: locale.tr("Bleu Phtalo / Prusse (Froid/Verdâtre)", "Phthalo / Prussian Blue (Cool)").to_string(),
                    role: locale.tr("Verts émeraude intenses", "Intense emerald greens").to_string(),
                    hex: "#003153",
                    position_desc: locale.tr("Bas Droit", "Bottom Right").to_string(),
                    safety_warning: Some(locale.tr("⚠️ Fort pouvoir colorant", "⚠️ High staining power").to_string()),
                },
                PalettePlacementSlot {
                    name: locale.tr("Terre d'Ombre Brûlée (Neutre)", "Burnt Umber (Neutral)").to_string(),
                    role: locale.tr("Désaturateur universel", "Universal desaturator").to_string(),
                    hex: "#8A3324",
                    position_desc: locale.tr("Bas Gauche", "Bottom Left").to_string(),
                    safety_warning: None,
                },
            ],
            Self::GlassSlabCustom => vec![
                PalettePlacementSlot {
                    name: locale.tr("Bandeau Supérieur : Teintes Claires", "Top Row: High Value Colors").to_string(),
                    role: locale.tr("Blancs, Jaunes, Chairs, Rehauts", "Whites, Yellows, Flesh, Highlights").to_string(),
                    hex: "#F5F5DC",
                    position_desc: locale.tr("Bord Supérieur (0-100%)", "Top Edge (0-100%)").to_string(),
                    safety_warning: None,
                },
                PalettePlacementSlot {
                    name: locale.tr("Bandeau Droit : Teintes Chaudes", "Right Row: Warm Tones").to_string(),
                    role: locale.tr("Rouges, Oranges, Terres de Sienne", "Reds, Oranges, Siennas").to_string(),
                    hex: "#C8102E",
                    position_desc: locale.tr("Bord Droit", "Right Edge").to_string(),
                    safety_warning: None,
                },
                PalettePlacementSlot {
                    name: locale.tr("Bandeau Inférieur : Teintes Sombres & Ombres", "Bottom Row: Darks & Shadows").to_string(),
                    role: locale.tr("Ombres brûlées, Noirs, Bleus de Prusse", "Burnt Umbers, Blacks, Prussian Blues").to_string(),
                    hex: "#2F1E19",
                    position_desc: locale.tr("Bord Inférieur", "Bottom Edge").to_string(),
                    safety_warning: Some(locale.tr("Maintenir à l'opposé des blancs", "Keep opposite of whites").to_string()),
                },
                PalettePlacementSlot {
                    name: locale.tr("Bandeau Gauche : Teintes Froides & Verts", "Left Row: Cool Tones & Greens").to_string(),
                    role: locale.tr("Bleus outremer, Viridians, Glacis", "Ultramarines, Viridians, Glazes").to_string(),
                    hex: "#0055A5",
                    position_desc: locale.tr("Bord Gauche", "Left Edge").to_string(),
                    safety_warning: None,
                },
                PalettePlacementSlot {
                    name: locale.tr("Zone Centrale : Arène de Mélange", "Center Zone: Mixing Arena").to_string(),
                    role: locale.tr("Espace stérile 20x15cm pour étaler les dégradés et tester la dilution", "Clean 20x15cm space for gradient pulling & wash tests").to_string(),
                    hex: "#A0A0A0",
                    position_desc: locale.tr("Plein Centre", "Dead Center").to_string(),
                    safety_warning: None,
                },
            ],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PalettePlacementSlot {
    pub name: String,
    pub role: String,
    pub hex: &'static str,
    pub position_desc: String,
    pub safety_warning: Option<String>,
}

// =========================================================================
// 2. Vehicle & AFV Weathering Filters (Abteilung 502 / Ammo Mig Equivalents)
// =========================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VehicleEffectKind {
    StarshipFilth,
    EngineGrime,
    DarkRust,
    IndustrialEarth,
    TrackWash,
    Rainmarks,
    FreshEngineOil,
}

impl VehicleEffectKind {
    pub const ALL: [VehicleEffectKind; 7] = [
        VehicleEffectKind::StarshipFilth,
        VehicleEffectKind::EngineGrime,
        VehicleEffectKind::DarkRust,
        VehicleEffectKind::IndustrialEarth,
        VehicleEffectKind::TrackWash,
        VehicleEffectKind::Rainmarks,
        VehicleEffectKind::FreshEngineOil,
    ];

    pub fn commercial_name(&self) -> &'static str {
        match self {
            Self::StarshipFilth => "Starship Filth (Abteilung 502 ABT160)",
            Self::EngineGrime => "Engine Grime (AK-026 / ABT100)",
            Self::DarkRust => "Dark Rust (Abteilung 502 ABT070)",
            Self::IndustrialEarth => "Industrial Earth (Ammo Mig A.MIG-1407)",
            Self::TrackWash => "Track Wash (AK-083 / ABT094)",
            Self::Rainmarks => "Rainmarks / Streaking Dust (AK-012)",
            Self::FreshEngineOil => "Fresh Engine Oil (Ammo Mig A.MIG-1408)",
        }
    }

    pub fn target_hex(&self) -> &'static str {
        match self {
            Self::StarshipFilth => "#3F3A36",
            Self::EngineGrime => "#2D261E",
            Self::DarkRust => "#652D19",
            Self::IndustrialEarth => "#886B52",
            Self::TrackWash => "#241D1A",
            Self::Rainmarks => "#D6CCA9",
            Self::FreshEngineOil => "#120F0C",
        }
    }

    pub fn category_i18n(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::StarshipFilth, Locale::Fr) => "Salissure Panneaux Blindés & Sci-Fi",
            (Self::StarshipFilth, Locale::En) => "Armor & Sci-Fi Panel Grime",
            (Self::EngineGrime, Locale::Fr) => "Moteur, Échappements & Graisse",
            (Self::EngineGrime, Locale::En) => "Engine, Exhausts & Grease",
            (Self::DarkRust, Locale::Fr) => "Rouille Sombre & Oxydation Métal",
            (Self::DarkRust, Locale::En) => "Dark Rust & Metal Oxidation",
            (Self::IndustrialEarth, Locale::Fr) => "Poussière de Terre Industrielle",
            (Self::IndustrialEarth, Locale::En) => "Industrial Earth & Dust",
            (Self::TrackWash, Locale::Fr) => "Chenilles & Barbotins de Chars",
            (Self::TrackWash, Locale::En) => "Tank Tracks & Sprockets Wash",
            (Self::Rainmarks, Locale::Fr) => "Coulures de Pluie & Poussière Verticale",
            (Self::Rainmarks, Locale::En) => "Rainmarks & Vertical Dust Streaks",
            (Self::FreshEngineOil, Locale::Fr) => "Fuite d'Huile de Moteur Fraîche",
            (Self::FreshEngineOil, Locale::En) => "Fresh Wet Engine Oil Leak",
        }
    }

    pub fn artist_oil_formula_i18n(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::StarshipFilth, Locale::Fr) => "45% Terre d'Ombre Brûlée + 45% Noir d'Ivoire + 10% Blanc de Titane",
            (Self::StarshipFilth, Locale::En) => "45% Burnt Umber + 45% Ivory Black + 10% Titanium White",
            (Self::EngineGrime, Locale::Fr) => "60% Terre d'Ombre Brûlée + 30% Vert Olive (ou Viridian + pointe d'Ocre) + 10% Noir",
            (Self::EngineGrime, Locale::En) => "60% Burnt Umber + 30% Olive Green (or Viridian + touch of Ochre) + 10% Black",
            (Self::DarkRust, Locale::Fr) => "70% Terre de Sienne Brûlée + 20% Rouge de Cadmium + 10% Terre d'Ombre Brûlée",
            (Self::DarkRust, Locale::En) => "70% Burnt Sienna + 20% Cadmium Red + 10% Burnt Umber",
            (Self::IndustrialEarth, Locale::Fr) => "60% Ocre Jaune + 30% Terre d'Ombre Brûlée + 10% Blanc de Titane",
            (Self::IndustrialEarth, Locale::En) => "60% Yellow Ochre + 30% Burnt Umber + 10% Titanium White",
            (Self::TrackWash, Locale::Fr) => "75% Terre d'Ombre Brûlée + 20% Noir d'Ivoire + 5% Bleu Outremer",
            (Self::TrackWash, Locale::En) => "75% Burnt Umber + 20% Ivory Black + 5% Ultramarine Blue",
            (Self::Rainmarks, Locale::Fr) => "70% Blanc de Titane + 25% Ocre Jaune + 5% Terre d'Ombre Naturelle",
            (Self::Rainmarks, Locale::En) => "70% Titanium White + 25% Yellow Ochre + 5% Raw Umber",
            (Self::FreshEngineOil, Locale::Fr) => "80% Noir d'Ivoire + 15% Terre d'Ombre Brûlée + 5% Laque de Garance (brillant)",
            (Self::FreshEngineOil, Locale::En) => "80% Ivory Black + 15% Burnt Umber + 5% Madder Lake (gloss finish)",
        }
    }

    pub fn solvent_and_dilution_i18n(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::FreshEngineOil, Locale::Fr) => "1 part de pâte pure + 2 gouttes de Vernis Brillant / Liquin. Ne PAS diluer au solvant inodore pour conserver l'effet liquide sirupeux.",
            (Self::FreshEngineOil, Locale::En) => "1 part pure paste + 2 drops Gloss Varnish / Liquin. Do NOT thin with solvent to maintain wet syrupy gloss.",
            (Self::Rainmarks, Locale::Fr) => "1 part de pâte pour 12 gouttes d'essence inodore (Sansodor/Gamsol). Déposer en micro-points puis étirer verticalement au pinceau plat propre imbibé de solvant.",
            (Self::Rainmarks, Locale::En) => "1 part paste to 12 drops odorless mineral spirit. Dot sparingly then streak vertically with a clean flat brush damp with solvent.",
            (_, Locale::Fr) => "1 part de pâte pour 8 à 10 gouttes de Sansodor / Gamsol (solvant inodore doux pour protéger le vernis acrylique préalable).",
            (_, Locale::En) => "1 part paste to 8-10 drops Sansodor / Gamsol (gentle odorless spirit to preserve underlying acrylic clear coat).",
        }
    }

    pub fn finish_i18n(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::FreshEngineOil, Locale::Fr) => "✨ Brillant miroir humide (Gloss)",
            (Self::FreshEngineOil, Locale::En) => "✨ Gloss wet look",
            (Self::Rainmarks | Self::IndustrialEarth, Locale::Fr) => "🌫️ Ultra-Mat poussiéreux",
            (Self::Rainmarks | Self::IndustrialEarth, Locale::En) => "🌫️ Dead matte chalky",
            (_, Locale::Fr) => "🛡️ Satiné naturel",
            (_, Locale::En) => "🛡️ Natural satin",
        }
    }
}

// =========================================================================
// 3. Wet-on-Wet Blending Matrix (Traction, Tearing & Brush Advice)
// =========================================================================

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

    pub fn label_i18n(&self, locale: Locale) -> &'static str {
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
    pub fn evaluate(
        base_consistency: OilConsistencyTechnique,
        accent_consistency: OilConsistencyTechnique,
        locale: Locale,
    ) -> BlendingEvaluation {
        match (base_consistency, accent_consistency) {
            (OilConsistencyTechnique::CreamOpr, OilConsistencyTechnique::CreamOpr) => BlendingEvaluation {
                rating: BlendingRating::Excellent,
                traction_analysis: locale.tr(
                    "Les deux couches ont exactement la même viscosité beurrée. Le pinceau tire les pigments sans marquer de sillon.",
                    "Both layers have matching buttery viscosity. The brush glides seamlessly without furrowing."
                ).to_string(),
                ideal_window_minutes: locale.tr("Immédiat jusqu'à 2 heures après la pose", "Immediate up to 2 hours after laydown").to_string(),
                recommended_brush: locale.tr("Pinceau langue de chat Kolinsky #1 ou estompe mousse silicone", "Kolinsky filbert #1 or silicone colour shaper").to_string(),
                medium_tweak_advice: locale.tr("Aucun additif requis : travaillez directement dans le frais.", "No additives needed: blend directly into the wet layer.").to_string(),
            },
            (OilConsistencyTechnique::PurePaste, OilConsistencyTechnique::PurePaste) => BlendingEvaluation {
                rating: BlendingRating::Good,
                traction_analysis: locale.tr(
                    "Pâte dense contre pâte dense : fondu très couvrant mais demande un pinceau ferme pour adoucir la frontière.",
                    "Dense paste against dense paste: high coverage but requires a firmer brush to soften the seam."
                ).to_string(),
                ideal_window_minutes: locale.tr("Dans les 30 à 45 minutes", "Within 30 to 45 minutes").to_string(),
                recommended_brush: locale.tr("Pinceau plat biseauté en martre ou poils synthétiques fermes", "Flat angled sable or firm synthetic brush").to_string(),
                medium_tweak_advice: locale.tr("Effleurer le pinceau sur une micro-goutte de Sansodor pour lubrifier la traction.", "Touch brush tip to a micro-drop of odorless spirit to lubricate stroke.").to_string(),
            },
            (OilConsistencyTechnique::CreamOpr, OilConsistencyTechnique::PurePaste) => BlendingEvaluation {
                rating: BlendingRating::Excellent,
                traction_analysis: locale.tr(
                    "Règle d'or de l'OPR : poser la lumière en pâte plus dense sur la crème de base. L'accroche est immédiate et ne bave pas.",
                    "Golden rule of OPR: lay firmer highlight paste over softer base cream. Grip is immediate with zero bleeding."
                ).to_string(),
                ideal_window_minutes: locale.tr("Attendre 10 min après la base, puis tirer pendant 90 min", "Wait 10 min after base, then blend for 90 min").to_string(),
                recommended_brush: locale.tr("Pinceau rond pointu #00 pour poser, puis langue de chat propre pour fondre", "Pointed round #00 to dab, clean filbert to feather").to_string(),
                medium_tweak_advice: locale.tr("La pâte pure en rehaut doit rester non diluée.", "Keep highlight paste undiluted for maximum opacity.").to_string(),
            },
            (OilConsistencyTechnique::PurePaste, OilConsistencyTechnique::TranslucentGlaze)
            | (OilConsistencyTechnique::CreamOpr, OilConsistencyTechnique::TranslucentGlaze) => BlendingEvaluation {
                rating: BlendingRating::Caution,
                traction_analysis: locale.tr(
                    "⚠️ Risque de dilution inversée : le solvant du glacis peut dissoudre et creuser la couche de base fraîche.",
                    "⚠️ Washout hazard: glaze solvent may dissolve and pit the fresh base coat."
                ).to_string(),
                ideal_window_minutes: locale.tr("Déconseillé dans le frais : attendre le séchage au toucher de la base (24h) !", "Avoid wet-on-wet: wait for base to be touch-dry (24h)!").to_string(),
                recommended_brush: locale.tr("Pinceau lavis très doux sans frotter", "Ultra-soft mop brush with feather touch").to_string(),
                medium_tweak_advice: locale.tr("Si urgent, utiliser du Liquin pur sans solvant pour le glacis pour éviter d'attaquer la base.", "If urgent, use pure Liquin without spirits to glaze over without dissolving base.").to_string(),
            },
            (OilConsistencyTechnique::TranslucentGlaze, OilConsistencyTechnique::TranslucentGlaze) => BlendingEvaluation {
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

// =========================================================================
// 4. Batch Mother Mix Calculator (Volumes for Squads & Armies)
// =========================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MiniatureScale {
    Scale28mm,
    Scale32mm,
    Scale54mm,
    Scale75mm,
    Bust1_10,
}

impl MiniatureScale {
    pub const ALL: [MiniatureScale; 5] = [
        MiniatureScale::Scale28mm,
        MiniatureScale::Scale32mm,
        MiniatureScale::Scale54mm,
        MiniatureScale::Scale75mm,
        MiniatureScale::Bust1_10,
    ];

    pub fn label_i18n(&self, locale: Locale) -> &'static str {
        match (self, locale) {
            (Self::Scale28mm, Locale::Fr) => "28mm (Infanterie classique / Historique)",
            (Self::Scale28mm, Locale::En) => "28mm (Classic infantry / Historical)",
            (Self::Scale32mm, Locale::Fr) => "32mm Héroïque (Space Marines, Warhammer 40k)",
            (Self::Scale32mm, Locale::En) => "32mm Heroic (Space Marines, 40k)",
            (Self::Scale54mm, Locale::Fr) => "54mm (Grand format d'infanterie)",
            (Self::Scale54mm, Locale::En) => "54mm (Large infantry scale)",
            (Self::Scale75mm, Locale::Fr) => "75mm (Figurine d'exposition)",
            (Self::Scale75mm, Locale::En) => "75mm (Display figure)",
            (Self::Bust1_10, Locale::Fr) => "Buste 1/10 ou 1/12 (Grande surface)",
            (Self::Bust1_10, Locale::En) => "1/10 or 1/12 Bust (Large surface)",
        }
    }

    pub fn surface_multiplier(&self) -> f64 {
        match self {
            Self::Scale28mm => 1.0,
            Self::Scale32mm => 1.4,
            Self::Scale54mm => 3.2,
            Self::Scale75mm => 6.0,
            Self::Bust1_10 => 12.0,
        }
    }
}

pub struct BatchMotherMixRequirements {
    pub volume_ml: f64,
    pub matchstick_heads: u32,
    pub hazelnuts: f64,
    pub open_palette_hours: u32,
    pub airtight_pot_days: u32,
    pub freezer_days: u32,
    pub advice: String,
}

pub struct BatchMotherMixCalculator;

impl BatchMotherMixCalculator {
    pub fn calculate(
        scale: MiniatureScale,
        model_count: u32,
        is_full_coverage: bool,
        locale: Locale,
    ) -> BatchMotherMixRequirements {
        let count = model_count.clamp(1, 150) as f64;
        let base_ml_per_model = if is_full_coverage { 0.08 } else { 0.03 };
        let volume_ml = (base_ml_per_model * scale.surface_multiplier() * count).max(0.05);

        let matchstick_heads = (volume_ml / 0.02).ceil() as u32;
        let hazelnuts = (volume_ml / 0.3 * 10.0).round() / 10.0;

        let advice = match locale {
            Locale::Fr => format!(
                "Pour {} figurine(s) à l'échelle {} : préparez environ {} ml de mélange mère (~{} noisettes de pâte). Conservez dans un petit godet hermétique pour éviter l'oxydation entre vos sessions.",
                model_count, scale.label_i18n(locale), (volume_ml * 100.0).round() / 100.0, hazelnuts
            ),
            Locale::En => format!(
                "For {} model(s) at {} scale: prepare approx {} ml of mother mix (~{} hazelnut portions). Store in an airtight screw pot between painting sessions.",
                model_count, scale.label_i18n(locale), (volume_ml * 100.0).round() / 100.0, hazelnuts
            ),
        };

        BatchMotherMixRequirements {
            volume_ml,
            matchstick_heads,
            hazelnuts,
            open_palette_hours: 12,
            airtight_pot_days: 7,
            freezer_days: 30,
            advice,
        }
    }
}

// =========================================================================
// 5. Live Palette Freshness Tracker (Session Degradation & Phase)
// =========================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FreshnessPhase {
    FreshCream,    // 0 - 2h
    ButteryOpr,    // 2 - 6h
    TackySkinning, // 6 - 12h
    Exhausted,     // > 12h
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
            (Self::FreshCream, Locale::Fr) => "🟢 Crème Fraîche (Phase Fluide 0-2h)",
            (Self::FreshCream, Locale::En) => "🟢 Fresh Cream (Fluid Stage 0-2h)",
            (Self::ButteryOpr, Locale::Fr) => "🟡 Beurre Onctueux (Phase OPR 2-6h)",
            (Self::ButteryOpr, Locale::En) => "🟡 Buttery Paste (OPR Stage 2-6h)",
            (Self::TackySkinning, Locale::Fr) => "🟠 Peau de Polymérisation (6-12h)",
            (Self::TackySkinning, Locale::En) => "🟠 Tacky Skinning Stage (6-12h)",
            (Self::Exhausted, Locale::Fr) => "🔴 Pâte Épuisée / Filmée (>12h)",
            (Self::Exhausted, Locale::En) => "🔴 Exhausted / Cured Film (>12h)",
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
    pub fn evaluate(elapsed_minutes: u32, temp_c: f64) -> FreshnessPhase {
        let temp_factor = if temp_c > 22.0 {
            1.0 + (temp_c - 22.0) * 0.05
        } else {
            1.0
        };
        let effective_minutes = (elapsed_minutes as f64 * temp_factor) as u32;

        if effective_minutes <= 120 {
            FreshnessPhase::FreshCream
        } else if effective_minutes <= 360 {
            FreshnessPhase::ButteryOpr
        } else if effective_minutes <= 720 {
            FreshnessPhase::TackySkinning
        } else {
            FreshnessPhase::Exhausted
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn miniature_acrylic_catalog_is_populated_and_valid_hex() {
        assert!(MINIATURE_ACRYLICS.len() >= 40);
        for acr in MINIATURE_ACRYLICS {
            let rgb = acr.rgb();
            assert!(rgb.r() >= 0.0 && rgb.r() <= 1.0);
            assert!(rgb.g() >= 0.0 && rgb.g() <= 1.0);
            assert!(rgb.b() >= 0.0 && rgb.b() <= 1.0);
        }
    }

    #[test]
    fn closest_acrylic_finds_khorne_red() {
        let target = Rgb::of_hex("#6A0001").unwrap();
        let closest = find_closest_acrylic(target).expect("found closest");
        assert_eq!(closest.name, "Khorne Red");
    }

    #[test]
    fn miniature_ladder_produces_5_ordered_contrast_steps() {
        let base = Rgb::of_hex("#9A1115").unwrap(); // Mephiston Red
        let ladder = MiniatureLadder::of_basecoat(base);
        let steps = ladder.steps_hex();
        assert_eq!(steps.len(), 5);
        // Deep shadow must be substantially darker than base
        assert!(ladder.deep_shadow.r() < base.r());
        // Edge highlight must be substantially brighter than base
        assert!(ladder.edge_highlight.r() > base.r() || ladder.edge_highlight.g() > base.g());
    }

    #[test]
    fn weathering_effects_all_have_labels_and_recipes() {
        for effect in WeatheringEffect::ALL {
            assert!(!effect.label_i18n(Locale::Fr).is_empty());
            assert!(!effect.oil_recipe_i18n(Locale::Fr).is_empty());
            assert!(Rgb::of_hex(effect.hex_color()).is_ok());
        }
    }

    #[test]
    fn batch_squad_tracks_progress() {
        let mut squad = BatchSquad::standard_space_marine_squad("Intercessors", 5);
        assert_eq!(squad.total_tasks(), 45); // 5 x 9
        assert_eq!(squad.completed_tasks(), 0);
        assert_eq!(squad.progress_pct(), 0.0);

        squad.toggle(0, 0);
        squad.toggle(0, 1);
        assert_eq!(squad.completed_tasks(), 2);
        assert!(squad.progress_pct() > 4.0);
        assert!(squad.mother_mix_multiplier() > 1.5);
    }

    #[test]
    fn nmm_metal_has_5_ordered_reflectance_steps() {
        for nmm in NmmMetal::ALL {
            assert_eq!(nmm.steps_hex().len(), 5);
            assert!(!nmm.oil_formula_i18n(Locale::Fr).is_empty());
            assert!(!nmm.ground_bounce_advice_i18n(Locale::Fr).is_empty());
        }
    }

    #[test]
    fn heat_box_accelerates_drying_time() {
        let calc_ambient = HeatBoxCalculator::new(20.0);
        assert!((calc_ambient.acceleration_factor() - 1.0).abs() < 1e-4);

        let calc_box = HeatBoxCalculator::new(40.0);
        assert!(calc_box.acceleration_factor() > 4.0);
        let drying_ambient = 24.0;
        let drying_box = calc_box.accelerated_hours(drying_ambient);
        assert!(drying_box < 6.0); // 24 hours reduced to under 6 hours!
        assert!(calc_box.is_safe_for_miniature_plastics());

        let hot_box = HeatBoxCalculator::new(50.0);
        assert!(!hot_box.is_safe_for_miniature_plastics());
        assert!(hot_box.safety_warning_i18n(Locale::Fr).is_some());
    }

    #[test]
    fn special_effects_and_brush_care() {
        for fx in SpecialEffectGore::ALL {
            assert!(Rgb::of_hex(fx.hex_color()).is_ok());
            assert!(!fx.recipe_i18n(Locale::Fr).is_empty());
        }

        let mut brush = BrushTracker::new("Test Brush", "Raphael", "#1", "Kolinsky");
        assert!(!brush.needs_deep_clean());
        brush.uses_count = 10;
        assert!(brush.needs_deep_clean());
    }

    #[test]
    fn solvents_and_mediums_definitions() {
        for sol in SolventType::ALL {
            assert!(!sol.name().is_empty());
            assert!(!sol.evaporation_speed_i18n(Locale::Fr).is_empty());
        }
        for med in OilMedium::ALL {
            assert!(!med.name().is_empty());
            assert!(!med.dosage_rule_i18n(Locale::Fr).is_empty());
        }
    }

    #[test]
    fn stripping_guide_flags_acetone_hazards() {
        let plastic_acetone = StrippingGuide::evaluate(MiniatureMaterial::PolystyrenePlastic, StrippingAgent::PureAcetone);
        assert_eq!(plastic_acetone.severity, StrippingVerdictSeverity::Forbidden);

        let plastic_ipa = StrippingGuide::evaluate(MiniatureMaterial::PolystyrenePlastic, StrippingAgent::Ipa99);
        assert_eq!(plastic_ipa.severity, StrippingVerdictSeverity::Safe);

        let resin_ipa = StrippingGuide::evaluate(MiniatureMaterial::ResinSla3d, StrippingAgent::Ipa99);
        assert_eq!(resin_ipa.severity, StrippingVerdictSeverity::Caution);

        let metal_acetone = StrippingGuide::evaluate(MiniatureMaterial::WhiteMetal, StrippingAgent::PureAcetone);
        assert_eq!(metal_acetone.severity, StrippingVerdictSeverity::Safe);
    }

    #[test]
    fn freezer_simulator_extends_open_time() {
        let open_air = FreezerStorageSimulator::estimated_usable_days(20.0, false, false);
        assert!(open_air <= 2.0);

        let freezer = FreezerStorageSimulator::estimated_usable_days(20.0, true, true);
        assert_eq!(freezer, 28.0);
    }

    #[test]
    fn dry_pigment_and_army_recipe_sheet() {
        for p in DryPigmentEffect::ALL {
            assert!(Rgb::of_hex(p.dry_hex()).is_ok());
            assert!(Rgb::of_hex(p.fixed_hex()).is_ok());
            assert!(!p.application_tip_i18n(Locale::Fr).is_empty());
        }

        let sheet = ArmyRecipeSheet::new_standard_marine();
        let md = sheet.to_markdown();
        assert!(md.contains("Blood Angels"));
        assert!(md.contains("Mephiston Red"));

        let html = sheet.to_printable_html();
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Tactical Squad Alpha"));
    }

    #[test]
    fn oil_harmonies_and_consistency_and_glazes() {
        let base_red = Rgb::of_hex("#C8102E").unwrap();
        let harmonies = OilHarmoniesCalculator::calculate(base_red);
        assert_eq!(harmonies.len(), 8);
        for h in &harmonies {
            assert!(Rgb::of_hex(&h.hex).is_ok());
            assert!(!h.role_i18n(Locale::Fr).is_empty());
            assert!(!h.oil_tip_i18n(Locale::Fr).is_empty());
        }

        for tech in OilConsistencyTechnique::ALL {
            assert!(!tech.label_i18n(Locale::Fr).is_empty());
            assert!(!tech.description_i18n(Locale::Fr).is_empty());
        }

        // Zenithal simulation test
        let white_catchlight = OilGlazeZenithalSimulator::simulate_glaze(base_red, 1.0, 0.7);
        let dark_occlusion = OilGlazeZenithalSimulator::simulate_glaze(base_red, 0.08, 0.7);
        assert!(white_catchlight.r() > dark_occlusion.r());
        assert!(dark_occlusion.r() < 0.15);
    }

    #[test]
    fn palette_layout_and_vehicle_and_blending_and_mother_mix_and_freshness() {
        for layout in PaletteLayoutType::ALL {
            assert!(!layout.title_i18n(Locale::Fr).is_empty());
            assert!(!layout.setup_advice_i18n(Locale::Fr).is_empty());
            let slots = layout.slots(Locale::Fr);
            assert!(!slots.is_empty());
            for s in slots {
                assert!(Rgb::of_hex(s.hex).is_ok());
            }
        }

        for fx in VehicleEffectKind::ALL {
            assert!(!fx.commercial_name().is_empty());
            assert!(Rgb::of_hex(fx.target_hex()).is_ok());
            assert!(!fx.artist_oil_formula_i18n(Locale::Fr).is_empty());
            assert!(!fx.solvent_and_dilution_i18n(Locale::Fr).is_empty());
        }

        let eval_opr = WetOnWetBlendingAdvisor::evaluate(
            OilConsistencyTechnique::CreamOpr,
            OilConsistencyTechnique::CreamOpr,
            Locale::Fr,
        );
        assert_eq!(eval_opr.rating, BlendingRating::Excellent);

        let eval_clash = WetOnWetBlendingAdvisor::evaluate(
            OilConsistencyTechnique::PurePaste,
            OilConsistencyTechnique::CapillaryPinWash,
            Locale::Fr,
        );
        assert_eq!(eval_clash.rating, BlendingRating::Difficult);

        let req = BatchMotherMixCalculator::calculate(MiniatureScale::Scale32mm, 10, true, Locale::Fr);
        assert!(req.volume_ml > 0.5);
        assert!(req.matchstick_heads >= 10);
        assert!(!req.advice.is_empty());

        assert_eq!(LivePaletteFreshnessTracker::evaluate(30, 20.0), FreshnessPhase::FreshCream);
        assert_eq!(LivePaletteFreshnessTracker::evaluate(180, 20.0), FreshnessPhase::ButteryOpr);
        assert_eq!(LivePaletteFreshnessTracker::evaluate(480, 20.0), FreshnessPhase::TackySkinning);
        assert_eq!(LivePaletteFreshnessTracker::evaluate(900, 20.0), FreshnessPhase::Exhausted);
    }

}
