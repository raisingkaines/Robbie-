use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub discord: DiscordConfig,
    pub database: DatabaseConfig,
    pub gateway: GatewayConfig,
    #[serde(default)]
    pub warning_system: WarningSystemConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordConfig {
    pub token: String,
    pub guild_id: Option<u64>,
    #[serde(default)]
    pub moderator_role_ids: Vec<u64>,
    #[serde(default)]
    pub moderator_role_names: Vec<String>,
    #[serde(default)]
    pub admin_role_ids: Vec<u64>,
    #[serde(default)]
    pub admin_role_names: Vec<String>,
    pub audit_log_channel_id: Option<u64>,
    #[serde(default = "default_presence_interval")]
    pub presence_interval_seconds: u64,
}

fn default_presence_interval() -> u64 { 30 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    #[serde(default = "default_db_provider")]
    pub provider: String,
    pub path: Option<PathBuf>,
    pub url: Option<String>,
    #[serde(default = "default_db_max_conn")]
    pub max_connections: u32,
    #[serde(default = "default_db_min_conn")]
    pub min_connections: u32,
    #[serde(default = "default_db_acquire_timeout")]
    pub acquire_timeout_seconds: u64,
    #[serde(default = "default_db_idle_timeout")]
    pub idle_timeout_seconds: u64,
}

fn default_db_provider() -> String { "sqlite".to_string() }
fn default_db_max_conn() -> u32 { 10 }
fn default_db_min_conn() -> u32 { 1 }
fn default_db_acquire_timeout() -> u64 { 10 }
fn default_db_idle_timeout() -> u64 { 600 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    pub api_url: String,
    pub api_key: String,
    #[serde(default = "default_gateway_timeout")]
    pub timeout_seconds: u64,
    #[serde(default = "default_gateway_timeout")]
    pub connect_timeout_seconds: u64,
}

