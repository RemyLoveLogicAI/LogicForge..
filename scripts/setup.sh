#!/bin/bash
# Genesis Engine Setup Script

set -e

echo "🧬 Genesis Engine Setup"
echo "========================"
echo

# Check for Rust
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust not found. Installing..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

echo "✅ Rust $(rustc --version | cut -d' ' -f2) found"

# Build the project
echo
echo "📦 Building Genesis Engine..."
cargo build --release

# Create directories
echo
echo "📁 Creating directories..."
mkdir -p config data templates/{micro-saas,api-service,content-site}

# Copy configuration
if [ ! -f config/default.toml ]; then
    cp config/example.toml config/default.toml
    echo "✅ Created config/default.toml"
fi

# Create .env file
if [ ! -f .env ]; then
    cat > .env << 'EOF'
# Genesis Engine Environment Variables
# Fill in your API keys below

GENSPARK_API_KEY=
GENSPARK_WORKSPACE_ID=
GITHUB_TOKEN=
VERCEL_TOKEN=
CLOUDFLARE_TOKEN=
CLOUDFLARE_ACCOUNT_ID=
EOF
    echo "✅ Created .env file"
fi

# Initialize database
echo
echo "💾 Initializing database..."
./target/release/genesis init --name "genesis-empire" --non-interactive

echo
echo "═══════════════════════════════════════════════════════════"
echo "✅ Genesis Engine setup complete!"
echo
echo "Next steps:"
echo "  1. Edit .env and add your API keys"
echo "  2. Run 'genesis scan' to discover opportunities"
echo "  3. Run 'genesis --help' for all commands"
echo
echo "Happy building! 🚀"
