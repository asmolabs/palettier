#![forbid(unsafe_code)]

mod actions;
mod snapshot;
mod state;
mod theme;
mod views;

use crate::state::{AppServices, AppState, Section, ThemeMode};
use crate::theme::{APP_CSS, ColorSwatch};
use crate::views::color::{AssistantView, EyedropperView, MixerView};
use crate::views::hud::PainterHudView;
use crate::views::miniature::MiniatureView;
use crate::views::reference::{CatalogView, DryingView, RecipesView};
use crate::views::settings::SettingsView;
use crate::views::workshop::{PalettesView, ProjectsView, TodayView};
use chrono::Utc;
use dioxus::desktop::{Config, WindowBuilder};
use dioxus::prelude::*;
use palettier_domain::drying::Workshop;
use palettier_domain::i18n::Locale;
use palettier_domain::paint::PaintSystem;
use palettier_domain::port::SettingsRepository;
use palettier_domain::workbench::ReadinessWatch;
use std::path::PathBuf;
use std::time::Duration;

fn default_db_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("PALETTIER_DB_DIR") {
        return PathBuf::from(dir);
    }
    // Prefer workspace .palettier if it exists, otherwise home directory ~/.palettier
    let local = PathBuf::from(".").join(".palettier");
    if local.exists() {
        return local;
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".palettier")
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "palettier=info,warn".into()),
        )
        .init();

    let window = WindowBuilder::new()
        .with_title("Palettier — Atelier de figurines")
        .with_inner_size(dioxus::desktop::LogicalSize::new(1460.0, 930.0))
        .with_min_inner_size(dioxus::desktop::LogicalSize::new(1160.0, 740.0));

    dioxus::LaunchBuilder::desktop()
        .with_cfg(Config::new().with_window(window))
        .launch(App);
}

#[component]
fn App() -> Element {
    let mut services_sig = use_signal(|| None::<AppServices>);
    let mut init_error = use_signal(|| None::<String>);

    use_effect(move || {
        spawn(async move {
            let dir = default_db_dir();
            match AppServices::init(dir.clone()).await {
                Ok(svc) => services_sig.set(Some(svc)),
                Err(primary_err) => {
                    let local = PathBuf::from(".").join(".palettier");
                    if local != dir {
                        tracing::warn!(
                            "Primary database at {:?} failed ({primary_err}), falling back to local {:?}",
                            dir,
                            local
                        );
                        if let Ok(svc) = AppServices::init(local).await {
                            services_sig.set(Some(svc));
                            return;
                        }
                    }
                    init_error.set(Some(primary_err.to_string()));
                }
            }
        });
    });

    rsx! {
        style { "{APP_CSS}" }
        if let Some(err) = init_error.read().clone() {
            div { class: "app-shell theme-dark u-ai-center u-jc-center u-p-40",
                div { class: "card u-maxw-560",
                    h2 { class: "card-title u-c-danger", "Studio initialization error / Erreur d'initialisation" }
                    p { "{err}" }
                }
            }
        } else if let Some(services) = services_sig.read().clone() {
            StudioShell { services }
        } else {
            div { class: "app-shell theme-dark u-ai-center u-jc-center",
                div { class: "muted", "Opening Palettier Studio… / Ouverture de l'atelier Palettier…" }
            }
        }
    }
}

