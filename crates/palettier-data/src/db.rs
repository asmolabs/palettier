use crate::migration::Migrator;
use palettier_domain::error::{DomainError, DomainResult};
use sea_orm::{ConnectOptions, ConnectionTrait, DatabaseConnection, DbErr};
use sea_orm_migration::MigratorTrait;
use std::path::Path;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct Database {
    conn: DatabaseConnection,
}

impl Database {
    /// In-memory SQLite database (a single connection kept open to preserve state).
    pub async fn in_memory() -> DomainResult<Self> {
        let mut opt = ConnectOptions::new("sqlite::memory:".to_string());
        opt.max_connections(1)
            .min_connections(1)
            .idle_timeout(Duration::from_secs(3600))
            .max_lifetime(Duration::from_secs(3600))
            .sqlx_logging(false);

        let conn = sea_orm::Database::connect(opt).await.map_err(map_db_err)?;
        conn.execute_unprepared("PRAGMA foreign_keys = ON;")
            .await
            .map_err(map_db_err)?;
        Migrator::up(&conn, None).await.map_err(map_db_err)?;
        Ok(Self { conn })
    }

    /// Opens or creates the on-disk SQLite database at `path` (or `palettier.db` if `path` is a directory).
    pub async fn open(path: impl AsRef<Path>) -> DomainResult<Self> {
        let path = path.as_ref();
        let db_file = if path.is_dir() || path.extension().is_none() {
            std::fs::create_dir_all(path)
                .map_err(|e| DomainError::Persistence(format!("Failed to create directory {:?}: {e}", path)))?;
            restrict_dir_permissions(path);
            path.join("palettier.db")
        } else {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    DomainError::Persistence(format!("Failed to create directory {:?}: {e}", parent))
                })?;
                restrict_dir_permissions(parent);
            }
            path.to_path_buf()
        };

        let abs_path = if db_file.is_absolute() {
            db_file
        } else {
            std::env::current_dir()
                .map(|cwd| cwd.join(&db_file))
                .unwrap_or(db_file)
        };

        let escaped_path = abs_path
            .display()
            .to_string()
            .replace('%', "%25")
            .replace('?', "%3F")
            .replace('#', "%23");
        let url = format!("sqlite://{escaped_path}?mode=rwc");
        let mut opt = ConnectOptions::new(url);
        opt.max_connections(5)
            .min_connections(1)
            .acquire_timeout(Duration::from_secs(10))
            .sqlx_logging(false);

        let conn = sea_orm::Database::connect(opt).await.map_err(map_db_err)?;
        conn.execute_unprepared(
            r#"
            PRAGMA foreign_keys = ON;
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA busy_timeout = 5000;
            "#,
        )
        .await
        .map_err(map_db_err)?;
        Migrator::up(&conn, None).await.map_err(map_db_err)?;
        restrict_file_permissions(&abs_path);
        Ok(Self { conn })
    }

    pub fn connection(&self) -> &DatabaseConnection {
        &self.conn
    }

    pub async fn close(self) -> DomainResult<()> {
        self.conn.close().await.map_err(map_db_err)
    }
}

#[cfg(unix)]
fn restrict_dir_permissions(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700));
}

#[cfg(not(unix))]
fn restrict_dir_permissions(_path: &Path) {}

#[cfg(unix)]
fn restrict_file_permissions(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    if path.exists() {
        let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
    }
}

#[cfg(not(unix))]
fn restrict_file_permissions(_path: &Path) {}

pub(crate) fn map_db_err(err: DbErr) -> DomainError {
    DomainError::Persistence(err.to_string())
}
