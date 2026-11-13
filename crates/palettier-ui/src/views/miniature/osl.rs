//! Light source (OSL): the colours of a glow and their mixes.

use super::shared::{best_mix, open_in_mixer, rgb_or_grey, swatch_option};
use crate::state::{AppServices, AppState};
use crate::theme::{ColorSwatch, DeltaEBadge};
use dioxus::prelude::*;
use palettier_domain::i18n::Locale;
use palettier_domain::miniature::OslPreset;
use palettier_domain::paint::Paint;

#[derive(Clone, Copy)]
pub(super) struct OslState {
    selected: Signal<&'static OslPreset>,
}

impl OslState {
    pub(super) fn new() -> Self {
        Self { selected: Signal::new(&OslPreset::ALL[0]) }
    }
}

#[component]
pub(super) fn OslEngine(candidates: Vec<Paint>) -> Element {
    let state = use_context::<AppState>();
    let services = use_context::<AppServices>();
    let selected_osl = use_context::<OslState>().selected;
    let locale = *state.locale.read();
    let osl = *selected_osl.read();
    let zones = [
        (locale.tr("1. Cœur Incandescent (Point chaud)", "1. White-Hot Core"), osl.core_hex),
        (locale.tr("2. Halo d'Énergie Surchauffé", "2. Saturated Glow"), osl.glow_hex),
        (locale.tr("3. Réflexion Ambiante Décroissante", "3. Ambient Reflection"), osl.ambient_decay_hex),
    ];

    rsx! {
        div { class: "grid-2 u-gtc-1fr-1p35fr",
            div { class: "card",
                h3 { class: "card-title", {locale.tr("Sources d'Énergie & Effets Luminescents (OSL)", "Object Source Lighting (OSL) Energy Sources")} }
                div { class: "u-d-flex u-fd-column u-gap-8",
                    for preset in &OslPreset::ALL {
                        {swatch_option(selected_osl, preset, preset.glow_hex, preset.name_i18n(locale))}
                    }
                }
            }

            div { class: "card",
                div { class: "row-between u-mb-14",
                    div { class: "row u-gap-12 u-ai-center",
                        ColorSwatch { hex: osl.glow_hex.to_string(), size: 40 }
                        div {
                            h3 { class: "card-title u-m-0", "{osl.name_i18n(locale)}" }
                            span { class: "pill pill-accent", "High-Intensity OSL Emitter" }
                        }
                    }

                    button {
                        class: "btn",
                        onclick: {
                            let h = osl.glow_hex.to_string();
                            move |_| open_in_mixer(state, h.clone())
                        },
                        "🧪 ",
                        {locale.tr("Envoyer au Mélangeur", "Send to Mixer")}
                    }
                }

                // 3-step OSL Radial Decay swatches with calculated mixes!
                div { class: "u-d-flex u-fd-column u-gap-8 u-mb-16",
                    for (zone_name, zone_hex) in zones {
                        {zone_row(&services, locale, zone_name, zone_hex, &candidates)}
                    }
                }

                div { class: "card-subtle u-lh-1p5 u-fs-12p5",
                    div { class: "u-fw-700 u-fs-13 u-mb-6 u-c-accent",
                        {locale.tr("🎯 Protocole de Réussite de l'OSL aux Huiles :", "🎯 Pro Oil OSL Painting Protocol:")}
                    }
                    ol { class: "u-ml-20 u-d-flex u-fd-column u-gap-6",
                        li {
                            strong { {locale.tr("Creux en blanc pur : ", "Pure white recess: ")} }
                            {locale.tr("Poser du blanc de titane pur au fond des rainures de plasma ou dans les yeux.", "Lay down pure titanium white in the plasma coils or eye lenses.")}
                        }
                        li {
                            strong { {locale.tr("Filtre d'huile saturé : ", "Saturated oil filter: ")} }
                            {locale.tr("Poser le halo coloré en glacis translucide par-dessus le blanc et sur les arêtes proches.", "Apply the glowing saturated glaze over the white and surrounding armor rims.")}
                        }
                    }
                }
            }
        }
    }
}

fn zone_row(services: &AppServices, locale: Locale, zone_name: &str, zone_hex: &str, candidates: &[Paint]) -> Element {
    let sugg = best_mix(services, rgb_or_grey(zone_hex, 0.5), candidates);
    rsx! {
        div { class: "card-subtle u-p-8-12",
            div { class: "row-between u-ai-center",
                div { class: "row u-gap-10 u-ai-center",
                    ColorSwatch { hex: zone_hex.to_string(), size: 28 }
                    div {
                        div { class: "u-fw-700 u-fs-12", "{zone_name}" }
                        div { class: "mono u-fs-10p5 u-c-text-dim", "{zone_hex}" }
                    }
                }

                if let Some(ref s) = sugg {
                    div { class: "row u-gap-8 u-ai-center",
                        ColorSwatch { hex: s.color.to_hex(), size: 28 }
                        div { class: "u-ta-right u-lh-1p3",
                            div { class: "u-fw-650 u-fs-12 u-c-accent", "{s.describe_i18n(locale)}" }
                            div { class: "u-d-flex u-gap-4 u-jc-flex-end u-ai-center u-mt-2",
                                DeltaEBadge { delta_e: s.delta_e }
                            }
                        }
                    }
                } else {
                    span { class: "muted u-fs-11", {locale.tr("Aucun tube disponible", "No tubes available")} }
                }
            }
        }
    }
}