#[component]
fn StudioShell(services: AppServices) -> Element {
    use_context_provider(|| services.clone());

    let section = use_signal(|| Section::Today);
    let locale = use_signal(|| Locale::Fr);
    let theme = use_signal(|| ThemeMode::Dark);
    let paints = use_signal(Vec::new);
    let palettes = use_signal(Vec::new);
    let projects = use_signal(Vec::new);
    let recipes = use_signal(Vec::new);
    let mixes = use_signal(Vec::new);
    let workshop = use_signal(Workshop::standard);
    let ai_settings = use_signal(Default::default);
    let toast = use_signal(|| None::<String>);
    let sampled_zones = use_signal(Vec::new);
    let mixer_target_hex = use_signal(|| "#C87A53".to_string());
    let last_sampled_hex = use_signal(|| None::<String>);
    let command_palette_open = use_signal(|| false);
    let miniature_tab = use_signal(|| 0usize);
    let paint_system = use_signal(PaintSystem::default);
    let mut is_hud = use_signal(|| false);

    let mut app_state = use_context_provider(|| AppState {
        section,
        locale,
        theme,
        paints,
        palettes,
        projects,
        recipes,
        mixes,
        workshop,
        ai_settings,
        toast,
        sampled_zones,
        mixer_target_hex,
        last_sampled_hex,
        command_palette_open,
        miniature_tab,
        paint_system,
    });

    // Design review: walk every screen and save its HTML, then quit (dev only).
    use_hook(|| {
        if let Some(dir) = snapshot::target_dir() {
            snapshot::run(app_state, dir);
        }
    });

    // Initial data load + periodic background check for newly ready-to-recoat layers
    use_effect({
        let services = services.clone();
        move || {
            let services = services.clone();
            spawn(async move {
                app_state.reload_all(&services).await;
                let mut watch = ReadinessWatch::new();
                loop {
                    let projs = app_state.projects.read().clone();
                    let bench = services.workbench.bench(&projs, Utc::now());
                    let freed = watch.newly_ready(&bench);
                    if !freed.is_empty() {
                        let loc = *app_state.locale.read();
                        let names: Vec<String> = freed.iter().map(|f| f.label()).collect();
                        let joined = names.join(", ");
                        let msg = match loc {
                            Locale::Fr => format!("Couche sèche et prête à reprendre : {joined}"),
                            Locale::En => format!("Layer dry and ready to recoat: {joined}"),
                        };
                        app_state.notify(msg);
                        send_macos_notification(
                            loc.tr(
                                "Palettier — Zone recouvrable",
                                "Palettier — Ready to recoat",
                            ),
                            &joined,
                        );
                    }
                    tokio::time::sleep(Duration::from_secs(30)).await;
                }
            });
        }
    });

    let current_section = *app_state.section.read();
    let current_locale = *app_state.locale.read();
    let current_theme = *app_state.theme.read();
    let current_system = *app_state.paint_system.read();
    let system_label = current_system.label(current_locale).to_lowercase();
    let owned_count = app_state
        .paints
        .read()
        .iter()
        .filter(|p| p.system == current_system && p.in_stock)
        .count();
    let total_paints = app_state
        .paints
        .read()
        .iter()
        .filter(|p| p.system == current_system)
        .count();
    let ready_pieces_count = {
        let projs = app_state.projects.read().clone();
        services.workbench.bench(&projs, Utc::now()).ready().len()
    };

    rsx! {
        if *is_hud.read() {
            PainterHudView {
                on_close: move |_| is_hud.set(false),
            }
        } else {
            div {
                class: "app-shell {current_theme.css_class()}",
                tabindex: "0",
                onkeydown: move |evt| {
                    if evt.key() == Key::Escape {
                        app_state.command_palette_open.set(false);
                        return;
                    }
                    if (evt.modifiers().meta() || evt.modifiers().ctrl())
                        && let Key::Character(ref ch) = evt.key() {
                            if ch.as_str() == "h" || ch.as_str() == "H" {
                                let cur = *is_hud.read();
                                is_hud.set(!cur);
                                return;
                            }
                            if ch.as_str() == "k" || ch.as_str() == "K" {
                                let cur = *app_state.command_palette_open.read();
                                app_state.command_palette_open.set(!cur);
                                return;
                            }
                            let target = match ch.as_str() {
                                "m" | "M" => Some(Section::Miniature),
                                "1" => Some(Section::Today),
                                "2" => Some(Section::Projects),
                                "3" => Some(Section::Palettes),
                                "4" => Some(Section::Mixer),
                                "5" => Some(Section::Eyedropper),
                                "6" => Some(Section::Assistant),
                                "7" => Some(Section::Catalog),
                                "8" => Some(Section::Drying),
                                "9" => Some(Section::Recipes),
                                "0" => Some(Section::Settings),
                                _ => None,
                            };
                            if let Some(sec) = target {
                                app_state.section.set(sec);
                            }
                        }
                },

            aside { class: "sidebar",
                div { class: "brand-header",
                    div { class: "app-mark" }
                    div {
                        div { class: "brand-title", "Palettier" }
                        div {
                            class: "brand-subtitle",
                            "{current_locale.tr(\"Atelier figurine · huile & acrylique\", \"Miniature studio · oil & acrylic\")}"
                        }
                    }
                }

                // What am I painting with: drives the catalog, mixer, drying and recipes.
                div { class: "system-segment-block",
                    div { class: "system-segment-label", {current_locale.tr("Je peins à", "Painting in")} }
                    div {
                        class: "system-segment",
                        title: "{current_locale.tr(\"Système des écrans hors projet (catalogue, mélangeur, séchage, recettes). Ouvrir un projet le règle sur celui du projet.\", \"System for screens outside a project (catalog, mixer, drying, recipes). Opening a project sets it to the project's system.\")}",
                        for sys in PaintSystem::ALL {
                            button {
                                class: match (sys, *app_state.paint_system.read() == sys) {
                                    (PaintSystem::Oil, true) => "segment active oil",
                                    (PaintSystem::Acrylic, true) => "segment active acrylic",
                                    _ => "segment",
                                },
                                onclick: {
                                    let svc = services.clone();
                                    move |_| app_state.set_paint_system(&svc, sys)
                                },
                                "{sys.label(current_locale)}"
                            }
                        }
                    }
                }

                // Search, theme, language and HUD as compact icon buttons.
                div { class: "quick-toggles",
                    button {
                        class: "icon-btn",
                        title: "{current_locale.tr(\"Recherche globale & commandes (⌘K)\", \"Quick search & commands (⌘K)\")}",
                        onclick: move |_| {
                            let cur = *app_state.command_palette_open.read();
                            app_state.command_palette_open.set(!cur);
                        },
                        "🔍"
                    }
                    button {
                        class: "icon-btn",
                        title: "{current_theme.button_label(current_locale)}",
                        onclick: {
                            let svc = services.clone();
                            move |_| {
                                let next = app_state.theme.read().toggle();
                                app_state.theme.set(next);
                                let svc = svc.clone();
                                spawn(async move {
                                    let _ = svc.settings.put(AppState::KEY_THEME, next.as_str()).await;
                                });
                            }
                        },
                        if current_theme == ThemeMode::Dark { "☀️" } else { "🌙" }
                    }
                    button {
                        class: "icon-btn",
                        title: "{current_locale.tr(\"Passer l'interface en anglais\", \"Switch interface to French\")}",
                        onclick: {
                            let svc = services.clone();
                            move |_| {
                                let next = app_state.locale.read().toggle();
                                app_state.locale.set(next);
                                let svc = svc.clone();
                                spawn(async move {
                                    let _ = svc.settings.put(AppState::KEY_LOCALE, next.code()).await;
                                });
                            }
                        },
                        "{current_locale.toggle().badge()}"
                    }
                    button {
                        class: if *is_hud.read() { "icon-btn active" } else { "icon-btn" },
                        title: "{current_locale.tr(\"Mode HUD compact pour l'établi (⌘H)\", \"Compact painter HUD (⌘H)\")}",
                        onclick: move |_| {
                            let cur = *is_hud.read();
                            is_hud.set(!cur);
                        },
                        "🎯"
                    }
                }

                div { class: "nav-group-label", "{current_locale.tr(\"Établi\", \"Workbench\")}" }
                for sec in Section::WORKSHOP {
                    NavButton {
                        section: sec,
                        current: current_section,
                        badge: if sec == Section::Today && ready_pieces_count > 0 {
                            Some(format!("{ready_pieces_count}"))
                        } else {
                            None
                        }
                    }
                }

                div { class: "nav-group-label", "{current_locale.tr(\"Couleur\", \"Colour\")}" }
                for sec in Section::COLOR {
                    NavButton { section: sec, current: current_section, badge: None }
                }

                div { class: "nav-group-label", "{current_locale.tr(\"Figurine & weathering\", \"Miniatures & weathering\")}" }
                for sec in Section::FIGURINE {
                    NavButton { section: sec, current: current_section, badge: None }
                }

                div { class: "nav-group-label", "{current_locale.tr(\"Référence\", \"Reference\")}" }
                for sec in Section::REFERENCE {
                    NavButton { section: sec, current: current_section, badge: None }
                }

                div { class: "sidebar-footer",
                    NavButton { section: Section::Settings, current: current_section, badge: None }
                    div { class: "u-p-8-10-0-10 u-lh-1p4",
                        div {
                            if current_locale.is_fr() {
                                "Tiroir {system_label} : {owned_count} / {total_paints}"
                            } else {
                                "{current_system.label(current_locale)} drawer: {owned_count} / {total_paints}"
                            }
                        }
                        div { class: "u-op-0p75 u-mt-2",
                            "{current_theme.status_label(current_locale)} · {current_locale.display_name()}"
                        }
                    }
                }
            }

            main { class: "main-pane",
                if let Some(msg) = app_state.toast.read().clone() {
                    div { class: "toast-banner",
                        span { "{msg}" }
                        button {
                            class: "btn",
                            onclick: move |_| app_state.toast.set(None),
                            "{current_locale.tr(\"Fermer\", \"Dismiss\")}"
                        }
                    }
                }

                match current_section {
                    Section::Today => rsx! { TodayView {} },
                    Section::Projects => rsx! { ProjectsView {} },
                    Section::Palettes => rsx! { PalettesView {} },
                    Section::Miniature => rsx! { MiniatureView {} },
                    Section::Eyedropper => rsx! { EyedropperView {} },
                    Section::Assistant => rsx! { AssistantView {} },
                    Section::Mixer => rsx! { MixerView {} },
                    Section::Catalog => rsx! { CatalogView {} },
                    Section::Drying => rsx! { DryingView {} },
                    Section::Recipes => rsx! { RecipesView {} },
                    Section::Settings => rsx! { SettingsView {} },
                }
            }

            if *app_state.command_palette_open.read() {
                CommandPaletteModal {}
            }
        }
    }
    }
}

