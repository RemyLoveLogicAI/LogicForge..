//! Genesis Engine - Autonomous AI Empire Orchestrator
//!
//! Genesis Engine is a production-grade CLI application that serves as the control plane
//! for an autonomous AI business empire. This system discovers opportunities, generates
//! products via AI, deploys them, and monitors revenue—all with minimal human intervention.
//!
//! # Architecture
//!
//! The crate is organized into several modules:
//!
//! - **cli**: Command-line interface components
//! - **core**: Core business logic (scanner, decision matrix, coordinator)
//! - **modules**: Functional modules (revenue tracking, market prediction, code generation)
//! - **integrations**: External service integrations (GenSpark, GitHub, Vercel, Cloudflare)
//! - **models**: Data models and types
//! - **db**: Database operations and schema
//! - **utils**: Utilities (error handling, configuration, logging)
//!
//! # Quick Start
//!
//! ```no_run
//! use genesis_engine::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // Load configuration
//!     let config = GenesisConfig::load(None)?;
//!
//!     // Initialize database
//!     let db = Database::new(&config.get_db_path()?)?;
//!     db.initialize()?;
//!
//!     // Create opportunity scanner
//!     let mut scanner = OpportunityScanner::from_config(&config, db.into());
//!
//!     // Scan for opportunities
//!     let opportunities = scanner.scan_all_sources(Some(10)).await?;
//!
//!     println!("Found {} opportunities", opportunities.len());
//!     Ok(())
//! }
//! ```

// #![warn(missing_docs)]  // Enable when all docs are written

pub mod cli;
pub mod core;
pub mod db;
pub mod integrations;
pub mod models;
pub mod modules;
pub mod utils;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::cli::{
        Cli, Commands, Formatter, JsonFormatter, OutputFormat, TextFormatter,
    };
    pub use crate::core::{
        DecisionMatrix, ExecutionCoordinator, LaunchOptions, OpportunityScanner, ResourceAllocator,
        ScannerConfig,
    };
    pub use crate::db::Database;
    pub use crate::integrations::{GenSparkClient, GitHubClient};
    pub use crate::models::{
        Decision, Deployment, DeploymentConfig, Opportunity, OpportunitySource, Platform, Portfolio,
        Project, ProjectSpec, TemplateType,
    };
    pub use crate::modules::{
        CodeGenerator, DeploymentMesh, MarketPredictor, RevenueTracker,
    };
    pub use crate::utils::{GenesisConfig, GenesisError, Result};
}

/// Application version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application name
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// Application description
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
