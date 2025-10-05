#!/bin/bash
echo "🔗 Protocol Health Monitoring..."
echo "A2A and MCP protocol health monitoring..."
echo ""
echo "📊 Protocol endpoint monitoring..."
while true; do
  if curl -s --max-time 3 "http://localhost:8001/v1/card" > /dev/null 2>&1; then
    echo "$(date '+%H:%M:%S') ✅ A2A Agent Card endpoint healthy"
  else
    echo "$(date '+%H:%M:%S') ❌ A2A Agent Card endpoint not responding"
  fi
  if curl -s --max-time 3 "http://localhost:8001/health" > /dev/null 2>&1; then
    echo "$(date '+%H:%M:%S') ✅ Protocol health endpoint healthy"
  else
    echo "$(date '+%H:%M:%S') ❌ Protocol health endpoint not responding"
  fi
  sleep 60
done
