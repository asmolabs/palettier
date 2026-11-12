use crate::color::{Rgb, delta_e_2000};
use crate::drying::Workshop;
use crate::error::DomainError;
use crate::i18n::Locale;
use crate::paint::{DryingClass, LayerThickness, Paint, PaintSystem, Technique, Ventilation};
use crate::palette::Palette;
use crate::recipe::Recipe;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum LayerKind {
    #[default]
    Ladder,
    Accent,
}

impl LayerKind {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            LayerKind::Ladder => "LADDER",
            LayerKind::Accent => "ACCENT",
        }
    }
}

impl FromStr for LayerKind {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_uppercase().as_str() {
            "LADDER" | "" => Ok(LayerKind::Ladder),
            "ACCENT" => Ok(LayerKind::Accent),
            other => Err(DomainError::InvalidArgument(format!(
                "LayerKind inconnu : {other}"
            ))),
        }
    }
}

/// Record of an applied coat, capturing the workshop conditions at application time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppliedCoat {
    pub at: DateTime<Utc>,
    pub workshop: Workshop,
    pub drying_class: DryingClass,
}

impl AppliedCoat {
    pub fn of(
        at: DateTime<Utc>,
        temperature: Option<f64>,
        humidity: Option<f64>,
        ventilation: Option<Ventilation>,
        drying: Option<DryingClass>,
    ) -> Self {
        let std_ws = Workshop::standard();
        Self {
            at,
            workshop: Workshop {
                temperature_celsius: temperature.unwrap_or(std_ws.temperature_celsius),
                relative_humidity: humidity.unwrap_or(std_ws.relative_humidity),
                ventilation: ventilation.unwrap_or(std_ws.ventilation),
            },
            drying_class: drying.unwrap_or(DryingClass::Medium),
        }
    }
}

/// A project layer: its role, target color, and technique.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectLayer {
    pub id: Option<i64>,
    pub role: String,
    pub target_hex: String,
    /// A known technique. A label the application did not recognise (from the assistant
    /// or an older version) is kept in `note` instead of being silently replaced.
    pub technique: Technique,
    pub note: String,
    pub kind: LayerKind,
    pub applied: Option<AppliedCoat>,
}

impl ProjectLayer {
    pub fn new(role: impl Into<String>, target_hex: impl Into<String>) -> Self {
        Self {
            id: None,
            role: role.into(),
            target_hex: target_hex.into(),
            technique: Technique::Glaze,
            note: String::new(),
            kind: LayerKind::Ladder,
            applied: None,
        }
    }

    pub fn target(&self) -> Rgb {
        Rgb::of_hex(&self.target_hex).unwrap_or_else(|_| Rgb::new(0.5, 0.5, 0.5))
    }

    pub fn is_applied(&self) -> bool {
        self.applied.is_some()
    }

    /// The system this coat is painted in, within a piece painted in `project`.
    pub fn system_within(&self, project: PaintSystem) -> PaintSystem {
        self.technique.system_within(project)
    }

    /// The same coat with another technique.
    pub fn with_technique(mut self, technique: Technique) -> Self {
        self.technique = technique;
        self
    }

    /// Reads a technique label that may not be one the application knows: a known one
    /// becomes the technique, an unknown one falls back to the default of `system` and is
    /// kept, word for word, at the head of the note.
    pub fn set_technique_label(&mut self, label: &str, system: PaintSystem) {
        let label = label.trim();
        match Technique::parse(label) {
            Some(t) => self.technique = t,
            None => {
                self.technique = Technique::default_for(system);
                if !label.is_empty() {
                    let kept = format!("Technique proposée : {label}.");
                    self.note = if self.note.trim().is_empty() {
                        kept
                    } else {
                        format!("{kept} {}", self.note)
                    };
                }
            }
        }
    }

    pub fn mark_applied(
        &self,
        at: DateTime<Utc>,
        workshop: Workshop,
        drying_class: DryingClass,
    ) -> Self {
        let mut next = self.clone();
        next.applied = Some(AppliedCoat {
            at,
            workshop,
            drying_class,
        });
        next
    }

    pub fn clear_applied(&self) -> Self {
        let mut next = self.clone();
        next.applied = None;
        next
    }
}

