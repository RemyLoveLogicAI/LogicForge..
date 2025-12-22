#!/bin/bash

# Genesis Engine - Production Deployment Script
# This script deploys Genesis Engine to a production VPS

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
DEPLOY_DIR="${DEPLOY_DIR:-/opt/genesis-engine}"
COMPOSE_PROFILES="${COMPOSE_PROFILES:-}"
BACKUP_ENABLED="${BACKUP_ENABLED:-true}"

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check Docker
    if ! command -v docker &> /dev/null; then
        log_error "Docker is not installed. Please install Docker first."
    fi
    
    # Check Docker Compose
    if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
        log_error "Docker Compose is not installed. Please install Docker Compose first."
    fi
    
    # Check .env file
    if [ ! -f ".env" ]; then
        if [ -f ".env.example" ]; then
            log_warning ".env file not found. Creating from .env.example..."
            cp .env.example .env
            log_warning "Please edit .env file with your API keys before continuing."
            exit 1
        else
            log_error ".env file not found and no .env.example available."
        fi
    fi
    
    log_success "Prerequisites check passed."
}

# Backup existing data
backup_data() {
    if [ "$BACKUP_ENABLED" != "true" ]; then
        return
    fi
    
    log_info "Backing up existing data..."
    
    BACKUP_DIR="${DEPLOY_DIR}/backups"
    TIMESTAMP=$(date +%Y%m%d_%H%M%S)
    BACKUP_FILE="${BACKUP_DIR}/genesis_backup_${TIMESTAMP}.tar.gz"
    
    mkdir -p "$BACKUP_DIR"
    
    # Backup database and config
    if [ -d "${DEPLOY_DIR}/data" ]; then
        tar -czf "$BACKUP_FILE" -C "${DEPLOY_DIR}" data config 2>/dev/null || true
        log_success "Backup created: $BACKUP_FILE"
    fi
    
    # Keep only last 7 backups
    ls -t "${BACKUP_DIR}"/genesis_backup_*.tar.gz 2>/dev/null | tail -n +8 | xargs rm -f 2>/dev/null || true
}

# Pull latest changes
pull_latest() {
    log_info "Pulling latest changes..."
    
    if [ -d ".git" ]; then
        git fetch origin
        git pull origin main
        log_success "Latest changes pulled."
    else
        log_warning "Not a git repository. Skipping pull."
    fi
}

# Build and deploy
deploy() {
    log_info "Building and deploying Genesis Engine..."
    
    # Build Docker images
    log_info "Building Docker images..."
    docker-compose build --no-cache
    
    # Stop existing containers
    log_info "Stopping existing containers..."
    docker-compose down --remove-orphans || true
    
    # Start new containers
    log_info "Starting containers..."
    if [ -n "$COMPOSE_PROFILES" ]; then
        docker-compose --profile "$COMPOSE_PROFILES" up -d
    else
        docker-compose up -d
    fi
    
    # Wait for health check
    log_info "Waiting for health check..."
    sleep 10
    
    # Check container status
    if docker-compose ps | grep -q "Up"; then
        log_success "Genesis Engine is running!"
    else
        log_error "Failed to start Genesis Engine. Check logs with: docker-compose logs"
    fi
}

# Show status
show_status() {
    echo ""
    log_info "Container Status:"
    docker-compose ps
    
    echo ""
    log_info "Recent Logs:"
    docker-compose logs --tail=20 genesis
    
    echo ""
    log_success "Deployment complete!"
    echo ""
    echo "Useful commands:"
    echo "  View logs:         docker-compose logs -f genesis"
    echo "  Run CLI commands:  docker-compose exec genesis genesis <command>"
    echo "  Stop:              docker-compose down"
    echo "  Restart:           docker-compose restart genesis"
    echo ""
}

# Main deployment flow
main() {
    echo ""
    echo "============================================"
    echo "  Genesis Engine - Production Deployment"
    echo "============================================"
    echo ""
    
    check_prerequisites
    backup_data
    pull_latest
    deploy
    show_status
}

# Parse arguments
case "${1:-}" in
    --help|-h)
        echo "Usage: $0 [OPTIONS]"
        echo ""
        echo "Options:"
        echo "  --help, -h      Show this help message"
        echo "  --no-backup     Skip backup step"
        echo "  --with-cache    Enable Redis caching"
        echo "  --with-monitoring  Enable Prometheus/Grafana"
        echo ""
        exit 0
        ;;
    --no-backup)
        BACKUP_ENABLED=false
        main
        ;;
    --with-cache)
        COMPOSE_PROFILES="with-cache"
        main
        ;;
    --with-monitoring)
        COMPOSE_PROFILES="with-monitoring"
        main
        ;;
    *)
        main
        ;;
esac
