//! The eyedropper: the reference photo, its dominant colours, and (in
//! `eyedropper_groups`) the picked colour, the sampled groups and the pinned zones.

use super::eyedropper_groups::{ActiveColorCard, PinZoneCard, SampledGroupsCard};
use crate::actions::eyedropper as eye;
use crate::state::{AppServices, AppState, Section};
use crate::theme::{ColorSwatch, SectionHeader};
use dioxus::prelude::*;
use palettier_domain::i18n::Locale;
use palettier_domain::image::{PixelMap, StudyMode};
use palettier_domain::plan::ColourGroup;
use palettier_domain::project::DominantColour;

#[component]
pub fn EyedropperView() -> Element {
    let mut state = use_context::<AppState>();
    let services = use_context::<AppServices>();

    let locale = *state.locale.read();

    let mut image_name = use_signal(String::new);
    let mut image_data_url = use_signal(|| None::<String>);
    let mut study_data_url = use_signal(|| None::<String>);
    let mut study_mode = use_signal(|| StudyMode::Color);
    let mut decoded_image = use_signal(|| None::<PixelMap>);
    let zoom_pct = use_signal(|| 100u32);
    let sample_radius = use_signal(|| 3isize);
    let pick_neutral_mode = use_signal(|| false);
    let last_click_coords = use_signal(|| None::<(isize, isize)>);

    let mut dominants = use_signal(Vec::new);
    let grey_ref_hex = use_signal(|| "#808080".to_string());
    let has_custom_neutral = use_signal(|| false);
    let zone_name = use_signal(|| "Tunic".to_string());
    let zone_material = use_signal(|| "Cloth".to_string());
    let mut picked_hex = use_signal(|| "#7A5442".to_string());

    // Sampled groups for building a measured project without AI (SampledPlanService)
    let groups = use_signal(|| {
        vec![ColourGroup {
            name: "Visage / Face".to_string(),
            material: "Skin".to_string(),
            samples: Vec::new(),
        }]
    });
    let selected_group_idx = use_signal(|| 0usize);
    let new_group_name = use_signal(String::new);
    let new_group_mat = use_signal(String::new);
    let group_project_name = use_signal(|| "Sampled Project".to_string());
    let group_palette_idx = use_signal(|| 0usize);

    let open_photo = move |_: Event<MouseData>| {
        let svc = services.clone();
        spawn(async move {
            if let Some(file) = rfd::AsyncFileDialog::new()
                .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
                .pick_file()
                .await
                && let Ok(bytes) = std::fs::read(file.path())
            {
                if let Some(photo) = eye::open_reference(&svc, &bytes) {
                    image_data_url.set(Some(photo.data_uri.clone()));
                    study_data_url.set(Some(photo.data_uri));
                    study_mode.set(StudyMode::Color);
                    decoded_image.set(Some(photo.pixels));
                }
                if let Some(doms) = eye::dominants(&svc, &bytes) {
                    image_name.set(file.file_name());
                    if let Some(first) = doms.first() {
                        let h = first.color.to_hex();
                        picked_hex.set(h.clone());
                        state.last_sampled_hex.set(Some(h));
                    }
                    dominants.set(doms);
                }
            }
        });
    };

    rsx! {
        div {
            SectionHeader {
                title: locale.tr("Pipette & Relevés sur référence", "Eyedropper & Reference Sampling").to_string(),
                subtitle: locale.tr(
                    "Cliquez directement sur la photo pour relever une teinte (moyennée en lumière linéaire), neutralisez l'éclairage et bâtissez un projet par groupes de mesures ou via l'IA.",
                    "Click directly on the photo to sample a color (averaged in linear light), neutralize studio lighting, and build a project from sampled groups or via AI."
                ).to_string(),
                actions: Some(rsx! {
                    button {
                        class: "btn-primary",
                        onclick: open_photo,
                        {locale.tr("Ouvrir une photo de référence…", "Open a reference photo…")}
                    }
                })
            }

            div { class: "grid-2", style: "grid-template-columns: 1.25fr 1fr;",
                // Left column: Interactive photo canvas + K-Means dominants
                div {
                    ReferencePhotoCard {
                        image_name,
                        image_data_url,
                        study_data_url,
                        study_mode,
                        decoded_image,
                        zoom_pct,
                        sample_radius,
                        pick_neutral_mode,
                        last_click_coords,
                        grey_ref_hex,
                        has_custom_neutral,
                        picked_hex,
                    }

                    // Dominant K-Means L*a*b* colors
                    if !dominants().is_empty() {
                        DominantsCard { dominants: dominants(), grey_ref_hex: grey_ref_hex(), picked_hex }
                    }
                }

                // Right column: Picked color + Sampled groups (SampledPlanService) + AI pinned zones
                div {
                    ActiveColorCard { picked_hex }

                    // Sampled groups card -> Create measured project (SampledPlanService)
                    SampledGroupsCard {
                        picked_hex,
                        groups,
                        selected_group_idx,
                        new_group_name,
                        new_group_mat,
                        group_project_name,
                        group_palette_idx,
                    }

                    // Pin base color for AI Assistant
                    PinZoneCard { picked_hex, zone_name, zone_material }
                }
            }
        }
    }
}

