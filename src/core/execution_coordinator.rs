//! Execution Coordinator - Orchestrates project lifecycle

use crate::db::Database;
use crate::integrations::{GenSparkClient, GitHubClient};
use crate::models::{
    Deployment, DeploymentStatus, Opportunity, Platform, Project,
    ProjectSpec, ProjectStatus, TemplateType,
};
use crate::utils::error::{GenesisError, Result};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

/// Configuration for the execution coordinator
#[derive(Debug, Clone)]
pub struct CoordinatorConfig {
    /// Enable automatic deployment after generation
    pub auto_deploy: bool,
    /// Default deployment platform
    pub default_platform: Platform,
    /// Enable automatic code review
    pub auto_review: bool,
    /// Maximum retries for failed operations
    pub max_retries: u32,
    /// Timeout for operations in seconds
    pub operation_timeout_secs: u64,
}

impl Default for CoordinatorConfig {
    fn default() -> Self {
        Self {
            auto_deploy: false,
            default_platform: Platform::Vercel,
            auto_review: true,
            max_retries: 3,
            operation_timeout_secs: 300,
        }
    }
}

/// Project launch options
#[derive(Debug, Clone)]
pub struct LaunchOptions {
    pub name: Option<String>,
    pub template: Option<TemplateType>,
    pub auto_deploy: bool,
    pub platform: Platform,
    pub skip_review: bool,
}

impl Default for LaunchOptions {
    fn default() -> Self {
        Self {
            name: None,
            template: None,
            auto_deploy: false,
            platform: Platform::Vercel,
            skip_review: false,
        }
    }
}

/// Execution coordinator for managing project lifecycle
pub struct ExecutionCoordinator {
    config: CoordinatorConfig,
    db: Arc<Database>,
    genspark: Option<GenSparkClient>,
    github: Option<GitHubClient>,
}

impl ExecutionCoordinator {
    /// Create a new execution coordinator
    pub fn new(
        config: CoordinatorConfig,
        db: Arc<Database>,
        genspark: Option<GenSparkClient>,
        github: Option<GitHubClient>,
    ) -> Self {
        Self {
            config,
            db,
            genspark,
            github,
        }
    }

    /// Launch a new project from an opportunity
    pub async fn launch_project(
        &self,
        opportunity: &Opportunity,
        options: LaunchOptions,
    ) -> Result<Project> {
        info!("Launching project from opportunity: {}", opportunity.id);

        // Generate project name
        let name = options
            .name
            .unwrap_or_else(|| self.generate_project_name(&opportunity.title));

        // Determine template type
        let template = options.template.unwrap_or_else(|| {
            self.suggest_template_for_opportunity(opportunity)
        });

        // Create project
        let mut project = Project::from_opportunity(&opportunity.id, &name);
        project.template_type = template.clone();
        project.description = opportunity.description.clone();

        // Create project specification
        let spec = self.create_project_spec(opportunity, &template);
        project.spec = Some(spec.clone());

        // Save initial project state
        self.db.save_project(&project)?;
        info!("Created project: {} ({})", project.id, project.name);

        // Step 1: Create GitHub repository (if configured)
        if let Some(ref github) = self.github {
            match self.create_github_repo(&project, github).await {
                Ok(repo_url) => {
                    project.set_github_repo(&repo_url);
                    self.db.save_project(&project)?;
                    info!("Created GitHub repository: {}", repo_url);
                }
                Err(e) => {
                    warn!("Failed to create GitHub repo: {}", e);
                    project.log_error(format!("GitHub repo creation failed: {}", e));
                }
            }
        }

        // Step 2: Generate code via GenSpark (if configured)
        if let Some(ref genspark) = self.genspark {
            project.set_status(ProjectStatus::Generating);
            self.db.save_project(&project)?;

            match self.generate_code(&project, &spec, genspark).await {
                Ok(genspark_id) => {
                    project.genspark_project_id = Some(genspark_id);
                    project.set_status(ProjectStatus::Generated);
                    self.db.save_project(&project)?;
                    info!("Code generation completed");
                }
                Err(e) => {
                    error!("Code generation failed: {}", e);
                    project.log_error(format!("Code generation failed: {}", e));
                    project.set_status(ProjectStatus::Failed);
                    self.db.save_project(&project)?;
                    return Ok(project);
                }
            }
        } else {
            // Skip generation, mark as ready for manual development
            project.set_status(ProjectStatus::Generated);
            self.db.save_project(&project)?;
        }

        // Step 3: Review generated code (if enabled)
        if self.config.auto_review && !options.skip_review {
            project.set_status(ProjectStatus::Reviewing);
            self.db.save_project(&project)?;

            // In production, this would call an AI to review the code
            // For now, we'll simulate a quick review
            debug!("Reviewing generated code...");

            // Mark review as complete
            project.set_status(ProjectStatus::Generated);
            self.db.save_project(&project)?;
        }

        // Step 4: Deploy (if auto-deploy enabled)
        if options.auto_deploy || self.config.auto_deploy {
            match self.deploy_project(&project, options.platform).await {
                Ok(deployment) => {
                    project.set_deployment_url(
                        deployment.primary_url().unwrap_or_default()
                    );
                    project.set_status(ProjectStatus::Live);
                    info!("Deployment successful: {:?}", deployment.primary_url());
                }
                Err(e) => {
                    warn!("Deployment failed: {}", e);
                    project.log_error(format!("Deployment failed: {}", e));
                }
            }
            self.db.save_project(&project)?;
        }

        Ok(project)
    }

