//! The sheet of the selected tube: pigments, safety, calibration and substitutes.

use crate::actions::catalog::{calibrate_tube, delete_tube, pigment_codes};
use crate::state::{AppServices, AppState};
use crate::theme::{DeltaEBadge, HexColorInput, PaintSwatch};
use dioxus::prelude::*;
use palettier_domain::paint::Paint;

#[component]
pub(super) fn PaintSheet(
    paint: Paint,
    substitutes: Vec<(Paint, f64)>,
    pipette_hex: Option<String>,
    selected_paint_id: Signal<Option<i64>>,
    calibrate_masstone_hex: Signal<String>,
    calibrate_tint_hex: Signal<String>,
) -> Element {
    let services = use_context::<AppServices>();
    let mut state = use_context::<AppState>();
    let locale = *state.locale.read();

    let pig_str = pigment_codes(&paint);
    let safety = paint.safety_info();
    let delete = move |pid: i64| {
        let services = services.clone();
        spawn(async move {
            let loc = *state.locale.read();
            if delete_tube(&services, pid).await.is_ok() {
                selected_paint_id.set(None);
                state.reload_all(&services).await;
                state.notify(loc.tr("Tube personnalisé supprimé.", "Custom tube deleted."));
            }
        });
    };

    rsx! {
        div {
            div { class: "row-between", style: "margin-bottom: 12px;",
                div { class: "row",
                    PaintSwatch {
                        masstone_hex: paint.hex_color.clone(),
                        tint_hex: paint.tint_hex.clone(),
                        width: 56,
                        height: 40,
                    }
                    div {
                        h3 { class: "card-title", style: "margin-bottom: 2px;", "{paint.name}" }
                        div { class: "muted", "{paint.brand} · #{paint.code} · {paint.hex_color}" }
                    }
                }
                if paint.user_added {
                    if let Some(pid) = paint.id {
                        button {
                            class: "btn-danger",
                            onclick: move |_| delete(pid),
                            {locale.tr("Supprimer", "Delete")}
                        }
                    }
                }
            }

            div { class: "row", style: "margin-bottom: 14px; flex-wrap: wrap; gap: 6px;",
                span { class: "pill pill-accent", "Pigments: {pig_str}" }
                span { class: "pill", {locale.tr("Opacité : ", "Opacity: ")}, "{paint.opacity.label(locale)}" }
                span { class: "pill", {locale.tr("Séchage : ", "Drying: ")}, "{paint.drying_class.label(locale)}" }
                span { class: "pill", "{paint.lightfastness().label(locale)}" }
                span {
                    class: "pill {safety.level.css_class()}",
                    title: "{safety.advice(locale)}",
                    "🛡️ {safety.level.badge_label(locale)}"
                }
            }
            if !safety.hazards.is_empty() {
                div { class: "card-subtle", style: "margin-bottom: 12px; border-color: var(--pill-warn-border); padding: 8px 12px;",
                    div { style: "font-weight: 600; font-size: 12px; color: var(--pill-warn-text);",
                        "⚠️ "
                        {locale.tr("Précautions atelier : ", "Studio precautions: ")}
                        "{safety.hazards.join(\", \")}"
                    }
                    div { class: "muted", style: "font-size: 11.5px; margin-top: 3px;",
                        "{safety.advice(locale)}"
                    }
                }
            }

            CalibrationCard {
                paint_id: paint.id,
                pipette_hex,
                calibrate_masstone_hex,
                calibrate_tint_hex,
            }

            h4 { style: "font-size: 13px; margin-bottom: 8px;", {locale.tr("Équivalents et substituts proches", "Closest Equivalents & Substitutes")} }
            div {
                for (sub, de) in substitutes {
                    SubstituteRow { sub, de }
                }
            }
        }
    }
}

