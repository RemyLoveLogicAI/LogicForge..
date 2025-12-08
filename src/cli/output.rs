//! Output formatting utilities for CLI

use crate::models::{Deployment, Opportunity, Project, ProjectStats};
use colored::*;
use std::io::Write;

/// Output formatter trait
pub trait Formatter {
    fn format_opportunity(&self, opportunity: &Opportunity) -> String;
    fn format_opportunities(&self, opportunities: &[Opportunity]) -> String;
    fn format_project(&self, project: &Project) -> String;
    fn format_projects(&self, projects: &[Project]) -> String;
    fn format_deployment(&self, deployment: &Deployment) -> String;
    fn format_stats(&self, stats: &ProjectStats) -> String;
}

/// Text formatter for human-readable output
pub struct TextFormatter;

impl TextFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TextFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl Formatter for TextFormatter {
    fn format_opportunity(&self, opp: &Opportunity) -> String {
        let score_color = match opp.score.unwrap_or(0.0) {
            s if s >= 80.0 => "green",
            s if s >= 60.0 => "yellow",
            _ => "red",
        };

        let score_str = opp
            .score
            .map(|s| format!("{:.1}", s))
            .unwrap_or_else(|| "N/A".to_string());

        format!(
            r#"
┌─────────────────────────────────────────────────────────────┐
│ {} {}
├─────────────────────────────────────────────────────────────┤
│ ID:     {}
│ Source: {}
│ Status: {}
│ Score:  {}
├─────────────────────────────────────────────────────────────┤
│ {}
└─────────────────────────────────────────────────────────────┘
"#,
            "📊",
            opp.title.bold(),
            opp.id.dimmed(),
            opp.source.to_string().cyan(),
            format_status(&opp.status.to_string()),
            match score_color {
                "green" => score_str.green().to_string(),
                "yellow" => score_str.yellow().to_string(),
                _ => score_str.red().to_string(),
            },
            truncate(&opp.description, 60)
        )
    }

    fn format_opportunities(&self, opportunities: &[Opportunity]) -> String {
        if opportunities.is_empty() {
            return "No opportunities found.".dimmed().to_string();
        }

        let mut output = format!(
            "\n{} {} opportunities found\n\n",
            "🔍",
            opportunities.len().to_string().bold()
        );

        output.push_str(&format!(
            "{:─<68}\n",
            "".to_string()
        ));
        output.push_str(&format!(
            " {:^8} │ {:^40} │ {:^8} │ {:^6}\n",
            "ID", "TITLE", "SOURCE", "SCORE"
        ));
        output.push_str(&format!(
            "{:─<68}\n",
            "".to_string()
        ));

        for opp in opportunities {
            let score_str = opp
                .score
                .map(|s| format!("{:.0}", s))
                .unwrap_or_else(|| "-".to_string());

            let colored_score = match opp.score.unwrap_or(0.0) {
                s if s >= 80.0 => score_str.green(),
                s if s >= 60.0 => score_str.yellow(),
                _ => score_str.red(),
            };

            output.push_str(&format!(
                " {:8} │ {:40} │ {:^8} │ {:^6}\n",
                truncate(&opp.id, 8),
                truncate(&opp.title, 40),
                opp.source.to_string(),
                colored_score
            ));
        }

        output.push_str(&format!(
            "{:─<68}\n",
            "".to_string()
        ));

        output
    }

