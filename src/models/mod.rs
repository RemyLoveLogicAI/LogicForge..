//! Data models for Genesis Engine

pub mod deployment;
pub mod opportunity;
pub mod project;

pub use deployment::{Deployment, DeploymentConfig, DeploymentStatus, Platform};
pub use opportunity::{
    MarketAnalysis, MonetizationAnalysis, Opportunity, OpportunityBuilder, OpportunitySource,
    OpportunityStatus, PainPoint, RankedOpportunity, RawOpportunityData, TechnicalAnalysis,
};
pub use project::{HealthStatus, Project, ProjectSpec, ProjectStats, ProjectStatus, TemplateType};

/// Revenue metrics for a project
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RevenueMetric {
    pub id: String,
    pub project_id: String,
    pub metric_date: chrono::NaiveDate,
    pub revenue_usd: f64,
    pub active_users: u64,
    pub conversions: u64,
    pub recorded_at: chrono::DateTime<chrono::Utc>,
}

impl RevenueMetric {
    pub fn new(project_id: impl Into<String>, date: chrono::NaiveDate) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            project_id: project_id.into(),
            metric_date: date,
            revenue_usd: 0.0,
            active_users: 0,
            conversions: 0,
            recorded_at: chrono::Utc::now(),
        }
    }
}

/// Portfolio summary
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Portfolio {
    pub total_projects: u64,
    pub active_projects: u64,
    pub total_revenue: f64,
    pub total_users: u64,
    pub projects: Vec<Project>,
}

impl Portfolio {
    pub fn new() -> Self {
        Self {
            total_projects: 0,
            active_projects: 0,
            total_revenue: 0.0,
            total_users: 0,
            projects: Vec::new(),
        }
    }

    pub fn add_project(&mut self, project: Project) {
        self.total_projects += 1;
        if project.is_active() {
            self.active_projects += 1;
        }
        self.total_revenue += project.total_revenue_usd;
        self.total_users += project.active_users;
        self.projects.push(project);
    }
}

impl Default for Portfolio {
    fn default() -> Self {
        Self::new()
    }
}

/// Decision result from the decision matrix
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Decision {
    Launch {
        confidence: f64,
        reasoning: String,
        suggested_template: TemplateType,
    },
    Defer {
        reason: String,
        revisit_after_days: u32,
    },
    Reject {
        reason: String,
    },
}

impl Decision {
    pub fn launch(confidence: f64, reasoning: impl Into<String>) -> Self {
        Self::Launch {
            confidence,
            reasoning: reasoning.into(),
            suggested_template: TemplateType::default(),
        }
    }

    pub fn defer(reason: impl Into<String>, revisit_days: u32) -> Self {
        Self::Defer {
            reason: reason.into(),
            revisit_after_days: revisit_days,
        }
    }

    pub fn reject(reason: impl Into<String>) -> Self {
        Self::Reject {
            reason: reason.into(),
        }
    }

    pub fn is_launch(&self) -> bool {
        matches!(self, Self::Launch { .. })
    }
}
