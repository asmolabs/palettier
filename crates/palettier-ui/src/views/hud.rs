#![forbid(unsafe_code)]

use crate::actions::eyedropper as eye;
use crate::actions::projects::{save_project, toggle_layer_applied};
use crate::actions::settings::{save_locale, save_theme};
use crate::views::color::show_study;
use crate::state::{AppServices, AppState};
use crate::theme::ColorSwatch;
use chrono::Utc;
use dioxus::prelude::*;
use palettier_domain::image::{PixelMap, StudyMode};
use palettier_domain::paint::{Paint, SafetyLevel};
use palettier_domain::project::{Project, ProjectLayer, ProjectZone};

#[component]
pub fn PainterHudView(on_close: EventHandler<()>) -> Element {
    let state = use_context::<AppState>();

    let theme = *state.theme.read();
    let projects = state.projects.read().clone();
    let palettes = state.palettes.read().clone();

    let selected_project_idx = use_signal(|| 0usize);
    let image_name = use_signal(String::new);
    let image_data_url = use_signal(|| None::<String>);
    let study_data_url = use_signal(|| None::<String>);
    let study_mode = use_signal(|| StudyMode::Color);
    let decoded_image = use_signal(|| None::<PixelMap>);
    let zoom_pct = use_signal(|| 100u32);
    let sampled_hex = use_signal(|| None::<String>);

    let current_project: Option<Project> = projects.get(selected_project_idx()).cloned();

    let eff_paints: Vec<Paint> = current_project
        .as_ref()
        .map(|p| p.effective_paints().to_vec())
        .unwrap_or_else(|| {
            palettes
                .first()
                .map(|p| p.paints.clone())
                .unwrap_or_default()
        });

    rsx! {
        div { class: "painter-hud {theme.css_class()}",
            // HUD Top Navigation Bar
            HudHeader { projects, selected_project_idx, on_close }

            // HUD Two-Column Workspace
            div { class: "painter-hud-body",
                // Left Pane: Reference Image & Academic Value Study Breakdown
                HudReferencePane {
                    image_name,
                    image_data_url,
                    study_data_url,
                    study_mode,
                    decoded_image,
                    zoom_pct,
                    sampled_hex,
                    on_close,
                }

                // Right Pane: Active Coats / Drying Layers & Palette Tubes
                HudCoatsPane { current_project, eff_paints }
            }
        }
    }
}

/// The HUD's bar: the piece followed, the climate, theme and language, and the way out.
#[component]
fn HudHeader(
    projects: Vec<Project>,
    selected_project_idx: Signal<usize>,
    on_close: EventHandler<()>,
) -> Element {
    let mut state = use_context::<AppState>();
    let services = use_context::<AppServices>();
    let locale = *state.locale.read();
    let theme = *state.theme.read();
    let workshop = *state.workshop.read();

    rsx! {
        header { class: "painter-hud-header",
            div { class: "row", style: "gap: 16px; align-items: center;",
                span { style: "font-size: 17px; font-weight: 750; letter-spacing: -0.2px; display: flex; align-items: center; gap: 8px;",
                    "🎯 ",
                    {locale.tr("HUD Établi Peintre", "Painter Studio HUD")}
                }

                if !projects.is_empty() {
                    div { class: "row", style: "gap: 8px; align-items: center;",
                        span { class: "muted", style: "font-size: 12px;", {locale.tr("Projet :", "Project:")} }
                        select {
                            style: "background: var(--bg-input); color: var(--text-main); border: 1px solid var(--border-main); border-radius: 6px; padding: 4px 10px; font-size: 12.5px; font-weight: 600;",
                            value: "{selected_project_idx}",
                            onchange: move |ev| {
                                if let Ok(idx) = ev.value().parse::<usize>() {
                                    selected_project_idx.set(idx);
                                }
                            },
                            for (idx, p) in projects.iter().enumerate() {
                                option { value: "{idx}", "{p.name}" }
                            }
                        }
                    }
                } else {
                    span { class: "pill", {locale.tr("Aucun projet actif", "No active project")} }
                }
            }

            div { class: "row", style: "gap: 10px; align-items: center;",
                // Climate quick readout
                div { class: "pill", style: "font-size: 11.5px;",
                    "🌡️ {workshop.temperature_celsius:.0}°C · 💧 {workshop.relative_humidity:.0}% · {workshop.ventilation.label(locale)}"
                }

                // Theme toggle
                button {
                    class: "toggle-chip",
                    onclick: {
                        let svc = services.clone();
                        move |_| {
                            let next = state.theme.read().toggle();
                            state.theme.set(next);
                            let svc = svc.clone();
                            spawn(async move {
                                let _ = save_theme(&svc, next).await;
                            });
                        }
                    },
                    "{theme.button_label(locale)}"
                }

                // Language toggle
                button {
                    class: "toggle-chip",
                    onclick: {
                        let svc = services.clone();
                        move |_| {
                            let next = state.locale.read().toggle();
                            state.locale.set(next);
                            let svc = svc.clone();
                            spawn(async move {
                                let _ = save_locale(&svc, next).await;
                            });
                        }
                    },
                    "🌐 {locale.badge()} → {locale.toggle().badge()}"
                }

                // Exit button
                button {
                    class: "btn-primary",
                    style: "padding: 6px 14px; font-weight: 700;",
                    onclick: move |_| on_close.call(()),
                    "✕ ",
                    {locale.tr("Quitter HUD", "Exit HUD")}
                }
            }
        }
    }
}