/// A zone of the subject in a project (e.g. face, cloak, belt).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectZone {
    pub id: Option<i64>,
    pub name: String,
    pub material: String,
    pub note: String,
    pub layers: Vec<ProjectLayer>,
}

impl ProjectZone {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: None,
            name: name.into(),
            material: String::new(),
            note: String::new(),
            layers: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PhotoRole {
    #[default]
    Piece,
    Reference,
    Progress,
}

impl PhotoRole {
    pub const ALL: [PhotoRole; 3] = [PhotoRole::Piece, PhotoRole::Reference, PhotoRole::Progress];

    pub fn label(&self) -> &'static str {
        match self {
            PhotoRole::Piece => "La piece",
            PhotoRole::Reference => "Reference",
            PhotoRole::Progress => "Avancement",
        }
    }

    pub fn label_i18n(&self, locale: Locale) -> &'static str {
        match locale {
            Locale::Fr => self.label(),
            Locale::En => match self {
                PhotoRole::Piece => "The piece",
                PhotoRole::Reference => "Reference",
                PhotoRole::Progress => "Progress",
            },
        }
    }

    pub fn as_db_str(&self) -> &'static str {
        match self {
            PhotoRole::Piece => "PIECE",
            PhotoRole::Reference => "REFERENCE",
            PhotoRole::Progress => "PROGRESS",
        }
    }
}

impl FromStr for PhotoRole {
    type Err = DomainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_uppercase().as_str() {
            "PIECE" | "" => Ok(PhotoRole::Piece),
            "REFERENCE" => Ok(PhotoRole::Reference),
            "PROGRESS" => Ok(PhotoRole::Progress),
            other => Err(DomainError::InvalidArgument(format!(
                "PhotoRole inconnu : {other}"
            ))),
        }
    }
}

/// A project photo: the piece itself, a reference image, or a work-in-progress shot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectPhoto {
    pub id: Option<i64>,
    pub data: Vec<u8>,
    pub role: PhotoRole,
    pub caption: String,
    pub added_at: DateTime<Utc>,
}

impl PartialEq for ProjectPhoto {
    fn eq(&self, other: &Self) -> bool {
        self.id.is_some() && self.id == other.id
    }
}

/// A piece in progress, along with its painting plan.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub id: Option<i64>,
    pub name: String,
    pub subject: String,
    pub approach: String,
    pub notes: String,
    /// Oil or acrylic: decides the thinners, techniques, drying model and rules offered.
    /// A coat whose technique belongs to the other system (an oil wash over an acrylic
    /// piece) keeps its own system, see [`ProjectLayer::system_within`].
    #[serde(default)]
    pub system: PaintSystem,
    pub palette: Option<Palette>,
    pub paints: Vec<Paint>,
    pub zones: Vec<ProjectZone>,
    pub photos: Vec<ProjectPhoto>,
    pub created_at: DateTime<Utc>,
}

impl Project {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: None,
            name: name.into(),
            subject: String::new(),
            approach: String::new(),
            notes: String::new(),
            system: PaintSystem::Oil,
            palette: None,
            paints: Vec::new(),
            zones: Vec::new(),
            photos: Vec::new(),
            created_at: Utc::now(),
        }
    }

    pub fn effective_paints(&self) -> &[Paint] {
        if !self.paints.is_empty() {
            &self.paints
        } else if let Some(pal) = &self.palette {
            &pal.paints
        } else {
            &[]
        }
    }

    pub fn diverges_from_palette(&self) -> bool {
        let Some(pal) = &self.palette else {
            return false;
        };
        if self.paints.is_empty() {
            return false;
        }
        let own_ids: Vec<Option<i64>> = self.paints.iter().map(|p| p.id).collect();
        let pal_ids: Vec<Option<i64>> = pal.paints.iter().map(|p| p.id).collect();
        own_ids != pal_ids
    }
}

/// Validates the fat-over-lean rule across a sequence of coats.
#[derive(Debug, Clone, Default)]
pub struct FatOverLeanService;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FatOverLeanRisk {
    pub where_name: String,
    pub under: String,
    pub over: String,
    pub drop: f64,
    pub explanation: String,
}

