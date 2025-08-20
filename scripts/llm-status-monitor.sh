#!/bin/bash

# LLM Status Monitor
# Clean system status monitoring for LLM tab

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Cleanup function
cleanup() {
    echo ""
    echo -e "${YELLOW}🛑 Stopping status monitor...${NC}"
    exit 0
}

# Set up signal handlers
trap cleanup SIGINT SIGTERM

echo -e "${CYAN}🔍 LLM System Status Monitor${NC}"
echo "============================="
echo "Monitoring LM Studio and Hecate Agent every 15 seconds"
echo "Press Ctrl+C to stop"
echo ""

while true; do
    echo "$(date '+%H:%M:%S') System Status:"
    
    # Check LM Studio API
    if curl -s --max-time 2 http://localhost:1234/v1/models >/dev/null 2>&1; then
        models=$(curl -s --max-time 2 http://localhost:1234/v1/models | jq -r '.data | length' 2>/dev/null || echo "?")
        echo -e "  ${GREEN}✅ LM Studio API: Online ($models models)${NC}"
    else
        echo -e "  ${RED}❌ LM Studio API: Offline${NC}"
    fi
    
    # Check Hecate Agent
    if curl -s --max-time 2 http://localhost:9002/health >/dev/null 2>&1; then
        echo -e "  ${GREEN}✅ Hecate Agent: Online${NC}"
    else
        echo -e "  ${RED}❌ Hecate Agent: Offline${NC}"
    fi
    
    # Show active connections
    connections=$(lsof -i :1234 2>/dev/null | grep ESTABLISHED | wc -l | tr -d ' ')
    echo -e "  ${BLUE}🔗 Active connections: $connections${NC}"
    
    echo -e "  ${CYAN}📝 Log files:${NC}"
    echo "     - Input/Output: logs/llm-monitor.log"
    echo "     - Agent logs: svc/nullblock-agents/logs/"
    echo ""
    
    sleep 15
done