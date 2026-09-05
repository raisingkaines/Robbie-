use crate::db::models::DbWarning;
use crate::embeds;
use crate::utils::{find_target, log_mod_action, parse_duration_str, require_moderator, send_reply_and_audit, ParsedDuration};
use crate::warnings::evaluate_warning_escalation;
use crate::{Context, Error};
use chrono::Utc;
use poise::serenity_prelude as serenity;

#[poise::command(
    slash_command,
    subcommands(
        "ban", "unban", "mute", "unmute", "kick", "warn", "warnings", "clearwarning",
        "lookup", "history", "reports", "resolve", "rename", "chatlogs", "commandlogs", "coords"
    )
)]
pub async fn mod_cmd(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command)]
pub async fn ban(
    ctx: Context<'_>,
    #[description = "Target player name"] player: String,
    #[description = "Duration (e.g. 7d, 2h, 30m, perm)"] duration: String,
    #[description = "Reason for the ban"] reason: Option<String>,
) -> Result<(), Error> {
    require_moderator(&ctx, &ctx.data().config)?;
    ctx.defer().await?;

    let parsed_dur = parse_duration_str(&duration)
        .ok_or_else(|| "Invalid duration format. Use e.g. 30m, 2h, 7d, or perm.")?;
    let (user, character) = match find_target(&ctx, &player).await? {
        Some(p) => p,
        None => return Ok(()),
    };

    let expires_at = parsed_dur.to_expiration_time();
    let reason_str = reason.unwrap_or_else(|| "No reason provided".to_string());
    ctx.data().database.set_user_locked_until(user.id, expires_at).await?;

    let kick_res = ctx.data().gateway.kick_player(&character.full_name, Some(&reason_str), 2).await;
    let kicked_live = kick_res.map(|r| r.success).unwrap_or(false);

    let dur_str = parsed_dur.to_string();
    log_mod_action(&ctx, user.id, &character.full_name, "Ban", Some(&reason_str), Some(&dur_str)).await;

    let points = ctx.data().database.get_active_warning_points(user.id).await.unwrap_or(0);
    let embed = embeds::ban_embed(
        &character.full_name, &dur_str, &reason_str, expires_at, &ctx.author().name, kicked_live, points,
    );
    send_reply_and_audit(&ctx, embed).await
}

#[poise::command(slash_command)]
pub async fn unban(
    ctx: Context<'_>,
    #[description = "Target player name"] player: String,
) -> Result<(), Error> {
    require_moderator(&ctx, &ctx.data().config)?;
    ctx.defer().await?;

    let (user, character) = match find_target(&ctx, &player).await? {
        Some(p) => p,
        None => return Ok(()),
    };

    ctx.data().database.set_user_locked_until(user.id, None).await?;
    log_mod_action(&ctx, user.id, &character.full_name, "Unban", None, None).await;

    let embed = embeds::unban_embed(&character.full_name, &ctx.author().name);
    send_reply_and_audit(&ctx, embed).await
}

#[poise::command(slash_command)]
pub async fn mute(
    ctx: Context<'_>,
    #[description = "Target player name"] player: String,
    #[description = "Duration (e.g. 30m, 2h, 1d)"] duration: String,
    #[description = "Reason for the mute"] reason: Option<String>,
) -> Result<(), Error> {
    require_moderator(&ctx, &ctx.data().config)?;
    ctx.defer().await?;

    let parsed_dur = parse_duration_str(&duration)
        .ok_or_else(|| "Invalid duration format. Use e.g. 15m, 1h, 1d.")?;
    let (user, character) = match find_target(&ctx, &player).await? {
        Some(p) => p,
        None => return Ok(()),
    };

    let expires_at = parsed_dur.to_expiration_time();
    let reason_str = reason.unwrap_or_else(|| "No reason provided".to_string());
    ctx.data().database.set_user_muted_until(user.id, expires_at).await?;

    let warn_msg = format!("You have been muted for {}. Reason: {}", parsed_dur, reason_str);
    let _ = ctx.data().gateway.send_in_game_warning(&character.full_name, &warn_msg, 2).await;

    let dur_str = parsed_dur.to_string();
    log_mod_action(&ctx, user.id, &character.full_name, "Mute", Some(&reason_str), Some(&dur_str)).await;

    let embed = embeds::mute_embed(
        &character.full_name, &dur_str, &reason_str, expires_at, &ctx.author().name, true,
    );
    send_reply_and_audit(&ctx, embed).await
}

