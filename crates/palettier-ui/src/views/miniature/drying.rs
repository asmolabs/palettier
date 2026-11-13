//! Drying & palette: heat box curing and how long mixes stay workable on the palette.

use crate::state::AppState;
use dioxus::prelude::*;
use palettier_domain::miniature::{HeatBoxCalculator, LivePaletteFreshnessTracker};

#[derive(Clone, Copy)]
pub(super) struct DryingState {
    heat_temp: Signal<f64>,
    heat_base_hours: Signal<f64>,
    freshness_minutes: Signal<u32>,
    freshness_temp: Signal<f64>,
}

impl DryingState {
    pub(super) fn new() -> Self {
        Self {
            heat_temp: Signal::new(40.0),
            heat_base_hours: Signal::new(24.0),
            freshness_minutes: Signal::new(90),
            freshness_temp: Signal::new(21.0),
        }
    }
}

#[component]
pub(super) fn HeatBox() -> Element {
    let state = use_context::<AppState>();
    let DryingState { mut heat_temp, heat_base_hours, .. } = use_context::<DryingState>();
    let locale = *state.locale.read();
    let heat_calc = HeatBoxCalculator::new(*heat_temp.read());

    rsx! {
        div { class: "grid-2 u-gtc-1fr-1fr",
            div { class: "card",
                h3 { class: "card-title", {locale.tr("Simulateur de Boîte Chauffante / Déshydrateur", "Heat Box / Dehydrator Curing Simulator")} }
                p { class: "muted u-fs-12 u-mb-14",
                    {locale.tr("L'huile polymérise beaucoup plus vite avec la chaleur (loi d'Arrhenius : le taux double tous les 10°C). Une pièce séchant en 24h à 20°C sèche en quelques heures à 40°C.", "Oil cross-linking accelerates under heat (Arrhenius rate: doubles every 10°C). A piece taking 24h at 20°C dries in a few hours at 40°C.")}
                }

                div { class: "card-subtle u-mb-16",
                    div { class: "row-between u-mb-8",
                        strong { {locale.tr("Température de la boîte chauffante :", "Chamber Temperature:")} }
                        span { class: "pill pill-accent u-fs-14 u-fw-800", "{heat_temp:.0} °C" }
                    }
                    input {
                        r#type: "range",
                        class: "u-w-100pct",
                        min: "20",
                        max: "55",
                        value: "{heat_temp}",
                        oninput: move |ev| {
                            if let Ok(v) = ev.value().parse::<f64>() {
                                heat_temp.set(v);
                            }
                        }
                    }
                }

                if let Some(warn) = heat_calc.safety_warning_i18n(locale) {
                    div { class: "card-subtle u-bc-danger u-bg-pill-danger-bg u-mb-12",
                        strong { class: "u-c-danger u-fs-12", "{warn}" }
                    }
                }
            }

            div { class: "card",
                h3 { class: "card-title", {locale.tr("Gain de Temps & Sécurité Figurine", "Time Savings & Miniature Safety")} }
                {
                    let factor = heat_calc.acceleration_factor();
                    let base_h = *heat_base_hours.read();
                    let accel_h = heat_calc.accelerated_hours(base_h);
                    rsx! {
                        div { class: "card-subtle u-ta-center u-p-20 u-mb-16",
                            div { class: "muted u-fs-12", {locale.tr("Facteur d'accélération thermique :", "Thermal Acceleration Factor:")} }
                            div { class: "mono u-fs-32 u-fw-800 u-c-accent u-m-6-0",
                                "{factor:.1}x plus rapide"
                            }
                            div { class: "u-fs-13p5 u-fw-600",
                                "{base_h:.0} h ", {locale.tr("à température ambiante (20°C)", "at ambient (20°C)")},
                                " ➔ ",
                                strong { class: "u-c-pill-ok-text", "{accel_h:.1} h " },
                                {locale.tr("en boîte chauffante", "in heat chamber")}
                            }
                        }

                        div { class: "card-subtle",
                            div { class: "u-fw-700 u-fs-12 u-mb-4",
                                {locale.tr("🛡️ Recommandation Matériaux Figurines :", "🛡️ Miniature Materials Guideline:")}
                            }
                            p { class: "muted u-m-0 u-fs-12 u-lh-1p45",
                                {locale.tr(
                                    "Réglez idéalement votre déshydrateur ou boîte entre 38°C et 42°C. C'est la zone idéale : 4x plus rapide et 100% sûr pour le plastique Games Workshop, l'impression 3D résine et le métal.",
                                    "Ideal dehydrator/box setting is 38°C to 42°C: achieves ~4x faster curing while remaining 100% safe for Games Workshop plastic, 3D resin prints, and white metal."
                                )}
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub(super) fn PaletteFreshness() -> Element {
    let state = use_context::<AppState>();
    let DryingState { mut freshness_minutes, mut freshness_temp, .. } = use_context::<DryingState>();
    let locale = *state.locale.read();

    rsx! {
        div { class: "u-d-flex u-fd-column u-gap-16",
            div { class: "card",
                div { class: "row-between u-mb-12 u-ai-center",
                    h3 { class: "card-title u-m-0",
                        "⏳ ", {locale.tr("Widget de Fraîcheur de Palette en Temps Réel", "Live Studio Palette Freshness Tracker")}
                    }
                    span { class: "pill pill-accent",
                        {locale.tr("Dégradation Viscosité & Polymérisation", "Viscosity & Polymerization Drift")}
                    }
                }
                p { class: "muted u-mt-0 u-fs-13",
                    {locale.tr(
                        "Suivez la transformation physique de vos plots de peinture à l'huile pendant votre session. Sachez exactement quand le solvant s'est évaporé (phase beurre OPR) et quand éviter de rajouter du diluant pour ne pas créer de grumeaux.",
                        "Track the physical drift of oil paint dollops during your session. Know exactly when solvent has flashed off (buttery OPR phase) and when to stop adding thinner to avoid curdling."
                    )}
                }

                div { class: "grid-2 u-gap-16 u-mb-16",
                    div { class: "card-subtle",
                        label { class: "u-fw-700 u-fs-13 u-d-block u-mb-6",
                            {locale.tr("Temps écoulé depuis la sortie du tube : ", "Elapsed time since squeeze: ")},
                            "{*freshness_minutes.read() / 60}h {*freshness_minutes.read() % 60}m"
                        }
                        input {
                            r#type: "range",
                            min: "0",
                            max: "900",
                            step: "15",
                            value: "{freshness_minutes}",
                            oninput: move |ev: Event<FormData>| {
                                if let Ok(v) = ev.value().parse::<u32>() {
                                    freshness_minutes.set(v);
                                }
                            }
                        }
                    }

                    div { class: "card-subtle",
                        label { class: "u-fw-700 u-fs-13 u-d-block u-mb-6",
                            {locale.tr("Température de l'atelier : ", "Workshop temperature: ")},
                            "{freshness_temp:.1}°C"
                        }
                        input {
                            r#type: "range",
                            min: "15",
                            max: "35",
                            step: "1",
                            value: "{freshness_temp}",
                            oninput: move |ev: Event<FormData>| {
                                if let Ok(v) = ev.value().parse::<f64>() {
                                    freshness_temp.set(v);
                                }
                            }
                        }
                    }
                }

                {
                    let phase = LivePaletteFreshnessTracker::evaluate(*freshness_minutes.read(), *freshness_temp.read());
                    rsx! {
                        div { class: "card-subtle u-bl-4-solid-accent u-p-16",
                            div { class: "row-between u-mb-10 u-ai-center",
                                span { class: "{phase.badge_class()}", class: "u-fs-13 u-p-5-12 u-fw-700",
                                    "{phase.title_i18n(locale)}"
                                }
                            }

                            div { class: "u-fs-13 u-lh-1p5 u-mb-10",
                                strong { {locale.tr("État physique de la peinture : ", "Physical state: ")} }
                                "{phase.physical_state_i18n(locale)}"
                            }

                            div { class: "u-fs-12p5 u-lh-1p5 u-p-10 u-bg-bg u-r-6",
                                strong { class: "u-c-accent",
                                    "💡 ", {locale.tr("Action recommandée : ", "Recommended action: ")}
                                }
                                "{phase.action_advice_i18n(locale)}"
                            }
                        }
                    }
                }
            }
        }
    }
}
