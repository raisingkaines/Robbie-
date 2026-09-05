use crate::config::Config;
use poise::serenity_prelude as serenity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionLevel {
    Player,
    Moderator,
    Administrator,
}

pub fn require_staff_guild(
    ctx: &poise::Context<'_, crate::Data, crate::Error>,
    config: &Config,
) -> Result<(), crate::Error> {
    if let Some(authorized_guild_id) = config.discord.guild_id {
        match ctx.guild_id() {
            Some(gid) if gid.get() == authorized_guild_id => Ok(()),
            _ => Err("Access Denied: Administrative and moderation commands are restricted to the authorized staff server.".into()),
        }
    } else if ctx.guild_id().is_none() {
        Err("Access Denied: Administrative and moderation commands cannot be executed in direct messages.".into())
    } else {
        Ok(())
    }
}

pub fn get_user_permission_level(
    ctx: &poise::Context<'_, crate::Data, crate::Error>,
    config: &Config,
) -> PermissionLevel {
    if let Some(authorized_guild_id) = config.discord.guild_id {
        if ctx.guild_id().map(|g| g.get()) != Some(authorized_guild_id) {
            return PermissionLevel::Player;
        }
    }

    let member = match ctx.author_member() {
        Some(m) => m,
        None => return PermissionLevel::Player,
    };

    if let Ok(perms) = member.permissions(ctx) {
        if perms.contains(serenity::Permissions::ADMINISTRATOR) {
            return PermissionLevel::Administrator;
        }
    }

    if let Some(guild) = ctx.guild() {
        if guild.owner_id == ctx.author().id {
            return PermissionLevel::Administrator;
        }
    }

    for role_id in &member.roles {
        if config.discord.admin_role_ids.contains(&role_id.get()) {
            return PermissionLevel::Administrator;
        }
    }

    if !config.discord.admin_role_names.is_empty() {
        if let Some(guild) = ctx.guild() {
            for role_id in &member.roles {
                if let Some(role) = guild.roles.get(role_id) {
                    for admin_name in &config.discord.admin_role_names {
                        if role.name.eq_ignore_ascii_case(admin_name) {
                            return PermissionLevel::Administrator;
                        }
                    }
                }
            }
        }
    }

    for role_id in &member.roles {
        if config.discord.moderator_role_ids.contains(&role_id.get()) {
            return PermissionLevel::Moderator;
        }
    }

    if !config.discord.moderator_role_names.is_empty() {
        if let Some(guild) = ctx.guild() {
            for role_id in &member.roles {
                if let Some(role) = guild.roles.get(role_id) {
                    for mod_name in &config.discord.moderator_role_names {
                        if role.name.eq_ignore_ascii_case(mod_name) {
                            return PermissionLevel::Moderator;
                        }
                    }
                }
            }
        }
    }

    PermissionLevel::Player
}

pub fn require_moderator(
    ctx: &poise::Context<'_, crate::Data, crate::Error>,
    config: &Config,
) -> Result<(), crate::Error> {
    require_staff_guild(ctx, config)?;
    match get_user_permission_level(ctx, config) {
        PermissionLevel::Moderator | PermissionLevel::Administrator => Ok(()),
        PermissionLevel::Player => {
            let role_desc = if !config.discord.moderator_role_names.is_empty() {
                format!("\"{}\"", config.discord.moderator_role_names.join("\" or \""))
            } else if !config.discord.moderator_role_ids.is_empty() {
                format!("Role ID(s): {:?}", config.discord.moderator_role_ids)
            } else {
                "Moderator role (or Server Administrator)".to_string()
            };

            Err(format!(
                "Access Denied: You must have the {role_desc} to use moderation commands."
            )
            .into())
        }
    }
}

pub fn require_admin(
    ctx: &poise::Context<'_, crate::Data, crate::Error>,
    config: &Config,
) -> Result<(), crate::Error> {
    require_staff_guild(ctx, config)?;
    match get_user_permission_level(ctx, config) {
        PermissionLevel::Administrator => Ok(()),
        _ => {
            let role_desc = if !config.discord.admin_role_names.is_empty() {
                format!("\"{}\"", config.discord.admin_role_names.join("\" or \""))
            } else if !config.discord.admin_role_ids.is_empty() {
                format!("Role ID(s): {:?}", config.discord.admin_role_ids)
            } else {
                "Administrator role (or Server Administrator)".to_string()
            };

            Err(format!(
                "Access Denied: You must have the {role_desc} to use this administrative command."
            )
            .into())
        }
    }
}
