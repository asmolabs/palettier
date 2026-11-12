#![forbid(unsafe_code)]

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
        .with_title("Palettier — Oil Painting Studio")
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
            div { class: "app-shell theme-dark", style: "align-items: center; justify-content: center; padding: 40px;",
                div { class: "card", style: "max-width: 560px;",
                    h2 { class: "card-title", style: "color: var(--danger);", "Studio initialization error / Erreur d'initialisation" }
                    p { "{err}" }
                }
            }
        } else if let Some(services) = services_sig.read().clone() {
            StudioShell { services }
        } else {
            div { class: "app-shell theme-dark", style: "align-items: center; justify-content: center;",
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
    let owned_count = app_state
        .paints
        .read()
        .iter()
        .filter(|p| p.in_stock)
        .count();
    let total_paints = app_state.paints.read().len();
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
                            "{current_locale.tr(\"Atelier figurine & huile · KM\", \"Miniature & Oil Studio · KM\")}"
                        }
                    }
                }

                // Instant Day / Night, Language & Command Palette toggles
                div { class: "quick-toggles",
                    button {
                        class: "toggle-chip",
                        title: "{current_locale.tr(\"Recherche Globale & Commandes (⌘K)\", \"Quick Search & Commands (⌘K)\")}",
                        onclick: move |_| {
                            let cur = *app_state.command_palette_open.read();
                            app_state.command_palette_open.set(!cur);
                        },
                        "🔍 ⌘K"
                    }
                    div {
                        class: "system-switch",
                        title: "{current_locale.tr(\"Système de peinture des écrans hors projet (catalogue, mélangeur, séchage, recettes). Chaque projet garde le sien.\", \"Paint system for screens outside a project (catalog, mixer, drying, recipes). Each project keeps its own.\")}",
                        for sys in PaintSystem::ALL {
                            button {
                                class: if *app_state.paint_system.read() == sys { "toggle-chip active" } else { "toggle-chip" },
                                onclick: {
                                    let svc = services.clone();
                                    move |_| app_state.set_paint_system(&svc, sys)
                                },
                                "{sys.label_i18n(current_locale)}"
                            }
                        }
                    }
                    button {
                        class: "toggle-chip",
                        title: "{current_locale.tr(\"Basculer entre le Mode Jour et le Mode Nuit\", \"Switch between Day Mode and Night Mode\")}",
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
                        "{current_theme.button_label(current_locale)}"
                    }
                    button {
                        class: "toggle-chip",
                        title: "{current_locale.tr(\"Passer l'interface en anglais (EN)\", \"Switch interface to French (FR)\")}",
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
                        "🌐 {current_locale.badge()} → {current_locale.toggle().badge()}"
                    }
                    button {
                        class: if *is_hud.read() { "toggle-chip active" } else { "toggle-chip" },
                        title: "{current_locale.tr(\"Basculer en mode HUD Établi compact (⌘H)\", \"Toggle compact Painter HUD mode (⌘H)\")}",
                        onclick: move |_| {
                            let cur = *is_hud.read();
                            is_hud.set(!cur);
                        },
                        "🎯 HUD"
                    }
                }

                div { class: "nav-group-label", "{current_locale.tr(\"Figurine & Modélisme\", \"Miniatures & Weathering\")}" }
                for sec in Section::FIGURINE {
                    NavButton { section: sec, current: current_section, badge: None }
                }

                div { class: "nav-group-label", "{current_locale.tr(\"Établi & Projets\", \"Workbench & Projects\")}" }
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

                div { class: "nav-group-label", "{current_locale.tr(\"Laboratoire Couleur (KM)\", \"Color Lab (KM)\")}" }
                for sec in Section::COLOR {
                    NavButton { section: sec, current: current_section, badge: None }
                }

                div { class: "nav-group-label", "{current_locale.tr(\"Matériel & Chimie\", \"Materials & Chemistry\")}" }
                for sec in Section::REFERENCE {
                    NavButton { section: sec, current: current_section, badge: None }
                }

                div { class: "sidebar-footer",
                    NavButton { section: Section::Settings, current: current_section, badge: None }
                    div { style: "padding: 8px 10px 0 10px; line-height: 1.4;",
                        div {
                            if current_locale.is_fr() {
                                "Tiroir : {owned_count} / {total_paints} tubes"
                            } else {
                                "Drawer: {owned_count} / {total_paints} tubes"
                            }
                        }
                        div { style: "opacity: 0.75; margin-top: 2px;",
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
                span { style: "font-size: 14px; margin-right: 4px; display: inline-block;", "{section.icon()}" }
                span { "{section.label_i18n(locale)}" }
                if let Some(b) = badge {
                    span { class: "pill pill-ok", style: "padding: 0 6px; font-size: 10px;", "{b}" }
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

    let matching_acrylics: Vec<_> = palettier_domain::miniature::MINIATURE_ACRYLICS
        .iter()
        .filter(|a| {
            !term.is_empty()
                && (a.name.to_lowercase().contains(&term)
                    || a.category.to_lowercase().contains(&term))
        })
        .take(6)
        .collect();

    let matching_oils: Vec<_> = all_paints
        .iter()
        .filter(|p| {
            !term.is_empty()
                && (p.name.to_lowercase().contains(&term) || p.brand.to_lowercase().contains(&term))
        })
        .take(6)
        .collect();

    rsx! {
        div {
            class: "modal-backdrop",
            style: "background: rgba(0, 0, 0, 0.65); position: fixed; inset: 0; z-index: 9999; display: flex; align-items: flex-start; justify-content: center; padding-top: 8vh;",
            onclick: move |_| state.command_palette_open.set(false),

            div {
                class: "card",
                style: "width: 640px; max-width: 92vw; max-height: 82vh; overflow-y: auto; box-shadow: 0 20px 40px rgba(0,0,0,0.4); border: 1.5px solid var(--accent);",
                onclick: move |ev| ev.stop_propagation(),

                div { class: "row-between", style: "align-items: center; margin-bottom: 12px; border-bottom: 1px solid var(--border-main); padding-bottom: 8px;",
                    div { class: "row", style: "gap: 8px; align-items: center;",
                        span { style: "font-size: 18px;", "🔍" }
                        strong { style: "font-size: 15px;", {locale.tr("Palette de Commandes & Recherche Globale", "Command Palette & Quick Search")} }
                    }
                    span { class: "pill", "Échap pour fermer" }
                }

                input {
                    class: "input",
                    style: "font-size: 14.5px; padding: 10px 14px; margin-bottom: 14px; width: 100%;",
                    placeholder: locale.tr("Rechercher un outil, acrylique Citadel/Vallejo, tube d'huile, effet...", "Search tool, Citadel/Vallejo paint, oil tube, effect..."),
                    autofocus: true,
                    value: "{search}",
                    oninput: move |ev| search.set(ev.value()),
                }

                if !matching_tools.is_empty() {
                    div { style: "font-weight: 700; font-size: 11px; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-dim); margin-bottom: 6px;",
                        {locale.tr("Outils & Sections", "Tools & Sections")}
                    }
                    div { style: "display: flex; flex-direction: column; gap: 4px; margin-bottom: 14px;",
                        for (icon, title, sec, tab) in matching_tools {
                            button {
                                class: "card-subtle",
                                style: "text-align: left; padding: 8px 12px; cursor: pointer; display: flex; justify-content: space-between; align-items: center; border: none; width: 100%;",
                                onclick: move |_| {
                                    state.section.set(sec);
                                    if sec == Section::Miniature {
                                        state.miniature_tab.set(tab);
                                    }
                                    state.command_palette_open.set(false);
                                },
                                div { class: "row", style: "gap: 8px; align-items: center;",
                                    span { style: "font-size: 15px;", "{icon}" }
                                    span { style: "font-weight: 600; font-size: 13px;", "{title}" }
                                }
                                span { class: "nav-shortcut", "{sec.shortcut()}" }
                            }
                        }
                    }
                }

                if !matching_acrylics.is_empty() {
                    div { style: "font-weight: 700; font-size: 11px; text-transform: uppercase; letter-spacing: 0.5px; color: var(--accent); margin-bottom: 6px;",
                        {locale.tr("Acryliques Figurines (Citadel, Vallejo, AK)", "Miniature Acrylics (Citadel, Vallejo, AK)")}
                    }
                    div { style: "display: flex; flex-direction: column; gap: 4px; margin-bottom: 14px;",
                        for acr in matching_acrylics {
                            {
                                let hex = acr.hex.to_string();
                                rsx! {
                                    button {
                                        class: "card-subtle",
                                        style: "text-align: left; padding: 8px 12px; cursor: pointer; display: flex; justify-content: space-between; align-items: center; border: none; width: 100%;",
                                        onclick: {
                                            let h = hex.clone();
                                            move |_| {
                                                state.mixer_target_hex.set(h.clone());
                                                state.section.set(Section::Miniature);
                                                state.miniature_tab.set(0); // Converter
                                                state.command_palette_open.set(false);
                                            }
                                        },
                                        div { class: "row", style: "gap: 10px; align-items: center;",
                                            ColorSwatch { hex: hex.clone(), size: 22 }
                                            div {
                                                strong { style: "font-size: 13px;", "{acr.name}" }
                                                span { class: "muted", style: "font-size: 11px; margin-left: 8px;", "{acr.brand.display_name()}" }
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
                    div { style: "font-weight: 700; font-size: 11px; text-transform: uppercase; letter-spacing: 0.5px; color: var(--text-dim); margin-bottom: 6px;",
                        {locale.tr("Tubes d'Huile du Catalogue", "Catalog Oil Paint Tubes")}
                    }
                    div { style: "display: flex; flex-direction: column; gap: 4px;",
                        for oil in matching_oils {
                            {
                                let hex = oil.color().to_hex();
                                rsx! {
                                    button {
                                        class: "card-subtle",
                                        style: "text-align: left; padding: 8px 12px; cursor: pointer; display: flex; justify-content: space-between; align-items: center; border: none; width: 100%;",
                                        onclick: {
                                            let h = hex.clone();
                                            move |_| {
                                                state.mixer_target_hex.set(h.clone());
                                                state.section.set(Section::Mixer);
                                                state.command_palette_open.set(false);
                                            }
                                        },
                                        div { class: "row", style: "gap: 10px; align-items: center;",
                                            ColorSwatch { hex: hex.clone(), size: 22 }
                                            div {
                                                strong { style: "font-size: 13px;", "{oil.name}" }
                                                span { class: "muted", style: "font-size: 11px; margin-left: 8px;", "{oil.brand}" }
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
