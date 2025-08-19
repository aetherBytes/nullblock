#!/bin/bash

# Simple Hecate Server Stop Script
# This stops all development services cleanly

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
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

print_header "🛑 Stopping Hecate Development Environment..."

# Check if we're in the right directory
if [ ! -f "package.json" ] || [ ! -d "svc" ]; then
    print_error "❌ Please run this script from the nullblock root directory"
    exit 1
fi

# Stop services by PID if PID files exist
print_status "🔄 Stopping services by PID..."

services=(
    "frontend"
    "hecate-agent"
    "general-agents"
    "erebus"
    "orchestration"
    "mcp"
    "ipfs"
)

for service in "${services[@]}"; do
    pid_file="logs/${service}.pid"
    if [ -f "$pid_file" ]; then
        pid=$(cat "$pid_file")
        if ps -p "$pid" > /dev/null 2>&1; then
            print_status "🛑 Stopping $service (PID: $pid)..."
            kill "$pid" 2>/dev/null || true
            sleep 1
            if ps -p "$pid" > /dev/null 2>&1; then
                print_warning "⚠️  Force killing $service..."
                kill -9 "$pid" 2>/dev/null || true
            fi
            print_success "✅ $service stopped"
        else
            print_warning "⚠️  $service not running (PID: $pid)"
        fi
        rm -f "$pid_file"
    else
        print_warning "⚠️  No PID file found for $service"
    fi
done

# Stop brew services
print_status "🔄 Stopping infrastructure services..."

print_status "Stopping PostgreSQL..."
brew services stop postgresql@17 2>/dev/null || true
print_success "✅ PostgreSQL stopped"

print_status "Stopping Redis..."
brew services stop redis 2>/dev/null || true
print_success "✅ Redis stopped"

# Kill any remaining processes on our ports
print_status "🔄 Cleaning up port usage..."

ports=(5173 8001 8002 9001 9002 3000)
for port in "${ports[@]}"; do
    pids=$(lsof -ti:$port 2>/dev/null || true)
    if [ -n "$pids" ]; then
        print_status "🛑 Killing processes on port $port..."
        echo "$pids" | xargs kill -9 2>/dev/null || true
        print_success "✅ Port $port cleared"
    fi
done

# Kill any remaining IPFS processes
print_status "🛑 Stopping IPFS daemon..."
pkill -f "ipfs daemon" 2>/dev/null || true
print_success "✅ IPFS daemon stopped"

# Clean up log files
print_status "🧹 Cleaning up log files..."
if [ -d "logs" ]; then
    rm -f logs/*.pid 2>/dev/null || true
    print_success "✅ PID files cleaned up"
fi

print_header "🎯 Hecate Development Environment Stopped!"
print_success "✅ All services have been stopped"
print_status "💡 Log files are preserved in logs/ directory"
print_status "🚀 Run './scripts/start-hecate-simple.sh' to start again"
