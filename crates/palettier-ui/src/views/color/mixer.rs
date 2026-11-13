//! The mixer: a free bench on the left, the inverse Kubelka-Munk solver on the right.

use super::mixer_solver::{GamutMapCard, MixerSolverCard};
use crate::actions::mixer::{self, BenchMix};
use crate::state::{AppServices, AppState};
use crate::theme::{ColorSwatch, DeltaEBadge, PaintSwatch, SectionHeader};
use dioxus::prelude::*;
use palettier_domain::color::{Rgb, delta_e_2000};
use palettier_domain::mix::{MixResult, MixSuggestion, PaintPart};
use palettier_domain::paint::Paint;

#[component]
pub fn MixerView() -> Element {
    let state = use_context::<AppState>();
    let services = use_context::<AppServices>();

    let locale = *state.locale.read();
    // Oil and acrylic never mix: the mixer only offers paints of the working system.
    let system = *state.paint_system.read();
    let all_paints = mixer::paints_of_system(&state.paints.read(), system);
    let palettes = mixer::palettes_of_system(&state.palettes.read(), system);
    let target_hex = state.mixer_target_hex.read().clone();

    // Left panel: Freeform bench simulator (ColorMixService::mix)
    let brands = mixer::brands(&all_paints);
    let sim_brand = use_signal(|| brands.first().cloned().unwrap_or_default());
    let sim_paint_idx = use_signal(|| 0usize);
    let sim_parts_input = use_signal(|| 1.0_f64);
    let manual_parts = use_signal(Vec::<PaintPart>::new);
    let save_mix_name = use_signal(String::new);

    // Right panel: Inverse Kubelka-Munk solver
    let scope_idx = use_signal(|| 0usize); // 0..N = Palette, 9998 = In stock, 9999 = Full catalog
    let max_paints = use_signal(|| 3usize);
    let suggestions = use_signal(Vec::<MixSuggestion>::new);

    let brand_paints = mixer::paints_of_brand(&all_paints, &sim_brand());

    let current_mix_result = {
        let parts_snapshot = manual_parts.read().clone();
        if parts_snapshot.is_empty() {
            None
        } else {
            services.mix_service.mix(&parts_snapshot).ok()
        }
    };

    rsx! {
        div {
            SectionHeader {
                title: locale.tr("Mélangeur & Simulateur Kubelka-Munk", "Kubelka-Munk Mixer & Simulator").to_string(),
                subtitle: locale.tr(
                    "Composez librement une recette en parts à gauche (avec répartition en volume vs poids pigmentaire), ou laissez le solveur Kubelka-Munk trouver le dosage optimal à droite.",
                    "Freely compose a part-based recipe on the left (comparing volume share vs pigment strength), or let the inverse Kubelka-Munk solver find optimal ratios on the right."
                ).to_string(),
                actions: None
            }

            div { class: "grid-2",
                // Left column: Bench simulator (ColorMixService::mix)
                div {
                    MixerBenchCard {
                        brands,
                        brand_paints,
                        mix: current_mix_result.clone(),
                        sim_brand,
                        sim_paint_idx,
                        sim_parts_input,
                        manual_parts,
                    }

                    // Simulated mix result card
                    if let Some(res) = current_mix_result {
                        MixerResultCard { res, target_hex: target_hex.clone(), manual_parts, save_mix_name }
                    }
                }

                // Right column: Inverse Kubelka-Munk solver
                div {
                    MixerSolverCard {
                        palettes: palettes.clone(),
                        all_paints: all_paints.clone(),
                        target_hex: target_hex.clone(),
                        scope_idx,
                        max_paints,
                        suggestions,
                        manual_parts,
                    }

                    // CIELAB Chromatic Gamut Map (a* / b*)
                    GamutMapCard {
                        gamut_paints: mixer::gamut_paints(&palettes, scope_idx(), &manual_parts.read(), &all_paints),
                        target_hex: target_hex.clone(),
                        best_achieved: suggestions().first().map(|s| s.color.to_hex()),
                    }
                }
            }
        }
    }
}

