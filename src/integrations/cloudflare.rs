//! Cloudflare Workers/Pages deployment client

use crate::models::{Deployment, DeploymentStatus, Platform};
use crate::utils::error::{GenesisError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// Cloudflare API client
#[derive(Clone)]
pub struct CloudflareClient {
    token: String,
    account_id: String,
    http_client: Client,
    base_url: String,
}

impl CloudflareClient {
    /// Create a new Cloudflare client
    pub fn new(token: String, account_id: String) -> Self {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            token,
            account_id,
            http_client,
            base_url: "https://api.cloudflare.com/client/v4".to_string(),
        }
    }

    /// Create from config
    pub fn from_config(token: &str, account_id: &str) -> Option<Self> {
        if token.is_empty() || account_id.is_empty() {
            return None;
        }

        Some(Self::new(token.to_string(), account_id.to_string()))
    }

    /// Create a new Pages project
    pub async fn create_pages_project(
        &self,
        name: &str,
        production_branch: &str,
    ) -> Result<PagesProject> {
        info!("Creating Cloudflare Pages project: {}", name);

        let url = format!(
            "{}/accounts/{}/pages/projects",
            self.base_url, self.account_id
        );

        let request = CreatePagesProjectRequest {
            name: name.to_string(),
            production_branch: production_branch.to_string(),
        };

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            // Check if project already exists
            if status.as_u16() == 409 {
                warn!("Project {} already exists, fetching existing", name);
                return self.get_pages_project(name).await;
            }

            return Err(GenesisError::api(
                format!("Cloudflare Pages project creation failed: {}", error_text),
                status.as_u16(),
            ));
        }

        let result: CloudflareResponse<PagesProject> = response.json().await?;

        if !result.success {
            return Err(GenesisError::custom("Cloudflare API returned unsuccessful response"));
        }

        let project = result.result.ok_or_else(|| {
            GenesisError::custom("No project in response")
        })?;

        info!("Created Pages project: {}", project.name);
        Ok(project)
    }

    /// Get an existing Pages project
    pub async fn get_pages_project(&self, name: &str) -> Result<PagesProject> {
        let url = format!(
            "{}/accounts/{}/pages/projects/{}",
            self.base_url, self.account_id, name
        );

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(GenesisError::api(
                "Pages project not found",
                response.status().as_u16(),
            ));
        }

        let result: CloudflareResponse<PagesProject> = response.json().await?;
        result.result.ok_or_else(|| {
            GenesisError::custom("No project in response")
        })
    }

    /// Create a Pages deployment
    pub async fn create_pages_deployment(
        &self,
        project_name: &str,
        branch: &str,
    ) -> Result<PagesDeployment> {
        info!("Creating Pages deployment for: {}", project_name);

        let url = format!(
            "{}/accounts/{}/pages/projects/{}/deployments",
            self.base_url, self.account_id, project_name
        );

        // For direct deployments, we'd need to upload assets
        // For GitHub integration, we trigger via webhook or wait for automatic deployment

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "branch": branch
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(GenesisError::api(
                format!("Pages deployment failed: {}", error_text),
                status.as_u16(),
            ));
        }

        let result: CloudflareResponse<PagesDeployment> = response.json().await?;
        result.result.ok_or_else(|| {
            GenesisError::custom("No deployment in response")
        })
    }

    /// Get deployment status
    pub async fn get_deployment_status(
        &self,
        project_name: &str,
        deployment_id: &str,
    ) -> Result<PagesDeployment> {
        let url = format!(
            "{}/accounts/{}/pages/projects/{}/deployments/{}",
            self.base_url, self.account_id, project_name, deployment_id
        );

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

        let result: CloudflareResponse<PagesDeployment> = response.json().await?;
        result.result.ok_or_else(|| {
            GenesisError::custom("No deployment in response")
        })
    }

    /// Wait for deployment to complete
    pub async fn wait_for_deployment(
        &self,
        project_name: &str,
        deployment_id: &str,
        timeout_secs: u64,
    ) -> Result<PagesDeployment> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(timeout_secs);

        loop {
            if start.elapsed() > timeout {
                return Err(GenesisError::custom("Deployment timed out"));
            }

            let deployment = self.get_deployment_status(project_name, deployment_id).await?;

            match deployment.latest_stage.name.as_str() {
                "deploy" if deployment.latest_stage.status == "success" => {
                    return Ok(deployment);
                }
                "failure" | "failed" => {
                    return Err(GenesisError::DeploymentFailed(
                        "Cloudflare deployment failed".to_string(),
                    ));
                }
                _ => {
                    debug!(
                        "Deployment {} stage: {} ({})",
                        deployment_id,
                        deployment.latest_stage.name,
                        deployment.latest_stage.status
                    );
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        }
    }

    /// List deployments for a project
    pub async fn list_deployments(&self, project_name: &str) -> Result<Vec<PagesDeployment>> {
        let url = format!(
            "{}/accounts/{}/pages/projects/{}/deployments",
            self.base_url, self.account_id, project_name
        );

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(Vec::new());
        }

        let result: CloudflareResponse<Vec<PagesDeployment>> = response.json().await?;
        Ok(result.result.unwrap_or_default())
    }

    /// Delete a Pages project
    pub async fn delete_project(&self, name: &str) -> Result<()> {
        let url = format!(
            "{}/accounts/{}/pages/projects/{}",
            self.base_url, self.account_id, name
        );

        let response = self
            .http_client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await?;

        if !response.status().is_success() {
            warn!("Failed to delete project: {}", response.status());
        }

        Ok(())
    }

    /// Set environment variables for a project
    pub async fn set_env_vars(
        &self,
        project_name: &str,
        env_vars: Vec<EnvironmentVariable>,
    ) -> Result<()> {
        for env_var in env_vars {
            let url = format!(
                "{}/accounts/{}/pages/projects/{}/deployments/production/env",
                self.base_url, self.account_id, project_name
            );

            let request = serde_json::json!({
                "name": env_var.key,
                "value": env_var.value,
                "type": if env_var.is_secret { "secret_text" } else { "plain_text" }
            });

            let response = self
                .http_client
                .put(&url)
                .header("Authorization", format!("Bearer {}", self.token))
                .json(&request)
                .send()
                .await?;

            if !response.status().is_success() {
                warn!("Failed to set env var {}: {}", env_var.key, response.status());
            }
        }

        Ok(())
    }

    /// Create a Worker
    pub async fn create_worker(&self, name: &str, script: &str) -> Result<Worker> {
        info!("Creating Cloudflare Worker: {}", name);

        let url = format!(
            "{}/accounts/{}/workers/scripts/{}",
            self.base_url, self.account_id, name
        );

        let response = self
            .http_client
            .put(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/javascript")
            .body(script.to_string())
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(GenesisError::api(
                format!("Worker creation failed: {}", error_text),
                status.as_u16(),
            ));
        }

        let result: CloudflareResponse<Worker> = response.json().await?;
        result.result.ok_or_else(|| {
            GenesisError::custom("No worker in response")
        })
    }

    /// Add a route for a Worker
    pub async fn add_worker_route(
        &self,
        zone_id: &str,
        pattern: &str,
        script_name: &str,
    ) -> Result<()> {
        let url = format!(
            "{}/zones/{}/workers/routes",
            self.base_url, zone_id
        );

        let request = serde_json::json!({
            "pattern": pattern,
            "script": script_name
        });

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            warn!("Failed to add worker route: {}", response.status());
        }

        Ok(())
    }

    /// Check if client is configured
    pub fn is_configured(&self) -> bool {
        !self.token.is_empty() && !self.account_id.is_empty()
    }
}

