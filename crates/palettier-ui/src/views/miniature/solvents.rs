//! Solvents & stripping: the stripping safety matrix, solvents and mediums, and cold
//! storage of the palette.

use crate::state::AppState;
use dioxus::prelude::*;
use palettier_domain::i18n::Locale;
use palettier_domain::miniature::{
    FreezerStorageSimulator, MiniatureMaterial, SolventType, StrippingAgent, StrippingGuide,
    StrippingVerdictSeverity,
};
use palettier_domain::paint::{Medium, PaintSystem};

#[derive(Clone, Copy)]
pub(super) struct SolventsState {
    material: Signal<MiniatureMaterial>,
    stripper: Signal<StrippingAgent>,
    ambient_temp: Signal<f64>,
    airtight: Signal<bool>,
    in_freezer: Signal<bool>,
}

impl SolventsState {
    pub(super) fn new() -> Self {
        Self {
            material: Signal::new(MiniatureMaterial::PolystyrenePlastic),
            stripper: Signal::new(StrippingAgent::Ipa99),
            ambient_temp: Signal::new(20.0),
            airtight: Signal::new(true),
            in_freezer: Signal::new(true),
        }
    }
}

/// Value of a material in its select.
fn material_value(material: MiniatureMaterial) -> &'static str {
    match material {
        MiniatureMaterial::PolystyrenePlastic => "plastic",
        MiniatureMaterial::ResinSla3d => "resin",
        MiniatureMaterial::WhiteMetal => "metal",
    }
}

/// The material of a select value; plastic for anything unknown.
fn material_of(value: &str) -> MiniatureMaterial {
    match value {
        "resin" => MiniatureMaterial::ResinSla3d,
        "metal" => MiniatureMaterial::WhiteMetal,
        _ => MiniatureMaterial::PolystyrenePlastic,
    }
}

/// Value of a stripping agent in its select.
fn stripper_value(agent: StrippingAgent) -> &'static str {
    match agent {
        StrippingAgent::Ipa99 => "ipa",
        StrippingAgent::Biostrip20 => "biostrip",
        StrippingAgent::Glanzer => "glanzer",
        StrippingAgent::PureAcetone => "acetone",
    }
}

/// The stripping agent of a select value; IPA for anything unknown.
fn stripper_of(value: &str) -> StrippingAgent {
    match value {
        "biostrip" => StrippingAgent::Biostrip20,
        "glanzer" => StrippingAgent::Glanzer,
        "acetone" => StrippingAgent::PureAcetone,
        _ => StrippingAgent::Ipa99,
    }
}

/// Border colour, background and badge class of a stripping verdict.
fn verdict_style(severity: StrippingVerdictSeverity) -> (&'static str, &'static str, &'static str) {
    match severity {
        StrippingVerdictSeverity::Safe => ("var(--accent-ok)", "background: rgba(34, 197, 94, 0.08);", "pill pill-ok"),
        StrippingVerdictSeverity::Caution => ("#F59E0B", "background: rgba(245, 158, 11, 0.08);", "pill pill-warn"),
        StrippingVerdictSeverity::Forbidden => ("#EF4444", "background: rgba(239, 68, 68, 0.12);", "pill pill-danger"),
    }
}

/// How long mixes keep, in weeks past a week, in days below.
fn usable_days_text(usable_days: f64, locale: Locale) -> String {
    if usable_days >= 7.0 {
        match locale {
            Locale::Fr => format!("{:.0} semaines (environ {:.0} jours)", usable_days / 7.0, usable_days),
            Locale::En => format!("{:.0} weeks (about {:.0} days)", usable_days / 7.0, usable_days),
        }
    } else {
        match locale {
            Locale::Fr => format!("{:.1} jours", usable_days),
            Locale::En => format!("{:.1} days", usable_days),
        }
    }
}

