//! The tube catalog: search, the list of tubes and the sheet of the selected one.

use super::catalog_sheet::PaintSheet;
use super::catalog_tubes::{CustomTubeForm, RecognizedTubesCard, use_tube_form};
use crate::actions::catalog::{
    brands_of, closest_substitutes, filter_paints, pigment_codes, recognize_tubes, set_in_stock,
};
use crate::state::{AppServices, AppState};
use crate::theme::{PaintSwatch, SectionHeader};
use dioxus::prelude::*;
use palettier_ai::RecognizedTube;
use palettier_domain::i18n::Locale;
use palettier_domain::paint::{Paint, PaintSystem};

#[component]
pub fn CatalogView() -> Element {
    let state = use_context::<AppState>();

    let locale = *state.locale.read();

    let search = use_signal(String::new);
    let brand_filter = use_signal(|| "ALL".to_string());
    let stock_only = use_signal(|| false);
    let selected_paint_id = use_signal(|| None::<i64>);
    let calibrate_masstone_hex = use_signal(String::new);
    let calibrate_tint_hex = use_signal(String::new);
    let recognizing = use_signal(|| false);
    let recognized_tubes = use_signal(Vec::<RecognizedTube>::new);

    // Custom tube creation form state
    let show_add_form = use_signal(|| false);
    let tube_form = use_tube_form();

    let all_paints = state.paints_of_system();
    let owned_count = all_paints.iter().filter(|p| p.in_stock).count();
    let pipette_hex = state.last_sampled_hex.read().clone();

    let brands = brands_of(&all_paints);
    let filtered = filter_paints(
        &all_paints,
        &search.read(),
        &brand_filter.read(),
        *stock_only.read(),
    );

    let selected_paint = selected_paint_id
        .read()
        .and_then(|id| all_paints.iter().find(|p| p.id == Some(id)).cloned());

    let substitutes: Vec<(Paint, f64)> = match selected_paint {
        Some(ref target) => closest_substitutes(&all_paints, target, 6),
        None => Vec::new(),
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
                title: match *state.paint_system.read() {
                    PaintSystem::Oil => locale.tr("Catalogue des peintures à l'huile", "Oil Paint Catalog"),
                    PaintSystem::Acrylic => locale.tr("Catalogue des acryliques", "Acrylic Paint Catalog"),
                }
                .to_string(),
                subtitle: subtitle_str,
                actions: Some(rsx! {
                    CatalogActions { show_add_form, recognizing, recognized_tubes }
                }),
            }

            if *show_add_form.read() {
                CustomTubeForm { form: tube_form, show_add_form, selected_paint_id }
            }

            if !recognized_tubes.read().is_empty() {
                RecognizedTubesCard { recognized_tubes }
            }

            CatalogSearch {
                search,
                brand_filter,
                stock_only,
                brands,
                owned_count,
                shown: filtered.len(),
            }

            div { class: "grid-2", style: "grid-template-columns: 1.6fr 1fr;",
                PaintTable {
                    paints: filtered,
                    selected_paint_id,
                    calibrate_masstone_hex,
                    calibrate_tint_hex,
                }

                div { class: "card",
                    if let Some(paint) = selected_paint {
                        PaintSheet {
                            paint,
                            substitutes,
                            pipette_hex,
                            selected_paint_id,
                            calibrate_masstone_hex,
                            calibrate_tint_hex,
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

/// The header buttons: the custom tube form toggle and the photo recognition.
#[component]
fn CatalogActions(
    show_add_form: Signal<bool>,
    recognizing: Signal<bool>,
    recognized_tubes: Signal<Vec<RecognizedTube>>,
) -> Element {
    let services = use_context::<AppServices>();
    let mut state = use_context::<AppState>();
    let locale = *state.locale.read();

    let recognize = move |_| {
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
            match recognize_tubes(&services, &bytes, &catalog_snapshot).await {
                Ok(tubes) => {
                    let count = tubes.len();
                    recognized_tubes.set(tubes);
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
    };

    rsx! {
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
            onclick: recognize,
            if *recognizing.read() {
                {locale.tr("Analyse photo IA…", "AI photo analysis…")}
            } else {
                {locale.tr("Identifier mes tubes par photo (IA)", "Identify tubes from photo (AI)")}
            }
        }

    }
}

/// The search bar: text, brand and drawer filters.
#[component]
fn CatalogSearch(
    search: Signal<String>,
    brand_filter: Signal<String>,
    stock_only: Signal<bool>,
    brands: Vec<String>,
    owned_count: usize,
    shown: usize,
) -> Element {
    let state = use_context::<AppState>();
    let locale = *state.locale.read();

    rsx! {
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
                span { class: "pill", "{shown} ", {locale.tr("tube(s) affiché(s)", "tube(s) shown")} }
            }
        }
    }
}

/// The list of tubes (the first 250 matches).
#[component]
fn PaintTable(
    paints: Vec<Paint>,
    selected_paint_id: Signal<Option<i64>>,
    calibrate_masstone_hex: Signal<String>,
    calibrate_tint_hex: Signal<String>,
) -> Element {
    let state = use_context::<AppState>();
    let locale = *state.locale.read();

    rsx! {
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
                    for paint in paints.into_iter().take(250) {
                        PaintRow {
                            paint,
                            selected_paint_id,
                            calibrate_masstone_hex,
                            calibrate_tint_hex,
                        }
                    }
                }
            }

        }
    }
}

/// One tube of the list: clicking it selects it, its checkbox puts it in the drawer.
#[component]
fn PaintRow(
    paint: Paint,
    selected_paint_id: Signal<Option<i64>>,
    calibrate_masstone_hex: Signal<String>,
    calibrate_tint_hex: Signal<String>,
) -> Element {
    let services = use_context::<AppServices>();
    let mut state = use_context::<AppState>();
    let locale = *state.locale.read();

    let pid = paint.id;
    let is_selected = *selected_paint_id.read() == pid && pid.is_some();
    let pig_str = pigment_codes(&paint);
    let select = {
        let masstone = paint.hex_color.clone();
        let tint = paint.tint_hex.clone().unwrap_or_else(|| "#E8DFD5".to_string());
        move |_| {
            selected_paint_id.set(pid);
            calibrate_masstone_hex.set(masstone.clone());
            calibrate_tint_hex.set(tint.clone());
        }
    };
    let toggle_stock = move |id: i64, e: Event<FormData>| {
        let checked = e.checked();
        let services = services.clone();
        spawn(async move {
            if set_in_stock(&services, id, checked).await.is_ok() {
                state.reload_all(&services).await;
            }
        });
    };

    rsx! {
        tr {
            style: if is_selected { "background-color: var(--bg-active); cursor: pointer;" } else { "cursor: pointer;" },
            onclick: select,
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
                span { class: "pill", "{paint.opacity.label(locale)}" }
                " "
                span { class: "pill", "{paint.drying_class.label(locale)}" }
            }
            td {
                if let Some(id) = pid {
                    input {
                        r#type: "checkbox",
                        checked: paint.in_stock,
                        onclick: move |e| e.stop_propagation(),
                        onchange: move |e| toggle_stock(id, e),
                    }
                }
            }
        }
    }
}
