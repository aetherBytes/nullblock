#!/bin/bash

# Simple Hecate Server Startup Script with Comprehensive Logging
# This bypasses tmuxinator issues and provides beautiful console logs

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
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

print_header() {
    echo -e "${PURPLE}[HEADER]${NC} $1"
}

print_subheader() {
    echo -e "${CYAN}[SUBHEADER]${NC} $1"
}

# Print startup banner
print_header "╔══════════════════════════════════════════════════════════════╗"
print_header "║                    🎯 HECATE DEVELOPMENT                    ║"
print_header "║                                                              ║"
print_header "║  🤖 AI-Powered Trading & Analysis Platform                  ║"
print_header "║  🌐 HTTP API Server for Frontend Integration                ║"
print_header "║  📊 Real-time Market Data & Portfolio Management            ║"
print_header "║  🔗 Connected to Nullblock Ecosystem                        ║"
print_header "╚══════════════════════════════════════════════════════════════╝"

print_status "🚀 Starting Hecate Development Environment..."
print_status "📁 Working directory: $(pwd)"

# Check if we're in the right directory
if [ ! -f "package.json" ] || [ ! -d "svc" ]; then
    print_error "❌ Please run this script from the nullblock root directory"
    print_status "📍 Current directory: $(pwd)"
    print_status "📝 Expected to find: package.json and svc/ directory"
    exit 1
fi

print_success "✅ Directory structure verified"

# Create logs directory
print_status "📝 Creating logs directory..."
mkdir -p logs
print_success "✅ Logs directory ready"

# Check system resources
print_subheader "💻 System Information"
print_status "Platform: $(uname -s) $(uname -r)"
print_status "CPU: $(sysctl -n hw.ncpu 2>/dev/null || echo "Unknown") cores"
print_status "Memory: $(vm_stat | grep "Pages free" | awk '{print $3}' | sed 's/\.//' | awk '{print $1/1024/1024}' | cut -d. -f1)GB available"
print_status "Disk: $(df -h . | tail -1 | awk '{print $4}') free"

# Start infrastructure services
print_subheader "🔧 Infrastructure Services"
print_status "Starting PostgreSQL..."
if brew services start postgresql@17 2>/dev/null; then
    print_success "✅ PostgreSQL started"
else
    print_warning "⚠️  PostgreSQL may already be running or failed to start"
fi

print_status "Starting Redis..."
if brew services start redis 2>/dev/null; then
    print_success "✅ Redis started"
else
    print_warning "⚠️  Redis may already be running or failed to start"
fi

print_status "Starting IPFS..."
if pgrep -f "ipfs daemon" > /dev/null; then
    print_success "✅ IPFS daemon already running"
else
    print_status "Starting IPFS daemon in background..."
    ipfs daemon --enable-gc > logs/ipfs.log 2>&1 &
    IPFS_PID=$!
    echo $IPFS_PID > logs/ipfs.pid
    print_success "✅ IPFS daemon started (PID: $IPFS_PID)"
fi

# Start backend services
print_subheader "🔗 Backend Services"

# Start MCP Server
print_status "Starting MCP Server..."
cd svc/nullblock-mcp
if [ -f "pyproject.toml" ]; then
    hatch -e default run pip install -e . > ../../logs/mcp-install.log 2>&1
    export MCP_SERVER_HOST=0.0.0.0
    export MCP_SERVER_PORT=8001
    print_status "🚀 Starting MCP Server on port 8001..."
    hatch run python3 -m mcp > ../../logs/mcp.log 2>&1 &
    MCP_PID=$!
    echo $MCP_PID > ../../logs/mcp.pid
    print_success "✅ MCP Server started (PID: $MCP_PID)"
else
    print_error "❌ MCP Server not found"
fi
cd ../..

# Start Orchestration
print_status "Starting Orchestration..."
cd svc/nullblock-orchestration
if [ -f "pyproject.toml" ]; then
    hatch -e default run pip install -e . > ../../logs/orchestration-install.log 2>&1
    export ORCHESTRATION_HOST=0.0.0.0
    export ORCHESTRATION_PORT=8002
    print_status "🚀 Starting Orchestration on port 8002..."
    hatch run python3 -m orchestration > ../../logs/orchestration.log 2>&1 &
    ORCHESTRATION_PID=$!
    echo $ORCHESTRATION_PID > ../../logs/orchestration.pid
    print_success "✅ Orchestration started (PID: $ORCHESTRATION_PID)"
else
    print_error "❌ Orchestration not found"
fi
cd ../..

# Start Erebus
print_status "Starting Erebus Server..."
cd svc/erebus
if [ -f "Cargo.toml" ]; then
    print_status "Updating Rust dependencies..."
    cargo update > ../../logs/erebus-update.log 2>&1
    export EREBUS_HOST=127.0.0.1
    export EREBUS_PORT=3000
    print_status "🚀 Starting Erebus on port 3000..."
    cargo run > ../../logs/erebus.log 2>&1 &
    EREBUS_PID=$!
    echo $EREBUS_PID > ../../logs/erebus.pid
    print_success "✅ Erebus started (PID: $EREBUS_PID)"
else
    print_error "❌ Erebus not found"
