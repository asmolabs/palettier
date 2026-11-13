//! Pin wash: wash tone and dilution, and the reductive cleaning timer.

use super::shared::{best_mix, rgb_or_grey};
use crate::state::{AppServices, AppState};
use crate::theme::{ColorSwatch, DeltaEBadge};
use chrono::{DateTime, Utc};
use dioxus::prelude::*;
use palettier_domain::i18n::Locale;
use palettier_domain::miniature::PinWashConfig;
use palettier_domain::paint::Paint;

#[derive(Clone, Copy)]
pub(super) struct PinWashState {
    solvent_ratio: Signal<u32>,
    tone_idx: Signal<usize>,
    timer_start: Signal<Option<DateTime<Utc>>>,
}

impl PinWashState {
    pub(super) fn new() -> Self {
        Self {
            solvent_ratio: Signal::new(6),
            tone_idx: Signal::new(0),
            timer_start: Signal::new(None),
        }
    }
}

/// The colour of wash tone `idx` (black, burnt umber, navy).
fn wash_hex(idx: usize) -> &'static str {
    match idx {
        0 => "#111111",
        1 => "#3B2215",
        _ => "#121D28",
    }
}

/// Where the wash stands `elapsed_mins` after it was laid: the pill class and its text.
fn cleaning_stage(elapsed_mins: i64, locale: Locale) -> (&'static str, &'static str) {
    if elapsed_mins < 15 {
        ("pill-warn", locale.tr("Phase 1 : Évaporation du solvant (Ne pas toucher)", "Stage 1: Solvent evaporating (Do not touch)"))
    } else if elapsed_mins <= 90 {
        ("pill-ok", locale.tr("Phase 2 : FENÊTRE D'OR DE NETTOYAGE RÉDUCTIF !", "Stage 2: GOLDEN REDUCTIVE CLEANING WINDOW!"))
    } else {
        ("pill-danger", locale.tr("Phase 3 : Polymérisation amorcée (Fixation)", "Stage 3: Curing started (Permanent set)"))
    }
}

