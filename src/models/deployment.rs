//! Deployment model and related types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Deployment platform
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Copy)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum Platform {
    #[default]
    Vercel,
    Cloudflare,
    Netlify,
    Railway,
    Render,
    Fly,
}


impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Vercel => write!(f, "vercel"),
            Self::Cloudflare => write!(f, "cloudflare"),
            Self::Netlify => write!(f, "netlify"),
            Self::Railway => write!(f, "railway"),
            Self::Render => write!(f, "render"),
            Self::Fly => write!(f, "fly"),
        }
    }
}

impl std::str::FromStr for Platform {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "vercel" => Ok(Self::Vercel),
            "cloudflare" | "cf" | "workers" => Ok(Self::Cloudflare),
            "netlify" => Ok(Self::Netlify),
            "railway" => Ok(Self::Railway),
            "render" => Ok(Self::Render),
            "fly" | "flyio" => Ok(Self::Fly),
            _ => Ok(Self::Vercel), // Default to Vercel
        }
    }
}

/// Deployment status
#[derive(Debug, Clone, PartialEq, Eq, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum DeploymentStatus {
    #[default]
    Queued,
    Building,
    Deploying,
    Ready,
    Failed,
    Cancelled,
}


impl std::fmt::Display for DeploymentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Queued => write!(f, "queued"),
            Self::Building => write!(f, "building"),
            Self::Deploying => write!(f, "deploying"),
            Self::Ready => write!(f, "ready"),
            Self::Failed => write!(f, "failed"),
            Self::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl std::str::FromStr for DeploymentStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "queued" => Ok(Self::Queued),
            "building" => Ok(Self::Building),
            "deploying" => Ok(Self::Deploying),
            "ready" | "success" | "live" => Ok(Self::Ready),
            "failed" | "error" => Ok(Self::Failed),
            "cancelled" | "canceled" => Ok(Self::Cancelled),
            _ => Err(format!("Invalid deployment status: {}", s)),
        }
    }
}

/// Build logs from deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildLog {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub message: String,
}

/// Environment variables for deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVar {
    pub key: String,
    pub value: String,
    pub is_secret: bool,
}

/// Domain configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainConfig {
    pub domain: String,
    pub is_primary: bool,
    pub ssl_enabled: bool,
    pub verified: bool,
}

/// A deployment record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Deployment {
    pub id: String,
    pub project_id: String,
    pub platform: Platform,
    pub platform_deployment_id: Option<String>,
    pub url: Option<String>,
    pub production_url: Option<String>,
    pub status: DeploymentStatus,
    pub commit_sha: Option<String>,
    pub commit_message: Option<String>,
    pub branch: String,
    pub build_logs: Vec<BuildLog>,
    pub env_vars: Vec<EnvVar>,
    pub domains: Vec<DomainConfig>,
    pub build_time_seconds: Option<u64>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub deployed_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