fn default_gateway_timeout() -> u64 { 5 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarningSystemConfig {
    #[serde(default = "default_mute_thresh")]
    pub auto_mute_threshold: i32,
    #[serde(default = "default_mute_mins")]
    pub auto_mute_duration_minutes: i64,
    #[serde(default = "default_ban_thresh_1")]
    pub auto_ban_threshold_1: i32,
    #[serde(default = "default_ban_hours_1")]
    pub auto_ban_duration_hours_1: i64,
    #[serde(default = "default_ban_thresh_2")]
    pub auto_ban_threshold_2: i32,
    #[serde(default = "default_ban_days_2")]
    pub auto_ban_duration_days_2: i64,
    #[serde(default = "default_ban_thresh_perm")]
    pub auto_ban_threshold_permanent: i32,
    #[serde(default = "default_warn_exp_days")]
    pub warning_expiration_days: i64,
}

fn default_mute_thresh() -> i32 { 3 }
fn default_mute_mins() -> i64 { 30 }
fn default_ban_thresh_1() -> i32 { 5 }
fn default_ban_hours_1() -> i64 { 24 }
fn default_ban_thresh_2() -> i32 { 7 }
fn default_ban_days_2() -> i64 { 7 }
fn default_ban_thresh_perm() -> i32 { 10 }
fn default_warn_exp_days() -> i64 { 30 }

impl Default for WarningSystemConfig {
    fn default() -> Self {
        Self {
            auto_mute_threshold: default_mute_thresh(),
            auto_mute_duration_minutes: default_mute_mins(),
            auto_ban_threshold_1: default_ban_thresh_1(),
            auto_ban_duration_hours_1: default_ban_hours_1(),
            auto_ban_threshold_2: default_ban_thresh_2(),
            auto_ban_duration_days_2: default_ban_days_2(),
            auto_ban_threshold_permanent: default_ban_thresh_perm(),
            warning_expiration_days: default_warn_exp_days(),
        }
    }
}

fn apply_env_str(var: &str, target: &mut String) {
    if let Ok(val) = std::env::var(var) {
        let t = val.trim();
        if !t.is_empty() { *target = t.to_string(); }
    }
}

fn apply_env_parse<T: std::str::FromStr>(var: &str, target: &mut T) {
    if let Ok(val) = std::env::var(var) {
        if let Ok(p) = val.trim().parse::<T>() { *target = p; }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let _ = dotenvy::dotenv();

        let mut cli_db_path: Option<String> = None;
        let mut cli_db_url: Option<String> = None;
        let mut cli_provider: Option<String> = None;
        let mut cli_config_path: Option<String> = None;

        let args: Vec<String> = std::env::args().collect();
        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--db" | "-d" | "--database" => { if i + 1 < args.len() { cli_db_path = Some(args[i + 1].clone()); i += 1; } }
                "--db-url" | "--url" => { if i + 1 < args.len() { cli_db_url = Some(args[i + 1].clone()); i += 1; } }
                "--provider" | "-p" => { if i + 1 < args.len() { cli_provider = Some(args[i + 1].clone()); i += 1; } }
                "--config" | "-c" => { if i + 1 < args.len() { cli_config_path = Some(args[i + 1].clone()); i += 1; } }
                _ => {}
            }
            i += 1;
        }

        let config_path = cli_config_path
            .or_else(|| std::env::var("SANCTUARY_CONFIG_PATH").ok())
            .unwrap_or_else(|| "config.toml".to_string());

        let mut config = if Path::new(&config_path).exists() {
            info!("Loading configuration from {config_path}");
            let content = std::fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read config file at {config_path}"))?;
            toml::from_str::<Config>(&content)
                .with_context(|| format!("Failed to parse config file at {config_path}"))?
        } else {
            Config {
                discord: DiscordConfig {
                    token: std::env::var("DISCORD_TOKEN").unwrap_or_default(),
                    guild_id: None,
                    moderator_role_ids: vec![],
                    moderator_role_names: vec![],
                    admin_role_ids: vec![],
                    admin_role_names: vec![],
                    audit_log_channel_id: None,
                    presence_interval_seconds: default_presence_interval(),
                },
                database: DatabaseConfig {
                    provider: std::env::var("DATABASE_PROVIDER").unwrap_or_else(|_| "sqlite".to_string()),
                    path: Some(PathBuf::from(std::env::var("DATABASE_PATH").unwrap_or_else(|_| "../Sanctuary-upstream/src/Sanctuary.Database.Sqlite/sanctuary.db".to_string()))),
                    url: std::env::var("DATABASE_URL").ok(),
                    max_connections: default_db_max_conn(),
                    min_connections: default_db_min_conn(),
                    acquire_timeout_seconds: default_db_acquire_timeout(),
                    idle_timeout_seconds: default_db_idle_timeout(),
                },
                gateway: GatewayConfig {
                    api_url: std::env::var("GATEWAY_API_URL").unwrap_or_else(|_| "http://127.0.0.1:5000".to_string()),
                    api_key: std::env::var("GATEWAY_API_KEY").unwrap_or_else(|_| "sanctuary_admin_secret_key".to_string()),
                    timeout_seconds: default_gateway_timeout(),
                    connect_timeout_seconds: default_gateway_timeout(),
                },
                warning_system: WarningSystemConfig::default(),
            }
        };

        if let Some(p) = cli_provider { config.database.provider = p; }
        if let Some(u) = cli_db_url { config.database.url = Some(u); }
        if let Some(p) = cli_db_path { config.database.path = Some(PathBuf::from(p)); }

        apply_env_str("DATABASE_PROVIDER", &mut config.database.provider);
        if let Ok(u) = std::env::var("DATABASE_URL") { if !u.trim().is_empty() { config.database.url = Some(u.trim().to_string()); } }
        if let Ok(p) = std::env::var("DATABASE_PATH") { if !p.trim().is_empty() { config.database.path = Some(PathBuf::from(p.trim())); } }
        apply_env_parse("DATABASE_MAX_CONNECTIONS", &mut config.database.max_connections);
        apply_env_parse("DATABASE_MIN_CONNECTIONS", &mut config.database.min_connections);
        apply_env_parse("DATABASE_ACQUIRE_TIMEOUT", &mut config.database.acquire_timeout_seconds);
        apply_env_parse("DATABASE_IDLE_TIMEOUT", &mut config.database.idle_timeout_seconds);

        apply_env_str("DISCORD_TOKEN", &mut config.discord.token);
        apply_env_parse("DISCORD_PRESENCE_INTERVAL", &mut config.discord.presence_interval_seconds);
        apply_env_str("GATEWAY_API_URL", &mut config.gateway.api_url);
        apply_env_str("GATEWAY_API_KEY", &mut config.gateway.api_key);
        apply_env_parse("GATEWAY_TIMEOUT_SECONDS", &mut config.gateway.timeout_seconds);
        apply_env_parse("GATEWAY_CONNECT_TIMEOUT_SECONDS", &mut config.gateway.connect_timeout_seconds);

        if let Ok(s) = std::env::var("MODERATOR_ROLE_ID") {
            if let Ok(id) = s.trim().parse::<u64>() {
                if !config.discord.moderator_role_ids.contains(&id) { config.discord.moderator_role_ids.push(id); }
            }
        }
        if let Ok(name) = std::env::var("MODERATOR_ROLE_NAME") {
            let t = name.trim();
            if !t.is_empty() && !config.discord.moderator_role_names.iter().any(|r| r.eq_ignore_ascii_case(t)) {
                config.discord.moderator_role_names.push(t.to_string());
            }
        }
        if let Ok(s) = std::env::var("ADMIN_ROLE_ID") {
            if let Ok(id) = s.trim().parse::<u64>() {
                if !config.discord.admin_role_ids.contains(&id) { config.discord.admin_role_ids.push(id); }
            }
        }
        if let Ok(name) = std::env::var("ADMIN_ROLE_NAME") {
            let t = name.trim();
            if !t.is_empty() && !config.discord.admin_role_names.iter().any(|r| r.eq_ignore_ascii_case(t)) {
                config.discord.admin_role_names.push(t.to_string());
            }
        }
        if let Ok(s) = std::env::var("AUDIT_LOG_CHANNEL_ID") {
            if let Ok(id) = s.trim().parse::<u64>() { config.discord.audit_log_channel_id = Some(id); }
        }

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_warning_config() {
        let cfg = WarningSystemConfig::default();
        assert_eq!(cfg.auto_mute_threshold, 3);
        assert_eq!(cfg.auto_mute_duration_minutes, 30);
        assert_eq!(cfg.auto_ban_threshold_1, 5);
        assert_eq!(cfg.auto_ban_threshold_permanent, 10);
    }

    #[test]
    fn test_parse_minimal_toml_config() {
        let toml_str = r#"
            [discord]
            token = "test_token"

            [database]
            provider = "sqlite"

            [gateway]
            api_url = "http://127.0.0.1:5000"
            api_key = "test_key"
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.discord.token, "test_token");
        assert_eq!(config.database.max_connections, 10);
        assert_eq!(config.database.min_connections, 1);
        assert_eq!(config.gateway.timeout_seconds, 5);
        assert_eq!(config.discord.presence_interval_seconds, 30);
    }
}