/// Convert Pages deployment to our Deployment model
impl From<PagesDeployment> for Deployment {
    fn from(pages: PagesDeployment) -> Self {
        let mut deployment = Deployment::new(&pages.project_name, Platform::Cloudflare);

        deployment.set_platform_id(&pages.id);

        if let Some(url) = pages.url {
            deployment.set_url(url);
        }

        let status = match pages.latest_stage.status.as_str() {
            "success" => DeploymentStatus::Ready,
            "active" | "idle" => DeploymentStatus::Building,
            "failure" | "failed" => DeploymentStatus::Failed,
            "canceled" => DeploymentStatus::Cancelled,
            _ => DeploymentStatus::Queued,
        };
        deployment.set_status(status);

        deployment
    }
}

// API Types

#[derive(Debug, Deserialize)]
struct CloudflareResponse<T> {
    success: bool,
    result: Option<T>,
    #[allow(dead_code)]
    errors: Vec<CloudflareError>,
    #[allow(dead_code)]
    messages: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct CloudflareError {
    #[allow(dead_code)]
    code: i32,
    #[allow(dead_code)]
    message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PagesProject {
    pub name: String,
    pub subdomain: String,
    pub production_branch: String,
    pub created_on: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PagesDeployment {
    pub id: String,
    pub project_name: String,
    pub url: Option<String>,
    pub latest_stage: DeploymentStage,
    pub created_on: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeploymentStage {
    pub name: String,
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Worker {
    pub id: String,
    pub etag: String,
}

#[derive(Debug, Serialize)]
struct CreatePagesProjectRequest {
    name: String,
    production_branch: String,
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
        let client = CloudflareClient::new(
            "test-token".to_string(),
            "account-123".to_string(),
        );
        assert!(client.is_configured());
    }

    #[test]
    fn test_from_config() {
        let client = CloudflareClient::from_config("token", "account");
        assert!(client.is_some());

        let empty_client = CloudflareClient::from_config("", "");
        assert!(empty_client.is_none());

        let partial_client = CloudflareClient::from_config("token", "");
        assert!(partial_client.is_none());
    }

    #[test]
    fn test_env_var_creation() {
        let env = EnvironmentVariable::new("API_KEY", "value123");
        assert!(!env.is_secret);

        let secret = EnvironmentVariable::secret("SECRET_KEY", "secret123");
        assert!(secret.is_secret);
    }
}
