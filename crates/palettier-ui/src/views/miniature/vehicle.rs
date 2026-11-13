//! Vehicle washes & filters: commercial weathering products redone with oil tubes.

use super::shared::{best_mix, rgb_or_grey};
use crate::state::{AppServices, AppState};
use crate::theme::{ColorSwatch, DeltaEBadge};
use dioxus::prelude::*;
use palettier_domain::i18n::Locale;
use palettier_domain::miniature::{VehicleEffectKind, thinner_drops_per_part};
use palettier_domain::paint::{Paint, PaintSystem};

#[derive(Clone, Copy)]
pub(super) struct VehicleState {
    selected: Signal<VehicleEffectKind>,
}

impl VehicleState {
    pub(super) fn new() -> Self {
        Self { selected: Signal::new(VehicleEffectKind::StarshipFilth) }
    }
}

#[component]
pub(super) fn VehicleWeathering(candidates: Vec<Paint>) -> Element {
    let state = use_context::<AppState>();
    let services = use_context::<AppServices>();
    let mut selected_vehicle_fx = use_context::<VehicleState>().selected;
    let locale = *state.locale.read();

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 16px;",
            div { class: "card",
                div { class: "row-between", style: "margin-bottom: 12px; align-items: center;",
                    h3 { class: "card-title", style: "margin: 0;",
                        "🪖 ", {locale.tr("Jus & Filtres Véhicules (Équivalents Abteilung 502 / Ammo Mig)", "AFV & Vehicle Weathering (Abteilung 502 / Mig Equivalents)")}
                    }
                    span { class: "pill pill-accent",
                        {locale.tr("Formules aux Tubes d'Huile d'Artiste", "Artist Oil Tube Formulas")}
                    }
                }
                p { class: "muted", style: "margin-top: 0; font-size: 13px;",
                    {locale.tr(
                        "Reproduisez fidèlement les teintes cultes de weathering pour chars et véhicules (Abteilung 502, AK, Ammo Mig) à partir de vos tubes d'huile classiques, avec le diluant inodore approprié pour préserver le vernis acrylique.",
                        "Faithfully recreate iconic armor and vehicle weathering washes (Abteilung 502, AK, Ammo Mig) from standard artist oil tubes, using odorless spirits to protect acrylic coats."
                    )}
                }

                div { class: "row", style: "gap: 8px; margin-bottom: 16px; flex-wrap: wrap;",
                    for fx in VehicleEffectKind::ALL {
                        button {
                            class: if *selected_vehicle_fx.read() == fx { "btn-primary" } else { "btn" },
                            onclick: move |_| selected_vehicle_fx.set(fx),
                            "{fx.commercial_name()}"
                        }
                    }
                }

                {effect_card(&services, locale, *selected_vehicle_fx.read(), &candidates)}
            }
        }
    }
}

/// The chosen product: its look, dilution and the mix of its tone on the palette.
fn effect_card(services: &AppServices, locale: Locale, current_fx: VehicleEffectKind, candidates: &[Paint]) -> Element {
    let sugg = best_mix(services, rgb_or_grey(current_fx.target_hex(), 0.3), candidates);
    let tech = current_fx.technique();
    let drops = thinner_drops_per_part(tech, PaintSystem::Oil);
    let thinner = tech.default_medium_in(PaintSystem::Oil).label_i18n(locale).to_lowercase();
    rsx! {
        div { class: "card-subtle", style: "padding: 16px;",
            div { class: "row-between", style: "align-items: center; margin-bottom: 14px;",
                div { class: "row", style: "gap: 12px; align-items: center;",
                    ColorSwatch { hex: current_fx.target_hex().to_string(), size: 40 }
                    div {
                        div { style: "font-weight: 750; font-size: 15px; color: var(--accent);",
                            "{current_fx.commercial_name()}"
                        }
                        div { class: "muted", style: "font-size: 12px;",
                            "{current_fx.category_i18n(locale)}"
                        }
                    }
                }
                span { class: "pill", "{current_fx.finish_i18n(locale)}" }
            }

            div { style: "display: grid; gap: 14px; margin-bottom: 12px;",
                div { class: "card", style: "background: var(--bg);",
                    strong { style: "font-size: 12.5px; color: var(--accent);",
                        {locale.tr("💧 Dilution & Solvant Recommandé :", "💧 Dilution & Recommended Solvent:")}
                    }
                    div { style: "font-size: 12px; margin-top: 6px;",
                        div { style: "font-weight: 600; margin-bottom: 4px;",
                            "{tech.label_i18n(locale)} · "
                            if drops == 0 {
                                {locale.tr("pâte pure", "neat paste")}
                            } else {
                                {locale.tr("1 part de pâte pour ", "1 part paste to ")}, "{drops} ", {locale.tr("gouttes de ", "drops of ")}, "{thinner}"
                            }
                        }
                        div { class: "muted", "{current_fx.application_note_i18n(locale)}" }
                    }
                }
            }

            div { class: "row-between", style: "align-items: center; padding-top: 10px; border-top: 1px solid var(--border);",
                span { style: "font-size: 12px; font-weight: 600;",
                    {locale.tr("Mélange calculé sur votre palette active :", "Mix computed from your active palette:")}
                }
                if let Some(ref s) = sugg {
                    div { class: "row", style: "gap: 8px; align-items: center;",
                        div { style: "font-weight: 650; font-size: 12px; color: var(--accent);",
                            "{s.describe_i18n(locale)}"
                        }
                        DeltaEBadge { delta_e: s.delta_e }
                    }
                } else {
                    span { class: "muted", style: "font-size: 11px;", {locale.tr("Aucun tube disponible", "No tubes available")} }
                }
            }
        }
    }
}
