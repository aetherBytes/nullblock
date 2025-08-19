#!/bin/bash

# Quick LM Studio Log Tailer
# Immediately starts tailing the most recent LM Studio log files

echo "📊 LM Studio Live Log Monitor"
echo "============================="

# Find the most recent log files
log_files=()

# Check server-logs directory (most common)
if [ -d "$HOME/.lmstudio/server-logs" ]; then
    server_logs=$(find "$HOME/.lmstudio/server-logs" -name "*.log" -type f -exec ls -t {} + 2>/dev/null | head -2)
    if [ -n "$server_logs" ]; then
        while IFS= read -r log_file; do
            if [ -n "$log_file" ]; then
                log_files+=("$log_file")
            fi
        done <<< "$server_logs"
    fi
fi

# Check Library logs directory
if [ -d "$HOME/Library/Logs/LM Studio" ]; then
    lib_logs=$(find "$HOME/Library/Logs/LM Studio" -name "*.log" -type f -exec ls -t {} + 2>/dev/null | head -1)
    if [ -n "$lib_logs" ]; then
        log_files+=("$lib_logs")
    fi
fi

# Check regular logs directory
if [ -d "$HOME/.lmstudio/logs" ]; then
    reg_logs=$(find "$HOME/.lmstudio/logs" -name "*.log" -type f -exec ls -t {} + 2>/dev/null | head -1)
    if [ -n "$reg_logs" ]; then
        log_files+=("$reg_logs")
    fi
fi

if [ ${#log_files[@]} -gt 0 ]; then
    echo "📄 Following log files:"
    for log_file in "${log_files[@]}"; do
        echo "   📄 $(basename "$log_file")"
    done
    echo ""
    echo "🔄 Live monitoring started (Ctrl+C to stop)..."
    echo "============================================="
    
    # Use tail to follow the log files with timestamps
    tail -f "${log_files[@]}" 2>/dev/null | while read -r line; do
        echo "[$(date '+%H:%M:%S')] $line"
    done
else
    echo "❌ No log files found"
    echo "🔄 Falling back to API monitoring..."
    echo "============================================="
    while true; do
        echo "[$(date '+%H:%M:%S')] API Status:"
        curl -s http://localhost:1234/v1/models | jq -r '.data[].id' 2>/dev/null || echo "   API not responding"
        sleep 10
    done
fi

