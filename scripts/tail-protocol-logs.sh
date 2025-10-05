#!/bin/bash
echo "🌐 Protocol Server Logs & Health"
echo "Monitoring protocol server logs and A2A/MCP operations..."
echo ""
cd ~/nullblock/svc/nullblock-protocols
mkdir -p logs
tail -f logs/protocols-server.log 2>/dev/null || echo "⚠️ Waiting for protocol server logs..."
