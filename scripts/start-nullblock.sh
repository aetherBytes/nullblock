#!/bin/bash

# Nullblock MVP Startup Script
# This script launches the complete Nullblock ecosystem

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if Docker is running
check_docker() {
    if ! docker info > /dev/null 2>&1; then
        print_error "Docker is not running. Please start Docker Desktop and try again."
        exit 1
    fi
    print_success "Docker is running"
}

# Check if docker-compose is available
check_docker_compose() {
    if ! command -v docker-compose &> /dev/null; then
        print_error "docker-compose is not installed. Please install it and try again."
        exit 1
    fi
    print_success "docker-compose is available"
}

# Create environment file if it doesn't exist
create_env_file() {
    if [ ! -f .env ]; then
        print_status "Creating .env file with default configuration..."
        cat > .env << EOF
# Nullblock MVP Environment Configuration

# Ethereum RPC URLs
ETHEREUM_RPC_URL=https://eth-mainnet.alchemyapi.io/v2/your-key
POLYGON_RPC_URL=https://polygon-mainnet.alchemyapi.io/v2/your-key
WEB3_RPC_URL=https://eth-mainnet.alchemyapi.io/v2/your-key

# Flashbots Configuration
FLASHBOTS_RPC_URL=https://relay.flashbots.net
FLASHBOTS_PRIVATE_KEY=your-flashbots-private-key
ENABLE_MEV_PROTECTION=true

# Bittensor Configuration
BITTENSOR_NETWORK=test
BITTENSOR_WALLET_PATH=your-bittensor-wallet-path

# Database Configuration
DATABASE_URL=postgresql://nullblock:nullblock_secure_pass@postgres:5432/nullblock
REDIS_URL=redis://redis:6379

# IPFS Configuration
IPFS_API_URL=http://ipfs:5001

# API Keys (comma-separated)
DEX_API_KEYS=your-dex-api-keys

# Solana Configuration
SOLANA_RPC_URL=https://api.devnet.solana.com

# Frontend Configuration
VITE_MCP_API_URL=http://localhost:8000
VITE_ORCHESTRATION_API_URL=http://localhost:8001
VITE_AGENTS_API_URL=http://localhost:8002
EOF
        print_warning "Please edit .env file with your actual API keys and configuration"
    else
        print_success ".env file already exists"
    fi
}

# Build all services
build_services() {
    print_status "Building all Nullblock services..."
    
    # Build MCP service
    print_status "Building Nullblock MCP..."
    docker-compose build nullblock-mcp
    
    # Build orchestration service
    print_status "Building Nullblock Orchestration..."
    docker-compose build nullblock-orchestration
    
    # Build agents service
    print_status "Building Nullblock Agents..."
    docker-compose build nullblock-agents
    
    # Build frontend
    print_status "Building Hecate Frontend..."
    docker-compose build hecate
    
    # Build Erebus contracts
    print_status "Building Erebus Contracts..."
    docker-compose build erebus
    
    print_success "All services built successfully"
}

# Start all services
start_services() {
    print_status "Starting Nullblock MVP..."
    
    # Start infrastructure services first
    print_status "Starting infrastructure services (PostgreSQL, Redis, IPFS)..."
    docker-compose up -d postgres redis ipfs
    
    # Wait for infrastructure to be ready
    print_status "Waiting for infrastructure services to be ready..."
    sleep 30
    
    # Verify databases are created
    print_status "Verifying databases are initialized..."
    verify_databases
    
    # Start core services
    print_status "Starting core services..."
    docker-compose up -d nullblock-mcp nullblock-orchestration nullblock-agents
    
    # Wait for core services
    print_status "Waiting for core services to be ready..."
    sleep 20
    
    # Start frontend and contracts
    print_status "Starting frontend and contracts..."
    docker-compose up -d hecate erebus
    
    print_success "All services started successfully"
}

# Verify databases are created
verify_databases() {
    local max_attempts=10
    local attempt=1
    
    while [ $attempt -le $max_attempts ]; do
        print_status "Checking databases (attempt $attempt/$max_attempts)..."
        
        if docker exec nullblock-postgres psql -U postgres -d postgres -c "SELECT 1 FROM pg_database WHERE datname='nullblock_mcp'" | grep -q 1; then
            print_success "All databases are ready"
            return 0
        else
            print_warning "Databases not ready yet, waiting..."
            sleep 10
            attempt=$((attempt + 1))
        fi
    done
    
    print_error "Database initialization failed after $max_attempts attempts"
    return 1
}

# Check service health
check_health() {
    print_status "Checking service health..."
    
    services=(
        "postgres:5432"
        "redis:6379"
        "ipfs:5001"
        "nullblock-mcp:8000"
        "nullblock-orchestration:8001"
        "nullblock-agents:8002"
        "hecate:5173"
        "erebus:8003"
    )
    
    for service in "${services[@]}"; do
        host_port=(${service//:/ })
        host=${host_port[0]}
        port=${host_port[1]}
        
        if docker-compose exec -T $host curl -f http://localhost:$port/health > /dev/null 2>&1; then
            print_success "$host is healthy"
        else
            print_warning "$host health check failed (this is normal during startup)"
        fi
    done
}

# Show service status
show_status() {
    print_status "Service Status:"
    docker-compose ps
    
    echo ""
    print_status "Service URLs:"
    echo "  Frontend (Hecate): http://localhost:5173"
    echo "  MCP API: http://localhost:8000"
    echo "  Orchestration API: http://localhost:8001"
    echo "  Agents API: http://localhost:8002"
    echo "  IPFS Gateway: http://localhost:8080"
    echo "  PostgreSQL: localhost:5432"
    echo "  Redis: localhost:6379"
}

# Stop all services
stop_services() {
    print_status "Stopping all services..."
    docker-compose down
    print_success "All services stopped"
}

# Clean up everything
cleanup() {
    print_status "Cleaning up all containers and volumes..."
    docker-compose down -v --remove-orphans
    docker system prune -f
    print_success "Cleanup completed"
}

# Show logs
show_logs() {
    if [ -z "$1" ]; then
        print_status "Showing logs for all services..."
        docker-compose logs -f
    else
        print_status "Showing logs for $1..."
        docker-compose logs -f $1
    fi
}

# Main script logic
main() {
    case "${1:-start}" in
        "start")
            check_docker
            check_docker_compose
            create_env_file
            build_services
            start_services
            check_health
            show_status
            ;;
        "stop")
            stop_services
            ;;
        "restart")
            stop_services
            start_services
            check_health
            show_status
            ;;
        "build")
            check_docker
            check_docker_compose
            build_services
            ;;
        "logs")
            show_logs $2
            ;;
        "status")
            show_status
            ;;
        "cleanup")
            cleanup
            ;;
        "health")
            check_health
            ;;
        *)
            echo "Usage: $0 {start|stop|restart|build|logs|status|cleanup|health}"
            echo ""
            echo "Commands:"
            echo "  start     - Start all services (default)"
            echo "  stop      - Stop all services"
            echo "  restart   - Restart all services"
            echo "  build     - Build all services"
            echo "  logs      - Show logs (optionally specify service name)"
            echo "  status    - Show service status and URLs"
            echo "  cleanup   - Clean up all containers and volumes"
            echo "  health    - Check service health"
            exit 1
            ;;
    esac
}

# Run main function with all arguments
main "$@" 