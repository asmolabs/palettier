//! Générateur de « Guide de mélanges » d'atelier (inspiré des fiches Norma Blue).
//!
//! Transforme une palette et un plan de peinture (ou un projet) en une fiche d'établi
//! directement lisible le pinceau à la main :
//! 1. **Ta palette** : chaque tube reçoit une abréviation courte (`B`, `JC`, `Rc`, `Ga`, `Pr`, `Vi`, `Oc`, `N`...)
//!    et une note de comportement (opacité, vitesse de séchage, puissance tinctoriale).
//! 2. **Mélanges intermédiaires à préparer en début de séance** : lorsque la palette est restreinte
//!    (ex. coffret de 8 tubes sans terre de Sienne ni terre d'ombre), calcule par Kubelka-Munk les
//!    trois mélanges-mères d'atelier (`Si` — Sienne, `BF` — Brun foncé, `NC` — Noir coloré).
//! 3. **Lavis sur sous-couche blanche (à l'huile)** : pré-ombrage global et lavis par grande famille de zones.
//! 4. **Zones et recettes en langage d'atelier** : exprime les dosages avec les codes courts (`4 B + 1 Oc + une pointe de Rc`,
//!    `Base + Si`, `une trace de Pr au cure-dent`) accompagnés des indications d'anatomie ou de sculpture.
//! 5. **Conseils numérotés** adaptés à la palette et au sujet, et export A4 imprimable (HTML/PDF).

use crate::color::{Rgb, delta_e_2000};
use crate::i18n::Locale;
use crate::mix::{ColorMixService, MixSuggestion, PaintPart};
use crate::paint::{DryingClass, Opacity, Paint};
use crate::plan::{PaintingPlan, PlanLayer, PlanZone, ProjectPlanner};
use crate::project::Project;
use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};

/// Entrée de la section « Ta palette » avec son code court et son comportement à l'établi.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GuidePaletteEntry {
    pub short_code: String,
    pub paint: Paint,
    pub behavior: String,
}

/// Mélange-mère préparé en début de séance (`Si`, `BF`, `NC`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MotherMix {
    pub code: String,
    pub name: String,
    pub hex_color: String,
    pub recipe_text: String,
    pub adjustment_note: String,
}

/// Lavis à l'huile sur sous-couche acrylique sèche.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UndercoatWash {
    pub name: String,
    pub hex_color: String,
    pub recipe_text: String,
}

/// Une ligne de couche dans une zone du guide de mélanges.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GuideLayerRow {
    pub role: String,
    pub swatch_hex: String,
    pub target_hex: String,
    pub painter_recipe: String,
    pub placement_note: String,
    pub delta_e: f64,
}

/// Une zone complète dans le guide de mélanges.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GuideZone {
    pub name: String,
    pub material: String,
    pub note: String,
    pub rows: Vec<GuideLayerRow>,
}

/// Le Guide de mélanges complet, prêt à être affiché à l'écran ou imprimé en fiche A4.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MixingGuide {
    pub title: String,
    pub subtitle: String,
    pub disclaimer: String,
    pub palette_entries: Vec<GuidePaletteEntry>,
    pub mother_mixes: Vec<MotherMix>,
    pub mother_mixes_note: String,
    pub washes: Vec<UndercoatWash>,
    pub washes_method: String,
    pub zones: Vec<GuideZone>,
    pub advice: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct MixingGuideBuilder {
    mixer: ColorMixService,
    planner: ProjectPlanner,
}