/// The reference photo: the eyedropper's toolbar, the value studies, and the photo itself.
#[component]
fn ReferencePhotoCard(
    image_name: Signal<String>,
    image_data_url: Signal<Option<String>>,
    study_data_url: Signal<Option<String>>,
    study_mode: Signal<StudyMode>,
    decoded_image: Signal<Option<PixelMap>>,
    zoom_pct: Signal<u32>,
    sample_radius: Signal<isize>,
    pick_neutral_mode: Signal<bool>,
    last_click_coords: Signal<Option<(isize, isize)>>,
    grey_ref_hex: Signal<String>,
    has_custom_neutral: Signal<bool>,
    picked_hex: Signal<String>,
) -> Element {
    let state = use_context::<AppState>();
    let services = use_context::<AppServices>();
    let locale = *state.locale.read();

    rsx! {
        div { class: "card",
            div { class: "row-between", style: "margin-bottom: 10px;",
                h3 { class: "card-title", style: "margin: 0;",
                    if image_name().is_empty() {
                        {locale.tr("Photo de référence (clic = pipette)", "Reference photo (click = eyedropper)")}
                    } else {
                        "Photo: {image_name}"
                    }
                }
                if let Some(ref img) = *decoded_image.read() {
                    span { class: "pill", "{img.width} × {img.height} px" }
                }
            }

            // Eyedropper toolbar (Radius, Zoom, Neutral grey point)
            div { class: "card-subtle", style: "margin-bottom: 12px;",
                div { class: "row-between",
                    div { class: "row",
                        span { class: "field-label", style: "margin: 0;", {locale.tr("Rayon :", "Radius:")} }
                        input {
                            r#type: "number",
                            style: "width: 64px;",
                            min: "0",
                            max: "30",
                            value: "{sample_radius}",
                            oninput: move |e| {
                                if let Ok(r) = e.value().parse::<isize>() {
                                    sample_radius.set(r.clamp(0, 30));
                                }
                            }
                        }
                        span { class: "muted", style: "font-size: 11.5px;", "px" }

                        button {
                            class: "btn",
                            onclick: move |_| zoom_pct.set((zoom_pct().saturating_sub(25)).max(25)),
                            "−"
                        }
                        span { class: "mono", "{zoom_pct} %" }
                        button {
                            class: "btn",
                            onclick: move |_| zoom_pct.set((zoom_pct() + 25).min(300)),
                            "+"
                        }
                        button {
                            class: "btn",
                            onclick: move |_| zoom_pct.set(100),
                            "100 %"
                        }
                    }

                    div { class: "row",
                        button {
                            class: if *pick_neutral_mode.read() { "btn-primary" } else { "btn" },
                            onclick: move |_| {
                                let cur = *pick_neutral_mode.read();
                                pick_neutral_mode.set(!cur);
                            },
                            if *pick_neutral_mode.read() {
                                {locale.tr("Cliquez un gris sur la photo…", "Click a neutral grey on photo…")}
                            } else {
                                {locale.tr("Définir le point neutre", "Set neutral grey point")}
                            }
                        }
                        if *has_custom_neutral.read() {
                            ColorSwatch { hex: grey_ref_hex(), size: 22 }
                            button {
                                class: "btn",
                                onclick: move |_| {
                                    grey_ref_hex.set("#808080".to_string());
                                    has_custom_neutral.set(false);
                                    pick_neutral_mode.set(false);
                                },
                                {locale.tr("Réinitialiser", "Reset")}
                            }
                        }
                    }
                }

                // Value study mode selector (True color / 9-value Grisaille / 3-mass Notan / Squint)
                if decoded_image.read().is_some() {
                    div { class: "row", style: "margin-top: 8px; padding-top: 8px; border-top: 1px solid var(--border-subtle);",
                        span { class: "field-label", style: "margin: 0;", {locale.tr("Étude des masses :", "Value study:")} }
                        for mode in StudyMode::ALL {
                            {
                                let is_active = *study_mode.read() == mode;
                                let svc = services.clone();
                                rsx! {
                                    button {
                                        class: if is_active { "btn-primary" } else { "btn" },
                                        style: "padding: 4px 10px; font-size: 11.5px;",
                                        onclick: move |_| show_study(&svc, mode, study_mode, image_data_url, study_data_url, decoded_image),
                                        "{mode.label(locale)}"
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if let Some(ref url) = *study_data_url.read() {
                SamplingImage {
                    url: url.clone(),
                    decoded_image,
                    zoom_pct,
                    sample_radius,
                    pick_neutral_mode,
                    last_click_coords,
                    grey_ref_hex,
                    has_custom_neutral,
                    picked_hex,
                }
                if let Some((cx, cy)) = *last_click_coords.read() {
                    div { class: "muted", style: "font-size: 11.5px; margin-top: 8px;",
                        if locale.is_fr() {
                            "Dernier relevé au point ({cx}, {cy}) · Rayon {sample_radius} px (moyenne en lumière linéaire)"
                        } else {
                            "Last sample at ({cx}, {cy}) · Radius {sample_radius} px (linear-light average)"
                        }
                    }
                }
            } else {
                div { class: "card-subtle", style: "text-align: center; padding: 36px 20px;",
                    p { class: "muted",
                        {locale.tr("Ouvrez une photo de référence pour activer la pipette au clic et extraire ses dominantes chromatiques.", "Open a reference photo to enable click sampling and extract its dominant color poles.")}
                    }
                }
            }
        }
    }
}

/// Shows the photo through a value study, or as it is.
pub(crate) fn show_study(
    services: &AppServices,
    mode: StudyMode,
    mut study_mode: Signal<StudyMode>,
    image_data_url: Signal<Option<String>>,
    mut study_data_url: Signal<Option<String>>,
    decoded_image: Signal<Option<PixelMap>>,
) {
    study_mode.set(mode);
    if mode == StudyMode::Color {
        let orig = image_data_url.read().clone();
        study_data_url.set(orig);
    } else if let Some(ref map) = *decoded_image.read()
        && let Some(uri) = eye::study_data_uri(services, map, mode)
    {
        study_data_url.set(Some(uri));
    }
}

/// The photo itself, at its zoom: a click samples it, or sets the neutral grey point.
#[component]
fn SamplingImage(
    url: String,
    decoded_image: Signal<Option<PixelMap>>,
    zoom_pct: Signal<u32>,
    sample_radius: Signal<isize>,
    pick_neutral_mode: Signal<bool>,
    last_click_coords: Signal<Option<(isize, isize)>>,
    grey_ref_hex: Signal<String>,
    has_custom_neutral: Signal<bool>,
    picked_hex: Signal<String>,
) -> Element {
    let mut state = use_context::<AppState>();
    let z = zoom_pct();
    let img_style = if let Some(ref decoded) = *decoded_image.read() {
        let disp_w = (decoded.width as f64 * (z as f64 / 100.0)).round() as u32;
        format!("width: {disp_w}px; max-width: none;")
    } else {
        "max-width: 100%;".to_string()
    };

    let sample = move |evt: Event<MouseData>| {
        let loc = *state.locale.read();
        let Some(ref decoded) = *decoded_image.read() else { return };
        let coords = evt.element_coordinates();
        let rad = *sample_radius.read();
        let Some(((cx, cy), raw_rgb)) = eye::sample_click(decoded, coords.x, coords.y, zoom_pct(), rad) else {
            return;
        };
        last_click_coords.set(Some((cx, cy)));
        if *pick_neutral_mode.read() {
            grey_ref_hex.set(raw_rgb.to_hex());
            has_custom_neutral.set(true);
            pick_neutral_mode.set(false);
            let msg = match loc {
                Locale::Fr => format!("Point gris neutre défini sur {} ({cx}, {cy}).", raw_rgb.to_hex()),
                Locale::En => format!("Neutral grey point set to {} ({cx}, {cy}).", raw_rgb.to_hex()),
            };
            state.notify(msg);
        } else {
            let final_rgb = if *has_custom_neutral.read() {
                eye::neutralised(raw_rgb, &grey_ref_hex())
            } else {
                raw_rgb
            };
            let hex = final_rgb.to_hex();
            picked_hex.set(hex.clone());
            state.last_sampled_hex.set(Some(hex));
        }
    };

    rsx! {
        div { class: "eyedropper-viewport",
            img {
                class: "eyedropper-img",
                style: "{img_style}",
                src: "{url}",
                onclick: sample,
            }
        }
    }
}

/// The eight dominant colours, raw and corrected by the neutral grey point.
#[component]
fn DominantsCard(dominants: Vec<DominantColour>, grey_ref_hex: String, picked_hex: Signal<String>) -> Element {
    let state = use_context::<AppState>();
    let locale = *state.locale.read();

    rsx! {
        div { class: "card",
            h3 { class: "card-title", {locale.tr("Dominantes chromatiques (8 pôles CIE L*a*b*)", "Dominant Colors (8 CIE L*a*b* Poles)")} }
            table { class: "data-table",
                thead {
                    tr {
                        th { {locale.tr("Brute", "Raw")} }
                        th { {locale.tr("Corrigée", "Neutralized")} }
                        th { {locale.tr("Part", "Share")} }
                        th { "Actions" }
                    }
                }
                tbody {
                    for dom in dominants {
                        DominantRow { dom, grey_ref_hex: grey_ref_hex.clone(), picked_hex }
                    }
                }
            }
        }
    }
}

/// One dominant colour, to pick or to send to the mixer.
#[component]
fn DominantRow(dom: DominantColour, grey_ref_hex: String, picked_hex: Signal<String>) -> Element {
    let mut state = use_context::<AppState>();
    let locale = *state.locale.read();
    let raw_hex = dom.color.to_hex();
    let neutral_hex = eye::neutralised(dom.color, &grey_ref_hex).to_hex();
    let share_pct = dom.share * 100.0;

    rsx! {
        tr {
            td {
                div { class: "row",
                    ColorSwatch { hex: raw_hex.clone(), size: 22 }
                    span { class: "mono", "{raw_hex}" }
                }
            }
            td {
                div { class: "row",
                    ColorSwatch { hex: neutral_hex.clone(), size: 22 }
                    span { class: "mono", "{neutral_hex}" }
                }
            }
            td { "{share_pct:.1} %" }
            td {
                div { class: "row",
                    button {
                        class: "btn",
                        onclick: {
                            let h = neutral_hex.clone();
                            move |_| {
                                picked_hex.set(h.clone());
                                state.last_sampled_hex.set(Some(h.clone()));
                            }
                        },
                        {locale.tr("Relever", "Pick")}
                    }
                    button {
                        class: "btn",
                        onclick: {
                            let h = neutral_hex.clone();
                            move |_| {
                                state.mixer_target_hex.set(h.clone());
                                state.section.set(Section::Mixer);
                            }
                        },
                        {locale.tr("→ Mélangeur", "→ Mixer")}
                    }
                }
            }
        }
    }
}
