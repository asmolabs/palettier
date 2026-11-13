//! Oil consistency & drops, and wet-on-wet blending of two consistencies.

use crate::state::AppState;
use dioxus::prelude::*;
use palettier_domain::i18n::Locale;
use palettier_domain::miniature::{Consistency, WetOnWetBlendingAdvisor, thinner_drops_per_part};
use palettier_domain::paint::{PaintSystem, Technique};

#[derive(Clone, Copy)]
pub(super) struct ConsistencyState {
    technique: Signal<Technique>,
    parts: Signal<u32>,
    blending_base: Signal<Technique>,
    blending_accent: Signal<Technique>,
}

impl ConsistencyState {
    pub(super) fn new() -> Self {
        Self {
            technique: Signal::new(Technique::Glaze),
            parts: Signal::new(2),
            blending_base: Signal::new(Technique::OilRendering),
            blending_accent: Signal::new(Technique::Highlight),
        }
    }
}

#[component]
pub(super) fn OilConsistency() -> Element {
    let state = use_context::<AppState>();
    let ConsistencyState { technique: consistency_technique, parts: mut consistency_parts, .. } =
        use_context::<ConsistencyState>();
    let locale = *state.locale.read();

    rsx! {
        div { class: "grid-2", style: "grid-template-columns: 1fr 1fr; gap: 16px;",
            div { class: "card",
                h3 { class: "card-title", {locale.tr("💧 Calculateur de Consistance & Gouttes de Dilution", "💧 Oil Consistency & Dropper Dosage Calculator")} }
                p { class: "muted", style: "font-size: 12px; margin-bottom: 14px;",
                    {locale.tr("Contrairement à l'acrylique fluide, l'huile sort en pâte visqueuse du tube (parts / noisettes), et ce sont les solvants minéraux et médiums qui se dosent au compte-gouttes.", "Unlike fluid acrylics, oil leaves the tube as a dense paste (parts / hazelnuts), while solvents and mediums are metered drop by drop.")}
                }

                // Technique selection
                div { style: "display: flex; flex-direction: column; gap: 8px; margin-bottom: 16px;",
                    for tech in Technique::for_system(PaintSystem::Oil) {
                        {technique_option(consistency_technique, locale, tech)}
                    }
                }

                // Parts slider
                div { class: "card-subtle",
                    div { class: "row-between", style: "margin-bottom: 8px;",
                        strong { style: "font-size: 12.5px;", {locale.tr("Volume de pâte d'huile préparé :", "Oil Paint Volume:")} }
                        span { class: "pill pill-ok", style: "font-size: 13px; font-weight: 750;", "{consistency_parts} ", {locale.tr("parts (noisettes)", "parts (hazelnut dabs)")} }
                    }
                    input {
                        r#type: "range",
                        min: "1",
                        max: "10",
                        value: "{consistency_parts}",
                        oninput: move |ev| {
                            if let Ok(p) = ev.value().parse::<u32>() {
                                consistency_parts.set(p);
                            }
                        },
                        style: "width: 100%;",
                    }
                }
            }

            // Right: Computed Dropper Formulation
            div { class: "card",
                {
                    let tech = *consistency_technique.read();
                    let parts = *consistency_parts.read();
                    let tot_solvent_drops = thinner_drops_per_part(tech, PaintSystem::Oil) * parts;
                    let thinner = tech.default_medium_in(PaintSystem::Oil);

                    rsx! {
                        h3 { class: "card-title", {locale.tr("📋 Formule de Dosage Précise", "📋 Exact Dropper Formulation")} }

                        div { class: "grid-2", style: "gap: 10px; margin-bottom: 16px;",
                            div { class: "card-subtle", style: "text-align: center; padding: 12px;",
                                div { style: "font-size: 26px; font-weight: 800; color: var(--accent);", "{parts}" }
                                strong { style: "font-size: 12px;", {locale.tr("Parts de Pâte d'Huile", "Oil Paste Parts")} }
                                div { class: "muted", style: "font-size: 10.5px; margin-top: 2px;", {locale.tr("Noisettes / têtes d'allumette", "Hazelnut dabs / matchstick heads")} }
                            }
                            div { class: "card-subtle", style: "text-align: center; padding: 12px;",
                                div { style: "font-size: 26px; font-weight: 800; color: var(--accent-ok);", "{tot_solvent_drops}" }
                                strong { style: "font-size: 12px;", {locale.tr("Gouttes de ", "Drops of ")}, "{thinner.label(locale).to_lowercase()}" }
                                div { class: "muted", style: "font-size: 10.5px; margin-top: 2px;", {locale.tr("Compte-gouttes / pipette", "Dropper / pipette")} }
                            }
                        }

                        div { class: "card-subtle", style: "line-height: 1.5; font-size: 12px;",
                            div { style: "font-weight: 700; margin-bottom: 4px; color: var(--accent);",
                                {locale.tr("💡 Conseil d'Atelier :", "💡 Studio Advice:")}
                            }
                            for tip in tech.tips_in(PaintSystem::Oil, locale) {
                                div { "• {tip}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn technique_option(mut consistency_technique: Signal<Technique>, locale: Locale, tech: Technique) -> Element {
    let is_sel = *consistency_technique.read() == tech;
    let drops = thinner_drops_per_part(tech, PaintSystem::Oil);
    let consistency = Consistency::of(tech);
    rsx! {
        div {
            class: if is_sel { "card-subtle active" } else { "card-subtle" },
            style: "cursor: pointer; padding: 10px;",
            onclick: move |_| consistency_technique.set(tech),
            div { class: "row-between", style: "align-items: center; margin-bottom: 4px;",
                strong { style: "font-size: 13px;", "{tech.label(locale)}" }
                span { class: "pill pill-accent",
                    if drops == 0 {
                        {locale.tr("pure", "neat")}
                    } else {
                        "{drops} ", {locale.tr("gtt / part", "drops / part")}
                    }
                }
            }
            div { class: "muted", style: "font-size: 11.5px;",
                strong { "{consistency.label(locale)}. " }
                "{consistency.description_i18n(locale)}"
            }
        }
    }
}

#[component]
pub(super) fn WetOnWetBlending() -> Element {
    let state = use_context::<AppState>();
    let ConsistencyState { blending_base: mut blending_base_tech, blending_accent: mut blending_accent_tech, .. } =
        use_context::<ConsistencyState>();
    let locale = *state.locale.read();

    rsx! {
        div { style: "display: flex; flex-direction: column; gap: 16px;",
            div { class: "card",
                div { class: "row-between", style: "margin-bottom: 12px; align-items: center;",
                    h3 { class: "card-title", style: "margin: 0;",
                        "🧪 ", {locale.tr("Matrice de Fondu dans le Frais (Wet-on-Wet Blending)", "Wet-on-Wet Blending & Feathering Matrix")}
                    }
                    span { class: "pill pill-accent",
                        {locale.tr("Compatibilité de Viscosité & Pinceaux", "Viscosity Compatibility & Brushes")}
                    }
                }
                p { class: "muted", style: "margin-top: 0; font-size: 13px;",
                    {locale.tr(
                        "Le fondu dans le frais est la technique suprême à l'huile pour créer des transitions lisses sans marquer d'escaliers. L'analyse ci-dessous prévient les risques d'arrachement de peinture (tearing) et d'auréoles (blooming).",
                        "Wet-on-wet blending is the ultimate oil technique for silky transitions. This evaluation prevents paint tearing and solvent blooming."
                    )}
                }

                div { class: "grid-2", style: "gap: 16px; margin-bottom: 16px;",
                    div { class: "card-subtle",
                        label { style: "font-weight: 700; font-size: 13px; display: block; margin-bottom: 8px;",
                            {locale.tr("1. Consistance de la Couche de Base :", "1. Base Coat Consistency:")}
                        }
                        div { style: "display: flex; flex-direction: column; gap: 6px;",
                            for tech in Technique::for_system(PaintSystem::Oil) {
                                button {
                                    class: if *blending_base_tech.read() == tech { "btn-primary" } else { "btn" },
                                    style: "justify-content: flex-start; text-align: left; font-size: 12px;",
                                    onclick: move |_| blending_base_tech.set(tech),
                                    "{tech.label(locale)}"
                                }
                            }
                        }
                    }

                    div { class: "card-subtle",
                        label { style: "font-weight: 700; font-size: 13px; display: block; margin-bottom: 8px;",
                            {locale.tr("2. Consistance du Rehaut ou de l'Ombre :", "2. Highlight or Shadow Consistency:")}
                        }
                        div { style: "display: flex; flex-direction: column; gap: 6px;",
                            for tech in Technique::for_system(PaintSystem::Oil) {
                                button {
                                    class: if *blending_accent_tech.read() == tech { "btn-primary" } else { "btn" },
                                    style: "justify-content: flex-start; text-align: left; font-size: 12px;",
                                    onclick: move |_| blending_accent_tech.set(tech),
                                    "{tech.label(locale)}"
                                }
                            }
                        }
                    }
                }

                {
                    let eval = WetOnWetBlendingAdvisor::evaluate(*blending_base_tech.read(), *blending_accent_tech.read(), locale);
                    rsx! {
                        div { class: "card-subtle", style: "border-left: 4px solid var(--accent); padding: 14px 16px;",
                            div { class: "row-between", style: "margin-bottom: 10px; align-items: center;",
                                span { class: eval.rating.badge_class(), style: "font-size: 12.5px; padding: 5px 10px;",
                                    "{eval.rating.label(locale)}"
                                }
                                span { class: "mono muted", style: "font-size: 11.5px;",
                                    {locale.tr("Fenêtre de tirage : ", "Working window: ")},
                                    "{eval.ideal_window_minutes}"
                                }
                            }

                            div { style: "font-size: 13px; line-height: 1.5; margin-bottom: 10px;",
                                strong { {locale.tr("Comportement mécanique : ", "Mechanical traction: ")} }
                                "{eval.traction_analysis}"
                            }

                            div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 12px; margin-top: 10px; padding-top: 10px; border-top: 1px solid var(--border);",
                                div {
                                    strong { style: "font-size: 12px; color: var(--accent);",
                                        "🖌️ ", {locale.tr("Pinceau Préconisé :", "Recommended Brush:")}
                                    }
                                    div { style: "font-size: 12px; margin-top: 4px;", "{eval.recommended_brush}" }
                                }
                                div {
                                    strong { style: "font-size: 12px; color: var(--accent);",
                                        "💡 ", {locale.tr("Conseil d'Atelier Médium :", "Medium Tweak Tip:")}
                                    }
                                    div { style: "font-size: 12px; margin-top: 4px;", "{eval.medium_tweak_advice}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