    /// Create GitHub repository for a project
    async fn create_github_repo(
        &self,
        project: &Project,
        github: &GitHubClient,
    ) -> Result<String> {
        let description = format!(
            "{} - Generated by Genesis Engine",
            project.description
        );

        let repo = github.create_repository(&project.name, &description).await?;

        // Set up branch protection
        if let Err(e) = github.setup_branch_protection(&repo.full_name).await {
            warn!("Failed to setup branch protection: {}", e);
        }

        Ok(repo.html_url)
    }

    /// Generate code using GenSpark
    async fn generate_code(
        &self,
        _project: &Project,
        spec: &ProjectSpec,
        genspark: &GenSparkClient,
    ) -> Result<String> {
        // Create project in GenSpark
        let project_id = genspark.create_project(spec).await?;

        // Poll for completion
        let max_attempts = 60; // 5 minutes with 5-second intervals
        for attempt in 0..max_attempts {
            let status = genspark.get_project_status(&project_id).await?;

            match status.state.as_str() {
                "completed" | "ready" => {
                    info!("GenSpark project {} completed", project_id);
                    return Ok(project_id);
                }
                "failed" | "error" => {
                    return Err(GenesisError::custom(format!(
                        "GenSpark generation failed: {}",
                        status.message.unwrap_or_default()
                    )));
                }
                _ => {
                    debug!(
                        "GenSpark project {} status: {} (attempt {}/{})",
                        project_id, status.state, attempt + 1, max_attempts
                    );
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }

        Err(GenesisError::custom("GenSpark generation timed out"))
    }

    /// Deploy a project
    pub async fn deploy_project(
        &self,
        project: &Project,
        platform: Platform,
    ) -> Result<Deployment> {
        info!("Deploying project {} to {:?}", project.id, platform);

        let mut deployment = Deployment::new(&project.id, platform);

        // Set commit info if available
        if let Some(ref _repo) = project.github_repo {
            deployment.set_commit(
                "main", // In production, get actual commit SHA
                Some(format!("Initial deployment of {}", project.name)),
            );
        }

        deployment.set_status(DeploymentStatus::Building);
        self.db.save_deployment(&deployment)?;

        // Simulate deployment (in production, call actual deployment API)
        match platform {
            Platform::Vercel => {
                deployment = self.simulate_vercel_deployment(deployment).await?;
            }
            Platform::Cloudflare => {
                deployment = self.simulate_cloudflare_deployment(deployment).await?;
            }
            _ => {
                deployment = self.simulate_generic_deployment(deployment).await?;
            }
        }

        self.db.save_deployment(&deployment)?;
        Ok(deployment)
    }

    /// Simulate Vercel deployment
    async fn simulate_vercel_deployment(&self, mut deployment: Deployment) -> Result<Deployment> {
        deployment.set_status(DeploymentStatus::Deploying);

        // Simulate build time
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        deployment.add_log("info", "Build started");
        deployment.add_log("info", "Installing dependencies...");
        deployment.add_log("info", "Building project...");
        deployment.add_log("info", "Build completed successfully");

        // Generate deployment URL
        let project_name = deployment.project_id.replace('-', "");
        let url = format!("https://{}.vercel.app", &project_name[..8.min(project_name.len())]);

        deployment.set_url(&url);
        deployment.set_production_url(&url);
        deployment.set_status(DeploymentStatus::Ready);
        deployment.build_time_seconds = Some(45);
        deployment.set_platform_id(format!("dpl_{}", uuid::Uuid::new_v4()));

        Ok(deployment)
    }

    /// Simulate Cloudflare deployment
    async fn simulate_cloudflare_deployment(&self, mut deployment: Deployment) -> Result<Deployment> {
        deployment.set_status(DeploymentStatus::Deploying);

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        deployment.add_log("info", "Deploying to Cloudflare Workers");
        deployment.add_log("info", "Deployment successful");

        let project_name = deployment.project_id.replace('-', "");
        let url = format!("https://{}.workers.dev", &project_name[..8.min(project_name.len())]);

        deployment.set_url(&url);
        deployment.set_production_url(&url);
        deployment.set_status(DeploymentStatus::Ready);
        deployment.build_time_seconds = Some(15);
        deployment.set_platform_id(format!("cf_{}", uuid::Uuid::new_v4()));

        Ok(deployment)
    }

    /// Simulate generic deployment
    async fn simulate_generic_deployment(&self, mut deployment: Deployment) -> Result<Deployment> {
        deployment.set_status(DeploymentStatus::Deploying);

        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        let url = format!(
            "https://{}.example.com",
            &deployment.project_id[..8.min(deployment.project_id.len())]
        );

        deployment.set_url(&url);
        deployment.set_status(DeploymentStatus::Ready);
        deployment.build_time_seconds = Some(30);

        Ok(deployment)
    }

    /// Monitor project health
    pub async fn monitor_project(&self, project_id: &str) -> Result<ProjectHealthReport> {
        let mut project = self.db.get_project(project_id)?
            .ok_or_else(|| GenesisError::ProjectNotFound(project_id.to_string()))?;

        let mut report = ProjectHealthReport {
            project_id: project_id.to_string(),
            status: project.status,
            is_healthy: true,
            checks: Vec::new(),
            recommendations: Vec::new(),
        };

        // Check deployment URL
        if let Some(ref url) = project.deployment_url {
            match self.check_deployment_health(url).await {
                Ok(health_check) => {
                    report.checks.push(health_check.clone());
                    if !health_check.passed {
                        report.is_healthy = false;
                        report.recommendations.push(
                            "Investigate deployment health issues".to_string()
                        );
                    }
                    project.update_health(
                        health_check.passed,
                        health_check.response_time_ms,
                        if health_check.passed { None } else { Some(health_check.message.clone()) },
                    );
                }
                Err(e) => {
                    report.is_healthy = false;
                    report.checks.push(HealthCheck {
                        name: "deployment_accessible".to_string(),
                        passed: false,
                        message: e.to_string(),
                        response_time_ms: None,
                    });
                }
            }
        } else {
            report.checks.push(HealthCheck {
                name: "deployment_url".to_string(),
                passed: false,
                message: "No deployment URL configured".to_string(),
                response_time_ms: None,
            });
            report.is_healthy = false;
        }

        // Check for recent errors
        if !project.error_log.is_empty() {
            let recent_errors = project.error_log.len();
            if recent_errors > 5 {
                report.is_healthy = false;
                report.recommendations.push(format!(
                    "Review {} recent errors in error log",
                    recent_errors
                ));
            }
        }

        // Save updated health status
        self.db.save_project(&project)?;

        Ok(report)
    }

    /// Check deployment health via HTTP
    async fn check_deployment_health(&self, url: &str) -> Result<HealthCheck> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()?;

        let start = std::time::Instant::now();

        match client.get(url).send().await {
            Ok(response) => {
                let response_time = start.elapsed().as_millis() as u64;

                if response.status().is_success() {
                    Ok(HealthCheck {
                        name: "http_health".to_string(),
                        passed: true,
                        message: format!("HTTP {} OK", response.status()),
                        response_time_ms: Some(response_time),
                    })
                } else {
                    Ok(HealthCheck {
                        name: "http_health".to_string(),
                        passed: false,
                        message: format!("HTTP {} error", response.status()),
                        response_time_ms: Some(response_time),
                    })
                }
            }
            Err(e) => Ok(HealthCheck {
                name: "http_health".to_string(),
                passed: false,
                message: format!("Connection failed: {}", e),
                response_time_ms: None,
            }),
        }
    }

    /// Generate a project name from opportunity title
    fn generate_project_name(&self, title: &str) -> String {
        let name: String = title
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect();

        let words: Vec<&str> = name.split_whitespace().take(3).collect();
        let base_name = words.join("-");

        // Add random suffix for uniqueness
        let suffix: String = uuid::Uuid::new_v4().to_string()[..6].to_string();

        format!("{}-{}", base_name, suffix)
    }

    /// Suggest template type based on opportunity content
    fn suggest_template_for_opportunity(&self, opportunity: &Opportunity) -> TemplateType {
        let text = format!("{} {}", opportunity.title, opportunity.description).to_lowercase();

        if text.contains("api") || text.contains("backend") || text.contains("service") {
            TemplateType::ApiService
        } else if text.contains("blog") || text.contains("content") || text.contains("seo") {
            TemplateType::ContentSite
        } else if text.contains("extension") || text.contains("chrome") {
            TemplateType::ChromeExtension
        } else if text.contains("cli") || text.contains("command") {
            TemplateType::CliTool
        } else {
            TemplateType::MicroSaas
        }
    }

    /// Create project specification from opportunity
    fn create_project_spec(&self, opportunity: &Opportunity, template: &TemplateType) -> ProjectSpec {
        let features = self.extract_features(opportunity);
        let tech_stack = self.suggest_tech_stack(template);

        ProjectSpec {
            name: self.generate_project_name(&opportunity.title),
            description: opportunity.description.clone(),
            template_type: template.clone(),
            features,
            tech_stack,
            monetization: opportunity.monetization_analysis.as_ref().map(|m| {
                m.suggested_models.first().cloned().unwrap_or_default()
            }),
            target_users: Some("General users".to_string()),
            additional_requirements: opportunity.pain_points.iter()
                .map(|p| p.description.clone())
                .collect(),
        }
    }

    /// Extract features from opportunity description
    fn extract_features(&self, opportunity: &Opportunity) -> Vec<String> {
        let mut features = Vec::new();

        // Extract from pain points
        for pain_point in &opportunity.pain_points {
            features.push(format!("Address: {}", pain_point.description));
        }

        // Add common features based on template
        features.push("User authentication".to_string());
        features.push("Dashboard".to_string());
        features.push("Basic analytics".to_string());

        features
    }

    /// Suggest tech stack based on template type
    fn suggest_tech_stack(&self, template: &TemplateType) -> Vec<String> {
        match template {
            TemplateType::MicroSaas => vec![
                "Next.js".to_string(),
                "TypeScript".to_string(),
                "Tailwind CSS".to_string(),
                "PostgreSQL".to_string(),
                "Prisma".to_string(),
            ],
            TemplateType::ApiService => vec![
                "Node.js".to_string(),
                "Express".to_string(),
                "TypeScript".to_string(),
                "PostgreSQL".to_string(),
            ],
            TemplateType::ContentSite => vec![
                "Astro".to_string(),
                "MDX".to_string(),
                "Tailwind CSS".to_string(),
            ],
            TemplateType::ChromeExtension => vec![
                "TypeScript".to_string(),
                "React".to_string(),
                "Webpack".to_string(),
            ],
            TemplateType::CliTool => vec![
                "Rust".to_string(),
                "Clap".to_string(),
            ],
            TemplateType::MobileApp => vec![
                "React Native".to_string(),
                "TypeScript".to_string(),
                "Expo".to_string(),
            ],
            TemplateType::Custom(_) => vec![
                "TypeScript".to_string(),
            ],
        }
    }
}

/// Health check result
#[derive(Debug, Clone)]
pub struct HealthCheck {
    pub name: String,
    pub passed: bool,
    pub message: String,
    pub response_time_ms: Option<u64>,
}

/// Project health report
#[derive(Debug, Clone)]
pub struct ProjectHealthReport {
    pub project_id: String,
    pub status: ProjectStatus,
    pub is_healthy: bool,
    pub checks: Vec<HealthCheck>,
    pub recommendations: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::OpportunitySource;

