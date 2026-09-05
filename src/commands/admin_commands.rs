use crate::utils::{find_target, log_mod_action, require_admin, send_reply_and_audit};
use crate::{Context, Error};
use chrono::Utc;
use poise::serenity_prelude as serenity;

#[poise::command(
    slash_command,
    subcommands("promote", "demote", "broadcast", "motd", "teleport", "move_player", "coords")
)]
pub async fn admin_cmd(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command)]
pub async fn promote(
    ctx: Context<'_>,
    #[description = "Target player name"] player: String,
) -> Result<(), Error> {
    require_admin(&ctx, &ctx.data().config)?;
    ctx.defer().await?;

    let (user, character) = match find_target(&ctx, &player).await? {
        Some(p) => p,
        None => return Ok(()),
    };

    ctx.data().database.set_user_is_mod(user.id, true).await?;
    log_mod_action(&ctx, user.id, &character.full_name, "Promote", Some("Promoted to In-Game Moderator"), None).await;

    let embed = serenity::CreateEmbed::new()
        .title("Player Promoted to Moderator")
        .colour(serenity::Colour::from_rgb(46, 204, 113))
        .field("Player", format!("**{}**", character.full_name), true)
        .field("Account", format!("`{}`", user.username), true)
        .field("Promoted By", &ctx.author().name, true)
        .field("Status", "In-Game Moderator permissions active immediately.", false);

    send_reply_and_audit(&ctx, embed).await
}

#[poise::command(slash_command)]
pub async fn demote(
    ctx: Context<'_>,
    #[description = "Target player name"] player: String,
) -> Result<(), Error> {
    require_admin(&ctx, &ctx.data().config)?;
    ctx.defer().await?;

    let (user, character) = match find_target(&ctx, &player).await? {
        Some(p) => p,
        None => return Ok(()),
    };

    ctx.data().database.set_user_is_mod(user.id, false).await?;
    log_mod_action(&ctx, user.id, &character.full_name, "Demote", Some("Demoted from In-Game Moderator"), None).await;

    let embed = serenity::CreateEmbed::new()
        .title("Player Demoted from Moderator")
        .colour(serenity::Colour::from_rgb(230, 126, 34))
        .field("Player", format!("**{}**", character.full_name), true)
        .field("Demoted By", &ctx.author().name, true)
        .field("Status", "Moderator permissions removed.", false);

    send_reply_and_audit(&ctx, embed).await
}

#[poise::command(slash_command)]
pub async fn broadcast(
    ctx: Context<'_>,
    #[description = "Message to broadcast in-game"] message: String,
) -> Result<(), Error> {
    require_admin(&ctx, &ctx.data().config)?;
    ctx.defer().await?;

    let trimmed = message.trim();
    if trimmed.is_empty() {
        ctx.say("Broadcast message cannot be empty.").await?;
        return Ok(());
    }

    match ctx.data().gateway.broadcast_message(trimmed).await {
        Ok(_) => {
            let embed = serenity::CreateEmbed::new()
                .title("In-Game Broadcast Sent")
                .colour(serenity::Colour::from_rgb(52, 152, 219))
                .field("Message", format!("*\"{trimmed}\"*"), false)
                .field("Sender", ctx.author().name.clone(), true);
            ctx.send(poise::CreateReply::default().embed(embed)).await?;
        }
        Err(e) => {
            ctx.say(format!("Failed to deliver broadcast to Gateway: {e}")).await?;
        }
    }
    Ok(())
}

#[poise::command(slash_command)]
pub async fn motd(
    ctx: Context<'_>,
    #[description = "Title of the login Message of the Day popup"] title: String,
    #[description = "Message body text displayed on login"] message: String,
) -> Result<(), Error> {
    require_admin(&ctx, &ctx.data().config)?;
    ctx.defer().await?;

    let title = title.trim();
    let message = message.trim();
    if title.is_empty() || message.is_empty() {
        ctx.say("MOTD title and message cannot be empty.").await?;
        return Ok(());
    }

    match ctx.data().gateway.set_motd(title, message).await {
        Ok(resp) => {
            if resp.success {
                let embed = serenity::CreateEmbed::new()
                    .title("Message of the Day Updated")
                    .colour(serenity::Colour::from_rgb(52, 152, 219))
                    .field("MOTD Title", format!("**{}**", title), false)
                    .field("Message Content", format!("*\"{}\"*", message), false)
                    .field("Updated By", &ctx.author().name, true)
                    .field("In-Game Effects", "PacketMOTD (OpCode 87) broadcasted to connecting players.", false)
                    .footer(serenity::CreateEmbedFooter::new("Robbie | Sanctuary Free Realms Administration"))
                    .timestamp(Utc::now());
                send_reply_and_audit(&ctx, embed).await?;
            } else {
                ctx.say(format!("MOTD update rejected: {}", resp.message.unwrap_or_default())).await?;
            }
        }
        Err(e) => {
            ctx.say(format!("Failed to reach Gateway server: {e}")).await?;
        }
    }
    Ok(())
}

