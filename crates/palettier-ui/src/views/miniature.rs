#![forbid(unsafe_code)]

//! Dedicated Miniature & Weathering Studio view:
//! Acrylic-to-oil converter, 5-stage contrast ladders, NMM metal studio, grimdark weathering,
//! pin wash & reductive cleaning timer, heat box forced curing, special effects gore,
//! Kolinsky brush care, batch squad painting, and OSL light engines.
//! Every tool dynamically computes Kubelka-Munk subtractive oil mixes from the user's chosen palette!

use crate::state::{AppServices, AppState};
use crate::theme::{ColorSwatch, DeltaEBadge, SectionHeader};
use chrono::Utc;
use dioxus::prelude::*;
use palettier_domain::color::Rgb;
use palettier_domain::miniature::{
    ArmyRecipeSheet, BatchMotherMixCalculator, BatchSquad, BrushTracker, DryPigmentEffect,
    FreezerStorageSimulator, HeatBoxCalculator, LivePaletteFreshnessTracker,
    MiniatureLadder, MiniatureMaterial, MiniatureScale, NmmMetal,
    Consistency, OilGlazeZenithalSimulator, OilHarmoniesCalculator,
    OslPreset, PaletteLayoutType, PinWashConfig, SolventType, SpecialEffectGore, StrippingAgent,
    StrippingGuide, StrippingVerdictSeverity, VarnishFinish, VehicleEffectKind, WeatheringEffect,
    WetOnWetBlendingAdvisor, thinner_drops_per_part,
};
use palettier_domain::i18n::Locale;
use palettier_domain::paint::{Medium, Paint, PaintSystem, Technique};

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

