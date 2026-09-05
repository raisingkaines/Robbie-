pub mod audit;
pub mod duration;
pub mod permissions;

pub use audit::*;
pub use duration::*;
pub use permissions::*;

use crate::db::models::{DbCharacter, DbModerationLog, DbUser};
use crate::{Context, Error};
use chrono::Utc;
use poise::serenity_prelude as serenity;

pub async fn find_target(
    ctx: &Context<'_>,
    player: &str,
) -> Result<Option<(DbUser, DbCharacter)>, Error> {
    let trimmed = player.trim();
    if trimmed.is_empty() {
        ctx.send(poise::CreateReply::default().content("Player name cannot be empty.").ephemeral(true)).await?;
        return Ok(None);
    }
    match ctx.data().database.find_user_and_character_by_name(trimmed).await? {
        Some(pair) => Ok(Some(pair)),
        None => {
            ctx.send(poise::CreateReply::default().content(format!("Player \"{trimmed}\" was not found in the Sanctuary database.")).ephemeral(true)).await?;
            Ok(None)
        }
    }
}

pub async fn log_mod_action(
    ctx: &Context<'_>,
    target_user_id: u64,
    target_name: &str,
    action: &str,
    reason: Option<&str>,
    duration: Option<&str>,
) {
    let log = DbModerationLog {
        id: 0,
        target_user_id,
        target_name: target_name.to_string(),
        actor_user_id: None,
        actor_name: format!("Discord:{}", ctx.author().name),
        actor_source: "Discord".to_string(),
        action: action.to_string(),
        reason: reason.map(|r| r.to_string()),
        duration: duration.map(|d| d.to_string()),
        created_at: Utc::now(),
    };
    let _ = ctx.data().database.log_moderation_action(&log).await;
}

pub async fn send_reply_and_audit(
    ctx: &Context<'_>,
    embed: serenity::CreateEmbed,
) -> Result<(), Error> {
    send_audit_log(ctx, embed.clone()).await;
    ctx.send(poise::CreateReply::default().embed(embed).ephemeral(true)).await?;
    Ok(())
}
