//! A zone of the open piece: its value ladder and the table of its coats, editable inline.

use super::projects::ProjectUi;
use crate::actions::projects;
use crate::state::{AppServices, AppState, Section};
use crate::theme::{ColorSwatch, DeltaEBadge, HexColorInput, ValueLadderStrip};
use dioxus::prelude::*;
use palettier_domain::paint::{PaintSystem, Technique};
use palettier_domain::project::{LayerKind, Project};

#[component]
pub fn ZoneCard(proj: Project, z_idx: usize) -> Element {
    let state = use_context::<AppState>();
    let services = use_context::<AppServices>();
    let mut ui = use_context::<ProjectUi>();
    let locale = *state.locale.read();
    let Some(zone) = proj.zones.get(z_idx).cloned() else {
        return rsx! {};
    };
    let ladder_steps: Vec<(String, String)> = zone
        .layers
        .iter()
        .filter(|l| l.kind == LayerKind::Ladder)
        .map(|l| (l.role.clone(), l.target_hex.clone()))
        .collect();

    let add_coat = {
        let proj = proj.clone();
        move |_| {
            let svc = services.clone();
            let proj = proj.clone();
            spawn(async move {
                if let Ok(change) = projects::add_accent_layer(&svc, &proj, z_idx).await {
                    ui.show(state, change);
                }
            });
        }
    };

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
                        onclick: add_coat,
                        {locale.tr("+ Couche", "+ Coat")}
                    }
                }
            }

            ValueLadderStrip { steps: ladder_steps }

            LayerTable { proj: proj.clone(), z_idx }
        }
    }
}

#[component]
fn LayerTable(proj: Project, z_idx: usize) -> Element {
    let state = use_context::<AppState>();
    let locale = *state.locale.read();
    let count = proj.zones.get(z_idx).map_or(0, |z| z.layers.len());

    rsx! {
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
                for l_idx in 0..count {
                    LayerRow { proj: proj.clone(), z_idx, l_idx }
                }
            }
        }
    }
}

#[component]
fn LayerRow(proj: Project, z_idx: usize, l_idx: usize) -> Element {
    let mut state = use_context::<AppState>();
    let services = use_context::<AppServices>();
    let mut ui = use_context::<ProjectUi>();
    let locale = *state.locale.read();
    let Some(layer) = proj.zones.get(z_idx).and_then(|z| z.layers.get(l_idx)).cloned() else {
        return rsx! {};
    };
    let is_editing = *ui.editing_layer.read() == Some((z_idx, l_idx));
    let computed = services.planner.recompute(&layer, proj.effective_paints());

    let save_edit = {
        let svc = services.clone();
        let proj = proj.clone();
        move |_| {
            let edit = ui.layer_edit();
            ui.editing_layer.set(None);
            let svc = svc.clone();
            let proj = proj.clone();
            spawn(async move {
                if let Ok(change) = projects::edit_layer(&svc, &proj, z_idx, l_idx, edit).await {
                    ui.show(state, change);
                }
            });
        }
    };

    let toggle_applied = {
        let proj = proj.clone();
        move |_| {
            let w = *state.workshop.read();
            let svc = services.clone();
            let proj = proj.clone();
            spawn(async move {
                if let Ok(change) = projects::mark_layer_applied(&svc, &proj, z_idx, l_idx, w).await {
                    ui.show(state, change);
                }
            });
        }
    };

    let start_edit = {
        let layer = layer.clone();
        move |_| {
            ui.edit_role.set(layer.role.clone());
            ui.edit_hex.set(layer.target_hex.clone());
            ui.edit_tech.set(layer.technique);
            ui.edit_note.set(layer.note.clone());
            ui.editing_layer.set(Some((z_idx, l_idx)));
        }
    };

    let mix = {
        let hex = layer.target_hex.clone();
        move |_| {
            state.mixer_target_hex.set(hex.clone());
            state.section.set(Section::Mixer);
        }
    };

    rsx! {
        tr {
            td {
                if is_editing {
                    input {
                        r#type: "text",
                        style: "width: 110px;",
                        value: "{ui.edit_role}",
                        oninput: move |e| ui.edit_role.set(e.value())
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
                        value: (ui.edit_hex)(),
                        onchange: move |v: String| ui.edit_hex.set(v),
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
                    LayerEditor { system: proj.system }
                } else {
                    if let Some(ref rec) = computed.recipe {
                        div { class: "row",
                            ColorSwatch { hex: computed.achieved.to_hex(), size: 18 }
                            strong { style: "font-size: 12.5px;", "{rec.describe_i18n(locale)}" }
                            DeltaEBadge { delta_e: computed.delta_e }
                        }
                    }
                    div { class: "muted", style: "font-size:11.5px; margin-top: 2px;",
                        "{layer.technique.label_i18n(locale)}"
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
                            onclick: save_edit,
                            {locale.tr("Valider", "Save")}
                        }
                        button {
                            class: "btn",
                            onclick: move |_| ui.editing_layer.set(None),
                            {locale.tr("Annuler", "Cancel")}
                        }
                    } else {
                        button {
                            class: "btn",
                            onclick: toggle_applied,
                            if layer.is_applied() {
                                {locale.tr("Annuler pose", "Unmark coat")}
                            } else {
                                {locale.tr("Marquer posée", "Mark applied")}
                            }
                        }
                        button {
                            class: "btn",
                            onclick: start_edit,
                            {locale.tr("Éditer", "Edit")}
                        }
                        button {
                            class: "btn",
                            onclick: mix,
                            {locale.tr("Mélanger", "Mix")}
                        }
                    }
                }
            }
        }
    }
}

/// Technique and note of the coat being edited.
#[component]
fn LayerEditor(system: PaintSystem) -> Element {
    let state = use_context::<AppState>();
    let mut ui = use_context::<ProjectUi>();
    let locale = *state.locale.read();
    let edit_tech = (ui.edit_tech)();
    let oil_on_top: Vec<Technique> = Technique::ALL
        .into_iter()
        .filter(|t| t.native_system() == Some(PaintSystem::Oil))
        .collect();

    rsx! {
        div { class: "row",
            select {
                onchange: move |e| {
                    if let Some(t) = Technique::parse(&e.value()) {
                        ui.edit_tech.set(t);
                    }
                },
                // The piece's own techniques first; on an acrylic piece the oil
                // techniques stay available (oil over dry acrylic is sound),
                // never the reverse.
                for t in Technique::for_system(system) {
                    option { value: "{t.as_db_str()}", selected: edit_tech == t, "{t.label_i18n(locale)}" }
                }
                if system == PaintSystem::Acrylic {
                    optgroup { label: "{locale.tr(\"À l'huile, par-dessus\", \"In oil, on top\")}",
                        for t in oil_on_top {
                            option { value: "{t.as_db_str()}", selected: edit_tech == t, "{t.label_i18n(locale)}" }
                        }
                    }
                }
            }
            input {
                r#type: "text",
                placeholder: "Note",
                value: "{ui.edit_note}",
                oninput: move |e| ui.edit_note.set(e.value())
            }
        }
    }
}
