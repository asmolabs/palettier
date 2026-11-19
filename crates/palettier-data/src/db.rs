use crate::migration::Migrator;
use async_trait::async_trait;
use palettier_domain::error::{DomainError, DomainResult};
use sea_orm::{
    AccessMode, ConnectOptions, ConnectionTrait, DatabaseConnection, DatabaseTransaction, DbBackend,
    DbErr, ExecResult, IsolationLevel, QueryResult, Statement, TransactionError,
    TransactionOptions, TransactionTrait,
};
use sea_orm_migration::MigratorTrait;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

/// Resolves the Palettier database directory: `PALETTIER_DB_DIR` when set and non-empty,
/// otherwise `~/.palettier`. A stray `.palettier` in the current working directory is only
/// used when the home directory one does not exist yet.
pub fn default_db_dir() -> PathBuf {
    let home = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".palettier");
    let local = PathBuf::from(".").join(".palettier");
    let env = std::env::var_os("PALETTIER_DB_DIR")
        .map(PathBuf::from)
        .filter(|p| !p.as_os_str().is_empty());
    let dir = choose_db_dir(env, &home, &local);
    if dir == local {
        tracing::warn!(
            "No database in {:?}: using the one in the current folder, {:?}",
            home,
            local
        );
    }
    dir
}

/// Selects the database directory given an optional environment override, the home directory
/// candidate, and the current-directory fallback.
pub fn choose_db_dir(env: Option<PathBuf>, home: &Path, local: &Path) -> PathBuf {
    if let Some(dir) = env.filter(|p| !p.as_os_str().is_empty()) {
        return dir;
    }
    if !home.exists() && local.exists() {
        return local.to_path_buf();
    }
    home.to_path_buf()
}

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
            .sqlx_logging(false)
            .map_sqlx_sqlite_opts(|sqlite| sqlite.foreign_keys(true));

        let conn = sea_orm::Database::connect(opt).await.map_err(map_db_err)?;
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
            .sqlx_logging(false)
            // Apply per-connection PRAGMAs on every SQLite connection created in the pool.
            .map_sqlx_sqlite_opts(|sqlite| {
                sqlite
                    .foreign_keys(true)
                    .busy_timeout(Duration::from_secs(5))
                    .pragma("journal_mode", "WAL")
                    .pragma("synchronous", "NORMAL")
            });

        let conn = sea_orm::Database::connect(opt).await.map_err(map_db_err)?;
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

/// Shared execution handle allowing repositories to run either against the pool
/// or inside a single caller-managed [`DatabaseTransaction`] (via SQLite savepoints).
#[derive(Clone)]
pub(crate) enum DbExec {
    Pool(DatabaseConnection),
    Txn(Arc<DatabaseTransaction>),
}

#[async_trait]
impl ConnectionTrait for DbExec {
    fn get_database_backend(&self) -> DbBackend {
        match self {
            Self::Pool(db) => db.get_database_backend(),
            Self::Txn(txn) => txn.get_database_backend(),
        }
    }

    async fn execute_raw(&self, stmt: Statement) -> Result<ExecResult, DbErr> {
        match self {
            Self::Pool(db) => db.execute_raw(stmt).await,
            Self::Txn(txn) => txn.execute_raw(stmt).await,
        }
    }

    async fn execute_unprepared(&self, sql: &str) -> Result<ExecResult, DbErr> {
        match self {
            Self::Pool(db) => db.execute_unprepared(sql).await,
            Self::Txn(txn) => txn.execute_unprepared(sql).await,
        }
    }

    async fn query_one_raw(&self, stmt: Statement) -> Result<Option<QueryResult>, DbErr> {
        match self {
            Self::Pool(db) => db.query_one_raw(stmt).await,
            Self::Txn(txn) => txn.query_one_raw(stmt).await,
        }
    }

    async fn query_all_raw(&self, stmt: Statement) -> Result<Vec<QueryResult>, DbErr> {
        match self {
            Self::Pool(db) => db.query_all_raw(stmt).await,
            Self::Txn(txn) => txn.query_all_raw(stmt).await,
        }
    }

    fn support_returning(&self) -> bool {
        match self {
            Self::Pool(db) => db.support_returning(),
            Self::Txn(txn) => txn.support_returning(),
        }
    }

    fn is_mock_connection(&self) -> bool {
        match self {
            Self::Pool(db) => db.is_mock_connection(),
            Self::Txn(txn) => txn.is_mock_connection(),
        }
    }
}

