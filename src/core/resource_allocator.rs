//! Resource Allocator - Manages resources across the empire

use crate::models::{Project, ProjectStatus, Portfolio};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::info;

/// Resource allocation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocatorConfig {
    /// Total budget available
    pub total_budget: f64,
    /// Maximum budget per project
    pub max_per_project: f64,
    /// Minimum budget per project
    pub min_per_project: f64,
    /// Reserve percentage (0-1)
    pub reserve_percentage: f64,
    /// Reallocation frequency in hours
    pub reallocation_interval_hours: u32,
}

impl Default for AllocatorConfig {
    fn default() -> Self {
        Self {
            total_budget: 5000.0,
            max_per_project: 1000.0,
            min_per_project: 100.0,
            reserve_percentage: 0.2,
            reallocation_interval_hours: 168, // Weekly
        }
    }
}

/// Resource allocation for a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectAllocation {
    pub project_id: String,
    pub allocated_budget: f64,
    pub priority_score: f64,
    pub allocation_reason: String,
}

/// Resource allocation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationResult {
    pub allocations: Vec<ProjectAllocation>,
    pub total_allocated: f64,
    pub reserve_budget: f64,
    pub unallocated: f64,
}

/// Resource allocator for managing empire resources
pub struct ResourceAllocator {
    config: AllocatorConfig,
}

impl ResourceAllocator {
    /// Create a new resource allocator
    pub fn new(config: AllocatorConfig) -> Self {
        Self { config }
    }

    /// Create with default configuration
    pub fn default_allocator() -> Self {
        Self::new(AllocatorConfig::default())
    }

    /// Allocate resources across all projects
    pub fn allocate(&self, portfolio: &Portfolio) -> AllocationResult {
        info!(
            "Allocating resources across {} projects",
            portfolio.projects.len()
        );

        // Calculate reserve
        let reserve_budget = self.config.total_budget * self.config.reserve_percentage;
        let available_budget = self.config.total_budget - reserve_budget;

        // Score and prioritize projects
        let mut project_scores: Vec<(String, f64, &Project)> = portfolio
            .projects
            .iter()
            .filter(|p| p.is_active())
            .map(|p| {
                let score = self.calculate_priority_score(p);
                (p.id.clone(), score, p)
            })
            .collect();

        // Sort by priority score (highest first)
        project_scores.sort_by(|a, b| {
            b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Allocate based on priority
        let total_score: f64 = project_scores.iter().map(|(_, s, _)| s).sum();
        let mut allocations = Vec::new();
        let mut total_allocated = 0.0;

        for (project_id, score, project) in project_scores {
            if total_allocated >= available_budget {
                break;
            }

            // Calculate proportional allocation
            let proportion = if total_score > 0.0 {
                score / total_score
            } else {
                1.0 / portfolio.projects.len() as f64
            };

            let mut allocation = proportion * available_budget;

            // Apply min/max constraints
            allocation = allocation.clamp(
                self.config.min_per_project,
                self.config.max_per_project,
            );

            // Don't exceed remaining budget
            allocation = allocation.min(available_budget - total_allocated);

            let reason = self.generate_allocation_reason(project, score);

            allocations.push(ProjectAllocation {
                project_id,
                allocated_budget: allocation,
                priority_score: score,
                allocation_reason: reason,
            });

            total_allocated += allocation;
        }

        AllocationResult {
            allocations,
            total_allocated,
            reserve_budget,
            unallocated: available_budget - total_allocated,
        }
    }

    /// Calculate priority score for a project
    fn calculate_priority_score(&self, project: &Project) -> f64 {
        let mut score = 50.0;

        // Status-based scoring
        match project.status {
            ProjectStatus::Live => score += 30.0,
            ProjectStatus::Deploying => score += 20.0,
            ProjectStatus::Generated | ProjectStatus::Reviewing => score += 15.0,
            ProjectStatus::Generating => score += 10.0,
            _ => {}
        }

        // Revenue-based scoring
        if project.total_revenue_usd > 0.0 {
            score += (project.total_revenue_usd.log10() * 10.0).min(30.0);
        }

        // User-based scoring
        if project.active_users > 0 {
            score += ((project.active_users as f64).log10() * 5.0).min(20.0);
        }

        // Health-based scoring
        if let Some(ref health) = project.health_status {
            if health.is_healthy {
                score += 10.0;
            } else {
                score -= 10.0;
            }

            // Response time penalty
            if let Some(rt) = health.response_time_ms {
                if rt > 2000 {
                    score -= 5.0;
                }
            }
        }

        // Age-based scoring (newer projects get slight boost for growth)
        if let Some(days) = project.days_since_launch() {
            if days < 30 {
                score += 5.0; // New project boost
            } else if days > 180 && project.total_revenue_usd == 0.0 {
                score -= 10.0; // Old project with no revenue
            }
        }

        // Error penalty
        if !project.error_log.is_empty() {
            score -= (project.error_log.len() as f64 * 2.0).min(15.0);
        }

        score.clamp(0.0, 100.0)
    }

    /// Generate human-readable allocation reason
    fn generate_allocation_reason(&self, project: &Project, score: f64) -> String {
        let mut reasons = Vec::new();

        if project.status == ProjectStatus::Live {
            reasons.push("live status");
        }

        if project.total_revenue_usd > 100.0 {
            reasons.push("generating revenue");
        }

        if project.active_users > 100 {
            reasons.push("active user base");
        }

        if let Some(ref health) = project.health_status {
            if health.is_healthy {
                reasons.push("healthy deployment");
            }
        }

        if reasons.is_empty() {
            reasons.push("standard allocation");
        }

        format!(
            "Priority score: {:.1}. Factors: {}",
            score,
            reasons.join(", ")
        )
    }

    /// Recommend reallocation actions
    pub fn recommend_actions(&self, portfolio: &Portfolio) -> Vec<ReallocationAction> {
        let mut actions = Vec::new();

        for project in &portfolio.projects {
            // Recommend sunsetting non-performing projects
            if project.status == ProjectStatus::Live {
                if let Some(days) = project.days_since_launch() {
                    if days > 90 && project.total_revenue_usd == 0.0 && project.active_users < 10 {
                        actions.push(ReallocationAction::ConsiderSunset {
                            project_id: project.id.clone(),
                            reason: "No revenue or users after 90 days".to_string(),
                        });
                    }
                }
            }

            // Recommend fixing unhealthy deployments
            if let Some(ref health) = project.health_status {
                if !health.is_healthy {
                    actions.push(ReallocationAction::InvestigateHealth {
                        project_id: project.id.clone(),
                        error: health.error_message.clone().unwrap_or_default(),
                    });
                }
            }

            // Recommend scaling high-performers
            if project.total_revenue_usd > 1000.0 && project.active_users > 500 {
                actions.push(ReallocationAction::ConsiderScaling {
                    project_id: project.id.clone(),
                    reason: "Strong revenue and user growth".to_string(),
                });
            }

            // Recommend retrying failed projects with potential
            if project.status == ProjectStatus::Failed
                && project.error_log.len() < 3 {
                    actions.push(ReallocationAction::ConsiderRetry {
                        project_id: project.id.clone(),
                        reason: "Limited failure attempts".to_string(),
                    });
                }
        }

        actions
    }

    /// Get budget summary
    pub fn get_budget_summary(&self, portfolio: &Portfolio) -> BudgetSummary {
        let allocation = self.allocate(portfolio);

        let mut by_status: HashMap<String, f64> = HashMap::new();
        for alloc in &allocation.allocations {
            if let Some(project) = portfolio.projects.iter().find(|p| p.id == alloc.project_id) {
                *by_status
                    .entry(project.status.to_string())
                    .or_insert(0.0) += alloc.allocated_budget;
            }
        }

        BudgetSummary {
            total_budget: self.config.total_budget,
            allocated: allocation.total_allocated,
            reserved: allocation.reserve_budget,
            unallocated: allocation.unallocated,
            by_status,
            active_projects: portfolio.active_projects as usize,
        }
    }
}

/// Recommended reallocation action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReallocationAction {
    ConsiderSunset {
        project_id: String,
        reason: String,
    },
    InvestigateHealth {
        project_id: String,
        error: String,
    },
    ConsiderScaling {
        project_id: String,
        reason: String,
    },
    ConsiderRetry {
        project_id: String,
        reason: String,
    },
}

/// Budget summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetSummary {
    pub total_budget: f64,
    pub allocated: f64,
    pub reserved: f64,
    pub unallocated: f64,
    pub by_status: HashMap<String, f64>,
    pub active_projects: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Project;

