//! Core modules for Genesis Engine

pub mod decision_matrix;
pub mod execution_coordinator;
pub mod opportunity_scanner;
pub mod resource_allocator;

pub use decision_matrix::{DecisionMatrix, EvaluationWeights, MatrixConfig};
pub use execution_coordinator::{
    CoordinatorConfig, ExecutionCoordinator, HealthCheck, LaunchOptions, ProjectHealthReport,
};
pub use opportunity_scanner::{OpportunityScanner, Scanner, ScannerConfig};
pub use resource_allocator::{
    AllocatorConfig, AllocationResult, BudgetSummary, ProjectAllocation, ReallocationAction,
    ResourceAllocator,
};
