use super::models::{DbCharacter, DbChatLog, DbCommandLog, DbModerationLog, DbPlayerReport, DbUser, DbWarning};
use super::Database;
use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::mysql::MySqlRow;
use sqlx::sqlite::SqliteRow;
use sqlx::Row;

fn parse_dt(s: Option<String>) -> Option<DateTime<Utc>> {
    s.and_then(|str_val| {
        DateTime::parse_from_rfc3339(&str_val)
            .map(|dt| dt.with_timezone(&Utc))
            .or_else(|_| {
                chrono::NaiveDateTime::parse_from_str(&str_val, "%Y-%m-%d %H:%M:%S")
                    .map(|ndt| DateTime::from_naive_utc_and_offset(ndt, Utc))
            })
            .ok()
    })
}

fn format_dt(dt: Option<DateTime<Utc>>) -> Option<String> {
    dt.map(|d| d.to_rfc3339())
}

fn sql_bool(r: &SqliteRow, idx: usize) -> bool {
    r.try_get::<bool, _>(idx).unwrap_or_else(|_| r.try_get::<i64, _>(idx).map(|v| v != 0).unwrap_or(false))
}
fn sql_opt_f32(r: &SqliteRow, idx: usize) -> Option<f32> {
    r.try_get::<f64, _>(idx).ok().map(|v| v as f32)
}
fn sql_dt(r: &SqliteRow, idx: usize) -> Option<DateTime<Utc>> {
    parse_dt(r.try_get(idx).ok())
}

fn map_sqlite_user(r: &SqliteRow, o: usize) -> Result<DbUser> {
    Ok(DbUser {
        id: r.try_get::<i64, _>(o)? as u64,
        username: r.try_get(o + 1)?,
        password: r.try_get(o + 2)?,
        session: r.try_get(o + 3)?,
        max_characters: r.try_get(o + 4)?,
        is_member: sql_bool(r, o + 5),
        is_admin: sql_bool(r, o + 6),
        is_mod: sql_bool(r, o + 7),
        locked_until: sql_dt(r, o + 8),
        muted_until: sql_dt(r, o + 9),
        created: sql_dt(r, o + 10).unwrap_or_else(Utc::now),
        last_login: sql_dt(r, o + 11),
    })
}

fn map_mysql_user(r: &MySqlRow, o: usize) -> Result<DbUser> {
    Ok(DbUser {
        id: r.try_get(o)?,
        username: r.try_get(o + 1)?,
        password: r.try_get(o + 2)?,
        session: r.try_get(o + 3)?,
        max_characters: r.try_get(o + 4)?,
        is_member: r.try_get(o + 5)?,
        is_admin: r.try_get(o + 6)?,
        is_mod: r.try_get(o + 7)?,
        locked_until: r.try_get(o + 8)?,
        muted_until: r.try_get(o + 9)?,
        created: r.try_get(o + 10)?,
        last_login: r.try_get(o + 11)?,
    })
}

fn map_sqlite_character(r: &SqliteRow, o: usize) -> Result<DbCharacter> {
    Ok(DbCharacter {
        id: r.try_get::<i64, _>(o)? as u64,
        user_id: r.try_get::<i64, _>(o + 1)? as u64,
        first_name: r.try_get(o + 2)?,
        last_name: r.try_get(o + 3)?,
        full_name: r.try_get(o + 4)?,
        coins: r.try_get(o + 5)?,
        station_cash: r.try_get(o + 6)?,
        play_time: r.try_get(o + 7)?,
        created: sql_dt(r, o + 8).unwrap_or_else(Utc::now),
        last_login: sql_dt(r, o + 9),
        position_x: sql_opt_f32(r, o + 10),
        position_y: sql_opt_f32(r, o + 11),
        position_z: sql_opt_f32(r, o + 12),
        rotation_x: sql_opt_f32(r, o + 13),
        rotation_z: sql_opt_f32(r, o + 14),
    })
}

