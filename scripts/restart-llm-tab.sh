#!/bin/bash

# Restart LLM Tab Processes
# Useful when LM Studio or monitoring gets stuck

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}🔄 Restarting LLM Tab Processes...${NC}"

# Kill existing processes
echo -e "${YELLOW}🛑 Stopping existing processes...${NC}"
pkill -f "better-llm-monitor" || true
pkill -f "simple-llm-monitor" || true
pkill -f "lms log stream" || true
pkill -f "start-lmstudio" || true

# Clean up any hanging files
rm -f /tmp/lm_studio_monitor.pid /tmp/status_monitor.pid /tmp/agent_monitor.pids

# Stop LM Studio server
echo -e "${YELLOW}🔧 Stopping LM Studio server...${NC}"
lms server stop 2>/dev/null || true

sleep 2

# Restart LM Studio
echo -e "${GREEN}🚀 Starting LM Studio server...${NC}"
lms server start &

# Wait for LM Studio to be ready
echo -e "${BLUE}⏳ Waiting for LM Studio to be ready...${NC}"
for i in {1..30}; do
    if curl -s --max-time 2 http://localhost:1234/v1/models >/dev/null 2>&1; then
        echo -e "${GREEN}✅ LM Studio is ready!${NC}"
        break
    fi
    echo -n "."
    sleep 1
done

# Check if we have models loaded
models=$(curl -s --max-time 2 http://localhost:1234/v1/models | jq -r '.data | length' 2>/dev/null || echo "0")
if [ "$models" -eq 0 ]; then
    echo -e "${YELLOW}⚠️  No models loaded. Loading Gemma3...${NC}"
    lms load gemma-3-270m-it-mlx -y
fi

echo -e "${GREEN}✅ LLM Tab processes restarted successfully!${NC}"
echo -e "${BLUE}💡 You can now restart the LLM monitoring in your tmux session${NC}"
echo ""
echo "Commands to run in tmux llm tab:"
echo "  Pane 2: ~/nullblock/scripts/better-llm-monitor.sh"
echo "  Pane 3: [Status monitor should auto-restart]"