#[component]
pub(super) fn PinWash(candidates: Vec<Paint>) -> Element {
    let state = use_context::<AppState>();
    let services = use_context::<AppServices>();
    let PinWashState { solvent_ratio: mut wash_solvent_ratio, tone_idx: mut wash_tone_idx, timer_start: mut wash_timer_start } =
        use_context::<PinWashState>();
    let locale = *state.locale.read();
    let wash_config = PinWashConfig {
        ratio_oil: 1,
        ratio_solvent: *wash_solvent_ratio.read(),
        min_cleaning_window_mins: 15,
        max_cleaning_window_mins: 90,
    };
    let sugg = best_mix(&services, rgb_or_grey(wash_hex(*wash_tone_idx.read()), 0.1), &candidates);
    // Redraw on its own while the timer runs: the minutes move without any click.
    let mut tick = use_signal(|| 0u64);
    use_future(move || async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(15)).await;
            if wash_timer_start.peek().is_some() {
                tick += 1;
            }
        }
    });
    let _ = tick();
    let elapsed = (*wash_timer_start.read()).map(|start| (Utc::now() - start).num_minutes());

    rsx! {
        div { class: "grid-2 u-gtc-1fr-1fr",
            div { class: "card",
                h3 { class: "card-title", {locale.tr("Dosage de Dilution & Teinte de Lavis", "Dilution Ratio & Wash Tone")} }
                p { class: "muted u-fs-12 u-mb-14",
                    {locale.tr("Le secret d'un pin wash réussi réside dans le bon ratio d'essence minérale sans odeur pour que le pigment glisse sans bavure.", "The key to a clean pin wash is the exact ratio of odorless mineral spirits so pigment flows instantly.")}
                }

                // Tone selector
                div { class: "row u-gap-6 u-mb-14",
                    for (idx, (t_name, _t_hex)) in [
                        (locale.tr("Noir Profond", "Deep Black"), "#111111"),
                        (locale.tr("Terre d'Ombre Brûlée", "Burnt Umber"), "#3B2215"),
                        (locale.tr("Bleu Marine Sombre", "Navy Blue"), "#121D28"),
                    ].iter().enumerate() {
                        {
                            let is_sel = *wash_tone_idx.read() == idx;
                            rsx! {
                                button {
                                    class: if is_sel { "pill pill-accent" } else { "pill" },
                                    onclick: move |_| wash_tone_idx.set(idx),
                                    "{t_name}"
                                }
                            }
                        }
                    }
                }

                // Calculated Wash Recipe
                div { class: "card-subtle u-mb-14 u-b-1-solid-accent-border",
                    div { class: "u-fw-700 u-fs-12 u-mb-6 u-c-accent",
                        {locale.tr("🧪 Recette de pin wash calculée avec votre palette :", "🧪 Calculated Pin Wash from your palette:")}
                    }
                    if let Some(ref s) = sugg {
                        div { class: "row-between u-ai-center",
                            div { class: "row u-gap-10 u-ai-center",
                                ColorSwatch { hex: s.color.to_hex(), size: 28 }
                                div {
                                    div { class: "u-fw-700 u-fs-12p5", "{s.describe_i18n(locale)}" }
                                    div { class: "muted u-fs-11",
                                        {locale.tr("Diluer dans ", "Dilute with ")},
                                        strong { "{wash_solvent_ratio} parts " },
                                        {locale.tr("d'essence sans odeur", "of odorless spirits")}
                                    }
                                }
                            }
                            DeltaEBadge { delta_e: s.delta_e }
                        }
                    } else {
                        span { class: "muted u-fs-11", {locale.tr("Aucun tube disponible dans la palette", "No tubes available in palette")} }
                    }
                }

                div { class: "card-subtle u-mb-14",
                    div { class: "row-between u-mb-8",
                        strong { {locale.tr("Ratio de Solvant :", "Solvent Ratio:")} }
                        span { class: "pill pill-accent", {locale.tr("1 part d'huile : ", "1 part oil : ")}, "{wash_solvent_ratio}", {locale.tr(" parts de diluant", " parts thinner")} }
                    }
                    input {
                        r#type: "range",
                        class: "u-w-100pct",
                        min: "1",
                        max: "15",
                        value: "{wash_solvent_ratio}",
                        oninput: move |ev| {
                            if let Ok(v) = ev.value().parse::<u32>() {
                                wash_solvent_ratio.set(v);
                            }
                        }
                    }
                    div { class: "u-fw-600 u-fs-12p5 u-mt-8",
                        "{wash_config.capillary_flow_desc(locale)}"
                    }
                }

                div { class: "row u-gap-8 u-fwrap-wrap",
                    button { class: "btn", onclick: move |_| wash_solvent_ratio.set(3), {locale.tr("Creux profonds (1:3)", "Deep recesses (1:3)")} }
                    button { class: "btn-primary", onclick: move |_| wash_solvent_ratio.set(6), {locale.tr("Pin wash standard (1:6)", "Standard Armor (1:6)")} }
                    button { class: "btn", onclick: move |_| wash_solvent_ratio.set(12), {locale.tr("Filtre d'ambiance (1:12)", "Glaze Filter (1:12)")} }
                }
            }

            div { class: "card",
                h3 { class: "card-title", {locale.tr("Fenêtre de Nettoyage Réductif", "Reductive Cleaning Window")} }
                div { class: "card-subtle u-mb-16 u-ta-center u-p-20",
                    if let Some(elapsed_mins) = elapsed {
                        {
                            let (status_class, status_text) = cleaning_stage(elapsed_mins, locale);
                            rsx! {
                                div { class: "mono u-fs-28 u-fw-800 u-mb-8", "{elapsed_mins} min" }
                                div { class: "pill {status_class}", class: "u-p-4-12 u-fs-12 u-fw-700", "{status_text}" }
                                div { class: "u-mt-14",
                                    button { class: "btn", onclick: move |_| wash_timer_start.set(None), {locale.tr("Réinitialiser le chrono", "Reset Timer")} }
                                }
                            }
                        }
                    } else {
                        div { class: "u-fs-14 u-fw-600 u-mb-10", {locale.tr("Chronomètre de séance inactif", "Session timer stopped")} }
                        button { class: "btn-primary", onclick: move |_| wash_timer_start.set(Some(Utc::now())), "⏱️ ", {locale.tr("Démarrer la pose du Pin Wash", "Start Pin Wash Application")} }
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
    fn the_cleaning_window_opens_at_15_minutes_and_closes_after_90() {
        let class = |m| cleaning_stage(m, Locale::Fr).0;
        assert_eq!("pill-warn", class(0));
        assert_eq!("pill-warn", class(14));
        assert_eq!("pill-ok", class(15));
        assert_eq!("pill-ok", class(90));
        assert_eq!("pill-danger", class(91));
    }

    #[test]
    fn each_tone_has_its_colour_and_navy_is_the_fallback() {
        assert_eq!("#111111", wash_hex(0));
        assert_eq!("#3B2215", wash_hex(1));
        assert_eq!("#121D28", wash_hex(2));
        assert_eq!("#121D28", wash_hex(7));
    }
}