fn map_mysql_character(r: &MySqlRow, o: usize) -> Result<DbCharacter> {
    Ok(DbCharacter {
        id: r.try_get(o)?,
        user_id: r.try_get(o + 1)?,
        first_name: r.try_get(o + 2)?,
        last_name: r.try_get(o + 3)?,
        full_name: r.try_get(o + 4)?,
        coins: r.try_get(o + 5)?,
        station_cash: r.try_get(o + 6)?,
        play_time: r.try_get(o + 7)?,
        created: r.try_get(o + 8)?,
        last_login: r.try_get(o + 9)?,
        position_x: r.try_get(o + 10)?,
        position_y: r.try_get(o + 11)?,
        position_z: r.try_get(o + 12)?,
        rotation_x: r.try_get(o + 13)?,
        rotation_z: r.try_get(o + 14)?,
    })
}

fn map_sqlite_mod_log(r: &SqliteRow) -> Result<DbModerationLog> {
    Ok(DbModerationLog {
        id: r.try_get(0)?,
        target_user_id: r.try_get::<i64, _>(1)? as u64,
        target_name: r.try_get(2)?,
        actor_user_id: r.try_get::<Option<i64>, _>(3)?.map(|v| v as u64),
        actor_name: r.try_get(4)?,
        actor_source: r.try_get(5)?,
        action: r.try_get(6)?,
        reason: r.try_get(7)?,
        duration: r.try_get(8)?,
        created_at: sql_dt(r, 9).unwrap_or_else(Utc::now),
    })
}

fn map_mysql_mod_log(r: &MySqlRow) -> Result<DbModerationLog> {
    Ok(DbModerationLog {
        id: r.try_get(0)?,
        target_user_id: r.try_get(1)?,
        target_name: r.try_get(2)?,
        actor_user_id: r.try_get(3)?,
        actor_name: r.try_get(4)?,
        actor_source: r.try_get(5)?,
        action: r.try_get(6)?,
        reason: r.try_get(7)?,
        duration: r.try_get(8)?,
        created_at: r.try_get(9)?,
    })
}

fn map_sqlite_warning(r: &SqliteRow) -> Result<DbWarning> {
    Ok(DbWarning {
        id: r.try_get(0)?,
        target_user_id: r.try_get::<i64, _>(1)? as u64,
        target_name: r.try_get(2)?,
        reason: r.try_get(3)?,
        issued_by: r.try_get(4)?,
        issued_by_source: r.try_get(5)?,
        severity: r.try_get(6)?,
        acknowledged: sql_bool(r, 7),
        active: sql_bool(r, 8),
        created_at: sql_dt(r, 9).unwrap_or_else(Utc::now),
        expires_at: sql_dt(r, 10),
    })
}

fn map_mysql_warning(r: &MySqlRow) -> Result<DbWarning> {
    Ok(DbWarning {
        id: r.try_get(0)?,
        target_user_id: r.try_get(1)?,
        target_name: r.try_get(2)?,
        reason: r.try_get(3)?,
        issued_by: r.try_get(4)?,
        issued_by_source: r.try_get(5)?,
        severity: r.try_get(6)?,
        acknowledged: r.try_get(7)?,
        active: r.try_get(8)?,
        created_at: r.try_get(9)?,
        expires_at: r.try_get(10)?,
    })
}

fn map_sqlite_report(r: &SqliteRow) -> Result<DbPlayerReport> {
    Ok(DbPlayerReport {
        id: r.try_get(0)?,
        target_user_id: r.try_get::<i64, _>(1)? as u64,
        target_name: r.try_get(2)?,
        reporter_user_id: r.try_get::<Option<i64>, _>(3)?.map(|v| v as u64),
        reporter_name: r.try_get(4)?,
        reporter_source: r.try_get(5)?,
        reason: r.try_get(6)?,
        description: r.try_get(7)?,
        zone_name: r.try_get(8)?,
        status: r.try_get(9)?,
        created_at: sql_dt(r, 10).unwrap_or_else(Utc::now),
        resolved_at: sql_dt(r, 11),
        resolved_by: r.try_get(12)?,
        resolved_notes: r.try_get(13)?,
    })
}

