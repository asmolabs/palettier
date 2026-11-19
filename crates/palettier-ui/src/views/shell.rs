use super::color::{AssistantView, EyedropperView, MixerView};
use super::command_palette::CommandPaletteModal;
use super::hud::PainterHudView;
use super::miniature::MiniatureView;
use super::reference::{CatalogView, DryingView, RecipesView};
use super::settings::SettingsView;
use super::workshop::{PalettesView, ProjectsView, TodayView};
use crate::actions::settings::{save_locale, save_sidebar_collapsed, save_theme};
use crate::snapshot;
use crate::icons::{
    IconAlert, IconCheck, IconChevronLeft, IconChevronRight, IconContrast, IconGlobe, IconMoon,
    IconSearch, IconSun, IconTarget, SectionIcon,
};
use crate::state::{AppServices, AppState, Section, ThemeMode, ToastNotice};
use chrono::Utc;
use dioxus::prelude::*;
use palettier_domain::drying::Workshop;
use palettier_domain::i18n::Locale;
use palettier_domain::paint::PaintSystem;
use palettier_domain::workbench::ReadinessWatch;
use std::time::Duration;

#[component]
pub fn StudioShell(services: AppServices) -> Element {
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
    let toast = use_signal(|| None::<ToastNotice>);
    let sampled_zones = use_signal(Vec::new);
    let mixer_target_hex = use_signal(|| "#C87A53".to_string());
    let last_sampled_hex = use_signal(|| None::<String>);
    let command_palette_open = use_signal(|| false);
    let miniature_tab = use_signal(|| 0usize);
    let paint_system = use_signal(PaintSystem::default);
    let loaded = use_signal(|| false);
    let sidebar_collapsed = use_signal(|| false);
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
        loaded,
        sidebar_collapsed,
    });

    // Design review: walk every screen and save its HTML, then quit (dev only).
    use_hook(|| {
        if let Some(dir) = snapshot::target_dir() {
            snapshot::run(app_state, dir);
        }
    });

    // Initial data load + periodic background check for newly ready-to-recoat layers.
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

            aside {
                class: if *app_state.sidebar_collapsed.read() {
                    "sidebar collapsed"
                } else {
                    "sidebar"
                },
                div { class: "brand-header",
                    div { class: "app-mark" }
                    div { class: "brand-text",
                        div { class: "brand-title", "Palettier" }
                        div {
                            class: "brand-subtitle",
                            "{current_locale.tr(\"Atelier figurine · huile & acrylique\", \"Miniature studio · oil & acrylic\")}"
                        }
                    }
                    button {
                        class: "sidebar-collapse-btn",
                        title: if *app_state.sidebar_collapsed.read() {
                            current_locale.tr("Déplier la barre latérale", "Expand sidebar")
                        } else {
                            current_locale.tr("Replier la barre latérale", "Collapse sidebar")
                        },
                        onclick: {
                            let svc = services.clone();
                            move |_| {
                                let next = !*app_state.sidebar_collapsed.read();
                                app_state.sidebar_collapsed.set(next);
                                let svc = svc.clone();
                                spawn(async move {
                                    let _ = save_sidebar_collapsed(&svc, next).await;
                                });
                            }
                        },
                        if *app_state.sidebar_collapsed.read() {
                            IconChevronRight {}
                        } else {
                            IconChevronLeft {}
                        }
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
                    div { class: "sidebar-stats u-p-8-10-0-10 u-lh-1p4",
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

            div { class: "main-shell",
                header { class: "app-header",
                    div { class: "header-left",
                        button {
                            class: "header-search-btn",
                            title: "{current_locale.tr(\"Recherche globale & commandes (⌘K)\", \"Quick search & commands (⌘K)\")}",
                            onclick: move |_| {
                                let cur = *app_state.command_palette_open.read();
                                app_state.command_palette_open.set(!cur);
                            },
                            IconSearch {}
                            span { class: "header-search-label",
                                {current_locale.tr("Rechercher…", "Search…")}
                            }
                            span { class: "kbd-chip", "⌘K" }
                        }
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

                    div { class: "quick-toggles",
                        button {
                            class: "icon-btn",
                            title: "{current_theme.next_button_label(current_locale)}",
                            onclick: {
                                let svc = services.clone();
                                move |_| {
                                    let next = app_state.theme.read().toggle();
                                    app_state.theme.set(next);
                                    let svc = svc.clone();
                                    spawn(async move {
                                        let _ = save_theme(&svc, next).await;
                                    });
                                }
                            },
                            match current_theme.toggle() {
                                ThemeMode::Dark => rsx! { IconMoon {} },
                                ThemeMode::Light => rsx! { IconSun {} },
                                ThemeMode::Neutral => rsx! { IconContrast {} },
                            }
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
                                        let _ = save_locale(&svc, next).await;
                                    });
                                }
                            },
                            IconGlobe {}
                            span { "{current_locale.toggle().badge()}" }
                        }
                        button {
                            class: if *is_hud.read() { "icon-btn active" } else { "icon-btn" },
                            title: "{current_locale.tr(\"Mode HUD compact pour l'établi (⌘H)\", \"Compact painter HUD (⌘H)\")}",
                            onclick: move |_| {
                                let cur = *is_hud.read();
                                is_hud.set(!cur);
                            },
                            IconTarget {}
                        }
                    }
                }

                main { class: "main-pane",
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
            }

            if let Some(notice) = app_state.toast.read().clone() {
                div {
                    class: if notice.is_error { "toast-banner toast-err" } else { "toast-banner" },
                    div { class: "toast-banner-msg",
                        if notice.is_error {
                            span { class: "toast-icon-err", IconAlert {} }
                        } else {
                            span { class: "toast-icon-ok", IconCheck {} }
                        }
                        span { "{notice.message}" }
                    }
                    button {
                        class: "btn",
                        onclick: move |_| app_state.toast.set(None),
                        "{current_locale.tr(\"Fermer\", \"Dismiss\")}"
                    }
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
            title: "{section.label(locale)} ({section.shortcut()})",
            onclick: move |_| state.section.set(section),
            div { class: "nav-btn-left",
                span { class: "nav-icon", SectionIcon { section } }
                span { class: "nav-label", "{section.label(locale)}" }
                if let Some(b) = badge {
                    span { class: "nav-label pill pill-ok u-p-0-6 u-fs-10", "{b}" }
                }
            }
            span { class: "nav-shortcut", "{section.shortcut()}" }
        }
    }
}

#[cfg(target_os = "macos")]
fn send_macos_notification(title: &str, body: &str) {
    // The view tests run the real shell: they must not pop up system notifications.
    if cfg!(test) {
        return;
    }
    // Pass notification title and body via argv so arbitrary text can never break AppleScript quoting.
    if let Err(e) = std::process::Command::new("osascript")
        .args([
            "-e",
            "on run argv",
            "-e",
            "display notification (item 1 of argv) with title (item 2 of argv)",
            "-e",
            "end run",
            "--",
            body,
            title,
        ])
        .spawn()
    {
        tracing::warn!("Failed to spawn osascript notification: {e}");
    }
}

#[cfg(not(target_os = "macos"))]
fn send_macos_notification(_title: &str, _body: &str) {}
