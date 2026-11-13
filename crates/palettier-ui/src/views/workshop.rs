use crate::state::{AppServices, AppState, Section};
use crate::theme::{
    ColorSwatch, DeltaEBadge, GamutMapSvg, HexColorInput, MixingGuideCard, PaintSwatch,
    SectionHeader, ValueLadderStrip, export_and_open_guide_html,
};
use base64::Engine;
use chrono::Utc;
use dioxus::prelude::*;
use palettier_domain::drying::Workshop;
use palettier_domain::guide::MixingGuideBuilder;
use palettier_domain::i18n::Locale;
use palettier_domain::paint::{DryingClass, PaintSystem, Technique, Ventilation};
use palettier_domain::palette::{Palette, PaletteMix};
use palettier_domain::port::{PaletteMixRepository, PaletteRepository, ProjectRepository};
use palettier_domain::project::{
    LayerKind, ObservedColour, PhotoRole, Project, ProjectLayer, ProjectPhoto, ProjectZone,
};
use std::str::FromStr;
use std::time::Duration;

pub fn format_duration(d: Duration, locale: Locale) -> String {
    let total_mins = d.as_secs() / 60;
    if total_mins == 0 {
        locale.tr("Prêt maintenant", "Ready now").to_string()
    } else if total_mins < 60 {
        format!("{total_mins} min")
    } else {
        let hours = total_mins / 60;
        let mins = total_mins % 60;
        if hours < 48 {
            if mins == 0 {
                format!("{hours} h")
            } else {
                format!("{hours} h {mins:02}")
            }
        } else {
            let days = hours / 24;
            let rem_h = hours % 24;
            let d_unit = locale.tr("j", "d");
            if rem_h == 0 {
                format!("{days} {d_unit}")
            } else {
                format!("{days} {d_unit} {rem_h} h")
            }
        }
    }
}