impl MixingGuideBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Construit un guide de mélanges complet à partir d'un projet existant.
    pub fn from_project(&self, project: &Project) -> MixingGuide {
        self.from_project_i18n(project, Locale::Fr)
    }

    /// Construit un guide de mélanges complet à partir d'un projet existant dans la langue choisie.
    pub fn from_project_i18n(&self, project: &Project, locale: Locale) -> MixingGuide {
        let plan = self.planner.plan(project);
        let paints = project.effective_paints();
        let mut guide = self.from_plan_i18n(&plan, paints, locale);
        if !project.name.trim().is_empty() {
            guide.title = project.name.clone();
        }
        if !project.notes.trim().is_empty() {
            for line in project
                .notes
                .lines()
                .map(str::trim)
                .filter(|l| !l.is_empty())
            {
                if !guide.advice.iter().any(|a| a == line) {
                    guide.advice.push(line.to_string());
                }
            }
        }
        guide
    }

    /// Construit un guide de mélanges complet à partir d'un `PaintingPlan` et des tubes de la palette.
    pub fn from_plan(&self, plan: &PaintingPlan, paints: &[Paint]) -> MixingGuide {
        self.from_plan_i18n(plan, paints, Locale::Fr)
    }

    /// Construit un guide de mélanges complet à partir d'un `PaintingPlan` et des tubes de la palette dans la langue choisie.
    pub fn from_plan_i18n(
        &self,
        plan: &PaintingPlan,
        paints: &[Paint],
        locale: Locale,
    ) -> MixingGuide {
        let palette_entries = Self::build_palette_entries_i18n(paints, locale);
        let code_map: IndexMap<String, String> = palette_entries
            .iter()
            .map(|e| (Self::paint_key(&e.paint), e.short_code.clone()))
            .collect();

        let mother_mixes = self.build_mother_mixes(paints, &code_map, locale);
        let washes = self.build_washes(&plan.zones, paints, &code_map, &mother_mixes);

        let zones: Vec<GuideZone> = plan
            .zones
            .iter()
            .map(|z| self.build_zone(z, paints, &code_map, &mother_mixes, locale))
            .collect();

        let subtitle = if plan.palette_name.trim().is_empty() {
            plan.subject.clone()
        } else {
            format!("{} — {}", plan.subject, plan.palette_name)
        };

        let advice = Self::build_advice(plan, &palette_entries, &mother_mixes, &zones);

        let (title, disclaimer, mm_note, wash_method) = match locale {
            Locale::Fr => (
                "Guide de mélanges".to_string(),
                "Les couleurs des pastilles sont approximatives : elles indiquent la direction du mélange, pas la teinte exacte. Les proportions sont indicatives, ajuste à l'œil.".to_string(),
                "Ces trois mélanges remplacent la sienne brûlée, la terre d'ombre et le noir pour les ombres. Prépares-en en quantité pour garder des couleurs cohérentes entre les zones. Proportions de départ à ajuster à l'œil. Pour vérifier un mélange, étire une touche fine avec un peu de blanc : sa vraie tendance (trop rouge, trop verte, trop bleue) se voit mieux éclaircie.".to_string(),
                "Sous-couche blanche ou gris clair acrylique bien sèche, puis lavis appliqué généreusement sur toute la zone. Laisse poser 5 à 10 minutes, puis retire l'excès sur les reliefs avec un pinceau sec et propre ou un coton-tige à peine humide de médium : le lavis reste dans les creux et dessine les volumes. Laisse sécher 24 à 48 h avant de peindre les bases par-dessus en couches fines.".to_string(),
            ),
            Locale::En => (
                "Mixing Guide".to_string(),
                "Color swatches are approximate: they indicate the direction of the mix, not the exact shade. Ratios are indicative, adjust by eye.".to_string(),
                "These three mixes replace burnt sienna, umber, and tube black for shadows. Prepare a good batch to keep hues consistent across all zones. Initial ratios to be adjusted by eye. To check a mix, pull a thin tint with a touch of white: its true undertone (too red, green, or blue) shows clearly when lightened.".to_string(),
                "Over a thoroughly dry acrylic white or pale grey ground, apply the wash liberally over the entire zone. Let sit 5-10 minutes, then wipe excess off highlights using a clean dry brush or lightly medium-moistened swab: the wash settles into recesses to sculpt volumes. Allow 24-48 hours to dry before blocking in base colors in thin coats.".to_string(),
            ),
        };

        MixingGuide {
            title,
            subtitle,
            disclaimer,
            palette_entries,
            mother_mixes,
            mother_mixes_note: mm_note,
            washes,
            washes_method: wash_method,
            zones,
            advice,
        }
    }

    fn paint_key(paint: &Paint) -> String {
        if let Some(id) = paint.id {
            format!("id:{id}")
        } else {
            format!("{}|{}|{}", paint.brand, paint.name, paint.code)
        }
    }

    /// Attribue à chaque tube un code mnémonique court (`B`, `JC`, `Rc`, `Ga`, `Pr`, `Vi`, `Oc`, `N`...)
    /// sans collision au sein de la palette, ainsi qu'une note de comportement à l'établi.
    pub fn build_palette_entries(paints: &[Paint]) -> Vec<GuidePaletteEntry> {
        Self::build_palette_entries_i18n(paints, Locale::Fr)
    }

    /// Attribue à chaque tube un code mnémonique court et une note de comportement dans la langue choisie.
    pub fn build_palette_entries_i18n(paints: &[Paint], locale: Locale) -> Vec<GuidePaletteEntry> {
        let mut used = IndexSet::new();
        let mut entries = Vec::with_capacity(paints.len());

        for paint in paints {
            let base_code = Self::preferred_short_code(paint);
            let mut candidate = base_code.clone();
            let mut n = 2;
            while used.contains(&candidate) {
                candidate = format!("{base_code}{n}");
                n += 1;
            }
            used.insert(candidate.clone());

            entries.push(GuidePaletteEntry {
                short_code: candidate,
                behavior: Self::describe_behavior_i18n(paint, locale),
                paint: paint.clone(),
            });
        }
        entries
    }

    fn preferred_short_code(paint: &Paint) -> String {
        let low = paint.name.to_lowercase();
        let pigs = &paint.pigments;

        if low.contains("blanc")
            || low.contains("white")
            || pigs.contains("PW6")
            || pigs.contains("PW4")
        {
            return "B".to_string();
        }
        if low.contains("citron") || low.contains("lemon") || pigs.contains("PY3") {
            return "JC".to_string();
        }
        if (low.contains("jaune") || low.contains("yellow"))
            && (low.contains("cadmium") || pigs.contains("PY35"))
        {
            return "JCd".to_string();
        }
        if low.contains("ocre")
            || low.contains("ochre")
            || pigs.contains("PY42")
            || pigs.contains("PY43")
        {
            return "Oc".to_string();
        }
        if low.contains("garance")
            || low.contains("alizarin")
            || low.contains("carmin")
            || low.contains("madder")
            || pigs.contains("PR83")
            || pigs.contains("PV19")
        {
            return "Ga".to_string();
        }
        if (low.contains("rouge")
            || low.contains("red")
            || low.contains("scarlet")
            || low.contains("vermilion"))
            && (low.contains("cadmium")
                || pigs.contains("PR108")
                || pigs.contains("PR254")
                || pigs.contains("PR255"))
        {
            return "Rc".to_string();
        }
        if low.contains("prusse")
            || low.contains("prussian")
            || low.contains("phtalo")
            || low.contains("phthalo")
            || pigs.contains("PB27")
            || pigs.contains("PB15")
        {
            return "Pr".to_string();
        }
        if low.contains("outremer") || low.contains("ultramarine") || pigs.contains("PB29") {
            return "Out".to_string();
        }
        if low.contains("cobalt")
            && (low.contains("bleu") || low.contains("blue") || pigs.contains("PB28"))
        {
            return "Co".to_string();
        }
        if low.contains("viridian")
            || low.contains("émeraude")
            || low.contains("emeraude")
            || low.contains("chrome feu")
            || pigs.contains("PG18")
            || pigs.contains("PG7")
        {
            return "Vi".to_string();
        }
        if low.contains("sienne") || low.contains("sienna") {
            return "Si".to_string();
        }
        if low.contains("ombre") || low.contains("umber") {
            return "TO".to_string();
        }
        if low.contains("noir")
            || low.contains("black")
            || pigs.contains("PBk9")
            || pigs.contains("PBk11")
            || pigs.contains("PBk7")
        {
            return "N".to_string();
        }

        // Initiales par défaut (1 majuscule + 1 minuscule)
        let words: Vec<&str> = paint
            .name
            .split(|c: char| !c.is_alphabetic())
            .filter(|w| w.len() > 2)
            .collect();
        if words.len() >= 2 {
            let c1 = words[0].chars().next().unwrap_or('T').to_uppercase();
            let c2 = words[1].chars().next().unwrap_or('x').to_lowercase();
            format!("{c1}{c2}")
        } else if let Some(w) = words.first() {
            let mut chars = w.chars();
            let c1 = chars.next().unwrap_or('T').to_uppercase();
            let c2 = chars.next().unwrap_or('x').to_lowercase();
            format!("{c1}{c2}")
        } else {
            "T".to_string()
        }
    }

    #[allow(dead_code)]
    fn describe_behavior(paint: &Paint) -> String {
        Self::describe_behavior_i18n(paint, Locale::Fr)
    }

    fn describe_behavior_i18n(paint: &Paint, locale: Locale) -> String {
        let low = paint.name.to_lowercase();
        let pigs = &paint.pigments;

        if low.contains("prusse")
            || low.contains("prussian")
            || pigs.contains("PB27")
            || paint.tinting_strength >= 0.88
        {
            return locale
                .tr(
                    "Très puissant : trace au cure-dent",
                    "Extremely potent: toothpick trace only",
                )
                .to_string();
        }
        if low.contains("ocre")
            || low.contains("ochre")
            || pigs.contains("PY42")
            || pigs.contains("PY43")
        {
            return locale
                .tr(
                    "Opaque, le « ciment » de la palette",
                    "Opaque, the studio palette backbone",
                )
                .to_string();
        }
        if low.contains("noir minéral") || low.contains("mineral black") || pigs.contains("PBk11")
        {
            return locale
                .tr(
                    "Surtout pour les olives et rabattre les verts",
                    "Mainly for olives and subduing greens",
                )
                .to_string();
        }
        if low.contains("viridian") || low.contains("chrome feu") || pigs.contains("PG18") {
            return locale
                .tr("Viridian, transparent", "Viridian, transparent")
                .to_string();
        }
        if low.contains("garance") || low.contains("alizarin") || pigs.contains("PR83") {
            return locale
                .tr("Froid, transparent", "Cool, transparent")
                .to_string();
        }
        if low.contains("citron") || low.contains("lemon") || pigs.contains("PY3") {
            return locale.tr("Froid, lumineux", "Cool, bright").to_string();
        }
        if low.contains("blanc") || low.contains("white") || pigs.contains("PW6") {
            let drying_str = match paint.drying_class {
                DryingClass::Slow | DryingClass::VerySlow => {
                    locale.tr("lent à sécher", "slow drying")
                }
                DryingClass::Medium => locale.tr("séchage moyen", "medium drying"),
                DryingClass::Fast => locale.tr("séchage rapide", "fast drying"),
            };
            return format!("{}, {}", paint.opacity.label(locale), drying_str);
        }

        let lab = paint.color().to_lab();
        let temp = if lab.b > 18.0 || lab.a > 15.0 {
            locale.tr("chaud", "warm")
        } else if lab.b < -10.0 || lab.a < -10.0 {
            locale.tr("froid", "cool")
        } else {
            locale.tr("neutre", "neutral")
        };
        format!("{}, {}", paint.opacity.label(locale), temp)
    }

    /// Synthétise les 3 mélanges intermédiaires (`Si` — Sienne, `BF` — Brun foncé, `NC` — Noir coloré)
    /// à partir des tubes présents dans la palette.
    fn build_mother_mixes(
        &self,
        paints: &[Paint],
        code_map: &IndexMap<String, String>,
        locale: Locale,
    ) -> Vec<MotherMix> {
        let sienna = locale.tr("Sienne", "Sienna");
        let dark_brown = locale.tr("Brun foncé", "Dark brown");
        let coloured_black = locale.tr("Noir coloré", "Coloured black");
        if paints.len() < 3 {
            return Vec::new();
        }

        // Cherche les archétypes dans la palette (Ocre, Rouge chaud, Garance/Carmin, Bleu puissant/froid)
        let ochre = Self::find_by_code(paints, code_map, "Oc");
        let warm_red = Self::find_by_code(paints, code_map, "Rc");
        let cool_red = Self::find_by_code(paints, code_map, "Ga").or(warm_red);
        let dark_blue = Self::find_by_code(paints, code_map, "Pr")
            .or_else(|| Self::find_by_code(paints, code_map, "Out"))
            .or_else(|| Self::find_by_code(paints, code_map, "Co"));

        let mut out = Vec::new();

        // 1. Si — Sienne
        let target_sienna = Rgb::of_hex("#8D472A").unwrap_or_else(|_| Rgb::new(0.55, 0.28, 0.16));
        if let (Some(oc), Some(rc)) = (ochre, warm_red) {
            let mut comps = Vec::new();
            if let Ok(p1) = PaintPart::new(oc.clone(), 3.0) {
                comps.push(p1);
            }
            if let Ok(p2) = PaintPart::new(rc.clone(), 1.0) {
                comps.push(p2);
            }
            if let Some(ga) = cool_red
                && Self::paint_key(ga) != Self::paint_key(rc)
                && let Ok(p3) = PaintPart::new(ga.clone(), 0.5)
            {
                comps.push(p3);
            }
            let hex = self
                .mixer
                .mix(&comps)
                .map(|r| r.color.to_hex())
                .unwrap_or_else(|_| target_sienna.to_hex());
            let recipe_text = if let Some(ga) = cool_red
                && Self::paint_key(ga) != Self::paint_key(rc)
            {
                let ga_c = Self::code_of(ga, code_map);
                let oc_c = Self::code_of(oc, code_map);
                let rc_c = Self::code_of(rc, code_map);
                match locale {
                    Locale::Fr => format!(
                        "3 {oc_c} + 1 {rc_c} + une pointe de {ga_c}. Plus de {rc_c} : sienne plus rouge ; plus d'{oc_c} : plus dorée."
                    ),
                    Locale::En => format!(
                        "3 {oc_c} + 1 {rc_c} + a touch of {ga_c}. More {rc_c}: redder sienna; more {oc_c}: more golden."
                    ),
                }
            } else {
                let oc_c = Self::code_of(oc, code_map);
                let rc_c = Self::code_of(rc, code_map);
                match locale {
                    Locale::Fr => format!(
                        "3 {oc_c} + 1 {rc_c}. Plus de {rc_c} : sienne plus rouge ; plus d'{oc_c} : plus dorée."
                    ),
                    Locale::En => format!(
                        "3 {oc_c} + 1 {rc_c}. More {rc_c}: redder sienna; more {oc_c}: more golden."
                    ),
                }
            };
            out.push(MotherMix {
                code: "Si".to_string(),
                name: sienna.to_string(),
                hex_color: hex,
                recipe_text,
                adjustment_note: String::new(),
            });
        } else if let Some(sugg) = self
            .mixer
            .suggest_mixes(target_sienna, paints, 1, Some(3))
            .into_iter()
            .next()
        {
            out.push(MotherMix {
                code: "Si".to_string(),
                name: sienna.to_string(),
                hex_color: sugg.color.to_hex(),
                recipe_text: format!(
                    "{}. {}",
                    Self::format_suggestion_short(&sugg, code_map, locale),
                    locale.tr(
                        "Base terreuse chaude pour les ombres de peau et les cuirs.",
                        "Warm earthy base for skin shadows and leather."
                    )
                ),
                adjustment_note: String::new(),
            });
        }

        // 2. BF — Brun foncé
        let target_umber = Rgb::of_hex("#452A1D").unwrap_or_else(|_| Rgb::new(0.27, 0.16, 0.11));
        if let (Some(oc), Some(rc), Some(pr)) = (ochre, warm_red, dark_blue) {
            let mut comps = Vec::new();
            if let Ok(p1) = PaintPart::new(oc.clone(), 4.0) {
                comps.push(p1);
            }
            if let Ok(p2) = PaintPart::new(rc.clone(), 2.0) {
                comps.push(p2);
            }
            if let Ok(p3) = PaintPart::new(pr.clone(), 0.5) {
                comps.push(p3);
            }
            let hex = self
                .mixer
                .mix(&comps)
                .map(|r| r.color.to_hex())
                .unwrap_or_else(|_| target_umber.to_hex());
            let pr_code = Self::code_of(pr, code_map);
            out.push(MotherMix {
                code: "BF".to_string(),
                name: dark_brown.to_string(),
                hex_color: hex,
                recipe_text: {
                    let oc_c = Self::code_of(oc, code_map);
                    let rc_c = Self::code_of(rc, code_map);
                    match locale {
                        Locale::Fr => format!(
                            "2 {oc_c} + 1 {rc_c}, puis {pr_code} trace par trace au cure-dent. Le mélange fonce vite : arrête-toi avant qu'il ne verdisse."
                        ),
                        Locale::En => format!(
                            "2 {oc_c} + 1 {rc_c}, then {pr_code} a trace at a time with a toothpick. The mix darkens fast: stop before it turns green."
                        ),
                    }
                },
                adjustment_note: String::new(),
            });
        } else if let Some(sugg) = self
            .mixer
            .suggest_mixes(target_umber, paints, 1, Some(3))
            .into_iter()
            .next()
        {
            out.push(MotherMix {
                code: "BF".to_string(),
                name: dark_brown.to_string(),
                hex_color: sugg.color.to_hex(),
                recipe_text: format!(
                    "{}. {}",
                    Self::format_suggestion_short(&sugg, code_map, locale),
                    locale.tr(
                        "Pour les creux soutenus, cheveux et bois sombres.",
                        "For deep recesses, hair and dark wood."
                    )
                ),
                adjustment_note: String::new(),
            });
        }

        // 3. NC — Noir coloré
        let target_black = Rgb::of_hex("#211D22").unwrap_or_else(|_| Rgb::new(0.13, 0.11, 0.13));
        if let (Some(ga), Some(pr)) = (cool_red, dark_blue) {
            let mut comps = Vec::new();
            if let Ok(p1) = PaintPart::new(ga.clone(), 2.0) {
                comps.push(p1);
            }
            if let Ok(p2) = PaintPart::new(pr.clone(), 1.0) {
                comps.push(p2);
            }
            let hex = self
                .mixer
                .mix(&comps)
                .map(|r| r.color.to_hex())
                .unwrap_or_else(|_| target_black.to_hex());
            let ga_c = Self::code_of(ga, code_map);
            let pr_c = Self::code_of(pr, code_map);
            out.push(MotherMix {
                code: "NC".to_string(),
                name: coloured_black.to_string(),
                hex_color: hex,
                recipe_text: match locale {
                    Locale::Fr => format!(
                        "2 {ga_c} + 1 {pr_c}. Commence par {ga_c}, ajoute {pr_c} peu à peu. Plus de {ga_c} : noir violacé chaud ; plus de {pr_c} : noir bleuté froid."
                    ),
                    Locale::En => format!(
                        "2 {ga_c} + 1 {pr_c}. Start with {ga_c}, add {pr_c} little by little. More {ga_c}: warm violet black; more {pr_c}: cool bluish black."
                    ),
                },
                adjustment_note: String::new(),
            });
        } else if let Some(sugg) = self
            .mixer
            .suggest_mixes(target_black, paints, 1, Some(2))
            .into_iter()
            .next()
        {
            out.push(MotherMix {
                code: "NC".to_string(),
                name: coloured_black.to_string(),
                hex_color: sugg.color.to_hex(),
                recipe_text: format!(
                    "{}. {}",
                    Self::format_suggestion_short(&sugg, code_map, locale),
                    locale.tr(
                        "Plus vivant qu'un noir pur pour les ombres profondes.",
                        "Livelier than pure black for deep shadows."
                    )
                ),
                adjustment_note: String::new(),
            });
        }

        out
    }

    fn build_washes(
        &self,
        zones: &[PlanZone],
        paints: &[Paint],
        code_map: &IndexMap<String, String>,
        mother_mixes: &[MotherMix],
    ) -> Vec<UndercoatWash> {
        let mut washes = Vec::new();
        let has_bf = mother_mixes.iter().any(|m| m.code == "BF");
        let has_si = mother_mixes.iter().any(|m| m.code == "Si");
        let ga_code = Self::find_by_code(paints, code_map, "Ga")
            .or_else(|| Self::find_by_code(paints, code_map, "Rc"))
            .map(|p| Self::code_of(p, code_map))
            .unwrap_or_else(|| "Rc".to_string());

        // Lavis global (pré-ombrage)
        washes.push(UndercoatWash {
            name: "Lavis global (pré-ombrage)".to_string(),
            hex_color: "#6E5244".to_string(),
            recipe_text: if has_bf {
                format!(
                    "1 BF + une pointe de {ga_code}, dilué avec environ 6 à 8 volumes de médium. Consistance d'un thé fort."
                )
            } else {
                "Terre d'ombre + une pointe de rouge transparent, dilué avec 6 à 8 volumes de médium (consistance d'un thé fort).".to_string()
            },
        });

        // Lavis peau si une zone concerne la peau / visage
        if zones.iter().any(|z| {
            let l = z.name.to_lowercase();
            let m = z.material.to_lowercase();
            l.contains("peau")
                || l.contains("visage")
                || l.contains("main")
                || m.contains("peau")
                || m.contains("chair")
        }) {
            washes.push(UndercoatWash {
                name: "Lavis peau".to_string(),
                hex_color: "#8E553D".to_string(),
                recipe_text: if has_si {
                    format!("Si + une pointe de {ga_code}, même dilution. Uniquement sur visage, cou, mains.")
                } else {
                    "Sienne + une pointe de rouge carmin, même dilution (visage, cou, mains).".to_string()
                },
            });
        }

        // Lavis froid ou vert selon les autres zones
        for z in zones {
            let l = z.name.to_lowercase();
            if (l.contains("dragon")
                || l.contains("créature")
                || l.contains("creature")
                || l.contains("vert"))
                && !washes
                    .iter()
                    .any(|w| w.name.contains("créature") || w.name.contains("vert"))
                && let Some(vi) = Self::find_by_code(paints, code_map, "Vi")
            {
                let vi_c = Self::code_of(vi, code_map);
                washes.push(UndercoatWash {
                    name: format!("Lavis {}", z.name.to_lowercase()),
                    hex_color: "#3F5338".to_string(),
                    recipe_text: format!("{vi_c} + une pointe de BF, même dilution."),
                });
            } else if (l.contains("katana")
                || l.contains("fourreau")
                || l.contains("bleu")
                || l.contains("armure")
                || l.contains("métal"))
                && !washes.iter().any(|w| w.name.contains("froid"))
                && let Some(pr) = Self::find_by_code(paints, code_map, "Pr")
            {
                let pr_c = Self::code_of(pr, code_map);
                washes.push(UndercoatWash {
                    name: format!("Lavis {}", z.name.to_lowercase()),
                    hex_color: "#2C3B59".to_string(),
                    recipe_text: format!(
                        "{pr_c} + une pointe de {ga_code}, plus dilué encore ({pr_c} très puissant)."
                    ),
                });
            }
        }

        washes
    }

    fn build_zone(
        &self,
        zone: &PlanZone,
        paints: &[Paint],
        code_map: &IndexMap<String, String>,
        mother_mixes: &[MotherMix],
        locale: Locale,
    ) -> GuideZone {
        let mut ordered_layers: Vec<PlanLayer> = Vec::new();
        // Dans les guides de référence, l'ordre de lecture d'une zone est :
        // Base -> Ombre 1 -> Ombre 2 -> Lumière 1 -> Lumière 2 -> Accents (Rougeurs, Demi-teinte, Lèvres, Yeux...)
        if let Some(base) = &zone.base {
            ordered_layers.push(base.clone());
        }
        // `zone.shadows` est stocké `[Ombre 2, Ombre 1]`, donc on itère dans le sens Ombre 1 puis Ombre 2 :
        for s in zone.shadows.iter().rev() {
            ordered_layers.push(s.clone());
        }
        for h in &zone.highlights {
            ordered_layers.push(h.clone());
        }
        for a in &zone.accents {
            ordered_layers.push(a.clone());
        }

        let base_layer = zone.base.as_ref();
        let rows = ordered_layers
            .iter()
            .map(|layer| self.build_layer_row(layer, base_layer, paints, code_map, mother_mixes, locale))
            .collect();

        GuideZone {
            name: zone.name.clone(),
            material: zone.material.clone(),
            note: zone.note.clone(),
            rows,
        }
    }

    fn build_layer_row(
        &self,
        layer: &PlanLayer,
        base_layer: Option<&PlanLayer>,
        paints: &[Paint],
        code_map: &IndexMap<String, String>,
        mother_mixes: &[MotherMix],
        locale: Locale,
    ) -> GuideLayerRow {
        let swatch_hex = layer.achieved.to_hex();
        let target_hex = layer.target.to_hex();

        let raw_recipe = match &layer.recipe {
            Some(sugg) => Self::format_painter_recipe(
                &layer.role,
                layer.target,
                sugg,
                base_layer,
                paints,
                code_map,
                mother_mixes,
                locale,
            ),
            None => locale
                .tr("Tube pur ou mélange à ajuster à l'œil", "Neat tube or mix to adjust by eye")
                .to_string(),
        };

        let placement_note = layer.note.trim().to_string();
        let painter_recipe = if !placement_note.is_empty() && !raw_recipe.contains(&placement_note)
        {
            if placement_note.starts_with('(') {
                format!("{raw_recipe} {placement_note}")
            } else {
                format!("{raw_recipe} ({placement_note})")
            }
        } else {
            raw_recipe
        };

        GuideLayerRow {
            role: layer.role.clone(),
            swatch_hex,
            target_hex,
            painter_recipe,
            placement_note,
            delta_e: layer.delta_e,
        }
    }

    /// Formate une suggestion Kubelka-Munk avec la syntaxe d'atelier des guides Norma Blue :
    /// - Codes courts (`B`, `Oc`, `Rc`, `Ga`, `Pr`, `Vi`, `JC`, `N`)
    /// - Petites parts transformées en « une pointe de X » (5–14 %) ou « une trace de X » (< 5 % ou pigment très puissant)
    /// - Réutilisation de `Base`, `Si`, `BF`, `NC` pour les ombres et lumières lorsque c'est pertinent.
    pub fn format_painter_recipe(
        role: &str,
        target: Rgb,
        sugg: &MixSuggestion,
        base_layer: Option<&PlanLayer>,
        paints: &[Paint],
        code_map: &IndexMap<String, String>,
        mother_mixes: &[MotherMix],
            locale: Locale,
    ) -> String {
        let role_low = role.to_lowercase();
        let white_code = Self::find_by_code(paints, code_map, "B")
            .map(|p| Self::code_of(p, code_map))
            .unwrap_or_else(|| "B".to_string());

        // Si un mélange-mère (`Si`, `BF`, `NC`) est très proche de la cible (ΔE < 4.5), on l'utilise directement !
        for mm in mother_mixes {
            if let Ok(mm_rgb) = Rgb::of_hex(&mm.hex_color) {
                let de = delta_e_2000(target, mm_rgb);
                if de < 3.8 {
                    return mm.code.clone();
                }
            }
        }

        // Pour Lumière 1 : si Base existe et que la suggestion contient surtout les composants de Base + Blanc
        let total_parts: f64 = sugg.parts.iter().map(|p| p.parts).sum::<f64>().max(1e-6);
        if (role_low == "lumière 1" || role_low == "lumiere 1")
            && base_layer.is_some()
            && sugg
                .parts
                .iter()
                .any(|c| Self::code_of(&c.paint, code_map) == white_code)
        {
            let extras: Vec<String> = sugg
                .parts
                .iter()
                .filter(|c| {
                    Self::code_of(&c.paint, code_map) != white_code
                        && (c.parts / total_parts) < 0.22
                })
                .map(|c| format!("une trace de {}", Self::code_of(&c.paint, code_map)))
                .collect();
            if extras.is_empty() {
                return format!("Base + {white_code}");
            } else {
                return format!("Base + {white_code} + {}", extras.join(" + "));
            }
        }

        // Pour Ombre 1 : si un mélange-mère (Si ou BF) est dans la direction chromatique de l'ombre
        if (role_low == "ombre 1" || role_low == "ombre 2")
            && let Some(base) = base_layer
        {
            let target_lab = target.to_lab();
            let base_lab = base.target.to_lab();
            if target_lab.l < base_lab.l {
                // Ombre chaude/terreuse -> Si ou BF
                if role_low == "ombre 1"
                    && target_lab.a > 6.0
                    && target_lab.b > 8.0
                    && mother_mixes.iter().any(|m| m.code == "Si")
                {
                    let extra_cool = sugg
                        .parts
                        .iter()
                        .find(|c| {
                            let code = Self::code_of(&c.paint, code_map);
                            (code == "Ga" || code == "Pr") && (c.parts / total_parts) <= 0.25
                        })
                        .map(|c| format!(" + une pointe de {}", Self::code_of(&c.paint, code_map)))
                        .unwrap_or_default();
                    return format!("Base + Si{extra_cool}");
                }
                if role_low == "ombre 2"
                    && target_lab.l < 28.0
                    && target_lab.a > 4.0
                    && mother_mixes.iter().any(|m| m.code == "Si")
                {
                    let has_ga = sugg
                        .parts
                        .iter()
                        .any(|c| Self::code_of(&c.paint, code_map) == "Ga");
                    let has_pr = sugg
                        .parts
                        .iter()
                        .any(|c| Self::code_of(&c.paint, code_map) == "Pr");
                    if has_ga && has_pr {
                        return format!("Si + Ga + {} Pr", locale.tr("une trace de", "a trace of"));
                    } else if mother_mixes.iter().any(|m| m.code == "BF") {
                        return format!("Si + {} BF", locale.tr("une trace de", "a trace of"));
                    }
                }
            }
        }

        Self::format_suggestion_short(sugg, code_map, locale)
    }

    /// Convertit les composants d'un `MixSuggestion` en notation courte d'atelier :
    /// ex. `4 B + 1 Oc + une pointe de Rc` ou `Vi + une trace de Pr`.
    pub fn format_suggestion_short(
        sugg: &MixSuggestion,
        code_map: &IndexMap<String, String>,
            locale: Locale,
    ) -> String {
        if sugg.parts.is_empty() {
            return String::new();
        }
        if sugg.parts.len() == 1 {
            let c = &sugg.parts[0];
            return match locale {
                Locale::Fr => format!("{} pur", Self::code_of(&c.paint, code_map)),
                Locale::En => format!("{} neat", Self::code_of(&c.paint, code_map)),
            };
        }

        let total_parts: f64 = sugg.parts.iter().map(|p| p.parts).sum::<f64>().max(1e-6);
        let mut sorted: Vec<&PaintPart> = sugg.parts.iter().collect();
        sorted.sort_by(|a, b| b.parts.total_cmp(&a.parts));

        let major: Vec<&&PaintPart> = sorted
            .iter()
            .filter(|c| {
                let share = c.parts / total_parts;
                share >= 0.15 && !(c.paint.tinting_strength >= 0.88 && share < 0.20)
            })
            .collect();
        let minor: Vec<&&PaintPart> = sorted
            .iter()
            .filter(|c| {
                !major
                    .iter()
                    .any(|m| Self::paint_key(&m.paint) == Self::paint_key(&c.paint))
            })
            .collect();

        let mut parts_str = Vec::new();

        if major.len() == 1 && !minor.is_empty() {
            parts_str.push(Self::code_of(&major[0].paint, code_map));
        } else if !major.is_empty() {
            let min_share = major
                .iter()
                .map(|c| c.parts / total_parts)
                .fold(f64::INFINITY, f64::min)
                .max(0.05);
            for comp in &major {
                let share = comp.parts / total_parts;
                let ratio = (share / min_share).round().clamp(1.0, 6.0) as usize;
                let code = Self::code_of(&comp.paint, code_map);
                if major.len() == 2 && ratio == 1 {
                    parts_str.push(code);
                } else {
                    parts_str.push(format!("{ratio} {code}"));
                }
            }
        }

        for comp in minor {
            let share = comp.parts / total_parts;
            let code = Self::code_of(&comp.paint, code_map);
            if share < 0.06 || comp.paint.tinting_strength >= 0.88 {
                parts_str.push(format!("{} {code}", locale.tr("une trace de", "a trace of")));
            } else {
                parts_str.push(format!("{} {code}", locale.tr("une pointe de", "a touch of")));
            }
        }

        parts_str.join(" + ")
    }

    fn code_of(paint: &Paint, code_map: &IndexMap<String, String>) -> String {
        code_map
            .get(&Self::paint_key(paint))
            .cloned()
            .unwrap_or_else(|| Self::preferred_short_code(paint))
    }

    fn find_by_code<'a>(
        paints: &'a [Paint],
        code_map: &IndexMap<String, String>,
        wanted_code: &str,
    ) -> Option<&'a Paint> {
        paints
            .iter()
            .find(|p| Self::code_of(p, code_map).eq_ignore_ascii_case(wanted_code))
    }

    fn build_advice(
        plan: &PaintingPlan,
        entries: &[GuidePaletteEntry],
        mother_mixes: &[MotherMix],
        zones: &[GuideZone],
    ) -> Vec<String> {
        let mut tips = Vec::new();

        if !plan.approach.trim().is_empty() {
            tips.push(plan.approach.trim().to_string());
        }

        tips.push(
            "Pose une sous-couche acrylique bien sèche (gris clair, ivoire ou zénithale noir/blanc) : elle révèle les volumes et accroche l'huile."
                .to_string(),
        );
        tips.push(
            "Travaille en couches fines. Un pinceau sec et propre retire l'excès et fond les transitions sans empâter la sculpture."
                .to_string(),
        );
        tips.push(
            "Ordre conseillé : base et ombres en frais sur frais, puis les lumières. Garde les lumières les plus claires et les points de brillance pour la fin, sur une couche sèche au toucher."
                .to_string(),
        );

        if let Some(pr) = entries
            .iter()
            .find(|e| e.paint.tinting_strength >= 0.85 || e.short_code == "Pr")
        {
            tips.push(format!(
                "{} ({}) : extrêmement puissant, prélève-le toujours à la pointe d'un cure-dent.",
                pr.paint.name, pr.short_code
            ));
        }

        let transparents: Vec<String> = entries
            .iter()
            .filter(|e| e.paint.opacity == Opacity::Transparent)
            .map(|e| format!("{} ({})", e.paint.name, e.short_code))
            .collect();
        let opaques: Vec<String> = entries
            .iter()
            .filter(|e| e.paint.opacity == Opacity::Opaque && e.short_code != "B")
            .map(|e| format!("{} ({})", e.paint.name, e.short_code))
            .collect();
        if !transparents.is_empty() && !opaques.is_empty() {
            tips.push(format!(
                "Transparents ({}) : parfaits en glacis et dans les ombres. Opaques ({}) : tes meilleures bases couvrantes.",
                transparents.join(", "),
                opaques.join(", ")
            ));
        }

        tips.push(
            "N'éclaircis pas les rouges et les verts uniquement au blanc (effet crayeux) : passe d'abord par le jaune ou l'ocre, garde le blanc pur pour l'éclat final."
                .to_string(),
        );

        if mother_mixes.iter().any(|m| m.code == "NC") {
            tips.push(
                "Pour les ombres profondes, préfère le Noir coloré (NC) au noir sorti du tube : l'ombre reste vibrante et transparente."
                    .to_string(),
            );
        }

        if entries.iter().any(|e| e.short_code == "Oc") {
            tips.push(
                "Une pointe d'ocre (Oc) glissée dans presque tous les mélanges harmonise l'ensemble de la pièce sous une même lumière."
                    .to_string(),
            );
        }

        if zones.len() > 1 {
            let names: Vec<&str> = zones.iter().map(|z| z.name.as_str()).collect();
            tips.push(format!(
                "Mène les zones de front ({}) : peins une zone pendant qu'une autre sèche (1 à 3 jours selon l'épaisseur).",
                names.join(", ")
            ));
        }

        tips.push(
            "Finition : vernis mat ou satiné uniforme après séchage complet ; une touche de vernis satiné ou brillant ponctuel sur les yeux, les lèvres et les ors."
                .to_string(),
        );

        tips
    }
}

