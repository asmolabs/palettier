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
        div { class: "grid-2", style: "grid-template-columns: 1fr 1.35fr;",
            // Left: NMM Family selector
            div { class: "card",
                h3 { class: "card-title", {locale.tr("Familles de Métaux Non-Métalliques (NMM)", "Non-Metallic Metal (NMM) Families")} }
                p { class: "muted", style: "font-size: 12px; margin-bottom: 14px;",
                    {locale.tr("Le NMM recrée l'illusion du métal poli sans particules scintillantes, par le contraste extrême d'ombres sourdes et de blancs spéculaires.", "NMM recreates the illusion of polished metal using extreme value contrast between deep shadows and pure white catchlights.")}
                }

                div { style: "display: flex; flex-direction: column; gap: 8px;",
                    for nmm in NmmMetal::ALL {
                        {swatch_option(selected_nmm, nmm, nmm.steps_hex()[2], nmm.label(locale))}
                    }
                }
            }

            // Right: NMM 5-Step Scale & Palette Mix Calculator
            div { class: "card",
                div { class: "row-between", style: "margin-bottom: 14px; flex-wrap: wrap; gap: 8px;",
                    div { class: "row", style: "gap: 12px; align-items: center;",
                        ColorSwatch { hex: steps[2].to_string(), size: 40 }
                        div {
                            h3 { class: "card-title", style: "margin: 0;", "{nmm.label(locale)}" }
                            span { class: "pill pill-accent", {locale.tr("Reflet spéculaire contrasté", "High-contrast specular")} }
                        }
                    }

                    span { class: "pill", style: "font-size: 11.5px;",
                        {locale.tr("Calculé avec : ", "Calculated with: ")},
                        strong { "{palette_name}" }
                    }
                }

                // Computed mix rows for the 5 NMM steps
                div { style: "display: flex; flex-direction: column; gap: 8px; margin-bottom: 16px;",
                    for (idx, step_hex_str) in steps.iter().enumerate() {
                        {step_row(state, &services, locale, idx, step_hex_str, &candidates)}
                    }
                }

                div { class: "card-subtle",
                    div { style: "font-weight: 700; font-size: 12px; margin-bottom: 4px; color: var(--accent);",
                        {locale.tr("🌍 Règle du reflet de sol (Ground Bounce) :", "🌍 Ground Bounce Reflection Physics:")}
                    }
                    div { style: "font-size: 12.5px; line-height: 1.45;",
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
        div { class: "card-subtle", style: "padding: 8px 12px;",
            div { class: "row-between", style: "align-items: center;",
                div { class: "row", style: "gap: 10px; align-items: center;",
                    ColorSwatch { hex: step_hex_str.to_string(), size: 28 }
                    div {
                        div { style: "font-weight: 700; font-size: 12px;", "{tag}" }
                        div { class: "mono", style: "font-size: 10.5px; color: var(--text-dim);", "{step_hex_str}" }
                    }
                }

                if let Some(ref s) = sugg {
                    div { class: "row", style: "gap: 8px; align-items: center;",
                        ColorSwatch { hex: s.color.to_hex(), size: 28 }
                        div { style: "text-align: right; line-height: 1.3;",
                            div { style: "font-weight: 650; font-size: 12px; color: var(--accent);", "{s.describe_i18n(locale)}" }
                            div { style: "display: flex; gap: 4px; justify-content: flex-end; align-items: center; margin-top: 2px;",
                                DeltaEBadge { delta_e: s.delta_e }
                            }
                        }
                        button {
                            class: "btn",
                            style: "padding: 3px 8px; font-size: 11px;",
                            title: locale.tr("Ouvrir cette teinte dans le Mélangeur", "Open target in Mixer"),
                            onclick: {
                                let h = step_hex_str.to_string();
                                move |_| open_in_mixer(state, h.clone())
                            },
                            "🧪"
                        }
                    }
                } else {
                    span { class: "muted", style: "font-size: 11.5px;",
                        {locale.tr("Aucun tube disponible dans la palette", "No tubes available in palette")}
                    }
                }
            }
        }
    }
}
