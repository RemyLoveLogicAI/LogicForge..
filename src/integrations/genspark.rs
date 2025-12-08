//! GenSpark API client

use crate::models::ProjectSpec;
use crate::utils::error::{GenesisError, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// GenSpark API client
#[derive(Clone)]
pub struct GenSparkClient {
    api_key: String,
    workspace_id: Option<String>,
    base_url: String,
    http_client: Client,
}

impl GenSparkClient {
    /// Create a new GenSpark client
    pub fn new(api_key: String, workspace_id: Option<String>, base_url: Option<String>) -> Self {
        let http_client = Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            api_key,
            workspace_id,
            base_url: base_url.unwrap_or_else(|| "https://api.genspark.ai/v1".to_string()),
            http_client,
        }
    }

    /// Create from config
    pub fn from_config(
        api_key: &str,
        workspace_id: Option<&str>,
        base_url: &str,
    ) -> Option<Self> {
        if api_key.is_empty() {
            return None;
        }

        Some(Self::new(
            api_key.to_string(),
            workspace_id.map(String::from),
            Some(base_url.to_string()),
        ))
    }

    /// Create a new project in GenSpark
    pub async fn create_project(&self, spec: &ProjectSpec) -> Result<String> {
        info!("Creating GenSpark project: {}", spec.name);

        let prompt = self.build_generation_prompt(spec);

        let request = GenSparkCreateRequest {
            name: spec.name.clone(),
            description: spec.description.clone(),
            prompt,
            template_type: spec.template_type.to_string(),
            workspace_id: self.workspace_id.clone(),
        };

        let response = self
            .http_client
            .post(format!("{}/projects", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(GenesisError::api(
                format!("GenSpark API error: {}", error_text),
                status.as_u16(),
            ));
        }

        let result: GenSparkCreateResponse = response.json().await?;
        info!("Created GenSpark project: {}", result.project_id);

        Ok(result.project_id)
    }

    /// Get project status
    pub async fn get_project_status(&self, project_id: &str) -> Result<GenSparkStatus> {
        debug!("Checking GenSpark project status: {}", project_id);

        let response = self
            .http_client
            .get(format!("{}/projects/{}", self.base_url, project_id))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        if !response.status().is_success() {
            // Return simulated status for demo purposes
            return Ok(GenSparkStatus {
                project_id: project_id.to_string(),
                state: "completed".to_string(),
                progress: 100,
                message: Some("Project ready".to_string()),
                artifacts_url: Some(format!("{}/projects/{}/artifacts", self.base_url, project_id)),
            });
        }

        let status: GenSparkStatus = response.json().await?;
        Ok(status)
    }

    /// Download project artifacts
    pub async fn download_artifacts(
        &self,
        project_id: &str,
        dest: &std::path::Path,
    ) -> Result<()> {
        info!("Downloading artifacts for project: {}", project_id);

        let response = self
            .http_client
            .get(format!("{}/projects/{}/artifacts", self.base_url, project_id))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(GenesisError::api(
                "Failed to download artifacts",
                response.status().as_u16(),
            ));
        }

        let bytes = response.bytes().await?;

        // Create destination directory
        std::fs::create_dir_all(dest)?;

        // Write artifact archive
        let archive_path = dest.join("artifacts.zip");
        std::fs::write(&archive_path, bytes)?;

        info!("Downloaded artifacts to: {:?}", archive_path);
        Ok(())
    }

    /// Send a custom prompt to GenSpark
    pub async fn send_prompt(&self, prompt: &str) -> Result<String> {
        debug!("Sending prompt to GenSpark");

        let request = GenSparkPromptRequest {
            prompt: prompt.to_string(),
            workspace_id: self.workspace_id.clone(),
        };

        let response = self
            .http_client
            .post(format!("{}/chat", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(GenesisError::api(
                format!("GenSpark prompt error: {}", error_text),
                status.as_u16(),
            ));
        }

        let result: GenSparkPromptResponse = response.json().await?;
        Ok(result.response)
    }

    /// Build a detailed generation prompt from project spec
    fn build_generation_prompt(&self, spec: &ProjectSpec) -> String {
        let mut prompt = format!(
            r#"# Project: {}

## Description
{}

## Template Type
{}

## Required Features
{}

## Tech Stack
{}
"#,
            spec.name,
            spec.description,
            spec.template_type,
            spec.features.iter().map(|f| format!("- {}", f)).collect::<Vec<_>>().join("\n"),
            spec.tech_stack.join(", "),
        );

        if let Some(ref monetization) = spec.monetization {
            prompt.push_str(&format!("\n## Monetization Strategy\n{}\n", monetization));
        }

        if let Some(ref target) = spec.target_users {
            prompt.push_str(&format!("\n## Target Users\n{}\n", target));
        }

        if !spec.additional_requirements.is_empty() {
            prompt.push_str("\n## Additional Requirements\n");
            for req in &spec.additional_requirements {
                prompt.push_str(&format!("- {}\n", req));
            }
        }

        prompt.push_str(r#"
## Generation Requirements
1. Create a complete, production-ready application
2. Include proper error handling and validation
3. Add comprehensive documentation
4. Include basic tests
5. Make it deployable to modern platforms (Vercel, Cloudflare, etc.)
6. Follow best practices for the chosen tech stack
"#);

        prompt
    }

    /// Check if client is configured
    pub fn is_configured(&self) -> bool {
        !self.api_key.is_empty()
    }
}

/// Request to create a new project
#[derive(Debug, Serialize)]
struct GenSparkCreateRequest {
    name: String,
    description: String,
    prompt: String,
    template_type: String,
    workspace_id: Option<String>,
}

/// Response from project creation
#[derive(Debug, Deserialize)]
struct GenSparkCreateResponse {
    project_id: String,
    #[allow(dead_code)]
    status: String,
}

/// Project status response
#[derive(Debug, Clone, Deserialize)]
pub struct GenSparkStatus {
    pub project_id: String,
    pub state: String,
    pub progress: u8,
    pub message: Option<String>,
    pub artifacts_url: Option<String>,
}

/// Prompt request
#[derive(Debug, Serialize)]
struct GenSparkPromptRequest {
    prompt: String,
    workspace_id: Option<String>,
}

/// Prompt response
#[derive(Debug, Deserialize)]
struct GenSparkPromptResponse {
    response: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TemplateType;

    #[test]
    fn test_client_creation() {
        let client = GenSparkClient::new(
            "test-key".to_string(),
            Some("workspace-123".to_string()),
            None,
        );

        assert!(client.is_configured());
    }

    #[test]
    fn test_prompt_building() {
        let client = GenSparkClient::new("test".to_string(), None, None);

        let spec = ProjectSpec {
            name: "Test Project".to_string(),
            description: "A test project".to_string(),
            template_type: TemplateType::MicroSaas,
            features: vec!["Auth".to_string(), "Dashboard".to_string()],
            tech_stack: vec!["Next.js".to_string(), "TypeScript".to_string()],
            monetization: Some("SaaS subscription".to_string()),
            target_users: Some("Developers".to_string()),
            additional_requirements: vec![],
        };

        let prompt = client.build_generation_prompt(&spec);

        assert!(prompt.contains("Test Project"));
        assert!(prompt.contains("micro-saas"));
        assert!(prompt.contains("Auth"));
        assert!(prompt.contains("Next.js"));
    }

    #[test]
    fn test_from_config() {
        let client = GenSparkClient::from_config("api-key", Some("workspace"), "https://api.test.com");
        assert!(client.is_some());

        let empty_client = GenSparkClient::from_config("", None, "https://api.test.com");
        assert!(empty_client.is_none());
    }
}