fi
cd ../..

# Start Hecate Agent
print_subheader "🤖 Agent Services"
print_status "Starting General Agents Server..."
cd svc/nullblock-agents
if [ -f "pyproject.toml" ]; then
    hatch -e default run pip install -e . > ../../logs/general-agents-install.log 2>&1
    mkdir -p logs
    export AGENTS_HOST=0.0.0.0
    export AGENTS_PORT=9001
    print_status "🚀 Starting General Agents on port 9001..."
    hatch run python3 -m agents > ../../logs/general-agents.log 2>&1 &
    GENERAL_AGENTS_PID=$!
    echo $GENERAL_AGENTS_PID > ../../logs/general-agents.pid
    print_success "✅ General Agents started (PID: $GENERAL_AGENTS_PID)"
else
    print_error "❌ General Agents not found"
fi
cd ../..

print_status "Starting Hecate Agent Server..."
cd svc/nullblock-agents
if [ -f "pyproject.toml" ]; then
    hatch -e default run pip install -e . > ../../logs/hecate-agent-install.log 2>&1
    mkdir -p logs
    export HECATE_HOST=0.0.0.0
    export HECATE_PORT=9002
    print_status "🚀 Starting Hecate Agent on port 9002..."
    hatch run python3 start_hecate_server.py > ../../logs/hecate-agent.log 2>&1 &
    HECATE_AGENT_PID=$!
    echo $HECATE_AGENT_PID > ../../logs/hecate-agent.pid
    print_success "✅ Hecate Agent started (PID: $HECATE_AGENT_PID)"
else
    print_error "❌ Hecate Agent not found"
fi
cd ../..

# Start Frontend
print_subheader "🌐 Frontend"
print_status "Starting Frontend Development Server..."
cd svc/hecate
if [ -f "package.json" ]; then
    print_status "Installing/updating npm dependencies..."
    npm install > ../../logs/frontend-install.log 2>&1
    
    # Set environment variables
    export VITE_MCP_API_URL=http://localhost:8001
    export VITE_EREBUS_API_URL=http://localhost:3000
    export VITE_ORCHESTRATION_API_URL=http://localhost:8002
    export VITE_AGENTS_API_URL=http://localhost:9001
    export VITE_HECATE_API_URL=http://localhost:9002
    
    print_status "🚀 Starting Vite development server on port 5173..."
    npm run develop > ../../logs/frontend.log 2>&1 &
    FRONTEND_PID=$!
    echo $FRONTEND_PID > ../../logs/frontend.pid
    print_success "✅ Frontend started (PID: $FRONTEND_PID)"
else
    print_error "❌ Frontend not found"
fi
cd ../..

# Wait a moment for services to start
print_status "⏳ Waiting for services to initialize..."
sleep 5

# Show service status
print_subheader "📊 Service Status"
print_status "🔍 Checking service status..."

# Check if services are running
services=(
    "PostgreSQL:postgresql@17"
    "Redis:redis"
    "IPFS:ipfs"
    "MCP Server:8001"
    "Orchestration:8002"
    "Erebus:3000"
    "General Agents:9001"
    "Hecate Agent:9002"
    "Frontend:5173"
)

for service in "${services[@]}"; do
    name="${service%:*}"
    port="${service#*:}"
    
    if [[ $port =~ ^[0-9]+$ ]]; then
        # Check port
        if lsof -i :$port > /dev/null 2>&1; then
            print_success "✅ $name (port $port) - Running"
        else
            print_warning "⚠️  $name (port $port) - Not responding"
        fi
    else
        # Check service
        if brew services list | grep -q "$port.*started"; then
            print_success "✅ $name - Running"
        else
            print_warning "⚠️  $name - Not running"
        fi
    fi
done

# Show log files
print_subheader "📝 Log Files"
print_status "Log files are being written to:"
ls -la logs/ | grep -E "\.(log|pid)$" | while read line; do
    print_status "  📄 $line"
done

# Show access URLs
print_subheader "🌐 Access URLs"
print_success "Frontend: http://localhost:5173"
print_success "Hecate Agent API: http://localhost:9002"
print_success "Hecate Agent Health: http://localhost:9002/health"
print_success "Hecate Agent Docs: http://localhost:9002/docs"
print_success "MCP Server: http://localhost:8001"
print_success "Orchestration: http://localhost:8002"
print_success "General Agents: http://localhost:9001"
print_success "Erebus: http://localhost:3000"

print_header "🎯 Hecate Development Environment Ready!"
print_status "💡 Use 'tail -f logs/*.log' to monitor logs"
print_status "🛑 Use './scripts/stop-hecate-simple.sh' to stop all services"
print_status "=" * 80

# Show master logs option
print_subheader "📊 Master Logs Available"
print_status "💡 For unified log monitoring, run: tail -f logs/*.log"
print_status "💡 For service health monitoring, run: ./scripts/monitor-health.sh"
print_status "=" * 80

# Keep script running and show logs
print_subheader "📊 Live Log Monitoring"
print_status "Press Ctrl+C to stop monitoring (services will continue running)"
print_status "=" * 80

# Monitor logs in real-time
tail -f logs/*.log
