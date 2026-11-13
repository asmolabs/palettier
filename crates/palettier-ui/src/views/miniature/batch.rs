//! Batches: tracking a squad step by step, and the mother mix it needs.

use crate::state::AppState;
use dioxus::prelude::*;
use palettier_domain::miniature::{BatchMotherMixCalculator, BatchSquad, MiniatureScale};

#[derive(Clone, Copy)]
pub(super) struct BatchState {
    squad: Signal<BatchSquad>,
    squad_size: Signal<usize>,
    squad_name: Signal<String>,
    mother_mix_scale: Signal<MiniatureScale>,
    mother_mix_count: Signal<u32>,
    mother_mix_full_coverage: Signal<bool>,
}

impl BatchState {
    pub(super) fn new() -> Self {
        Self {
            squad: Signal::new(BatchSquad::standard_space_marine_squad("Escouade Intercessors", 5)),
            squad_size: Signal::new(5),
            squad_name: Signal::new("Escouade Intercessors".to_string()),
            mother_mix_scale: Signal::new(MiniatureScale::Scale32mm),
            mother_mix_count: Signal::new(10),
            mother_mix_full_coverage: Signal::new(true),
        }
    }
}

#[component]
pub(super) fn BatchSquadTool() -> Element {
    let state = use_context::<AppState>();
    let BatchState { mut squad, mut squad_size, mut squad_name, .. } = use_context::<BatchState>();
    let locale = *state.locale.read();

    rsx! {
        div { class: "card",
            div { class: "row-between u-mb-14",
                div {
                    h3 { class: "card-title u-m-0",
                        {locale.tr("Peinture d'Armée & Escouade à la Chaîne (Batch Painting)", "Batch Squad Painting Tracker")}
                    }
                    div { class: "muted u-fs-12 u-mt-2",
                        {locale.tr("Suivi rigoureux par figurine et par étape pour peindre une escouade homogène.", "Rigorous step-by-step progress tracking to paint an entire homogeneous squad.")}
                    }
                }

                div { class: "row u-gap-12 u-ai-center",
                    div { class: "pill pill-ok u-fs-13 u-fw-750",
                        "{squad.read().progress_pct():.0}% ", {locale.tr("terminé", "completed")}
                    }
                    div { class: "pill pill-accent",
                        {locale.tr("Multiplicateur mélange mère : ", "Mother Mix Multiplier: ")},
                        "{squad.read().mother_mix_multiplier():.1}x"
                    }
                }
            }

            div { class: "card-subtle u-mb-16",
                div { class: "row u-gap-16 u-ai-center",
                    div {
                        label { class: "field-label", {locale.tr("Nom de l'escouade", "Squad Name")} }
                        input {
                            r#type: "text",
                            value: "{squad_name}",
                            oninput: move |ev| {
                                let n = ev.value();
                                squad_name.set(n.clone());
                                let cnt = *squad_size.read();
                                squad.set(BatchSquad::standard_space_marine_squad(&n, cnt));
                            }
                        }
                    }
                    div {
                        label { class: "field-label", {locale.tr("Effectif (Figurines)", "Headcount (Miniatures)")} }
                        input {
                            r#type: "number",
                            class: "u-w-80",
                            min: "1",
                            max: "50",
                            value: "{squad_size}",
                            oninput: move |ev| {
                                if let Ok(c) = ev.value().parse::<usize>() {
                                    squad_size.set(c);
                                    let n = squad_name.read().clone();
                                    squad.set(BatchSquad::standard_space_marine_squad(&n, c));
                                }
                            }
                        }
                    }
                }
            }

            div { class: "u-ox-auto",
                table { class: "data-table",
                    thead {
                        tr {
                            th { class: "u-w-140", {locale.tr("Figurine", "Miniature")} }
                            for step in &squad.read().steps {
                                th { class: "u-fs-11 u-ta-center", "{step}" }
                            }
                        }
                    }
                    tbody {
                        for m_idx in 0..squad.read().miniature_count {
                            tr {
                                td { strong { "Figurine #{m_idx + 1}" } }
                                for s_idx in 0..squad.read().steps.len() {
                                    {squad_cell(squad, m_idx, s_idx)}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Whether miniature `m_idx` has step `s_idx` done (false outside the squad).
fn is_done(squad: &BatchSquad, m_idx: usize, s_idx: usize) -> bool {
    squad.progress.get(m_idx).and_then(|r| r.get(s_idx)).copied().unwrap_or(false)
}

fn squad_cell(mut squad: Signal<BatchSquad>, m_idx: usize, s_idx: usize) -> Element {
    let is_done = is_done(&squad.read(), m_idx, s_idx);
    rsx! {
        td { class: "u-ta-center u-cur-pointer",
            onclick: move |_| squad.write().toggle(m_idx, s_idx),
            span {
                class: if is_done { "pill pill-ok" } else { "pill" },
                class: "u-p-2-8 u-fs-11",
                if is_done { "✓" } else { "○" }
            }
        }
    }
}

#[component]
pub(super) fn BatchMotherMix() -> Element {
    let state = use_context::<AppState>();
    let BatchState { mut mother_mix_scale, mut mother_mix_count, mut mother_mix_full_coverage, .. } =
        use_context::<BatchState>();
    let locale = *state.locale.read();

    rsx! {
        div { class: "u-d-flex u-fd-column u-gap-16",
            div { class: "card",
                div { class: "row-between u-mb-12 u-ai-center",
                    h3 { class: "card-title u-m-0",
                        "🏺 ", {locale.tr("Calculateur de Mélanges Mères pour Escouades & Armées", "Batch Squad Mother Mix Calculator")}
                    }
                    span { class: "pill pill-accent",
                        {locale.tr("Volumes en Noisettes & Pots Scellés", "Hazelnut Volumes & Sealed Pots")}
                    }
                }
                p { class: "muted u-mt-0 u-fs-13",
                    {locale.tr(
                        "Préparer un mélange mère en amont garantit une teinte 100% identique sur toute une armée de figurines. Calculez exactement le volume de pâte nécessaire et découvrez les astuces pour le garder frais plusieurs semaines.",
                        "Mixing a mother mix beforehand ensures identical uniform color across an entire army. Compute the exact paste volume needed and tips to keep it usable for weeks."
                    )}
                }

                div { class: "grid-3 u-gap-14 u-mb-16",
                    div { class: "card-subtle",
                        label { class: "u-fw-700 u-fs-12p5 u-d-block u-mb-6",
                            {locale.tr("Échelle des figurines :", "Miniature Scale:")}
                        }
                        select {
                            class: "input",
                            value: "{mother_mix_scale.read().label(locale)}",
                            onchange: move |ev: Event<FormData>| {
                                for s in MiniatureScale::ALL {
                                    if s.label(locale) == ev.value() {
                                        mother_mix_scale.set(s);
                                        break;
                                    }
                                }
                            },
                            for s in MiniatureScale::ALL {
                                option { value: "{s.label(locale)}", "{s.label(locale)}" }
                            }
                        }
                    }

                    div { class: "card-subtle",
                        label { class: "u-fw-700 u-fs-12p5 u-d-block u-mb-6",
                            {locale.tr("Nombre de figurines : ", "Number of models: ")},
                            "{mother_mix_count}"
                        }
                        input {
                            r#type: "range",
                            min: "1",
                            max: "60",
                            value: "{mother_mix_count}",
                            oninput: move |ev: Event<FormData>| {
                                if let Ok(v) = ev.value().parse::<u32>() {
                                    mother_mix_count.set(v);
                                }
                            }
                        }
                    }

                    div { class: "card-subtle",
                        label { class: "u-fw-700 u-fs-12p5 u-d-block u-mb-6",
                            {locale.tr("Surface couverte :", "Covered Surface:")}
                        }
                        button {
                            class: if *mother_mix_full_coverage.read() { "btn-primary" } else { "btn" },
                            class: "u-w-100pct",
                            onclick: move |_| {
                                let cur = *mother_mix_full_coverage.read();
                                mother_mix_full_coverage.set(!cur);
                            },
                            if *mother_mix_full_coverage.read() {
                                "{locale.tr(\"🛡️ Couverture Intégrale (Blindage/Cape)\", \"🛡️ Full Coverage (Armor/Cape)\")}"
                            } else {
                                "{locale.tr(\"✨ Ombres / Rehauts Ciblés\", \"✨ Targeted Shadows / Highlights\")}"
                            }
                        }
                    }
                }

                {
                    let req = BatchMotherMixCalculator::calculate(*mother_mix_scale.read(), *mother_mix_count.read(), *mother_mix_full_coverage.read(), locale);
                    rsx! {
                        div { class: "card-subtle u-bl-4-solid-accent u-p-16",
                            div { class: "row-between u-ai-center u-mb-12",
                                span { class: "u-fw-750 u-fs-14 u-c-accent",
                                    "📊 ", {locale.tr("Volume Total Requis :", "Total Volume Needed:")}
                                }
                                div { class: "row u-gap-8 u-fwrap-wrap",
                                    span { class: "pill pill-accent", "{req.volume_ml:.2} ml" }
                                    span { class: "pill", "~{req.hazelnuts} ", {locale.tr("noisettes", "hazelnut dabs")} }
                                    span { class: "pill", "{req.matchstick_heads} ", {locale.tr("têtes d'allumette", "matchstick heads")} }
                                }
                            }

                            div { class: "u-fs-13 u-lh-1p5 u-mb-12",
                                "{req.advice}"
                            }

                            div { class: "u-d-grid u-gtc-repeat-3-1fr u-gap-10 u-mt-10 u-pt-12 u-bt-1-solid-border",
                                div { class: "card u-bg-bg u-ta-center",
                                    div { class: "muted u-fs-11", {locale.tr("Palette Ouverte", "Open Palette")} }
                                    div { class: "u-fw-750 u-fs-14 u-mt-2", "~{req.open_palette_hours} h" }
                                }
                                div { class: "card u-bg-bg u-ta-center",
                                    div { class: "muted u-fs-11", {locale.tr("Godet Hermétique", "Sealed Pot")} }
                                    div { class: "u-fw-750 u-fs-14 u-mt-2 u-c-accent", "{req.airtight_pot_days} jours" }
                                }
                                div { class: "card u-bg-bg u-ta-center",
                                    div { class: "muted u-fs-11", {locale.tr("Congélateur (-18°C)", "Freezer (-18°C)")} }
                                    div { class: "u-fw-750 u-fs-14 u-mt-2 u-c-h3b82f6", "{req.freezer_days} jours" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_step_is_done_once_toggled_and_out_of_range_is_not_done() {
        let mut squad = BatchSquad::standard_space_marine_squad("Escouade", 3);
        assert!(!is_done(&squad, 1, 0));
        squad.toggle(1, 0);
        assert!(is_done(&squad, 1, 0));
        assert!(!is_done(&squad, 0, 0));
        assert!(!is_done(&squad, 9, 0));
        assert!(!is_done(&squad, 0, 99));
    }
}
