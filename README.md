# Genesis Engine 🧬

> Autonomous AI Empire Orchestrator - Build, Deploy, and Monetize AI Products While You Sleep

[![CI](https://github.com/genesis-engine/genesis-engine/workflows/CI/badge.svg)](https://github.com/genesis-engine/genesis-engine/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-blue.svg)](https://www.rust-lang.org)

## What It Does

Genesis Engine is a self-operating business platform that:

- **🔍 Discovers** profitable opportunities by scanning markets 24/7
- **🤖 Generates** complete products using GenSpark AI Developer
- **🚀 Deploys** automatically to production infrastructure
- **📊 Monitors** revenue and performance across your entire portfolio
- **⚡ Optimizes** resource allocation based on real-time data

## Quick Start

### Installation

```bash
# From source
cargo install --path .

# Or download from releases
curl -L https://github.com/genesis-engine/genesis-engine/releases/latest/download/genesis-linux-amd64 -o genesis
chmod +x genesis
```

### Initialize Your Empire

```bash
# Initialize Genesis in your current directory
genesis init --name "my-empire"

# Configure your API keys
cp .env.example .env
# Edit .env with your keys
```

### Basic Workflow

```bash
# 1. Scan for opportunities
genesis scan --sources github,reddit --limit 10

# 2. Review and launch a project
genesis launch --opportunity-id <id> --auto-deploy

# 3. Monitor your empire
genesis monitor --watch

# 4. View analytics
genesis analytics --days 30
```

## Commands

| Command | Description |
|---------|-------------|
| `genesis init` | Initialize Genesis in current directory |
| `genesis scan` | Scan for new opportunities |
| `genesis launch` | Launch a project from an opportunity |
| `genesis monitor` | Monitor all active projects |
| `genesis deploy` | Deploy a project to a platform |
| `genesis analytics` | Show empire analytics |
| `genesis config` | Manage configuration |
| `genesis list` | List resources |
| `genesis show` | Show resource details |
| `genesis ai` | AI-powered operations |
| `genesis health` | System health check |

## Configuration

Genesis uses TOML configuration files. The main config is at `config/default.toml`:

```toml
[empire]
name = "genesis-empire"
auto_mode = false
max_concurrent_projects = 5

[scanning]
enabled_sources = ["github", "reddit", "producthunt", "hackernews"]
scan_interval_hours = 6
min_opportunity_score = 75.0

[ai]
primary_provider = "genspark"

[deployment]
default_platform = "vercel"
auto_deploy = false
```

### Environment Variables

| Variable | Description |
|----------|-------------|
| `GENSPARK_API_KEY` | GenSpark API key |
| `GENSPARK_WORKSPACE_ID` | GenSpark workspace ID |
| `GITHUB_TOKEN` | GitHub personal access token |
| `VERCEL_TOKEN` | Vercel API token |
| `CLOUDFLARE_TOKEN` | Cloudflare API token |
| `CLOUDFLARE_ACCOUNT_ID` | Cloudflare account ID |

## Architecture

```
genesis-engine/
├── src/
│   ├── cli/              # Command-line interface
│   ├── core/             # Core business logic
│   │   ├── opportunity_scanner    # Market opportunity discovery
│   │   ├── decision_matrix        # AI-powered evaluation
│   │   ├── resource_allocator     # Resource management
│   │   └── execution_coordinator  # Project lifecycle
│   ├── modules/          # Functional modules
│   │   ├── revenue_tracker        # Financial monitoring
│   │   ├── market_predictor       # Trend analysis
│   │   ├── code_generator         # Project scaffolding
│   │   └── deployment_mesh        # Multi-platform deployment
│   ├── integrations/     # External services
│   │   ├── genspark      # GenSpark AI
│   │   ├── github        # GitHub API
│   │   ├── vercel        # Vercel deployment
│   │   └── cloudflare    # Cloudflare deployment
│   ├── models/           # Data models
│   ├── db/               # Database operations
│   └── utils/            # Utilities
├── config/               # Configuration files
├── templates/            # Project templates
├── tests/                # Test suite
└── docs/                 # Documentation
```

## Development

### Prerequisites

- Rust 1.75+
- SQLite (bundled)

### Build from Source

```bash
# Clone the repository
git clone https://github.com/genesis-engine/genesis-engine.git
cd genesis-engine

# Build
cargo build --release

# Run tests
cargo test

# Run clippy
cargo clippy

# Format code
cargo fmt
```

### Running Locally

```bash
# Development build
cargo run -- init

# With verbose logging
cargo run -- -v scan --sources github

# Run specific tests
cargo test core::opportunity_scanner
```

## How It Works

### 1. Opportunity Discovery

Genesis scans multiple sources for business opportunities:

- **GitHub**: Trending repos, issues with "help wanted" labels
- **Reddit**: Posts from r/SaaS, r/startups, r/IndieHackers
- **Hacker News**: Show HN posts, Ask HN threads
- **Product Hunt**: Launched products and comments

Each opportunity is scored based on:
- Market demand signals (upvotes, comments)
- Competition analysis
- Technical feasibility
- Revenue potential

### 2. Decision Matrix

The AI-powered decision matrix evaluates opportunities:

```
Score = (Market × 0.25) + (Competition × 0.15) + (Technical × 0.20) + (Revenue × 0.25) + (Strategic × 0.15)
```

Decisions: **Launch**, **Defer**, or **Reject**

### 3. Project Launch

When an opportunity is approved:

1. Create GitHub repository
2. Generate code via GenSpark AI
3. Review generated code
4. Deploy to platform (Vercel/Cloudflare)
5. Set up monitoring

### 4. Autonomous Mode

When `auto_mode = true`, Genesis will:

- Scan at configured intervals
- Auto-launch projects scoring above threshold
- Auto-deploy to configured platform
- Track revenue and reallocate resources
- Sunset underperforming projects

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](docs/CONTRIBUTING.md) for guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

- Built with [Rust](https://www.rust-lang.org/)
- AI powered by [GenSpark](https://www.genspark.ai/)
- CLI framework by [clap](https://clap.rs/)
