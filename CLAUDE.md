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
- **Crossroads** (`/svc/erebus/src/resources/crossroads/`): Marketplace and discovery subsystem integrated into Erebus
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
- **3000**: Erebus (unified backend router + Crossroads marketplace)
- **5173**: Hecate frontend (development)
- **8001**: MCP server
- **9001**: General agents API
- **9003**: Hecate agent API (Rust service)

## 🏗️ Architecture

### Erebus Unified Router (Port 3000) - GOLDEN RULE
🚨 **CRITICAL ARCHITECTURE RULE**: ALL frontend communication MUST route through Erebus. NO direct service connections allowed.

```
Frontend → Erebus → {
  Wallet operations → Internal wallet handlers
  Agent chat → Hecate agent (port 9003)  
  Agent search → Hecate agent (port 9003)
  MCP operations → MCP server (port 8001)
  Marketplace operations → Crossroads subsystem (internal)
}
```

**NEVER allow frontend to bypass Erebus by connecting directly to:**
- Hecate agent (port 9003)
- MCP server (port 8001)
- Any other backend services
- Crossroads marketplace is now INTERNAL to Erebus (no separate port)

**This prevents CORS issues and maintains proper request routing/logging.**

### Key API Endpoints
- **Wallets**: `/api/wallets/*` - Authentication, session management
- **Agents**: `/api/agents/*` - Chat, status, orchestration
- **MCP**: `/mcp/*` - Protocol operations
- **Marketplace**: `/api/marketplace/*` - Listing management, search, featured items
- **Discovery**: `/api/discovery/*` - Service discovery, health monitoring
- **Admin**: `/api/admin/*` - Marketplace moderation, system management
- **Health**: `/health` - Service status

### Directory Structure
```
svc/erebus/src/resources/
├── wallets/          # 👛 Wallet subsystem (MetaMask, Phantom)
├── agents/           # 🤖 Agent routing & proxy
├── mcp/              # 🔗 MCP protocol handlers
├── templates/        # 🔒 RESERVED - MCP templates
└── definitions/      # 🔒 RESERVED - MCP schemas

svc/erebus/src/resources/crossroads/
├── routes.rs         # 🛣️ API endpoints (marketplace, discovery, admin)  
├── services/         # 📦 Business logic (marketplace, discovery, health)
├── models.rs         # 🗂️ Data structures and types
└── mod.rs            # 📦 Module integration
```

## 🤖 Agent System

### Hecate Agent (Primary Interface)
- **Purpose**: Main conversational interface and orchestration engine
- **Default Model**: DeepSeek Chat v3.1 Free (cost: $0.00) for all conversations
- **Features**: Multi-model LLM support, intent analysis, agent delegation
- **Integration**: Full frontend chat with real-time model display
- **Chat Logging**: Real-time conversation logs with timestamps, model info, and cost tracking
- **Logging**: Standardized cyberpunk-themed logs in `logs/` directory

### LLM Service Factory
- **Cloud Models**: OpenRouter (DeepSeek, GPT-4o, Claude), OpenAI, Anthropic, Groq, HuggingFace
- **Default Model**: DeepSeek Chat v3.1 Free ($0.00/request) for cost-effective testing
- **Routing**: Automatic model selection based on task requirements
- **Optimization**: Quality, speed, cost, or balanced strategies
- **Timeout Configuration**: 5-minute timeout for thinking models (DeepSeek-R1, etc.) to handle complex reasoning

### Specialized Agents
- **Information Gathering**: Market data, DeFi protocols, social sentiment
- **Social Trading**: Twitter monitoring, sentiment analysis, risk assessment
- **Arbitrage**: Price monitoring, strategy execution with MEV protection

## 🛣️ Crossroads Marketplace System

### Core Purpose - **"Craigslist for AI Agents"**
- **Focused Marketplace**: Simple listing and discovery of agents, workflows, tools, and MCP servers
- **Service Discovery**: Automatic discovery and cataloging of available Nullblock services
- **Integration Hub**: Connect with other Erebus subsystems for advanced functionality
- **Unified Interface**: Single place to find and list AI services in the ecosystem

### Core Features

#### Marketplace Operations
- **Listing Management**: Create, update, approve, delete marketplace listings
- **Advanced Search**: Filter by type, tags, author, rating, price with full-text search
- **Featured Listings**: Curated showcase of premium content
- **Service Integration**: Connects to Nullblock Agent/MCP/Orchestration services for data