#[async_trait]
impl TransactionTrait for DbExec {
    type Transaction = DatabaseTransaction;

    async fn begin(&self) -> Result<Self::Transaction, DbErr> {
        match self {
            Self::Pool(db) => db.begin().await,
            Self::Txn(txn) => txn.begin().await,
        }
    }

    async fn begin_with_config(
        &self,
        isolation_level: Option<IsolationLevel>,
        access_mode: Option<AccessMode>,
    ) -> Result<Self::Transaction, DbErr> {
        match self {
            Self::Pool(db) => db.begin_with_config(isolation_level, access_mode).await,
            Self::Txn(txn) => txn.begin_with_config(isolation_level, access_mode).await,
        }
    }

    async fn begin_with_options(
        &self,
        options: TransactionOptions,
    ) -> Result<Self::Transaction, DbErr> {
        match self {
            Self::Pool(db) => db.begin_with_options(options).await,
            Self::Txn(txn) => txn.begin_with_options(options).await,
        }
    }

    async fn transaction<F, T, E>(&self, callback: F) -> Result<T, TransactionError<E>>
    where
        F: for<'c> FnOnce(
                &'c Self::Transaction,
            ) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'c>>
            + Send,
        T: Send,
        E: std::fmt::Display + std::fmt::Debug + Send,
    {
        match self {
            Self::Pool(db) => db.transaction(callback).await,
            Self::Txn(txn) => txn.transaction(callback).await,
        }
    }

    async fn transaction_with_config<F, T, E>(
        &self,
        callback: F,
        isolation_level: Option<IsolationLevel>,
        access_mode: Option<AccessMode>,
    ) -> Result<T, TransactionError<E>>
    where
        F: for<'c> FnOnce(
                &'c Self::Transaction,
            ) -> Pin<Box<dyn Future<Output = Result<T, E>> + Send + 'c>>
            + Send,
        T: Send,
        E: std::fmt::Display + std::fmt::Debug + Send,
    {
        match self {
            Self::Pool(db) => {
                db.transaction_with_config(callback, isolation_level, access_mode)
                    .await
            }
            Self::Txn(txn) => {
                txn.transaction_with_config(callback, isolation_level, access_mode)
                    .await
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{ConnectionTrait, Statement, TransactionTrait};

    #[test]
    fn the_home_database_wins_over_a_stray_local_one() {
        let tmp = std::env::temp_dir().join(format!("palettier-dbdir-{}", std::process::id()));
        let home = tmp.join("home");
        let local = tmp.join("local");
        std::fs::create_dir_all(&local).expect("temp dir");

        // Only the local one exists: it is used.
        assert_eq!(choose_db_dir(None, &home, &local), local);
        // Empty environment override is ignored.
        assert_eq!(
            choose_db_dir(Some(PathBuf::new()), &home, &local),
            local
        );
        // Both exist: the home one wins.
        std::fs::create_dir_all(&home).expect("temp dir");
        assert_eq!(choose_db_dir(None, &home, &local), home);
        // The environment variable wins over both.
        let env = PathBuf::from("/elsewhere");
        assert_eq!(choose_db_dir(Some(env.clone()), &home, &local), env);

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[tokio::test]
    async fn every_pooled_connection_enforces_foreign_keys_and_busy_timeout() {
        let tmp = std::env::temp_dir().join(format!(
            "palettier-pool-pragmas-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        let db = Database::open(&tmp).await.expect("open db");

        // Hold 5 concurrent transactions so SQLx opens all 5 connections in the pool.
        let mut txns = Vec::with_capacity(5);
        for _ in 0..5 {
            txns.push(db.connection().begin().await.expect("begin txn"));
        }

        for txn in &txns {
            let fk = txn
                .query_one_raw(Statement::from_string(
                    sea_orm::DatabaseBackend::Sqlite,
                    "PRAGMA foreign_keys;",
                ))
                .await
                .expect("query foreign_keys")
                .expect("row");
            let fk_val: i64 = fk.try_get_by_index(0).expect("fk value");
            assert_eq!(1, fk_val);

            let timeout = txn
                .query_one_raw(Statement::from_string(
                    sea_orm::DatabaseBackend::Sqlite,
                    "PRAGMA busy_timeout;",
                ))
                .await
                .expect("query busy_timeout")
                .expect("row");
            let timeout_val: i64 = timeout.try_get_by_index(0).expect("timeout value");
            assert_eq!(5000, timeout_val);
        }

        for txn in txns {
            txn.rollback().await.expect("rollback");
        }
        db.close().await.expect("close");
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