    fn format_project(&self, project: &Project) -> String {
        let status_icon = match project.status {
            crate::models::ProjectStatus::Live => "🟢",
            crate::models::ProjectStatus::Deploying => "🔵",
            crate::models::ProjectStatus::Failed => "🔴",
            crate::models::ProjectStatus::Paused => "🟡",
            _ => "⚪",
        };

        format!(
            r#"
┌─────────────────────────────────────────────────────────────┐
│ {} {} {}
├─────────────────────────────────────────────────────────────┤
│ ID:         {}
│ Status:     {}
│ Template:   {}
│ GitHub:     {}
│ URL:        {}
│ Revenue:    {}
│ Users:      {}
├─────────────────────────────────────────────────────────────┤
│ Created:    {}
│ Launched:   {}
└─────────────────────────────────────────────────────────────┘
"#,
            status_icon,
            project.name.bold(),
            format!("({})", project.template_type).dimmed(),
            project.id.dimmed(),
            format_status(&project.status.to_string()),
            project.template_type.to_string().cyan(),
            project
                .github_repo
                .as_deref()
                .unwrap_or("Not set"),
            project
                .deployment_url
                .as_deref()
                .unwrap_or("Not deployed"),
            format!("${:.2}", project.total_revenue_usd).green(),
            project.active_users.to_string().cyan(),
            project.created_at.format("%Y-%m-%d %H:%M"),
            project
                .launched_at
                .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "Not launched".dimmed().to_string())
        )
    }

    fn format_projects(&self, projects: &[Project]) -> String {
        if projects.is_empty() {
            return "No projects found.".dimmed().to_string();
        }

        let mut output = format!(
            "\n{} {} projects\n\n",
            "🚀",
            projects.len().to_string().bold()
        );

        output.push_str(&format!(
            "{:─<80}\n",
            "".to_string()
        ));
        output.push_str(&format!(
            " {:^8} │ {:^30} │ {:^12} │ {:^10} │ {:^10}\n",
            "ID", "NAME", "STATUS", "REVENUE", "USERS"
        ));
        output.push_str(&format!(
            "{:─<80}\n",
            "".to_string()
        ));

        for project in projects {
            let status_icon = match project.status {
                crate::models::ProjectStatus::Live => "🟢",
                crate::models::ProjectStatus::Deploying => "🔵",
                crate::models::ProjectStatus::Failed => "🔴",
                crate::models::ProjectStatus::Paused => "🟡",
                _ => "⚪",
            };

            output.push_str(&format!(
                " {:8} │ {:30} │ {} {:10} │ {:>10} │ {:>10}\n",
                truncate(&project.id, 8),
                truncate(&project.name, 30),
                status_icon,
                project.status.to_string(),
                format!("${:.2}", project.total_revenue_usd),
                project.active_users
            ));
        }

        output.push_str(&format!(
            "{:─<80}\n",
            "".to_string()
        ));

        output
    }

    fn format_deployment(&self, deployment: &Deployment) -> String {
        let status_icon = match deployment.status {
            crate::models::DeploymentStatus::Ready => "✅",
            crate::models::DeploymentStatus::Building => "🔨",
            crate::models::DeploymentStatus::Deploying => "🚀",
            crate::models::DeploymentStatus::Failed => "❌",
            _ => "⏳",
        };

        format!(
            r#"
┌─────────────────────────────────────────────────────────────┐
│ {} Deployment {}
├─────────────────────────────────────────────────────────────┤
│ ID:       {}
│ Platform: {}
│ Status:   {} {}
│ Branch:   {}
│ URL:      {}
│ Deployed: {}
└─────────────────────────────────────────────────────────────┘
"#,
            status_icon,
            deployment.id.dimmed(),
            deployment.id,
            deployment.platform.to_string().cyan(),
            status_icon,
            deployment.status,
            deployment.branch,
            deployment.primary_url().unwrap_or("Pending..."),
            deployment
                .deployed_at
                .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or_else(|| "In progress...".to_string())
        )
    }

    fn format_stats(&self, stats: &ProjectStats) -> String {
        format!(
            r#"
╔═══════════════════════════════════════════════════════════════╗
║                    {} EMPIRE ANALYTICS                       ║
╠═══════════════════════════════════════════════════════════════╣
║                                                               ║
║   Total Projects:     {:>10}                               ║
║   Live Projects:      {:>10}  {}                               ║
║   Failed Projects:    {:>10}  {}                               ║
║                                                               ║
╠═══════════════════════════════════════════════════════════════╣
║                                                               ║
║   Total Revenue:      {:>10}                               ║
║   Total Users:        {:>10}                               ║
║   Avg Revenue/Project:{:>10}                               ║
║                                                               ║
╚═══════════════════════════════════════════════════════════════╝
"#,
            "🧬",
            stats.total_projects,
            "🟢",
            stats.live_projects,
            "🔴",
            stats.failed_projects,
            format!("${:.2}", stats.total_revenue).green(),
            stats.total_users.to_string().cyan(),
            format!("${:.2}", stats.avg_revenue_per_project)
        )
    }
}

/// JSON formatter for machine-readable output
pub struct JsonFormatter;

