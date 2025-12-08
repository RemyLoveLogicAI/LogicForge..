#!/bin/bash
# Manual deployment helper script

set -e

PROJECT_ID=${1:-}
PLATFORM=${2:-vercel}

if [ -z "$PROJECT_ID" ]; then
    echo "Usage: ./scripts/deploy.sh <project-id> [platform]"
    echo
    echo "Platforms: vercel (default), cloudflare"
    exit 1
fi

echo "🚀 Deploying project: $PROJECT_ID to $PLATFORM"
echo

# Run deployment
genesis deploy --project-id "$PROJECT_ID" --platform "$PLATFORM"

echo
echo "✅ Deployment initiated!"
echo "Run 'genesis monitor --project $PROJECT_ID' to watch progress"