/// The reference photo with its value studies, and the tone sampled from it.
#[component]
fn HudReferencePane(
    image_name: Signal<String>,
    image_data_url: Signal<Option<String>>,
    study_data_url: Signal<Option<String>>,
    study_mode: Signal<StudyMode>,
    decoded_image: Signal<Option<PixelMap>>,
    zoom_pct: Signal<u32>,
    sampled_hex: Signal<Option<String>>,
    on_close: EventHandler<()>,
) -> Element {
    let mut state = use_context::<AppState>();
    let services = use_context::<AppServices>();
    let locale = *state.locale.read();

    let open_photo = {
        let svc = services.clone();
        move |_: Event<MouseData>| {
            let svc = svc.clone();
            spawn(async move {
                if let Some(file) = rfd::AsyncFileDialog::new()
                    .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
                    .pick_file()
                    .await
                    && let Ok(bytes) = std::fs::read(file.path())
                    && let Some(photo) = eye::open_reference(&svc, &bytes)
                {
                    image_data_url.set(Some(photo.data_uri.clone()));
                    study_data_url.set(Some(photo.data_uri));
                    study_mode.set(StudyMode::Color);
                    decoded_image.set(Some(photo.pixels));
                    image_name.set(file.file_name());
                }
            });
        }
    };

    rsx! {
        div { class: "painter-hud-pane",
            div { class: "row-between", style: "margin-bottom: 12px;",
                h3 { class: "card-title", style: "margin: 0; display: flex; align-items: center; gap: 8px;",
                    "🖼️ ",
                    {locale.tr("Modèle & Décomposition des Valeurs", "Reference Model & Value Breakdown")}
                }

                div { class: "row", style: "gap: 8px;",
                    button {
                        class: "btn-primary",
                        style: "padding: 4px 10px; font-size: 12px;",
                        onclick: open_photo,
                        "📂 ",
                        {locale.tr("Ouvrir photo…", "Open photo…")}
                    }

                    if decoded_image.read().is_some() {
                        div { class: "row", style: "gap: 4px;",
                            button {
                                class: "btn",
                                onclick: move |_| zoom_pct.set((zoom_pct().saturating_sub(25)).max(25)),
                                "−"
                            }
                            span { class: "mono", style: "font-size: 11px; align-self: center;", "{zoom_pct}%" }
                            button {
                                class: "btn",
                                onclick: move |_| zoom_pct.set((zoom_pct() + 25).min(300)),
                                "+"
                            }
                            button {
                                class: "btn",
                                onclick: move |_| zoom_pct.set(100),
                                "100%"
                            }
                        }
                    }
                }
            }

            // Study Mode Switcher Bar
            if decoded_image.read().is_some() {
                div { class: "row", style: "gap: 6px; margin-bottom: 12px; flex-wrap: wrap; background: var(--bg-subtle); padding: 8px; border-radius: 8px; border: 1px solid var(--border-subtle);",
                    for mode in StudyMode::ALL {
                        {
                            let is_active = *study_mode.read() == mode;
                            let svc = services.clone();
                            rsx! {
                                button {
                                    class: if is_active { "btn-primary" } else { "btn" },
                                    style: "padding: 4px 9px; font-size: 11.5px; font-weight: 600;",
                                    onclick: move |_| {
                                        show_study(&svc, mode, study_mode, image_data_url, study_data_url, decoded_image)
                                    },
                                    "{mode.label(locale)}"
                                }
                            }
                        }
                    }
                }
            }

            // Photo Viewer / Canvas
            div {
                style: "flex: 1; min-height: 280px; display: flex; align-items: center; justify-content: center; background: var(--bg-subtle); border-radius: 8px; border: 1px dashed var(--border-main); overflow: auto; position: relative;",
                if let Some(ref uri) = *study_data_url.read() {
                    img {
                        src: "{uri}",
                        style: "width: {zoom_pct}%; max-width: none; border-radius: 4px; cursor: crosshair; user-select: none; display: block; margin: auto;",
                        title: locale.tr("Cliquez pour prélever une teinte", "Click to sample color"),
                        onclick: move |_ev| {
                            if let Some(ref map) = *decoded_image.read() {
                                let cx = (map.width / 2) as isize;
                                let cy = (map.height / 2) as isize;
                                if let Some(rgb) = map.sample_around(cx, cy, 4) {
                                    let h = rgb.to_hex();
                                    sampled_hex.set(Some(h.clone()));
                                    state.mixer_target_hex.set(h.clone());
                                    state.last_sampled_hex.set(Some(h));
                                }
                            }
                        }
                    }
                } else {
                    div { style: "text-align: center; padding: 40px; color: var(--text-muted);",
                        div { style: "font-size: 38px; margin-bottom: 10px;", "🎨" }
                        div { style: "font-size: 14px; font-weight: 600; color: var(--text-main);",
                            {locale.tr("Aucune photo de référence chargée", "No reference photo loaded")}
                        }
                        div { style: "font-size: 12px; margin-top: 4px;",
                            {locale.tr("Cliquez sur 'Ouvrir photo…' pour afficher votre modèle avec les 6 filtres de valeurs (Notan, Grisaille, Terminateur).", "Click 'Open photo…' to inspect your model with 6 visual study filters (Notan, Grisaille, Terminator).")}
                        }
                    }
                }
            }

            // Sampled Eyedropper Card
            if let Some(ref h) = *sampled_hex.read() {
                div { class: "card-subtle", style: "margin-top: 12px; display: flex; align-items: center; justify-content: space-between;",
                    div { class: "row", style: "gap: 12px; align-items: center;",
                        ColorSwatch { hex: h.clone(), size: 36 }
                        div {
                            div { class: "mono", style: "font-weight: 700; font-size: 14px;", "{h}" }
                            div { class: "muted", style: "font-size: 11px;", {locale.tr("Échantillon prélevé", "Sampled tone")} }
                        }
                    }
                    button {
                        class: "btn",
                        onclick: {
                            let h = h.clone();
                            move |_| {
                                state.mixer_target_hex.set(h.clone());
                                state.section.set(crate::state::Section::Mixer);
                                on_close.call(());
                            }
                        },
                        "🧪 ",
                        {locale.tr("Ouvrir dans le Mélangeur", "Open in Mixer")}
                    }
                }
            }
        }
    }
}

