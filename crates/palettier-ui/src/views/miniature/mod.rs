#![forbid(unsafe_code)]

//! Dedicated Miniature & Weathering Studio view:
//! Acrylic-to-oil converter, 5-stage contrast ladders, NMM metal studio, grimdark weathering,
//! pin wash & reductive cleaning timer, heat box forced curing, special effects gore,
//! Kolinsky brush care, batch squad painting, and OSL light engines.
//! Every tool dynamically computes Kubelka-Munk subtractive oil mixes from the user's chosen palette!
//!
//! This module holds the tool list, the palette banner and the side navigation; each tool
//! is a component in its own sub-module. The tools' state is created here and shared by
//! context, so a tool keeps what was typed in it while the painter visits another one.

mod batch;
mod care;
mod consistency;
mod converter;
mod drying;
mod glaze;
mod harmonies;
mod nmm;
mod osl;
mod palette_layout;
mod pigments;
mod pin_wash;
mod recipe_sheet;
mod shared;
mod solvents;
mod vehicle;
mod weathering;

use crate::state::AppState;
use crate::theme::SectionHeader;
use dioxus::prelude::*;
use palettier_domain::i18n::Locale;
use palettier_domain::paint::{Paint, PaintSystem};
use shared::{active_candidates, oil_candidates, working_palettes};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MiniatureTab {
    Converter,
    NmmStudio,
    Weathering,
    PinWash,
    SolventsAndStripping,
    Pigments,
    RecipeSheet,
    HeatBox,
    SpecialEffects,
    BrushCare,
    VarnishStudio,
    BatchSquad,
    OslEngine,
    OilHarmonies,
    OilConsistency,
    GlazePreview,
    PaletteLayout,
    VehicleWeathering,
    WetOnWetBlending,
    BatchMotherMix,
    PaletteFreshness,
}

impl MiniatureTab {
    pub const ALL: [Self; 21] = [
        Self::Converter,
        Self::OilHarmonies,
        Self::PaletteLayout,
        Self::OslEngine,
        Self::NmmStudio,
        Self::PinWash,
        Self::GlazePreview,
        Self::OilConsistency,
        Self::WetOnWetBlending,
        Self::Weathering,
        Self::VehicleWeathering,
        Self::Pigments,
        Self::SpecialEffects,
        Self::HeatBox,
        Self::PaletteFreshness,
        Self::BatchSquad,
        Self::BatchMotherMix,
        Self::RecipeSheet,
        Self::VarnishStudio,
        Self::SolventsAndStripping,
        Self::BrushCare,
    ];

