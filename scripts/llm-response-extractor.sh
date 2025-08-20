#!/bin/bash

# LLM Response Extractor
# Monitors agent logs to extract and log actual LLM responses

echo "🧠 LLM Response Extractor"
echo "=========================="

# Create logs directory and output file
mkdir -p ~/nullblock/logs
local response_log="$HOME/nullblock/logs/llm-responses.log"

echo "🚀 Starting LLM response extraction..." | tee -a "$response_log"
echo "📁 Response log file: $response_log" | tee -a "$response_log"
echo "" | tee -a "$response_log"

# Function to extract responses from Hecate server logs
extract_hecate_responses() {
    local hecate_server_log="/Users/sage/nullblock/svc/nullblock-agents/logs/hecate-server.log"
    
    if [ ! -f "$hecate_server_log" ]; then
        echo "❌ Hecate server log not found: $hecate_server_log" | tee -a "$response_log"
        return 1
    fi
    
    echo "📄 Monitoring Hecate server log for responses..." | tee -a "$response_log"
    
    # Monitor the log file for new entries
    tail -f "$hecate_server_log" | while IFS= read -r line; do
        local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
        
        # Look for successful chat responses
        if [[ "$line" =~ "Chat response sent successfully" ]]; then
            echo "[$timestamp] ✅ LLM Response Sent" | tee -a "$response_log"
        fi
        
        # Look for response timing
        if [[ "$line" =~ "POST /chat" ]] && [[ "$line" =~ "s)" ]]; then
            local timing=$(echo "$line" | grep -o '([0-9.]*s)')
            echo "[$timestamp] ⏱️  Response Time: $timing" | tee -a "$response_log"
        fi
        
        # Look for model information
        if [[ "$line" =~ "Model:" ]]; then
            echo "[$timestamp] 🤖 $line" | tee -a "$response_log"
        fi
    done
}

# Function to extract detailed responses from HTTP requests
monitor_http_responses() {
    echo "🌐 Starting HTTP response monitoring..." | tee -a "$response_log"
    
    # Monitor network connections to Hecate API
    while true; do
        local connections=$(lsof -i :9002 -P 2>/dev/null | grep ESTABLISHED | wc -l | tr -d ' ')
        
        if [ "$connections" -gt 0 ]; then
            local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
            echo "[$timestamp] 🔗 Active connections to Hecate API: $connections" | tee -a "$response_log"
            
            # Test API to see if it's responding
            local api_test=$(curl -s --max-time 1 -X GET http://localhost:9002/health 2>/dev/null)
            if [ $? -eq 0 ]; then
                echo "[$timestamp] ✅ Hecate API healthy" | tee -a "$response_log"
            fi
        fi
        
        sleep 5
    done
}

# Function to capture actual response content using curl log analysis
capture_response_content() {
    echo "📊 Capturing response content..." | tee -a "$response_log"
    
    # Monitor curl requests to capture actual responses
    # This works by monitoring the agent logs for response patterns
    local hecate_log="/Users/sage/nullblock/svc/nullblock-agents/logs/hecate.log"
    
    if [ -f "$hecate_log" ]; then
        tail -f "$hecate_log" | while IFS= read -r line; do
            local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
            
            # Look for request starts
            if [[ "$line" =~ "chat request started" ]]; then
                local user=$(echo "$line" | grep -o '0x[a-fA-F0-9]*\.\.\.')
                echo "[$timestamp] 📥 REQUEST from $user" | tee -a "$response_log"
            fi
            
            # Look for successful completions with timing
            if [[ "$line" =~ "request completed.*SUCCESS.*ms" ]]; then
                local timing=$(echo "$line" | grep -o '[0-9]*ms')
                echo "[$timestamp] 📤 RESPONSE completed in $timing" | tee -a "$response_log"
            fi
            
            # Look for model usage
            if [[ "$line" =~ "Model:.*gemma.*Cost:" ]]; then
                local model=$(echo "$line" | grep -o 'gemma-[^(]*')
                local cost=$(echo "$line" | grep -o '\$[0-9.]*')
                echo "[$timestamp] 🧠 MODEL: $model | COST: $cost" | tee -a "$response_log"
            fi
            
            # Look for confidence scores
            if [[ "$line" =~ "Confidence:.*Tokens:" ]]; then
                local confidence=$(echo "$line" | grep -o 'Confidence: [0-9.]*')
                local tokens=$(echo "$line" | grep -o 'Tokens: [0-9]*')
                echo "[$timestamp] 💯 $confidence | $tokens" | tee -a "$response_log"
            fi
        done
    fi
}

# Main function
main() {
    echo "🎯 Starting comprehensive LLM response monitoring..." | tee -a "$response_log"
    echo "" | tee -a "$response_log"
    
    # Start all monitoring functions in background
    extract_hecate_responses &
    local hecate_pid=$!
    
    capture_response_content &
    local content_pid=$!
    
    monitor_http_responses &
    local http_pid=$!
    
    # Handle cleanup
    cleanup() {
        echo "" | tee -a "$response_log"
        echo "🛑 Stopping LLM response monitoring..." | tee -a "$response_log"
        kill $hecate_pid $content_pid $http_pid 2>/dev/null || true
        exit 0
    }
    
    trap cleanup SIGINT SIGTERM
    
    # Wait for processes
    wait $hecate_pid
}

# Run main function
main