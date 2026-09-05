use thiserror::Error;

#[derive(Error, Debug)]
pub enum BotError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Gateway API error: {0}")]
    Gateway(#[from] reqwest::Error),

    #[error("Discord error: {0}")]
    Discord(#[from] poise::serenity_prelude::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Access denied: {0}")]
    AccessDenied(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Operation failed: {0}")]
    OperationFailed(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<&str> for BotError {
    fn from(s: &str) -> Self {
        BotError::OperationFailed(s.to_string())
    }
}

impl From<String> for BotError {
    fn from(s: String) -> Self {
        BotError::OperationFailed(s)
    }
}

pub type BotResult<T> = Result<T, BotError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_from_str() {
        let err: BotError = "Player not found".into();
        assert_eq!(err.to_string(), "Operation failed: Player not found");
    }

    #[test]
    fn test_error_from_string() {
        let err: BotError = String::from("Database locked").into();
        assert_eq!(err.to_string(), "Operation failed: Database locked");
    }

    #[test]
    fn test_validation_error_format() {
        let err = BotError::Validation("Coordinate X must be finite".to_string());
        assert_eq!(err.to_string(), "Validation error: Coordinate X must be finite");
    }

    #[test]
    fn test_access_denied_error_format() {
        let err = BotError::AccessDenied("Requires Administrator role".to_string());
        assert_eq!(err.to_string(), "Access denied: Requires Administrator role");
    }
}
