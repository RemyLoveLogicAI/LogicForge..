//! GitHub API client

use crate::utils::error::{GenesisError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// GitHub API client
#[derive(Clone)]
pub struct GitHubClient {
    token: String,
    org: Option<String>,
    http_client: Client,
    base_url: String,
}

impl GitHubClient {
    /// Create a new GitHub client
    pub fn new(token: String, org: Option<String>) -> Self {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("Genesis-Engine/0.1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            token,
            org,
            http_client,
            base_url: "https://api.github.com".to_string(),
        }
    }

    /// Create from config
    pub fn from_config(token: &str, org: Option<&str>) -> Option<Self> {
        if token.is_empty() {
            return None;
        }

        Some(Self::new(token.to_string(), org.map(String::from)))
    }

    /// Create a new repository
    pub async fn create_repository(
        &self,
        name: &str,
        description: &str,
    ) -> Result<Repository> {
        info!("Creating GitHub repository: {}", name);

        let request = CreateRepoRequest {
            name: name.to_string(),
            description: Some(description.to_string()),
            private: false,
            auto_init: true,
            gitignore_template: Some("Node".to_string()),
            license_template: Some("mit".to_string()),
        };

        let url = if let Some(ref org) = self.org {
            format!("{}/orgs/{}/repos", self.base_url, org)
        } else {
            format!("{}/user/repos", self.base_url)
        };

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github.v3+json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();

            // Check if repo already exists
            if status.as_u16() == 422 && error_text.contains("already exists") {
                warn!("Repository {} already exists, fetching existing repo", name);
                return self.get_repository(name).await;
            }

            return Err(GenesisError::api(
                format!("GitHub API error: {}", error_text),
                status.as_u16(),
            ));
        }

        let repo: Repository = response.json().await?;
        info!("Created repository: {}", repo.html_url);

        Ok(repo)
    }

    /// Get an existing repository
    pub async fn get_repository(&self, name: &str) -> Result<Repository> {
        let owner = self.org.as_deref().unwrap_or("user");
        let url = format!("{}/repos/{}/{}", self.base_url, owner, name);

        let response = self
            .http_client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(GenesisError::api(
                "Repository not found",
                response.status().as_u16(),
            ));
        }

        let repo: Repository = response.json().await?;
        Ok(repo)
    }

    /// Setup branch protection for main branch
    pub async fn setup_branch_protection(&self, repo_full_name: &str) -> Result<()> {
        info!("Setting up branch protection for: {}", repo_full_name);

        let url = format!(
            "{}/repos/{}/branches/main/protection",
            self.base_url, repo_full_name
        );

        let protection = BranchProtection {
            required_status_checks: Some(RequiredStatusChecks {
                strict: true,
                contexts: vec!["ci".to_string()],
            }),
            enforce_admins: Some(true),
            required_pull_request_reviews: Some(RequiredPullRequestReviews {
                dismiss_stale_reviews: true,
                require_code_owner_reviews: false,
                required_approving_review_count: 1,
            }),
            restrictions: None,
        };

        let response = self
            .http_client
            .put(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github.v3+json")
            .json(&protection)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            warn!("Failed to set branch protection: {} - {}", status, error_text);
            // Don't fail the operation, just warn
        }

        Ok(())
    }

    /// Create a workflow file in the repository
    pub async fn create_workflow_file(
        &self,
        repo_full_name: &str,
        workflow_name: &str,
        content: &str,
    ) -> Result<()> {
        info!("Creating workflow file: {}", workflow_name);

        let path = format!(".github/workflows/{}.yml", workflow_name);
        let encoded_content = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            content,
        );

        let url = format!(
            "{}/repos/{}/contents/{}",
            self.base_url, repo_full_name, path
        );

        let request = CreateFileRequest {
            message: format!("Add {} workflow", workflow_name),
            content: encoded_content,
            branch: "main".to_string(),
        };

        let response = self
            .http_client
            .put(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github.v3+json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            warn!("Failed to create workflow file: {}", status);
        }

        Ok(())
    }

    /// Create default CI workflow
    pub async fn create_ci_workflow(&self, repo_full_name: &str) -> Result<()> {
        let ci_content = r#"name: CI

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
      - name: Install dependencies
        run: npm ci
      - name: Run tests
        run: npm test
      - name: Build
        run: npm run build
"#;

        self.create_workflow_file(repo_full_name, "ci", ci_content).await
    }

    /// Push files to repository
    pub async fn push_files(
        &self,
        repo_full_name: &str,
        files: Vec<FileContent>,
        commit_message: &str,
    ) -> Result<()> {
        info!("Pushing {} files to {}", files.len(), repo_full_name);

        for file in files {
            let encoded_content = base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                &file.content,
            );

            let url = format!(
                "{}/repos/{}/contents/{}",
                self.base_url, repo_full_name, file.path
            );

            let request = CreateFileRequest {
                message: commit_message.to_string(),
                content: encoded_content,
                branch: "main".to_string(),
            };

            let response = self
                .http_client
                .put(&url)
                .header("Authorization", format!("Bearer {}", self.token))
                .header("Accept", "application/vnd.github.v3+json")
                .json(&request)
                .send()
                .await?;

            if !response.status().is_success() {
                let status = response.status();
                let error = response.text().await.unwrap_or_default();
                warn!("Failed to push file {}: {} - {}", file.path, status, error);
            }
        }

        Ok(())
    }

    /// Create a webhook for the repository
    pub async fn create_webhook(
        &self,
        repo_full_name: &str,
        webhook_url: &str,
        events: Vec<String>,
    ) -> Result<()> {
        debug!("Creating webhook for: {}", repo_full_name);

        let url = format!("{}/repos/{}/hooks", self.base_url, repo_full_name);

        let request = CreateWebhookRequest {
            name: "web".to_string(),
            config: WebhookConfig {
                url: webhook_url.to_string(),
                content_type: "json".to_string(),
                insecure_ssl: "0".to_string(),
            },
            events,
            active: true,
        };

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github.v3+json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            warn!("Failed to create webhook: {}", status);
        }

        Ok(())
    }

    /// Check if client is configured
    pub fn is_configured(&self) -> bool {
        !self.token.is_empty()
    }

    /// Get authenticated user
    pub async fn get_authenticated_user(&self) -> Result<GitHubUser> {
        let response = self
            .http_client
            .get(format!("{}/user", self.base_url))
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(GenesisError::AuthenticationFailed(
                "Invalid GitHub token".to_string(),
            ));
        }

        let user: GitHubUser = response.json().await?;
        Ok(user)
    }
}