/// The bench: pick tubes by brand, dose them, and read their volume and pigment shares.
#[component]
fn MixerBenchCard(
    brands: Vec<String>,
    brand_paints: Vec<Paint>,
    mix: Option<MixResult>,
    sim_brand: Signal<String>,
    sim_paint_idx: Signal<usize>,
    sim_parts_input: Signal<f64>,
    manual_parts: Signal<Vec<PaintPart>>,
) -> Element {
    let state = use_context::<AppState>();
    let locale = *state.locale.read();

    rsx! {
        div { class: "card",
            div { class: "row-between", style: "margin-bottom: 10px;",
                h3 { class: "card-title", style: "margin: 0;",
                    {locale.tr("Composition du mélange sur la palette", "Palette Mix Composition")}
                }
                if !manual_parts.read().is_empty() {
                    button {
                        class: "btn",
                        onclick: move |_| manual_parts.set(Vec::new()),
                        {locale.tr("Vider la paillasse", "Clear bench")}
                    }
                }
            }

            div { class: "card-subtle", style: "margin-bottom: 12px;",
                div { class: "row",
                    select {
                        style: "width: 155px;",
                        value: "{sim_brand}",
                        onchange: move |e| {
                            sim_brand.set(e.value());
                            sim_paint_idx.set(0);
                        },
                        for b in &brands {
                            option { value: "{b}", "{b}" }
                        }
                    }
                    select {
                        style: "flex: 1; min-width: 180px;",
                        onchange: move |e| {
                            if let Ok(i) = e.value().parse::<usize>() {
                                sim_paint_idx.set(i);
                            }
                        },
                        for (i, pt) in brand_paints.iter().enumerate() {
                            option {
                                value: "{i}",
                                selected: *sim_paint_idx.read() == i,
                                "{pt.name} ({pt.hex_color})"
                            }
                        }
                    }
                    input {
                        r#type: "number",
                        style: "width: 68px;",
                        min: "0.5",
                        max: "20",
                        step: "0.5",
                        value: "{sim_parts_input}",
                        oninput: move |e| {
                            if let Ok(v) = e.value().parse::<f64>() {
                                sim_parts_input.set(v.max(0.5));
                            }
                        }
                    }
                    button {
                        class: "btn-primary",
                        onclick: {
                            let bp = brand_paints.clone();
                            move |_| {
                                if let Some(pt) = bp.get(*sim_paint_idx.read()).cloned() {
                                    let dose = *sim_parts_input.read();
                                    let list = mixer::add_to_bench(&manual_parts.read(), pt, dose);
                                    manual_parts.set(list);
                                }
                            }
                        },
                        {locale.tr("+ Ajouter", "+ Add")}
                    }
                }
            }

            if manual_parts.read().is_empty() {
                p { class: "muted", style: "padding: 14px 4px;",
                    {locale.tr("Ajoutez des tubes ci-dessus ou cliquez sur « ← Simuler » en face d'une recette suggérée à droite pour simuler le mélange.", "Add tubes above or click '← Simulate' next to a suggested recipe on the right to simulate the mix.")}
                }
            } else {
                table { class: "data-table",
                    thead {
                        tr {
                            th { {locale.tr("Peinture", "Paint")} }
                            th { "Parts" }
                            th { "Volume" }
                            th { {locale.tr("Poids dans la couleur", "Pigment Weight")} }
                            th { "" }
                        }
                    }
                    tbody {
                        for (idx, part) in manual_parts.read().clone().into_iter().enumerate() {
                            {
                                let comp = mix.as_ref().and_then(|r| r.components.get(idx));
                                rsx! {
                                    BenchRow {
                                        idx,
                                        part,
                                        vol_pct: comp.map(|c| c.volume_share * 100.0).unwrap_or(0.0),
                                        pig_pct: comp.map(|c| c.pigment_share * 100.0).unwrap_or(0.0),
                                        manual_parts,
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// One tube on the bench, with its dose and its shares.
#[component]
fn BenchRow(
    idx: usize,
    part: PaintPart,
    vol_pct: f64,
    pig_pct: f64,
    manual_parts: Signal<Vec<PaintPart>>,
) -> Element {
    rsx! {
        tr {
            td {
                div { class: "row",
                    PaintSwatch {
                        masstone_hex: part.paint.hex_color.clone(),
                        tint_hex: part.paint.tint_hex.clone(),
                        width: 28,
                        height: 20,
                    }
                    div {
                        div { style: "font-weight: 600;", "{part.paint.name}" }
                        div { class: "muted", style: "font-size: 11px;", "{part.paint.brand}" }
                    }
                }
            }
            td {
                div { class: "row",
                    button {
                        class: "btn",
                        style: "padding: 2px 7px;",
                        onclick: move |_| {
                            let list = mixer::nudge_part(&manual_parts.read(), idx, -0.5);
                            manual_parts.set(list);
                        },
                        "−"
                    }
                    span { class: "mono", "{part.parts:.1}" }
                    button {
                        class: "btn",
                        style: "padding: 2px 7px;",
                        onclick: move |_| {
                            let list = mixer::nudge_part(&manual_parts.read(), idx, 0.5);
                            manual_parts.set(list);
                        },
                        "+"
                    }
                }
            }
            td {
                div { class: "mono", "{vol_pct:.0} %" }
                div { class: "share-bar-bg",
                    div { class: "share-bar-fill", style: "width: {vol_pct:.0}%;" }
                }
            }
            td {
                div { class: "mono", "{pig_pct:.0} %" }
                div { class: "share-bar-bg",
                    div { class: "share-bar-fill", style: "width: {pig_pct:.0}%;" }
                }
            }
            td {
                button {
                    class: "btn-danger",
                    onclick: move |_| {
                        let mut list = manual_parts.read().clone();
                        if idx < list.len() {
                            list.remove(idx);
                            manual_parts.set(list);
                        }
                    },
                    "✕"
                }
            }
        }
    }
}

/// The simulated mix against the target, and laying it on the palette.
#[component]
fn MixerResultCard(
    res: MixResult,
    target_hex: String,
    manual_parts: Signal<Vec<PaintPart>>,
    save_mix_name: Signal<String>,
) -> Element {
    let mut state = use_context::<AppState>();
    let services = use_context::<AppServices>();
    let locale = *state.locale.read();

    let res_hex = res.hex();
    let gap_to_target = Rgb::of_hex(&target_hex)
        .map(|t| delta_e_2000(t, res.color))
        .unwrap_or(0.0);
    let pigs = res.pigments.iter().cloned().collect::<Vec<_>>().join(", ");
    let recipe_desc = mixer::bench_recipe(&manual_parts.read());

    rsx! {
        div { class: "card",
            h3 { class: "card-title", {locale.tr("Résultat du mélange simulé", "Simulated Mix Result")} }
            div { class: "row-between", style: "margin-bottom: 12px;",
                div { class: "row",
                    div {
                        div { class: "muted", style: "font-size: 10.5px; margin-bottom: 3px;",
                            {locale.tr("Obtenu", "Achieved")}
                        }
                        ColorSwatch { hex: res_hex.clone(), size: 68 }
                    }
                    div {
                        div { class: "muted", style: "font-size: 10.5px; margin-bottom: 3px;",
                            {locale.tr("Cible", "Target")}
                        }
                        ColorSwatch { hex: target_hex.clone(), size: 68 }
                    }
                    div { style: "margin-left: 8px;",
                        div { class: "mono", style: "font-size: 20px; font-weight: 700;", "{res_hex}" }
                        div { class: "row", style: "margin-top: 4px;",
                            DeltaEBadge { delta_e: gap_to_target }
                            span { class: "pill", {locale.tr("Séchage : ", "Drying: ")}, "{res.drying_class.label(locale)}" }
                        }
                        div { class: "muted mono", style: "margin-top: 5px; font-size: 11.5px;",
                            "Pigments: {pigs}"
                        }
                    }
                }
            }

            for w in &res.warnings {
                div { class: "card-subtle", style: "border-color: var(--pill-warn-border); margin-bottom: 8px; font-size: 12px;",
                    "• {w}"
                }
            }

            div { class: "row", style: "margin-top: 10px;",
                input {
                    r#type: "text",
                    style: "flex: 1;",
                    placeholder: locale.tr("Nom pour poser ce mélange sur la palette (ex. Ombre chaude)…", "Name to place this mix on the palette (e.g. Warm Shadow)…"),
                    value: "{save_mix_name}",
                    oninput: move |e| save_mix_name.set(e.value())
                }
                button {
                    class: "btn-primary",
                    onclick: {
                        let svc = services.clone();
                        let hex_to_save = res_hex.clone();
                        let dc = res.drying_class;
                        let rec_str = recipe_desc.clone();
                        move |_| {
                            let loc = *state.locale.read();
                            let mix = BenchMix {
                                name: save_mix_name(),
                                hex_color: hex_to_save.clone(),
                                recipe: rec_str.clone(),
                                drying_class: dc,
                                system: *state.paint_system.peek(),
                                workshop: *state.workshop.read(),
                            };
                            let svc = svc.clone();
                            spawn(async move {
                                if let Ok(all) = mixer::place_bench_mix(&svc, mix, loc).await {
                                    state.mixes.set(all);
                                    save_mix_name.set(String::new());
                                    state.notify(loc.tr("Mélange posé sur la palette (suivi dans Aujourd'hui).", "Mix placed on palette (tracked in Today)."));
                                }
                            });
                        }
                    },
                    {locale.tr("+ Poser sur la palette", "+ Place on palette")}
                }
            }
        }
    }
}
