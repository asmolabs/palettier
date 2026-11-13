//! The selected recipe: fat-over-lean check, timeline, and the form to add a step.

use super::format_duration;
use crate::actions::recipes::{NewStep, add_recipe_step, recipe_outlook, step_techniques};
use crate::state::{AppServices, AppState};
use dioxus::prelude::*;
use palettier_domain::paint::Technique;
use palettier_domain::project::FatOverLeanRisk;
use palettier_domain::recipe::{Recipe, Timeline};

/// The fields of the « Ajouter une étape » form. They live in the recipes screen, so
/// switching recipes keeps what was typed.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct StepDraft {
    pub technique: Signal<Technique>,
    pub mix: Signal<String>,
    pub notes: Signal<String>,
}

#[component]
pub(super) fn RecipeDetail(recipe: Recipe, draft: StepDraft) -> Element {
    let services = use_context::<AppServices>();
    let state = use_context::<AppState>();
    let locale = *state.locale.read();
    let ws = *state.workshop.read();

    let outlook = recipe_outlook(&services, &recipe, ws, locale);
    let timeline = outlook.timeline;

    rsx! {
        div { class: "card",
            div { class: "row-between u-mb-12",
                div {
                    h3 { class: "card-title u-mb-2", "{recipe.name}" }
                    div { class: "muted", {locale.tr("Sujet : ", "Subject: ")}, "{recipe.subject}" }
                }
                div { class: "row",
                    span { class: "pill pill-accent", {locale.tr("Durée active : ", "Active span: ")}, "{format_duration(timeline.total_active_span, locale)}" }
                    span { class: "pill", {locale.tr("Vernis après : ", "Varnish after: ")}, "{format_duration(timeline.until_varnish, locale)}" }
                }
            }

            FatOverLeanCheck { risks: outlook.risks }

            TimelineTable { timeline }

            AddStepForm { recipe, draft }
        }
    }
}

#[component]
fn FatOverLeanCheck(risks: Vec<FatOverLeanRisk>) -> Element {
    let state = use_context::<AppState>();
    let locale = *state.locale.read();

    rsx! {
        if risks.is_empty() {
            div { class: "card-subtle u-mb-14 u-bc-pill-ok-border",
                span { class: "pill pill-ok", {locale.tr("Gras sur maigre respecté", "Fat over lean respected")} }
                span { class: "u-ml-10 u-fs-12p5",
                    {locale.tr("L'enchaînement des médiums et épaisseurs ne présente pas de risque de gerçure.", "The sequence of mediums and coat thicknesses presents no cracking risk.")}
                }
            }
        } else {
            for risk in &risks {
                div { class: "card-subtle u-mb-12 u-bc-pill-danger-border",
                    div { class: "u-fw-600 u-c-danger",
                        {locale.tr("Alerte gras sur maigre", "Fat over lean warning")}, " ({risk.under} → {risk.over})"
                    }
                    div { class: "u-fs-12p5 u-mt-4", "{risk.explanation}" }
                }
            }
        }
    }
}

#[component]
fn TimelineTable(timeline: Timeline) -> Element {
    let state = use_context::<AppState>();
    let locale = *state.locale.read();

    rsx! {
        table { class: "data-table",
            thead {
                tr {
                    th { "#" }
                    th { "Technique" }
                    th { {locale.tr("Mélange", "Mix")} }
                    th { {locale.tr("Médium & Épaisseur", "Medium & Thickness")} }
                    th { {locale.tr("Départ", "Start")} }
                    th { {locale.tr("Attente avant suite", "Wait before next")} }
                }
            }
            tbody {
                for entry in &timeline.entries {
                    tr {
                        td { class: "mono", "{entry.position}" }
                        td {
                            span { class: "pill pill-accent", "{entry.step.technique.label(locale)}" }
                        }
                        td {
                            div { class: "u-fw-600", "{entry.step.paint_mix}" }
                            if !entry.step.notes.is_empty() {
                                div { class: "muted u-fs-11p5", "{entry.step.notes}" }
                            }
                        }
                        td {
                            "{entry.step.medium.label(locale)} ({entry.step.medium_ratio * 100.0:.0} %) · {entry.step.thickness.label(locale)}"
                        }
                        td { class: "mono", "T + {format_duration(entry.start_offset, locale)}" }
                        td { class: "mono", "{format_duration(entry.wait_after, locale)}" }
                    }
                }
            }
        }
    }
}

#[component]
fn AddStepForm(recipe: Recipe, draft: StepDraft) -> Element {
    let services = use_context::<AppServices>();
    let mut state = use_context::<AppState>();
    let locale = *state.locale.read();
    let StepDraft {
        mut technique,
        mut mix,
        mut notes,
    } = draft;

    let add = move |_| {
        let step = NewStep {
            technique: *technique.read(),
            system: *state.paint_system.read(),
            paint_mix: mix.read().clone(),
            notes: notes.read().clone(),
        };
        let recipe = recipe.clone();
        let services = services.clone();
        spawn(async move {
            let loc = *state.locale.read();
            if let Ok(Some(_)) = add_recipe_step(&services, recipe, step).await {
                mix.set(String::new());
                notes.set(String::new());
                state.reload_all(&services).await;
                state.notify(loc.tr("Étape ajoutée à la recette.", "Step added to recipe."));
            }
        });
    };

    rsx! {
        div { class: "card-subtle u-mt-16",
            h4 { class: "u-fs-13 u-mb-10", {locale.tr("Ajouter une étape à cette recette", "Add a step to this recipe")} }
            div { class: "grid-3 u-mb-10",
                div {
                    label { class: "field-label", "Technique" }
                    select {
                        class: "u-w-100pct",
                        onchange: move |e| {
                            technique.set(Technique::by_label(Some(&e.value()), Technique::Glaze));
                        },
                        for t in step_techniques(*state.paint_system.read()) {
                            option {
                                value: "{t.as_db_str()}",
                                selected: *technique.read() == t,
                                "{t.label(locale)}"
                            }
                        }
                    }
                }
                div {
                    label { class: "field-label", {locale.tr("Mélange de tubes", "Tube Mix")} }
                    input {
                        r#type: "text",
                        class: "u-w-100pct",
                        placeholder: locale.tr("Ex: Ocre jaune + Blanc de titane", "e.g. Yellow Ochre + Titanium White"),
                        value: "{mix}",
                        oninput: move |e| mix.set(e.value()),
                    }
                }
                div {
                    label { class: "field-label", "Note" }
                    input {
                        r#type: "text",
                        class: "u-w-100pct",
                        placeholder: locale.tr("Ex: Tirer vers les tempes", "e.g. Feather toward the temples"),
                        value: "{notes}",
                        oninput: move |e| notes.set(e.value()),
                    }
                }
            }
            button {
                class: "btn-primary",
                onclick: add,
                {locale.tr("Ajouter l'étape", "Add step")}
            }
        }
    }
}