fn map_mysql_report(r: &MySqlRow) -> Result<DbPlayerReport> {
    Ok(DbPlayerReport {
        id: r.try_get(0)?,
        target_user_id: r.try_get(1)?,
        target_name: r.try_get(2)?,
        reporter_user_id: r.try_get(3)?,
        reporter_name: r.try_get(4)?,
        reporter_source: r.try_get(5)?,
        reason: r.try_get(6)?,
        description: r.try_get(7)?,
        zone_name: r.try_get(8)?,
        status: r.try_get(9)?,
        created_at: r.try_get(10)?,
        resolved_at: r.try_get(11)?,
        resolved_by: r.try_get(12)?,
        resolved_notes: r.try_get(13)?,
    })
}

fn map_sqlite_chat(r: &SqliteRow) -> Result<DbChatLog> {
    Ok(DbChatLog {
        id: r.try_get(0)?,
        sender_user_id: r.try_get::<Option<i64>, _>(1)?.map(|v| v as u64),
        sender_name: r.try_get(2)?,
        channel: r.try_get(3)?,
        recipient_name: r.try_get(4)?,
        message: r.try_get(5)?,
        zone_name: r.try_get(6)?,
        created_at: sql_dt(r, 7).unwrap_or_else(Utc::now),
    })
}

fn map_mysql_chat(r: &MySqlRow) -> Result<DbChatLog> {
    Ok(DbChatLog {
        id: r.try_get(0)?,
        sender_user_id: r.try_get(1)?,
        sender_name: r.try_get(2)?,
        channel: r.try_get(3)?,
        recipient_name: r.try_get(4)?,
        message: r.try_get(5)?,
        zone_name: r.try_get(6)?,
        created_at: r.try_get(7)?,
    })
}

fn map_sqlite_cmd(r: &SqliteRow) -> Result<DbCommandLog> {
    Ok(DbCommandLog {
        id: r.try_get(0)?,
        actor_user_id: r.try_get::<Option<i64>, _>(1)?.map(|v| v as u64),
        actor_name: r.try_get(2)?,
        actor_source: r.try_get(3)?,
        command: r.try_get(4)?,
        arguments: r.try_get(5)?,
        target: r.try_get(6)?,
        success: sql_bool(r, 7),
        created_at: sql_dt(r, 8).unwrap_or_else(Utc::now),
    })
}

fn map_mysql_cmd(r: &MySqlRow) -> Result<DbCommandLog> {
    Ok(DbCommandLog {
        id: r.try_get(0)?,
        actor_user_id: r.try_get(1)?,
        actor_name: r.try_get(2)?,
        actor_source: r.try_get(3)?,
        command: r.try_get(4)?,
        arguments: r.try_get(5)?,
        target: r.try_get(6)?,
        success: r.try_get(7)?,
        created_at: r.try_get(8)?,
    })
}

impl Database {
    pub async fn find_user_and_character_by_name(
        &self,
        character_name: &str,
    ) -> Result<Option<(DbUser, DbCharacter)>> {
        let sql = r#"
            SELECT 
                u.Id, u.Username, u.Password, u.Session, u.MaxCharacters,
                u.IsMember, u.IsAdmin, u.IsMod, u.LockedUntil, u.MutedUntil,
                u.Created, u.LastLogin,
                c.Id, c.UserId, c.FirstName, c.LastName, c.FullName,
                c.Coins, c.StationCash, c.PlayTime, c.Created, c.LastLogin,
                c.PositionX, c.PositionY, c.PositionZ, c.RotationX, c.RotationZ
            FROM Characters c
            JOIN Users u ON u.Id = c.UserId
            WHERE LOWER(c.FullName) = LOWER(?) OR LOWER(c.FirstName) = LOWER(?)
            LIMIT 1
        "#;

        match self {
            Database::Sqlite(pool) => {
                let row = sqlx::query(sql)
                    .bind(character_name)
                    .bind(character_name)
                    .fetch_optional(pool)
                    .await?;
                match row {
                    Some(r) => Ok(Some((map_sqlite_user(&r, 0)?, map_sqlite_character(&r, 12)?))),
                    None => Ok(None),
                }
            }
            Database::MySql(pool) => {
                let row = sqlx::query(sql)
                    .bind(character_name)
                    .bind(character_name)
                    .fetch_optional(pool)
                    .await?;
                match row {
                    Some(r) => Ok(Some((map_mysql_user(&r, 0)?, map_mysql_character(&r, 12)?))),
                    None => Ok(None),
                }
            }
        }
    }

