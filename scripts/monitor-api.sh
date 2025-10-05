#!/bin/bash
# API endpoint testing for NullBlock services

while true; do
  echo "🧪 API Endpoint Tests - $(date '+%H:%M:%S')"
  echo "curl http://localhost:3000/health (Erebus)"
  curl -s --max-time 3 "http://localhost:3000/health" | jq . 2>/dev/null || echo "❌ Failed"
  echo ""
  echo "curl http://localhost:9003/health (Agents)"
  curl -s --max-time 3 "http://localhost:9003/health" | jq . 2>/dev/null || echo "❌ Failed"
  echo ""
  echo "curl http://localhost:3000/api/agents/tasks (Task Management via Erebus)"
  curl -s --max-time 3 "http://localhost:3000/api/agents/tasks" | jq '.data // .total // "error"' 2>/dev/null || echo "❌ Failed"
  sleep 60
done
