//! Recipes: the saved ones, a new one, and the selected one in detail.

use super::recipe_detail::{RecipeDetail, StepDraft};
use crate::actions::recipes::{create_recipe, delete_recipe};
use crate::state::{AppServices, AppState};
use crate::theme::SectionHeader;
use dioxus::prelude::*;
use palettier_domain::paint::{PaintSystem, Technique};
use palettier_domain::recipe::Recipe;

#[component]
pub fn RecipesView() -> Element {
    let state = use_context::<AppState>();

    let locale = *state.locale.read();

    let selected_idx = use_signal(|| 0usize);
    let new_name = use_signal(String::new);
    let new_subject = use_signal(String::new);
    let draft = StepDraft {
        technique: use_signal(|| Technique::Glaze),
        mix: use_signal(String::new),
        notes: use_signal(String::new),
    };

    let recipes = state.recipes.read().clone();
    let current = recipes.get(*selected_idx.read()).cloned();

    rsx! {
        div {
            SectionHeader {
                title: locale.tr("Recettes & séquences d'étapes", "Recipes & Step Sequences").to_string(),
                subtitle: match *state.paint_system.read() {
                    PaintSystem::Oil => locale.tr(
                        "Construisez vos enchaînements de couches à l'huile, vérifiez la règle du gras sur maigre et déroulez le calendrier jusqu'au vernis.",
                        "Build multi-step oil sequences, validate the fat-over-lean rule, and preview the timeline through final varnish.",
                    ),
                    PaintSystem::Acrylic => locale.tr(
                        "Construisez vos enchaînements de couches acryliques, avec ou sans finition à l'huile, et déroulez le calendrier jusqu'au vernis.",
                        "Build acrylic layer sequences, with or without an oil finish, and preview the timeline through final varnish.",
                    ),
                }
                .to_string(),
                actions: None,
            }

            div { class: "grid-2 u-gtc-1fr-2fr",
                div {
                    div { class: "card",
                        h3 { class: "card-title", {locale.tr("Recettes enregistrées", "Saved Recipes")}, " ({recipes.len()})" }
                        for (idx, rec) in recipes.into_iter().enumerate() {
                            RecipeItem { idx, recipe: rec, selected_idx }
                        }
                    }

                    NewRecipeCard { new_name, new_subject }

                }

                div {
                    if let Some(recipe) = current {
                        RecipeDetail { recipe, draft }
                    } else {
                        div { class: "card",
                            p { class: "muted", {locale.tr("Sélectionnez ou créez une recette pour visualiser sa chronologie de séchage.", "Select or create a recipe to view its drying timeline.")} }
                        }
                    }

                }
            }
        }
    }
}

/// One saved recipe: clicking selects it, its button deletes it.
#[component]
fn RecipeItem(idx: usize, recipe: Recipe, selected_idx: Signal<usize>) -> Element {
    let services = use_context::<AppServices>();
    let mut state = use_context::<AppState>();
    let locale = *state.locale.read();

    let is_sel = *selected_idx.read() == idx;
    let delete = move |id: i64, e: Event<MouseData>| {
        e.stop_propagation();
        let services = services.clone();
        spawn(async move {
            let loc = *state.locale.read();
            if delete_recipe(&services, id).await.is_ok() {
                selected_idx.set(0);
                state.reload_all(&services).await;
                state.notify(loc.tr("Recette supprimée.", "Recipe deleted."));
            }
        });
    };

    rsx! {
        div {
            class: "card-subtle row-between",
            style: if is_sel { "margin-bottom: 8px; border-color: var(--accent); cursor: pointer;" } else { "margin-bottom: 8px; cursor: pointer;" },
            onclick: move |_| selected_idx.set(idx),
            div {
                div { class: "u-fw-600", "{recipe.name}" }
                div { class: "muted u-fs-11p5", "{recipe.subject} · {recipe.steps.len()} ", {locale.tr("étape(s)", "step(s)")} }
            }
            if let Some(id) = recipe.id {
                button {
                    class: "btn-danger",
                    onclick: move |e| delete(id, e),
                    {locale.tr("Supprimer", "Delete")}
                }
            }
        }
    }
}

#[component]
fn NewRecipeCard(new_name: Signal<String>, new_subject: Signal<String>) -> Element {
    let services = use_context::<AppServices>();
    let mut state = use_context::<AppState>();
    let locale = *state.locale.read();

    let create = move |_| {
        let name = new_name.read().clone();
        let subject = new_subject.read().clone();
        let services = services.clone();
        spawn(async move {
            let loc = *state.locale.read();
            if let Ok(Some(_)) = create_recipe(&services, &name, &subject).await {
                new_name.set(String::new());
                new_subject.set(String::new());
                state.reload_all(&services).await;
                state.notify(loc.tr("Recette créée.", "Recipe created."));
            }
        });
    };

    rsx! {
        div { class: "card",
            h3 { class: "card-title", {locale.tr("Nouvelle recette", "New Recipe")} }
            div { class: "u-mb-8",
                label { class: "field-label", {locale.tr("Nom de la recette", "Recipe Name")} }
                input {
                    r#type: "text",
                    class: "u-w-100pct",
                    placeholder: locale.tr("Ex: Carnation chaude en 4 étapes", "e.g. Warm Skin Tone in 4 Steps"),
                    value: "{new_name}",
                    oninput: move |e| new_name.set(e.value()),
                }
            }
            div { class: "u-mb-12",
                label { class: "field-label", {locale.tr("Sujet / Matière", "Subject / Material")} }
                input {
                    r#type: "text",
                    class: "u-w-100pct",
                    placeholder: locale.tr("Ex: Visage, Cuir usé, Drapé rouge…", "e.g. Face, Worn Leather, Red Cloak…"),
                    value: "{new_subject}",
                    oninput: move |e| new_subject.set(e.value()),
                }
            }
            button {
                class: "btn-primary",
                onclick: create,
                {locale.tr("Créer la recette", "Create recipe")}
            }
        }
    }
}
