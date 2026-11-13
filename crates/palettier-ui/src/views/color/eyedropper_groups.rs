//! The eyedropper's right column: the picked colour, the sampled groups that become a
//! measured project, and the zones pinned for the AI assistant.

use crate::actions::eyedropper as eye;
use crate::state::{AppServices, AppState, Section};
use crate::theme::{ColorSwatch, HexColorInput, ValueLadderStrip};
use dioxus::prelude::*;
use palettier_ai::SampledZone;
use palettier_domain::color::Rgb;
use palettier_domain::i18n::Locale;
use palettier_domain::plan::ColourGroup;

/// The colour last picked, editable, and sent to the mixer.
#[component]
pub fn ActiveColorCard(picked_hex: Signal<String>) -> Element {
    let mut state = use_context::<AppState>();
    let locale = *state.locale.read();

    rsx! {
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
                        div { class: "muted u-fs-11p5 u-mt-4",
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
    }
}

/// Groups of samples, one per part of the subject, and the measured project made of them.
#[component]
pub fn SampledGroupsCard(
    picked_hex: Signal<String>,
    groups: Signal<Vec<ColourGroup>>,
    selected_group_idx: Signal<usize>,
    new_group_name: Signal<String>,
    new_group_mat: Signal<String>,
    group_project_name: Signal<String>,
    group_palette_idx: Signal<usize>,
) -> Element {
    let mut state = use_context::<AppState>();
    let services = use_context::<AppServices>();
    let locale = *state.locale.read();
    let palettes = state.palettes.read().clone();

    let create_project = {
        let pals = palettes.clone();
        move |_: Event<MouseData>| {
            let loc = *state.locale.read();
            let chosen_pal = pals.get(group_palette_idx()).cloned();
            let grp_list = groups.read().clone();
            let name = group_project_name();
            let taken: Vec<String> = state.projects.read().iter().map(|p| p.name.clone()).collect();
            let svc = services.clone();
            spawn(async move {
                let created =
                    eye::create_project_from_groups(&svc, &name, chosen_pal.as_ref(), &grp_list, &taken, loc).await;
                match created {
                    Ok(Some(saved)) => {
                        state.reload_all(&svc).await;
                        let msg = match loc {
                            Locale::Fr => format!("Projet « {} » créé à partir de vos relevés.", saved.name),
                            Locale::En => format!("Project '{}' created from your sampled groups.", saved.name),
                        };
                        state.notify(msg);
                        state.section.set(Section::Projects);
                    }
                    Ok(None) => {}
                    Err(e) => state.notify(format!("{e}")),
                }
            });
        }
    };

    rsx! {
        div { class: "card",
            h3 { class: "card-title", {locale.tr("Groupes de relevés → Créer un projet mesuré", "Sampled Groups → Build Measured Project")} }
            p { class: "muted u-fs-12 u-mb-10",
                {locale.tr("Relevez plusieurs points par partie du sujet (ombre profonde, base, lumière) : Palettier les trie par luminance et calcule les recettes de chaque marche.", "Sample multiple points per zone (deep shadow, base, highlight): Palettier sorts them by luminance and computes recipes for each step.")}
            }

            div { class: "row u-mb-10",
                input {
                    r#type: "text",
                    placeholder: locale.tr("Nouvelle zone (ex. Cape)", "New zone (e.g. Cloak)"),
                    value: "{new_group_name}",
                    oninput: move |e| new_group_name.set(e.value())
                }
                input {
                    r#type: "text",
                    class: "u-w-120",
                    placeholder: locale.tr("Matière (ex. Drap)", "Material (e.g. Cloth)"),
                    value: "{new_group_mat}",
                    oninput: move |e| new_group_mat.set(e.value())
                }
                button {
                    class: "btn",
                    onclick: move |_| {
                        let Some(list) = eye::add_group(&groups.read(), &new_group_name(), &new_group_mat()) else {
                            return;
                        };
                        selected_group_idx.set(list.len() - 1);
                        groups.set(list);
                        new_group_name.set(String::new());
                        new_group_mat.set(String::new());
                    },
                    {locale.tr("+ Groupe", "+ Group")}
                }
            }

            // Group list
            for (g_idx, grp) in groups.read().clone().into_iter().enumerate() {
                SampledGroupRow { g_idx, grp, picked_hex, groups, selected_group_idx }
            }

            div { class: "row u-mt-12",
                input {
                    r#type: "text",
                    class: "u-flex-1",
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
                    onclick: create_project,
                    {locale.tr("Créer le projet →", "Create project →")}
                }
            }
        }
    }
}

/// One group: its samples as a value ladder, a button to add the picked colour.
#[component]
fn SampledGroupRow(
    g_idx: usize,
    grp: ColourGroup,
    picked_hex: Signal<String>,
    groups: Signal<Vec<ColourGroup>>,
    selected_group_idx: Signal<usize>,
) -> Element {
    let is_sel = *selected_group_idx.read() == g_idx;
    let ladder_steps = eye::group_ladder(&grp);

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
                    span { class: "pill u-ml-8", "{grp.samples.len()} pt(s)" }
                }
                div { class: "row",
                    button {
                        class: "btn-primary",
                        onclick: move |e: Event<MouseData>| {
                            e.stop_propagation();
                            let next = eye::add_sample(&groups.read(), g_idx, &picked_hex());
                            if let Some(list) = next {
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

/// Pins the picked colour as a zone for the AI assistant.
#[component]
pub fn PinZoneCard(picked_hex: Signal<String>, zone_name: Signal<String>, zone_material: Signal<String>) -> Element {
    let mut state = use_context::<AppState>();
    let locale = *state.locale.read();
    let sampled = state.sampled_zones.read().clone();

    rsx! {
        div { class: "card",
            h3 { class: "card-title", {locale.tr("Épingler une teinte de base pour l'Assistant IA", "Pin a Base Color for the AI Assistant")} }
            div { class: "row u-mb-10",
                input {
                    r#type: "text",
                    placeholder: locale.tr("Zone (ex. Tunique)", "Zone (e.g. Tunic)"),
                    value: "{zone_name}",
                    oninput: move |ev| zone_name.set(ev.value())
                }
                input {
                    r#type: "text",
                    class: "u-w-120",
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