impl MixingGuide {
    /// Génère une fiche d'atelier A4 autonome en HTML/CSS prête à imprimer ou enregistrer en PDF,
    /// fidèle à la mise en page des guides `references/guide-melanges-*.pdf`.
    pub fn to_printable_html(&self, photo_data_uri: Option<&str>) -> String {
        self.to_printable_html_i18n(photo_data_uri, Locale::Fr)
    }

    /// Génère une fiche d'atelier A4 autonome en HTML/CSS prête à imprimer dans la langue choisie.
    pub fn to_printable_html_i18n(&self, photo_data_uri: Option<&str>, locale: Locale) -> String {
        let mut html = String::with_capacity(16384);
        html.push_str("<!DOCTYPE html>\n<html lang=\"");
        html.push_str(locale.code());
        html.push_str(
            r#"">
<head>
<meta charset="utf-8">
<title>"#,
        );
        html.push_str(&Self::escape_html(&self.title));
        html.push_str(" — ");
        html.push_str(&Self::escape_html(&self.subtitle));
        html.push_str(r#"</title>
<style>
  @page { size: A4 portrait; margin: 16mm 16mm 18mm 16mm; }
  * { box-sizing: border-box; -webkit-print-color-adjust: exact !important; print-color-adjust: exact !important; }
  body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    color: #262320;
    background: #FFFFFF;
    margin: 0;
    padding: 24px 32px;
    font-size: 10.5pt;
    line-height: 1.42;
    max-width: 210mm;
    margin-left: auto;
    margin-right: auto;
  }
  .toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: #FAF6F0;
    border: 1px solid #E5DEC9;
    padding: 10px 16px;
    border-radius: 8px;
    margin-bottom: 22px;
  }
  .toolbar button {
    background: #7C2D12;
    color: #FFFFFF;
    border: none;
    border-radius: 6px;
    padding: 8px 16px;
    font-weight: 600;
    cursor: pointer;
    font-size: 10pt;
  }
  @media print {
    body { padding: 0; max-width: none; }
    .toolbar { display: none !important; }
  }
  .header-grid {
    display: flex;
    gap: 24px;
    align-items: flex-start;
    margin-bottom: 18px;
  }
  .piece-photo {
    width: 175px;
    max-height: 225px;
    object-fit: cover;
    border-radius: 3px;
    border: 1px solid #D6D0C4;
    flex-shrink: 0;
  }
  .header-main { flex: 1; }
  h1 {
    font-size: 22pt;
    font-weight: 800;
    margin: 0 0 4px 0;
    color: #1F1C18;
    letter-spacing: -0.02em;
  }
  .subtitle {
    font-size: 11.5pt;
    color: #57524A;
    margin: 0 0 10px 0;
  }
  .disclaimer {
    font-size: 9pt;
    color: #6E675F;
    margin: 0 0 14px 0;
  }
  h2 {
    font-size: 13pt;
    font-weight: 700;
    color: #7C2D12;
    margin: 18px 0 8px 0;
    page-break-after: avoid;
  }
  table.guide-table {
    width: 100%;
    border-collapse: collapse;
    margin-bottom: 10px;
  }
  table.guide-table tr {
    border-bottom: 1px solid #EBE6DC;
    page-break-inside: avoid;
  }
  table.guide-table td {
    padding: 5px 8px 5px 0;
    vertical-align: middle;
  }
  .swatch-cell {
    width: 44px;
    padding-right: 10px !important;
  }
  .swatch-box {
    width: 36px;
    height: 22px;
    border: 1px solid #B8B0A4;
    border-radius: 2px;
  }
  .code-cell {
    width: 42px;
    font-weight: 700;
    color: #1F1C18;
  }
  .role-cell {
    width: 135px;
    font-weight: 700;
    color: #1F1C18;
  }
  .behavior-cell {
    color: #57524A;
    font-size: 9.5pt;
    width: 36%;
  }
  .note-paragraph {
    font-size: 9pt;
    color: #4A453E;
    margin: 6px 0 14px 0;
    line-height: 1.45;
  }
  ol.advice-list {
    margin: 6px 0 0 0;
    padding-left: 20px;
  }
  ol.advice-list li {
    margin-bottom: 6px;
    page-break-inside: avoid;
  }
  .footer {
    margin-top: 24px;
    padding-top: 8px;
    border-top: 1px solid #EBE6DC;
    text-align: right;
    font-size: 8.5pt;
    color: #8C8479;
  }
</style>
</head>
<body>
<div class="toolbar">
  <div><strong>"#);
        html.push_str(locale.tr("Fiche d'établi Palettier", "Palettier Workbench Sheet"));
        html.push_str("</strong> — ");
        html.push_str(locale.tr(
            "Prête à imprimer en A4 ou enregistrer en PDF",
            "Ready to print on A4 or save as PDF",
        ));
        html.push_str(
            r#"</div>
  <button onclick="window.print()">"#,
        );
        html.push_str(locale.tr(
            "Imprimer / Enregistrer en PDF (⌘P)",
            "Print / Save to PDF (⌘P)",
        ));
        html.push_str(
            r#"</button>
</div>
<div class="header-grid">
"#,
        );

