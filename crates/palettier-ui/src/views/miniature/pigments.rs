//! Dry pigments: dry and fixed tones, the equivalent oil glaze and how to apply them.

use super::shared::{best_mix, rgb_or_grey};
use crate::state::{AppServices, AppState};
use crate::theme::{ColorSwatch, DeltaEBadge};
use dioxus::prelude::*;
use palettier_domain::i18n::Locale;
use palettier_domain::miniature::DryPigmentEffect;
use palettier_domain::paint::Paint;

#[derive(Clone, Copy)]
pub(super) struct PigmentsState {
    selected: Signal<DryPigmentEffect>,
}

impl PigmentsState {
    pub(super) fn new() -> Self {
        Self { selected: Signal::new(DryPigmentEffect::BurntUmberDust) }
    }
}

#[component]
pub(super) fn Pigments(candidates: Vec<Paint>) -> Element {
    let state = use_context::<AppState>();
    let services = use_context::<AppServices>();
    let selected_pigment = use_context::<PigmentsState>().selected;
    let locale = *state.locale.read();
    let pig = *selected_pigment.read();
    let sugg = best_mix(&services, rgb_or_grey(pig.dry_hex(), 0.5), &candidates);

    rsx! {
        div { class: "grid-2 u-gtc-1fr-1p35fr u-gap-16",
            // Left: Pigment Selector
            div { class: "card",
                h3 { class: "card-title", {locale.tr("🏜️ Poudres de Pigments de Weathering", "🏜️ Weathering Dry Pigments")} }
                p { class: "muted u-fs-12 u-mb-14",
                    {locale.tr("Les pigments purs permettent de recréer l'accumulation de poussière, terre calcinée et suie avec une matité absolue impossible à atteindre avec un liant.", "Dry pigments recreate dust, baked earth and engine soot with dead-flat velvety texture.")}
                }

                div { class: "u-d-flex u-fd-column u-gap-8",
                    for pig in DryPigmentEffect::ALL {
                        {pigment_option(selected_pigment, locale, pig)}
                    }
                }
            }

            // Right: Wet Shift Comparison & Active Palette Mix
            div { class: "card",
                div { class: "row-between u-mb-14",
                    div { class: "row u-gap-12 u-ai-center",
                        ColorSwatch { hex: pig.dry_hex().to_string(), size: 36 }
                        div {
                            h3 { class: "card-title u-m-0", "{pig.label(locale)}" }
                            span { class: "pill pill-accent", "Weathering Pigment Simulation" }
                        }
                    }
                }

                // Wet-shift visual comparison
                div { class: "card-subtle u-mb-14",
                    div { class: "u-fw-700 u-fs-12p5 u-mb-8",
                        {locale.tr("🔍 Altération de Teinte : Effet Sec vs Fixé au Solvant", "🔍 Color Shift: Dry Powder vs Fixed Tone")}
                    }
                    div { class: "grid-2 u-gap-10",
                        div { class: "card u-p-10 u-ta-center",
                            ColorSwatch { hex: pig.dry_hex().to_string(), size: 48 }
                            div { class: "u-fw-700 u-fs-12 u-mt-6",
                                {locale.tr("Poudre Sèche Pure", "Pure Dry Powder")}
                            }
                            span { class: "pill u-fs-10p5 u-mt-4", {locale.tr("Ultra-mat / velouté", "Ultra-matt / velvety")} }
                        }
                        div { class: "card u-p-10 u-ta-center",
                            ColorSwatch { hex: pig.fixed_hex().to_string(), size: 48 }
                            div { class: "u-fw-700 u-fs-12 u-mt-6",
                                {locale.tr("Fixé à l'Essence Sans Odeur", "Fixed with Odorless Spirits")}
                            }
                            span { class: "pill pill-warn u-fs-10p5 u-mt-4", "+2 crans plus sombre" }
                        }
                    }
                }

                // Live mix from chosen palette!
                div { class: "card-subtle u-mb-14 u-b-1-solid-accent-border",
                    div { class: "u-fw-700 u-fs-12 u-mb-6 u-c-accent",
                        {locale.tr("🧪 Reproduire ce ton de poussière en jus d'huile avec votre palette :", "🧪 Equivalent oil glaze mix from your palette:")}
                    }
                    if let Some(ref s) = sugg {
                        div { class: "row-between u-ai-center",
                            div { class: "row u-gap-10 u-ai-center",
                                ColorSwatch { hex: s.color.to_hex(), size: 30 }
                                div {
                                    div { class: "u-fw-700 u-fs-13", "{s.describe_i18n(locale)}" }
                                    div { class: "muted u-fs-11", "{s.verdict(locale)}" }
                                }
                            }
                            DeltaEBadge { delta_e: s.delta_e }
                        }
                    } else {
                        span { class: "muted u-fs-11p5", {locale.tr("Aucun tube disponible dans la palette sélectionnée", "No matching tubes available in selected palette")} }
                    }
                }

                // Pro Tip Card
                div { class: "card-subtle",
                    div { class: "u-fw-700 u-fs-12 u-mb-4 u-c-accent",
                        {locale.tr("🖌️ Technique d'application sans souffler la poudre :", "🖌️ Pro Application Technique:")}
                    }
                    div { class: "u-fs-12p5 u-lh-1p45",
                        "{pig.application_tip_i18n(locale)}"
                    }
                }
            }
        }
    }
}

fn pigment_option(mut selected_pigment: Signal<DryPigmentEffect>, locale: Locale, pig: DryPigmentEffect) -> Element {
    let is_sel = *selected_pigment.read() == pig;
    rsx! {
        div {
            class: if is_sel { "card-subtle active" } else { "card-subtle" },
            class: "u-cur-pointer u-p-10",
            onclick: move |_| selected_pigment.set(pig),
            div { class: "row-between u-ai-center",
                div { class: "row u-gap-10 u-ai-center",
                    ColorSwatch { hex: pig.dry_hex().to_string(), size: 28 }
                    strong { class: "u-fs-13", "{pig.label(locale)}" }
                }
                span { class: "mono u-fs-11", "{pig.dry_hex()}" }
            }
        }
    }
}
