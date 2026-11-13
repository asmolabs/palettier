//! Adding tubes: the custom tube form and the tubes recognized on a photo.

use crate::actions::catalog::{NewTube, add_custom_tube, set_in_stock};
use crate::state::{AppServices, AppState};
use crate::theme::{ColorSwatch, HexColorInput};
use dioxus::prelude::*;
use palettier_ai::RecognizedTube;
use palettier_domain::i18n::Locale;
use palettier_domain::paint::{DryingClass, Opacity};

/// The fields of the custom tube form. They live in the catalog screen, so closing the
/// form keeps what was typed.
#[derive(Clone, Copy, PartialEq)]
pub(super) struct TubeForm {
    brand: Signal<String>,
    name: Signal<String>,
    code: Signal<String>,
    pigments: Signal<String>,
    hex: Signal<String>,
    opacity: Signal<Opacity>,
    drying: Signal<DryingClass>,
}

pub(super) fn use_tube_form() -> TubeForm {
    TubeForm {
        brand: use_signal(|| "Studio".to_string()),
        name: use_signal(String::new),
        code: use_signal(String::new),
        pigments: use_signal(String::new),
        hex: use_signal(|| "#A85638".to_string()),
        opacity: use_signal(|| Opacity::SemiOpaque),
        drying: use_signal(|| DryingClass::Medium),
    }
}

fn opacity_of(value: &str) -> Opacity {
    match value {
        "Transparent" => Opacity::Transparent,
        "SemiTransparent" => Opacity::SemiTransparent,
        "Opaque" => Opacity::Opaque,
        _ => Opacity::SemiOpaque,
    }
}

fn drying_class_of(value: &str) -> DryingClass {
    match value {
        "Fast" => DryingClass::Fast,
        "Slow" => DryingClass::Slow,
        "VerySlow" => DryingClass::VerySlow,
        _ => DryingClass::Medium,
    }
}

#[component]
pub(super) fn CustomTubeForm(
    form: TubeForm,
    show_add_form: Signal<bool>,
    selected_paint_id: Signal<Option<i64>>,
) -> Element {
    let services = use_context::<AppServices>();
    let mut state = use_context::<AppState>();
    let locale = *state.locale.read();
    let TubeForm {
        mut brand,
        mut name,
        mut code,
        mut pigments,
        mut hex,
        mut opacity,
        mut drying,
    } = form;

    let save = move |_| {
        let loc = *state.locale.read();
        let tube = NewTube {
            brand: brand(),
            name: name(),
            code: code(),
            pigments: pigments(),
            hex_color: hex(),
            opacity: *opacity.read(),
            drying_class: *drying.read(),
            system: *state.paint_system.read(),
        };
        let services = services.clone();
        spawn(async move {
            match add_custom_tube(&services, tube).await {
                Ok(None) => {
                    state.notify(loc.tr("Indiquez au moins une marque et un nom de couleur.", "Please enter both a brand and a color name."));
                }
                Ok(Some(saved)) => {
                    state.reload_all(&services).await;
                    selected_paint_id.set(saved.id);
                    show_add_form.set(false);
                    name.set(String::new());
                    code.set(String::new());
                    let msg = match loc {
                        Locale::Fr => format!("Tube « {} » ajouté au catalogue et au tiroir.", saved.display_name()),
                        Locale::En => format!("Tube '{}' added to catalog and drawer.", saved.display_name()),
                    };
                    state.notify(msg);
                }
                Err(e) => state.notify(format!("Error: {e}")),
            }
        });
    };

    rsx! {
        div { class: "card u-bc-accent",
            h3 { class: "card-title", {locale.tr("Ajouter un tube personnalisé au catalogue", "Add a custom tube to the catalog")} }
            div { class: "grid-3 u-mb-12",
                div {
                    label { class: "field-label", {locale.tr("Marque / Gamme", "Brand / Line")} }
                    input {
                        r#type: "text",
                        class: "u-w-100pct",
                        value: "{brand}",
                        oninput: move |e| brand.set(e.value()),
                    }
                }
                div {
                    label { class: "field-label", {locale.tr("Nom de la couleur", "Color Name")} }
                    input {
                        r#type: "text",
                        class: "u-w-100pct",
                        placeholder: locale.tr("ex. Ocre d'or maison", "e.g. Custom Gold Ochre"),
                        value: "{name}",
                        oninput: move |e| name.set(e.value()),
                    }
                }
                div {
                    label { class: "field-label", {locale.tr("Code / Référence", "Code / Reference")} }
                    input {
                        r#type: "text",
                        class: "u-w-100pct",
                        placeholder: "042",
                        value: "{code}",
                        oninput: move |e| code.set(e.value()),
                    }
                }
            }
            div { class: "grid-3 u-mb-12",
                div {
                    label { class: "field-label", {locale.tr("Pigments C.I. (séparés par virgule)", "C.I. Pigments (comma-separated)")} }
                    input {
                        r#type: "text",
                        class: "u-w-100pct",
                        placeholder: "PY42, PR101",
                        value: "{pigments}",
                        oninput: move |e| pigments.set(e.value()),
                    }
                }
                div {
                    label { class: "field-label", {locale.tr("Opacité", "Opacity")} }
                    select {
                        class: "u-w-100pct",
                        onchange: move |e| opacity.set(opacity_of(&e.value())),
                        for op in Opacity::ALL {
                            option {
                                value: "{op:?}",
                                selected: *opacity.read() == op,
                                "{op.label(locale)}"
                            }
                        }
                    }
                }
                div {
                    label { class: "field-label", {locale.tr("Classe siccative", "Drying Class")} }
                    select {
                        class: "u-w-100pct",
                        onchange: move |e| drying.set(drying_class_of(&e.value())),
                        for dc in DryingClass::ALL {
                            option {
                                value: "{dc:?}",
                                selected: *drying.read() == dc,
                                "{dc.label(locale)}"
                            }
                        }
                    }
                }
            }
            div { class: "row-between",
                div { class: "row",
                    label { class: "field-label u-m-0", {locale.tr("Teinte de sortie de tube :", "Masstone color:")} }
                    HexColorInput {
                        value: hex(),
                        onchange: move |v: String| hex.set(v),
                    }
                }
                button {
                    class: "btn-primary",
                    onclick: save,
                    {locale.tr("Enregistrer ce tube", "Save tube")}
                }
            }

        }
    }
}

