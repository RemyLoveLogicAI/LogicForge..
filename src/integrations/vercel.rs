//! Vercel deployment client

use crate::models::{Deployment, DeploymentStatus, Platform};
use crate::utils::error::{GenesisError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// Vercel API client
#[derive(Clone)]
pub struct VercelClient {
    token: String,
    team_id: Option<String>,
    http_client: Client,
    base_url: String,
}

impl VercelClient {
    /// Create a new Vercel client
    pub fn new(token: String, team_id: Option<String>) -> Self {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            token,
            team_id,
            http_client,
            base_url: "https://api.vercel.com".to_string(),
        }
    }

    /// Create from config
    pub fn from_config(token: &str, team_id: Option<&str>) -> Option<Self> {
        if token.is_empty() {
            return None;
        }

        Some(Self::new(token.to_string(), team_id.map(String::from)))
    }

    /// Deploy a project from a GitHub repository
    pub async fn deploy_from_git(
        &self,
        project_name: &str,
        github_repo: &str,
        branch: &str,
    ) -> Result<VercelDeployment> {
        info!("Deploying {} from {} (branch: {})", project_name, github_repo, branch);

        // First, create or get the project
        let project = self.create_or_get_project(project_name, github_repo).await?;

        // Trigger deployment
        let deployment = self.trigger_deployment(&project.id, github_repo, branch).await?;

        Ok(deployment)
    }

    /// Create or get an existing project
    async fn create_or_get_project(
        &self,
        name: &str,
        github_repo: &str,
    ) -> Result<VercelProject> {
        // Try to get existing project first
        if let Ok(project) = self.get_project(name).await {
            return Ok(project);
        }

        // Create new project
        let mut url = format!("{}/v10/projects", self.base_url);
        if let Some(ref team_id) = self.team_id {
            url.push_str(&format!("?teamId={}", team_id));
        }

        let request = CreateProjectRequest {
            name: name.to_string(),
            git_repository: Some(GitRepository {
                repo: github_repo.to_string(),
                type_: "github".to_string(),
            }),
            framework: None,
            build_command: None,
            output_directory: None,
        };

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(GenesisError::api(
                format!("Vercel project creation failed: {}", error_text),
                status.as_u16(),
            ));
        }

        let project: VercelProject = response.json().await?;
        info!("Created Vercel project: {}", project.id);

        Ok(project)
    }

    /// Get an existing project
    async fn get_project(&self, name: &str) -> Result<VercelProject> {
        let mut url = format!("{}/v9/projects/{}", self.base_url, name);
        if let Some(ref team_id) = self.team_id {
            url.push_str(&format!("?teamId={}", team_id));
        }

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(GenesisError::api(
                "Project not found",
                response.status().as_u16(),
            ));
        }

        let project: VercelProject = response.json().await?;
        Ok(project)
    }

    /// Trigger a deployment
    async fn trigger_deployment(
        &self,
        project_id: &str,
        github_repo: &str,
        branch: &str,
    ) -> Result<VercelDeployment> {
        let mut url = format!("{}/v13/deployments", self.base_url);
        if let Some(ref team_id) = self.team_id {
            url.push_str(&format!("?teamId={}", team_id));
        }

        let request = CreateDeploymentRequest {
            name: project_id.to_string(),
            git_source: Some(GitSource {
                type_: "github".to_string(),
                repo: github_repo.to_string(),
                ref_: branch.to_string(),
            }),
            project: project_id.to_string(),
        };

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(GenesisError::api(
                format!("Vercel deployment failed: {}", error_text),
                status.as_u16(),
            ));
        }

        let deployment: VercelDeployment = response.json().await?;
        info!("Created Vercel deployment: {}", deployment.id);

        Ok(deployment)
    }

    /// Get deployment status
    pub async fn get_deployment_status(&self, deployment_id: &str) -> Result<VercelDeployment> {
        let mut url = format!("{}/v13/deployments/{}", self.base_url, deployment_id);
        if let Some(ref team_id) = self.team_id {
            url.push_str(&format!("?teamId={}", team_id));
        }

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(GenesisError::api(
                "Failed to get deployment status",
                response.status().as_u16(),
            ));
        }

        let deployment: VercelDeployment = response.json().await?;
        Ok(deployment)
    }

    /// Wait for deployment to complete
    pub async fn wait_for_deployment(
        &self,
        deployment_id: &str,
        timeout_secs: u64,
    ) -> Result<VercelDeployment> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(timeout_secs);

        loop {
            if start.elapsed() > timeout {
                return Err(GenesisError::custom("Deployment timed out"));
            }

            let deployment = self.get_deployment_status(deployment_id).await?;

            match deployment.ready_state.as_str() {
                "READY" => return Ok(deployment),
                "ERROR" | "CANCELED" => {
                    return Err(GenesisError::DeploymentFailed(
                        deployment.error_message.unwrap_or_else(|| "Unknown error".to_string()),
                    ));
                }
                _ => {
                    debug!("Deployment {} status: {}", deployment_id, deployment.ready_state);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        }
    }

    /// List deployments for a project
    pub async fn list_deployments(&self, project_id: &str) -> Result<Vec<VercelDeployment>> {
        let mut url = format!(
            "{}/v6/deployments?projectId={}",
            self.base_url, project_id
        );
        if let Some(ref team_id) = self.team_id {
            url.push_str(&format!("&teamId={}", team_id));
        }

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(Vec::new());
        }

        let result: DeploymentsResponse = response.json().await?;
        Ok(result.deployments)
    }

    /// Delete a deployment
    pub async fn delete_deployment(&self, deployment_id: &str) -> Result<()> {
        let mut url = format!("{}/v13/deployments/{}", self.base_url, deployment_id);
        if let Some(ref team_id) = self.team_id {
            url.push_str(&format!("?teamId={}", team_id));
        }

        let response = self
            .http_client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await?;

        if !response.status().is_success() {
            warn!("Failed to delete deployment: {}", response.status());
        }

        Ok(())
    }

    /// Set environment variables for a project
    pub async fn set_env_vars(
        &self,
        project_id: &str,
        env_vars: Vec<EnvironmentVariable>,
    ) -> Result<()> {
        let mut url = format!("{}/v10/projects/{}/env", self.base_url, project_id);
        if let Some(ref team_id) = self.team_id {
            url.push_str(&format!("?teamId={}", team_id));
        }

        for env_var in env_vars {
            let request = CreateEnvVarRequest {
                key: env_var.key,
                value: env_var.value,
                target: vec!["production".to_string(), "preview".to_string()],
                type_: if env_var.is_secret { "encrypted" } else { "plain" }.to_string(),
            };

            let response = self
                .http_client
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.token))
                .json(&request)
                .send()
                .await?;

            if !response.status().is_success() {
                warn!("Failed to set env var: {}", response.status());
            }
        }

        Ok(())
    }

    /// Check if client is configured
    pub fn is_configured(&self) -> bool {
        !self.token.is_empty()
    }
}

