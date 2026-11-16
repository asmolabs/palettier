use crate::db::{Database, map_db_err};
use crate::entity::setting;
use async_trait::async_trait;
use palettier_domain::error::{DomainError, DomainResult};
use palettier_domain::port::SettingsRepository;
use sea_orm::{DatabaseConnection, EntityTrait, Set};

#[derive(Clone)]
pub struct SeaOrmSettingsRepository {
    db: DatabaseConnection,
    use_keyring: bool,
}

impl SeaOrmSettingsRepository {
    const KEYRING_SERVICE: &'static str = "be.asmolabs.palettier";

    pub fn new(database: &Database) -> Self {
        Self {
            db: database.connection().clone(),
            use_keyring: false,
        }
    }

    pub fn with_keyring(database: &Database) -> Self {
        Self {
            db: database.connection().clone(),
            use_keyring: true,
        }
    }

    fn is_secret_key(key: &str) -> bool {
        key.ends_with(".apiKey") || key.ends_with(".api_key")
    }
}

#[async_trait]
impl SettingsRepository for SeaOrmSettingsRepository {
    async fn get(&self, key: &str) -> DomainResult<Option<String>> {
        if self.use_keyring
            && Self::is_secret_key(key)
            && let Ok(entry) = keyring::Entry::new(Self::KEYRING_SERVICE, key)
            && let Ok(secret) = entry.get_password()
            && !secret.is_empty()
        {
            return Ok(Some(secret));
        }
        let row = setting::Entity::find_by_id(key.to_string())
            .one(&self.db)
            .await
            .map_err(map_db_err)?;
        if self.use_keyring && Self::is_secret_key(key) && row.is_some() {
            tracing::warn!(
                key = key,
                "Clé secrète lue depuis la table SQLite héritée ; ré-enregistrez-la pour la migrer vers le trousseau système."
            );
        }
        Ok(row.map(|r| r.value))
    }

    async fn put(&self, key: &str, value: &str) -> DomainResult<()> {
        if self.use_keyring && Self::is_secret_key(key) {
            let entry = keyring::Entry::new(Self::KEYRING_SERVICE, key).map_err(|e| {
                DomainError::Persistence(format!(
                    "Impossible d'accéder au trousseau système pour {key}: {e}"
                ))
            })?;
            entry.set_password(value).map_err(|e| {
                DomainError::Persistence(format!(
                    "Impossible d'enregistrer la clé secrète dans le trousseau système : {e}"
                ))
            })?;
            // Remove any plaintext key from the SQLite table
            let _ = setting::Entity::delete_by_id(key.to_string())
                .exec(&self.db)
                .await;
            return Ok(());
        }
        let active = setting::ActiveModel {
            key: Set(key.to_string()),
            value: Set(value.to_string()),
        };
        setting::Entity::insert(active)
            .on_conflict(
                sea_orm::sea_query::OnConflict::column(setting::Column::Key)
                    .update_column(setting::Column::Value)
                    .to_owned(),
            )
            .exec(&self.db)
            .await
            .map_err(map_db_err)?;
        Ok(())
    }

    async fn remove(&self, key: &str) -> DomainResult<()> {
        if self.use_keyring
            && Self::is_secret_key(key)
            && let Ok(entry) = keyring::Entry::new(Self::KEYRING_SERVICE, key)
        {
            let _ = entry.delete_credential();
        }
        setting::Entity::delete_by_id(key.to_string())
            .exec(&self.db)
            .await
            .map_err(map_db_err)?;
        Ok(())
    }
}
