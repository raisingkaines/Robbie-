use poise::serenity_prelude as serenity;
use serenity::all::{ChannelId, CreateEmbed, CreateMessage};
use tracing::{error, info};

pub async fn send_audit_log(
    ctx: &poise::Context<'_, crate::Data, crate::Error>,
    embed: CreateEmbed,
) {
    let channel_id = match ctx.data().config.discord.audit_log_channel_id {
        Some(id) => ChannelId::new(id),
        None => return,
    };

    let builder = CreateMessage::new().embed(embed);
    if let Err(e) = channel_id.send_message(ctx.http(), builder).await {
        error!("Failed to send audit log message to channel {channel_id}: {e}");
    } else {
        info!("Audit log sent to channel {channel_id}");
    }
}
