//! Genesis Engine CLI
//!
//! Main entry point for the Genesis Engine command-line interface.

use clap::Parser;
use genesis_engine::cli::{
    commands::*, output::*, Cli, Commands, OutputFormat,
};
use genesis_engine::core::{
    DecisionMatrix, ExecutionCoordinator, LaunchOptions, OpportunityScanner, CoordinatorConfig,
};
use genesis_engine::db::Database;
use genesis_engine::integrations::{GenSparkClient, GitHubClient};
use genesis_engine::models::{Platform, Portfolio};
use genesis_engine::modules::RevenueTracker;
use genesis_engine::utils::{config::GenesisConfig, error::Result, logger};
use std::path::PathBuf;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        print_error(&format!("{}", e));
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    logger::init_logger(cli.verbose);

    // Load configuration
    let config = GenesisConfig::load(cli.config.as_ref().map(std::path::Path::new))?;

    // Execute command
    match cli.command {
        Commands::Init(args) => cmd_init(args, &config).await,
        Commands::Scan(args) => cmd_scan(args, &config, cli.format).await,
        Commands::Launch(args) => cmd_launch(args, &config).await,
        Commands::Monitor(args) => cmd_monitor(args, &config, cli.format).await,
        Commands::Deploy(args) => cmd_deploy(args, &config).await,
        Commands::Analytics(args) => cmd_analytics(args, &config, cli.format).await,
        Commands::Config(args) => cmd_config(args, &config).await,
        Commands::List(args) => cmd_list(args, &config, cli.format).await,
        Commands::Show(args) => cmd_show(args, &config, cli.format).await,
        Commands::Ai(args) => cmd_ai(args, &config).await,
        Commands::Health(args) => cmd_health(args, &config).await,
    }
}

/// Initialize Genesis in the current directory
async fn cmd_init(args: InitArgs, config: &GenesisConfig) -> Result<()> {
    print_header("🧬 Genesis Engine Initialization");

    let empire_name = args.name.unwrap_or_else(|| {
        if args.non_interactive {
            config.empire.name.clone()
        } else {
            prompt("Empire name")
        }
    });

    if empire_name.is_empty() {
        return Err(genesis_engine::utils::error::GenesisError::Validation(
            "Empire name cannot be empty".to_string(),
        ));
    }

    // Create directories
    let spinner = create_spinner("Creating project structure...");

    std::fs::create_dir_all("config")?;
    std::fs::create_dir_all("data")?;
    std::fs::create_dir_all("templates")?;

    spinner.finish_with_message("Project structure created");

    // Create default config
    let spinner = create_spinner("Creating configuration...");

    let mut new_config = config.clone();
    new_config.empire.name = empire_name.clone();
    new_config.save(std::path::Path::new("config/default.toml"))?;

    // Create example config
    let example_config = include_str!("../config/example.toml");
    std::fs::write("config/example.toml", example_config).ok();

    spinner.finish_with_message("Configuration created");

    // Initialize database
    let spinner = create_spinner("Initializing database...");

    let db_path = PathBuf::from(&config.database.path);
    let db = Database::new(&db_path)?;
    db.initialize()?;

    spinner.finish_with_message("Database initialized");

    // Create .env template
    std::fs::write(
        ".env.example",
        r#"# Genesis Engine Environment Variables
GENSPARK_API_KEY=
GENSPARK_WORKSPACE_ID=
GITHUB_TOKEN=
VERCEL_TOKEN=
CLOUDFLARE_TOKEN=
CLOUDFLARE_ACCOUNT_ID=
"#,
    )?;

    print_success(&format!(
        "Genesis Engine initialized successfully!\n\n\
        Empire: {}\n\
        Config: config/default.toml\n\
        Database: {}\n\n\
        Next steps:\n\
        1. Copy .env.example to .env and add your API keys\n\
        2. Run 'genesis scan' to discover opportunities\n\
        3. Run 'genesis launch' to launch your first project",
        empire_name,
        db_path.display()
    ));

    Ok(())
}

