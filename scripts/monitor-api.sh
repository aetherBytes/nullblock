#!/bin/bash
# API endpoint testing with detailed diagnostics

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

mkdir -p logs

while true; do
  timestamp=$(date '+%Y-%m-%d %H:%M:%S')
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" | tee -a logs/api-tests.log
  echo "🧪 API Endpoint Tests - $timestamp" | tee -a logs/api-tests.log
  echo ""

  # Erebus Health
  echo -e "${CYAN}Testing Erebus Health Endpoint...${NC}"
  if erebus_health=$(curl -s --max-time 3 "http://localhost:3000/health" 2>/dev/null); then
    if echo "$erebus_health" | jq -e '.status' > /dev/null 2>&1; then
      status=$(echo "$erebus_health" | jq -r '.status')
      version=$(echo "$erebus_health" | jq -r '.version')
      echo -e "${GREEN}✅ GET /health${NC}" | tee -a logs/api-tests.log
      echo "   Status: $status | Version: $version" | tee -a logs/api-tests.log
      echo "$erebus_health" | jq '.' 2>/dev/null | head -10
    else
      echo -e "${YELLOW}⚠️  GET /health - Unexpected format${NC}" | tee -a logs/api-tests.log
      echo "$erebus_health" | head -c 200
    fi
  else
    echo -e "${RED}❌ GET /health - No response${NC}" | tee -a logs/api-tests.log
    echo -e "${YELLOW}   → Is Erebus running? Check: lsof -ti:3000${NC}"
  fi
  echo ""

  # Agents Health
  echo -e "${CYAN}Testing Agents Health Endpoint...${NC}"
  if agents_health=$(curl -s --max-time 3 "http://localhost:9003/health" 2>/dev/null); then
    if echo "$agents_health" | jq -e '.status' > /dev/null 2>&1; then
      status=$(echo "$agents_health" | jq -r '.status')
      version=$(echo "$agents_health" | jq -r '.version')
      echo -e "${GREEN}✅ GET /health${NC}" | tee -a logs/api-tests.log
      echo "   Status: $status | Version: $version" | tee -a logs/api-tests.log
      echo "$agents_health" | jq '.' 2>/dev/null | head -10
    else
      echo -e "${YELLOW}⚠️  GET /health - Unexpected format${NC}" | tee -a logs/api-tests.log
    fi
  else
    echo -e "${RED}❌ GET /health - No response${NC}" | tee -a logs/api-tests.log
    echo -e "${YELLOW}   → Is Agents service running? Check: lsof -ti:9003${NC}"
  fi
  echo ""

  # Task Management via Erebus
  echo -e "${CYAN}Testing Task Management (Erebus → Agents)...${NC}"
  if task_response=$(curl -s --max-time 3 "http://localhost:3000/api/agents/tasks" 2>/dev/null); then
    if echo "$task_response" | jq -e '.success' > /dev/null 2>&1; then
      success=$(echo "$task_response" | jq -r '.success')
      total=$(echo "$task_response" | jq -r '.total // 0')
      echo -e "${GREEN}✅ GET /api/agents/tasks${NC}" | tee -a logs/api-tests.log
      echo "   Total tasks: $total" | tee -a logs/api-tests.log

      if [ "$total" -gt 0 ]; then
        echo "   Recent tasks:" | tee -a logs/api-tests.log
        echo "$task_response" | jq -r '.data[] | "   - " + .name + " (" + .status.state + ")"' 2>/dev/null | head -5 | tee -a logs/api-tests.log
      fi
    else
      error=$(echo "$task_response" | jq -r '.error // "unknown"')
      echo -e "${YELLOW}⚠️  GET /api/agents/tasks - Error: $error${NC}" | tee -a logs/api-tests.log
    fi
  else
    echo -e "${RED}❌ GET /api/agents/tasks - No response${NC}" | tee -a logs/api-tests.log
    echo -e "${YELLOW}   → Check both Erebus and Agents are running${NC}"
  fi
  echo ""

  # Protocols Service
  echo -e "${CYAN}Testing Protocols Service...${NC}"
  if protocols_health=$(curl -s --max-time 3 "http://localhost:8001/health" 2>/dev/null); then
    if echo "$protocols_health" | jq -e '.status' > /dev/null 2>&1; then
      status=$(echo "$protocols_health" | jq -r '.status')
      echo -e "${GREEN}✅ GET /health (Protocols)${NC}" | tee -a logs/api-tests.log
      echo "   Status: $status" | tee -a logs/api-tests.log
    else
      echo -e "${YELLOW}⚠️  GET /health - Unexpected format${NC}" | tee -a logs/api-tests.log
    fi
  else
    echo -e "${RED}❌ GET /health - No response${NC}" | tee -a logs/api-tests.log
    echo -e "${YELLOW}   → Is Protocols service running? Check: lsof -ti:8001${NC}"
  fi
  echo ""

  # Content Service
  echo -e "${CYAN}Testing Content Service...${NC}"
  if content_health=$(curl -s --max-time 3 "http://localhost:8002/health" 2>/dev/null); then
    if echo "$content_health" | jq -e '.status' > /dev/null 2>&1; then
      status=$(echo "$content_health" | jq -r '.status')
      version=$(echo "$content_health" | jq -r '.version // "unknown"')
      echo -e "${GREEN}✅ GET /health (Content)${NC}" | tee -a logs/api-tests.log
      echo "   Status: $status | Version: $version" | tee -a logs/api-tests.log
    else
      echo -e "${YELLOW}⚠️  GET /health - Unexpected format${NC}" | tee -a logs/api-tests.log
    fi
  else
    echo -e "${RED}❌ GET /health - No response${NC}" | tee -a logs/api-tests.log
    echo -e "${YELLOW}   → Is Content service running? Check: lsof -ti:8002${NC}"
  fi
  echo ""

  # Content Queue Test
  echo -e "${CYAN}Testing Content Queue...${NC}"
  if queue_response=$(curl -s --max-time 3 "http://localhost:8002/api/content/queue" 2>/dev/null); then
    if echo "$queue_response" | jq -e '.total' > /dev/null 2>&1; then
      total=$(echo "$queue_response" | jq -r '.total')
      echo -e "${GREEN}✅ GET /api/content/queue${NC}" | tee -a logs/api-tests.log
      echo "   Total items: $total" | tee -a logs/api-tests.log
    else
      echo -e "${YELLOW}⚠️  GET /api/content/queue - Unexpected format${NC}" | tee -a logs/api-tests.log
    fi
  else
    echo -e "${RED}❌ GET /api/content/queue - No response${NC}" | tee -a logs/api-tests.log
  fi
  echo ""

  echo "Next test in 60 seconds..." | tee -a logs/api-tests.log
  sleep 60
done
