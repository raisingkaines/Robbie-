use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbUser {
    pub id: u64,
    pub username: String,
    pub password: Option<String>,
    pub session: Option<String>,
    pub max_characters: i32,
    pub is_member: bool,
    pub is_admin: bool,
    pub is_mod: bool,
    pub locked_until: Option<DateTime<Utc>>,
    pub muted_until: Option<DateTime<Utc>>,
    pub created: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbCharacter {
    pub id: u64,
    pub user_id: u64,
    pub first_name: String,
    pub last_name: Option<String>,
    pub full_name: String,
    pub coins: i32,
    pub station_cash: i32,
    pub play_time: i32,
    pub created: DateTime<Utc>,
    pub last_login: Option<DateTime<Utc>>,
    pub position_x: Option<f32>,
    pub position_y: Option<f32>,
    pub position_z: Option<f32>,
    pub rotation_x: Option<f32>,
    pub rotation_z: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbModerationLog {
    pub id: i64,
    pub target_user_id: u64,
    pub target_name: String,
    pub actor_user_id: Option<u64>,
    pub actor_name: String,
    pub actor_source: String,
    pub action: String,
    pub reason: Option<String>,
    pub duration: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbWarning {
    pub id: i64,
    pub target_user_id: u64,
    pub target_name: String,
    pub reason: String,
    pub issued_by: String,
    pub issued_by_source: String,
    pub severity: i32,
    pub acknowledged: bool,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbPlayerReport {
    pub id: i64,
    pub target_user_id: u64,
    pub target_name: String,
    pub reporter_user_id: Option<u64>,
    pub reporter_name: String,
    pub reporter_source: String,
    pub reason: String,
    pub description: Option<String>,
    pub zone_name: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolved_by: Option<String>,
    pub resolved_notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbChatLog {
    pub id: i64,
    pub sender_user_id: Option<u64>,
    pub sender_name: String,
    pub channel: String,
    pub recipient_name: Option<String>,
    pub message: String,
    pub zone_name: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbCommandLog {
    pub id: i64,
    pub actor_user_id: Option<u64>,
    pub actor_name: String,
    pub actor_source: String,
    pub command: String,
    pub arguments: Option<String>,
    pub target: Option<String>,
    pub success: bool,
    pub created_at: DateTime<Utc>,
}
