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
        "JUST-CMD") echo "${BRIGHT_YELLOW}" ;;
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
        "mcp-server"|"mcp"|"mcp-install")
            echo "MCP"
            ;;
        "mcp-server-errors")
            echo "MCP"
            ;;
        "mcp-server-access")
            echo "MCP"
            ;;
        "orchestration"|"orchestration-install") 
            echo "ORCHESTRATION"
            ;;
        "orchestration-errors")
            echo "ORCHESTRATION"
            ;;
        "orchestration-workflows")
            echo "ORCHESTRATION"
            ;;
        "erebus"|"erebus-update")
            echo "EREBUS"
            ;;
        "erebus-errors")
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
        "just-commands")
            echo "JUST-CMD"
            ;;
        *)
            echo "UNKNOWN"
            ;;
    esac
}


# Function to filter out noisy log lines
should_filter_line() {
    local line="$1"
    
    # Filter out excessive health check logs
    if [[ "$line" =~ "Health check requested" ]] || \
       [[ "$line" =~ "Health check successful" ]] || \
       [[ "$line" =~ "GET /health" ]] || \
       [[ "$line" =~ "📥 GET /health" ]] || \
       [[ "$line" =~ "📤 GET /health" ]] || \
       [[ "$line" =~ "🏥 Health check" ]]; then
        return 0  # Filter out (return true)
    fi
    
    # Filter out pip upgrade notices (too frequent)
    if [[ "$line" =~ "A new release of pip is available" ]] || \
       [[ "$line" =~ "To update, run: pip install --upgrade pip" ]]; then
        return 0  # Filter out
    fi
    
    # Filter out npm audit suggestions (not critical)
    if [[ "$line" =~ "npm audit fix" ]] || \
       [[ "$line" =~ "Run \`npm audit\` for details" ]]; then
        return 0  # Filter out
    fi
    
    return 1  # Don't filter (return false)
}

# Function to detect and format JSON in log lines
format_json_if_present() {
    local line="$1"
    
    # Try to detect JSON patterns (starts with { or [, ends with } or ])
    if [[ "$line" =~ ^\s*[\[{] ]] && [[ "$line" =~ [\]}]\s*$ ]]; then
        # Try to format as JSON, fall back to original if it fails
        echo "$line" | jq '.' 2>/dev/null && return 0
    fi
    
    # Not JSON or failed to parse, return original
    echo "$line"
    return 1
}

# Function to format log line with service tag and color
format_log_line() {
    local service="$1"
    local line="$2"
    local color=$(get_service_color "$service")
    local timestamp=$(date '+%H:%M:%S')
    
    # Skip noisy lines
    if should_filter_line "$line"; then
        return
    fi
    
    # Try to format JSON in the line
    local formatted_line
    formatted_line=$(format_json_if_present "$line")
    
    # Extract timestamp from log line if it exists
    if [[ "$line" =~ ^[0-9]{4}-[0-9]{2}-[0-9]{2}[[:space:]]+[0-9]{2}:[0-9]{2}:[0-9]{2} ]]; then
        # Line already has timestamp, use it
        printf "${color}[%-14s]${NC} %s\n" "$service" "$formatted_line"
    elif [[ "$line" =~ ^\[[0-9]{2}:[0-9]{2}:[0-9]{2}\] ]]; then
        # Line has short timestamp
        printf "${color}[%-14s]${NC} %s\n" "$service" "$formatted_line"
    else
        # No timestamp in line, add our own
        printf "${color}[%-14s]${NC} ${GRAY}${timestamp}${NC} %s\n" "$service" "$formatted_line"
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
    local services=("MCP" "ORCHESTRATION" "EREBUS" "HECATE-AGENT" "HECATE-SERVER" "HECATE-STARTUP" "FRONTEND" "IPFS" "AGENTS" "JUST-CMD" "INSTALL" "UPDATE")
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
            "JUST-CMD") echo " - Just Command Executions & Tests" ;;
            "INSTALL") echo " - Installation & Setup Logs" ;;
            "UPDATE") echo " - Update & Dependency Logs" ;;
            *) echo "" ;;
        esac
    done
    echo ""
    echo -e "${BRIGHT_CYAN}═══════════════════════════════════════════════════════════════════════════════════${NC}"
    echo ""
}

