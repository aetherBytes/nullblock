#!/bin/bash

# Hecate Agent Startup with Dependencies
# This script ensures LM Studio is ready before starting Hecate

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

echo "🎯 Hecate Agent Startup with Dependencies"
echo "=========================================="

# Function to wait for LM Studio
wait_for_lm_studio() {
    print_status "Waiting for LM Studio to be ready..."
    
    local max_attempts=60  # 5 minutes max wait
    local attempt=0
    
    while [ $attempt -lt $max_attempts ]; do
        if curl -s http://localhost:1234/v1/models >/dev/null 2>&1; then
            print_success "LM Studio API is responding"
            
            # Check if models are loaded
            local models=$(curl -s http://localhost:1234/v1/models | jq -r '.data[].id' 2>/dev/null || echo "")
            if [ -n "$models" ]; then
                print_success "LM Studio models loaded: $models"
                return 0
            else
                print_warning "LM Studio running but no models detected yet"
            fi
        fi
        
        attempt=$((attempt + 1))
        print_status "Attempt $attempt/$max_attempts - waiting 5 seconds..."
        sleep 5
    done
    
    print_error "LM Studio failed to start within timeout"
    return 1
}

# Function to start Hecate
start_hecate() {
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
    
    print_status "Starting Hecate HTTP API server on port 9002..."
    print_status "Logs will be written to logs/hecate-server.log"
    
    # Start Hecate
    exec hatch run python3 start_hecate_server.py
}

# Main execution
main() {
    # Wait for LM Studio to be ready
    if wait_for_lm_studio; then
        print_success "LM Studio is ready, starting Hecate..."
        start_hecate
    else
        print_error "Failed to start Hecate due to LM Studio dependency"
        exit 1
    fi
}

# Run main function
main "$@"