/// Records the measured masstone and white tint of the tube.
#[component]
fn CalibrationCard(
    paint_id: Option<i64>,
    pipette_hex: Option<String>,
    calibrate_masstone_hex: Signal<String>,
    calibrate_tint_hex: Signal<String>,
) -> Element {
    let services = use_context::<AppServices>();
    let mut state = use_context::<AppState>();
    let locale = *state.locale.read();

    let save = move |pid: i64| {
        let m_hex = calibrate_masstone_hex.read().clone();
        let t_hex = calibrate_tint_hex.read().clone();
        let services = services.clone();
        spawn(async move {
            let loc = *state.locale.read();
            let _ = calibrate_tube(&services, pid, &m_hex, &t_hex).await;
            state.reload_all(&services).await;
            state.notify(loc.tr("Calibration enregistrée pour ce tube.", "Calibration saved for this tube."));
        });
    };

    rsx! {
        div { class: "card-subtle", style: "margin-bottom: 14px;",
            h4 { style: "font-size: 13px; margin-bottom: 8px;", {locale.tr("Calibration optique du tube", "Optical Tube Calibration")} }
            p { class: "muted", style: "font-size: 12px; margin-bottom: 10px;",
                {locale.tr("Relevez la sortie de tube (masstone) et la coupe 1:10 au blanc de titane pour affiner la courbe de Kubelka-Munk.", "Record the pure masstone and 1:10 Titanium White tint to refine the Kubelka-Munk scattering curve.")}
            }
            div { class: "grid-2", style: "margin-bottom: 10px;",
                div {
                    label { class: "field-label", {locale.tr("Sortie de tube (Masstone)", "Pure Tube (Masstone)")} }
                    HexColorInput {
                        value: calibrate_masstone_hex(),
                        onchange: move |v: String| calibrate_masstone_hex.set(v),
                    }
                    if let Some(ref p_hex) = pipette_hex {
                        PipetteButton { hex: p_hex.clone(), target: calibrate_masstone_hex }
                    }
                }
                div {
                    label { class: "field-label", {locale.tr("Coupe 1:10 au blanc", "1:10 White Tint")} }
                    HexColorInput {
                        value: calibrate_tint_hex(),
                        onchange: move |v: String| calibrate_tint_hex.set(v),
                    }
                    if let Some(ref p_hex) = pipette_hex {
                        PipetteButton { hex: p_hex.clone(), target: calibrate_tint_hex }
                    }
                }
            }
            if let Some(pid) = paint_id {
                div { class: "row",
                    button {
                        class: "btn-primary",
                        onclick: move |_| save(pid),
                        {locale.tr("Enregistrer la calibration", "Save calibration")}
                    }
                }
            }
        }
    }
}

/// Copies the colour last picked with the eyedropper into a calibration field.
#[component]
fn PipetteButton(hex: String, target: Signal<String>) -> Element {
    let state = use_context::<AppState>();
    let locale = *state.locale.read();
    let picked = hex.clone();

    rsx! {
        button {
            class: "btn",
            style: "margin-top: 6px; font-size: 11px; padding: 3px 8px;",
            onclick: move |_| target.set(picked.clone()),
            "← ", {locale.tr("Pipette", "Eyedropper")}, " ({hex})"
        }
    }
}

#[component]
fn SubstituteRow(sub: Paint, de: f64) -> Element {
    let state = use_context::<AppState>();
    let locale = *state.locale.read();
    let sub_pigs = pigment_codes(&sub);

    rsx! {
        div { class: "row-between", style: "padding: 6px 0; border-bottom: 1px solid var(--border-subtle);",
            div { class: "row",
                PaintSwatch {
                    masstone_hex: sub.hex_color.clone(),
                    tint_hex: sub.tint_hex.clone(),
                    width: 28,
                    height: 18,
                }
                div {
                    div { style: "font-size: 12.5px; font-weight: 600;", "{sub.name}" }
                    div { class: "muted", style: "font-size: 11px;",
                        "{sub.brand} · {sub_pigs}"
                        if sub.in_stock {
                            span { " · (", {locale.tr("En stock", "In stock")}, ")" }
                        }
                    }
                }
            }
            DeltaEBadge { delta_e: de }
        }
    }
}
