use crate::{Context, Error};
use chrono::Utc;
use poise::serenity_prelude as serenity;

#[poise::command(slash_command)]
pub async fn help(
    ctx: Context<'_>,
    #[description = "Specific command name for detailed syntax help (e.g. ban, move, shutdown)"]
    command: Option<String>,
) -> Result<(), Error> {
    ctx.defer().await?;

    if let Some(cmd_name) = command {
        let name = cmd_name.trim().to_lowercase();
        let embed = match name.as_str() {
            "ban" => help_detail(
                "/mod ban <player> <duration> [reason]",
                "Locks the target user account and delivers an in-game Warning! popup before disconnecting.",
                &[("<player>", "Character or account username"), ("<duration>", "Duration string: 30m, 2h, 7d, perm"), ("[reason]", "Reason recorded in logs and shown to player")],
                "/mod ban PlayerName 7d Speed hacking in Snowhill",
            ),
            "unban" => help_detail(
                "/mod unban <player>",
                "Removes account lock, allowing the player to log in immediately.",
                &[("<player>", "Character or account username")],
                "/mod unban PlayerName",
            ),
            "mute" => help_detail(
                "/mod mute <player> <duration> [reason]",
                "Silences in-game chat in real time and persists expiration timestamp to the database.",
                &[("<player>", "Character name"), ("<duration>", "Duration string: 15m, 1h, 1d"), ("[reason]", "Reason for the mute")],
                "/mod mute PlayerName 1h Chat spamming in Seaside",
            ),
            "unmute" => help_detail(
                "/mod unmute <player>",
                "Removes active chat restrictions immediately.",
                &[("<player>", "Character name")],
                "/mod unmute PlayerName",
            ),
            "kick" => help_detail(
                "/mod kick <player> [reason]",
                "Dispatches a ForceDisconnect packet (Reason 1) with an in-game popup displaying your reason.",
                &[("<player>", "Character name"), ("[reason]", "Custom reason displayed in player dialog")],
                "/mod kick PlayerName Disruptive behavior",
            ),
            "warn" => help_detail(
                "/mod warn <player> <severity> <reason>",
                "Issues an infraction warning (1-3 pts). Automatically escalates to mute or ban at thresholds.",
                &[("<player>", "Character name"), ("<severity>", "1=Minor (1pt), 2=Major (2pts), 3=Severe (3pts)"), ("<reason>", "Infraction explanation")],
                "/mod warn PlayerName 2 Inappropriate language in public channel",
            ),
            "warnings" => help_detail(
                "/mod warnings <player>",
                "Inspects all active warning records and cumulative points for a player.",
                &[("<player>", "Character name")],
                "/mod warnings PlayerName",
            ),
            "clearwarning" => help_detail(
                "/mod clearwarning <id>",
                "Deactivates a warning by ID so it no longer contributes to cumulative infraction points.",
                &[("<id>", "Numerical ID of the warning to deactivate")],
                "/mod clearwarning 14",
            ),
            "lookup" => help_detail(
                "/mod lookup <player>",
                "Displays full player profile, character list, current coordinates, and punishment history.",
                &[("<player>", "Character name")],
                "/mod lookup PlayerName",
            ),
            "history" => help_detail(
                "/mod history <player>",
                "Retrieves recent moderation audit logs recorded for a player.",
                &[("<player>", "Character name")],
                "/mod history PlayerName",
            ),
            "reports" => help_detail(
                "/mod reports [player] [status]",
                "Queries player reports submitted in-game via the built-in report pipeline (OpCode 68).",
                &[("[player]", "Optional filter by reported player"), ("[status]", "Optional filter: Pending, Resolved, Dismissed")],
                "/mod reports status:Pending",
            ),
            "resolve" => help_detail(
                "/mod resolve <report_id> <status> [notes]",
                "Marks a player report as resolved or dismissed with moderator audit notes.",
                &[("<report_id>", "ID of the report"), ("<status>", "'resolved' or 'dismissed'"), ("[notes]", "Resolution notes")],
                "/mod resolve 5 status:resolved notes:Handled with verbal warning",
            ),
            "rename" => help_detail(
                "/mod rename <player> <new_name> [reason]",
                "Force-renames an offensive character name in the database and broadcasts live network rename.",
                &[("<player>", "Current character name"), ("<new_name>", "Replacement character name (3-32 chars)"), ("[reason]", "Reason for rename")],
                "/mod rename BadName123 CleanHero Inappropriate Name",
            ),
            "coords" => help_detail(
                "/mod coords <player> or /admin coords <player>",
                "Displays current in-game coordinates (X, Y, Z), rotation heading, and online zone.",
                &[("<player>", "Character name")],
                "/mod coords PlayerName",
            ),
            "chatlogs" => help_detail(
                "/mod chatlogs [player] [channel] [limit]",
                "Searches in-game chat history with filters for sender, channel, and volume.",
                &[("[player]", "Optional sender filter"), ("[channel]", "Optional channel: Say, Tell, Shout, Guild"), ("[limit]", "Max records (1-50, default 20)")],
                "/mod chatlogs channel:Shout limit:25",
            ),
            "commandlogs" => help_detail(
                "/mod commandlogs [actor] [command] [limit]",
                "Queries the staff and player command audit trail across Discord and in-game commands.",
                &[("[actor]", "Optional actor filter"), ("[command]", "Optional command filter"), ("[limit]", "Max records (1-50, default 20)")],
                "/mod commandlogs command:ban",
            ),
            "move" => help_detail(
                "/admin move <player> <x> <y> <z> [rotation]",
                "Relocates a player in real time (OpCode 12) and persists new coordinates to the database.",
                &[("<player>", "Character name"), ("<x>", "X coordinate float"), ("<y>", "Y coordinate float"), ("<z>", "Z coordinate float"), ("[rotation]", "Optional heading degrees (0-360)")],
                "/admin move PlayerName 100.5 45.0 -250.0 90.0",
            ),
            "motd" => help_detail(
                "/admin motd <title> <message>",
                "Sets and broadcasts the in-game login Message of the Day popup (OpCode 87).",
                &[("<title>", "Popup header title"), ("<message>", "Announcement body text")],
                "/admin motd \"Server Event\" \"Double XP weekend active across all jobs!\"",
            ),
            "teleport" => help_detail(
                "/admin teleport <player> <zone>",
                "Dispatches a PacketZoneTeleportRequest (OpCode 90) to warp a player to another zone.",
                &[("<player>", "Character name"), ("<zone>", "Target zone name (e.g. sanctuary, seaside, snowhill)")],
                "/admin teleport PlayerName seaside",
            ),
            "broadcast" => help_detail(
                "/admin broadcast <message>",
                "Transmits an immediate server-wide announcement overlay to all zones.",
                &[("<message>", "Announcement text to broadcast")],
                "/admin broadcast Server restart in 15 minutes",
            ),
            "shutdown" => help_detail(
                "/server shutdown <countdown> [reason]",
                "Triggers the in-game countdown timer overlay (OpCode 92) and disconnects players at 0s.",
                &[("<countdown>", "Countdown length in seconds"), ("[reason]", "Shutdown reason")],
                "/server shutdown 300 Scheduled server upgrade",
            ),
            "cancel_shutdown" => help_detail(
                "/server cancel_shutdown",
                "Aborts a pending shutdown countdown and broadcasts an all-clear notice to players.",
                &[],
                "/server cancel_shutdown",
            ),
            "maintenance" => help_detail(
                "/server maintenance <on|off> [reason]",
                "Locks or unlocks the login server to prevent non-admin logins during maintenance.",
                &[("<on|off>", "'on' to lock logins, 'off' to unlock"), ("[reason]", "Maintenance reason")],
                "/server maintenance on Database backup in progress",
            ),
            "status" => help_detail(
                "/server status",
                "Inspects real-time server health, player count, uptime, DB connectivity, and maintenance state.",
                &[],
                "/server status",
            ),
            "players" => help_detail(
                "/server players",
                "Lists all connected players and their active zones.",
                &[],
                "/server players",
            ),
            _ => {
                let emb = serenity::CreateEmbed::new()
                    .title(format!("Unknown Command: {cmd_name}"))
                    .colour(serenity::Colour::from_rgb(231, 76, 60))
                    .description("Use `/help` without arguments to see the complete directory of available commands.")
                    .footer(serenity::CreateEmbedFooter::new("Robbie | Sanctuary Free Realms"))
                    .timestamp(Utc::now());
                ctx.send(poise::CreateReply::default().embed(emb)).await?;
                return Ok(());
            }
        };
        ctx.send(poise::CreateReply::default().embed(embed)).await?;
        return Ok(());
    }

    let embed = serenity::CreateEmbed::new()
        .title("Robbie - Command Directory")
        .colour(serenity::Colour::from_rgb(52, 152, 219))
        .description("Robbie is the Free Realms (Sanctuary) moderation and administration bot. Use `/help <command>` for detailed parameter syntax and examples.")
        .field(
            "Server Management (/server)",
            "`/server status` - Live player count, uptime, DB & Gateway health\n`/server players` - Connected player list and active zones\n`/server shutdown <seconds> [reason]` - In-game countdown overlay & kick\n`/server cancel_shutdown` - Abort countdown and send all-clear\n`/server maintenance <on|off> [reason]` - Toggle login server lock",
            false,
        )
        .field(
            "Moderation Commands (/mod)",
            "`/mod ban <player> <duration> [reason]` - Temp/perm ban with warning popup\n`/mod unban <player>` - Lift active ban\n`/mod kick <player> [reason]` - Disconnect online player with popup\n`/mod mute <player> <duration> [reason]` - Silence player chat in real time\n`/mod unmute <player>` - Lift chat silence\n`/mod warn <player> <severity> <reason>` - Issue warning (1-3 pts) with escalation\n`/mod warnings <player>` - Check active infraction points\n`/mod clearwarning <id>` - Revoke warning point\n`/mod lookup <player>` - Profile, characters, coordinates & history\n`/mod history <player>` - Audit log of staff punishments\n`/mod reports [player] [status]` - In-game player reports queue\n`/mod resolve <id> <status> [notes]` - Resolve/dismiss player report\n`/mod rename <player> <new_name>` - Force character rename\n`/mod coords <player>` - Inspect live position and heading\n`/mod chatlogs [player] [channel] [limit]` - Search in-game chat\n`/mod commandlogs [actor] [cmd] [limit]` - Staff & player audit trail",
            false,
        )
        .field(
            "Administration Commands (/admin)",
            "`/admin promote <player>` - Grant in-game moderator permissions\n`/admin demote <player>` - Revoke in-game moderator permissions\n`/admin broadcast <message>` - Instant server-wide announcement\n`/admin motd <title> <message>` - Set login MOTD popup (OpCode 87)\n`/admin teleport <player> <zone>` - Warp player to zone (OpCode 90)\n`/admin move <player> <x> <y> <z> [rot]` - Live coordinate relocation (OpCode 12)\n`/admin coords <player>` - Inspect player coordinates and zone",
            false,
        )
        .footer(serenity::CreateEmbedFooter::new("Robbie | Sanctuary Free Realms"))
        .timestamp(Utc::now());

    ctx.send(poise::CreateReply::default().embed(embed)).await?;
    Ok(())
}

fn help_detail(
    syntax: &str,
    description: &str,
    params: &[(&str, &str)],
    example: &str,
) -> serenity::CreateEmbed {
    let title = syntax.split_whitespace().take(2).collect::<Vec<_>>().join(" ");
    let mut embed = serenity::CreateEmbed::new()
        .title(format!("Command Help: {title}"))
        .colour(serenity::Colour::from_rgb(52, 152, 219))
        .description(description)
        .field("Syntax", format!("`{syntax}`"), false);

    let mut param_str = String::new();
    for (name, desc) in params {
        param_str.push_str(&format!("- `{name}`: {desc}\n"));
    }
    if !param_str.is_empty() {
        embed = embed.field("Parameters", param_str, false);
    }

    embed
        .field("Example", format!("`{example}`"), false)
        .footer(serenity::CreateEmbedFooter::new("Robbie | Sanctuary Free Realms"))
        .timestamp(Utc::now())
}
