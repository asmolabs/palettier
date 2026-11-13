//! Recipes: step sequences, checked for fat over lean and laid out until the varnish.

use crate::state::AppServices;
use palettier_domain::drying::Workshop;
use palettier_domain::error::DomainResult;
use palettier_domain::i18n::Locale;
use palettier_domain::paint::{DryingClass, PaintSystem, Technique};
use palettier_domain::port::RecipeRepository;
use palettier_domain::project::FatOverLeanRisk;
use palettier_domain::recipe::{Recipe, RecipeStep, Timeline};

/// What the painter typed in « Ajouter une étape à cette recette ».
#[derive(Debug, Clone)]
pub struct NewStep {
    pub technique: Technique,
    /// The system the painter is working in: a technique shared by both systems
    /// follows it, one that belongs to a single system imposes its own.
    pub system: PaintSystem,
    pub paint_mix: String,
    pub notes: String,
}

/// What a recipe risks and how long it takes.
#[derive(Debug, Clone)]
pub struct RecipeOutlook {
    pub risks: Vec<FatOverLeanRisk>,
    pub timeline: Timeline,
}

/// The techniques offered for a step: those of the system, plus the oil ones on top
/// of an acrylic recipe.
pub fn step_techniques(system: PaintSystem) -> Vec<Technique> {
    Technique::ALL
        .into_iter()
        .filter(|t| {
            t.fits(system)
                || (system == PaintSystem::Acrylic && t.native_system() == Some(PaintSystem::Oil))
        })
        .collect()
}

/// The fat-over-lean risks of a recipe and its timeline (medium drying pigments, in
/// the given workshop).
pub fn recipe_outlook(
    services: &AppServices,
    recipe: &Recipe,
    workshop: Workshop,
    locale: Locale,
) -> RecipeOutlook {
    RecipeOutlook {
        risks: services.fat_over_lean.inspect_recipe_i18n(recipe, locale),
        timeline: services.recipe_timeline.plan(recipe, DryingClass::Medium, workshop),
    }
}

/// Saves a new empty recipe. Returns `None` when the name is blank (nothing is saved).
pub async fn create_recipe(
    services: &AppServices,
    name: &str,
    subject: &str,
) -> DomainResult<Option<Recipe>> {
    let name = name.trim();
    if name.is_empty() {
        return Ok(None);
    }
    services
        .recipes
        .save(&Recipe::new(name, subject.trim()))
        .await
        .map(Some)
}

/// Appends a step to a recipe, with the medium and ratio its technique takes in its
/// system, and saves the recipe. Returns `None` when the mix is blank (nothing is saved).
pub async fn add_recipe_step(
    services: &AppServices,
    recipe: Recipe,
    step: NewStep,
) -> DomainResult<Option<Recipe>> {
    let mix = step.paint_mix.trim();
    if mix.is_empty() {
        return Ok(None);
    }
    let mut next = recipe;
    let mut new_step = RecipeStep::of_in(step.technique, step.system, mix);
    new_step.notes = step.notes.trim().to_string();
    next.steps.push(new_step);
    services.recipes.save(&next).await.map(Some)
}

