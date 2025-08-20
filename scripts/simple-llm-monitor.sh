#!/bin/bash

# Simple LLM Monitor
# Single process to monitor both LM Studio inputs and agent outputs
# Prevents duplicate processes and provides clean input/output tracking

LOG_FILE="$HOME/nullblock/logs/llm-monitor.log"
mkdir -p "$(dirname "$LOG_FILE")"

echo "Simple LLM Monitor Started" | tee -a "$LOG_FILE"
echo "============================" | tee -a "$LOG_FILE" 
echo "$(date): Single-process LLM input/output monitoring" | tee -a "$LOG_FILE"
echo "" | tee -a "$LOG_FILE"

# Function to capture LM Studio inputs (single stream)
capture_inputs() {
    echo "Starting INPUT capture (lms log stream)..." | tee -a "$LOG_FILE"
    
    # Single lms log stream process
    lms log stream 2>/dev/null | while IFS= read -r line; do
        local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
        
        # Only log relevant input information, not full prompts
        if [[ "$line" =~ "timestamp:" ]]; then
            echo "[$timestamp] LM Studio Input: $line" | tee -a "$LOG_FILE"
        elif [[ "$line" =~ "modelIdentifier:" ]]; then
            echo "[$timestamp] Model: $line" | tee -a "$LOG_FILE"
        fi
    done &
    
    echo "LM Studio input monitoring started" | tee -a "$LOG_FILE"
}

# Function to capture agent outputs
capture_outputs() {
    echo "Starting OUTPUT capture (agent logs)..." | tee -a "$LOG_FILE"
    
    local hecate_log="/Users/sage/nullblock/svc/nullblock-agents/logs/hecate.log"
    local hecate_server_log="/Users/sage/nullblock/svc/nullblock-agents/logs/hecate-server.log"
    
    if [ -f "$hecate_log" ] && [ -f "$hecate_server_log" ]; then
        # Monitor both logs for response indicators (strip ANSI codes)
        tail -f "$hecate_log" "$hecate_server_log" 2>/dev/null | \
        sed 's/\x1b\[[0-9;]*m//g' | \
        grep -E "(Model:|SUCCESS|completion|response|Confidence|Tokens)" | \
        while IFS= read -r line; do
            local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
            
            # Extract key information only
            if [[ "$line" =~ "Model:".*"gemma".*"Cost:" ]]; then
                local model=$(echo "$line" | grep -o 'gemma-[^(]*' | head -1)
                local cost=$(echo "$line" | grep -o '\$[0-9.]*' | head -1)
                echo "[$timestamp] OUTPUT: Model=$model Cost=$cost" | tee -a "$LOG_FILE"
            elif [[ "$line" =~ "request completed".*"SUCCESS" ]]; then
                local timing=$(echo "$line" | grep -o '[0-9]*ms' | head -1)
                echo "[$timestamp] OUTPUT: Completed in $timing" | tee -a "$LOG_FILE"
            elif [[ "$line" =~ "Confidence:".*"Tokens:" ]]; then
                local confidence=$(echo "$line" | grep -o 'Confidence: [0-9.]*' | head -1)
                local tokens=$(echo "$line" | grep -o 'Tokens: [0-9]*' | head -1)
                echo "[$timestamp] OUTPUT: $confidence $tokens" | tee -a "$LOG_FILE"
            elif [[ "$line" =~ "Chat response sent successfully" ]]; then
                echo "[$timestamp] OUTPUT: Response delivered to user" | tee -a "$LOG_FILE"
            fi
        done &
        
        echo "Agent output monitoring started" | tee -a "$LOG_FILE"
    else
        echo "Warning: Agent log files not found, output monitoring disabled" | tee -a "$LOG_FILE"
    fi
}

# Function to show periodic status
show_status() {
    while true; do
        sleep 30
        local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
        local connections=$(lsof -i :1234 2>/dev/null | grep ESTABLISHED | wc -l | tr -d ' ')
        echo "[$timestamp] STATUS: LM Studio connections=$connections" | tee -a "$LOG_FILE"
    done &
}

# Main execution
echo "Starting simple LLM monitoring..." | tee -a "$LOG_FILE"

# Start all monitoring in background
capture_inputs &
input_pid=$!

capture_outputs &
output_pid=$!

show_status &
status_pid=$!

# Handle cleanup
cleanup() {
    echo "" | tee -a "$LOG_FILE"
    echo "Stopping simple LLM monitor..." | tee -a "$LOG_FILE"
    kill $input_pid $output_pid $status_pid 2>/dev/null || true
    exit 0
}

trap cleanup SIGINT SIGTERM

echo "All monitoring processes started. Press Ctrl+C to stop." | tee -a "$LOG_FILE"
echo "" | tee -a "$LOG_FILE"

# Wait for processes
wait $input_pid