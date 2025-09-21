#!/bin/bash

# Nullblock Health Monitor
# Compact service status monitoring for the logs tab

# Note: Removed 'set -e' to prevent script exit on service check failures

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
GRAY='\033[0;90m'
BRIGHT_GREEN='\033[1;32m'
BRIGHT_BLUE='\033[1;34m'
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

# Function to check if a service is running
check_service() {
    local name="$1"
    local port="$2"
    local url="$3"
    local check_type="$4"

    if [ "$check_type" = "port" ]; then
        if lsof -i :$port > /dev/null 2>&1; then
            echo "✅"
        else
            echo "❌"
        fi
    elif [ "$check_type" = "http" ]; then
        if curl -s --max-time 2 "$url" > /dev/null 2>&1; then
            echo "✅"
        else
            echo "❌"
        fi
    elif [ "$check_type" = "openrouter" ]; then
        # Check OpenRouter API with authentication (no tokens spent)
        local api_key="${OPENROUTER_API_KEY:-}"
        if [ -z "$api_key" ] && [ -f "$HOME/nullblock/.env.dev" ]; then
            api_key=$(grep "OPENROUTER_API_KEY" "$HOME/nullblock/.env.dev" 2>/dev/null | cut -d'=' -f2 | tr -d '"' | tr -d ' ')
        fi

        if [ -n "$api_key" ]; then
            local response=$(curl -s --max-time 5 -H "Authorization: Bearer $api_key" "https://openrouter.ai/api/v1/models" 2>/dev/null)
            if echo "$response" | grep -q '"data"' && ! echo "$response" | grep -q '"error"'; then
                echo "✅"
            else
                echo "❌"
            fi
        else
            echo "❓"
        fi
    elif [ "$check_type" = "systemctl" ]; then
        # Check systemctl services (Linux)
        case $OS in
            "macos")
                if brew services list | grep -q "$name.*started"; then
                    echo "✅"
                else
                    echo "❌"
                fi
                ;;
            *)
                if systemctl --user is-active "$name" > /dev/null 2>&1; then
                    echo "✅"
                else
                    echo "❌"
                fi
                ;;
        esac
    elif [ "$check_type" = "process" ]; then
        if pgrep -f "$name" > /dev/null; then
            echo "✅"
        else
            echo "❌"
        fi
    else
        echo "❓"
    fi
}