    fn create_test_portfolio() -> Portfolio {
        let mut portfolio = Portfolio::new();

        let mut project1 = Project::new("High Performer", "Revenue generating project");
        project1.status = ProjectStatus::Live;
        project1.total_revenue_usd = 500.0;
        project1.active_users = 200;
        portfolio.add_project(project1);

        let mut project2 = Project::new("New Project", "Recently launched");
        project2.status = ProjectStatus::Live;
        project2.total_revenue_usd = 0.0;
        project2.active_users = 10;
        portfolio.add_project(project2);

        portfolio
    }

    #[test]
    fn test_resource_allocation() {
        let allocator = ResourceAllocator::default_allocator();
        let portfolio = create_test_portfolio();

        let result = allocator.allocate(&portfolio);

        assert!(!result.allocations.is_empty());
        assert!(result.total_allocated > 0.0);
        assert!(result.reserve_budget > 0.0);
    }

    #[test]
    fn test_priority_scoring() {
        let allocator = ResourceAllocator::default_allocator();

        let mut high_performer = Project::new("High", "High performer");
        high_performer.status = ProjectStatus::Live;
        high_performer.total_revenue_usd = 1000.0;
        high_performer.active_users = 500;

        let mut low_performer = Project::new("Low", "Low performer");
        low_performer.status = ProjectStatus::Live;
        low_performer.total_revenue_usd = 0.0;
        low_performer.active_users = 0;

        let high_score = allocator.calculate_priority_score(&high_performer);
        let low_score = allocator.calculate_priority_score(&low_performer);

        assert!(high_score > low_score);
    }

    #[test]
    fn test_budget_summary() {
        let allocator = ResourceAllocator::default_allocator();
        let portfolio = create_test_portfolio();

        let summary = allocator.get_budget_summary(&portfolio);

        assert_eq!(summary.active_projects, 2);
        assert!(summary.allocated <= summary.total_budget);
    }

    #[test]
    fn test_recommend_actions() {
        let allocator = ResourceAllocator::default_allocator();
        let mut portfolio = Portfolio::new();

        // Add a failed project
        let mut failed = Project::new("Failed", "Failed project");
        failed.status = ProjectStatus::Failed;
        portfolio.add_project(failed);

        let actions = allocator.recommend_actions(&portfolio);

        assert!(actions.iter().any(|a| matches!(a, ReallocationAction::ConsiderRetry { .. })));
    }
}
