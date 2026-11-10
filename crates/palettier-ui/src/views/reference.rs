use crate::state::{AppServices, AppState};
use crate::theme::{ColorSwatch, DeltaEBadge, HexColorInput, PaintSwatch, SectionHeader};
use dioxus::prelude::*;
use palettier_ai::RecognizedTube;
use palettier_domain::color::{Rgb, delta_e_2000};
use palettier_domain::drying::{CustomMedium, DryingContext, Workshop};
use palettier_domain::i18n::Locale;
use palettier_domain::paint::{
    DryingClass, LayerThickness, Medium, Opacity, Paint, Technique, Ventilation,
};
use palettier_domain::port::{PaintCatalogRepository, RecipeRepository};
use palettier_domain::recipe::{Recipe, RecipeStep};
use std::collections::BTreeSet;
use std::time::Duration;

fn format_duration(d: Duration, locale: Locale) -> String {
    let total_mins = d.as_secs() / 60;
    if total_mins == 0 {
        return locale.tr("immédiat", "immediate").to_string();
    }
    let days = total_mins / (60 * 24);
    let hours = (total_mins % (60 * 24)) / 60;
    let mins = total_mins % 60;

    let (m_unit, d_unit) = match locale {
        Locale::Fr => ("mois", "j"),
        Locale::En => ("mo", "d"),
    };

    if days > 30 {
        let months = days / 30;
        let rem_days = days % 30;
        if rem_days > 0 {
            format!("{months} {m_unit} {rem_days} {d_unit}")
        } else {
            format!("{months} {m_unit}")
        }
    } else if days > 0 {
        if hours > 0 {
            format!("{days} {d_unit} {hours} h")
        } else {
            format!("{days} {d_unit}")
        }
    } else if hours > 0 {
        if mins > 0 {
            format!("{hours} h {mins:02} min")
        } else {
            format!("{hours} h")
        }
    } else {
        format!("{mins} min")
    }
}

fn pigment_codes_str(paint: &Paint) -> String {
    paint.pigments.iter().cloned().collect::<Vec<_>>().join(", ")
}

