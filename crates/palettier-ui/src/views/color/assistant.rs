//! The painting plan assistant: the request card and the plan it returns.

use crate::actions::assistant::{self as plan_actions, PlanRequest};
use crate::actions::eyedropper::scaled_data_uri;
use crate::actions::mixer;
use crate::state::{AppServices, AppState, Section};
use crate::theme::{ColorSwatch, DeltaEBadge, MixingGuideCard, SectionHeader, ValueLadderStrip};
use dioxus::prelude::*;
use palettier_domain::i18n::Locale;
use palettier_domain::paint::Paint;
use palettier_domain::palette::Palette;
use palettier_domain::plan::{PaintingPlan, PlanZone};

#[component]
pub fn AssistantView() -> Element {
    let mut state = use_context::<AppState>();

    let locale = *state.locale.read();
    let system = *state.paint_system.read();
    let palettes = mixer::palettes_of_system(&state.palettes.read(), system);
    // Only the working system: an acrylic plan is mixed from acrylics, an oil plan from oils.
    let all_paints = mixer::paints_of_system(&state.paints.read(), system);
    let ai = state.ai_settings.read().clone();

    let description = use_signal(|| {
        "Buste de grognard de la Garde Impériale au 1/10 : visage buriné, revers de manteau bleu impérial et épaulettes rouges.".to_string()
    });
    let palette_idx = use_signal(|| 0usize);
    let image_bytes = use_signal(|| None::<Vec<u8>>);
    let image_label = use_signal(String::new);
    let busy = use_signal(|| false);
    let generated_plan = use_signal(|| None::<PaintingPlan>);
    let error_msg = use_signal(|| None::<String>);
    let show_guide_mode = use_signal(|| true);

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

            AssistantRequestCard {
                palettes: palettes.clone(),
                all_paints: all_paints.clone(),
                description,
                palette_idx,
                image_bytes,
                image_label,
                busy,
                generated_plan,
                error_msg,
            }

            if let Some(plan) = generated_plan() {
                AssistantPlanCard { plan, palettes, all_paints, palette_idx, image_bytes, show_guide_mode }
            }
        }
    }
}

/// What the assistant is asked: the subject, the palette, a photo, the pinned zones.
#[component]
fn AssistantRequestCard(
    palettes: Vec<Palette>,
    all_paints: Vec<Paint>,
    description: Signal<String>,
    palette_idx: Signal<usize>,
    image_bytes: Signal<Option<Vec<u8>>>,
    image_label: Signal<String>,
    busy: Signal<bool>,
    generated_plan: Signal<Option<PaintingPlan>>,
    error_msg: Signal<Option<String>>,
) -> Element {
    let state = use_context::<AppState>();
    let services = use_context::<AppServices>();
    let locale = *state.locale.read();
    let sampled = state.sampled_zones.read().clone();

    let attach_photo = move |_: Event<MouseData>| {
        spawn(async move {
            if let Some(file) = rfd::AsyncFileDialog::new()
                .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
                .pick_file()
                .await
                && let Ok(bytes) = std::fs::read(file.path())
            {
                image_label.set(file.file_name());
                image_bytes.set(Some(bytes));
            }
        });
    };

    rsx! {
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
                            onclick: attach_photo,
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
                            let (palette_name, paints) = plan_actions::plan_paints(&pals, palette_idx(), &pts, loc);
                            let request = PlanRequest {
                                image: image_bytes(),
                                description: description(),
                                palette_name,
                                paints,
                                sampled: smp.clone(),
                            };
                            let svc = svc.clone();
                            spawn(async move {
                                match plan_actions::generate_plan(&svc, request, loc).await {
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
    }
}

/// The plan returned: its mixing guide sheet or its technical view, and saving it.
#[component]
fn AssistantPlanCard(
    plan: PaintingPlan,
    palettes: Vec<Palette>,
    all_paints: Vec<Paint>,
    palette_idx: Signal<usize>,
    image_bytes: Signal<Option<Vec<u8>>>,
    show_guide_mode: Signal<bool>,
) -> Element {
    let mut state = use_context::<AppState>();
    let services = use_context::<AppServices>();
    let locale = *state.locale.read();

    let active_paints = palettes
        .get(palette_idx())
        .map(|p| p.paints.clone())
        .unwrap_or_else(|| plan_actions::stock_or_all(&all_paints));
    let guide = services.guide_builder.from_plan_i18n(&plan, &active_paints, locale);
    let photo_uri = image_bytes().map(|bytes| scaled_data_uri(&services, bytes, 600));

    let save_as_project = {
        let svc = services.clone();
        let pals = palettes.clone();
        let pts = all_paints.clone();
        let plan = plan.clone();
        move |_: Event<MouseData>| {
            let loc = *state.locale.read();
            let chosen_pal = pals.get(palette_idx()).cloned();
            let taken: Vec<String> = state.projects.read().iter().map(|p| p.name.clone()).collect();
            let attached_img = image_bytes();
            let svc = svc.clone();
            let plan = plan.clone();
            let fallback = pts.clone();
            spawn(async move {
                let created = plan_actions::create_project_from_plan(
                    &svc,
                    &plan,
                    chosen_pal.as_ref(),
                    fallback,
                    &taken,
                    attached_img,
                    loc,
                )
                .await;
                if let Ok((saved, all)) = created {
                    if let Some(all) = all {
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
    };

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
                        onclick: save_as_project,
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
                for z in plan.zones.iter().cloned() {
                    PlanZoneTable { zone: z }
                }
            }
        }
    }
}

/// One zone of the plan: its value ladder and, step by step, the recipe and its gap.
#[component]
fn PlanZoneTable(zone: PlanZone) -> Element {
    let state = use_context::<AppState>();
    let locale = *state.locale.read();
    let z = &zone;
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
                                div { "{l.technique.label(locale)}" }
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
