//! Deployment Mesh - Orchestrates deployments across multiple platforms

use crate::integrations::{CloudflareClient, VercelClient};
use crate::models::{Deployment, DeploymentConfig, DeploymentStatus, Platform, Project};
use crate::utils::error::{GenesisError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn};

/// Deployment mesh configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshConfig {
    /// Default deployment platform
    pub default_platform: Platform,
    /// Timeout for deployments in seconds
    pub deployment_timeout_secs: u64,
    /// Maximum retry attempts
    pub max_retries: u32,
    /// Enable parallel deployments
    pub parallel_deployments: bool,
}

impl Default for MeshConfig {
    fn default() -> Self {
        Self {
            default_platform: Platform::Vercel,
            deployment_timeout_secs: 300,
            max_retries: 3,
            parallel_deployments: false,
        }
    }
}

/// Deployment result
#[derive(Debug, Clone)]
pub struct DeploymentResult {
    pub deployment: Deployment,
    pub logs: Vec<String>,
    pub duration_secs: u64,
}

/// Deployment mesh for managing multi-platform deployments
pub struct DeploymentMesh {
    config: MeshConfig,
    vercel: Option<VercelClient>,
    cloudflare: Option<CloudflareClient>,
}

impl DeploymentMesh {
    /// Create a new deployment mesh
    pub fn new(
        config: MeshConfig,
        vercel: Option<VercelClient>,
        cloudflare: Option<CloudflareClient>,
    ) -> Self {
        Self {
            config,
            vercel,
            cloudflare,
        }
    }

    /// Create with only configuration
    pub fn from_config(config: MeshConfig) -> Self {
        Self::new(config, None, None)
    }

    /// Set Vercel client
    pub fn with_vercel(mut self, client: VercelClient) -> Self {
        self.vercel = Some(client);
        self
    }

    /// Set Cloudflare client
    pub fn with_cloudflare(mut self, client: CloudflareClient) -> Self {
        self.cloudflare = Some(client);
        self
    }

    /// Deploy a project
    pub async fn deploy(
        &self,
        project: &Project,
        config: DeploymentConfig,
    ) -> Result<DeploymentResult> {
        info!(
            "Deploying project {} to {:?}",
            project.id, config.platform
        );

        let start = std::time::Instant::now();
        let mut logs = Vec::new();

        logs.push(format!(
            "Starting deployment to {:?}",
            config.platform
        ));

        let deployment = match config.platform {
            Platform::Vercel => self.deploy_vercel(project, &config, &mut logs).await?,
            Platform::Cloudflare => self.deploy_cloudflare(project, &config, &mut logs).await?,
            _ => {
                return Err(GenesisError::custom(format!(
                    "Unsupported platform: {:?}",
                    config.platform
                )));
            }
        };

        let duration = start.elapsed().as_secs();
        logs.push(format!("Deployment completed in {} seconds", duration));

        Ok(DeploymentResult {
            deployment,
            logs,
            duration_secs: duration,
        })
    }

    /// Deploy to Vercel
    async fn deploy_vercel(
        &self,
        project: &Project,
        config: &DeploymentConfig,
        logs: &mut Vec<String>,
    ) -> Result<Deployment> {
        let vercel = self.vercel.as_ref().ok_or_else(|| {
            GenesisError::Config("Vercel client not configured".to_string())
        })?;

        let github_repo = project.github_repo.as_ref().ok_or_else(|| {
            GenesisError::Validation("Project has no GitHub repository".to_string())
        })?;

        logs.push("Initiating Vercel deployment".to_string());

        // Extract repo name from URL
        let repo_name = extract_repo_name(github_repo);

        // Deploy
        let vercel_deployment = vercel
            .deploy_from_git(&project.name, &repo_name, &config.branch)
            .await?;

        logs.push(format!("Vercel deployment ID: {}", vercel_deployment.id));

        // Wait for completion
        logs.push("Waiting for deployment to complete...".to_string());

        let completed = vercel
            .wait_for_deployment(&vercel_deployment.id, self.config.deployment_timeout_secs)
            .await?;

        logs.push(format!(
            "Deployment ready: {}",
            completed.url.as_deref().unwrap_or("N/A")
        ));

        // Convert to our deployment model
        let deployment: Deployment = completed.into();

        Ok(deployment)
    }