    pub async fn get_user_characters(&self, user_id: u64) -> Result<Vec<DbCharacter>> {
        let sql = r#"
            SELECT Id, UserId, FirstName, LastName, FullName, Coins, StationCash, PlayTime,
                   Created, LastLogin, PositionX, PositionY, PositionZ, RotationX, RotationZ
            FROM Characters WHERE UserId = ? ORDER BY Created ASC
        "#;
        match self {
            Database::Sqlite(p) => {
                let rows = sqlx::query(sql).bind(user_id as i64).fetch_all(p).await?;
                rows.iter().map(|r| map_sqlite_character(r, 0)).collect()
            }
            Database::MySql(p) => {
                let rows = sqlx::query(sql).bind(user_id).fetch_all(p).await?;
                rows.iter().map(|r| map_mysql_character(r, 0)).collect()
            }
        }
    }

    pub async fn set_user_locked_until(&self, user_id: u64, locked_until: Option<DateTime<Utc>>) -> Result<()> {
        let sql = "UPDATE Users SET LockedUntil = ? WHERE Id = ?";
        match self {
            Database::Sqlite(p) => { sqlx::query(sql).bind(format_dt(locked_until)).bind(user_id as i64).execute(p).await?; }
            Database::MySql(p) => { sqlx::query(sql).bind(locked_until).bind(user_id).execute(p).await?; }
        }
        Ok(())
    }

    pub async fn set_user_muted_until(&self, user_id: u64, muted_until: Option<DateTime<Utc>>) -> Result<()> {
        let sql = "UPDATE Users SET MutedUntil = ? WHERE Id = ?";
        match self {
            Database::Sqlite(p) => { sqlx::query(sql).bind(format_dt(muted_until)).bind(user_id as i64).execute(p).await?; }
            Database::MySql(p) => { sqlx::query(sql).bind(muted_until).bind(user_id).execute(p).await?; }
        }
        Ok(())
    }

    pub async fn set_user_is_mod(&self, user_id: u64, is_mod: bool) -> Result<bool> {
        let sql = "UPDATE Users SET IsMod = ? WHERE Id = ?";
        let aff = match self {
            Database::Sqlite(p) => sqlx::query(sql).bind(if is_mod { 1i64 } else { 0i64 }).bind(user_id as i64).execute(p).await?.rows_affected(),
            Database::MySql(p) => sqlx::query(sql).bind(is_mod).bind(user_id).execute(p).await?.rows_affected(),
        };
        Ok(aff > 0)
    }

    pub async fn log_moderation_action(&self, log: &DbModerationLog) -> Result<i64> {
        let sql = r#"
            INSERT INTO ModerationLogs (TargetUserId, TargetName, ActorUserId, ActorName, ActorSource, Action, Reason, Duration, CreatedAt)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#;
        match self {
            Database::Sqlite(p) => {
                let res = sqlx::query(sql)
                    .bind(log.target_user_id as i64).bind(&log.target_name)
                    .bind(log.actor_user_id.map(|v| v as i64)).bind(&log.actor_name)
                    .bind(&log.actor_source).bind(&log.action).bind(&log.reason)
                    .bind(&log.duration).bind(format_dt(Some(log.created_at)))
                    .execute(p).await?;
                Ok(res.last_insert_rowid())
            }
            Database::MySql(p) => {
                let res = sqlx::query(sql)
                    .bind(log.target_user_id).bind(&log.target_name)
                    .bind(log.actor_user_id).bind(&log.actor_name)
                    .bind(&log.actor_source).bind(&log.action).bind(&log.reason)
                    .bind(&log.duration).bind(log.created_at)
                    .execute(p).await?;
                Ok(res.last_insert_id() as i64)
            }
        }
    }

