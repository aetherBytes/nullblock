#!/bin/bash

# Nullblock Development Setup Script
# This script sets up the development environment for local development

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

# Check if Python 3.12 is installed
check_python() {
    if ! command -v python3 &> /dev/null; then
        print_error "Python 3 is required but not installed."
        print_status "Please install Python 3 and try again."
        exit 1
    fi
    print_success "Python 3 is available"
}

# Check if Node.js is installed
check_node() {
    if ! command -v node &> /dev/null; then
        print_error "Node.js is required but not installed."
        print_status "Please install Node.js 18+ and try again."
        exit 1
    fi
    print_success "Node.js is available"
}

# Check if Rust is installed
check_rust() {
    if ! command -v cargo &> /dev/null; then
        print_error "Rust is required but not installed."
        print_status "Please install Rust and try again."
        exit 1
    fi
    print_success "Rust is available"
}

# Install Python dependencies
install_python_deps() {
    print_status "Installing Python dependencies..."

    # Note: nullblock-mcp has been replaced by nullblock-protocols (Rust)
    # Keeping orchestration and agents for now
    
    # Install orchestration dependencies
    cd svc/nullblock-orchestration
    python3.12 -m pip install -e .
    cd ../..
    
    # Install agents dependencies
    cd svc/nullblock-agents
    python3.12 -m pip install -e .
    cd ../..
    
    print_success "Python dependencies installed"
}

# Install Node.js dependencies
install_node_deps() {
    print_status "Installing Node.js dependencies..."
    
    cd svc/hecate
    npm install
    cd ../..
    
    print_success "Node.js dependencies installed"
}

# Install Rust dependencies
install_rust_deps() {
    print_status "Installing Rust dependencies..."
    
    cd svc/erebus
    cargo build
    cd ../..
    
    print_success "Rust dependencies installed"
}

# Create development environment file
create_dev_env() {
    if [ ! -f .env.dev ]; then
        print_status "Creating development environment file..."
        cat > .env.dev << EOF
# Nullblock Development Environment

# Ethereum RPC URLs (use testnet for development)
ETHEREUM_RPC_URL=https://eth-sepolia.alchemyapi.io/v2/your-key
POLYGON_RPC_URL=https://polygon-mumbai.alchemyapi.io/v2/your-key
WEB3_RPC_URL=https://eth-sepolia.alchemyapi.io/v2/your-key

# Flashbots Configuration (use testnet)
FLASHBOTS_RPC_URL=https://relay-goerli.flashbots.net
FLASHBOTS_PRIVATE_KEY=your-flashbots-private-key
ENABLE_MEV_PROTECTION=true

# Bittensor Configuration
BITTENSOR_NETWORK=test
BITTENSOR_WALLET_PATH=your-bittensor-wallet-path

# Database Configuration (local)
# Docker Database URLs (using mapped ports)
DATABASE_URL=postgresql://postgres:REDACTED_DB_PASS@localhost:5441/agents
# Note: Protocols service uses Agents database for integration (no separate database needed)
ORCHESTRATION_DATABASE_URL=postgresql://postgres:REDACTED_DB_PASS@localhost:5443/orchestration
AGENTS_DATABASE_URL=postgresql://postgres:REDACTED_DB_PASS@localhost:5441/agents
EREBUS_DATABASE_URL=postgresql://postgres:REDACTED_DB_PASS@localhost:5440/erebus
HECATE_DATABASE_URL=postgresql://postgres:REDACTED_DB_PASS@localhost:5441/agents
PLATFORM_DATABASE_URL=postgresql://postgres:REDACTED_DB_PASS@localhost:5444/analytics
REDIS_URL=redis://localhost:6379

# IPFS Configuration (local)
IPFS_API_URL=http://localhost:5001

# API Keys (comma-separated)
DEX_API_KEYS=your-dex-api-keys

# Solana Configuration (devnet)
SOLANA_RPC_URL=https://api.devnet.solana.com

# Frontend Configuration
VITE_PROTOCOLS_API_URL=http://localhost:8001
VITE_ORCHESTRATION_API_URL=http://localhost:8002
VITE_AGENTS_API_URL=http://localhost:9001
EOF
        print_warning "Please edit .env.dev file with your actual API keys and configuration"
    else
        print_success ".env.dev file already exists"
    fi
}

