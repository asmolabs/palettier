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
        div { style: "display: flex; flex-direction: column; gap: 16px;",
            div { class: "card",
                div { class: "row-between", style: "margin-bottom: 12px; align-items: center;",
                    div {
                        h3 { class: "card-title", style: "margin: 0;",
                            {locale.tr("🎡 Roue Chromatique & Harmonies d'Ombrage à l'Huile", "🎡 Color Harmonies & Oil Shadow Wheel")}
                        }
                        p { class: "muted", style: "font-size: 12px; margin: 2px 0 0 0;",
                            {locale.tr("Contrairement aux aplats acryliques, l'huile permet d'éteindre une teinte avec sa complémentaire translucide (Ombre d'Extinction) plutôt qu'avec du noir qui salit.", "Unlike flat acrylics, oils allow desaturating via a translucent complementary filter (Extinction Shadow) instead of muddying tube black.")}
                        }
                    }
                }

                div { class: "row", style: "gap: 12px; align-items: center; margin-bottom: 12px; flex-wrap: wrap;",
                    strong { style: "font-size: 13px;", {locale.tr("Teinte maîtresse de départ :", "Base Color:")} }
                    div { class: "row", style: "gap: 8px; align-items: center;",
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
            div { class: "grid-2", style: "gap: 14px;",
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
        div { class: "card", style: "padding: 14px;",
            div { class: "row-between", style: "align-items: center; margin-bottom: 8px;",
                div { class: "row", style: "gap: 10px; align-items: center;",
                    ColorSwatch { hex: item.hex.clone(), size: 32 }
                    div {
                        strong { style: "font-size: 13.5px;", "{item.name_i18n(locale)}" }
                        span { class: "mono", style: "font-size: 11px; margin-left: 6px; color: var(--text-dim);", "{item.hex}" }
                    }
                }
                button {
                    class: "btn",
                    style: "padding: 3px 8px; font-size: 11px;",
                    onclick: move |_| open_in_mixer(state, h.clone()),
                    "🧪"
                }
            }

            div { class: "card-subtle", style: "padding: 8px 10px; margin-bottom: 8px; font-size: 12px;",
                strong { "🎯 " },
                span { "{item.role_i18n(locale)}" }
            }

            div { style: "font-size: 11.5px; line-height: 1.4; color: var(--accent); margin-bottom: 10px;",
                "🖌️ {item.oil_tip_i18n(locale)}"
            }

            // Calculated recipe from active palette
            div { class: "card-subtle", style: "border: 1px solid var(--accent-border); padding: 8px 10px;",
                div { style: "font-weight: 700; font-size: 11px; margin-bottom: 4px; color: var(--text-dim);",
                    {locale.tr("Recette calculée avec votre palette :", "Calculated Palette Mix:")}
                }
                if let Some(ref s) = sugg {
                    div { class: "row-between", style: "align-items: center;",
                        div { class: "row", style: "gap: 8px; align-items: center;",
                            ColorSwatch { hex: s.color.to_hex(), size: 24 }
                            div { style: "font-weight: 650; font-size: 12px;", "{s.describe_i18n(locale)}" }
                        }
                        DeltaEBadge { delta_e: s.delta_e }
                    }
                } else {
                    span { class: "muted", style: "font-size: 11px;", {locale.tr("Aucun tube disponible", "No matching tubes in palette")} }
                }
            }
        }
    }
}
