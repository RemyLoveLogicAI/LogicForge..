//! CLI command definitions using clap

use clap::{Args, Parser, Subcommand, ValueEnum};

/// Genesis Engine - Autonomous AI Empire Orchestrator
#[derive(Parser, Debug)]
#[command(
    name = "genesis",
    author = "Genesis Team",
    version,
    about = "🧬 Autonomous AI Empire Orchestrator",
    long_about = "Genesis Engine is a self-operating business platform that discovers opportunities, \
                  generates products via AI, deploys them, and monitors revenue—all with minimal human intervention."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Path to configuration file
    #[arg(short, long, global = true, env = "GENESIS_CONFIG")]
    pub config: Option<String>,

    /// Output format
    #[arg(long, global = true, default_value = "text")]
    pub format: OutputFormat,
}

/// Output format options
#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
    Table,
}

/// Main commands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize Genesis in the current directory
    Init(InitArgs),

    /// Scan for new opportunities across configured sources
    Scan(ScanArgs),

    /// Launch a new project from an opportunity
    Launch(LaunchArgs),

    /// Monitor all active projects
    Monitor(MonitorArgs),

    /// Deploy a project to a platform
    Deploy(DeployArgs),

    /// Show empire analytics and statistics
    Analytics(AnalyticsArgs),

    /// Manage configuration settings
    Config(ConfigArgs),

    /// List resources (opportunities, projects, deployments)
    List(ListArgs),

    /// Show detailed information about a resource
    Show(ShowArgs),

    /// Execute AI-powered operations
    #[command(name = "ai")]
    Ai(AiArgs),

    /// Health check for the system
    Health(HealthArgs),
}

/// Arguments for init command
#[derive(Args, Debug)]
pub struct InitArgs {
    /// Name for the empire
    #[arg(short, long)]
    pub name: Option<String>,

    /// Skip interactive prompts
    #[arg(long)]
    pub non_interactive: bool,

    /// Force initialization even if already initialized
    #[arg(short, long)]
    pub force: bool,
}

/// Arguments for scan command
#[derive(Args, Debug)]
pub struct ScanArgs {
    /// Sources to scan (comma-separated: github,reddit,producthunt,hackernews)
    #[arg(short, long, default_value = "all")]
    pub sources: String,

    /// Maximum number of opportunities to return
    #[arg(short, long)]
    pub limit: Option<usize>,

    /// Minimum score threshold (0-100)
    #[arg(long)]
    pub min_score: Option<f64>,

    /// Keywords to filter by (comma-separated)
    #[arg(short, long)]
    pub keywords: Option<String>,

    /// Run scan continuously at configured interval
    #[arg(long)]
    pub continuous: bool,

    /// Save results to database
    #[arg(long, default_value = "true")]
    pub save: bool,
}

/// Arguments for launch command
#[derive(Args, Debug)]
pub struct LaunchArgs {
    /// ID of the opportunity to launch
    #[arg(short, long)]
    pub opportunity_id: String,

    /// Project name (defaults to generated name)
    #[arg(short, long)]
    pub name: Option<String>,

    /// Template type to use
    #[arg(short, long)]
    pub template: Option<String>,

    /// Automatically deploy after generation
    #[arg(short, long)]
    pub auto_deploy: bool,

    /// Platform to deploy to
    #[arg(short, long, default_value = "vercel")]
    pub platform: String,

    /// Skip AI review of generated code
    #[arg(long)]
    pub skip_review: bool,
}

/// Arguments for monitor command
#[derive(Args, Debug)]
pub struct MonitorArgs {
    /// Watch mode - continuously update
    #[arg(short, long)]
    pub watch: bool,

    /// Refresh interval in seconds (for watch mode)
    #[arg(long, default_value = "30")]
    pub interval: u64,

    /// Only show projects with issues
    #[arg(long)]
    pub issues_only: bool,

    /// Project ID to monitor (monitors all if not specified)
    #[arg(short, long)]
    pub project: Option<String>,
}

/// Arguments for deploy command
#[derive(Args, Debug)]
pub struct DeployArgs {
    /// ID of the project to deploy
    #[arg(short = 'i', long)]
    pub project_id: String,

    /// Platform to deploy to
    #[arg(short = 'P', long, default_value = "vercel")]
    pub platform: String,

    /// Git branch to deploy
    #[arg(short, long, default_value = "main")]
    pub branch: String,