/// Scan for opportunities
async fn cmd_scan(args: ScanArgs, config: &GenesisConfig, format: OutputFormat) -> Result<()> {
    print_header("🔍 Scanning for Opportunities");

    let db = Arc::new(Database::new(&config.get_db_path()?)?);
    db.initialize()?;

    let mut scanner = OpportunityScanner::from_config(config, db.clone());

    let spinner = create_spinner("Scanning sources...");

    let mut opportunities = scanner.scan_all_sources(args.limit).await?;

    spinner.finish_with_message(format!("Found {} opportunities", opportunities.len()));

    // Score opportunities
    for opp in &mut opportunities {
        scanner.score_opportunity(opp);
    }

    // Filter by minimum score
    if let Some(min_score) = args.min_score {
        opportunities.retain(|o| o.score.unwrap_or(0.0) >= min_score);
    }

    // Save to database
    if args.save {
        scanner.save_opportunities(&opportunities).await?;
        print_info(&format!("Saved {} opportunities to database", opportunities.len()));
    }

    // Output results
    match format {
        OutputFormat::Json => {
            let formatter = JsonFormatter::new();
            println!("{}", formatter.format_opportunities(&opportunities));
        }
        _ => {
            let formatter = TextFormatter::new();
            println!("{}", formatter.format_opportunities(&opportunities));
        }
    }

    Ok(())
}

/// Launch a project from an opportunity
async fn cmd_launch(args: LaunchArgs, config: &GenesisConfig) -> Result<()> {
    print_header("🚀 Launching Project");

    let db = Arc::new(Database::new(&config.get_db_path()?)?);
    db.initialize()?;

    // Get opportunity
    let opportunity = db
        .get_opportunity(&args.opportunity_id)?
        .ok_or_else(|| genesis_engine::utils::error::GenesisError::OpportunityNotFound(
            args.opportunity_id.clone(),
        ))?;

    print_info(&format!("Launching from opportunity: {}", opportunity.title));

    // Create clients
    let genspark = config
        .ai
        .genspark
        .api_key
        .as_ref()
        .filter(|k| !k.is_empty())
        .map(|key| {
            GenSparkClient::new(
                key.clone(),
                config.ai.genspark.workspace_id.clone(),
                Some(config.ai.genspark.base_url.clone()),
            )
        });

    let github = config
        .github
        .token
        .as_ref()
        .filter(|t| !t.is_empty())
        .map(|token| GitHubClient::new(token.clone(), config.github.org.clone()));

    // Create coordinator
    let coordinator = ExecutionCoordinator::new(
        CoordinatorConfig {
            auto_deploy: args.auto_deploy,
            ..Default::default()
        },
        db,
        genspark,
        github,
    );

    // Launch options
    let options = LaunchOptions {
        name: args.name,
        template: args.template.map(|t| t.parse().unwrap_or_default()),
        auto_deploy: args.auto_deploy,
        platform: args.platform.parse().unwrap_or(Platform::Vercel),
        skip_review: args.skip_review,
    };

    let spinner = create_spinner("Launching project...");

    let project = coordinator.launch_project(&opportunity, options).await?;

    spinner.finish_with_message("Project launched!");

    print_success(&format!(
        "Project launched successfully!\n\n\
        ID: {}\n\
        Name: {}\n\
        Status: {}\n\
        GitHub: {}\n\
        URL: {}",
        project.id,
        project.name,
        project.status,
        project.github_repo.as_deref().unwrap_or("Not set"),
        project.deployment_url.as_deref().unwrap_or("Not deployed")
    ));

    Ok(())
}

/// Monitor projects
async fn cmd_monitor(args: MonitorArgs, config: &GenesisConfig, format: OutputFormat) -> Result<()> {
    print_header("📊 Project Monitor");

    let db = Arc::new(Database::new(&config.get_db_path()?)?);
    db.initialize()?;

    loop {
        let projects = if let Some(ref project_id) = args.project {
            db.get_project(project_id)?
                .map(|p| vec![p])
                .unwrap_or_default()
        } else {
            db.list_projects(None, None)?
        };

        if projects.is_empty() {
            print_warning("No projects found");
            return Ok(());
        }

        // Filter for issues only if requested
        let projects: Vec<_> = if args.issues_only {
            projects
                .into_iter()
                .filter(|p| {
                    p.health_status
                        .as_ref()
                        .map(|h| !h.is_healthy)
                        .unwrap_or(false)
                        || !p.error_log.is_empty()
                })
                .collect()
        } else {
            projects
        };

        match format {
            OutputFormat::Json => {
                let formatter = JsonFormatter::new();
                println!("{}", formatter.format_projects(&projects));
            }
            _ => {
                // Clear screen in watch mode
                if args.watch {
                    print!("\x1B[2J\x1B[1;1H");
                }

                let formatter = TextFormatter::new();
                println!("{}", formatter.format_projects(&projects));

                // Show summary
                let stats = db.get_portfolio_stats()?;
                println!("{}", formatter.format_stats(&stats));
            }
        }

        if !args.watch {
            break;
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(args.interval)).await;
    }

    Ok(())
}

