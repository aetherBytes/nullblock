#!/bin/bash

# LM Studio Monitoring Script
# This script monitors LM Studio logs and API status

echo "🔍 LM Studio Monitoring Script"
echo "=============================="

# Function to check if directory exists and has log files
check_log_directory() {
    local dir="$1"
    local name="$2"
    
    if [ -d "$dir" ]; then
        echo "✅ Found $name directory: $dir"
        local log_files=$(find "$dir" -name "*.log" -type f 2>/dev/null | head -5)
        if [ -n "$log_files" ]; then
            echo "📄 Log files found:"
            echo "$log_files" | while read -r file; do
                echo "   - $(basename "$file") ($(stat -f%z "$file" 2>/dev/null || echo "unknown") bytes)"
            done
            return 0
        else
            echo "⚠️  No .log files found in $name directory"
            return 1
        fi
    else
        echo "❌ $name directory not found: $dir"
        return 1
    fi
}

# Function to monitor log files
monitor_logs() {
    local dir="$1"
    local name="$2"
    
    echo "📊 Monitoring $name logs..."
    local log_files=$(find "$dir" -name "*.log" -type f -newer /tmp/lmstudio_monitor_start 2>/dev/null | head -3)
    
    if [ -n "$log_files" ]; then
        echo "🔄 Following log files:"
        echo "$log_files" | while read -r file; do
            echo "   📄 $(basename "$file")"
        done
        
        # Use tail to follow multiple log files
        tail -f $log_files 2>/dev/null &
        local tail_pid=$!
        
        # Wait for user interrupt
        echo "Press Ctrl+C to stop monitoring..."
        wait $tail_pid
    else
        echo "⚠️  No recent log files found in $name"
        return 1
    fi
}

# Function to check API status
check_api_status() {
    echo "🌐 Checking LM Studio API status..."
    
    # Check if API is responding
    local api_response=$(curl -s http://localhost:1234/v1/models 2>/dev/null)
    
    if [ $? -eq 0 ] && [ -n "$api_response" ]; then
        echo "✅ LM Studio API is responding"
        echo "📋 Available models:"
        echo "$api_response" | jq -r '.data[].id' 2>/dev/null || echo "   (Could not parse model list)"
        
        # Check server info
        local server_info=$(curl -s http://localhost:1234/v1/models 2>/dev/null | jq '.data[0]' 2>/dev/null)
        if [ -n "$server_info" ]; then
            echo "🔧 Server Info:"
            echo "$server_info" | jq -r '.id, .object, .created' 2>/dev/null
        fi
    else
        echo "❌ LM Studio API not responding"
        echo "   Make sure LM Studio server is running on localhost:1234"
    fi
}

# Function to check LM Studio process
check_lmstudio_process() {
    echo "🔍 Checking LM Studio processes..."
    
    local processes=$(ps aux | grep -i "lmstudio\|lms" | grep -v grep)
    
    if [ -n "$processes" ]; then
        echo "✅ LM Studio processes found:"
        echo "$processes" | while read -r line; do
            echo "   🔧 $line"
        done
    else
        echo "❌ No LM Studio processes found"
    fi
}

# Main monitoring function
main() {
    # Create timestamp for monitoring
    touch /tmp/lmstudio_monitor_start
    
    echo "🚀 Starting LM Studio monitoring..."
    echo ""
    
    # Check LM Studio process
    check_lmstudio_process
    echo ""
    
    # Check API status
    check_api_status
    echo ""
    
    # Check various log directories
    echo "📁 Checking log directories..."
    
    local log_found=false
    
    # Check common LM Studio log locations
    check_log_directory "$HOME/.lmstudio/logs" "LM Studio logs" && log_found=true
    check_log_directory "$HOME/.lmstudio/server-logs" "LM Studio server logs" && log_found=true
    check_log_directory "$HOME/Library/Application Support/LM Studio/logs" "LM Studio App Support logs" && log_found=true
    check_log_directory "$HOME/Library/Logs/LM Studio" "LM Studio Library logs" && log_found=true
    
    echo ""
    
    if [ "$log_found" = true ]; then
        echo "📊 Starting log monitoring..."
        echo "Press Ctrl+C to stop"
        echo ""
        
        # Try to monitor logs from the first available directory
        if check_log_directory "$HOME/.lmstudio/logs" "LM Studio logs" >/dev/null; then
            monitor_logs "$HOME/.lmstudio/logs" "LM Studio logs"
        elif check_log_directory "$HOME/.lmstudio/server-logs" "LM Studio server logs" >/dev/null; then
            monitor_logs "$HOME/.lmstudio/server-logs" "LM Studio server logs"
        elif check_log_directory "$HOME/Library/Application Support/LM Studio/logs" "LM Studio App Support logs" >/dev/null; then
            monitor_logs "$HOME/Library/Application Support/LM Studio/logs" "LM Studio App Support logs"
        elif check_log_directory "$HOME/Library/Logs/LM Studio" "LM Studio Library logs" >/dev/null; then
            monitor_logs "$HOME/Library/Logs/LM Studio" "LM Studio Library logs"
        fi
    else
        echo "⚠️  No log directories found. Monitoring API only..."
        echo ""
        echo "🔄 Continuous API monitoring (Ctrl+C to stop):"
        while true; do
            check_api_status
            sleep 10
            echo "---"
        done
    fi
}

# Handle script interruption
trap 'echo ""; echo "🛑 Monitoring stopped"; exit 0' INT

# Run main function
main
