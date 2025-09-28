#!/bin/bash

# Cross-platform service killer script
# This script kills all development services in an OS-aware manner

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Detect operating system
detect_os() {
    if [[ "$OSTYPE" == "darwin"* ]]; then
        OS="macos"
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        if command -v pacman &> /dev/null; then
            OS="arch"
        elif command -v apt &> /dev/null; then
            OS="ubuntu"
        elif command -v dnf &> /dev/null; then
            OS="fedora"
        else
            OS="linux"
        fi
    else
        OS="unknown"
    fi
}

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

# Kill development services
kill_services() {
    detect_os

    echo "💀 Killing all development services..."

    # Stop Docker containers first
    echo "Stopping Docker containers..."
    docker-compose down 2>/dev/null || true

    # Stop individual containers if they're still running
    echo "Killing individual Docker containers..."
    docker stop nullblock-postgres-agents nullblock-postgres-erebus nullblock-postgres-protocols nullblock-postgres-orchestration nullblock-postgres-analytics 2>/dev/null || true
    docker stop nullblock-redis nullblock-kafka nullblock-zookeeper nullblock-ipfs 2>/dev/null || true
    docker stop nullblock-protocols nullblock-orchestration nullblock-agents nullblock-hecate nullblock-erebus nullblock-nginx 2>/dev/null || true

    # Remove containers
    docker rm -f nullblock-postgres-agents nullblock-postgres-erebus nullblock-postgres-protocols nullblock-postgres-orchestration nullblock-postgres-analytics 2>/dev/null || true
    docker rm -f nullblock-redis nullblock-kafka nullblock-zookeeper nullblock-ipfs 2>/dev/null || true
    docker rm -f nullblock-protocols nullblock-orchestration nullblock-agents nullblock-hecate nullblock-erebus nullblock-nginx 2>/dev/null || true

    # Stop PostgreSQL (user mode - fallback)
    echo "Stopping local PostgreSQL (if any)..."
    case $OS in
        "macos")
            brew services stop postgresql@17 2>/dev/null || true
            ;;
        *)
            systemctl --user stop postgresql 2>/dev/null || true
            ;;
    esac

    # Stop Redis (user mode - fallback)
    echo "Stopping local Redis (if any)..."
    case $OS in
        "macos")
            brew services stop redis 2>/dev/null || true
            ;;
        "arch")
            systemctl --user stop redis 2>/dev/null || true
            ;;
        *)
            systemctl --user stop redis-server 2>/dev/null || true
            ;;
    esac

    # Kill IPFS daemon
    echo "Killing IPFS daemon..."
    pkill -f "ipfs daemon" 2>/dev/null || true

    # Kill services on development ports
    echo "Killing services on development ports..."
    lsof -ti:8001 | xargs kill -9 2>/dev/null || true  # Protocol Server
    lsof -ti:8002 | xargs kill -9 2>/dev/null || true  # Orchestration
    lsof -ti:9001 | xargs kill -9 2>/dev/null || true  # General Agents
    lsof -ti:9003 | xargs kill -9 2>/dev/null || true  # Hecate Agent (Rust)
    lsof -ti:3000 | xargs kill -9 2>/dev/null || true  # Erebus
    lsof -ti:5173 | xargs kill -9 2>/dev/null || true  # Vite dev server
    lsof -ti:1234 | xargs kill -9 2>/dev/null || true  # LM Studio

    # Stop LM Studio server
    echo "Stopping LM Studio server..."
    lms server stop 2>/dev/null || true

    # Kill tmuxinator sessions
    echo "Killing tmuxinator sessions..."
    tmux kill-session -t nullblock-dev 2>/dev/null || true

    # Clean up PID files
    echo "Cleaning up PID files..."
    rm -f logs/*.pid 2>/dev/null || true

    # Optional: Clean up Docker volumes (commented out by default)
    # echo "Cleaning up Docker volumes..."
    # docker volume prune -f 2>/dev/null || true

    # Clean up Docker networks
    echo "Cleaning up Docker networks..."
    docker network rm nullblock-network 2>/dev/null || true

    echo "✅ All development services killed and cleaned up"
}

# Main execution
kill_services