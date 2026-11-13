//! Studio care: sable brushes and varnish.

use crate::state::AppState;
use dioxus::prelude::*;
use palettier_domain::i18n::Locale;
use palettier_domain::miniature::{BrushTracker, VarnishFinish};

#[derive(Clone, Copy)]
pub(super) struct CareState {
    brushes: Signal<Vec<BrushTracker>>,
    varnish: Signal<VarnishFinish>,
}

impl CareState {
    pub(super) fn new() -> Self {
        Self {
            brushes: Signal::new(BrushTracker::default_sable_kit()),
            varnish: Signal::new(VarnishFinish::UltraMatte),
        }
    }
}

#[component]
pub(super) fn BrushCare() -> Element {
    let state = use_context::<AppState>();
    let brushes = use_context::<CareState>().brushes;
    let locale = *state.locale.read();

    rsx! {
        div { class: "card",
            div { class: "row-between u-mb-14",
                div {
                    h3 { class: "card-title u-m-0", {locale.tr("Soin des Pinceaux de Martre Kolinsky & Entretien", "Kolinsky Sable Brush Care & Maintenance")} }
                    div { class: "muted u-fs-12 u-mt-2",
                        {locale.tr("Préservez la pointe fine de vos pinceaux les plus précieux face aux solvants et à l'huile.", "Protect the fine needle tip of your sable brushes from solvent wear.")}
                    }
                }
            }

            div { class: "grid-2 u-gtc-1p2fr-0p8fr u-gap-16",
                div { class: "u-d-flex u-fd-column u-gap-8",
                    for (idx, brush) in brushes.read().iter().enumerate() {
                        {brush_row(brushes, locale, idx, brush)}
                    }
                }

                div { class: "card-subtle",
                    div { class: "u-fw-700 u-fs-13 u-mb-8 u-c-danger",
                        "⚠️ ", {locale.tr("Règles d'or de longévité du poil de martre :", "Golden Rules of Sable Brush Longevity:")}
                    }
                    ul { class: "u-ml-20 u-fs-12p5 u-lh-1p5 u-d-flex u-fd-column u-gap-8",
                        li {
                            strong { {locale.tr("Ne JAMAIS tremper la virole : ", "NEVER submerge the ferrule: ")} }
                            {locale.tr("L'essence sans odeur pénètre par capillarité sous la bague en métal et détruit la colle liant les poils.", "Spirits creep inside the metal ferrule and dissolve the binding glue.")}
                        }
                        li {
                            strong { {locale.tr("Savon de conditionnement régulier : ", "Regular conditioning soap: ")} }
                            {locale.tr("Laver au savon Masters Brush Cleaner ou savon de Marseille toutes les 6 à 8 séances pour réhydrater le poil naturel.", "Wash with Masters Brush Cleaner or olive oil soap every 6-8 sessions to restore natural oils.")}
                        }
                        li {
                            strong { {locale.tr("Séchage horizontal ou tête en bas : ", "Dry horizontal or tip down: ")} }
                            {locale.tr("Ne jamais laisser sécher un pinceau la pointe vers le haut avec du solvant résiduel.", "Never leave a brush drying upright with solvent residues.")}
                        }
                    }
                }
            }
        }
    }
}

/// Counts one more session for brush `idx`.
fn add_use(brushes: &mut [BrushTracker], idx: usize) {
    if let Some(b) = brushes.get_mut(idx) {
        b.uses_count += 1;
    }
}

/// Brush `idx` was washed: its session count starts again.
fn mark_cleaned(brushes: &mut [BrushTracker], idx: usize) {
    if let Some(b) = brushes.get_mut(idx) {
        b.uses_count = 0;
    }
}

