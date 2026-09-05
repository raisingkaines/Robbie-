use crate::config::WarningSystemConfig;
use crate::db::models::DbUser;
use crate::db::Database;
use chrono::{Duration, Utc};
use anyhow::Result;
use tracing::info;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutoAction {
    None,
    AutoMute {
        duration_minutes: i64,
        points: i32,
    },
    AutoBan {
        duration: Option<Duration>,
        duration_desc: String,
        points: i32,
    },
}

pub fn calculate_escalation_action(
    points: i32,
    config: &WarningSystemConfig,
) -> AutoAction {
    if points >= config.auto_ban_threshold_permanent {
        AutoAction::AutoBan {
            duration: None,
            duration_desc: "Permanent".to_string(),
            points,
        }
    } else if points >= config.auto_ban_threshold_2 {
        AutoAction::AutoBan {
            duration: Some(Duration::days(config.auto_ban_duration_days_2)),
            duration_desc: format!("{} days", config.auto_ban_duration_days_2),
            points,
        }
    } else if points >= config.auto_ban_threshold_1 {
        AutoAction::AutoBan {
            duration: Some(Duration::hours(config.auto_ban_duration_hours_1)),
            duration_desc: format!("{} hours", config.auto_ban_duration_hours_1),
            points,
        }
    } else if points >= config.auto_mute_threshold {
        AutoAction::AutoMute {
            duration_minutes: config.auto_mute_duration_minutes,
            points,
        }
    } else {
        AutoAction::None
    }
}

pub async fn evaluate_warning_escalation(
    db: &Database,
    user: &DbUser,
    character_name: &str,
    config: &WarningSystemConfig,
) -> Result<AutoAction> {
    let user_id = user.id;
    let points = db.get_active_warning_points(user_id).await?;

    info!("User {user_id} ({character_name}) has {points} active warning point(s)");

    let action = calculate_escalation_action(points, config);

    match &action {
        AutoAction::AutoBan { duration: None, .. } => {
            db.set_user_locked_until(user_id, Some(crate::utils::permanent_ban_date())).await?;
        }
        AutoAction::AutoBan { duration: Some(dur), .. } => {
            let ban_until = Utc::now() + *dur;
            db.set_user_locked_until(user_id, Some(ban_until)).await?;
        }
        AutoAction::AutoMute { duration_minutes, .. } => {
            let mute_until = Utc::now() + Duration::minutes(*duration_minutes);
            db.set_user_muted_until(user_id, Some(mute_until)).await?;
        }
        AutoAction::None => {}
    }

    Ok(action)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_escalation_no_action() {
        let config = WarningSystemConfig::default();
        let action = calculate_escalation_action(2, &config);
        assert_eq!(action, AutoAction::None);
    }

    #[test]
    fn test_calculate_escalation_auto_mute() {
        let config = WarningSystemConfig::default();
        let action = calculate_escalation_action(3, &config);
        assert_eq!(
            action,
            AutoAction::AutoMute {
                duration_minutes: 30,
                points: 3,
            }
        );

        let action4 = calculate_escalation_action(4, &config);
        assert_eq!(
            action4,
            AutoAction::AutoMute {
                duration_minutes: 30,
                points: 4,
            }
        );
    }

    #[test]
    fn test_calculate_escalation_auto_ban_tier_1() {
        let config = WarningSystemConfig::default();
        let action = calculate_escalation_action(5, &config);
        assert_eq!(
            action,
            AutoAction::AutoBan {
                duration: Some(Duration::hours(24)),
                duration_desc: "24 hours".to_string(),
                points: 5,
            }
        );
    }

    #[test]
    fn test_calculate_escalation_auto_ban_tier_2() {
        let config = WarningSystemConfig::default();
        let action = calculate_escalation_action(7, &config);
        assert_eq!(
            action,
            AutoAction::AutoBan {
                duration: Some(Duration::days(7)),
                duration_desc: "7 days".to_string(),
                points: 7,
            }
        );
    }

    #[test]
    fn test_calculate_escalation_permanent_ban() {
        let config = WarningSystemConfig::default();
        let action = calculate_escalation_action(10, &config);
        assert_eq!(
            action,
            AutoAction::AutoBan {
                duration: None,
                duration_desc: "Permanent".to_string(),
                points: 10,
            }
        );

        let action_excess = calculate_escalation_action(15, &config);
        assert_eq!(
            action_excess,
            AutoAction::AutoBan {
                duration: None,
                duration_desc: "Permanent".to_string(),
                points: 15,
            }
        );
    }
}
