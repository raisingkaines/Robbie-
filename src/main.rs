mod commands;
mod config;
mod db;
mod embeds;
pub mod error;
mod gateway;
mod utils;
mod warnings;

use anyhow::Result;
use config::Config;
use db::Database;
use error::BotError;
use gateway::GatewayClient;
use poise::serenity_prelude as serenity;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};

pub struct Data {
    pub config: Config,
    pub database: Database,
    pub gateway: GatewayClient,
}

pub type Error = BotError;
pub type Context<'a> = poise::Context<'a, Data, Error>;

async fn update_presence_loop(gateway: GatewayClient, http: Arc<serenity::Http>, interval_secs: u64) {
    let mut interval = tokio::time::interval(Duration::from_secs(interval_secs.max(10)));
    loop {
        interval.tick().await;
        match gateway.get_status().await {
            Ok(status) => {
                let activity_name = format!("Robbie | {} online", status.online_players);
                http.set_presence(
                    Some(serenity::ActivityData::watching(&activity_name)),
                    serenity::OnlineStatus::Online,
                );
            }
            Err(_) => {
                http.set_presence(
                    Some(serenity::ActivityData::playing("Robbie | Server Offline")),
                    serenity::OnlineStatus::DoNotDisturb,
                );
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,sanctuary_discord=debug".into()),
        )
        .init();

    info!("Starting Robbie - Sanctuary Free Realms Discord Moderation Bot...");

    let config = Config::load()?;
    info!("Configuration loaded.");

    let database = Database::connect(&config.database).await?;
    match database.ping().await {
        Ok(_) => info!("Database health check passed."),
        Err(e) => warn!("Database health check returned warning: {e}"),
    }

    let gateway = GatewayClient::new(
        config.gateway.api_url.clone(),
        config.gateway.api_key.clone(),
        config.gateway.timeout_seconds,
        config.gateway.connect_timeout_seconds,
    );

    let config_clone = config.clone();
    let database_clone = database.clone();
    let gateway_clone = gateway.clone();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::help(),
                commands::mod_cmd(),
                commands::admin_cmd(),
                commands::server_cmd(),
            ],
            on_error: |error| {
                Box::pin(async move {
                    match error {
                        poise::FrameworkError::Command { error, ctx, .. } => {
                            warn!("Error in command '{}': {:?}", ctx.command().name, error);
                            let _ = ctx.send(poise::CreateReply::default().content(format!("Error: {error}")).ephemeral(true)).await;
                        }
                        poise::FrameworkError::Setup { error, .. } => {
                            error!("Error in bot setup: {:?}", error);
                        }
                        other => {
                            if let Err(e) = poise::builtins::on_error(other).await {
                                error!("Error while handling error: {:?}", e);
                            }
                        }
                    }
                })
            },
            post_command: |ctx| {
                Box::pin(async move {
                    let db = &ctx.data().database;
                    let actor = ctx.author().name.clone();
                    let cmd = ctx.command().qualified_name.clone();
                    let log = db::models::DbCommandLog {
                        id: 0,
                        actor_user_id: None,
                        actor_name: actor,
                        actor_source: "Discord".to_string(),
                        command: cmd,
                        arguments: None,
                        target: None,
                        success: true,
                        created_at: chrono::Utc::now(),
                    };
                    let _ = db.log_command_execution(&log).await;
                })
            },
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                info!("Robbie connected as {}", _ready.user.name);

                if _ready.user.name != "Robbie" {
                    info!("Attempting to update Discord bot name to 'Robbie'...");
                    let _ = _ready.user.edit(ctx, serenity::EditProfile::new().username("Robbie")).await;
                }

                if let Some(guild_id) = config_clone.discord.guild_id {
                    info!("Registering slash commands to guild ID: {guild_id}");
                    let _ = serenity::Command::set_global_commands(ctx, vec![]).await;
                    poise::builtins::register_in_guild(ctx, &framework.options().commands, serenity::GuildId::new(guild_id)).await?;
                } else {
                    warn!("WARNING: No guild_id configured. Commands will be registered globally. Set guild_id to restrict commands to your staff server.");
                    poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                }

                let gw = gateway_clone.clone();
                let http = ctx.http.clone();
                let presence_interval = config_clone.discord.presence_interval_seconds;
                tokio::spawn(async move {
                    update_presence_loop(gw, http, presence_interval).await;
                });

                Ok(Data {
                    config: config_clone,
                    database: database_clone,
                    gateway: gateway_clone,
                })
            })
        })
        .build();

    let intents = serenity::GatewayIntents::non_privileged();

    let client = serenity::ClientBuilder::new(&config.discord.token, intents)
        .framework(framework)
        .await;

    match client {
        Ok(mut c) => {
            let shard_manager = c.shard_manager.clone();
            let db_for_shutdown = database.clone();

            tokio::spawn(async move {
                let _ = tokio::signal::ctrl_c().await;
                info!("Termination signal received. Initiating graceful shutdown...");
                shard_manager.shutdown_all().await;
                info!("Discord connection shards disconnected.");
                db_for_shutdown.close().await;
                info!("Database connection pool drained and closed.");
            });

            info!("Connecting to Discord Gateway...");
            c.start().await?;
            info!("Bot process terminated cleanly.");
        }
        Err(e) => {
            error!("Failed to create Discord client: {e}");
            anyhow::bail!("Failed to create Discord client: {e}");
        }
    }

    Ok(())
}