impl JsonFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for JsonFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl Formatter for JsonFormatter {
    fn format_opportunity(&self, opportunity: &Opportunity) -> String {
        serde_json::to_string_pretty(opportunity).unwrap_or_default()
    }

    fn format_opportunities(&self, opportunities: &[Opportunity]) -> String {
        serde_json::to_string_pretty(opportunities).unwrap_or_default()
    }

    fn format_project(&self, project: &Project) -> String {
        serde_json::to_string_pretty(project).unwrap_or_default()
    }

    fn format_projects(&self, projects: &[Project]) -> String {
        serde_json::to_string_pretty(projects).unwrap_or_default()
    }

    fn format_deployment(&self, deployment: &Deployment) -> String {
        serde_json::to_string_pretty(deployment).unwrap_or_default()
    }

    fn format_stats(&self, stats: &ProjectStats) -> String {
        serde_json::to_string_pretty(stats).unwrap_or_default()
    }
}

/// Helper functions
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

fn format_status(status: &str) -> ColoredString {
    match status.to_lowercase().as_str() {
        "live" | "ready" | "approved" | "completed" => status.green(),
        "pending" | "queued" | "initializing" | "evaluating" => status.yellow(),
        "failed" | "rejected" | "error" => status.red(),
        "deploying" | "building" | "generating" => status.blue(),
        "paused" | "deferred" => status.dimmed(),
        _ => status.normal(),
    }
}

/// Print a success message
pub fn print_success(message: &str) {
    println!("{} {}", "✓".green().bold(), message.green());
}

/// Print an error message
pub fn print_error(message: &str) {
    eprintln!("{} {}", "✗".red().bold(), message.red());
}

/// Print a warning message
pub fn print_warning(message: &str) {
    println!("{} {}", "⚠".yellow().bold(), message.yellow());
}

/// Print an info message
pub fn print_info(message: &str) {
    println!("{} {}", "ℹ".blue().bold(), message);
}

/// Print a header
pub fn print_header(message: &str) {
    println!("\n{}\n{}", message.bold(), "═".repeat(message.len()));
}

/// Create a progress spinner
pub fn create_spinner(message: &str) -> indicatif::ProgressBar {
    let pb = indicatif::ProgressBar::new_spinner();
    pb.set_style(
        indicatif::ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("{spinner:.blue} {msg}")
            .unwrap(),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    pb
}

/// Create a progress bar
pub fn create_progress_bar(len: u64, message: &str) -> indicatif::ProgressBar {
    let pb = indicatif::ProgressBar::new(len);
    pb.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("{msg} [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("█▓░"),
    );
    pb.set_message(message.to_string());
    pb
}

/// Prompt for confirmation
pub fn confirm(message: &str) -> bool {
    print!("{} {} [y/N] ", "?".blue().bold(), message);
    std::io::stdout().flush().unwrap();

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}

/// Prompt for input
pub fn prompt(message: &str) -> String {
    print!("{} {}: ", "?".blue().bold(), message);
    std::io::stdout().flush().unwrap();

    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{OpportunitySource, OpportunityStatus};

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world", 8), "hello...");
    }

    #[test]
    fn test_text_formatter() {
        let formatter = TextFormatter::new();
        let opp = Opportunity {
            id: "test-123".to_string(),
            title: "Test Opportunity".to_string(),
            description: "A test description".to_string(),
            source: OpportunitySource::GitHub,
            raw_data: Default::default(),
            pain_points: vec![],
            market_analysis: None,
            technical_analysis: None,
            monetization_analysis: None,
            score: Some(85.0),
            status: OpportunityStatus::Pending,
            rejection_reason: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let output = formatter.format_opportunity(&opp);
        assert!(output.contains("Test Opportunity"));
        assert!(output.contains("test-123"));
    }

    #[test]
    fn test_json_formatter() {
        let formatter = JsonFormatter::new();
        let opp = Opportunity {
            id: "test-123".to_string(),
            title: "Test Opportunity".to_string(),
            description: "A test description".to_string(),
            source: OpportunitySource::Manual,
            raw_data: Default::default(),
            pain_points: vec![],
            market_analysis: None,
            technical_analysis: None,
            monetization_analysis: None,
            score: Some(75.0),
            status: OpportunityStatus::Approved,
            rejection_reason: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let output = formatter.format_opportunity(&opp);
        assert!(output.contains("\"id\": \"test-123\""));
    }
}
