use anyhow::{Context, Result};
use reqwest::Client;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone)]
pub struct GatewayClient {
    client: Client,
    base_url: String,
    api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatusResponse {
    pub is_online: bool,
    pub online_players: usize,
    pub max_players: usize,
    pub uptime_seconds: u64,
    pub server_version: Option<String>,
    #[serde(default)]
    pub maintenance_mode: bool,
    #[serde(default)]
    pub shutdown_pending: bool,
    #[serde(default)]
    pub shutdown_seconds_remaining: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnlinePlayerInfo {
    pub name: String,
    pub guid: u64,
    pub zone_name: String,
    pub is_mod: bool,
    pub is_admin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KickRequest {
    pub player: String,
    pub reason: Option<String>,
    pub reason_code: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KickResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarnRequest {
    pub player: String,
    pub message: String,
    pub severity: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastRequest {
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShutdownRequest {
    pub countdown_seconds: i32,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceRequest {
    pub enabled: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotdRequest {
    pub title: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeleportRequest {
    pub player: String,
    pub zone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovePlayerCoordsRequest {
    pub player: String,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenameRequest {
    pub old_name: String,
    pub new_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminApiResponse {
    pub success: bool,
    pub message: Option<String>,
    #[serde(default)]
    pub maintenance_mode: Option<bool>,
    #[serde(default)]
    pub countdown_seconds: Option<i32>,
    pub reason: Option<String>,
}

impl GatewayClient {
    pub fn new(
        base_url: String,
        api_key: String,
        timeout_seconds: u64,
        connect_timeout_seconds: u64,
    ) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_seconds))
            .connect_timeout(Duration::from_secs(connect_timeout_seconds))
            .tcp_keepalive(Some(Duration::from_secs(60)))
            .pool_idle_timeout(Some(Duration::from_secs(90)))
            .build()
            .unwrap_or_default();

        Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
        }
    }

    pub fn with_defaults(base_url: String, api_key: String) -> Self {
        Self::new(base_url, api_key, 5, 5)
    }

    async fn get<R: DeserializeOwned>(&self, path: &str) -> Result<R> {
        let url = format!("{}{path}", self.base_url);
        let resp = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .with_context(|| format!("Failed to reach Gateway server at {url}"))?;

        if !resp.status().is_success() {
            anyhow::bail!("Gateway server returned HTTP status {}", resp.status());
        }
        Ok(resp.json::<R>().await?)
    }

    async fn post<T: Serialize, R: DeserializeOwned>(&self, path: &str, body: &T) -> Result<R> {
        let url = format!("{}{path}", self.base_url);
        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(body)
            .send()
            .await
            .with_context(|| format!("Failed to reach Gateway server at {url}"))?;

        if !resp.status().is_success() {
            anyhow::bail!("Gateway server returned HTTP status {}", resp.status());
        }
        Ok(resp.json::<R>().await?)
    }

    async fn post_empty<R: DeserializeOwned>(&self, path: &str) -> Result<R> {
        let url = format!("{}{path}", self.base_url);
        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .with_context(|| format!("Failed to reach Gateway server at {url}"))?;

        if !resp.status().is_success() {
            anyhow::bail!("Gateway server returned HTTP status {}", resp.status());
        }
        Ok(resp.json::<R>().await?)
    }

    pub async fn get_status(&self) -> Result<ServerStatusResponse> {
        self.get("/api/admin/status").await
    }

    pub async fn get_online_players(&self) -> Result<Vec<OnlinePlayerInfo>> {
        self.get("/api/admin/players").await
    }

    pub async fn kick_player(&self, player: &str, reason: Option<&str>, reason_code: i32) -> Result<KickResponse> {
        let url = format!("{}/api/admin/kick", self.base_url);
        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&KickRequest {
                player: player.to_string(),
                reason: reason.map(|r| r.to_string()),
                reason_code,
            })
            .send()
            .await
            .with_context(|| format!("Failed to send kick request to {url}"))?;

        if !resp.status().is_success() {
            let error_text = resp.text().await.unwrap_or_default();
            return Ok(KickResponse {
                success: false,
                message: format!("HTTP error: {error_text}"),
            });
        }
        Ok(resp.json::<KickResponse>().await?)
    }

    pub async fn send_in_game_warning(&self, player: &str, message: &str, severity: i32) -> Result<AdminApiResponse> {
        self.post("/api/admin/warn", &WarnRequest {
            player: player.to_string(),
            message: message.to_string(),
            severity,
        }).await
    }

    pub async fn broadcast_message(&self, message: &str) -> Result<AdminApiResponse> {
        self.post("/api/admin/broadcast", &BroadcastRequest {
            message: message.to_string(),
        }).await
    }

    pub async fn initiate_shutdown(&self, countdown_seconds: i32, reason: Option<&str>) -> Result<AdminApiResponse> {
        self.post("/api/admin/shutdown", &ShutdownRequest {
            countdown_seconds,
            reason: reason.map(|r| r.to_string()),
        }).await
    }

    pub async fn cancel_shutdown(&self) -> Result<AdminApiResponse> {
        self.post_empty("/api/admin/shutdown/cancel").await
    }

    pub async fn set_maintenance_mode(&self, enabled: bool, reason: Option<&str>) -> Result<AdminApiResponse> {
        self.post("/api/admin/maintenance", &MaintenanceRequest {
            enabled,
            reason: reason.map(|r| r.to_string()),
        }).await
    }

    pub async fn set_motd(&self, title: &str, message: &str) -> Result<AdminApiResponse> {
        self.post("/api/admin/motd", &MotdRequest {
            title: title.to_string(),
            message: message.to_string(),
        }).await
    }

    pub async fn teleport_player(&self, player: &str, zone: &str) -> Result<AdminApiResponse> {
        self.post("/api/admin/teleport", &TeleportRequest {
            player: player.to_string(),
            zone: zone.to_string(),
        }).await
    }

    pub async fn move_player_coords(
        &self,
        player: &str,
        x: f32,
        y: f32,
        z: f32,
        rotation: Option<f32>,
    ) -> Result<AdminApiResponse> {
        self.post("/api/admin/move", &MovePlayerCoordsRequest {
            player: player.to_string(),
            x,
            y,
            z,
            rotation,
        }).await
    }

    pub async fn rename_player(&self, old_name: &str, new_name: &str) -> Result<AdminApiResponse> {
        self.post("/api/admin/rename", &RenameRequest {
            old_name: old_name.to_string(),
            new_name: new_name.to_string(),
        }).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gateway_url_normalization() {
        let client = GatewayClient::with_defaults("http://127.0.0.1:5000/".to_string(), "key".to_string());
        assert_eq!(client.base_url, "http://127.0.0.1:5000");

        let client2 = GatewayClient::with_defaults("http://127.0.0.1:5000///".to_string(), "key".to_string());
        assert_eq!(client2.base_url, "http://127.0.0.1:5000");
    }

    #[test]
    fn test_move_player_coords_request_serialization() {
        let req = MovePlayerCoordsRequest {
            player: "TestHero".to_string(),
            x: 100.5,
            y: 50.25,
            z: -300.0,
            rotation: Some(1.57),
        };
        let json_val = serde_json::to_string(&req).unwrap();
        assert!(json_val.contains("\"player\":\"TestHero\""));
        assert!(json_val.contains("\"x\":100.5"));
        assert!(json_val.contains("\"rotation\":1.57"));
    }

    #[test]
    fn test_move_player_coords_without_rotation() {
        let req = MovePlayerCoordsRequest {
            player: "TestHero".to_string(),
            x: 10.0,
            y: 20.0,
            z: 30.0,
            rotation: None,
        };
        let json_val = serde_json::to_string(&req).unwrap();
        assert!(!json_val.contains("rotation"));
    }
}