#[poise::command(slash_command)]
pub async fn unmute(
    ctx: Context<'_>,
    #[description = "Target player name"] player: String,
) -> Result<(), Error> {
    require_moderator(&ctx, &ctx.data().config)?;
    ctx.defer().await?;

    let (user, character) = match find_target(&ctx, &player).await? {
        Some(p) => p,
        None => return Ok(()),
    };

    ctx.data().database.set_user_muted_until(user.id, None).await?;
    log_mod_action(&ctx, user.id, &character.full_name, "Unmute", None, None).await;

    let embed = embeds::unmute_embed(&character.full_name, &ctx.author().name);
    send_reply_and_audit(&ctx, embed).await
}

#[poise::command(slash_command)]
pub async fn kick(
    ctx: Context<'_>,
    #[description = "Target player name"] player: String,
    #[description = "Reason for the kick"] reason: Option<String>,
) -> Result<(), Error> {
    require_moderator(&ctx, &ctx.data().config)?;
    ctx.defer().await?;

    let (user, character) = match find_target(&ctx, &player).await? {
        Some(p) => p,
        None => return Ok(()),
    };

    let reason_str = reason.unwrap_or_else(|| "Kicked by moderator".to_string());
    let kick_res = ctx.data().gateway.kick_player(&character.full_name, Some(&reason_str), 1).await;
    let live_success = kick_res.map(|r| r.success).unwrap_or(false);

    log_mod_action(&ctx, user.id, &character.full_name, "Kick", Some(&reason_str), None).await;

    let embed = embeds::kick_embed(&character.full_name, &reason_str, &ctx.author().name, live_success);
    send_reply_and_audit(&ctx, embed).await
}

#[poise::command(slash_command)]
pub async fn warn(
    ctx: Context<'_>,
    #[description = "Target player name"] player: String,
    #[description = "Severity: 1=Minor, 2=Major, 3=Severe"]
    #[min = 1]
    #[max = 3]
    severity: i32,
    #[description = "Reason for the warning"] reason: String,
) -> Result<(), Error> {
    require_moderator(&ctx, &ctx.data().config)?;
    ctx.defer().await?;

    let reason = reason.trim().to_string();
    if reason.is_empty() {
        ctx.say("Warning reason cannot be empty.").await?;
        return Ok(());
    }

    let (user, character) = match find_target(&ctx, &player).await? {
        Some(p) => p,
        None => return Ok(()),
    };

    let exp_days = ctx.data().config.warning_system.warning_expiration_days;
    let expires_at = Some(Utc::now() + chrono::Duration::days(exp_days));

    let warning = DbWarning {
        id: 0,
        target_user_id: user.id,
        target_name: character.full_name.clone(),
        reason: reason.clone(),
        issued_by: format!("Discord:{}", ctx.author().name),
        issued_by_source: "Discord".to_string(),
        severity,
        acknowledged: false,
        active: true,
        created_at: Utc::now(),
        expires_at,
    };
    ctx.data().database.create_warning(&warning).await?;

    let warn_chat = format!("Moderator Warning: {reason}");
    let _ = ctx.data().gateway.send_in_game_warning(&character.full_name, &warn_chat, severity).await;

    let auto_action = evaluate_warning_escalation(
        &ctx.data().database, &user, &character.full_name, &ctx.data().config.warning_system,
    ).await?;

    if let crate::warnings::AutoAction::AutoBan { .. } = &auto_action {
        let _ = ctx.data().gateway.kick_player(&character.full_name, Some(&format!("Warning Escalation: {}", reason)), 2).await;
    }

    let total_points = ctx.data().database.get_active_warning_points(user.id).await.unwrap_or(severity);
    log_mod_action(&ctx, user.id, &character.full_name, "Warn", Some(&format!("[Sev {}] {}", severity, reason)), None).await;

    let embed = embeds::warn_embed(&character.full_name, severity, &reason, &ctx.author().name, total_points, &auto_action);
    send_reply_and_audit(&ctx, embed).await
}

