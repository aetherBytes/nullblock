#!/bin/bash
# Enhanced task management monitoring with clear diagnostics

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

mkdir -p logs

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📋 Task System Monitor"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

while true; do
    timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" | tee -a logs/task-metrics.log
    echo "$timestamp [TASK MONITOR]" | tee -a logs/task-metrics.log
    echo ""

    # Check if Erebus is up first
    if ! lsof -ti:3000 > /dev/null 2>&1; then
        echo -e "${RED}❌ Erebus not running (port 3000 not open)${NC}" | tee -a logs/task-metrics.log
        echo -e "${YELLOW}   → Start Erebus to enable task routing${NC}" | tee -a logs/task-metrics.log
        sleep 45
        continue
    fi

    # Check if Agents service is up
    if ! lsof -ti:9003 > /dev/null 2>&1; then
        echo -e "${RED}❌ Agents service not running (port 9003 not open)${NC}" | tee -a logs/task-metrics.log
        echo -e "${YELLOW}   → Start Agents service for task processing${NC}" | tee -a logs/task-metrics.log
        sleep 45
        continue
    fi

    # Test task endpoint via Erebus
    echo -e "${CYAN}Testing Task API (Erebus → Agents)...${NC}"
    if task_response=$(curl -s --max-time 5 "http://localhost:3000/api/agents/tasks" 2>/dev/null); then
        # Check if we got a valid JSON response
        if echo "$task_response" | jq -e '.success' > /dev/null 2>&1; then
            success=$(echo "$task_response" | jq -r '.success')
            if [ "$success" = "true" ]; then
                total=$(echo "$task_response" | jq -r '.total // 0')
                echo -e "${GREEN}✅ Task API is WORKING${NC}" | tee -a logs/task-metrics.log
                echo -e "   Total tasks: $total" | tee -a logs/task-metrics.log

                # Show task breakdown if any exist
                if [ "$total" -gt 0 ]; then
                    echo "$task_response" | jq -r '.data[] | "   - " + .name + " (" + .status.state + ")"' 2>/dev/null | head -5 | tee -a logs/task-metrics.log
                fi
            else
                error=$(echo "$task_response" | jq -r '.error // "unknown error"')
                echo -e "${YELLOW}⚠️  Task API returned error: $error${NC}" | tee -a logs/task-metrics.log
            fi
        else
            echo -e "${YELLOW}⚠️  Task API responded but with unexpected format${NC}" | tee -a logs/task-metrics.log
            echo -e "   Response: $(echo "$task_response" | head -c 100)..." | tee -a logs/task-metrics.log
        fi
    else
        echo -e "${RED}❌ Task API not responding${NC}" | tee -a logs/task-metrics.log
        echo -e "${YELLOW}   → Check Erebus logs: tail -f svc/erebus/logs/erebus.log${NC}" | tee -a logs/task-metrics.log
        echo -e "${YELLOW}   → Check Agents logs: tail -f svc/nullblock-agents/logs/*.log${NC}" | tee -a logs/task-metrics.log
    fi
    echo ""

    # Hecate agent model status
    echo -e "${CYAN}Agent Model Status:${NC}"
    if model_status=$(curl -s --max-time 3 "http://localhost:9003/hecate/model-status" 2>/dev/null); then
        current_model=$(echo "$model_status" | jq -r '.current_model // "unknown"')
        status=$(echo "$model_status" | jq -r '.status // "unknown"')
        echo -e "${GREEN}✅ Hecate Agent Model: $current_model${NC}" | tee -a logs/task-metrics.log
        echo -e "   Status: $status" | tee -a logs/task-metrics.log
    else
        echo -e "${YELLOW}⚠️  Could not fetch model status${NC}" | tee -a logs/task-metrics.log
    fi
    echo ""

    echo "Next check in 45 seconds..." | tee -a logs/task-metrics.log
    sleep 45
done
