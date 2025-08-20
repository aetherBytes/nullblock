#!/bin/bash

# Better LLM Monitor
# Properly captures LM Studio and agent interactions

set -e

LOG_FILE="$HOME/nullblock/logs/llm-monitor.log"
mkdir -p "$(dirname "$LOG_FILE")"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}🤖 Better LLM Monitor Started${NC}" | tee -a "$LOG_FILE"
echo "============================" | tee -a "$LOG_FILE" 
echo "$(date): LLM input/output monitoring with proper process management" | tee -a "$LOG_FILE"
echo "" | tee -a "$LOG_FILE"

# Check if LM Studio is running
check_lm_studio() {
    if curl -s --max-time 2 http://localhost:1234/v1/models >/dev/null 2>&1; then
        local models=$(curl -s --max-time 2 http://localhost:1234/v1/models | jq -r '.data | length' 2>/dev/null || echo "?")
        echo -e "${GREEN}✅ LM Studio: Online ($models models)${NC}" | tee -a "$LOG_FILE"
        return 0
    else
        echo -e "${RED}❌ LM Studio: Offline${NC}" | tee -a "$LOG_FILE"
        return 1
    fi
}

# Function to monitor LM Studio logs directly
monitor_lm_studio_logs() {
    local lm_log_dir="$HOME/Library/Logs/LM Studio"
    local latest_log=$(find "$lm_log_dir" -name "*.log" -type f -exec ls -t {} + 2>/dev/null | head -1)
    
    if [[ -f "$latest_log" ]]; then
        echo -e "${BLUE}📥 Monitoring LM Studio log: $(basename "$latest_log")${NC}" | tee -a "$LOG_FILE"
        
        tail -f "$latest_log" 2>/dev/null | while IFS= read -r line; do
            local timestamp=$(date '+%H:%M:%S')
            
            # Filter for important events
            if [[ "$line" =~ "POST /v1/chat/completions" ]] || \
               [[ "$line" =~ "POST /v1/completions" ]] || \
               [[ "$line" =~ "model.*gemma" ]] || \
               [[ "$line" =~ "tokens" ]]; then
                echo "[$timestamp] LM-STUDIO: $line" | tee -a "$LOG_FILE"
            fi
        done &
        
        echo $! > /tmp/lm_studio_monitor.pid
    else
        echo -e "${YELLOW}⚠️  No LM Studio logs found${NC}" | tee -a "$LOG_FILE"
    fi
}

# Function to monitor agent logs
monitor_agent_logs() {
    local hecate_log="/Users/sage/nullblock/svc/nullblock-agents/logs/hecate.log"
    local hecate_server_log="/Users/sage/nullblock/svc/nullblock-agents/logs/hecate-server.log"
    
    echo -e "${GREEN}📤 Monitoring agent logs...${NC}" | tee -a "$LOG_FILE"
    
    # Monitor both agent logs
    for log_file in "$hecate_log" "$hecate_server_log"; do
        if [[ -f "$log_file" ]]; then
            tail -f "$log_file" 2>/dev/null | \
            sed 's/\x1b\[[0-9;]*m//g' | \
            while IFS= read -r line; do
                local timestamp=$(date '+%H:%M:%S')
                local service=$(basename "$log_file" .log | tr '[:lower:]' '[:upper:]')
                
                # Filter for important events
                if [[ "$line" =~ "request completed" ]] || \
                   [[ "$line" =~ "Model:" ]] || \
                   [[ "$line" =~ "SUCCESS" ]] || \
                   [[ "$line" =~ "Chat response" ]] || \
                   [[ "$line" =~ "LLM response" ]]; then
                    echo "[$timestamp] $service: $line" | tee -a "$LOG_FILE"
                fi
            done &
            
            echo $! >> /tmp/agent_monitor.pids
        fi
    done
}

# Function to show periodic status
show_periodic_status() {
    while true; do
        sleep 30
        local timestamp=$(date '+%H:%M:%S')
        
        # Check connections
        local connections=$(lsof -i :1234 2>/dev/null | grep ESTABLISHED | wc -l | tr -d ' ')
        
        # Check service health
        local hecate_status="Offline"
        if curl -s --max-time 2 http://localhost:9002/health >/dev/null 2>&1; then
            hecate_status="Online"
        fi
        
        echo "[$timestamp] STATUS: LM Studio connections=$connections, Hecate=$hecate_status" | tee -a "$LOG_FILE"
    done &
    
    echo $! > /tmp/status_monitor.pid
}

# Cleanup function
cleanup() {
    echo "" | tee -a "$LOG_FILE"
    echo -e "${YELLOW}🛑 Stopping LLM monitor...${NC}" | tee -a "$LOG_FILE"
    
    # Kill background processes
    [[ -f /tmp/lm_studio_monitor.pid ]] && kill $(cat /tmp/lm_studio_monitor.pid) 2>/dev/null || true
    [[ -f /tmp/status_monitor.pid ]] && kill $(cat /tmp/status_monitor.pid) 2>/dev/null || true
    [[ -f /tmp/agent_monitor.pids ]] && {
        while read -r pid; do
            kill "$pid" 2>/dev/null || true
        done < /tmp/agent_monitor.pids
    }
    
    # Clean up pid files
    rm -f /tmp/lm_studio_monitor.pid /tmp/status_monitor.pid /tmp/agent_monitor.pids
    
    exit 0
}

# Set up signal handlers
trap cleanup SIGINT SIGTERM

# Initialize pid files
> /tmp/agent_monitor.pids

# Main execution
echo -e "${CYAN}🔄 Starting LLM monitoring components...${NC}"

# Check initial status
check_lm_studio

# Start monitoring components
monitor_lm_studio_logs
monitor_agent_logs  
show_periodic_status

echo -e "${GREEN}✅ All monitoring started. Press Ctrl+C to stop.${NC}" | tee -a "$LOG_FILE"
echo "" | tee -a "$LOG_FILE"

# Keep script running
while true; do
    sleep 5
    
    # Check if background processes are still running
    if [[ -f /tmp/lm_studio_monitor.pid ]] && ! kill -0 $(cat /tmp/lm_studio_monitor.pid) 2>/dev/null; then
        echo -e "${YELLOW}⚠️  LM Studio monitor died, restarting...${NC}" | tee -a "$LOG_FILE"
        monitor_lm_studio_logs
    fi
done