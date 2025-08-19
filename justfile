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

# Comprehensive health check with curl
health-check:
    @mkdir -p logs
    @echo "$(date '+%Y-%m-%d %H:%M:%S') [HEALTH-CHECK] 🏥 Starting comprehensive health check" | tee -a logs/just-commands.log
    @echo "🏥 Comprehensive Health Check - All Services"
    @echo "=============================================="
    @echo ""
    @echo "🔍 MCP Server (Port 8001):"
    @curl -s --max-time 5 http://localhost:8001/health | jq '.' 2>/dev/null | tee -a logs/just-commands.log || (echo "❌ MCP Server not responding" | tee -a logs/just-commands.log)
    @echo ""
    @echo "🔍 Orchestration (Port 8002):"
    @curl -s --max-time 5 http://localhost:8002/health | jq '.' 2>/dev/null | tee -a logs/just-commands.log || (echo "❌ Orchestration not responding" | tee -a logs/just-commands.log)
    @echo ""
    @echo "🔍 Erebus Server (Port 3000):"
    @curl -s --max-time 5 http://localhost:3000/health | jq '.' 2>/dev/null | tee -a logs/just-commands.log || (echo "❌ Erebus Server not responding" | tee -a logs/just-commands.log)
    @echo ""
    @echo "🔍 General Agents (Port 9001):"
    @curl -s --max-time 5 http://localhost:9001/health | jq '.' 2>/dev/null | tee -a logs/just-commands.log || (echo "❌ General Agents not responding" | tee -a logs/just-commands.log)
    @echo ""
    @echo "🔍 Hecate Agent (Port 9002):"
    @curl -s --max-time 5 http://localhost:9002/health | jq '.' 2>/dev/null | tee -a logs/just-commands.log || (echo "❌ Hecate Agent not responding" | tee -a logs/just-commands.log)
    @echo ""
    @echo "🔍 Frontend (Port 5173):"
    @curl -s --max-time 5 http://localhost:5173 >/dev/null && (echo "✅ Frontend responding" | tee -a logs/just-commands.log) || (echo "❌ Frontend not responding" | tee -a logs/just-commands.log)
    @echo ""
    @echo "🔍 LM Studio (Port 1234):"
    @curl -s --max-time 5 http://localhost:1234/v1/models >/dev/null && (echo "✅ LM Studio responding" | tee -a logs/just-commands.log) || (echo "❌ LM Studio not responding" | tee -a logs/just-commands.log)
    @echo "$(date '+%Y-%m-%d %H:%M:%S') [HEALTH-CHECK] ✅ Health check completed" | tee -a logs/just-commands.log
    @echo ""

# Mock API tests for base endpoints
test-api:
    @mkdir -p logs
    @echo "$(date '+%Y-%m-%d %H:%M:%S') [API-TEST] 🧪 Starting API endpoint tests" | tee -a logs/just-commands.log
    @echo "🧪 API Endpoint Tests - Mock Data"
    @echo "=================================="
    @echo ""
    @echo "🔹 Testing MCP Server endpoints:"
    @curl -s --max-time 5 http://localhost:8001/mcp/data-sources | jq '.' 2>/dev/null | tee -a logs/just-commands.log || (echo "❌ MCP data sources not available" | tee -a logs/just-commands.log)
    @echo ""
    @echo "🔹 Testing Orchestration endpoints:"
    @curl -s --max-time 5 http://localhost:8002/orchestration/status | jq '.' 2>/dev/null | tee -a logs/just-commands.log || (echo "❌ Orchestration status not available" | tee -a logs/just-commands.log)
    @echo ""
    @echo "🔹 Testing Erebus wallet endpoints:"
    @curl -s --max-time 5 http://localhost:3000/api/wallets | jq '.' 2>/dev/null | tee -a logs/just-commands.log || (echo "❌ Erebus wallets not available" | tee -a logs/just-commands.log)
    @echo ""
    @echo "🔹 Testing Hecate Agent endpoints:"
    @curl -s --max-time 5 http://localhost:9002/model-status | jq '.' 2>/dev/null | tee -a logs/just-commands.log || (echo "❌ Hecate model status not available" | tee -a logs/just-commands.log)
    @echo ""
    @echo "🔹 Testing LLM Service endpoints:"
    @curl -s --max-time 5 http://localhost:1234/v1/models | jq '.' 2>/dev/null | tee -a logs/just-commands.log || (echo "❌ LM Studio models not available" | tee -a logs/just-commands.log)
    @echo "$(date '+%Y-%m-%d %H:%M:%S') [API-TEST] ✅ API tests completed" | tee -a logs/just-commands.log
    @echo ""