impl Deployment {
    /// Create a new deployment
    pub fn new(project_id: impl Into<String>, platform: Platform) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            project_id: project_id.into(),
            platform,
            platform_deployment_id: None,
            url: None,
            production_url: None,
            status: DeploymentStatus::Queued,
            commit_sha: None,
            commit_message: None,
            branch: "main".to_string(),
            build_logs: Vec::new(),
            env_vars: Vec::new(),
            domains: Vec::new(),
            build_time_seconds: None,
            error_message: None,
            created_at: now,
            deployed_at: None,
            updated_at: now,
        }
    }

    /// Set the platform deployment ID
    pub fn set_platform_id(&mut self, id: impl Into<String>) {
        self.platform_deployment_id = Some(id.into());
        self.updated_at = Utc::now();
    }

    /// Update status
    pub fn set_status(&mut self, status: DeploymentStatus) {
        self.status = status;
        self.updated_at = Utc::now();

        if status == DeploymentStatus::Ready && self.deployed_at.is_none() {
            self.deployed_at = Some(Utc::now());
        }
    }

    /// Set deployment URL
    pub fn set_url(&mut self, url: impl Into<String>) {
        self.url = Some(url.into());
        self.updated_at = Utc::now();
    }

    /// Set production URL
    pub fn set_production_url(&mut self, url: impl Into<String>) {
        self.production_url = Some(url.into());
        self.updated_at = Utc::now();
    }

    /// Mark as failed
    pub fn fail(&mut self, error: impl Into<String>) {
        self.status = DeploymentStatus::Failed;
        self.error_message = Some(error.into());
        self.updated_at = Utc::now();
    }

    /// Add a build log entry
    pub fn add_log(&mut self, level: impl Into<String>, message: impl Into<String>) {
        self.build_logs.push(BuildLog {
            timestamp: Utc::now(),
            level: level.into(),
            message: message.into(),
        });
    }

    /// Add an environment variable
    pub fn add_env_var(&mut self, key: impl Into<String>, value: impl Into<String>, is_secret: bool) {
        self.env_vars.push(EnvVar {
            key: key.into(),
            value: value.into(),
            is_secret,
        });
    }

    /// Add a domain
    pub fn add_domain(&mut self, domain: impl Into<String>, is_primary: bool) {
        self.domains.push(DomainConfig {
            domain: domain.into(),
            is_primary,
            ssl_enabled: true,
            verified: false,
        });
    }

    /// Check if deployment is in progress
    pub fn is_in_progress(&self) -> bool {
        matches!(
            self.status,
            DeploymentStatus::Queued | DeploymentStatus::Building | DeploymentStatus::Deploying
        )
    }

    /// Check if deployment succeeded
    pub fn is_successful(&self) -> bool {
        self.status == DeploymentStatus::Ready
    }

    /// Get the primary URL
    pub fn primary_url(&self) -> Option<&str> {
        self.production_url
            .as_deref()
            .or(self.url.as_deref())
    }

    /// Set commit information
    pub fn set_commit(&mut self, sha: impl Into<String>, message: Option<String>) {
        self.commit_sha = Some(sha.into());
        self.commit_message = message;
        self.updated_at = Utc::now();
    }
}

/// Deployment configuration for creating new deployments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    pub platform: Platform,
    pub branch: String,
    pub env_vars: Vec<EnvVar>,
    pub build_command: Option<String>,
    pub output_directory: Option<String>,
    pub framework: Option<String>,
}

impl Default for DeploymentConfig {
    fn default() -> Self {
        Self {
            platform: Platform::default(),
            branch: "main".to_string(),
            env_vars: Vec::new(),
            build_command: None,
            output_directory: None,
            framework: None,
        }
    }
}

impl DeploymentConfig {
    pub fn new(platform: Platform) -> Self {
        Self {
            platform,
            ..Default::default()
        }
    }

    pub fn with_branch(mut self, branch: impl Into<String>) -> Self {
        self.branch = branch.into();
        self
    }

    pub fn add_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env_vars.push(EnvVar {
            key: key.into(),
            value: value.into(),
            is_secret: false,
        });
        self
    }

    pub fn add_secret(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env_vars.push(EnvVar {
            key: key.into(),
            value: value.into(),
            is_secret: true,
        });
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_deployment() {
        let deployment = Deployment::new("project-123", Platform::Vercel);
        assert!(!deployment.id.is_empty());
        assert_eq!(deployment.project_id, "project-123");
        assert_eq!(deployment.platform, Platform::Vercel);
        assert_eq!(deployment.status, DeploymentStatus::Queued);
    }

    #[test]
    fn test_deployment_lifecycle() {
        let mut deployment = Deployment::new("project-123", Platform::Cloudflare);
        assert!(deployment.deployed_at.is_none());

        deployment.set_status(DeploymentStatus::Building);
        assert!(deployment.deployed_at.is_none());

        deployment.set_status(DeploymentStatus::Ready);
        assert!(deployment.deployed_at.is_some());
        assert!(deployment.is_successful());
    }

    #[test]
    fn test_platform_parsing() {
        assert_eq!("vercel".parse::<Platform>().unwrap(), Platform::Vercel);
        assert_eq!("cf".parse::<Platform>().unwrap(), Platform::Cloudflare);
        assert_eq!("workers".parse::<Platform>().unwrap(), Platform::Cloudflare);
    }

    #[test]
    fn test_deployment_config_builder() {
        let config = DeploymentConfig::new(Platform::Vercel)
            .with_branch("develop")
            .add_env("NODE_ENV", "production")
            .add_secret("API_KEY", "secret123");

        assert_eq!(config.branch, "develop");
        assert_eq!(config.env_vars.len(), 2);
        assert!(config.env_vars[1].is_secret);
    }
}
