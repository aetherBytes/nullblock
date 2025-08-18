#!/bin/bash

# Wait for LM Studio server to be ready, then start monitoring

echo "⏳ Waiting for LM Studio server to be ready..."
echo "📡 Checking server status every 5 seconds..."

# Wait for LM Studio API to respond
while ! curl -s http://localhost:1234/v1/models >/dev/null 2>&1; do 
    echo "⏳ Server not ready yet, waiting..."
    sleep 5
done

echo "✅ LM Studio server is ready!"
echo "🚀 Starting monitoring in 2 seconds..."
sleep 2

# Change to nullblock directory and start monitoring
cd ~/nullblock
exec ./scripts/monitor-lmstudio.sh