#[component]
pub fn TodayView() -> Element {
    let mut state = use_context::<AppState>();
    let services = use_context::<AppServices>();

    let locale = *state.locale.read();
    let projects = state.projects.read().clone();
    let mixes = state.mixes.read().clone();
    let workshop = *state.workshop.read();
    let now = Utc::now();

    let bench = services.workbench.bench(&projects, now);
    let mix_states = services.palette_mix_service.states(&mixes, now);

    let mut new_mix_name = use_signal(String::new);
    let mut new_mix_hex = use_signal(|| "#8C6248".to_string());
    let mut new_mix_recipe = use_signal(String::new);
    let mut new_mix_drying = use_signal(|| DryingClass::Medium);

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

            // Studio climate banner
            div { class: "card",
                div { class: "row-between",
                    div {
                        div { class: "card-title", style: "margin-bottom:2px;",
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
                                style: "width:85px;",
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
                                style: "width:85px;",
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
                                    option { value: "{v.as_db_str()}", "{v.label_i18n(locale)}" }
                                }
                            }
                        }
                    }
                }
            }

            div { class: "grid-2",
                // Left column: Pieces on the workbench
                div {
                    div { class: "card",
                        div { class: "row-between", style: "margin-bottom:12px;",
                            h2 { class: "card-title", style: "margin:0;",
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
                            for piece in &bench.pieces {
                                div { class: "card-subtle", style: "margin-bottom:10px;",
                                    div { class: "row-between",
                                        div { class: "row", style: "gap: 6px;",
                                            strong { "{piece.project.name}" }
                                            span {
                                                class: if piece.project.system == PaintSystem::Acrylic { "pill pill-acrylic" } else { "pill pill-oil" },
                                                "{piece.project.system.label_i18n(locale)}"
                                            }
                                        }
                                        span { class: "pill",
                                            "{piece.applied_coats()}/{piece.total_coats()} ", {locale.tr("couches", "coats")}
                                        }
                                    }
                                    if !piece.project.subject.is_empty() {
                                        div { class: "muted", style: "font-size:12px;margin-bottom:6px;", "{piece.project.subject}" }
                                    }
                                    for z in &piece.zones {
                                        div { class: "row-between", style: "padding:4px 0;border-top:1px solid var(--border-subtle);",
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
                                                    "{last.stage.label_i18n(locale)} · ", {locale.tr("suite dans", "next in")}, " {format_duration(z.remaining(), locale)}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Right column: Fresh mixes on the palette
                div {
                    div { class: "card",
                        h2 { class: "card-title",
                            {locale.tr("Mélanges posés sur la palette", "Mixes on the Palette")}, " ({mix_states.len()})"
                        }
                        div { class: "card-subtle", style: "margin-bottom:12px;",
                            div { class: "grid-2", style: "gap:8px;margin-bottom:8px;",
                                div {
                                    label { class: "field-label", {locale.tr("Nom du mélange", "Mix Name")} }
                                    input {
                                        r#type: "text",
                                        style: "width:100%;",
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
                            div { class: "grid-2", style: "gap:8px;margin-bottom:8px;",
                                div {
                                    label { class: "field-label", {locale.tr("Recette", "Recipe")} }
                                    input {
                                        r#type: "text",
                                        style: "width:100%;",
                                        placeholder: locale.tr("2 parts Ocre + 1 part Blanc", "2 parts Ochre + 1 part White"),
                                        value: "{new_mix_recipe}",
                                        oninput: move |ev| new_mix_recipe.set(ev.value())
                                    }
                                }
                                div {
                                    label { class: "field-label", {locale.tr("Vitesse du pigment le plus lent", "Slowest Pigment Speed")} }
                                    select {
                                        style: "width:100%;",
                                        value: "{new_mix_drying().as_db_str()}",
                                        onchange: move |ev| {
                                            if let Ok(dc) = DryingClass::from_str(&ev.value()) {
                                                new_mix_drying.set(dc);
                                            }
                                        },
                                        for dc in DryingClass::ALL {
                                            option { value: "{dc.as_db_str()}", "{dc.label_i18n(locale)}" }
                                        }
                                    }
                                }
                            }
                            button {
                                class: "btn-primary",
                                onclick: {
                                    let svc = services.clone();
                                    move |_| {
                                        let name = new_mix_name().trim().to_string();
                                        if name.is_empty() {
                                            return;
                                        }
                                        let hex = new_mix_hex().trim().to_string();
                                        let recipe = new_mix_recipe().trim().to_string();
                                        let dc = new_mix_drying();
                                        let w = *state.workshop.read();
                                        let svc = svc.clone();
                                        spawn(async move {
                                            let mix = PaletteMix {
                                                id: None,
                                                name,
                                                hex_color: if hex.starts_with('#') { hex } else { format!("#{hex}") },
                                                recipe,
                                                drying_class: dc,
                                                system: *state.paint_system.peek(),
                                                mixed_at: Utc::now(),
                                                workshop: w,
                                            };
                                            if svc.mixes.save(&mix).await.is_ok() {
                                                if let Ok(all) = svc.mixes.all().await {
                                                    state.mixes.set(all);
                                                }
                                                new_mix_name.set(String::new());
                                                new_mix_recipe.set(String::new());
                                            }
                                        });
                                    }
                                },
                                {locale.tr("+ Poser sur la palette", "+ Place on palette")}
                            }
                        }

                        if mix_states.is_empty() {
                            p { class: "muted", {locale.tr("Aucun mélange suivi actuellement.", "No palette mixes currently tracked.")} }
                        } else {
                            for ms in &mix_states {
                                div { class: "card-subtle", style: "margin-bottom:8px;",
                                    div { class: "row-between",
                                        div { class: "row",
                                            ColorSwatch { hex: ms.mix.hex_color.clone(), size: 24 }
                                            div {
                                                strong { "{ms.mix.name}" }
                                                div { class: "muted", style: "font-size:11.5px;", "{ms.mix.recipe}" }
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
                                                    onclick: {
                                                        let svc = services.clone();
                                                        move |_| {
                                                            let svc = svc.clone();
                                                            spawn(async move {
                                                                let _ = svc.mixes.delete(id).await;
                                                                if let Ok(all) = svc.mixes.all().await {
                                                                    state.mixes.set(all);
                                                                }
                                                            });
                                                        }
                                                    },
                                                    {locale.tr("Retirer", "Remove")}
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
pub fn ProjectsView() -> Element {
    let mut state = use_context::<AppState>();
    let services = use_context::<AppServices>();

    let locale = *state.locale.read();
    let projects = state.projects.read().clone();
    let mut new_proj_system = use_signal(|| *state.paint_system.peek());
    // A new piece only offers the palettes of its own system.
    let palettes: Vec<Palette> = state
        .palettes
        .read()
        .iter()
        .filter(|p| p.paints.is_empty() || p.system() == new_proj_system())
        .cloned()
        .collect();
    let mut selected_id = use_signal(|| projects.first().and_then(|p| p.id));
    let mut full_project = use_signal(|| None::<Project>);

    let mut new_proj_name = use_signal(String::new);
    let mut new_proj_subject = use_signal(String::new);
    let mut new_proj_palette_idx = use_signal(|| 0usize);

    let mut new_zone_name = use_signal(String::new);
    let mut new_zone_mat = use_signal(String::new);
    let mut photo_role_to_add = use_signal(|| PhotoRole::Progress);
    let mut progress_observations = use_signal(Vec::<ObservedColour>::new);

    // Inline layer editing state (zone_idx, layer_idx)
    let mut editing_layer = use_signal(|| None::<(usize, usize)>);
    let mut edit_role = use_signal(String::new);
    let mut edit_hex = use_signal(String::new);
    let mut edit_tech = use_signal(String::new);
    let mut edit_note = use_signal(String::new);
    let mut show_project_guide = use_signal(|| false);

    // Load the selected project along with its photos
    let current_id = selected_id();
    let svc_for_effect = services.clone();
    use_effect(use_reactive!(|current_id| {
        let svc = svc_for_effect.clone();
        spawn(async move {
            if let Some(id) = current_id {
                if let Ok(p) = svc.projects.find_with_photos(id).await {
                    full_project.set(p);
                }
            } else {
                full_project.set(None);
            }
            progress_observations.set(Vec::new());
            editing_layer.set(None);
        });
    }));

    let active_proj =
        full_project().or_else(|| projects.iter().find(|p| p.id == selected_id()).cloned());

    // The open piece sets the working system of the other screens.
    let active_system = active_proj.as_ref().map(|p| p.system);
    let svc_for_system = services.clone();
    use_effect(use_reactive!(|active_system| {
        if let Some(sys) = active_system {
            state.set_paint_system(&svc_for_system, sys);
        }
    }));

    rsx! {
        div {
            SectionHeader {
                title: locale.tr("Projets & Pièces", "Projects & Pieces").to_string(),
                subtitle: locale.tr(
                    "Échelles de valeurs par zone, recettes Kubelka-Munk en direct, suivi de pose, journal photo et contrôle d'avancement.",
                    "Per-zone value ladders, live Kubelka-Munk recipes, coat tracking, photo journal, and progress check."
                ).to_string(),
                actions: None
            }

            div { class: "grid-3",
                // Column 1: Project list + creation
                div {
                    div { class: "card",
                        h3 { class: "card-title", {locale.tr("Nouveau projet", "New Project")} }
                        div { style: "margin-bottom:8px;",
                            label { class: "field-label", {locale.tr("Nom de la pièce", "Piece Name")} }
                            input {
                                r#type: "text",
                                style: "width:100%;",
                                placeholder: locale.tr("ex. Grognard de la Garde", "e.g. Old Guard Grenadier"),
                                value: "{new_proj_name}",
                                oninput: move |ev| new_proj_name.set(ev.value())
                            }
                        }
                        div { style: "margin-bottom:8px;",
                            label { class: "field-label", {locale.tr("Sujet / Échelle", "Subject / Scale")} }
                            input {
                                r#type: "text",
                                style: "width:100%;",
                                placeholder: locale.tr("ex. Buste 1/10", "e.g. 1/10 Bust"),
                                value: "{new_proj_subject}",
                                oninput: move |ev| new_proj_subject.set(ev.value())
                            }
                        }
                        div { style: "margin-bottom:8px;",
                            label { class: "field-label", {locale.tr("Peinte à", "Painted in")} }
                            div { class: "system-switch",
                                for sys in PaintSystem::ALL {
                                    button {
                                        class: if new_proj_system() == sys { "toggle-chip active" } else { "toggle-chip" },
                                        onclick: move |_| {
                                            new_proj_system.set(sys);
                                            new_proj_palette_idx.set(0);
                                        },
                                        "{sys.label_i18n(locale)}"
                                    }
                                }
                            }
                            p { class: "muted", style: "font-size:11px; margin-top:4px;",
                                "{new_proj_system().summary_i18n(locale)}"
                            }
                        }
                        div { style: "margin-bottom:10px;",
                            label { class: "field-label", {locale.tr("Palette associée", "Linked Palette")} }
                            select {
                                style: "width:100%;",
                                onchange: move |ev| {
                                    if let Ok(idx) = ev.value().parse::<usize>() {
                                        new_proj_palette_idx.set(idx);
                                    }
                                },
                                for (idx, pal) in palettes.iter().enumerate() {
                                    option { value: "{idx}", "{pal.name} ({pal.paints.len()} tubes)" }
                                }
                            }
                        }
                        button {
                            class: "btn-primary",
                            style: "width:100%;",
                            onclick: {
                                let svc = services.clone();
                                let pals = palettes.clone();
                                move |_| {
                                    let loc = *state.locale.read();
                                    let name = new_proj_name().trim().to_string();
                                    if name.is_empty() {
                                        return;
                                    }
                                    let subject = new_proj_subject().trim().to_string();
                                    let chosen_pal = pals.get(new_proj_palette_idx()).cloned();
                                    let paints = chosen_pal.as_ref().map(|p| p.paints.clone()).unwrap_or_default();
                                    let svc = svc.clone();
                                    spawn(async move {
                                        let proj = Project {
                                            id: None,
                                            name,
                                            subject,
                                            approach: String::new(),
                                            notes: String::new(),
                                            system: new_proj_system(),
                                            palette: chosen_pal,
                                            paints,
                                            zones: vec![ProjectZone {
                                                id: None,
                                                name: loc.tr("Visage", "Face").to_string(),
                                                material: loc.tr("Peau", "Skin").to_string(),
                                                note: String::new(),
                                                layers: vec![
                                                    ProjectLayer::new(loc.tr("Ombre 2", "Shadow 2"), "#4A2E24"),
                                                    ProjectLayer::new(loc.tr("Ombre 1", "Shadow 1"), "#7D5140"),
                                                    ProjectLayer::new("Base", "#B87E63"),
                                                    ProjectLayer::new(loc.tr("Lumiere 1", "Highlight 1"), "#D9A68B"),
                                                    ProjectLayer::new(loc.tr("Lumiere 2", "Highlight 2"), "#F2CFB8"),
                                                ],
                                            }],
                                            photos: Vec::new(),
                                            created_at: Utc::now(),
                                        };
                                        if let Ok(saved) = svc.projects.save(&proj).await {
                                            if let Ok(all) = svc.projects.all().await {
                                                state.projects.set(all);
                                            }
                                            selected_id.set(saved.id);
                                            full_project.set(Some(saved));
                                            new_proj_name.set(String::new());
                                            new_proj_subject.set(String::new());
                                        }
                                    });
                                }
                            },
                            {locale.tr("+ Créer le projet", "+ Create project")}
                        }
                    }

                    div { class: "card",
                        h3 { class: "card-title", {locale.tr("Mes pièces", "My Pieces")}, " ({projects.len()})" }
                        for p in &projects {
                            div {
                                class: "card-subtle",
                                style: if p.id == selected_id() {
                                    "margin-bottom:8px;border-color:var(--accent);cursor:pointer;"
                                } else {
                                    "margin-bottom:8px;cursor:pointer;"
                                },
                                onclick: {
                                    let pid = p.id;
                                    move |_| selected_id.set(pid)
                                },
                                div { class: "row-between", style: "align-items: center;",
                                    div {
                                        strong { "{p.name}" }
                                        if !p.subject.is_empty() {
                                            div { class: "muted", style: "font-size:12px;", "{p.subject}" }
                                        }
                                    }
                                    div { class: "row", style: "gap: 6px; align-items: center;",
                                        span { class: "pill", "{p.zones.len()} zones" }
                                        button {
                                            class: "btn-danger",
                                            style: "padding: 3px 7px; font-size: 11px;",
                                            title: locale.tr("Supprimer définitivement ce projet", "Delete this project permanently"),
                                            onclick: {
                                                let svc = services.clone();
                                                let pid = p.id;
                                                let pname = p.name.clone();
                                                move |ev: Event<MouseData>| {
                                                    ev.stop_propagation();
                                                    let svc = svc.clone();
                                                    let pname = pname.clone();
                                                    spawn(async move {
                                                        if let Some(id) = pid {
                                                            let _ = svc.projects.delete(id).await;
                                                            if let Ok(all) = svc.projects.all().await {
                                                                selected_id.set(all.first().and_then(|pr| pr.id));
                                                                state.projects.set(all);
                                                                state.notify(format!("Projet « {pname} » supprimé."));
                                                            }
                                                        }
                                                    });
                                                }
                                            },
                                            "🗑️"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Columns 2 & 3: Active project details
                div { style: "grid-column: span 2;",
                    if let Some(proj) = active_proj {
                        div { class: "card",
                            div { class: "row-between", style: "margin-bottom:12px;",
                                div {
                                    h2 { class: "card-title", style: "font-size:19px;margin-bottom:2px;",
                                        "{proj.name} "
                                        span {
                                            class: if proj.system == PaintSystem::Acrylic { "pill pill-acrylic" } else { "pill pill-oil" },
                                            title: "{proj.system.summary_i18n(locale)}",
                                            "{proj.system.label_i18n(locale)}"
                                        }
                                    }
                                    span { class: "muted",
                                        "{locale.tr(\"Sujet\", \"Subject\")} : {proj.subject} · {proj.effective_paints().len()} ",
                                        {locale.tr("tubes figés", "locked tubes")}
                                    }
                                }
                                div { class: "row project-toolbar",
                                    button {
                                        class: if *show_project_guide.read() { "btn-primary" } else { "btn" },
                                        onclick: move |_| {
                                            let cur = *show_project_guide.read();
                                            show_project_guide.set(!cur);
                                        },
                                        if *show_project_guide.read() {
                                            {locale.tr("← Retour aux couches", "← Back to layers")}
                                        } else {
                                            {locale.tr("Guide de mélanges (Norma Blue)", "Mixing Guide (Norma Blue)")}
                                        }
                                    }
                                    button {
                                        class: "btn",
                                        onclick: {
                                            let svc = services.clone();
                                            let p_export = proj.clone();
                                            move |_| {
                                                let loc = *state.locale.read();
                                                let guide = svc.guide_builder.from_project_i18n(&p_export, loc);
                                                let photo_uri = p_export
                                                    .photos
                                                    .iter()
                                                    .find(|ph| ph.role == PhotoRole::Piece || ph.role == PhotoRole::Reference)
                                                    .or_else(|| p_export.photos.first())
                                                    .map(|ph| {
                                                        let b64 = base64::engine::general_purpose::STANDARD.encode(&ph.data);
                                                        format!("data:image/jpeg;base64,{b64}")
                                                    });
                                                match export_and_open_guide_html(&guide, photo_uri.as_deref(), loc) {
                                                    Ok(path) => {
                                                        let msg = match loc {
                                                            Locale::Fr => format!("Guide de mélanges A4 ouvert : {}", path.display()),
                                                            Locale::En => format!("A4 Mixing Guide opened: {}", path.display()),
                                                        };
                                                        state.notify(msg);
                                                    }
                                                    Err(e) => state.notify(e),
                                                }
                                            }
                                        },
                                        {locale.tr("Imprimer / Exporter Guide A4", "Print / Export A4 Guide")}
                                    }
                                    span { class: "toolbar-sep" }
                                    select {
                                        value: "{photo_role_to_add().as_db_str()}",
                                        onchange: move |e| {
                                            if let Ok(r) = PhotoRole::from_str(&e.value()) {
                                                photo_role_to_add.set(r);
                                            }
                                        },
                                        for r in PhotoRole::ALL {
                                            option { value: "{r.as_db_str()}", "Photo: {r.label_i18n(locale)}" }
                                        }
                                    }
                                    button {
                                        class: "btn",
                                        onclick: {
                                            let svc = services.clone();
                                            let pid = proj.id;
                                            let p_for_check = proj.clone();
                                            move |_| {
                                                let svc = svc.clone();
                                                let p_for_check = p_for_check.clone();
                                                let role = *photo_role_to_add.read();
                                                spawn(async move {
                                                    let loc = *state.locale.read();
                                                    if let Some(id) = pid
                                                        && let Some(file) = rfd::AsyncFileDialog::new()
                                                            .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
                                                            .pick_file()
                                                            .await
                                                            && let Ok(bytes) = std::fs::read(file.path()) {
                                                                if role == PhotoRole::Progress
                                                                    && let Ok(doms) = svc.decoder.dominant_colours(&bytes, 6) {
                                                                        let obs = svc.progress_service.compare(&p_for_check, &doms);
                                                                        progress_observations.set(obs);
                                                                    }
                                                                let photo = ProjectPhoto {
                                                                    id: None,
                                                                    data: bytes,
                                                                    role,
                                                                    caption: file.file_name(),
                                                                    added_at: Utc::now(),
                                                                };
                                                                if let Ok(updated) = svc.projects.add_photo(id, &photo).await {
                                                                    full_project.set(Some(updated));
                                                                    state.notify(loc.tr("Photo ajoutée au journal de la pièce.", "Photo added to piece journal."));
                                                                }
                                                            }
                                                });
                                            }
                                        },
                                        {locale.tr("+ Ajouter photo", "+ Add photo")}
                                    }
                                    button {
                                        class: "btn",
                                        onclick: {
                                            let svc = services.clone();
                                            let p_card = proj.clone();
                                            move |_| {
                                                let loc = *state.locale.read();
                                                let photo_bytes = p_card
                                                    .photos
                                                    .iter()
                                                    .find(|ph| ph.role == PhotoRole::Piece || ph.role == PhotoRole::Reference)
                                                    .or_else(|| p_card.photos.first())
                                                    .map(|ph| ph.data.as_slice());

                                                let swatches: Vec<String> = p_card
                                                    .effective_paints()
                                                    .iter()
                                                    .map(|p| p.hex_color.clone())
                                                    .collect();

                                                if let Ok(card_png) = svc.decoder.render_social_recipe_card(photo_bytes, &swatches)
                                                    && let Some(dest) = rfd::FileDialog::new()
                                                        .add_filter("PNG Image", &["png"])
                                                        .set_file_name(format!("{}-recipe-card.png", p_card.name))
                                                        .save_file()
                                                        && std::fs::write(&dest, card_png).is_ok() {
                                                            state.notify(loc.tr(
                                                                "Carte recette PNG exportée avec succès !",
                                                                "Recipe card PNG exported successfully!"
                                                            ));
                                                        }
                                            }
                                        },
                                        "📷 "
                                        {locale.tr("Carte PNG", "PNG Card")}
                                    }
                                    span { class: "toolbar-spacer" }
                                    button {
                                        class: "btn-danger",
                                        onclick: {
                                            let svc = services.clone();
                                            let pid = proj.id;
                                            move |_| {
                                                let svc = svc.clone();
                                                spawn(async move {
                                                    if let Some(id) = pid {
                                                        let _ = svc.projects.delete(id).await;
                                                        if let Ok(all) = svc.projects.all().await {
                                                            selected_id.set(all.first().and_then(|p| p.id));
                                                            state.projects.set(all);
                                                        }
                                                    }
                                                });
                                            }
                                        },
                                        {locale.tr("Supprimer", "Delete")}
                                    }
                                }
                            }

                            if *show_project_guide.read() {
                                {
                                    let guide = services.guide_builder.from_project_i18n(&proj, locale);
                                    let photo_uri = proj
                                        .photos
                                        .iter()
                                        .find(|ph| ph.role == PhotoRole::Piece || ph.role == PhotoRole::Reference)
                                        .or_else(|| proj.photos.first())
                                        .map(|ph| {
                                            let b64 = base64::engine::general_purpose::STANDARD.encode(&ph.data);
                                            format!("data:image/jpeg;base64,{b64}")
                                        });
                                    rsx! {
                                        div { style: "margin-bottom: 18px;",
                                            MixingGuideCard {
                                                guide: guide,
                                                photo_data_uri: photo_uri,
                                                on_notify: move |msg: String| state.notify(msg),
                                            }
                                        }
                                    }
                                }
                            }

                            // Locked project paints banner + Sync button if palette changed
                            if !proj.effective_paints().is_empty() {
                                div { class: "card-subtle", style: "margin-bottom: 14px;",
                                    div { class: "row-between", style: "margin-bottom: 8px;",
                                        span { class: "field-label", style: "margin: 0;",
                                            {locale.tr("Tubes du projet", "Project Tubes")}, " ({proj.effective_paints().len()})"
                                        }
                                        if proj.diverges_from_palette() {
                                            button {
                                                class: "btn-primary",
                                                onclick: {
                                                    let svc = services.clone();
                                                    let mut p_sync = proj.clone();
                                                    move |_| {
                                                        if let Some(ref pal) = p_sync.palette {
                                                            p_sync.paints = pal.paints.clone();
                                                        }
                                                        let svc = svc.clone();
                                                        let p_sync = p_sync.clone();
                                                        spawn(async move {
                                                            let loc = *state.locale.read();
                                                            if let Ok(saved) = svc.projects.save(&p_sync).await {
                                                                state.reload_all(&svc).await;
                                                                if let Some(id) = saved.id
                                                                    && let Ok(full) = svc.projects.find_with_photos(id).await {
                                                                        full_project.set(full);
                                                                    }
                                                                state.notify(loc.tr("Palette du projet synchronisée avec la composition actuelle.", "Project palette synced with current composition."));
                                                            }
                                                        });
                                                    }
                                                },
                                                {locale.tr("Reprendre la palette actuelle", "Sync with current palette")}
                                            }
                                        }
                                    }
                                    div { class: "row",
                                        for pt in proj.effective_paints() {
                                            div { class: "pill", style: "gap: 6px; padding: 3px 9px;",
                                                PaintSwatch {
                                                    masstone_hex: pt.hex_color.clone(),
                                                    tint_hex: pt.tint_hex.clone(),
                                                    width: 22,
                                                    height: 14,
                                                }
                                                span { "{pt.name}" }
                                                if !pt.in_stock {
                                                    span { style: "color: var(--danger); font-size: 10px;",
                                                        " (", {locale.tr("hors stock", "out of stock")}, ")"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // Fat-over-lean & missing tube substitutes
                            {
                                let risks = services.fat_over_lean.inspect_project_i18n(&proj, locale);
                                let owned_paints: Vec<_> = state
                                    .paints
                                    .read()
                                    .iter()
                                    .filter(|p| p.in_stock)
                                    .cloned()
                                    .collect();
                                let missing = services.substitute_service.missing_from(&proj, &owned_paints);
                                rsx! {
                                    if !risks.is_empty() {
                                        div { class: "card-subtle", style: "border-color:var(--pill-warn-border);margin-bottom:14px;",
                                            strong { style: "color:var(--pill-warn-text);",
                                                {locale.tr("Avertissements Gras sur Maigre", "Fat Over Lean Warnings")}, " ({risks.len()})"
                                            }
                                            for r in risks {
                                                div { class: "muted", style: "font-size:12px;margin-top:4px;", "• {r.where_name} ({r.under} → {r.over}) : {r.explanation}" }
                                            }
                                        }
                                    }
                                    if !missing.is_empty() {
                                        div { class: "card-subtle", style: "border-color:var(--accent);margin-bottom:14px;",
                                            strong {
                                                {locale.tr("Tubes hors stock et substituts sur votre étagère", "Out-of-stock tubes & substitutes in your drawer")}, " ({missing.len()})"
                                            }
                                            for m in missing {
                                                {
                                                    let absent_id = m.paint.id;
                                                    let replacement_opt = m.nearest.clone();
                                                    let p_for_sub = proj.clone();
                                                    let svc = services.clone();
                                                    rsx! {
                                                        div { class: "row-between", style: "margin-top:8px; padding-top:6px; border-top:1px solid var(--border-subtle);",
                                                            div { class: "row",
                                                                ColorSwatch { hex: m.paint.hex_color.clone(), size: 20 }
                                                                strong { "{m.paint.display_name()}" }
                                                                span { class: "muted", "→" }
                                                                if let Some(ref near) = m.nearest {
                                                                    ColorSwatch { hex: near.hex_color.clone(), size: 20 }
                                                                    DeltaEBadge { delta_e: m.delta_e }
                                                                }
                                                                span { class: "muted", style: "font-size:12px;", "{m.verdict_i18n(locale)}" }
                                                            }
                                                            if let Some(replacement) = replacement_opt {
                                                                button {
                                                                    class: "btn-primary",
                                                                    style: "padding: 3px 10px; font-size: 11.5px;",
                                                                    onclick: move |_| {
                                                                        let mut updated = p_for_sub.clone();
                                                                        if updated.paints.is_empty()
                                                                            && let Some(ref pal) = updated.palette {
                                                                                updated.paints = pal.paints.clone();
                                                                            }
                                                                        for pt in &mut updated.paints {
                                                                            if pt.id == absent_id {
                                                                                *pt = replacement.clone();
                                                                            }
                                                                        }
                                                                        let svc = svc.clone();
                                                                        spawn(async move {
                                                                            let loc = *state.locale.read();
                                                                            if let Ok(saved) = svc.projects.save(&updated).await {
                                                                                state.reload_all(&svc).await;
                                                                                if let Some(id) = saved.id
                                                                                    && let Ok(full) = svc.projects.find_with_photos(id).await {
                                                                                        full_project.set(full);
                                                                                    }
                                                                                state.notify(loc.tr("Tube hors stock remplacé par son équivalent en stock.", "Out-of-stock tube replaced with in-stock substitute."));
                                                                            }
                                                                        });
                                                                    },
                                                                    {locale.tr("Remplacer dans le projet", "Replace in project")}
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

                            // Zones and layers with ValueLadderStrip + live Kubelka-Munk recipe + inline edit
                            for (z_idx, zone) in proj.zones.iter().enumerate() {
                                {
                                    let ladder_steps: Vec<(String, String)> = zone
                                        .layers
                                        .iter()
                                        .filter(|l| l.kind == LayerKind::Ladder)
                                        .map(|l| (l.role.clone(), l.target_hex.clone()))
                                        .collect();
                                    let eff_paints = proj.effective_paints();

                                    rsx! {
                                        div { class: "card-subtle", style: "margin-bottom:14px;",
                                            div { class: "row-between", style: "margin-bottom:4px;",
                                                div {
                                                    strong { style: "font-size:14.5px;", "{zone.name}" }
                                                    if !zone.material.is_empty() {
                                                        span { class: "muted", " · {zone.material}" }
                                                    }
                                                }
                                                div { class: "row",
                                                    span { class: "pill",
                                                        "{zone.layers.iter().filter(|l| l.is_applied()).count()}/{zone.layers.len()} ", {locale.tr("posées", "applied")}
                                                    }
                                                    button {
                                                        class: "btn",
                                                        style: "padding: 3px 9px; font-size: 11.5px;",
                                                        onclick: {
                                                            let svc = services.clone();
                                                            let mut p_clone = proj.clone();
                                                            move |_| {
                                                                let loc = *state.locale.read();
                                                                if let Some(z) = p_clone.zones.get_mut(z_idx) {
                                                                    let mut nl = ProjectLayer::new("Accent", "#C89B6E");
                                                                    nl.kind = LayerKind::Accent;
                                                                    nl.technique = Technique::Glaze.label_i18n(loc).to_string();
                                                                    z.layers.push(nl);
                                                                }
                                                                let svc = svc.clone();
                                                                let p_to_save = p_clone.clone();
                                                                spawn(async move {
                                                                    if let Ok(saved) = svc.projects.save(&p_to_save).await {
                                                                        if let Ok(all) = svc.projects.all().await {
                                                                            state.projects.set(all);
                                                                        }
                                                                        if let Some(id) = saved.id
                                                                            && let Ok(full) = svc.projects.find_with_photos(id).await {
                                                                                full_project.set(full);
                                                                            }
                                                                    }
                                                                });
                                                            }
                                                        },
                                                        {locale.tr("+ Couche", "+ Coat")}
                                                    }
                                                }
                                            }

                                            ValueLadderStrip { steps: ladder_steps }

                                            table { class: "data-table",
                                                thead {
                                                    tr {
                                                        th { {locale.tr("Rôle", "Role")} }
                                                        th { {locale.tr("Cible", "Target")} }
                                                        th { {locale.tr("Recette Kubelka-Munk & Technique", "Kubelka-Munk Recipe & Technique")} }
                                                        th { {locale.tr("État de pose", "Coat Status")} }
                                                        th { "" }
                                                    }
                                                }
                                                tbody {
                                                    for (l_idx, layer) in zone.layers.iter().enumerate() {
                                                        {
                                                            let is_editing = *editing_layer.read() == Some((z_idx, l_idx));
                                                            let computed = services.planner.recompute(layer, eff_paints);
                                                            rsx! {
                                                                tr {
                                                                    td {
                                                                        if is_editing {
                                                                            input {
                                                                                r#type: "text",
                                                                                style: "width: 110px;",
                                                                                value: "{edit_role}",
                                                                                oninput: move |e| edit_role.set(e.value())
                                                                            }
                                                                        } else {
                                                                            strong { "{layer.role}" }
                                                                            if layer.kind == LayerKind::Accent {
                                                                                span { class: "pill pill-accent", style: "margin-left:6px;", "Accent" }
                                                                            }
                                                                        }
                                                                    }
                                                                    td {
                                                                        if is_editing {
                                                                            HexColorInput {
                                                                                value: edit_hex(),
                                                                                onchange: move |v: String| edit_hex.set(v),
                                                                            }
                                                                        } else {
                                                                            div { class: "row",
                                                                                ColorSwatch { hex: layer.target_hex.clone(), size: 22 }
                                                                                span { class: "mono", "{layer.target_hex}" }
                                                                            }
                                                                        }
                                                                    }
                                                                    td {
                                                                        if is_editing {
                                                                            div { class: "row",
                                                                                select {
                                                                                    value: "{edit_tech}",
                                                                                    onchange: move |e| edit_tech.set(e.value()),
                                                                                    // The piece's own techniques first; on an acrylic piece the oil
                                                                                    // techniques stay available (oil over dry acrylic is sound),
                                                                                    // never the reverse.
                                                                                    for t in Technique::for_system(proj.system) {
                                                                                        option { value: "{t.label_i18n(locale)}", "{t.label_i18n(locale)}" }
                                                                                    }
                                                                                    if proj.system == PaintSystem::Acrylic {
                                                                                        optgroup { label: "{locale.tr(\"À l'huile, par-dessus\", \"In oil, on top\")}",
                                                                                            for t in Technique::ALL.into_iter().filter(|t| t.native_system() == Some(PaintSystem::Oil)) {
                                                                                                option { value: "{t.label_i18n(locale)}", "{t.label_i18n(locale)}" }
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                }
                                                                                input {
                                                                                    r#type: "text",
                                                                                    placeholder: "Note",
                                                                                    value: "{edit_note}",
                                                                                    oninput: move |e| edit_note.set(e.value())
                                                                                }
                                                                            }
                                                                        } else {
                                                                            if let Some(ref rec) = computed.recipe {
                                                                                div { class: "row",
                                                                                    ColorSwatch { hex: computed.achieved.to_hex(), size: 18 }
                                                                                    strong { style: "font-size: 12.5px;", "{rec.describe_i18n(locale)}" }
                                                                                    DeltaEBadge { delta_e: computed.delta_e }
                                                                                }
                                                                            }
                                                                            div { class: "muted", style: "font-size:11.5px; margin-top: 2px;",
                                                                                "{layer.technique}"
                                                                                if !layer.note.is_empty() {
                                                                                    " · {layer.note}"
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                    td {
                                                                        if let Some(ref app) = layer.applied {
                                                                            span {
                                                                                class: "pill pill-ok",
                                                                                {locale.tr("Posée", "Applied")}, " ({app.workshop.temperature_celsius:.0}°C)"
                                                                            }
                                                                        } else {
                                                                            span { class: "pill", {locale.tr("À peindre", "Pending")} }
                                                                        }
                                                                    }
                                                                    td {
                                                                        div { class: "row",
                                                                            if is_editing {
                                                                                button {
                                                                                    class: "btn-primary",
                                                                                    onclick: {
                                                                                        let svc = services.clone();
                                                                                        let mut p_clone = proj.clone();
                                                                                        move |_| {
                                                                                            if let Some(z) = p_clone.zones.get_mut(z_idx)
                                                                                                && let Some(l) = z.layers.get_mut(l_idx) {
                                                                                                    l.role = edit_role().trim().to_string();
                                                                                                    l.target_hex = edit_hex().trim().to_string();
                                                                                                    l.technique = edit_tech().trim().to_string();
                                                                                                    l.note = edit_note().trim().to_string();
                                                                                                }
                                                                                            editing_layer.set(None);
                                                                                            let svc = svc.clone();
                                                                                            let p_to_save = p_clone.clone();
                                                                                            spawn(async move {
                                                                                                if let Ok(saved) = svc.projects.save(&p_to_save).await {
                                                                                                    if let Ok(all) = svc.projects.all().await {
                                                                                                        state.projects.set(all);
                                                                                                    }
                                                                                                    if let Some(id) = saved.id
                                                                                                        && let Ok(full) = svc.projects.find_with_photos(id).await {
                                                                                                            full_project.set(full);
                                                                                                        }
                                                                                                }
                                                                                            });
                                                                                        }
                                                                                    },
                                                                                    {locale.tr("Valider", "Save")}
                                                                                }
                                                                                button {
                                                                                    class: "btn",
                                                                                    onclick: move |_| editing_layer.set(None),
                                                                                    {locale.tr("Annuler", "Cancel")}
                                                                                }
                                                                            } else {
                                                                                button {
                                                                                    class: "btn",
                                                                                    onclick: {
                                                                                        let svc = services.clone();
                                                                                        let mut p_clone = proj.clone();
                                                                                        let is_applied = layer.is_applied();
                                                                                        move |_| {
                                                                                            let w = *state.workshop.read();
                                                                                            let slowest = p_clone
                                                                                                .effective_paints()
                                                                                                .iter()
                                                                                                .fold(DryingClass::Medium, |acc, pt| acc.slowest(pt.drying_class));
                                                                                            if let Some(z) = p_clone.zones.get_mut(z_idx)
                                                                                                && let Some(l) = z.layers.get_mut(l_idx) {
                                                                                                    *l = if is_applied {
                                                                                                        l.clear_applied()
                                                                                                    } else {
                                                                                                        l.mark_applied(Utc::now(), w, slowest)
                                                                                                    };
                                                                                                }
                                                                                            let svc = svc.clone();
                                                                                            let p_to_save = p_clone.clone();
                                                                                            spawn(async move {
                                                                                                if let Ok(saved) = svc.projects.save(&p_to_save).await {
                                                                                                    if let Ok(all) = svc.projects.all().await {
                                                                                                        state.projects.set(all);
                                                                                                    }
                                                                                                    if let Some(id) = saved.id
                                                                                                        && let Ok(full) = svc.projects.find_with_photos(id).await {
                                                                                                            full_project.set(full);
                                                                                                        }
                                                                                                }
                                                                                            });
                                                                                        }
                                                                                    },
                                                                                    if layer.is_applied() {
                                                                                        {locale.tr("Annuler pose", "Unmark coat")}
                                                                                    } else {
                                                                                        {locale.tr("Marquer posée", "Mark applied")}
                                                                                    }
                                                                                }
                                                                                button {
                                                                                    class: "btn",
                                                                                    onclick: {
                                                                                        let r = layer.role.clone();
                                                                                        let h = layer.target_hex.clone();
                                                                                        let t = if layer.technique.is_empty() {
                                                                                            Technique::Glaze.label_i18n(locale).to_string()
                                                                                        } else {
                                                                                            layer.technique.clone()
                                                                                        };
                                                                                        let n = layer.note.clone();
                                                                                        move |_| {
                                                                                            edit_role.set(r.clone());
                                                                                            edit_hex.set(h.clone());
                                                                                            edit_tech.set(t.clone());
                                                                                            edit_note.set(n.clone());
                                                                                            editing_layer.set(Some((z_idx, l_idx)));
                                                                                        }
                                                                                    },
                                                                                    {locale.tr("Éditer", "Edit")}
                                                                                }
                                                                                button {
                                                                                    class: "btn",
                                                                                    onclick: {
                                                                                        let hex = layer.target_hex.clone();
                                                                                        move |_| {
                                                                                            state.mixer_target_hex.set(hex.clone());
                                                                                            state.section.set(Section::Mixer);
                                                                                        }
                                                                                    },
                                                                                    {locale.tr("Mélanger", "Mix")}
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

                            // Add zone
                            div { class: "row", style: "margin-top:10px;",
                                input {
                                    r#type: "text",
                                    placeholder: locale.tr("Nouvelle zone (ex. Manteau)", "New zone (e.g. Coat)"),
                                    value: "{new_zone_name}",
                                    oninput: move |ev| new_zone_name.set(ev.value())
                                }
                                input {
                                    r#type: "text",
                                    placeholder: locale.tr("Matière (ex. Drap de laine)", "Material (e.g. Wool cloth)"),
                                    value: "{new_zone_mat}",
                                    oninput: move |ev| new_zone_mat.set(ev.value())
                                }
                                button {
                                    class: "btn",
                                    onclick: {
                                        let svc = services.clone();
                                        let mut p_clone = proj.clone();
                                        move |_| {
                                            let loc = *state.locale.read();
                                            let zname = new_zone_name().trim().to_string();
                                            if zname.is_empty() {
                                                return;
                                            }
                                            let zmat = new_zone_mat().trim().to_string();
                                            p_clone.zones.push(ProjectZone {
                                                id: None,
                                                name: zname,
                                                material: zmat,
                                                note: String::new(),
                                                layers: vec![
                                                    ProjectLayer::new(loc.tr("Ombre 2", "Shadow 2"), "#2B303A"),
                                                    ProjectLayer::new(loc.tr("Ombre 1", "Shadow 1"), "#434A59"),
                                                    ProjectLayer::new("Base", "#5E687D"),
                                                    ProjectLayer::new(loc.tr("Lumiere 1", "Highlight 1"), "#8591A8"),
                                                    ProjectLayer::new(loc.tr("Lumiere 2", "Highlight 2"), "#B2BDD1"),
                                                ],
                                            });
                                            let svc = svc.clone();
                                            let p_to_save = p_clone.clone();
                                            spawn(async move {
                                                if let Ok(saved) = svc.projects.save(&p_to_save).await {
                                                    if let Ok(all) = svc.projects.all().await {
                                                        state.projects.set(all);
                                                    }
                                                    if let Some(id) = saved.id
                                                        && let Ok(full) = svc.projects.find_with_photos(id).await {
                                                            full_project.set(full);
                                                        }
                                                    new_zone_name.set(String::new());
                                                    new_zone_mat.set(String::new());
                                                }
                                            });
                                        }
                                    },
                                    {locale.tr("+ Ajouter la zone", "+ Add zone")}
                                }
                            }

                            // Progress check observations
                            if !progress_observations.read().is_empty() {
                                div { class: "card-subtle", style: "margin-top: 16px; border-color: var(--accent);",
                                    div { class: "row-between", style: "margin-bottom: 8px;",
                                        strong { {locale.tr("Comparaison d'avancement sur la photo relevée", "Progress comparison on sampled photo")} }
                                        button {
                                            class: "btn",
                                            onclick: move |_| progress_observations.set(Vec::new()),
                                            {locale.tr("Fermer", "Close")}
                                        }
                                    }
                                    for obs in progress_observations.read().iter() {
                                        div { class: "row-between", style: "padding: 5px 0; border-top: 1px solid var(--border-subtle);",
                                            div { class: "row",
                                                ColorSwatch { hex: obs.measured.to_hex(), size: 22 }
                                                if let Some(target) = obs.target {
                                                    span { class: "muted", "vs" }
                                                    ColorSwatch { hex: target.to_hex(), size: 22 }
                                                }
                                                span { style: "font-size: 12.5px;", "{obs.verdict_i18n(locale)}" }
                                            }
                                            if obs.matches_plan() {
                                                DeltaEBadge { delta_e: obs.delta_e }
                                            }
                                        }
                                    }
                                }
                            }

                            // Photo journal with thumbnails
                            if !proj.photos.is_empty() {
                                div { style: "margin-top:16px;",
                                    h3 { class: "card-title", {locale.tr("Journal photo", "Photo Journal")}, " ({proj.photos.len()})" }
                                    div { class: "photo-grid",
                                        for ph in &proj.photos {
                                            {
                                                let b64 = base64::engine::general_purpose::STANDARD.encode(&ph.data);
                                                let src = format!("data:image/jpeg;base64,{b64}");
                                                let ph_data = ph.data.clone();
                                                let p_for_cmp = proj.clone();
                                                let ph_id = ph.id;
                                                let proj_id = proj.id;
                                                rsx! {
                                                    div { class: "card-subtle",
                                                        img {
                                                            class: "photo-thumb",
                                                            src: "{src}",
                                                            alt: "{ph.caption}",
                                                        }
                                                        div { class: "row-between",
                                                            strong { style: "font-size: 12px; overflow: hidden; text-overflow: ellipsis;", "{ph.caption}" }
                                                            span { class: "pill", "{ph.role.label_i18n(locale)}" }
                                                        }
                                                        div { class: "row", style: "margin-top: 6px;",
                                                            button {
                                                                class: "btn",
                                                                style: "padding: 3px 8px; font-size: 11px;",
                                                                onclick: {
                                                                    let svc = services.clone();
                                                                    move |_| {
                                                                        if let Ok(doms) = svc.decoder.dominant_colours(&ph_data, 6) {
                                                                            let obs = svc.progress_service.compare(&p_for_cmp, &doms);
                                                                            progress_observations.set(obs);
                                                                        }
                                                                    }
                                                                },
                                                                {locale.tr("Comparer au plan", "Compare to plan")}
                                                            }
                                                            if let (Some(pid), Some(fid)) = (proj_id, ph_id) {
                                                                button {
                                                                    class: "btn-danger",
                                                                    style: "padding: 3px 8px; font-size: 11px;",
                                                                    onclick: {
                                                                        let svc = services.clone();
                                                                        move |_| {
                                                                            let svc = svc.clone();
                                                                            spawn(async move {
                                                                                if let Ok(updated) = svc.projects.remove_photo(pid, fid).await {
                                                                                    full_project.set(Some(updated));
                                                                                }
                                                                            });
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
                    } else {
                        div { class: "card",
                            p { class: "muted", {locale.tr("Sélectionnez ou créez un projet dans la colonne de gauche.", "Select or create a project in the left column.")} }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn PalettesView() -> Element {
    let mut state = use_context::<AppState>();
    let services = use_context::<AppServices>();

    let locale = *state.locale.read();
    let palettes: Vec<Palette> = {
        let system = *state.paint_system.read();
        state
            .palettes
            .read()
            .iter()
            .filter(|p| p.paints.is_empty() || p.system() == system)
            .cloned()
            .collect()
    };
    let all_paints = state.paints_of_system();

    let mut selected_idx = use_signal(|| 0usize);
    let mut new_name = use_signal(String::new);
    let mut new_purpose = use_signal(String::new);
    let mut new_notes = use_signal(String::new);
    let mut paint_to_add_idx = use_signal(|| 0usize);
    let mut show_matrix = use_signal(|| false);

    let active_pal = palettes.get(selected_idx()).cloned();

    rsx! {
        div {
            SectionHeader {
                title: locale.tr("Palettes de travail", "Working Palettes").to_string(),
                subtitle: locale.tr(
                    "Sélections cohérentes de tubes pour vos sujets (Zorn, carnations, blindages...).",
                    "Curated tube selections for your subjects (Zorn, skin tones, armor...)."
                ).to_string(),
                actions: None
            }

            div { class: "grid-3",
                div {
                    div { class: "card",
                        h3 { class: "card-title", {locale.tr("Nouvelle palette", "New Palette")} }
                        div { style: "margin-bottom:8px;",
                            label { class: "field-label", {locale.tr("Nom", "Name")} }
                            input {
                                r#type: "text",
                                style: "width:100%;",
                                placeholder: locale.tr("ex. Drap impérial bleu", "e.g. Imperial Blue Cloth"),
                                value: "{new_name}",
                                oninput: move |ev| new_name.set(ev.value())
                            }
                        }
                        div { style: "margin-bottom:8px;",
                            label { class: "field-label", {locale.tr("Usage / Sujet", "Purpose / Subject")} }
                            input {
                                r#type: "text",
                                style: "width:100%;",
                                placeholder: locale.tr("ex. Uniformes 75mm", "e.g. 75mm Uniforms"),
                                value: "{new_purpose}",
                                oninput: move |ev| new_purpose.set(ev.value())
                            }
                        }
                        div { style: "margin-bottom:10px;",
                            label { class: "field-label", {locale.tr("Notes d'atelier", "Studio Notes")} }
                            textarea {
                                style: "width:100%;height:60px;",
                                value: "{new_notes}",
                                oninput: move |ev| new_notes.set(ev.value())
                            }
                        }
                        button {
                            class: "btn-primary",
                            style: "width:100%;",
                            onclick: {
                                let svc = services.clone();
                                move |_| {
                                    let name = new_name().trim().to_string();
                                    if name.is_empty() {
                                        return;
                                    }
                                    let purpose = new_purpose().trim().to_string();
                                    let notes = new_notes().trim().to_string();
                                    let svc = svc.clone();
                                    spawn(async move {
                                        let pal = Palette {
                                            id: None,
                                            name,
                                            purpose,
                                            notes,
                                            paints: Vec::new(),
                                        };
                                        if svc.palettes.save(&pal).await.is_ok() {
                                            if let Ok(all) = svc.palettes.all().await {
                                                state.palettes.set(all);
                                            }
                                            new_name.set(String::new());
                                            new_purpose.set(String::new());
                                            new_notes.set(String::new());
                                        }
                                    });
                                }
                            },
                            {locale.tr("+ Créer la palette", "+ Create palette")}
                        }
                        button {
                            class: "btn",
                            style: "width:100%; margin-top: 8px;",
                            onclick: {
                                let svc = services.clone();
                                let pts = all_paints.clone();
                                move |_| {
                                    let loc = *state.locale.read();
                                    let codes = ["21114", "21236", "21311", "21318", "21418", "21502", "21600", "21705"];
                                    let norma_paints: Vec<_> = codes
                                        .iter()
                                        .filter_map(|c| {
                                            pts.iter()
                                                .find(|p| p.brand.contains("Norma Blue") && p.code == *c)
                                                .cloned()
                                        })
                                        .collect();
                                    let svc = svc.clone();
                                    spawn(async move {
                                        let pal = Palette {
                                            id: None,
                                            name: loc.tr("Schmincke Norma Blue (Coffret 71200097)", "Schmincke Norma Blue (Set 71200097)").to_string(),
                                            purpose: loc.tr("Coffret 8 tubes — Bustes & figurines", "8-tube set — Busts & miniatures").to_string(),
                                            notes: loc.tr(
                                                "Palette de 8 tubes avec mélanges-mères Si (Sienne), BF (Brun foncé) et NC (Noir coloré).",
                                                "8-tube palette with mother mixes Si (Burnt Sienna), BF (Dark Brown), and NC (Chromatic Black)."
                                            ).to_string(),
                                            paints: norma_paints,
                                        };
                                        if svc.palettes.save(&pal).await.is_ok() {
                                            if let Ok(all) = svc.palettes.all().await {
                                                selected_idx.set(all.len().saturating_sub(1));
                                                state.palettes.set(all);
                                            }
                                            state.notify(loc.tr("Palette Schmincke Norma Blue (8 tubes) ajoutée.", "Schmincke Norma Blue (8 tubes) palette added."));
                                        }
                                    });
                                }
                            },
                            {locale.tr("+ Coffret Norma Blue (8 tubes)", "+ Norma Blue Set (8 tubes)")}
                        }
                    }

                    div { class: "card",
                        h3 { class: "card-title", {locale.tr("Palettes enregistrées", "Saved Palettes")}, " ({palettes.len()})" }
                        for (idx, pal) in palettes.iter().enumerate() {
                            div {
                                class: "card-subtle",
                                style: if idx == selected_idx() {
                                    "margin-bottom:8px;border-color:var(--accent);cursor:pointer;"
                                } else {
                                    "margin-bottom:8px;cursor:pointer;"
                                },
                                onclick: move |_| selected_idx.set(idx),
                                div { class: "row-between",
                                    strong { "{pal.name}" }
                                    span { class: "pill", "{pal.paints.len()} tubes" }
                                }
                                if !pal.purpose.is_empty() {
                                    div { class: "muted", style: "font-size:12px;", "{pal.purpose}" }
                                }
                                if !pal.paints.is_empty() {
                                    div { class: "row", style: "margin-top: 6px; gap: 4px;",
                                        for pt in &pal.paints {
                                            PaintSwatch {
                                                masstone_hex: pt.hex_color.clone(),
                                                tint_hex: pt.tint_hex.clone(),
                                                width: 24,
                                                height: 14,
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                div { style: "grid-column: span 2;",
                    if let Some(pal) = active_pal {
                        div { class: "card",
                            div { class: "row-between", style: "margin-bottom:10px;",
                                div {
                                    h2 { class: "card-title", style: "font-size:18px;margin-bottom:2px;", "{pal.name}" }
                                    span { class: "muted", "{pal.purpose}" }
                                }
                                div { class: "row",
                                    span {
                                        class: "pill pill-accent",
                                        {locale.tr("Rythme imposé", "Pacing set by")}, " : {pal.slowest_drying_class().label_i18n(locale)}"
                                    }
                                    button {
                                        class: if *show_matrix.read() { "btn-primary" } else { "btn" },
                                        style: "font-size: 11px; padding: 4px 10px;",
                                        onclick: move |_| {
                                            let cur = *show_matrix.read();
                                            show_matrix.set(!cur);
                                        },
                                        "🎨 "
                                        {locale.tr("Nuancier Matriciel", "Mixing Matrix")}
                                    }
                                    if let Some(id) = pal.id {
                                        button {
                                            class: "btn-danger",
                                            onclick: {
                                                let svc = services.clone();
                                                move |_| {
                                                    let svc = svc.clone();
                                                    spawn(async move {
                                                        let _ = svc.palettes.delete(id).await;
                                                        if let Ok(all) = svc.palettes.all().await {
                                                            selected_idx.set(0);
                                                            state.palettes.set(all);
                                                        }
                                                    });
                                                }
                                            },
                                            {locale.tr("Supprimer", "Delete")}
                                        }
                                    }
                                }
                            }
                            if !pal.notes.is_empty() {
                                p { class: "muted", style: "margin-bottom:14px;white-space:pre-line;", "{pal.notes}" }
                            }

                            div { class: "row", style: "margin-bottom:14px;",
                                select {
                                    style: "flex:1;",
                                    onchange: move |ev| {
                                        if let Ok(i) = ev.value().parse::<usize>() {
                                            paint_to_add_idx.set(i);
                                        }
                                    },
                                    for (i, pt) in all_paints.iter().enumerate() {
                                        option { value: "{i}", "{pt.brand} — {pt.name} ({pt.hex_color})" }
                                    }
                                }
                                button {
                                    class: "btn-primary",
                                    onclick: {
                                        let svc = services.clone();
                                        let pal_clone = pal.clone();
                                        let pts = all_paints.clone();
                                        move |_| {
                                            if let Some(pt) = pts.get(paint_to_add_idx()).cloned() {
                                                let updated = pal_clone.with_paint(pt);
                                                let svc = svc.clone();
                                                spawn(async move {
                                                    if svc.palettes.save(&updated).await.is_ok()
                                                        && let Ok(all) = svc.palettes.all().await {
                                                            state.palettes.set(all);
                                                        }
                                                });
                                            }
                                        }
                                    },
                                    {locale.tr("+ Ajouter ce tube", "+ Add tube")}
                                }
                            }

                            table { class: "data-table",
                                thead {
                                    tr {
                                        th { "Code" }
                                        th { {locale.tr("Masstone / Coupe 1:10", "Masstone / 1:10 Tint")} }
                                        th { {locale.tr("Marque & Nom", "Brand & Name")} }
                                        th { {locale.tr("Comportement à l'établi", "Workbench Behavior")} }
                                        th { "Pigments" }
                                        th { {locale.tr("Séchage", "Drying")} }
                                        th { "" }
                                    }
                                }
                                tbody {
                                    for entry in MixingGuideBuilder::build_palette_entries_i18n(&pal.paints, locale) {
                                        {
                                            let pt = entry.paint.clone();
                                            rsx! {
                                                tr {
                                                    td {
                                                        span { class: "pill pill-accent mono", style: "font-weight: 800;", "{entry.short_code}" }
                                                    }
                                                    td {
                                                        PaintSwatch {
                                                            masstone_hex: pt.hex_color.clone(),
                                                            tint_hex: pt.tint_hex.clone(),
                                                            width: 38,
                                                            height: 24,
                                                        }
                                                    }
                                                    td {
                                                        strong { "{pt.name}" }
                                                        if !pt.code.is_empty() {
                                                            span { class: "muted", " ({pt.code})" }
                                                        }
                                                        div { class: "muted", style: "font-size:11.5px;", "{pt.brand}" }
                                                    }
                                                    td { class: "muted", style: "font-size:12px;", "{entry.behavior}" }
                                                    td { class: "mono", "{pt.pigments.iter().cloned().collect::<Vec<_>>().join(\", \")}" }
                                                    td { "{pt.drying_class.label_i18n(locale)}" }
                                                    td {
                                                        button {
                                                            class: "btn-danger",
                                                            onclick: {
                                                                let svc = services.clone();
                                                                let pal_clone = pal.clone();
                                                                let pt_clone = pt.clone();
                                                                move |_| {
                                                                    let updated = pal_clone.without_paint(&pt_clone);
                                                                    let svc = svc.clone();
                                                                    spawn(async move {
                                                                        if svc.palettes.save(&updated).await.is_ok()
                                                                            && let Ok(all) = svc.palettes.all().await {
                                                                                state.palettes.set(all);
                                                                            }
                                                                    });
                                                                }
                                                            },
                                                            {locale.tr("Retirer", "Remove")}
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            if *show_matrix.read() && !pal.paints.is_empty() {
                                {
                                    let matrix = pal.generate_matrix();
                                    rsx! {
                                        div { class: "card-subtle", style: "margin-top: 16px; overflow-x: auto;",
                                            h4 { style: "margin-bottom: 6px;",
                                                "🎨 "
                                                {locale.tr("Nuancier Matriciel 2 à 2 (Pur / Coupe au blanc)", "2-Tube Mixing Matrix (Masstone / Tint)")}
                                            }
                                            div { class: "muted", style: "font-size: 11.5px; margin-bottom: 12px;",
                                                {locale.tr(
                                                    "Chaque cellule présente le mélange pur 1:1 à gauche et la coupe au blanc de titane à droite.",
                                                    "Each cell displays the 1:1 pure mixture on the left and 1:10 Titanium White tint on the right."
                                                )}
                                            }
                                            table { class: "data-table", style: "font-size: 11px;",
                                                thead {
                                                    tr {
                                                        th { "×" }
                                                        for p in &matrix.paints {
                                                            th { style: "text-align: center; max-width: 90px; overflow: hidden; text-overflow: ellipsis;", "{p.name}" }
                                                        }
                                                    }
                                                }
                                                tbody {
                                                    for (r_idx, row) in matrix.rows.iter().enumerate() {
                                                        tr {
                                                            td {
                                                                div { class: "row", style: "gap: 6px;",
                                                                    ColorSwatch { hex: matrix.paints[r_idx].hex_color.clone(), size: 16 }
                                                                    span { style: "font-weight: 600;", "{matrix.paints[r_idx].name}" }
                                                                }
                                                            }
                                                            for cell in row {
                                                                td { style: "text-align: center; padding: 4px;",
                                                                    div { class: "row", style: "justify-content: center; gap: 3px;",
                                                                        ColorSwatch { hex: cell.pure_mix_hex.clone(), size: 20 }
                                                                        ColorSwatch { hex: cell.tint_mix_hex.clone(), size: 20 }
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

                            // Substitutes if some palette tubes are out of stock
                            {
                                let owned: Vec<_> = all_paints.iter().filter(|p| p.in_stock).cloned().collect();
                                let missing = services.substitute_service.missing_among(&pal.paints, &owned);
                                if !missing.is_empty() {
                                    rsx! {
                                        div { class: "card-subtle", style: "margin-top: 14px; border-color: var(--accent);",
                                            strong {
                                                {locale.tr("Tubes hors stock dans cette palette", "Out-of-stock tubes in this palette")}, " ({missing.len()})"
                                            }
                                            for m in missing {
                                                {
                                                    let absent = m.paint.clone();
                                                    let repl_opt = m.nearest.clone();
                                                    let pal_for_sub = pal.clone();
                                                    let svc = services.clone();
                                                    rsx! {
                                                        div { class: "row-between", style: "margin-top: 8px; padding-top: 6px; border-top: 1px solid var(--border-subtle);",
                                                            div { class: "row",
                                                                ColorSwatch { hex: m.paint.hex_color.clone(), size: 20 }
                                                                strong { "{m.paint.display_name()}" }
                                                                span { class: "muted", "→" }
                                                                if let Some(ref near) = m.nearest {
                                                                    ColorSwatch { hex: near.hex_color.clone(), size: 20 }
                                                                    DeltaEBadge { delta_e: m.delta_e }
                                                                }
                                                                span { class: "muted", style: "font-size: 12px;", "{m.verdict_i18n(locale)}" }
                                                            }
                                                            if let Some(repl) = repl_opt {
                                                                button {
                                                                    class: "btn-primary",
                                                                    style: "padding: 3px 10px; font-size: 11.5px;",
                                                                    onclick: move |_| {
                                                                        let next_pal = pal_for_sub.without_paint(&absent).with_paint(repl.clone());
                                                                        let svc = svc.clone();
                                                                        spawn(async move {
                                                                            let loc = *state.locale.read();
                                                                            if svc.palettes.save(&next_pal).await.is_ok() {
                                                                                if let Ok(all) = svc.palettes.all().await {
                                                                                    state.palettes.set(all);
                                                                                }
                                                                                state.notify(loc.tr("Tube remplacé dans la palette.", "Tube replaced in palette."));
                                                                            }
                                                                        });
                                                                    },
                                                                    {locale.tr("Remplacer dans la palette", "Replace in palette")}
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    rsx! {}
                                }
                            }
                        }

                        // 2-column grid: CIELAB Gamut Map + Intermediate Mother Mixes (Si, BF, NC)
                        if !pal.paints.is_empty() {
                            {
                                let dummy_plan = palettier_domain::plan::PaintingPlan {
                                    subject: pal.name.clone(),
                                    palette_name: pal.name.clone(),
                                    approach: String::new(),
                                    system: pal.system(),
                                    zones: Vec::new(),
                                };
                                let guide = services.guide_builder.from_plan_i18n(&dummy_plan, &pal.paints, locale);
                                rsx! {
                                    div { class: "grid-2",
                                        div { class: "card",
                                            h3 { class: "card-title", {locale.tr("Couverture Gamut CIELAB (a* / b*)", "CIELAB Gamut Coverage (a* / b*)")} }
                                            p { class: "muted", style: "font-size: 11.5px; margin-bottom: 10px;",
                                                if locale.is_fr() {
                                                    "Polygone chromatique couvert par les {pal.paints.len()} tubes de la palette et leurs coupes au blanc."
                                                } else {
                                                    "Chromatic polygon spanned by the {pal.paints.len()} tubes in the palette and their white tints."
                                                }
                                            }
                                            GamutMapSvg {
                                                paints: pal.paints.clone(),
                                                target_hex: None,
                                                achieved_hex: None,
                                                size: 250,
                                            }
                                        }
                                        div { class: "card",
                                            h3 { class: "card-title", {locale.tr("Mélanges intermédiaires de début de séance", "Session-Start Intermediate Mother Mixes")} }
                                            p { class: "muted", style: "font-size: 11.5px; margin-bottom: 10px;",
                                                {locale.tr("Mélanges-mères (Sienne, Brun foncé, Noir coloré) calculés à partir de cette palette pour harmoniser les ombres.", "Mother mixes (Burnt Sienna, Dark Brown, Chromatic Black) computed from this palette to harmonize shadows.")}
                                            }
                                            for mm in &guide.mother_mixes {
                                                div { class: "card-subtle", style: "margin-bottom: 8px;",
                                                    div { class: "row", style: "margin-bottom: 4px;",
                                                        ColorSwatch { hex: mm.hex_color.clone(), size: 24 }
                                                        span { class: "pill pill-accent mono", style: "font-weight: 800;", "{mm.code}" }
                                                        strong { "{mm.name}" }
                                                    }
                                                    div { class: "muted", style: "font-size: 12px;", "{mm.recipe_text}" }
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