# Function to print compact status table
print_status() {
    # Detect OS first
    detect_os

    local timestamp=$(date '+%H:%M:%S')
    echo -e "${BRIGHT_BLUE}┌─ NULLBLOCK SERVICE STATUS ─ $timestamp ──────────────────────────┐${NC}"

    # Infrastructure
    local postgres15_status=$(check_service "postgresql@15" "" "" "systemctl")
    local postgres17_status=$(check_service "postgresql" "" "" "systemctl")
    local redis_status=$(check_service "redis" "" "" "systemctl")
    local ipfs_status=$(check_service "ipfs daemon" "" "" "process")
    
    # Backend Services
    local mcp_status=$(check_service "MCP" "8001" "http://localhost:8001/health" "http")
    local orch_status=$(check_service "Orchestration" "8002" "http://localhost:8002/health" "http")
    local erebus_status=$(check_service "Erebus" "3000" "http://localhost:3000/health" "http")
    
    # Agent Services  
    local agents_status=$(check_service "General Agents" "9001" "http://localhost:9001/health" "http")
    local hecate_status=$(check_service "Hecate Agent" "9003" "http://localhost:9003/hecate/health" "http")
    
    # Frontend & LLM
    local frontend_status=$(check_service "Frontend" "5173" "http://localhost:5173" "http")
    local openrouter_status=$(check_service "OpenRouter" "" "" "openrouter")
    
    echo -e "${WHITE}│ Infrastructure: ${postgres15_status} PG@15 ${postgres17_status} PG@17 ${redis_status} Redis ${ipfs_status} IPFS                        │${NC}"
    echo -e "${WHITE}│ Backend:        ${mcp_status} MCP:8001 ${orch_status} Orchestration:8002 ${erebus_status} Erebus:3000            │${NC}"
    echo -e "${WHITE}│ Agents:         ${agents_status} General:9001 ${hecate_status} Hecate:9003                              │${NC}"
    echo -e "${WHITE}│ Frontend:       ${frontend_status} Vite:5173 ${openrouter_status} OpenRouter API                        │${NC}"
    
    # Count services
    local all_statuses=("$postgres15_status" "$postgres17_status" "$redis_status" "$ipfs_status" "$mcp_status" "$orch_status" "$erebus_status" "$agents_status" "$hecate_status" "$frontend_status" "$openrouter_status")
    local online_count=0
    for status in "${all_statuses[@]}"; do
        if [[ "$status" == "✅" ]]; then
            ((online_count++))
        fi
    done
    local total_count=${#all_statuses[@]}
    local offline_count=$((total_count - online_count))
    
    if [ $offline_count -eq 0 ]; then
        echo -e "${WHITE}│ Status:         ${BRIGHT_GREEN}🎉 ALL $total_count SERVICES ONLINE${NC} ${WHITE}                                    │${NC}"
    else
        echo -e "${WHITE}│ Status:         ${online_count}/${total_count} online (${YELLOW}${offline_count} offline${NC}${WHITE})                           │${NC}"
    fi
    
    echo -e "${BRIGHT_BLUE}└────────────────────────────────────────────────────────────────────┘${NC}"
}

# Function to show detailed log file status
show_log_summary() {
    echo -e "${CYAN}📋 Log File Status:${NC}"

    # Updated log file paths to match actual file locations
    local log_files=(
        # Main logs directory
        "$HOME/nullblock/logs/just-commands.log"
        "$HOME/nullblock/logs/frontend.log"
        "$HOME/nullblock/logs/ipfs.log"
        "$HOME/nullblock/logs/erebus.log"
        "$HOME/nullblock/logs/mcp.log"
        "$HOME/nullblock/logs/orchestration.log"

        # Agent logs (actual paths)
        "$HOME/nullblock/svc/nullblock-agents/logs/hecate.log"
        "$HOME/nullblock/svc/nullblock-agents/logs/hecate-server.log"
        "$HOME/nullblock/svc/nullblock-agents/logs/hecate-startup.log"
        "$HOME/nullblock/svc/nullblock-agents/agents.log"

        # Service-specific logs
        "$HOME/nullblock/svc/nullblock-mcp/logs/mcp-server.log"
        "$HOME/nullblock/svc/nullblock-orchestration/logs/orchestration.log"

        # Erebus logs
        "$HOME/nullblock/svc/erebus/logs/erebus.log"

        # LLM Service logs (OpenRouter)
        "$HOME/nullblock/svc/nullblock-agents/logs/llm-health.log"
        "$HOME/nullblock/svc/nullblock-agents/logs/model-performance.log"
    )
    
    local active_logs=0
    local total_size=0
    
    for log_file in "${log_files[@]}"; do
        if [ -f "$log_file" ]; then
            # Cross-platform stat command
            local size_bytes
            if [[ "$OSTYPE" == "darwin"* ]]; then
                size_bytes=$(stat -f%z "$log_file" 2>/dev/null || echo "0")
            else
                size_bytes=$(stat -c%s "$log_file" 2>/dev/null || echo "0")
            fi
            local size_human=$(du -h "$log_file" 2>/dev/null | cut -f1)
            local lines=$(wc -l < "$log_file" 2>/dev/null | tr -d ' ')
            
            echo -e "  ${GREEN}✓${NC} $(basename "$log_file") (${size_human}, ${lines} lines)"
            active_logs=$((active_logs + 1))
            total_size=$((total_size + size_bytes))
        else
            echo -e "  ${GRAY}✗${NC} $(basename "$log_file") ${GRAY}(not found)${NC}"
        fi
    done
    
    # Better size calculation
    local size_display
    if [ $total_size -gt 1048576 ]; then
        size_display="$((total_size / 1024 / 1024))MB"
    elif [ $total_size -gt 1024 ]; then
        size_display="$((total_size / 1024))KB"
    else
        size_display="${total_size}B"
    fi
    echo -e "${CYAN}📊 Summary: ${active_logs}/${#log_files[@]} active, ${size_display} total${NC}"
    echo ""
}

# Main monitoring loop
main() {
    echo -e "${BRIGHT_BLUE}🔍 NULLBLOCK HEALTH MONITOR${NC}"
    echo -e "${GRAY}Compact service status monitoring${NC}"
    echo -e "${GRAY}Press Ctrl+C to stop${NC}"
    echo ""
    
    # Initial status - Log files first, then services
    show_log_summary
    print_status
    echo ""
    
    # Continuous monitoring with adaptive intervals
    local check_count=0
    while true; do
        check_count=$((check_count + 1))
        
        # First 30 seconds: check every 10 seconds (3 checks = 30 seconds)
        if [ $check_count -le 3 ]; then
            sleep 10
            echo -e "${GRAY}[Check $check_count/3 - Fast mode: 10s intervals]${NC}"
        else
            # After 30 seconds: check every 5 minutes (standardized)
            sleep 300
            echo -e "${GRAY}[Check $check_count - Standard mode: 5min intervals]${NC}"
        fi
        
        clear
        echo -e "${BRIGHT_BLUE}🔍 NULLBLOCK HEALTH MONITOR${NC}"
        echo -e "${GRAY}Compact service status monitoring${NC}"
        echo -e "${GRAY}Press Ctrl+C to stop${NC}"
        echo ""
        show_log_summary
        print_status
        echo ""
    done
}

# Run main function
main "$@"