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
        div { class: "grid-2", style: "grid-template-columns: 1fr 1fr;",
            div { class: "card",
                h3 { class: "card-title", {locale.tr("Simulateur de Boîte Chauffante / Déshydrateur", "Heat Box / Dehydrator Curing Simulator")} }
                p { class: "muted", style: "font-size: 12px; margin-bottom: 14px;",
                    {locale.tr("L'huile polymérise beaucoup plus vite avec la chaleur (loi d'Arrhenius : le taux double tous les 10°C). Une pièce séchant en 24h à 20°C sèche en quelques heures à 40°C.", "Oil cross-linking accelerates under heat (Arrhenius rate: doubles every 10°C). A piece taking 24h at 20°C dries in a few hours at 40°C.")}
                }

                div { class: "card-subtle", style: "margin-bottom: 16px;",
                    div { class: "row-between", style: "margin-bottom: 8px;",
                        strong { {locale.tr("Température de la boîte chauffante :", "Chamber Temperature:")} }
                        span { class: "pill pill-accent", style: "font-size: 14px; font-weight: 800;", "{heat_temp:.0} °C" }
                    }
                    input {
                        r#type: "range",
                        style: "width: 100%;",
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
                    div { class: "card-subtle", style: "border-color: var(--danger); background: var(--pill-danger-bg); margin-bottom: 12px;",
                        strong { style: "color: var(--danger); font-size: 12px;", "{warn}" }
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
                        div { class: "card-subtle", style: "text-align: center; padding: 20px; margin-bottom: 16px;",
                            div { class: "muted", style: "font-size: 12px;", {locale.tr("Facteur d'accélération thermique :", "Thermal Acceleration Factor:")} }
                            div { class: "mono", style: "font-size: 32px; font-weight: 800; color: var(--accent); margin: 6px 0;",
                                "{factor:.1}x plus rapide"
                            }
                            div { style: "font-size: 13.5px; font-weight: 600;",
                                "{base_h:.0} h ", {locale.tr("à température ambiante (20°C)", "at ambient (20°C)")},
                                " ➔ ",
                                strong { style: "color: var(--pill-ok-text);", "{accel_h:.1} h " },
                                {locale.tr("en boîte chauffante", "in heat chamber")}
                            }
                        }

                        div { class: "card-subtle",
                            div { style: "font-weight: 700; font-size: 12px; margin-bottom: 4px;",
                                {locale.tr("🛡️ Recommandation Matériaux Figurines :", "🛡️ Miniature Materials Guideline:")}
                            }
                            p { class: "muted", style: "margin: 0; font-size: 12px; line-height: 1.45;",
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
        div { style: "display: flex; flex-direction: column; gap: 16px;",
            div { class: "card",
                div { class: "row-between", style: "margin-bottom: 12px; align-items: center;",
                    h3 { class: "card-title", style: "margin: 0;",
                        "⏳ ", {locale.tr("Widget de Fraîcheur de Palette en Temps Réel", "Live Studio Palette Freshness Tracker")}
                    }
                    span { class: "pill pill-accent",
                        {locale.tr("Dégradation Viscosité & Polymérisation", "Viscosity & Polymerization Drift")}
                    }
                }
                p { class: "muted", style: "margin-top: 0; font-size: 13px;",
                    {locale.tr(
                        "Suivez la transformation physique de vos plots de peinture à l'huile pendant votre session. Sachez exactement quand le solvant s'est évaporé (phase beurre OPR) et quand éviter de rajouter du diluant pour ne pas créer de grumeaux.",
                        "Track the physical drift of oil paint dollops during your session. Know exactly when solvent has flashed off (buttery OPR phase) and when to stop adding thinner to avoid curdling."
                    )}
                }

                div { class: "grid-2", style: "gap: 16px; margin-bottom: 16px;",
                    div { class: "card-subtle",
                        label { style: "font-weight: 700; font-size: 13px; display: block; margin-bottom: 6px;",
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
                        label { style: "font-weight: 700; font-size: 13px; display: block; margin-bottom: 6px;",
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
                        div { class: "card-subtle", style: "border-left: 4px solid var(--accent); padding: 16px;",
                            div { class: "row-between", style: "margin-bottom: 10px; align-items: center;",
                                span { class: phase.badge_class(), style: "font-size: 13px; padding: 5px 12px; font-weight: 700;",
                                    "{phase.title_i18n(locale)}"
                                }
                            }

                            div { style: "font-size: 13px; line-height: 1.5; margin-bottom: 10px;",
                                strong { {locale.tr("État physique de la peinture : ", "Physical state: ")} }
                                "{phase.physical_state_i18n(locale)}"
                            }

                            div { style: "font-size: 12.5px; line-height: 1.5; padding: 10px; background: var(--bg); border-radius: 6px;",
                                strong { style: "color: var(--accent);",
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
