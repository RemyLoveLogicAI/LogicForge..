//! Revenue Tracker - Monitors and reports revenue across the empire

use crate::db::Database;
use crate::models::RevenueMetric;
use crate::utils::error::Result;
use chrono::{Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::debug;

/// Configuration for revenue tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueConfig {
    /// Track interval in hours
    pub track_interval_hours: u32,
    /// Alert threshold for revenue drop percentage
    pub alert_threshold_drop_percent: f64,
    /// Number of days to keep detailed metrics
    pub retention_days: u32,
}

impl Default for RevenueConfig {
    fn default() -> Self {
        Self {
            track_interval_hours: 24,
            alert_threshold_drop_percent: 20.0,
            retention_days: 365,
        }
    }
}

/// Revenue tracker for monitoring financial performance
pub struct RevenueTracker {
    config: RevenueConfig,
    db: Arc<Database>,
}

impl RevenueTracker {
    /// Create a new revenue tracker
    pub fn new(config: RevenueConfig, db: Arc<Database>) -> Self {
        Self { config, db }
    }

    /// Record revenue for a project
    pub fn record_revenue(
        &self,
        project_id: &str,
        date: NaiveDate,
        revenue_usd: f64,
        active_users: u64,
        conversions: u64,
    ) -> Result<RevenueMetric> {
        debug!("Recording revenue for project {}: ${:.2}", project_id, revenue_usd);

        let mut metric = RevenueMetric::new(project_id, date);
        metric.revenue_usd = revenue_usd;
        metric.active_users = active_users;
        metric.conversions = conversions;

        self.db.record_revenue(&metric)?;

        Ok(metric)
    }

    /// Get revenue summary for a project
    pub fn get_project_summary(
        &self,
        project_id: &str,
        days: u32,
    ) -> Result<RevenueSummary> {
        let end_date = Utc::now().date_naive();
        let start_date = end_date - Duration::days(days as i64);

        let metrics = self.db.get_revenue_metrics(
            project_id,
            Some(start_date),
            Some(end_date),
        )?;

        let total_revenue: f64 = metrics.iter().map(|m| m.revenue_usd).sum();
        let avg_daily_revenue = if !metrics.is_empty() {
            total_revenue / metrics.len() as f64
        } else {
            0.0
        };

        let total_users: u64 = metrics.iter().map(|m| m.active_users).max().unwrap_or(0);
        let total_conversions: u64 = metrics.iter().map(|m| m.conversions).sum();

        // Calculate trend
        let trend = self.calculate_trend(&metrics);

        // Check for alerts
        let alerts = self.check_alerts(&metrics);

        Ok(RevenueSummary {
            project_id: project_id.to_string(),
            period_days: days,
            total_revenue,
            avg_daily_revenue,
            peak_users: total_users,
            total_conversions,
            trend,
            alerts,
            metrics,
        })
    }

    /// Get empire-wide revenue summary
    pub fn get_empire_summary(&self, days: u32) -> Result<EmpireSummary> {
        let projects = self.db.list_projects(None, None)?;

        let mut total_revenue = 0.0;
        let mut total_users = 0u64;
        let mut project_summaries = Vec::new();
        let mut revenue_by_project: HashMap<String, f64> = HashMap::new();

        for project in &projects {
            if let Ok(summary) = self.get_project_summary(&project.id, days) {
                total_revenue += summary.total_revenue;
                total_users = total_users.max(summary.peak_users);
                revenue_by_project.insert(project.name.clone(), summary.total_revenue);
                project_summaries.push(summary);
            }
        }

        // Find top performers
        let mut top_performers: Vec<_> = revenue_by_project.into_iter().collect();
        top_performers.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        let top_performers: Vec<_> = top_performers.into_iter().take(5).collect();

        // Calculate overall trend
        let overall_trend = self.calculate_overall_trend(&project_summaries);

        // Collect all alerts
        let all_alerts: Vec<_> = project_summaries
            .iter()
            .flat_map(|s| s.alerts.clone())
            .collect();

        Ok(EmpireSummary {
            period_days: days,
            total_revenue,
            total_projects: projects.len() as u64,
            active_projects: projects.iter().filter(|p| p.is_active()).count() as u64,
            peak_users: total_users,
            overall_trend,
            top_performers,
            alerts: all_alerts,
            project_summaries,
        })
    }

