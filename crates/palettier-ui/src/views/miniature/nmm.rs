//! Non-metallic metal: the five tones of a metal family and their mixes.

use super::shared::{best_mix, open_in_mixer, rgb_or_grey, swatch_option};
use crate::state::{AppServices, AppState};
use crate::theme::{ColorSwatch, DeltaEBadge};
use dioxus::prelude::*;
use palettier_domain::i18n::Locale;
use palettier_domain::miniature::NmmMetal;
use palettier_domain::paint::Paint;

#[derive(Clone, Copy)]
pub(super) struct NmmState {
    selected: Signal<NmmMetal>,
}

impl NmmState {
    pub(super) fn new() -> Self {
        Self { selected: Signal::new(NmmMetal::Gold) }
    }
}

/// Name of the NMM step `idx`.
fn step_label(locale: Locale, idx: usize) -> &'static str {
    match idx {
        0 => locale.tr("1. Ombre d'occlusion (Max)", "1. Max Occlusion Shadow"),
        1 => locale.tr("2. Ombre de forme", "2. Form Shadow"),
        2 => locale.tr("3. Base métal", "3. Metal Midtone Base"),
        3 => locale.tr("4. Pleine lumière", "4. Key Highlight"),
        4 => locale.tr("5. Rehaut spéculaire", "5. Specular Catchlight"),
        _ => "",
    }
}

#[component]
pub(super) fn NmmStudio(candidates: Vec<Paint>, palette_name: String) -> Element {
    let state = use_context::<AppState>();
    let services = use_context::<AppServices>();
    let selected_nmm = use_context::<NmmState>().selected;
    let locale = *state.locale.read();
    let nmm = *selected_nmm.read();
    let steps = nmm.steps_hex();

    rsx! {
        div { class: "grid-2 u-gtc-1fr-1p35fr",
            // Left: NMM Family selector
            div { class: "card",
                h3 { class: "card-title", {locale.tr("Familles de Métaux Non-Métalliques (NMM)", "Non-Metallic Metal (NMM) Families")} }
                p { class: "muted u-fs-12 u-mb-14",
                    {locale.tr("Le NMM recrée l'illusion du métal poli sans particules scintillantes, par le contraste extrême d'ombres sourdes et de blancs spéculaires.", "NMM recreates the illusion of polished metal using extreme value contrast between deep shadows and pure white catchlights.")}
                }

                div { class: "u-d-flex u-fd-column u-gap-8",
                    for nmm in NmmMetal::ALL {
                        {swatch_option(selected_nmm, nmm, nmm.steps_hex()[2], nmm.label(locale))}
                    }
                }
            }

            // Right: NMM 5-Step Scale & Palette Mix Calculator
            div { class: "card",
                div { class: "row-between u-mb-14 u-fwrap-wrap u-gap-8",
                    div { class: "row u-gap-12 u-ai-center",
                        ColorSwatch { hex: steps[2].to_string(), size: 40 }
                        div {
                            h3 { class: "card-title u-m-0", "{nmm.label(locale)}" }
                            span { class: "pill pill-accent", {locale.tr("Reflet spéculaire contrasté", "High-contrast specular")} }
                        }
                    }

                    span { class: "pill u-fs-11p5",
                        {locale.tr("Calculé avec : ", "Calculated with: ")},
                        strong { "{palette_name}" }
                    }
                }

                // Computed mix rows for the 5 NMM steps
                div { class: "u-d-flex u-fd-column u-gap-8 u-mb-16",
                    for (idx, step_hex_str) in steps.iter().enumerate() {
                        {step_row(state, &services, locale, idx, step_hex_str, &candidates)}
                    }
                }

                div { class: "card-subtle",
                    div { class: "u-fw-700 u-fs-12 u-mb-4 u-c-accent",
                        {locale.tr("🌍 Règle du reflet de sol (Ground Bounce) :", "🌍 Ground Bounce Reflection Physics:")}
                    }
                    div { class: "u-fs-12p5 u-lh-1p45",
                        "{nmm.ground_bounce_advice_i18n(locale)}"
                    }
                }
            }
        }
    }
}

fn step_row(
    state: AppState,
    services: &AppServices,
    locale: Locale,
    idx: usize,
    step_hex_str: &str,
    candidates: &[Paint],
) -> Element {
    let tag = step_label(locale, idx);
    let sugg = best_mix(services, rgb_or_grey(step_hex_str, 0.5), candidates);
    rsx! {
        div { class: "card-subtle u-p-8-12",
            div { class: "row-between u-ai-center",
                div { class: "row u-gap-10 u-ai-center",
                    ColorSwatch { hex: step_hex_str.to_string(), size: 28 }
                    div {
                        div { class: "u-fw-700 u-fs-12", "{tag}" }
                        div { class: "mono u-fs-10p5 u-c-text-dim", "{step_hex_str}" }
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
                        button {
                            class: "btn",
                            class: "u-p-3-8 u-fs-11",
                            title: locale.tr("Ouvrir cette teinte dans le Mélangeur", "Open target in Mixer"),
                            onclick: {
                                let h = step_hex_str.to_string();
                                move |_| open_in_mixer(state, h.clone())
                            },
                            "🧪"
                        }
                    }
                } else {
                    span { class: "muted u-fs-11p5",
                        {locale.tr("Aucun tube disponible dans la palette", "No tubes available in palette")}
                    }
                }
            }
        }
    }
}