    pub async fn get_moderation_logs_for_target(&self, target_user_id: u64, limit: i64) -> Result<Vec<DbModerationLog>> {
        let sql = "SELECT Id, TargetUserId, TargetName, ActorUserId, ActorName, ActorSource, Action, Reason, Duration, CreatedAt FROM ModerationLogs WHERE TargetUserId = ? ORDER BY CreatedAt DESC LIMIT ?";
        match self {
            Database::Sqlite(p) => {
                let rows = sqlx::query(sql).bind(target_user_id as i64).bind(limit).fetch_all(p).await?;
                rows.iter().map(map_sqlite_mod_log).collect()
            }
            Database::MySql(p) => {
                let rows = sqlx::query(sql).bind(target_user_id).bind(limit).fetch_all(p).await?;
                rows.iter().map(map_mysql_mod_log).collect()
            }
        }
    }

    pub async fn create_warning(&self, warning: &DbWarning) -> Result<i64> {
        let sql = r#"
            INSERT INTO Warnings (TargetUserId, TargetName, Reason, IssuedBy, IssuedBySource, Severity, Acknowledged, Active, CreatedAt, ExpiresAt)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#;
        match self {
            Database::Sqlite(p) => {
                let res = sqlx::query(sql)
                    .bind(warning.target_user_id as i64).bind(&warning.target_name)
                    .bind(&warning.reason).bind(&warning.issued_by).bind(&warning.issued_by_source)
                    .bind(warning.severity).bind(if warning.acknowledged { 1i64 } else { 0i64 })
                    .bind(if warning.active { 1i64 } else { 0i64 })
                    .bind(format_dt(Some(warning.created_at))).bind(format_dt(warning.expires_at))
                    .execute(p).await?;
                Ok(res.last_insert_rowid())
            }
            Database::MySql(p) => {
                let res = sqlx::query(sql)
                    .bind(warning.target_user_id).bind(&warning.target_name)
                    .bind(&warning.reason).bind(&warning.issued_by).bind(&warning.issued_by_source)
                    .bind(warning.severity).bind(warning.acknowledged).bind(warning.active)
                    .bind(warning.created_at).bind(warning.expires_at)
                    .execute(p).await?;
                Ok(res.last_insert_id() as i64)
            }
        }
    }

    pub async fn get_active_warnings_for_user(&self, user_id: u64) -> Result<Vec<DbWarning>> {
        let sql = "SELECT Id, TargetUserId, TargetName, Reason, IssuedBy, IssuedBySource, Severity, Acknowledged, Active, CreatedAt, ExpiresAt FROM Warnings WHERE TargetUserId = ? AND Active = 1 ORDER BY CreatedAt DESC";
        match self {
            Database::Sqlite(p) => {
                let rows = sqlx::query(sql).bind(user_id as i64).fetch_all(p).await?;
                rows.iter().map(map_sqlite_warning).collect()
            }
            Database::MySql(p) => {
                let rows = sqlx::query(sql).bind(user_id).fetch_all(p).await?;
                rows.iter().map(map_mysql_warning).collect()
            }
        }
    }

    pub async fn get_active_warning_points(&self, user_id: u64) -> Result<i32> {
        let sql = "SELECT COALESCE(SUM(Severity), 0) FROM Warnings WHERE TargetUserId = ? AND Active = 1";
        match self {
            Database::Sqlite(p) => {
                let row = sqlx::query(sql).bind(user_id as i64).fetch_one(p).await?;
                Ok(row.try_get::<i64, _>(0).unwrap_or(0) as i32)
            }
            Database::MySql(p) => {
                let row = sqlx::query(sql).bind(user_id).fetch_one(p).await?;
                Ok(row.try_get::<i64, _>(0).unwrap_or(0) as i32)
            }
        }
    }