    /// Calculate revenue trend
    fn calculate_trend(&self, metrics: &[RevenueMetric]) -> RevenueTrend {
        if metrics.len() < 2 {
            return RevenueTrend::Stable;
        }

        let mid = metrics.len() / 2;
        let (first_half, second_half) = metrics.split_at(mid);

        let first_avg: f64 = first_half.iter().map(|m| m.revenue_usd).sum::<f64>()
            / first_half.len() as f64;
        let second_avg: f64 = second_half.iter().map(|m| m.revenue_usd).sum::<f64>()
            / second_half.len() as f64;

        if first_avg == 0.0 {
            if second_avg > 0.0 {
                return RevenueTrend::Growing;
            }
            return RevenueTrend::Stable;
        }

        let change_percent = ((second_avg - first_avg) / first_avg) * 100.0;

        if change_percent > 10.0 {
            RevenueTrend::Growing
        } else if change_percent < -10.0 {
            RevenueTrend::Declining
        } else {
            RevenueTrend::Stable
        }
    }

    /// Calculate overall trend from multiple summaries
    fn calculate_overall_trend(&self, summaries: &[RevenueSummary]) -> RevenueTrend {
        let growing = summaries.iter().filter(|s| matches!(s.trend, RevenueTrend::Growing)).count();
        let declining = summaries.iter().filter(|s| matches!(s.trend, RevenueTrend::Declining)).count();

        if growing > declining * 2 {
            RevenueTrend::Growing
        } else if declining > growing * 2 {
            RevenueTrend::Declining
        } else {
            RevenueTrend::Stable
        }
    }

    /// Check for revenue alerts
    fn check_alerts(&self, metrics: &[RevenueMetric]) -> Vec<RevenueAlert> {
        let mut alerts = Vec::new();

        if metrics.len() < 2 {
            return alerts;
        }

        // Check for sudden drops
        for window in metrics.windows(2) {
            let prev = &window[0];
            let curr = &window[1];

            if prev.revenue_usd > 0.0 {
                let drop_percent = ((prev.revenue_usd - curr.revenue_usd) / prev.revenue_usd) * 100.0;

                if drop_percent > self.config.alert_threshold_drop_percent {
                    alerts.push(RevenueAlert {
                        alert_type: AlertType::SuddenDrop,
                        message: format!(
                            "Revenue dropped {:.1}% from ${:.2} to ${:.2}",
                            drop_percent, prev.revenue_usd, curr.revenue_usd
                        ),
                        severity: AlertSeverity::High,
                        date: curr.metric_date,
                    });
                }
            }
        }

        // Check for zero revenue days
        let zero_days: Vec<_> = metrics.iter().filter(|m| m.revenue_usd == 0.0).collect();
        if zero_days.len() > 3 {
            alerts.push(RevenueAlert {
                alert_type: AlertType::NoRevenue,
                message: format!("{} days with zero revenue", zero_days.len()),
                severity: AlertSeverity::Medium,
                date: Utc::now().date_naive(),
            });
        }

        // Check for conversion rate issues
        let total_users: u64 = metrics.iter().map(|m| m.active_users).sum();
        let total_conversions: u64 = metrics.iter().map(|m| m.conversions).sum();

        if total_users > 100 && total_conversions == 0 {
            alerts.push(RevenueAlert {
                alert_type: AlertType::LowConversion,
                message: "No conversions despite significant traffic".to_string(),
                severity: AlertSeverity::Medium,
                date: Utc::now().date_naive(),
            });
        }

        alerts
    }

    /// Generate revenue report
    pub fn generate_report(&self, days: u32) -> Result<RevenueReport> {
        let summary = self.get_empire_summary(days)?;

        // Calculate MRR (Monthly Recurring Revenue) estimate
        let mrr = summary.total_revenue / (days as f64 / 30.0);

        // Calculate ARR (Annual Recurring Revenue) estimate
        let arr = mrr * 12.0;

        // Calculate average revenue per user (ARPU)
        let arpu = if summary.peak_users > 0 {
            summary.total_revenue / summary.peak_users as f64
        } else {
            0.0
        };

        Ok(RevenueReport {
            generated_at: Utc::now(),
            period_days: days,
            total_revenue: summary.total_revenue,
            estimated_mrr: mrr,
            estimated_arr: arr,
            arpu,
            total_projects: summary.total_projects,
            active_projects: summary.active_projects,
            peak_users: summary.peak_users,
            trend: summary.overall_trend,
            top_performers: summary.top_performers.clone(),
            alerts: summary.alerts.clone(),
            recommendations: self.generate_recommendations(&summary),
        })
    }

    /// Generate recommendations based on revenue data
    fn generate_recommendations(&self, summary: &EmpireSummary) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Check for underperforming projects
        for project_summary in &summary.project_summaries {
            if matches!(project_summary.trend, RevenueTrend::Declining)
                && project_summary.total_revenue < 100.0
            {
                recommendations.push(format!(
                    "Consider reviewing or sunsetting project '{}' (declining revenue)",
                    project_summary.project_id
                ));
            }
        }

        // Check for high performers to scale
        for (name, revenue) in &summary.top_performers {
            if *revenue > 1000.0 {
                recommendations.push(format!(
                    "Consider scaling '{}' - strong revenue performance",
                    name
                ));
            }
        }

