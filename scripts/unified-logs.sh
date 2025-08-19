#!/usr/bin/env bash

# Nullblock Unified Log Stream
# Real-time monitoring of all service logs with color coding and service identification

set -e

# Colors for output (cyberpunk theme)
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
GRAY='\033[0;90m'
BRIGHT_RED='\033[1;31m'
BRIGHT_GREEN='\033[1;32m'
BRIGHT_BLUE='\033[1;34m'
BRIGHT_MAGENTA='\033[1;35m'
BRIGHT_CYAN='\033[1;36m'
NC='\033[0m' # No Color

# Service color mapping function (cyberpunk-themed)
get_service_color() {
    local service="$1"
    case "$service" in
        "MCP") echo "${BRIGHT_CYAN}" ;;
        "ORCHESTRATION") echo "${BRIGHT_MAGENTA}" ;;
        "EREBUS") echo "${BRIGHT_RED}" ;;
        "HECATE-AGENT") echo "${BRIGHT_GREEN}" ;;
        "HECATE-SERVER") echo "${GREEN}" ;;
        "HECATE-STARTUP") echo "${YELLOW}" ;;
        "FRONTEND") echo "${BLUE}" ;;
        "IPFS") echo "${MAGENTA}" ;;
        "AGENTS") echo "${CYAN}" ;;
        "INSTALL") echo "${GRAY}" ;;
        "UPDATE") echo "${GRAY}" ;;
        *) echo "${WHITE}" ;;
    esac
}

# Function to get service name from log file path
get_service_name() {
    local filepath="$1"
    local filename=$(basename "$filepath" .log)
    
    case "$filename" in
        "mcp"|"mcp-install")
            echo "MCP"
            ;;
        "orchestration"|"orchestration-install") 
            echo "ORCHESTRATION"
            ;;
        "erebus"|"erebus-update")
            echo "EREBUS"
            ;;
        "hecate-agent"|"hecate-agent-install")
            echo "HECATE-AGENT"
            ;;
        "hecate-server")
            echo "HECATE-SERVER"
            ;;
        "hecate-startup")
            echo "HECATE-STARTUP"
            ;;
        "hecate")
            echo "HECATE-AGENT"
            ;;
        "frontend"|"frontend-install")
            echo "FRONTEND"
            ;;
        "ipfs")
            echo "IPFS"
            ;;
        "agents")
            echo "AGENTS"
            ;;
        *)
            echo "UNKNOWN"
            ;;
    esac
}


# Function to format log line with service tag and color
format_log_line() {
    local service="$1"
    local line="$2"
    local color=$(get_service_color "$service")
    local timestamp=$(date '+%H:%M:%S')
    
    # Extract timestamp from log line if it exists
    if [[ "$line" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}[[:space:]]+[0-9]{2}:[0-9]{2}:[0-9]{2} ]]; then
        # Line already has timestamp, use it
        printf "${color}[%-14s]${NC} %s\n" "$service" "$line"
    elif [[ "$line" =~ ^\[[0-9]{2}:[0-9]{2}:[0-9]{2}\] ]]; then
        # Line has short timestamp
        printf "${color}[%-14s]${NC} %s\n" "$service" "$line"
    else
        # No timestamp in line, add our own
        printf "${color}[%-14s]${NC} ${GRAY}${timestamp}${NC} %s\n" "$service" "$line"
    fi
}

# Function to check if log file exists and is readable
check_log_file() {
    local filepath="$1"
    if [[ -f "$filepath" && -r "$filepath" ]]; then
        return 0
    else
        return 1
    fi
}