/// Repository information
#[derive(Debug, Clone, Deserialize)]
pub struct Repository {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub html_url: String,
    pub clone_url: String,
    pub default_branch: String,
}

/// GitHub user
#[derive(Debug, Clone, Deserialize)]
pub struct GitHubUser {
    pub login: String,
    pub id: u64,
    pub name: Option<String>,
    pub email: Option<String>,
}

/// File content for pushing
#[derive(Debug, Clone)]
pub struct FileContent {
    pub path: String,
    pub content: String,
}

// Request/Response structures

#[derive(Debug, Serialize)]
struct CreateRepoRequest {
    name: String,
    description: Option<String>,
    private: bool,
    auto_init: bool,
    gitignore_template: Option<String>,
    license_template: Option<String>,
}

#[derive(Debug, Serialize)]
struct BranchProtection {
    required_status_checks: Option<RequiredStatusChecks>,
    enforce_admins: Option<bool>,
    required_pull_request_reviews: Option<RequiredPullRequestReviews>,
    restrictions: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct RequiredStatusChecks {
    strict: bool,
    contexts: Vec<String>,
}

#[derive(Debug, Serialize)]
struct RequiredPullRequestReviews {
    dismiss_stale_reviews: bool,
    require_code_owner_reviews: bool,
    required_approving_review_count: u8,
}

#[derive(Debug, Serialize)]
struct CreateFileRequest {
    message: String,
    content: String,
    branch: String,
}

#[derive(Debug, Serialize)]
struct CreateWebhookRequest {
    name: String,
    config: WebhookConfig,
    events: Vec<String>,
    active: bool,
}

#[derive(Debug, Serialize)]
struct WebhookConfig {
    url: String,
    content_type: String,
    insecure_ssl: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = GitHubClient::new("test-token".to_string(), Some("test-org".to_string()));
        assert!(client.is_configured());
    }

    #[test]
    fn test_from_config() {
        let client = GitHubClient::from_config("token", Some("org"));
        assert!(client.is_some());

        let empty_client = GitHubClient::from_config("", None);
        assert!(empty_client.is_none());
    }
}
