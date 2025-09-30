# CLAUDE.md

```
 _   _       _ _ ____  _            _
| \ | |_   _| | | __ )| | ___   ___| | __
|  \| | | | | | |  _ \| |/ _ \ / __| |/ /
| |\  | |_| | | | |_) | | (_) | (__|   <
|_| \_|\__,_|_|_|____/|_|\___/ \___|_|\_\
```

**🎯 Mission**:
In a rapidly expanding onchain automated world, we are building the picks and axes for this digital gold rush.
NullBlock empowers builders with the essential tools to create, deploy, and profit from intelligent agent workflows.
Together, we shape the future of autonomous commerce.

## 🌐 Connect & Follow

**🏢 Official**: [@Nullblock_io](https://x.com/Nullblock_io)
**📦 SDK**: [nullblock-sdk](https://github.com/aetherBytes/nullblock-sdk)
**🌍 Site**: NullBlock.io _(Coming Soon)_

**NullBlock** is a revolutionary agentic platform that democratizes AI automation. Built on Rust for performance and reliability, it enables users to create, deploy, and monetize intelligent agent workflows without complex infrastructure. Protocol-agnostic design seamlessly integrates with any system - from MCP servers to custom APIs.

### 🚧 Current Development Focus

**Next 3 Priority Items:**

1. **✅ A2A Task Schema Alignment** - COMPLETED: Task database schema and models now fully comply with A2A Protocol v0.3.0 (context_id, kind, status object, history, artifacts, Message/Part types)
2. **🔄 A2A Core Operations** - Connect protocol handlers to Agents service: message/send, tasks/get, tasks/list, tasks/cancel. Implement format converters between internal Task model and A2A protocol format.
3. **🔄 A2A Streaming (SSE)** - Implement Server-Sent Events for message/stream and tasks/resubscribe. Bridge Kafka task.lifecycle events to SSE streams for real-time updates.

## Architecture Overview

```
┌─────────────┐    ┌──────────────┐    ┌─────────────────┐
│   Frontend  │    │    Erebus    │    │   Backend       │
│   (Hecate)  │◄──►│   Router     │◄──►│   Services      │
│   Port 5173 │    │   Port 3000  │    │   Various Ports │
└─────────────┘    └──────────────┘    └─────────────────┘
                           │
                   ┌───────┴────────┐
                   │   Crossroads   │
                   │  Marketplace   │
                   │   (Internal)   │
                   └────────────────┘
```

### 🚀 Key Features

- **🤖 Agent Orchestration**: Multi-model LLM coordination via Hecate
- **🛣️ Unified Router**: Single entry point through Erebus (Port 3000)
- **💰 Marketplace**: Crossroads AI service discovery and monetization
- **🔗 Protocol Agnostic**: A2A (Agent-to-Agent), MCP (Model Context Protocol), custom protocols
- **⚡ Real-time**: WebSocket chat, live task management, instant feedback

## 🎯 Core Services

### Production-Ready ✅

- **NullBlock.protocols** (`/svc/nullblock-protocols/`): Multi-protocol server (A2A, MCP)
- **NullBlock.agents** (`/svc/nullblock-agents/`): Agent suite (Hecate orchestrator, trading, monitoring, LLM)
- **Erebus** (`/svc/erebus/`): Unified routing server
- **Crossroads** (`/svc/erebus/src/resources/crossroads/`): Marketplace subsystem
- **Hecate Frontend** (`/svc/hecate/`): React interface with real-time agent discovery

### Legacy (Transitioning)

- **Helios** → Replaced by NullBlock.protocols
- **NullBlock.orchestration** → Integrated into agents

## 🚀 Quick Start

```bash
./scripts/dev-tmux

# Individual services:
# cd svc/nullblock-protocols && cargo run  # Port 8001
# cd svc/nullblock-agents && cargo run     # Port 9003
# cd svc/erebus && cargo run               # Port 3000
# cd svc/hecate && npm run develop         # Port 5173
```

### Key Ports

- **3000**: Erebus (unified router + Crossroads)
- **5173**: Hecate frontend
- **8001**: Protocol server (A2A/MCP)
- **9003**: Hecate agent API

## 🏗️ Architecture Rules

### Erebus Unified Router (Port 3000) - GOLDEN RULES

🚨 **CRITICAL**: ALL frontend communication MUST route through Erebus. NO direct service connections.

```
Frontend → Erebus → {
  Wallet operations → Internal handlers
  Agent chat → Hecate (9003)
  A2A/MCP → Protocols (8001)
  Marketplace → Crossroads (internal)
}
```

🚨 **CRITICAL**: EREBUS OWNS ALL USER CRUD (Port 5440) - Use `/api/users/*` endpoints ONLY. NO direct database access.

### API Endpoints

- **🔐 Users**: `/api/users/*` - Registration, lookup, management (EREBUS OWNED)
- **👛 Wallets**: `/api/wallets/*` - Authentication, sessions
- **🤖 Agents**: `/api/agents/*` - Chat, status, orchestration
- **📋 Tasks**: `/api/agents/tasks/*` - Task management, lifecycle
- **🔗 Protocols**: `/api/protocols/*` - A2A/MCP operations
- **🛣️ Marketplace**: `/api/marketplace/*` - Listings, search
- **🔍 Discovery**: `/api/discovery/*` - Service discovery, health
- **⚙️ Admin**: `/api/admin/*` - Moderation, management

## 🔐 User Authentication

### Source-Agnostic System ✅

Supports: **Web3 Wallets**, **API Keys**, **Email Auth**, **OAuth**, **System Agents**

#### Web3 Wallet Flow

```
1. Wallet Connect → 2. POST /api/wallets/challenge
3. User signs → 4. POST /api/wallets/verify
5. Auto POST /api/users/register → 6. Kafka sync → 7. Session token
```

### User Endpoints (Erebus Port 3000)

- **POST `/api/users/register`** - Create/update user (all source types)
- **POST `/api/users/lookup`** - Find by source identifier + network
- **GET `/api/users/:user_id`** - Get by UUID

## 🤖 Agent System

### Hecate Agent

- **Purpose**: Main conversational interface, orchestration engine
- **Default Model**: DeepSeek Chat v3.1 Free ($0.00)
- **Features**: Multi-model LLM, intent analysis, delegation, task management
- **Timeout**: 5-minute for thinking models

### LLM Factory

- **Providers**: OpenRouter, OpenAI, Anthropic, Groq, HuggingFace
- **Strategies**: Quality, speed, cost, balanced

### Specialized Agents

- **Information Gathering**: Market data, DeFi, social sentiment
- **Social Trading**: Twitter monitoring, sentiment, risk
- **Arbitrage**: Price monitoring, MEV protection

## 🌐 A2A Protocol

NullBlock implements [A2A Protocol v0.3.0](https://a2a-protocol.org/latest/specification/)

### Implementation Status

**✅ Completed:**
- Task schema aligned with A2A spec (Task, TaskStatus, TaskState, Message, Artifact, Part types)
- Agent Card with full schema compliance
- JSON-RPC 2.0 and REST/HTTP+JSON endpoints defined
- All 11 protocol methods scaffolded

**🔄 In Progress:**
- Handler implementations (currently stubs)
- Server-Sent Events (SSE) streaming
- Service integration (Protocols ↔ Agents ↔ Erebus)

**❌ Not Implemented:**
- Push notifications (webhook system)
- Authentication middleware (security schemes)
- Format converters (A2A ↔ internal models)

### Endpoints

**JSON-RPC** (POST /a2a/jsonrpc): `message/send`, `message/stream`, `tasks/*`, `tasks/pushNotificationConfig/*`, `agent/getAuthenticatedExtendedCard`
**REST**: `/a2a/v1/card`, `/a2a/v1/messages`, `/a2a/v1/tasks/*`

## 📋 Task Management

### Implementation ✅

- **Storage**: PostgreSQL with full CRUD
- **Schema**: A2A Protocol v0.3.0 compliant (context_id, kind, status object, history, artifacts)
- **States**: submitted, working, input-required, completed, canceled, failed, rejected, auth-required, unknown
- **Events**: Kafka streaming (task.lifecycle topic)
- **Lifecycle**: create → start → process → complete
- **Processing**: `/api/agents/tasks/:id/process` endpoint
- **Tracking**: `actioned_at`, `action_result`, `action_duration`
- **Integration**: Hecate auto-assigned, chat display

### Database Architecture

**Erebus DB (5440)** - OWNS user_references
**Agents DB (5441)** - OWNS tasks, agents, READ-ONLY user_references replica

**Sync**: PostgreSQL logical replication (`erebus_user_sync` publication → `agents_user_sync` subscription)

**Benefits**: Service isolation, real-time sync (<1s), 99.9% reliability, zero maintenance

## 🛣️ Crossroads Marketplace

"Craigslist for AI Agents"

### Features

- **Listings**: Agents, workflows, tools, MCP servers
- **Discovery**: Auto-discovery, health monitoring
- **Search**: Advanced filtering, featured content
- **Admin**: Moderation, quality control

### Endpoints

- `/api/marketplace/*` - Listings, search, stats
- `/api/discovery/*` - Agents, workflows, MCP servers, health
- `/api/admin/*` - Approve, reject, feature

## 📋 Common Commands

```bash
# Quality
cargo fmt && cargo clippy          # Rust
ruff format . && ruff check . --fix # Python

# Testing
cargo test                          # Rust
pytest -v                           # Python

# Database
docker-compose up postgres kafka zookeeper -d
export DATABASE_URL="postgresql://postgres:postgres_secure_pass@localhost:5441/agents"
export KAFKA_BOOTSTRAP_SERVERS="localhost:9092"

# Monitoring
tail -f svc/nullblock-agents/logs/chats/hecate-chat.log
tail -f svc/erebus/logs/erebus.log
docker exec -it nullblock-kafka kafka-console-consumer --bootstrap-server localhost:9092 --topic task.lifecycle --from-beginning

# API Testing
curl http://localhost:3000/api/discovery/agents
curl -X POST http://localhost:3000/api/agents/tasks -H "Content-Type: application/json" -d '{"name":"Test","description":"Test task","task_type":"system","category":"user_assigned","priority":"medium"}'
```

## ⚠️ Organizational Rules

### Code Standards

- **NEVER** add comments unless explicitly requested
- **ALWAYS** prefer editing existing files over creating new
- **NEVER** proactively create documentation files
- Follow existing conventions and patterns

### Architecture Enforcement

- **ALL** frontend requests → Erebus (3000) - NO EXCEPTIONS
- **ALL** user CRUD → Erebus `/api/users/*` - NO direct DB access
- Keep CLAUDE.md top section static and updated
- NEVER use test credentials in production
- Subsystems in own directories (wallets/, mcp/, agents/)

## 🎨 UI/UX Standards

- **NullEye Animations**: 8 lightning arcs, silver-gold (#e8e8e8, #e6c200)
- **State-Responsive**: Colors change based on agent state
- **Responsive**: 4-column grids optimized for small screens
- **Gentle Messaging**: Blue info (#4a90e2), red errors (#ff3333)

## 🔧 Environment Variables

### Backend

```bash
EREBUS_BASE_URL=http://localhost:3000
PROTOCOLS_SERVICE_URL=http://localhost:8001
AGENTS_SERVICE_URL=http://localhost:9003
DATABASE_URL=postgresql://postgres:postgres_secure_pass@localhost:5441/agents
EREBUS_DATABASE_URL=postgresql://postgres:postgres_secure_pass@localhost:5440/erebus
KAFKA_BOOTSTRAP_SERVERS=localhost:9092
OLLAMA_BASE_URL=http://localhost:11434
```

### Frontend

```bash
VITE_EREBUS_API_URL=http://localhost:3000
VITE_PROTOCOLS_API_URL=http://localhost:8001
VITE_HECATE_API_URL=http://localhost:9003
CORS_ORIGINS=http://localhost:5173,http://localhost:3000
```

### LLM APIs

```bash
OPENAI_API_KEY=
ANTHROPIC_API_KEY=
GROQ_API_KEY=
OPENROUTER_API_KEY=
DEFAULT_LLM_MODEL=x-ai/grok-4-fast:free
LLM_REQUEST_TIMEOUT_MS=300000
```

## 💰 Monetization

- **Financial Automation**: 0.5-1% fees
- **Content & Communication**: $10-$100/month
- **Data Intelligence**: $50-$500/month
- **Marketplace Fee**: 5-10% revenue share
- **Task Execution**: $0.01-$0.05 per task
- **Premium Hosting**: $10-$100/month

## 🚧 Development Status

### Recently Completed ✅

- A2A Protocol v0.3.0 task schema alignment (context_id, kind, status object, history, artifacts)
- TaskState enum with all A2A states (submitted, working, input-required, etc.)
- Message and Artifact types with Part union (Text, File, Data)
- Database migration applied with A2A-compliant schema
- Source-agnostic user system with SourceType enum
- PostgreSQL logical replication for user sync

### In Progress 🔄

- A2A protocol handler implementations
- Service integration (Protocols ↔ Agents ↔ Erebus)
- Format converters (A2A ↔ internal models)

### Next Up 📋

**Phase 1 - Core Operations (High Priority):**
1. Connect message/send to Agents service (create tasks, route to agents)
2. Implement tasks/get, tasks/list, tasks/cancel with database integration
3. Build A2A ↔ Internal Task format converters

**Phase 2 - Streaming (High Priority):**
4. Implement Server-Sent Events (SSE) for message/stream
5. Build Kafka → SSE bridge for real-time task updates
6. Implement tasks/resubscribe for resuming streams

**Phase 3 - Push Notifications (Medium Priority):**
7. Create push_notification_configs database table
8. Implement webhook delivery system with retry logic
9. Add event filtering and webhook authentication

**Phase 4 - Security (Medium Priority):**
10. Implement authentication middleware
11. Add security scheme support (API Key, OAuth2, Bearer tokens)
12. Agent Card signatures (JWS) for integrity verification

**Phase 5 - Polish (Lower Priority):**
13. Standardize error handling (JSON-RPC error codes)
14. State transition history tracking
15. Extensions support
16. Comprehensive A2A compliance testing

---

_NullBlock implements a cyberpunk aesthetic with neon styling and maintains immersive error messages throughout the user experience while building toward a comprehensive MCP-powered agentic ecosystem._
- add the above todos to memory and the claude md