#[component]
pub(super) fn RecognizedTubesCard(recognized_tubes: Signal<Vec<RecognizedTube>>) -> Element {
    let state = use_context::<AppState>();
    let locale = *state.locale.read();

    rsx! {
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
                    RecognizedTubeRow { tube }
                }
            }

        }
    }
}

#[component]
fn RecognizedTubeRow(tube: RecognizedTube) -> Element {
    let services = use_context::<AppServices>();
    let mut state = use_context::<AppState>();
    let locale = *state.locale.read();

    let add_to_drawer = move |pid: i64| {
        let services = services.clone();
        spawn(async move {
            let loc = *state.locale.read();
            if set_in_stock(&services, pid, true).await.is_ok() {
                state.reload_all(&services).await;
                state.notify(loc.tr("Tube ajouté à votre tiroir.", "Tube added to your drawer."));
            }
        });
    };

    rsx! {
        div { class: "card-subtle row-between",
            div {
                div { class: "u-fw-600", "{tube.read_brand} — {tube.read_name}" }
                if !tube.read_code.is_empty() {
                    div { class: "muted mono", {locale.tr("Code lu : ", "Read code: ")}, "{tube.read_code}" }
                }
                if let Some(ref matched) = tube.matched {
                    div { class: "row u-mt-4",
                        ColorSwatch { hex: matched.hex_color.clone(), size: 18 }
                        span { class: "pill pill-ok", {locale.tr("Correspondance : ", "Match: ")}, "{matched.display_name()}" }
                    }
                } else {
                    span { class: "pill pill-warn", {locale.tr("Hors catalogue", "Not in catalog")} }
                }
            }
            if let Some(pid) = tube.matched.as_ref().and_then(|m| m.id) {
                button {
                    class: "btn-primary",
                    onclick: move |_| add_to_drawer(pid),
                    {locale.tr("Ajouter au tiroir", "Add to drawer")}
                }
            }
        }
    }
}
