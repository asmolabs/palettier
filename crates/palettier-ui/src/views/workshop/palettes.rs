//! Working palettes: the saved selections of tubes, their table, matrix, gamut and mother mixes.

use crate::actions::palettes;
use crate::state::{AppServices, AppState};
use crate::theme::{ColorSwatch, DeltaEBadge, GamutMapSvg, PaintSwatch, SectionHeader};
use dioxus::prelude::*;
use palettier_domain::guide::MixingGuideBuilder;
use palettier_domain::paint::Paint;
use palettier_domain::palette::Palette;
use palettier_domain::project::MissingPaint;

/// The palettes of the working system (an empty palette belongs to both).
fn palettes_of_system(state: &AppState) -> Vec<Palette> {
    let system = *state.paint_system.read();
    state
        .palettes
        .read()
        .iter()
        .filter(|p| p.paints.is_empty() || p.system() == system)
        .cloned()
        .collect()
}

#[component]
pub fn PalettesView() -> Element {
    let state = use_context::<AppState>();
    let locale = *state.locale.read();
    let palettes = palettes_of_system(&state);
    let selected_idx = use_signal(|| 0usize);
    let active_pal = palettes.get(selected_idx()).cloned();

    rsx! {
        div {
            SectionHeader {
                title: locale.tr("Palettes de travail", "Working Palettes").to_string(),
                subtitle: locale.tr(
                    "Sélections cohérentes de tubes pour vos sujets (Zorn, carnations, blindages...).",
                    "Curated tube selections for your subjects (Zorn, skin tones, armor...)."
                ).to_string(),
                actions: None
            }

            div { class: "grid-3",
                div {
                    NewPaletteCard { selected_idx }
                    PaletteList { selected_idx }
                }

                div { style: "grid-column: span 2;",
                    if let Some(pal) = active_pal {
                        PaletteDetail { pal: pal.clone(), selected_idx }
                        // 2-column grid: CIELAB Gamut Map + Intermediate Mother Mixes (Si, BF, NC)
                        if !pal.paints.is_empty() {
                            PaletteGamut { pal }
                        }
                    }
                }
            }
        }
    }
}

/// Name, purpose and notes of a new palette, and the Norma Blue set.
#[component]
fn NewPaletteCard(selected_idx: Signal<usize>) -> Element {
    let mut state = use_context::<AppState>();
    let services = use_context::<AppServices>();
    let locale = *state.locale.read();
    let all_paints = state.paints_of_system();

    let mut new_name = use_signal(String::new);
    let mut new_purpose = use_signal(String::new);
    let mut new_notes = use_signal(String::new);

    let create = {
        let svc = services.clone();
        move |_| {
            let name = new_name();
            if name.trim().is_empty() {
                return;
            }
            let (purpose, notes) = (new_purpose(), new_notes());
            let svc = svc.clone();
            spawn(async move {
                if let Ok(Some(all)) = palettes::create_palette(&svc, &name, &purpose, &notes).await {
                    state.palettes.set(all);
                    new_name.set(String::new());
                    new_purpose.set(String::new());
                    new_notes.set(String::new());
                }
            });
        }
    };

    let add_norma_blue = move |_| {
        let loc = *state.locale.read();
        let svc = services.clone();
        let pts = all_paints.clone();
        spawn(async move {
            if let Ok(all) = palettes::add_norma_blue_set(&svc, &pts, loc).await {
                selected_idx.set(all.len().saturating_sub(1));
                state.palettes.set(all);
                state.notify(loc.tr("Palette Schmincke Norma Blue (8 tubes) ajoutée.", "Schmincke Norma Blue (8 tubes) palette added."));
            }
        });
    };

    rsx! {
        div { class: "card",
            h3 { class: "card-title", {locale.tr("Nouvelle palette", "New Palette")} }
            div { style: "margin-bottom:8px;",
                label { class: "field-label", {locale.tr("Nom", "Name")} }
                input {
                    r#type: "text",
                    style: "width:100%;",
                    placeholder: locale.tr("ex. Drap impérial bleu", "e.g. Imperial Blue Cloth"),
                    value: "{new_name}",
                    oninput: move |ev| new_name.set(ev.value())
                }
            }
            div { style: "margin-bottom:8px;",
                label { class: "field-label", {locale.tr("Usage / Sujet", "Purpose / Subject")} }
                input {
                    r#type: "text",
                    style: "width:100%;",
                    placeholder: locale.tr("ex. Uniformes 75mm", "e.g. 75mm Uniforms"),
                    value: "{new_purpose}",
                    oninput: move |ev| new_purpose.set(ev.value())
                }
            }
            div { style: "margin-bottom:10px;",
                label { class: "field-label", {locale.tr("Notes d'atelier", "Studio Notes")} }
                textarea {
                    style: "width:100%;height:60px;",
                    value: "{new_notes}",
                    oninput: move |ev| new_notes.set(ev.value())
                }
            }
            button {
                class: "btn-primary",
                style: "width:100%;",
                onclick: create,
                {locale.tr("+ Créer la palette", "+ Create palette")}
            }
            button {
                class: "btn",
                style: "width:100%; margin-top: 8px;",
                onclick: add_norma_blue,
                {locale.tr("+ Coffret Norma Blue (8 tubes)", "+ Norma Blue Set (8 tubes)")}
            }
        }
    }
}

