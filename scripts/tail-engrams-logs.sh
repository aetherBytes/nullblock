#!/bin/bash
echo "📊 Engram Service Logs & Health Monitoring"
echo "Monitoring engram service logs and operations..."
echo ""
cd ~/nullblock/svc/nullblock-engrams
mkdir -p logs
echo "🧠 Watching Engram logs..."
sleep 10
tail -f logs/engrams.log 2>/dev/null || echo "⚠️ Waiting for Engram service logs..."