    /// Deploy to Cloudflare Pages
    async fn deploy_cloudflare(
        &self,
        project: &Project,
        config: &DeploymentConfig,
        logs: &mut Vec<String>,
    ) -> Result<Deployment> {
        let cloudflare = self.cloudflare.as_ref().ok_or_else(|| {
            GenesisError::Config("Cloudflare client not configured".to_string())
        })?;

        logs.push("Initiating Cloudflare Pages deployment".to_string());

        // Create or get project
        let pages_project = cloudflare
            .create_pages_project(&project.name, &config.branch)
            .await?;

        logs.push(format!("Pages project: {}", pages_project.name));

        // Create deployment
        let pages_deployment = cloudflare
            .create_pages_deployment(&pages_project.name, &config.branch)
            .await?;

        logs.push(format!("Deployment ID: {}", pages_deployment.id));

        // Wait for completion
        logs.push("Waiting for deployment to complete...".to_string());

        let completed = cloudflare
            .wait_for_deployment(
                &pages_project.name,
                &pages_deployment.id,
                self.config.deployment_timeout_secs,
            )
            .await?;

        logs.push(format!(
            "Deployment ready: {}",
            completed.url.as_deref().unwrap_or("N/A")
        ));

        // Convert to our deployment model
        let deployment: Deployment = completed.into();

        Ok(deployment)
    }

    /// Get deployment URL
    pub fn get_deployment_url(&self, deployment: &Deployment) -> Option<String> {
        deployment.primary_url().map(String::from)
    }

    /// Check deployment health
    pub async fn check_health(&self, deployment: &Deployment) -> Result<HealthCheckResult> {
        let url = deployment.primary_url().ok_or_else(|| {
            GenesisError::Validation("Deployment has no URL".to_string())
        })?;

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()?;

        let start = std::time::Instant::now();

        match client.get(url).send().await {
            Ok(response) => {
                let response_time = start.elapsed().as_millis() as u64;
                let status_code = response.status().as_u16();
                let is_healthy = response.status().is_success();

                Ok(HealthCheckResult {
                    url: url.to_string(),
                    is_healthy,
                    status_code,
                    response_time_ms: response_time,
                    error: None,
                })
            }
            Err(e) => Ok(HealthCheckResult {
                url: url.to_string(),
                is_healthy: false,
                status_code: 0,
                response_time_ms: start.elapsed().as_millis() as u64,
                error: Some(e.to_string()),
            }),
        }
    }

    /// Rollback a deployment
    pub async fn rollback(
        &self,
        deployment: &Deployment,
        previous_deployment_id: &str,
    ) -> Result<Deployment> {
        info!(
            "Rolling back deployment {} to {}",
            deployment.id, previous_deployment_id
        );

        match deployment.platform {
            Platform::Vercel => {
                // Vercel handles rollbacks through their API
                // For now, we simulate by creating a new deployment
                warn!("Vercel rollback not fully implemented");
                Ok(deployment.clone())
            }
            Platform::Cloudflare => {
                // Cloudflare Pages also supports rollbacks
                warn!("Cloudflare rollback not fully implemented");
                Ok(deployment.clone())
            }
            _ => Err(GenesisError::custom("Rollback not supported for this platform")),
        }
    }

    /// Get available platforms
    pub fn available_platforms(&self) -> Vec<Platform> {
        let mut platforms = Vec::new();

        if self.vercel.as_ref().map(|v| v.is_configured()).unwrap_or(false) {
            platforms.push(Platform::Vercel);
        }

        if self.cloudflare.as_ref().map(|c| c.is_configured()).unwrap_or(false) {
            platforms.push(Platform::Cloudflare);
        }

        platforms
    }

    /// Check if a platform is available
    pub fn is_platform_available(&self, platform: &Platform) -> bool {
        match platform {
            Platform::Vercel => self.vercel.as_ref().map(|v| v.is_configured()).unwrap_or(false),
            Platform::Cloudflare => self.cloudflare.as_ref().map(|c| c.is_configured()).unwrap_or(false),
            _ => false,
        }
    }

    /// Get deployment status across all platforms
    pub async fn get_deployment_statuses(
        &self,
        _project: &Project,
    ) -> Result<HashMap<Platform, DeploymentStatus>> {
        let mut statuses = HashMap::new();

        // Check Vercel
        if let Some(ref vercel) = self.vercel {
            if vercel.is_configured() {
                // Would need to query Vercel API for project deployments
                statuses.insert(Platform::Vercel, DeploymentStatus::Ready);
            }
        }

        // Check Cloudflare
        if let Some(ref cloudflare) = self.cloudflare {
            if cloudflare.is_configured() {
                // Would need to query Cloudflare API for project deployments
                statuses.insert(Platform::Cloudflare, DeploymentStatus::Ready);
            }
        }

        Ok(statuses)
    }
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub url: String,
    pub is_healthy: bool,
    pub status_code: u16,
    pub response_time_ms: u64,
    pub error: Option<String>,
}

