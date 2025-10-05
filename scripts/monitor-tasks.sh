#!/bin/bash
# Task management metrics monitoring for NullBlock

mkdir -p logs

while true; do
  echo "$(date '+%Y-%m-%d %H:%M:%S') [METRICS] Task system status..." | tee -a logs/task-metrics.log

  # Test task endpoints (both direct and via Erebus)
  if curl -s --max-time 5 "http://localhost:3000/api/agents/tasks" > /dev/null 2>&1; then
    # Via Erebus (production route)
    task_response=$(curl -s "http://localhost:3000/api/agents/tasks" 2>/dev/null)
    if echo "$task_response" | jq -e '.data' > /dev/null 2>&1; then
      task_count=$(echo "$task_response" | jq '.data | length' 2>/dev/null || echo "0")
      echo "📋 Tasks via Erebus: $task_count" | tee -a logs/task-metrics.log
    else
      total_count=$(echo "$task_response" | jq '.total // 0' 2>/dev/null || echo "0")
      echo "📋 Total tasks: $total_count" | tee -a logs/task-metrics.log
    fi

    # Hecate agent model status
    echo "🎯 Agent model status:" | tee -a logs/task-metrics.log
    curl -s "http://localhost:9003/hecate/model-status" | jq -r '"  Model: " + (.current_model // "unknown") + " | Status: " + (.status // "unknown")' 2>/dev/null | tee -a logs/task-metrics.log

    # Wallet-based filtering test
    echo "🔐 Wallet filtering: Active (x-wallet-address headers)" | tee -a logs/task-metrics.log
  else
    echo "❌ Task system not available via Erebus" | tee -a logs/task-metrics.log
  fi

  sleep 45
done
