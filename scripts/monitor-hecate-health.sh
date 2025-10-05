#!/bin/bash
echo "🎯 Hecate Agent Health & Task Monitoring"
echo "Real-time monitoring of agent health and task management..."
echo ""
mkdir -p ~/nullblock/logs
while true; do
  echo "$(date '+%Y-%m-%d %H:%M:%S') [AGENT] Hecate health check..." | tee -a ~/nullblock/logs/hecate-health.log
  if curl -s --max-time 5 "http://localhost:9003/hecate/health" > /dev/null 2>&1; then
    echo "✅ Hecate agent healthy" | tee -a ~/nullblock/logs/hecate-health.log
    curl -s "http://localhost:9003/hecate/model-status" | jq -r '"🧠 Model: " + (.current_model // "unknown") + " | Status: " + (.status // "unknown")' 2>/dev/null | tee -a ~/nullblock/logs/hecate-health.log
    task_count=$(curl -s "http://localhost:9003/tasks" | jq '.total // 0' 2>/dev/null || echo "0")
    echo "📋 Current tasks: $task_count" | tee -a ~/nullblock/logs/hecate-health.log
    if curl -s --max-time 3 "http://localhost:9003/health" | jq -r '.components.database.status' 2>/dev/null | grep -q "healthy"; then
      echo "🗄️  Database migrations: ✅ Complete" | tee -a ~/nullblock/logs/hecate-health.log
    else
      echo "🗄️  Database migrations: ⚠️  Check required" | tee -a ~/nullblock/logs/hecate-health.log
    fi
  else
    echo "❌ Hecate agent not responding" | tee -a ~/nullblock/logs/hecate-health.log
  fi
  sleep 30
done
