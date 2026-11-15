//! Today on the workbench: the studio climate, the pieces in progress and the mixes on the palette.

use super::format_duration;
use crate::actions::mixes::{self, NewMix};
use crate::state::{AppServices, AppState, Section};
use crate::theme::{ColorSwatch, HexColorInput, SectionHeader};
use chrono::Utc;
use dioxus::prelude::*;
use palettier_domain::drying::Workshop;
use palettier_domain::paint::{DryingClass, PaintSystem, Ventilation};
use palettier_domain::palette::MixState;
use palettier_domain::workbench::PieceState;
use std::str::FromStr;

#[component]
pub fn TodayView() -> Element {
    let mut state = use_context::<AppState>();
    let locale = *state.locale.read();

    rsx! {
        div {
            SectionHeader {
                title: locale.tr("Aujourd'hui à l'établi", "Today on the Workbench").to_string(),
                subtitle: locale.tr(
                    "Pièces reprenables maintenant, couches en séchage et mélanges ouverts sur la palette.",
                    "Pieces ready to recoat now, drying layers, and open mixes on your palette."
                ).to_string(),
                actions: Some(rsx! {
                    button {
                        class: "btn",
                        onclick: move |_| state.section.set(Section::Projects),
                        {locale.tr("Voir tous les projets →", "View all projects →")}
                    }
                })
            }

            ClimateCard {}

            div { class: "grid-2",
                // Left column: Pieces on the workbench
                div {
                    BenchCard {}
                }

                // Right column: Fresh mixes on the palette
                div {
                    PaletteMixesCard {}
                }
            }
        }
    }
}