        // General recommendations
        if summary.total_projects > 0 && summary.active_projects == 0 {
            recommendations.push("No active projects - consider launching new products".to_string());
        }

        if !summary.alerts.is_empty() {
            recommendations.push(format!(
                "Address {} revenue alerts to maintain growth",
                summary.alerts.len()
            ));
        }

        recommendations
    }
}

/// Revenue trend direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RevenueTrend {
    Growing,
    Stable,
    Declining,
}

impl std::fmt::Display for RevenueTrend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Growing => write!(f, "📈 Growing"),
            Self::Stable => write!(f, "➡️ Stable"),
            Self::Declining => write!(f, "📉 Declining"),
        }
    }
}

/// Revenue alert types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    SuddenDrop,
    NoRevenue,
    LowConversion,
    UnusualSpike,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Revenue alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueAlert {
    pub alert_type: AlertType,
    pub message: String,
    pub severity: AlertSeverity,
    pub date: NaiveDate,
}

/// Revenue summary for a single project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueSummary {
    pub project_id: String,
    pub period_days: u32,
    pub total_revenue: f64,
    pub avg_daily_revenue: f64,
    pub peak_users: u64,
    pub total_conversions: u64,
    pub trend: RevenueTrend,
    pub alerts: Vec<RevenueAlert>,
    pub metrics: Vec<RevenueMetric>,
}

/// Empire-wide revenue summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmpireSummary {
    pub period_days: u32,
    pub total_revenue: f64,
    pub total_projects: u64,
    pub active_projects: u64,
    pub peak_users: u64,
    pub overall_trend: RevenueTrend,
    pub top_performers: Vec<(String, f64)>,
    pub alerts: Vec<RevenueAlert>,
    pub project_summaries: Vec<RevenueSummary>,
}

/// Full revenue report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueReport {
    pub generated_at: chrono::DateTime<Utc>,
    pub period_days: u32,
    pub total_revenue: f64,
    pub estimated_mrr: f64,
    pub estimated_arr: f64,
    pub arpu: f64,
    pub total_projects: u64,
    pub active_projects: u64,
    pub peak_users: u64,
    pub trend: RevenueTrend,
    pub top_performers: Vec<(String, f64)>,
    pub alerts: Vec<RevenueAlert>,
    pub recommendations: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Project;

    fn create_test_tracker() -> RevenueTracker {
        let db = Arc::new(Database::in_memory().unwrap());
        RevenueTracker::new(RevenueConfig::default(), db)
    }

    #[test]
    fn test_record_revenue() {
        let tracker = create_test_tracker();
        let today = Utc::now().date_naive();

        // First, create a project
        let project = Project::new("test-project", "Test");
        tracker.db.save_project(&project).unwrap();

        let metric = tracker
            .record_revenue(&project.id, today, 100.0, 50, 5)
            .unwrap();

        assert_eq!(metric.revenue_usd, 100.0);
        assert_eq!(metric.active_users, 50);
        assert_eq!(metric.conversions, 5);
    }

    #[test]
    fn test_trend_calculation() {
        let tracker = create_test_tracker();

        // Growing metrics
        let growing_metrics: Vec<RevenueMetric> = (0..10)
            .map(|i| {
                let mut m = RevenueMetric::new(
                    "test",
                    Utc::now().date_naive() - Duration::days(10 - i),
                );
                m.revenue_usd = 100.0 + (i as f64 * 20.0);
                m
            })
            .collect();

        assert!(matches!(
            tracker.calculate_trend(&growing_metrics),
            RevenueTrend::Growing
        ));

        // Declining metrics
        let declining_metrics: Vec<RevenueMetric> = (0..10)
            .map(|i| {
                let mut m = RevenueMetric::new(
                    "test",
                    Utc::now().date_naive() - Duration::days(10 - i),
                );
                m.revenue_usd = 200.0 - (i as f64 * 15.0);
                m
            })
            .collect();

        assert!(matches!(
            tracker.calculate_trend(&declining_metrics),
            RevenueTrend::Declining
        ));
    }

    #[test]
    fn test_revenue_alerts() {
        let tracker = create_test_tracker();
        let today = Utc::now().date_naive();

        let metrics = vec![
            {
                let mut m = RevenueMetric::new("test", today - Duration::days(2));
                m.revenue_usd = 100.0;
                m
            },
            {
                let mut m = RevenueMetric::new("test", today - Duration::days(1));
                m.revenue_usd = 50.0; // 50% drop
                m
            },
        ];

        let alerts = tracker.check_alerts(&metrics);
        assert!(!alerts.is_empty());
        assert!(alerts.iter().any(|a| matches!(a.alert_type, AlertType::SuddenDrop)));
    }
}