# Function to show initial log file status and recent entries
show_log_status() {
    echo -e "${WHITE}📋 Log File Status (showing last 15 minutes):${NC}"
    
    # Calculate timestamp for 15 minutes ago
    local since_time
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        since_time=$(date -v-15M '+%Y-%m-%d %H:%M:%S')
    else
        # Linux
        since_time=$(date -d '15 minutes ago' '+%Y-%m-%d %H:%M:%S')
    fi
    
    # Updated log file paths to include new services
    local main_logs=(
        "/Users/sage/nullblock/logs/mcp-server.log"
        "/Users/sage/nullblock/logs/orchestration.log" 
        "/Users/sage/nullblock/logs/erebus.log"
        "/Users/sage/nullblock/logs/frontend.log"
        "/Users/sage/nullblock/logs/ipfs.log"
    )
    
    # Updated agent logs directory paths
    local agent_logs=(
        "/Users/sage/nullblock/svc/nullblock-agents/logs/hecate.log"
        "/Users/sage/nullblock/svc/nullblock-agents/logs/hecate-server.log"
        "/Users/sage/nullblock/svc/nullblock-agents/logs/hecate-startup.log"
        "/Users/sage/nullblock/svc/nullblock-agents/logs/agents.log"
    )
    
    # Service-specific MCP logs
    local mcp_logs=(
        "/Users/sage/nullblock/svc/nullblock-mcp/logs/mcp-server.log"
        "/Users/sage/nullblock/svc/nullblock-mcp/logs/mcp-server-errors.log"
        "/Users/sage/nullblock/svc/nullblock-mcp/logs/mcp-server-access.log"
    )
    
    # Service-specific Orchestration logs
    local orchestration_logs=(
        "/Users/sage/nullblock/svc/nullblock-orchestration/logs/orchestration.log"
        "/Users/sage/nullblock/svc/nullblock-orchestration/logs/orchestration-errors.log"
        "/Users/sage/nullblock/svc/nullblock-orchestration/logs/orchestration-workflows.log"
    )
    
    # Service-specific Erebus logs
    local erebus_logs=(
        "/Users/sage/nullblock/svc/erebus/logs/erebus.log"
        "/Users/sage/nullblock/svc/erebus/logs/erebus-errors.log"
    )
    
    for log_file in "${main_logs[@]}" "${agent_logs[@]}" "${mcp_logs[@]}" "${orchestration_logs[@]}" "${erebus_logs[@]}"; do
        local service=$(get_service_name "$log_file")
        local color=$(get_service_color "$service")
        
        if check_log_file "$log_file"; then
            local size=$(du -h "$log_file" 2>/dev/null | cut -f1)
            local lines=$(wc -l < "$log_file" 2>/dev/null || echo "0")
            local recent_lines
            
            # Get recent lines (last 15 minutes)
            if [[ "$OSTYPE" == "darwin"* ]]; then
                # macOS: use a simpler approach with tail
                recent_lines=$(tail -100 "$log_file" 2>/dev/null | wc -l | tr -d ' ')
            else
                # Linux: can use more sophisticated time filtering
                recent_lines=$(awk -v since="$since_time" '$0 >= since' "$log_file" 2>/dev/null | wc -l | tr -d ' ')
            fi
            
            echo -e "  ${color}✓${NC} $(basename "$log_file") (${size}, ${lines} total, ${recent_lines} recent)"
        else
            echo -e "  ${GRAY}✗${NC} $(basename "$log_file") ${GRAY}(not found)${NC}"
        fi
    done
    echo ""
    echo -e "${CYAN}⏱️  Showing entries from the last 15 minutes by default${NC}"
    echo -e "${GRAY}   Log files are automatically rotated and archived for persistence${NC}"
    echo ""
}

# Function to start log monitoring with recent entries
start_log_monitoring() {
    # All potential log files to monitor (updated paths)
    local log_files=(
        # Main logs directory
        "/Users/sage/nullblock/logs/mcp-server.log"
        "/Users/sage/nullblock/logs/orchestration.log"
        "/Users/sage/nullblock/logs/erebus.log"
        "/Users/sage/nullblock/logs/frontend.log"
        "/Users/sage/nullblock/logs/ipfs.log"
        "/Users/sage/nullblock/logs/mcp-install.log"
        "/Users/sage/nullblock/logs/orchestration-install.log"
        "/Users/sage/nullblock/logs/erebus-update.log"
        "/Users/sage/nullblock/logs/hecate-agent-install.log"
        "/Users/sage/nullblock/logs/frontend-install.log"
        "/Users/sage/nullblock/logs/just-commands.log"
        
        # Agent logs
        "/Users/sage/nullblock/svc/nullblock-agents/logs/hecate.log"
        "/Users/sage/nullblock/svc/nullblock-agents/logs/hecate-server.log"
        "/Users/sage/nullblock/svc/nullblock-agents/logs/hecate-startup.log"
        "/Users/sage/nullblock/svc/nullblock-agents/logs/agents.log"
        
        # MCP service logs
        "/Users/sage/nullblock/svc/nullblock-mcp/logs/mcp-server.log"
        "/Users/sage/nullblock/svc/nullblock-mcp/logs/mcp-server-errors.log"
        "/Users/sage/nullblock/svc/nullblock-mcp/logs/mcp-server-access.log"
        
        # Orchestration service logs
        "/Users/sage/nullblock/svc/nullblock-orchestration/logs/orchestration.log"
        "/Users/sage/nullblock/svc/nullblock-orchestration/logs/orchestration-errors.log"
        "/Users/sage/nullblock/svc/nullblock-orchestration/logs/orchestration-workflows.log"
        
        # Erebus service logs
        "/Users/sage/nullblock/svc/erebus/logs/erebus.log"
        "/Users/sage/nullblock/svc/erebus/logs/erebus-errors.log"
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
            
            # Start tail process that formats each line with better real-time options
            (
                tail -F -n 10 "$log_file" 2>/dev/null | while IFS= read -r line; do
                    # Skip empty lines
                    if [[ -n "$line" ]]; then
                        format_log_line "$service" "$line"
                    fi
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
                                tail -F -n 10 "$log_file" 2>/dev/null | while IFS= read -r line; do
                                    if [[ -n "$line" ]]; then
                                        format_log_line "$service" "$line"
                                    fi
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