/// The piece's coats zone by zone, to log as they are laid, and the working palette.
#[component]
fn HudCoatsPane(current_project: Option<Project>, eff_paints: Vec<Paint>) -> Element {
    let state = use_context::<AppState>();
    let locale = *state.locale.read();

    rsx! {
        div { class: "painter-hud-pane",
            div { class: "row-between", style: "margin-bottom: 12px;",
                h3 { class: "card-title", style: "margin: 0; display: flex; align-items: center; gap: 8px;",
                    "🖌️ ",
                    {locale.tr("Couches & Suivi de Séchage", "Layers & Drying Progress")}
                }
                if let Some(ref proj) = current_project {
                    span { class: "pill pill-accent", "{proj.name}" }
                }
            }

            if let Some(ref proj) = current_project {
                div { style: "display: flex; flex-direction: column; gap: 10px; margin-bottom: 16px;",
                    for (z_idx, zone) in proj.zones.iter().cloned().enumerate() {
                        HudZoneCard { project: proj.clone(), z_idx, zone }
                    }
                }
            } else {
                div { class: "muted", style: "padding: 20px 0; text-align: center;",
                    {locale.tr("Sélectionnez ou créez un projet pour suivre vos couches.", "Select or create a project to track your coats.")}
                }
            }

            // Active Working Palette & Safety
            HudPalette { eff_paints }
        }
    }
}