/// Deploy a project
async fn cmd_deploy(args: DeployArgs, config: &GenesisConfig) -> Result<()> {
    print_header("🚢 Deploying Project");

    let db = Arc::new(Database::new(&config.get_db_path()?)?);
    db.initialize()?;

    let project = db
        .get_project(&args.project_id)?
        .ok_or_else(|| genesis_engine::utils::error::GenesisError::ProjectNotFound(
            args.project_id.clone(),
        ))?;

    print_info(&format!("Deploying: {} to {}", project.name, args.platform));

    let coordinator = ExecutionCoordinator::new(
        CoordinatorConfig::default(),
        db.clone(),
        None,
        None,
    );

    let platform: Platform = args.platform.parse().unwrap_or(Platform::Vercel);

    let spinner = create_spinner("Deploying...");

    let deployment = coordinator.deploy_project(&project, platform).await?;

    spinner.finish_with_message("Deployment complete!");

    db.save_deployment(&deployment)?;

    let formatter = TextFormatter::new();
    println!("{}", formatter.format_deployment(&deployment));

    Ok(())
}

/// Show analytics
async fn cmd_analytics(args: AnalyticsArgs, config: &GenesisConfig, format: OutputFormat) -> Result<()> {
    print_header("📈 Empire Analytics");

    let db = Arc::new(Database::new(&config.get_db_path()?)?);
    db.initialize()?;

    let tracker = RevenueTracker::new(
        genesis_engine::modules::RevenueConfig::default(),
        db.clone(),
    );

    let report = tracker.generate_report(args.days)?;

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        _ => {
            println!(
                r#"
╔═══════════════════════════════════════════════════════════════╗
║                    🧬 EMPIRE ANALYTICS                        ║
║                    Period: {} days                            ║
╠═══════════════════════════════════════════════════════════════╣
║                                                               ║
║   Total Revenue:      ${:<12.2}                             ║
║   Estimated MRR:      ${:<12.2}                             ║
║   Estimated ARR:      ${:<12.2}                             ║
║   ARPU:               ${:<12.2}                             ║
║                                                               ║
║   Total Projects:     {:<10}                                ║
║   Active Projects:    {:<10}                                ║
║   Peak Users:         {:<10}                                ║
║                                                               ║
║   Trend:              {}                                     ║
╚═══════════════════════════════════════════════════════════════╝
"#,
                report.period_days,
                report.total_revenue,
                report.estimated_mrr,
                report.estimated_arr,
                report.arpu,
                report.total_projects,
                report.active_projects,
                report.peak_users,
                report.trend
            );

            if !report.top_performers.is_empty() {
                println!("\n📊 Top Performers:");
                for (name, revenue) in &report.top_performers {
                    println!("  • {} - ${:.2}", name, revenue);
                }
            }

            if !report.recommendations.is_empty() {
                println!("\n💡 Recommendations:");
                for rec in &report.recommendations {
                    println!("  • {}", rec);
                }
            }

            if !report.alerts.is_empty() {
                println!("\n⚠️ Alerts:");
                for alert in &report.alerts {
                    println!("  • {}", alert.message);
                }
            }
        }
    }

    // Export if requested
    if let Some(path) = args.export {
        let json = serde_json::to_string_pretty(&report)?;
        std::fs::write(&path, json)?;
        print_success(&format!("Exported to {}", path));
    }

    Ok(())
}

