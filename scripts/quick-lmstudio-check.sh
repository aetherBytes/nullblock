#!/bin/bash

# Quick LM Studio Status Check
# This script quickly checks LM Studio status and log locations

echo "🔍 Quick LM Studio Status Check"
echo "==============================="

# Check if LM Studio is running
echo "1️⃣ Checking LM Studio processes..."
if pgrep -f "lmstudio\|lms" > /dev/null; then
    echo "✅ LM Studio is running"
    ps aux | grep -i "lmstudio\|lms" | grep -v grep | head -3
else
    echo "❌ LM Studio is not running"
fi

echo ""

# Check API status
echo "2️⃣ Checking LM Studio API..."
if curl -s http://localhost:1234/v1/models > /dev/null 2>&1; then
    echo "✅ LM Studio API is responding"
    echo "📋 Available models:"
    curl -s http://localhost:1234/v1/models | jq -r '.data[].id' 2>/dev/null || echo "   (Could not parse model list)"
else
    echo "❌ LM Studio API is not responding"
fi

echo ""

# Check log directories
echo "3️⃣ Checking log directories..."

log_dirs=(
    "$HOME/.lmstudio/logs"
    "$HOME/.lmstudio/server-logs"
    "$HOME/Library/Application Support/LM Studio/logs"
    "$HOME/Library/Logs/LM Studio"
)

for dir in "${log_dirs[@]}"; do
    if [ -d "$dir" ]; then
        echo "✅ Found: $dir"
        log_count=$(find "$dir" -name "*.log" -type f 2>/dev/null | wc -l)
        if [ "$log_count" -gt 0 ]; then
            echo "   📄 $log_count log files found"
            # Show the most recent log file
            latest_log=$(find "$dir" -name "*.log" -type f -exec ls -t {} + 2>/dev/null | head -1)
            if [ -n "$latest_log" ]; then
                echo "   📄 Latest: $(basename "$latest_log")"
            fi
        else
            echo "   ⚠️  No .log files found"
        fi
    else
        echo "❌ Not found: $dir"
    fi
done

echo ""

# Check LM Studio CLI
echo "4️⃣ Checking LM Studio CLI..."
if command -v lms > /dev/null; then
    echo "✅ LM Studio CLI is available"
    echo "📋 Current status:"
    lms status 2>/dev/null || echo "   (Could not get status)"
else
    echo "❌ LM Studio CLI not found"
fi

echo ""
echo "🎯 Quick check complete!"

# Check if user wants live tail
echo ""
read -p "🔄 Would you like to start live log monitoring? (y/n): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "📊 Starting live log monitoring..."
    echo "Press Ctrl+C to stop"
    echo ""
    
    # Find the most recent log file and tail it
    log_files=()
    
    # Check server-logs directory
    if [ -d "$HOME/.lmstudio/server-logs" ]; then
        server_logs=$(find "$HOME/.lmstudio/server-logs" -name "*.log" -type f -exec ls -t {} + 2>/dev/null | head -1)
        if [ -n "$server_logs" ]; then
            log_files+=("$server_logs")
        fi
    fi
    
    # Check Library logs directory
    if [ -d "$HOME/Library/Logs/LM Studio" ]; then
        lib_logs=$(find "$HOME/Library/Logs/LM Studio" -name "*.log" -type f -exec ls -t {} + 2>/dev/null | head -1)
        if [ -n "$lib_logs" ]; then
            log_files+=("$lib_logs")
        fi
    fi
    
    if [ ${#log_files[@]} -gt 0 ]; then
        echo "📄 Following log files:"
        for log_file in "${log_files[@]}"; do
            echo "   📄 $(basename "$log_file")"
        done
        echo ""
        
        # Use tail to follow the log files
        tail -f "${log_files[@]}" 2>/dev/null
    else
        echo "❌ No log files found for live monitoring"
        echo "🔄 Falling back to API monitoring..."
        while true; do
            echo "--- $(date) ---"
            curl -s http://localhost:1234/v1/models | jq -r '.data[].id' 2>/dev/null || echo "API not responding"
            sleep 5
        done
    fi
fi