# Quick connectivity test
ping-services:
    @mkdir -p logs
    @echo "$(date '+%Y-%m-%d %H:%M:%S') [PING-TEST] 🏓 Starting connectivity ping" | tee -a logs/just-commands.log
    @echo "🏓 Quick Connectivity Test"
    @echo "=========================="
    @echo -n "MCP (8001): "; curl -s --max-time 2 http://localhost:8001/health >/dev/null && (echo "✅ UP" | tee -a logs/just-commands.log) || (echo "❌ DOWN" | tee -a logs/just-commands.log)
    @echo -n "Orchestration (8002): "; curl -s --max-time 2 http://localhost:8002/health >/dev/null && (echo "✅ UP" | tee -a logs/just-commands.log) || (echo "❌ DOWN" | tee -a logs/just-commands.log)
    @echo -n "Erebus (3000): "; curl -s --max-time 2 http://localhost:3000/health >/dev/null && (echo "✅ UP" | tee -a logs/just-commands.log) || (echo "❌ DOWN" | tee -a logs/just-commands.log)
    @echo -n "General Agents (9001): "; curl -s --max-time 2 http://localhost:9001/health >/dev/null && (echo "✅ UP" | tee -a logs/just-commands.log) || (echo "❌ DOWN" | tee -a logs/just-commands.log)
    @echo -n "Hecate Agent (9002): "; curl -s --max-time 2 http://localhost:9002/health >/dev/null && (echo "✅ UP" | tee -a logs/just-commands.log) || (echo "❌ DOWN" | tee -a logs/just-commands.log)
    @echo -n "Frontend (5173): "; curl -s --max-time 2 http://localhost:5173 >/dev/null && (echo "✅ UP" | tee -a logs/just-commands.log) || (echo "❌ DOWN" | tee -a logs/just-commands.log)
    @echo -n "LM Studio (1234): "; curl -s --max-time 2 http://localhost:1234/v1/models >/dev/null && (echo "✅ UP" | tee -a logs/just-commands.log) || (echo "❌ DOWN" | tee -a logs/just-commands.log)
    @echo "$(date '+%Y-%m-%d %H:%M:%S') [PING-TEST] ✅ Ping test completed" | tee -a logs/just-commands.log

# Test Hecate Agent chat endpoint with mock message
test-chat:
    @mkdir -p logs
    @echo "$(date '+%Y-%m-%d %H:%M:%S') [CHAT-TEST] 💬 Starting Hecate chat test" | tee -a logs/just-commands.log
    @echo "💬 Testing Hecate Agent Chat"
    @echo "============================="
    @echo "Sending test message..."
    @echo "$(date '+%Y-%m-%d %H:%M:%S') [CHAT-TEST] 📤 REQUEST:" | tee -a logs/just-commands.log
    @echo '{"message":"Hello, this is a test message from the health check system."}' | jq '.' | tee -a logs/just-commands.log
    @echo "$(date '+%Y-%m-%d %H:%M:%S') [CHAT-TEST] 📥 RESPONSE:" | tee -a logs/just-commands.log
    @curl -s --max-time 10 -X POST http://localhost:9002/chat \
        -H "Content-Type: application/json" \
        -d '{"message":"Hello, this is a test message from the health check system."}' | \
        jq '.' 2>/dev/null | tee -a logs/just-commands.log || (echo "❌ Chat endpoint not responding" | tee -a logs/just-commands.log)
    @echo "$(date '+%Y-%m-%d %H:%M:%S') [CHAT-TEST] ✅ Chat test completed" | tee -a logs/just-commands.log
    @echo ""

# Test MCP data sources with mock query
test-mcp-data:
    @mkdir -p logs
    @echo "$(date '+%Y-%m-%d %H:%M:%S') [MCP-DATA-TEST] 📊 Starting MCP data test" | tee -a logs/just-commands.log
    @echo "📊 Testing MCP Data Sources"
    @echo "==========================="
    @echo "Available data sources:"
    @curl -s --max-time 5 http://localhost:8001/mcp/data-sources | jq '.' 2>/dev/null | tee -a logs/just-commands.log || (echo "❌ No data sources available" | tee -a logs/just-commands.log)
    @echo ""
    @echo "Testing price oracle data:"
    @curl -s --max-time 5 "http://localhost:8001/mcp/data/price_oracle/coingecko?symbols=bitcoin" | jq '.' 2>/dev/null | tee -a logs/just-commands.log || (echo "❌ Price data not available" | tee -a logs/just-commands.log)
    @echo "$(date '+%Y-%m-%d %H:%M:%S') [MCP-DATA-TEST] ✅ MCP data test completed" | tee -a logs/just-commands.log
    @echo ""

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
    @echo "Running comprehensive test suite..."
    @echo ""
    just health-check
    @echo ""
    just ping-services
    @echo ""
    just test-api
    @echo ""
    just test-chat
    @echo ""
    just test-mcp-data
    @echo ""
    @echo "✅ Complete test suite finished"

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
    @echo "  just test         - Test complete setup"
    @echo "  just health-check - Comprehensive health check with curl"
    @echo "  just ping-services - Quick connectivity test"
    @echo "  just test-api     - Test API endpoints with mock data"
    @echo "  just test-chat    - Test Hecate Agent chat endpoint"
    @echo "  just test-mcp-data - Test MCP data sources"
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
