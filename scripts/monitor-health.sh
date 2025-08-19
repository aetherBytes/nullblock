#!/bin/bash

# Health Monitoring Script for Nullblock Development Environment
# Provides real-time service status and log statistics

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

# Check if we're in the right directory
if [ ! -f "package.json" ] || [ ! -d "svc" ]; then
    print_error "❌ Please run this script from the nullblock root directory"
    exit 1
fi

print_header "📈 Nullblock Health Monitor"
print_status "🔍 Monitoring service status and log statistics"
print_status "💡 Press Ctrl+C to stop monitoring"
print_status "=" * 80

while true; do
    # Clear screen for fresh display
    clear
    
    print_header "📈 Nullblock Health Monitor"
    print_status "🕐 Last updated: $(date '+%Y-%m-%d %H:%M:%S')"
    print_status "=" * 80
    
    # Service Status Section
    print_subheader "🔧 Service Status"
    
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
                print_error "❌ $name (port $port) - Not responding"
            fi
        else
            # Check service
            if brew services list | grep -q "$port.*started"; then
                print_success "✅ $name - Running"
            else
                print_error "❌ $name - Not running"
            fi
        fi
    done
    
    echo ""
    
    # Log Statistics Section
    print_subheader "📊 Log Statistics"
    
    if [ -d logs ]; then
        log_files=($(ls logs/*.log 2>/dev/null || true))
        
        if [ ${#log_files[@]} -eq 0 ]; then
            print_warning "⚠️  No log files found in logs/ directory"
        else
            for logfile in "${log_files[@]}"; do
                if [ -f "$logfile" ]; then
                    size=$(du -h "$logfile" | cut -f1)
                    lines=$(wc -l < "$logfile" 2>/dev/null || echo "0")
                    last_modified=$(stat -f "%Sm" -t "%H:%M:%S" "$logfile" 2>/dev/null || echo "unknown")
                    
                    # Check if log file is actively being written to
                    if [ "$lines" -gt 0 ]; then
                        print_success "📄 $(basename "$logfile"): $size ($lines lines) - Last: $last_modified"
                    else
                        print_warning "📄 $(basename "$logfile"): $size ($lines lines) - Last: $last_modified"
                    fi
                fi
            done
        fi
    else
        print_error "❌ Logs directory not found"
    fi
    
    echo ""
    
    # System Resources Section
    print_subheader "💻 System Resources"
    
    # CPU usage
    cpu_usage=$(top -l 1 | grep "CPU usage" | awk '{print $3}' | sed 's/%//')
    print_status "🖥️  CPU Usage: ${cpu_usage}%"
    
    # Memory usage
    memory_info=$(vm_stat | grep "Pages free" | awk '{print $3}' | sed 's/\.//')
    memory_gb=$(echo "scale=2; $memory_info/1024/1024" | bc 2>/dev/null || echo "unknown")
    print_status "🧠 Available Memory: ${memory_gb}GB"
    
    # Disk usage
    disk_usage=$(df -h . | tail -1 | awk '{print $5}')
    print_status "💾 Disk Usage: $disk_usage"
    
    echo ""
    
    # Quick Actions Section
    print_subheader "⚡ Quick Actions"
    print_status "💡 Commands you can run in another terminal:"
    print_status "  tail -f logs/*.log          # Monitor all logs"
    print_status "  curl http://localhost:9002/health  # Check Hecate health"
    print_status "  curl http://localhost:8001/health  # Check MCP health"
    print_status "  ./scripts/stop-hecate-simple.sh   # Stop all services"
    
    echo ""
    print_status "🔄 Refreshing in 10 seconds... (Press Ctrl+C to stop)"
    print_status "=" * 80
    
    sleep 10
done