/// « Palettes enregistrées »: every palette of the working system.
#[component]
fn PaletteList(selected_idx: Signal<usize>) -> Element {
    let state = use_context::<AppState>();
    let locale = *state.locale.read();
    let palettes = palettes_of_system(&state);

    rsx! {
        div { class: "card",
            h3 { class: "card-title", {locale.tr("Palettes enregistrées", "Saved Palettes")}, " ({palettes.len()})" }
            for (idx, pal) in palettes.iter().enumerate() {
                div {
                    class: "card-subtle",
                    style: if idx == selected_idx() {
                        "margin-bottom:8px;border-color:var(--accent);cursor:pointer;"
                    } else {
                        "margin-bottom:8px;cursor:pointer;"
                    },
                    onclick: move |_| selected_idx.set(idx),
                    div { class: "row-between",
                        strong { "{pal.name}" }
                        span { class: "pill", "{pal.paints.len()} tubes" }
                    }
                    if !pal.purpose.is_empty() {
                        div { class: "muted", style: "font-size:12px;", "{pal.purpose}" }
                    }
                    if !pal.paints.is_empty() {
                        div { class: "row", style: "margin-top: 6px; gap: 4px;",
                            for pt in &pal.paints {
                                PaintSwatch {
                                    masstone_hex: pt.hex_color.clone(),
                                    tint_hex: pt.tint_hex.clone(),
                                    width: 24,
                                    height: 14,
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// The selected palette: its header, tube picker, tube table, matrix and substitutes.
#[component]
fn PaletteDetail(pal: Palette, selected_idx: Signal<usize>) -> Element {
    let mut state = use_context::<AppState>();
    let services = use_context::<AppServices>();
    let locale = *state.locale.read();
    let all_paints = state.paints_of_system();

    let mut paint_to_add_idx = use_signal(|| 0usize);
    let mut show_matrix = use_signal(|| false);

    let delete = {
        let svc = services.clone();
        move |id: i64| {
            let svc = svc.clone();
            spawn(async move {
                if let Ok(all) = palettes::delete_palette(&svc, id).await {
                    selected_idx.set(0);
                    state.palettes.set(all);
                }
            });
        }
    };

    let add_tube = {
        let pal = pal.clone();
        let pts = all_paints.clone();
        move |_| {
            if let Some(pt) = pts.get(paint_to_add_idx()).cloned() {
                let svc = services.clone();
                let pal = pal.clone();
                spawn(async move {
                    if let Ok(all) = palettes::add_paint(&svc, &pal, pt).await {
                        state.palettes.set(all);
                    }
                });
            }
        }
    };

    rsx! {
        div { class: "card",
            div { class: "row-between", style: "margin-bottom:10px;",
                div {
                    h2 { class: "card-title", style: "font-size:18px;margin-bottom:2px;", "{pal.name}" }
                    span { class: "muted", "{pal.purpose}" }
                }
                div { class: "row",
                    span {
                        class: "pill pill-accent",
                        {locale.tr("Rythme imposé", "Pacing set by")}, " : {pal.slowest_drying_class().label_i18n(locale)}"
                    }
                    button {
                        class: if show_matrix() { "btn-primary" } else { "btn" },
                        style: "font-size: 11px; padding: 4px 10px;",
                        onclick: move |_| show_matrix.set(!show_matrix()),
                        "🎨 "
                        {locale.tr("Nuancier Matriciel", "Mixing Matrix")}
                    }
                    if let Some(id) = pal.id {
                        button {
                            class: "btn-danger",
                            onclick: move |_| delete(id),
                            {locale.tr("Supprimer", "Delete")}
                        }
                    }
                }
            }
            if !pal.notes.is_empty() {
                p { class: "muted", style: "margin-bottom:14px;white-space:pre-line;", "{pal.notes}" }
            }

            div { class: "row", style: "margin-bottom:14px;",
                select {
                    style: "flex:1;",
                    onchange: move |ev| {
                        if let Ok(i) = ev.value().parse::<usize>() {
                            paint_to_add_idx.set(i);
                        }
                    },
                    for (i, pt) in all_paints.iter().enumerate() {
                        option { value: "{i}", "{pt.brand} — {pt.name} ({pt.hex_color})" }
                    }
                }
                button {
                    class: "btn-primary",
                    onclick: add_tube,
                    {locale.tr("+ Ajouter ce tube", "+ Add tube")}
                }
            }

            PaletteTubeTable { pal: pal.clone() }

            if show_matrix() && !pal.paints.is_empty() {
                PaletteMatrix { pal: pal.clone() }
            }

            // Substitutes if some palette tubes are out of stock
            PaletteSubstitutes { pal }
        }
    }
}

#[component]
fn PaletteTubeTable(pal: Palette) -> Element {
    let state = use_context::<AppState>();
    let locale = *state.locale.read();

    rsx! {
        table { class: "data-table",
            thead {
                tr {
                    th { "Code" }
                    th { {locale.tr("Masstone / Coupe 1:10", "Masstone / 1:10 Tint")} }
                    th { {locale.tr("Marque & Nom", "Brand & Name")} }
                    th { {locale.tr("Comportement à l'établi", "Workbench Behavior")} }
                    th { "Pigments" }
                    th { {locale.tr("Séchage", "Drying")} }
                    th { "" }
                }
            }
            tbody {
                for entry in MixingGuideBuilder::build_palette_entries_i18n(&pal.paints, locale) {
                    PaletteTubeRow {
                        pal: pal.clone(),
                        pt: entry.paint.clone(),
                        short_code: entry.short_code.clone(),
                        behavior: entry.behavior.clone(),
                    }
                }
            }
        }
    }
}

#[component]
fn PaletteTubeRow(pal: Palette, pt: Paint, short_code: String, behavior: String) -> Element {
    let mut state = use_context::<AppState>();
    let services = use_context::<AppServices>();
    let locale = *state.locale.read();

    let remove = {
        let pt = pt.clone();
        move |_| {
            let svc = services.clone();
            let pal = pal.clone();
            let pt = pt.clone();
            spawn(async move {
                if let Ok(all) = palettes::remove_paint(&svc, &pal, &pt).await {
                    state.palettes.set(all);
                }
            });
        }
    };

    rsx! {
        tr {
            td {
                span { class: "pill pill-accent mono", style: "font-weight: 800;", "{short_code}" }
            }
            td {
                PaintSwatch {
                    masstone_hex: pt.hex_color.clone(),
                    tint_hex: pt.tint_hex.clone(),
                    width: 38,
                    height: 24,
                }
            }
            td {
                strong { "{pt.name}" }
                if !pt.code.is_empty() {
                    span { class: "muted", " ({pt.code})" }
                }
                div { class: "muted", style: "font-size:11.5px;", "{pt.brand}" }
            }
            td { class: "muted", style: "font-size:12px;", "{behavior}" }
            td { class: "mono", "{pt.pigments.iter().cloned().collect::<Vec<_>>().join(\", \")}" }
            td { "{pt.drying_class.label_i18n(locale)}" }
            td {
                button {
                    class: "btn-danger",
                    onclick: remove,
                    {locale.tr("Retirer", "Remove")}
                }
            }
        }
    }
}

/// Every pair of tubes mixed 1:1, pure and cut with white.
#[component]
fn PaletteMatrix(pal: Palette) -> Element {
    let state = use_context::<AppState>();
    let locale = *state.locale.read();
    let matrix = pal.generate_matrix();

    rsx! {
        div { class: "card-subtle", style: "margin-top: 16px; overflow-x: auto;",
            h4 { style: "margin-bottom: 6px;",
                "🎨 "
                {locale.tr("Nuancier Matriciel 2 à 2 (Pur / Coupe au blanc)", "2-Tube Mixing Matrix (Masstone / Tint)")}
            }
            div { class: "muted", style: "font-size: 11.5px; margin-bottom: 12px;",
                {locale.tr(
                    "Chaque cellule présente le mélange pur 1:1 à gauche et la coupe au blanc de titane à droite.",
                    "Each cell displays the 1:1 pure mixture on the left and 1:10 Titanium White tint on the right."
                )}
            }
            table { class: "data-table", style: "font-size: 11px;",
                thead {
                    tr {
                        th { "×" }
                        for p in &matrix.paints {
                            th { style: "text-align: center; max-width: 90px; overflow: hidden; text-overflow: ellipsis;", "{p.name}" }
                        }
                    }
                }
                tbody {
                    for (r_idx, row) in matrix.rows.iter().enumerate() {
                        tr {
                            td {
                                div { class: "row", style: "gap: 6px;",
                                    ColorSwatch { hex: matrix.paints[r_idx].hex_color.clone(), size: 16 }
                                    span { style: "font-weight: 600;", "{matrix.paints[r_idx].name}" }
                                }
                            }
                            for cell in row {
                                td { style: "text-align: center; padding: 4px;",
                                    div { class: "row", style: "justify-content: center; gap: 3px;",
                                        ColorSwatch { hex: cell.pure_mix_hex.clone(), size: 20 }
                                        ColorSwatch { hex: cell.tint_mix_hex.clone(), size: 20 }
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

/// Out-of-stock tubes of the palette and the nearest tube on the shelf.
#[component]
fn PaletteSubstitutes(pal: Palette) -> Element {
    let state = use_context::<AppState>();
    let services = use_context::<AppServices>();
    let locale = *state.locale.read();
    let owned: Vec<_> = state.paints_of_system().into_iter().filter(|p| p.in_stock).collect();
    let missing = services.substitute_service.missing_among(&pal.paints, &owned);
    if missing.is_empty() {
        return rsx! {};
    }

    rsx! {
        div { class: "card-subtle", style: "margin-top: 14px; border-color: var(--accent);",
            strong {
                {locale.tr("Tubes hors stock dans cette palette", "Out-of-stock tubes in this palette")}, " ({missing.len()})"
            }
            for m in missing {
                PaletteSubstituteRow { pal: pal.clone(), m }
            }
        }
    }
}

#[component]
fn PaletteSubstituteRow(pal: Palette, m: MissingPaint) -> Element {
    let mut state = use_context::<AppState>();
    let services = use_context::<AppServices>();
    let locale = *state.locale.read();

    let replace = {
        let absent = m.paint.clone();
        move |repl: Paint| {
            let svc = services.clone();
            let pal = pal.clone();
            let absent = absent.clone();
            spawn(async move {
                let loc = *state.locale.read();
                if let Ok(all) = palettes::substitute_paint(&svc, &pal, &absent, repl).await {
                    state.palettes.set(all);
                    state.notify(loc.tr("Tube remplacé dans la palette.", "Tube replaced in palette."));
                }
            });
        }
    };

    rsx! {
        div { class: "row-between", style: "margin-top: 8px; padding-top: 6px; border-top: 1px solid var(--border-subtle);",
            div { class: "row",
                ColorSwatch { hex: m.paint.hex_color.clone(), size: 20 }
                strong { "{m.paint.display_name()}" }
                span { class: "muted", "→" }
                if let Some(ref near) = m.nearest {
                    ColorSwatch { hex: near.hex_color.clone(), size: 20 }
                    DeltaEBadge { delta_e: m.delta_e }
                }
                span { class: "muted", style: "font-size: 12px;", "{m.verdict_i18n(locale)}" }
            }
            if let Some(repl) = m.nearest.clone() {
                button {
                    class: "btn-primary",
                    style: "padding: 3px 10px; font-size: 11.5px;",
                    onclick: move |_| replace(repl.clone()),
                    {locale.tr("Remplacer dans la palette", "Replace in palette")}
                }
            }
        }
    }
}

/// The CIELAB gamut of the palette and its session-start mother mixes.
#[component]
fn PaletteGamut(pal: Palette) -> Element {
    let state = use_context::<AppState>();
    let services = use_context::<AppServices>();
    let locale = *state.locale.read();
    let dummy_plan = palettier_domain::plan::PaintingPlan {
        subject: pal.name.clone(),
        palette_name: pal.name.clone(),
        approach: String::new(),
        system: pal.system(),
        zones: Vec::new(),
    };
    let guide = services.guide_builder.from_plan_i18n(&dummy_plan, &pal.paints, locale);

    rsx! {
        div { class: "grid-2",
            div { class: "card",
                h3 { class: "card-title", {locale.tr("Couverture Gamut CIELAB (a* / b*)", "CIELAB Gamut Coverage (a* / b*)")} }
                p { class: "muted", style: "font-size: 11.5px; margin-bottom: 10px;",
                    if locale.is_fr() {
                        "Polygone chromatique couvert par les {pal.paints.len()} tubes de la palette et leurs coupes au blanc."
                    } else {
                        "Chromatic polygon spanned by the {pal.paints.len()} tubes in the palette and their white tints."
                    }
                }
                GamutMapSvg {
                    paints: pal.paints.clone(),
                    target_hex: None,
                    achieved_hex: None,
                    size: 250,
                }
            }
            div { class: "card",
                h3 { class: "card-title", {locale.tr("Mélanges intermédiaires de début de séance", "Session-Start Intermediate Mother Mixes")} }
                p { class: "muted", style: "font-size: 11.5px; margin-bottom: 10px;",
                    {locale.tr("Mélanges-mères (Sienne, Brun foncé, Noir coloré) calculés à partir de cette palette pour harmoniser les ombres.", "Mother mixes (Burnt Sienna, Dark Brown, Chromatic Black) computed from this palette to harmonize shadows.")}
                }
                for mm in &guide.mother_mixes {
                    div { class: "card-subtle", style: "margin-bottom: 8px;",
                        div { class: "row", style: "margin-bottom: 4px;",
                            ColorSwatch { hex: mm.hex_color.clone(), size: 24 }
                            span { class: "pill pill-accent mono", style: "font-weight: 800;", "{mm.code}" }
                            strong { "{mm.name}" }
                        }
                        div { class: "muted", style: "font-size: 12px;", "{mm.recipe_text}" }
                    }
                }
            }
        }
    }
}
