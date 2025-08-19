# Nullblock MVP Commands
# Run with: just <command>

# Start the complete Nullblock MVP with Docker
start:
    @echo "🚀 Starting Nullblock MVP..."
    ./scripts/start-nullblock.sh start

# Stop all Nullblock services
stop:
    @echo "🛑 Stopping Nullblock services..."
    ./scripts/start-nullblock.sh stop

# Kill all development services by port
kill-services:
    @echo "💀 Killing all development services..."
    @echo "Stopping PostgreSQL..."
    -brew services stop postgresql@17 2>/dev/null || true
    @echo "Stopping Redis..."
    -brew services stop redis 2>/dev/null || true
    @echo "Killing IPFS daemon..."
    -pkill -f "ipfs daemon" 2>/dev/null || true
    @echo "Killing services on development ports..."
    -lsof -ti:8001 | xargs kill -9 2>/dev/null || true  # MCP Server
    -lsof -ti:8002 | xargs kill -9 2>/dev/null || true  # Orchestration
    -lsof -ti:9001 | xargs kill -9 2>/dev/null || true  # General Agents
    -lsof -ti:9002 | xargs kill -9 2>/dev/null || true  # Hecate Agent
    -lsof -ti:3000 | xargs kill -9 2>/dev/null || true  # Erebus
    -lsof -ti:5173 | xargs kill -9 2>/dev/null || true  # Frontend
    -lsof -ti:1234 | xargs kill -9 2>/dev/null || true  # LM Studio
    @echo "Stopping LM Studio server..."
    -lms server stop 2>/dev/null || true
    @echo "Killing tmuxinator sessions..."
    -tmux kill-session -t nullblock-dev 2>/dev/null || true
    @echo "Cleaning up PID files..."
    -rm -f logs/*.pid 2>/dev/null || true
    @echo "✅ All development services killed"

# Restart all services
restart:
    @echo "🔄 Restarting Nullblock services..."
    ./scripts/start-nullblock.sh restart

# Build all services
build:
    @echo "🔨 Building Nullblock services..."
    ./scripts/start-nullblock.sh build

# Build Erebus Rust server
build-erebus:
    @echo "🦀 Building Erebus Rust server..."
    ./scripts/build-erebus

# Show service logs
logs service="":
    @echo "📋 Showing logs..."
    ./scripts/start-nullblock.sh logs {{service}}

# Check service status
status:
    @echo "📊 Checking service status..."
    ./scripts/start-nullblock.sh status

# Health check all services
health:
    @echo "🏥 Running health checks..."
    ./scripts/start-nullblock.sh health

# Clean up everything
cleanup:
    @echo "🧹 Cleaning up..."
    ./scripts/start-nullblock.sh cleanup

# Setup development environment
dev-setup:
    @echo "⚙️ Setting up development environment..."
    ./scripts/dev-setup.sh setup

# Install dependencies only
dev-install:
    @echo "📦 Installing dependencies..."
    ./scripts/dev-setup.sh install

# Create development environment file
dev-env:
    @echo "🔧 Creating development environment..."
    ./scripts/dev-setup.sh env

# Create development scripts
dev-scripts:
    @echo "📝 Creating development scripts..."
    ./scripts/dev-setup.sh scripts

# Start development services
dev-services:
    @echo "🚀 Starting development services..."
    ./scripts/dev-setup.sh services

# Start Erebus Rust server
start-erebus:
    @echo "🦀 Starting Erebus Rust server..."
    ./scripts/start-erebus

# Test the complete setup
test:
    @echo "🧪 Testing Nullblock setup..."
    ./scripts/test-setup.sh

# Quick start (build + start)
quick:
    @echo "⚡ Quick start..."
    ./scripts/start-nullblock.sh build
    ./scripts/start-nullblock.sh start

# Launch development environment with tmuxinator
dev-tmux:
    @echo "🖥️ Launching development environment with tmuxinator..."
    ./scripts/dev-tmux

# Install tmuxinator and setup
dev-tmux-install:
    @echo "📦 Installing tmuxinator and setup..."
    ./scripts/dev-tmux install

# Setup tmuxinator configuration
dev-tmux-setup:
    @echo "⚙️ Setting up tmuxinator configuration..."
    ./scripts/dev-tmux setup

# Initialize databases
init-db:
    @echo "🗄️ Initializing PostgreSQL databases..."
    ./scripts/init-databases.sh

# Show Docker database connections
docker-db-info:
    @echo "🗄️ Docker Database Connection Info..."
    cat scripts/docker-database-connections.txt

# Show help
help:
    @echo "Nullblock MVP Commands:"
    @echo ""
    @echo "Docker Commands:"
    @echo "  just start     - Start all services"
    @echo "  just stop      - Stop all services"
    @echo "  just kill-services - Kill all development services by port"
    @echo "  just restart   - Restart all services"
    @echo "  just build     - Build all services"
    @echo "  just logs      - Show all logs"
    @echo "  just logs <service> - Show specific service logs"
    @echo "  just status    - Check service status"
    @echo "  just health    - Run health checks"
    @echo "  just cleanup   - Clean up everything"
    @echo "  just quick     - Quick start (build + start)"
    @echo ""
    @echo "Development Commands:"
    @echo "  just dev-setup    - Complete development setup"
    @echo "  just dev-install  - Install dependencies only"
    @echo "  just dev-env      - Create environment file"
    @echo "  just dev-scripts  - Create development scripts"
    @echo "  just dev-services - Start development services"
    @echo "  just start-erebus - Start Erebus Rust server"
    @echo "  just build-erebus - Build Erebus Rust server"
    @echo "  just init-db      - Initialize PostgreSQL databases (local)"
    @echo "  just docker-db-info - Show Docker database connections"
    @echo ""
    @echo "Testing Commands:"
    @echo "  just test      - Test complete setup"
    @echo ""
    @echo "Development Environment:"
    @echo "  just dev-tmux        - Launch complete dev environment with tmuxinator"
    @echo "  just dev-tmux-install - Install tmuxinator and setup"
    @echo "  just dev-tmux-setup   - Setup tmuxinator configuration"
    @echo ""
    @echo "Service URLs:"
    @echo "  Frontend: http://localhost:5173"
    @echo "  MCP API: http://localhost:8001"
    @echo "  Orchestration API: http://localhost:8002"
    @echo "  Agents API: http://localhost:8003"
    @echo "  Erebus API: http://localhost:3000"
    @echo ""
    @echo "  IPFS Gateway: http://localhost:8080"