#### Service Discovery Engine
- **Agent Discovery**: Finds agents from Nullblock Agents service (port 9001)
- **Workflow Discovery**: Finds workflows from Orchestration service (port 8002)
- **MCP Server Discovery**: Finds MCP servers from MCP service (port 8001)
- **Health Monitoring**: Continuous health checks and service availability tracking
- **Real-time Scanning**: On-demand discovery scans with performance metrics

#### Marketplace Administration
- **Listing Moderation**: Approve, reject, and feature marketplace listings
- **Quality Control**: Ensure marketplace integrity and content standards
- **Admin Dashboard**: Administrative controls for marketplace management

### API Endpoints (via Erebus port 3000)

#### Core Marketplace
- **`/api/marketplace/listings`**: CRUD operations for listings
- **`/api/marketplace/search`**: Advanced search functionality
- **`/api/marketplace/featured`**: Featured content management
- **`/api/marketplace/stats`**: Marketplace statistics and metrics

#### Service Discovery  
- **`/api/discovery/agents`**: Agent discovery with Nullblock service integration
- **`/api/discovery/workflows`**: Workflow discovery from orchestration service
- **`/api/discovery/mcp-servers`**: MCP server discovery and scanning
- **`/api/discovery/scan`**: Trigger full discovery scans
- **`/api/discovery/health/:endpoint`**: Check individual service health

#### Administration
- **`/api/admin/listings/approve/:id`**: Approve marketplace listings
- **`/api/admin/listings/reject/:id`**: Reject marketplace listings  
- **`/api/admin/listings/feature/:id`**: Feature marketplace listings

#### Health & Status
- **`/api/crossroads/health`**: Crossroads subsystem health monitoring

### Service Integration Architecture
Crossroads integrates with other Nullblock services for extended functionality:

- **For MCP Operations**: Use MCP service endpoints (`/svc/nullblock-mcp/`)
- **For Blockchain/Tokenization**: Use dedicated blockchain service (to be implemented)
- **For Wealth Distribution**: Use dedicated rewards service (to be implemented)  
- **For Agent Interoperability**: Use extended Agents service functionality

### Integration Benefits
- **Focused Scope**: Clean separation of marketplace vs. advanced functionality
- **Service Composition**: Leverage other Erebus subsystems for complex operations
- **Unified Routing**: All requests go through Erebus logging and middleware
- **CORS Compliance**: No cross-origin issues since everything routes through port 3000
- **Clear Responsibilities**: Marketplace discovery vs. service-specific functionality

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

# Monitor chat conversations (real-time)
tail -f svc/nullblock-agents/logs/chats/hecate-chat.log

# View chat session data (JSON format)
cat svc/nullblock-agents/logs/chats/session_*.jsonl
```

### Crossroads Marketplace Development
```bash
# Start Erebus server (includes Crossroads)
cd svc/erebus && cargo run

# Monitor Erebus logs (includes Crossroads operations)
tail -f svc/erebus/logs/erebus.log

# Test Crossroads endpoints via Erebus
curl http://localhost:3000/api/crossroads/health
curl http://localhost:3000/api/marketplace/listings
curl http://localhost:3000/api/discovery/agents
```

### Chat Logging Structure
- **Real-time Chat Log**: `svc/nullblock-agents/logs/chats/hecate-chat.log`
  - Human-readable format with timestamps, emojis, and model info
  - Continuous log of all conversations across sessions
- **Session-specific Logs**: `svc/nullblock-agents/logs/chats/session_*.jsonl`
  - Structured JSON data with full metadata
  - Individual file per agent startup session
  - Includes user context, model costs, latency metrics

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
- **GOLDEN RULE**: ALL frontend requests MUST route through Erebus (port 3000) - NO EXCEPTIONS
- **main.rs**: Only subsystem entry points and core routes
- **Subsystem Organization**: Each feature gets own directory (wallets/, mcp/, agents/)
- **Wallet Subsystem**: All wallets implement `WalletProvider` trait
- **Shared Types**: Use `resources/types.rs` for cross-subsystem types
- **Agent Timeout**: 5-minute proxy timeout for thinking models and complex agent operations
- **Frontend Discipline**: If you see direct service calls (localhost:9003, localhost:8001) in frontend code, FIX IMMEDIATELY by routing through Erebus

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