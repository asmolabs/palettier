//! The mixer's right column: the inverse solver and the gamut map.

use crate::actions::mixer;
use crate::state::{AppServices, AppState};
use crate::theme::{ColorSwatch, DeltaEBadge, GamutMapSvg, HexColorInput};
use dioxus::prelude::*;
use palettier_domain::mix::{MixSuggestion, PaintPart};
use palettier_domain::paint::Paint;
use palettier_domain::palette::Palette;

/// « Comment obtenir cette couleur ? »: the target, the scope, and the recipes found.
#[component]
pub fn MixerSolverCard(
    palettes: Vec<Palette>,
    all_paints: Vec<Paint>,
    target_hex: String,
    scope_idx: Signal<usize>,
    max_paints: Signal<usize>,
    suggestions: Signal<Vec<MixSuggestion>>,
    manual_parts: Signal<Vec<PaintPart>>,
) -> Element {
    let mut state = use_context::<AppState>();
    let services = use_context::<AppServices>();
    let locale = *state.locale.read();
    let (in_stock, catalog) = (mixer::SCOPE_IN_STOCK, mixer::SCOPE_CATALOG);

    rsx! {
        div { class: "card",
            h3 { class: "card-title",
                {locale.tr("Comment obtenir cette couleur ? (Solveur inverse)", "How to Mix This Color? (Inverse Solver)")}
            }
            div { class: "row-between", style: "margin-bottom: 12px;",
                div { class: "row",
                    div {
                        label { class: "field-label", {locale.tr("Couleur visée", "Target Color")} }
                        HexColorInput {
                            value: target_hex.clone(),
                            onchange: move |v: String| state.mixer_target_hex.set(v),
                        }
                    }
                    div {
                        label { class: "field-label", {locale.tr("Chercher dans", "Search Scope")} }
                        select {
                            onchange: move |ev| {
                                if let Ok(v) = ev.value().parse::<usize>() {
                                    scope_idx.set(v);
                                }
                            },
                            for (idx, pal) in palettes.iter().enumerate() {
                                option { value: "{idx}", "Palette: {pal.name} ({pal.paints.len()} tubes)" }
                            }
                            option { value: "{in_stock}", {locale.tr("Mes tubes en stock", "My in-stock tubes")} }
                            option { value: "{catalog}", {locale.tr("Tout le catalogue", "Full catalog")}, " ({all_paints.len()} tubes)" }
                        }
                    }
                    div {
                        label { class: "field-label", {locale.tr("Tubes max", "Max Tubes")} }
                        select {
                            value: "{max_paints}",
                            onchange: move |ev| {
                                if let Ok(v) = ev.value().parse::<usize>() {
                                    max_paints.set(v);
                                }
                            },
                            option { value: "1", {locale.tr("1 tube seul", "1 tube only")} }
                            option { value: "2", {locale.tr("2 tubes au plus", "Up to 2 tubes")} }
                            option { value: "3", {locale.tr("3 tubes au plus", "Up to 3 tubes")} }
                            option { value: "4", {locale.tr("4 tubes au plus", "Up to 4 tubes")} }
                            option { value: "5", {locale.tr("5 tubes au plus", "Up to 5 tubes")} }
                        }
                    }
                }
                button {
                    class: "btn-primary",
                    onclick: {
                        let pals = palettes.clone();
                        let pts = all_paints.clone();
                        move |_| {
                            let pool = mixer::search_pool(&pals, &pts, scope_idx());
                            let target = state.mixer_target_hex.read().clone();
                            if let Some(found) = mixer::suggest_recipes(&services, &target, &pool, max_paints()) {
                                suggestions.set(found);
                            }
                        }
                    },
                    {locale.tr("Proposer des recettes", "Suggest recipes")}
                }
            }

            if suggestions().is_empty() {
                p { class: "muted",
                    {locale.tr("Choisissez une teinte cible et cliquez sur « Proposer des recettes » pour calculer les meilleurs mélanges en parts entières.", "Choose a target color and click 'Suggest recipes' to compute the best whole-part mixes.")}
                }
            } else {
                table { class: "data-table",
                    thead {
                        tr {
                            th { {locale.tr("Cible / Obtenu", "Target / Achieved")} }
                            th { {locale.tr("Écart", "Gap")} }
                            th { {locale.tr("Dosage proposé", "Suggested Ratio")} }
                            th { "" }
                        }
                    }
                    tbody {
                        for sug in suggestions() {
                            SuggestionRow { sug, target_hex: target_hex.clone(), manual_parts }
                        }
                    }
                }
            }
        }
    }
}

/// One recipe found by the solver, ready to load on the bench.
#[component]
fn SuggestionRow(sug: MixSuggestion, target_hex: String, manual_parts: Signal<Vec<PaintPart>>) -> Element {
    let state = use_context::<AppState>();
    let locale = *state.locale.read();
    let parts_to_load = sug.parts.clone();

    rsx! {
        tr {
            td {
                div { class: "row",
                    ColorSwatch { hex: target_hex.clone(), size: 24 }
                    span { class: "muted", "→" }
                    ColorSwatch { hex: sug.color.to_hex(), size: 24 }
                    span { class: "mono", "{sug.color.to_hex()}" }
                }
            }
            td { DeltaEBadge { delta_e: sug.delta_e } }
            td {
                div { style: "font-weight: 600;", "{sug.describe_i18n(locale)}" }
                div { class: "muted", style: "font-size: 11px;",
                    {locale.tr("Séchage", "Drying")},
                    " : {sug.drying_class().label(locale)}"
                }
            }
            td {
                button {
                    class: "btn",
                    onclick: move |_| {
                        manual_parts.set(parts_to_load.clone());
                    },
                    {locale.tr("← Simuler", "← Simulate")}
                }
            }
        }
    }
}

/// The CIELAB a*/b* map of the tubes in play, with the target and the best recipe found.
#[component]
pub fn GamutMapCard(gamut_paints: Vec<Paint>, target_hex: String, best_achieved: Option<String>) -> Element {
    let state = use_context::<AppState>();
    let locale = *state.locale.read();

    rsx! {
        div { class: "card",
            h3 { class: "card-title",
                {locale.tr("Carte de Gamut Chromatique CIELAB (a* / b*)", "CIELAB Chromatic Gamut Map (a* / b*)")}
            }
            p { class: "muted", style: "font-size: 11.5px; margin-bottom: 10px;",
                {locale.tr("Polygone de couverture chromatique de la palette choisie : visualisez la position de chaque tube (code court) et vérifiez si la couleur visée est atteignable.", "Chromatic coverage polygon of the selected palette: visualize each tube position (short code) and check whether the target color is reachable.")}
            }
            GamutMapSvg {
                paints: gamut_paints,
                target_hex: Some(target_hex.clone()),
                achieved_hex: best_achieved,
                size: 260,
            }
        }
    }
}
