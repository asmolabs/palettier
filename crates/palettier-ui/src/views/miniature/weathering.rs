//! Weathering & grimdark, and blood, ice & FX: an effect, its mix from the palette and how
//! to apply it.

use super::shared::{best_mix, open_in_mixer, rgb_or_grey, swatch_option};
use crate::state::{AppServices, AppState};
use crate::theme::{ColorSwatch, DeltaEBadge};
use dioxus::prelude::*;
use palettier_domain::i18n::Locale;
use palettier_domain::miniature::{SpecialEffectGore, WeatheringEffect};
use palettier_domain::mix::MixSuggestion;
use palettier_domain::paint::Paint;

#[derive(Clone, Copy)]
pub(super) struct WeatheringState {
    weathering: Signal<WeatheringEffect>,
    gore: Signal<SpecialEffectGore>,
}

impl WeatheringState {
    pub(super) fn new() -> Self {
        Self {
            weathering: Signal::new(WeatheringEffect::StreakingGrime),
            gore: Signal::new(SpecialEffectGore::FreshGore),
        }
    }
}

#[component]
pub(super) fn Weathering(candidates: Vec<Paint>, palette_name: String) -> Element {
    let state = use_context::<AppState>();
    let services = use_context::<AppServices>();
    let selected_weathering = use_context::<WeatheringState>().weathering;
    let locale = *state.locale.read();
    let eff = *selected_weathering.read();
    let sugg = best_mix(&services, rgb_or_grey(eff.hex_color(), 0.5), &candidates);

    rsx! {
        div { class: "grid-2 u-gtc-1fr-1p35fr",
            div { class: "card",
                h3 { class: "card-title", {locale.tr("Effets d'Intempéries & Grimdark aux Huiles", "Oil Weathering & Grimdark Effects")} }
                p { class: "muted u-fs-12 u-mb-14",
                    {locale.tr("Les huiles sont reines pour la patine réaliste : temps de travail illimité, estompage sans traces et capillarité naturelle.", "Oils excel at realistic weathering: unlimited open-time, seamless feathering, and natural capillary action.")}
                }

                div { class: "u-d-flex u-fd-column u-gap-8",
                    for eff in WeatheringEffect::ALL {
                        {swatch_option(selected_weathering, eff, eff.hex_color(), eff.label(locale))}
                    }
                }
            }

            div { class: "card",
                div { class: "row-between u-mb-14",
                    div { class: "row u-gap-12 u-ai-center",
                        ColorSwatch { hex: eff.hex_color().to_string(), size: 40 }
                        div {
                            h3 { class: "card-title u-m-0", "{eff.label(locale)}" }
                            span { class: "pill pill-accent u-mt-2", "Grimdark Oil Weathering" }
                        }
                    }

                    button {
                        class: "btn",
                        onclick: {
                            let h = eff.hex_color().to_string();
                            move |_| open_in_mixer(state, h.clone())
                        },
                        "🧪 ",
                        {locale.tr("Ouvrir dans le Mélangeur", "Open in Mixer")}
                    }
                }

                // Dynamic calculated mix from chosen palette!
                {palette_mix(locale, &palette_name, sugg.as_ref())}

                div { class: "card-subtle",
                    div { class: "u-fw-700 u-fs-12 u-mb-4 u-c-accent",
                        {locale.tr("🖌️ Technique d'application sur figurine :", "🖌️ Miniature Application Technique:")}
                    }
                    div { class: "u-fs-12p5 u-lh-1p45",
                        "{eff.application_technique_i18n(locale)}"
                    }
                }
            }
        }
    }
}

#[component]
pub(super) fn SpecialEffects(candidates: Vec<Paint>, palette_name: String) -> Element {
    let state = use_context::<AppState>();
    let services = use_context::<AppServices>();
    let selected_gore = use_context::<WeatheringState>().gore;
    let locale = *state.locale.read();
    let gore = *selected_gore.read();
    let sugg = best_mix(&services, rgb_or_grey(gore.hex_color(), 0.5), &candidates);

    rsx! {
        div { class: "grid-2 u-gtc-1fr-1p35fr",
            div { class: "card",
                h3 { class: "card-title", {locale.tr("Effets Spéciaux & Textures aux Huiles", "Special Effects & Oil Textures")} }
                div { class: "u-d-flex u-fd-column u-gap-8",
                    for gore in SpecialEffectGore::ALL {
                        {swatch_option(selected_gore, gore, gore.hex_color(), gore.label(locale))}
                    }
                }
            }

            div { class: "card",
                div { class: "row-between u-mb-14",
                    div { class: "row u-gap-12 u-ai-center",
                        ColorSwatch { hex: gore.hex_color().to_string(), size: 40 }
                        div {
                            h3 { class: "card-title u-m-0", "{gore.label(locale)}" }
                            span { class: "pill pill-accent", "Special FX Miniature Protocol" }
                        }
                    }
                }

                // Dynamic calculated mix from chosen palette!
                {palette_mix(locale, &palette_name, sugg.as_ref())}

                div { class: "card-subtle",
                    div { class: "u-fw-700 u-fs-12 u-mb-4 u-c-accent",
                        {locale.tr("🖌️ Technique d'application :", "🖌️ Application Technique:")}
                    }
                    div { class: "u-fs-12p5 u-lh-1p45",
                        "{gore.technique_i18n(locale)}"
                    }
                }
            }
        }
    }
}

/// The mix of the effect computed on the active palette.
fn palette_mix(locale: Locale, palette_name: &str, sugg: Option<&MixSuggestion>) -> Element {
    rsx! {
        div { class: "card-subtle u-mb-14 u-b-1-solid-accent-border",
            div { class: "u-fw-700 u-fs-12 u-mb-6 u-c-accent",
                {locale.tr("🧪 Recette calculée avec votre palette : ", "🧪 Mix calculated with your palette: ")},
                strong { "{palette_name}" }
            }
            if let Some(s) = sugg {
                div { class: "row-between u-ai-center",
                    div { class: "row u-gap-10 u-ai-center",
                        ColorSwatch { hex: s.color.to_hex(), size: 32 }
                        div {
                            div { class: "u-fw-750 u-fs-13p5", "{s.describe_i18n(locale)}" }
                            div { class: "muted u-fs-11", "{s.verdict(locale)}" }
                        }
                    }
                    DeltaEBadge { delta_e: s.delta_e }
                }
            } else {
                div { class: "muted u-fs-12",
                    {locale.tr("Aucun tube disponible dans la palette sélectionnée pour reproduire cet effet.", "No matching tubes available in selected palette for this effect.")}
                }
            }
        }
    }
}
