//! Colour tools: value ladders, harmonies, non-metallic metal, light sources, glaze preview and palette layout.

use super::*;

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

    pub fn label(&self, locale: Locale) -> &'static str {
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