#[component]
fn NavButton(section: Section, current: Section, badge: Option<String>) -> Element {
    let mut state = use_context::<AppState>();
    let locale = *state.locale.read();
    let cls = if section == current {
        "nav-btn active"
    } else {
        "nav-btn"
    };
    rsx! {
        button {
            class: "{cls}",
            onclick: move |_| state.section.set(section),
            div { class: "nav-btn-left",
                span { class: "u-fs-14 u-mr-4 u-d-inline-block", "{section.icon()}" }
                span { "{section.label(locale)}" }
                if let Some(b) = badge {
                    span { class: "pill pill-ok u-p-0-6 u-fs-10", "{b}" }
                }
            }
            span { class: "nav-shortcut", "{section.shortcut()}" }
        }
    }
}

#[component]
fn CommandPaletteModal() -> Element {
    let mut state = use_context::<AppState>();
    let locale = *state.locale.read();
    let mut search = use_signal(String::new);
    let term = search.read().to_lowercase().trim().to_string();

    let all_paints = state.paints.read().clone();

    // Section and tab matches
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
        (
            "📚",
            locale.tr(
                "Catalogue Tubes d'Huile (680 références)",
                "Oil Paint Catalog (680 tubes)",
            ),
            Section::Catalog,
            0,
        ),
        (
            "⏳",
            locale.tr("Temps de Séchage & Siccatifs", "Drying Times & Mediums"),
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

    let matches = |system: PaintSystem| -> Vec<_> {
        all_paints
            .iter()
            .filter(|p| {
                p.system == system
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
                        span { class: "u-fs-18", "🔍" }
                        strong { class: "u-fs-15", {locale.tr("Palette de Commandes & Recherche Globale", "Command Palette & Quick Search")} }
                    }
                    span { class: "pill", "Échap pour fermer" }
                }

                input {
                    class: "input",
                    class: "u-fs-14p5 u-p-10-14 u-mb-14 u-w-100pct",
                    placeholder: locale.tr("Rechercher un outil, acrylique Citadel/Vallejo, tube d'huile, effet...", "Search tool, Citadel/Vallejo paint, oil tube, effect..."),
                    autofocus: true,
                    value: "{search}",
                    oninput: move |ev| search.set(ev.value()),
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
                                                state.miniature_tab.set(0); // Converter
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

fn send_macos_notification(title: &str, body: &str) {
    let safe_title = title.replace('"', "\\\"");
    let safe_body = body.replace('"', "\\\"");
    let script = format!("display notification \"{safe_body}\" with title \"{safe_title}\"");
    let _ = std::process::Command::new("osascript")
        .arg("-e")
        .arg(script)
        .spawn();
}
