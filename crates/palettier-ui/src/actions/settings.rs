//! Settings: appearance, language, the AI engine and the studio backups.

use crate::state::{AppServices, AppState, ThemeMode};
use palettier_ai::{AiProvider, AiSettings, ModelOption};
use palettier_data::backup::{ExportReport, ImportReport};
use palettier_data::{BackupWriter, ZipBackupArchive, ZipBackupWriter};
use palettier_domain::drying::Workshop;
use palettier_domain::error::{DomainError, DomainResult};
use palettier_domain::i18n::Locale;
use palettier_domain::paint::Ventilation;
use palettier_domain::port::SettingsRepository;
use std::path::Path;
use std::str::FromStr;

/// Remembers the studio's lighting theme.
pub async fn save_theme(services: &AppServices, theme: ThemeMode) -> DomainResult<()> {
    services.settings.put(AppState::KEY_THEME, theme.as_str()).await
}

/// Remembers the interface language.
pub async fn save_locale(services: &AppServices, locale: Locale) -> DomainResult<()> {
    services.settings.put(AppState::KEY_LOCALE, locale.code()).await
}

/// Remembers whether the navigation sidebar is collapsed.
pub async fn save_sidebar_collapsed(services: &AppServices, collapsed: bool) -> DomainResult<()> {
    services
        .settings
        .put(
            AppState::KEY_SIDEBAR_COLLAPSED,
            if collapsed { "true" } else { "false" },
        )
        .await
}

/// Remembers the studio's climate conditions (temperature, humidity, ventilation).
pub async fn save_workshop(services: &AppServices, workshop: Workshop) -> DomainResult<()> {
    let encoded = format!(
        "{};{};{}",
        workshop.temperature_celsius,
        workshop.relative_humidity,
        workshop.ventilation.as_db_str()
    );
    services.settings.put(AppState::KEY_WORKSHOP, &encoded).await
}

/// Loads saved studio climate conditions, if present and valid.
pub async fn load_workshop(services: &AppServices) -> Option<Workshop> {
    let raw = services.settings.get(AppState::KEY_WORKSHOP).await.ok()??;
    parse_workshop(&raw)
}

fn parse_workshop(raw: &str) -> Option<Workshop> {
    let mut parts = raw.split(';');
    let temp = parts.next()?.trim().parse::<f64>().ok()?;
    let humidity = parts.next()?.trim().parse::<f64>().ok()?;
    let vent = Ventilation::from_str(parts.next()?.trim()).ok()?;
    if parts.next().is_some() {
        return None;
    }
    Workshop::new(temp, humidity, vent).ok()
}

/// The settings saved for a provider, or its defaults (no key) when none are.
pub async fn provider_settings(services: &AppServices, provider: AiProvider) -> AiSettings {
    services.ai_store.load_for(provider).await.unwrap_or_else(|_| {
        AiSettings::new(provider, provider.default_url(), provider.default_model(), String::new())
    })
}

/// The models the provider offers with these settings.
pub async fn list_models(services: &AppServices, settings: &AiSettings) -> DomainResult<Vec<ModelOption>> {
    services.model_catalog.available_for(settings).await
}

/// Saves the AI settings (the key goes to the OS keyring) and switches the engines to them.
pub async fn save_ai_settings(services: &AppServices, settings: &AiSettings) -> DomainResult<()> {
    services.ai_store.save(settings).await?;
    services.chat_engines.configure(settings.clone());
    Ok(())
}

/// Writes the whole studio to a ZIP archive.
pub async fn export_backup(services: &AppServices, path: &Path) -> DomainResult<ExportReport> {
    let mut writer = ZipBackupWriter::new(path);
    let report = services.exporter().export(&mut writer).await?;
    writer.finish()?;
    Ok(report)
}

/// Why a backup could not be restored.
#[derive(Debug)]
pub enum ImportFailure {
    /// The file is not a Palettier archive.
    InvalidArchive(DomainError),
    /// The archive was read but its content could not be imported.
    Import(DomainError),
}

