#!/bin/bash

# LM Studio API Monitor Script
# This script monitors actual API requests and responses to capture both input and output

echo "LM Studio API Monitor"
echo "====================="

# Create logs directory and monitoring output file
mkdir -p ~/nullblock/logs
monitor_log="$HOME/nullblock/logs/lm-studio-api-monitor.log"

echo "Starting LM Studio API monitoring..." | tee -a "$monitor_log"
echo "API monitor log file: $monitor_log" | tee -a "$monitor_log"
echo "" | tee -a "$monitor_log"

# Function to monitor API traffic using curl interceptor
monitor_api_traffic() {
    echo "Monitoring LM Studio API traffic on localhost:1234" | tee -a "$monitor_log"
    echo "Capturing both requests and responses..." | tee -a "$monitor_log"
    echo "" | tee -a "$monitor_log"
    
    # Monitor network traffic to LM Studio API
    while true; do
        # Check if there are active connections to LM Studio
        local connections=$(lsof -i :1234 -P 2>/dev/null | grep ESTABLISHED | wc -l | tr -d ' ')
        
        if [ "$connections" -gt 0 ]; then
            local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
            echo "[$timestamp] Active connections to LM Studio: $connections" | tee -a "$monitor_log"
            
            # Try to capture recent API activity
            if command -v tcpdump >/dev/null 2>&1; then
                # Use tcpdump if available (requires sudo)
                timeout 5 sudo tcpdump -i lo0 -A -s 0 port 1234 2>/dev/null | \
                grep -E "(POST|completion|content)" | head -5 | \
                while read -r line; do
                    echo "[$timestamp] $line" | tee -a "$monitor_log"
                done
            else
                # Fallback: Monitor using netstat and API probes
                echo "[$timestamp] API Activity detected" | tee -a "$monitor_log"
                
                # Test API health and log response
                local health_response=$(curl -s --max-time 2 http://localhost:1234/v1/models 2>/dev/null)
                if [ $? -eq 0 ] && [ -n "$health_response" ]; then
                    local model_count=$(echo "$health_response" | jq -r '.data | length' 2>/dev/null || echo "unknown")
                    echo "[$timestamp] ✅ API healthy, $model_count models loaded" | tee -a "$monitor_log"
                fi
            fi
        fi
        
        sleep 2
    done
}

# Function to enhance with response capture
capture_responses() {
    echo "Enhanced response capture starting..." | tee -a "$monitor_log"
    
    # Monitor the actual LLM service logs for responses
    local hecate_log="/Users/sage/nullblock/svc/nullblock-agents/logs/hecate.log"
    local hecate_server_log="/Users/sage/nullblock/svc/nullblock-agents/logs/hecate-server.log"
    
    if [ -f "$hecate_log" ] && [ -f "$hecate_server_log" ]; then
        echo "Monitoring agent logs for LLM responses..." | tee -a "$monitor_log"
        
        # Tail both logs and extract LLM-related entries
        tail -f "$hecate_log" "$hecate_server_log" 2>/dev/null | \
        grep -E "(Model:|SUCCESS|response|completion)" | \
        while read -r line; do
            local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
            echo "[$timestamp] LLM: $line" | tee -a "$monitor_log"
        done &
        
        local tail_pid=$!
        echo "Response monitoring started (PID: $tail_pid)" | tee -a "$monitor_log"
    else
        echo "⚠️  Agent log files not found for response monitoring" | tee -a "$monitor_log"
    fi
}

# Main monitoring function
main() {
    echo "Starting comprehensive LM Studio monitoring..." | tee -a "$monitor_log"
    echo "" | tee -a "$monitor_log"
    
    # Start response capture in background
    capture_responses &
    local response_pid=$!
    
    # Start API traffic monitoring
    monitor_api_traffic &
    local traffic_pid=$!
    
    # Handle cleanup
    cleanup() {
        echo "" | tee -a "$monitor_log"
        echo "Stopping LM Studio API monitoring..." | tee -a "$monitor_log"
        kill $response_pid $traffic_pid 2>/dev/null || true
        exit 0
    }
    
    trap cleanup SIGINT SIGTERM
    
    # Wait for processes
    wait $traffic_pid
}

# Run main function
main