    /// Environment variables (KEY=VALUE format, can be repeated)
    #[arg(short, long)]
    pub env: Vec<String>,

    /// Production deployment
    #[arg(long)]
    pub production: bool,
}

/// Arguments for analytics command
#[derive(Args, Debug)]
pub struct AnalyticsArgs {
    /// Number of days to analyze
    #[arg(short, long, default_value = "30")]
    pub days: u32,

    /// Group by (day, week, month)
    #[arg(short, long, default_value = "day")]
    pub group_by: String,

    /// Show detailed breakdown
    #[arg(long)]
    pub detailed: bool,

    /// Export analytics to file
    #[arg(long)]
    pub export: Option<String>,
}

/// Arguments for config command
#[derive(Args, Debug)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub action: ConfigAction,
}

/// Config subcommands
#[derive(Subcommand, Debug)]
pub enum ConfigAction {
    /// Show current configuration
    Show {
        /// Show sensitive values (tokens, keys)
        #[arg(long)]
        show_secrets: bool,
    },

    /// Set a configuration value
    Set {
        /// Configuration key (e.g., empire.name)
        key: String,
        /// Value to set
        value: String,
    },

    /// Get a specific configuration value
    Get {
        /// Configuration key
        key: String,
    },

    /// Open configuration in editor
    Edit,

    /// Reset configuration to defaults
    Reset {
        /// Skip confirmation
        #[arg(short, long)]
        force: bool,
    },

    /// Validate configuration
    Validate,
}

/// Arguments for list command
#[derive(Args, Debug)]
pub struct ListArgs {
    /// Resource type to list
    #[arg(value_enum)]
    pub resource: ResourceType,

    /// Filter by status
    #[arg(short, long)]
    pub status: Option<String>,

    /// Maximum items to show
    #[arg(short, long)]
    pub limit: Option<usize>,

    /// Sort by field
    #[arg(long)]
    pub sort: Option<String>,

    /// Sort descending
    #[arg(long)]
    pub desc: bool,
}

/// Resource types for list command
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ResourceType {
    Opportunities,
    Projects,
    Deployments,
    Revenue,
}

/// Arguments for show command
#[derive(Args, Debug)]
pub struct ShowArgs {
    /// Resource type
    #[arg(value_enum)]
    pub resource: ResourceType,

    /// Resource ID
    pub id: String,

    /// Show full details including logs
    #[arg(long)]
    pub full: bool,
}

/// Arguments for AI command
#[derive(Args, Debug)]
pub struct AiArgs {
    #[command(subcommand)]
    pub action: AiAction,
}

/// AI subcommands
#[derive(Subcommand, Debug)]
pub enum AiAction {
    /// Analyze an opportunity
    Analyze {
        /// Opportunity ID
        opportunity_id: String,
    },

    /// Generate code for a project
    Generate {
        /// Project ID
        project_id: String,

        /// Additional instructions
        #[arg(short, long)]
        instructions: Option<String>,
    },

    /// Review generated code
    Review {
        /// Project ID
        project_id: String,
    },

    /// Send a custom prompt
    Prompt {
        /// The prompt to send
        prompt: String,

        /// Save response to file
        #[arg(short, long)]
        output: Option<String>,
    },
}

/// Arguments for health command
#[derive(Args, Debug)]
pub struct HealthArgs {
    /// Check specific service
    #[arg(short, long)]
    pub service: Option<String>,

    /// Verbose health check output
    #[arg(long)]
    pub detailed: bool,
}

impl Cli {
    /// Parse CLI arguments
    pub fn parse_args() -> Self {
        Self::parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert();
    }

    #[test]
    fn test_parse_init() {
        let cli = Cli::parse_from(["genesis", "init", "--name", "my-empire"]);
        if let Commands::Init(args) = cli.command {
            assert_eq!(args.name, Some("my-empire".to_string()));
        } else {
            panic!("Expected Init command");
        }
    }

    #[test]
    fn test_parse_scan_with_sources() {
        let cli = Cli::parse_from(["genesis", "scan", "--sources", "github,reddit", "--limit", "10"]);
        if let Commands::Scan(args) = cli.command {
            assert_eq!(args.sources, "github,reddit");
            assert_eq!(args.limit, Some(10));
        } else {
            panic!("Expected Scan command");
        }
    }

    #[test]
    fn test_verbose_flag() {
        let cli = Cli::parse_from(["genesis", "-v", "init"]);
        assert!(cli.verbose);
    }
}
