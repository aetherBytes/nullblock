#!/bin/bash
echo "📋 Task Management System"
echo "The orchestration service has been integrated into the Hecate Agent (Rust)"
echo "Task management is now handled directly by the agents service on port 9003"
echo ""
echo "🔄 Monitoring Task System Performance..."
while true; do
  echo "$(date '+%Y-%m-%d %H:%M:%S') [TASK-SYS] Task system metrics..."
  if curl -s --max-time 3 "http://localhost:3000/api/agents/tasks" > /dev/null 2>&1; then
    echo "✅ Task API via Erebus - Healthy"
  else
    echo "❌ Task API via Erebus - Failed"
  fi
  if curl -s --max-time 3 "http://localhost:9003/health" > /dev/null 2>&1; then
    echo "✅ Hecate Agent Direct API - Healthy"
  else
    echo "❌ Hecate Agent Direct API - Failed"
  fi
  sleep 60
done