struct Coat {
    label: String,
    role: String,
    system: PaintSystem,
    fatness: f64,
    thickness: LayerThickness,
}

impl FatOverLeanService {
    const SIGNIFICANT_DROP: f64 = 0.40;

    pub fn new() -> Self {
        Self
    }

    pub fn inspect_project(&self, project: &Project) -> Vec<FatOverLeanRisk> {
        self.inspect_project_i18n(project, Locale::Fr)
    }

    pub fn inspect_project_i18n(&self, project: &Project, locale: Locale) -> Vec<FatOverLeanRisk> {
        project
            .zones
            .iter()
            .flat_map(|zone| {
                let coats: Vec<Coat> = zone
                    .layers
                    .iter()
                    .map(|l| Self::coat_of(l, project.system, locale))
                    .collect();
                Self::inspect(&zone.name, &coats, locale)
            })
            .collect()
    }

    pub fn inspect_recipe(&self, recipe: &Recipe) -> Vec<FatOverLeanRisk> {
        self.inspect_recipe_i18n(recipe, Locale::Fr)
    }

    pub fn inspect_recipe_i18n(&self, recipe: &Recipe, locale: Locale) -> Vec<FatOverLeanRisk> {
        let step_prefix = locale.tr("Etape", "Step");
        let coats: Vec<Coat> = recipe
            .steps
            .iter()
            .enumerate()
            .map(|(idx, step)| Coat {
                label: step.technique.label_i18n(locale).to_string(),
                role: format!("{step_prefix} {}", idx + 1),
                system: step.system(),
                fatness: step.medium.fatness(step.medium_ratio),
                thickness: step.thickness,
            })
            .collect();
        Self::inspect(&recipe.name, &coats, locale)
    }

    pub fn fatness_of(&self, technique: Technique) -> f64 {
        Self::fatness(technique)
    }

    pub fn is_veil(&self, technique: Technique) -> bool {
        technique.typical_thickness() == LayerThickness::Glaze
    }

    fn inspect(where_name: &str, coats: &[Coat], locale: Locale) -> Vec<FatOverLeanRisk> {
        let mut out = Vec::new();
        for i in 1..coats.len() {
            if let Some(risk) = Self::compare(where_name, &coats[i - 1], &coats[i], locale) {
                out.push(risk);
            }
        }
        out
    }

    fn compare(
        where_name: &str,
        under: &Coat,
        over: &Coat,
        locale: Locale,
    ) -> Option<FatOverLeanRisk> {
        match (under.system, over.system) {
            // Acrylic does not hold on oil: the water-based film finds no grip on an
            // oily surface and peels. The reverse (oil over acrylic) is the classic
            // miniature workflow and is fine once the acrylic is dry.
            (PaintSystem::Oil, PaintSystem::Acrylic) => {
                return Some(FatOverLeanRisk {
                    where_name: where_name.to_string(),
                    under: under.role.clone(),
                    over: over.role.clone(),
                    drop: 1.0,
                    explanation: Self::explain_acrylic_over_oil(&under.label, &over.label, locale),
                });
            }
            // Fat over lean is an oil rule only.
            (PaintSystem::Oil, PaintSystem::Oil) => {}
            _ => return None,
        }
        let drop = under.fatness - over.fatness;
        if drop < Self::SIGNIFICANT_DROP {
            return None;
        }
        if over.thickness < under.thickness {
            return None;
        }
        Some(FatOverLeanRisk {
            where_name: where_name.to_string(),
            under: under.role.clone(),
            over: over.role.clone(),
            drop,
            explanation: Self::explain(&under.label, &over.label, locale),
        })
    }

    fn coat_of(layer: &ProjectLayer, project: PaintSystem, locale: Locale) -> Coat {
        let technique = layer.technique;
        Coat {
            label: technique.label_i18n(locale).to_string(),
            role: layer.role.clone(),
            system: technique.system_within(project),
            fatness: Self::fatness(technique),
            thickness: technique.typical_thickness(),
        }
    }