#[poise::command(slash_command)]
pub async fn teleport(
    ctx: Context<'_>,
    #[description = "Target player name"] player: String,
    #[description = "Destination zone name (e.g. sanctuary, seaside, snowhill)"] zone: String,
) -> Result<(), Error> {
    require_admin(&ctx, &ctx.data().config)?;
    ctx.defer().await?;

    let player = player.trim();
    let zone = zone.trim();
    if player.is_empty() || zone.is_empty() {
        ctx.say("Player name and destination zone cannot be empty.").await?;
        return Ok(());
    }

    match ctx.data().gateway.teleport_player(player, zone).await {
        Ok(resp) => {
            if resp.success {
                let embed = serenity::CreateEmbed::new()
                    .title("Player Teleported")
                    .colour(serenity::Colour::from_rgb(46, 204, 113))
                    .field("Player", format!("**{}**", player), true)
                    .field("Destination Zone", format!("`{}`", zone), true)
                    .field("Admin", &ctx.author().name, true)
                    .field("In-Game Effects", "PacketZoneTeleportRequest (OpCode 90) dispatched.", false)
                    .footer(serenity::CreateEmbedFooter::new("Robbie | Sanctuary Free Realms Administration"))
                    .timestamp(Utc::now());
                send_reply_and_audit(&ctx, embed).await?;
            } else {
                ctx.say(format!("Teleport rejected: {}", resp.message.unwrap_or_else(|| "Player offline or invalid zone.".to_string()))).await?;
            }
        }
        Err(e) => {
            ctx.say(format!("Failed to reach Gateway server: {e}")).await?;
        }
    }
    Ok(())
}

#[poise::command(slash_command, rename = "move")]
pub async fn move_player(
    ctx: Context<'_>,
    #[description = "Target player name"] player: String,
    #[description = "X coordinate"] x: f32,
    #[description = "Y coordinate"] y: f32,
    #[description = "Z coordinate"] z: f32,
    #[description = "Optional heading rotation in degrees (0 - 360)"] rotation: Option<f32>,
) -> Result<(), Error> {
    require_admin(&ctx, &ctx.data().config)?;
    ctx.defer().await?;

    if !x.is_finite() || !y.is_finite() || !z.is_finite() || rotation.map_or(false, |r| !r.is_finite()) {
        ctx.say("Invalid coordinates: values must be finite numbers.").await?;
        return Ok(());
    }

    let (user, character) = match find_target(&ctx, &player).await? {
        Some(p) => p,
        None => return Ok(()),
    };

    let (rot_x, rot_z) = match rotation {
        Some(deg) => {
            let rad = deg.to_radians();
            (Some(rad.sin()), Some(rad.cos()))
        }
        None => (None, None),
    };

    let db_updated = ctx.data().database.update_character_coordinates(character.id, x, y, z, rot_x, rot_z).await?;
    let gateway_res = ctx.data().gateway.move_player_coords(&character.full_name, x, y, z, rotation).await;

    let (status_title, live_status, color) = match gateway_res {
        Ok(r) if r.success => (
            "Player Relocated In-Game",
            "Live in-game entity updated immediately via OpCode 12 (ClientUpdatePacketUpdateLocation). Coordinates persisted to database.",
            serenity::Colour::from_rgb(46, 204, 113),
        ),
        Ok(_) => (
            "Player Coordinates Updated (Offline)",
            "Coordinates saved to database. Player is offline; position will apply upon next login.",
            serenity::Colour::from_rgb(52, 152, 219),
        ),
        Err(_) => (
            "Player Coordinates Updated (Gateway Unreachable)",
            "Coordinates saved to database. Gateway server was unreachable; position will apply upon next login.",
            serenity::Colour::from_rgb(241, 196, 15),
        ),
    };

    log_mod_action(&ctx, user.id, &character.full_name, "MoveCoords", Some(&format!("Relocated to X: {:.2}, Y: {:.2}, Z: {:.2}", x, y, z)), None).await;

    let rot_display = rotation.map(|deg| format!("{:.1} deg", deg)).unwrap_or_else(|| "Default".to_string());
    let embed = serenity::CreateEmbed::new()
        .title(status_title)
        .colour(color)
        .field("Player", format!("**{}**", character.full_name), true)
        .field("Account", format!("`{}`", user.username), true)
        .field("Admin", &ctx.author().name, true)
        .field("Coordinates", format!("X: `{:.2}`\nY: `{:.2}`\nZ: `{:.2}`", x, y, z), true)
        .field("Rotation", rot_display, true)
        .field("Database Sync", if db_updated { "Saved to Characters table" } else { "Failed to update DB" }, true)
        .field("Status", live_status, false)
        .footer(serenity::CreateEmbedFooter::new("Robbie | Sanctuary Free Realms Administration"))
        .timestamp(Utc::now());

    send_reply_and_audit(&ctx, embed).await
}

#[poise::command(slash_command)]
pub async fn coords(
    ctx: Context<'_>,
    #[description = "Target player name"] player: String,
) -> Result<(), Error> {
    require_admin(&ctx, &ctx.data().config)?;
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
        .field("Storage", "Characters table", true)
        .footer(serenity::CreateEmbedFooter::new("Robbie | Sanctuary Free Realms Administration"))
        .timestamp(Utc::now());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}
