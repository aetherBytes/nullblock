#!/bin/bash
# Enhanced health monitoring dashboard with clear diagnostics

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

mkdir -p logs

while true; do
  clear
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "🎯 NullBlock System Health Dashboard"
  echo "$(date '+%Y-%m-%d %H:%M:%S')"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo ""

  # Core Services
  echo -e "${CYAN}━━━ Core Services ━━━${NC}"

  # Erebus
  if curl -s --max-time 2 "http://localhost:3000/health" > /dev/null 2>&1; then
    erebus_health=$(curl -s --max-time 2 "http://localhost:3000/health")
    version=$(echo "$erebus_health" | jq -r '.version // "unknown"' 2>/dev/null)
    echo -e "${GREEN}✅${NC} Erebus Router (port 3000) - ${GREEN}HEALTHY${NC} [v$version]"
  else
    echo -e "${RED}❌${NC} Erebus Router (port 3000) - ${RED}NOT RESPONDING${NC}"
    echo -e "${YELLOW}   → Check: tail -f ~/nullblock/svc/erebus/logs/erebus.log${NC}"
    echo -e "${YELLOW}   → Start: cd ~/nullblock/svc/erebus && cargo run${NC}"
  fi

  # Agents Service
  if curl -s --max-time 2 "http://localhost:9003/health" > /dev/null 2>&1; then
    agents_health=$(curl -s --max-time 2 "http://localhost:9003/health")
    version=$(echo "$agents_health" | jq -r '.version // "unknown"' 2>/dev/null)
    echo -e "${GREEN}✅${NC} Agents Service (port 9003) - ${GREEN}HEALTHY${NC} [v$version]"
  else
    echo -e "${RED}❌${NC} Agents Service (port 9003) - ${RED}NOT RESPONDING${NC}"
    echo -e "${YELLOW}   → Check: tail -f ~/nullblock/svc/nullblock-agents/logs/*.log${NC}"
    echo -e "${YELLOW}   → Start: cd ~/nullblock/svc/nullblock-agents && cargo run${NC}"
  fi

  # Protocols Service
  if curl -s --max-time 2 "http://localhost:8001/health" > /dev/null 2>&1; then
    protocols_health=$(curl -s --max-time 2 "http://localhost:8001/health")
    version=$(echo "$protocols_health" | jq -r '.version // "unknown"' 2>/dev/null)
    echo -e "${GREEN}✅${NC} Protocols Service (port 8001) - ${GREEN}HEALTHY${NC} [v$version]"
  else
    echo -e "${RED}❌${NC} Protocols Service (port 8001) - ${RED}NOT RESPONDING${NC}"
    echo -e "${YELLOW}   → Check: tail -f ~/nullblock/svc/nullblock-protocols/logs/*.log${NC}"
    echo -e "${YELLOW}   → Start: cd ~/nullblock/svc/nullblock-protocols && cargo run${NC}"
  fi

  # Engram Service
  if curl -s --max-time 2 "http://localhost:9004/health" > /dev/null 2>&1; then
    engrams_health=$(curl -s --max-time 2 "http://localhost:9004/health")
    version=$(echo "$engrams_health" | jq -r '.version // "unknown"' 2>/dev/null)
    echo -e "${GREEN}✅${NC} Engram Service (port 9004) - ${GREEN}HEALTHY${NC} [v$version]"
  else
    echo -e "${RED}❌${NC} Engram Service (port 9004) - ${RED}NOT RESPONDING${NC}"
    echo -e "${YELLOW}   → Check: tail -f ~/nullblock/svc/nullblock-engrams/logs/*.log${NC}"
    echo -e "${YELLOW}   → Start: cd ~/nullblock/svc/nullblock-engrams && cargo run${NC}"
  fi

  # Frontend
  if lsof -ti:5173 > /dev/null 2>&1; then
    echo -e "${GREEN}✅${NC} Frontend (port 5173) - ${GREEN}RUNNING${NC}"
  else
    echo -e "${RED}❌${NC} Frontend (port 5173) - ${RED}NOT RUNNING${NC}"
    echo -e "${YELLOW}   → Start: cd ~/nullblock/svc/hecate && npm run develop${NC}"
  fi
  echo ""

  # Infrastructure
  echo -e "${CYAN}━━━ Infrastructure ━━━${NC}"

  # Erebus Database
  if docker ps --filter name=nullblock-postgres-erebus --format '{{.Names}}' | grep -q postgres; then
    if docker exec nullblock-postgres-erebus pg_isready -U postgres > /dev/null 2>&1; then
      echo -e "${GREEN}✅${NC} Erebus Database (port 5440) - ${GREEN}HEALTHY${NC}"
    else
      echo -e "${YELLOW}⚠️${NC}  Erebus Database (port 5440) - ${YELLOW}CONTAINER UP BUT NOT READY${NC}"
    fi
  else
    echo -e "${RED}❌${NC} Erebus Database (port 5440) - ${RED}NOT RUNNING${NC}"
    echo -e "${YELLOW}   → Start: just start${NC}"
  fi

  # Agents Database
  if docker ps --filter name=nullblock-postgres-agents --format '{{.Names}}' | grep -q postgres; then
    if docker exec nullblock-postgres-agents pg_isready -U postgres > /dev/null 2>&1; then
      echo -e "${GREEN}✅${NC} Agents Database (port 5441) - ${GREEN}HEALTHY${NC}"
    else
      echo -e "${YELLOW}⚠️${NC}  Agents Database (port 5441) - ${YELLOW}CONTAINER UP BUT NOT READY${NC}"
    fi
  else
    echo -e "${RED}❌${NC} Agents Database (port 5441) - ${RED}NOT RUNNING${NC}"
    echo -e "${YELLOW}   → Start: just start${NC}"
  fi

  # Redis
  if docker ps --filter name=nullblock-redis --format '{{.Names}}' | grep -q redis; then
    if docker exec nullblock-redis redis-cli ping > /dev/null 2>&1; then
      echo -e "${GREEN}✅${NC} Redis (port 6379) - ${GREEN}HEALTHY${NC}"
    else
      echo -e "${YELLOW}⚠️${NC}  Redis (port 6379) - ${YELLOW}CONTAINER UP BUT NOT RESPONDING${NC}"
    fi
  else
    echo -e "${RED}❌${NC} Redis (port 6379) - ${RED}NOT RUNNING${NC}"
    echo -e "${YELLOW}   → Start: just start${NC}"
  fi

  # Kafka
  if docker ps --filter name=nullblock-kafka --format '{{.Names}}' | grep -q kafka; then
    echo -e "${GREEN}✅${NC} Kafka + Zookeeper - ${GREEN}RUNNING${NC}"
  else
    echo -e "${RED}❌${NC} Kafka + Zookeeper - ${RED}NOT RUNNING${NC}"
    echo -e "${YELLOW}   → Start: just start${NC}"
  fi
  echo ""

  # API Connectivity
  echo -e "${CYAN}━━━ API Connectivity ━━━${NC}"

  # Task API
  if task_response=$(curl -s --max-time 2 "http://localhost:3000/api/agents/tasks" 2>/dev/null); then
    if echo "$task_response" | jq -e '.success' > /dev/null 2>&1; then
      total=$(echo "$task_response" | jq -r '.total // 0')
      echo -e "${GREEN}✅${NC} Task API (Erebus → Agents) - ${GREEN}WORKING${NC} ($total tasks)"
    else
      echo -e "${YELLOW}⚠️${NC}  Task API - ${YELLOW}UNEXPECTED RESPONSE${NC}"
    fi
  else
    echo -e "${RED}❌${NC} Task API (Erebus → Agents) - ${RED}NOT ACCESSIBLE${NC}"
    echo -e "${YELLOW}   → Check Erebus and Agents services are running${NC}"
  fi

  # Health API
  if curl -s --max-time 2 "http://localhost:3000/health" > /dev/null 2>&1; then
    echo -e "${GREEN}✅${NC} Health API (Erebus) - ${GREEN}WORKING${NC}"
  else
    echo -e "${RED}❌${NC} Health API (Erebus) - ${RED}NOT ACCESSIBLE${NC}"
  fi
  echo ""

  # COWs (Constellations of Work)
  echo -e "${CYAN}━━━ COWs (Constellations of Work) ━━━${NC}"

  # COW #1: ArbFarm - Solana MEV Agent Swarm
  if curl -s --max-time 2 "http://localhost:9007/health" > /dev/null 2>&1; then
    arb_health=$(curl -s --max-time 2 "http://localhost:9007/health" 2>/dev/null)
    arb_status=$(echo "$arb_health" | jq -r '.status // "unknown"' 2>/dev/null)

    # Get scanner status
    scanner=$(curl -s --max-time 2 "http://localhost:9007/scanner/status" 2>/dev/null)
    scanner_running=$(echo "$scanner" | jq -r '.is_running // false' 2>/dev/null)

    # Get executor status
    executor=$(curl -s --max-time 2 "http://localhost:9007/executor/stats" 2>/dev/null)
    executor_running=$(echo "$executor" | jq -r '.is_running // false' 2>/dev/null)
    executions=$(echo "$executor" | jq -r '.executions_succeeded // 0' 2>/dev/null)

    # Get positions
    positions=$(curl -s --max-time 2 "http://localhost:9007/positions" 2>/dev/null)
    pos_count=$(echo "$positions" | jq -r '.positions | length // 0' 2>/dev/null)

    # Get wallet balance
    wallet=$(curl -s --max-time 2 "http://localhost:9007/wallet/balance" 2>/dev/null)
    balance=$(echo "$wallet" | jq -r '.balance_sol // "?"' 2>/dev/null)

    # Get risk config
    risk=$(curl -s --max-time 2 "http://localhost:9007/config/risk" 2>/dev/null)
    max_pos=$(echo "$risk" | jq -r '.max_position_sol // "?"' 2>/dev/null)

    echo -e "${GREEN}✅${NC} ArbFarm (port 9007) - ${GREEN}$arb_status${NC}"
    echo -e "   Scanner: $([ "$scanner_running" = "true" ] && echo -e "${GREEN}ON${NC}" || echo -e "${RED}OFF${NC}") | Executor: $([ "$executor_running" = "true" ] && echo -e "${GREEN}ON${NC}" || echo -e "${RED}OFF${NC}") | Trades: $executions"
    echo -e "   Positions: $pos_count | Wallet: ${balance} SOL | Max: ${max_pos} SOL"
  else
    echo -e "${RED}❌${NC} ArbFarm (port 9007) - ${RED}NOT RESPONDING${NC}"
    echo -e "${YELLOW}   → Start: cd ~/nullblock/svc/arb-farm && cargo run${NC}"
  fi
  echo ""

  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "Press Ctrl+C to exit | Refreshing in 30s..."
  sleep 30
done
