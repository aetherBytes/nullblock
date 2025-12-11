#!/bin/bash
# Enhanced task system monitoring with diagnostics

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

mkdir -p logs

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📋 Task Management System Monitor"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "The orchestration service has been integrated into the Hecate Agent (Rust)"
echo "Task management is handled by the agents service on port 9003"
echo "All task API requests route through Erebus on port 3000"
echo ""

while true; do
  timestamp=$(date '+%Y-%m-%d %H:%M:%S')
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" | tee -a logs/task-system.log
  echo "$timestamp [TASK-SYS]" | tee -a logs/task-system.log
  echo ""

  # Check Erebus first
  if ! lsof -ti:3000 > /dev/null 2>&1; then
    echo -e "${RED}❌ Erebus Router not running (port 3000)${NC}" | tee -a logs/task-system.log
    echo -e "${YELLOW}   → Task routing requires Erebus${NC}" | tee -a logs/task-system.log
    echo -e "${YELLOW}   → Start: cd ~/nullblock/svc/erebus && cargo run${NC}" | tee -a logs/task-system.log
    echo ""
    sleep 45
    continue
  fi

  # Check Agents Service
  if ! lsof -ti:9003 > /dev/null 2>&1; then
    echo -e "${RED}❌ Agents Service not running (port 9003)${NC}" | tee -a logs/task-system.log
    echo -e "${YELLOW}   → Task processing requires Agents service${NC}" | tee -a logs/task-system.log
    echo -e "${YELLOW}   → Start: cd ~/nullblock/svc/nullblock-agents && cargo run${NC}" | tee -a logs/task-system.log
    echo ""
    sleep 45
    continue
  fi

  # Test Direct Agents API
  echo -e "${CYAN}Testing Direct Agents API...${NC}"
  if agents_health=$(curl -s --max-time 3 "http://localhost:9003/health" 2>/dev/null); then
    if echo "$agents_health" | jq -e '.status' > /dev/null 2>&1; then
      status=$(echo "$agents_health" | jq -r '.status')
      echo -e "${GREEN}✅ Hecate Agent Direct API - $status${NC}" | tee -a logs/task-system.log
    else
      echo -e "${YELLOW}⚠️  Hecate Agent - Unexpected response${NC}" | tee -a logs/task-system.log
    fi
  else
    echo -e "${RED}❌ Hecate Agent Direct API - Not responding${NC}" | tee -a logs/task-system.log
  fi
  echo ""

  # Test Task API via Erebus
  echo -e "${CYAN}Testing Task API (Erebus → Agents)...${NC}"
  if task_response=$(curl -s --max-time 5 "http://localhost:3000/api/agents/tasks" 2>/dev/null); then
    if echo "$task_response" | jq -e '.success' > /dev/null 2>&1; then
      success=$(echo "$task_response" | jq -r '.success')

      if [ "$success" = "true" ]; then
        total=$(echo "$task_response" | jq -r '.total // 0')
        echo -e "${GREEN}✅ Task API via Erebus - WORKING${NC}" | tee -a logs/task-system.log
        echo -e "   Total tasks: $total" | tee -a logs/task-system.log

        if [ "$total" -gt 0 ]; then
          echo "   Recent tasks:" | tee -a logs/task-system.log
          echo "$task_response" | jq -r '.data[] | "   - " + .name + " (" + .status.state + ")"' 2>/dev/null | head -5 | tee -a logs/task-system.log

          # Task state breakdown
          working_count=$(echo "$task_response" | jq '[.data[] | select(.status.state == "working")] | length' 2>/dev/null || echo "0")
          completed_count=$(echo "$task_response" | jq '[.data[] | select(.status.state == "completed")] | length' 2>/dev/null || echo "0")
          failed_count=$(echo "$task_response" | jq '[.data[] | select(.status.state == "failed")] | length' 2>/dev/null || echo "0")

          echo "   Status breakdown:" | tee -a logs/task-system.log
          [ "$working_count" -gt 0 ] && echo -e "   ${CYAN}⏳ Working: $working_count${NC}" | tee -a logs/task-system.log
          [ "$completed_count" -gt 0 ] && echo -e "   ${GREEN}✅ Completed: $completed_count${NC}" | tee -a logs/task-system.log
          [ "$failed_count" -gt 0 ] && echo -e "   ${RED}❌ Failed: $failed_count${NC}" | tee -a logs/task-system.log
        fi
      else
        error=$(echo "$task_response" | jq -r '.error // "unknown"')
        echo -e "${YELLOW}⚠️  Task API returned error: $error${NC}" | tee -a logs/task-system.log
      fi
    else
      echo -e "${YELLOW}⚠️  Task API - Unexpected response format${NC}" | tee -a logs/task-system.log
      echo -e "   Response preview: $(echo "$task_response" | head -c 100)..." | tee -a logs/task-system.log
    fi
  else
    echo -e "${RED}❌ Task API via Erebus - NOT ACCESSIBLE${NC}" | tee -a logs/task-system.log
    echo -e "${YELLOW}   → Check Erebus logs: tail -f ~/nullblock/svc/erebus/logs/erebus.log${NC}" | tee -a logs/task-system.log
    echo -e "${YELLOW}   → Check Agents logs: tail -f ~/nullblock/svc/nullblock-agents/logs/*.log${NC}" | tee -a logs/task-system.log
  fi
  echo ""

  # Database connectivity check
  echo -e "${CYAN}Database Connectivity:${NC}"
  if docker exec -i nullblock-postgres-agents psql -U postgres -d agents -c "SELECT COUNT(*) FROM tasks" > /dev/null 2>&1; then
    task_count=$(docker exec -i nullblock-postgres-agents psql -U postgres -d agents -t -c "SELECT COUNT(*) FROM tasks" 2>/dev/null | tr -d ' ')
    echo -e "${GREEN}✅ Agents Database - Connected (${task_count} tasks in DB)${NC}" | tee -a logs/task-system.log
  else
    echo -e "${RED}❌ Agents Database - Connection failed${NC}" | tee -a logs/task-system.log
  fi
  echo ""

  echo "Next check in 45 seconds..." | tee -a logs/task-system.log
  sleep 45
done
