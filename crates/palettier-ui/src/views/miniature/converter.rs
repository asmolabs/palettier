//! Acrylic → oil converter: a reference acrylic, its 5-tone ladder and the oil mix of
//! each tone.

use super::shared::{acrylic_catalog, best_mix, open_in_mixer, paint_key, pick, rgb_or_grey};
use crate::state::{AppServices, AppState};
use crate::theme::{ColorSwatch, DeltaEBadge};
use dioxus::prelude::*;
use palettier_domain::i18n::Locale;
use palettier_domain::miniature::MiniatureLadder;
use palettier_domain::paint::Paint;

/// The brand chip, the search and the chosen acrylic, kept while switching tools.
#[derive(Clone, Copy)]
pub(super) struct ConverterState {
    brand: Signal<Option<String>>,
    search: Signal<String>,
    selected: Signal<Option<String>>,
}

impl ConverterState {
    pub(super) fn new() -> Self {
        Self {
            brand: Signal::new(None),
            search: Signal::new(String::new()),
            selected: Signal::new(None),
        }
    }
}

/// The brands of `paints`, sorted and without duplicates.
fn brands(paints: &[Paint]) -> Vec<String> {
    paints
        .iter()
        .map(|p| p.brand.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
}

/// The paints of `brand` (all when `None`) whose name, brand or code contains `search`.
fn filter_acrylics(paints: &[Paint], brand: Option<&str>, search: &str) -> Vec<Paint> {
    let term = search.trim().to_lowercase();
    paints
        .iter()
        .filter(|p| brand.is_none_or(|b| p.brand == b))
        .filter(|p| {
            term.is_empty()
                || p.name.to_lowercase().contains(&term)
                || p.brand.to_lowercase().contains(&term)
                || p.code.to_lowercase().contains(&term)
        })
        .cloned()
        .collect()
}

/// Name of the ladder step `idx`.
fn step_label(locale: Locale, idx: usize) -> &'static str {
    match idx {
        0 => locale.tr("1. Ombre profonde (Occlusion / Pin wash)", "1. Deep Occlusion (Pin wash)"),
        1 => locale.tr("2. Ombre de forme (Wet blending)", "2. Form Shadow (Wet blending)"),
        2 => locale.tr("3. Couleur de base (Couverture)", "3. Basecoat (Midtone)"),
        3 => locale.tr("4. Éclaircissement volumétrique (Zénithal)", "4. Volumetric Highlight (Zenithal)"),
        4 => locale.tr("5. Rehaut d'arête / Spéculaire", "5. Edge Highlight / Specular"),
        _ => "",
    }
}

#[component]
pub(super) fn Converter(oil_candidates: Vec<Paint>, palette_name: String) -> Element {
    let state = use_context::<AppState>();
    let services = use_context::<AppServices>();
    let tool = use_context::<ConverterState>();
    let locale = *state.locale.read();
    let acrylics = acrylic_catalog(&state.paints.read());
    let current = pick(&acrylics, tool.selected.read().as_deref());

    rsx! {
        div { class: "grid-2", style: "grid-template-columns: 1fr 1.35fr;",
            // Left: Acrylic Selector & Search
            {acrylic_picker(tool, locale, &acrylics, current.as_ref())}

            // Right: 5-Stage Ladder with Calculated Mixes from chosen palette
            {ladder_card(state, &services, locale, current.as_ref(), &oil_candidates, &palette_name)}
        }
    }
}

fn acrylic_picker(tool: ConverterState, locale: Locale, acrylics: &[Paint], current: Option<&Paint>) -> Element {
    let ConverterState { mut brand, mut search, .. } = tool;
    let filtered = filter_acrylics(acrylics, brand.read().as_deref(), &search.read());
    rsx! {
        div { class: "card",
            div { class: "row-between", style: "margin-bottom: 10px;",
                h3 { class: "card-title", style: "margin: 0;",
                    {locale.tr("Acryliques Figurines de Référence", "Miniature Acrylic Reference Paints")}
                }
                span { class: "pill", "{filtered.len()} ", {locale.tr("teintes", "colours")} }
            }

            // Brand Filter chips
            div { class: "row", style: "gap: 6px; margin-bottom: 10px; flex-wrap: wrap;",
                button {
                    class: if brand.read().is_none() { "pill pill-accent" } else { "pill" },
                    onclick: move |_| brand.set(None),
                    {locale.tr("Toutes marques", "All brands")}
                }
                for b in brands(acrylics) {
                    {
                        let is_sel = brand.read().as_deref() == Some(b.as_str());
                        let label = b.clone();
                        rsx! {
                            button {
                                class: if is_sel { "pill pill-accent" } else { "pill" },
                                onclick: move |_| brand.set(Some(b.clone())),
                                "{label}"
                            }
                        }
                    }
                }
            }

            // Search bar
            input {
                r#type: "text",
                style: "width: 100%; margin-bottom: 12px;",
                placeholder: locale.tr("Rechercher (ex: Mephiston, Blue, Nurgle, Flesh)…", "Search (e.g. Mephiston, Blue, Nurgle, Flesh)…"),
                value: "{search}",
                oninput: move |ev| search.set(ev.value()),
            }

            // Scrollable list
            div { style: "max-height: 480px; overflow-y: auto; display: flex; flex-direction: column; gap: 4px;",
                for acr in filtered {
                    {acrylic_row(tool, acr, current)}
                }
            }
        }
    }
}

