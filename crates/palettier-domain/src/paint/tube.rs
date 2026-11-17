//! Paint tubes, pigments, safety ratings, and catalog matching.

use super::properties::{DryingClass, Opacity, PaintSystem};
use crate::color::{Colorant, Rgb, delta_e_2000};
use crate::i18n::Locale;
use indexmap::IndexSet;
use serde::{Deserialize, Serialize};

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
    /// Oil tube or acrylic pot.
    #[serde(default)]
    pub system: PaintSystem,
}

impl Paint {
    pub fn new(
        brand: impl Into<String>,
        name: impl Into<String>,
        hex_color: impl Into<String>,
    ) -> Self {
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
            system: PaintSystem::Oil,
        }
    }

    /// Same paint, declared as belonging to another system.
    pub fn in_system(mut self, system: PaintSystem) -> Self {
        self.system = system;
        self
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
            if p.starts_with("PR108")
                || p.starts_with("PY35")
                || p.starts_with("PY37")
                || p.starts_with("PO20")
            {
                hazards.push("Cadmium (sulfoséléniure)".to_string());
                level = SafetyLevel::Hazard;
            } else if p.starts_with("PB28")
                || p.starts_with("PB35")
                || p.starts_with("PB36")
                || p.starts_with("PB72")
                || p.starts_with("PB74")
                || p.starts_with("PG19")
                || p.starts_with("PG26")
                || p.starts_with("PV14")
            {
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

    /// True for metallic, pearl or effect paints (`ALU` or `EFFEKT` pigment tag) whose
    /// reflective flakes do not mix linearly like flat matte pigments.
    pub fn is_effect(&self) -> bool {
        self.pigments
            .iter()
            .any(|p| p.eq_ignore_ascii_case("ALU") || p.eq_ignore_ascii_case("EFFEKT"))
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
    pub fn advice(&self, locale: Locale) -> &'static str {
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
    pub fn label(&self, locale: Locale) -> &'static str {
        match locale {
            Locale::Fr => match self {
                Self::ClassI => "ASTM I (Excellent)",
                Self::ClassII => "ASTM II (Très bon)",
                Self::ClassIII => "ASTM III (Fugitif)",
            },
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