    fn explain_acrylic_over_oil(below: &str, above: &str, locale: Locale) -> String {
        match locale {
            Locale::Fr => format!(
                "{above} est a l'acrylique, posee sur {below} a l'huile : l'acrylique n'accroche pas sur un film gras et s'ecaillera. Gardez l'ordre acrylique puis huile, ou faites cette etape a l'huile."
            ),
            Locale::En => format!(
                "{above} is acrylic, applied over {below} in oil: acrylic does not grip an oily film and will flake off. Keep the order acrylic then oil, or do this step in oil."
            ),
        }
    }

    fn fatness(technique: Technique) -> f64 {
        technique
            .default_medium()
            .fatness(technique.default_ratio())
    }

    fn explain(below: &str, above: &str, locale: Locale) -> String {
        match locale {
            Locale::Fr => format!(
                "{above} est plus maigre que {below}, et pas plus fine. Posee dessus, elle sechera avant elle et tirera dessus en vieillissant. Faites l'inverse, ou attendez le sechage a coeur de la couche du dessous et allegez la main."
            ),
            Locale::En => format!(
                "{above} is leaner than {below}, and no thinner. Applied on top, it will dry faster and pull on the underlying coat as it ages. Reverse the order, or wait until the lower coat is through-dry and apply with a lighter touch."
            ),
        }
    }
}

/// Finds owned substitutes for tubes required by a project that are not in stock.
#[derive(Debug, Clone, Default)]
pub struct SubstituteService;

#[derive(Debug, Clone, PartialEq)]
pub struct MissingPaint {
    pub paint: Paint,
    pub nearest: Option<Paint>,
    pub delta_e: f64,
}

impl MissingPaint {
    const VISIBLE_GAP: f64 = 5.0;

    pub fn is_comfortable(&self) -> bool {
        self.nearest.is_some() && self.delta_e < Self::VISIBLE_GAP
    }

    pub fn verdict(&self) -> String {
        self.verdict_i18n(Locale::Fr)
    }

    pub fn verdict_i18n(&self, locale: Locale) -> String {
        let Some(nearest) = &self.nearest else {
            return locale
                .tr(
                    "Rien sur l'etagere pour le remplacer.",
                    "Nothing on the shelf to replace it.",
                )
                .to_string();
        };
        if self.delta_e < 2.0 {
            match locale {
                Locale::Fr => format!(
                    "{} le remplace sans que cela se voie.",
                    nearest.display_name()
                ),
                Locale::En => format!(
                    "{} replaces it with no visible difference.",
                    nearest.display_name()
                ),
            }
        } else if self.delta_e < Self::VISIBLE_GAP {
            match locale {
                Locale::Fr => format!(
                    "{} en approche : l'ecart ne se verra pas sur la piece.",
                    nearest.display_name()
                ),
                Locale::En => format!(
                    "{} is close enough: the gap won't be noticeable on the piece.",
                    nearest.display_name()
                ),
            }
        } else if self.delta_e < 12.0 {
            match locale {
                Locale::Fr => format!(
                    "{} est ce qui s'en rapproche le plus, mais l'ecart se voit : a rattraper au glacis.",
                    nearest.display_name()
                ),
                Locale::En => format!(
                    "{} is the closest match, but the gap is visible: adjust with a glaze.",
                    nearest.display_name()
                ),
            }
        } else {
            locale
                .tr(
                    "Rien d'approchant sur l'etagere : celui-la, il faut l'acheter.",
                    "Nothing close on the shelf: you'll need to buy this one.",
                )
                .to_string()
        }
    }
}

impl SubstituteService {
    pub fn new() -> Self {
        Self
    }

    pub fn missing_from(&self, project: &Project, owned: &[Paint]) -> Vec<MissingPaint> {
        self.missing_among(project.effective_paints(), owned)
    }

    pub fn missing_among(&self, wanted: &[Paint], owned: &[Paint]) -> Vec<MissingPaint> {
        wanted
            .iter()
            .filter(|p| !p.in_stock)
            .map(|paint| {
                let mut best: Option<(Paint, f64)> = None;
                for candidate in owned {
                    if candidate.id.is_some() && candidate.id == paint.id {
                        continue;
                    }
                    let gap = delta_e_2000(paint.color(), candidate.color());
                    if best.as_ref().is_none_or(|(_, b_gap)| gap < *b_gap) {
                        best = Some((candidate.clone(), gap));
                    }
                }
                match best {
                    Some((nearest, delta_e)) => MissingPaint {
                        paint: paint.clone(),
                        nearest: Some(nearest),
                        delta_e,
                    },
                    None => MissingPaint {
                        paint: paint.clone(),
                        nearest: None,
                        delta_e: f64::MAX,
                    },
                }
            })
            .collect()
    }
}