fn acrylic_row(tool: ConverterState, acr: Paint, current: Option<&Paint>) -> Element {
    let mut selected = tool.selected;
    let k = paint_key(&acr);
    let is_active = current.is_some_and(|c| paint_key(c) == k);
    rsx! {
        div {
            class: if is_active { "card-subtle active" } else { "card-subtle" },
            style: if is_active { "border-color: var(--accent); cursor: pointer; padding: 6px 10px;" } else { "cursor: pointer; padding: 6px 10px;" },
            onclick: move |_| selected.set(Some(k.clone())),
            div { class: "row-between", style: "align-items: center;",
                div { class: "row", style: "gap: 10px; align-items: center;",
                    ColorSwatch { hex: acr.hex_color.clone(), size: 24 }
                    div {
                        div { style: "font-weight: 650; font-size: 13px;", "{acr.name}" }
                        div { class: "muted", style: "font-size: 11px;", "{acr.brand}" }
                    }
                }
                if !acr.code.is_empty() {
                    span { class: "pill mono", style: "font-size: 10.5px;", "{acr.code}" }
                }
            }
        }
    }
}

fn ladder_card(
    state: AppState,
    services: &AppServices,
    locale: Locale,
    current: Option<&Paint>,
    oil_candidates: &[Paint],
    palette_name: &str,
) -> Element {
    let ladder = current.map(|p| MiniatureLadder::of_basecoat(p.color()));
    let name = current.map(|p| p.name.clone()).unwrap_or_default();
    let hex = current.map(|p| p.hex_color.clone()).unwrap_or_default();
    let steps = ladder.as_ref().map(|l| l.steps_hex().to_vec()).unwrap_or_default();
    rsx! {
        div { class: "card",
            div { class: "row-between", style: "margin-bottom: 12px;",
                div {
                    div { class: "card-title", style: "margin: 0;",
                        {locale.tr("Échelle 5 Étapes & Recettes Calculées", "5-Stage Miniature Ladder & Calculated Mixes")}
                    }
                    div { class: "muted", style: "font-size: 12px; margin-top: 2px;",
                        {locale.tr("Calculé pour ", "Calculated for ")}
                        strong { "{name}" }
                        " ({hex}) "
                        {locale.tr("avec ", "with ")}
                        span { class: "pill pill-accent", style: "padding: 1px 6px; font-size: 11px;", "{palette_name}" }
                    }
                }

                button {
                    class: "btn",
                    onclick: {
                        let h = hex.clone();
                        move |_| open_in_mixer(state, h.clone())
                    },
                    "🧪 ",
                    {locale.tr("Tester dans le Mélangeur", "Send to Mixer")}
                }
            }

            // 5-Stage Computed Rows
            div { style: "display: flex; flex-direction: column; gap: 8px;",
                for (idx, step_hex) in steps.iter().enumerate() {
                    {ladder_row(state, services, locale, idx, step_hex, oil_candidates)}
                }
            }
        }
    }
}

fn ladder_row(
    state: AppState,
    services: &AppServices,
    locale: Locale,
    idx: usize,
    step_hex: &str,
    oil_candidates: &[Paint],
) -> Element {
    let short_label = step_label(locale, idx);
    let sugg = best_mix(services, rgb_or_grey(step_hex, 0.5), oil_candidates);
    rsx! {
        div { class: "card-subtle", style: "padding: 10px 12px;",
            div { class: "row-between", style: "align-items: center;",
                div { class: "row", style: "gap: 10px; align-items: center;",
                    ColorSwatch { hex: step_hex.to_string(), size: 30 }
                    div {
                        div { style: "font-weight: 700; font-size: 12.5px;", "{short_label}" }
                        div { class: "mono", style: "font-size: 10.5px; color: var(--text-dim);", "{step_hex}" }
                    }
                }

                if let Some(ref s) = sugg {
                    div { class: "row", style: "gap: 10px; align-items: center;",
                        ColorSwatch { hex: s.color.to_hex(), size: 30 }
                        div { style: "text-align: right; line-height: 1.3;",
                            div { style: "font-weight: 650; font-size: 12px; color: var(--accent);", "{s.describe_i18n(locale)}" }
                            div { style: "display: flex; gap: 4px; justify-content: flex-end; align-items: center; margin-top: 2px;",
                                DeltaEBadge { delta_e: s.delta_e }
                            }
                        }
                        button {
                            class: "btn",
                            style: "padding: 4px 8px; font-size: 11px;",
                            title: locale.tr("Ouvrir cette teinte dans le Mélangeur", "Open target in Mixer"),
                            onclick: {
                                let h = step_hex.to_string();
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

#[cfg(test)]
mod tests {
    use super::*;
    use palettier_domain::paint::PaintSystem;

    fn pot(brand: &str, name: &str, code: &str) -> Paint {
        let mut p = Paint::new(brand, name, "#808080").in_system(PaintSystem::Acrylic);
        p.code = code.to_string();
        p
    }

    #[test]
    fn acrylics_filter_by_brand_and_by_name_brand_or_code() {
        let pots = vec![
            pot("Citadel", "Mephiston Red", "21-03"),
            pot("Citadel", "Macragge Blue", "21-08"),
            pot("Vallejo", "Flat Red", "70.957"),
        ];
        let names = |v: Vec<Paint>| v.into_iter().map(|p| p.name).collect::<Vec<_>>();
        assert_eq!(3, filter_acrylics(&pots, None, "  ").len());
        assert_eq!(vec!["Mephiston Red", "Macragge Blue"], names(filter_acrylics(&pots, Some("Citadel"), "")));
        assert_eq!(vec!["Mephiston Red", "Flat Red"], names(filter_acrylics(&pots, None, " RED ")));
        assert_eq!(vec!["Flat Red"], names(filter_acrylics(&pots, None, "70.9")));
        assert_eq!(vec!["Macragge Blue"], names(filter_acrylics(&pots, Some("Citadel"), "blue")));
        assert!(filter_acrylics(&pots, Some("Vallejo"), "blue").is_empty());
        assert_eq!(vec!["Citadel", "Vallejo"], brands(&pots));
    }
}
