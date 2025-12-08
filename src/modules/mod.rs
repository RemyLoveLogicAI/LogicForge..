//! Functional modules for Genesis Engine

pub mod code_generator;
pub mod deployment_mesh;
pub mod market_predictor;
pub mod revenue_tracker;

pub use code_generator::{CodeGenerator, GeneratedCode, GeneratedFile, GeneratorConfig};
pub use deployment_mesh::{
    DeploymentMesh, DeploymentResult, DeploymentStrategy, HealthCheckResult, MeshConfig,
    MultiDeploymentResult,
};
pub use market_predictor::{
    ConfidenceLevel, MarketAnalysisResult, MarketPrediction, MarketPredictor, MarketSegment,
    MarketTrend, PredictorConfig,
};
pub use revenue_tracker::{
    EmpireSummary, RevenueAlert, RevenueConfig, RevenueReport, RevenueSummary, RevenueTracker,
    RevenueTrend,
};
