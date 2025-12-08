//! Configuration loading and management

use crate::utils::error::{GenesisError, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisConfig {
    pub empire: EmpireConfig,
    pub scanning: ScanningConfig,
    pub ai: AiConfig,
    pub deployment: DeploymentConfig,
    pub github: GitHubConfig,
    pub revenue: RevenueConfig,
    pub database: DatabaseConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmpireConfig {
    pub name: String,
    pub auto_mode: bool,
    pub max_concurrent_projects: u32,
    pub budget_per_project_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanningConfig {
    pub enabled_sources: Vec<String>,
    pub scan_interval_hours: u32,
    pub min_opportunity_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub primary_provider: String,
    pub fallback_provider: Option<String>,
    pub max_retries: u32,
    pub genspark: GenSparkConfig,
    pub openai: OpenAiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenSparkConfig {
    pub api_key: Option<String>,
    pub workspace_id: Option<String>,
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiConfig {
    pub api_key: Option<String>,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    pub default_platform: String,
    pub auto_deploy: bool,
    pub vercel: VercelConfig,
    pub cloudflare: CloudflareConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VercelConfig {
    pub api_token: Option<String>,
    pub team_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareConfig {
    pub api_token: Option<String>,
    pub account_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    pub token: Option<String>,
    pub org: Option<String>,
    pub auto_create_repos: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueConfig {
    pub track_interval_hours: u32,
    pub alert_threshold_drop_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub path: String,
    pub auto_backup: bool,
}

impl Default for GenesisConfig {
    fn default() -> Self {
        Self {
            empire: EmpireConfig {
                name: "genesis-empire".to_string(),
                auto_mode: false,
                max_concurrent_projects: 5,
                budget_per_project_usd: 500.0,
            },
            scanning: ScanningConfig {
                enabled_sources: vec![
                    "github".to_string(),
                    "reddit".to_string(),
                    "producthunt".to_string(),
                    "hackernews".to_string(),
                ],
                scan_interval_hours: 6,
                min_opportunity_score: 75.0,
            },
            ai: AiConfig {
                primary_provider: "genspark".to_string(),
                fallback_provider: Some("openai".to_string()),
                max_retries: 3,
                genspark: GenSparkConfig {
                    api_key: None,
                    workspace_id: None,
                    base_url: "https://api.genspark.ai/v1".to_string(),
                },
                openai: OpenAiConfig {
                    api_key: None,
                    model: "gpt-4-turbo-preview".to_string(),
                },
            },
            deployment: DeploymentConfig {
                default_platform: "vercel".to_string(),
                auto_deploy: false,
                vercel: VercelConfig {
                    api_token: None,
                    team_id: None,
                },
                cloudflare: CloudflareConfig {
                    api_token: None,
                    account_id: None,
                },
            },
            github: GitHubConfig {
                token: None,
                org: None,
                auto_create_repos: true,
            },
            revenue: RevenueConfig {
                track_interval_hours: 24,
                alert_threshold_drop_percent: 20.0,
            },
            database: DatabaseConfig {
                path: "./data/genesis.db".to_string(),
                auto_backup: true,
            },
        }
    }
}

impl GenesisConfig {
    /// Load configuration from file and environment
    pub fn load(config_path: Option<&Path>) -> Result<Self> {
        // Load .env file if present
        let _ = dotenvy::dotenv();

        let config_file = config_path
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("config/default.toml"));

        let mut config = if config_file.exists() {
            let content = std::fs::read_to_string(&config_file)
                .map_err(|e| GenesisError::Config(format!("Failed to read config: {}", e)))?;
            toml::from_str(&content)
                .map_err(|e| GenesisError::Config(format!("Invalid TOML: {}", e)))?
        } else {
            Self::default()
        };

        // Override with environment variables
        config.apply_env_overrides();

        Ok(config)
    }

    /// Apply environment variable overrides
    fn apply_env_overrides(&mut self) {
        if let Ok(val) = std::env::var("GENSPARK_API_KEY") {
            self.ai.genspark.api_key = Some(val);
        }
        if let Ok(val) = std::env::var("GENSPARK_WORKSPACE_ID") {
            self.ai.genspark.workspace_id = Some(val);
        }
        if let Ok(val) = std::env::var("OPENAI_API_KEY") {
            self.ai.openai.api_key = Some(val);
        }
        if let Ok(val) = std::env::var("GITHUB_TOKEN") {
            self.github.token = Some(val);
        }
        if let Ok(val) = std::env::var("GITHUB_ORG") {
            self.github.org = Some(val);
        }
        if let Ok(val) = std::env::var("VERCEL_TOKEN") {
            self.deployment.vercel.api_token = Some(val);
        }
        if let Ok(val) = std::env::var("VERCEL_TEAM_ID") {
            self.deployment.vercel.team_id = Some(val);
        }
        if let Ok(val) = std::env::var("CLOUDFLARE_TOKEN") {
            self.deployment.cloudflare.api_token = Some(val);
        }
        if let Ok(val) = std::env::var("CLOUDFLARE_ACCOUNT_ID") {
            self.deployment.cloudflare.account_id = Some(val);
        }
        if let Ok(val) = std::env::var("GENESIS_DB_PATH") {
            self.database.path = val;
        }
    }

    /// Save configuration to file
    pub fn save(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| GenesisError::Config(format!("Failed to serialize config: {}", e)))?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(path, content)?;
        Ok(())
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        if self.empire.name.is_empty() {
            return Err(GenesisError::Validation("Empire name cannot be empty".to_string()));
        }

        if self.scanning.enabled_sources.is_empty() {
            return Err(GenesisError::Validation(
                "At least one scanning source must be enabled".to_string(),
            ));
        }

        if self.scanning.min_opportunity_score < 0.0 || self.scanning.min_opportunity_score > 100.0 {
            return Err(GenesisError::Validation(
                "Opportunity score must be between 0 and 100".to_string(),
            ));
        }

        Ok(())
    }

    /// Get the database path, ensuring parent directories exist
    pub fn get_db_path(&self) -> Result<PathBuf> {
        let path = PathBuf::from(&self.database.path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Ok(path)
    }
}

/// Get the default configuration directory
pub fn get_config_dir() -> PathBuf {
    std::env::var("HOME")
        .map(|h| PathBuf::from(h).join(".config").join("genesis"))
        .unwrap_or_else(|_| PathBuf::from(".genesis"))
}

/// Get the default data directory
pub fn get_data_dir() -> PathBuf {
    std::env::var("HOME")
        .map(|h| PathBuf::from(h).join(".local").join("share").join("genesis"))
        .unwrap_or_else(|_| PathBuf::from("./data"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = GenesisConfig::default();
        assert_eq!(config.empire.name, "genesis-empire");
        assert!(!config.empire.auto_mode);
        assert_eq!(config.scanning.min_opportunity_score, 75.0);
    }

    #[test]
    fn test_config_validation() {
        let mut config = GenesisConfig::default();
        assert!(config.validate().is_ok());

        config.empire.name = String::new();
        assert!(config.validate().is_err());
    }
}
