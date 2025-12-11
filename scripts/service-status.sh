#!/bin/bash

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🚀 NullBlock Services Status Dashboard"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

check_service() {
    local name=$1
    local url=$2
    local port=$3

    # Check if port is listening
    if lsof -ti:$port > /dev/null 2>&1; then
        # Port is open, now check if service responds
        if curl -s --max-time 2 "$url" > /dev/null 2>&1; then
            echo -e "${GREEN}✅${NC} $name (port $port) - ${GREEN}HEALTHY${NC}"
            return 0
        else
            echo -e "${YELLOW}⚠️${NC}  $name (port $port) - ${YELLOW}PORT OPEN BUT NOT RESPONDING${NC}"
            return 1
        fi
    else
        echo -e "${RED}❌${NC} $name (port $port) - ${RED}NOT RUNNING${NC}"
        return 1
    fi
}

check_database() {
    local name=$1
    local port=$2
    local db=$3

    if lsof -ti:$port > /dev/null 2>&1; then
        if docker exec -i nullblock-postgres-$db psql -U postgres -d $db -c "SELECT 1" > /dev/null 2>&1; then
            echo -e "${GREEN}✅${NC} $name (port $port) - ${GREEN}HEALTHY${NC}"
            return 0
        else
            echo -e "${YELLOW}⚠️${NC}  $name (port $port) - ${YELLOW}CONTAINER UP BUT DB UNREACHABLE${NC}"
            return 1
        fi
    else
        echo -e "${RED}❌${NC} $name (port $port) - ${RED}NOT RUNNING${NC}"
        return 1
    fi
}

# Infrastructure
echo -e "${CYAN}━━━ Infrastructure Services ━━━${NC}"
check_database "Erebus Database" 5440 "erebus"
check_database "Agents Database" 5441 "agents"

if docker ps --filter name=nullblock-kafka --format '{{.Names}}' | grep -q kafka; then
    echo -e "${GREEN}✅${NC} Kafka - ${GREEN}RUNNING${NC}"
else
    echo -e "${RED}❌${NC} Kafka - ${RED}NOT RUNNING${NC}"
fi
echo ""

# Core Services
echo -e "${CYAN}━━━ Core Services ━━━${NC}"
check_service "Erebus Router" "http://localhost:3000/health" 3000
check_service "Agents Service" "http://localhost:9003/health" 9003
check_service "Protocols Service" "http://localhost:8001/health" 8001
echo ""

# Frontend
echo -e "${CYAN}━━━ Frontend ━━━${NC}"
check_service "Hecate UI" "http://localhost:5173" 5173
echo ""

# API Endpoints Test
echo -e "${CYAN}━━━ API Connectivity ━━━${NC}"

# Test task endpoint
if curl -s --max-time 2 "http://localhost:3000/api/agents/tasks" > /dev/null 2>&1; then
    task_response=$(curl -s --max-time 2 "http://localhost:3000/api/agents/tasks" 2>/dev/null)
    if echo "$task_response" | jq -e '.success' > /dev/null 2>&1; then
        total=$(echo "$task_response" | jq -r '.total // 0')
        echo -e "${GREEN}✅${NC} Task API (Erebus → Agents) - ${GREEN}WORKING${NC} ($total tasks)"
    else
        echo -e "${YELLOW}⚠️${NC}  Task API (Erebus → Agents) - ${YELLOW}RESPONDS BUT UNEXPECTED FORMAT${NC}"
    fi
else
    echo -e "${RED}❌${NC} Task API (Erebus → Agents) - ${RED}NOT ACCESSIBLE${NC}"
fi

# Test user endpoint
if curl -s --max-time 2 "http://localhost:3000/health" > /dev/null 2>&1; then
    echo -e "${GREEN}✅${NC} Health API (Erebus) - ${GREEN}WORKING${NC}"
else
    echo -e "${RED}❌${NC} Health API (Erebus) - ${RED}NOT ACCESSIBLE${NC}"
fi
echo ""

# Environment Check
echo -e "${CYAN}━━━ Environment ━━━${NC}"
if [ -f ".env.dev" ]; then
    echo -e "${GREEN}✅${NC} .env.dev file - ${GREEN}EXISTS${NC}"
    if grep -q "ENCRYPTION_MASTER_KEY" .env.dev; then
        echo -e "${GREEN}✅${NC} Encryption key - ${GREEN}CONFIGURED${NC}"
    else
        echo -e "${YELLOW}⚠️${NC}  Encryption key - ${YELLOW}NOT CONFIGURED${NC}"
    fi
    if grep -q "OPENROUTER_API_KEY" .env.dev; then
        echo -e "${GREEN}✅${NC} OpenRouter API key - ${GREEN}CONFIGURED${NC}"
    else
        echo -e "${YELLOW}⚠️${NC}  OpenRouter API key - ${YELLOW}NOT CONFIGURED${NC}"
    fi
else
    echo -e "${RED}❌${NC} .env.dev file - ${RED}MISSING${NC}"
fi
echo ""

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Dashboard last updated: $(date '+%Y-%m-%d %H:%M:%S')"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