/// One zone and its coats.
#[component]
fn HudZoneCard(project: Project, z_idx: usize, zone: ProjectZone) -> Element {
    let state = use_context::<AppState>();
    let locale = *state.locale.read();
    let applied_count = zone.layers.iter().filter(|l| l.is_applied()).count();

    rsx! {
        div { class: "hud-mix-card",
            div { class: "row-between", style: "margin-bottom: 6px;",
                strong { style: "font-size: 13.5px;", "{zone.name}" }
                span { class: "pill", "{applied_count}/{zone.layers.len()} ", {locale.tr("couches", "coats")} }
            }

            for (l_idx, layer) in zone.layers.iter().cloned().enumerate() {
                HudCoatRow { project: project.clone(), z_idx, l_idx, layer }
            }
        }
    }
}

/// One coat: laid or not, and the button that logs it (or takes it back).
#[component]
fn HudCoatRow(project: Project, z_idx: usize, l_idx: usize, layer: ProjectLayer) -> Element {
    let mut state = use_context::<AppState>();
    let services = use_context::<AppServices>();
    let locale = *state.locale.read();
    let is_applied = layer.is_applied();
    let tech_text = layer.technique.label(locale).to_string();

    rsx! {
        div { class: "row-between", style: "padding: 8px 0; border-top: 1px solid var(--border-subtle); align-items: center;",
            div { class: "row", style: "gap: 10px; align-items: center;",
                ColorSwatch { hex: layer.target_hex.clone(), size: 28 }
                div {
                    div { style: "font-weight: 600; font-size: 12.5px;", "{layer.role}" }
                    if !tech_text.is_empty() {
                        div { class: "muted", style: "font-size: 11px;", "{tech_text}" }
                    }
                }
            }

            div { class: "row", style: "gap: 8px; align-items: center;",
                if is_applied {
                    span { class: "pill pill-ok", style: "font-size: 11px;",
                        "✓ ",
                        {locale.tr("Posée", "Applied")}
                    }
                }

                button {
                    class: if is_applied { "btn" } else { "btn-primary" },
                    style: "padding: 4px 10px; font-size: 11.5px;",
                    onclick: move |_| {
                        let w = *state.workshop.read();
                        let next = toggle_layer_applied(&project, z_idx, l_idx, w, Utc::now());
                        let svc = services.clone();
                        spawn(async move {
                            if let Ok(change) = save_project(&svc, &next).await {
                                state.projects.set(change.projects);
                            }
                        });
                    },
                    if is_applied {
                        {locale.tr("Annuler", "Undo")}
                    } else {
                        {locale.tr("Poser", "Log Coat")}
                    }
                }
            }
        }
    }
}

/// The tubes of the working palette with their safety and lightfastness.
#[component]
fn HudPalette(eff_paints: Vec<Paint>) -> Element {
    let state = use_context::<AppState>();
    let locale = *state.locale.read();

    rsx! {
        div { style: "margin-top: auto; padding-top: 14px; border-top: 1px solid var(--border-main);",
            div { class: "row-between", style: "margin-bottom: 8px;",
                span { style: "font-weight: 700; font-size: 12.5px;",
                    {locale.tr("Palette de travail & Sécurité pigmentaire", "Working Palette & Pigment Safety")}
                }
                span { class: "muted", style: "font-size: 11px;", "{eff_paints.len()} tubes" }
            }

            div { style: "display: flex; flex-wrap: wrap; gap: 6px; max-height: 140px; overflow-y: auto;",
                for paint in &eff_paints {
                    {
                        let s_info = paint.safety_info();
                        let s_badge = match s_info.level {
                            SafetyLevel::Safe => ("pill-ok", "✓"),
                            SafetyLevel::Caution => ("pill-warn", "⚠️"),
                            SafetyLevel::Hazard => ("pill-danger", "☠️"),
                        };
                        let lf = paint.lightfastness();
                        rsx! {
                            div {
                                class: "card-subtle",
                                style: "padding: 4px 8px; display: inline-flex; align-items: center; gap: 6px; border-radius: 6px;",
                                ColorSwatch { hex: paint.hex_color.clone(), size: 16 }
                                span { style: "font-size: 11.5px; font-weight: 600;", "{paint.name}" }
                                span {
                                    class: "pill {s_badge.0}",
                                    style: "padding: 1px 5px; font-size: 9.5px;",
                                    title: "{s_info.advice(locale)}",
                                    "{s_badge.1} {s_info.level.badge_label(locale)}"
                                }
                                span {
                                    class: "pill",
                                    style: "padding: 1px 5px; font-size: 9.5px; opacity: 0.8;",
                                    "{lf.label(locale)}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
