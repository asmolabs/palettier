//! Palette layout: where to place each tube on the palette.

use crate::state::AppState;
use crate::theme::ColorSwatch;
use dioxus::prelude::*;
use palettier_domain::miniature::PaletteLayoutType;

#[derive(Clone, Copy)]
pub(super) struct PaletteLayoutState {
    selected: Signal<PaletteLayoutType>,
}

impl PaletteLayoutState {
    pub(super) fn new() -> Self {
        Self { selected: Signal::new(PaletteLayoutType::NormaBlueStudio) }
    }
}

#[component]
pub(super) fn PaletteLayout() -> Element {
    let state = use_context::<AppState>();
    let mut selected_palette_layout = use_context::<PaletteLayoutState>().selected;
    let locale = *state.locale.read();

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 16px;",
            div { class: "card",
                div { class: "row-between", style: "margin-bottom: 12px; align-items: center;",
                    h3 { class: "card-title", style: "margin: 0;",
                        "🗺️ ", {locale.tr("Guide d'Agencement de la Palette Physique d'Atelier", "Physical Studio Palette Layout Guide")}
                    }
                    span { class: "pill pill-accent",
                        {locale.tr("Prévention de la contamination des pigments", "Pigment Contamination Prevention")}
                    }
                }
                p { class: "muted", style: "margin-top: 0; font-size: 13px;",
                    {locale.tr(
                        "À l'huile, la disposition des noisettes de couleur sur la dalle de verre ou la palette étanche détermine la pureté de vos mélanges. Les pigments ultra-staining (Bleu de Prusse, Phtalo) doivent impérativement être isolés des blancs.",
                        "In miniature oils, paint dollop placement on your glass slab or stay-wet box governs mix purity. High-staining pigments (Prussian Blue, Phthalo) must be strictly isolated from whites."
                    )}
                }

                div { class: "row", style: "gap: 8px; margin-bottom: 16px; flex-wrap: wrap;",
                    for l in PaletteLayoutType::ALL {
                        button {
                            class: if *selected_palette_layout.read() == l { "btn-primary" } else { "btn" },
                            onclick: move |_| selected_palette_layout.set(l),
                            "{l.title_i18n(locale)}"
                        }
                    }
                }

                div { class: "card-subtle", style: "margin-bottom: 16px;",
                    div { class: "row-between", style: "margin-bottom: 8px;",
                        strong { style: "font-size: 14px; color: var(--accent);",
                            "🛡️ ", {selected_palette_layout.read().title_i18n(locale)}
                        }
                    }
                    div { style: "font-size: 12.5px; line-height: 1.5; margin-bottom: 8px;",
                        strong { {locale.tr("Support recommandé : ", "Recommended surface: ")} }
                        {selected_palette_layout.read().surface_recommendation_i18n(locale)}
                    }
                    div { style: "font-size: 12.5px; line-height: 1.5;",
                        strong { {locale.tr("Règle d'agencement d'atelier : ", "Studio placement rule: ")} }
                        {selected_palette_layout.read().setup_advice_i18n(locale)}
                    }
                }

                h4 { style: "font-size: 13px; margin: 12px 0 8px 0;",
                    {locale.tr("Disposition des Noisettes de Peinture (Pourtour) :", "Paint Dollop Perimeter Placement:")}
                }
                div { style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 10px;",
                    for slot in selected_palette_layout.read().slots(locale) {
                        div { class: "card-subtle", style: "padding: 10px 12px; display: flex; flex-direction: column; justify-content: space-between;",
                            div { class: "row", style: "gap: 10px; align-items: center; margin-bottom: 6px;",
                                ColorSwatch { hex: slot.hex.to_string(), size: 28 }
                                div {
                                    div { style: "font-weight: 700; font-size: 13px;", "{slot.name}" }
                                    div { class: "muted", style: "font-size: 11px;", "{slot.position_desc}" }
                                }
                            }
                            div { style: "font-size: 11.5px; color: var(--fg); opacity: 0.85; margin-bottom: 6px;",
                                "{slot.role}"
                            }
                            if let Some(ref w) = slot.safety_warning {
                                div { class: "badge badge-warning", style: "font-size: 10px; white-space: normal; padding: 4px 6px;",
                                    "{w}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
