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
        div { class: "grid-2", style: "grid-template-columns: 1fr 1.35fr; gap: 16px;",
            // Left: Pigment Selector
            div { class: "card",
                h3 { class: "card-title", {locale.tr("🏜️ Poudres de Pigments de Weathering", "🏜️ Weathering Dry Pigments")} }
                p { class: "muted", style: "font-size: 12px; margin-bottom: 14px;",
                    {locale.tr("Les pigments purs permettent de recréer l'accumulation de poussière, terre calcinée et suie avec une matité absolue impossible à atteindre avec un liant.", "Dry pigments recreate dust, baked earth and engine soot with dead-flat velvety texture.")}
                }

                div { style: "display: flex; flex-direction: column; gap: 8px;",
                    for pig in DryPigmentEffect::ALL {
                        {pigment_option(selected_pigment, locale, pig)}
                    }
                }
            }

            // Right: Wet Shift Comparison & Active Palette Mix
            div { class: "card",
                div { class: "row-between", style: "margin-bottom: 14px;",
                    div { class: "row", style: "gap: 12px; align-items: center;",
                        ColorSwatch { hex: pig.dry_hex().to_string(), size: 36 }
                        div {
                            h3 { class: "card-title", style: "margin: 0;", "{pig.label_i18n(locale)}" }
                            span { class: "pill pill-accent", "Weathering Pigment Simulation" }
                        }
                    }
                }

                // Wet-shift visual comparison
                div { class: "card-subtle", style: "margin-bottom: 14px;",
                    div { style: "font-weight: 700; font-size: 12.5px; margin-bottom: 8px;",
                        {locale.tr("🔍 Altération de Teinte : Effet Sec vs Fixé au Solvant", "🔍 Color Shift: Dry Powder vs Fixed Tone")}
                    }
                    div { class: "grid-2", style: "gap: 10px;",
                        div { class: "card", style: "padding: 10px; text-align: center;",
                            ColorSwatch { hex: pig.dry_hex().to_string(), size: 48 }
                            div { style: "font-weight: 700; font-size: 12px; margin-top: 6px;",
                                {locale.tr("Poudre Sèche Pure", "Pure Dry Powder")}
                            }
                            span { class: "pill", style: "font-size: 10.5px; margin-top: 4px;", "Ultra-mat / Velouté" }
                        }
                        div { class: "card", style: "padding: 10px; text-align: center;",
                            ColorSwatch { hex: pig.fixed_hex().to_string(), size: 48 }
                            div { style: "font-weight: 700; font-size: 12px; margin-top: 6px;",
                                {locale.tr("Fixé à l'Essence Sans Odeur", "Fixed with Odorless Spirits")}
                            }
                            span { class: "pill pill-warn", style: "font-size: 10.5px; margin-top: 4px;", "+2 crans plus sombre" }
                        }
                    }
                }

                // Live mix from chosen palette!
                div { class: "card-subtle", style: "margin-bottom: 14px; border: 1px solid var(--accent-border);",
                    div { style: "font-weight: 700; font-size: 12px; margin-bottom: 6px; color: var(--accent);",
                        {locale.tr("🧪 Reproduire ce ton de poussière en jus d'huile avec votre palette :", "🧪 Equivalent oil glaze mix from your palette:")}
                    }
                    if let Some(ref s) = sugg {
                        div { class: "row-between", style: "align-items: center;",
                            div { class: "row", style: "gap: 10px; align-items: center;",
                                ColorSwatch { hex: s.color.to_hex(), size: 30 }
                                div {
                                    div { style: "font-weight: 700; font-size: 13px;", "{s.describe_i18n(locale)}" }
                                    div { class: "muted", style: "font-size: 11px;", "{s.verdict_i18n(locale)}" }
                                }
                            }
                            DeltaEBadge { delta_e: s.delta_e }
                        }
                    } else {
                        span { class: "muted", style: "font-size: 11.5px;", {locale.tr("Aucun tube disponible dans la palette sélectionnée", "No matching tubes available in selected palette")} }
                    }
                }

                // Pro Tip Card
                div { class: "card-subtle",
                    div { style: "font-weight: 700; font-size: 12px; margin-bottom: 4px; color: var(--accent);",
                        {locale.tr("🖌️ Technique d'application sans souffler la poudre :", "🖌️ Pro Application Technique:")}
                    }
                    div { style: "font-size: 12.5px; line-height: 1.45;",
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
            style: "cursor: pointer; padding: 10px;",
            onclick: move |_| selected_pigment.set(pig),
            div { class: "row-between", style: "align-items: center;",
                div { class: "row", style: "gap: 10px; align-items: center;",
                    ColorSwatch { hex: pig.dry_hex().to_string(), size: 28 }
                    strong { style: "font-size: 13px;", "{pig.label_i18n(locale)}" }
                }
                span { class: "mono", style: "font-size: 11px;", "{pig.dry_hex()}" }
            }
        }
    }
}
