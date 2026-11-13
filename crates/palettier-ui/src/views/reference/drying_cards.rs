//! The drying calculator's cards: studio climate, medium formulator and milestones.

use super::format_duration;
use crate::state::AppState;
use dioxus::prelude::*;
use palettier_domain::drying::{DryingEstimate, Workshop};
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
        h3 { class: "card-title", style: "margin-top: 18px;", {locale.tr("Conditions de l'atelier", "Studio Climate")} }
        div { class: "grid-3",
            div {
                label { class: "field-label", {locale.tr("Température (°C)", "Temperature (°C)")} }
                input {
                    r#type: "number",
                    style: "width: 100%;",
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
                    style: "width: 100%;",
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
                    style: "width: 100%;",
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
                            "{v.label_i18n(locale)}"
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
                style: "width: 100%;",
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
        div { class: "card-subtle", style: "margin-top: 18px; border-color: var(--accent);",
            h4 { style: "margin-bottom: 8px;",
                "🧪 "
                {locale.tr("Formulateur de Médium Personnalisé", "Custom Medium Formulator")}
            }
            div { class: "muted", style: "font-size: 11.5px; margin-bottom: 10px;",
                {locale.tr(
                    "Composez votre propre mélange de liants et solvants. L'indice gras et le multiplicateur de séchage s'ajustent en temps réel.",
                    "Blend your own ratio of oils, thinners, and resins. The fat index and drying multiplier update in real time."
                )}
            }
            div { class: "row", style: "gap: 6px; margin-bottom: 12px;",
                button {
                    class: "btn",
                    style: "font-size: 11px; padding: 3px 8px;",
                    onclick: move |_| shares.set(33, 33, 34, 0),
                    "1/3-1/3-1/3 "
                    {locale.tr("Flamand", "Flemish")}
                }
                button {
                    class: "btn",
                    style: "font-size: 11px; padding: 3px 8px;",
                    onclick: move |_| shares.set(20, 80, 0, 0),
                    {locale.tr("Lavis maigre (20/80)", "Lean wash (20/80)")}
                }
                button {
                    class: "btn",
                    style: "font-size: 11px; padding: 3px 8px;",
                    onclick: move |_| shares.set(50, 25, 25, 1),
                    {locale.tr("Glacis gras (50/25/25)", "Fat glaze (50/25/25)")}
                }
            }
            div { class: "grid-3", style: "margin-bottom: 8px;",
                ShareInput { label: locale.tr("Huile (%)", "Oil (%)"), share: shares.oil }
                ShareInput { label: locale.tr("Solvant (%)", "Thinner (%)"), share: shares.solvent }
                ShareInput { label: locale.tr("Résine (%)", "Resin (%)"), share: shares.resin }
            }
            div { class: "row-between", style: "margin-top: 10px; padding-top: 8px; border-top: 1px solid var(--border-subtle);",
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
            div { class: "grid-2", style: "margin-bottom: 16px;",
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
            div { class: "card-subtle", style: "margin-bottom: 16px;",
                div { class: "row-between",
                    span { class: "muted", {locale.tr("Polymérisation complète avant vernis définitif", "Full polymerization before final varnish")} }
                    span { class: "pill pill-accent", "{format_duration(estimate.full_cure, locale)}" }
                }
            }

            h4 { style: "font-size: 13px; margin-bottom: 8px;", {locale.tr("Conseils d'atelier & précautions", "Studio Tips & Precautions")} }
            div { class: "card-subtle", style: "margin-bottom: 8px; font-size: 12.5px;",
                "• {current_medium.advice_i18n(locale)}"
            }
            // The estimate repeats the medium's advice (untranslated): shown above already.
            for tip in estimate.advice.iter().filter(|t| t.as_str() != current_medium.advice()) {
                div { class: "card-subtle", style: "margin-bottom: 8px; font-size: 12.5px;",
                    "• {tip}"
                }
            }

        }
    }
}
