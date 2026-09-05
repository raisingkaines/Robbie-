use crate::db::models::{DbCharacter, DbModerationLog, DbUser, DbWarning};
use crate::gateway::OnlinePlayerInfo;
use crate::warnings::AutoAction;
use chrono::{DateTime, Utc};
use serenity::all::{Colour, CreateEmbed, CreateEmbedFooter};

const COLOR_SUCCESS: Colour = Colour::from_rgb(46, 204, 113);
const COLOR_WARNING: Colour = Colour::from_rgb(241, 196, 15);
const COLOR_DANGER: Colour = Colour::from_rgb(231, 76, 60);
const COLOR_INFO: Colour = Colour::from_rgb(52, 152, 219);
const COLOR_PURPLE: Colour = Colour::from_rgb(155, 89, 182);

pub fn ban_embed(
    player: &str,
    duration: &str,
    reason: &str,
    expires_at: Option<DateTime<Utc>>,
    moderator: &str,
    kicked_live: bool,
    warning_points: i32,
) -> CreateEmbed {
    let expires_str = match expires_at {
        Some(dt) if crate::utils::is_permanent_ban(&dt) => "Never (Permanent)".to_string(),
        Some(dt) => format!("<t:{}:F> (<t:{}:R>)", dt.timestamp(), dt.timestamp()),
        None => "Never (Permanent)".to_string(),
    };

    let live_notice = if kicked_live {
        "Player was online and kicked with in-game Warning! popup."
    } else {
        "Player is offline. Ban will be enforced on next login."
    };

    CreateEmbed::new()
        .title("Player Banned")
        .colour(COLOR_DANGER)
        .field("Player", format!("**{player}**"), true)
        .field("Duration", duration, true)
        .field("Moderator", moderator, true)
        .field("Reason", reason, false)
        .field("Expires", expires_str, true)
        .field("Active Warning Points", warning_points.to_string(), true)
        .field("Live Server Status", live_notice, false)
        .footer(CreateEmbedFooter::new("Robbie | Sanctuary Free Realms Moderation"))
        .timestamp(Utc::now())
}

pub fn unban_embed(player: &str, moderator: &str) -> CreateEmbed {
    CreateEmbed::new()
        .title("Player Unbanned")
        .colour(COLOR_SUCCESS)
        .field("Player", format!("**{player}**"), true)
        .field("Moderator", moderator, true)
        .field("Status", "Account lock removed. Player can now log in.", false)
        .footer(CreateEmbedFooter::new("Robbie | Sanctuary Free Realms Moderation"))
        .timestamp(Utc::now())
}

pub fn mute_embed(
    player: &str,
    duration: &str,
    reason: &str,
    expires_at: Option<DateTime<Utc>>,
    moderator: &str,
    live_updated: bool,
) -> CreateEmbed {
    let expires_str = match expires_at {
        Some(dt) => format!("<t:{}:F> (<t:{}:R>)", dt.timestamp(), dt.timestamp()),
        None => "Permanent".to_string(),
    };

    let live_notice = if live_updated {
        "In-game chat block applied immediately."
    } else {
        "Player is offline. Mute will be active on next login."
    };

    CreateEmbed::new()
        .title("Player Muted")
        .colour(COLOR_WARNING)
        .field("Player", format!("**{player}**"), true)
        .field("Duration", duration, true)
        .field("Moderator", moderator, true)
        .field("Reason", reason, false)
        .field("Expires", expires_str, true)
        .field("Live Status", live_notice, false)
        .footer(CreateEmbedFooter::new("Robbie | Sanctuary Free Realms Moderation"))
        .timestamp(Utc::now())
}

pub fn unmute_embed(player: &str, moderator: &str) -> CreateEmbed {
    CreateEmbed::new()
        .title("Player Unmuted")
        .colour(COLOR_SUCCESS)
        .field("Player", format!("**{player}**"), true)
        .field("Moderator", moderator, true)
        .field("Status", "Chat restriction removed.", false)
        .footer(CreateEmbedFooter::new("Robbie | Sanctuary Free Realms Moderation"))
        .timestamp(Utc::now())
}

pub fn kick_embed(player: &str, reason: &str, moderator: &str, live_success: bool) -> CreateEmbed {
    let status_str = if live_success {
        "Disconnect packet sent with Warning! popup displayed to player."
    } else {
        "Player was not found online in active zones."
    };

    CreateEmbed::new()
        .title("Player Kicked")
        .colour(COLOR_DANGER)
        .field("Player", format!("**{player}**"), true)
        .field("Moderator", moderator, true)
        .field("Reason", reason, false)
        .field("Action Result", status_str, false)
        .footer(CreateEmbedFooter::new("Robbie | Sanctuary Free Realms Moderation"))
        .timestamp(Utc::now())
}

