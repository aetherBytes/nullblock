# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

NullBlock is an Agentic Platform providing resources and tooling for building, deploying, and monetizing intelligent agent workflows. Built on Model Context Protocol (MCP) architecture, it enables agents to interact with various systems through standardized interfaces.

**Core Architecture**: Client ↔ NullBlock (Agentic Platform) ↔ Server (Web3 wallets, APIs, etc.)

## 🎯 Current System Status

### Production-Ready Components ✅
- **NullBlock.mcp** (`/svc/nullblock-mcp/`): Complete MCP server with authentication, context storage, security middleware
- **NullBlock.agents** (`/svc/nullblock-agents/`): Agent suite including Hecate orchestrator, trading, monitoring, LLM coordination
- **Erebus** (`/svc/erebus/`): Unified routing server for wallet interactions and agent communication
- **Hecate Frontend** (`/svc/hecate/`): React interface with agent integration

### Legacy Components (Transitioning)
- **Helios** (`/svc/helios/`): Original FastAPI backend → **Replaced by NullBlock.mcp**
- **NullBlock.orchestration** (`/svc/nullblock-orchestration/`): Workflow engine → **Integrated into agents**

## 🚀 Quick Start

### Complete Development Environment
```bash
# Start all services with tmux
./scripts/dev-tmux

# Individual service startup:
# - MCP Server: cd svc/nullblock-mcp && python -m mcp.server
# - Hecate Agent: cd svc/nullblock-agents && python -m agents.hecate.server  
# - Erebus Server: cd svc/erebus && cargo run
# - Frontend: cd svc/hecate && npm run develop
```

### Key Ports
- **3000**: Erebus (unified backend router)
- **5173**: Hecate frontend (development)
- **8001**: MCP server
- **9001**: General agents API
- **9002**: Hecate agent API

## 🏗️ Architecture

### Erebus Unified Router (Port 3000)
All frontend communication routes through Erebus, which proxies to appropriate services:

```
Frontend → Erebus → {
  Wallet operations → Internal wallet handlers
  Agent chat → Hecate agent (port 8001)
  MCP operations → MCP server (port 8000)
}
```

### Key API Endpoints
- **Wallets**: `/api/wallets/*` - Authentication, session management
- **Agents**: `/api/agents/*` - Chat, status, orchestration
- **MCP**: `/mcp/*` - Protocol operations
- **Health**: `/health` - Service status

### Directory Structure
```
svc/erebus/src/resources/
├── wallets/          # 👛 Wallet subsystem (MetaMask, Phantom)
├── agents/           # 🤖 Agent routing & proxy
├── mcp/              # 🔗 MCP protocol handlers
├── templates/        # 🔒 RESERVED - MCP templates
└── definitions/      # 🔒 RESERVED - MCP schemas
```

## 🤖 Agent System

### Hecate Agent (Primary Interface)
- **Purpose**: Main conversational interface and orchestration engine
- **Features**: Multi-model LLM support, intent analysis, agent delegation
- **Integration**: Full frontend chat with real-time model display
- **Logging**: Standardized cyberpunk-themed logs in `logs/` directory

### LLM Service Factory
- **Cloud Models**: OpenRouter (DeepSeek, GPT-4o, Claude), OpenAI, Anthropic, Groq, HuggingFace
- **Routing**: Automatic model selection based on task requirements
- **Optimization**: Quality, speed, cost, or balanced strategies
- **Timeout Configuration**: 5-minute timeout for thinking models (DeepSeek-R1, etc.) to handle complex reasoning

### Specialized Agents
- **Information Gathering**: Market data, DeFi protocols, social sentiment
- **Social Trading**: Twitter monitoring, sentiment analysis, risk assessment
- **Arbitrage**: Price monitoring, strategy execution with MEV protection

## 📋 Common Commands

### Service Management
```bash
# Code quality (all Python services)
ruff format . && ruff check . --fix && mypy .

# Code quality (Rust)
cargo fmt && cargo clippy

# Testing
pytest -v                    # Python services
cargo test                   # Rust services
```

### Development Servers
```bash
# Python services
python -m [service_name]
uvicorn [module]:app --reload

# Rust services  
cargo run
cargo watch -x run          # Auto-reload

# Frontend
npm run develop
ssr-boost dev
```

### Agent Development
```bash
# Start Hecate agent server
cd svc/nullblock-agents && python -m agents.hecate.server

# Run demos
python demo_integration.py
python -m agents.information_gathering.demo

# Monitor logs
tail -f logs/hecate-server.log
```

## ⚠️ Organizational Rules

### Reserved Directories (Do Not Modify)
- `svc/erebus/src/resources/templates/` - MCP template definitions
- `svc/erebus/src/resources/definitions/` - MCP type definitions and schemas

### Code Standards
- **NEVER** add comments unless explicitly requested
- **ALWAYS** prefer editing existing files over creating new ones
- **NEVER** proactively create documentation files
- Follow existing code conventions and patterns
- Use existing libraries already present in the codebase

### Erebus Architecture Rules
- **main.rs**: Only subsystem entry points and core routes
- **Subsystem Organization**: Each feature gets own directory (wallets/, mcp/, agents/)
- **Wallet Subsystem**: All wallets implement `WalletProvider` trait
- **Shared Types**: Use `resources/types.rs` for cross-subsystem types
- **Agent Timeout**: 5-minute proxy timeout for thinking models and complex agent operations

## 🎨 UI/UX Standards

### Visual Design
- **NullEye Animations**: 8 randomized lightning arcs with silver-gold effects (#e8e8e8, #e6c200)
- **State-Responsive**: Colors change based on agent state, red/dimmed when offline
- **Responsive Design**: 4-column grids optimized for small screens
- **Gentle Messaging**: Blue info (#4a90e2), red errors (#ff3333), no aggressive animations

### User Interface
- **Command Lens**: Compact grid with instant access styling
- **Intelligent Tooltips**: Hover-based help system
- **HecateHud**: Personalized user stats (wallet info, session time, connection status)
- **Universal Navigation**: All NullEyes route to agent interfaces

## 🔧 Environment Setup

### Required Environment Variables
```bash
# MCP Server
ETHEREUM_RPC_URL=
FLASHBOTS_PRIVATE_KEY=
IPFS_API=

# Frontend  
VITE_FAST_API_BACKEND_URL=http://localhost:3000

# Optional LLM APIs
OPENAI_API_KEY=
ANTHROPIC_API_KEY=
```

### Health Monitoring
All services implement `/health` endpoints with standardized JSON responses:
```json
{
  "status": "healthy|unhealthy",
  "service": "service-name",
  "timestamp": "2025-08-20T...",
  "components": {...}
}
```

## 💰 Monetization Strategy

### Revenue Streams
- **Financial Automation**: 0.5-1% fees on trading, portfolio management
- **Content & Communication**: $10-$100/month subscriptions
- **Data Intelligence**: $50-$500/month for analysis and insights
- **Workflow Automation**: $25-$250/month for complex agent workflows

### Platform Revenue
- **Marketplace Fee**: 5-10% of user-created agent revenue
- **Task Execution**: $0.01-$0.05 per automated task
- **Premium Hosting**: $10-$100/month for advanced features

---

*NullBlock implements a cyberpunk aesthetic with neon styling and maintains immersive error messages throughout the user experience while building toward a comprehensive MCP-powered agentic ecosystem.*