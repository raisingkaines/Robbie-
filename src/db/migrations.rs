use anyhow::Result;
use sqlx::{MySqlPool, SqlitePool};
use tracing::info;

pub async fn run_sqlite_migrations(pool: &SqlitePool) -> Result<()> {
    info!("Running SQLite moderation migrations...");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS ModerationLogs (
            Id INTEGER PRIMARY KEY AUTOINCREMENT,
            TargetUserId INTEGER NOT NULL,
            TargetName TEXT NOT NULL,
            ActorUserId INTEGER,
            ActorName TEXT NOT NULL,
            ActorSource TEXT NOT NULL DEFAULT 'Discord',
            Action TEXT NOT NULL,
            Reason TEXT,
            Duration TEXT,
            CreatedAt TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE INDEX IF NOT EXISTS IX_ModerationLogs_TargetUserId ON ModerationLogs(TargetUserId);
        CREATE INDEX IF NOT EXISTS IX_ModerationLogs_TargetName ON ModerationLogs(TargetName);

        CREATE TABLE IF NOT EXISTS Warnings (
            Id INTEGER PRIMARY KEY AUTOINCREMENT,
            TargetUserId INTEGER NOT NULL,
            TargetName TEXT NOT NULL,
            Reason TEXT NOT NULL,
            IssuedBy TEXT NOT NULL,
            IssuedBySource TEXT NOT NULL DEFAULT 'Discord',
            Severity INTEGER NOT NULL DEFAULT 1,
            Acknowledged INTEGER NOT NULL DEFAULT 0,
            Active INTEGER NOT NULL DEFAULT 1,
            CreatedAt TEXT NOT NULL DEFAULT (datetime('now')),
            ExpiresAt TEXT
        );

        CREATE INDEX IF NOT EXISTS IX_Warnings_TargetUserId ON Warnings(TargetUserId);
        CREATE INDEX IF NOT EXISTS IX_Warnings_TargetName ON Warnings(TargetName);
        CREATE INDEX IF NOT EXISTS IX_Warnings_Active ON Warnings(Active);

        CREATE TABLE IF NOT EXISTS PlayerReports (
            Id INTEGER PRIMARY KEY AUTOINCREMENT,
            TargetUserId INTEGER NOT NULL,
            TargetName TEXT NOT NULL,
            ReporterUserId INTEGER,
            ReporterName TEXT NOT NULL,
            ReporterSource TEXT NOT NULL DEFAULT 'InGame',
            Reason TEXT NOT NULL,
            Description TEXT,
            ZoneName TEXT,
            Status TEXT NOT NULL DEFAULT 'Pending',
            CreatedAt TEXT NOT NULL DEFAULT (datetime('now')),
            ResolvedAt TEXT,
            ResolvedBy TEXT,
            ResolvedNotes TEXT
        );

        CREATE INDEX IF NOT EXISTS IX_PlayerReports_TargetName ON PlayerReports(TargetName);
        CREATE INDEX IF NOT EXISTS IX_PlayerReports_Status ON PlayerReports(Status);

        CREATE TABLE IF NOT EXISTS ChatLogs (
            Id INTEGER PRIMARY KEY AUTOINCREMENT,
            SenderUserId INTEGER,
            SenderName TEXT NOT NULL,
            Channel TEXT NOT NULL DEFAULT 'Say',
            RecipientName TEXT,
            Message TEXT NOT NULL,
            ZoneName TEXT,
            CreatedAt TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE INDEX IF NOT EXISTS IX_ChatLogs_SenderName ON ChatLogs(SenderName);
        CREATE INDEX IF NOT EXISTS IX_ChatLogs_RecipientName ON ChatLogs(RecipientName);
        CREATE INDEX IF NOT EXISTS IX_ChatLogs_CreatedAt ON ChatLogs(CreatedAt);

        CREATE TABLE IF NOT EXISTS CommandLogs (
            Id INTEGER PRIMARY KEY AUTOINCREMENT,
            ActorUserId INTEGER,
            ActorName TEXT NOT NULL,
            ActorSource TEXT NOT NULL DEFAULT 'Discord',
            Command TEXT NOT NULL,
            Arguments TEXT,
            Target TEXT,
            Success INTEGER NOT NULL DEFAULT 1,
            CreatedAt TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE INDEX IF NOT EXISTS IX_CommandLogs_ActorName ON CommandLogs(ActorName);
        CREATE INDEX IF NOT EXISTS IX_CommandLogs_Command ON CommandLogs(Command);
        CREATE INDEX IF NOT EXISTS IX_CommandLogs_CreatedAt ON CommandLogs(CreatedAt);
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn run_mysql_migrations(pool: &MySqlPool) -> Result<()> {
    info!("Running MySQL/MariaDB moderation migrations...");

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS ModerationLogs (
            Id BIGINT AUTO_INCREMENT PRIMARY KEY,
            TargetUserId BIGINT UNSIGNED NOT NULL,
            TargetName VARCHAR(255) NOT NULL,
            ActorUserId BIGINT UNSIGNED NULL,
            ActorName VARCHAR(255) NOT NULL,
            ActorSource VARCHAR(50) NOT NULL DEFAULT 'Discord',
            Action VARCHAR(50) NOT NULL,
            Reason TEXT NULL,
            Duration VARCHAR(100) NULL,
            CreatedAt DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            INDEX IX_ModerationLogs_TargetUserId (TargetUserId),
            INDEX IX_ModerationLogs_TargetName (TargetName)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS Warnings (
            Id BIGINT AUTO_INCREMENT PRIMARY KEY,
            TargetUserId BIGINT UNSIGNED NOT NULL,
            TargetName VARCHAR(255) NOT NULL,
            Reason TEXT NOT NULL,
            IssuedBy VARCHAR(255) NOT NULL,
            IssuedBySource VARCHAR(50) NOT NULL DEFAULT 'Discord',
            Severity INT NOT NULL DEFAULT 1,
            Acknowledged TINYINT(1) NOT NULL DEFAULT 0,
            Active TINYINT(1) NOT NULL DEFAULT 1,
            CreatedAt DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            ExpiresAt DATETIME NULL,
            INDEX IX_Warnings_TargetUserId (TargetUserId),
            INDEX IX_Warnings_TargetName (TargetName),
            INDEX IX_Warnings_Active (Active)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS PlayerReports (
            Id BIGINT AUTO_INCREMENT PRIMARY KEY,
            TargetUserId BIGINT UNSIGNED NOT NULL,
            TargetName VARCHAR(255) NOT NULL,
            ReporterUserId BIGINT UNSIGNED NULL,
            ReporterName VARCHAR(255) NOT NULL,
            ReporterSource VARCHAR(50) NOT NULL DEFAULT 'InGame',
            Reason VARCHAR(255) NOT NULL,
            Description TEXT NULL,
            ZoneName VARCHAR(100) NULL,
            Status VARCHAR(50) NOT NULL DEFAULT 'Pending',
            CreatedAt DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            ResolvedAt DATETIME NULL,
            ResolvedBy VARCHAR(255) NULL,
            ResolvedNotes TEXT NULL,
            INDEX IX_PlayerReports_TargetName (TargetName),
            INDEX IX_PlayerReports_Status (Status)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS ChatLogs (
            Id BIGINT AUTO_INCREMENT PRIMARY KEY,
            SenderUserId BIGINT UNSIGNED NULL,
            SenderName VARCHAR(255) NOT NULL,
            Channel VARCHAR(50) NOT NULL DEFAULT 'Say',
            RecipientName VARCHAR(255) NULL,
            Message TEXT NOT NULL,
            ZoneName VARCHAR(100) NULL,
            CreatedAt DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            INDEX IX_ChatLogs_SenderName (SenderName),
            INDEX IX_ChatLogs_RecipientName (RecipientName),
            INDEX IX_ChatLogs_CreatedAt (CreatedAt)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS CommandLogs (
            Id BIGINT AUTO_INCREMENT PRIMARY KEY,
            ActorUserId BIGINT UNSIGNED NULL,
            ActorName VARCHAR(255) NOT NULL,
            ActorSource VARCHAR(50) NOT NULL DEFAULT 'Discord',
            Command VARCHAR(100) NOT NULL,
            Arguments TEXT NULL,
            Target VARCHAR(255) NULL,
            Success TINYINT(1) NOT NULL DEFAULT 1,
            CreatedAt DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            INDEX IX_CommandLogs_ActorName (ActorName),
            INDEX IX_CommandLogs_Command (Command),
            INDEX IX_CommandLogs_CreatedAt (CreatedAt)
        ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4;
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}