    pub async fn clear_warning_by_id(&self, warning_id: i64) -> Result<bool> {
        let sql = "UPDATE Warnings SET Active = 0 WHERE Id = ?";
        let aff = match self {
            Database::Sqlite(p) => sqlx::query(sql).bind(warning_id).execute(p).await?.rows_affected(),
            Database::MySql(p) => sqlx::query(sql).bind(warning_id).execute(p).await?.rows_affected(),
        };
        Ok(aff > 0)
    }

    pub async fn create_player_report(&self, r: &DbPlayerReport) -> Result<i64> {
        let sql = r#"
            INSERT INTO PlayerReports (TargetUserId, TargetName, ReporterUserId, ReporterName, ReporterSource, Reason, Description, ZoneName, Status, CreatedAt, ResolvedAt, ResolvedBy, ResolvedNotes)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#;
        match self {
            Database::Sqlite(p) => {
                let res = sqlx::query(sql)
                    .bind(r.target_user_id as i64).bind(&r.target_name)
                    .bind(r.reporter_user_id.map(|v| v as i64)).bind(&r.reporter_name)
                    .bind(&r.reporter_source).bind(&r.reason).bind(&r.description)
                    .bind(&r.zone_name).bind(&r.status).bind(format_dt(Some(r.created_at)))
                    .bind(format_dt(r.resolved_at)).bind(&r.resolved_by).bind(&r.resolved_notes)
                    .execute(p).await?;
                Ok(res.last_insert_rowid())
            }
            Database::MySql(p) => {
                let res = sqlx::query(sql)
                    .bind(r.target_user_id).bind(&r.target_name)
                    .bind(r.reporter_user_id).bind(&r.reporter_name)
                    .bind(&r.reporter_source).bind(&r.reason).bind(&r.description)
                    .bind(&r.zone_name).bind(&r.status).bind(r.created_at)
                    .bind(r.resolved_at).bind(&r.resolved_by).bind(&r.resolved_notes)
                    .execute(p).await?;
                Ok(res.last_insert_id() as i64)
            }
        }
    }

    pub async fn get_player_reports(&self, target_player: Option<&str>, status: Option<&str>, limit: i64) -> Result<Vec<DbPlayerReport>> {
        let mut query = String::from("SELECT Id, TargetUserId, TargetName, ReporterUserId, ReporterName, ReporterSource, Reason, Description, ZoneName, Status, CreatedAt, ResolvedAt, ResolvedBy, ResolvedNotes FROM PlayerReports WHERE 1=1");
        if target_player.is_some() { query.push_str(" AND LOWER(TargetName) = LOWER(?)"); }
        if status.is_some() { query.push_str(" AND LOWER(Status) = LOWER(?)"); }
        query.push_str(" ORDER BY CreatedAt DESC LIMIT ?");

        match self {
            Database::Sqlite(p) => {
                let mut q = sqlx::query(&query);
                if let Some(t) = target_player { q = q.bind(t); }
                if let Some(s) = status { q = q.bind(s); }
                let rows = q.bind(limit).fetch_all(p).await?;
                rows.iter().map(map_sqlite_report).collect()
            }
            Database::MySql(p) => {
                let mut q = sqlx::query(&query);
                if let Some(t) = target_player { q = q.bind(t); }
                if let Some(s) = status { q = q.bind(s); }
                let rows = q.bind(limit).fetch_all(p).await?;
                rows.iter().map(map_mysql_report).collect()
            }
        }
    }

    pub async fn resolve_player_report(&self, report_id: i64, status: &str, resolved_by: &str, notes: Option<&str>) -> Result<bool> {
        let sql = "UPDATE PlayerReports SET Status = ?, ResolvedAt = ?, ResolvedBy = ?, ResolvedNotes = ? WHERE Id = ?";
        let now = Utc::now();
        let aff = match self {
            Database::Sqlite(p) => sqlx::query(sql).bind(status).bind(format_dt(Some(now))).bind(resolved_by).bind(notes).bind(report_id).execute(p).await?.rows_affected(),
            Database::MySql(p) => sqlx::query(sql).bind(status).bind(now).bind(resolved_by).bind(notes).bind(report_id).execute(p).await?.rows_affected(),
        };
        Ok(aff > 0)
    }