fn brush_row(mut brushes: Signal<Vec<BrushTracker>>, locale: Locale, idx: usize, brush: &BrushTracker) -> Element {
    let needs_soap = brush.needs_deep_clean();
    rsx! {
        div { class: "card-subtle u-p-10-14",
            div { class: "row-between u-ai-center",
                div {
                    strong { "{brush.name} ({brush.size})" }
                    div { class: "muted u-fs-11", "{brush.brand} · {brush.hair_type}" }
                }
                div { class: "row u-gap-8 u-ai-center",
                    span { class: if needs_soap { "pill pill-warn" } else { "pill pill-ok" },
                        "{brush.uses_count} ", {locale.tr("sessions", "sessions")}
                    }
                    button {
                        class: "btn",
                        onclick: move |_| add_use(&mut brushes.write(), idx),
                        {locale.tr("+ 1 utilisation", "+ 1 use")}
                    }
                    button {
                        class: "btn",
                        onclick: move |_| mark_cleaned(&mut brushes.write(), idx),
                        "🧼 ", {locale.tr("Nettoyé", "Cleaned")}
                    }
                }
            }
        }
    }
}

#[component]
pub(super) fn VarnishStudio() -> Element {
    let state = use_context::<AppState>();
    let mut selected_varnish = use_context::<CareState>().varnish;
    let locale = *state.locale.read();

    rsx! {
        div { class: "grid-2 u-gtc-1fr-1p25fr",
            div { class: "card",
                h3 { class: "card-title", {locale.tr("Simulateur d'Harmonisation par Vernis", "Varnish Finish Harmonizer")} }
                p { class: "muted u-fs-12 u-mb-14",
                    {locale.tr("L'alliance acrylique et huile crée des disparités de brillance. Le vernis final scelle et harmonise l'ensemble de la figurine.", "Mixing acrylics and oils causes uneven shine. The final varnish unifies the entire miniature.")}
                }

                div { class: "u-d-flex u-fd-column u-gap-8",
                    for v in VarnishFinish::ALL {
                        {
                            let is_sel = *selected_varnish.read() == v;
                            rsx! {
                                div {
                                    class: if is_sel { "card-subtle active" } else { "card-subtle" },
                                    style: if is_sel { "border-color: var(--accent); cursor: pointer; padding: 10px;" } else { "cursor: pointer; padding: 10px;" },
                                    onclick: move |_| selected_varnish.set(v),
                                    div { class: "u-fw-700 u-fs-13", "{v.label(locale)}" }
                                }
                            }
                        }
                    }
                }
            }

            div { class: "card",
                {
                    let v = *selected_varnish.read();
                    rsx! {
                        h3 { class: "card-title", "{v.label(locale)}" }
                        div { class: "card-subtle u-fs-13p5 u-lh-1p5 u-mb-16",
                            "{v.description_i18n(locale)}"
                        }

                        div { class: "card-subtle",
                            div { class: "u-fw-700 u-fs-12p5 u-mb-6",
                                {locale.tr("💡 Conseil de pro pour figurines de concours :", "💡 Competition Miniature Pro Tip:")}
                            }
                            p { class: "muted u-m-0 u-fs-12 u-lh-1p45",
                                {locale.tr(
                                    "Appliquez un vernis Ultra-Mat (AK Ultra Matte) à l'aérographe sur toute la figurine pour tuer les reflets et sublimer les fondus. Puis, repassez au pinceau fin avec un vernis brillant pur UNIQUEMENT sur les yeux, lentilles de visée et sang frais.",
                                    "Airbrush an Ultra-Matte varnish over the entire miniature to kill gloss and maximize contrast. Then, selectively brush pure gloss varnish ONLY on eye lenses, cockpit glass, and fresh gore."
                                )}
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
    fn a_session_counts_and_cleaning_resets() {
        let mut kit = BrushTracker::default_sable_kit();
        let before = kit[0].uses_count;
        add_use(&mut kit, 0);
        assert_eq!(before + 1, kit[0].uses_count);
        mark_cleaned(&mut kit, 0);
        assert_eq!(0, kit[0].uses_count);
        // Out of range: nothing happens.
        let copy = kit.clone();
        add_use(&mut kit, 99);
        mark_cleaned(&mut kit, 99);
        assert_eq!(copy, kit);
    }
}
