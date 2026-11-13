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
        div { class: "u-d-flex u-fd-column u-gap-16",
            div { class: "card",
                div { class: "row-between u-mb-12 u-ai-center",
                    h3 { class: "card-title u-m-0",
                        "🗺️ ", {locale.tr("Guide d'Agencement de la Palette Physique d'Atelier", "Physical Studio Palette Layout Guide")}
                    }
                    span { class: "pill pill-accent",
                        {locale.tr("Prévention de la contamination des pigments", "Pigment Contamination Prevention")}
                    }
                }
                p { class: "muted u-mt-0 u-fs-13",
                    {locale.tr(
                        "À l'huile, la disposition des noisettes de couleur sur la dalle de verre ou la palette étanche détermine la pureté de vos mélanges. Les pigments ultra-staining (Bleu de Prusse, Phtalo) doivent impérativement être isolés des blancs.",
                        "In miniature oils, paint dollop placement on your glass slab or stay-wet box governs mix purity. High-staining pigments (Prussian Blue, Phthalo) must be strictly isolated from whites."
                    )}
                }

                div { class: "row u-gap-8 u-mb-16 u-fwrap-wrap",
                    for l in PaletteLayoutType::ALL {
                        button {
                            class: if *selected_palette_layout.read() == l { "btn-primary" } else { "btn" },
                            onclick: move |_| selected_palette_layout.set(l),
                            "{l.title_i18n(locale)}"
                        }
                    }
                }

                div { class: "card-subtle u-mb-16",
                    div { class: "row-between u-mb-8",
                        strong { class: "u-fs-14 u-c-accent",
                            "🛡️ ", {selected_palette_layout.read().title_i18n(locale)}
                        }
                    }
                    div { class: "u-fs-12p5 u-lh-1p5 u-mb-8",
                        strong { {locale.tr("Support recommandé : ", "Recommended surface: ")} }
                        {selected_palette_layout.read().surface_recommendation_i18n(locale)}
                    }
                    div { class: "u-fs-12p5 u-lh-1p5",
                        strong { {locale.tr("Règle d'agencement d'atelier : ", "Studio placement rule: ")} }
                        {selected_palette_layout.read().setup_advice_i18n(locale)}
                    }
                }

                h4 { class: "u-fs-13 u-m-12-0-8-0",
                    {locale.tr("Disposition des Noisettes de Peinture (Pourtour) :", "Paint Dollop Perimeter Placement:")}
                }
                div { class: "u-d-grid u-gtc-repeat-auto-fill-minmax-280-1fr u-gap-10",
                    for slot in selected_palette_layout.read().slots(locale) {
                        div { class: "card-subtle u-p-10-12 u-d-flex u-fd-column u-jc-space-between",
                            div { class: "row u-gap-10 u-ai-center u-mb-6",
                                ColorSwatch { hex: slot.hex.to_string(), size: 28 }
                                div {
                                    div { class: "u-fw-700 u-fs-13", "{slot.name}" }
                                    div { class: "muted u-fs-11", "{slot.position_desc}" }
                                }
                            }
                            div { class: "u-fs-11p5 u-c-fg u-op-0p85 u-mb-6",
                                "{slot.role}"
                            }
                            if let Some(ref w) = slot.safety_warning {
                                div { class: "badge badge-warning u-fs-10 u-ws-normal u-p-4-6",
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