#[poise::command(slash_command)]
pub async fn warnings(
    ctx: Context<'_>,
    #[description = "Target player name"] player: String,
) -> Result<(), Error> {
    require_moderator(&ctx, &ctx.data().config)?;
    ctx.defer().await?;

    let (_, character) = match find_target(&ctx, &player).await? {
        Some(p) => p,
        None => return Ok(()),
    };

    let warn_list = ctx.data().database.get_active_warnings_for_user(character.user_id).await?;
    let total_points: i32 = warn_list.iter().map(|w| w.severity).sum();

    let mut embed = serenity::CreateEmbed::new()
        .title(format!("Active Warnings: {}", character.full_name))
        .colour(serenity::Colour::from_rgb(241, 196, 15))
        .field("Total Active Points", format!("**{total_points}**"), true)
        .field("Warning Count", warn_list.len().to_string(), true);

    if warn_list.is_empty() {
        embed = embed.description("This player has no active warnings.");
    } else {
        for w in &warn_list {
            let sev_desc = match w.severity { 1 => "Minor (1 pt)", 2 => "Major (2 pts)", 3 => "Severe (3 pts)", _ => "Unknown" };
            embed = embed.field(
                format!("Warning #{}", w.id),
                format!("**Severity:** {}\n**Issued by:** {}\n**Date:** <t:{}:F>\n**Reason:** {}", sev_desc, w.issued_by, w.created_at.timestamp(), w.reason),
                false,
            );
        }
    }

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn clearwarning(
    ctx: Context<'_>,
    #[description = "Warning ID to clear"] id: i64,
) -> Result<(), Error> {
    require_moderator(&ctx, &ctx.data().config)?;
    ctx.defer().await?;

    let cleared = ctx.data().database.clear_warning_by_id(id).await?;
    let msg = if cleared {
        format!("Warning #{id} has been deactivated and will no longer count towards points.")
    } else {
        format!("Warning #{id} was not found.")
    };
    ctx.say(msg).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn lookup(
    ctx: Context<'_>,
    #[description = "Target player name"] player: String,
) -> Result<(), Error> {
    require_moderator(&ctx, &ctx.data().config)?;
    ctx.defer().await?;

    let (user, character) = match find_target(&ctx, &player).await? {
        Some(p) => p,
        None => return Ok(()),
    };

    let db = &ctx.data().database;
    let characters = db.get_user_characters(user.id).await.unwrap_or_default();
    let active_warnings = db.get_active_warnings_for_user(user.id).await.unwrap_or_default();
    let recent_logs = db.get_moderation_logs_for_target(user.id, 5).await.unwrap_or_default();

    let online_players = ctx.data().gateway.get_online_players().await.ok();
    let online_info = online_players.as_ref().and_then(|list| {
        list.iter().find(|p| p.name.eq_ignore_ascii_case(&character.full_name))
    });

    let embed = embeds::lookup_embed(&user, &character, &characters, online_info, &active_warnings, &recent_logs);
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn history(
    ctx: Context<'_>,
    #[description = "Target player name"] player: String,
) -> Result<(), Error> {
    require_moderator(&ctx, &ctx.data().config)?;
    ctx.defer().await?;

    let (user, _) = match find_target(&ctx, &player).await? {
        Some(p) => p,
        None => return Ok(()),
    };

    let logs = ctx.data().database.get_moderation_logs_for_target(user.id, 10).await?;
    let embed = embeds::history_embed(&player, &logs);
    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn reports(
    ctx: Context<'_>,
    #[description = "Filter by target player name (optional)"] player: Option<String>,
    #[description = "Filter by status: Pending, Resolved, Dismissed (optional)"] status: Option<String>,
) -> Result<(), Error> {
    require_moderator(&ctx, &ctx.data().config)?;
    ctx.defer().await?;

    let reports = ctx.data().database.get_player_reports(player.as_deref(), status.as_deref(), 15).await?;
    if reports.is_empty() {
        ctx.say("No player reports found matching criteria.").await?;
        return Ok(());
    }

    let mut description = String::new();
    for r in reports.iter().take(10) {
        let tag = match r.status.to_lowercase().as_str() { "pending" => "[Pending]", "resolved" => "[Resolved]", _ => "[Dismissed]" };
        let zone = r.zone_name.as_deref().unwrap_or("Unknown");
        let details = r.description.as_deref().unwrap_or("No details");
        let entry = format!("**#{}** {} **{}** reported by *{}* ({})\nReason: **{}** | Zone: *{}*\nDetails: *{}*\n\n", r.id, tag, r.target_name, r.reporter_name, r.reporter_source, r.reason, zone, details);
        if description.len() + entry.len() > 3800 {
            description.push_str("*...truncated to fit Discord limits.*");
            break;
        }
        description.push_str(&entry);
    }

    let embed = serenity::CreateEmbed::new()
        .title(format!("Player Reports ({})", reports.len()))
        .colour(serenity::Colour::from_rgb(230, 126, 34))
        .description(description)
        .footer(serenity::CreateEmbedFooter::new("Robbie | Sanctuary Free Realms Moderation"))
        .timestamp(Utc::now());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn resolve(
    ctx: Context<'_>,
    #[description = "Report ID to resolve"] report_id: i64,
    #[description = "Status: 'resolved' or 'dismissed'"] status: String,
    #[description = "Resolution notes"] notes: Option<String>,
) -> Result<(), Error> {
    require_moderator(&ctx, &ctx.data().config)?;
    ctx.defer().await?;

    let st = match status.trim().to_lowercase().as_str() {
        "resolve" | "resolved" => "Resolved",
        "dismiss" | "dismissed" => "Dismissed",
        _ => return Err("Invalid status. Choose 'resolved' or 'dismissed'.".into()),
    };

    let author = ctx.author().name.clone();
    let success = ctx.data().database.resolve_player_report(report_id, st, &author, notes.as_deref()).await?;
    if success {
        let embed = serenity::CreateEmbed::new()
            .title(format!("Report #{} Updated", report_id))
            .colour(serenity::Colour::from_rgb(46, 204, 113))
            .field("Status", st, true)
            .field("Moderator", &author, true)
            .field("Notes", notes.as_deref().unwrap_or("None"), false)
            .footer(serenity::CreateEmbedFooter::new("Robbie | Sanctuary Free Realms Moderation"))
            .timestamp(Utc::now());
        send_reply_and_audit(&ctx, embed).await?;
    } else {
        ctx.say(format!("Report #{report_id} was not found.")).await?;
    }
    Ok(())
}

#[poise::command(slash_command)]
pub async fn rename(
    ctx: Context<'_>,
    #[description = "Current character name"] player: String,
    #[description = "New character name"] new_name: String,
    #[description = "Reason for renaming (e.g. Inappropriate Name)"] reason: Option<String>,
) -> Result<(), Error> {
    require_moderator(&ctx, &ctx.data().config)?;
    ctx.defer().await?;

    let trimmed_new = new_name.trim();
    if trimmed_new.len() < 3 || trimmed_new.len() > 32 {
        ctx.say("New character name must be between 3 and 32 characters long.").await?;
        return Ok(());
    }

    let (user, character) = match find_target(&ctx, &player).await? {
        Some(p) => p,
        None => return Ok(()),
    };

    let success = ctx.data().database.rename_player_character(&character.full_name, trimmed_new).await?;
    if success {
        let reason_str = reason.unwrap_or_else(|| "Inappropriate Name".to_string());
        log_mod_action(&ctx, user.id, trimmed_new, "Rename", Some(&format!("Old: {}. Reason: {}", character.full_name, reason_str)), None).await;
        let _ = ctx.data().gateway.rename_player(&character.full_name, trimmed_new).await;

        let embed = serenity::CreateEmbed::new()
            .title("Character Renamed")
            .colour(serenity::Colour::from_rgb(52, 152, 219))
            .field("Previous Name", &character.full_name, true)
            .field("New Name", trimmed_new, true)
            .field("Moderator", &ctx.author().name, true)
            .field("Reason", reason_str, false)
            .footer(serenity::CreateEmbedFooter::new("Robbie | Sanctuary Free Realms Moderation"))
            .timestamp(Utc::now());
        send_reply_and_audit(&ctx, embed).await?;
    } else {
        ctx.say(format!("Failed to rename player \"{player}\".")).await?;
    }
    Ok(())
}

#[poise::command(slash_command)]
pub async fn chatlogs(
    ctx: Context<'_>,
    #[description = "Player name (optional filter)"] player: Option<String>,
    #[description = "Chat channel (e.g. Say, Tell, Shout, Guild) (optional)"] channel: Option<String>,
    #[description = "Number of logs to fetch (default 20, max 50)"] limit: Option<usize>,
) -> Result<(), Error> {
    require_moderator(&ctx, &ctx.data().config)?;
    ctx.defer().await?;

    let max_rows = limit.unwrap_or(20).clamp(1, 50) as i64;
    let logs = ctx.data().database.get_chat_logs(player.as_deref(), channel.as_deref(), max_rows).await?;
    if logs.is_empty() {
        ctx.say("No chat logs found matching criteria.").await?;
        return Ok(());
    }

    let mut body = String::new();
    for log in logs.iter().take(15) {
        let target = log.recipient_name.as_ref().map(|r| format!(" -> *{}*", r)).unwrap_or_default();
        let line = format!("`{}` [{}] **{}**{}: {}\n", log.created_at.format("%H:%M:%S"), log.channel, log.sender_name, target, log.message);
        if body.len() + line.len() > 3800 {
            body.push_str("\n*...truncated to fit Discord limit.*");
            break;
        }
        body.push_str(&line);
    }

    let embed = serenity::CreateEmbed::new()
        .title(format!("Chat Logs ({})", logs.len()))
        .colour(serenity::Colour::from_rgb(52, 152, 219))
        .description(body)
        .footer(serenity::CreateEmbedFooter::new("Robbie | Sanctuary In-Game Chat Monitor"))
        .timestamp(Utc::now());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn commandlogs(
    ctx: Context<'_>,
    #[description = "Filter by actor / player name (optional)"] actor: Option<String>,
    #[description = "Filter by command name (optional)"] command: Option<String>,
    #[description = "Number of logs to fetch (default 20, max 50)"] limit: Option<usize>,
) -> Result<(), Error> {
    require_moderator(&ctx, &ctx.data().config)?;
    ctx.defer().await?;

    let max_rows = limit.unwrap_or(20).clamp(1, 50) as i64;
    let logs = ctx.data().database.get_command_logs(actor.as_deref(), command.as_deref(), max_rows).await?;
    if logs.is_empty() {
        ctx.say("No command audit logs found matching criteria.").await?;
        return Ok(());
    }

    let mut body = String::new();
    for log in logs.iter().take(15) {
        let target = log.target.as_ref().map(|t| format!(" on **{}**", t)).unwrap_or_default();
        let status = if log.success { "[OK]" } else { "[FAIL]" };
        let args = log.arguments.as_deref().unwrap_or("");
        let line = format!("`{}` {} **{}** ({}) ran `{}` {}{}\n", log.created_at.format("%Y-%m-%d %H:%M:%S"), status, log.actor_name, log.actor_source, log.command, args, target);
        if body.len() + line.len() > 3800 {
            body.push_str("\n*...truncated to fit Discord limit.*");
            break;
        }
        body.push_str(&line);
    }

    let embed = serenity::CreateEmbed::new()
        .title(format!("Command Audit Logs ({})", logs.len()))
        .colour(serenity::Colour::from_rgb(155, 89, 182))
        .description(body)
        .footer(serenity::CreateEmbedFooter::new("Robbie | Sanctuary Command Audit Trail"))
        .timestamp(Utc::now());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn coords(
    ctx: Context<'_>,
    #[description = "Target player name"] player: String,
) -> Result<(), Error> {
    require_moderator(&ctx, &ctx.data().config)?;
    ctx.defer().await?;

    let (user, character) = match find_target(&ctx, &player).await? {
        Some(p) => p,
        None => return Ok(()),
    };

    let online_players = ctx.data().gateway.get_online_players().await.ok();
    let online_info = online_players.as_ref().and_then(|list| {
        list.iter().find(|p| p.name.eq_ignore_ascii_case(&character.full_name))
    });

    let (cx, cy, cz) = match (character.position_x, character.position_y, character.position_z) {
        (Some(x), Some(y), Some(z)) => (format!("{:.2}", x), format!("{:.2}", y), format!("{:.2}", z)),
        _ => ("Not Set".to_string(), "Not Set".to_string(), "Not Set".to_string()),
    };
    let rx = character.rotation_x.map(|r| format!("{:.4}", r)).unwrap_or_else(|| "None".to_string());
    let rz = character.rotation_z.map(|r| format!("{:.4}", r)).unwrap_or_else(|| "None".to_string());
    let status_str = match online_info {
        Some(info) => format!("Online in **{}**", info.zone_name),
        None => "Offline".to_string(),
    };

    let embed = serenity::CreateEmbed::new()
        .title(format!("Player Coordinates: {}", character.full_name))
        .colour(serenity::Colour::from_rgb(52, 152, 219))
        .field("Player", format!("**{}**", character.full_name), true)
        .field("Account", format!("`{}`", user.username), true)
        .field("Online Status", status_str, true)
        .field("Coordinates", format!("X: `{}`\nY: `{}`\nZ: `{}`", cx, cy, cz), true)
        .field("Rotation", format!("RotX: `{}` | RotZ: `{}`", rx, rz), true)
        .footer(serenity::CreateEmbedFooter::new("Robbie | Sanctuary Free Realms Moderation"))
        .timestamp(Utc::now());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
