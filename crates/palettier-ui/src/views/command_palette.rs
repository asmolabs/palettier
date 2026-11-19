use crate::icons::{IconClose, IconSearch};
use crate::state::{AppState, Section};
use crate::theme::ColorSwatch;
use dioxus::prelude::*;
use palettier_domain::i18n::Locale;
use palettier_domain::paint::PaintSystem;

#[component]
pub fn CommandPaletteModal() -> Element {
    let mut state = use_context::<AppState>();
    let locale = *state.locale.read();
    let mut search = use_signal(String::new);
    let term = search.read().to_lowercase().trim().to_string();

    let all_paints = state.paints.read().clone();
    let system = *state.paint_system.read();
    let system_count = all_paints.iter().filter(|p| p.system == system).count();
    let catalog_title = match (system, locale) {
        (PaintSystem::Oil, Locale::Fr) => {
            format!("Catalogue Tubes d'Huile ({system_count} références)")
        }
        (PaintSystem::Oil, Locale::En) => format!("Oil Paint Catalog ({system_count} tubes)"),
        (PaintSystem::Acrylic, Locale::Fr) => {
            format!("Catalogue Acryliques ({system_count} références)")
        }
        (PaintSystem::Acrylic, Locale::En) => {
            format!("Acrylic Paint Catalog ({system_count} paints)")
        }
    };

    let tools = [
        (
            "⚔️",
            locale.tr(
                "Figurine : Convertisseur & Échelle 5 Étapes",
                "Miniature: Converter & 5-Stage Ladder",
            ),
            Section::Miniature,
            0,
        ),
        (
            "⚔️",
            locale.tr(
                "Figurine : NMM Métal Non-Métallique (Or, Acier, Cuivre)",
                "Miniature: NMM Non-Metallic Metal Studio",
            ),
            Section::Miniature,
            1,
        ),
        (
            "🌧️",
            locale.tr(
                "Figurine : Weathering & Grimdark (Streaking Grime, Rouille)",
                "Miniature: Weathering & Grimdark",
            ),
            Section::Miniature,
            2,
        ),
        (
            "🛡️",
            locale.tr(
                "Figurine : Pin Wash & Travail Réductif au Solvant",
                "Miniature: Pin Wash & Reductive Technique",
            ),
            Section::Miniature,
            3,
        ),
        (
            "🧪",
            locale.tr(
                "Figurine : Solvants, Médiums, Décapage & Palette Congélateur",
                "Miniature: Solvents, Mediums, Stripping & Freezer",
            ),
            Section::Miniature,
            4,
        ),
        (
            "🏜️",
            locale.tr(
                "Figurine : Pigments Secs de Weathering & Fixateurs",
                "Miniature: Weathering Dry Pigments & Fixers",
            ),
            Section::Miniature,
            5,
        ),
        (
            "📄",
            locale.tr(
                "Figurine : Fiche de Recette d'Armée & Modélisme",
                "Miniature: Army Recipe Sheet Generator",
            ),
            Section::Miniature,
            6,
        ),
        (
            "♨️",
            locale.tr(
                "Figurine : Séchage Forcé (Boîte Chauffante / Déshydrateur)",
                "Miniature: Heat Box Curing Simulator",
            ),
            Section::Miniature,
            7,
        ),
        (
            "🩸",
            locale.tr(
                "Figurine : Sang & FX Spéciaux (Gore, Glace, Gemmes)",
                "Miniature: Blood & Special FX",
            ),
            Section::Miniature,
            8,
        ),
        (
            "🖌️",
            locale.tr(
                "Figurine : Soin Pinceaux de Martre Kolinsky",
                "Miniature: Kolinsky Brush Care Tracker",
            ),
            Section::Miniature,
            9,
        ),
        (
            "🪞",
            locale.tr(
                "Figurine : Harmonisation par Vernis (Ultra-Mat, Satin, Gloss)",
                "Miniature: Varnish Harmonizer",
            ),
            Section::Miniature,
            10,
        ),
        (
            "👥",
            locale.tr(
                "Figurine : Peinture d'Armée en Série (Batch Squad)",
                "Miniature: Batch Squad Painting",
            ),
            Section::Miniature,
            11,
        ),
        (
            "💡",
            locale.tr(
                "Figurine : Simulateur OSL (Lueur & Énergie Plasma)",
                "Miniature: OSL Light Engine",
            ),
            Section::Miniature,
            12,
        ),
        (
            "🧪",
            locale.tr(
                "Mélangeur Kubelka-Munk (Calcul de ratios)",
                "Kubelka-Munk Color Mixer",
            ),
            Section::Mixer,
            0,
        ),
        (
            "⏱️",
            locale.tr(
                "Aujourd'hui : Tableau de bord & Séchage",
                "Today: Workbench & Drying",
            ),
            Section::Today,
            0,
        ),
        (
            "📋",
            locale.tr(
                "Projets & Figurines en cours",
                "Projects & Miniature Pieces",
            ),
            Section::Projects,
            0,
        ),
        (
            "🎨",
            locale.tr("Mes Palettes d'Atelier", "Saved Palettes"),
            Section::Palettes,
            0,
        ),
        (
            "💧",
            locale.tr("Pipette & Analyse Photo", "Eyedropper & Photo Analyzer"),
            Section::Eyedropper,
            0,
        ),
        (
            "🤖",
            locale.tr("Assistant IA (Conseils & Formulations)", "AI Assistant"),
            Section::Assistant,
            0,
        ),
        ("📚", catalog_title.as_str(), Section::Catalog, 0),
        (
            "⏳",
            match system {
                PaintSystem::Oil => {
                    locale.tr("Temps de Séchage & Siccatifs", "Drying Times & Mediums")
                }
                PaintSystem::Acrylic => {
                    locale.tr("Temps de Séchage acrylique", "Acrylic Drying Times")
                }
            },
            Section::Drying,
            0,
        ),
        (
            "📖",
            locale.tr("Recettes Mères & Guides", "Mother Mixes & Guides"),
            Section::Recipes,
            0,
        ),
        (
            "⚙️",
            locale.tr("Paramètres de l'Atelier", "Workshop Settings"),
            Section::Settings,
            0,
        ),
    ];

    let matching_tools: Vec<_> = tools
        .into_iter()
        .filter(|(_, title, _, _)| term.is_empty() || title.to_lowercase().contains(&term))
        .take(6)
        .collect();

    let matches = |sys: PaintSystem| -> Vec<_> {
        all_paints
            .iter()
            .filter(|p| {
                p.system == sys
                    && !term.is_empty()
                    && (p.name.to_lowercase().contains(&term)
                        || p.brand.to_lowercase().contains(&term))
            })
            .take(6)
            .cloned()
            .collect()
    };
    let matching_acrylics = matches(PaintSystem::Acrylic);
    let matching_oils = matches(PaintSystem::Oil);

    rsx! {
        div {
            class: "modal-backdrop",
            class: "u-bg-rgba-0-0-0-0p65 u-pos-fixed u-inset-0 u-z-9999 u-d-flex u-ai-flex-start u-jc-center u-pt-8vh",
            onclick: move |_| state.command_palette_open.set(false),

            div {
                class: "card",
                class: "u-w-640 u-maxw-92vw u-maxh-82vh u-oy-auto u-sh-0-20-40-rgba-0-0-0-0p4 u-b-1p5-solid-accent",
                onclick: move |ev| ev.stop_propagation(),

                div { class: "row-between u-ai-center u-mb-12 u-bb-1-solid-border-main u-pb-8",
                    div { class: "row u-gap-8 u-ai-center",
                        span { class: "nav-icon", IconSearch {} }
                        strong { class: "u-fs-15", {locale.tr("Palette de Commandes & Recherche Globale", "Command Palette & Quick Search")} }
                    }
                    div { class: "row u-gap-8 u-ai-center",
                        span { class: "pill", {locale.tr("Échap pour fermer", "Esc to close")} }
                        button {
                            class: "icon-btn",
                            title: locale.tr("Fermer (Échap)", "Close (Esc)"),
                            onclick: move |_| state.command_palette_open.set(false),
                            IconClose {}
                        }
                    }
                }

                div { class: "cmd-search-wrap u-mb-14",
                    input {
                        class: "input u-fs-14p5 u-p-10-14 u-w-100pct",
                        placeholder: locale.tr("Rechercher un outil, acrylique Citadel/Vallejo, tube d'huile, effet...", "Search tool, Citadel/Vallejo paint, oil tube, effect..."),
                        autofocus: true,
                        value: "{search}",
                        oninput: move |ev| search.set(ev.value()),
                    }
                    if !search.read().is_empty() {
                        button {
                            class: "cmd-search-clear",
                            title: locale.tr("Effacer", "Clear"),
                            onclick: move |_| search.set(String::new()),
                            IconClose {}
                        }
                    }
                }

                if !matching_tools.is_empty() {
                    div { class: "u-fw-700 u-fs-11 u-tt-uppercase u-ls-0p5 u-c-text-dim u-mb-6",
                        {locale.tr("Outils & Sections", "Tools & Sections")}
                    }
                    div { class: "u-d-flex u-fd-column u-gap-4 u-mb-14",
                        for (icon, title, sec, tab) in matching_tools {
                            button {
                                class: "card-subtle",
                                class: "u-ta-left u-p-8-12 u-cur-pointer u-d-flex u-jc-space-between u-ai-center u-b-none u-w-100pct",
                                onclick: move |_| {
                                    state.section.set(sec);
                                    if sec == Section::Miniature {
                                        state.miniature_tab.set(tab);
                                    }
                                    state.command_palette_open.set(false);
                                },
                                div { class: "row u-gap-8 u-ai-center",
                                    span { class: "u-fs-15", "{icon}" }
                                    span { class: "u-fw-600 u-fs-13", "{title}" }
                                }
                                span { class: "nav-shortcut", "{sec.shortcut()}" }
                            }
                        }
                    }
                }

                if !matching_acrylics.is_empty() {
                    div { class: "u-fw-700 u-fs-11 u-tt-uppercase u-ls-0p5 u-c-accent u-mb-6",
                        {locale.tr("Acryliques Figurines (Citadel, Vallejo, AK)", "Miniature Acrylics (Citadel, Vallejo, AK)")}
                    }
                    div { class: "u-d-flex u-fd-column u-gap-4 u-mb-14",
                        for acr in matching_acrylics {
                            {
                                let hex = acr.hex_color.clone();
                                rsx! {
                                    button {
                                        class: "card-subtle",
                                        class: "u-ta-left u-p-8-12 u-cur-pointer u-d-flex u-jc-space-between u-ai-center u-b-none u-w-100pct",
                                        onclick: {
                                            let h = hex.clone();
                                            move |_| {
                                                state.mixer_target_hex.set(h.clone());
                                                state.section.set(Section::Miniature);
                                                state.miniature_tab.set(0);
                                                state.command_palette_open.set(false);
                                            }
                                        },
                                        div { class: "row u-gap-10 u-ai-center",
                                            ColorSwatch { hex: hex.clone(), size: 22 }
                                            div {
                                                strong { class: "u-fs-13", "{acr.name}" }
                                                span { class: "muted u-fs-11 u-ml-8", "{acr.brand}" }
                                            }
                                        }
                                        span { class: "pill pill-accent", "{hex}" }
                                    }
                                }
                            }
                        }
                    }
                }

                if !matching_oils.is_empty() {
                    div { class: "u-fw-700 u-fs-11 u-tt-uppercase u-ls-0p5 u-c-text-dim u-mb-6",
                        {locale.tr("Tubes d'Huile du Catalogue", "Catalog Oil Paint Tubes")}
                    }
                    div { class: "u-d-flex u-fd-column u-gap-4",
                        for oil in matching_oils {
                            {
                                let hex = oil.color().to_hex();
                                rsx! {
                                    button {
                                        class: "card-subtle",
                                        class: "u-ta-left u-p-8-12 u-cur-pointer u-d-flex u-jc-space-between u-ai-center u-b-none u-w-100pct",
                                        onclick: {
                                            let h = hex.clone();
                                            move |_| {
                                                state.mixer_target_hex.set(h.clone());
                                                state.section.set(Section::Mixer);
                                                state.command_palette_open.set(false);
                                            }
                                        },
                                        div { class: "row u-gap-10 u-ai-center",
                                            ColorSwatch { hex: hex.clone(), size: 22 }
                                            div {
                                                strong { class: "u-fs-13", "{oil.name}" }
                                                span { class: "muted u-fs-11 u-ml-8", "{oil.brand}" }
                                            }
                                        }
                                        span { class: "pill", "{hex}" }
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