/// A dominant color sampled from an image and the area share it occupies.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DominantColour {
    pub color: Rgb,
    pub share: f64,
}

/// Compares colors measured on a work-in-progress photo against the planned targets.
#[derive(Debug, Clone, Default)]
pub struct ProgressCheckService;

#[derive(Debug, Clone, PartialEq)]
pub struct ObservedColour {
    pub measured: Rgb,
    pub share: f64,
    pub zone: Option<String>,
    pub role: Option<String>,
    pub target: Option<Rgb>,
    pub delta_e: f64,
}

impl ObservedColour {
    const UNPLANNED: f64 = 12.0;

    pub fn matches_plan(&self) -> bool {
        self.zone.is_some() && self.delta_e < Self::UNPLANNED
    }

    pub fn verdict(&self) -> String {
        self.verdict_i18n(Locale::Fr)
    }

    pub fn verdict_i18n(&self, locale: Locale) -> String {
        if !self.matches_plan() {
            return locale
                .tr(
                    "Rien de prevu ne ressemble a cette teinte : appret, socle, ou une couche improvisee.",
                    "Nothing in the plan matches this hue: primer, base, or an improvised coat.",
                )
                .to_string();
        }
        let z = self.zone.as_deref().unwrap_or("");
        let r = self.role.as_deref().unwrap_or("");
        if self.delta_e < 2.0 {
            match locale {
                Locale::Fr => format!("{z} / {r} : vous y etes."),
                Locale::En => format!("{z} / {r}: spot on."),
            }
        } else if self.delta_e < 5.0 {
            match locale {
                Locale::Fr => format!("{z} / {r} : l'ecart ne se verra pas sur la piece."),
                Locale::En => format!("{z} / {r}: the gap won't be noticeable on the piece."),
            }
        } else {
            match locale {
                Locale::Fr => format!("{z} / {r} : l'ecart se voit, un glacis le rattraperait."),
                Locale::En => format!("{z} / {r}: visible gap, a glaze would bring it back."),
            }
        }
    }
}

impl ProgressCheckService {
    pub fn new() -> Self {
        Self
    }

