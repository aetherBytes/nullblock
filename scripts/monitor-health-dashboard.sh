#!/bin/bash

# Enhanced health monitoring with clear visual feedback

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🏥 NullBlock Health Monitor"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

while true; do
    clear
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "🏥 NullBlock Health Monitor"
    echo "$(date '+%Y-%m-%d %H:%M:%S')"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""

    # Erebus Health
    echo -e "${CYAN}Erebus Router (Port 3000):${NC}"
    if erebus_health=$(curl -s --max-time 3 "http://localhost:3000/health" 2>/dev/null); then
        status=$(echo "$erebus_health" | jq -r '.status // "unknown"')
        if [ "$status" = "healthy" ]; then
            echo -e "  ${GREEN}✅ Status: HEALTHY${NC}"
            echo "$erebus_health" | jq -r '"  Service: " + .service + " v" + .version'
        else
            echo -e "  ${YELLOW}⚠️  Status: $status${NC}"
        fi
    else
        echo -e "  ${RED}❌ UNREACHABLE${NC}"
        echo -e "  ${RED}   → Check if Erebus is running${NC}"
        echo -e "  ${RED}   → Check logs: tail -f svc/erebus/logs/erebus.log${NC}"
    fi
    echo ""

    # Agents Health
    echo -e "${CYAN}Agents Service (Port 9003):${NC}"
    if agents_health=$(curl -s --max-time 3 "http://localhost:9003/health" 2>/dev/null); then
        status=$(echo "$agents_health" | jq -r '.status // "unknown"')
        if [ "$status" = "healthy" ]; then
            echo -e "  ${GREEN}✅ Status: HEALTHY${NC}"
            echo "$agents_health" | jq -r '"  Service: " + .service + " v" + .version'
        else
            echo -e "  ${YELLOW}⚠️  Status: $status${NC}"
        fi
    else
        echo -e "  ${RED}❌ UNREACHABLE${NC}"
        echo -e "  ${RED}   → Check if Agents service is running${NC}"
        echo -e "  ${RED}   → Check logs: tail -f svc/nullblock-agents/logs/*.log${NC}"
    fi
    echo ""

    # Protocols Health
    echo -e "${CYAN}Protocols Service (Port 8001):${NC}"
    if protocols_health=$(curl -s --max-time 3 "http://localhost:8001/health" 2>/dev/null); then
        status=$(echo "$protocols_health" | jq -r '.status // "unknown"')
        if [ "$status" = "healthy" ]; then
            echo -e "  ${GREEN}✅ Status: HEALTHY${NC}"
            echo "$protocols_health" | jq -r '"  Service: " + .service + " v" + .version'
        else
            echo -e "  ${YELLOW}⚠️  Status: $status${NC}"
        fi
    else
        echo -e "  ${RED}❌ UNREACHABLE${NC}"
        echo -e "  ${RED}   → Check if Protocols service is running${NC}"
    fi
    echo ""

    # Database Health
    echo -e "${CYAN}Databases:${NC}"
    if docker exec -i nullblock-postgres-erebus psql -U postgres -d erebus -c "SELECT 1" > /dev/null 2>&1; then
        echo -e "  ${GREEN}✅ Erebus DB (Port 5440): HEALTHY${NC}"
    else
        echo -e "  ${RED}❌ Erebus DB (Port 5440): UNREACHABLE${NC}"
    fi

    if docker exec -i nullblock-postgres-agents psql -U postgres -d agents -c "SELECT 1" > /dev/null 2>&1; then
        echo -e "  ${GREEN}✅ Agents DB (Port 5441): HEALTHY${NC}"
    else
        echo -e "  ${RED}❌ Agents DB (Port 5441): UNREACHABLE${NC}"
    fi
    echo ""

    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "Press Ctrl+C to exit | Refreshing in 10s..."
    sleep 10
done