# Function to print header
print_header() {
    echo -e "${BRIGHT_CYAN}╔════════════════════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BRIGHT_CYAN}║${NC}                        ${BRIGHT_GREEN}🚀 NULLBLOCK UNIFIED LOG STREAM${NC}                        ${BRIGHT_CYAN}║${NC}"
    echo -e "${BRIGHT_CYAN}╠════════════════════════════════════════════════════════════════════════════════╣${NC}"
    echo -e "${BRIGHT_CYAN}║${NC} ${YELLOW}📊 Real-time monitoring of all Nullblock services                             ${BRIGHT_CYAN}║${NC}"
    echo -e "${BRIGHT_CYAN}║${NC} ${GRAY}Press Ctrl+C to stop monitoring                                               ${BRIGHT_CYAN}║${NC}"
    echo -e "${BRIGHT_CYAN}╚════════════════════════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    
    # Show service color legend
    echo -e "${WHITE}Service Legend:${NC}"
    local services=("MCP" "ORCHESTRATION" "EREBUS" "HECATE-AGENT" "HECATE-SERVER" "HECATE-STARTUP" "FRONTEND" "IPFS" "AGENTS" "INSTALL" "UPDATE")
    for service in "${services[@]}"; do
        local color=$(get_service_color "$service")
        printf "  ${color}● %-14s${NC}" "$service"
        case "$service" in
            "MCP") echo " - Model Context Protocol Server" ;;
            "ORCHESTRATION") echo " - Workflow Orchestration Engine" ;;
            "EREBUS") echo " - Rust Backend & Wallet Server" ;;
            "HECATE-AGENT") echo " - Primary Agent Interface" ;;
            "HECATE-SERVER") echo " - Agent HTTP API Server" ;;
            "HECATE-STARTUP") echo " - Agent Initialization" ;;
            "FRONTEND") echo " - React Frontend & Vite Dev Server" ;;
            "IPFS") echo " - InterPlanetary File System" ;;
            "AGENTS") echo " - General Agent Services" ;;
            "INSTALL") echo " - Installation & Setup Logs" ;;
            "UPDATE") echo " - Update & Dependency Logs" ;;
            *) echo "" ;;
        esac
    done
    echo ""
    echo -e "${BRIGHT_CYAN}═══════════════════════════════════════════════════════════════════════════════════${NC}"
    echo ""
}

# Function to show initial log file status
show_log_status() {
    echo -e "${WHITE}📋 Log File Status:${NC}"
    
    # Main logs directory
    local main_logs=(
        "/Users/sage/nullblock/logs/mcp.log"
        "/Users/sage/nullblock/logs/orchestration.log" 
        "/Users/sage/nullblock/logs/erebus.log"
        "/Users/sage/nullblock/logs/frontend.log"
        "/Users/sage/nullblock/logs/ipfs.log"
    )
    
    # Agent logs directory
    local agent_logs=(
        "/Users/sage/nullblock/svc/nullblock-agents/logs/hecate-agent.log"
        "/Users/sage/nullblock/svc/nullblock-agents/logs/hecate-server.log"
        "/Users/sage/nullblock/svc/nullblock-agents/logs/hecate-startup.log"
    )
    
    for log_file in "${main_logs[@]}" "${agent_logs[@]}"; do
        local service=$(get_service_name "$log_file")
        local color=$(get_service_color "$service")
        
        if check_log_file "$log_file"; then
            local size=$(du -h "$log_file" 2>/dev/null | cut -f1)
            local lines=$(wc -l < "$log_file" 2>/dev/null || echo "0")
            echo -e "  ${color}✓${NC} $(basename "$log_file") (${size}, ${lines} lines)"
        else
            echo -e "  ${GRAY}✗${NC} $(basename "$log_file") ${GRAY}(not found)${NC}"
        fi
    done
    echo ""
}

