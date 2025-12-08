//! Project model and related types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Status of a project
#[derive(Debug, Clone, PartialEq, Eq, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum ProjectStatus {
    #[default]
    Initializing,
    Generating,
    Generated,
    Reviewing,
    Deploying,
    Live,
    Paused,
    Archived,
    Failed,
}


impl std::fmt::Display for ProjectStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Initializing => write!(f, "initializing"),
            Self::Generating => write!(f, "generating"),
            Self::Generated => write!(f, "generated"),
            Self::Reviewing => write!(f, "reviewing"),
            Self::Deploying => write!(f, "deploying"),
            Self::Live => write!(f, "live"),
            Self::Paused => write!(f, "paused"),
            Self::Archived => write!(f, "archived"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

impl std::str::FromStr for ProjectStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "initializing" => Ok(Self::Initializing),
            "generating" => Ok(Self::Generating),
            "generated" => Ok(Self::Generated),
            "reviewing" => Ok(Self::Reviewing),
            "deploying" => Ok(Self::Deploying),
            "live" => Ok(Self::Live),
            "paused" => Ok(Self::Paused),
            "archived" => Ok(Self::Archived),
            "failed" => Ok(Self::Failed),
            _ => Err(format!("Invalid project status: {}", s)),
        }
    }
}

/// Template type for project generation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[derive(Default)]
pub enum TemplateType {
    #[default]
    MicroSaas,
    ApiService,
    ContentSite,
    ChromeExtension,
    MobileApp,
    CliTool,
    Custom(String),
}


impl std::fmt::Display for TemplateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MicroSaas => write!(f, "micro-saas"),
            Self::ApiService => write!(f, "api-service"),
            Self::ContentSite => write!(f, "content-site"),
            Self::ChromeExtension => write!(f, "chrome-extension"),
            Self::MobileApp => write!(f, "mobile-app"),
            Self::CliTool => write!(f, "cli-tool"),
            Self::Custom(s) => write!(f, "{}", s),
        }
    }
}

impl std::str::FromStr for TemplateType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('_', "-").as_str() {
            "micro-saas" | "microsaas" | "saas" => Ok(Self::MicroSaas),
            "api-service" | "api" => Ok(Self::ApiService),
            "content-site" | "content" | "website" => Ok(Self::ContentSite),
            "chrome-extension" | "extension" => Ok(Self::ChromeExtension),
            "mobile-app" | "mobile" => Ok(Self::MobileApp),
            "cli-tool" | "cli" => Ok(Self::CliTool),
            other => Ok(Self::Custom(other.to_string())),
        }
    }
}

/// Project specification for code generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSpec {
    pub name: String,
    pub description: String,
    pub template_type: TemplateType,
    pub features: Vec<String>,
    pub tech_stack: Vec<String>,
    pub monetization: Option<String>,
    pub target_users: Option<String>,
    pub additional_requirements: Vec<String>,
}

impl ProjectSpec {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            template_type: TemplateType::default(),
            features: Vec::new(),
            tech_stack: Vec::new(),
            monetization: None,
            target_users: None,
            additional_requirements: Vec::new(),
        }
    }

    pub fn with_template(mut self, template: TemplateType) -> Self {
        self.template_type = template;
        self
    }

    pub fn add_feature(mut self, feature: impl Into<String>) -> Self {
        self.features.push(feature.into());
        self
    }

    pub fn add_tech(mut self, tech: impl Into<String>) -> Self {
        self.tech_stack.push(tech.into());
        self
    }
}

/// Health status of a deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub response_time_ms: Option<u64>,
    pub last_checked: DateTime<Utc>,
    pub error_message: Option<String>,
}

/// A project in the Genesis empire
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub opportunity_id: Option<String>,
    pub name: String,
    pub description: String,
    pub github_repo: Option<String>,
    pub deployment_url: Option<String>,
    pub status: ProjectStatus,
    pub template_type: TemplateType,
    pub spec: Option<ProjectSpec>,
    pub genspark_project_id: Option<String>,
    pub health_status: Option<HealthStatus>,
    pub total_revenue_usd: f64,
    pub active_users: u64,
    pub error_log: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub launched_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

