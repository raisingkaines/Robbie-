# Robbie: Sanctuary Free Realms Discord Moderation Bot

<div align="center">
  <img src="assets/robbie_avatar.png" alt="Robbie - Sanctuary Discord Moderation Bot" width="180" />
  <p><strong>Robbie</strong> is the dedicated, high-performance Rust Discord moderation and administration bridge for the Open-Source Free Realms (Sanctuary) server emulator, supporting both SQLite and MySQL / MariaDB databases.</p>
</div>

### Visual Assets (`assets/` directory)
- `assets/robbie_avatar.png`: 512x512 square profile picture centered on Robbie's helmet, ears, and grumpy expression (ready to set as your Discord bot avatar in the Developer Portal).
- `assets/robbie_face_closeup.png`: 512x512 high-resolution close-up focusing directly on Robbie's face.
- `assets/robbie_full.png`: Full character art (1000x1000).

---

## Key Features

1. **Live In-Game Punishments**:
   - **`/mod ban <player> <duration> [reason]`**: Locks the user account in database and (if online) delivers the in-game Warning! popup before disconnecting the player.
   - **`/mod kick <player> [reason]`**: Sends the `ForceDisconnect` packet (`Reason = 1`) displaying the in-game popup with your custom reason before dropping the connection.
   - **`/mod mute <player> <duration> [reason]`**: Silences player chat in real-time and persists the expiration timestamp to the database.
   - **`/mod unban` & `/mod unmute`**: Lifts active restrictions.

2. **Infraction & Warning Escalation System**:
   - **`/mod warn <player> <severity> <reason>`**: Issues a persistent warning record (Minor = 1 pt, Major = 2 pts, Severe = 3 pts).
   - Automatically calculates cumulative active infraction points:
     - 3 points: Automatic 30-minute mute
     - 5 points: Automatic 24-hour ban with Warning popup
     - 7 points: Automatic 7-day ban with Warning popup
     - 10 points: Automatic Permanent ban
   - **`/mod warnings <player>`**: Inspects active warning records.
   - **`/mod clearwarning <id>`**: Revokes or deactivates a warning.

3. **In-Game Player Reports Queue (`OpCode 68`)**:
   - **`/mod reports [player] [status]`**: View reports submitted by players in-game via the built-in reporting system (shows reporter, target, reason category, description, and zone).
   - **`/mod resolve <report_id> <resolved/dismissed> [notes]`**: Mark a report as resolved or dismissed with moderator audit notes.

4. **In-Game Chat Logs & Command Audit Trail**:
   - **`/mod chatlogs [player] [channel] [limit]`**: View in-game chat logs across channels (Say, Tell/Whisper, Shout, Guild) with player and channel filters.
   - **`/mod commandlogs [actor] [command] [limit]`**: Inspect audit records of all staff and player command executions (both Discord bot commands and in-game commands).

5. **Character Management & Inappropriate Names**:
   - **`/mod rename <player> <new_name> [reason]`**: Force-renames offensive or rule-breaking character names directly in the database with full audit logging.
   - **`/mod lookup <player>`**: Displays full account overview, all characters owned by the account, live zone/online status, membership, roles, active punishments, coordinates, and recent moderation history.
   - **`/mod history <player>`**: Audit log of all moderation actions taken against a player.

6. **In-Game Administration & Player Relocation**:
   - **`/admin promote <player>`** & **`/admin demote <player>`**: Promote or demote in-game moderators.
   - **`/admin broadcast <message>`**: Transmit an instant server-wide announcement to all zones.
   - **`/admin motd <title> <message>`**: Set and broadcast the in-game login Message of the Day popup (`PacketMOTD`, OpCode 87).
   - **`/admin teleport <player> <zone>`**: Warp a player to any active zone (`PacketZoneTeleportRequest`, OpCode 90).
   - **`/admin move <player> <x> <y> <z> [rotation]`**: Move a player's coordinates in-game in real time (`ClientUpdatePacketUpdateLocation`, OpCode 12, Teleport = true) and persist the new position to the database (`Characters` table `PositionX`, `PositionY`, `PositionZ`, `RotationX`, `RotationZ`).
   - **`/admin coords <player>`** & **`/mod coords <player>`**: Inspect a player's current coordinates, rotation, zone, and online status.