# Start development services
start_dev_services() {
    print_status "Starting development services..."
    
    # Start PostgreSQL (if available)
    if command -v brew &> /dev/null; then
        print_status "Starting PostgreSQL via Homebrew..."
        brew services start postgresql@15 || print_warning "PostgreSQL not available via Homebrew"
    fi
    
    # Start Redis (if available)
    if command -v brew &> /dev/null; then
        print_status "Starting Redis via Homebrew..."
        brew services start redis || print_warning "Redis not available via Homebrew"
    fi
    
    # Start IPFS (if available)
    if command -v ipfs &> /dev/null; then
        print_status "Starting IPFS..."
        ipfs daemon --enable-gc &
        sleep 5
    else
        print_warning "IPFS not installed. Please install IPFS for full functionality."
    fi
}

# Create development scripts
create_dev_scripts() {
    print_status "Creating development scripts..."
    
    # Create script to start Protocols server
    cat > start-protocols.sh << 'EOF'
#!/bin/bash
cd svc/nullblock-protocols
export $(cat ../../.env.dev | xargs)
# Protocols service uses Agents database for integration
export DATABASE_URL="$AGENTS_DATABASE_URL"
cargo run
EOF
    
    # Create script to start orchestration
    cat > start-orchestration.sh << 'EOF'
#!/bin/bash
cd svc/nullblock-orchestration
export $(cat ../../.env.dev | xargs)
python3 -m orchestration.server
EOF
    
    # Create script to start agents
    cat > start-agents.sh << 'EOF'
#!/bin/bash
cd svc/nullblock-agents
export $(cat ../../.env.dev | xargs)
python3 -m agents.server
EOF
    
    # Create script to start frontend
    cat > start-frontend.sh << 'EOF'
#!/bin/bash
cd svc/hecate
export $(cat ../../.env.dev | xargs)
npm run develop
EOF
    
    # Create script to start Erebus
    cat > start-erebus.sh << 'EOF'
#!/bin/bash
cd svc/erebus
export $(cat ../../.env.dev | xargs)
cargo run
EOF
    
    # Create script to build Erebus
    cat > build-erebus.sh << 'EOF'
#!/bin/bash
cd svc/erebus
cargo build --release
echo "Erebus build complete"
EOF
    
    # Make scripts executable
    chmod +x start-protocols.sh start-orchestration.sh start-agents.sh start-frontend.sh start-erebus.sh build-erebus.sh
    
    print_success "Development scripts created"
}

# Show development instructions
show_instructions() {
    echo ""
    print_status "Development Setup Complete!"
    echo ""
    echo "To start development:"
    echo "  1. Edit .env.dev with your API keys"
    echo "  2. Start infrastructure services:"
    echo "     - PostgreSQL: brew services start postgresql@15"
    echo "     - Redis: brew services start redis"
    echo "     - IPFS: ipfs daemon --enable-gc"
    echo ""
    echo "  3. Start services in separate terminals:"
    echo "     - Protocols: ./start-protocols.sh"
    echo "     - Orchestration: ./start-orchestration.sh"
    echo "     - Agents: ./start-agents.sh"
    echo "     - Erebus: ./start-erebus.sh"
    echo "     - Frontend: ./start-frontend.sh"
    echo ""
    echo "Service URLs:"
    echo "  Frontend: http://localhost:5173"
    echo "  Protocols API: http://localhost:8001"
    echo "  Orchestration API: http://localhost:8002"
    echo "  Agents API: http://localhost:9001"
    echo "  Hecate Agent API: http://localhost:9003"
    echo "  Erebus API: http://localhost:3000"
    echo ""
}

# Main function
main() {
    case "${1:-setup}" in
        "setup")
            check_python
            check_node
            check_rust
            install_python_deps
            install_node_deps
            install_rust_deps
            create_dev_env
            start_dev_services
            create_dev_scripts
            show_instructions
            ;;
        "install")
            install_python_deps
            install_node_deps
            install_rust_deps
            ;;
        "env")
            create_dev_env
            ;;
        "scripts")
            create_dev_scripts
            ;;
        "services")
            start_dev_services
            ;;
        *)
            echo "Usage: $0 {setup|install|env|scripts|services}"
            echo ""
            echo "Commands:"
            echo "  setup     - Complete development setup (default)"
            echo "  install   - Install all dependencies"
            echo "  env       - Create development environment file"
            echo "  scripts   - Create development scripts"
            echo "  services  - Start development services"
            exit 1
            ;;
    esac
}

# Run main function
main "$@" 