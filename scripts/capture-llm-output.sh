#!/bin/bash

# LLM Output Capture
# Monitors LM Studio API responses to capture actual output content

LOG_FILE="$HOME/nullblock/logs/llm-output.log"
mkdir -p "$(dirname "$LOG_FILE")"

echo "LLM Output Capture Started" | tee -a "$LOG_FILE"
echo "============================" | tee -a "$LOG_FILE"
echo "$(date): Monitoring LM Studio API responses..." | tee -a "$LOG_FILE"
echo "" | tee -a "$LOG_FILE"

# Function to monitor HTTP responses
monitor_http_responses() {
    echo "Starting HTTP response monitoring..." | tee -a "$LOG_FILE"
    
    # Use netcat to monitor the LM Studio API port
    while true; do
        local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
        
        # Check for active connections
        local connections=$(lsof -i :1234 | grep ESTABLISHED | wc -l | tr -d ' ')
        
        if [ "$connections" -gt 0 ]; then
            echo "[$timestamp] Active API connections: $connections" | tee -a "$LOG_FILE"
            
            # Try to capture API responses using tcpdump (if available)
            if command -v tcpdump >/dev/null 2>&1; then
                # Capture HTTP responses for 3 seconds
                timeout 3 sudo tcpdump -i lo0 -A -s 0 'port 1234 and tcp[((tcp[12:1] & 0xf0) >> 2):4] = 0x48545450' 2>/dev/null | \
                grep -A 20 "HTTP/1.1 200" | \
                grep -E "(text|content|choices)" | \
                head -10 | \
                while read -r line; do
                    echo "[$timestamp] Response: $line" | tee -a "$LOG_FILE"
                done
            else
                # Fallback: Test the API and capture response
                local test_response=$(curl -s --max-time 2 "http://localhost:1234/v1/models" 2>/dev/null)
                if [ $? -eq 0 ] && [ -n "$test_response" ]; then
                    echo "[$timestamp] API responsive - models available" | tee -a "$LOG_FILE"
                fi
            fi
        fi
        
        sleep 5
    done
}

# Function to parse agent logs for successful responses
parse_agent_responses() {
    echo "Monitoring agent logs for response metrics..." | tee -a "$LOG_FILE"
    
    local hecate_log="/Users/sage/nullblock/svc/nullblock-agents/logs/hecate.log"
    local hecate_server_log="/Users/sage/nullblock/svc/nullblock-agents/logs/hecate-server.log"
    
    if [ -f "$hecate_log" ] && [ -f "$hecate_server_log" ]; then
        # Monitor both logs for response indicators
        tail -f "$hecate_log" "$hecate_server_log" 2>/dev/null | \
        grep -E "(Model:|SUCCESS|completion|response|Confidence|Tokens)" | \
        while IFS= read -r line; do
            local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
            
            # Extract useful information from the line
            if [[ "$line" =~ "Model:".*"Cost:" ]]; then
                echo "[$timestamp] Model Usage: $line" | tee -a "$LOG_FILE"
            elif [[ "$line" =~ "request completed".*"SUCCESS" ]]; then
                local timing=$(echo "$line" | grep -o '[0-9]*ms')
                echo "[$timestamp] ✅ Request completed in $timing" | tee -a "$LOG_FILE"
            elif [[ "$line" =~ "Confidence:".*"Tokens:" ]]; then
                echo "[$timestamp] Metrics: $line" | tee -a "$LOG_FILE"
            elif [[ "$line" =~ "Chat response sent successfully" ]]; then
                echo "[$timestamp] ✅ Response delivered to user" | tee -a "$LOG_FILE"
            fi
        done &
        
        echo "Agent response monitoring started" | tee -a "$LOG_FILE"
    else
        echo "Warning: Agent log files not found" | tee -a "$LOG_FILE"
    fi
}

# Main execution
echo "Starting LLM output monitoring..." | tee -a "$LOG_FILE"

# Start agent response parsing in background
parse_agent_responses &
parse_pid=$!

# Start HTTP response monitoring
monitor_http_responses &
monitor_pid=$!

# Handle cleanup
cleanup() {
    echo "" | tee -a "$LOG_FILE"
    echo "Stopping LLM output capture..." | tee -a "$LOG_FILE"
    kill $parse_pid $monitor_pid 2>/dev/null || true
    exit 0
}

trap cleanup SIGINT SIGTERM

# Wait for processes
wait $monitor_pid