    pub fn compare(&self, project: &Project, measured: &[DominantColour]) -> Vec<ObservedColour> {
        measured
            .iter()
            .map(|dominant| {
                let mut closest: Option<ObservedColour> = None;
                for zone in &project.zones {
                    for layer in &zone.layers {
                        let gap = delta_e_2000(dominant.color, layer.target());
                        if closest.as_ref().is_none_or(|c| gap < c.delta_e) {
                            closest = Some(ObservedColour {
                                measured: dominant.color,
                                share: dominant.share,
                                zone: Some(zone.name.clone()),
                                role: Some(layer.role.clone()),
                                target: Some(layer.target()),
                                delta_e: gap,
                            });
                        }
                    }
                }
                closest.unwrap_or(ObservedColour {
                    measured: dominant.color,
                    share: dominant.share,
                    zone: None,
                    role: None,
                    target: None,
                    delta_e: f64::MAX,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paint::Medium;
    use crate::recipe::RecipeStep;

    fn piece_with(techniques: &[&str]) -> Project {
        let mut p = Project::new("Piece");
        let mut z = ProjectZone::new("Visage");
        for (i, t) in techniques.iter().enumerate() {
            let mut l = ProjectLayer::new(format!("Couche {}", i + 1), "#C98F72");
            l.technique = Technique::parse(t).expect("technique connue");
            z.layers.push(l);
        }
        p.zones.push(z);
        p
    }

    #[test]
    fn unknown_technique_label_is_kept_in_the_note() {
        let mut known = ProjectLayer::new("Base", "#808080");
        known.set_technique_label("Glacis", PaintSystem::Acrylic);
        assert_eq!(Technique::Glaze, known.technique);
        assert!(known.note.is_empty());

        let mut unknown = ProjectLayer::new("Lumière", "#F0E0D0");
        unknown.note = "sur les pommettes".to_string();
        unknown.set_technique_label("Pointillé", PaintSystem::Acrylic);
        assert_eq!(Technique::Layering, unknown.technique);
        assert_eq!("Technique proposée : Pointillé. sur les pommettes", unknown.note);
    }

    #[test]
    fn fat_over_lean_is_not_applied_between_acrylic_coats() {
        let service = FatOverLeanService::new();
        // In oil, a lean filter over an opaque highlight would be flagged; in acrylic it is normal.
        let mut p = piece_with(&[
            Technique::BaseLayer.label(),
            Technique::Highlight.label(),
            Technique::Filter.label(),
            Technique::Layering.label(),
        ]);
        p.system = PaintSystem::Acrylic;
        assert!(service.inspect_project(&p).is_empty());
    }

    #[test]
    fn oil_over_acrylic_is_fine_acrylic_over_oil_is_flagged() {
        let service = FatOverLeanService::new();
        let mut fine = piece_with(&[Technique::Layering.label(), Technique::OilWash.label()]);
        fine.system = PaintSystem::Acrylic;
        assert!(service.inspect_project(&fine).is_empty());

        let mut wrong = piece_with(&[Technique::OilWash.label(), Technique::EdgeHighlight.label()]);
        wrong.system = PaintSystem::Acrylic;
        let risks = service.inspect_project(&wrong);
        assert_eq!(1, risks.len());
        assert!(risks[0].explanation.contains("n'accroche pas"));
    }

    #[test]
    fn all_glaze_plan_triggers_no_risk() {
        let service = FatOverLeanService::new();
        let p = piece_with(&[
            Technique::Glaze.label(),
            Technique::Glaze.label(),
            Technique::Glaze.label(),
        ]);
        assert!(service.inspect_project(&p).is_empty());
    }

    #[test]
    fn glaze_over_base_layer_triggers_no_risk() {
        let service = FatOverLeanService::new();
        assert!(
            service.fatness_of(Technique::Glaze)
                < service.fatness_of(Technique::BaseLayer)
        );
        assert!(service.is_veil(Technique::Glaze));
        let p = piece_with(&[Technique::BaseLayer.label(), Technique::Glaze.label()]);
        assert!(service.inspect_project(&p).is_empty());
    }

    #[test]
    fn lean_streaking_over_fat_blending_at_equal_thickness_is_flagged() {
        let service = FatOverLeanService::new();
        let p = piece_with(&[
            Technique::Blending.label(),
            Technique::StreakingGrime.label(),
        ]);
        let risks = service.inspect_project(&p);
        assert_eq!(1, risks.len());
        let risk = &risks[0];
        assert_eq!("Visage", risk.where_name);
        assert_eq!("Couche 1", risk.under);
        assert_eq!("Couche 2", risk.over);
        assert!(risk.drop > 0.4);
        assert!(risk.explanation.contains("tirera"));
    }

    #[test]
    fn recipe_is_checked_on_entered_values() {
        let service = FatOverLeanService::new();
        let recipe = Recipe::new("Cape rouge", "Tissu")
            .with_step(RecipeStep {
                id: None,
                technique: Technique::Blending,
                paint_mix: "Rouge + noir".to_string(),
                medium: Medium::LinseedOil,
                medium_ratio: 0.30,
                thickness: LayerThickness::Normal,
                notes: String::new(),
            })
            .with_step(RecipeStep {
                id: None,
                technique: Technique::Blending,
                paint_mix: "Rouge pur".to_string(),
                medium: Medium::OdorlessThinner,
                medium_ratio: 0.80,
                thickness: LayerThickness::Normal,
                notes: String::new(),
            });
        let risks = service.inspect_recipe(&recipe);
        assert_eq!(1, risks.len());
        assert_eq!("Cape rouge", risks[0].where_name);
        assert_eq!("Etape 1", risks[0].under);
        assert_eq!("Etape 2", risks[0].over);
    }
}