/// Convert Vercel deployment to our Deployment model
impl From<VercelDeployment> for Deployment {
    fn from(vercel: VercelDeployment) -> Self {
        let mut deployment = Deployment::new(vercel.project_id.unwrap_or_default(), Platform::Vercel);

        deployment.set_platform_id(&vercel.id);

        if let Some(url) = vercel.url {
            deployment.set_url(format!("https://{}", url));
        }

        let status = match vercel.ready_state.as_str() {
            "READY" => DeploymentStatus::Ready,
            "BUILDING" => DeploymentStatus::Building,
            "INITIALIZING" | "ANALYZING" => DeploymentStatus::Queued,
            "ERROR" | "CANCELED" => DeploymentStatus::Failed,
            _ => DeploymentStatus::Deploying,
        };
        deployment.set_status(status);

        if let Some(err) = vercel.error_message {
            deployment.error_message = Some(err);
        }

        deployment
    }
}

// API Types

#[derive(Debug, Clone, Deserialize)]
pub struct VercelProject {
    pub id: String,
    pub name: String,
    pub framework: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VercelDeployment {
    pub id: String,
    pub url: Option<String>,
    #[serde(rename = "readyState")]
    pub ready_state: String,
    #[serde(rename = "projectId")]
    pub project_id: Option<String>,
    #[serde(rename = "errorMessage")]
    pub error_message: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct DeploymentsResponse {
    deployments: Vec<VercelDeployment>,
}

#[derive(Debug, Serialize)]
struct CreateProjectRequest {
    name: String,
    #[serde(rename = "gitRepository")]
    git_repository: Option<GitRepository>,
    framework: Option<String>,
    #[serde(rename = "buildCommand")]
    build_command: Option<String>,
    #[serde(rename = "outputDirectory")]
    output_directory: Option<String>,
}

#[derive(Debug, Serialize)]
struct GitRepository {
    repo: String,
    #[serde(rename = "type")]
    type_: String,
}

#[derive(Debug, Serialize)]
struct CreateDeploymentRequest {
    name: String,
    #[serde(rename = "gitSource")]
    git_source: Option<GitSource>,
    project: String,
}

#[derive(Debug, Serialize)]
struct GitSource {
    #[serde(rename = "type")]
    type_: String,
    repo: String,
    #[serde(rename = "ref")]
    ref_: String,
}

#[derive(Debug, Serialize)]
struct CreateEnvVarRequest {
    key: String,
    value: String,
    target: Vec<String>,
    #[serde(rename = "type")]
    type_: String,
}

/// Environment variable configuration
#[derive(Debug, Clone)]
pub struct EnvironmentVariable {
    pub key: String,
    pub value: String,
    pub is_secret: bool,
}

impl EnvironmentVariable {
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            is_secret: false,
        }
    }

    pub fn secret(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            is_secret: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = VercelClient::new("test-token".to_string(), Some("team-123".to_string()));
        assert!(client.is_configured());
    }

    #[test]
    fn test_from_config() {
        let client = VercelClient::from_config("token", Some("team"));
        assert!(client.is_some());

        let empty_client = VercelClient::from_config("", None);
        assert!(empty_client.is_none());
    }

    #[test]
    fn test_env_var_creation() {
        let env = EnvironmentVariable::new("API_KEY", "value123");
        assert!(!env.is_secret);

        let secret = EnvironmentVariable::secret("SECRET_KEY", "secret123");
        assert!(secret.is_secret);
    }
}
