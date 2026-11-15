//! Colour harmonies computed from a base tube of the palette.

use super::shared::{best_mix, open_in_mixer, paint_key, pick, rgb_or_grey};
use crate::state::{AppServices, AppState};
use crate::theme::{ColorSwatch, DeltaEBadge};
use dioxus::prelude::*;
use palettier_domain::color::Rgb;
use palettier_domain::i18n::Locale;
use palettier_domain::miniature::{OilHarmoniesCalculator, OilHarmonyItem};
use palettier_domain::paint::Paint;

#[derive(Clone, Copy)]
pub(super) struct HarmoniesState {
    key: Signal<Option<String>>,
}

impl HarmoniesState {
    pub(super) fn new() -> Self {
        Self { key: Signal::new(None) }
    }
}

#[component]
pub(super) fn OilHarmonies(candidates: Vec<Paint>) -> Element {
    let state = use_context::<AppState>();
    let services = use_context::<AppServices>();
    let mut harmony_key = use_context::<HarmoniesState>().key;
    let locale = *state.locale.read();
    let harmony_paint = pick(&candidates, harmony_key.read().as_deref());
    let base_rgb = harmony_paint.as_ref().map(|p| p.color()).unwrap_or_else(|| Rgb::new(0.5, 0.5, 0.5));
    let items = OilHarmoniesCalculator::calculate(base_rgb);

    rsx! {
        div { class: "u-d-flex u-fd-column u-gap-16",
            div { class: "card",
                div { class: "row-between u-mb-12 u-ai-center",
                    div {
                        h3 { class: "card-title u-m-0",
                            {locale.tr("🎡 Roue Chromatique & Harmonies d'Ombrage à l'Huile", "🎡 Color Harmonies & Oil Shadow Wheel")}
                        }
                        p { class: "muted u-fs-12 u-m-2-0-0-0",
                            {locale.tr("Contrairement aux aplats acryliques, l'huile permet d'éteindre une teinte avec sa complémentaire translucide (Ombre d'Extinction) plutôt qu'avec du noir qui salit.", "Unlike flat acrylics, oils allow desaturating via a translucent complementary filter (Extinction Shadow) instead of muddying tube black.")}
                        }
                    }
                }

                div { class: "row u-gap-12 u-ai-center u-mb-12 u-fwrap-wrap",
                    strong { class: "u-fs-13", {locale.tr("Teinte maîtresse de départ :", "Base Color:")} }
                    div { class: "row u-gap-8 u-ai-center",
                        ColorSwatch { hex: harmony_paint.as_ref().map(|p| p.hex_color.clone()).unwrap_or_default(), size: 28 }
                        select {
                            class: "input",
                            value: "{harmony_paint.as_ref().map(paint_key).unwrap_or_default()}",
                            onchange: move |ev| harmony_key.set(Some(ev.value())),
                            for p in candidates.iter() {
                                option { value: "{paint_key(p)}", "{p.name} ({p.brand})" }
                            }
                        }
                    }
                }
            }

            // Grid of 8 calculated harmonies
            div { class: "grid-2 u-gap-14",
                for item in items {
                    {harmony_card(state, &services, locale, item, &candidates)}
                }
            }
        }
    }
}

fn harmony_card(
    state: AppState,
    services: &AppServices,
    locale: Locale,
    item: OilHarmonyItem,
    candidates: &[Paint],
) -> Element {
    let sugg = best_mix(services, rgb_or_grey(&item.hex, 0.5), candidates);
    let h = item.hex.clone();
    rsx! {
        div { class: "card u-p-14",
            div { class: "row-between u-ai-center u-mb-8",
                div { class: "row u-gap-10 u-ai-center",
                    ColorSwatch { hex: item.hex.clone(), size: 32 }
                    div {
                        strong { class: "u-fs-13p5", "{item.name_i18n(locale)}" }
                        span { class: "mono u-fs-11 u-ml-6 u-c-text-dim", "{item.hex}" }
                    }
                }
                button {
                    class: "btn",
                    class: "u-p-3-8 u-fs-11",
                    onclick: move |_| open_in_mixer(state, h.clone()),
                    "🧪"
                }
            }

            div { class: "card-subtle u-p-8-10 u-mb-8 u-fs-12",
                strong { "🎯 " },
                span { "{item.role_i18n(locale)}" }
            }

            div { class: "u-fs-11p5 u-lh-1p4 u-c-accent u-mb-10",
                "🖌️ {item.oil_tip_i18n(locale)}"
            }

            // Calculated recipe from active palette
            div { class: "card-subtle u-b-1-solid-accent-border u-p-8-10",
                div { class: "u-fw-700 u-fs-11 u-mb-4 u-c-text-dim",
                    {locale.tr("Recette calculée avec votre palette :", "Calculated Palette Mix:")}
                }
                if let Some(ref s) = sugg {
                    div { class: "row-between u-ai-center",
                        div { class: "row u-gap-8 u-ai-center",
                            ColorSwatch { hex: s.color.to_hex(), size: 24 }
                            div { class: "u-fw-650 u-fs-12", "{s.describe_i18n(locale)}" }
                        }
                        DeltaEBadge { delta_e: s.delta_e }
                    }
                } else {
                    span { class: "muted u-fs-11", {locale.tr("Aucun tube disponible", "No matching tubes in palette")} }
                }
            }
        }
    }
}
