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
            div { class: "row-between u-mb-12",
                div { class: "row",
                    PaintSwatch {
                        masstone_hex: paint.hex_color.clone(),
                        tint_hex: paint.tint_hex.clone(),
                        width: 56,
                        height: 40,
                    }
                    div {
                        h3 { class: "card-title u-mb-2", "{paint.name}" }
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

            div { class: "row u-mb-14 u-fwrap-wrap u-gap-6",
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
                div { class: "card-subtle u-mb-12 u-bc-pill-warn-border u-p-8-12",
                    div { class: "u-fw-600 u-fs-12 u-c-pill-warn-text",
                        "⚠️ "
                        {locale.tr("Précautions atelier : ", "Studio precautions: ")}
                        "{safety.hazards.join(\", \")}"
                    }
                    div { class: "muted u-fs-11p5 u-mt-3",
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

            h4 { class: "u-fs-13 u-mb-8", {locale.tr("Équivalents et substituts proches", "Closest Equivalents & Substitutes")} }
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
        div { class: "card-subtle u-mb-14",
            h4 { class: "u-fs-13 u-mb-8", {locale.tr("Calibration optique du tube", "Optical Tube Calibration")} }
            p { class: "muted u-fs-12 u-mb-10",
                {locale.tr("Relevez la sortie de tube (masstone) et la coupe 1:10 au blanc de titane pour affiner la courbe de Kubelka-Munk.", "Record the pure masstone and 1:10 Titanium White tint to refine the Kubelka-Munk scattering curve.")}
            }
            div { class: "grid-2 u-mb-10",
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
            class: "u-mt-6 u-fs-11 u-p-3-8",
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
        div { class: "row-between u-p-6-0 u-bb-1-solid-border-subtle",
            div { class: "row",
                PaintSwatch {
                    masstone_hex: sub.hex_color.clone(),
                    tint_hex: sub.tint_hex.clone(),
                    width: 28,
                    height: 18,
                }
                div {
                    div { class: "u-fs-12p5 u-fw-600", "{sub.name}" }
                    div { class: "muted u-fs-11",
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
