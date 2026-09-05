pub mod migrations;
pub mod models;
pub mod queries;

use anyhow::{Context, Result};
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;
use tracing::info;
use crate::config::DatabaseConfig;
use crate::error::{BotError, BotResult};

#[derive(Clone)]
pub enum Database {
    Sqlite(sqlx::SqlitePool),
    MySql(sqlx::MySqlPool),
}

impl Database {
    pub async fn connect(config: &DatabaseConfig) -> Result<Self> {
        let provider = config.provider.to_lowercase();

        if provider == "mysql" || provider == "mariadb" {
            let url = config
                .url
                .as_deref()
                .unwrap_or("mysql://root@127.0.0.1:3306/sanctuary");

            info!(
                "Connecting to MySQL/MariaDB database (max_connections: {}, acquire_timeout: {}s)...",
                config.max_connections,
                config.acquire_timeout_seconds
            );

            let pool = sqlx::mysql::MySqlPoolOptions::new()
                .max_connections(config.max_connections)
                .min_connections(config.min_connections)
                .acquire_timeout(Duration::from_secs(config.acquire_timeout_seconds))
                .idle_timeout(Some(Duration::from_secs(config.idle_timeout_seconds)))
                .max_lifetime(Some(Duration::from_secs(1800)))
                .connect(url)
                .await
                .with_context(|| format!("Failed to connect to MySQL/MariaDB database at {url}"))?;

            migrations::run_mysql_migrations(&pool).await?;
            info!("Connected to MySQL/MariaDB database successfully.");
            Ok(Database::MySql(pool))
        } else {
            let path = config
                .path
                .as_deref()
                .unwrap_or_else(|| Path::new("sanctuary.db"));

            info!(
                "Connecting to SQLite database at {:?} (max_connections: {}, busy_timeout: 5s, WAL mode)...",
                path,
                config.max_connections
            );

            if let Some(parent) = path.parent() {
                if !parent.exists() && !parent.as_os_str().is_empty() {
                    let _ = std::fs::create_dir_all(parent);
                }
            }

            let path_str = path.to_string_lossy().replace('\\', "/");
            let connection_str = format!("sqlite://{}?mode=rwc", path_str);

            let connect_opts = sqlx::sqlite::SqliteConnectOptions::from_str(&connection_str)
                .with_context(|| format!("Failed to parse SQLite connection URI: {connection_str}"))?
                .busy_timeout(Duration::from_secs(5))
                .pragma("journal_mode", "WAL")
                .pragma("synchronous", "NORMAL")
                .pragma("foreign_keys", "ON");

            let pool = sqlx::sqlite::SqlitePoolOptions::new()
                .max_connections(config.max_connections)
                .min_connections(config.min_connections)
                .acquire_timeout(Duration::from_secs(config.acquire_timeout_seconds))
                .idle_timeout(Some(Duration::from_secs(config.idle_timeout_seconds)))
                .connect_with(connect_opts)
                .await
                .with_context(|| format!("Failed to open SQLite database at {:?}", path))?;

            migrations::run_sqlite_migrations(&pool).await?;
            info!("Connected to SQLite database successfully.");
            Ok(Database::Sqlite(pool))
        }
    }

    pub async fn ping(&self) -> BotResult<()> {
        match self {
            Database::Sqlite(pool) => {
                sqlx::query("SELECT 1")
                    .execute(pool)
                    .await
                    .map_err(BotError::Database)?;
            }
            Database::MySql(pool) => {
                sqlx::query("SELECT 1")
                    .execute(pool)
                    .await
                    .map_err(BotError::Database)?;
            }
        }
        Ok(())
    }

    pub async fn close(&self) {
        match self {
            Database::Sqlite(pool) => pool.close().await,
            Database::MySql(pool) => pool.close().await,
        }
    }
}
