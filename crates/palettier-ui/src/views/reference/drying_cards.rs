//! The drying calculator's cards: studio climate, medium formulator and milestones.

use super::format_duration;
use crate::state::AppState;
use dioxus::prelude::*;
use palettier_domain::drying::{DryingAdvice, DryingEstimate, Workshop};
use palettier_domain::paint::{Medium, Ventilation};

/// The shares of the painter's own medium. They live in the drying screen, so switching
/// to acrylic and back keeps them.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct MediumShares {
    pub oil: Signal<u32>,
    pub solvent: Signal<u32>,
    pub resin: Signal<u32>,
    pub driers: Signal<u32>,
}

impl MediumShares {
    fn set(mut self, oil: u32, solvent: u32, resin: u32, driers: u32) {
        self.oil.set(oil);
        self.solvent.set(solvent);
        self.resin.set(resin);
        self.driers.set(driers);
    }
}

/// Temperature, humidity and ventilation of the workshop, shared by every screen.
#[component]
pub(super) fn StudioClimate() -> Element {
    let mut state = use_context::<AppState>();
    let locale = *state.locale.read();
    let ws = *state.workshop.read();

    rsx! {
        h3 { class: "card-title u-mt-18", {locale.tr("Conditions de l'atelier", "Studio Climate")} }
        div { class: "grid-3",
            div {
                label { class: "field-label", {locale.tr("Température (°C)", "Temperature (°C)")} }
                input {
                    r#type: "number",
                    class: "u-w-100pct",
                    value: "{ws.temperature_celsius}",
                    oninput: move |e| {
                        if let Ok(t) = e.value().parse::<f64>()
                            && let Ok(next) = Workshop::new(t, ws.relative_humidity, ws.ventilation) {
                                state.workshop.set(next);
                            }
                    },
                }
            }
            div {
                label { class: "field-label", {locale.tr("Humidité relative (%)", "Relative Humidity (%)")} }
                input {
                    r#type: "number",
                    class: "u-w-100pct",
                    value: "{ws.relative_humidity}",
                    oninput: move |e| {
                        if let Ok(h) = e.value().parse::<f64>()
                            && let Ok(next) = Workshop::new(ws.temperature_celsius, h, ws.ventilation) {
                                state.workshop.set(next);
                            }
                    },
                }
            }
            div {
                label { class: "field-label", "Ventilation" }
                select {
                    class: "u-w-100pct",
                    onchange: move |e| {
                        let v = match e.value().as_str() {
                            "Confined" => Ventilation::Confined,
                            "Good" => Ventilation::Good,
                            _ => Ventilation::Normal,
                        };
                        state.workshop.set(Workshop {
                            ventilation: v,
                            ..ws
                        });
                    },
                    for v in Ventilation::ALL {
                        option {
                            value: "{v:?}",
                            selected: ws.ventilation == v,
                            "{v.label(locale)}"
                        }
                    }
                }
            }
        }
    }
}

/// One share of the medium formulator, in percent.
#[component]
fn ShareInput(#[props(into)] label: String, share: Signal<u32>) -> Element {
    rsx! {
        div {
            label { class: "field-label", "{label}" }
            input {
                r#type: "number",
                class: "u-w-100pct",
                min: "0",
                max: "100",
                value: "{share}",
                oninput: move |e| {
                    if let Ok(v) = e.value().parse::<u32>() {
                        share.set(v.min(100));
                    }
                }
            }
        }
    }
}