/// Deletes a recipe.
pub async fn delete_recipe(services: &AppServices, id: i64) -> DomainResult<()> {
    services.recipes.delete(id).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn step(technique: Technique, system: PaintSystem, mix: &str) -> NewStep {
        NewStep {
            technique,
            system,
            paint_mix: mix.into(),
            notes: " Tirer vers les tempes ".into(),
        }
    }

    async fn find(svc: &AppServices, id: i64) -> Recipe {
        svc.recipes
            .all()
            .await
            .unwrap()
            .into_iter()
            .find(|r| r.id == Some(id))
            .unwrap()
    }

    #[tokio::test]
    async fn creates_a_recipe_and_adds_steps_in_their_system() {
        let svc = AppServices::in_memory().await.unwrap();
        let recipe = create_recipe(&svc, "  Carnation  ", " Visage ")
            .await
            .unwrap()
            .expect("a named recipe is saved");
        let id = recipe.id.unwrap();
        assert_eq!(recipe.name, "Carnation");
        assert_eq!(recipe.subject, "Visage");
        assert!(recipe.steps.is_empty());

        let recipe = add_recipe_step(&svc, recipe, step(Technique::Glaze, PaintSystem::Acrylic, " Ocre + Blanc "))
            .await
            .unwrap()
            .unwrap();
        let recipe = add_recipe_step(&svc, recipe, step(Technique::Glaze, PaintSystem::Oil, "Terre d'ombre"))
            .await
            .unwrap()
            .unwrap();
        // An oil-only technique on an acrylic recipe stays oil.
        let recipe = add_recipe_step(&svc, recipe, step(Technique::OilWash, PaintSystem::Acrylic, "Noir"))
            .await
            .unwrap()
            .unwrap();

        let saved = find(&svc, id).await;
        assert_eq!(saved.steps.len(), recipe.steps.len());
        assert_eq!(saved.steps.len(), 3);
        assert_eq!(saved.steps[0].paint_mix, "Ocre + Blanc");
        assert_eq!(saved.steps[0].notes, "Tirer vers les tempes");
        assert_eq!(saved.steps[0].system(), PaintSystem::Acrylic);
        assert_eq!(saved.steps[0].medium, Technique::Glaze.default_medium_in(PaintSystem::Acrylic));
        assert_eq!(saved.steps[1].system(), PaintSystem::Oil);
        assert_eq!(saved.steps[1].medium, Technique::Glaze.default_medium_in(PaintSystem::Oil));
        assert_eq!(saved.steps[2].system(), PaintSystem::Oil);
    }

    #[tokio::test]
    async fn blank_names_and_mixes_save_nothing() {
        let svc = AppServices::in_memory().await.unwrap();
        let before = svc.recipes.all().await.unwrap().len();
        assert!(create_recipe(&svc, "   ", "Visage").await.unwrap().is_none());
        assert_eq!(svc.recipes.all().await.unwrap().len(), before);

        let recipe = create_recipe(&svc, "Cuir", "").await.unwrap().unwrap();
        let id = recipe.id.unwrap();
        let untouched = add_recipe_step(&svc, recipe, step(Technique::Glaze, PaintSystem::Oil, "  "))
            .await
            .unwrap();
        assert!(untouched.is_none());
        assert!(find(&svc, id).await.steps.is_empty());
    }

    #[tokio::test]
    async fn deletes_a_recipe() {
        let svc = AppServices::in_memory().await.unwrap();
        let recipe = create_recipe(&svc, "Drapé", "Cape").await.unwrap().unwrap();
        let id = recipe.id.unwrap();
        delete_recipe(&svc, id).await.unwrap();
        assert!(svc.recipes.all().await.unwrap().iter().all(|r| r.id != Some(id)));
    }

    #[tokio::test]
    async fn an_acrylic_step_over_an_oil_step_is_flagged() {
        let svc = AppServices::in_memory().await.unwrap();
        let recipe = create_recipe(&svc, "Mixte", "Socle").await.unwrap().unwrap();
        let recipe = add_recipe_step(&svc, recipe, step(Technique::BaseLayer, PaintSystem::Acrylic, "Gris"))
            .await
            .unwrap()
            .unwrap();
        let recipe = add_recipe_step(&svc, recipe, step(Technique::OilWash, PaintSystem::Acrylic, "Brun"))
            .await
            .unwrap()
            .unwrap();
        let ws = Workshop::default();
        // Oil over acrylic is fine.
        assert!(recipe_outlook(&svc, &recipe, ws, Locale::Fr).risks.is_empty());

        let recipe = add_recipe_step(&svc, recipe, step(Technique::Layering, PaintSystem::Acrylic, "Ocre"))
            .await
            .unwrap()
            .unwrap();
        let outlook = recipe_outlook(&svc, &recipe, ws, Locale::Fr);
        assert_eq!(outlook.risks.len(), 1);
        assert!(outlook.risks[0].over.contains("Étape 3"), "{:?}", outlook.risks[0]);
        assert_eq!(outlook.timeline.entries.len(), 3);
    }

    #[test]
    fn an_acrylic_recipe_offers_oil_techniques_too() {
        let acrylic = step_techniques(PaintSystem::Acrylic);
        assert!(acrylic.contains(&Technique::OilWash));
        assert!(acrylic.contains(&Technique::Layering));
        let oil = step_techniques(PaintSystem::Oil);
        assert!(oil.contains(&Technique::OilWash));
        assert!(!oil.contains(&Technique::Layering));
    }
}