    pub async fn get_pending_report_count_for_player(&self, target_name: &str) -> Result<i32> {
        let sql = "SELECT COUNT(*) FROM PlayerReports WHERE LOWER(TargetName) = LOWER(?) AND LOWER(Status) = 'pending'";
        let count = match self {
            Database::Sqlite(p) => sqlx::query(sql).bind(target_name).fetch_one(p).await?.try_get::<i64, _>(0).unwrap_or(0),
            Database::MySql(p) => sqlx::query(sql).bind(target_name).fetch_one(p).await?.try_get::<i64, _>(0).unwrap_or(0),
        };
        Ok(count as i32)
    }

    pub async fn rename_player_character(&self, old_name: &str, new_name: &str) -> Result<bool> {
        let sql = "UPDATE Characters SET FullName = ?, FirstName = ? WHERE LOWER(FullName) = LOWER(?)";
        let aff = match self {
            Database::Sqlite(p) => sqlx::query(sql).bind(new_name).bind(new_name).bind(old_name).execute(p).await?.rows_affected(),
            Database::MySql(p) => sqlx::query(sql).bind(new_name).bind(new_name).bind(old_name).execute(p).await?.rows_affected(),
        };
        Ok(aff > 0)
    }

    pub async fn update_character_coordinates(&self, character_id: u64, x: f32, y: f32, z: f32, rot_x: Option<f32>, rot_z: Option<f32>) -> Result<bool> {
        let sql = "UPDATE Characters SET PositionX = ?, PositionY = ?, PositionZ = ?, RotationX = ?, RotationZ = ? WHERE Id = ?";
        let aff = match self {
            Database::Sqlite(p) => sqlx::query(sql).bind(x as f64).bind(y as f64).bind(z as f64).bind(rot_x.map(|v| v as f64)).bind(rot_z.map(|v| v as f64)).bind(character_id as i64).execute(p).await?.rows_affected(),
            Database::MySql(p) => sqlx::query(sql).bind(x).bind(y).bind(z).bind(rot_x).bind(rot_z).bind(character_id).execute(p).await?.rows_affected(),
        };
        Ok(aff > 0)
    }

    pub async fn update_character_coordinates_by_name(&self, player_name: &str, x: f32, y: f32, z: f32, rot_x: Option<f32>, rot_z: Option<f32>) -> Result<bool> {
        let sql = "UPDATE Characters SET PositionX = ?, PositionY = ?, PositionZ = ?, RotationX = ?, RotationZ = ? WHERE LOWER(FullName) = LOWER(?) OR LOWER(FirstName) = LOWER(?)";
        let aff = match self {
            Database::Sqlite(p) => sqlx::query(sql).bind(x as f64).bind(y as f64).bind(z as f64).bind(rot_x.map(|v| v as f64)).bind(rot_z.map(|v| v as f64)).bind(player_name).bind(player_name).execute(p).await?.rows_affected(),
            Database::MySql(p) => sqlx::query(sql).bind(x).bind(y).bind(z).bind(rot_x).bind(rot_z).bind(player_name).bind(player_name).execute(p).await?.rows_affected(),
        };
        Ok(aff > 0)
    }

    pub async fn log_chat_message(&self, log: &DbChatLog) -> Result<i64> {
        let sql = "INSERT INTO ChatLogs (SenderUserId, SenderName, Channel, RecipientName, Message, ZoneName, CreatedAt) VALUES (?, ?, ?, ?, ?, ?, ?)";
        match self {
            Database::Sqlite(p) => {
                let res = sqlx::query(sql).bind(log.sender_user_id.map(|v| v as i64)).bind(&log.sender_name).bind(&log.channel).bind(&log.recipient_name).bind(&log.message).bind(&log.zone_name).bind(format_dt(Some(log.created_at))).execute(p).await?;
                Ok(res.last_insert_rowid())
            }
            Database::MySql(p) => {
                let res = sqlx::query(sql).bind(log.sender_user_id).bind(&log.sender_name).bind(&log.channel).bind(&log.recipient_name).bind(&log.message).bind(&log.zone_name).bind(log.created_at).execute(p).await?;
                Ok(res.last_insert_id() as i64)
            }
        }
    }

