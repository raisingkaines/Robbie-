use crate::utils::{parse_duration_str, require_admin, require_moderator, send_audit_log, ParsedDuration};
use crate::{Context, Error};
use chrono::Utc;
use poise::serenity_prelude as serenity;

#[poise::command(
    slash_command,
    subcommands("status", "players", "shutdown", "cancel_shutdown", "maintenance")
)]
pub async fn server_cmd(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command)]
pub async fn status(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let gateway = &ctx.data().gateway;
    let database = &ctx.data().database;
    let status_res = gateway.get_status().await;
    let db_status = match database.ping().await {
        Ok(_) => "Connected (Healthy)",
        Err(_) => "Degraded / Unreachable",
    };

    match status_res {
        Ok(status) => {
            let uptime_hours = status.uptime_seconds / 3600;
            let uptime_mins = (status.uptime_seconds % 3600) / 60;
            let uptime_str = format!("{}h {}m", uptime_hours, uptime_mins);

            let maint_str = if status.maintenance_mode {
                "Active (Logins Locked)"
            } else {
                "Disabled (Normal)"
            };

            let shutdown_str = if status.shutdown_pending {
                format!("In Progress ({}s remaining)", status.shutdown_seconds_remaining)
            } else {
                "None"
            };

            let embed = serenity::CreateEmbed::new()
                .title("Sanctuary Server Status")
                .colour(serenity::Colour::from_rgb(46, 204, 113))
                .field("Status", "Online", true)
                .field("Players", format!("{}/{}", status.online_players, status.max_players), true)
                .field("Uptime", uptime_str, true)
                .field("Maintenance Mode", maint_str, true)
                .field("Shutdown Status", shutdown_str, true)
                .field("Gateway API", "Connected", true)
                .field("Database", db_status, true)
                .footer(serenity::CreateEmbedFooter::new("Robbie | Sanctuary Free Realms Emulator"))
                .timestamp(Utc::now());

            ctx.send(poise::CreateReply::default().embed(embed)).await?;
        }
        Err(e) => {
            let embed = serenity::CreateEmbed::new()
                .title("Sanctuary Server Status")
                .colour(serenity::Colour::from_rgb(231, 76, 60))
                .field("Status", "Offline / Unreachable", true)
                .field("Database", db_status, true)
                .field("Gateway Error", format!("`{e}`"), false)
                .footer(serenity::CreateEmbedFooter::new("Robbie | Sanctuary Free Realms Emulator"))
                .timestamp(Utc::now());

            ctx.send(poise::CreateReply::default().embed(embed)).await?;
        }
    }

    Ok(())
}

#[poise::command(slash_command)]
pub async fn players(ctx: Context<'_>) -> Result<(), Error> {
    require_moderator(&ctx, &ctx.data().config)?;
    ctx.defer().await?;

    let gateway = &ctx.data().gateway;
    let players_res = gateway.get_online_players().await;

    match players_res {
        Ok(player_list) => {
            let mut embed = serenity::CreateEmbed::new()
                .title(format!("Online Players ({})", player_list.len()))
                .colour(serenity::Colour::from_rgb(52, 152, 219));

            if player_list.is_empty() {
                embed = embed.description("No players are currently logged in.");
            } else {
                let mut list_str = String::new();
                for p in player_list.iter().take(30) {
                    let badge = if p.is_admin {
                        " [Admin]"
                    } else if p.is_mod {
                        " [Mod]"
                    } else {
                        ""
                    };

                    list_str.push_str(&format!("• **{}**{} - *{}*\n", p.name, badge, p.zone_name));
                }

                if player_list.len() > 30 {
                    list_str.push_str(&format!("\n*...and {} more*", player_list.len() - 30));
                }

                embed = embed.description(list_str);
            }

            ctx.send(poise::CreateReply::default().embed(embed)).await?;
        }
        Err(e) => {
            ctx.say(format!("Could not fetch online players from Gateway: {e}")).await?;
        }
    }

    Ok(())
}

#[poise::command(slash_command)]
pub async fn shutdown(
    ctx: Context<'_>,
    #[description = "Countdown before shutdown (e.g. 5m, 2m, 30s)"] countdown: String,
    #[description = "Reason displayed in-game to players"] reason: Option<String>,
) -> Result<(), Error> {
    require_admin(&ctx, &ctx.data().config)?;
    ctx.defer().await?;

    let parsed_dur = parse_duration_str(&countdown)
        .ok_or_else(|| "Invalid countdown format. Use e.g. 30s, 2m, 5m, 10m.")?;

    let seconds = match parsed_dur {
        ParsedDuration::Temporary(dur) => dur.num_seconds() as i32,
        ParsedDuration::Permanent => {
            return Err("Shutdown countdown cannot be permanent. Specify a valid duration.".into());
        }
    };

    if seconds <= 0 {
        return Err("Countdown must be greater than 0 seconds.".into());
    }

    let default_reason = "Scheduled maintenance and updates.".to_string();
    let reason_str = reason.as_ref().unwrap_or(&default_reason);
    let author_name = ctx.author().name.clone();

    let gateway = &ctx.data().gateway;
    let res = gateway.initiate_shutdown(seconds, Some(reason_str)).await;

    match res {
        Ok(api_resp) => {
            if api_resp.success {
                let embed = serenity::CreateEmbed::new()
                    .title("Server Shutdown Initiated")
                    .colour(serenity::Colour::from_rgb(231, 76, 60))
                    .field("Countdown", format!("{} second(s) ({})", seconds, parsed_dur), true)
                    .field("Initiated By", &author_name, true)
                    .field("Reason", reason_str, false)
                    .field(
                        "In-Game Effects",
                        "PacketWorldShutdownNotice (OpCode 92) broadcasted with countdown overlay to all online players.",
                        false,
                    )
                    .footer(serenity::CreateEmbedFooter::new("Robbie | Sanctuary Free Realms Administration"))
                    .timestamp(Utc::now());

                send_audit_log(&ctx, embed.clone()).await;
                ctx.send(poise::CreateReply::default().embed(embed)).await?;
            } else {
                let msg = api_resp.message.unwrap_or_else(|| "Failed to initiate shutdown.".to_string());
                ctx.say(format!("Shutdown request rejected: {msg}")).await?;
            }
        }
        Err(e) => {
            ctx.say(format!("Failed to reach Gateway server: {e}")).await?;
        }
    }

    Ok(())
}