        if let Some(uri) = photo_data_uri {
            let alt_txt = locale.tr("Pièce de référence", "Reference Piece");
            html.push_str(&format!(
                r#"  <img class="piece-photo" src="{uri}" alt="{alt_txt}"/>"#
            ));
        }

        html.push_str(r#"  <div class="header-main">"#);
        html.push_str(&format!(
            "<h1>{}</h1>\n<p class=\"subtitle\">{}</p>\n<p class=\"disclaimer\">{}</p>\n",
            Self::escape_html(&self.title),
            Self::escape_html(&self.subtitle),
            Self::escape_html(&self.disclaimer)
        ));

        // Section Ta palette / Your palette
        if !self.palette_entries.is_empty() {
            html.push_str(&format!(
                "<h2>{}</h2>\n<table class=\"guide-table\">\n",
                locale.tr("Ta palette", "Your Palette")
            ));
            for entry in &self.palette_entries {
                let code_suffix = if entry.paint.code.trim().is_empty() {
                    String::new()
                } else {
                    format!(
                        " <span style=\"color:#78716A;\">({})</span>",
                        Self::escape_html(&entry.paint.code)
                    )
                };
                html.push_str(&format!(
                    "<tr><td class=\"swatch-cell\"><div class=\"swatch-box\" style=\"background:{};\"></div></td><td class=\"code-cell\">{}</td><td>{}{code_suffix}</td><td class=\"behavior-cell\">{}</td></tr>\n",
                    Self::escape_html(&entry.paint.hex_color),
                    Self::escape_html(&entry.short_code),
                    Self::escape_html(&entry.paint.name),
                    Self::escape_html(&entry.behavior),
                ));
            }
            html.push_str("</table>\n");
        }
        html.push_str("  </div>\n</div>\n");

        // Mélanges intermédiaires / Mother mixes
        if !self.mother_mixes.is_empty() {
            html.push_str(&format!(
                "<h2>{}</h2>\n<table class=\"guide-table\">\n",
                locale.tr(
                    "Mélanges intermédiaires à préparer en début de séance",
                    "Intermediate Mother Mixes to Prepare Before Painting"
                )
            ));
            for mm in &self.mother_mixes {
                html.push_str(&format!(
                    "<tr><td class=\"swatch-cell\"><div class=\"swatch-box\" style=\"background:{};\"></div></td><td class=\"role-cell\"><strong>{}</strong> — {}</td><td>{}</td></tr>\n",
                    Self::escape_html(&mm.hex_color),
                    Self::escape_html(&mm.code),
                    Self::escape_html(&mm.name),
                    Self::escape_html(&mm.recipe_text),
                ));
            }
            html.push_str("</table>\n");
            html.push_str(&format!(
                "<p class=\"note-paragraph\">{}</p>\n",
                Self::escape_html(&self.mother_mixes_note)
            ));
        }

        // Lavis sur sous-couche blanche / Washes
        if !self.washes.is_empty() {
            html.push_str(&format!(
                "<h2>{}</h2>\n<table class=\"guide-table\">\n",
                locale.tr(
                    "Lavis sur sous-couche blanche (à l'huile)",
                    "Oil Washes Over White Undercoat"
                )
            ));
            for w in &self.washes {
                html.push_str(&format!(
                    "<tr><td class=\"swatch-cell\"><div class=\"swatch-box\" style=\"background:{};\"></div></td><td class=\"role-cell\">{}</td><td>{}</td></tr>\n",
                    Self::escape_html(&w.hex_color),
                    Self::escape_html(&w.name),
                    Self::escape_html(&w.recipe_text),
                ));
            }
            html.push_str("</table>\n");
            html.push_str(&format!(
                "<p class=\"note-paragraph\"><strong>{}</strong> {}</p>\n",
                locale.tr("Méthode :", "Method:"),
                Self::escape_html(&self.washes_method)
            ));
        }

        // Zones
        for zone in &self.zones {
            let zone_heading = if zone.material.trim().is_empty()
                || zone
                    .name
                    .to_lowercase()
                    .contains(&zone.material.to_lowercase())
            {
                zone.name.clone()
            } else {
                format!("{} ({})", zone.name, zone.material)
            };
            html.push_str(&format!(
                "<h2>{}</h2>\n<table class=\"guide-table\">\n",
                Self::escape_html(&zone_heading)
            ));
            for row in &zone.rows {
                html.push_str(&format!(
                    "<tr><td class=\"swatch-cell\"><div class=\"swatch-box\" style=\"background:{};\"></div></td><td class=\"role-cell\">{}</td><td>{}</td></tr>\n",
                    Self::escape_html(&row.swatch_hex),
                    Self::escape_html(&row.role),
                    Self::escape_html(&row.painter_recipe),
                ));
            }
            html.push_str("</table>\n");
            if !zone.note.trim().is_empty() {
                html.push_str(&format!(
                    "<p class=\"note-paragraph\">{}</p>\n",
                    Self::escape_html(&zone.note)
                ));
            }
        }

        // Conseils / Studio tips
        if !self.advice.is_empty() {
            html.push_str(&format!(
                "<h2>{}</h2>\n<ol class=\"advice-list\">\n",
                locale.tr("Conseils", "Studio Tips")
            ));
            for tip in &self.advice {
                html.push_str(&format!("  <li>{}</li>\n", Self::escape_html(tip)));
            }
            html.push_str("</ol>\n");
        }

        html.push_str(&format!(
            "<div class=\"footer\">{} — {}</div>\n</body>\n</html>",
            Self::escape_html(&self.title),
            Self::escape_html(&self.subtitle)
        ));
        html
    }