    pub async fn get_chat_logs(&self, sender_name: Option<&str>, channel: Option<&str>, limit: i64) -> Result<Vec<DbChatLog>> {
        let mut query = String::from("SELECT Id, SenderUserId, SenderName, Channel, RecipientName, Message, ZoneName, CreatedAt FROM ChatLogs WHERE 1=1");
        if sender_name.is_some() { query.push_str(" AND LOWER(SenderName) = LOWER(?)"); }
        if channel.is_some() { query.push_str(" AND LOWER(Channel) = LOWER(?)"); }
        query.push_str(" ORDER BY CreatedAt DESC LIMIT ?");

        match self {
            Database::Sqlite(p) => {
                let mut q = sqlx::query(&query);
                if let Some(s) = sender_name { q = q.bind(s); }
                if let Some(c) = channel { q = q.bind(c); }
                let rows = q.bind(limit).fetch_all(p).await?;
                rows.iter().map(map_sqlite_chat).collect()
            }
            Database::MySql(p) => {
                let mut q = sqlx::query(&query);
                if let Some(s) = sender_name { q = q.bind(s); }
                if let Some(c) = channel { q = q.bind(c); }
                let rows = q.bind(limit).fetch_all(p).await?;
                rows.iter().map(map_mysql_chat).collect()
            }
        }
    }

    pub async fn log_command_execution(&self, cmd: &DbCommandLog) -> Result<i64> {
        let sql = "INSERT INTO CommandLogs (ActorUserId, ActorName, ActorSource, Command, Arguments, Target, Success, CreatedAt) VALUES (?, ?, ?, ?, ?, ?, ?, ?)";
        match self {
            Database::Sqlite(p) => {
                let res = sqlx::query(sql).bind(cmd.actor_user_id.map(|v| v as i64)).bind(&cmd.actor_name).bind(&cmd.actor_source).bind(&cmd.command).bind(&cmd.arguments).bind(&cmd.target).bind(if cmd.success { 1i64 } else { 0i64 }).bind(format_dt(Some(cmd.created_at))).execute(p).await?;
                Ok(res.last_insert_rowid())
            }
            Database::MySql(p) => {
                let res = sqlx::query(sql).bind(cmd.actor_user_id).bind(&cmd.actor_name).bind(&cmd.actor_source).bind(&cmd.command).bind(&cmd.arguments).bind(&cmd.target).bind(cmd.success).bind(cmd.created_at).execute(p).await?;
                Ok(res.last_insert_id() as i64)
            }
        }
    }

    pub async fn get_command_logs(&self, actor_name: Option<&str>, command: Option<&str>, limit: i64) -> Result<Vec<DbCommandLog>> {
        let mut query = String::from("SELECT Id, ActorUserId, ActorName, ActorSource, Command, Arguments, Target, Success, CreatedAt FROM CommandLogs WHERE 1=1");
        if actor_name.is_some() { query.push_str(" AND LOWER(ActorName) = LOWER(?)"); }
        if command.is_some() { query.push_str(" AND LOWER(Command) = LOWER(?)"); }
        query.push_str(" ORDER BY CreatedAt DESC LIMIT ?");

        match self {
            Database::Sqlite(p) => {
                let mut q = sqlx::query(&query);
                if let Some(a) = actor_name { q = q.bind(a); }
                if let Some(c) = command { q = q.bind(c); }
                let rows = q.bind(limit).fetch_all(p).await?;
                rows.iter().map(map_sqlite_cmd).collect()
            }
            Database::MySql(p) => {
                let mut q = sqlx::query(&query);
                if let Some(a) = actor_name { q = q.bind(a); }
                if let Some(c) = command { q = q.bind(c); }
                let rows = q.bind(limit).fetch_all(p).await?;
                rows.iter().map(map_mysql_cmd).collect()
            }
        }
    }
}
