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
        div { class: "u-d-flex u-fd-column u-gap-16",
            div { class: "card",
                div { class: "row-between u-mb-12 u-ai-center",
                    h3 { class: "card-title u-m-0",
                        "🪖 ", {locale.tr("Jus & Filtres Véhicules (Équivalents Abteilung 502 / Ammo Mig)", "AFV & Vehicle Weathering (Abteilung 502 / Mig Equivalents)")}
                    }
                    span { class: "pill pill-accent",
                        {locale.tr("Formules aux Tubes d'Huile d'Artiste", "Artist Oil Tube Formulas")}
                    }
                }
                p { class: "muted u-mt-0 u-fs-13",
                    {locale.tr(
                        "Reproduisez fidèlement les teintes cultes de weathering pour chars et véhicules (Abteilung 502, AK, Ammo Mig) à partir de vos tubes d'huile classiques, avec le diluant inodore approprié pour préserver le vernis acrylique.",
                        "Faithfully recreate iconic armor and vehicle weathering washes (Abteilung 502, AK, Ammo Mig) from standard artist oil tubes, using odorless spirits to protect acrylic coats."
                    )}
                }

                div { class: "row u-gap-8 u-mb-16 u-fwrap-wrap",
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
    let thinner = tech.default_medium_in(PaintSystem::Oil).label(locale).to_lowercase();
    rsx! {
        div { class: "card-subtle u-p-16",
            div { class: "row-between u-ai-center u-mb-14",
                div { class: "row u-gap-12 u-ai-center",
                    ColorSwatch { hex: current_fx.target_hex().to_string(), size: 40 }
                    div {
                        div { class: "u-fw-750 u-fs-15 u-c-accent",
                            "{current_fx.commercial_name()}"
                        }
                        div { class: "muted u-fs-12",
                            "{current_fx.category_i18n(locale)}"
                        }
                    }
                }
                span { class: "pill", "{current_fx.finish_i18n(locale)}" }
            }

            div { class: "u-d-grid u-gap-14 u-mb-12",
                div { class: "card u-bg-bg",
                    strong { class: "u-fs-12p5 u-c-accent",
                        {locale.tr("💧 Dilution & Solvant Recommandé :", "💧 Dilution & Recommended Solvent:")}
                    }
                    div { class: "u-fs-12 u-mt-6",
                        div { class: "u-fw-600 u-mb-4",
                            "{tech.label(locale)} · "
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

            div { class: "row-between u-ai-center u-pt-10 u-bt-1-solid-border",
                span { class: "u-fs-12 u-fw-600",
                    {locale.tr("Mélange calculé sur votre palette active :", "Mix computed from your active palette:")}
                }
                if let Some(ref s) = sugg {
                    div { class: "row u-gap-8 u-ai-center",
                        div { class: "u-fw-650 u-fs-12 u-c-accent",
                            "{s.describe_i18n(locale)}"
                        }
                        DeltaEBadge { delta_e: s.delta_e }
                    }
                } else {
                    span { class: "muted u-fs-11", {locale.tr("Aucun tube disponible", "No tubes available")} }
                }
            }
        }
    }
}