7. **Server Maintenance & Shutdown Operations**:
   - **`/server shutdown <countdown> [reason]`**: Triggers an in-game shutdown countdown with `PacketWorldShutdownNotice` (OpCode 92). Displays the native client countdown timer overlay and periodic chat reminders. At 0 seconds, disconnects all players.
   - **`/server cancel_shutdown`**: Cancels an active countdown and broadcasts an all-clear notice to all online players.
   - **`/server maintenance <on|off> [reason]`**: Toggles maintenance mode. When enabled, locks the login server (`LoginReply` Status 2) so non-admin logins are rejected with the "Server Locked" message, and broadcasts an in-game maintenance notice.

8. **Sustainable Production Architecture**:
   - **Domain Error Hierarchy**: Strongly typed `BotError` taxonomy powered by `thiserror`, eliminating untracked panics and providing structured error reporting across Discord and log traces.
   - **Connection Lifecycle Management**: Configurable pool boundaries (`max_connections`, `min_connections`, `acquire_timeout_seconds`, `idle_timeout_seconds`).
   - **SQLite Concurrency Optimization**: Enforces Write-Ahead Logging (`WAL`), `busy_timeout` (5s), and foreign keys pragma to prevent lock contention (`SQLITE_BUSY`).
   - **Health Checks**: Real-time `/server status` includes live connectivity ping tests against both the Gateway HTTP API and the database pool.
   - **Graceful Shutdown**: Listens for termination signals (SIGINT / SIGTERM) to cleanly disconnect Serenity gateway shards and drain connection pools before process termination.
   - **Automated Test Suites**: Built-in unit tests covering duration parsing, warning points escalation tiers, coordinate validation, and configuration deserialization.

9. **Security, Privacy & Guild Isolation**:
   - **Ephemeral Player Data**: All moderation lookups, private chat logs, command audit trails, coordinates, and infraction history respond ephemerally to the invoking staff member. Sensitive player data is never leaked into public Discord channels.
   - **Dedicated Audit Channel**: Permanent moderation and admin records are dispatched directly to your private staff audit channel (`audit_log_channel_id`).
   - **Guild-Gated Execution**: Commands enforce `guild_only`, Discord native `default_member_permissions` (`ADMINISTRATOR` / `MODERATE_MEMBERS`), and guild authorization checks. Unapproved servers and direct messages are strictly blocked.
   - **Safe Permanent Bans**: Permanent bans safely persist the indefinite timestamp (`9999-12-31T23:59:59Z`, matching Sanctuary C# `DateTimeOffset.MaxValue`), ensuring permanent locking on both SQLite and MySQL while preventing database overflow.

---

## Setup & Installation

### 1. Requirements
- Rust Toolchain: 1.75+ (install via https://rustup.rs/)
- Sanctuary Server: Sanctuary.Gateway running with the Admin API enabled.
- Discord Bot Token: Created in the Discord Developer Portal (https://discord.com/developers/applications).

### 2. Configuration
Copy `config.toml.example` to `config.toml` (or set environment variables in `.env`):

#### Using SQLite:
```toml
[discord]
token = "YOUR_DISCORD_BOT_TOKEN"
moderator_role_names = ["Moderator", "Staff"]
admin_role_names = ["Administrator", "Admin"]
presence_interval_seconds = 30

[database]
provider = "sqlite"
path = "../Sanctuary-upstream/src/Sanctuary.Database.Sqlite/sanctuary.db"
max_connections = 10
min_connections = 1
acquire_timeout_seconds = 10
idle_timeout_seconds = 600

[gateway]
api_url = "http://127.0.0.1:5000"
api_key = "sanctuary_admin_secret_key"
timeout_seconds = 5
connect_timeout_seconds = 5
```

#### Using MySQL / MariaDB:
```toml
[discord]
token = "YOUR_DISCORD_BOT_TOKEN"
moderator_role_names = ["Moderator", "Staff"]
admin_role_names = ["Administrator", "Admin"]
presence_interval_seconds = 30

[database]
provider = "mysql"
url = "mysql://sanctuary_user:password@127.0.0.1:3306/sanctuary"
max_connections = 25
min_connections = 5
acquire_timeout_seconds = 15
idle_timeout_seconds = 300

[gateway]
api_url = "http://127.0.0.1:5000"
api_key = "sanctuary_admin_secret_key"
timeout_seconds = 5
connect_timeout_seconds = 5
```

### 3. Build & Run
```bash
# Run automated tests
cargo test

# Build release binary
cargo build --release

# Run bot
cargo run --release
```
BASED ON:
https://github.com/Open-Source-Free-Realms/Chatty
