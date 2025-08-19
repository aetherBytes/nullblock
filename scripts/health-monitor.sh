#!/bin/bash

# Nullblock Health Monitor
# Compact service status monitoring for the logs tab

set -e

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
    elif [ "$check_type" = "brew" ]; then
        if brew services list | grep -q "$name.*started"; then
            echo "✅"
        else
            echo "❌"
        fi
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
    local timestamp=$(date '+%H:%M:%S')
    echo -e "${BRIGHT_BLUE}┌─ NULLBLOCK SERVICE STATUS ─ $timestamp ──────────────────────────┐${NC}"
    
    # Infrastructure
    local postgres15_status=$(check_service "postgresql@15" "" "" "brew")
    local postgres17_status=$(check_service "postgresql@17" "" "" "brew")
    local redis_status=$(check_service "redis" "" "" "brew") 
    local ipfs_status=$(check_service "ipfs daemon" "" "" "process")
    
    # Backend Services
    local mcp_status=$(check_service "MCP" "8001" "http://localhost:8001/health" "http")
    local orch_status=$(check_service "Orchestration" "8002" "http://localhost:8002/health" "http")
    local erebus_status=$(check_service "Erebus" "3000" "http://localhost:3000/health" "http")
    
    # Agent Services  
    local agents_status=$(check_service "General Agents" "9001" "http://localhost:9001/health" "http")
    local hecate_status=$(check_service "Hecate Agent" "9002" "http://localhost:9002/health" "http")
    
    # Frontend & LLM
    local frontend_status=$(check_service "Frontend" "5173" "http://localhost:5173" "http")
    local lm_status=$(check_service "LM Studio" "1234" "" "port")
    
    echo -e "${WHITE}│ Infrastructure: ${postgres15_status} PG@15 ${postgres17_status} PG@17 ${redis_status} Redis ${ipfs_status} IPFS                        │${NC}"
    echo -e "${WHITE}│ Backend:        ${mcp_status} MCP:8001 ${orch_status} Orchestration:8002 ${erebus_status} Erebus:3000            │${NC}"
    echo -e "${WHITE}│ Agents:         ${agents_status} General:9001 ${hecate_status} Hecate:9002                              │${NC}"
    echo -e "${WHITE}│ Frontend:       ${frontend_status} Vite:5173 ${lm_status} LM Studio:1234                             │${NC}"
    
    # Count services
    local all_statuses=("$postgres15_status" "$postgres17_status" "$redis_status" "$ipfs_status" "$mcp_status" "$orch_status" "$erebus_status" "$agents_status" "$hecate_status" "$frontend_status" "$lm_status")
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
        "/Users/sage/nullblock/logs/just-commands.log"
        "/Users/sage/nullblock/logs/frontend.log"
        "/Users/sage/nullblock/logs/ipfs.log"
        "/Users/sage/nullblock/logs/erebus.log"
        "/Users/sage/nullblock/logs/erebus-temp.log"
        "/Users/sage/nullblock/logs/mcp.log"
        "/Users/sage/nullblock/logs/orchestration.log"
        
        # Agent logs (actual paths)
        "/Users/sage/nullblock/svc/nullblock-agents/logs/hecate.log"
        "/Users/sage/nullblock/svc/nullblock-agents/logs/hecate-server.log"
        "/Users/sage/nullblock/svc/nullblock-agents/logs/hecate-startup.log"
        "/Users/sage/nullblock/svc/nullblock-agents/agents.log"
        
        # Service-specific logs
        "/Users/sage/nullblock/svc/nullblock-mcp/logs/mcp-server.log"
        "/Users/sage/nullblock/svc/nullblock-orchestration/logs/orchestration.log"
        
        # LLM Service logs (input/output tracking)
        "/Users/sage/Library/Logs/LM Studio/main.log"
        "/Users/sage/nullblock/logs/lm-studio-stream.log"
        "/Users/sage/nullblock/logs/lm-studio-monitor.log"
    )
    
    local active_logs=0
    local total_size=0
    
    for log_file in "${log_files[@]}"; do
        if [ -f "$log_file" ]; then
            local size_bytes=$(stat -f%z "$log_file" 2>/dev/null || echo "0")
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
    
    # Initial status
    print_status
    echo ""
    show_log_summary
    
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
        print_status
        echo ""
        show_log_summary
    done
}

# Run main function
main "$@"