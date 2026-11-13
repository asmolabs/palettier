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
            div { class: "row-between", style: "margin-bottom: 14px;",
                div {
                    h3 { class: "card-title", style: "margin: 0;",
                        {locale.tr("Peinture d'Armée & Escouade à la Chaîne (Batch Painting)", "Batch Squad Painting Tracker")}
                    }
                    div { class: "muted", style: "font-size: 12px; margin-top: 2px;",
                        {locale.tr("Suivi rigoureux par figurine et par étape pour peindre une escouade homogène.", "Rigorous step-by-step progress tracking to paint an entire homogeneous squad.")}
                    }
                }

                div { class: "row", style: "gap: 12px; align-items: center;",
                    div { class: "pill pill-ok", style: "font-size: 13px; font-weight: 750;",
                        "{squad.read().progress_pct():.0}% ", {locale.tr("terminé", "completed")}
                    }
                    div { class: "pill pill-accent",
                        {locale.tr("Multiplicateur mélange mère : ", "Mother Mix Multiplier: ")},
                        "{squad.read().mother_mix_multiplier():.1}x"
                    }
                }
            }

            div { class: "card-subtle", style: "margin-bottom: 16px;",
                div { class: "row", style: "gap: 16px; align-items: center;",
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
                            style: "width: 80px;",
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

            div { style: "overflow-x: auto;",
                table { class: "data-table",
                    thead {
                        tr {
                            th { style: "width: 140px;", {locale.tr("Figurine", "Miniature")} }
                            for step in &squad.read().steps {
                                th { style: "font-size: 11px; text-align: center;", "{step}" }
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
        td { style: "text-align: center; cursor: pointer;",
            onclick: move |_| squad.write().toggle(m_idx, s_idx),
            span {
                class: if is_done { "pill pill-ok" } else { "pill" },
                style: "padding: 2px 8px; font-size: 11px;",
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
        div { style: "display: flex; flex-direction: column; gap: 16px;",
            div { class: "card",
                div { class: "row-between", style: "margin-bottom: 12px; align-items: center;",
                    h3 { class: "card-title", style: "margin: 0;",
                        "🏺 ", {locale.tr("Calculateur de Mélanges Mères pour Escouades & Armées", "Batch Squad Mother Mix Calculator")}
                    }
                    span { class: "pill pill-accent",
                        {locale.tr("Volumes en Noisettes & Pots Scellés", "Hazelnut Volumes & Sealed Pots")}
                    }
                }
                p { class: "muted", style: "margin-top: 0; font-size: 13px;",
                    {locale.tr(
                        "Préparer un mélange mère en amont garantit une teinte 100% identique sur toute une armée de figurines. Calculez exactement le volume de pâte nécessaire et découvrez les astuces pour le garder frais plusieurs semaines.",
                        "Mixing a mother mix beforehand ensures identical uniform color across an entire army. Compute the exact paste volume needed and tips to keep it usable for weeks."
                    )}
                }

                div { class: "grid-3", style: "gap: 14px; margin-bottom: 16px;",
                    div { class: "card-subtle",
                        label { style: "font-weight: 700; font-size: 12.5px; display: block; margin-bottom: 6px;",
                            {locale.tr("Échelle des figurines :", "Miniature Scale:")}
                        }
                        select {
                            class: "input",
                            value: "{mother_mix_scale.read().label_i18n(locale)}",
                            onchange: move |ev: Event<FormData>| {
                                for s in MiniatureScale::ALL {
                                    if s.label_i18n(locale) == ev.value() {
                                        mother_mix_scale.set(s);
                                        break;
                                    }
                                }
                            },
                            for s in MiniatureScale::ALL {
                                option { value: "{s.label_i18n(locale)}", "{s.label_i18n(locale)}" }
                            }
                        }
                    }

                    div { class: "card-subtle",
                        label { style: "font-weight: 700; font-size: 12.5px; display: block; margin-bottom: 6px;",
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
                        label { style: "font-weight: 700; font-size: 12.5px; display: block; margin-bottom: 6px;",
                            {locale.tr("Surface couverte :", "Covered Surface:")}
                        }
                        button {
                            class: if *mother_mix_full_coverage.read() { "btn-primary" } else { "btn" },
                            style: "width: 100%;",
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
                        div { class: "card-subtle", style: "border-left: 4px solid var(--accent); padding: 16px;",
                            div { class: "row-between", style: "align-items: center; margin-bottom: 12px;",
                                span { style: "font-weight: 750; font-size: 14px; color: var(--accent);",
                                    "📊 ", {locale.tr("Volume Total Requis :", "Total Volume Needed:")}
                                }
                                div { class: "row", style: "gap: 8px; flex-wrap: wrap;",
                                    span { class: "pill pill-accent", "{req.volume_ml:.2} ml" }
                                    span { class: "pill", "~{req.hazelnuts} noisettes" }
                                    span { class: "pill", "{req.matchstick_heads} allumettes" }
                                }
                            }

                            div { style: "font-size: 13px; line-height: 1.5; margin-bottom: 12px;",
                                "{req.advice}"
                            }

                            div { style: "display: grid; grid-template-columns: repeat(3, 1fr); gap: 10px; margin-top: 10px; padding-top: 12px; border-top: 1px solid var(--border);",
                                div { class: "card", style: "background: var(--bg); text-align: center;",
                                    div { class: "muted", style: "font-size: 11px;", {locale.tr("Palette Ouverte", "Open Palette")} }
                                    div { style: "font-weight: 750; font-size: 14px; margin-top: 2px;", "~{req.open_palette_hours} h" }
                                }
                                div { class: "card", style: "background: var(--bg); text-align: center;",
                                    div { class: "muted", style: "font-size: 11px;", {locale.tr("Godet Hermétique", "Sealed Pot")} }
                                    div { style: "font-weight: 750; font-size: 14px; margin-top: 2px; color: var(--accent);", "{req.airtight_pot_days} jours" }
                                }
                                div { class: "card", style: "background: var(--bg); text-align: center;",
                                    div { class: "muted", style: "font-size: 11px;", {locale.tr("Congélateur (-18°C)", "Freezer (-18°C)")} }
                                    div { style: "font-weight: 750; font-size: 14px; margin-top: 2px; color: #3b82f6;", "{req.freezer_days} jours" }
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