/// Manage configuration
async fn cmd_config(args: ConfigArgs, config: &GenesisConfig) -> Result<()> {
    match args.action {
        ConfigAction::Show { show_secrets } => {
            print_header("⚙️ Configuration");

            let mut display_config = config.clone();

            if !show_secrets {
                // Mask sensitive values
                if display_config.ai.genspark.api_key.is_some() {
                    display_config.ai.genspark.api_key = Some("***".to_string());
                }
                if display_config.ai.openai.api_key.is_some() {
                    display_config.ai.openai.api_key = Some("***".to_string());
                }
                if display_config.github.token.is_some() {
                    display_config.github.token = Some("***".to_string());
                }
                if display_config.deployment.vercel.api_token.is_some() {
                    display_config.deployment.vercel.api_token = Some("***".to_string());
                }
                if display_config.deployment.cloudflare.api_token.is_some() {
                    display_config.deployment.cloudflare.api_token = Some("***".to_string());
                }
            }

            let toml = toml::to_string_pretty(&display_config)?;
            println!("{}", toml);
        }

        ConfigAction::Set { key, value } => {
            print_warning(&format!(
                "Setting configuration values via CLI not yet implemented.\n\
                 Please edit config/default.toml directly.\n\
                 Key: {}, Value: {}",
                key, value
            ));
        }

        ConfigAction::Get { key } => {
            let toml_str = toml::to_string(config)?;
            let value: toml::Value = toml::from_str(&toml_str)?;

            let parts: Vec<&str> = key.split('.').collect();
            let mut current = &value;

            for part in &parts {
                current = current.get(part).ok_or_else(|| {
                    genesis_engine::utils::error::GenesisError::Config(format!(
                        "Key not found: {}",
                        key
                    ))
                })?;
            }

            println!("{} = {}", key, current);
        }

        ConfigAction::Edit => {
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
            std::process::Command::new(editor)
                .arg("config/default.toml")
                .status()?;
        }

        ConfigAction::Reset { force } => {
            if !force && !confirm("Reset configuration to defaults?") {
                print_info("Cancelled");
                return Ok(());
            }

            let default = GenesisConfig::default();
            default.save(std::path::Path::new("config/default.toml"))?;
            print_success("Configuration reset to defaults");
        }

        ConfigAction::Validate => {
            match config.validate() {
                Ok(()) => print_success("Configuration is valid"),
                Err(e) => print_error(&format!("Configuration error: {}", e)),
            }
        }
    }

    Ok(())
}

/// List resources
async fn cmd_list(args: ListArgs, config: &GenesisConfig, format: OutputFormat) -> Result<()> {
    let db = Database::new(&config.get_db_path()?)?;
    db.initialize()?;

    match args.resource {
        ResourceType::Opportunities => {
            let opportunities = db.list_opportunities(
                args.status.as_deref(),
                None,
                None,
                args.limit,
            )?;

            match format {
                OutputFormat::Json => {
                    let formatter = JsonFormatter::new();
                    println!("{}", formatter.format_opportunities(&opportunities));
                }
                _ => {
                    let formatter = TextFormatter::new();
                    println!("{}", formatter.format_opportunities(&opportunities));
                }
            }
        }

        ResourceType::Projects => {
            let projects = db.list_projects(args.status.as_deref(), args.limit)?;

            match format {
                OutputFormat::Json => {
                    let formatter = JsonFormatter::new();
                    println!("{}", formatter.format_projects(&projects));
                }
                _ => {
                    let formatter = TextFormatter::new();
                    println!("{}", formatter.format_projects(&projects));
                }
            }
        }

        ResourceType::Deployments => {
            print_info("Listing all deployments...");
            let projects = db.list_projects(None, None)?;

            for project in projects {
                let deployments = db.list_deployments_for_project(&project.id)?;
                if !deployments.is_empty() {
                    println!("\nProject: {}", project.name);
                    let formatter = TextFormatter::new();
                    for d in deployments {
                        println!("{}", formatter.format_deployment(&d));
                    }
                }
            }
        }

        ResourceType::Revenue => {
            let stats = db.get_portfolio_stats()?;
            let formatter = TextFormatter::new();
            println!("{}", formatter.format_stats(&stats));
        }
    }

    Ok(())
}

