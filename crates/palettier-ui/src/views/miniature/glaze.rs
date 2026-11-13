//! Glaze over zenithal: what a transparent glaze gives over each value of the undercoat.

use super::shared::{best_mix, paint_key, pick};
use crate::state::{AppServices, AppState};
use crate::theme::{ColorSwatch, DeltaEBadge};
use dioxus::prelude::*;
use palettier_domain::color::Rgb;
use palettier_domain::i18n::Locale;
use palettier_domain::miniature::OilGlazeZenithalSimulator;
use palettier_domain::paint::Paint;

#[derive(Clone, Copy)]
pub(super) struct GlazeState {
    key: Signal<Option<String>>,
    strength: Signal<f64>,
}

impl GlazeState {
    pub(super) fn new() -> Self {
        Self { key: Signal::new(None), strength: Signal::new(0.70) }
    }
}

#[component]
pub(super) fn GlazePreview(candidates: Vec<Paint>) -> Element {
    let state = use_context::<AppState>();
    let services = use_context::<AppServices>();
    let GlazeState { key: mut glaze_key, strength: mut glaze_strength } = use_context::<GlazeState>();
    let locale = *state.locale.read();
    let glaze_paint = pick(&candidates, glaze_key.read().as_deref());
    let glaze_rgb = glaze_paint.as_ref().map(|p| p.color()).unwrap_or_else(|| Rgb::new(0.5, 0.5, 0.5));
    let strength = *glaze_strength.read();

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 16px;",
            div { class: "card",
                div { class: "row-between", style: "align-items: center; margin-bottom: 12px;",
                    div {
                        h3 { class: "card-title", style: "margin: 0;",
                            {locale.tr("🖼️ Simulation Optique de Glacis d'Huile sur Sous-Couche Zénithale", "🖼️ Optical Oil Glaze on Zenithal Primer Simulator")}
                        }
                        p { class: "muted", style: "font-size: 12px; margin: 2px 0 0 0;",
                            {locale.tr("Visualisez la physique du glacis d'huile : la lumière traverse le pigment transparent, rebondit sur le blanc spéculaire zénithal et s'éteint dans les ombres physiques.", "Simulate physical oil glazes: light transmits through transparent pigments, reflects off zenithal white, and extinguishes in deep recesses.")}
                        }
                    }
                }

                div { class: "grid-2", style: "gap: 16px; margin-bottom: 16px;",
                    div {
                        label { style: "font-size: 12px; font-weight: 600; display: block; margin-bottom: 6px;",
                            {locale.tr("Teinte d'huile à vitrifier :", "Glaze Oil Pigment Tone:")}
                        }
                        div { class: "row", style: "gap: 8px; align-items: center;",
                            ColorSwatch { hex: glaze_paint.as_ref().map(|p| p.hex_color.clone()).unwrap_or_default(), size: 28 }
                            select {
                                class: "input",
                                style: "flex: 1;",
                                value: "{glaze_paint.as_ref().map(paint_key).unwrap_or_default()}",
                                onchange: move |ev| glaze_key.set(Some(ev.value())),
                                for p in candidates.iter() {
                                    option { value: "{paint_key(p)}", "{p.name} ({p.brand})" }
                                }
                            }
                        }
                    }

                    div {
                        div { class: "row-between", style: "margin-bottom: 6px;",
                            label { style: "font-size: 12px; font-weight: 600;", {locale.tr("Intensité / Saturation du glacis :", "Glaze Saturation Strength:")} }
                            span { class: "pill pill-accent", "{glaze_strength * 100.0:.0}%" }
                        }
                        input {
                            r#type: "range",
                            min: "0.20",
                            max: "0.95",
                            step: "0.05",
                            value: "{glaze_strength}",
                            oninput: move |ev| {
                                if let Ok(v) = ev.value().parse::<f64>() {
                                    glaze_strength.set(v);
                                }
                            },
                            style: "width: 100%;",
                        }
                    }
                }
            }

            // 5-step Zenithal Glaze Visualizer
            div { class: "card",
                h3 { class: "card-title", {locale.tr("Échelle Optique Zénithale Glacée", "Optical Zenithal Glazed Ladder")} }
                div { style: "display: flex; flex-direction: column; gap: 8px;",
                    for (name_fr, name_en, lum) in OilGlazeZenithalSimulator::ZENITHAL_LEVELS {
                        {
                            let step_name = if locale.is_fr() { name_fr } else { name_en };
                            let glazed_rgb = OilGlazeZenithalSimulator::simulate_glaze(glaze_rgb, lum, strength);
                            glaze_row(&services, locale, step_name, lum, glazed_rgb, &candidates)
                        }
                    }
                }
            }
        }
    }
}

/// The grey of an undercoat of value `lum` (0 = black, 1 = white).
fn grey_hex(lum: f64) -> String {
    let g = (lum * 255.0).round() as u8;
    format!("#{g:02X}{g:02X}{g:02X}")
}

fn glaze_row(
    services: &AppServices,
    locale: Locale,
    step_name: &str,
    lum: f64,
    glazed_rgb: Rgb,
    candidates: &[Paint],
) -> Element {
    let undercoat_hex = grey_hex(lum);
    let glazed_hex = glazed_rgb.to_hex();
    let sugg = best_mix(services, glazed_rgb, candidates);
    rsx! {
        div { class: "card-subtle", style: "padding: 10px 14px;",
            div { class: "row-between", style: "align-items: center;",
                div { class: "row", style: "gap: 12px; align-items: center;",
                    // Left: Primer value
                    div { style: "text-align: center;",
                        ColorSwatch { hex: undercoat_hex.clone(), size: 28 }
                        div { class: "mono", style: "font-size: 9.5px; margin-top: 2px;", "{undercoat_hex}" }
                    }
                    span { style: "font-size: 16px; color: var(--accent);", "➔" }
                    // Right: Glazed result
                    div { style: "text-align: center;",
                        ColorSwatch { hex: glazed_hex.clone(), size: 36 }
                        div { class: "mono", style: "font-size: 10px; font-weight: 700; margin-top: 2px;", "{glazed_hex}" }
                    }
                    div {
                        strong { style: "font-size: 13px;", "{step_name}" }
                        div { class: "muted", style: "font-size: 11px;",
                            {locale.tr("Luminance physique sous-jacente : ", "Physical undercoat luminance: ")},
                            "{lum * 100.0:.0}%"
                        }
                    }
                }

                // Palette mix match
                if let Some(ref s) = sugg {
                    div { class: "row", style: "gap: 8px; align-items: center;",
                        div { style: "text-align: right;",
                            div { style: "font-weight: 650; font-size: 12px;", "{s.describe_i18n(locale)}" }
                        }
                        DeltaEBadge { delta_e: s.delta_e }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_undercoat_is_a_grey_of_its_value() {
        assert_eq!("#000000", grey_hex(0.0));
        assert_eq!("#808080", grey_hex(0.5));
        assert_eq!("#FFFFFF", grey_hex(1.0));
    }
}
