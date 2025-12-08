# Genesis Engine Architecture

## Overview

Genesis Engine is built as a modular, extensible system designed for autonomous business operation. The architecture follows clean code principles with clear separation of concerns.

## System Components

```
┌─────────────────────────────────────────────────────────────────┐
│                         CLI Layer                                │
│  Commands: init, scan, launch, monitor, deploy, analytics       │
└─────────────────────────────────┬───────────────────────────────┘
                                  │
┌─────────────────────────────────▼───────────────────────────────┐
│                        Core Layer                                │
│  ┌──────────────┐  ┌─────────────┐  ┌──────────────────────┐   │
│  │ Opportunity  │  │  Decision   │  │     Execution        │   │
│  │   Scanner    │  │   Matrix    │  │    Coordinator       │   │
│  └──────────────┘  └─────────────┘  └──────────────────────┘   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                  Resource Allocator                       │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────┬───────────────────────────────┘
                                  │
┌─────────────────────────────────▼───────────────────────────────┐
│                      Modules Layer                               │
│  ┌──────────────┐  ┌─────────────┐  ┌──────────────────────┐   │
│  │   Revenue    │  │   Market    │  │       Code           │   │
│  │   Tracker    │  │  Predictor  │  │     Generator        │   │
│  └──────────────┘  └─────────────┘  └──────────────────────┘   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                   Deployment Mesh                         │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────┬───────────────────────────────┘
                                  │
┌─────────────────────────────────▼───────────────────────────────┐
│                   Integration Layer                              │
│  ┌────────────┐  ┌────────────┐  ┌────────────┐  ┌───────────┐ │
│  │  GenSpark  │  │   GitHub   │  │   Vercel   │  │Cloudflare │ │
│  └────────────┘  └────────────┘  └────────────┘  └───────────┘ │
└─────────────────────────────────┬───────────────────────────────┘
                                  │
┌─────────────────────────────────▼───────────────────────────────┐
│                    Data Layer                                    │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                   SQLite Database                         │   │
│  │  Tables: opportunities, projects, deployments, revenue    │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

## Data Flow

### Opportunity Discovery Flow

```
External Sources     Scanner        Database       Decision Matrix
     │                  │              │                 │
     │   Fetch data     │              │                 │
     │ ─────────────────>              │                 │
     │                  │              │                 │
     │   Parse & Score  │              │                 │
     │                  │   Save       │                 │
     │                  │─────────────>│                 │
     │                  │              │   Evaluate      │
     │                  │              │────────────────>│
     │                  │              │                 │
     │                  │              │    Decision     │
     │                  │              │<────────────────│
```

### Project Launch Flow

```
User          Coordinator      GenSpark       GitHub         Vercel
 │                │                │             │              │
 │   Launch       │                │             │              │
 │ ───────────────>                │             │              │
 │                │  Create repo   │             │              │
 │                │ ──────────────────────────────>             │
 │                │                │             │              │
 │                │  Generate code │             │              │
 │                │ ──────────────>│             │              │
 │                │                │             │              │
 │                │   Code ready   │             │              │
 │                │ <──────────────│             │              │
 │                │                │             │              │
 │                │   Push code    │             │              │
 │                │ ──────────────────────────────>             │
 │                │                │             │              │
 │                │   Deploy       │             │              │
 │                │ ─────────────────────────────────────────────>
 │                │                │             │              │
 │   Complete     │                │             │              │
 │ <───────────────                │             │              │
```

## Module Details

### Core Modules

#### Opportunity Scanner
- Scans GitHub, Reddit, Hacker News, Product Hunt
- Extracts and structures opportunities
- Scores based on engagement and keywords
- Deduplicates across sources

#### Decision Matrix
- Evaluates market demand (25%)
- Assesses competition level (15%)
- Checks technical feasibility (20%)
- Estimates revenue potential (25%)
- Measures strategic fit (15%)

#### Execution Coordinator
- Manages project lifecycle
- Orchestrates integrations
- Handles error recovery
- Tracks project state

#### Resource Allocator
- Prioritizes projects by performance
- Allocates budget proportionally
- Recommends actions (scale, sunset)
- Maintains reserve budget

### Functional Modules

#### Revenue Tracker
- Records daily metrics
- Calculates MRR/ARR/ARPU
- Detects revenue anomalies
- Generates reports

#### Market Predictor
- Analyzes keyword trends
- Identifies market segments
- Predicts growth potential
- Provides insights

#### Code Generator
- Scaffolds projects by template
- Generates TypeScript/Rust code
- Creates CI/CD configurations
- Produces documentation

#### Deployment Mesh
- Supports Vercel, Cloudflare
- Handles multi-platform strategy
- Monitors deployment health
- Enables rollbacks

## Database Schema

```sql
-- Opportunities
CREATE TABLE opportunities (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    source TEXT NOT NULL,
    score REAL,
    status TEXT DEFAULT 'pending',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Projects
CREATE TABLE projects (
    id TEXT PRIMARY KEY,
    opportunity_id TEXT REFERENCES opportunities(id),
    name TEXT NOT NULL,
    status TEXT DEFAULT 'initializing',
    template_type TEXT NOT NULL,
    github_repo TEXT,
    deployment_url TEXT,
    total_revenue_usd REAL DEFAULT 0,
    active_users INTEGER DEFAULT 0,
    created_at TEXT NOT NULL,
    launched_at TEXT
);

-- Deployments
CREATE TABLE deployments (
    id TEXT PRIMARY KEY,
    project_id TEXT REFERENCES projects(id),
    platform TEXT NOT NULL,
    url TEXT,
    status TEXT DEFAULT 'queued',
    deployed_at TEXT
);

-- Revenue Metrics
CREATE TABLE revenue_metrics (
    id TEXT PRIMARY KEY,
    project_id TEXT REFERENCES projects(id),
    metric_date TEXT NOT NULL,
    revenue_usd REAL DEFAULT 0,
    active_users INTEGER DEFAULT 0,
    conversions INTEGER DEFAULT 0
);
```

## Error Handling

Genesis uses a custom error type hierarchy:

```rust
pub enum GenesisError {
    Config(String),
    Database(rusqlite::Error),
    Http(reqwest::Error),
    Api { message: String, status_code: u16 },
    OpportunityNotFound(String),
    ProjectNotFound(String),
    DeploymentFailed(String),
    RateLimited { retry_after: u64 },
    // ...
}
```

All errors are:
- Typed for pattern matching
- Retryable detection
- Contextual messages
- Traceable via spans

## Configuration

Configuration is loaded in order:
1. Default values (hardcoded)
2. `config/default.toml`
3. Environment variables

Environment variables override TOML values.

## Security Considerations

- API keys stored in environment, never logged
- HTTPS for all external requests
- Input validation on all endpoints
- Rate limiting awareness
- No eval or dynamic code execution
