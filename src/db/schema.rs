//! Database schema definitions

/// SQL schema for creating tables
pub const CREATE_SCHEMA: &str = r#"
-- Opportunities table
CREATE TABLE IF NOT EXISTS opportunities (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    source TEXT NOT NULL,
    raw_data TEXT NOT NULL DEFAULT '{}',
    pain_points TEXT NOT NULL DEFAULT '[]',
    market_analysis TEXT,
    technical_analysis TEXT,
    monetization_analysis TEXT,
    score REAL,
    status TEXT NOT NULL DEFAULT 'pending',
    rejection_reason TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Projects table
CREATE TABLE IF NOT EXISTS projects (
    id TEXT PRIMARY KEY,
    opportunity_id TEXT REFERENCES opportunities(id),
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    github_repo TEXT,
    deployment_url TEXT,
    status TEXT NOT NULL DEFAULT 'initializing',
    template_type TEXT NOT NULL DEFAULT 'micro-saas',
    spec TEXT,
    genspark_project_id TEXT,
    health_status TEXT,
    total_revenue_usd REAL NOT NULL DEFAULT 0,
    active_users INTEGER NOT NULL DEFAULT 0,
    error_log TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL,
    launched_at TEXT,
    updated_at TEXT NOT NULL
);

-- Deployments table
CREATE TABLE IF NOT EXISTS deployments (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id),
    platform TEXT NOT NULL,
    platform_deployment_id TEXT,
    url TEXT,
    production_url TEXT,
    status TEXT NOT NULL DEFAULT 'queued',
    commit_sha TEXT,
    commit_message TEXT,
    branch TEXT NOT NULL DEFAULT 'main',
    build_logs TEXT NOT NULL DEFAULT '[]',
    env_vars TEXT NOT NULL DEFAULT '[]',
    domains TEXT NOT NULL DEFAULT '[]',
    build_time_seconds INTEGER,
    error_message TEXT,
    created_at TEXT NOT NULL,
    deployed_at TEXT,
    updated_at TEXT NOT NULL
);

-- Revenue metrics table
CREATE TABLE IF NOT EXISTS revenue_metrics (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id),
    metric_date TEXT NOT NULL,
    revenue_usd REAL NOT NULL DEFAULT 0,
    active_users INTEGER NOT NULL DEFAULT 0,
    conversions INTEGER NOT NULL DEFAULT 0,
    recorded_at TEXT NOT NULL
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_opportunities_score ON opportunities(score DESC);
CREATE INDEX IF NOT EXISTS idx_opportunities_status ON opportunities(status);
CREATE INDEX IF NOT EXISTS idx_opportunities_source ON opportunities(source);
CREATE INDEX IF NOT EXISTS idx_opportunities_created ON opportunities(created_at DESC);

CREATE INDEX IF NOT EXISTS idx_projects_status ON projects(status);
CREATE INDEX IF NOT EXISTS idx_projects_opportunity ON projects(opportunity_id);
CREATE INDEX IF NOT EXISTS idx_projects_created ON projects(created_at DESC);

CREATE INDEX IF NOT EXISTS idx_deployments_project ON deployments(project_id);
CREATE INDEX IF NOT EXISTS idx_deployments_status ON deployments(status);
CREATE INDEX IF NOT EXISTS idx_deployments_created ON deployments(created_at DESC);

CREATE INDEX IF NOT EXISTS idx_revenue_project_date ON revenue_metrics(project_id, metric_date);
CREATE INDEX IF NOT EXISTS idx_revenue_date ON revenue_metrics(metric_date DESC);
"#;

/// SQL for dropping all tables (for testing/reset)
pub const DROP_SCHEMA: &str = r#"
DROP TABLE IF EXISTS revenue_metrics;
DROP TABLE IF EXISTS deployments;
DROP TABLE IF EXISTS projects;
DROP TABLE IF EXISTS opportunities;
"#;
