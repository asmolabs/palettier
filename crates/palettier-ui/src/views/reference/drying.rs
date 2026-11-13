//! The drying calculator: one coat, the workshop of the day, the milestones.

use super::drying_cards::{DryingMilestones, MediumFormulator, MediumShares, StudioClimate};
use crate::actions::drying::{CoatSettings, estimate_coat, formulate_medium};
use crate::state::{AppServices, AppState};
use crate::theme::SectionHeader;
use dioxus::prelude::*;
use palettier_domain::paint::{DryingClass, LayerThickness, Medium, PaintSystem};

#[component]
pub fn DryingView() -> Element {
    let services = use_context::<AppServices>();
    let state = use_context::<AppState>();

    let locale = *state.locale.read();

    let drying_class = use_signal(|| DryingClass::Medium);
    let medium = use_signal(|| Medium::None);
    let medium_ratio_pct = use_signal(|| 0.0_f64);
    let thickness = use_signal(|| LayerThickness::Normal);

    let shares = MediumShares {
        oil: use_signal(|| 33u32),
        solvent: use_signal(|| 33u32),
        resin: use_signal(|| 34u32),
        driers: use_signal(|| 0u32),
    };

    let formulated_medium = formulate_medium(
        *shares.oil.read(),
        *shares.solvent.read(),
        *shares.resin.read(),
        *shares.driers.read(),
    );

    let ws = *state.workshop.read();
    let system = *state.paint_system.read();
    let is_oil = system == PaintSystem::Oil;
    let drying = estimate_coat(
        &services,
        CoatSettings {
            system,
            drying_class: *drying_class.read(),
            medium: *medium.read(),
            medium_ratio_pct: *medium_ratio_pct.read(),
            thickness: *thickness.read(),
            workshop: ws,
        },
    );

    rsx! {
        div {
            SectionHeader {
                title: if is_oil {
                    locale.tr("Calculateur de séchage à l'huile", "Oil Drying Calculator").to_string()
                } else {
                    locale.tr("Calculateur de séchage acrylique", "Acrylic Drying Calculator").to_string()
                },
                subtitle: if is_oil {
                    locale.tr(
                        "Estimez le temps ouvert, le sec au toucher, le recouvrable et le séchage à cœur selon vos pigments, votre médium et le climat de l'atelier.",
                        "Estimate open time, touch dry, recoat window, and full cure based on your pigments, medium, and studio climate."
                    ).to_string()
                } else {
                    locale.tr(
                        "L'acrylique sèche par évaporation : l'épaisseur, l'humidité et l'aération comptent, le pigment presque pas. Les temps se lisent en minutes.",
                        "Acrylic dries by evaporation: thickness, humidity and air flow matter, the pigment barely does. Times read in minutes."
                    ).to_string()
                },
                actions: None,
            }

            div { class: "grid-2",
                div { class: "card",
                    CoatParameters {
                        system,
                        drying_class,
                        medium,
                        current_medium: drying.medium,
                        medium_ratio_pct,
                        fatness: drying.fatness,
                        thickness,
                    }

                    StudioClimate {}

                    // Custom Medium Formulation Card
                    if is_oil {
                        MediumFormulator {
                            shares,
                            fat_index: formulated_medium.fat_index(),
                            drying_multiplier: formulated_medium.drying_multiplier(),
                        }
                    }
                }

                DryingMilestones { estimate: drying.estimate, current_medium: drying.medium }
            }
        }
    }
}

/// The pigment drying class (oil only), the medium, its share and the coat thickness.
#[component]
fn CoatParameters(
    system: PaintSystem,
    drying_class: Signal<DryingClass>,
    medium: Signal<Medium>,
    current_medium: Medium,
    medium_ratio_pct: Signal<f64>,
    fatness: f64,
    thickness: Signal<LayerThickness>,
) -> Element {
    let state = use_context::<AppState>();
    let locale = *state.locale.read();
    let is_oil = system == PaintSystem::Oil;

    rsx! {
        h3 { class: "card-title", {locale.tr("Paramètres de la couche", "Coat Parameters")} }

        if is_oil {
        div { style: "margin-bottom: 12px;",
            label { class: "field-label", {locale.tr("Classe siccative du pigment", "Pigment Drying Class")} }
            select {
                style: "width: 100%;",
                onchange: move |e| {
                    let v = match e.value().as_str() {
                        "Fast" => DryingClass::Fast,
                        "Slow" => DryingClass::Slow,
                        "VerySlow" => DryingClass::VerySlow,
                        _ => DryingClass::Medium,
                    };
                    drying_class.set(v);
                },
                for dc in DryingClass::ALL {
                    option {
                        value: "{dc:?}",
                        selected: *drying_class.read() == dc,
                        "{dc.label(locale)} — {dc.typical_pigments(locale)}"
                    }
                }
            }
        }
        }

        div { style: "margin-bottom: 12px;",
            label { class: "field-label", {locale.tr("Médium ou diluant ajouté", "Added Medium or Thinner")} }
            select {
                style: "width: 100%;",
                onchange: move |e| {
                    let m = Medium::ALL
                        .into_iter()
                        .find(|m| format!("{m:?}") == e.value())
                        .unwrap_or(Medium::None);
                    medium.set(m);
                },
                for m in Medium::for_system(system) {
                    option {
                        value: "{m:?}",
                        selected: current_medium == m,
                        "{m.label(locale)}"
                    }
                }
            }
        }

        div { style: "margin-bottom: 12px;",
            label {
                class: "field-label",
                if locale.is_fr() {
                    "Proportion de médium : {medium_ratio_pct.read():.0} % (indice gras : {fatness:.2})"
                } else {
                    "Medium ratio: {medium_ratio_pct.read():.0}% (fatness index: {fatness:.2})"
                }
            }
            input {
                r#type: "range",
                style: "width: 100%;",
                min: "0",
                max: "95",
                step: "5",
                value: "{medium_ratio_pct}",
                oninput: move |e| {
                    if let Ok(v) = e.value().parse::<f64>() {
                        medium_ratio_pct.set(v);
                    }
                },
            }
        }

        div { style: "margin-bottom: 16px;",
            label { class: "field-label", {locale.tr("Épaisseur de la touche", "Coat Thickness")} }
            div { class: "row",
                for th in LayerThickness::ALL {
                    button {
                        class: if *thickness.read() == th { "btn-primary" } else { "btn" },
                        onclick: move |_| thickness.set(th),
                        "{th.label(locale)}"
                    }
                }
            }
        }
    }
}