#[component]
pub fn CatalogView() -> Element {
    let services = use_context::<AppServices>();
    let mut state = use_context::<AppState>();

    let locale = *state.locale.read();

    let mut search = use_signal(String::new);
    let mut brand_filter = use_signal(|| "ALL".to_string());
    let mut stock_only = use_signal(|| false);
    let mut selected_paint_id = use_signal(|| None::<i64>);
    let mut calibrate_masstone_hex = use_signal(String::new);
    let mut calibrate_tint_hex = use_signal(String::new);
    let mut recognizing = use_signal(|| false);
    let mut recognized_tubes = use_signal(Vec::<RecognizedTube>::new);

    // Custom tube creation form state
    let mut show_add_form = use_signal(|| false);
    let mut custom_brand = use_signal(|| "Studio".to_string());
    let mut custom_name = use_signal(String::new);
    let mut custom_code = use_signal(String::new);
    let mut custom_pigments = use_signal(String::new);
    let mut custom_hex = use_signal(|| "#A85638".to_string());
    let mut custom_opacity = use_signal(|| Opacity::SemiOpaque);
    let mut custom_drying = use_signal(|| DryingClass::Medium);

    let all_paints = state.paints.read().clone();
    let owned_count = all_paints.iter().filter(|p| p.in_stock).count();
    let pipette_hex = state.last_sampled_hex.read().clone();

    let brands: Vec<String> = {
        let mut set = BTreeSet::new();
        for p in &all_paints {
            set.insert(p.brand.clone());
        }
        set.into_iter().collect()
    };

    let q = search.read().trim().to_lowercase();
    let bf = brand_filter.read().clone();
    let so = *stock_only.read();

    let filtered: Vec<Paint> = all_paints
        .iter()
        .filter(|p| {
            if so && !p.in_stock {
                return false;
            }
            if bf != "ALL" && p.brand != bf {
                return false;
            }
            if !q.is_empty() {
                let hay = format!(
                    "{} {} {} {}",
                    p.brand,
                    p.name,
                    p.code,
                    pigment_codes_str(p)
                )
                .to_lowercase();
                if !hay.contains(&q) {
                    return false;
                }
            }
            true
        })
        .cloned()
        .collect();

    let selected_paint = selected_paint_id
        .read()
        .and_then(|id| all_paints.iter().find(|p| p.id == Some(id)).cloned());

    let substitutes: Vec<(Paint, f64)> = if let Some(ref target) = selected_paint {
        let mut list: Vec<(Paint, f64)> = all_paints
            .iter()
            .filter(|p| p.id != target.id)
            .map(|p| (p.clone(), delta_e_2000(target.color(), p.color())))
            .collect();
        list.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        list.into_iter().take(6).collect()
    } else {
        Vec::new()
    };

    let subtitle_str = match locale {
        Locale::Fr => format!(
            "{} références issues de {} gammes · {} tubes dans votre tiroir.",
            all_paints.len(),
            brands.len(),
            owned_count
        ),
        Locale::En => format!(
            "{} references across {} lines · {} tubes in your drawer.",
            all_paints.len(),
            brands.len(),
            owned_count
        ),
    };

    rsx! {
        div {
            SectionHeader {
                title: locale.tr("Catalogue des peintures à l'huile", "Oil Paint Catalog").to_string(),
                subtitle: subtitle_str,
                actions: Some(rsx! {
                    button {
                        class: "btn",
                        onclick: move |_| {
                            let cur = *show_add_form.read();
                            show_add_form.set(!cur);
                        },
                        if *show_add_form.read() {
                            {locale.tr("Fermer l'ajout de tube", "Close custom tube form")}
                        } else {
                            {locale.tr("+ Nouveau tube personnalisé", "+ New custom tube")}
                        }
                    }
                    button {
                        class: "btn-primary",
                        disabled: *recognizing.read(),
                        onclick: {
                            let services = services.clone();
                            move |_| {
                                let services = services.clone();
                                spawn(async move {
                                    let loc = *state.locale.read();
                                    let file = rfd::AsyncFileDialog::new()
                                        .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
                                        .pick_file()
                                        .await;
                                    let Some(handle) = file else { return };
                                    recognizing.set(true);
                                    let bytes = handle.read().await;
                                    let catalog_snapshot = state.paints.read().clone();
                                    match services.recognition_service.recognize(&bytes, &catalog_snapshot).await {
                                        Ok(res) => {
                                            let count = res.tubes.len();
                                            recognized_tubes.set(res.tubes);
                                            let msg = match loc {
                                                Locale::Fr => format!("{count} tube(s) identifié(s) sur la photo."),
                                                Locale::En => format!("{count} tube(s) identified from photo."),
                                            };
                                            state.notify(msg);
                                        }
                                        Err(e) => state.notify(format!("AI: {e}")),
                                    }
                                    recognizing.set(false);
                                });
                            }
                        },
                        if *recognizing.read() {
                            {locale.tr("Analyse photo IA…", "AI photo analysis…")}
                        } else {
                            {locale.tr("Identifier mes tubes par photo (IA)", "Identify tubes from photo (AI)")}
                        }
                    }

                }),
            }

            if *show_add_form.read() {
                div { class: "card", style: "border-color: var(--accent);",
                    h3 { class: "card-title", {locale.tr("Ajouter un tube personnalisé au catalogue", "Add a custom tube to the catalog")} }
                    div { class: "grid-3", style: "margin-bottom: 12px;",
                        div {
                            label { class: "field-label", {locale.tr("Marque / Gamme", "Brand / Line")} }
                            input {
                                r#type: "text",
                                style: "width: 100%;",
                                value: "{custom_brand}",
                                oninput: move |e| custom_brand.set(e.value()),
                            }
                        }
                        div {
                            label { class: "field-label", {locale.tr("Nom de la couleur", "Color Name")} }
                            input {
                                r#type: "text",
                                style: "width: 100%;",
                                placeholder: locale.tr("ex. Ocre d'or maison", "e.g. Custom Gold Ochre"),
                                value: "{custom_name}",
                                oninput: move |e| custom_name.set(e.value()),
                            }
                        }
                        div {
                            label { class: "field-label", {locale.tr("Code / Référence", "Code / Reference")} }
                            input {
                                r#type: "text",
                                style: "width: 100%;",
                                placeholder: "042",
                                value: "{custom_code}",
                                oninput: move |e| custom_code.set(e.value()),
                            }
                        }
                    }
                    div { class: "grid-3", style: "margin-bottom: 12px;",
                        div {
                            label { class: "field-label", {locale.tr("Pigments C.I. (séparés par virgule)", "C.I. Pigments (comma-separated)")} }
                            input {
                                r#type: "text",
                                style: "width: 100%;",
                                placeholder: "PY42, PR101",
                                value: "{custom_pigments}",
                                oninput: move |e| custom_pigments.set(e.value()),
                            }
                        }
                        div {
                            label { class: "field-label", {locale.tr("Opacité", "Opacity")} }
                            select {
                                style: "width: 100%;",
                                onchange: move |e| {
                                    let op = match e.value().as_str() {
                                        "Transparent" => Opacity::Transparent,
                                        "SemiTransparent" => Opacity::SemiTransparent,
                                        "Opaque" => Opacity::Opaque,
                                        _ => Opacity::SemiOpaque,
                                    };
                                    custom_opacity.set(op);
                                },
                                for op in Opacity::ALL {
                                    option {
                                        value: "{op:?}",
                                        selected: *custom_opacity.read() == op,
                                        "{op.label_i18n(locale)}"
                                    }
                                }
                            }
                        }
                        div {
                            label { class: "field-label", {locale.tr("Classe siccative", "Drying Class")} }
                            select {
                                style: "width: 100%;",
                                onchange: move |e| {
                                    let dc = match e.value().as_str() {
                                        "Fast" => DryingClass::Fast,
                                        "Slow" => DryingClass::Slow,
                                        "VerySlow" => DryingClass::VerySlow,
                                        _ => DryingClass::Medium,
                                    };
                                    custom_drying.set(dc);
                                },
                                for dc in DryingClass::ALL {
                                    option {
                                        value: "{dc:?}",
                                        selected: *custom_drying.read() == dc,
                                        "{dc.label_i18n(locale)}"
                                    }
                                }
                            }
                        }
                    }
                    div { class: "row-between",
                        div { class: "row",
                            label { class: "field-label", style: "margin: 0;", {locale.tr("Teinte de sortie de tube :", "Masstone color:")} }
                            HexColorInput {
                                value: custom_hex(),
                                onchange: move |v: String| custom_hex.set(v),
                            }
                        }
                        button {
                            class: "btn-primary",
                            onclick: {
                                let services = services.clone();
                                move |_| {
                                    let loc = *state.locale.read();
                                    let b = custom_brand().trim().to_string();
                                    let n = custom_name().trim().to_string();
                                    if b.is_empty() || n.is_empty() {
                                        state.notify(loc.tr("Indiquez au moins une marque et un nom de couleur.", "Please enter both a brand and a color name."));
                                        return;
                                    }
                                    let mut paint = Paint::new(b, n, custom_hex().trim());
                                    paint.code = custom_code().trim().to_string();
                                    paint.pigments = custom_pigments()
                                        .split(',')
                                        .map(|s| s.trim().to_uppercase())
                                        .filter(|s| !s.is_empty())
                                        .collect();
                                    paint.opacity = *custom_opacity.read();
                                    paint.drying_class = *custom_drying.read();
                                    paint.user_added = true;
                                    paint.in_stock = true;
                                    let services = services.clone();
                                    spawn(async move {
                                        match services.catalog.save(&paint).await {
                                            Ok(saved) => {
                                                state.reload_all(&services).await;
                                                selected_paint_id.set(saved.id);
                                                show_add_form.set(false);
                                                custom_name.set(String::new());
                                                custom_code.set(String::new());
                                                let msg = match loc {
                                                    Locale::Fr => format!("Tube « {} » ajouté au catalogue et au tiroir.", saved.display_name()),
                                                    Locale::En => format!("Tube '{}' added to catalog and drawer.", saved.display_name()),
                                                };
                                                state.notify(msg);
                                            }
                                            Err(e) => state.notify(format!("Error: {e}")),
                                        }
                                    });
                                }
                            },
                            {locale.tr("Enregistrer ce tube", "Save tube")}
                        }
                    }

                }
            }

            if !recognized_tubes.read().is_empty() {
                div { class: "card",
                    div { class: "row-between",
                        h3 { class: "card-title", {locale.tr("Tubes reconnus sur la photo", "Tubes recognized from photo")} }
                        button {
                            class: "btn",
                            onclick: move |_| recognized_tubes.set(Vec::new()),
                            {locale.tr("Fermer", "Close")}
                        }
                    }
                    div { class: "grid-2",
                        for tube in recognized_tubes.read().clone() {
                            div { class: "card-subtle row-between",
                                div {
                                    div { style: "font-weight: 600;", "{tube.read_brand} — {tube.read_name}" }
                                    if !tube.read_code.is_empty() {
                                        div { class: "muted mono", {locale.tr("Code lu : ", "Read code: ")}, "{tube.read_code}" }
                                    }
                                    if let Some(ref matched) = tube.matched {
                                        div { class: "row", style: "margin-top: 4px;",
                                            ColorSwatch { hex: matched.hex_color.clone(), size: 18 }
                                            span { class: "pill pill-ok", {locale.tr("Correspondance : ", "Match: ")}, "{matched.display_name()}" }
                                        }
                                    } else {
                                        span { class: "pill pill-warn", {locale.tr("Hors catalogue", "Not in catalog")} }
                                    }
                                }
                                if let Some(matched) = tube.matched.clone() {
                                    if let Some(pid) = matched.id {
                                        button {
                                            class: "btn-primary",
                                            onclick: {
                                                let services = services.clone();
                                                move |_| {
                                                    let services = services.clone();
                                                    spawn(async move {
                                                        let loc = *state.locale.read();
                                                        if services.catalog.set_owned(pid, true).await.is_ok() {
                                                            state.reload_all(&services).await;
                                                            state.notify(loc.tr("Tube ajouté à votre tiroir.", "Tube added to your drawer."));
                                                        }
                                                    });
                                                }
                                            },
                                            {locale.tr("Ajouter au tiroir", "Add to drawer")}
                                        }
                                    }
                                }
                            }
                        }
                    }

                }
            }

            div { class: "card",
                div { class: "row-between",
                    div { class: "row", style: "flex: 1;",
                        input {
                            r#type: "text",
                            style: "min-width: 280px;",
                            placeholder: locale.tr("Rechercher par nom, marque, code ou pigment (ex: PB29, PR108, Zorn)…", "Search by name, brand, code, or pigment (e.g. PB29, PR108, Zorn)…"),
                            value: "{search}",
                            oninput: move |e| search.set(e.value()),
                        }
                        select {
                            value: "{brand_filter}",
                            onchange: move |e| brand_filter.set(e.value()),
                            option { value: "ALL", {locale.tr("Toutes les marques", "All brands")}, " ({brands.len()})" }
                            for b in &brands {
                                option { value: "{b}", "{b}" }
                            }
                        }
                        label { class: "row", style: "cursor: pointer;",
                            input {
                                r#type: "checkbox",
                                checked: *stock_only.read(),
                                onchange: move |e| stock_only.set(e.checked()),
                            }
                            span { {locale.tr("Mon tiroir uniquement", "My drawer only")}, " ({owned_count})" }
                        }
                    }
                    span { class: "pill", "{filtered.len()} ", {locale.tr("tube(s) affiché(s)", "tube(s) shown")} }
                }
            }

            div { class: "grid-2", style: "grid-template-columns: 1.6fr 1fr;",
                div { class: "card", style: "max-height: 68vh; overflow-y: auto;",
                    table { class: "data-table",
                        thead {
                            tr {
                                th { {locale.tr("Masstone / Coupe", "Masstone / Tint")} }
                                th { {locale.tr("Marque & Nom", "Brand & Name")} }
                                th { "Pigments" }
                                th { {locale.tr("Opacité / Séchage", "Opacity / Drying")} }
                                th { {locale.tr("Tiroir", "Drawer")} }
                            }
                        }
                        tbody {
                            for paint in filtered.iter().take(250) {
                                {
                                    let pid = paint.id;
                                    let is_selected = *selected_paint_id.read() == pid && pid.is_some();
                                    let p_for_select = paint.clone();
                                    let pig_str = pigment_codes_str(paint);
                                    rsx! {
                                        tr {
                                            style: if is_selected { "background-color: var(--bg-active); cursor: pointer;" } else { "cursor: pointer;" },
                                            onclick: move |_| {
                                                selected_paint_id.set(pid);
                                                calibrate_masstone_hex.set(p_for_select.hex_color.clone());
                                                calibrate_tint_hex.set(
                                                    p_for_select
                                                        .tint_hex
                                                        .clone()
                                                        .unwrap_or_else(|| "#E8DFD5".to_string()),
                                                );
                                            },
                                            td {
                                                PaintSwatch {
                                                    masstone_hex: paint.hex_color.clone(),
                                                    tint_hex: paint.tint_hex.clone(),
                                                    width: 38,
                                                    height: 24,
                                                }
                                            }
                                            td {
                                                div { style: "font-weight: 600;", "{paint.name}" }
                                                div { class: "muted", style: "font-size: 11.5px;",
                                                    "{paint.brand}"
                                                    if !paint.code.is_empty() {
                                                        " · #{paint.code}"
                                                    }
                                                }
                                            }
                                            td { class: "mono muted", "{pig_str}" }
                                            td {
                                                span { class: "pill", "{paint.opacity.label_i18n(locale)}" }
                                                " "
                                                span { class: "pill", "{paint.drying_class.label_i18n(locale)}" }
                                            }
                                            td {
                                                if let Some(id) = pid {
                                                    input {
                                                        r#type: "checkbox",
                                                        checked: paint.in_stock,
                                                        onclick: move |e| e.stop_propagation(),
                                                        onchange: {
                                                            let services = services.clone();
                                                            move |e: Event<FormData>| {
                                                                let checked = e.checked();
                                                                let services = services.clone();
                                                                spawn(async move {
                                                                    if services.catalog.set_owned(id, checked).await.is_ok() {
                                                                        state.reload_all(&services).await;
                                                                    }
                                                                });
                                                            }
                                                        },
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

                div { class: "card",
                    if let Some(paint) = selected_paint {
                        {
                            let pig_str = pigment_codes_str(&paint);
                            rsx! {
                                div {
                                    div { class: "row-between", style: "margin-bottom: 12px;",
                                        div { class: "row",
                                            PaintSwatch {
                                                masstone_hex: paint.hex_color.clone(),
                                                tint_hex: paint.tint_hex.clone(),
                                                width: 56,
                                                height: 40,
                                            }
                                            div {
                                                h3 { class: "card-title", style: "margin-bottom: 2px;", "{paint.name}" }
                                                div { class: "muted", "{paint.brand} · #{paint.code} · {paint.hex_color}" }
                                            }
                                        }
                                        if paint.user_added {
                                            if let Some(pid) = paint.id {
                                                button {
                                                    class: "btn-danger",
                                                    onclick: {
                                                        let services = services.clone();
                                                        move |_| {
                                                            let services = services.clone();
                                                            spawn(async move {
                                                                let loc = *state.locale.read();
                                                                if services.catalog.delete(pid).await.is_ok() {
                                                                    selected_paint_id.set(None);
                                                                    state.reload_all(&services).await;
                                                                    state.notify(loc.tr("Tube personnalisé supprimé.", "Custom tube deleted."));
                                                                }
                                                            });
                                                        }
                                                    },
                                                    {locale.tr("Supprimer", "Delete")}
                                                }
                                            }
                                        }
                                    }

                                    div { class: "row", style: "margin-bottom: 14px; flex-wrap: wrap; gap: 6px;",
                                        span { class: "pill pill-accent", "Pigments: {pig_str}" }
                                        span { class: "pill", {locale.tr("Opacité : ", "Opacity: ")}, "{paint.opacity.label_i18n(locale)}" }
                                        span { class: "pill", {locale.tr("Séchage : ", "Drying: ")}, "{paint.drying_class.label_i18n(locale)}" }
                                        span { class: "pill", "{paint.lightfastness().label_i18n(locale)}" }
                                        {
                                            let safety = paint.safety_info();
                                            rsx! {
                                                span {
                                                    class: "pill {safety.level.css_class()}",
                                                    title: "{safety.advice_i18n(locale)}",
                                                    "🛡️ {safety.level.badge_label(locale)}"
                                                }
                                            }
                                        }
                                    }
                                    {
                                        let safety = paint.safety_info();
                                        if !safety.hazards.is_empty() {
                                            rsx! {
                                                div { class: "card-subtle", style: "margin-bottom: 12px; border-color: var(--pill-warn-border); padding: 8px 12px;",
                                                    div { style: "font-weight: 600; font-size: 12px; color: var(--pill-warn-text);",
                                                        "⚠️ "
                                                        {locale.tr("Précautions atelier : ", "Studio precautions: ")}
                                                        "{safety.hazards.join(\", \")}"
                                                    }
                                                    div { class: "muted", style: "font-size: 11.5px; margin-top: 3px;",
                                                        "{safety.advice_i18n(locale)}"
                                                    }
                                                }
                                            }
                                        } else {
                                            rsx! {}
                                        }
                                    }

                                    div { class: "card-subtle", style: "margin-bottom: 14px;",
                                        h4 { style: "font-size: 13px; margin-bottom: 8px;", {locale.tr("Calibration optique du tube", "Optical Tube Calibration")} }
                                        p { class: "muted", style: "font-size: 12px; margin-bottom: 10px;",
                                            {locale.tr("Relevez la sortie de tube (masstone) et la coupe 1:10 au blanc de titane pour affiner la courbe de Kubelka-Munk.", "Record the pure masstone and 1:10 Titanium White tint to refine the Kubelka-Munk scattering curve.")}
                                        }
                                        div { class: "grid-2", style: "margin-bottom: 10px;",
                                            div {
                                                label { class: "field-label", {locale.tr("Sortie de tube (Masstone)", "Pure Tube (Masstone)")} }
                                                HexColorInput {
                                                    value: calibrate_masstone_hex(),
                                                    onchange: move |v: String| calibrate_masstone_hex.set(v),
                                                }
                                                if let Some(ref p_hex) = pipette_hex {
                                                    button {
                                                        class: "btn",
                                                        style: "margin-top: 6px; font-size: 11px; padding: 3px 8px;",
                                                        onclick: {
                                                            let h = p_hex.clone();
                                                            move |_| calibrate_masstone_hex.set(h.clone())
                                                        },
                                                        "← ", {locale.tr("Pipette", "Eyedropper")}, " ({p_hex})"
                                                    }
                                                }
                                            }
                                            div {
                                                label { class: "field-label", {locale.tr("Coupe 1:10 au blanc", "1:10 White Tint")} }
                                                HexColorInput {
                                                    value: calibrate_tint_hex(),
                                                    onchange: move |v: String| calibrate_tint_hex.set(v),
                                                }
                                                if let Some(ref p_hex) = pipette_hex {
                                                    button {
                                                        class: "btn",
                                                        style: "margin-top: 6px; font-size: 11px; padding: 3px 8px;",
                                                        onclick: {
                                                            let h = p_hex.clone();
                                                            move |_| calibrate_tint_hex.set(h.clone())
                                                        },
                                                        "← ", {locale.tr("Pipette", "Eyedropper")}, " ({p_hex})"
                                                    }
                                                }
                                            }
                                        }
                                        if let Some(pid) = paint.id {
                                            div { class: "row",
                                                button {
                                                    class: "btn-primary",
                                                    onclick: {
                                                        let services = services.clone();
                                                        move |_| {
                                                            let m_hex = calibrate_masstone_hex.read().clone();
                                                            let t_hex = calibrate_tint_hex.read().clone();
                                                            let services = services.clone();
                                                            spawn(async move {
                                                                let loc = *state.locale.read();
                                                                if let Ok(rgb) = Rgb::of_hex(&m_hex) {
                                                                    let _ = services.catalog.record_masstone(pid, rgb).await;
                                                                }
                                                                if let Ok(rgb) = Rgb::of_hex(&t_hex) {
                                                                    let _ = services.catalog.record_tint(pid, rgb).await;
                                                                }
                                                                state.reload_all(&services).await;
                                                                state.notify(loc.tr("Calibration enregistrée pour ce tube.", "Calibration saved for this tube."));
                                                            });
                                                        }
                                                    },
                                                    {locale.tr("Enregistrer la calibration", "Save calibration")}
                                                }
                                            }
                                        }
                                    }

                                    h4 { style: "font-size: 13px; margin-bottom: 8px;", {locale.tr("Équivalents et substituts proches", "Closest Equivalents & Substitutes")} }
                                    div {
                                        for (sub, de) in substitutes {
                                            {
                                                let sub_pigs = pigment_codes_str(&sub);
                                                rsx! {
                                                    div { class: "row-between", style: "padding: 6px 0; border-bottom: 1px solid var(--border-subtle);",
                                                        div { class: "row",
                                                            PaintSwatch {
                                                                masstone_hex: sub.hex_color.clone(),
                                                                tint_hex: sub.tint_hex.clone(),
                                                                width: 28,
                                                                height: 18,
                                                            }
                                                            div {
                                                                div { style: "font-size: 12.5px; font-weight: 600;", "{sub.name}" }
                                                                div { class: "muted", style: "font-size: 11px;",
                                                                    "{sub.brand} · {sub_pigs}"
                                                                    if sub.in_stock {
                                                                        span { " · (", {locale.tr("En stock", "In stock")}, ")" }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                        DeltaEBadge { delta_e: de }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        p { class: "muted",
                            {locale.tr("Sélectionnez un tube dans la liste pour afficher sa fiche pigmentaire, calibrer sa teinte ou chercher des équivalents.", "Select a tube from the list to view its pigment sheet, calibrate its masstone/tint, or find substitutes.")}
                        }
                    }

                }
            }
        }
    }
}

#[component]
pub fn DryingView() -> Element {
    let services = use_context::<AppServices>();
    let mut state = use_context::<AppState>();

    let locale = *state.locale.read();

    let mut drying_class = use_signal(|| DryingClass::Medium);
    let mut medium = use_signal(|| Medium::None);
    let mut medium_ratio_pct = use_signal(|| 0.0_f64);
    let mut thickness = use_signal(|| LayerThickness::Normal);

    let mut custom_oil = use_signal(|| 33u32);
    let mut custom_solvent = use_signal(|| 33u32);
    let mut custom_resin = use_signal(|| 34u32);
    let mut custom_driers = use_signal(|| 0u32);

    let formulated_medium = CustomMedium::new(
        "Medium Atelier",
        *custom_oil.read() as f64,
        *custom_solvent.read() as f64,
        *custom_resin.read() as f64,
        *custom_driers.read(),
    );

    let ws = *state.workshop.read();
    let ratio = (*medium_ratio_pct.read() / 100.0).clamp(0.0, 1.0);
    let context = DryingContext {
        drying_class: *drying_class.read(),
        medium: *medium.read(),
        medium_ratio: ratio,
        thickness: *thickness.read(),
        workshop: ws,
    };
    let estimate = services.drying_service.estimate(&context);
    let fatness = medium.read().fatness(ratio);

    rsx! {
        div {
            SectionHeader {
                title: locale.tr("Calculateur de séchage à l'huile", "Oil Drying Calculator").to_string(),
                subtitle: locale.tr(
                    "Estimez le temps ouvert, le sec au toucher, le recouvrable et le séchage à cœur selon vos pigments, votre médium et le climat de l'atelier.",
                    "Estimate open time, touch dry, recoat window, and full cure based on your pigments, medium, and studio climate."
                ).to_string(),
                actions: None,
            }

            div { class: "grid-2",
                div { class: "card",
                    h3 { class: "card-title", {locale.tr("Paramètres de la couche", "Coat Parameters")} }

                    div { style: "margin-bottom: 12px;",
                        label { class: "field-label", {locale.tr("Classe siccative du pigment", "Pigment Drying Class")} }
                        select {
                            style: "width: 100%;",
                            onchange: move |e| {
                                let v = match e.value().as_str() {
                                    "Fast" => DryingClass::Fast,
                                    "Slow" => DryingClass::Slow,
                                    "VerySlow" => DryingClass::VerySlow,
                                    _ => DryingClass::Medium,
                                };
                                drying_class.set(v);
                            },
                            for dc in DryingClass::ALL {
                                option {
                                    value: "{dc:?}",
                                    selected: *drying_class.read() == dc,
                                    "{dc.label_i18n(locale)} — {dc.typical_pigments_i18n(locale)}"
                                }
                            }
                        }
                    }

                    div { style: "margin-bottom: 12px;",
                        label { class: "field-label", {locale.tr("Médium ou diluant ajouté", "Added Medium or Thinner")} }
                        select {
                            style: "width: 100%;",
                            onchange: move |e| {
                                let m = match e.value().as_str() {
                                    "OdorlessThinner" => Medium::OdorlessThinner,
                                    "WhiteSpirit" => Medium::WhiteSpirit,
                                    "Turpentine" => Medium::Turpentine,
                                    "AlkydMedium" => Medium::AlkydMedium,
                                    "LinseedOil" => Medium::LinseedOil,
                                    "WalnutOil" => Medium::WalnutOil,
                                    "StandOil" => Medium::StandOil,
                                    "CobaltDrier" => Medium::CobaltDrier,
                                    _ => Medium::None,
                                };
                                medium.set(m);
                            },
                            for m in Medium::ALL {
                                option {
                                    value: "{m:?}",
                                    selected: *medium.read() == m,
                                    "{m.label_i18n(locale)}"
                                }
                            }
                        }
                    }

                    div { style: "margin-bottom: 12px;",
                        label {
                            class: "field-label",
                            if locale.is_fr() {
                                "Proportion de médium : {medium_ratio_pct.read():.0} % (indice gras : {fatness:.2})"
                            } else {
                                "Medium ratio: {medium_ratio_pct.read():.0}% (fatness index: {fatness:.2})"
                            }
                        }
                        input {
                            r#type: "range",
                            style: "width: 100%;",
                            min: "0",
                            max: "95",
                            step: "5",
                            value: "{medium_ratio_pct}",
                            oninput: move |e| {
                                if let Ok(v) = e.value().parse::<f64>() {
                                    medium_ratio_pct.set(v);
                                }
                            },
                        }
                    }

                    div { style: "margin-bottom: 16px;",
                        label { class: "field-label", {locale.tr("Épaisseur de la touche", "Coat Thickness")} }
                        div { class: "row",
                            for th in LayerThickness::ALL {
                                button {
                                    class: if *thickness.read() == th { "btn-primary" } else { "btn" },
                                    onclick: move |_| thickness.set(th),
                                    "{th.label_i18n(locale)}"
                                }
                            }
                        }
                    }

                    h3 { class: "card-title", style: "margin-top: 18px;", {locale.tr("Conditions de l'atelier", "Studio Climate")} }
                    div { class: "grid-3",
                        div {
                            label { class: "field-label", {locale.tr("Température (°C)", "Temperature (°C)")} }
                            input {
                                r#type: "number",
                                style: "width: 100%;",
                                value: "{ws.temperature_celsius}",
                                oninput: move |e| {
                                    if let Ok(t) = e.value().parse::<f64>()
                                        && let Ok(next) = Workshop::new(t, ws.relative_humidity, ws.ventilation) {
                                            state.workshop.set(next);
                                        }
                                },
                            }
                        }
                        div {
                            label { class: "field-label", {locale.tr("Humidité relative (%)", "Relative Humidity (%)")} }
                            input {
                                r#type: "number",
                                style: "width: 100%;",
                                value: "{ws.relative_humidity}",
                                oninput: move |e| {
                                    if let Ok(h) = e.value().parse::<f64>()
                                        && let Ok(next) = Workshop::new(ws.temperature_celsius, h, ws.ventilation) {
                                            state.workshop.set(next);
                                        }
                                },
                            }
                        }
                        div {
                            label { class: "field-label", "Ventilation" }
                            select {
                                style: "width: 100%;",
                                onchange: move |e| {
                                    let v = match e.value().as_str() {
                                        "Confined" => Ventilation::Confined,
                                        "Good" => Ventilation::Good,
                                        _ => Ventilation::Normal,
                                    };
                                    state.workshop.set(Workshop {
                                        ventilation: v,
                                        ..ws
                                    });
                                },
                                for v in Ventilation::ALL {
                                    option {
                                        value: "{v:?}",
                                        selected: ws.ventilation == v,
                                        "{v.label_i18n(locale)}"
                                    }
                                }
                            }
                        }
                    }

                    // Custom Medium Formulation Card
                    div { class: "card-subtle", style: "margin-top: 18px; border-color: var(--accent);",
                        h4 { style: "margin-bottom: 8px;",
                            "🧪 "
                            {locale.tr("Formulateur de Médium Personnalisé", "Custom Medium Formulator")}
                        }
                        div { class: "muted", style: "font-size: 11.5px; margin-bottom: 10px;",
                            {locale.tr(
                                "Composez votre propre mélange de liants et solvants. L'indice gras et le multiplicateur de séchage s'ajustent en temps réel.",
                                "Blend your own ratio of oils, thinners, and resins. The fat index and drying multiplier update in real time."
                            )}
                        }
                        div { class: "row", style: "gap: 6px; margin-bottom: 12px;",
                            button {
                                class: "btn",
                                style: "font-size: 11px; padding: 3px 8px;",
                                onclick: move |_| {
                                    custom_oil.set(33);
                                    custom_solvent.set(33);
                                    custom_resin.set(34);
                                    custom_driers.set(0);
                                },
                                "1/3-1/3-1/3 "
                                {locale.tr("Flamand", "Flemish")}
                            }
                            button {
                                class: "btn",
                                style: "font-size: 11px; padding: 3px 8px;",
                                onclick: move |_| {
                                    custom_oil.set(20);
                                    custom_solvent.set(80);
                                    custom_resin.set(0);
                                    custom_driers.set(0);
                                },
                                {locale.tr("Lavis maigre (20/80)", "Lean wash (20/80)")}
                            }
                            button {
                                class: "btn",
                                style: "font-size: 11px; padding: 3px 8px;",
                                onclick: move |_| {
                                    custom_oil.set(50);
                                    custom_solvent.set(25);
                                    custom_resin.set(25);
                                    custom_driers.set(1);
                                },
                                {locale.tr("Glacis gras (50/25/25)", "Fat glaze (50/25/25)")}
                            }
                        }
                        div { class: "grid-3", style: "margin-bottom: 8px;",
                            div {
                                label { class: "field-label", {locale.tr("Huile (%)", "Oil (%)")} }
                                input {
                                    r#type: "number",
                                    style: "width: 100%;",
                                    min: "0",
                                    max: "100",
                                    value: "{custom_oil}",
                                    oninput: move |e| {
                                        if let Ok(v) = e.value().parse::<u32>() {
                                            custom_oil.set(v.min(100));
                                        }
                                    }
                                }
                            }
                            div {
                                label { class: "field-label", {locale.tr("Solvant (%)", "Thinner (%)")} }
                                input {
                                    r#type: "number",
                                    style: "width: 100%;",
                                    min: "0",
                                    max: "100",
                                    value: "{custom_solvent}",
                                    oninput: move |e| {
                                        if let Ok(v) = e.value().parse::<u32>() {
                                            custom_solvent.set(v.min(100));
                                        }
                                    }
                                }
                            }
                            div {
                                label { class: "field-label", {locale.tr("Résine (%)", "Resin (%)")} }
                                input {
                                    r#type: "number",
                                    style: "width: 100%;",
                                    min: "0",
                                    max: "100",
                                    value: "{custom_resin}",
                                    oninput: move |e| {
                                        if let Ok(v) = e.value().parse::<u32>() {
                                            custom_resin.set(v.min(100));
                                        }
                                    }
                                }
                            }
                        }
                        div { class: "row-between", style: "margin-top: 10px; padding-top: 8px; border-top: 1px solid var(--border-subtle);",
                            div { class: "row",
                                span { class: "pill pill-accent",
                                    {locale.tr("Indice gras calculé", "Calculated fat index")},
                                    ": {(formulated_medium.fat_index() * 100.0).round() as u32}%"
                                }
                                span { class: "pill",
                                    {locale.tr("Facteur séchage", "Drying speed factor")},
                                    ": {formulated_medium.drying_multiplier():.2}×"
                                }
                            }
                        }
                    }
                }

                div { class: "card",
                    h3 { class: "card-title", {locale.tr("Jalons de séchage estimés", "Estimated Drying Milestones")} }
                    div { class: "grid-2", style: "margin-bottom: 16px;",
                        div { class: "card-subtle",
                            div { class: "muted", {locale.tr("Temps ouvert (fondu dans le frais)", "Open time (wet-in-wet blending)")} }
                            div { style: "font-size: 22px; font-weight: 700; color: var(--accent);",
                                "{format_duration(estimate.open_time, locale)}"
                            }
                        }
                        div { class: "card-subtle",
                            div { class: "muted", {locale.tr("Sec au toucher (hors poussière)", "Touch dry (dust-free)")} }
                            div { style: "font-size: 22px; font-weight: 700;",
                                "{format_duration(estimate.touch_dry, locale)}"
                            }
                        }
                        div { class: "card-subtle",
                            div { class: "muted", {locale.tr("Recouvrable (glacis ou couche suivante)", "Recoatable (glaze or next coat)")} }
                            div { style: "font-size: 22px; font-weight: 700; color: var(--pill-ok-text);",
                                "{format_duration(estimate.recoat, locale)}"
                            }
                        }
                        div { class: "card-subtle",
                            div { class: "muted", {locale.tr("Sec à cœur (empâtement / brossage)", "Through dry (impasto / drybrush)")} }
                            div { style: "font-size: 22px; font-weight: 700;",
                                "{format_duration(estimate.through_dry, locale)}"
                            }
                        }
                    }
                    div { class: "card-subtle", style: "margin-bottom: 16px;",
                        div { class: "row-between",
                            span { class: "muted", {locale.tr("Polymérisation complète avant vernis définitif", "Full polymerization before final varnish")} }
                            span { class: "pill pill-accent", "{format_duration(estimate.full_cure, locale)}" }
                        }
                    }

                    h4 { style: "font-size: 13px; margin-bottom: 8px;", {locale.tr("Conseils d'atelier & précautions", "Studio Tips & Precautions")} }
                    div { class: "card-subtle", style: "margin-bottom: 8px; font-size: 12.5px;",
                        "• {medium.read().advice_i18n(locale)}"
                    }
                    for tip in &estimate.advice {
                        div { class: "card-subtle", style: "margin-bottom: 8px; font-size: 12.5px;",
                            "• {tip}"
                        }
                    }

                }
            }
        }
    }
}

#[component]
pub fn RecipesView() -> Element {
    let services = use_context::<AppServices>();
    let mut state = use_context::<AppState>();

    let locale = *state.locale.read();

    let mut selected_idx = use_signal(|| 0usize);
    let mut new_name = use_signal(String::new);
    let mut new_subject = use_signal(String::new);
    let mut step_technique = use_signal(|| Technique::Glaze);
    let mut step_mix = use_signal(String::new);
    let mut step_notes = use_signal(String::new);

    let recipes = state.recipes.read().clone();
    let current = recipes.get(*selected_idx.read()).cloned();
    let ws = *state.workshop.read();

    rsx! {
        div {
            SectionHeader {
                title: locale.tr("Recettes & séquences d'étapes", "Recipes & Step Sequences").to_string(),
                subtitle: locale.tr(
                    "Construisez vos enchaînements de couches à l'huile, vérifiez la règle du gras sur maigre et déroulez le calendrier jusqu'au vernis.",
                    "Build multi-step oil sequences, validate the fat-over-lean rule, and preview the timeline through final varnish."
                ).to_string(),
                actions: None,
            }

            div { class: "grid-2", style: "grid-template-columns: 1fr 2fr;",
                div {
                    div { class: "card",
                        h3 { class: "card-title", {locale.tr("Recettes enregistrées", "Saved Recipes")}, " ({recipes.len()})" }
                        for (idx, rec) in recipes.iter().enumerate() {
                            {
                                let is_sel = *selected_idx.read() == idx;
                                let rid = rec.id;
                                rsx! {
                                    div {
                                        class: "card-subtle row-between",
                                        style: if is_sel { "margin-bottom: 8px; border-color: var(--accent); cursor: pointer;" } else { "margin-bottom: 8px; cursor: pointer;" },
                                        onclick: move |_| selected_idx.set(idx),
                                        div {
                                            div { style: "font-weight: 600;", "{rec.name}" }
                                            div { class: "muted", style: "font-size: 11.5px;", "{rec.subject} · {rec.steps.len()} ", {locale.tr("étape(s)", "step(s)")} }
                                        }
                                        if let Some(id) = rid {
                                            button {
                                                class: "btn-danger",
                                                onclick: {
                                                    let services = services.clone();
                                                    move |e: Event<MouseData>| {
                                                        e.stop_propagation();
                                                        let services = services.clone();
                                                        spawn(async move {
                                                            let loc = *state.locale.read();
                                                            if services.recipes.delete(id).await.is_ok() {
                                                                selected_idx.set(0);
                                                                state.reload_all(&services).await;
                                                                state.notify(loc.tr("Recette supprimée.", "Recipe deleted."));
                                                            }
                                                        });
                                                    }
                                                },
                                                {locale.tr("Supprimer", "Delete")}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    div { class: "card",
                        h3 { class: "card-title", {locale.tr("Nouvelle recette", "New Recipe")} }
                        div { style: "margin-bottom: 8px;",
                            label { class: "field-label", {locale.tr("Nom de la recette", "Recipe Name")} }
                            input {
                                r#type: "text",
                                style: "width: 100%;",
                                placeholder: locale.tr("Ex: Carnation chaude en 4 étapes", "e.g. Warm Skin Tone in 4 Steps"),
                                value: "{new_name}",
                                oninput: move |e| new_name.set(e.value()),
                            }
                        }
                        div { style: "margin-bottom: 12px;",
                            label { class: "field-label", {locale.tr("Sujet / Matière", "Subject / Material")} }
                            input {
                                r#type: "text",
                                style: "width: 100%;",
                                placeholder: locale.tr("Ex: Visage, Cuir usé, Drapé rouge…", "e.g. Face, Worn Leather, Red Cloak…"),
                                value: "{new_subject}",
                                oninput: move |e| new_subject.set(e.value()),
                            }
                        }
                        button {
                            class: "btn-primary",
                            onclick: {
                                let services = services.clone();
                                move |_| {
                                    let name = new_name.read().trim().to_string();
                                    let subj = new_subject.read().trim().to_string();
                                    if name.is_empty() {
                                        return;
                                    }
                                    let services = services.clone();
                                    spawn(async move {
                                        let loc = *state.locale.read();
                                        let rec = Recipe::new(name, subj);
                                        if services.recipes.save(&rec).await.is_ok() {
                                            new_name.set(String::new());
                                            new_subject.set(String::new());
                                            state.reload_all(&services).await;
                                            state.notify(loc.tr("Recette créée.", "Recipe created."));
                                        }
                                    });
                                }
                            },
                            {locale.tr("Créer la recette", "Create recipe")}
                        }
                    }

                }

                div {
                    if let Some(recipe) = current {
                        {
                            let risks = services.fat_over_lean.inspect_recipe_i18n(&recipe, locale);
                            let timeline = services.recipe_timeline.plan(&recipe, DryingClass::Medium, ws);
                            rsx! {
                                div { class: "card",
                                    div { class: "row-between", style: "margin-bottom: 12px;",
                                        div {
                                            h3 { class: "card-title", style: "margin-bottom: 2px;", "{recipe.name}" }
                                            div { class: "muted", {locale.tr("Sujet : ", "Subject: ")}, "{recipe.subject}" }
                                        }
                                        div { class: "row",
                                            span { class: "pill pill-accent", {locale.tr("Durée active : ", "Active span: ")}, "{format_duration(timeline.total_active_span, locale)}" }
                                            span { class: "pill", {locale.tr("Vernis après : ", "Varnish after: ")}, "{format_duration(timeline.until_varnish, locale)}" }
                                        }
                                    }

                                    if risks.is_empty() {
                                        div { class: "card-subtle", style: "margin-bottom: 14px; border-color: var(--pill-ok-border);",
                                            span { class: "pill pill-ok", {locale.tr("Gras sur maigre respecté", "Fat over lean respected")} }
                                            span { style: "margin-left: 10px; font-size: 12.5px;",
                                                {locale.tr("L'enchaînement des médiums et épaisseurs ne présente pas de risque de gerçure.", "The sequence of mediums and coat thicknesses presents no cracking risk.")}
                                            }
                                        }
                                    } else {
                                        for risk in &risks {
                                            div { class: "card-subtle", style: "margin-bottom: 12px; border-color: var(--pill-danger-border);",
                                                div { style: "font-weight: 600; color: var(--danger);",
                                                    {locale.tr("Alerte gras sur maigre", "Fat over lean warning")}, " ({risk.under} → {risk.over})"
                                                }
                                                div { style: "font-size: 12.5px; margin-top: 4px;", "{risk.explanation}" }
                                            }
                                        }
                                    }

                                    table { class: "data-table",
                                        thead {
                                            tr {
                                                th { "#" }
                                                th { "Technique" }
                                                th { {locale.tr("Mélange", "Mix")} }
                                                th { {locale.tr("Médium & Épaisseur", "Medium & Thickness")} }
                                                th { {locale.tr("Départ", "Start")} }
                                                th { {locale.tr("Attente avant suite", "Wait before next")} }
                                            }
                                        }
                                        tbody {
                                            for entry in &timeline.entries {
                                                tr {
                                                    td { class: "mono", "{entry.position}" }
                                                    td {
                                                        span { class: "pill pill-accent", "{entry.step.technique.label_i18n(locale)}" }
                                                    }
                                                    td {
                                                        div { style: "font-weight: 600;", "{entry.step.paint_mix}" }
                                                        if !entry.step.notes.is_empty() {
                                                            div { class: "muted", style: "font-size: 11.5px;", "{entry.step.notes}" }
                                                        }
                                                    }
                                                    td {
                                                        "{entry.step.medium.label_i18n(locale)} ({entry.step.medium_ratio * 100.0:.0} %) · {entry.step.thickness.label_i18n(locale)}"
                                                    }
                                                    td { class: "mono", "T + {format_duration(entry.start_offset, locale)}" }
                                                    td { class: "mono", "{format_duration(entry.wait_after, locale)}" }
                                                }
                                            }
                                        }
                                    }

                                    div { class: "card-subtle", style: "margin-top: 16px;",
                                        h4 { style: "font-size: 13px; margin-bottom: 10px;", {locale.tr("Ajouter une étape à cette recette", "Add a step to this recipe")} }
                                        div { class: "grid-3", style: "margin-bottom: 10px;",
                                            div {
                                                label { class: "field-label", "Technique" }
                                                select {
                                                    style: "width: 100%;",
                                                    onchange: move |e| {
                                                        let t = Technique::by_label(Some(&e.value()), Technique::Glaze);
                                                        step_technique.set(t);
                                                    },
                                                    for t in Technique::ALL {
                                                        option {
                                                            value: "{t.label()}",
                                                            selected: *step_technique.read() == t,
                                                            "{t.label_i18n(locale)}"
                                                        }
                                                    }
                                                }
                                            }
                                            div {
                                                label { class: "field-label", {locale.tr("Mélange de tubes", "Tube Mix")} }
                                                input {
                                                    r#type: "text",
                                                    style: "width: 100%;",
                                                    placeholder: locale.tr("Ex: Ocre jaune + Blanc de titane", "e.g. Yellow Ochre + Titanium White"),
                                                    value: "{step_mix}",
                                                    oninput: move |e| step_mix.set(e.value()),
                                                }
                                            }
                                            div {
                                                label { class: "field-label", "Note" }
                                                input {
                                                    r#type: "text",
                                                    style: "width: 100%;",
                                                    placeholder: locale.tr("Ex: Tirer vers les tempes", "e.g. Feather toward the temples"),
                                                    value: "{step_notes}",
                                                    oninput: move |e| step_notes.set(e.value()),
                                                }
                                            }
                                        }
                                        button {
                                            class: "btn-primary",
                                            onclick: {
                                                let services = services.clone();
                                                let recipe_clone = recipe.clone();
                                                move |_| {
                                                    let mix = step_mix.read().trim().to_string();
                                                    if mix.is_empty() {
                                                        return;
                                                    }
                                                    let tech = *step_technique.read();
                                                    let notes = step_notes.read().trim().to_string();
                                                    let mut next_recipe = recipe_clone.clone();
                                                    let mut step = RecipeStep::of(tech, mix);
                                                    step.notes = notes;
                                                    next_recipe.steps.push(step);
                                                    let services = services.clone();
                                                    spawn(async move {
                                                        let loc = *state.locale.read();
                                                        if services.recipes.save(&next_recipe).await.is_ok() {
                                                            step_mix.set(String::new());
                                                            step_notes.set(String::new());
                                                            state.reload_all(&services).await;
                                                            state.notify(loc.tr("Étape ajoutée à la recette.", "Step added to recipe."));
                                                        }
                                                    });
                                                }
                                            },
                                            {locale.tr("Ajouter l'étape", "Add step")}
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        div { class: "card",
                            p { class: "muted", {locale.tr("Sélectionnez ou créez une recette pour visualiser sa chronologie de séchage.", "Select or create a recipe to view its drying timeline.")} }
                        }
                    }

                }
            }
        }
    }
}