/// The painter's own oil medium: presets, shares, and its fat index and drying factor.
#[component]
pub(super) fn MediumFormulator(shares: MediumShares, fat_index: f64, drying_multiplier: f64) -> Element {
    let state = use_context::<AppState>();
    let locale = *state.locale.read();

    rsx! {
        div { class: "card-subtle u-mt-18 u-bc-accent",
            h4 { class: "u-mb-8",
                "🧪 "
                {locale.tr("Formulateur de Médium Personnalisé", "Custom Medium Formulator")}
            }
            div { class: "muted u-fs-11p5 u-mb-10",
                {locale.tr(
                    "Composez votre propre mélange de liants et solvants. L'indice gras et le multiplicateur de séchage s'ajustent en temps réel.",
                    "Blend your own ratio of oils, thinners, and resins. The fat index and drying multiplier update in real time."
                )}
            }
            div { class: "row u-gap-6 u-mb-12",
                button {
                    class: "btn",
                    class: "u-fs-11 u-p-3-8",
                    onclick: move |_| shares.set(33, 33, 34, 0),
                    "1/3-1/3-1/3 "
                    {locale.tr("Flamand", "Flemish")}
                }
                button {
                    class: "btn",
                    class: "u-fs-11 u-p-3-8",
                    onclick: move |_| shares.set(20, 80, 0, 0),
                    {locale.tr("Lavis maigre (20/80)", "Lean wash (20/80)")}
                }
                button {
                    class: "btn",
                    class: "u-fs-11 u-p-3-8",
                    onclick: move |_| shares.set(50, 25, 25, 1),
                    {locale.tr("Glacis gras (50/25/25)", "Fat glaze (50/25/25)")}
                }
            }
            div { class: "grid-3 u-mb-8",
                ShareInput { label: locale.tr("Huile (%)", "Oil (%)"), share: shares.oil }
                ShareInput { label: locale.tr("Solvant (%)", "Thinner (%)"), share: shares.solvent }
                ShareInput { label: locale.tr("Résine (%)", "Resin (%)"), share: shares.resin }
            }
            div { class: "row-between u-mt-10 u-pt-8 u-bt-1-solid-border-subtle",
                div { class: "row",
                    span { class: "pill pill-accent",
                        {locale.tr("Indice gras calculé", "Calculated fat index")},
                        ": {(fat_index * 100.0).round() as u32}%"
                    }
                    span { class: "pill",
                        {locale.tr("Facteur séchage", "Drying speed factor")},
                        ": {drying_multiplier:.2}×"
                    }
                }
            }
        }
    }
}

/// One milestone of the estimate.
#[component]
fn Milestone(#[props(into)] label: String, value: String, #[props(into)] style: String) -> Element {
    rsx! {
        div { class: "card-subtle",
            div { class: "muted", "{label}" }
            div { style: "{style}", "{value}" }
        }
    }
}

#[component]
pub(super) fn DryingMilestones(estimate: DryingEstimate, current_medium: Medium) -> Element {
    let state = use_context::<AppState>();
    let locale = *state.locale.read();
    let big = "font-size: 22px; font-weight: 700;";

    rsx! {
        div { class: "card",
            h3 { class: "card-title", {locale.tr("Jalons de séchage estimés", "Estimated Drying Milestones")} }
            div { class: "grid-2 u-mb-16",
                Milestone {
                    label: locale.tr("Temps ouvert (fondu dans le frais)", "Open time (wet-in-wet blending)"),
                    value: format_duration(estimate.open_time, locale),
                    style: format!("{big} color: var(--accent);"),
                }
                Milestone {
                    label: locale.tr("Sec au toucher (hors poussière)", "Touch dry (dust-free)"),
                    value: format_duration(estimate.touch_dry, locale),
                    style: big,
                }
                Milestone {
                    label: locale.tr("Recouvrable (glacis ou couche suivante)", "Recoatable (glaze or next coat)"),
                    value: format_duration(estimate.recoat, locale),
                    style: format!("{big} color: var(--pill-ok-text);"),
                }
                Milestone {
                    label: locale.tr("Sec à cœur (empâtement / brossage)", "Through dry (impasto / drybrush)"),
                    value: format_duration(estimate.through_dry, locale),
                    style: big,
                }
            }
            div { class: "card-subtle u-mb-16",
                div { class: "row-between",
                    span { class: "muted", {locale.tr("Polymérisation complète avant vernis définitif", "Full polymerization before final varnish")} }
                    span { class: "pill pill-accent", "{format_duration(estimate.full_cure, locale)}" }
                }
            }

            h4 { class: "u-fs-13 u-mb-8", {locale.tr("Conseils d'atelier & précautions", "Studio Tips & Precautions")} }
            div { class: "card-subtle u-mb-8 u-fs-12p5",
                "• {current_medium.advice(locale)}"
            }
            // The estimate repeats the medium's advice: shown above already.
            for tip in estimate.advice.iter().filter(|t| **t != DryingAdvice::Medium(current_medium)) {
                div { class: "card-subtle u-mb-8 u-fs-12p5",
                    "• {tip.text(locale)}"
                }
            }

        }
    }
}
