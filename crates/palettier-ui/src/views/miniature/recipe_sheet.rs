//! Army recipe sheet: a printable card to paint a whole army the same way. The oil mix of
//! the base acrylic is computed on the palette.

use super::shared::{acrylic_catalog, copy_to_clipboard, paint_key};
use crate::state::{AppServices, AppState};
use crate::theme::ColorSwatch;
use dioxus::prelude::*;
use palettier_domain::i18n::Locale;
use palettier_domain::miniature::ArmyRecipeSheet;
use palettier_domain::mix::MixSuggestion;
use palettier_domain::paint::Paint;

#[derive(Clone, Copy)]
pub(super) struct RecipeSheetState {
    sheet: Signal<ArmyRecipeSheet>,
}

impl RecipeSheetState {
    pub(super) fn new() -> Self {
        Self { sheet: Signal::new(ArmyRecipeSheet::default()) }
    }
}

/// Puts `acrylic` as the base colour of the sheet, with its oil mix (cleared when no mix
/// was found).
fn set_base_acrylic(sheet: &mut ArmyRecipeSheet, acrylic: &Paint, mix: Option<&MixSuggestion>, locale: Locale) {
    sheet.base_acrylic = format!("{} ({})", acrylic.name, acrylic.brand);
    sheet.base_hex = acrylic.hex_color.clone();
    sheet.oil_mix_formula = mix.map(|m| m.describe_i18n(locale)).unwrap_or_default();
    sheet.mixed_hex = mix.map(|m| m.color.to_hex()).unwrap_or_default();
}