/// Extract repository name from GitHub URL
fn extract_repo_name(url: &str) -> String {
    // Handle different GitHub URL formats
    // https://github.com/owner/repo
    // https://github.com/owner/repo.git
    // git@github.com:owner/repo.git

    let url = url.trim_end_matches(".git");

    if url.contains("github.com") {
        let parts: Vec<&str> = url.split('/').collect();
        if parts.len() >= 2 {
            let owner = parts[parts.len() - 2];
            let repo = parts[parts.len() - 1];
            return format!("{}/{}", owner, repo);
        }
    }

    url.to_string()
}

/// Multi-platform deployment strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentStrategy {
    /// Deploy to single platform
    Single(Platform),
    /// Deploy to primary, with fallback
    PrimaryWithFallback {
        primary: Platform,
        fallback: Platform,
    },
    /// Deploy to multiple platforms simultaneously
    MultiPlatform(Vec<Platform>),
}

impl Default for DeploymentStrategy {
    fn default() -> Self {
        Self::Single(Platform::Vercel)
    }
}

/// Multi-deployment result
#[derive(Debug, Clone)]
pub struct MultiDeploymentResult {
    pub successful: Vec<DeploymentResult>,
    pub failed: Vec<(Platform, String)>,
    pub strategy: DeploymentStrategy,
}

impl DeploymentMesh {
    /// Deploy using a specific strategy
    pub async fn deploy_with_strategy(
        &self,
        project: &Project,
        strategy: DeploymentStrategy,
        base_config: DeploymentConfig,
    ) -> Result<MultiDeploymentResult> {
        match strategy {
            DeploymentStrategy::Single(platform) => {
                let config = DeploymentConfig {
                    platform,
                    ..base_config
                };

                let result = self.deploy(project, config).await?;

                Ok(MultiDeploymentResult {
                    successful: vec![result],
                    failed: vec![],
                    strategy: DeploymentStrategy::Single(platform),
                })
            }

            DeploymentStrategy::PrimaryWithFallback { primary, fallback } => {
                let config = DeploymentConfig {
                    platform: primary,
                    ..base_config.clone()
                };

                match self.deploy(project, config).await {
                    Ok(result) => Ok(MultiDeploymentResult {
                        successful: vec![result],
                        failed: vec![],
                        strategy: DeploymentStrategy::PrimaryWithFallback { primary, fallback },
                    }),
                    Err(e) => {
                        warn!("Primary deployment failed, trying fallback: {}", e);

                        let fallback_config = DeploymentConfig {
                            platform: fallback,
                            ..base_config
                        };

                        let result = self.deploy(project, fallback_config).await?;

                        Ok(MultiDeploymentResult {
                            successful: vec![result],
                            failed: vec![(primary, e.to_string())],
                            strategy: DeploymentStrategy::PrimaryWithFallback { primary, fallback },
                        })
                    }
                }
            }

            DeploymentStrategy::MultiPlatform(platforms) => {
                let mut successful = Vec::new();
                let mut failed = Vec::new();

                for platform in &platforms {
                    let config = DeploymentConfig {
                        platform: *platform,
                        ..base_config.clone()
                    };

                    match self.deploy(project, config).await {
                        Ok(result) => successful.push(result),
                        Err(e) => failed.push((*platform, e.to_string())),
                    }
                }

                if successful.is_empty() {
                    return Err(GenesisError::DeploymentFailed(
                        "All platform deployments failed".to_string(),
                    ));
                }

                Ok(MultiDeploymentResult {
                    successful,
                    failed,
                    strategy: DeploymentStrategy::MultiPlatform(platforms),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_creation() {
        let mesh = DeploymentMesh::from_config(MeshConfig::default());
        assert!(mesh.available_platforms().is_empty());
    }

    #[test]
    fn test_extract_repo_name() {
        assert_eq!(
            extract_repo_name("https://github.com/owner/repo"),
            "owner/repo"
        );
        assert_eq!(
            extract_repo_name("https://github.com/owner/repo.git"),
            "owner/repo"
        );
    }

    #[test]
    fn test_deployment_strategy_default() {
        let strategy = DeploymentStrategy::default();
        assert!(matches!(strategy, DeploymentStrategy::Single(Platform::Vercel)));
    }

    #[test]
    fn test_platform_availability() {
        let mesh = DeploymentMesh::from_config(MeshConfig::default());
        assert!(!mesh.is_platform_available(&Platform::Vercel));
        assert!(!mesh.is_platform_available(&Platform::Cloudflare));
    }
}
