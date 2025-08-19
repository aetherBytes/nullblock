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
        "LLM-SERVICE") echo "${BRIGHT_YELLOW}" ;;
        "LLM-FACTORY") echo "${YELLOW}" ;;
        "LM-STUDIO") echo "${MAGENTA}" ;;
        "LLM-STREAM") echo "${BRIGHT_MAGENTA}" ;;
        "LLM-MONITOR") echo "${CYAN}" ;;
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
        "llm-service")
            echo "LLM-SERVICE"
            ;;
        "llm-factory")
            echo "LLM-FACTORY"
            ;;
        "main")
            # LM Studio log file
            echo "LM-STUDIO"
            ;;
        "lm-studio-stream")
            # LM Studio stream capture log
            echo "LLM-STREAM"
            ;;
        "lm-studio-monitor")
            # LM Studio monitor output log
            echo "LLM-MONITOR"
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
    local services=("MCP" "ORCHESTRATION" "EREBUS" "HECATE-AGENT" "HECATE-SERVER" "HECATE-STARTUP" "FRONTEND" "IPFS" "AGENTS" "LLM-SERVICE" "LLM-FACTORY" "LM-STUDIO" "LLM-STREAM" "LLM-MONITOR" "JUST-CMD" "INSTALL" "UPDATE")
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
            "LLM-SERVICE") echo " - LLM Service Input/Output Logs" ;;
            "LLM-FACTORY") echo " - LLM Factory & Model Selection" ;;
            "LM-STUDIO") echo " - Local LM Studio Server Logs" ;;
            "LLM-STREAM") echo " - LM Studio Stream Capture (Input)" ;;
            "LLM-MONITOR") echo " - LM Studio Monitor Output (Input/Output)" ;;
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

# Function to show startup message
show_startup_message() {
    echo -e "${CYAN}🔄 Initializing log stream monitoring...${NC}"
    echo -e "${GRAY}Monitoring all Nullblock service logs in real-time${NC}"
    echo ""
}

# Function to start log monitoring with recent entries
start_log_monitoring() {
    # All potential log files to monitor (verified actual files)
    local log_files=(
        # Main logs directory (verified existing files)
        "/Users/sage/nullblock/logs/mcp.log"
        "/Users/sage/nullblock/logs/orchestration.log"
        "/Users/sage/nullblock/logs/erebus.log"
        "/Users/sage/nullblock/logs/erebus-temp.log"
        "/Users/sage/nullblock/logs/frontend.log"
        "/Users/sage/nullblock/logs/ipfs.log"
        "/Users/sage/nullblock/logs/mcp-install.log"
        "/Users/sage/nullblock/logs/orchestration-install.log"
        "/Users/sage/nullblock/logs/erebus-update.log"
        "/Users/sage/nullblock/logs/hecate-agent-install.log"
        "/Users/sage/nullblock/logs/hecate-agent.log"
        "/Users/sage/nullblock/logs/frontend-install.log"
        "/Users/sage/nullblock/logs/just-commands.log"
        
        # Agent logs (verified existing files)
        "/Users/sage/nullblock/svc/nullblock-agents/logs/hecate.log"
        "/Users/sage/nullblock/svc/nullblock-agents/logs/hecate-server.log"
        "/Users/sage/nullblock/svc/nullblock-agents/logs/hecate-startup.log"
        "/Users/sage/nullblock/svc/nullblock-agents/logs/agents.log"
        "/Users/sage/nullblock/svc/nullblock-agents/logs/llm-service.log"
        "/Users/sage/nullblock/svc/nullblock-agents/logs/llm-factory.log"
        
        # MCP service logs (verified existing files)
        "/Users/sage/nullblock/svc/nullblock-mcp/logs/mcp-server.log"
        
        # Orchestration service logs (verified existing files)
        "/Users/sage/nullblock/svc/nullblock-orchestration/logs/orchestration.log"
        
        # LLM Service logs (input/output tracking)
        "/Users/sage/Library/Logs/LM Studio/main.log"
        "/Users/sage/nullblock/logs/lm-studio-stream.log"
        "/Users/sage/nullblock/logs/lm-studio-monitor.log"
    )
    
    # Also dynamically find any dated log files
    local dated_files=(
        $(find "/Users/sage/nullblock/svc/erebus/logs/" -name "*.log.*" -type f 2>/dev/null)
    )
    
    # Combine static and dynamic file lists
    log_files+=("${dated_files[@]}")
    
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
        
        # Show initial recent entries from each log file
        echo -e "${GRAY}📖 Showing recent entries and starting live monitoring...${NC}"
        echo ""
        
        echo -e "${GREEN}🟢 Starting live monitoring for ${#existing_files[@]} log files${NC}"
        echo ""
        
        # Use a much simpler approach - just use tail with multiple files
        echo -e "${CYAN}📊 Monitoring files:${NC}"
        for log_file in "${existing_files[@]}"; do
            echo "   $(get_service_name "$log_file"): $(basename "$log_file")"
        done
        echo ""
        
        # Simple multitail approach - combine all files
        if [ ${#existing_files[@]} -gt 0 ]; then
            # Build a single tail command for all files
            exec tail -f "${existing_files[@]}" | while IFS= read -r line; do
                # Try to determine which service this line is from
                # This is a simplified approach but will work
                printf "%s %s\n" "$(date '+%H:%M:%S')" "$line"
            done
        fi
    fi
}

# Main execution
main() {
    clear
    print_header
    show_startup_message
    start_log_monitoring
}

# Run main function
main "$@"