    pub fn icon(self) -> &'static str {
        match self {
            Self::Converter => "🎨",
            Self::NmmStudio => "⚔️",
            Self::Weathering => "🌧️",
            Self::PinWash => "🛡️",
            Self::SolventsAndStripping => "🧪",
            Self::Pigments => "🏜️",
            Self::RecipeSheet => "📄",
            Self::HeatBox => "♨️",
            Self::SpecialEffects => "🩸",
            Self::BrushCare => "🖌️",
            Self::VarnishStudio => "🪞",
            Self::BatchSquad => "👥",
            Self::OslEngine => "💡",
            Self::OilHarmonies => "🎡",
            Self::OilConsistency => "💧",
            Self::GlazePreview => "🖼️",
            Self::PaletteLayout => "🗺️",
            Self::VehicleWeathering => "🪖",
            Self::WetOnWetBlending => "〰️",
            Self::BatchMotherMix => "🏺",
            Self::PaletteFreshness => "⏳",
        }
    }

    pub fn label(self, locale: Locale) -> &'static str {
        match self {
            Self::Converter => locale.tr("Acrylique → huile & échelle 5 tons", "Acrylic → oil & 5-tone ladder"),
            Self::NmmStudio => locale.tr("Métal non métallique (NMM)", "Non-metallic metal (NMM)"),
            Self::Weathering => locale.tr("Usure & grimdark", "Weathering & grimdark"),
            Self::PinWash => locale.tr("Jus capillaire & retrait", "Pin wash & reductive"),
            Self::SolventsAndStripping => locale.tr("Solvants & décapage", "Solvents & stripping"),
            Self::Pigments => locale.tr("Pigments secs & fixateurs", "Dry pigments & fixers"),
            Self::RecipeSheet => locale.tr("Fiche recette d'armée", "Army recipe sheet"),
            Self::HeatBox => locale.tr("Boîte chauffante", "Heat box curing"),
            Self::SpecialEffects => locale.tr("Sang, glace & effets", "Blood, ice & FX"),
            Self::BrushCare => locale.tr("Soin des pinceaux", "Brush care"),
            Self::VarnishStudio => locale.tr("Vernis", "Varnish"),
            Self::BatchSquad => locale.tr("Escouade en série", "Batch squad"),
            Self::OslEngine => locale.tr("Source de lumière (OSL)", "Light source (OSL)"),
            Self::OilHarmonies => locale.tr("Harmonies de couleurs", "Colour harmonies"),
            Self::OilConsistency => locale.tr("Consistance & gouttes", "Consistency & drops"),
            Self::GlazePreview => locale.tr("Glacis sur zénithale", "Glaze over zenithal"),
            Self::PaletteLayout => locale.tr("Disposition de la palette", "Palette layout"),
            Self::VehicleWeathering => locale.tr("Jus & filtres véhicules", "Vehicle washes & filters"),
            Self::WetOnWetBlending => locale.tr("Fondu dans le frais", "Wet-on-wet blending"),
            Self::BatchMotherMix => locale.tr("Mélanges mères d'escouade", "Squad mother mixes"),
            Self::PaletteFreshness => locale.tr("Fraîcheur de la palette", "Palette freshness"),
        }
    }

    /// One line saying what the tool answers, shown under its title and as a tooltip.
    pub fn blurb(self, locale: Locale) -> &'static str {
        match self {
            Self::Converter => locale.tr("Retrouver à l'huile la teinte d'une acrylique de référence, et son échelle ombre → lumière.", "Match a reference acrylic in oil, with its shadow → light ladder."),
            Self::NmmStudio => locale.tr("Les cinq tons d'un or, d'un acier ou d'un cuivre peints sans peinture métallique.", "The five tones of gold, steel or copper painted without metallic paint."),
            Self::Weathering => locale.tr("Poussière, boue, rouille, huile fraîche : teinte et façon de poser.", "Dust, mud, rust, fresh oil: tone and how to apply it."),
            Self::PinWash => locale.tr("Dosage et temps de retrait d'un jus dans les creux.", "Ratio and wipe-back time of a wash in the recesses."),
            Self::SolventsAndStripping => locale.tr("Quel solvant sur quel matériau, et comment décaper sans abîmer.", "Which solvent on which material, and how to strip safely."),
            Self::Pigments => locale.tr("Pigments en poudre et leur fixation.", "Powder pigments and how to fix them."),
            Self::RecipeSheet => locale.tr("Une fiche imprimable pour peindre toute une armée à l'identique.", "A printable sheet to paint a whole army the same way."),
            Self::HeatBox => locale.tr("Accélérer le séchage de l'huile par la chaleur, sans déformer le plastique.", "Speed up oil drying with heat without warping plastic."),
            Self::SpecialEffects => locale.tr("Sang frais, sang séché, glace, slime : recettes et pose.", "Fresh and dried blood, ice, slime: recipes and application."),
            Self::BrushCare => locale.tr("Nettoyer et faire durer les pinceaux en martre.", "Clean and preserve sable brushes."),
            Self::VarnishStudio => locale.tr("Mat, satiné ou brillant : quand et comment vernir.", "Matt, satin or gloss: when and how to varnish."),
            Self::BatchSquad => locale.tr("Suivre étape par étape une série de figurines.", "Track a batch of miniatures step by step."),
            Self::OslEngine => locale.tr("Couleurs d'une lueur projetée par une source de lumière.", "Colours of a glow cast by a light source."),
            Self::OilHarmonies => locale.tr("Complémentaires et analogues calculées sur votre palette.", "Complementary and analogous colours computed from your palette."),
            Self::OilConsistency => locale.tr("Gouttes de médium pour un glacis, une crème ou une pâte.", "Drops of medium for a glaze, a cream or a paste."),
            Self::GlazePreview => locale.tr("Ce que donne un glacis transparent sur une sous-couche zénithale.", "What a transparent glaze looks like over a zenithal undercoat."),
            Self::PaletteLayout => locale.tr("Où placer chaque tube sur la palette.", "Where to place each tube on the palette."),
            Self::VehicleWeathering => locale.tr("Filtres et jus pour blindés et véhicules.", "Filters and washes for armour and vehicles."),
            Self::WetOnWetBlending => locale.tr("Deux consistances fondent-elles bien l'une dans l'autre ?", "Do two consistencies blend well into each other?"),
            Self::BatchMotherMix => locale.tr("Quantité de mélange de base pour toute une escouade.", "How much base mix for a whole squad."),
            Self::PaletteFreshness => locale.tr("Combien de temps les mélanges restent travaillables sur la palette.", "How long mixes stay workable on the palette."),
        }
    }

    /// Theme of the tool in the side navigation.
    fn group_label(self, locale: Locale) -> &'static str {
        match self {
            Self::Converter
            | Self::OilHarmonies
            | Self::PaletteLayout
            | Self::OslEngine
            | Self::NmmStudio => locale.tr("Couleurs & mélanges", "Colours & mixing"),
            Self::PinWash
            | Self::GlazePreview
            | Self::OilConsistency
            | Self::WetOnWetBlending => locale.tr("Pose & techniques", "Application & techniques"),
            Self::Weathering
            | Self::VehicleWeathering
            | Self::Pigments
            | Self::SpecialEffects => locale.tr("Weathering & effets", "Weathering & effects"),
            Self::HeatBox | Self::PaletteFreshness => locale.tr("Séchage & palette", "Drying & palette"),
            Self::BatchSquad | Self::BatchMotherMix | Self::RecipeSheet => {
                locale.tr("Séries & recettes", "Batches & recipes")
            }
            Self::VarnishStudio | Self::SolventsAndStripping | Self::BrushCare => {
                locale.tr("Atelier & entretien", "Studio & care")
            }
        }
    }

    /// Tools grouped by theme, in display order, filtered by `query`. In acrylic, the
    /// oil-only tools are gathered in a last group instead of being scattered.
    pub fn grouped(system: PaintSystem, locale: Locale, query: &str) -> Vec<(&'static str, Vec<Self>)> {
        let query = query.trim().to_lowercase();
        let mut groups: Vec<(&'static str, Vec<Self>)> = Vec::new();
        for tab in Self::ALL {
            if !query.is_empty()
                && !tab.label(locale).to_lowercase().contains(&query)
                && !tab.blurb(locale).to_lowercase().contains(&query)
            {
                continue;
            }
            let label = if system == PaintSystem::Acrylic && tab.is_oil_only() {
                locale.tr("🛢 Finition à l'huile", "🛢 Oil finishing")
            } else {
                tab.group_label(locale)
            };
            match groups.iter_mut().find(|(l, _)| *l == label) {
                Some((_, tabs)) => tabs.push(tab),
                None => groups.push((label, vec![tab])),
            }
        }
        if let Some(pos) = groups.iter().position(|(l, _)| l.starts_with('🛢')) {
            let oil = groups.remove(pos);
            groups.push(oil);
        }
        groups
    }

    /// Tools that only make sense in oil: curing, open time, wet-on-wet, oil consistency,
    /// and the acrylic-to-oil converter (an acrylic base finished in oil).
    pub fn is_oil_only(self) -> bool {
        matches!(
            self,
            Self::Converter
                | Self::HeatBox
                | Self::OilHarmonies
                | Self::OilConsistency
                | Self::GlazePreview
                | Self::WetOnWetBlending
                | Self::PaletteFreshness
        )
    }

    pub fn from_idx(idx: usize) -> Self {
        match idx {
            1 => Self::NmmStudio,
            2 => Self::Weathering,
            3 => Self::PinWash,
            4 => Self::SolventsAndStripping,
            5 => Self::Pigments,
            6 => Self::RecipeSheet,
            7 => Self::HeatBox,
            8 => Self::SpecialEffects,
            9 => Self::BrushCare,
            10 => Self::VarnishStudio,
            11 => Self::BatchSquad,
            12 => Self::OslEngine,
            13 => Self::OilHarmonies,
            14 => Self::OilConsistency,
            15 => Self::GlazePreview,
            16 => Self::PaletteLayout,
            17 => Self::VehicleWeathering,
            18 => Self::WetOnWetBlending,
            19 => Self::BatchMotherMix,
            20 => Self::PaletteFreshness,
            _ => Self::Converter,
        }
    }

    pub fn to_idx(self) -> usize {
        match self {
            Self::Converter => 0,
            Self::NmmStudio => 1,
            Self::Weathering => 2,
            Self::PinWash => 3,
            Self::SolventsAndStripping => 4,
            Self::Pigments => 5,
            Self::RecipeSheet => 6,
            Self::HeatBox => 7,
            Self::SpecialEffects => 8,
            Self::BrushCare => 9,
            Self::VarnishStudio => 10,
            Self::BatchSquad => 11,
            Self::OslEngine => 12,
            Self::OilHarmonies => 13,
            Self::OilConsistency => 14,
            Self::GlazePreview => 15,
            Self::PaletteLayout => 16,
            Self::VehicleWeathering => 17,
            Self::WetOnWetBlending => 18,
            Self::BatchMotherMix => 19,
            Self::PaletteFreshness => 20,
        }
    }
}