/// Restores a ZIP archive into the studio.
pub async fn import_backup(
    services: &AppServices,
    path: &Path,
    locale: Locale,
) -> Result<ImportReport, ImportFailure> {
    let mut archive =
        ZipBackupArchive::open_in(path, locale).map_err(ImportFailure::InvalidArchive)?;
    services
        .importer()
        .import_in(&mut archive, locale)
        .await
        .map_err(ImportFailure::Import)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::catalog::{NewTube, add_custom_tube, calibrate_tube};
    use crate::actions::projects::{NewProject, create_project};
    use palettier_domain::paint::{DryingClass, Opacity, PaintSystem};
    use palettier_domain::port::{PaintCatalogRepository, ProjectRepository};

    #[tokio::test]
    async fn remembers_theme_and_language() {
        let svc = AppServices::in_memory().await.unwrap();
        save_theme(&svc, ThemeMode::Light).await.unwrap();
        save_locale(&svc, Locale::En).await.unwrap();
        let theme = svc.settings.get(AppState::KEY_THEME).await.unwrap();
        let locale = svc.settings.get(AppState::KEY_LOCALE).await.unwrap();
        assert_eq!(theme.as_deref(), Some(ThemeMode::Light.as_str()));
        assert_eq!(locale.as_deref(), Some(Locale::En.code()));
    }

    #[tokio::test]
    async fn an_unknown_provider_starts_from_its_defaults() {
        let svc = AppServices::in_memory().await.unwrap();
        for provider in AiProvider::ALL {
            let settings = provider_settings(&svc, provider).await;
            assert_eq!(settings.provider, provider);
            assert!(!settings.model.is_empty());
        }
    }

    #[tokio::test]
    async fn a_backup_round_trips_and_a_bad_archive_is_refused() {
        let svc = AppServices::in_memory().await.unwrap();
        let draft = NewProject {
            name: "Sauvegardé".into(),
            subject: String::new(),
            system: PaintSystem::Oil,
            palette: None,
        };
        create_project(&svc, draft, Locale::Fr).await.unwrap();
        let cat_id = svc.catalog.all().await.unwrap()[0].id.unwrap();
        calibrate_tube(&svc, cat_id, "#123456", "#D0C8BE").await.unwrap();
        let custom = add_custom_tube(
            &svc,
            NewTube {
                brand: "Atelier".into(),
                name: "Terre Calibrée".into(),
                code: "AT-01".into(),
                pigments: "PBr7".into(),
                hex_color: "#553322".into(),
                opacity: Opacity::Opaque,
                drying_class: DryingClass::Fast,
                system: PaintSystem::Acrylic,
            },
        )
        .await
        .unwrap()
        .unwrap();
        assert!(custom.user_added);
        save_theme(&svc, ThemeMode::Light).await.unwrap();
        save_locale(&svc, Locale::En).await.unwrap();
        svc.settings.put(AppState::KEY_PAINT_SYSTEM, PaintSystem::Acrylic.as_db_str()).await.unwrap();
        save_workshop(&svc, Workshop::new(23.5, 62.0, Ventilation::Good).unwrap()).await.unwrap();
        svc.settings.put("ai.openai.api_key", "sk-never-export").await.unwrap();
        let dir = std::env::temp_dir().join(format!("palettier-backup-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let zip = dir.join("sauvegarde.zip");

        let exported = export_backup(&svc, &zip).await.unwrap();
        assert!(exported.projects >= 1);

        let fresh = AppServices::in_memory().await.unwrap();
        let imported = import_backup(&fresh, &zip, Locale::Fr).await.unwrap();
        assert!(imported.projects_added >= 1);
        assert!(fresh.projects.all().await.unwrap().iter().any(|p| p.name == "Sauvegardé"));
        let restored_cal = fresh.catalog.all().await.unwrap()[0].clone();
        assert_eq!(restored_cal.hex_color.to_uppercase(), "#123456");
        assert_eq!(restored_cal.tint_hex.as_deref().map(str::to_uppercase).as_deref(), Some("#D0C8BE"));
        let restored_custom = fresh
            .catalog
            .find_by_natural_key("Atelier", "Terre Calibrée")
            .await
            .unwrap()
            .unwrap();
        assert!(restored_custom.user_added);
        assert_eq!(restored_custom.system, PaintSystem::Acrylic);
        assert_eq!(fresh.settings.get(AppState::KEY_THEME).await.unwrap().as_deref(), Some(ThemeMode::Light.as_str()));
        assert_eq!(fresh.settings.get(AppState::KEY_LOCALE).await.unwrap().as_deref(), Some(Locale::En.code()));
        assert_eq!(fresh.settings.get(AppState::KEY_PAINT_SYSTEM).await.unwrap().as_deref(), Some("ACRYLIC"));
        let restored_w = load_workshop(&fresh).await.unwrap();
        assert!((restored_w.temperature_celsius - 23.5).abs() < f64::EPSILON);
        assert_eq!(restored_w.ventilation, Ventilation::Good);
        assert!(fresh.settings.get("ai.openai.api_key").await.unwrap().is_none());

        let junk = dir.join("pas-une-archive.zip");
        std::fs::write(&junk, b"pas un zip").unwrap();
        assert!(matches!(
            import_backup(&fresh, &junk, Locale::Fr).await,
            Err(ImportFailure::InvalidArchive(_))
        ));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[tokio::test]
    async fn remembers_workshop_climate() {
        let svc = AppServices::in_memory().await.unwrap();
        assert!(load_workshop(&svc).await.is_none());

        let climate = Workshop::new(24.5, 68.0, Ventilation::Good).unwrap();
        save_workshop(&svc, climate).await.unwrap();

        let loaded = load_workshop(&svc).await.unwrap();
        assert!((loaded.temperature_celsius - 24.5).abs() < f64::EPSILON);
        assert!((loaded.relative_humidity - 68.0).abs() < f64::EPSILON);
        assert_eq!(loaded.ventilation, Ventilation::Good);

        svc.settings.put(AppState::KEY_WORKSHOP, "bad;data").await.unwrap();
        assert!(load_workshop(&svc).await.is_none());
    }
}