#[component]
pub fn MiniatureView() -> Element {
    let mut state = use_context::<AppState>();
    let services = use_context::<AppServices>();
    let locale = *state.locale.read();
    let system = *state.paint_system.read();
    // Mixes are only computed from palettes of the working system.
    let palettes: Vec<palettier_domain::palette::Palette> = state
        .palettes
        .read()
        .iter()
        .filter(|p| p.paints.is_empty() || p.system() == system)
        .cloned()
        .collect();

    let active_tab = MiniatureTab::from_idx(*state.miniature_tab.read());
    let mut tool_filter = use_signal(String::new);

    // Converter: the acrylic pots of the catalog, filtered by brand and search.
    let mut brand_filter = use_signal(|| None::<String>);
    let mut acrylic_search = use_signal(String::new);
    let mut selected_acrylic = use_signal(|| None::<String>);

    // Global Palette Selector for ALL Miniature Tools
    let mut selected_palette_idx = use_signal(|| 0usize);

    // NMM state
    let mut selected_nmm = use_signal(|| NmmMetal::Gold);

    // Pin wash state
    let mut wash_solvent_ratio = use_signal(|| 6u32);
    let mut wash_tone_idx = use_signal(|| 0usize);
    let mut wash_timer_start = use_signal(|| None::<chrono::DateTime<Utc>>);

    // Heat box state
    let mut heat_temp = use_signal(|| 40.0f64);
    let heat_base_hours = use_signal(|| 24.0f64);

    // Special effects state
    let mut selected_gore = use_signal(|| SpecialEffectGore::FreshGore);

    // Brush care state
    let mut brushes = use_signal(BrushTracker::default_sable_kit);

    // Varnish state
    let mut selected_varnish = use_signal(|| VarnishFinish::UltraMatte);

    // Batch squad state
    let mut squad =
        use_signal(|| BatchSquad::standard_space_marine_squad("Escouade Intercessors", 5));
    let mut squad_size = use_signal(|| 5usize);
    let mut squad_name = use_signal(|| "Escouade Intercessors".to_string());

    // Weathering selected
    let mut selected_weathering = use_signal(|| WeatheringEffect::StreakingGrime);

    // OSL selected
    let mut selected_osl = use_signal(|| &OslPreset::ALL[0]);

    // Solvents, Stripping & Freezer state
    let mut selected_material = use_signal(|| MiniatureMaterial::PolystyrenePlastic);
    let mut selected_stripper = use_signal(|| StrippingAgent::Ipa99);
    let mut freezer_ambient_temp = use_signal(|| 20.0f64);
    let mut freezer_airtight = use_signal(|| true);
    let mut freezer_is_freezer = use_signal(|| true);

    // Pigments state
    let mut selected_pigment = use_signal(|| DryPigmentEffect::BurntUmberDust);

    // Army Recipe Sheet state
    let mut recipe_sheet = use_signal(ArmyRecipeSheet::default);

    // Oil Harmonies state
    let mut harmony_key = use_signal(|| None::<String>);

    // Oil Consistency & Drops state
    let mut consistency_technique = use_signal(|| Technique::Glaze);
    let mut consistency_parts = use_signal(|| 2u32);

    // Zenithal Glaze Preview state
    let mut glaze_key = use_signal(|| None::<String>);
    let mut glaze_strength = use_signal(|| 0.70f64);

    // 5 New Advanced Oil Miniature Features state
    let mut selected_palette_layout = use_signal(|| PaletteLayoutType::NormaBlueStudio);
    let mut selected_vehicle_fx = use_signal(|| VehicleEffectKind::StarshipFilth);
    let mut blending_base_tech = use_signal(|| Technique::OilRendering);
    let mut blending_accent_tech = use_signal(|| Technique::Highlight);
    let mut mother_mix_scale = use_signal(|| MiniatureScale::Scale32mm);
    let mut mother_mix_count = use_signal(|| 10u32);
    let mut mother_mix_full_coverage = use_signal(|| true);
    let mut freshness_minutes = use_signal(|| 90u32);
    let mut freshness_temp = use_signal(|| 21.0f64);

    let acrylic_paints: Vec<Paint> = {
        let mut all: Vec<Paint> = state
            .paints
            .read()
            .iter()
            .filter(|p| p.system == PaintSystem::Acrylic)
            .cloned()
            .collect();
        all.sort_by(|a, b| a.brand.cmp(&b.brand).then(a.name.cmp(&b.name)));
        all
    };
    let acrylic_brands: Vec<String> = acrylic_paints
        .iter()
        .map(|p| p.brand.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    let filtered_acrylics: Vec<Paint> = {
        let term = acrylic_search.read().trim().to_lowercase();
        let brand = brand_filter.read().clone();
        acrylic_paints
            .iter()
            .filter(|p| brand.as_ref().is_none_or(|b| &p.brand == b))
            .filter(|p| {
                term.is_empty()
                    || p.name.to_lowercase().contains(&term)
                    || p.brand.to_lowercase().contains(&term)
                    || p.code.to_lowercase().contains(&term)
            })
            .cloned()
            .collect()
    };
    let current_acrylic: Option<Paint> = pick(&acrylic_paints, selected_acrylic.read().as_deref());
    let current_ladder = current_acrylic
        .as_ref()
        .map(|p| MiniatureLadder::of_basecoat(p.color()));
    let wash_config = PinWashConfig {
        ratio_oil: 1,
        ratio_solvent: *wash_solvent_ratio.read(),
        min_cleaning_window_mins: 15,
        max_cleaning_window_mins: 90,
    };
    let heat_calc = HeatBoxCalculator::new(*heat_temp.read());

    // Active candidate paints from selected palette (or full in-stock drawer)
    let active_candidates: Vec<Paint> = if *selected_palette_idx.read() < palettes.len() {
        palettes[*selected_palette_idx.read()].paints.clone()
    } else {
        state
            .paints_of_system()
            .into_iter()
            .filter(|p| p.in_stock)
            .collect()
    };

    // The converter always lands in oil: in acrylic mode it searches the in-stock oil tubes.
    let oil_candidates: Vec<Paint> = if system == PaintSystem::Oil {
        active_candidates.clone()
    } else {
        state
            .paints
            .read()
            .iter()
            .filter(|p| p.system == PaintSystem::Oil && p.in_stock)
            .cloned()
            .collect()
    };
    let harmony_paint = pick(&active_candidates, harmony_key.read().as_deref());
    let glaze_paint = pick(&active_candidates, glaze_key.read().as_deref());

    let palette_name_display = if *selected_palette_idx.read() < palettes.len() {
        palettes[*selected_palette_idx.read()].name.clone()
    } else {
        locale
            .tr("Stock complet en tiroir", "Full in-stock drawer")
            .to_string()
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
                        span { class: "pill pill-ok", style: "font-weight: 700;", "{active_candidates.len()} ", {locale.tr("tubes disponibles", "tubes available")} }
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

                    match active_tab {
                        MiniatureTab::Converter => rsx! {
                            div { class: "grid-2", style: "grid-template-columns: 1fr 1.35fr;",
                                // Left: Acrylic Selector & Search
                                div { class: "card",
                                    div { class: "row-between", style: "margin-bottom: 10px;",
                                        h3 { class: "card-title", style: "margin: 0;",
                                            {locale.tr("Acryliques Figurines de Référence", "Miniature Acrylic Reference Paints")}
                                        }
                                        span { class: "pill", "{filtered_acrylics.len()} ", {locale.tr("teintes", "colours")} }
                                    }

                                    // Brand Filter chips
                                    div { class: "row", style: "gap: 6px; margin-bottom: 10px; flex-wrap: wrap;",
                                        button {
                                            class: if brand_filter.read().is_none() { "pill pill-accent" } else { "pill" },
                                            onclick: move |_| brand_filter.set(None),
                                            {locale.tr("Toutes marques", "All brands")}
                                        }
                                        for b in acrylic_brands.iter().cloned() {
                                            {
                                                let is_sel = brand_filter.read().as_deref() == Some(b.as_str());
                                                let label = b.clone();
                                                rsx! {
                                                    button {
                                                        class: if is_sel { "pill pill-accent" } else { "pill" },
                                                        onclick: move |_| brand_filter.set(Some(b.clone())),
                                                        "{label}"
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    // Search bar
                                    input {
                                        r#type: "text",
                                        style: "width: 100%; margin-bottom: 12px;",
                                        placeholder: locale.tr("Rechercher (ex: Mephiston, Blue, Nurgle, Flesh)…", "Search (e.g. Mephiston, Blue, Nurgle, Flesh)…"),
                                        value: "{acrylic_search}",
                                        oninput: move |ev| acrylic_search.set(ev.value()),
                                    }

                                    // Scrollable list
                                    div { style: "max-height: 480px; overflow-y: auto; display: flex; flex-direction: column; gap: 4px;",
                                        for acr in filtered_acrylics {
                                            {
                                                let k = paint_key(&acr);
                                                let is_active = current_acrylic.as_ref().is_some_and(|c| paint_key(c) == k);
                                                rsx! {
                                                    div {
                                                        class: if is_active { "card-subtle active" } else { "card-subtle" },
                                                        style: if is_active { "border-color: var(--accent); cursor: pointer; padding: 6px 10px;" } else { "cursor: pointer; padding: 6px 10px;" },
                                                        onclick: move |_| selected_acrylic.set(Some(k.clone())),
                                                        div { class: "row-between", style: "align-items: center;",
                                                            div { class: "row", style: "gap: 10px; align-items: center;",
                                                                ColorSwatch { hex: acr.hex_color.clone(), size: 24 }
                                                                div {
                                                                    div { style: "font-weight: 650; font-size: 13px;", "{acr.name}" }
                                                                    div { class: "muted", style: "font-size: 11px;", "{acr.brand}" }
                                                                }
                                                            }
                                                            if !acr.code.is_empty() {
                                                                span { class: "pill mono", style: "font-size: 10.5px;", "{acr.code}" }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                // Right: 5-Stage Ladder with Calculated Mixes from chosen palette
                                div { class: "card",
                                    div { class: "row-between", style: "margin-bottom: 12px;",
                                        div {
                                            div { class: "card-title", style: "margin: 0;",
                                                {locale.tr("Échelle 5 Étapes & Recettes Calculées", "5-Stage Miniature Ladder & Calculated Mixes")}
                                            }
                                            div { class: "muted", style: "font-size: 12px; margin-top: 2px;",
                                                {locale.tr("Calculé pour ", "Calculated for ")}
                                                strong { "{current_acrylic.as_ref().map(|p| p.name.clone()).unwrap_or_default()}" }
                                                " ({current_acrylic.as_ref().map(|p| p.hex_color.clone()).unwrap_or_default()}) "
                                                {locale.tr("avec ", "with ")}
                                                span { class: "pill pill-accent", style: "padding: 1px 6px; font-size: 11px;", "{palette_name_display}" }
                                            }
                                        }

                                        button {
                                            class: "btn",
                                            onclick: {
                                                let h = current_acrylic.as_ref().map(|p| p.hex_color.clone()).unwrap_or_default();
                                                move |_| {
                                                    state.mixer_target_hex.set(h.clone());
                                                    state.section.set(crate::state::Section::Mixer);
                                                }
                                            },
                                            "🧪 ",
                                            {locale.tr("Tester dans le Mélangeur", "Send to Mixer")}
                                        }
                                    }

                                    // 5-Stage Computed Rows
                                    div { style: "display: flex; flex-direction: column; gap: 8px;",
                                        for (idx, step_hex) in current_ladder.as_ref().map(|l| l.steps_hex().to_vec()).unwrap_or_default().iter().enumerate() {
                                            {
                                                let short_label = match idx {
                                                    0 => locale.tr("1. Ombre profonde (Occlusion / Pin wash)", "1. Deep Occlusion (Pin wash)"),
                                                    1 => locale.tr("2. Ombre de forme (Wet blending)", "2. Form Shadow (Wet blending)"),
                                                    2 => locale.tr("3. Couleur de base (Couverture)", "3. Basecoat (Midtone)"),
                                                    3 => locale.tr("4. Éclaircissement volumétrique (Zénithal)", "4. Volumetric Highlight (Zenithal)"),
                                                    4 => locale.tr("5. Rehaut d'arête / Spéculaire", "5. Edge Highlight / Specular"),
                                                    _ => "",
                                                };
                                                let tgt_rgb = Rgb::of_hex(step_hex).unwrap_or_else(|_| Rgb::new(0.5, 0.5, 0.5));
                                                let sugg = services.mix_service.suggest_mixes(tgt_rgb, &oil_candidates, 1, Some(3)).into_iter().next();

                                                rsx! {
                                                    div { class: "card-subtle", style: "padding: 10px 12px;",
                                                        div { class: "row-between", style: "align-items: center;",
                                                            div { class: "row", style: "gap: 10px; align-items: center;",
                                                                ColorSwatch { hex: step_hex.clone(), size: 30 }
                                                                div {
                                                                    div { style: "font-weight: 700; font-size: 12.5px;", "{short_label}" }
                                                                    div { class: "mono", style: "font-size: 10.5px; color: var(--text-dim);", "{step_hex}" }
                                                                }
                                                            }

                                                            if let Some(ref s) = sugg {
                                                                div { class: "row", style: "gap: 10px; align-items: center;",
                                                                    ColorSwatch { hex: s.color.to_hex(), size: 30 }
                                                                    div { style: "text-align: right; line-height: 1.3;",
                                                                        div { style: "font-weight: 650; font-size: 12px; color: var(--accent);", "{s.describe_i18n(locale)}" }
                                                                        div { style: "display: flex; gap: 4px; justify-content: flex-end; align-items: center; margin-top: 2px;",
                                                                            DeltaEBadge { delta_e: s.delta_e }
                                                                        }
                                                                    }
                                                                    button {
                                                                        class: "btn",
                                                                        style: "padding: 4px 8px; font-size: 11px;",
                                                                        title: locale.tr("Ouvrir cette teinte dans le Mélangeur", "Open target in Mixer"),
                                                                        onclick: {
                                                                            let h = step_hex.clone();
                                                                            move |_| {
                                                                                state.mixer_target_hex.set(h.clone());
                                                                                state.section.set(crate::state::Section::Mixer);
                                                                            }
                                                                        },
                                                                        "🧪"
                                                                    }
                                                                }
                                                            } else {
                                                                span { class: "muted", style: "font-size: 11.5px;",
                                                                    {locale.tr("Aucun tube disponible dans la palette", "No tubes available in palette")}
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        },

                        MiniatureTab::NmmStudio => rsx! {
                            div { class: "grid-2", style: "grid-template-columns: 1fr 1.35fr;",
                                // Left: NMM Family selector
                                div { class: "card",
                                    h3 { class: "card-title", {locale.tr("Familles de Métaux Non-Métalliques (NMM)", "Non-Metallic Metal (NMM) Families")} }
                                    p { class: "muted", style: "font-size: 12px; margin-bottom: 14px;",
                                        {locale.tr("Le NMM recrée l'illusion du métal poli sans particules scintillantes, par le contraste extrême d'ombres sourdes et de blancs spéculaires.", "NMM recreates the illusion of polished metal using extreme value contrast between deep shadows and pure white catchlights.")}
                                    }

                                    div { style: "display: flex; flex-direction: column; gap: 8px;",
                                        for nmm in NmmMetal::ALL {
                                            {
                                                let is_sel = *selected_nmm.read() == nmm;
                                                let sample_hex = nmm.steps_hex()[2];
                                                rsx! {
                                                    div {
                                                        class: if is_sel { "card-subtle active" } else { "card-subtle" },
                                                        style: if is_sel { "border-color: var(--accent); cursor: pointer; padding: 10px;" } else { "cursor: pointer; padding: 10px;" },
                                                        onclick: move |_| selected_nmm.set(nmm),
                                                        div { class: "row", style: "gap: 10px; align-items: center;",
                                                            ColorSwatch { hex: sample_hex.to_string(), size: 28 }
                                                            div {
                                                                div { style: "font-weight: 700; font-size: 13px;", "{nmm.label_i18n(locale)}" }
                                                                div { class: "mono", style: "font-size: 11px; opacity: 0.7;", "{sample_hex}" }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                // Right: NMM 5-Step Scale & Palette Mix Calculator
                                div { class: "card",
                                    {
                                        let nmm = *selected_nmm.read();
                                        let steps = nmm.steps_hex();
                                        rsx! {
                                            div { class: "row-between", style: "margin-bottom: 14px; flex-wrap: wrap; gap: 8px;",
                                                div { class: "row", style: "gap: 12px; align-items: center;",
                                                    ColorSwatch { hex: steps[2].to_string(), size: 40 }
                                                    div {
                                                        h3 { class: "card-title", style: "margin: 0;", "{nmm.label_i18n(locale)}" }
                                                        span { class: "pill pill-accent", "High-Contrast NMM Specular" }
                                                    }
                                                }

                                                span { class: "pill", style: "font-size: 11.5px;",
                                                    {locale.tr("Calculé avec : ", "Calculated with: ")},
                                                    strong { "{palette_name_display}" }
                                                }
                                            }

                                            // Computed mix rows for the 5 NMM steps
                                            div { style: "display: flex; flex-direction: column; gap: 8px; margin-bottom: 16px;",
                                                for (idx, step_hex_str) in steps.iter().enumerate() {
                                                    {
                                                        let tag = match idx {
                                                            0 => locale.tr("1. Ombre d'occlusion (Max)", "1. Max Occlusion Shadow"),
                                                            1 => locale.tr("2. Ombre de forme", "2. Form Shadow"),
                                                            2 => locale.tr("3. Base métal", "3. Metal Midtone Base"),
                                                            3 => locale.tr("4. Pleine lumière", "4. Key Highlight"),
                                                            4 => locale.tr("5. Rehaut spéculaire", "5. Specular Catchlight"),
                                                            _ => "",
                                                        };
                                                        let tgt_rgb = Rgb::of_hex(step_hex_str).unwrap_or_else(|_| Rgb::new(0.5, 0.5, 0.5));
                                                        let sugg = services.mix_service.suggest_mixes(tgt_rgb, &active_candidates, 1, Some(3)).into_iter().next();

                                                        rsx! {
                                                            div { class: "card-subtle", style: "padding: 8px 12px;",
                                                                div { class: "row-between", style: "align-items: center;",
                                                                    div { class: "row", style: "gap: 10px; align-items: center;",
                                                                        ColorSwatch { hex: step_hex_str.to_string(), size: 28 }
                                                                        div {
                                                                            div { style: "font-weight: 700; font-size: 12px;", "{tag}" }
                                                                            div { class: "mono", style: "font-size: 10.5px; color: var(--text-dim);", "{step_hex_str}" }
                                                                        }
                                                                    }

                                                                    if let Some(ref s) = sugg {
                                                                        div { class: "row", style: "gap: 8px; align-items: center;",
                                                                            ColorSwatch { hex: s.color.to_hex(), size: 28 }
                                                                            div { style: "text-align: right; line-height: 1.3;",
                                                                                div { style: "font-weight: 650; font-size: 12px; color: var(--accent);", "{s.describe_i18n(locale)}" }
                                                                                div { style: "display: flex; gap: 4px; justify-content: flex-end; align-items: center; margin-top: 2px;",
                                                                                    DeltaEBadge { delta_e: s.delta_e }
                                                                                }
                                                                            }
                                                                            button {
                                                                                class: "btn",
                                                                                style: "padding: 3px 8px; font-size: 11px;",
                                                                                title: locale.tr("Ouvrir cette teinte dans le Mélangeur", "Open target in Mixer"),
                                                                                onclick: {
                                                                                    let h = step_hex_str.to_string();
                                                                                    move |_| {
                                                                                        state.mixer_target_hex.set(h.clone());
                                                                                        state.section.set(crate::state::Section::Mixer);
                                                                                    }
                                                                                },
                                                                                "🧪"
                                                                            }
                                                                        }
                                                                    } else {
                                                                        span { class: "muted", style: "font-size: 11.5px;",
                                                                            {locale.tr("Aucun tube disponible dans la palette", "No tubes available in palette")}
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }

                                            div { class: "card-subtle",
                                                div { style: "font-weight: 700; font-size: 12px; margin-bottom: 4px; color: var(--accent);",
                                                    {locale.tr("🌍 Règle du reflet de sol (Ground Bounce) :", "🌍 Ground Bounce Reflection Physics:")}
                                                }
                                                div { style: "font-size: 12.5px; line-height: 1.45;",
                                                    "{nmm.ground_bounce_advice_i18n(locale)}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        },

                        MiniatureTab::Weathering => rsx! {
                            div { class: "grid-2", style: "grid-template-columns: 1fr 1.35fr;",
                                div { class: "card",
                                    h3 { class: "card-title", {locale.tr("Effets d'Intempéries & Grimdark aux Huiles", "Oil Weathering & Grimdark Effects")} }
                                    p { class: "muted", style: "font-size: 12px; margin-bottom: 14px;",
                                        {locale.tr("Les huiles sont reines pour la patine réaliste : temps de travail illimité, estompage sans traces et capillarité naturelle.", "Oils excel at realistic weathering: unlimited open-time, seamless feathering, and natural capillary action.")}
                                    }

                                    div { style: "display: flex; flex-direction: column; gap: 8px;",
                                        for eff in WeatheringEffect::ALL {
                                            {
                                                let is_sel = *selected_weathering.read() == eff;
                                                rsx! {
                                                    div {
                                                        class: if is_sel { "card-subtle active" } else { "card-subtle" },
                                                        style: if is_sel { "border-color: var(--accent); cursor: pointer; padding: 10px;" } else { "cursor: pointer; padding: 10px;" },
                                                        onclick: move |_| selected_weathering.set(eff),
                                                        div { class: "row", style: "gap: 10px; align-items: center;",
                                                            ColorSwatch { hex: eff.hex_color().to_string(), size: 28 }
                                                            div {
                                                                div { style: "font-weight: 700; font-size: 13px;", "{eff.label_i18n(locale)}" }
                                                                div { class: "mono", style: "font-size: 11px; opacity: 0.7;", "{eff.hex_color()}" }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                div { class: "card",
                                    {
                                        let eff = *selected_weathering.read();
                                        let tgt_rgb = Rgb::of_hex(eff.hex_color()).unwrap_or_else(|_| Rgb::new(0.5, 0.5, 0.5));
                                        let sugg = services.mix_service.suggest_mixes(tgt_rgb, &active_candidates, 1, Some(3)).into_iter().next();

                                        rsx! {
                                            div { class: "row-between", style: "margin-bottom: 14px;",
                                                div { class: "row", style: "gap: 12px; align-items: center;",
                                                    ColorSwatch { hex: eff.hex_color().to_string(), size: 40 }
                                                    div {
                                                        h3 { class: "card-title", style: "margin: 0;", "{eff.label_i18n(locale)}" }
                                                        span { class: "pill pill-accent", style: "margin-top: 2px;", "Grimdark Oil Weathering" }
                                                    }
                                                }

                                                button {
                                                    class: "btn",
                                                    onclick: {
                                                        let h = eff.hex_color().to_string();
                                                        move |_| {
                                                            state.mixer_target_hex.set(h.clone());
                                                            state.section.set(crate::state::Section::Mixer);
                                                        }
                                                    },
                                                    "🧪 ",
                                                    {locale.tr("Ouvrir dans le Mélangeur", "Open in Mixer")}
                                                }
                                            }

                                            // Dynamic calculated mix from chosen palette!
                                            div { class: "card-subtle", style: "margin-bottom: 14px; border: 1px solid var(--accent-border);",
                                                div { style: "font-weight: 700; font-size: 12px; margin-bottom: 6px; color: var(--accent);",
                                                    {locale.tr("🧪 Recette calculée avec votre palette : ", "🧪 Mix calculated with your palette: ")},
                                                    strong { "{palette_name_display}" }
                                                }
                                                if let Some(ref s) = sugg {
                                                    div { class: "row-between", style: "align-items: center;",
                                                        div { class: "row", style: "gap: 10px; align-items: center;",
                                                            ColorSwatch { hex: s.color.to_hex(), size: 32 }
                                                            div {
                                                                div { style: "font-weight: 750; font-size: 13.5px;", "{s.describe_i18n(locale)}" }
                                                                div { class: "muted", style: "font-size: 11px;", "{s.verdict_i18n(locale)}" }
                                                            }
                                                        }
                                                        DeltaEBadge { delta_e: s.delta_e }
                                                    }
                                                } else {
                                                    div { class: "muted", style: "font-size: 12px;",
                                                        {locale.tr("Aucun tube disponible dans la palette sélectionnée pour reproduire cet effet.", "No matching tubes available in selected palette for this effect.")}
                                                    }
                                                }
                                            }

                                            div { class: "card-subtle",
                                                div { style: "font-weight: 700; font-size: 12px; margin-bottom: 4px; color: var(--accent);",
                                                    {locale.tr("🖌️ Technique d'application sur figurine :", "🖌️ Miniature Application Technique:")}
                                                }
                                                div { style: "font-size: 12.5px; line-height: 1.45;",
                                                    "{eff.application_technique_i18n(locale)}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        },

                        MiniatureTab::PinWash => rsx! {
                            div { class: "grid-2", style: "grid-template-columns: 1fr 1fr;",
                                div { class: "card",
                                    h3 { class: "card-title", {locale.tr("Dosage de Dilution & Teinte de Lavis", "Dilution Ratio & Wash Tone")} }
                                    p { class: "muted", style: "font-size: 12px; margin-bottom: 14px;",
                                        {locale.tr("Le secret d'un pin wash réussi réside dans le bon ratio d'essence minérale sans odeur pour que le pigment glisse sans bavure.", "The key to a clean pin wash is the exact ratio of odorless mineral spirits so pigment flows instantly.")}
                                    }

                                    // Tone selector
                                    div { class: "row", style: "gap: 6px; margin-bottom: 14px;",
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
                                    {
                                        let wash_hex = match *wash_tone_idx.read() {
                                            0 => "#111111",
                                            1 => "#3B2215",
                                            _ => "#121D28",
                                        };
                                        let tgt_rgb = Rgb::of_hex(wash_hex).unwrap_or_else(|_| Rgb::new(0.1, 0.1, 0.1));
                                        let sugg = services.mix_service.suggest_mixes(tgt_rgb, &active_candidates, 1, Some(3)).into_iter().next();

                                        rsx! {
                                            div { class: "card-subtle", style: "margin-bottom: 14px; border: 1px solid var(--accent-border);",
                                                div { style: "font-weight: 700; font-size: 12px; margin-bottom: 6px; color: var(--accent);",
                                                    {locale.tr("🧪 Recette de pin wash calculée avec votre palette :", "🧪 Calculated Pin Wash from your palette:")}
                                                }
                                                if let Some(ref s) = sugg {
                                                    div { class: "row-between", style: "align-items: center;",
                                                        div { class: "row", style: "gap: 10px; align-items: center;",
                                                            ColorSwatch { hex: s.color.to_hex(), size: 28 }
                                                            div {
                                                                div { style: "font-weight: 700; font-size: 12.5px;", "{s.describe_i18n(locale)}" }
                                                                div { class: "muted", style: "font-size: 11px;",
                                                                    {locale.tr("Diluer dans ", "Dilute with ")},
                                                                    strong { "{wash_solvent_ratio} parts " },
                                                                    {locale.tr("d'essence sans odeur", "of odorless spirits")}
                                                                }
                                                            }
                                                        }
                                                        DeltaEBadge { delta_e: s.delta_e }
                                                    }
                                                } else {
                                                    span { class: "muted", style: "font-size: 11px;", {locale.tr("Aucun tube disponible dans la palette", "No tubes available in palette")} }
                                                }
                                            }
                                        }
                                    }

                                    div { class: "card-subtle", style: "margin-bottom: 14px;",
                                        div { class: "row-between", style: "margin-bottom: 8px;",
                                            strong { {locale.tr("Ratio de Solvant :", "Solvent Ratio:")} }
                                            span { class: "pill pill-accent", "1 part Huile : {wash_solvent_ratio} parts Essence" }
                                        }
                                        input {
                                            r#type: "range",
                                            style: "width: 100%;",
                                            min: "1",
                                            max: "15",
                                            value: "{wash_solvent_ratio}",
                                            oninput: move |ev| {
                                                if let Ok(v) = ev.value().parse::<u32>() {
                                                    wash_solvent_ratio.set(v);
                                                }
                                            }
                                        }
                                        div { style: "font-weight: 600; font-size: 12.5px; margin-top: 8px;",
                                            "{wash_config.capillary_flow_desc(locale)}"
                                        }
                                    }

                                    div { class: "row", style: "gap: 8px; flex-wrap: wrap;",
                                        button { class: "btn", onclick: move |_| wash_solvent_ratio.set(3), {locale.tr("Creux profonds (1:3)", "Deep recesses (1:3)")} }
                                        button { class: "btn-primary", onclick: move |_| wash_solvent_ratio.set(6), {locale.tr("Pin wash standard (1:6)", "Standard Armor (1:6)")} }
                                        button { class: "btn", onclick: move |_| wash_solvent_ratio.set(12), {locale.tr("Filtre d'ambiance (1:12)", "Glaze Filter (1:12)")} }
                                    }
                                }

                                div { class: "card",
                                    h3 { class: "card-title", {locale.tr("Fenêtre de Nettoyage Réductif", "Reductive Cleaning Window")} }
                                    div { class: "card-subtle", style: "margin-bottom: 16px; text-align: center; padding: 20px;",
                                        if let Some(start) = *wash_timer_start.read() {
                                            {
                                                let elapsed_mins = (Utc::now() - start).num_minutes();
                                                let (status_class, status_text) = if elapsed_mins < 15 {
                                                    ("pill-warn", locale.tr("Phase 1 : Évaporation du solvant (Ne pas toucher)", "Stage 1: Solvent evaporating (Do not touch)"))
                                                } else if elapsed_mins <= 90 {
                                                    ("pill-ok", locale.tr("Phase 2 : FENÊTRE D'OR DE NETTOYAGE RÉDUCTIF !", "Stage 2: GOLDEN REDUCTIVE CLEANING WINDOW!"))
                                                } else {
                                                    ("pill-danger", locale.tr("Phase 3 : Polymérisation amorcée (Fixation)", "Stage 3: Curing started (Permanent set)"))
                                                };

                                                rsx! {
                                                    div { class: "mono", style: "font-size: 28px; font-weight: 800; margin-bottom: 8px;", "{elapsed_mins} min" }
                                                    div { class: "pill {status_class}", style: "padding: 4px 12px; font-size: 12px; font-weight: 700;", "{status_text}" }
                                                    div { style: "margin-top: 14px;",
                                                        button { class: "btn", onclick: move |_| wash_timer_start.set(None), {locale.tr("Réinitialiser le chrono", "Reset Timer")} }
                                                    }
                                                }
                                            }
                                        } else {
                                            div { style: "font-size: 14px; font-weight: 600; margin-bottom: 10px;", {locale.tr("Chronomètre de séance inactif", "Session timer stopped")} }
                                            button { class: "btn-primary", onclick: move |_| wash_timer_start.set(Some(Utc::now())), "⏱️ ", {locale.tr("Démarrer la pose du Pin Wash", "Start Pin Wash Application")} }
                                        }
                                    }
                                }
                            }
                        },

                        MiniatureTab::SolventsAndStripping => rsx! {
                            div { style: "display: flex; flex-direction: column; gap: 16px;",
                                // Matrice de Décapage Sécurisé
                                div { class: "card",
                                    h3 { class: "card-title", {locale.tr("🧼 Matrice de Décapage Sécurisé par Matériau", "🧼 Miniature Paint Stripping Safety Matrix")} }
                                    p { class: "muted", style: "font-size: 12px; margin-bottom: 14px;",
                                        {locale.tr("Calculateur de sauvetage : vérifiez la compatibilité chimique pour décaper vos figurines sans risquer la destruction irréversible des détails ou la fonte du plastique.", "Rescue calculator: verify chemical safety before stripping miniatures to avoid melting fine plastics or warping resins.")}
                                    }

                                    div { class: "grid-2", style: "gap: 16px; margin-bottom: 16px;",
                                        div {
                                            label { style: "font-size: 12px; font-weight: 600; display: block; margin-bottom: 6px;",
                                                {locale.tr("Matériau de la figurine :", "Miniature Material:")}
                                            }
                                            select {
                                                class: "input",
                                                value: match *selected_material.read() {
                                                    MiniatureMaterial::PolystyrenePlastic => "plastic",
                                                    MiniatureMaterial::ResinSla3d => "resin",
                                                    MiniatureMaterial::WhiteMetal => "metal",
                                                },
                                                onchange: move |ev| {
                                                    let mat = match ev.value().as_str() {
                                                        "resin" => MiniatureMaterial::ResinSla3d,
                                                        "metal" => MiniatureMaterial::WhiteMetal,
                                                        _ => MiniatureMaterial::PolystyrenePlastic,
                                                    };
                                                    selected_material.set(mat);
                                                },
                                                option { value: "plastic", "{MiniatureMaterial::PolystyrenePlastic.label_i18n(locale)}" }
                                                option { value: "resin", "{MiniatureMaterial::ResinSla3d.label_i18n(locale)}" }
                                                option { value: "metal", "{MiniatureMaterial::WhiteMetal.label_i18n(locale)}" }
                                            }
                                        }

                                        div {
                                            label { style: "font-size: 12px; font-weight: 600; display: block; margin-bottom: 6px;",
                                                {locale.tr("Agent / Produit décapant :", "Chemical Stripping Agent:")}
                                            }
                                            select {
                                                class: "input",
                                                value: match *selected_stripper.read() {
                                                    StrippingAgent::Ipa99 => "ipa",
                                                    StrippingAgent::Biostrip20 => "biostrip",
                                                    StrippingAgent::Glanzer => "glanzer",
                                                    StrippingAgent::PureAcetone => "acetone",
                                                },
                                                onchange: move |ev| {
                                                    let ag = match ev.value().as_str() {
                                                        "biostrip" => StrippingAgent::Biostrip20,
                                                        "glanzer" => StrippingAgent::Glanzer,
                                                        "acetone" => StrippingAgent::PureAcetone,
                                                        _ => StrippingAgent::Ipa99,
                                                    };
                                                    selected_stripper.set(ag);
                                                },
                                                option { value: "ipa", "{StrippingAgent::Ipa99.label_i18n(locale)}" }
                                                option { value: "biostrip", "{StrippingAgent::Biostrip20.label_i18n(locale)}" }
                                                option { value: "glanzer", "{StrippingAgent::Glanzer.label_i18n(locale)}" }
                                                option { value: "acetone", "{StrippingAgent::PureAcetone.label_i18n(locale)}" }
                                            }
                                        }
                                    }

                                    // Stripping Verdict Card
                                    {
                                        let verdict = StrippingGuide::evaluate(*selected_material.read(), *selected_stripper.read());
                                        let (border_color, bg_style, badge_cls) = match verdict.severity {
                                            StrippingVerdictSeverity::Safe => ("var(--accent-ok)", "background: rgba(34, 197, 94, 0.08);", "pill pill-ok"),
                                            StrippingVerdictSeverity::Caution => ("#F59E0B", "background: rgba(245, 158, 11, 0.08);", "pill pill-warn"),
                                            StrippingVerdictSeverity::Forbidden => ("#EF4444", "background: rgba(239, 68, 68, 0.12);", "pill pill-danger"),
                                        };
                                        let advice = if locale.is_fr() { verdict.advice_fr } else { verdict.advice_en };

                                        rsx! {
                                            div {
                                                style: "border: 1.5px solid {border_color}; border-radius: 8px; padding: 14px; {bg_style}",
                                                div { class: "row-between", style: "align-items: center; margin-bottom: 8px;",
                                                    strong { style: "font-size: 14px; letter-spacing: 0.5px;", "{verdict.title}" }
                                                    span { class: "{badge_cls}", "Durée conseillée : {verdict.recommended_duration}" }
                                                }
                                                p { style: "margin: 0; font-size: 13px; line-height: 1.45;", "{advice}" }
                                            }
                                        }
                                    }
                                }

                                // Guide Solvants & Médiums
                                div { class: "grid-2", style: "gap: 16px;",
                                    // Solvants
                                    div { class: "card",
                                        h3 { class: "card-title", {locale.tr("🧪 Guide des Solvants pour Lavis d'Huile", "🧪 Oil Solvents & Wash Dilution Guide")} }
                                        div { style: "display: flex; flex-direction: column; gap: 10px;",
                                            for sol in SolventType::ALL {
                                                {
                                                    let (safety_text, badge_type) = sol.acrylic_safety_level_i18n(locale);
                                                    let badge_cls = match badge_type {
                                                        "ok" => "pill pill-ok",
                                                        "warn" => "pill pill-warn",
                                                        _ => "pill pill-danger",
                                                    };
                                                    rsx! {
                                                        div { class: "card-subtle", style: "padding: 10px;",
                                                            div { class: "row-between", style: "margin-bottom: 4px;",
                                                                strong { style: "font-size: 13px;", "{sol.name()}" }
                                                                span { class: "{badge_cls}", "{safety_text}" }
                                                            }
                                                            div { class: "muted", style: "font-size: 11.5px; margin-bottom: 4px;",
                                                                "⏱️ {sol.evaporation_speed_i18n(locale)}"
                                                            }
                                                            div { style: "font-size: 11px; opacity: 0.85;",
                                                                "🫁 {sol.odor_and_toxicity_i18n(locale)}"
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    // Médiums
                                    div { class: "card",
                                        h3 { class: "card-title", {locale.tr("🍯 Médiums Spécialisés Figurines", "🍯 Specialized Miniature Oil Mediums")} }
                                        div { style: "display: flex; flex-direction: column; gap: 10px;",
                                            // The same mediums, factors and limits as the drying model.
                                            for med in Medium::for_system(PaintSystem::Oil).into_iter().filter(|m| *m != Medium::None) {
                                                {
                                                    let max = med.max_useful_ratio();
                                                    let factor = med.drying_factor(max);
                                                    rsx! {
                                                        div { class: "card-subtle", style: "padding: 10px;",
                                                            div { class: "row-between", style: "margin-bottom: 4px;",
                                                                strong { style: "font-size: 13px;", "{med.label_i18n(locale)}" }
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
                                                            div { class: "muted", style: "font-size: 11.5px; margin-bottom: 4px;",
                                                                "{med.advice_i18n(locale)}"
                                                            }
                                                            div { style: "font-size: 11px; color: var(--accent); font-weight: 600;",
                                                                "📏 ", {locale.tr("Au plus ", "At most ")}, "{(max * 100.0).round()} %"
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                // Simulateur Conservation au Froid (Congélateur)
                                div { class: "card",
                                    h3 { class: "card-title", {locale.tr("❄️ Conservation de Palette d'Huile au Froid / Congélateur", "❄️ Oil Palette Preservation & Freezer Storage")} }
                                    p { class: "muted", style: "font-size: 12px; margin-bottom: 14px;",
                                        {locale.tr("Contrairement aux acryliques, l'huile ne sèche pas par évaporation mais par oxydo-polymérisation. Abaisser la température ralentit exponentiellement la réaction chimique (loi d'Arrhenius).", "Unlike acrylics, oils cure via oxidation rather than evaporation. Freezing drastically slows the cross-linking reaction rate.")}
                                    }

                                    div { class: "grid-3", style: "gap: 14px; margin-bottom: 14px;",
                                        div { class: "card-subtle",
                                            label { class: "row", style: "gap: 8px; align-items: center; cursor: pointer; font-size: 13px; font-weight: 600;",
                                                input {
                                                    r#type: "checkbox",
                                                    checked: *freezer_airtight.read(),
                                                    onchange: move |ev| freezer_airtight.set(ev.value().parse::<bool>().unwrap_or(true)),
                                                }
                                                span { {locale.tr("Boîte hermétique (Palette Seal)", "Airtight Palette Seal Box")} }
                                            }
                                            p { class: "muted", style: "font-size: 11px; margin-top: 6px; margin-bottom: 0;",
                                                {locale.tr("Empêche le renouvellement d'oxygène frais.", "Prevents fresh atmospheric oxygen exposure.")}
                                            }
                                        }

                                        div { class: "card-subtle",
                                            label { class: "row", style: "gap: 8px; align-items: center; cursor: pointer; font-size: 13px; font-weight: 600;",
                                                input {
                                                    r#type: "checkbox",
                                                    checked: *freezer_is_freezer.read(),
                                                    onchange: move |ev| freezer_is_freezer.set(ev.value().parse::<bool>().unwrap_or(true)),
                                                }
                                                span { {locale.tr("Placer au congélateur (-18°C)", "Store in Freezer (-18°C)")} }
                                            }
                                            p { class: "muted", style: "font-size: 11px; margin-top: 6px; margin-bottom: 0;",
                                                {locale.tr("L'huile d'artiste pure ne gèle pas en bloc à -18°C.", "Pure oil paint does not freeze solid at -18°C.")}
                                            }
                                        }

                                        div { class: "card-subtle",
                                            div { style: "font-weight: 600; font-size: 12px; margin-bottom: 4px;",
                                                {locale.tr("Température ambiante de l'atelier :", "Workshop Ambient Temperature:")}
                                            }
                                            div { class: "row-between",
                                                span { class: "mono", style: "font-size: 13px;", "{freezer_ambient_temp} °C" }
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
                                                style: "width: 100%; margin-top: 6px;",
                                            }
                                        }
                                    }

                                    {
                                        let usable_days = FreezerStorageSimulator::estimated_usable_days(
                                            *freezer_ambient_temp.read(),
                                            *freezer_airtight.read(),
                                            *freezer_is_freezer.read(),
                                        );
                                        rsx! {
                                            div { class: "card-subtle", style: "border: 1px solid var(--accent-border);",
                                                div { class: "row-between", style: "align-items: center;",
                                                    div {
                                                        strong { style: "color: var(--accent); font-size: 14px;",
                                                            {locale.tr("Temps de réutilisation garanti de vos mélanges :", "Estimated Safe Working Palette Lifetime:")}
                                                        }
                                                        div { class: "muted", style: "font-size: 12px; margin-top: 2px;",
                                                            {
                                                                if *freezer_is_freezer.read() {
                                                                    locale.tr("Sortez la boîte du congélateur 10 minutes avant de peindre pour la laisser revenir à température ambiante.", "Remove palette from freezer 10 minutes before painting to let it warm up.")
                                                                } else {
                                                                    locale.tr("À l'air libre, les couches fines forment une pellicule sèche en quelques heures.", "Open to room air, thin films skin over within hours.")
                                                                }
                                                            }
                                                        }
                                                    }
                                                    span { class: "pill pill-accent", style: "font-size: 14px; padding: 6px 14px; font-weight: 750;",
                                                        {
                                                            if usable_days >= 7.0 {
                                                                format!("{:.0} semaines (environ {:.0} jours)", usable_days / 7.0, usable_days)
                                                            } else {
                                                                format!("{:.1} jours", usable_days)
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        },

                        MiniatureTab::Pigments => rsx! {
                            div { class: "grid-2", style: "grid-template-columns: 1fr 1.35fr; gap: 16px;",
                                // Left: Pigment Selector
                                div { class: "card",
                                    h3 { class: "card-title", {locale.tr("🏜️ Poudres de Pigments de Weathering", "🏜️ Weathering Dry Pigments")} }
                                    p { class: "muted", style: "font-size: 12px; margin-bottom: 14px;",
                                        {locale.tr("Les pigments purs permettent de recréer l'accumulation de poussière, terre calcinée et suie avec une matité absolue impossible à atteindre avec un liant.", "Dry pigments recreate dust, baked earth and engine soot with dead-flat velvety texture.")}
                                    }

                                    div { style: "display: flex; flex-direction: column; gap: 8px;",
                                        for pig in DryPigmentEffect::ALL {
                                            {
                                                let is_sel = *selected_pigment.read() == pig;
                                                rsx! {
                                                    div {
                                                        class: if is_sel { "card-subtle active" } else { "card-subtle" },
                                                        style: "cursor: pointer; padding: 10px;",
                                                        onclick: move |_| selected_pigment.set(pig),
                                                        div { class: "row-between", style: "align-items: center;",
                                                            div { class: "row", style: "gap: 10px; align-items: center;",
                                                                ColorSwatch { hex: pig.dry_hex().to_string(), size: 28 }
                                                                strong { style: "font-size: 13px;", "{pig.label_i18n(locale)}" }
                                                            }
                                                            span { class: "mono", style: "font-size: 11px;", "{pig.dry_hex()}" }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                // Right: Wet Shift Comparison & Active Palette Mix
                                div { class: "card",
                                    {
                                        let pig = *selected_pigment.read();
                                        let tgt_rgb = Rgb::of_hex(pig.dry_hex()).unwrap_or_else(|_| Rgb::new(0.5, 0.5, 0.5));
                                        let sugg = services.mix_service.suggest_mixes(tgt_rgb, &active_candidates, 1, Some(3)).into_iter().next();

                                        rsx! {
                                            div { class: "row-between", style: "margin-bottom: 14px;",
                                                div { class: "row", style: "gap: 12px; align-items: center;",
                                                    ColorSwatch { hex: pig.dry_hex().to_string(), size: 36 }
                                                    div {
                                                        h3 { class: "card-title", style: "margin: 0;", "{pig.label_i18n(locale)}" }
                                                        span { class: "pill pill-accent", "Weathering Pigment Simulation" }
                                                    }
                                                }
                                            }

                                            // Wet-shift visual comparison
                                            div { class: "card-subtle", style: "margin-bottom: 14px;",
                                                div { style: "font-weight: 700; font-size: 12.5px; margin-bottom: 8px;",
                                                    {locale.tr("🔍 Altération de Teinte : Effet Sec vs Fixé au Solvant", "🔍 Color Shift: Dry Powder vs Fixed Tone")}
                                                }
                                                div { class: "grid-2", style: "gap: 10px;",
                                                    div { class: "card", style: "padding: 10px; text-align: center;",
                                                        ColorSwatch { hex: pig.dry_hex().to_string(), size: 48 }
                                                        div { style: "font-weight: 700; font-size: 12px; margin-top: 6px;",
                                                            {locale.tr("Poudre Sèche Pure", "Pure Dry Powder")}
                                                        }
                                                        span { class: "pill", style: "font-size: 10.5px; margin-top: 4px;", "Ultra-mat / Velouté" }
                                                    }
                                                    div { class: "card", style: "padding: 10px; text-align: center;",
                                                        ColorSwatch { hex: pig.fixed_hex().to_string(), size: 48 }
                                                        div { style: "font-weight: 700; font-size: 12px; margin-top: 6px;",
                                                            {locale.tr("Fixé à l'Essence Sans Odeur", "Fixed with Odorless Spirits")}
                                                        }
                                                        span { class: "pill pill-warn", style: "font-size: 10.5px; margin-top: 4px;", "+2 crans plus sombre" }
                                                    }
                                                }
                                            }

                                            // Live mix from chosen palette!
                                            div { class: "card-subtle", style: "margin-bottom: 14px; border: 1px solid var(--accent-border);",
                                                div { style: "font-weight: 700; font-size: 12px; margin-bottom: 6px; color: var(--accent);",
                                                    {locale.tr("🧪 Reproduire ce ton de poussière en jus d'huile avec votre palette :", "🧪 Equivalent oil glaze mix from your palette:")}
                                                }
                                                if let Some(ref s) = sugg {
                                                    div { class: "row-between", style: "align-items: center;",
                                                        div { class: "row", style: "gap: 10px; align-items: center;",
                                                            ColorSwatch { hex: s.color.to_hex(), size: 30 }
                                                            div {
                                                                div { style: "font-weight: 700; font-size: 13px;", "{s.describe_i18n(locale)}" }
                                                                div { class: "muted", style: "font-size: 11px;", "{s.verdict_i18n(locale)}" }
                                                            }
                                                        }
                                                        DeltaEBadge { delta_e: s.delta_e }
                                                    }
                                                } else {
                                                    span { class: "muted", style: "font-size: 11.5px;", {locale.tr("Aucun tube disponible dans la palette sélectionnée", "No matching tubes available in selected palette")} }
                                                }
                                            }

                                            // Pro Tip Card
                                            div { class: "card-subtle",
                                                div { style: "font-weight: 700; font-size: 12px; margin-bottom: 4px; color: var(--accent);",
                                                    {locale.tr("🖌️ Technique d'application sans souffler la poudre :", "🖌️ Pro Application Technique:")}
                                                }
                                                div { style: "font-size: 12.5px; line-height: 1.45;",
                                                    "{pig.application_tip_i18n(locale)}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        },

                        MiniatureTab::RecipeSheet => rsx! {
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
                                                        let mut sheet = recipe_sheet.write();
                                                        sheet.base_acrylic = format!("{} ({})", p.name, p.brand);
                                                        sheet.base_hex = p.hex_color.clone();
                                                        sheet.oil_mix_formula = mix.as_ref().map(|m| m.describe_i18n(locale)).unwrap_or_default();
                                                        sheet.mixed_hex = mix.map(|m| m.color.to_hex()).unwrap_or_default();
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
                        },

                        MiniatureTab::HeatBox => rsx! {
                            div { class: "grid-2", style: "grid-template-columns: 1fr 1fr;",
                                div { class: "card",
                                    h3 { class: "card-title", {locale.tr("Simulateur de Boîte Chauffante / Déshydrateur", "Heat Box / Dehydrator Curing Simulator")} }
                                    p { class: "muted", style: "font-size: 12px; margin-bottom: 14px;",
                                        {locale.tr("L'huile polymérise beaucoup plus vite avec la chaleur (loi d'Arrhenius : le taux double tous les 10°C). Une pièce séchant en 24h à 20°C sèche en quelques heures à 40°C.", "Oil cross-linking accelerates under heat (Arrhenius rate: doubles every 10°C). A piece taking 24h at 20°C dries in a few hours at 40°C.")}
                                    }

                                    div { class: "card-subtle", style: "margin-bottom: 16px;",
                                        div { class: "row-between", style: "margin-bottom: 8px;",
                                            strong { {locale.tr("Température de la boîte chauffante :", "Chamber Temperature:")} }
                                            span { class: "pill pill-accent", style: "font-size: 14px; font-weight: 800;", "{heat_temp:.0} °C" }
                                        }
                                        input {
                                            r#type: "range",
                                            style: "width: 100%;",
                                            min: "20",
                                            max: "55",
                                            value: "{heat_temp}",
                                            oninput: move |ev| {
                                                if let Ok(v) = ev.value().parse::<f64>() {
                                                    heat_temp.set(v);
                                                }
                                            }
                                        }
                                    }

                                    if let Some(warn) = heat_calc.safety_warning_i18n(locale) {
                                        div { class: "card-subtle", style: "border-color: var(--danger); background: var(--pill-danger-bg); margin-bottom: 12px;",
                                            strong { style: "color: var(--danger); font-size: 12px;", "{warn}" }
                                        }
                                    }
                                }

                                div { class: "card",
                                    h3 { class: "card-title", {locale.tr("Gain de Temps & Sécurité Figurine", "Time Savings & Miniature Safety")} }
                                    {
                                        let factor = heat_calc.acceleration_factor();
                                        let base_h = *heat_base_hours.read();
                                        let accel_h = heat_calc.accelerated_hours(base_h);
                                        rsx! {
                                            div { class: "card-subtle", style: "text-align: center; padding: 20px; margin-bottom: 16px;",
                                                div { class: "muted", style: "font-size: 12px;", {locale.tr("Facteur d'accélération thermique :", "Thermal Acceleration Factor:")} }
                                                div { class: "mono", style: "font-size: 32px; font-weight: 800; color: var(--accent); margin: 6px 0;",
                                                    "{factor:.1}x plus rapide"
                                                }
                                                div { style: "font-size: 13.5px; font-weight: 600;",
                                                    "{base_h:.0} h ", {locale.tr("à température ambiante (20°C)", "at ambient (20°C)")},
                                                    " ➔ ",
                                                    strong { style: "color: var(--pill-ok-text);", "{accel_h:.1} h " },
                                                    {locale.tr("en boîte chauffante", "in heat chamber")}
                                                }
                                            }

                                            div { class: "card-subtle",
                                                div { style: "font-weight: 700; font-size: 12px; margin-bottom: 4px;",
                                                    {locale.tr("🛡️ Recommandation Matériaux Figurines :", "🛡️ Miniature Materials Guideline:")}
                                                }
                                                p { class: "muted", style: "margin: 0; font-size: 12px; line-height: 1.45;",
                                                    {locale.tr(
                                                        "Réglez idéalement votre déshydrateur ou boîte entre 38°C et 42°C. C'est la zone idéale : 4x plus rapide et 100% sûr pour le plastique Games Workshop, l'impression 3D résine et le métal.",
                                                        "Ideal dehydrator/box setting is 38°C to 42°C: achieves ~4x faster curing while remaining 100% safe for Games Workshop plastic, 3D resin prints, and white metal."
                                                    )}
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        },

                        MiniatureTab::SpecialEffects => rsx! {
                            div { class: "grid-2", style: "grid-template-columns: 1fr 1.35fr;",
                                div { class: "card",
                                    h3 { class: "card-title", {locale.tr("Effets Spéciaux & Textures aux Huiles", "Special Effects & Oil Textures")} }
                                    div { style: "display: flex; flex-direction: column; gap: 8px;",
                                        for gore in SpecialEffectGore::ALL {
                                            {
                                                let is_sel = *selected_gore.read() == gore;
                                                rsx! {
                                                    div {
                                                        class: if is_sel { "card-subtle active" } else { "card-subtle" },
                                                        style: if is_sel { "border-color: var(--accent); cursor: pointer; padding: 10px;" } else { "cursor: pointer; padding: 10px;" },
                                                        onclick: move |_| selected_gore.set(gore),
                                                        div { class: "row", style: "gap: 10px; align-items: center;",
                                                            ColorSwatch { hex: gore.hex_color().to_string(), size: 28 }
                                                            div {
                                                                div { style: "font-weight: 700; font-size: 13px;", "{gore.label_i18n(locale)}" }
                                                                div { class: "mono", style: "font-size: 11px; opacity: 0.7;", "{gore.hex_color()}" }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                div { class: "card",
                                    {
                                        let gore = *selected_gore.read();
                                        let tgt_rgb = Rgb::of_hex(gore.hex_color()).unwrap_or_else(|_| Rgb::new(0.5, 0.5, 0.5));
                                        let sugg = services.mix_service.suggest_mixes(tgt_rgb, &active_candidates, 1, Some(3)).into_iter().next();

                                        rsx! {
                                            div { class: "row-between", style: "margin-bottom: 14px;",
                                                div { class: "row", style: "gap: 12px; align-items: center;",
                                                    ColorSwatch { hex: gore.hex_color().to_string(), size: 40 }
                                                    div {
                                                        h3 { class: "card-title", style: "margin: 0;", "{gore.label_i18n(locale)}" }
                                                        span { class: "pill pill-accent", "Special FX Miniature Protocol" }
                                                    }
                                                }
                                            }

                                            // Dynamic calculated mix from chosen palette!
                                            div { class: "card-subtle", style: "margin-bottom: 14px; border: 1px solid var(--accent-border);",
                                                div { style: "font-weight: 700; font-size: 12px; margin-bottom: 6px; color: var(--accent);",
                                                    {locale.tr("🧪 Recette calculée avec votre palette : ", "🧪 Mix calculated with your palette: ")},
                                                    strong { "{palette_name_display}" }
                                                }
                                                if let Some(ref s) = sugg {
                                                    div { class: "row-between", style: "align-items: center;",
                                                        div { class: "row", style: "gap: 10px; align-items: center;",
                                                            ColorSwatch { hex: s.color.to_hex(), size: 32 }
                                                            div {
                                                                div { style: "font-weight: 750; font-size: 13.5px;", "{s.describe_i18n(locale)}" }
                                                                div { class: "muted", style: "font-size: 11px;", "{s.verdict_i18n(locale)}" }
                                                            }
                                                        }
                                                        DeltaEBadge { delta_e: s.delta_e }
                                                    }
                                                } else {
                                                    div { class: "muted", style: "font-size: 12px;",
                                                        {locale.tr("Aucun tube disponible dans la palette sélectionnée pour reproduire cet effet.", "No matching tubes available in selected palette for this effect.")}
                                                    }
                                                }
                                            }

                                            div { class: "card-subtle",
                                                div { style: "font-weight: 700; font-size: 12px; margin-bottom: 4px; color: var(--accent);",
                                                    {locale.tr("🖌️ Technique d'application :", "🖌️ Application Technique:")}
                                                }
                                                div { style: "font-size: 12.5px; line-height: 1.45;",
                                                    "{gore.technique_i18n(locale)}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        },

                        MiniatureTab::BrushCare => rsx! {
                            div { class: "card",
                                div { class: "row-between", style: "margin-bottom: 14px;",
                                    div {
                                        h3 { class: "card-title", style: "margin: 0;", {locale.tr("Soin des Pinceaux de Martre Kolinsky & Entretien", "Kolinsky Sable Brush Care & Maintenance")} }
                                        div { class: "muted", style: "font-size: 12px; margin-top: 2px;",
                                            {locale.tr("Préservez la pointe fine de vos pinceaux les plus précieux face aux solvants et à l'huile.", "Protect the fine needle tip of your sable brushes from solvent wear.")}
                                        }
                                    }
                                }

                                div { class: "grid-2", style: "grid-template-columns: 1.2fr 0.8fr; gap: 16px;",
                                    div { style: "display: flex; flex-direction: column; gap: 8px;",
                                        for (idx, brush) in brushes.read().iter().enumerate() {
                                            {
                                                let needs_soap = brush.needs_deep_clean();
                                                rsx! {
                                                    div { class: "card-subtle", style: "padding: 10px 14px;",
                                                        div { class: "row-between", style: "align-items: center;",
                                                            div {
                                                                strong { "{brush.name} ({brush.size})" }
                                                                div { class: "muted", style: "font-size: 11px;", "{brush.brand} · {brush.hair_type}" }
                                                            }
                                                            div { class: "row", style: "gap: 8px; align-items: center;",
                                                                span { class: if needs_soap { "pill pill-warn" } else { "pill pill-ok" },
                                                                    "{brush.uses_count} ", {locale.tr("sessions", "sessions")}
                                                                }
                                                                button {
                                                                    class: "btn",
                                                                    onclick: move |_| {
                                                                        let mut b_list = brushes.read().clone();
                                                                        if let Some(b) = b_list.get_mut(idx) {
                                                                            b.uses_count += 1;
                                                                        }
                                                                        brushes.set(b_list);
                                                                    },
                                                                    "+ 1 use"
                                                                }
                                                                button {
                                                                    class: "btn",
                                                                    onclick: move |_| {
                                                                        let mut b_list = brushes.read().clone();
                                                                        if let Some(b) = b_list.get_mut(idx) {
                                                                            b.uses_count = 0;
                                                                        }
                                                                        brushes.set(b_list);
                                                                    },
                                                                    "🧼 ", {locale.tr("Nettoyé", "Cleaned")}
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    div { class: "card-subtle",
                                        div { style: "font-weight: 700; font-size: 13px; margin-bottom: 8px; color: var(--danger);",
                                            "⚠️ ", {locale.tr("Règles d'or de longévité du poil de martre :", "Golden Rules of Sable Brush Longevity:")}
                                        }
                                        ul { style: "margin-left: 20px; font-size: 12.5px; line-height: 1.5; display: flex; flex-direction: column; gap: 8px;",
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
                        },

                        MiniatureTab::VarnishStudio => rsx! {
                            div { class: "grid-2", style: "grid-template-columns: 1fr 1.25fr;",
                                div { class: "card",
                                    h3 { class: "card-title", {locale.tr("Simulateur d'Harmonisation par Vernis", "Varnish Finish Harmonizer")} }
                                    p { class: "muted", style: "font-size: 12px; margin-bottom: 14px;",
                                        {locale.tr("L'alliance acrylique et huile crée des disparités de brillance. Le vernis final scelle et harmonise l'ensemble de la figurine.", "Mixing acrylics and oils causes uneven shine. The final varnish unifies the entire miniature.")}
                                    }

                                    div { style: "display: flex; flex-direction: column; gap: 8px;",
                                        for v in VarnishFinish::ALL {
                                            {
                                                let is_sel = *selected_varnish.read() == v;
                                                rsx! {
                                                    div {
                                                        class: if is_sel { "card-subtle active" } else { "card-subtle" },
                                                        style: if is_sel { "border-color: var(--accent); cursor: pointer; padding: 10px;" } else { "cursor: pointer; padding: 10px;" },
                                                        onclick: move |_| selected_varnish.set(v),
                                                        div { style: "font-weight: 700; font-size: 13px;", "{v.label_i18n(locale)}" }
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
                                            h3 { class: "card-title", "{v.label_i18n(locale)}" }
                                            div { class: "card-subtle", style: "font-size: 13.5px; line-height: 1.5; margin-bottom: 16px;",
                                                "{v.description_i18n(locale)}"
                                            }

                                            div { class: "card-subtle",
                                                div { style: "font-weight: 700; font-size: 12.5px; margin-bottom: 6px;",
                                                    {locale.tr("💡 Conseil de pro pour figurines de concours :", "💡 Competition Miniature Pro Tip:")}
                                                }
                                                p { class: "muted", style: "margin: 0; font-size: 12px; line-height: 1.45;",
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
                        },

                        MiniatureTab::BatchSquad => rsx! {
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
                                                        {
                                                            let is_done = squad.read().progress.get(m_idx).and_then(|r| r.get(s_idx)).copied().unwrap_or(false);
                                                            rsx! {
                                                                td { style: "text-align: center; cursor: pointer;",
                                                                    onclick: move |_| {
                                                                        let mut s = squad.read().clone();
                                                                        s.toggle(m_idx, s_idx);
                                                                        squad.set(s);
                                                                    },
                                                                    span {
                                                                        class: if is_done { "pill pill-ok" } else { "pill" },
                                                                        style: "padding: 2px 8px; font-size: 11px;",
                                                                        if is_done { "✓" } else { "○" }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        },

                        MiniatureTab::OslEngine => rsx! {
                            div { class: "grid-2", style: "grid-template-columns: 1fr 1.35fr;",
                                div { class: "card",
                                    h3 { class: "card-title", {locale.tr("Sources d'Énergie & Effets Luminescents (OSL)", "Object Source Lighting (OSL) Energy Sources")} }
                                    div { style: "display: flex; flex-direction: column; gap: 8px;",
                                        for preset in &OslPreset::ALL {
                                            {
                                                let is_sel = selected_osl.read().name_fr == preset.name_fr;
                                                rsx! {
                                                    div {
                                                        class: if is_sel { "card-subtle active" } else { "card-subtle" },
                                                        style: if is_sel { "border-color: var(--accent); cursor: pointer; padding: 10px;" } else { "cursor: pointer; padding: 10px;" },
                                                        onclick: move |_| selected_osl.set(preset),
                                                        div { class: "row", style: "gap: 10px; align-items: center;",
                                                            ColorSwatch { hex: preset.glow_hex.to_string(), size: 28 }
                                                            div {
                                                                div { style: "font-weight: 700; font-size: 13px;", "{preset.name_i18n(locale)}" }
                                                                div { class: "mono", style: "font-size: 11px; opacity: 0.7;", "{preset.glow_hex}" }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }

                                div { class: "card",
                                    {
                                        let osl = *selected_osl.read();
                                        rsx! {
                                            div { class: "row-between", style: "margin-bottom: 14px;",
                                                div { class: "row", style: "gap: 12px; align-items: center;",
                                                    ColorSwatch { hex: osl.glow_hex.to_string(), size: 40 }
                                                    div {
                                                        h3 { class: "card-title", style: "margin: 0;", "{osl.name_i18n(locale)}" }
                                                        span { class: "pill pill-accent", "High-Intensity OSL Emitter" }
                                                    }
                                                }

                                                button {
                                                    class: "btn",
                                                    onclick: {
                                                        let h = osl.glow_hex.to_string();
                                                        move |_| {
                                                            state.mixer_target_hex.set(h.clone());
                                                            state.section.set(crate::state::Section::Mixer);
                                                        }
                                                    },
                                                    "🧪 ",
                                                    {locale.tr("Envoyer au Mélangeur", "Send to Mixer")}
                                                }
                                            }

                                            // 3-step OSL Radial Decay swatches with calculated mixes!
                                            div { style: "display: flex; flex-direction: column; gap: 8px; margin-bottom: 16px;",
                                                for (_idx, (zone_name, zone_hex)) in [
                                                    (locale.tr("1. Cœur Incandescent (Point chaud)", "1. White-Hot Core"), osl.core_hex),
                                                    (locale.tr("2. Halo d'Énergie Surchauffé", "2. Saturated Glow"), osl.glow_hex),
                                                    (locale.tr("3. Réflexion Ambiante Décroissante", "3. Ambient Reflection"), osl.ambient_decay_hex),
                                                ].iter().enumerate() {
                                                    {
                                                        let tgt_rgb = Rgb::of_hex(zone_hex).unwrap_or_else(|_| Rgb::new(0.5, 0.5, 0.5));
                                                        let sugg = services.mix_service.suggest_mixes(tgt_rgb, &active_candidates, 1, Some(3)).into_iter().next();

                                                        rsx! {
                                                            div { class: "card-subtle", style: "padding: 8px 12px;",
                                                                div { class: "row-between", style: "align-items: center;",
                                                                    div { class: "row", style: "gap: 10px; align-items: center;",
                                                                        ColorSwatch { hex: zone_hex.to_string(), size: 28 }
                                                                        div {
                                                                            div { style: "font-weight: 700; font-size: 12px;", "{zone_name}" }
                                                                            div { class: "mono", style: "font-size: 10.5px; color: var(--text-dim);", "{zone_hex}" }
                                                                        }
                                                                    }

                                                                    if let Some(ref s) = sugg {
                                                                        div { class: "row", style: "gap: 8px; align-items: center;",
                                                                            ColorSwatch { hex: s.color.to_hex(), size: 28 }
                                                                            div { style: "text-align: right; line-height: 1.3;",
                                                                                div { style: "font-weight: 650; font-size: 12px; color: var(--accent);", "{s.describe_i18n(locale)}" }
                                                                                div { style: "display: flex; gap: 4px; justify-content: flex-end; align-items: center; margin-top: 2px;",
                                                                                    DeltaEBadge { delta_e: s.delta_e }
                                                                                }
                                                                            }
                                                                        }
                                                                    } else {
                                                                        span { class: "muted", style: "font-size: 11px;", {locale.tr("Aucun tube disponible", "No tubes available")} }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }

                                            div { class: "card-subtle", style: "line-height: 1.5; font-size: 12.5px;",
                                                div { style: "font-weight: 700; font-size: 13px; margin-bottom: 6px; color: var(--accent);",
                                                    {locale.tr("🎯 Protocole de Réussite de l'OSL aux Huiles :", "🎯 Pro Oil OSL Painting Protocol:")}
                                                }
                                                ol { style: "margin-left: 20px; display: flex; flex-direction: column; gap: 6px;",
                                                    li {
                                                        strong { {locale.tr("Creux en blanc pur : ", "Pure white recess: ")} }
                                                        {locale.tr("Poser du blanc de titane pur au fond des rainures de plasma ou dans les yeux.", "Lay down pure titanium white in the plasma coils or eye lenses.")}
                                                    }
                                                    li {
                                                        strong { {locale.tr("Filtre d'huile saturé : ", "Saturated oil filter: ")} }
                                                        {locale.tr("Poser le halo coloré en glacis translucide par-dessus le blanc et sur les arêtes proches.", "Apply the glowing saturated glaze over the white and surrounding armor rims.")}
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        },

                        MiniatureTab::OilHarmonies => rsx! {
                            div { style: "display: flex; flex-direction: column; gap: 16px;",
                                div { class: "card",
                                    div { class: "row-between", style: "margin-bottom: 12px; align-items: center;",
                                        div {
                                            h3 { class: "card-title", style: "margin: 0;",
                                                {locale.tr("🎡 Roue Chromatique & Harmonies d'Ombrage à l'Huile", "🎡 Color Harmonies & Oil Shadow Wheel")}
                                            }
                                            p { class: "muted", style: "font-size: 12px; margin: 2px 0 0 0;",
                                                {locale.tr("Contrairement aux aplats acryliques, l'huile permet d'éteindre une teinte avec sa complémentaire translucide (Ombre d'Extinction) plutôt qu'avec du noir qui salit.", "Unlike flat acrylics, oils allow desaturating via a translucent complementary filter (Extinction Shadow) instead of muddying tube black.")}
                                            }
                                        }
                                    }

                                    div { class: "row", style: "gap: 12px; align-items: center; margin-bottom: 12px; flex-wrap: wrap;",
                                        strong { style: "font-size: 13px;", {locale.tr("Teinte maîtresse de départ :", "Base Color:")} }
                                        div { class: "row", style: "gap: 8px; align-items: center;",
                                            ColorSwatch { hex: harmony_paint.as_ref().map(|p| p.hex_color.clone()).unwrap_or_default(), size: 28 }
                                            select {
                                                class: "input",
                                                value: "{harmony_paint.as_ref().map(paint_key).unwrap_or_default()}",
                                                onchange: move |ev| harmony_key.set(Some(ev.value())),
                                                for p in active_candidates.iter() {
                                                    option { value: "{paint_key(p)}", "{p.name} ({p.brand})" }
                                                }
                                            }
                                        }
                                    }
                                }

                                // Grid of 8 calculated harmonies
                                {
                                    let base_rgb = harmony_paint.as_ref().map(|p| p.color()).unwrap_or_else(|| Rgb::new(0.5, 0.5, 0.5));
                                    let items = OilHarmoniesCalculator::calculate(base_rgb);

                                    rsx! {
                                        div { class: "grid-2", style: "gap: 14px;",
                                            for item in items {
                                                {
                                                    let tgt_rgb = Rgb::of_hex(&item.hex).unwrap_or_else(|_| Rgb::new(0.5, 0.5, 0.5));
                                                    let sugg = services.mix_service.suggest_mixes(tgt_rgb, &active_candidates, 1, Some(3)).into_iter().next();
                                                    let h = item.hex.clone();

                                                    rsx! {
                                                        div { class: "card", style: "padding: 14px;",
                                                            div { class: "row-between", style: "align-items: center; margin-bottom: 8px;",
                                                                div { class: "row", style: "gap: 10px; align-items: center;",
                                                                    ColorSwatch { hex: item.hex.clone(), size: 32 }
                                                                    div {
                                                                        strong { style: "font-size: 13.5px;", "{item.name_i18n(locale)}" }
                                                                        span { class: "mono", style: "font-size: 11px; margin-left: 6px; color: var(--text-dim);", "{item.hex}" }
                                                                    }
                                                                }
                                                                button {
                                                                    class: "btn",
                                                                    style: "padding: 3px 8px; font-size: 11px;",
                                                                    onclick: {
                                                                        let target = h.clone();
                                                                        move |_| {
                                                                            state.mixer_target_hex.set(target.clone());
                                                                            state.section.set(crate::state::Section::Mixer);
                                                                        }
                                                                    },
                                                                    "🧪"
                                                                }
                                                            }

                                                            div { class: "card-subtle", style: "padding: 8px 10px; margin-bottom: 8px; font-size: 12px;",
                                                                strong { "🎯 " },
                                                                span { "{item.role_i18n(locale)}" }
                                                            }

                                                            div { style: "font-size: 11.5px; line-height: 1.4; color: var(--accent); margin-bottom: 10px;",
                                                                "🖌️ {item.oil_tip_i18n(locale)}"
                                                            }

                                                            // Calculated recipe from active palette
                                                            div { class: "card-subtle", style: "border: 1px solid var(--accent-border); padding: 8px 10px;",
                                                                div { style: "font-weight: 700; font-size: 11px; margin-bottom: 4px; color: var(--text-dim);",
                                                                    {locale.tr("Recette calculée avec votre palette :", "Calculated Palette Mix:")}
                                                                }
                                                                if let Some(ref s) = sugg {
                                                                    div { class: "row-between", style: "align-items: center;",
                                                                        div { class: "row", style: "gap: 8px; align-items: center;",
                                                                            ColorSwatch { hex: s.color.to_hex(), size: 24 }
                                                                            div { style: "font-weight: 650; font-size: 12px;", "{s.describe_i18n(locale)}" }
                                                                        }
                                                                        DeltaEBadge { delta_e: s.delta_e }
                                                                    }
                                                                } else {
                                                                    span { class: "muted", style: "font-size: 11px;", {locale.tr("Aucun tube disponible", "No matching tubes in palette")} }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        },

                        MiniatureTab::OilConsistency => rsx! {
                            div { class: "grid-2", style: "grid-template-columns: 1fr 1fr; gap: 16px;",
                                div { class: "card",
                                    h3 { class: "card-title", {locale.tr("💧 Calculateur de Consistance & Gouttes de Dilution", "💧 Oil Consistency & Dropper Dosage Calculator")} }
                                    p { class: "muted", style: "font-size: 12px; margin-bottom: 14px;",
                                        {locale.tr("Contrairement à l'acrylique fluide, l'huile sort en pâte visqueuse du tube (parts / noisettes), et ce sont les solvants minéraux et médiums qui se dosent au compte-gouttes.", "Unlike fluid acrylics, oil leaves the tube as a dense paste (parts / hazelnuts), while solvents and mediums are metered drop by drop.")}
                                    }

                                    // Technique selection
                                    div { style: "display: flex; flex-direction: column; gap: 8px; margin-bottom: 16px;",
                                        for tech in Technique::for_system(PaintSystem::Oil) {
                                            {
                                                let is_sel = *consistency_technique.read() == tech;
                                                let drops = thinner_drops_per_part(tech, PaintSystem::Oil);
                                                let consistency = Consistency::of(tech);
                                                rsx! {
                                                    div {
                                                        class: if is_sel { "card-subtle active" } else { "card-subtle" },
                                                        style: "cursor: pointer; padding: 10px;",
                                                        onclick: move |_| consistency_technique.set(tech),
                                                        div { class: "row-between", style: "align-items: center; margin-bottom: 4px;",
                                                            strong { style: "font-size: 13px;", "{tech.label_i18n(locale)}" }
                                                            span { class: "pill pill-accent",
                                                                if drops == 0 {
                                                                    {locale.tr("pure", "neat")}
                                                                } else {
                                                                    "{drops} ", {locale.tr("gtt / part", "drops / part")}
                                                                }
                                                            }
                                                        }
                                                        div { class: "muted", style: "font-size: 11.5px;",
                                                            strong { "{consistency.label_i18n(locale)}. " }
                                                            "{consistency.description_i18n(locale)}"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    // Parts slider
                                    div { class: "card-subtle",
                                        div { class: "row-between", style: "margin-bottom: 8px;",
                                            strong { style: "font-size: 12.5px;", {locale.tr("Volume de pâte d'huile préparé :", "Oil Paint Volume:")} }
                                            span { class: "pill pill-ok", style: "font-size: 13px; font-weight: 750;", "{consistency_parts} parts (noisettes)" }
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
                                                    div { class: "muted", style: "font-size: 10.5px; margin-top: 2px;", "Noisettes / Têtes d'allumette" }
                                                }
                                                div { class: "card-subtle", style: "text-align: center; padding: 12px;",
                                                    div { style: "font-size: 26px; font-weight: 800; color: var(--accent-ok);", "{tot_solvent_drops}" }
                                                    strong { style: "font-size: 12px;", {locale.tr("Gouttes de ", "Drops of ")}, "{thinner.label_i18n(locale).to_lowercase()}" }
                                                    div { class: "muted", style: "font-size: 10.5px; margin-top: 2px;", "Compte-gouttes / Pipette" }
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
                        },

                        MiniatureTab::GlazePreview => rsx! {
                            div { style: "display: flex; flex-direction: column; gap: 16px;",
                                div { class: "card",
                                    div { class: "row-between", style: "align-items: center; margin-bottom: 12px;",
                                        div {
                                            h3 { class: "card-title", style: "margin: 0;",
                                                {locale.tr("🖼️ Simulation Optique de Glacis d'Huile sur Sous-Couche Zénithale", "🖼️ Optical Oil Glaze on Zenithal Primer Simulator")}
                                            }
                                            p { class: "muted", style: "font-size: 12px; margin: 2px 0 0 0;",
                                                {locale.tr("Visualisez la physique du glacis d'huile : la lumière traverse le pigment transparent, rebondit sur le blanc spéculaire zénithal et s'éteint dans les ombres physiques.", "Simulate physical oil glazes: light transmits through transparent pigments, reflects off zenithal white, and extinguishes in deep recesses.")}
                                            }
                                        }
                                    }

                                    div { class: "grid-2", style: "gap: 16px; margin-bottom: 16px;",
                                        div {
                                            label { style: "font-size: 12px; font-weight: 600; display: block; margin-bottom: 6px;",
                                                {locale.tr("Teinte d'huile à vitrifier :", "Glaze Oil Pigment Tone:")}
                                            }
                                            div { class: "row", style: "gap: 8px; align-items: center;",
                                                ColorSwatch { hex: glaze_paint.as_ref().map(|p| p.hex_color.clone()).unwrap_or_default(), size: 28 }
                                                select {
                                                    class: "input",
                                                    style: "flex: 1;",
                                                    value: "{glaze_paint.as_ref().map(paint_key).unwrap_or_default()}",
                                                    onchange: move |ev| glaze_key.set(Some(ev.value())),
                                                    for p in active_candidates.iter() {
                                                        option { value: "{paint_key(p)}", "{p.name} ({p.brand})" }
                                                    }
                                                }
                                            }
                                        }

                                        div {
                                            div { class: "row-between", style: "margin-bottom: 6px;",
                                                label { style: "font-size: 12px; font-weight: 600;", {locale.tr("Intensité / Saturation du glacis :", "Glaze Saturation Strength:")} }
                                                span { class: "pill pill-accent", "{glaze_strength * 100.0:.0}%" }
                                            }
                                            input {
                                                r#type: "range",
                                                min: "0.20",
                                                max: "0.95",
                                                step: "0.05",
                                                value: "{glaze_strength}",
                                                oninput: move |ev| {
                                                    if let Ok(v) = ev.value().parse::<f64>() {
                                                        glaze_strength.set(v);
                                                    }
                                                },
                                                style: "width: 100%;",
                                            }
                                        }
                                    }
                                }

                                // 5-step Zenithal Glaze Visualizer
                                div { class: "card",
                                    h3 { class: "card-title", {locale.tr("Échelle Optique Zénithale Glacée", "Optical Zenithal Glazed Ladder")} }
                                    div { style: "display: flex; flex-direction: column; gap: 8px;",
                                        for (name_fr, name_en, lum) in OilGlazeZenithalSimulator::ZENITHAL_LEVELS {
                                            {
                                                let step_name = if locale.is_fr() { name_fr } else { name_en };
                                                let undercoat_hex = {
                                                    let g = (lum * 255.0).round() as u8;
                                                    format!("#{g:02X}{g:02X}{g:02X}")
                                                };
                                                let glazed_rgb = OilGlazeZenithalSimulator::simulate_glaze(glaze_paint.as_ref().map(|p| p.color()).unwrap_or_else(|| Rgb::new(0.5, 0.5, 0.5)), lum, *glaze_strength.read());
                                                let glazed_hex = glazed_rgb.to_hex();
                                                let sugg = services.mix_service.suggest_mixes(glazed_rgb, &active_candidates, 1, Some(3)).into_iter().next();

                                                rsx! {
                                                    div { class: "card-subtle", style: "padding: 10px 14px;",
                                                        div { class: "row-between", style: "align-items: center;",
                                                            div { class: "row", style: "gap: 12px; align-items: center;",
                                                                // Left: Primer value
                                                                div { style: "text-align: center;",
                                                                    ColorSwatch { hex: undercoat_hex.clone(), size: 28 }
                                                                    div { class: "mono", style: "font-size: 9.5px; margin-top: 2px;", "{undercoat_hex}" }
                                                                }
                                                                span { style: "font-size: 16px; color: var(--accent);", "➔" }
                                                                // Right: Glazed result
                                                                div { style: "text-align: center;",
                                                                    ColorSwatch { hex: glazed_hex.clone(), size: 36 }
                                                                    div { class: "mono", style: "font-size: 10px; font-weight: 700; margin-top: 2px;", "{glazed_hex}" }
                                                                }
                                                                div {
                                                                    strong { style: "font-size: 13px;", "{step_name}" }
                                                                    div { class: "muted", style: "font-size: 11px;",
                                                                        {locale.tr("Luminance physique sous-jacente : ", "Physical undercoat luminance: ")},
                                                                        "{lum * 100.0:.0}%"
                                                                    }
                                                                }
                                                            }

                                                            // Palette mix match
                                                            if let Some(ref s) = sugg {
                                                                div { class: "row", style: "gap: 8px; align-items: center;",
                                                                    div { style: "text-align: right;",
                                                                        div { style: "font-weight: 650; font-size: 12px;", "{s.describe_i18n(locale)}" }
                                                                    }
                                                                    DeltaEBadge { delta_e: s.delta_e }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        MiniatureTab::PaletteLayout => rsx! {
                            div { style: "display: flex; flex-direction: column; gap: 16px;",
                                div { class: "card",
                                    div { class: "row-between", style: "margin-bottom: 12px; align-items: center;",
                                        h3 { class: "card-title", style: "margin: 0;",
                                            "🗺️ ", {locale.tr("Guide d'Agencement de la Palette Physique d'Atelier", "Physical Studio Palette Layout Guide")}
                                        }
                                        span { class: "pill pill-accent",
                                            {locale.tr("Prévention de la contamination des pigments", "Pigment Contamination Prevention")}
                                        }
                                    }
                                    p { class: "muted", style: "margin-top: 0; font-size: 13px;",
                                        {locale.tr(
                                            "À l'huile, la disposition des noisettes de couleur sur la dalle de verre ou la palette étanche détermine la pureté de vos mélanges. Les pigments ultra-staining (Bleu de Prusse, Phtalo) doivent impérativement être isolés des blancs.",
                                            "In miniature oils, paint dollop placement on your glass slab or stay-wet box governs mix purity. High-staining pigments (Prussian Blue, Phthalo) must be strictly isolated from whites."
                                        )}
                                    }

                                    div { class: "row", style: "gap: 8px; margin-bottom: 16px; flex-wrap: wrap;",
                                        for l in PaletteLayoutType::ALL {
                                            button {
                                                class: if *selected_palette_layout.read() == l { "btn-primary" } else { "btn" },
                                                onclick: move |_| selected_palette_layout.set(l),
                                                "{l.title_i18n(locale)}"
                                            }
                                        }
                                    }

                                    div { class: "card-subtle", style: "margin-bottom: 16px;",
                                        div { class: "row-between", style: "margin-bottom: 8px;",
                                            strong { style: "font-size: 14px; color: var(--accent);",
                                                "🛡️ ", {selected_palette_layout.read().title_i18n(locale)}
                                            }
                                        }
                                        div { style: "font-size: 12.5px; line-height: 1.5; margin-bottom: 8px;",
                                            strong { {locale.tr("Support recommandé : ", "Recommended surface: ")} }
                                            {selected_palette_layout.read().surface_recommendation_i18n(locale)}
                                        }
                                        div { style: "font-size: 12.5px; line-height: 1.5;",
                                            strong { {locale.tr("Règle d'agencement d'atelier : ", "Studio placement rule: ")} }
                                            {selected_palette_layout.read().setup_advice_i18n(locale)}
                                        }
                                    }

                                    h4 { style: "font-size: 13px; margin: 12px 0 8px 0;",
                                        {locale.tr("Disposition des Noisettes de Peinture (Pourtour) :", "Paint Dollop Perimeter Placement:")}
                                    }
                                    div { style: "display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: 10px;",
                                        for slot in selected_palette_layout.read().slots(locale) {
                                            div { class: "card-subtle", style: "padding: 10px 12px; display: flex; flex-direction: column; justify-content: space-between;",
                                                div { class: "row", style: "gap: 10px; align-items: center; margin-bottom: 6px;",
                                                    ColorSwatch { hex: slot.hex.to_string(), size: 28 }
                                                    div {
                                                        div { style: "font-weight: 700; font-size: 13px;", "{slot.name}" }
                                                        div { class: "muted", style: "font-size: 11px;", "{slot.position_desc}" }
                                                    }
                                                }
                                                div { style: "font-size: 11.5px; color: var(--fg); opacity: 0.85; margin-bottom: 6px;",
                                                    "{slot.role}"
                                                }
                                                if let Some(ref w) = slot.safety_warning {
                                                    div { class: "badge badge-warning", style: "font-size: 10px; white-space: normal; padding: 4px 6px;",
                                                        "{w}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        },

                        MiniatureTab::VehicleWeathering => rsx! {
                            div { style: "display: flex; flex-direction: column; gap: 16px;",
                                div { class: "card",
                                    div { class: "row-between", style: "margin-bottom: 12px; align-items: center;",
                                        h3 { class: "card-title", style: "margin: 0;",
                                            "🪖 ", {locale.tr("Jus & Filtres Véhicules (Équivalents Abteilung 502 / Ammo Mig)", "AFV & Vehicle Weathering (Abteilung 502 / Mig Equivalents)")}
                                        }
                                        span { class: "pill pill-accent",
                                            {locale.tr("Formules aux Tubes d'Huile d'Artiste", "Artist Oil Tube Formulas")}
                                        }
                                    }
                                    p { class: "muted", style: "margin-top: 0; font-size: 13px;",
                                        {locale.tr(
                                            "Reproduisez fidèlement les teintes cultes de weathering pour chars et véhicules (Abteilung 502, AK, Ammo Mig) à partir de vos tubes d'huile classiques, avec le diluant inodore approprié pour préserver le vernis acrylique.",
                                            "Faithfully recreate iconic armor and vehicle weathering washes (Abteilung 502, AK, Ammo Mig) from standard artist oil tubes, using odorless spirits to protect acrylic coats."
                                        )}
                                    }

                                    div { class: "row", style: "gap: 8px; margin-bottom: 16px; flex-wrap: wrap;",
                                        for fx in VehicleEffectKind::ALL {
                                            button {
                                                class: if *selected_vehicle_fx.read() == fx { "btn-primary" } else { "btn" },
                                                onclick: move |_| selected_vehicle_fx.set(fx),
                                                "{fx.commercial_name()}"
                                            }
                                        }
                                    }

                                    {
                                        let current_fx = *selected_vehicle_fx.read();
                                        let target_rgb = Rgb::of_hex(current_fx.target_hex()).unwrap_or_else(|_| Rgb::new(0.3, 0.3, 0.3));
                                        let sugg = services.mix_service.suggest_mixes(target_rgb, &active_candidates, 1, Some(3)).into_iter().next();

                                        rsx! {
                                            div { class: "card-subtle", style: "padding: 16px;",
                                                div { class: "row-between", style: "align-items: center; margin-bottom: 14px;",
                                                    div { class: "row", style: "gap: 12px; align-items: center;",
                                                        ColorSwatch { hex: current_fx.target_hex().to_string(), size: 40 }
                                                        div {
                                                            div { style: "font-weight: 750; font-size: 15px; color: var(--accent);",
                                                                "{current_fx.commercial_name()}"
                                                            }
                                                            div { class: "muted", style: "font-size: 12px;",
                                                                "{current_fx.category_i18n(locale)}"
                                                            }
                                                        }
                                                    }
                                                    span { class: "pill", "{current_fx.finish_i18n(locale)}" }
                                                }

                                                div { style: "display: grid; gap: 14px; margin-bottom: 12px;",
                                                    div { class: "card", style: "background: var(--bg);",
                                                        strong { style: "font-size: 12.5px; color: var(--accent);",
                                                            {locale.tr("💧 Dilution & Solvant Recommandé :", "💧 Dilution & Recommended Solvent:")}
                                                        }
                                                        div { style: "font-size: 12px; margin-top: 6px;",
                                                            {
                                                                let tech = current_fx.technique();
                                                                let drops = thinner_drops_per_part(tech, PaintSystem::Oil);
                                                                let thinner = tech.default_medium_in(PaintSystem::Oil).label_i18n(locale).to_lowercase();
                                                                rsx! {
                                                                    div { style: "font-weight: 600; margin-bottom: 4px;",
                                                                        "{tech.label_i18n(locale)} · "
                                                                        if drops == 0 {
                                                                            {locale.tr("pâte pure", "neat paste")}
                                                                        } else {
                                                                            {locale.tr("1 part de pâte pour ", "1 part paste to ")}, "{drops} ", {locale.tr("gouttes de ", "drops of ")}, "{thinner}"
                                                                        }
                                                                    }
                                                                    div { class: "muted", "{current_fx.application_note_i18n(locale)}" }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }

                                                div { class: "row-between", style: "align-items: center; padding-top: 10px; border-top: 1px solid var(--border);",
                                                    span { style: "font-size: 12px; font-weight: 600;",
                                                        {locale.tr("Mélange calculé sur votre palette active :", "Mix computed from your active palette:")}
                                                    }
                                                    if let Some(ref s) = sugg {
                                                        div { class: "row", style: "gap: 8px; align-items: center;",
                                                            div { style: "font-weight: 650; font-size: 12px; color: var(--accent);",
                                                                "{s.describe_i18n(locale)}"
                                                            }
                                                            DeltaEBadge { delta_e: s.delta_e }
                                                        }
                                                    } else {
                                                        span { class: "muted", style: "font-size: 11px;", {locale.tr("Aucun tube disponible", "No tubes available")} }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        },

                        MiniatureTab::WetOnWetBlending => rsx! {
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
                                                        "{tech.label_i18n(locale)}"
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
                                                        "{tech.label_i18n(locale)}"
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
                                                        "{eval.rating.label_i18n(locale)}"
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
                        },

                        MiniatureTab::BatchMotherMix => rsx! {
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
                        },

                        MiniatureTab::PaletteFreshness => rsx! {
                            div { style: "display: flex; flex-direction: column; gap: 16px;",
                                div { class: "card",
                                    div { class: "row-between", style: "margin-bottom: 12px; align-items: center;",
                                        h3 { class: "card-title", style: "margin: 0;",
                                            "⏳ ", {locale.tr("Widget de Fraîcheur de Palette en Temps Réel", "Live Studio Palette Freshness Tracker")}
                                        }
                                        span { class: "pill pill-accent",
                                            {locale.tr("Dégradation Viscosité & Polymérisation", "Viscosity & Polymerization Drift")}
                                        }
                                    }
                                    p { class: "muted", style: "margin-top: 0; font-size: 13px;",
                                        {locale.tr(
                                            "Suivez la transformation physique de vos plots de peinture à l'huile pendant votre session. Sachez exactement quand le solvant s'est évaporé (phase beurre OPR) et quand éviter de rajouter du diluant pour ne pas créer de grumeaux.",
                                            "Track the physical drift of oil paint dollops during your session. Know exactly when solvent has flashed off (buttery OPR phase) and when to stop adding thinner to avoid curdling."
                                        )}
                                    }

                                    div { class: "grid-2", style: "gap: 16px; margin-bottom: 16px;",
                                        div { class: "card-subtle",
                                            label { style: "font-weight: 700; font-size: 13px; display: block; margin-bottom: 6px;",
                                                {locale.tr("Temps écoulé depuis la sortie du tube : ", "Elapsed time since squeeze: ")},
                                                "{*freshness_minutes.read() / 60}h {*freshness_minutes.read() % 60}m"
                                            }
                                            input {
                                                r#type: "range",
                                                min: "0",
                                                max: "900",
                                                step: "15",
                                                value: "{freshness_minutes}",
                                                oninput: move |ev: Event<FormData>| {
                                                    if let Ok(v) = ev.value().parse::<u32>() {
                                                        freshness_minutes.set(v);
                                                    }
                                                }
                                            }
                                        }

                                        div { class: "card-subtle",
                                            label { style: "font-weight: 700; font-size: 13px; display: block; margin-bottom: 6px;",
                                                {locale.tr("Température de l'atelier : ", "Workshop temperature: ")},
                                                "{freshness_temp:.1}°C"
                                            }
                                            input {
                                                r#type: "range",
                                                min: "15",
                                                max: "35",
                                                step: "1",
                                                value: "{freshness_temp}",
                                                oninput: move |ev: Event<FormData>| {
                                                    if let Ok(v) = ev.value().parse::<f64>() {
                                                        freshness_temp.set(v);
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    {
                                        let phase = LivePaletteFreshnessTracker::evaluate(*freshness_minutes.read(), *freshness_temp.read());
                                        rsx! {
                                            div { class: "card-subtle", style: "border-left: 4px solid var(--accent); padding: 16px;",
                                                div { class: "row-between", style: "margin-bottom: 10px; align-items: center;",
                                                    span { class: phase.badge_class(), style: "font-size: 13px; padding: 5px 12px; font-weight: 700;",
                                                        "{phase.title_i18n(locale)}"
                                                    }
                                                }

                                                div { style: "font-size: 13px; line-height: 1.5; margin-bottom: 10px;",
                                                    strong { {locale.tr("État physique de la peinture : ", "Physical state: ")} }
                                                    "{phase.physical_state_i18n(locale)}"
                                                }

                                                div { style: "font-size: 12.5px; line-height: 1.5; padding: 10px; background: var(--bg); border-radius: 6px;",
                                                    strong { style: "color: var(--accent);",
                                                        "💡 ", {locale.tr("Action recommandée : ", "Recommended action: ")}
                                                    }
                                                    "{phase.action_advice_i18n(locale)}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Stable key of a catalog paint in a select: brand and name are unique together.
fn paint_key(p: &Paint) -> String {
    format!("{}|{}", p.brand, p.name)
}

/// The paint whose key is `key`, or the first one when nothing (valid) is selected.
fn pick(paints: &[Paint], key: Option<&str>) -> Option<Paint> {
    key.and_then(|k| paints.iter().find(|p| paint_key(p) == k))
        .or_else(|| paints.first())
        .cloned()
}

fn copy_to_clipboard(text: &str) {
    use std::io::Write;
    if let Ok(mut child) = std::process::Command::new("pbcopy")
        .stdin(std::process::Stdio::piped())
        .spawn()
    {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(text.as_bytes());
        }
        let _ = child.wait();
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
