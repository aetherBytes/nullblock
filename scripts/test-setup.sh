#!/bin/bash

# Nullblock MVP Test Script
# This script tests the complete Nullblock ecosystem

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

# Test service health
test_service_health() {
    local service_name=$1
    local url=$2
    local description=$3
    
    print_status "Testing $description..."
    
    if curl -f -s "$url/health" > /dev/null 2>&1; then
        print_success "$description is healthy"
        return 0
    else
        print_error "$description health check failed"
        return 1
    fi
}

# Test API endpoints
test_api_endpoints() {
    print_status "Testing API endpoints..."
    
    # Test MCP API
    if curl -f -s "http://localhost:8000/" > /dev/null 2>&1; then
        print_success "MCP API is responding"
    else
        print_error "MCP API is not responding"
    fi
    
    # Test Orchestration API
    if curl -f -s "http://localhost:8001/" > /dev/null 2>&1; then
        print_success "Orchestration API is responding"
    else
        print_error "Orchestration API is not responding"
    fi
    
    # Test Agents API
    if curl -f -s "http://localhost:8002/" > /dev/null 2>&1; then
        print_success "Agents API is responding"
    else
        print_error "Agents API is not responding"
    fi
    
    # Test Erebus API
    if curl -f -s "http://localhost:8003/" > /dev/null 2>&1; then
        print_success "Erebus API is responding"
    else
        print_warning "Erebus API is not responding (this is expected if not implemented)"
    fi
    
    # Test Frontend
    if curl -f -s "http://localhost:5173/" > /dev/null 2>&1; then
        print_success "Frontend is responding"
    else
        print_error "Frontend is not responding"
    fi
}

# Test arbitrage functionality
test_arbitrage() {
    print_status "Testing arbitrage functionality..."
    
    # Test getting arbitrage opportunities
    if curl -f -s "http://localhost:8002/arbitrage/opportunities" > /dev/null 2>&1; then
        print_success "Arbitrage opportunities endpoint is working"
    else
        print_error "Arbitrage opportunities endpoint failed"
    fi
    
    # Test market summary
    if curl -f -s "http://localhost:8002/arbitrage/summary" > /dev/null 2>&1; then
        print_success "Market summary endpoint is working"
    else
        print_error "Market summary endpoint failed"
    fi
}

# Test workflow functionality
test_workflows() {
    print_status "Testing workflow functionality..."
    
    # Test workflow creation (simulated)
    workflow_response=$(curl -s -X POST "http://localhost:8001/workflows" \
        -H "Content-Type: application/json" \
        -d '{
            "name": "Test Arbitrage",
            "description": "Test arbitrage workflow",
            "goal_description": "Find arbitrage opportunities",
            "target_metric": "profit",
            "target_value": 1.0,
            "user_id": "test-user"
        }' 2>/dev/null || echo "{}")
    
    if echo "$workflow_response" | grep -q "workflow_id"; then
        print_success "Workflow creation is working"
    else
        print_warning "Workflow creation test failed (this is expected in MVP)"
    fi
}

# Test Erebus functionality
test_erebus() {
    print_status "Testing Erebus Rust server..."
    
    # Test Erebus health endpoint
    if curl -f -s "http://localhost:8003/health" > /dev/null 2>&1; then
        print_success "Erebus health endpoint is working"
    else
        print_warning "Erebus health endpoint failed (this is expected if not implemented)"
    fi
    
    # Test Erebus root endpoint
    if curl -f -s "http://localhost:8003/" > /dev/null 2>&1; then
        print_success "Erebus root endpoint is working"
    else
        print_warning "Erebus root endpoint failed (this is expected if not implemented)"
    fi
}

# Test Docker containers
test_docker_containers() {
    print_status "Testing Docker containers..."
    
    containers=(
        "nullblock-postgres"
        "nullblock-redis"
        "nullblock-ipfs"
        "nullblock-mcp"
        "nullblock-orchestration"
        "nullblock-agents"
        "nullblock-hecate"
        "nullblock-erebus"
    )
    
    for container in "${containers[@]}"; do
        if docker ps --format "table {{.Names}}" | grep -q "$container"; then
            print_success "Container $container is running"
        else
            print_error "Container $container is not running"
        fi
    done
}

# Test database connectivity
test_database() {
    print_status "Testing database connectivity..."
    
    if docker exec nullblock-postgres pg_isready -U postgres > /dev/null 2>&1; then
        print_success "PostgreSQL is ready"
    else
        print_error "PostgreSQL is not ready"
    fi
    
    # Test individual databases
    databases=("nullblock_mcp" "nullblock_orchestration" "nullblock_agents" "nullblock_erebus" "nullblock_hecate" "nullblock_platform")
    
    for db in "${databases[@]}"; do
        if docker exec nullblock-postgres psql -U postgres -d postgres -c "SELECT 1 FROM pg_database WHERE datname='$db'" | grep -q 1; then
            print_success "Database $db exists"
        else
            print_error "Database $db not found"
        fi
    done
}

# Test Redis connectivity
test_redis() {
    print_status "Testing Redis connectivity..."
    
    if docker exec nullblock-redis redis-cli ping > /dev/null 2>&1; then
        print_success "Redis is responding"
    else
        print_error "Redis is not responding"
    fi
}

# Test IPFS connectivity
test_ipfs() {
    print_status "Testing IPFS connectivity..."
    
    if curl -f -s "http://localhost:5001/api/v0/version" > /dev/null 2>&1; then
        print_success "IPFS API is responding"
    else
        print_error "IPFS API is not responding"
    fi
}

# Main test function
main() {
    print_status "Starting Nullblock MVP tests..."
    
    # Wait for services to be ready
    print_status "Waiting for services to be ready..."
    sleep 10
    
    # Test infrastructure
    test_docker_containers
    test_database
    test_redis
    test_ipfs
    
    # Test service health
    test_service_health "mcp" "http://localhost:8000" "MCP Server"
    test_service_health "orchestration" "http://localhost:8001" "Orchestration Engine"
    test_service_health "agents" "http://localhost:8002" "Agents Service"
    test_service_health "erebus" "http://localhost:8003" "Erebus Rust Server"
    
    # Test API endpoints
    test_api_endpoints
    
    # Test functionality
    test_arbitrage
    test_workflows
    test_erebus
    
    print_success "All tests completed!"
    
    echo ""
    print_status "Service URLs:"
    echo "  Frontend: http://localhost:5173"
    echo "  MCP API: http://localhost:8000"
    echo "  Orchestration API: http://localhost:8001"
    echo "  Agents API: http://localhost:8002"
    echo "  Erebus API: http://localhost:8003"
    echo "  IPFS Gateway: http://localhost:8080"
    echo ""
    print_status "You can now access the Nullblock MVP!"
}

# Run main function
main "$@"
