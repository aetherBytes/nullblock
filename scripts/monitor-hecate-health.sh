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
    llm_status=$(curl -s --max-time 3 "http://localhost:9003/health" | jq -r '.components.llm_service.overall_status // "unknown"' 2>/dev/null)
    llm_model=$(curl -s --max-time 3 "http://localhost:9003/health" | jq -r '.components.validated_model.model // "unknown"' 2>/dev/null)
    echo "🧠 LLM: $llm_status | Model: $llm_model" | tee -a ~/nullblock/logs/hecate-health.log
  else
    echo "❌ Hecate agent not responding" | tee -a ~/nullblock/logs/hecate-health.log
  fi
  sleep 30
done
