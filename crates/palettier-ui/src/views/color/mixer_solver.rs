//! The mixer's right column: cross-system target picker, inverse solver, and gamut map.

use crate::actions::mixer;
use crate::state::{AppServices, AppState};
use crate::theme::{ColorSwatch, DeltaEBadge, GamutMapSvg, HexColorInput};
use dioxus::prelude::*;
use palettier_domain::mix::{MixSuggestion, PaintPart};
use palettier_domain::paint::{Paint, PaintSystem};
use palettier_domain::palette::Palette;

fn opposite_system(system: PaintSystem) -> PaintSystem {
    match system {
        PaintSystem::Oil => PaintSystem::Acrylic,
        PaintSystem::Acrylic => PaintSystem::Oil,
    }
}

/// « Comment obtenir cette couleur ? »: target paint/hex, search scope, and recipes found.
#[component]
pub fn MixerSolverCard(
    system: PaintSystem,
    catalog_paints: Vec<Paint>,
    palettes: Vec<Palette>,
    all_paints: Vec<Paint>,
    target_hex: String,
    effective_scope: usize,
    scope_idx: Signal<usize>,
    max_paints: Signal<usize>,
    suggestions: Signal<Vec<MixSuggestion>>,
    manual_parts: Signal<Vec<PaintPart>>,
) -> Element {
    let mut state = use_context::<AppState>();
    let services = use_context::<AppServices>();
    let locale = *state.locale.read();
    let (in_stock, catalog) = (mixer::SCOPE_IN_STOCK, mixer::SCOPE_CATALOG);
    let mut solving = use_signal(|| false);
    let mut solve_seq = use_signal(|| 0u64);

    // Cross-system target paint picker state (defaults to the opposite system).
    let mut target_sys_filter = use_signal(|| Some(opposite_system(system)));
    let mut target_brand = use_signal(|| None::<String>);
    let mut target_query = use_signal(String::new);
    let mut browse_targets = use_signal(|| false);
    let mut selected_target = use_signal(|| None::<Paint>);

    use_effect(use_reactive!(|system| {
        target_sys_filter.set(Some(opposite_system(system)));
        target_brand.set(None);
    }));

    let trigger_solve = {
        let pals = palettes.clone();
        let pts = all_paints.clone();
        let svc = services.clone();
        move |hex: String| {
            let pool = mixer::search_pool(&pals, &pts, effective_scope);
            let max = max_paints();
            let svc = svc.clone();
            let seq = *solve_seq.peek() + 1;
            solve_seq.set(seq);
            solving.set(true);
            spawn(async move {
                let res = tokio::task::spawn_blocking(move || {
                    mixer::suggest_recipes(&svc, &hex, &pool, max)
                })
                .await;
                if *solve_seq.peek() == seq {
                    if let Ok(Some(found)) = res {
                        suggestions.set(found);
                    }
                    solving.set(false);
                }
            });
        }
    };

    let sys_pool: Vec<Paint> = catalog_paints
        .iter()
        .filter(|p| target_sys_filter().is_none_or(|sys| p.system == sys))
        .cloned()
        .collect();
    let target_brands = mixer::brands(&sys_pool);
    let active_brand = {
        let cur = target_brand();
        cur.filter(|b| target_brands.iter().any(|known| known == b))
    };
    let query_text = target_query();
    let picker_open = browse_targets() || !query_text.trim().is_empty() || active_brand.is_some();
    let matching_targets = if picker_open {
        mixer::search_target_paints(
            &catalog_paints,
            target_sys_filter(),
            active_brand.as_deref(),
            &query_text,
            24,
        )
    } else {
        Vec::new()
    };

    let active_ref = selected_target().filter(|p| p.hex_color.eq_ignore_ascii_case(&target_hex));

    rsx! {
        div { class: "card",
            h3 { class: "card-title",
                {locale.tr("Comment obtenir cette couleur ? (Solveur inverse)", "How to Mix This Color? (Inverse Solver)")}
            }

            // Cross-system catalog color target picker (e.g. Acrylic reference -> Oil mix)
            div { class: "target-picker-box",
                div { class: "row-between u-mb-8 u-fwrap-wrap u-gap-6",
                    span { class: "u-fw-650 u-fs-12",
                        "🎯 ",
                        {locale.tr(
                            "Choisir un pot / tube cible du catalogue",
                            "Pick a catalog paint as target",
                        )}
                    }
                    div { class: "row u-gap-4",
                        for (label, opt) in [
                            (PaintSystem::Acrylic.label(locale), Some(PaintSystem::Acrylic)),
                            (PaintSystem::Oil.label(locale), Some(PaintSystem::Oil)),
                            (locale.tr("Tous", "All"), None),
                        ] {
                            button {
                                class: if target_sys_filter() == opt { "pill pill-accent u-cur-pointer" } else { "pill u-cur-pointer" },
                                onclick: move |_| {
                                    target_sys_filter.set(opt);
                                    target_brand.set(None);
                                },
                                "{label}"
                            }
                        }
                    }
                }

                div { class: "row u-gap-6",
                    div { class: "target-search-wrap",
                        input {
                            r#type: "text",
                            placeholder: locale.tr(
                                "Chercher un pot/tube cible (ex. Mephiston, 70.951, Burnt Umber)…",
                                "Search target paint (e.g. Mephiston, 70.951, Burnt Umber)…",
                            ),
                            value: "{query_text}",
                            oninput: move |e| target_query.set(e.value()),
                        }
                        if !query_text.is_empty() {
                            button {
                                class: "target-search-clear",
                                title: locale.tr("Effacer", "Clear"),
                                onclick: move |_| target_query.set(String::new()),
                                "✕"
                            }
                        }
                    }
                    button {
                        class: if browse_targets() { "btn pill-accent" } else { "btn" },
                        onclick: move |_| browse_targets.set(!browse_targets()),
                        if browse_targets() {
                            {locale.tr("Masquer", "Hide")}
                        } else {
                            {locale.tr("Parcourir", "Browse")}
                        }
                    }
                }

                if picker_open {
                    div { class: "row u-gap-4 u-fwrap-wrap u-mt-8",
                        button {
                            class: if active_brand.is_none() { "pill pill-accent u-cur-pointer" } else { "pill u-cur-pointer" },
                            onclick: move |_| target_brand.set(None),
                            {locale.tr("Toutes marques", "All brands")}
                        }
                        for b in &target_brands {
                            {
                                let b_name = b.clone();
                                let is_sel = active_brand.as_deref() == Some(b.as_str());
                                rsx! {
                                    button {
                                        class: if is_sel { "pill pill-accent u-cur-pointer" } else { "pill u-cur-pointer" },
                                        onclick: move |_| target_brand.set(Some(b_name.clone())),
                                        "{b}"
                                    }
                                }
                            }
                        }
                    }

                    if matching_targets.is_empty() {
                        p { class: "muted u-fs-12 u-mt-8",
                            {locale.tr("Aucune teinte correspondante dans ce filtre.", "No matching paint in this filter.")}
                        }
                    } else {
                        div { class: "target-paint-grid",
                            for pt in matching_targets {
                                {
                                    let is_cur = active_ref
                                        .as_ref()
                                        .is_some_and(|sel| sel.brand == pt.brand && sel.name == pt.name);
                                    let hex = pt.hex_color.clone();
                                    let pt_pick = pt.clone();
                                    let mut solve_now = trigger_solve.clone();
                                    rsx! {
                                        button {
                                            class: if is_cur { "target-paint-option active" } else { "target-paint-option" },
                                            onclick: move |_| {
                                                state.mixer_target_hex.set(hex.clone());
                                                selected_target.set(Some(pt_pick.clone()));
                                                browse_targets.set(false);
                                                target_query.set(String::new());
                                                solve_now(hex.clone());
                                            },
                                            div { class: "row u-gap-8 u-ai-center u-minw-0",
                                                ColorSwatch { hex: pt.hex_color.clone(), size: 22 }
                                                div { class: "u-minw-0",
                                                    div { class: "u-fw-600 u-fs-12", "{pt.name}" }
                                                    div { class: "muted u-fs-10p5",
                                                        if pt.code.is_empty() {
                                                            "{pt.brand}"
                                                        } else {
                                                            "{pt.brand} · {pt.code}"
                                                        }
                                                    }
                                                }
                                            }
                                            span { class: "pill u-fs-10", "{pt.system.label(locale)}" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if let Some(ref_pt) = active_ref {
                div { class: "target-active-pill",
                    ColorSwatch { hex: ref_pt.hex_color.clone(), size: 18 }
                    span { class: "u-fw-600", "{ref_pt.brand} · {ref_pt.name}" }
                    span { class: "muted",
                        "({ref_pt.system.label(locale)} → {system.label(locale)})"
                    }
                    button {
                        class: "btn u-p-0-6 u-fs-11",
                        title: locale.tr("Détacher la référence", "Clear reference"),
                        onclick: move |_| selected_target.set(None),
                        "✕"
                    }
                }
            }

            div { class: "row-between u-mb-12",
                div { class: "row",
                    div {
                        label { class: "field-label", {locale.tr("Couleur visée", "Target Color")} }
                        HexColorInput {
                            value: target_hex.clone(),
                            onchange: move |v: String| {
                                selected_target.set(None);
                                state.mixer_target_hex.set(v);
                            },
                        }
                    }
                    div {
                        label { class: "field-label", {locale.tr("Chercher dans", "Search Scope")} }
                        select {
                            value: "{effective_scope}",
                            onchange: move |ev| {
                                if let Ok(v) = ev.value().parse::<usize>() {
                                    scope_idx.set(v);
                                }
                            },
                            for (idx, pal) in palettes.iter().enumerate() {
                                option {
                                    value: "{idx}",
                                    selected: effective_scope == idx,
                                    "Palette: {pal.name} ({pal.paints.len()} tubes)"
                                }
                            }
                            option {
                                value: "{in_stock}",
                                selected: effective_scope == in_stock,
                                {locale.tr("Mes tubes en stock", "My in-stock tubes")}
                            }
                            option {
                                value: "{catalog}",
                                selected: effective_scope == catalog,
                                {locale.tr("Tout le catalogue", "Full catalog")}, " ({all_paints.len()} tubes)"
                            }
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
                    disabled: solving(),
                    onclick: {
                        let mut solve_now = trigger_solve.clone();
                        move |_| {
                            let target = state.mixer_target_hex.read().clone();
                            solve_now(target);
                        }
                    },
                    if solving() {
                        {locale.tr("Calcul en cours…", "Computing…")}
                    } else {
                        {locale.tr("Proposer des recettes", "Suggest recipes")}
                    }
                }
            }

            if suggestions().is_empty() {
                p { class: "muted",
                    {locale.tr("Choisissez une teinte cible (hex ou pot/tube du catalogue) et cliquez sur « Proposer des recettes » pour calculer les meilleurs mélanges en parts entières.", "Choose a target color (hex or catalog paint) and click 'Suggest recipes' to compute the best whole-part mixes.")}
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
                div { class: "u-fw-600", "{sug.describe(locale)}" }
                div { class: "muted u-fs-11",
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

/// CIELAB chromatic gamut map around the palette.
#[component]
pub fn GamutMapCard(
    gamut_paints: Vec<Paint>,
    target_hex: String,
    best_achieved: Option<String>,
) -> Element {
    let state = use_context::<AppState>();
    let locale = *state.locale.read();

    rsx! {
        div { class: "card",
            h3 { class: "card-title",
                {locale.tr("Carte du gamut chromatique (plan CIELAB a* b*)", "Chromatic Gamut Map (CIELAB a* b* plane)")}
            }
            div { class: "row-between u-ai-center u-gap-16",
                GamutMapSvg {
                    paints: gamut_paints,
                    target_hex: Some(target_hex),
                    achieved_hex: best_achieved,
                    size: 250,
                }
                div { class: "u-flex-1 u-fs-12 u-lh-1p55",
                    p { class: "muted u-mb-8",
                        {locale.tr("Chaque point représente la position chromatique d'un tube du scope actif. Le polygone jaune délimite l'enveloppe des teintes pures atteignables par mélange.", "Each dot shows a paint's chromatic position in the active scope. The amber polygon outlines the convex hull of achievable hues.")}
                    }
                    div { class: "u-mb-4",
                        span { class: "u-fw-700 u-c-accent", "⊕ " }
                        {locale.tr("Couleur cible souhaitée", "Target color")}
                    }
                    div {
                        span { class: "u-fw-700 u-c-pill-ok-text", "◆ " }
                        {locale.tr("Meilleur mélange calculé par Kubelka-Munk", "Best Kubelka-Munk mix found")}
                    }
                }
            }
        }
    }
}
