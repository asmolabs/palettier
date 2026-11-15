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

    // The piece followed, by id: a project added or removed does not shift it.
    let selected_project_id = use_signal(|| None::<i64>);
    let image_name = use_signal(String::new);
    let image_data_url = use_signal(|| None::<String>);
    let study_data_url = use_signal(|| None::<String>);
    let study_mode = use_signal(|| StudyMode::Color);
    let decoded_image = use_signal(|| None::<PixelMap>);
    let zoom_pct = use_signal(|| 100u32);
    let sampled_hex = use_signal(|| None::<String>);

    let current_project: Option<Project> = selected_project_id()
        .and_then(|id| projects.iter().find(|p| p.id == Some(id)))
        .or_else(|| projects.first())
        .cloned();
    let current_id = current_project.as_ref().and_then(|p| p.id);

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
            HudHeader { projects, current_id, selected_project_id, on_close }

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
    current_id: Option<i64>,
    selected_project_id: Signal<Option<i64>>,
    on_close: EventHandler<()>,
) -> Element {
    let mut state = use_context::<AppState>();
    let services = use_context::<AppServices>();
    let locale = *state.locale.read();
    let theme = *state.theme.read();
    let workshop = *state.workshop.read();

    rsx! {
        header { class: "painter-hud-header",
            div { class: "row u-gap-16 u-ai-center",
                span { class: "u-fs-17 u-fw-750 u-ls-0p2 u-d-flex u-ai-center u-gap-8",
                    "🎯 ",
                    {locale.tr("HUD Établi Peintre", "Painter Studio HUD")}
                }

                if !projects.is_empty() {
                    div { class: "row u-gap-8 u-ai-center",
                        span { class: "muted u-fs-12", {locale.tr("Projet :", "Project:")} }
                        select {
                            class: "u-bg-bg-input u-c-text-main u-b-1-solid-border-main u-r-6 u-p-4-10 u-fs-12p5 u-fw-600",
                            title: locale.tr("Pièce suivie", "Piece followed"),
                            onchange: move |ev| {
                                if let Ok(id) = ev.value().parse::<i64>() {
                                    selected_project_id.set(Some(id));
                                }
                            },
                            for p in projects.iter() {
                                option {
                                    value: "{p.id.unwrap_or_default()}",
                                    selected: p.id == current_id,
                                    "{p.name}"
                                }
                            }
                        }
                    }
                } else {
                    span { class: "pill", {locale.tr("Aucun projet actif", "No active project")} }
                }
            }

            div { class: "row u-gap-10 u-ai-center",
                // Climate quick readout
                div { class: "pill u-fs-11p5",
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
                    class: "u-p-6-14 u-fw-700",
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
    let img_width = match *decoded_image.read() {
        Some(ref map) => format!("{}px", (map.width as f64 * zoom_pct() as f64 / 100.0).round()),
        None => format!("{zoom_pct}%"),
    };

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
            div { class: "row-between u-mb-12",
                h3 { class: "card-title u-m-0 u-d-flex u-ai-center u-gap-8",
                    "🖼️ ",
                    {locale.tr("Modèle & Décomposition des Valeurs", "Reference Model & Value Breakdown")}
                }

                div { class: "row u-gap-8",
                    button {
                        class: "btn-primary",
                        class: "u-p-4-10 u-fs-12",
                        onclick: open_photo,
                        "📂 ",
                        {locale.tr("Ouvrir photo…", "Open photo…")}
                    }

                    if decoded_image.read().is_some() {
                        div { class: "row u-gap-4",
                            button {
                                class: "btn",
                                onclick: move |_| zoom_pct.set((zoom_pct().saturating_sub(25)).max(25)),
                                "−"
                            }
                            span { class: "mono u-fs-11 u-as-center", "{zoom_pct}%" }
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
                div { class: "row u-gap-6 u-mb-12 u-fwrap-wrap u-bg-bg-subtle u-p-8 u-r-8 u-b-1-solid-border-subtle",
                    for mode in StudyMode::ALL {
                        {
                            let is_active = *study_mode.read() == mode;
                            let svc = services.clone();
                            rsx! {
                                button {
                                    class: if is_active { "btn-primary" } else { "btn" },
                                    class: "u-p-4-9 u-fs-11p5 u-fw-600",
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
                class: "u-flex-1 u-minh-280 u-d-flex u-ai-center u-jc-center u-bg-bg-subtle u-r-8 u-b-1-dashed-border-main u-o-auto u-pos-relative",
                if let Some(ref uri) = *study_data_url.read() {
                    img {
                        src: "{uri}",
                        // Sized in pixels from the photo, like the eyedropper, so that a
                        // click maps back to the pixel under the cursor.
                        style: "width: {img_width}; max-width: none; border-radius: 4px; cursor: crosshair; user-select: none; display: block; margin: auto;",
                        title: locale.tr("Cliquez pour prélever une teinte", "Click to sample color"),
                        onclick: move |ev: Event<MouseData>| {
                            if let Some(ref map) = *decoded_image.read() {
                                let at = ev.element_coordinates();
                                if let Some((_, rgb)) = eye::sample_click(map, at.x, at.y, zoom_pct(), 4) {
                                    let h = rgb.to_hex();
                                    sampled_hex.set(Some(h.clone()));
                                    state.mixer_target_hex.set(h.clone());
                                    state.last_sampled_hex.set(Some(h));
                                }
                            }
                        }
                    }
                } else {
                    div { class: "u-ta-center u-p-40 u-c-text-muted",
                        div { class: "u-fs-38 u-mb-10", "🎨" }
                        div { class: "u-fs-14 u-fw-600 u-c-text-main",
                            {locale.tr("Aucune photo de référence chargée", "No reference photo loaded")}
                        }
                        div { class: "u-fs-12 u-mt-4",
                            {locale.tr("Cliquez sur 'Ouvrir photo…' pour afficher votre modèle avec les 6 filtres de valeurs (Notan, Grisaille, Terminateur).", "Click 'Open photo…' to inspect your model with 6 visual study filters (Notan, Grisaille, Terminator).")}
                        }
                    }
                }
            }

            // Sampled Eyedropper Card
            if let Some(ref h) = *sampled_hex.read() {
                div { class: "card-subtle u-mt-12 u-d-flex u-ai-center u-jc-space-between",
                    div { class: "row u-gap-12 u-ai-center",
                        ColorSwatch { hex: h.clone(), size: 36 }
                        div {
                            div { class: "mono u-fw-700 u-fs-14", "{h}" }
                            div { class: "muted u-fs-11", {locale.tr("Échantillon prélevé", "Sampled tone")} }
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
            div { class: "row-between u-mb-12",
                h3 { class: "card-title u-m-0 u-d-flex u-ai-center u-gap-8",
                    "🖌️ ",
                    {locale.tr("Couches & Suivi de Séchage", "Layers & Drying Progress")}
                }
                if let Some(ref proj) = current_project {
                    span { class: "pill pill-accent", "{proj.name}" }
                }
            }

            if let Some(ref proj) = current_project {
                div { class: "u-d-flex u-fd-column u-gap-10 u-mb-16",
                    for (z_idx, zone) in proj.zones.iter().cloned().enumerate() {
                        HudZoneCard { project: proj.clone(), z_idx, zone }
                    }
                }
            } else {
                div { class: "muted u-p-20-0 u-ta-center",
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
            div { class: "row-between u-mb-6",
                strong { class: "u-fs-13p5", "{zone.name}" }
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
        div { class: "row-between u-p-8-0 u-bt-1-solid-border-subtle u-ai-center",
            div { class: "row u-gap-10 u-ai-center",
                ColorSwatch { hex: layer.target_hex.clone(), size: 28 }
                div {
                    div { class: "u-fw-600 u-fs-12p5", "{layer.role}" }
                    if !tech_text.is_empty() {
                        div { class: "muted u-fs-11", "{tech_text}" }
                    }
                }
            }

            div { class: "row u-gap-8 u-ai-center",
                if is_applied {
                    span { class: "pill pill-ok u-fs-11",
                        "✓ ",
                        {locale.tr("Posée", "Applied")}
                    }
                }

                button {
                    class: if is_applied { "btn" } else { "btn-primary" },
                    class: "u-p-4-10 u-fs-11p5",
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
        div { class: "u-mt-auto u-pt-14 u-bt-1-solid-border-main",
            div { class: "row-between u-mb-8",
                span { class: "u-fw-700 u-fs-12p5",
                    {locale.tr("Palette de travail & Sécurité pigmentaire", "Working Palette & Pigment Safety")}
                }
                span { class: "muted u-fs-11", "{eff_paints.len()} tubes" }
            }

            div { class: "u-d-flex u-fwrap-wrap u-gap-6 u-maxh-140 u-oy-auto",
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
                                class: "u-p-4-8 u-d-inline-flex u-ai-center u-gap-6 u-r-6",
                                ColorSwatch { hex: paint.hex_color.clone(), size: 16 }
                                span { class: "u-fs-11p5 u-fw-600", "{paint.name}" }
                                span {
                                    class: "pill {s_badge.0}",
                                    class: "u-p-1-5 u-fs-9p5",
                                    title: "{s_info.advice(locale)}",
                                    "{s_badge.1} {s_info.level.badge_label(locale)}"
                                }
                                span {
                                    class: "pill",
                                    class: "u-p-1-5 u-fs-9p5 u-op-0p8",
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
