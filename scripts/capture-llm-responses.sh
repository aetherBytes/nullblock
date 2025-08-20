#!/bin/bash

# Simple LLM Response Capture
# Monitors agent logs and extracts actual response content

LOG_FILE="$HOME/nullblock/logs/llm-responses.log"
mkdir -p "$(dirname "$LOG_FILE")"

echo "LLM Response Capture Started" | tee -a "$LOG_FILE"
echo "=============================" | tee -a "$LOG_FILE"
echo "$(date): Monitoring for LLM responses..." | tee -a "$LOG_FILE"
echo "" | tee -a "$LOG_FILE"

# Function to monitor for actual API responses (no automatic testing)
monitor_api_responses() {
    echo "Monitoring for natural API responses..." | tee -a "$LOG_FILE"
    
    # Just monitor agent logs for response patterns without making test calls
    local hecate_log="/Users/sage/nullblock/svc/nullblock-agents/logs/hecate.log"
    
    if [ -f "$hecate_log" ]; then
        # Track the last position in the log to detect new entries
        local last_line_count=$(wc -l < "$hecate_log" 2>/dev/null || echo "0")
        
        while true; do
            sleep 2  # Check every 2 seconds instead of making requests
            
            local current_line_count=$(wc -l < "$hecate_log" 2>/dev/null || echo "0")
            
            # If new lines have been added, process them
            if [ "$current_line_count" -gt "$last_line_count" ]; then
                local new_lines=$((current_line_count - last_line_count))
                local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
                
                # Get the new lines and look for response completions
                tail -n "$new_lines" "$hecate_log" | while IFS= read -r line; do
                    if [[ "$line" =~ "request completed.*SUCCESS" ]]; then
                        local timing=$(echo "$line" | grep -o '[0-9]*ms')
                        echo "[$timestamp] Response completed in $timing" | tee -a "$LOG_FILE"
                    elif [[ "$line" =~ "Model:.*gemma.*Cost:" ]]; then
                        local model=$(echo "$line" | grep -o 'gemma-[^(]*')
                        local cost=$(echo "$line" | grep -o '\$[0-9.]*')
                        echo "[$timestamp] Model: $model | Cost: $cost" | tee -a "$LOG_FILE"
                    elif [[ "$line" =~ "Confidence:.*Tokens:" ]]; then
                        local confidence=$(echo "$line" | grep -o 'Confidence: [0-9.]*')
                        local tokens=$(echo "$line" | grep -o 'Tokens: [0-9]*')
                        echo "[$timestamp] $confidence | $tokens" | tee -a "$LOG_FILE"
                    fi
                done
                
                last_line_count=$current_line_count
            fi
        done
    fi
}

# Function to parse existing logs for response content
parse_existing_responses() {
    local hecate_log="/Users/sage/nullblock/svc/nullblock-agents/logs/hecate.log"
    local hecate_server_log="/Users/sage/nullblock/svc/nullblock-agents/logs/hecate-server.log"
    
    echo "Parsing existing agent logs for response patterns..." | tee -a "$LOG_FILE"
    
    # Monitor both logs for new entries
    tail -f "$hecate_log" "$hecate_server_log" 2>/dev/null | while IFS= read -r line; do
        local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
        
        # Extract relevant information
        if [[ "$line" =~ "Model:.*gemma.*Cost:" ]]; then
            echo "[$timestamp] $line" | tee -a "$LOG_FILE"
        elif [[ "$line" =~ "request completed.*SUCCESS" ]]; then
            echo "[$timestamp] $line" | tee -a "$LOG_FILE"
        elif [[ "$line" =~ "Confidence:" ]]; then
            echo "[$timestamp] $line" | tee -a "$LOG_FILE"
        elif [[ "$line" =~ "Chat response sent successfully" ]]; then
            echo "[$timestamp] Response sent to user" | tee -a "$LOG_FILE"
        fi
    done
}

# Main execution
echo "Starting LLM response monitoring..." | tee -a "$LOG_FILE"

# Start monitoring functions (no automatic testing)
parse_existing_responses &
local parse_pid=$!

monitor_api_responses &
local monitor_pid=$!

# Handle cleanup
cleanup() {
    echo "" | tee -a "$LOG_FILE"
    echo "Stopping LLM response capture..." | tee -a "$LOG_FILE"
    kill $parse_pid $monitor_pid 2>/dev/null || true
    exit 0
}

trap cleanup SIGINT SIGTERM

# Wait for processes
wait $parse_pid