#[component]
pub(super) fn RecipeSheet(oil_candidates: Vec<Paint>) -> Element {
    let mut state = use_context::<AppState>();
    let services = use_context::<AppServices>();
    let mut recipe_sheet = use_context::<RecipeSheetState>().sheet;
    let locale = *state.locale.read();
    let acrylic_paints = acrylic_catalog(&state.paints.read());

    rsx! {
        div { class: "grid-2", style: "grid-template-columns: 1fr 1.25fr; gap: 16px;",
            // Left: Interactive Sheet Form
            div { class: "card",
                h3 { class: "card-title", {locale.tr("📄 Fiche de Recette d'Armée & Modélisme", "📄 Miniature Army Recipe Card Generator")} }
                p { class: "muted", style: "font-size: 12px; margin-bottom: 14px;",
                    {locale.tr("Enregistrez et exportez vos recettes d'escouades pour garantir une cohérence absolue de vos figurines au fil des mois.", "Record and export your squad painting recipes to ensure perfect army-wide consistency over time.")}
                }

                div { style: "display: flex; flex-direction: column; gap: 10px;",
                    div {
                        label { style: "font-size: 11.5px; font-weight: 600;", {locale.tr("Nom de l'Armée / Compagnie :", "Army / Faction Name:")} }
                        input {
                            class: "input",
                            value: "{recipe_sheet.read().army_name}",
                            placeholder: locale.tr("ex. Compagnie rouge", "e.g. Red company"),
                            oninput: move |ev| recipe_sheet.write().army_name = ev.value(),
                        }
                    }

                    div {
                        label { style: "font-size: 11.5px; font-weight: 600;", {locale.tr("Nom de l'Unité / Figurine :", "Unit / Squad Name:")} }
                        input {
                            class: "input",
                            value: "{recipe_sheet.read().unit_name}",
                            placeholder: locale.tr("ex. Escouade tactique", "e.g. Tactical squad"),
                            oninput: move |ev| recipe_sheet.write().unit_name = ev.value(),
                        }
                    }

                    div {
                        label { style: "font-size: 11.5px; font-weight: 600;", {locale.tr("Sous-couche (Primer) :", "Primer Base:")} }
                        input {
                            class: "input",
                            value: "{recipe_sheet.read().primer}",
                            placeholder: locale.tr("ex. Sous-couche zénithale noir puis blanc", "e.g. Zenithal black then white"),
                            oninput: move |ev| recipe_sheet.write().primer = ev.value(),
                        }
                    }

                    div {
                        label { style: "font-size: 11.5px; font-weight: 600;", {locale.tr("Acrylique de base :", "Base Acrylic Paint:")} }
                        // Picked from the catalog; the oil mix below follows from it.
                        select {
                            class: "input",
                            onchange: {
                                let acrylics = acrylic_paints.clone();
                                let oils = oil_candidates.clone();
                                let mixer = services.mix_service.clone();
                                move |ev: FormEvent| {
                                    let Some(p) = acrylics.iter().find(|p| paint_key(p) == ev.value()) else { return };
                                    let mix = mixer.suggest_mixes(p.color(), &oils, 1, Some(3)).into_iter().next();
                                    set_base_acrylic(&mut recipe_sheet.write(), p, mix.as_ref(), locale);
                                }
                            },
                            option { value: "", {locale.tr("— Choisir une acrylique du catalogue —", "— Pick an acrylic from the catalog —")} }
                            for p in acrylic_paints.iter() {
                                option {
                                    value: "{paint_key(p)}",
                                    selected: recipe_sheet.read().base_hex == p.hex_color && recipe_sheet.read().base_acrylic.starts_with(&p.name),
                                    "{p.name} ({p.brand})"
                                }
                            }
                        }
                    }

                    div {
                        label { style: "font-size: 11.5px; font-weight: 600;", {locale.tr("Mélange d'huiles calculé sur la palette :", "Oil mix computed from the palette:")} }
                        div { class: "card-subtle", style: "padding: 8px 10px; font-size: 12.5px;",
                            if recipe_sheet.read().oil_mix_formula.is_empty() {
                                span { class: "muted", {locale.tr("Choisissez une acrylique de base.", "Pick a base acrylic.")} }
                            } else {
                                div { class: "row", style: "gap: 8px;",
                                    ColorSwatch { hex: recipe_sheet.read().mixed_hex.clone(), size: 18 }
                                    strong { "{recipe_sheet.read().oil_mix_formula}" }
                                }
                            }
                        }
                    }

                    div {
                        label { style: "font-size: 11.5px; font-weight: 600;", {locale.tr("Lavis & Pin Wash :", "Wash & Pin Wash:")} }
                        input {
                            class: "input",
                            value: "{recipe_sheet.read().wash_technique}",
                            placeholder: locale.tr("ex. Jus capillaire terre d'ombre", "e.g. Burnt umber pin wash"),
                            oninput: move |ev| recipe_sheet.write().wash_technique = ev.value(),
                        }
                    }

                    div {
                        label { style: "font-size: 11.5px; font-weight: 600;", {locale.tr("Weathering & Altérations :", "Weathering & Effects:")} }
                        input {
                            class: "input",
                            value: "{recipe_sheet.read().weathering}",
                            placeholder: locale.tr("ex. Coulures sur les jambières", "e.g. Streaks on the greaves"),
                            oninput: move |ev| recipe_sheet.write().weathering = ev.value(),
                        }
                    }

                    div {
                        label { style: "font-size: 11.5px; font-weight: 600;", {locale.tr("Vernis final :", "Final Varnish:")} }
                        input {
                            class: "input",
                            value: "{recipe_sheet.read().varnish}",
                            placeholder: locale.tr("ex. Vernis mat, deux voiles", "e.g. Matt varnish, two thin coats"),
                            oninput: move |ev| recipe_sheet.write().varnish = ev.value(),
                        }
                    }

                    div {
                        label { style: "font-size: 11.5px; font-weight: 600;", {locale.tr("Notes d'Atelier & Conseils :", "Studio Notes & Tips:")} }
                        textarea {
                            class: "input",
                            rows: "3",
                            value: "{recipe_sheet.read().notes}",
                            placeholder: locale.tr("ex. Attendre 48 h avant le vernis", "e.g. Wait 48 h before varnishing"),
                            oninput: move |ev| recipe_sheet.write().notes = ev.value(),
                        }
                    }
                }
            }

            // Right: Live Sheet Preview & Export Actions
            div { class: "card",
                div { class: "row-between", style: "margin-bottom: 12px; align-items: center;",
                    h3 { class: "card-title", style: "margin: 0;", {locale.tr("Aperçu de la Fiche de Recette", "Recipe Sheet Preview")} }
                    div { class: "row", style: "gap: 6px;",
                        button {
                            class: "btn",
                            onclick: {
                                let md = recipe_sheet.read().to_markdown();
                                move |_| {
                                    copy_to_clipboard(&md);
                                    state.notify(locale.tr("📋 Fiche copiée dans le presse-papiers (Markdown) !", "📋 Recipe card copied to clipboard (Markdown)!"));
                                }
                            },
                            "📋 Markdown"
                        }
                        button {
                            class: "btn btn-primary",
                            onclick: {
                                let html = recipe_sheet.read().to_printable_html();
                                move |_| {
                                    copy_to_clipboard(&html);
                                    state.notify(locale.tr("🖨️ HTML imprimable copié dans le presse-papiers !", "🖨️ Printable HTML copied to clipboard!"));
                                }
                            },
                            "🖨️ Imprimer / HTML"
                        }
                    }
                }

                // Styled Card Preview
                div { style: "border: 1px solid var(--border-main); border-radius: 8px; padding: 18px; background: var(--bg-surface); line-height: 1.45;",
                    h2 { style: "margin-top: 0; font-size: 18px; margin-bottom: 2px;", "{recipe_sheet.read().army_name}" }
                    div { class: "muted", style: "font-size: 12.5px; margin-bottom: 16px;",
                        "{recipe_sheet.read().unit_name} · ",
                        strong { "{recipe_sheet.read().faction}" }
                    }

                    div { class: "card-subtle", style: "margin-bottom: 10px;",
                        div { style: "font-weight: 700; font-size: 11.5px; text-transform: uppercase; margin-bottom: 4px; color: var(--accent);",
                            "1. Sous-couche"
                        }
                        div { style: "font-size: 12.5px;", "{recipe_sheet.read().primer}" }
                    }

                    div { class: "card-subtle", style: "margin-bottom: 10px;",
                        div { style: "font-weight: 700; font-size: 11.5px; text-transform: uppercase; margin-bottom: 6px; color: var(--accent);",
                            "2. Couleur de Base & Formule Huile"
                        }
                        div { class: "row", style: "gap: 8px; align-items: center; margin-bottom: 4px;",
                            if !recipe_sheet.read().base_hex.is_empty() {
                                ColorSwatch { hex: recipe_sheet.read().base_hex.clone(), size: 20 }
                            }
                            span { style: "font-size: 12.5px;", "{recipe_sheet.read().base_acrylic}" }
                        }
                        div { class: "row", style: "gap: 8px; align-items: center;",
                            if !recipe_sheet.read().mixed_hex.is_empty() {
                                ColorSwatch { hex: recipe_sheet.read().mixed_hex.clone(), size: 20 }
                            }
                            span { style: "font-size: 12.5px; font-weight: 600;", "{recipe_sheet.read().oil_mix_formula}" }
                        }
                    }

                    div { class: "card-subtle", style: "margin-bottom: 10px;",
                        div { style: "font-weight: 700; font-size: 11.5px; text-transform: uppercase; margin-bottom: 4px; color: var(--accent);",
                            "3. Lavis & Pin Wash"
                        }
                        div { style: "font-size: 12.5px;", "{recipe_sheet.read().wash_technique}" }
                    }

                    div { class: "card-subtle", style: "margin-bottom: 10px;",
                        div { style: "font-weight: 700; font-size: 11.5px; text-transform: uppercase; margin-bottom: 4px; color: var(--accent);",
                            "4. Weathering & Usure"
                        }
                        div { style: "font-size: 12.5px;", "{recipe_sheet.read().weathering}" }
                    }

                    div { class: "card-subtle", style: "margin-bottom: 10px;",
                        div { style: "font-weight: 700; font-size: 11.5px; text-transform: uppercase; margin-bottom: 4px; color: var(--accent);",
                            "5. Vernis de Protection"
                        }
                        div { style: "font-size: 12.5px;", "{recipe_sheet.read().varnish}" }
                    }

                    div { class: "card-subtle",
                        div { style: "font-weight: 700; font-size: 11.5px; text-transform: uppercase; margin-bottom: 4px; color: var(--text-dim);",
                            "Notes d'Atelier"
                        }
                        div { style: "font-size: 12px; font-style: italic;", "{recipe_sheet.read().notes}" }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use palettier_domain::color::Rgb;
    use palettier_domain::paint::PaintSystem;
    use palettier_domain::mix::ColorMixService;

    #[test]
    fn the_base_acrylic_brings_its_oil_mix_or_clears_it() {
        let acrylic = Paint::new("Citadel", "Mephiston Red", "#9A1115").in_system(PaintSystem::Acrylic);
        let oils = vec![Paint::new("W&N", "Cadmium Red", "#B0201C")];
        let mix = ColorMixService.suggest_mixes(Rgb::of_hex("#9A1115").unwrap(), &oils, 1, Some(3));
        let mut sheet = ArmyRecipeSheet::default();
        set_base_acrylic(&mut sheet, &acrylic, mix.first(), Locale::Fr);
        assert_eq!("Mephiston Red (Citadel)", sheet.base_acrylic);
        assert_eq!("#9A1115", sheet.base_hex);
        assert!(sheet.oil_mix_formula.contains("Cadmium Red"));
        assert!(!sheet.mixed_hex.is_empty());

        set_base_acrylic(&mut sheet, &acrylic, None, Locale::Fr);
        assert!(sheet.oil_mix_formula.is_empty());
        assert!(sheet.mixed_hex.is_empty());
    }
}