/// Show resource details
async fn cmd_show(args: ShowArgs, config: &GenesisConfig, format: OutputFormat) -> Result<()> {
    let db = Database::new(&config.get_db_path()?)?;
    db.initialize()?;

    match args.resource {
        ResourceType::Opportunities => {
            let opp = db.get_opportunity(&args.id)?.ok_or_else(|| {
                genesis_engine::utils::error::GenesisError::OpportunityNotFound(args.id.clone())
            })?;

            match format {
                OutputFormat::Json => {
                    let formatter = JsonFormatter::new();
                    println!("{}", formatter.format_opportunity(&opp));
                }
                _ => {
                    let formatter = TextFormatter::new();
                    println!("{}", formatter.format_opportunity(&opp));
                }
            }
        }

        ResourceType::Projects => {
            let project = db.get_project(&args.id)?.ok_or_else(|| {
                genesis_engine::utils::error::GenesisError::ProjectNotFound(args.id.clone())
            })?;

            match format {
                OutputFormat::Json => {
                    let formatter = JsonFormatter::new();
                    println!("{}", formatter.format_project(&project));
                }
                _ => {
                    let formatter = TextFormatter::new();
                    println!("{}", formatter.format_project(&project));

                    if args.full && !project.error_log.is_empty() {
                        println!("\n📋 Error Log:");
                        for error in &project.error_log {
                            println!("  {}", error);
                        }
                    }
                }
            }
        }

        ResourceType::Deployments => {
            let deployment = db.get_deployment(&args.id)?.ok_or_else(|| {
                genesis_engine::utils::error::GenesisError::custom(format!(
                    "Deployment not found: {}",
                    args.id
                ))
            })?;

            let formatter = TextFormatter::new();
            println!("{}", formatter.format_deployment(&deployment));
        }

        ResourceType::Revenue => {
            print_info("Use 'genesis analytics' for revenue details");
        }
    }

    Ok(())
}

/// AI operations
async fn cmd_ai(args: AiArgs, config: &GenesisConfig) -> Result<()> {
    match args.action {
        AiAction::Analyze { opportunity_id } => {
            print_header("🤖 AI Analysis");

            let db = Database::new(&config.get_db_path()?)?;
            db.initialize()?;

            let opp = db.get_opportunity(&opportunity_id)?.ok_or_else(|| {
                genesis_engine::utils::error::GenesisError::OpportunityNotFound(
                    opportunity_id.clone(),
                )
            })?;

            let matrix = DecisionMatrix::default_matrix();
            let portfolio = Portfolio::new();

            let decision = matrix.should_launch(&opp, &portfolio);

            println!("\nOpportunity: {}", opp.title);
            println!("Score: {}", opp.score.unwrap_or(0.0));
            println!("\nDecision: {:?}", decision);
        }

        AiAction::Generate { project_id, instructions } => {
            print_warning("Code generation requires GenSpark API key");
            print_info(&format!("Project: {}", project_id));
            if let Some(inst) = instructions {
                print_info(&format!("Instructions: {}", inst));
            }
        }

        AiAction::Review { project_id } => {
            print_warning("Code review requires GenSpark API key");
            print_info(&format!("Project: {}", project_id));
        }

        AiAction::Prompt { prompt, output } => {
            print_warning("AI prompt requires GenSpark API key");
            print_info(&format!("Prompt: {}", prompt));
            if let Some(out) = output {
                print_info(&format!("Output: {}", out));
            }
        }
    }

    Ok(())
}

/// Health check
async fn cmd_health(_args: HealthArgs, config: &GenesisConfig) -> Result<()> {
    print_header("🏥 System Health Check");

    let mut all_healthy = true;

    // Check database
    let db_check = match Database::new(&config.get_db_path()?) {
        Ok(db) => {
            db.initialize()?;
            ("Database", true, "OK".to_string())
        }
        Err(e) => {
            all_healthy = false;
            ("Database", false, e.to_string())
        }
    };

    // Check configuration
    let config_check = match config.validate() {
        Ok(()) => ("Configuration", true, "Valid".to_string()),
        Err(e) => {
            all_healthy = false;
            ("Configuration", false, e.to_string())
        }
    };

    // Check API keys
    let genspark_check = if config.ai.genspark.api_key.as_ref().filter(|k| !k.is_empty()).is_some() {
        ("GenSpark API", true, "Configured".to_string())
    } else {
        ("GenSpark API", false, "Not configured".to_string())
    };

    let github_check = if config.github.token.as_ref().filter(|t| !t.is_empty()).is_some() {
        ("GitHub API", true, "Configured".to_string())
    } else {
        ("GitHub API", false, "Not configured".to_string())
    };

    let vercel_check = if config.deployment.vercel.api_token.as_ref().filter(|t| !t.is_empty()).is_some() {
        ("Vercel API", true, "Configured".to_string())
    } else {
        ("Vercel API", false, "Not configured".to_string())
    };

    // Display results
    let checks = vec![db_check, config_check, genspark_check, github_check, vercel_check];

    println!();
    for (name, healthy, message) in checks {
        let status = if healthy { "✅" } else { "❌" };
        println!("  {} {} - {}", status, name, message);
    }
    println!();

    if all_healthy {
        print_success("All systems operational");
    } else {
        print_warning("Some components need attention");
    }

    Ok(())
}