    fn escape_html(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::CatalogLoader;
    use crate::plan::{PaintingPlan, PlanLayer, PlanZone};

    fn norma_blue_starter_box() -> Vec<Paint> {
        let all = CatalogLoader::default().load();
        let codes = [
            "21114", "21236", "21311", "21318", "21418", "21502", "21600", "21705",
        ];
        codes
            .iter()
            .filter_map(|code| {
                all.iter()
                    .find(|p| p.brand.contains("Norma Blue") && p.code == *code)
                    .cloned()
            })
            .collect()
    }

    #[test]
    fn norma_blue_starter_box_produces_expected_short_codes_and_mother_mixes() {
        let box_paints = norma_blue_starter_box();
        assert_eq!(8, box_paints.len());

        let entries = MixingGuideBuilder::build_palette_entries(&box_paints);
        let short_codes: Vec<&str> = entries.iter().map(|e| e.short_code.as_str()).collect();
        assert_eq!(
            vec!["B", "JC", "Rc", "Ga", "Pr", "Vi", "Oc", "N"],
            short_codes
        );

        let builder = MixingGuideBuilder::new();
        let plan = PaintingPlan {
            subject: "Buste de samouraï".to_string(),
            palette_name: "Schmincke Norma Blue (coffret 71200097)".to_string(),
            approach: String::new(),
            system: crate::paint::PaintSystem::Oil,
            zones: vec![PlanZone {
                name: "Peau".to_string(),
                material: "Chair chaude".to_string(),
                note: String::new(),
                base: Some(PlanLayer {
                    role: "Base".to_string(),
                    target: Rgb::of_hex("#DFB190").unwrap(),
                    technique: crate::paint::Technique::Blending,
                    note: "visage et cou".to_string(),
                    recipe: None,
                    achieved: Rgb::of_hex("#DFB190").unwrap(),
                    delta_e: 1.2,
                }),
                shadows: Vec::new(),
                highlights: Vec::new(),
                accents: Vec::new(),
            }],
        };

        let guide = builder.from_plan(&plan, &box_paints);
        let mm_codes: Vec<&str> = guide.mother_mixes.iter().map(|m| m.code.as_str()).collect();
        assert_eq!(vec!["Si", "BF", "NC"], mm_codes);
        assert!(!guide.washes.is_empty());
        assert!(!guide.advice.is_empty());

        let html = guide.to_printable_html(None);
        assert!(html.contains("Ta palette"));
        assert!(html.contains("Mélanges intermédiaires"));
        assert!(html.contains("Lavis sur sous-couche blanche"));
        assert!(html.contains("Buste de samouraï"));
    }
}
