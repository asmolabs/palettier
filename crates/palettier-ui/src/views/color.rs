use crate::state::{AppServices, AppState, Section};
use crate::theme::{
    ColorSwatch, DeltaEBadge, GamutMapSvg, HexColorInput, MixingGuideCard, PaintSwatch,
    SectionHeader, ValueLadderStrip,
};
use base64::Engine;
use chrono::Utc;
use dioxus::prelude::*;
use palettier_ai::SampledZone;
use palettier_domain::color::{Rgb, delta_e_2000, neutralise};
use palettier_domain::i18n::Locale;
use palettier_domain::image::{PixelMap, StudyMode};
use palettier_domain::mix::{MixSuggestion, PaintPart};
use palettier_domain::palette::PaletteMix;
use palettier_domain::plan::{ColourGroup, PaintingPlan};
use palettier_domain::port::{PaletteMixRepository, ProjectRepository};
use std::collections::BTreeSet;

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
    let mut zoom_pct = use_signal(|| 100u32);
    let mut sample_radius = use_signal(|| 3isize);
    let mut pick_neutral_mode = use_signal(|| false);
    let mut last_click_coords = use_signal(|| None::<(isize, isize)>);

    let mut dominants = use_signal(Vec::new);
    let mut grey_ref_hex = use_signal(|| "#808080".to_string());
    let mut has_custom_neutral = use_signal(|| false);
    let mut zone_name = use_signal(|| "Tunic".to_string());
    let mut zone_material = use_signal(|| "Cloth".to_string());
    let mut picked_hex = use_signal(|| "#7A5442".to_string());

    // Sampled groups for building a measured project without AI (SampledPlanService)
    let mut groups = use_signal(|| {
        vec![ColourGroup {
            name: "Visage / Face".to_string(),
            material: "Skin".to_string(),
            samples: Vec::new(),
        }]
    });
    let mut selected_group_idx = use_signal(|| 0usize);
    let mut new_group_name = use_signal(String::new);
    let mut new_group_mat = use_signal(String::new);
    let mut group_project_name = use_signal(|| "Sampled Project".to_string());
    let mut group_palette_idx = use_signal(|| 0usize);

    let sampled = state.sampled_zones.read().clone();
    let palettes = state.palettes.read().clone();

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
                        onclick: {
                            let svc = services.clone();
                            move |_| {
                                let svc = svc.clone();
                                spawn(async move {
                                    if let Some(file) = rfd::AsyncFileDialog::new()
                                        .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
                                        .pick_file()
                                        .await
                                        && let Ok(bytes) = std::fs::read(file.path()) {
                                            if let Ok(decoded) = svc.decoder.decode(&bytes) {
                                                let scaled_jpeg = svc
                                                    .decoder
                                                    .scale_to(&bytes, 1400)
                                                    .unwrap_or_else(|_| bytes.clone());
                                                let b64 = base64::engine::general_purpose::STANDARD.encode(&scaled_jpeg);
                                                let uri = format!("data:image/jpeg;base64,{b64}");
                                                image_data_url.set(Some(uri.clone()));
                                                study_data_url.set(Some(uri));
                                                study_mode.set(StudyMode::Color);
                                                decoded_image.set(Some(decoded));
                                            }
                                            if let Ok(doms) = svc.decoder.dominant_colours(&bytes, 8) {
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
                            }
                        },
                        {locale.tr("Ouvrir une photo de référence…", "Open a reference photo…")}
                    }
                })
            }

            div { class: "grid-2", style: "grid-template-columns: 1.25fr 1fr;",
                // Left column: Interactive photo canvas + K-Means dominants
                div {
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
                                                    onclick: move |_| {
                                                        study_mode.set(mode);
                                                        if mode == StudyMode::Color {
                                                            let orig = image_data_url.read().clone();
                                                            study_data_url.set(orig);
                                                        } else if let Some(ref map) = *decoded_image.read() {
                                                            let transformed = map.to_study_map(mode);
                                                            if let Ok(png_bytes) = svc.decoder.encode_png(&transformed) {
                                                                let b64 = base64::engine::general_purpose::STANDARD.encode(&png_bytes);
                                                                study_data_url.set(Some(format!("data:image/png;base64,{b64}")));
                                                            }
                                                        }
                                                    },
                                                    "{mode.label_i18n(locale)}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        if let Some(ref url) = *study_data_url.read() {
                            {
                                let z = zoom_pct();
                                let img_style = if let Some(ref decoded) = *decoded_image.read() {
                                    let disp_w = (decoded.width as f64 * (z as f64 / 100.0)).round() as u32;
                                    format!("width: {disp_w}px; max-width: none;")
                                } else {
                                    "max-width: 100%;".to_string()
                                };
                                rsx! {
                                    div { class: "eyedropper-viewport",
                                        img {
                                            class: "eyedropper-img",
                                            style: "{img_style}",
                                            src: "{url}",
                                            onclick: move |evt: Event<MouseData>| {
                                                let loc = *state.locale.read();
                                                let Some(ref decoded) = *decoded_image.read() else { return };
                                                let coords = evt.element_coordinates();
                                                let scale = (zoom_pct() as f64 / 100.0).max(0.05);
                                                let cx = (coords.x / scale).floor() as isize;
                                                let cy = (coords.y / scale).floor() as isize;
                                                let rad = *sample_radius.read();
                                                if let Some(raw_rgb) = decoded.sample_around(cx, cy, rad) {
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
                                                            Rgb::of_hex(&grey_ref_hex())
                                                                .map(|g| neutralise(raw_rgb, g))
                                                                .unwrap_or(raw_rgb)
                                                        } else {
                                                            raw_rgb
                                                        };
                                                        let hex = final_rgb.to_hex();
                                                        picked_hex.set(hex.clone());
                                                        state.last_sampled_hex.set(Some(hex));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
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

                    // Dominant K-Means L*a*b* colors
                    if !dominants().is_empty() {
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
                                    for dom in dominants() {
                                        {
                                            let raw_hex = dom.color.to_hex();
                                            let neutral_rgb = Rgb::of_hex(&grey_ref_hex())
                                                .map(|g| neutralise(dom.color, g))
                                                .unwrap_or(dom.color);
                                            let neutral_hex = neutral_rgb.to_hex();
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
                                    }
                                }
                            }
                        }
                    }
                }

                // Right column: Picked color + Sampled groups (SampledPlanService) + AI pinned zones
                div {
                    div { class: "card",
                        h3 { class: "card-title", {locale.tr("Teinte active relevée", "Active Sampled Color")} }
                        div { class: "row-between",
                            div { class: "row",
                                ColorSwatch { hex: picked_hex(), size: 54 }
                                div {
                                    HexColorInput {
                                        value: picked_hex(),
                                        onchange: move |v: String| {
                                            picked_hex.set(v.clone());
                                            state.last_sampled_hex.set(Some(v));
                                        }
                                    }
                                    div { class: "muted", style: "font-size: 11.5px; margin-top: 4px;",
                                        {locale.tr("Disponible aussi dans le Catalogue pour calibrer un tube.", "Also available in the Catalog to calibrate a tube.")}
                                    }
                                }
                            }
                            div { class: "row",
                                button {
                                    class: "btn-primary",
                                    onclick: move |_| {
                                        let h = picked_hex();
                                        state.mixer_target_hex.set(h);
                                        state.section.set(Section::Mixer);
                                    },
                                    {locale.tr("Chercher au Mélangeur →", "Find in Mixer →")}
                                }
                            }
                        }
                    }

                    // Sampled groups card -> Create measured project (SampledPlanService)
                    div { class: "card",
                        h3 { class: "card-title", {locale.tr("Groupes de relevés → Créer un projet mesuré", "Sampled Groups → Build Measured Project")} }
                        p { class: "muted", style: "font-size: 12px; margin-bottom: 10px;",
                            {locale.tr("Relevez plusieurs points par partie du sujet (ombre profonde, base, lumière) : Palettier les trie par luminance et calcule les recettes de chaque marche.", "Sample multiple points per zone (deep shadow, base, highlight): Palettier sorts them by luminance and computes recipes for each step.")}
                        }

                        div { class: "row", style: "margin-bottom: 10px;",
                            input {
                                r#type: "text",
                                placeholder: locale.tr("Nouvelle zone (ex. Cape)", "New zone (e.g. Cloak)"),
                                value: "{new_group_name}",
                                oninput: move |e| new_group_name.set(e.value())
                            }
                            input {
                                r#type: "text",
                                style: "width: 120px;",
                                placeholder: locale.tr("Matière (ex. Drap)", "Material (e.g. Cloth)"),
                                value: "{new_group_mat}",
                                oninput: move |e| new_group_mat.set(e.value())
                            }
                            button {
                                class: "btn",
                                onclick: move |_| {
                                    let name = new_group_name().trim().to_string();
                                    if name.is_empty() {
                                        return;
                                    }
                                    let mat = new_group_mat().trim().to_string();
                                    let mut list = groups.read().clone();
                                    list.push(ColourGroup {
                                        name,
                                        material: mat,
                                        samples: Vec::new(),
                                    });
                                    selected_group_idx.set(list.len() - 1);
                                    groups.set(list);
                                    new_group_name.set(String::new());
                                    new_group_mat.set(String::new());
                                },
                                {locale.tr("+ Groupe", "+ Group")}
                            }
                        }

                        // Group list
                        for (g_idx, grp) in groups.read().clone().iter().enumerate() {
                            {
                                let is_sel = *selected_group_idx.read() == g_idx;
                                let mut sorted_samples = grp.samples.clone();
                                sorted_samples.sort_by(|a, b| a.relative_luminance().total_cmp(&b.relative_luminance()));
                                let ladder_steps: Vec<(String, String)> = sorted_samples
                                    .iter()
                                    .enumerate()
                                    .map(|(i, s)| (format!("#{}", i + 1), s.to_hex()))
                                    .collect();

                                rsx! {
                                    div {
                                        class: "card-subtle",
                                        style: if is_sel { "margin-bottom: 8px; border-color: var(--accent); cursor: pointer;" } else { "margin-bottom: 8px; cursor: pointer;" },
                                        onclick: move |_| selected_group_idx.set(g_idx),
                                        div { class: "row-between",
                                            div {
                                                strong { "{grp.name}" }
                                                if !grp.material.is_empty() {
                                                    span { class: "muted", " · {grp.material}" }
                                                }
                                                span { class: "pill", style: "margin-left: 8px;", "{grp.samples.len()} pt(s)" }
                                            }
                                            div { class: "row",
                                                button {
                                                    class: "btn-primary",
                                                    onclick: move |e: Event<MouseData>| {
                                                        e.stop_propagation();
                                                        if let Ok(rgb) = Rgb::of_hex(&picked_hex()) {
                                                            let mut list = groups.read().clone();
                                                            if let Some(g) = list.get_mut(g_idx) {
                                                                g.samples.push(rgb);
                                                            }
                                                            groups.set(list);
                                                        }
                                                    },
                                                    "+ {picked_hex}"
                                                }
                                                if groups.read().len() > 1 {
                                                    button {
                                                        class: "btn-danger",
                                                        onclick: move |e: Event<MouseData>| {
                                                            e.stop_propagation();
                                                            let mut list = groups.read().clone();
                                                            if g_idx < list.len() {
                                                                list.remove(g_idx);
                                                                selected_group_idx.set(0);
                                                                groups.set(list);
                                                            }
                                                        },
                                                        "✕"
                                                    }
                                                }
                                            }
                                        }
                                        if !ladder_steps.is_empty() {
                                            ValueLadderStrip { steps: ladder_steps }
                                        }
                                    }
                                }
                            }
                        }

                        div { class: "row", style: "margin-top: 12px;",
                            input {
                                r#type: "text",
                                style: "flex: 1;",
                                placeholder: locale.tr("Nom du projet à créer", "Project name to create"),
                                value: "{group_project_name}",
                                oninput: move |e| group_project_name.set(e.value())
                            }
                            select {
                                onchange: move |e| {
                                    if let Ok(idx) = e.value().parse::<usize>() {
                                        group_palette_idx.set(idx);
                                    }
                                },
                                for (idx, pal) in palettes.iter().enumerate() {
                                    option { value: "{idx}", "{pal.name} ({pal.paints.len()} tubes)" }
                                }
                            }
                            button {
                                class: "btn-primary",
                                onclick: {
                                    let svc = services.clone();
                                    let pals = palettes.clone();
                                    move |_| {
                                        let loc = *state.locale.read();
                                        let chosen_pal = pals.get(group_palette_idx()).cloned();
                                        let grp_list = groups.read().clone();
                                        let raw_name = group_project_name().trim().to_string();
                                        match svc.sampled_plan_service.plan(
                                            if raw_name.is_empty() { loc.tr("Projet relevé", "Sampled Project") } else { &raw_name },
                                            chosen_pal.as_ref(),
                                            &grp_list,
                                            3,
                                        ) {
                                            Ok(plan) => {
                                                let existing: Vec<String> = state.projects.read().iter().map(|p| p.name.clone()).collect();
                                                let unique = palettier_domain::plan::unique_name(
                                                    &plan.subject,
                                                    existing.iter().map(String::as_str),
                                                );
                                                let proj = plan.as_project(unique, chosen_pal.as_ref());
                                                let svc = svc.clone();
                                                spawn(async move {
                                                    if let Ok(saved) = svc.projects.save(&proj).await {
                                                        state.reload_all(&svc).await;
                                                        let msg = match loc {
                                                            Locale::Fr => format!("Projet « {} » créé à partir de vos relevés.", saved.name),
                                                            Locale::En => format!("Project '{}' created from your sampled groups.", saved.name),
                                                        };
                                                        state.notify(msg);
                                                        state.section.set(Section::Projects);
                                                    }
                                                });
                                            }
                                            Err(e) => state.notify(format!("{e}")),
                                        }
                                    }
                                },
                                {locale.tr("Créer le projet →", "Create project →")}
                            }
                        }
                    }

                    // Pin base color for AI Assistant
                    div { class: "card",
                        h3 { class: "card-title", {locale.tr("Épingler une teinte de base pour l'Assistant IA", "Pin a Base Color for the AI Assistant")} }
                        div { class: "row", style: "margin-bottom: 10px;",
                            input {
                                r#type: "text",
                                placeholder: locale.tr("Zone (ex. Tunique)", "Zone (e.g. Tunic)"),
                                value: "{zone_name}",
                                oninput: move |ev| zone_name.set(ev.value())
                            }
                            input {
                                r#type: "text",
                                style: "width: 120px;",
                                placeholder: locale.tr("Matière", "Material"),
                                value: "{zone_material}",
                                oninput: move |ev| zone_material.set(ev.value())
                            }
                            button {
                                class: "btn",
                                onclick: move |_| {
                                    if let Ok(rgb) = Rgb::of_hex(&picked_hex()) {
                                        let mut list = state.sampled_zones.read().clone();
                                        list.push(SampledZone::new(zone_name().trim(), zone_material().trim(), rgb));
                                        state.sampled_zones.set(list);
                                    }
                                },
                                {locale.tr("+ Épingler", "+ Pin")}
                            }
                            button {
                                class: "btn",
                                onclick: move |_| state.section.set(Section::Assistant),
                                {locale.tr("Assistant IA", "AI Assistant")},
                                " ({sampled.len()})"
                            }
                        }

                        if !sampled.is_empty() {
                            div { class: "row",
                                for (idx, sz) in sampled.iter().enumerate() {
                                    div { class: "card-subtle row",
                                        ColorSwatch { hex: sz.target_hex.clone(), size: 18 }
                                        strong { "{sz.name}" }
                                        span { class: "muted", "({sz.material})" }
                                        button {
                                            class: "btn-danger",
                                            onclick: move |_| {
                                                let mut list = state.sampled_zones.read().clone();
                                                if idx < list.len() {
                                                    list.remove(idx);
                                                    state.sampled_zones.set(list);
                                                }
                                            },
                                            "✕"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn AssistantView() -> Element {
    let mut state = use_context::<AppState>();
    let services = use_context::<AppServices>();

    let locale = *state.locale.read();
    let palettes = state.palettes.read().clone();
    let all_paints = state.paints.read().clone();
    let sampled = state.sampled_zones.read().clone();
    let ai = state.ai_settings.read().clone();

    let mut description = use_signal(|| {
        "Buste de grognard de la Garde Impériale au 1/10 : visage buriné, revers de manteau bleu impérial et épaulettes rouges.".to_string()
    });
    let mut palette_idx = use_signal(|| 0usize);
    let mut image_bytes = use_signal(|| None::<Vec<u8>>);
    let mut image_label = use_signal(String::new);
    let mut busy = use_signal(|| false);
    let mut generated_plan = use_signal(|| None::<PaintingPlan>);
    let mut error_msg = use_signal(|| None::<String>);
    let mut show_guide_mode = use_signal(|| true);

    let subtitle_str = match locale {
        Locale::Fr => format!(
            "Moteur actif : {} ({}) · L'IA propose les échelles de valeurs à 5 marches et le solveur Kubelka-Munk calcule les dosages.",
            ai.provider.label(),
            ai.model
        ),
        Locale::En => format!(
            "Active engine: {} ({}) · AI proposes 5-step value ladders and the Kubelka-Munk solver computes exact ratios.",
            ai.provider.label(),
            ai.model
        ),
    };

    rsx! {
        div {
            SectionHeader {
                title: locale.tr("Assistant de plan de peinture", "Painting Plan Assistant").to_string(),
                subtitle: subtitle_str,
                actions: Some(rsx! {
                    button {
                        class: "btn",
                        onclick: move |_| state.section.set(Section::Settings),
                        {locale.tr("Configurer le moteur IA…", "Configure AI engine…")}
                    }
                })
            }

            div { class: "card",
                div { class: "grid-2", style: "margin-bottom:12px;",
                    div {
                        label { class: "field-label",
                            {locale.tr("Description du sujet et de l'ambiance lumineuse", "Subject Description & Lighting Mood")}
                        }
                        textarea {
                            style: "width:100%;height:88px;",
                            value: "{description}",
                            oninput: move |ev| description.set(ev.value())
                        }
                    }
                    div {
                        div { style: "margin-bottom:10px;",
                            label { class: "field-label",
                                {locale.tr("Palette à utiliser pour les mélanges", "Palette to Use for Mixes")}
                            }
                            select {
                                style: "width:100%;",
                                onchange: move |ev| {
                                    if let Ok(idx) = ev.value().parse::<usize>() {
                                        palette_idx.set(idx);
                                    }
                                },
                                for (idx, pal) in palettes.iter().enumerate() {
                                    option { value: "{idx}", "{pal.name} ({pal.paints.len()} tubes)" }
                                }
                                option { value: "9999", {locale.tr("Tous mes tubes en stock", "All my in-stock tubes")} }
                            }
                        }
                        div { class: "row",
                            button {
                                class: "btn",
                                onclick: move |_| {
                                    spawn(async move {
                                        if let Some(file) = rfd::AsyncFileDialog::new()
                                            .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
                                            .pick_file()
                                            .await
                                            && let Ok(bytes) = std::fs::read(file.path()) {
                                                image_label.set(file.file_name());
                                                image_bytes.set(Some(bytes));
                                            }
                                    });
                                },
                                if image_label().is_empty() {
                                    {locale.tr("Joindre une photo de référence…", "Attach a reference photo…")}
                                } else {
                                    "Photo: {image_label}"
                                }
                            }
                            if !sampled.is_empty() {
                                span { class: "pill pill-accent",
                                    "{sampled.len()} ",
                                    {locale.tr("zones issues de la Pipette", "zones pinned from Eyedropper")}
                                }
                            }
                        }
                    }
                }

                div { class: "row",
                    button {
                        class: "btn-primary",
                        disabled: busy(),
                        onclick: {
                            let svc = services.clone();
                            let pals = palettes.clone();
                            let pts = all_paints.clone();
                            let smp = sampled.clone();
                            move |_| {
                                let loc = *state.locale.read();
                                busy.set(true);
                                error_msg.set(None);
                                let desc = description();
                                let idx = palette_idx();
                                let img = image_bytes();
                                let (pal_name, active_paints) = if let Some(pal) = pals.get(idx) {
                                    (pal.name.clone(), pal.paints.clone())
                                } else {
                                    let owned: Vec<_> = pts.iter().filter(|p| p.in_stock).cloned().collect();
                                    (loc.tr("Stock atelier", "Studio Stock").to_string(), if owned.is_empty() { pts.clone() } else { owned })
                                };
                                let svc = svc.clone();
                                let smp = smp.clone();
                                spawn(async move {
                                    match svc
                                        .plan_service
                                        .plan_i18n(img.as_deref(), &desc, &pal_name, &active_paints, &smp, loc)
                                        .await
                                    {
                                        Ok(plan) => generated_plan.set(Some(plan)),
                                        Err(e) => error_msg.set(Some(e.to_string())),
                                    }
                                    busy.set(false);
                                });
                            }
                        },
                        if busy() {
                            {locale.tr("Calcul du plan en cours…", "Generating painting plan…")}
                        } else {
                            {locale.tr("Générer le plan de peinture", "Generate painting plan")}
                        }
                    }
                }

                if let Some(err) = error_msg() {
                    div { class: "card-subtle", style: "border-color:var(--danger);margin-top:12px;color:var(--danger);",
                        "{err}"
                    }
                }
            }

            if let Some(plan) = generated_plan() {
                {
                    let active_paints = palettes
                        .get(palette_idx())
                        .map(|p| p.paints.clone())
                        .unwrap_or_else(|| {
                            let owned: Vec<_> = all_paints.iter().filter(|p| p.in_stock).cloned().collect();
                            if owned.is_empty() { all_paints.clone() } else { owned }
                        });
                    let guide = services.guide_builder.from_plan_i18n(&plan, &active_paints, locale);
                    let photo_uri = image_bytes().map(|bytes| {
                        let scaled = services.decoder.scale_to(&bytes, 600).unwrap_or(bytes);
                        let b64 = base64::engine::general_purpose::STANDARD.encode(&scaled);
                        format!("data:image/jpeg;base64,{b64}")
                    });

                    rsx! {
                        div { class: "card",
                            div { class: "row-between", style: "margin-bottom:14px;",
                                div {
                                    h2 { class: "card-title", style: "font-size:18px;margin-bottom:2px;", "{plan.subject}" }
                                    p { class: "muted", "{plan.approach}" }
                                }
                                div { class: "row",
                                    button {
                                        class: if *show_guide_mode.read() { "btn-primary" } else { "btn" },
                                        onclick: move |_| show_guide_mode.set(true),
                                        {locale.tr("Fiche Guide de mélanges (Norma Blue)", "Mixing Guide Sheet (Norma Blue)")}
                                    }
                                    button {
                                        class: if !*show_guide_mode.read() { "btn-primary" } else { "btn" },
                                        onclick: move |_| show_guide_mode.set(false),
                                        {locale.tr("Vue Technique (ΔE & Échelles)", "Technical View (ΔE & Ladders)")}
                                    }
                                    button {
                                        class: "btn-primary",
                                        onclick: {
                                            let svc = services.clone();
                                            let pals = palettes.clone();
                                            let pts = all_paints.clone();
                                            let plan_clone = plan.clone();
                                            move |_| {
                                                let loc = *state.locale.read();
                                                let chosen_pal = pals.get(palette_idx()).cloned();
                                                let active_paints = chosen_pal
                                                    .as_ref()
                                                    .map(|p| p.paints.clone())
                                                    .unwrap_or_else(|| pts.clone());
                                                let existing_names: Vec<String> = state
                                                    .projects
                                                    .read()
                                                    .iter()
                                                    .map(|p| p.name.clone())
                                                    .collect();
                                                let name = palettier_domain::plan::unique_name(
                                                    &plan_clone.subject,
                                                    existing_names.iter().map(String::as_str),
                                                );
                                                let mut proj = plan_clone.as_project(name, chosen_pal.as_ref());
                                                if proj.paints.is_empty() {
                                                    proj.paints = active_paints;
                                                }
                                                let attached_img = image_bytes();
                                                let svc = svc.clone();
                                                spawn(async move {
                                                    if let Ok(saved) = svc.projects.save(&proj).await {
                                                        if let (Some(pid), Some(bytes)) = (saved.id, attached_img) {
                                                            let photo = palettier_domain::project::ProjectPhoto {
                                                                id: None,
                                                                data: bytes,
                                                                role: palettier_domain::project::PhotoRole::Piece,
                                                                caption: loc.tr("Photo de référence", "Reference photo").to_string(),
                                                                added_at: Utc::now(),
                                                            };
                                                            let _ = svc.projects.add_photo(pid, &photo).await;
                                                        }
                                                        if let Ok(all) = svc.projects.all().await {
                                                            state.projects.set(all);
                                                        }
                                                        let msg = match loc {
                                                            Locale::Fr => format!("Projet « {} » créé depuis le plan.", saved.name),
                                                            Locale::En => format!("Project '{}' created from painting plan.", saved.name),
                                                        };
                                                        state.notify(msg);
                                                        state.section.set(Section::Projects);
                                                    }
                                                });
                                            }
                                        },
                                        {locale.tr("Enregistrer comme Projet →", "Save as Project →")}
                                    }
                                }
                            }

                            if *show_guide_mode.read() {
                                MixingGuideCard {
                                    guide: guide,
                                    photo_data_uri: photo_uri,
                                    on_notify: move |msg: String| state.notify(msg),
                                }
                            } else {
                                for z in &plan.zones {
                                    {
                                        let ladder_steps: Vec<(String, String)> = z
                                            .layers()
                                            .iter()
                                            .map(|l| (l.role.clone(), l.target.to_hex()))
                                            .collect();
                                        rsx! {
                                            div { class: "card-subtle", style: "margin-bottom:12px;",
                                                strong { style: "font-size:14.5px;", "{z.name} ({z.material})" }
                                                ValueLadderStrip { steps: ladder_steps }
                                                table { class: "data-table", style: "margin-top:8px;",
                                                    thead {
                                                        tr {
                                                            th { {locale.tr("Marche", "Step")} }
                                                            th { {locale.tr("Cible", "Target")} }
                                                            th { {locale.tr("Obtenu & Recette Kubelka-Munk", "Achieved & Kubelka-Munk Recipe")} }
                                                            th { {locale.tr("Écart", "Gap")} }
                                                            th { {locale.tr("Technique & Placement", "Technique & Placement")} }
                                                        }
                                                    }
                                                    tbody {
                                                        for l in z.layers().iter().chain(z.accents.iter()) {
                                                            tr {
                                                                td { strong { "{l.role}" } }
                                                                td {
                                                                    div { class: "row",
                                                                        ColorSwatch { hex: l.target.to_hex(), size: 22 }
                                                                        span { class: "mono", "{l.target.to_hex()}" }
                                                                    }
                                                                }
                                                                td {
                                                                    if let Some(ref rec) = l.recipe {
                                                                        div { class: "row",
                                                                            ColorSwatch { hex: l.achieved.to_hex(), size: 22 }
                                                                            span { "{rec.describe_i18n(locale)}" }
                                                                        }
                                                                    } else {
                                                                        span { class: "muted", {locale.tr("Aucune recette", "No recipe")} }
                                                                    }
                                                                }
                                                                td { DeltaEBadge { delta_e: l.delta_e } }
                                                                td {
                                                                    div { "{l.technique}" }
                                                                    if !l.note.is_empty() {
                                                                        div { class: "muted", style: "font-size: 11px;", "{l.note}" }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn MixerView() -> Element {
    let mut state = use_context::<AppState>();
    let services = use_context::<AppServices>();

    let locale = *state.locale.read();
    let all_paints = state.paints.read().clone();
    let palettes = state.palettes.read().clone();
    let target_hex = state.mixer_target_hex.read().clone();

    // Left panel: Freeform bench simulator (ColorMixService::mix)
    let brands: Vec<String> = {
        let mut s = BTreeSet::new();
        for p in &all_paints {
            s.insert(p.brand.clone());
        }
        s.into_iter().collect()
    };
    let mut sim_brand = use_signal(|| brands.first().cloned().unwrap_or_default());
    let mut sim_paint_idx = use_signal(|| 0usize);
    let mut sim_parts_input = use_signal(|| 1.0_f64);
    let mut manual_parts = use_signal(Vec::<PaintPart>::new);
    let mut save_mix_name = use_signal(String::new);

    // Right panel: Inverse Kubelka-Munk solver
    let mut scope_idx = use_signal(|| 0usize); // 0..N = Palette, 9998 = In stock, 9999 = Full catalog
    let mut max_paints = use_signal(|| 3usize);
    let mut suggestions = use_signal(Vec::<MixSuggestion>::new);

    let brand_paints: Vec<_> = all_paints
        .iter()
        .filter(|p| sim_brand().is_empty() || p.brand == sim_brand())
        .cloned()
        .collect();

    let current_mix_result = {
        let parts_snapshot = manual_parts.read().clone();
        if parts_snapshot.is_empty() {
            None
        } else {
            services.mix_service.mix(&parts_snapshot).ok()
        }
    };

    let run_search = {
        let svc = services.clone();
        let pts = all_paints.clone();
        let pals = palettes.clone();
        move |hex_str: String, sc: usize, mp: usize| {
            let Ok(target_rgb) = Rgb::of_hex(&hex_str) else {
                return;
            };
            let pool: Vec<_> = if let Some(pal) = pals.get(sc) {
                pal.paints.clone()
            } else if sc == 9998 {
                pts.iter().filter(|p| p.in_stock).cloned().collect()
            } else {
                pts.clone()
            };
            let found = svc.mix_service.suggest_mixes(target_rgb, &pool, 6, Some(mp));
            suggestions.set(found);
        }
    };

    rsx! {
        div {
            SectionHeader {
                title: locale.tr("Mélangeur & Simulateur Kubelka-Munk", "Kubelka-Munk Mixer & Simulator").to_string(),
                subtitle: locale.tr(
                    "Composez librement une recette en parts à gauche (avec répartition en volume vs poids pigmentaire), ou laissez le solveur Kubelka-Munk trouver le dosage optimal à droite.",
                    "Freely compose a part-based recipe on the left (comparing volume share vs pigment strength), or let the inverse Kubelka-Munk solver find optimal ratios on the right."
                ).to_string(),
                actions: None
            }

            div { class: "grid-2",
                // Left column: Bench simulator (ColorMixService::mix)
                div {
                    div { class: "card",
                        div { class: "row-between", style: "margin-bottom: 10px;",
                            h3 { class: "card-title", style: "margin: 0;",
                                {locale.tr("Composition du mélange sur la palette", "Palette Mix Composition")}
                            }
                            if !manual_parts.read().is_empty() {
                                button {
                                    class: "btn",
                                    onclick: move |_| manual_parts.set(Vec::new()),
                                    {locale.tr("Vider la paillasse", "Clear bench")}
                                }
                            }
                        }

                        div { class: "card-subtle", style: "margin-bottom: 12px;",
                            div { class: "row",
                                select {
                                    style: "width: 155px;",
                                    value: "{sim_brand}",
                                    onchange: move |e| {
                                        sim_brand.set(e.value());
                                        sim_paint_idx.set(0);
                                    },
                                    for b in &brands {
                                        option { value: "{b}", "{b}" }
                                    }
                                }
                                select {
                                    style: "flex: 1; min-width: 180px;",
                                    onchange: move |e| {
                                        if let Ok(i) = e.value().parse::<usize>() {
                                            sim_paint_idx.set(i);
                                        }
                                    },
                                    for (i, pt) in brand_paints.iter().enumerate() {
                                        option {
                                            value: "{i}",
                                            selected: *sim_paint_idx.read() == i,
                                            "{pt.name} ({pt.hex_color})"
                                        }
                                    }
                                }
                                input {
                                    r#type: "number",
                                    style: "width: 68px;",
                                    min: "0.5",
                                    max: "20",
                                    step: "0.5",
                                    value: "{sim_parts_input}",
                                    oninput: move |e| {
                                        if let Ok(v) = e.value().parse::<f64>() {
                                            sim_parts_input.set(v.max(0.5));
                                        }
                                    }
                                }
                                button {
                                    class: "btn-primary",
                                    onclick: {
                                        let bp = brand_paints.clone();
                                        move |_| {
                                            if let Some(pt) = bp.get(*sim_paint_idx.read()).cloned() {
                                                let dose = (*sim_parts_input.read()).max(0.5);
                                                let mut list = manual_parts.read().clone();
                                                if let Some(existing) = list.iter_mut().find(|p| p.paint.brand == pt.brand && p.paint.name == pt.name) {
                                                    existing.parts += dose;
                                                } else if let Ok(pp) = PaintPart::new(pt, dose) {
                                                    list.push(pp);
                                                }
                                                manual_parts.set(list);
                                            }
                                        }
                                    },
                                    {locale.tr("+ Ajouter", "+ Add")}
                                }
                            }
                        }

                        if manual_parts.read().is_empty() {
                            p { class: "muted", style: "padding: 14px 4px;",
                                {locale.tr("Ajoutez des tubes ci-dessus ou cliquez sur « ← Simuler » en face d'une recette suggérée à droite pour simuler le mélange.", "Add tubes above or click '← Simulate' next to a suggested recipe on the right to simulate the mix.")}
                            }
                        } else {
                            table { class: "data-table",
                                thead {
                                    tr {
                                        th { {locale.tr("Huile", "Oil Paint")} }
                                        th { "Parts" }
                                        th { "Volume" }
                                        th { {locale.tr("Poids dans la couleur", "Pigment Weight")} }
                                        th { "" }
                                    }
                                }
                                tbody {
                                    for (idx, part) in manual_parts.read().clone().iter().enumerate() {
                                        {
                                            let comp = current_mix_result
                                                .as_ref()
                                                .and_then(|r| r.components.get(idx));
                                            let vol_pct = comp.map(|c| c.volume_share * 100.0).unwrap_or(0.0);
                                            let pig_pct = comp.map(|c| c.pigment_share * 100.0).unwrap_or(0.0);
                                            rsx! {
                                                tr {
                                                    td {
                                                        div { class: "row",
                                                            PaintSwatch {
                                                                masstone_hex: part.paint.hex_color.clone(),
                                                                tint_hex: part.paint.tint_hex.clone(),
                                                                width: 28,
                                                                height: 20,
                                                            }
                                                            div {
                                                                div { style: "font-weight: 600;", "{part.paint.name}" }
                                                                div { class: "muted", style: "font-size: 11px;", "{part.paint.brand}" }
                                                            }
                                                        }
                                                    }
                                                    td {
                                                        div { class: "row",
                                                            button {
                                                                class: "btn",
                                                                style: "padding: 2px 7px;",
                                                                onclick: move |_| {
                                                                    let mut list = manual_parts.read().clone();
                                                                    if let Some(item) = list.get_mut(idx) {
                                                                        item.parts = (item.parts - 0.5).max(0.5);
                                                                    }
                                                                    manual_parts.set(list);
                                                                },
                                                                "−"
                                                            }
                                                            span { class: "mono", "{part.parts:.1}" }
                                                            button {
                                                                class: "btn",
                                                                style: "padding: 2px 7px;",
                                                                onclick: move |_| {
                                                                    let mut list = manual_parts.read().clone();
                                                                    if let Some(item) = list.get_mut(idx) {
                                                                        item.parts += 0.5;
                                                                    }
                                                                    manual_parts.set(list);
                                                                },
                                                                "+"
                                                            }
                                                        }
                                                    }
                                                    td {
                                                        div { class: "mono", "{vol_pct:.0} %" }
                                                        div { class: "share-bar-bg",
                                                            div { class: "share-bar-fill", style: "width: {vol_pct:.0}%;" }
                                                        }
                                                    }
                                                    td {
                                                        div { class: "mono", "{pig_pct:.0} %" }
                                                        div { class: "share-bar-bg",
                                                            div { class: "share-bar-fill", style: "width: {pig_pct:.0}%;" }
                                                        }
                                                    }
                                                    td {
                                                        button {
                                                            class: "btn-danger",
                                                            onclick: move |_| {
                                                                let mut list = manual_parts.read().clone();
                                                                if idx < list.len() {
                                                                    list.remove(idx);
                                                                    manual_parts.set(list);
                                                                }
                                                            },
                                                            "✕"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Simulated mix result card
                    if let Some(res) = current_mix_result {
                        {
                            let res_hex = res.hex();
                            let gap_to_target = Rgb::of_hex(&target_hex)
                                .map(|t| delta_e_2000(t, res.color))
                                .unwrap_or(0.0);
                            let pigs = res.pigments.iter().cloned().collect::<Vec<_>>().join(", ");
                            let recipe_desc = manual_parts
                                .read()
                                .iter()
                                .map(|p| format!("{:.1}× {}", p.parts, p.paint.display_name()))
                                .collect::<Vec<_>>()
                                .join(" + ");

                            rsx! {
                                div { class: "card",
                                    h3 { class: "card-title", {locale.tr("Résultat du mélange simulé", "Simulated Mix Result")} }
                                    div { class: "row-between", style: "margin-bottom: 12px;",
                                        div { class: "row",
                                            div {
                                                div { class: "muted", style: "font-size: 10.5px; margin-bottom: 3px;",
                                                    {locale.tr("Obtenu", "Achieved")}
                                                }
                                                ColorSwatch { hex: res_hex.clone(), size: 68 }
                                            }
                                            div {
                                                div { class: "muted", style: "font-size: 10.5px; margin-bottom: 3px;",
                                                    {locale.tr("Cible", "Target")}
                                                }
                                                ColorSwatch { hex: target_hex.clone(), size: 68 }
                                            }
                                            div { style: "margin-left: 8px;",
                                                div { class: "mono", style: "font-size: 20px; font-weight: 700;", "{res_hex}" }
                                                div { class: "row", style: "margin-top: 4px;",
                                                    DeltaEBadge { delta_e: gap_to_target }
                                                    span { class: "pill", {locale.tr("Séchage : ", "Drying: ")}, "{res.drying_class.label_i18n(locale)}" }
                                                }
                                                div { class: "muted mono", style: "margin-top: 5px; font-size: 11.5px;",
                                                    "Pigments: {pigs}"
                                                }
                                            }
                                        }
                                    }

                                    for w in &res.warnings {
                                        div { class: "card-subtle", style: "border-color: var(--pill-warn-border); margin-bottom: 8px; font-size: 12px;",
                                            "• {w}"
                                        }
                                    }

                                    div { class: "row", style: "margin-top: 10px;",
                                        input {
                                            r#type: "text",
                                            style: "flex: 1;",
                                            placeholder: locale.tr("Nom pour poser ce mélange sur la palette (ex. Ombre chaude)…", "Name to place this mix on the palette (e.g. Warm Shadow)…"),
                                            value: "{save_mix_name}",
                                            oninput: move |e| save_mix_name.set(e.value())
                                        }
                                        button {
                                            class: "btn-primary",
                                            onclick: {
                                                let svc = services.clone();
                                                let hex_to_save = res_hex.clone();
                                                let dc = res.drying_class;
                                                let rec_str = recipe_desc.clone();
                                                move |_| {
                                                    let loc = *state.locale.read();
                                                    let name = {
                                                        let n = save_mix_name().trim().to_string();
                                                        if n.is_empty() {
                                                            format!("{} {hex_to_save}", loc.tr("Mélange", "Mix"))
                                                        } else {
                                                            n
                                                        }
                                                    };
                                                    let w = *state.workshop.read();
                                                    let svc = svc.clone();
                                                    let hex_to_save = hex_to_save.clone();
                                                    let rec_str = rec_str.clone();
                                                    spawn(async move {
                                                        let mix = PaletteMix {
                                                            id: None,
                                                            name,
                                                            hex_color: hex_to_save,
                                                            recipe: rec_str,
                                                            drying_class: dc,
                                                            mixed_at: Utc::now(),
                                                            workshop: w,
                                                        };
                                                        if svc.mixes.save(&mix).await.is_ok() {
                                                            if let Ok(all) = svc.mixes.all().await {
                                                                state.mixes.set(all);
                                                            }
                                                            save_mix_name.set(String::new());
                                                            state.notify(loc.tr("Mélange posé sur la palette (suivi dans Aujourd'hui).", "Mix placed on palette (tracked in Today)."));
                                                        }
                                                    });
                                                }
                                            },
                                            {locale.tr("+ Poser sur la palette", "+ Place on palette")}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Right column: Inverse Kubelka-Munk solver
                div {
                    div { class: "card",
                        h3 { class: "card-title",
                            {locale.tr("Comment obtenir cette couleur ? (Solveur inverse)", "How to Mix This Color? (Inverse Solver)")}
                        }
                        div { class: "row-between", style: "margin-bottom: 12px;",
                            div { class: "row",
                                div {
                                    label { class: "field-label", {locale.tr("Couleur visée", "Target Color")} }
                                    HexColorInput {
                                        value: target_hex.clone(),
                                        onchange: move |v: String| state.mixer_target_hex.set(v),
                                    }
                                }
                                div {
                                    label { class: "field-label", {locale.tr("Chercher dans", "Search Scope")} }
                                    select {
                                        onchange: move |ev| {
                                            if let Ok(v) = ev.value().parse::<usize>() {
                                                scope_idx.set(v);
                                            }
                                        },
                                        for (idx, pal) in palettes.iter().enumerate() {
                                            option { value: "{idx}", "Palette: {pal.name} ({pal.paints.len()} tubes)" }
                                        }
                                        option { value: "9998", {locale.tr("Mes tubes en stock", "My in-stock tubes")} }
                                        option { value: "9999", {locale.tr("Tout le catalogue", "Full catalog")}, " ({all_paints.len()} tubes)" }
                                    }
                                }
                                div {
                                    label { class: "field-label", {locale.tr("Tubes max", "Max Tubes")} }
                                    select {
                                        value: "{max_paints}",
                                        onchange: move |ev| {
                                            if let Ok(v) = ev.value().parse::<usize>() {
                                                max_paints.set(v);
                                            }
                                        },
                                        option { value: "1", {locale.tr("1 tube seul", "1 tube only")} }
                                        option { value: "2", {locale.tr("2 tubes au plus", "Up to 2 tubes")} }
                                        option { value: "3", {locale.tr("3 tubes au plus", "Up to 3 tubes")} }
                                        option { value: "4", {locale.tr("4 tubes au plus", "Up to 4 tubes")} }
                                        option { value: "5", {locale.tr("5 tubes au plus", "Up to 5 tubes")} }
                                    }
                                }
                            }
                            button {
                                class: "btn-primary",
                                onclick: {
                                    let mut search = run_search.clone();
                                    move |_| search(state.mixer_target_hex.read().clone(), scope_idx(), max_paints())
                                },
                                {locale.tr("Proposer des recettes", "Suggest recipes")}
                            }
                        }

                        if suggestions().is_empty() {
                            p { class: "muted",
                                {locale.tr("Choisissez une teinte cible et cliquez sur « Proposer des recettes » pour calculer les meilleurs mélanges en parts entières.", "Choose a target color and click 'Suggest recipes' to compute the best whole-part mixes.")}
                            }
                        } else {
                            table { class: "data-table",
                                thead {
                                    tr {
                                        th { {locale.tr("Cible / Obtenu", "Target / Achieved")} }
                                        th { {locale.tr("Écart", "Gap")} }
                                        th { {locale.tr("Dosage proposé", "Suggested Ratio")} }
                                        th { "" }
                                    }
                                }
                                tbody {
                                    for sug in suggestions() {
                                        {
                                            let parts_to_load = sug.parts.clone();
                                            rsx! {
                                                tr {
                                                    td {
                                                        div { class: "row",
                                                            ColorSwatch { hex: target_hex.clone(), size: 24 }
                                                            span { class: "muted", "→" }
                                                            ColorSwatch { hex: sug.color.to_hex(), size: 24 }
                                                            span { class: "mono", "{sug.color.to_hex()}" }
                                                        }
                                                    }
                                                    td { DeltaEBadge { delta_e: sug.delta_e } }
                                                    td {
                                                        div { style: "font-weight: 600;", "{sug.describe_i18n(locale)}" }
                                                        div { class: "muted", style: "font-size: 11px;",
                                                            {locale.tr("Séchage", "Drying")},
                                                            " : {sug.drying_class().label_i18n(locale)}"
                                                        }
                                                    }
                                                    td {
                                                        button {
                                                            class: "btn",
                                                            onclick: move |_| {
                                                                manual_parts.set(parts_to_load.clone());
                                                            },
                                                            {locale.tr("← Simuler", "← Simulate")}
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // CIELAB Chromatic Gamut Map (a* / b*)
                    {
                        let gamut_paints: Vec<_> = if let Some(pal) = palettes.get(scope_idx()) {
                            pal.paints.clone()
                        } else if !manual_parts.read().is_empty() {
                            manual_parts.read().iter().map(|p| p.paint.clone()).collect()
                        } else {
                            all_paints.iter().filter(|p| p.in_stock).take(12).cloned().collect()
                        };
                        let best_achieved = suggestions()
                            .first()
                            .map(|s| s.color.to_hex());
                        rsx! {
                            div { class: "card",
                                h3 { class: "card-title",
                                    {locale.tr("Carte de Gamut Chromatique CIELAB (a* / b*)", "CIELAB Chromatic Gamut Map (a* / b*)")}
                                }
                                p { class: "muted", style: "font-size: 11.5px; margin-bottom: 10px;",
                                    {locale.tr("Polygone de couverture chromatique de la palette choisie : visualisez la position de chaque tube (code court) et vérifiez si la couleur visée est atteignable.", "Chromatic coverage polygon of the selected palette: visualize each tube position (short code) and check whether the target color is reachable.")}
                                }
                                GamutMapSvg {
                                    paints: gamut_paints,
                                    target_hex: Some(target_hex.clone()),
                                    achieved_hex: best_achieved,
                                    size: 260,
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