/// Creates the state of every tool, once, for the whole studio.
fn provide_tool_states() {
    use_context_provider(converter::ConverterState::new);
    use_context_provider(nmm::NmmState::new);
    use_context_provider(pin_wash::PinWashState::new);
    use_context_provider(drying::DryingState::new);
    use_context_provider(weathering::WeatheringState::new);
    use_context_provider(care::CareState::new);
    use_context_provider(batch::BatchState::new);
    use_context_provider(osl::OslState::new);
    use_context_provider(solvents::SolventsState::new);
    use_context_provider(pigments::PigmentsState::new);
    use_context_provider(recipe_sheet::RecipeSheetState::new);
    use_context_provider(harmonies::HarmoniesState::new);
    use_context_provider(consistency::ConsistencyState::new);
    use_context_provider(glaze::GlazeState::new);
    use_context_provider(palette_layout::PaletteLayoutState::new);
    use_context_provider(vehicle::VehicleState::new);
}

#[component]
pub fn MiniatureView() -> Element {
    let mut state = use_context::<AppState>();
    let locale = *state.locale.read();
    let system = *state.paint_system.read();
    // Mixes are only computed from palettes of the working system.
    let palettes = working_palettes(&state.palettes.read(), system);

    let active_tab = MiniatureTab::from_idx(*state.miniature_tab.read());
    let mut tool_filter = use_signal(String::new);

    // Global Palette Selector for ALL Miniature Tools
    let mut selected_palette_idx = use_signal(|| 0usize);
    provide_tool_states();

    // Active candidate paints from selected palette (or full in-stock drawer)
    let palette_idx = *selected_palette_idx.read();
    let candidates = active_candidates(&palettes, palette_idx, &state.paints_of_system());
    let palette_name_display = match palettes.get(palette_idx) {
        Some(palette) => palette.name.clone(),
        None => locale
            .tr("Stock complet en tiroir", "Full in-stock drawer")
            .to_string(),
    };

    rsx! {
        div {
            SectionHeader {
                title: locale.tr("Atelier Figurine & Modélisme", "Miniature & Weathering Studio").to_string(),
                subtitle: locale.tr(
                    "Les outils du figuriniste, rangés par thème dans la colonne de gauche. Ils suivent le système choisi en haut ; en acrylique, les outils propres à l'huile sont réunis sous « Finition à l'huile ».",
                    "Miniature painter's tools, grouped by theme in the left column. They follow the system chosen at the top; in acrylic, oil-only tools are gathered under “Oil finishing”."
                ).to_string(),
                actions: None
            }

            // Global Palette Selector Banner for ALL Miniature Tools
            div { class: "card-subtle", style: "margin-bottom: 12px; padding: 10px 16px; border: 1px solid var(--accent-border);",
                div { class: "row-between", style: "align-items: center; flex-wrap: wrap; gap: 10px;",
                    div { class: "row", style: "gap: 10px; align-items: center;",
                        span { style: "font-weight: 700; font-size: 13px; display: flex; align-items: center; gap: 6px;",
                            "🎨 ",
                            {locale.tr("Palette active pour le calcul des mélanges :", "Active Palette for Mix Calculations:")}
                        }
                        select {
                            style: "background: var(--bg-input); color: var(--text-main); border: 1px solid var(--border-main); border-radius: 6px; padding: 5px 12px; font-size: 13px; font-weight: 650;",
                            value: "{selected_palette_idx}",
                            onchange: move |ev| {
                                if let Ok(idx) = ev.value().parse::<usize>() {
                                    selected_palette_idx.set(idx);
                                }
                            },
                            for (idx, pal) in palettes.iter().enumerate() {
                                option { value: "{idx}", "{pal.name} ({pal.paints.len()} tubes)" }
                            }
                            option { value: "{palettes.len()}", {locale.tr("Tout mon stock en tiroir", "All in-stock drawer tubes")} }
                        }
                    }
                    div { class: "row", style: "gap: 8px; align-items: center;",
                        span { class: "pill pill-ok", style: "font-weight: 700;", "{candidates.len()} ", {locale.tr("tubes disponibles", "tubes available")} }
                        span { class: "muted", style: "font-size: 11.5px;",
                            {locale.tr("Mélanges calculés en direct pour chaque outil", "Live Kubelka-Munk mixes for every tool")}
                        }
                    }
                }
            }

            // Studio navigation: tools grouped by theme in a side column, with search.
            div { class: "mini-layout",
                nav { class: "mini-nav",
                    input {
                        r#type: "search",
                        class: "mini-nav-search",
                        placeholder: locale.tr("Filtrer les outils…", "Filter tools…"),
                        value: "{tool_filter}",
                        oninput: move |e| tool_filter.set(e.value()),
                    }
                    for (group_label, tabs) in MiniatureTab::grouped(system, locale, &tool_filter.read()) {
                        div { class: "mini-nav-group",
                            div { class: "mini-nav-group-label", "{group_label}" }
                            for tab in tabs {
                                button {
                                    class: if active_tab == tab { "mini-nav-item active" } else { "mini-nav-item" },
                                    title: "{tab.blurb(locale)}",
                                    onclick: move |_| state.miniature_tab.set(tab.to_idx()),
                                    span { class: "mini-nav-icon", "{tab.icon()}" }
                                    span { "{tab.label(locale)}" }
                                }
                            }
                        }
                    }
                }
                div { class: "mini-content",
                    div { class: "mini-content-header",
                        h2 { "{active_tab.icon()} {active_tab.label(locale)}" }
                        if active_tab.is_oil_only() {
                            span { class: "pill pill-oil", {locale.tr("Huile", "Oil")} }
                        }
                        p { class: "muted", "{active_tab.blurb(locale)}" }
                    }
                    if system == PaintSystem::Acrylic && active_tab.is_oil_only() {
                        div { class: "card-subtle", style: "margin-bottom: 12px; border-color: var(--warning, #c98a2b);",
                            strong { {locale.tr("Outil propre à l'huile. ", "Oil-only tool. ")} }
                            {locale.tr(
                                "Vous travaillez en acrylique : cet outil ne s'applique que si vous terminez la pièce à l'huile sur une base acrylique sèche. Passez en Huile dans la barre du haut pour l'utiliser avec vos tubes.",
                                "You are working in acrylic: this tool only applies if you finish the piece in oil over a dry acrylic base. Switch to Oil in the top bar to use it with your tubes."
                            )}
                        }
                    }

                    {tool_view(state, active_tab, candidates, palette_name_display)}
                }
            }
        }
    }
}