pub fn warn_embed(
    player: &str,
    severity: i32,
    reason: &str,
    moderator: &str,
    total_points: i32,
    auto_action: &AutoAction,
) -> CreateEmbed {
    let (sev_name, color) = match severity {
        1 => ("Minor (1 pt)", COLOR_WARNING),
        2 => ("Major (2 pts)", Colour::from_rgb(230, 126, 34)),
        3 => ("Severe (3 pts)", COLOR_DANGER),
        _ => ("Standard", COLOR_WARNING),
    };

    let mut embed = CreateEmbed::new()
        .title("Infraction Warning Issued")
        .colour(color)
        .field("Player", format!("**{player}**"), true)
        .field("Severity", sev_name, true)
        .field("Moderator", moderator, true)
        .field("Reason", reason, false)
        .field("Total Active Points", format!("**{total_points}**"), true);

    match auto_action {
        AutoAction::None => {}
        AutoAction::AutoMute { duration_minutes, points } => {
            embed = embed.field(
                "Auto-Escalation: Mute Triggered",
                format!("Player reached {points} points. Automatically muted for {duration_minutes} minutes."),
                false,
            );
        }
        AutoAction::AutoBan { duration_desc, points, .. } => {
            embed = embed.field(
                "Auto-Escalation: Ban Triggered",
                format!("Player reached {points} points. Automatically banned ({duration_desc}) with in-game Warning! popup."),
                false,
            );
        }
    }

    embed
        .footer(CreateEmbedFooter::new("Robbie | Sanctuary Free Realms Moderation"))
        .timestamp(Utc::now())
}

pub fn lookup_embed(
    user: &DbUser,
    character: &DbCharacter,
    characters: &[DbCharacter],
    online_info: Option<&OnlinePlayerInfo>,
    active_warnings: &[DbWarning],
    recent_logs: &[DbModerationLog],
) -> CreateEmbed {
    let online_str = match online_info {
        Some(info) => format!("Online in **{}**", info.zone_name),
        None => "Offline".to_string(),
    };

    let member_badge = if user.is_member { "Member" } else { "Free Player" };
    let role_badge = if user.is_admin {
        "Admin"
    } else if user.is_mod {
        "Moderator"
    } else {
        "Player"
    };

    let char_list: Vec<String> = characters
        .iter()
        .map(|c| {
            if c.id == character.id {
                format!("- **{}** (Active)", c.full_name)
            } else {
                format!("- {}", c.full_name)
            }
        })
        .collect();

    let ban_status = match user.locked_until {
        Some(dt) if dt > Utc::now() => {
            if crate::utils::is_permanent_ban(&dt) {
                "Permanently Banned".to_string()
            } else {
                format!("Banned until <t:{}:F>", dt.timestamp())
            }
        }
        _ => "Clean (No Active Ban)".to_string(),
    };

    let mute_status = match user.muted_until {
        Some(dt) if dt > Utc::now() => format!("Muted until <t:{}:R>", dt.timestamp()),
        _ => "Clean (No Active Mute)".to_string(),
    };

    let total_points: i32 = active_warnings.iter().map(|w| w.severity).sum();
    let warn_status = if active_warnings.is_empty() {
        "0 Active Warnings".to_string()
    } else {
        format!("**{}** Active Warning(s) ({} points)", active_warnings.len(), total_points)
    };

    let mut history_str = String::new();
    if recent_logs.is_empty() {
        history_str.push_str("No prior moderation actions recorded.");
    } else {
        for log in recent_logs.iter().take(4) {
            history_str.push_str(&format!(
                "- **{}** by {} (<t:{}:R>){}\n",
                log.action,
                log.actor_name,
                log.created_at.timestamp(),
                log.reason.as_ref().map(|r| format!(" - *{}*", r)).unwrap_or_default()
            ));
        }
    }

    let coords_str = match (character.position_x, character.position_y, character.position_z) {
        (Some(x), Some(y), Some(z)) => format!("X: `{:.2}`, Y: `{:.2}`, Z: `{:.2}`", x, y, z),
        _ => "Default Spawn".to_string(),
    };

    CreateEmbed::new()
        .title(format!("Player Lookup: {}", character.full_name))
        .colour(COLOR_INFO)
        .field("Account", format!("`{}` (ID: {})", user.username, user.id), true)
        .field("Status", online_str, true)
        .field("Account Type", format!("{} | {}", member_badge, role_badge), true)
        .field("In-Game Coordinates", coords_str, true)
        .field("Characters on Account", char_list.join("\n"), false)
        .field("Moderation Status", format!("{}\n{}\n{}", ban_status, mute_status, warn_status), false)
        .field("Recent Moderation Actions", history_str, false)
        .footer(CreateEmbedFooter::new(format!("Robbie | Registered on {}", user.created.format("%Y-%m-%d"))))
        .timestamp(Utc::now())
}

pub fn history_embed(target_name: &str, logs: &[DbModerationLog]) -> CreateEmbed {
    let mut embed = CreateEmbed::new()
        .title(format!("Moderation History: {target_name}"))
        .colour(COLOR_PURPLE);

    if logs.is_empty() {
        embed = embed.description("No moderation records found for this player.");
    } else {
        for log in logs {
            let desc = format!(
                "**Action:** {}\n**Moderator:** {} ({})\n**Date:** <t:{}:F> (<t:{}:R>)\n**Duration:** {}\n**Reason:** {}",
                log.action,
                log.actor_name,
                log.actor_source,
                log.created_at.timestamp(),
                log.created_at.timestamp(),
                log.duration.as_deref().unwrap_or("N/A"),
                log.reason.as_deref().unwrap_or("No reason provided")
            );
            embed = embed.field(format!("Entry #{}", log.id), desc, false);
        }
    }

    embed
        .footer(CreateEmbedFooter::new("Robbie | Sanctuary Free Realms Moderation"))
        .timestamp(Utc::now())
}
