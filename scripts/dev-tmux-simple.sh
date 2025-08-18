#!/bin/bash

# Simple tmux development environment setup
# This script creates a tmux session with all the necessary windows and panes

SESSION_NAME="nullblock-dev"

# Kill existing session if it exists
tmux kill-session -t $SESSION_NAME 2>/dev/null || true

# Create new session
tmux new-session -d -s $SESSION_NAME -n infrastructure

# Infrastructure window - 3 panes
tmux split-window -h -t $SESSION_NAME:infrastructure
tmux split-window -h -t $SESSION_NAME:infrastructure
tmux select-layout -t $SESSION_NAME:infrastructure even-horizontal

# Send commands to infrastructure panes
tmux send-keys -t $SESSION_NAME:infrastructure.0 "brew services start postgresql@17" Enter
tmux send-keys -t $SESSION_NAME:infrastructure.1 "brew services start redis" Enter
tmux send-keys -t $SESSION_NAME:infrastructure.2 "ipfs daemon --enable-gc" Enter

# Backend window
tmux new-window -t $SESSION_NAME -n backend
tmux split-window -h -t $SESSION_NAME:backend
tmux split-window -h -t $SESSION_NAME:backend
tmux select-layout -t $SESSION_NAME:backend even-horizontal

# Send commands to backend panes
tmux send-keys -t $SESSION_NAME:backend.0 "cd svc/nullblock-mcp && export MCP_SERVER_HOST=0.0.0.0 && export MCP_SERVER_PORT=8001 && python3 -m mcp" Enter
tmux send-keys -t $SESSION_NAME:backend.1 "cd svc/nullblock-orchestration && export ORCHESTRATION_HOST=0.0.0.0 && export ORCHESTRATION_PORT=8002 && python3 -m orchestration" Enter
tmux send-keys -t $SESSION_NAME:backend.2 "cd svc/nullblock-agents && export AGENTS_HOST=0.0.0.0 && export AGENTS_PORT=8003 && python3 -m agents" Enter

# Rust window
tmux new-window -t $SESSION_NAME -n rust
tmux split-window -h -t $SESSION_NAME:rust
tmux select-layout -t $SESSION_NAME:rust even-horizontal

# Send commands to rust panes
tmux send-keys -t $SESSION_NAME:rust.0 "cd svc/erebus && export EREBUS_HOST=127.0.0.1 && export EREBUS_PORT=3000 && cargo run" Enter
tmux send-keys -t $SESSION_NAME:rust.1 "cd svc/erebus && cargo build --release" Enter

# Frontend window
tmux new-window -t $SESSION_NAME -n frontend
tmux split-window -h -t $SESSION_NAME:frontend
tmux select-layout -t $SESSION_NAME:frontend even-horizontal

# Send commands to frontend panes
tmux send-keys -t $SESSION_NAME:frontend.0 "cd svc/hecate && export VITE_MCP_API_URL=http://localhost:8001 && export VITE_EREBUS_API_URL=http://localhost:3000 && export VITE_ORCHESTRATION_API_URL=http://localhost:8002 && export VITE_AGENTS_API_URL=http://localhost:8003 && npm run develop" Enter
tmux send-keys -t $SESSION_NAME:frontend.1 "cd svc/hecate && tail -f hecate.log" Enter

# LLM window
tmux new-window -t $SESSION_NAME -n llm
tmux split-window -h -t $SESSION_NAME:llm
tmux split-window -h -t $SESSION_NAME:llm
tmux select-layout -t $SESSION_NAME:llm even-horizontal

# Send commands to LLM panes
tmux send-keys -t $SESSION_NAME:llm.0 "echo 'Starting LM Studio...' && lms status && echo 'Loading Gemma3 270M...' && lms load gemma-3-270m-it-mlx -y && echo 'Model loaded successfully! Starting API server...' && lms server start" Enter
tmux send-keys -t $SESSION_NAME:llm.1 "echo 'Waiting for LM Studio server to start...' && sleep 10 && echo 'Streaming LM Studio logs...' && lms log stream" Enter
tmux send-keys -t $SESSION_NAME:llm.2 "echo 'Starting LM Studio monitoring...' && sleep 3 && ./scripts/monitor-lmstudio.sh" Enter

# Monitoring window
tmux new-window -t $SESSION_NAME -n monitoring
tmux split-window -h -t $SESSION_NAME:monitoring
tmux select-layout -t $SESSION_NAME:monitoring even-horizontal

# Send commands to monitoring panes
tmux send-keys -t $SESSION_NAME:monitoring.0 "echo 'Health Check - Services:'" Enter
tmux send-keys -t $SESSION_NAME:monitoring.0 "echo 'MCP: http://localhost:8001/health'" Enter
tmux send-keys -t $SESSION_NAME:monitoring.0 "echo 'Orchestration: http://localhost:8002/health'" Enter
tmux send-keys -t $SESSION_NAME:monitoring.0 "echo 'Agents: http://localhost:8003/health'" Enter
tmux send-keys -t $SESSION_NAME:monitoring.0 "echo 'Erebus: http://localhost:3000/health'" Enter
tmux send-keys -t $SESSION_NAME:monitoring.0 "echo 'Frontend: http://localhost:5173'" Enter
tmux send-keys -t $SESSION_NAME:monitoring.0 "echo 'LM Studio: http://localhost:1234/v1/models'" Enter
tmux send-keys -t $SESSION_NAME:monitoring.1 "./scripts/quick-lmstudio-check.sh" Enter

# Tools window
tmux new-window -t $SESSION_NAME -n tools
tmux split-window -h -t $SESSION_NAME:tools
tmux select-layout -t $SESSION_NAME:tools even-horizontal

# Send commands to tools panes
tmux send-keys -t $SESSION_NAME:tools.0 "git status" Enter
tmux send-keys -t $SESSION_NAME:tools.1 "echo 'Development shell ready...'" Enter
tmux send-keys -t $SESSION_NAME:tools.1 "echo 'Useful commands:'" Enter
tmux send-keys -t $SESSION_NAME:tools.1 "echo 'just test      - Test setup'" Enter
tmux send-keys -t $SESSION_NAME:tools.1 "echo 'just status    - Check services'" Enter
tmux send-keys -t $SESSION_NAME:tools.1 "echo 'just health    - Health check'" Enter
tmux send-keys -t $SESSION_NAME:tools.1 "echo 'just logs      - View logs'" Enter

# Go back to infrastructure window
tmux select-window -t $SESSION_NAME:infrastructure

# Attach to session
echo "Development environment created! Attaching to tmux session..."
tmux attach-session -t $SESSION_NAME
