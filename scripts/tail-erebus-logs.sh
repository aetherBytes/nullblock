#!/bin/bash
echo "📡 Erebus Service Logs & Monitoring"
echo "Monitoring Erebus unified router logs and user events..."
echo ""
cd ~/nullblock/svc/erebus
mkdir -p logs
echo "📝 Watching Erebus logs..."
tail -f logs/erebus.log 2>/dev/null || echo "⚠️ Waiting for Erebus logs..."
