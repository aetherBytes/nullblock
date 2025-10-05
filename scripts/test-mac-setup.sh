#!/bin/bash

# Mac Development Environment Setup Validation Script
# This script checks for all required tools and services for NullBlock development on macOS

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
NC='\033[0m' # No Color

# Counters
PASS_COUNT=0
FAIL_COUNT=0
WARN_COUNT=0

print_header() {
    echo -e "${MAGENTA}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${MAGENTA}  $1${NC}"
    echo -e "${MAGENTA}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

print_check() {
    echo -e "${BLUE}🔍 Checking:${NC} $1"
}

print_pass() {
    echo -e "${GREEN}✅ PASS:${NC} $1"
    ((PASS_COUNT++))
}

print_fail() {
    echo -e "${RED}❌ FAIL:${NC} $1"
    ((FAIL_COUNT++))
}

print_warn() {
    echo -e "${YELLOW}⚠️  WARN:${NC} $1"
    ((WARN_COUNT++))
}

print_info() {
    echo -e "${BLUE}ℹ️  INFO:${NC} $1"
}

print_header "NullBlock Mac Development Environment Validation"

# Check OS
print_header "System Information"
if [[ "$OSTYPE" == "darwin"* ]]; then
    print_pass "Running on macOS"
    print_info "OS: $(sw_vers -productName) $(sw_vers -productVersion)"
    print_info "Architecture: $(uname -m)"
else
    print_fail "Not running on macOS (detected: $OSTYPE)"
    exit 1
fi

# Check Homebrew
print_header "Package Managers"
print_check "Homebrew"
if command -v brew &> /dev/null; then
    BREW_VERSION=$(brew --version | head -n 1)
    print_pass "Homebrew installed ($BREW_VERSION)"
    print_info "Homebrew prefix: $(brew --prefix)"
else
    print_fail "Homebrew not found"
    print_info "Install from: https://brew.sh"
fi

# Check Docker
print_header "Container Runtime"
print_check "Docker"
if command -v docker &> /dev/null; then
    DOCKER_VERSION=$(docker --version)
    print_pass "Docker installed ($DOCKER_VERSION)"

    if docker info &> /dev/null; then
        print_pass "Docker daemon is running"
        print_info "Docker is ready"
    else
        print_fail "Docker daemon is not running"
        print_info "Start Docker Desktop from Applications"
    fi
else
    print_fail "Docker not found"
    print_info "Install Docker Desktop from: https://www.docker.com/products/docker-desktop"
fi

# Check Rust/Cargo
print_header "Development Tools - Rust"
print_check "Rust toolchain (cargo, rustc)"
if command -v cargo &> /dev/null; then
    CARGO_VERSION=$(cargo --version)
    RUSTC_VERSION=$(rustc --version)
    print_pass "Cargo installed ($CARGO_VERSION)"
    print_pass "Rustc installed ($RUSTC_VERSION)"
else
    print_fail "Rust toolchain not found"
    print_info "Install from: https://rustup.rs"
fi

# Check Node.js/npm
print_header "Development Tools - Node.js"
print_check "Node.js and npm"
if command -v node &> /dev/null; then
    NODE_VERSION=$(node --version)
    NPM_VERSION=$(npm --version)
    print_pass "Node.js installed ($NODE_VERSION)"
    print_pass "npm installed ($NPM_VERSION)"
else
    print_fail "Node.js not found"
    print_info "Install via: brew install node"
fi

# Check tmux
print_header "Terminal Multiplexer"
print_check "tmux"
if command -v tmux &> /dev/null; then
    TMUX_VERSION=$(tmux -V)
    print_pass "tmux installed ($TMUX_VERSION)"
else
    print_fail "tmux not found"
    print_info "Install via: brew install tmux"
fi

# Check Ruby
print_check "Ruby (for tmuxinator)"
if command -v ruby &> /dev/null; then
    RUBY_VERSION=$(ruby --version | cut -d' ' -f2)
    print_pass "Ruby installed ($RUBY_VERSION)"

    if command -v gem &> /dev/null; then
        print_pass "RubyGems available"

        if gem list tmuxinator -i &> /dev/null; then
            print_pass "tmuxinator gem installed"
        else
            print_warn "tmuxinator not installed"
            print_info "Install via: gem install --user-install tmuxinator"
        fi
    fi
else
    print_fail "Ruby not found"
    print_info "Install via: brew install ruby"
fi

# Check Just
print_check "Just command runner"
if command -v just &> /dev/null; then
    JUST_VERSION=$(just --version)
    print_pass "Just installed ($JUST_VERSION)"
else
    print_warn "Just not found (optional but recommended)"
    print_info "Install via: brew install just"
fi

# Check Git
print_check "Git"
if command -v git &> /dev/null; then
    GIT_VERSION=$(git --version)
    print_pass "Git installed ($GIT_VERSION)"
else
    print_fail "Git not found"
    print_info "Install via: xcode-select --install"
fi

# Check curl
print_check "curl"
if command -v curl &> /dev/null; then
    CURL_VERSION=$(curl --version | head -n 1)
    print_pass "curl installed ($CURL_VERSION)"
else
    print_fail "curl not found"
fi

# Check jq
print_check "jq (JSON processor)"
if command -v jq &> /dev/null; then
    JQ_VERSION=$(jq --version)
    print_pass "jq installed ($JQ_VERSION)"
else
    print_warn "jq not found (optional but useful for testing)"
    print_info "Install via: brew install jq"
fi

# Check cmake
print_check "cmake (required for Rust Kafka client)"
if command -v cmake &> /dev/null; then
    CMAKE_VERSION=$(cmake --version | head -n 1)
    print_pass "cmake installed ($CMAKE_VERSION)"
else
    print_fail "cmake not found (required for building rdkafka-sys)"
    print_info "Install via: brew install cmake"
fi

# Check PostgreSQL client
print_header "Database Tools"
print_check "PostgreSQL client (psql)"
if command -v psql &> /dev/null; then
    PSQL_VERSION=$(psql --version)
    print_pass "psql installed ($PSQL_VERSION)"
else
    print_warn "PostgreSQL client not found (optional - using Docker)"
    print_info "Install via: brew install postgresql@17"
fi

# Check Redis client
print_check "Redis client (redis-cli)"
if command -v redis-cli &> /dev/null; then
    REDIS_VERSION=$(redis-cli --version)
    print_pass "redis-cli installed ($REDIS_VERSION)"
else
    print_warn "Redis client not found (optional - using Docker)"
    print_info "Install via: brew install redis"
fi

# Check Docker containers
if docker info &> /dev/null; then
    print_header "Docker Containers Status"

    CONTAINERS=(
        "nullblock-postgres-erebus:Erebus Database"
        "nullblock-postgres-agents:Agents Database"
        "nullblock-redis:Redis Cache"
        "nullblock-kafka:Kafka Broker"
        "nullblock-zookeeper:Zookeeper"
    )

    for container_info in "${CONTAINERS[@]}"; do
        IFS=':' read -r container_name description <<< "$container_info"
        print_check "$description"
        if docker ps --format '{{.Names}}' | grep -q "^${container_name}$"; then
            print_pass "$description is running"
        else
            print_warn "$description not running"
            print_info "Will be started by 'just start' or dev-mac workflow"
        fi
    done
fi

# Check environment files
print_header "Configuration Files"
print_check ".env.dev file"
if [ -f ".env.dev" ]; then
    print_pass ".env.dev exists"

    if grep -q "OPENROUTER_API_KEY" .env.dev; then
        if grep -q "OPENROUTER_API_KEY=your-openrouter-key-here" .env.dev || grep -q "OPENROUTER_API_KEY=$" .env.dev; then
            print_warn "OPENROUTER_API_KEY needs to be configured"
            print_info "Get free API key from: https://openrouter.ai"
        else
            print_pass "OPENROUTER_API_KEY is configured"
        fi
    fi
else
    print_warn ".env.dev not found"
    print_info "Will be created by dev-mac workflow"
fi

# Check tmuxinator config
print_check "Tmuxinator config"
if [ -f "$HOME/.config/tmuxinator/nullblock-dev.yml" ] || [ -L "$HOME/.config/tmuxinator/nullblock-dev.yml" ]; then
    print_pass "Tmuxinator config exists"
else
    print_warn "Tmuxinator config not found"
    print_info "Will be created by './scripts/dev-mac setup'"
fi

# Check project structure
print_header "Project Structure"
REQUIRED_DIRS=(
    "svc/erebus"
    "svc/nullblock-agents"
    "svc/nullblock-protocols"
    "svc/hecate"
    "scripts"
)

for dir in "${REQUIRED_DIRS[@]}"; do
    print_check "$dir directory"
    if [ -d "$dir" ]; then
        print_pass "$dir exists"
    else
        print_fail "$dir not found"
    fi
done

# Summary
print_header "Validation Summary"
echo -e "${GREEN}✅ Passed:  $PASS_COUNT${NC}"
echo -e "${YELLOW}⚠️  Warnings: $WARN_COUNT${NC}"
echo -e "${RED}❌ Failed:  $FAIL_COUNT${NC}"

if [ $FAIL_COUNT -eq 0 ]; then
    echo ""
    print_pass "System is ready for NullBlock development!"
    echo ""
    print_info "Next steps:"
    echo "  1. Configure API keys in .env.dev (if needed)"
    echo "  2. Run: just start (to start infrastructure)"
    echo "  3. Run: just dev-mac (to launch full dev environment)"
    exit 0
else
    echo ""
    print_fail "System has $FAIL_COUNT critical issue(s) that must be resolved"
    echo ""
    print_info "Please install missing tools and re-run this script"
    exit 1
fi