    fn create_test_coordinator() -> ExecutionCoordinator {
        let db = Arc::new(Database::in_memory().unwrap());
        ExecutionCoordinator::new(
            CoordinatorConfig::default(),
            db,
            None,
            None,
        )
    }

    #[test]
    fn test_generate_project_name() {
        let coordinator = create_test_coordinator();
        let name = coordinator.generate_project_name("AI-Powered Analytics Dashboard");

        assert!(name.contains("ai"));
        assert!(name.contains("-"));
    }

    #[test]
    fn test_suggest_template() {
        let coordinator = create_test_coordinator();

        let api_opp = Opportunity::new(
            "REST API Service",
            "A backend API for data processing",
            OpportunitySource::Manual,
        );
        assert_eq!(
            coordinator.suggest_template_for_opportunity(&api_opp),
            TemplateType::ApiService
        );

        let saas_opp = Opportunity::new(
            "Project Management Tool",
            "A SaaS for teams",
            OpportunitySource::Manual,
        );
        assert_eq!(
            coordinator.suggest_template_for_opportunity(&saas_opp),
            TemplateType::MicroSaas
        );
    }

    #[test]
    fn test_tech_stack_suggestion() {
        let coordinator = create_test_coordinator();

        let saas_stack = coordinator.suggest_tech_stack(&TemplateType::MicroSaas);
        assert!(saas_stack.contains(&"Next.js".to_string()));

        let cli_stack = coordinator.suggest_tech_stack(&TemplateType::CliTool);
        assert!(cli_stack.contains(&"Rust".to_string()));
    }

    #[tokio::test]
    async fn test_simulate_deployment() {
        let coordinator = create_test_coordinator();
        let deployment = Deployment::new("test-project", Platform::Vercel);

        let result = coordinator.simulate_vercel_deployment(deployment).await;
        assert!(result.is_ok());

        let deployed = result.unwrap();
        assert_eq!(deployed.status, DeploymentStatus::Ready);
        assert!(deployed.url.is_some());
    }
}