/// The body of the active tool.
fn tool_view(state: AppState, tab: MiniatureTab, candidates: Vec<Paint>, palette_name: String) -> Element {
    // The converter and the recipe sheet always land in oil.
    let oil = |candidates: &[Paint]| {
        oil_candidates(*state.paint_system.read(), candidates, &state.paints.read())
    };
    match tab {
        MiniatureTab::Converter => rsx! {
            converter::Converter { oil_candidates: oil(&candidates), palette_name }
        },
        MiniatureTab::NmmStudio => rsx! { nmm::NmmStudio { candidates, palette_name } },
        MiniatureTab::Weathering => rsx! { weathering::Weathering { candidates, palette_name } },
        MiniatureTab::PinWash => rsx! { pin_wash::PinWash { candidates } },
        MiniatureTab::SolventsAndStripping => rsx! { solvents::SolventsAndStripping {} },
        MiniatureTab::Pigments => rsx! { pigments::Pigments { candidates } },
        MiniatureTab::RecipeSheet => rsx! {
            recipe_sheet::RecipeSheet { oil_candidates: oil(&candidates) }
        },
        MiniatureTab::HeatBox => rsx! { drying::HeatBox {} },
        MiniatureTab::SpecialEffects => rsx! {
            weathering::SpecialEffects { candidates, palette_name }
        },
        MiniatureTab::BrushCare => rsx! { care::BrushCare {} },
        MiniatureTab::VarnishStudio => rsx! { care::VarnishStudio {} },
        MiniatureTab::BatchSquad => rsx! { batch::BatchSquadTool {} },
        MiniatureTab::OslEngine => rsx! { osl::OslEngine { candidates } },
        MiniatureTab::OilHarmonies => rsx! { harmonies::OilHarmonies { candidates } },
        MiniatureTab::OilConsistency => rsx! { consistency::OilConsistency {} },
        MiniatureTab::GlazePreview => rsx! { glaze::GlazePreview { candidates } },
        MiniatureTab::PaletteLayout => rsx! { palette_layout::PaletteLayout {} },
        MiniatureTab::VehicleWeathering => rsx! { vehicle::VehicleWeathering { candidates } },
        MiniatureTab::WetOnWetBlending => rsx! { consistency::WetOnWetBlending {} },
        MiniatureTab::BatchMotherMix => rsx! { batch::BatchMotherMix {} },
        MiniatureTab::PaletteFreshness => rsx! { drying::PaletteFreshness {} },
    }
}

