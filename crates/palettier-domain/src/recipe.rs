use crate::drying::{DryingContext, DryingEstimate, DryingTimeService, Workshop};
use crate::paint::{DryingClass, LayerThickness, Medium, Technique};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// A recipe step: a technique, a paint mix, and a dilution setting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecipeStep {
    pub id: Option<i64>,
    pub technique: Technique,
    pub paint_mix: String,
    pub medium: Medium,
    pub medium_ratio: f64,
    pub thickness: LayerThickness,
    pub notes: String,
}

impl RecipeStep {
    pub fn of(technique: Technique, paint_mix: impl Into<String>) -> Self {
        Self {
            id: None,
            technique,
            paint_mix: paint_mix.into(),
            medium: technique.default_medium(),
            medium_ratio: technique.default_ratio(),
            thickness: technique.typical_thickness(),
            notes: String::new(),
        }
    }
}

/// Ordered sequence of oil painting steps for a given subject.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Recipe {
    pub id: Option<i64>,
    pub name: String,
    pub subject: String,
    pub notes: String,
    pub steps: Vec<RecipeStep>,
}

impl Recipe {
    pub fn new(name: impl Into<String>, subject: impl Into<String>) -> Self {
        Self {
            id: None,
            name: name.into(),
            subject: subject.into(),
            notes: String::new(),
            steps: Vec::new(),
        }
    }

    pub fn with_step(mut self, step: RecipeStep) -> Self {
        self.steps.push(step);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimelineEntry {
    pub position: usize,
    pub step: RecipeStep,
    pub start_offset: Duration,
    pub drying: DryingEstimate,
    pub wait_after: Duration,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Timeline {
    pub recipe: Recipe,
    pub entries: Vec<TimelineEntry>,
    pub total_active_span: Duration,
    pub until_varnish: Duration,
}

/// Schedules a recipe over time.
#[derive(Debug, Clone, Default)]
pub struct RecipeTimelineService {
    drying_time_service: DryingTimeService,
}

impl RecipeTimelineService {
    pub fn new(drying_time_service: DryingTimeService) -> Self {
        Self {
            drying_time_service,
        }
    }

    pub fn plan(&self, recipe: &Recipe, drying_class: DryingClass, workshop: Workshop) -> Timeline {
        let mut entries = Vec::with_capacity(recipe.steps.len());
        let mut offset = Duration::ZERO;
        let mut last: Option<DryingEstimate> = None;

        for (i, step) in recipe.steps.iter().enumerate() {
            let context = DryingContext {
                drying_class,
                medium: step.medium,
                medium_ratio: step.medium_ratio,
                thickness: step.thickness,
                workshop,
            };
            let estimate = self.drying_time_service.estimate(&context);

            let wait = if i + 1 == recipe.steps.len() {
                Duration::ZERO
            } else if recipe.steps[i + 1].technique.requires_cured_base() {
                estimate.through_dry
            } else {
                estimate.recoat
            };

            entries.push(TimelineEntry {
                position: i + 1,
                step: step.clone(),
                start_offset: offset,
                drying: estimate.clone(),
                wait_after: wait,
            });
            offset += wait;
            last = Some(estimate);
        }

        let until_varnish = last
            .map(|l| offset + l.full_cure)
            .unwrap_or(Duration::ZERO);
        Timeline {
            recipe: recipe.clone(),
            entries,
            total_active_span: offset,
            until_varnish,
        }
    }
}