# Function to start log monitoring
start_log_monitoring() {
    # All potential log files to monitor
    local log_files=(
        "/Users/sage/nullblock/logs/mcp.log"
        "/Users/sage/nullblock/logs/orchestration.log"
        "/Users/sage/nullblock/logs/erebus.log"
        "/Users/sage/nullblock/logs/frontend.log"
        "/Users/sage/nullblock/logs/ipfs.log"
        "/Users/sage/nullblock/logs/mcp-install.log"
        "/Users/sage/nullblock/logs/orchestration-install.log"
        "/Users/sage/nullblock/logs/erebus-update.log"
        "/Users/sage/nullblock/logs/hecate-agent-install.log"
        "/Users/sage/nullblock/logs/frontend-install.log"
        "/Users/sage/nullblock/svc/nullblock-agents/logs/hecate.log"
        "/Users/sage/nullblock/svc/nullblock-agents/logs/hecate-server.log"
        "/Users/sage/nullblock/svc/nullblock-agents/logs/hecate-startup.log"
    )
    
    # Create array of existing log files
    local existing_files=()
    for log_file in "${log_files[@]}"; do
        if check_log_file "$log_file"; then
            existing_files+=("$log_file")
        fi
    done
    
    if [ ${#existing_files[@]} -eq 0 ]; then
        echo -e "${YELLOW}⚠️  No log files found yet. Waiting for services to start...${NC}"
        echo -e "${GRAY}   Services will create log files as they initialize.${NC}"
        echo ""
        
        # Wait for log files to appear
        while [ ${#existing_files[@]} -eq 0 ]; do
            sleep 2
            existing_files=()
            for log_file in "${log_files[@]}"; do
                if check_log_file "$log_file"; then
                    existing_files+=("$log_file")
                fi
            done
        done
        
        echo -e "${GREEN}✓ Log files detected! Starting monitoring...${NC}"
        echo ""
    fi
    
    # Start multitail with color formatting
    echo -e "${BRIGHT_BLUE}🔄 Starting unified log stream for ${#existing_files[@]} log files...${NC}"
    echo ""
    
    # Use multitail if available, otherwise fall back to custom tail solution
    if command -v multitail >/dev/null 2>&1; then
        # Build multitail command with color schemes
        local multitail_cmd="multitail"
        for log_file in "${existing_files[@]}"; do
            local service=$(get_service_name "$log_file")
            multitail_cmd="$multitail_cmd -l \"tail -f $log_file\" -L \"$service\""
        done
        
        # Execute multitail
        eval "$multitail_cmd"
    else
        # Custom solution using tail with process substitution
        echo -e "${YELLOW}📝 Using custom log aggregation (install 'multitail' for enhanced experience)${NC}"
        echo ""
        
        # Create named pipes for each log file and start background tail processes
        local pids=()
        local temp_dir=$(mktemp -d)
        
        # Cleanup function
        cleanup() {
            echo -e "\n${YELLOW}🛑 Stopping log monitoring...${NC}"
            for pid in "${pids[@]}"; do
                kill "$pid" 2>/dev/null || true
            done
            rm -rf "$temp_dir"
            exit 0
        }
        
        # Set up signal handlers
        trap cleanup SIGINT SIGTERM
        
        # Start tail processes for each log file
        for log_file in "${existing_files[@]}"; do
            local service=$(get_service_name "$log_file")
            
            # Start tail process that formats each line
            (
                tail -f "$log_file" 2>/dev/null | while IFS= read -r line; do
                    format_log_line "$service" "$line"
                done
            ) &
            
            pids+=($!)
        done
        
        # Also monitor for new log files
        (
            while true; do
                sleep 10
                for log_file in "${log_files[@]}"; do
                    if check_log_file "$log_file"; then
                        # Check if we're already monitoring this file
                        local already_monitoring=false
                        for existing_file in "${existing_files[@]}"; do
                            if [[ "$existing_file" == "$log_file" ]]; then
                                already_monitoring=true
                                break
                            fi
                        done
                        
                        if [ "$already_monitoring" = false ]; then
                            echo -e "${GREEN}🆕 New log file detected: $(basename "$log_file")${NC}"
                            existing_files+=("$log_file")
                            
                            local service=$(get_service_name "$log_file")
                            (
                                tail -f "$log_file" 2>/dev/null | while IFS= read -r line; do
                                    format_log_line "$service" "$line"
                                done
                            ) &
                            pids+=($!)
                        fi
                    fi
                done
            done
        ) &
        pids+=($!)
        
        # Wait for all background processes
        wait
    fi
}

# Main execution
main() {
    clear
    print_header
    show_log_status
    start_log_monitoring
}

# Run main function
main "$@"