#[component]
pub(super) fn SolventsAndStripping() -> Element {
    let state = use_context::<AppState>();
    let tool = use_context::<SolventsState>();
    let locale = *state.locale.read();

    rsx! {
        div { class: "u-d-flex u-fd-column u-gap-16",
            // Matrice de Décapage Sécurisé
            {stripping_card(tool, locale)}

            // Guide Solvants & Médiums
            div { class: "grid-2 u-gap-16",
                // Solvants
                {solvents_card(locale)}

                // Médiums
                {mediums_card(locale)}
            }

            // Simulateur Conservation au Froid (Congélateur)
            {freezer_card(tool, locale)}
        }
    }
}

fn stripping_card(tool: SolventsState, locale: Locale) -> Element {
    let SolventsState { material: mut selected_material, stripper: mut selected_stripper, .. } = tool;
    let verdict = StrippingGuide::evaluate(*selected_material.read(), *selected_stripper.read());
    let (border_color, bg_style, badge_cls) = verdict_style(verdict.severity);
    let advice = if locale.is_fr() { verdict.advice_fr } else { verdict.advice_en };
    rsx! {
        div { class: "card",
            h3 { class: "card-title", {locale.tr("🧼 Matrice de Décapage Sécurisé par Matériau", "🧼 Miniature Paint Stripping Safety Matrix")} }
            p { class: "muted u-fs-12 u-mb-14",
                {locale.tr("Calculateur de sauvetage : vérifiez la compatibilité chimique pour décaper vos figurines sans risquer la destruction irréversible des détails ou la fonte du plastique.", "Rescue calculator: verify chemical safety before stripping miniatures to avoid melting fine plastics or warping resins.")}
            }

            div { class: "grid-2 u-gap-16 u-mb-16",
                div {
                    label { class: "u-fs-12 u-fw-600 u-d-block u-mb-6",
                        {locale.tr("Matériau de la figurine :", "Miniature Material:")}
                    }
                    select {
                        class: "input",
                        value: material_value(*selected_material.read()),
                        onchange: move |ev| selected_material.set(material_of(&ev.value())),
                        option { value: "plastic", "{MiniatureMaterial::PolystyrenePlastic.label(locale)}" }
                        option { value: "resin", "{MiniatureMaterial::ResinSla3d.label(locale)}" }
                        option { value: "metal", "{MiniatureMaterial::WhiteMetal.label(locale)}" }
                    }
                }

                div {
                    label { class: "u-fs-12 u-fw-600 u-d-block u-mb-6",
                        {locale.tr("Agent / Produit décapant :", "Chemical Stripping Agent:")}
                    }
                    select {
                        class: "input",
                        value: stripper_value(*selected_stripper.read()),
                        onchange: move |ev| selected_stripper.set(stripper_of(&ev.value())),
                        option { value: "ipa", "{StrippingAgent::Ipa99.label(locale)}" }
                        option { value: "biostrip", "{StrippingAgent::Biostrip20.label(locale)}" }
                        option { value: "glanzer", "{StrippingAgent::Glanzer.label(locale)}" }
                        option { value: "acetone", "{StrippingAgent::PureAcetone.label(locale)}" }
                    }
                }
            }

            // Stripping Verdict Card
            div {
                style: "border: 1.5px solid {border_color}; border-radius: 8px; padding: 14px; {bg_style}",
                div { class: "row-between u-ai-center u-mb-8",
                    strong { class: "u-fs-14 u-ls-0p5", "{verdict.title}" }
                    span { class: "{badge_cls}", {locale.tr("Durée conseillée : ", "Recommended time: ")}, "{verdict.recommended_duration}" }
                }
                p { class: "u-m-0 u-fs-13 u-lh-1p45", "{advice}" }
            }
        }
    }
}

