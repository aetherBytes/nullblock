#!/bin/bash

# Hecate Agent Restart Script
# This script restarts the Hecate agent to detect newly available LM Studio models

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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

echo "🔄 Hecate Agent Restart Script"
echo "=============================="

# Check if LM Studio is running
print_status "Checking LM Studio status..."
if curl -s http://localhost:1234/v1/models > /dev/null 2>&1; then
    print_success "LM Studio is running and accessible"
    models=$(curl -s http://localhost:1234/v1/models | jq -r '.data[].id' 2>/dev/null || echo "Unknown")
    print_status "Available models: $models"
else
    print_warning "LM Studio is not accessible on port 1234"
    print_status "Make sure LM Studio is running before restarting Hecate"
fi

# Kill existing Hecate processes
print_status "Stopping existing Hecate agent processes..."
pkill -f "start_hecate_server.py" 2>/dev/null || true
pkill -f "hecate.*server" 2>/dev/null || true

# Wait for processes to fully stop
sleep 2

# Check if Hecate is still running
if pgrep -f "start_hecate_server.py" > /dev/null; then
    print_warning "Hecate process still running, forcing termination..."
    pkill -9 -f "start_hecate_server.py" 2>/dev/null || true
    sleep 1
fi

# Verify Hecate is stopped
if pgrep -f "start_hecate_server.py" > /dev/null; then
    print_error "Failed to stop Hecate agent"
    exit 1
else
    print_success "Hecate agent stopped successfully"
fi

# Start Hecate agent
print_status "Starting Hecate agent..."
cd svc/nullblock-agents

# Ensure dependencies are installed
print_status "Ensuring dependencies are installed..."
hatch -e default run pip install -e . > /dev/null 2>&1

# Create logs directory
mkdir -p logs

# Set environment variables
export HECATE_HOST=0.0.0.0
export HECATE_PORT=9002

# Start Hecate in background
print_status "Starting Hecate HTTP API server on port 9002..."
nohup hatch run python3 start_hecate_server.py > logs/hecate-server.log 2>&1 &
HECATE_PID=$!

# Wait for Hecate to start
print_status "Waiting for Hecate to start..."
sleep 5

# Check if Hecate started successfully
if curl -s http://localhost:9002/health > /dev/null 2>&1; then
    print_success "Hecate agent started successfully"
    
    # Check model status
    print_status "Checking model availability..."
    sleep 2
    
    model_status=$(curl -s http://localhost:9002/model-status | jq -r '.health.models_available' 2>/dev/null || echo "unknown")
    overall_status=$(curl -s http://localhost:9002/model-status | jq -r '.health.overall_status' 2>/dev/null || echo "unknown")
    
    print_status "Models available: $model_status"
    print_status "Overall status: $overall_status"
    
    if [ "$model_status" = "0" ] && [ "$overall_status" = "unhealthy" ]; then
        print_warning "Hecate started but no models detected"
        print_status "This might be normal if LM Studio was started after Hecate"
        print_status "Try testing a chat request to see if it works:"
        echo ""
        echo "curl -X POST \"http://localhost:9002/chat\" \\"
        echo "  -H \"Content-Type: application/json\" \\"
        echo "  -d '{\"message\": \"Hello!\", \"user_context\": {\"wallet_address\": \"0x1234567890abcdef\"}}'"
    else
        print_success "Hecate is ready with models available!"
    fi
    
else
    print_error "Failed to start Hecate agent"
    print_status "Check logs at svc/nullblock-agents/logs/hecate-server.log"
    exit 1
fi

echo ""
print_success "Hecate agent restart completed!"
print_status "PID: $HECATE_PID"
print_status "Health check: http://localhost:9002/health"
print_status "API docs: http://localhost:9002/docs"
print_status "Logs: tail -f svc/nullblock-agents/logs/hecate-server.log"
