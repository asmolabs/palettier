use crate::state::{AppServices, AppState, ThemeMode};
use crate::theme::SectionHeader;
use dioxus::prelude::*;
use palettier_ai::{AiProvider, AiSettings, ModelOption};
use palettier_data::{BackupWriter, ZipBackupArchive, ZipBackupWriter};
use palettier_domain::i18n::Locale;
use palettier_domain::port::SettingsRepository;
use std::str::FromStr;

#[component]
pub fn SettingsView() -> Element {
    let services = use_context::<AppServices>();
    let mut state = use_context::<AppState>();

    let locale = *state.locale.read();
    let theme = *state.theme.read();
    let current_ai = state.ai_settings.read().clone();

    let mut provider = use_signal(|| current_ai.provider);
    let mut base_url = use_signal(|| current_ai.base_url.clone());
    let mut model = use_signal(|| current_ai.model.clone());
    let mut api_key = use_signal(|| current_ai.api_key_str().to_string());
    let mut available_models = use_signal(Vec::<ModelOption>::new);
    let mut loading_models = use_signal(|| false);

    rsx! {
        div {
            SectionHeader {
                title: locale.tr("Paramètres & Sauvegardes", "Settings & Backups").to_string(),
                subtitle: locale.tr(
                    "Apparence de l'atelier (Jour / Nuit), langue de l'interface (FR / EN), moteur d'IA (trousseau OS sécurisé) et archivage ZIP.",
                    "Studio appearance (Day / Night), interface language (FR / EN), AI engine (secure OS Keychain), and ZIP backups."
                ).to_string(),
                actions: None,
            }

            // Appearance & Language card
            div { class: "card",
                h3 { class: "card-title", {locale.tr("Apparence & Langue de l'atelier", "Studio Appearance & Language")} }
                div { class: "grid-2",
                    div { class: "card-subtle row-between",
                        div {
                            div { style: "font-weight: 600;", {locale.tr("Thème d'éclairage (Mode Jour / Nuit)", "Lighting Theme (Day / Night Mode)")} }
                            div { class: "muted", style: "font-size: 12px;", {locale.tr("Basculez entre le thème sombre d'atelier et le thème clair lumière du jour.", "Switch between the dark studio theme and the daylight high-contrast theme.")} }
                        }
                        div { class: "row",
                            button {
                                class: if theme == ThemeMode::Dark { "btn-primary" } else { "btn" },
                                onclick: {
                                    let svc = services.clone();
                                    move |_| {
                                        state.theme.set(ThemeMode::Dark);
                                        let svc = svc.clone();
                                        spawn(async move {
                                            let _ = svc.settings.put(AppState::KEY_THEME, ThemeMode::Dark.as_str()).await;
                                        });
                                    }
                                }, {locale.tr("🌙 Mode Nuit", "🌙 Night Mode")} }
                            button {
                                class: if theme == ThemeMode::Light { "btn-primary" } else { "btn" },
                                onclick: {
                                    let svc = services.clone();
                                    move |_| {
                                        state.theme.set(ThemeMode::Light);
                                        let svc = svc.clone();
                                        spawn(async move {
                                            let _ = svc.settings.put(AppState::KEY_THEME, ThemeMode::Light.as_str()).await;
                                        });
                                    }
                                }, {locale.tr("☀️ Mode Jour", "☀️ Day Mode")} }
                        }
                    }

                    div { class: "card-subtle row-between",
                        div {
                            div { style: "font-weight: 600;", {locale.tr("Langue de l'application (i18n)", "Application Language (i18n)")} }
                            div { class: "muted", style: "font-size: 12px;", {locale.tr("Traduit instantanément toute l'interface, les guides de mélanges et les prompts IA.", "Instantly translates the entire UI, mixing guides, and AI prompts.")} }
                        }
                        div { class: "row",
                            for l in Locale::ALL {
                                button {
                                    class: if locale == l { "btn-primary" } else { "btn" },
                                    onclick: {
                                        let svc = services.clone();
                                        move |_| {
                                            state.locale.set(l);
                                            let svc = svc.clone();
                                            spawn(async move {
                                                let _ = svc.settings.put(AppState::KEY_LOCALE, l.code()).await;
                                            });
                                        }
                                    },
                                    "{l.display_name()} ({l.badge()})"
                                }
                            }
                        }
                    }
                }
            }

            div { class: "grid-2",
                div { class: "card",
                    h3 { class: "card-title", {locale.tr("Moteur d'Intelligence Artificielle", "Artificial Intelligence Engine")} }
                    p { class: "muted", style: "margin-bottom: 14px; font-size: 12.5px;",
                        {locale.tr("Utilisé par l'Assistant de plan de peinture et la reconnaissance de tubes sur photo. Les clés API sont protégées par le trousseau système (macOS Keychain / Secret Service).", "Used by the Painting Plan Assistant and photo tube recognition. API keys are stored securely in the OS keyring (macOS Keychain / Secret Service).")}
                    }

                    div { style: "margin-bottom: 12px;",
                        label { class: "field-label", {locale.tr("Fournisseur IA", "AI Provider")} }
                        select {
                            style: "width: 100%;",
                            onchange: {
                                let services = services.clone();
                                move |e: Event<FormData>| {
                                    if let Ok(p) = AiProvider::from_str(&e.value()) {
                                        provider.set(p);
                                        let services = services.clone();
                                        spawn(async move {
                                            if let Ok(loaded) = services.ai_store.load_for(p).await {
                                                base_url.set(loaded.base_url.clone());
                                                model.set(loaded.model.clone());
                                                api_key.set(loaded.api_key_str().to_string());
                                            } else {
                                                base_url.set(p.default_url().to_string());
                                                model.set(p.default_model().to_string());
                                                api_key.set(String::new());
                                            }
                                            available_models.set(Vec::new());
                                        });
                                    }
                                }
                            },
                            for p in AiProvider::ALL {
                                option {
                                    value: "{p.as_str()}",
                                    selected: *provider.read() == p,
                                    "{p.label()}"
                                }
                            }
                        }
                    }

                    div { style: "margin-bottom: 12px;",
                        label { class: "field-label", {locale.tr("URL du serveur (http / https)", "Server URL (http / https)")} }
                        input {
                            r#type: "text",
                            style: "width: 100%;",
                            value: "{base_url}",
                            oninput: move |e| base_url.set(e.value()),
                        }
                    }

                    if provider.read().needs_api_key() {
                        div { style: "margin-bottom: 12px;",
                            label { class: "field-label", {locale.tr("Clé API (stockée dans le trousseau OS)", "API Key (stored in OS Keychain)")} }
                            input {
                                r#type: "password",
                                style: "width: 100%;",
                                placeholder: "sk-... / AIza...",
                                value: "{api_key}",
                                oninput: move |e| api_key.set(e.value()),
                            }
                        }
                    }

                    div { style: "margin-bottom: 14px;",
                        label { class: "field-label", {locale.tr("Modèle multimodal", "Multimodal Model")} }
                        div { class: "row",
                            input {
                                r#type: "text",
                                style: "flex: 1;",
                                value: "{model}",
                                oninput: move |e| model.set(e.value()),
                            }
                            button {
                                class: "btn",
                                disabled: *loading_models.read(),
                                onclick: {
                                    let services = services.clone();
                                    move |_| {
                                        let temp = AiSettings::new(
                                            *provider.read(),
                                            base_url.read().clone(),
                                            model.read().clone(),
                                            api_key.read().clone(),
                                        );
                                        let services = services.clone();
                                        spawn(async move {
                                            loading_models.set(true);
                                            match services.model_catalog.available_for(&temp).await {
                                                Ok(list) => {
                                                    if let Some(first) = list.first()
                                                        && model.read().trim().is_empty() {
                                                            model.set(first.id.clone());
                                                        }
                                                    let count = list.len();
                                                    available_models.set(list);
                                                    let msg = match *state.locale.read() {
                                                        Locale::Fr => format!("{count} modèle(s) disponible(s) détecté(s)."),
                                                        Locale::En => format!("{count} available model(s) detected."),
                                                    };
                                                    state.notify(msg);
                                                }
                                                Err(e) => {
                                                    let msg = match *state.locale.read() {
                                                        Locale::Fr => format!("Impossible de lister les modèles : {e}"),
                                                        Locale::En => format!("Unable to list models: {e}"),
                                                    };
                                                    state.notify(msg);
                                                }
                                            }
                                            loading_models.set(false);
                                        });
                                    }
                                },
                                if *loading_models.read() {
                                    {locale.tr("Interrogation…", "Querying…")}
                                } else {
                                    {locale.tr("Lister les modèles disponibles", "List available models")}
                                }
                            }
                        }

                        if !available_models.read().is_empty() {
                            div { style: "margin-top: 8px;",
                                select {
                                    style: "width: 100%;",
                                    onchange: move |e| model.set(e.value()),
                                    for m in available_models.read().iter() {
                                        option {
                                            value: "{m.id}",
                                            selected: *model.read() == m.id,
                                            "{m.label()}"
                                        }
                                    }
                                }
                            }
                        }
                    }

                    button {
                        class: "btn-primary",
                        onclick: {
                            let services = services.clone();
                            move |_| {
                                let next = AiSettings::new(
                                    *provider.read(),
                                    base_url.read().trim().to_string(),
                                    model.read().trim().to_string(),
                                    api_key.read().trim().to_string(),
                                );
                                if let Err(e) = next.validated_base_url() {
                                    state.notify(format!("Invalid URL: {e}"));
                                    return;
                                }
                                let services = services.clone();
                                spawn(async move {
                                    let loc = *state.locale.read();
                                    match services.ai_store.save(&next).await {
                                        Ok(()) => {
                                            services.chat_engines.configure(next.clone());
                                            state.ai_settings.set(next);
                                            state.notify(loc.tr("Configuration IA enregistrée.", "AI settings saved."));
                                        }
                                        Err(e) => state.notify(format!("{}: {e}", loc.tr("Erreur de sauvegarde IA", "AI save error"))),
                                    }
                                });
                            }
                        }, {locale.tr("Enregistrer les réglages IA", "Save AI settings")} }
                }

                div { class: "card",
                    h3 { class: "card-title", {locale.tr("Sauvegarde & Restauration de l'atelier", "Studio Backup & Restore")} }
                    p { class: "muted", style: "margin-bottom: 14px; font-size: 12.5px;", {locale.tr("Exportez ou restaurez une archive ZIP comprenant votre tiroir de tubes calibrés, vos palettes, vos recettes, vos projets et leurs photos.", "Export or restore a ZIP archive containing your calibrated paint drawer, palettes, recipes, projects, and photos.")} }

                    div { class: "card-subtle", style: "margin-bottom: 14px;",
                        div { class: "muted", {locale.tr("Emplacement de la base SQLite (WAL activé)", "SQLite database path (WAL enabled)")} }
                        div { class: "mono", style: "margin-top: 4px;", "{services.db_dir.display()}/palettier.db" }
                    }

                    div { class: "row",
                        button {
                            class: "btn-primary",
                            onclick: {
                                let services = services.clone();
                                move |_| {
                                    let services = services.clone();
                                    spawn(async move {
                                        let loc = *state.locale.read();
                                        let file = rfd::AsyncFileDialog::new()
                                            .set_file_name(loc.tr("sauvegarde-palettier.zip", "palettier-backup.zip"))
                                            .add_filter("Palettier ZIP", &["zip"])
                                            .save_file()
                                            .await;
                                        let Some(handle) = file else { return };
                                        let path = handle.path().to_path_buf();
                                        let mut writer = ZipBackupWriter::new(&path);
                                        match services.exporter().export(&mut writer).await.and_then(|rep| {
                                            writer.finish().map(|()| rep)
                                        }) {
                                            Ok(rep) => {
                                                let msg = match loc {
                                                    Locale::Fr => format!(
                                                        "Sauvegarde exportée ({} palettes, {} projets, {} recettes, {} photos).",
                                                        rep.palettes, rep.projects, rep.recipes, rep.photos
                                                    ),
                                                    Locale::En => format!(
                                                        "Backup exported ({} palettes, {} projects, {} recipes, {} photos).",
                                                        rep.palettes, rep.projects, rep.recipes, rep.photos
                                                    ),
                                                };
                                                state.notify(msg);
                                            }
                                            Err(e) => state.notify(format!("{}: {e}", loc.tr("Erreur d'export", "Export error"))),
                                        }
                                    });
                                }
                            },
                            {locale.tr("Exporter une sauvegarde (.zip)…", "Export backup (.zip)…")}
                        }

                        button {
                            class: "btn",
                            onclick: {
                                let services = services.clone();
                                move |_| {
                                    let services = services.clone();
                                    spawn(async move {
                                        let loc = *state.locale.read();
                                        let file = rfd::AsyncFileDialog::new()
                                            .add_filter("Palettier ZIP", &["zip"])
                                            .pick_file()
                                            .await;
                                        let Some(handle) = file else { return };
                                        let path = handle.path().to_path_buf();
                                        match ZipBackupArchive::open(&path) {
                                            Ok(mut archive) => match services.importer().import(&mut archive).await {
                                                Ok(rep) => {
                                                    state.reload_all(&services).await;
                                                    let msg = match loc {
                                                        Locale::Fr => format!(
                                                            "Import réussi : {} tubes mis à jour, +{} palettes, +{} projets ({} photos), +{} recettes.",
                                                            rep.paints_updated,
                                                            rep.palettes_added,
                                                            rep.projects_added,
                                                            rep.photos,
                                                            rep.recipes_added
                                                        ),
                                                        Locale::En => format!(
                                                            "Import complete: {} tubes updated, +{} palettes, +{} projects ({} photos), +{} recipes.",
                                                            rep.paints_updated,
                                                            rep.palettes_added,
                                                            rep.projects_added,
                                                            rep.photos,
                                                            rep.recipes_added
                                                        ),
                                                    };
                                                    state.notify(msg);
                                                }
                                                Err(e) => state.notify(format!("{}: {e}", loc.tr("Erreur d'import", "Import error"))),
                                            },
                                            Err(e) => state.notify(format!("{}: {e}", loc.tr("Archive invalide", "Invalid archive"))),
                                        }
                                    });
                                }
                            },
                            {locale.tr("Importer une sauvegarde (.zip)…", "Import backup (.zip)…")}
                        }
                    }
                }
            }
        }
    }
}