fn solvents_card(locale: Locale) -> Element {
    rsx! {
        div { class: "card",
            h3 { class: "card-title", {locale.tr("🧪 Guide des Solvants pour Lavis d'Huile", "🧪 Oil Solvents & Wash Dilution Guide")} }
            div { class: "u-d-flex u-fd-column u-gap-10",
                for sol in SolventType::ALL {
                    {
                        let (safety_text, badge_type) = sol.acrylic_safety_level_i18n(locale);
                        let badge_cls = match badge_type {
                            "ok" => "pill pill-ok",
                            "warn" => "pill pill-warn",
                            _ => "pill pill-danger",
                        };
                        rsx! {
                            div { class: "card-subtle u-p-10",
                                div { class: "row-between u-mb-4",
                                    strong { class: "u-fs-13", "{sol.name()}" }
                                    span { class: "{badge_cls}", "{safety_text}" }
                                }
                                div { class: "muted u-fs-11p5 u-mb-4",
                                    "⏱️ {sol.evaporation_speed_i18n(locale)}"
                                }
                                div { class: "u-fs-11 u-op-0p85",
                                    "🫁 {sol.odor_and_toxicity_i18n(locale)}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn mediums_card(locale: Locale) -> Element {
    rsx! {
        div { class: "card",
            h3 { class: "card-title", {locale.tr("🍯 Médiums Spécialisés Figurines", "🍯 Specialized Miniature Oil Mediums")} }
            div { class: "u-d-flex u-fd-column u-gap-10",
                // The same mediums, factors and limits as the drying model.
                for med in Medium::for_system(PaintSystem::Oil).into_iter().filter(|m| *m != Medium::None) {
                    {medium_row(locale, med)}
                }
            }
        }
    }
}

fn medium_row(locale: Locale, med: Medium) -> Element {
    let max = med.max_useful_ratio();
    let factor = med.drying_factor(max);
    rsx! {
        div { class: "card-subtle u-p-10",
            div { class: "row-between u-mb-4",
                strong { class: "u-fs-13", "{med.label(locale)}" }
                span {
                    class: if factor < 1.0 { "pill pill-ok" } else { "pill pill-warn" },
                    if factor < 1.0 {
                        {locale.tr("accélère", "speeds up")}
                    } else {
                        {locale.tr("ralentit", "slows down")}
                    }
                    " ×{factor:.2}"
                }
            }
            div { class: "muted u-fs-11p5 u-mb-4",
                "{med.advice(locale)}"
            }
            div { class: "u-fs-11 u-c-accent u-fw-600",
                "📏 ", {locale.tr("Au plus ", "At most ")}, "{(max * 100.0).round()} %"
            }
        }
    }
}

fn freezer_card(tool: SolventsState, locale: Locale) -> Element {
    let SolventsState {
        ambient_temp: mut freezer_ambient_temp,
        airtight: mut freezer_airtight,
        in_freezer: mut freezer_is_freezer,
        ..
    } = tool;
    let usable_days = FreezerStorageSimulator::estimated_usable_days(
        *freezer_ambient_temp.read(),
        *freezer_airtight.read(),
        *freezer_is_freezer.read(),
    );
    rsx! {
        div { class: "card",
            h3 { class: "card-title", {locale.tr("❄️ Conservation de Palette d'Huile au Froid / Congélateur", "❄️ Oil Palette Preservation & Freezer Storage")} }
            p { class: "muted u-fs-12 u-mb-14",
                {locale.tr("Contrairement aux acryliques, l'huile ne sèche pas par évaporation mais par oxydo-polymérisation. Abaisser la température ralentit exponentiellement la réaction chimique (loi d'Arrhenius).", "Unlike acrylics, oils cure via oxidation rather than evaporation. Freezing drastically slows the cross-linking reaction rate.")}
            }

            div { class: "grid-3 u-gap-14 u-mb-14",
                div { class: "card-subtle",
                    label { class: "row u-gap-8 u-ai-center u-cur-pointer u-fs-13 u-fw-600",
                        input {
                            r#type: "checkbox",
                            checked: *freezer_airtight.read(),
                            onchange: move |ev| freezer_airtight.set(ev.value().parse::<bool>().unwrap_or(true)),
                        }
                        span { {locale.tr("Boîte hermétique (Palette Seal)", "Airtight Palette Seal Box")} }
                    }
                    p { class: "muted u-fs-11 u-mt-6 u-mb-0",
                        {locale.tr("Empêche le renouvellement d'oxygène frais.", "Prevents fresh atmospheric oxygen exposure.")}
                    }
                }

                div { class: "card-subtle",
                    label { class: "row u-gap-8 u-ai-center u-cur-pointer u-fs-13 u-fw-600",
                        input {
                            r#type: "checkbox",
                            checked: *freezer_is_freezer.read(),
                            onchange: move |ev| freezer_is_freezer.set(ev.value().parse::<bool>().unwrap_or(true)),
                        }
                        span { {locale.tr("Placer au congélateur (-18°C)", "Store in Freezer (-18°C)")} }
                    }
                    p { class: "muted u-fs-11 u-mt-6 u-mb-0",
                        {locale.tr("L'huile d'artiste pure ne gèle pas en bloc à -18°C.", "Pure oil paint does not freeze solid at -18°C.")}
                    }
                }

                div { class: "card-subtle",
                    div { class: "u-fw-600 u-fs-12 u-mb-4",
                        {locale.tr("Température ambiante de l'atelier :", "Workshop Ambient Temperature:")}
                    }
                    div { class: "row-between",
                        span { class: "mono u-fs-13", "{freezer_ambient_temp} °C" }
                    }
                    input {
                        r#type: "range",
                        min: "15",
                        max: "35",
                        step: "1",
                        value: "{freezer_ambient_temp}",
                        oninput: move |ev| {
                            if let Ok(v) = ev.value().parse::<f64>() {
                                freezer_ambient_temp.set(v);
                            }
                        },
                        class: "u-w-100pct u-mt-6",
                    }
                }
            }

            div { class: "card-subtle u-b-1-solid-accent-border",
                div { class: "row-between u-ai-center",
                    div {
                        strong { class: "u-c-accent u-fs-14",
                            {locale.tr("Temps de réutilisation garanti de vos mélanges :", "Estimated Safe Working Palette Lifetime:")}
                        }
                        div { class: "muted u-fs-12 u-mt-2",
                            {
                                if *freezer_is_freezer.read() {
                                    locale.tr("Sortez la boîte du congélateur 10 minutes avant de peindre pour la laisser revenir à température ambiante.", "Remove palette from freezer 10 minutes before painting to let it warm up.")
                                } else {
                                    locale.tr("À l'air libre, les couches fines forment une pellicule sèche en quelques heures.", "Open to room air, thin films skin over within hours.")
                                }
                            }
                        }
                    }
                    span { class: "pill pill-accent u-fs-14 u-p-6-14 u-fw-750",
                        {usable_days_text(usable_days, locale)}
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
    fn select_values_round_trip() {
        for m in [
            MiniatureMaterial::PolystyrenePlastic,
            MiniatureMaterial::ResinSla3d,
            MiniatureMaterial::WhiteMetal,
        ] {
            assert_eq!(m, material_of(material_value(m)));
        }
        for a in [
            StrippingAgent::Ipa99,
            StrippingAgent::Biostrip20,
            StrippingAgent::Glanzer,
            StrippingAgent::PureAcetone,
        ] {
            assert_eq!(a, stripper_of(stripper_value(a)));
        }
        assert_eq!(MiniatureMaterial::PolystyrenePlastic, material_of("?"));
        assert_eq!(StrippingAgent::Ipa99, stripper_of("?"));
    }

    #[test]
    fn usable_time_reads_in_weeks_past_a_week() {
        assert_eq!("2.5 jours", usable_days_text(2.5, Locale::Fr));
        assert_eq!("2 semaines (environ 14 jours)", usable_days_text(14.0, Locale::Fr));
        assert_eq!("2 weeks (about 14 days)", usable_days_text(14.0, Locale::En));
    }
}
