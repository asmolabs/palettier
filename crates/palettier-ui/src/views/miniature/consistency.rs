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
        div { class: "grid-2 u-gtc-1fr-1fr u-gap-16",
            div { class: "card",
                h3 { class: "card-title", {locale.tr("💧 Calculateur de Consistance & Gouttes de Dilution", "💧 Oil Consistency & Dropper Dosage Calculator")} }
                p { class: "muted u-fs-12 u-mb-14",
                    {locale.tr("Contrairement à l'acrylique fluide, l'huile sort en pâte visqueuse du tube (parts / noisettes), et ce sont les solvants minéraux et médiums qui se dosent au compte-gouttes.", "Unlike fluid acrylics, oil leaves the tube as a dense paste (parts / hazelnuts), while solvents and mediums are metered drop by drop.")}
                }

                // Technique selection
                div { class: "u-d-flex u-fd-column u-gap-8 u-mb-16",
                    for tech in Technique::for_system(PaintSystem::Oil) {
                        {technique_option(consistency_technique, locale, tech)}
                    }
                }

                // Parts slider
                div { class: "card-subtle",
                    div { class: "row-between u-mb-8",
                        strong { class: "u-fs-12p5", {locale.tr("Volume de pâte d'huile préparé :", "Oil Paint Volume:")} }
                        span { class: "pill pill-ok u-fs-13 u-fw-750", "{consistency_parts} ", {locale.tr("parts (noisettes)", "parts (hazelnut dabs)")} }
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
                        class: "u-w-100pct",
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

                        div { class: "grid-2 u-gap-10 u-mb-16",
                            div { class: "card-subtle u-ta-center u-p-12",
                                div { class: "u-fs-26 u-fw-800 u-c-accent", "{parts}" }
                                strong { class: "u-fs-12", {locale.tr("Parts de Pâte d'Huile", "Oil Paste Parts")} }
                                div { class: "muted u-fs-10p5 u-mt-2", {locale.tr("Noisettes / têtes d'allumette", "Hazelnut dabs / matchstick heads")} }
                            }
                            div { class: "card-subtle u-ta-center u-p-12",
                                div { class: "u-fs-26 u-fw-800 u-c-accent-ok", "{tot_solvent_drops}" }
                                strong { class: "u-fs-12", {locale.tr("Gouttes de ", "Drops of ")}, "{thinner.label(locale).to_lowercase()}" }
                                div { class: "muted u-fs-10p5 u-mt-2", {locale.tr("Compte-gouttes / pipette", "Dropper / pipette")} }
                            }
                        }

                        div { class: "card-subtle u-lh-1p5 u-fs-12",
                            div { class: "u-fw-700 u-mb-4 u-c-accent",
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
            class: "u-cur-pointer u-p-10",
            onclick: move |_| consistency_technique.set(tech),
            div { class: "row-between u-ai-center u-mb-4",
                strong { class: "u-fs-13", "{tech.label(locale)}" }
                span { class: "pill pill-accent",
                    if drops == 0 {
                        {locale.tr("pure", "neat")}
                    } else {
                        "{drops} ", {locale.tr("gtt / part", "drops / part")}
                    }
                }
            }
            div { class: "muted u-fs-11p5",
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
        div { class: "u-d-flex u-fd-column u-gap-16",
            div { class: "card",
                div { class: "row-between u-mb-12 u-ai-center",
                    h3 { class: "card-title u-m-0",
                        "🧪 ", {locale.tr("Matrice de Fondu dans le Frais (Wet-on-Wet Blending)", "Wet-on-Wet Blending & Feathering Matrix")}
                    }
                    span { class: "pill pill-accent",
                        {locale.tr("Compatibilité de Viscosité & Pinceaux", "Viscosity Compatibility & Brushes")}
                    }
                }
                p { class: "muted u-mt-0 u-fs-13",
                    {locale.tr(
                        "Le fondu dans le frais est la technique suprême à l'huile pour créer des transitions lisses sans marquer d'escaliers. L'analyse ci-dessous prévient les risques d'arrachement de peinture (tearing) et d'auréoles (blooming).",
                        "Wet-on-wet blending is the ultimate oil technique for silky transitions. This evaluation prevents paint tearing and solvent blooming."
                    )}
                }

                div { class: "grid-2 u-gap-16 u-mb-16",
                    div { class: "card-subtle",
                        label { class: "u-fw-700 u-fs-13 u-d-block u-mb-8",
                            {locale.tr("1. Consistance de la Couche de Base :", "1. Base Coat Consistency:")}
                        }
                        div { class: "u-d-flex u-fd-column u-gap-6",
                            for tech in Technique::for_system(PaintSystem::Oil) {
                                button {
                                    class: if *blending_base_tech.read() == tech { "btn-primary" } else { "btn" },
                                    class: "u-jc-flex-start u-ta-left u-fs-12",
                                    onclick: move |_| blending_base_tech.set(tech),
                                    "{tech.label(locale)}"
                                }
                            }
                        }
                    }

                    div { class: "card-subtle",
                        label { class: "u-fw-700 u-fs-13 u-d-block u-mb-8",
                            {locale.tr("2. Consistance du Rehaut ou de l'Ombre :", "2. Highlight or Shadow Consistency:")}
                        }
                        div { class: "u-d-flex u-fd-column u-gap-6",
                            for tech in Technique::for_system(PaintSystem::Oil) {
                                button {
                                    class: if *blending_accent_tech.read() == tech { "btn-primary" } else { "btn" },
                                    class: "u-jc-flex-start u-ta-left u-fs-12",
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
                        div { class: "card-subtle u-bl-4-solid-accent u-p-14-16",
                            div { class: "row-between u-mb-10 u-ai-center",
                                span { class: "{eval.rating.badge_class()}", class: "u-fs-12p5 u-p-5-10",
                                    "{eval.rating.label(locale)}"
                                }
                                span { class: "mono muted u-fs-11p5",
                                    {locale.tr("Fenêtre de tirage : ", "Working window: ")},
                                    "{eval.ideal_window_minutes}"
                                }
                            }

                            div { class: "u-fs-13 u-lh-1p5 u-mb-10",
                                strong { {locale.tr("Comportement mécanique : ", "Mechanical traction: ")} }
                                "{eval.traction_analysis}"
                            }

                            div { class: "u-d-grid u-gtc-1fr-1fr u-gap-12 u-mt-10 u-pt-10 u-bt-1-solid-border",
                                div {
                                    strong { class: "u-fs-12 u-c-accent",
                                        "🖌️ ", {locale.tr("Pinceau Préconisé :", "Recommended Brush:")}
                                    }
                                    div { class: "u-fs-12 u-mt-4", "{eval.recommended_brush}" }
                                }
                                div {
                                    strong { class: "u-fs-12 u-c-accent",
                                        "💡 ", {locale.tr("Conseil d'Atelier Médium :", "Medium Tweak Tip:")}
                                    }
                                    div { class: "u-fs-12 u-mt-4", "{eval.medium_tweak_advice}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
