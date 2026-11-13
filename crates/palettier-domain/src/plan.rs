use crate::color::{Rgb, delta_e_2000};
use crate::drying::{
    DryingContext, DryingEstimate, DryingTimeService, Workshop, waits_for_through_dry,
};
use crate::error::{DomainError, DomainResult};
use crate::i18n::Locale;
use crate::mix::{ColorMixService, MixSuggestion};
use crate::paint::{DryingClass, Paint, PaintSystem, Technique};
use crate::palette::Palette;
use crate::project::{LayerKind, Project, ProjectLayer, ProjectZone};
use chrono::Utc;
use indexmap::IndexSet;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Zone-by-zone painting plan.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaintingPlan {
    pub subject: String,
    pub palette_name: String,
    pub approach: String,
    /// Oil or acrylic: the system the plan is painted in.
    #[serde(default)]
    pub system: PaintSystem,
    pub zones: Vec<PlanZone>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanZone {
    pub name: String,
    pub material: String,
    pub note: String,
    pub base: Option<PlanLayer>,
    pub shadows: Vec<PlanLayer>,
    pub highlights: Vec<PlanLayer>,
    pub accents: Vec<PlanLayer>,
}

impl PlanZone {
    /// All value-ladder layers ordered from darkest shadow to brightest highlight.
    pub fn layers(&self) -> Vec<PlanLayer> {
        let mut out = Vec::with_capacity(
            self.shadows.len() + usize::from(self.base.is_some()) + self.highlights.len(),
        );
        for s in self.shadows.iter().rev() {
            out.push(s.clone());
        }
        if let Some(b) = &self.base {
            out.push(b.clone());
        }
        for h in &self.highlights {
            out.push(h.clone());
        }
        out
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanLayer {
    pub role: String,
    pub target: Rgb,
    pub technique: Technique,
    pub note: String,
    pub recipe: Option<MixSuggestion>,
    pub achieved: Rgb,
    pub delta_e: f64,
}

impl PlanLayer {
    pub fn reachability(&self, locale: Locale) -> &'static str {
        if self.recipe.is_none() {
            locale.tr("palette vide", "empty palette")
        } else if self.delta_e < 2.0 {
            locale.tr("atteignable tel quel", "reachable as-is")
        } else if self.delta_e < 5.0 {
            locale.tr(
                "très proche, écart invisible sur la pièce",
                "very close, gap invisible on the piece",
            )
        } else if self.delta_e < 12.0 {
            locale.tr(
                "approche, à rattraper au glacis",
                "approximate, refine with a glaze",
            )
        } else {
            locale.tr(
                "hors de portee de cette palette",
                "out of reach for this palette",
            )
        }
    }

    fn as_project_layer(&self, kind: LayerKind) -> ProjectLayer {
        ProjectLayer {
            id: None,
            role: self.role.clone(),
            target_hex: self.target.to_hex(),
            technique: self.technique,
            note: self.note.clone(),
            kind,
            applied: None,
        }
    }
}

impl PaintingPlan {
    pub fn as_project(&self, name: impl Into<String>, palette: Option<&Palette>) -> Project {
        let zones = self
            .zones
            .iter()
            .map(|zone| {
                let mut layers: Vec<ProjectLayer> = zone
                    .layers()
                    .iter()
                    .map(|l| l.as_project_layer(LayerKind::Ladder))
                    .collect();
                for accent in &zone.accents {
                    layers.push(accent.as_project_layer(LayerKind::Accent));
                }
                ProjectZone {
                    id: None,
                    name: zone.name.clone(),
                    material: zone.material.clone(),
                    note: zone.note.clone(),
                    layers,
                }
            })
            .collect();

        Project {
            id: None,
            name: name.into(),
            subject: self.subject.clone(),
            approach: self.approach.clone(),
            notes: String::new(),
            system: self.system,
            palette: palette.cloned(),
            paints: palette.map(|p| p.paints.clone()).unwrap_or_default(),
            zones,
            photos: Vec::new(),
            created_at: Utc::now(),
        }
    }
}

pub fn unique_name<'a>(wanted: &str, taken: impl IntoIterator<Item = &'a str>) -> String {
    let trimmed = wanted.trim();
    let base = if trimmed.is_empty() {
        "Projet"
    } else {
        trimmed
    };
    let used: IndexSet<String> = taken.into_iter().map(|s| s.to_lowercase()).collect();
    if !used.contains(&base.to_lowercase()) {
        return base.to_string();
    }
    let mut suffix = 2;
    loop {
        let candidate = format!("{base} {suffix}");
        if !used.contains(&candidate.to_lowercase()) {
            return candidate;
        }
        suffix += 1;
    }
}

/// Schedules a painting plan over time.
#[derive(Debug, Clone, Default)]
pub struct PlanDryingService {
    drying_time_service: DryingTimeService,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LayerSchedule {
    pub role: String,
    pub mix: String,
    pub technique: Technique,
    pub drying_class: DryingClass,
    pub estimate: DryingEstimate,
    pub start_offset: Duration,
    pub wait_after: Duration,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ZoneSchedule {
    pub zone: String,
    pub material: String,
    pub layers: Vec<LayerSchedule>,
    pub span: Duration,
}

impl ZoneSchedule {
    pub fn slowest(&self) -> DryingClass {
        self.layers.iter().fold(DryingClass::Fast, |slowest, l| {
            slowest.slowest(l.drying_class)
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlanSchedule {
    pub zones: Vec<ZoneSchedule>,
    pub sequential: Duration,
    pub parallel: Duration,
    pub advice: Vec<String>,
}

impl PlanDryingService {
    pub fn new(drying_time_service: DryingTimeService) -> Self {
        Self {
            drying_time_service,
        }
    }

    pub fn drying_class_of(layer: &PlanLayer) -> DryingClass {
        layer
            .recipe
            .as_ref()
            .map(|r| r.drying_class())
            .unwrap_or(DryingClass::Medium)
    }

    pub fn schedule(&self, plan: &PaintingPlan, workshop: Workshop) -> PlanSchedule {
        let zones: Vec<ZoneSchedule> = plan
            .zones
            .iter()
            .map(|z| self.schedule_zone(z, plan.system, workshop))
            .collect();
        let sequential = zones.iter().fold(Duration::ZERO, |acc, z| acc + z.span);
        let parallel = zones.iter().map(|z| z.span).max().unwrap_or(Duration::ZERO);
        let advice = Self::advice(&zones, sequential, parallel);
        PlanSchedule {
            zones,
            sequential,
            parallel,
            advice,
        }
    }

    fn schedule_zone(
        &self,
        zone: &PlanZone,
        system: PaintSystem,
        workshop: Workshop,
    ) -> ZoneSchedule {
        let mut layers = zone.layers();
        layers.extend(zone.accents.iter().cloned());

        let mut scheduled = Vec::with_capacity(layers.len());
        let mut offset = Duration::ZERO;

        for (i, layer) in layers.iter().enumerate() {
            let technique = layer.technique;
            let drying = Self::drying_class_of(layer);

            let estimate = self
                .drying_time_service
                .estimate(&DryingContext::for_technique_in(
                    technique, system, drying, workshop,
                ));

            let wait = if i + 1 == layers.len() {
                Duration::ZERO
            } else if {
                let next = layers[i + 1].technique;
                waits_for_through_dry(
                    technique.system_within(system),
                    next.system_within(system),
                    next,
                )
            } {
                estimate.through_dry
            } else {
                estimate.recoat
            };

            scheduled.push(LayerSchedule {
                role: layer.role.clone(),
                mix: layer
                    .recipe
                    .as_ref()
                    .map(|r| r.describe())
                    .unwrap_or_default(),
                technique,
                drying_class: drying,
                estimate,
                start_offset: offset,
                wait_after: wait,
            });
            offset += wait;
        }

        ZoneSchedule {
            zone: zone.name.clone(),
            material: zone.material.clone(),
            layers: scheduled,
            span: offset,
        }
    }

    fn advice(zones: &[ZoneSchedule], sequential: Duration, parallel: Duration) -> Vec<String> {
        if zones.is_empty() {
            return Vec::new();
        }
        let mut out = Vec::new();
        if sequential > parallel {
            out.push(
                "Menez les zones de front plutôt que l'une après l'autre : pendant qu'une couche sèche, les autres zones avancent. Il n'y a rien à gagner a attendre."
                    .to_string(),
            );
        }
        if zones.len() > 1
            && let Some(longest) = zones.iter().max_by_key(|z| z.span)
        {
            out.push(format!(
                "La zone {} commande le planning a elle seule : commencez par elle.",
                longest.zone
            ));
        }
        for z in zones.iter().filter(|z| z.slowest() >= DryingClass::Slow) {
            out.push(format!(
                "{} contient un pigment à séchage {} : c'est lui qui impose les délais de cette zone.",
                z.zone,
                z.slowest().label(Locale::Fr).to_lowercase()
            ));
        }
        out
    }
}

/// Rebuilds a project's painting plan with recomputed mix ratios.
#[derive(Debug, Clone, Default)]
pub struct ProjectPlanner {
    mixer: ColorMixService,
}

impl ProjectPlanner {
    const MAX_PAINTS: usize = 3;

    pub fn new(mixer: ColorMixService) -> Self {
        Self { mixer }
    }

    pub fn plan(&self, project: &Project) -> PaintingPlan {
        let paints = project.effective_paints();
        let zones = project
            .zones
            .iter()
            .map(|z| self.rebuild(z, paints))
            .collect();
        let palette_name = project
            .palette
            .as_ref()
            .map(|p| p.name.clone())
            .unwrap_or_else(|| format!("{} tubes conserves avec le projet", paints.len()));

        PaintingPlan {
            subject: project.subject.clone(),
            palette_name,
            approach: project.approach.clone(),
            system: project.system,
            zones,
        }
    }

    fn rebuild(&self, zone: &ProjectZone, paints: &[Paint]) -> PlanZone {
        let ladder: Vec<PlanLayer> = zone
            .layers
            .iter()
            .filter(|l| l.kind == LayerKind::Ladder)
            .map(|l| self.recompute(l, paints))
            .collect();
        let accents: Vec<PlanLayer> = zone
            .layers
            .iter()
            .filter(|l| l.kind == LayerKind::Accent)
            .map(|l| self.recompute(l, paints))
            .collect();

        let mut shadows: Vec<PlanLayer> = ladder
            .iter()
            .filter(|l| {
                let low = l.role.to_lowercase();
                low.starts_with("ombre") || low.starts_with("shadow")
            })
            .cloned()
            .collect();
        shadows.reverse();

        let highlights: Vec<PlanLayer> = ladder
            .iter()
            .filter(|l| {
                let low = l.role.to_lowercase();
                low.starts_with("lumiere")
                    || low.starts_with("lumière")
                    || low.starts_with("highlight")
            })
            .cloned()
            .collect();

        let base = ladder
            .iter()
            .find(|l| l.role.eq_ignore_ascii_case("base"))
            .cloned()
            .or_else(|| ladder.get(ladder.len() / 2).cloned());

        PlanZone {
            name: zone.name.clone(),
            material: zone.material.clone(),
            note: zone.note.clone(),
            base,
            shadows,
            highlights,
            accents,
        }
    }

    pub fn recompute(&self, layer: &ProjectLayer, paints: &[Paint]) -> PlanLayer {
        let target = layer.target();
        let found = if paints.is_empty() {
            Vec::new()
        } else {
            self.mixer
                .suggest_mixes(target, paints, 1, Some(Self::MAX_PAINTS))
        };

        let Some(best) = found.into_iter().next() else {
            return PlanLayer {
                role: layer.role.clone(),
                target,
                technique: layer.technique,
                note: layer.note.clone(),
                recipe: None,
                achieved: target,
                delta_e: 0.0,
            };
        };

        let achieved = best.color;
        let delta_e = delta_e_2000(target, achieved);
        PlanLayer {
            role: layer.role.clone(),
            target,
            technique: layer.technique,
            note: layer.note.clone(),
            recipe: Some(best),
            achieved,
            delta_e,
        }
    }
}

/// Builds a painting plan from colors sampled on a reference image.
#[derive(Debug, Clone, Default)]
pub struct SampledPlanService {
    mixer: ColorMixService,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColourGroup {
    pub name: String,
    pub material: String,
    pub samples: Vec<Rgb>,
}

impl SampledPlanService {
    pub fn new(mixer: ColorMixService) -> Self {
        Self { mixer }
    }

    pub fn plan(
        &self,
        subject: &str,
        palette: Option<&Palette>,
        groups: &[ColourGroup],
        max_paints: usize,
    ) -> DomainResult<PaintingPlan> {
        let usable: Vec<&ColourGroup> = groups.iter().filter(|g| !g.samples.is_empty()).collect();
        if usable.is_empty() {
            return Err(DomainError::InvalidArgument(
                "Aucun point relevé : ajoutez au moins une couleur à un groupe.".to_string(),
            ));
        }

        let total_samples: usize = usable.iter().map(|g| g.samples.len()).sum();
        let zones = usable
            .into_iter()
            .map(|g| self.build_zone(g, palette, max_paints))
            .collect();

        Ok(PaintingPlan {
            subject: subject.to_string(),
            palette_name: palette
                .map(|p| p.name.clone())
                .unwrap_or_else(|| "sans palette".to_string()),
            approach: format!(
                "Plan établi à partir de {total_samples} points relevés sur la référence."
            ),
            system: palette.map(|p| p.system()).unwrap_or_default(),
            zones,
        })
    }

    fn build_zone(
        &self,
        group: &ColourGroup,
        palette: Option<&Palette>,
        max_paints: usize,
    ) -> PlanZone {
        let mut ordered = group.samples.clone();
        ordered.sort_by(|a, b| a.relative_luminance().total_cmp(&b.relative_luminance()));

        let base_index = ordered.len() / 2;
        let system = palette.map(|p| p.system()).unwrap_or_default();
        let (shadow_technique, light_technique) = match system {
            PaintSystem::Oil => (Technique::Glaze, Technique::Highlight),
            PaintSystem::Acrylic => (Technique::ShadeWash, Technique::Layering),
        };
        let shadows: Vec<PlanLayer> = (0..base_index)
            .rev()
            .enumerate()
            .map(|(rank, idx)| {
                self.make_layer(
                    format!("Ombre {}", rank + 1),
                    ordered[idx],
                    shadow_technique,
                    palette,
                    max_paints,
                )
            })
            .collect();

        let highlights: Vec<PlanLayer> = ((base_index + 1)..ordered.len())
            .enumerate()
            .map(|(rank, idx)| {
                self.make_layer(
                    format!("Lumière {}", rank + 1),
                    ordered[idx],
                    light_technique,
                    palette,
                    max_paints,
                )
            })
            .collect();

        PlanZone {
            name: group.name.clone(),
            material: group.material.clone(),
            note: Self::note_for(ordered.len()),
            base: Some(self.make_layer(
                "Base".to_string(),
                ordered[base_index],
                Technique::BaseLayer,
                palette,
                max_paints,
            )),
            shadows,
            highlights,
            accents: Vec::new(),
        }
    }

    fn note_for(sample_count: usize) -> String {
        match sample_count {
            1 => "Un seul point relevé : la valeur locale, sans modèle. Relevez une ombre et une lumière pour obtenir un dégradé.".to_string(),
            2 => "Deux points : une base et un écart. Un troisième de l'autre côté donnerait un vrai dégradé.".to_string(),
            n => format!("{n} points releves sur la reference."),
        }
    }

    fn make_layer(
        &self,
        role: String,
        target: Rgb,
        technique: Technique,
        palette: Option<&Palette>,
        max_paints: usize,
    ) -> PlanLayer {
        let paints = palette.map(|p| p.paints.as_slice()).unwrap_or(&[]);
        let best = if paints.is_empty() {
            None
        } else {
            self.mixer
                .suggest_mixes(target, paints, 1, Some(max_paints))
                .into_iter()
                .next()
        };

        match best {
            None => PlanLayer {
                role,
                target,
                technique,
                note: String::new(),
                recipe: None,
                achieved: target,
                delta_e: 0.0,
            },
            Some(sugg) => {
                let achieved = sugg.color;
                let delta_e = delta_e_2000(target, achieved);
                PlanLayer {
                    role,
                    target,
                    technique,
                    note: String::new(),
                    recipe: Some(sugg),
                    achieved,
                    delta_e,
                }
            }
        }
    }
}