/// Temperature, humidity and ventilation of the studio, used for new coats and mixes.
#[component]
fn ClimateCard() -> Element {
    let mut state = use_context::<AppState>();
    let locale = *state.locale.read();
    let workshop = *state.workshop.read();

    rsx! {
        div { class: "card",
            div { class: "row-between",
                div {
                    div { class: "card-title u-mb-2",
                        {locale.tr("Conditions actuelles de l'atelier", "Current Studio Climate")}
                    }
                    span { class: "muted",
                        {locale.tr("Appliquées aux nouvelles couches posées et aux mélanges préparés.", "Applied to newly recorded coats and fresh palette mixes.")}
                    }
                }
                div { class: "row",
                    div {
                        label { class: "field-label", {locale.tr("Température (°C)", "Temperature (°C)")} }
                        input {
                            r#type: "number",
                            class: "u-w-85",
                            value: "{workshop.temperature_celsius}",
                            oninput: move |ev| {
                                if let Ok(v) = ev.value().parse::<f64>()
                                    && let Ok(w) = Workshop::new(v, workshop.relative_humidity, workshop.ventilation) {
                                        state.workshop.set(w);
                                    }
                            }
                        }
                    }
                    div {
                        label { class: "field-label", {locale.tr("Humidité (%)", "Humidity (%)")} }
                        input {
                            r#type: "number",
                            class: "u-w-85",
                            value: "{workshop.relative_humidity}",
                            oninput: move |ev| {
                                if let Ok(v) = ev.value().parse::<f64>()
                                    && let Ok(w) = Workshop::new(workshop.temperature_celsius, v, workshop.ventilation) {
                                        state.workshop.set(w);
                                    }
                            }
                        }
                    }
                    div {
                        label { class: "field-label", "Ventilation" }
                        select {
                            value: "{workshop.ventilation.as_db_str()}",
                            onchange: move |ev| {
                                if let Ok(vent) = Ventilation::from_str(&ev.value())
                                    && let Ok(w) = Workshop::new(workshop.temperature_celsius, workshop.relative_humidity, vent) {
                                        state.workshop.set(w);
                                    }
                            },
                            for v in Ventilation::ALL {
                                option { value: "{v.as_db_str()}", "{v.label(locale)}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// The pieces in progress, zone by zone, with what can be recoated now.
#[component]
fn BenchCard() -> Element {
    let state = use_context::<AppState>();
    let services = use_context::<AppServices>();
    let locale = *state.locale.read();
    let projects = state.projects.read().clone();
    let bench = services.workbench.bench(&projects, Utc::now());

    rsx! {
        div { class: "card",
            div { class: "row-between u-mb-12",
                h2 { class: "card-title u-m-0",
                    {locale.tr("Pièces en cours", "Active Pieces")}, " ({bench.pieces.len()})"
                }
                span { class: "pill pill-ok",
                    "{bench.ready().len()} ", {locale.tr("prêtes", "ready")}
                }
            }
            if bench.pieces.is_empty() {
                p { class: "muted",
                    {locale.tr("Aucun projet actif. Créez un projet ou demandez un plan à l'Assistant IA.", "No active projects. Create a project or ask the AI Assistant for a painting plan.")}
                }
            } else {
                for piece in bench.pieces {
                    BenchPiece { piece }
                }
            }
        }
    }
}

#[component]
fn BenchPiece(piece: PieceState) -> Element {
    let state = use_context::<AppState>();
    let locale = *state.locale.read();

    rsx! {
        div { class: "card-subtle u-mb-10",
            div { class: "row-between",
                div { class: "row u-gap-6",
                    strong { "{piece.project.name}" }
                    span {
                        class: if piece.project.system == PaintSystem::Acrylic { "pill pill-acrylic" } else { "pill pill-oil" },
                        "{piece.project.system.label(locale)}"
                    }
                }
                span { class: "pill",
                    "{piece.applied_coats()}/{piece.total_coats()} ", {locale.tr("couches", "coats")}
                }
            }
            if !piece.project.subject.is_empty() {
                div { class: "muted u-fs-12 u-mb-6", "{piece.project.subject}" }
            }
            for z in &piece.zones {
                div { class: "row-between u-p-4-0 u-bt-1-solid-border-subtle",
                    div { class: "row",
                        span { "{z.name}" }
                        span { class: "muted", "({z.applied}/{z.total})" }
                    }
                    if z.done() {
                        span { class: "pill pill-ok", {locale.tr("Terminée", "Completed")} }
                    } else if z.ready_now() {
                        span { class: "pill pill-accent", {locale.tr("Reprenable maintenant", "Ready to recoat now")} }
                    } else if let Some(ref last) = z.last {
                        span {
                            class: "pill pill-warn",
                            "{last.stage.label(locale)} · ", {locale.tr("suite dans", "next in")}, " {format_duration(z.remaining(), locale)}"
                        }
                    }
                }
            }
        }
    }
}

/// The mixes laid on the palette, with the form to lay a new one.
#[component]
fn PaletteMixesCard() -> Element {
    let state = use_context::<AppState>();
    let services = use_context::<AppServices>();
    let locale = *state.locale.read();
    let mixes = state.mixes.read().clone();
    let mix_states = services.palette_mix_service.states(&mixes, Utc::now());

    rsx! {
        div { class: "card",
            h2 { class: "card-title",
                {locale.tr("Mélanges posés sur la palette", "Mixes on the Palette")}, " ({mix_states.len()})"
            }
            NewMixForm {}

            if mix_states.is_empty() {
                p { class: "muted", {locale.tr("Aucun mélange suivi actuellement.", "No palette mixes currently tracked.")} }
            } else {
                for ms in mix_states {
                    MixRow { ms }
                }
            }
        }
    }
}

#[component]
fn NewMixForm() -> Element {
    let mut state = use_context::<AppState>();
    let services = use_context::<AppServices>();
    let locale = *state.locale.read();

    let mut new_mix_name = use_signal(String::new);
    let mut new_mix_hex = use_signal(|| "#8C6248".to_string());
    let mut new_mix_recipe = use_signal(String::new);
    let mut new_mix_drying = use_signal(|| DryingClass::Medium);

    let place = move |_| {
        let mix = NewMix {
            name: new_mix_name(),
            hex_color: new_mix_hex(),
            recipe: new_mix_recipe(),
            drying_class: new_mix_drying(),
            system: *state.paint_system.peek(),
            workshop: *state.workshop.read(),
        };
        if mix.name.trim().is_empty() {
            return;
        }
        let svc = services.clone();
        spawn(async move {
            if let Ok(Some(all)) = mixes::place_mix(&svc, mix).await {
                state.mixes.set(all);
                new_mix_name.set(String::new());
                new_mix_recipe.set(String::new());
            }
        });
    };

    rsx! {
        div { class: "card-subtle u-mb-12",
            div { class: "grid-2 u-gap-8 u-mb-8",
                div {
                    label { class: "field-label", {locale.tr("Nom du mélange", "Mix Name")} }
                    input {
                        r#type: "text",
                        class: "u-w-100pct",
                        placeholder: locale.tr("ex. Chair d'ombre chaude", "e.g. Warm Shadow Flesh"),
                        value: "{new_mix_name}",
                        oninput: move |ev| new_mix_name.set(ev.value())
                    }
                }
                div {
                    label { class: "field-label", {locale.tr("Couleur (#RRGGBB)", "Color (#RRGGBB)")} }
                    HexColorInput {
                        value: new_mix_hex(),
                        onchange: move |v: String| new_mix_hex.set(v),
                    }
                }
            }
            div { class: "grid-2 u-gap-8 u-mb-8",
                div {
                    label { class: "field-label", {locale.tr("Recette", "Recipe")} }
                    input {
                        r#type: "text",
                        class: "u-w-100pct",
                        placeholder: locale.tr("2 parts Ocre + 1 part Blanc", "2 parts Ochre + 1 part White"),
                        value: "{new_mix_recipe}",
                        oninput: move |ev| new_mix_recipe.set(ev.value())
                    }
                }
                div {
                    label { class: "field-label", {locale.tr("Vitesse du pigment le plus lent", "Slowest Pigment Speed")} }
                    select {
                        class: "u-w-100pct",
                        value: "{new_mix_drying().as_db_str()}",
                        onchange: move |ev| {
                            if let Ok(dc) = DryingClass::from_str(&ev.value()) {
                                new_mix_drying.set(dc);
                            }
                        },
                        for dc in DryingClass::ALL {
                            option { value: "{dc.as_db_str()}", "{dc.label(locale)}" }
                        }
                    }
                }
            }
            button {
                class: "btn-primary",
                onclick: place,
                {locale.tr("+ Poser sur la palette", "+ Place on palette")}
            }
        }
    }
}

#[component]
fn MixRow(ms: MixState) -> Element {
    let mut state = use_context::<AppState>();
    let services = use_context::<AppServices>();
    let locale = *state.locale.read();

    let remove = move |id: i64| {
        let svc = services.clone();
        spawn(async move {
            if let Ok(all) = mixes::remove_mix(&svc, id).await {
                state.mixes.set(all);
            }
        });
    };

    rsx! {
        div { class: "card-subtle u-mb-8",
            div { class: "row-between",
                div { class: "row",
                    ColorSwatch { hex: ms.mix.hex_color.clone(), size: 24 }
                    div {
                        strong { "{ms.mix.name}" }
                        div { class: "muted u-fs-11p5", "{ms.mix.recipe}" }
                    }
                }
                div { class: "row",
                    if ms.is_open() {
                        span {
                            class: "pill pill-ok",
                            {locale.tr("Ouvert", "Open")}, " ({format_duration(ms.workable, locale)})"
                        }
                    } else if ms.is_spent() {
                        span { class: "pill pill-danger", {locale.tr("Figé / pris", "Set / cured")} }
                    } else {
                        span { class: "pill pill-warn", {locale.tr("En prise", "Setting")} }
                    }
                    if let Some(id) = ms.mix.id {
                        button {
                            class: "btn-danger",
                            onclick: move |_| remove(id),
                            {locale.tr("Retirer", "Remove")}
                        }
                    }
                }
            }
        }
    }
}