impl Project {
    /// Create a new project
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            opportunity_id: None,
            name: name.into(),
            description: description.into(),
            github_repo: None,
            deployment_url: None,
            status: ProjectStatus::Initializing,
            template_type: TemplateType::default(),
            spec: None,
            genspark_project_id: None,
            health_status: None,
            total_revenue_usd: 0.0,
            active_users: 0,
            error_log: Vec::new(),
            created_at: now,
            launched_at: None,
            updated_at: now,
        }
    }

    /// Create project from an opportunity
    pub fn from_opportunity(opportunity_id: &str, name: impl Into<String>) -> Self {
        let mut project = Self::new(name, "");
        project.opportunity_id = Some(opportunity_id.to_string());
        project
    }

    /// Set the project specification
    pub fn with_spec(mut self, spec: ProjectSpec) -> Self {
        self.description = spec.description.clone();
        self.template_type = spec.template_type.clone();
        self.spec = Some(spec);
        self
    }

    /// Update status
    pub fn set_status(&mut self, status: ProjectStatus) {
        self.status = status;
        self.updated_at = Utc::now();

        if status == ProjectStatus::Live && self.launched_at.is_none() {
            self.launched_at = Some(Utc::now());
        }
    }

    /// Set GitHub repository
    pub fn set_github_repo(&mut self, repo: impl Into<String>) {
        self.github_repo = Some(repo.into());
        self.updated_at = Utc::now();
    }

    /// Set deployment URL
    pub fn set_deployment_url(&mut self, url: impl Into<String>) {
        self.deployment_url = Some(url.into());
        self.updated_at = Utc::now();
    }

    /// Add error to log
    pub fn log_error(&mut self, error: impl Into<String>) {
        self.error_log.push(format!(
            "[{}] {}",
            Utc::now().format("%Y-%m-%d %H:%M:%S"),
            error.into()
        ));
        self.updated_at = Utc::now();
    }

    /// Check if project is active
    pub fn is_active(&self) -> bool {
        matches!(
            self.status,
            ProjectStatus::Live | ProjectStatus::Deploying | ProjectStatus::Generating
        )
    }

    /// Check if project is deployable
    pub fn is_deployable(&self) -> bool {
        matches!(
            self.status,
            ProjectStatus::Generated | ProjectStatus::Reviewing
        ) && self.github_repo.is_some()
    }

    /// Get days since launch
    pub fn days_since_launch(&self) -> Option<i64> {
        self.launched_at.map(|launched| {
            (Utc::now() - launched).num_days()
        })
    }

    /// Update health status
    pub fn update_health(&mut self, is_healthy: bool, response_time_ms: Option<u64>, error: Option<String>) {
        self.health_status = Some(HealthStatus {
            is_healthy,
            response_time_ms,
            last_checked: Utc::now(),
            error_message: error,
        });
        self.updated_at = Utc::now();
    }
}

/// Project statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStats {
    pub total_projects: u64,
    pub live_projects: u64,
    pub failed_projects: u64,
    pub total_revenue: f64,
    pub total_users: u64,
    pub avg_revenue_per_project: f64,
}

impl ProjectStats {
    pub fn new() -> Self {
        Self {
            total_projects: 0,
            live_projects: 0,
            failed_projects: 0,
            total_revenue: 0.0,
            total_users: 0,
            avg_revenue_per_project: 0.0,
        }
    }

    pub fn calculate_average(&mut self) {
        if self.live_projects > 0 {
            self.avg_revenue_per_project = self.total_revenue / self.live_projects as f64;
        }
    }
}

impl Default for ProjectStats {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_project() {
        let project = Project::new("Test Project", "A test description");
        assert!(!project.id.is_empty());
        assert_eq!(project.name, "Test Project");
        assert_eq!(project.status, ProjectStatus::Initializing);
    }

    #[test]
    fn test_project_status_lifecycle() {
        let mut project = Project::new("Test", "Test");
        assert!(project.launched_at.is_none());

        project.set_status(ProjectStatus::Generating);
        assert!(project.launched_at.is_none());

        project.set_status(ProjectStatus::Live);
        assert!(project.launched_at.is_some());
    }

    #[test]
    fn test_project_spec_builder() {
        let spec = ProjectSpec::new("My SaaS", "A micro SaaS product")
            .with_template(TemplateType::MicroSaas)
            .add_feature("User authentication")
            .add_feature("Stripe integration")
            .add_tech("Next.js")
            .add_tech("PostgreSQL");

        assert_eq!(spec.features.len(), 2);
        assert_eq!(spec.tech_stack.len(), 2);
        assert_eq!(spec.template_type, TemplateType::MicroSaas);
    }

    #[test]
    fn test_template_type_parsing() {
        assert_eq!(
            "micro-saas".parse::<TemplateType>().unwrap(),
            TemplateType::MicroSaas
        );
        assert_eq!(
            "api".parse::<TemplateType>().unwrap(),
            TemplateType::ApiService
        );
    }
}