#[poise::command(slash_command)]
pub async fn cancel_shutdown(ctx: Context<'_>) -> Result<(), Error> {
    require_admin(&ctx, &ctx.data().config)?;
    ctx.defer().await?;

    let author_name = ctx.author().name.clone();
    let gateway = &ctx.data().gateway;
    let res = gateway.cancel_shutdown().await;

    match res {
        Ok(api_resp) => {
            if api_resp.success {
                let msg = api_resp.message.unwrap_or_else(|| "Shutdown countdown cancelled.".to_string());
                let embed = serenity::CreateEmbed::new()
                    .title("Server Shutdown Cancelled")
                    .colour(serenity::Colour::from_rgb(46, 204, 113))
                    .field("Status", msg, false)
                    .field("Cancelled By", &author_name, true)
                    .field(
                        "In-Game Effects",
                        "In-game countdown aborted and all-clear notice sent to all players.",
                        false,
                    )
                    .footer(serenity::CreateEmbedFooter::new("Robbie | Sanctuary Free Realms Administration"))
                    .timestamp(Utc::now());

                send_audit_log(&ctx, embed.clone()).await;
                ctx.send(poise::CreateReply::default().embed(embed)).await?;
            } else {
                let msg = api_resp.message.unwrap_or_else(|| "No active shutdown to cancel.".to_string());
                ctx.say(format!("Cancel failed: {msg}")).await?;
            }
        }
        Err(e) => {
            ctx.say(format!("Failed to reach Gateway server: {e}")).await?;
        }
    }

    Ok(())
}

#[poise::command(slash_command)]
pub async fn maintenance(
    ctx: Context<'_>,
    #[description = "Action: 'on' to enable maintenance mode, 'off' to disable"] action: String,
    #[description = "Optional reason for maintenance"] reason: Option<String>,
) -> Result<(), Error> {
    require_admin(&ctx, &ctx.data().config)?;
    ctx.defer().await?;

    let act = action.trim().to_lowercase();
    let enabled = match act.as_str() {
        "on" | "enable" | "true" | "1" => true,
        "off" | "disable" | "false" | "0" => false,
        _ => return Err("Invalid action. Specify 'on' to enable or 'off' to disable maintenance mode.".into()),
    };

    let author_name = ctx.author().name.clone();
    let gateway = &ctx.data().gateway;
    let res = gateway.set_maintenance_mode(enabled, reason.as_deref()).await;

    match res {
        Ok(api_resp) => {
            if api_resp.success {
                let (title, colour, status_desc, effects) = if enabled {
                    (
                        "Maintenance Mode Enabled",
                        serenity::Colour::from_rgb(230, 126, 34),
                        "Login server is now locked. Non-admin logins will receive 'Server Locked' (Status 2).",
                        "In-game announcement broadcasted to all active zones informing players of maintenance.",
                    )
                } else {
                    (
                        "Maintenance Mode Disabled",
                        serenity::Colour::from_rgb(46, 204, 113),
                        "Login server unlocked. All players can now log in normally.",
                        "In-game all-clear announcement broadcasted to all players.",
                    )
                };

                let mut embed = serenity::CreateEmbed::new()
                    .title(title)
                    .colour(colour)
                    .field("Status", status_desc, false)
                    .field("Updated By", &author_name, true);

                if let Some(r) = reason {
                    embed = embed.field("Reason", r, false);
                }

                embed = embed
                    .field("In-Game Effects", effects, false)
                    .footer(serenity::CreateEmbedFooter::new("Robbie | Sanctuary Free Realms Administration"))
                    .timestamp(Utc::now());

                send_audit_log(&ctx, embed.clone()).await;
                ctx.send(poise::CreateReply::default().embed(embed)).await?;
            } else {
                let msg = api_resp.message.unwrap_or_else(|| "Failed to update maintenance mode.".to_string());
                ctx.say(format!("Maintenance request failed: {msg}")).await?;
            }
        }
        Err(e) => {
            ctx.say(format!("Failed to reach Gateway server: {e}")).await?;
        }
    }

    Ok(())
}
