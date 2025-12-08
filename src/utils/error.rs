//! Custom error types for Genesis Engine

use thiserror::Error;

/// Main error type for Genesis Engine
#[derive(Error, Debug)]
pub enum GenesisError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Git operation failed: {0}")]
    Git(#[from] git2::Error),

    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    #[error("TOML deserialization error: {0}")]
    TomlDe(#[from] toml::de::Error),

    #[error("API error: {message} (status: {status_code})")]
    Api {
        message: String,
        status_code: u16,
    },

    #[error("Opportunity not found: {0}")]
    OpportunityNotFound(String),

    #[error("Project not found: {0}")]
    ProjectNotFound(String),

    #[error("Deployment failed: {0}")]
    DeploymentFailed(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Rate limited. Retry after: {retry_after} seconds")]
    RateLimited { retry_after: u64 },

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Not initialized. Run 'genesis init' first.")]
    NotInitialized,

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Template not found: {0}")]
    TemplateNotFound(String),

    #[error("Scanner error: {scanner_source} - {message}")]
    Scanner {
        scanner_source: String,
        message: String,
    },

    #[error("External service unavailable: {service}")]
    ServiceUnavailable { service: String },

    #[error("{0}")]
    Custom(String),
}

impl GenesisError {
    /// Create a custom error with a message
    pub fn custom<S: Into<String>>(msg: S) -> Self {
        GenesisError::Custom(msg.into())
    }

    /// Create an API error
    pub fn api<S: Into<String>>(message: S, status_code: u16) -> Self {
        GenesisError::Api {
            message: message.into(),
            status_code,
        }
    }

    /// Create a scanner error
    pub fn scanner<S: Into<String>, M: Into<String>>(scanner_source: S, message: M) -> Self {
        GenesisError::Scanner {
            scanner_source: scanner_source.into(),
            message: message.into(),
        }
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            GenesisError::RateLimited { .. }
                | GenesisError::ServiceUnavailable { .. }
                | GenesisError::Http(_)
        )
    }
}

/// Result type alias for Genesis operations
pub type Result<T> = std::result::Result<T, GenesisError>;

/// Implement From<String> for GenesisError
impl From<String> for GenesisError {
    fn from(s: String) -> Self {
        GenesisError::Custom(s)
    }
}

/// Implement From<&str> for GenesisError
impl From<&str> for GenesisError {
    fn from(s: &str) -> Self {
        GenesisError::Custom(s.to_string())
    }
}

/// Extension trait for adding context to errors
pub trait ResultExt<T> {
    fn context<C: Into<String>>(self, context: C) -> Result<T>;
}

impl<T, E: Into<GenesisError>> ResultExt<T> for std::result::Result<T, E> {
    fn context<C: Into<String>>(self, context: C) -> Result<T> {
        self.map_err(|e| {
            let base_error = e.into();
            GenesisError::Custom(format!("{}: {}", context.into(), base_error))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = GenesisError::Config("missing API key".to_string());
        assert_eq!(err.to_string(), "Configuration error: missing API key");
    }

    #[test]
    fn test_retryable_errors() {
        assert!(GenesisError::RateLimited { retry_after: 60 }.is_retryable());
        assert!(GenesisError::ServiceUnavailable {
            service: "github".to_string()
        }
        .is_retryable());
        assert!(!GenesisError::NotInitialized.is_retryable());
    }
}