#[cfg(test)]
mod nav_tests {
    use super::*;

    fn flat(system: PaintSystem, query: &str) -> Vec<MiniatureTab> {
        MiniatureTab::grouped(system, Locale::Fr, query)
            .into_iter()
            .flat_map(|(_, tabs)| tabs)
            .collect()
    }

    #[test]
    fn every_tool_is_listed_exactly_once_and_keeps_its_index() {
        for system in PaintSystem::ALL {
            let tabs = flat(system, "");
            assert_eq!(21, tabs.len());
            for tab in MiniatureTab::ALL {
                assert_eq!(1, tabs.iter().filter(|t| **t == tab).count(), "{tab:?}");
                assert_eq!(tab, MiniatureTab::from_idx(tab.to_idx()));
            }
        }
    }

    #[test]
    fn in_acrylic_oil_tools_are_gathered_last() {
        let groups = MiniatureTab::grouped(PaintSystem::Acrylic, Locale::Fr, "");
        let (label, tabs) = groups.last().unwrap();
        assert!(label.contains("huile"));
        assert!(tabs.iter().all(|t| t.is_oil_only()));
        assert!(groups[..groups.len() - 1]
            .iter()
            .all(|(_, tabs)| tabs.iter().all(|t| !t.is_oil_only())));
        // In oil, no such group: tools stay in their theme.
        assert!(MiniatureTab::grouped(PaintSystem::Oil, Locale::Fr, "")
            .iter()
            .all(|(l, _)| !l.starts_with('🛢')));
    }

    #[test]
    fn filter_matches_label_or_description() {
        assert_eq!(vec![MiniatureTab::VarnishStudio], flat(PaintSystem::Oil, "vernir"));
        assert!(flat(PaintSystem::Oil, "rouille").contains(&MiniatureTab::Weathering));
    }
}
