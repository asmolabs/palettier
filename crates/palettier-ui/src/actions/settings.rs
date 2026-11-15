//! Settings: appearance, language, the AI engine and the studio backups.

use crate::state::{AppServices, AppState, ThemeMode};
use palettier_ai::{AiProvider, AiSettings, ModelOption};
use palettier_data::backup::{ExportReport, ImportReport};
use palettier_data::{BackupWriter, ZipBackupArchive, ZipBackupWriter};
use palettier_domain::error::{DomainError, DomainResult};
use palettier_domain::i18n::Locale;
use palettier_domain::port::SettingsRepository;
use std::path::Path;

/// Remembers the studio's lighting theme.
pub async fn save_theme(services: &AppServices, theme: ThemeMode) -> DomainResult<()> {
    services.settings.put(AppState::KEY_THEME, theme.as_str()).await
}

/// Remembers the interface language.
pub async fn save_locale(services: &AppServices, locale: Locale) -> DomainResult<()> {
    services.settings.put(AppState::KEY_LOCALE, locale.code()).await
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
pub async fn import_backup(services: &AppServices, path: &Path) -> Result<ImportReport, ImportFailure> {
    let mut archive = ZipBackupArchive::open(path).map_err(ImportFailure::InvalidArchive)?;
    services
        .importer()
        .import(&mut archive)
        .await
        .map_err(ImportFailure::Import)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::projects::{NewProject, create_project};
    use palettier_domain::port::ProjectRepository;
    use palettier_domain::paint::PaintSystem;

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
        let dir = std::env::temp_dir().join(format!("palettier-backup-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let zip = dir.join("sauvegarde.zip");

        let exported = export_backup(&svc, &zip).await.unwrap();
        assert!(exported.projects >= 1);

        let fresh = AppServices::in_memory().await.unwrap();
        let imported = import_backup(&fresh, &zip).await.unwrap();
        assert!(imported.projects_added >= 1);
        assert!(fresh.projects.all().await.unwrap().iter().any(|p| p.name == "Sauvegardé"));

        let junk = dir.join("pas-une-archive.zip");
        std::fs::write(&junk, b"pas un zip").unwrap();
        assert!(matches!(
            import_backup(&fresh, &junk).await,
            Err(ImportFailure::InvalidArchive(_))
        ));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
