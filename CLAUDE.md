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

1. **✅ A2A Task Schema & Integration** - COMPLETED: Full A2A Protocol v0.3.0 compliance with task schema, repository methods, handler population of history/artifacts, and Protocols→Agents HTTP integration
2. **🔄 Compilation & Testing** - Fix remaining Axum router state type issues in protocols service, test end-to-end A2A task flow (create→process→retrieve via A2A endpoints)
3. **📋 A2A Streaming (SSE)** - Implement Server-Sent Events for message/stream and tasks/resubscribe, bridge Kafka task.lifecycle events to SSE streams for real-time updates

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

**"App Store for AI Services"** - Web3-native marketplace for discovering, deploying, and monetizing AI agents, workflows, tools, and MCP servers.

### Architecture

**Location**: `/svc/erebus/src/resources/crossroads/`
**Integration**: Native Erebus subsystem (no separate service)
**Frontend**: `/svc/hecate/src/components/crossroads/`

### Features

**Discovery & Marketplace:**
- Browse agents, workflows, tools, MCP servers, datasets, models
- Full-text search with PostgreSQL indexing
- Advanced filtering (category, price, rating, tags)
- Featured listings and trending services
- Real-time health monitoring

**Publishing:**
- Multi-step wizard for service submission
- Configuration schemas and validation
- Pricing models (Free, Subscription, OneTime, PayPerUse, TokenStaking)
- Automatic discovery integration

**User Management:**
- Published services dashboard
- Deployed services monitoring
- Analytics and earnings tracking
- Reviews and ratings

**Web3 Integration:**
- OnchainKit Identity (ENS/Basename)
- Wallet-gated features
- On-chain payments and transactions
- Service ownership verification

### Database Schema

**Tables:**
- `crossroads_listings` - Service marketplace listings
- `crossroads_reviews` - User reviews and ratings
- `crossroads_deployments` - Active service instances
- `crossroads_favorites` - User bookmarks
- `crossroads_discovery_scans` - Discovery operation tracking
- `crossroads_analytics_events` - User interaction analytics

### API Endpoints

**Marketplace:**
- `GET/POST /api/marketplace/listings` - Browse/create listings
- `GET /api/marketplace/listings/:id` - Service details
- `POST /api/marketplace/search` - Advanced search
- `GET /api/marketplace/featured` - Featured services
- `GET /api/marketplace/stats` - Marketplace statistics

**Discovery:**
- `GET /api/discovery/agents` - Auto-discover agents
- `GET /api/discovery/workflows` - Auto-discover workflows
- `GET /api/discovery/tools` - Auto-discover tools
- `GET /api/discovery/mcp-servers` - Auto-discover MCP servers
- `POST /api/discovery/scan` - Trigger discovery scan
- `GET /api/discovery/health/:endpoint` - Service health check

**Deployments:**
- `POST /api/marketplace/listings/:id/deploy` - Deploy service
- `GET /api/marketplace/deployments` - User deployments
- `POST /api/marketplace/deployments/:id/start` - Start instance
- `POST /api/marketplace/deployments/:id/stop` - Stop instance

**Reviews & Social:**
- `GET/POST /api/marketplace/listings/:id/reviews` - Reviews
- `POST/DELETE /api/marketplace/listings/:id/favorite` - Favorites

**Admin:**
- `POST /api/admin/listings/approve/:id` - Approve listing
- `POST /api/admin/listings/reject/:id` - Reject listing
- `POST /api/admin/listings/feature/:id` - Feature listing

### Design Documentation

See `CROSSROADS_UI_DESIGN.md` and `CROSSROADS_BACKEND_PLAN.md` for complete architecture, component breakdown, and implementation roadmap.

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
export DATABASE_URL="postgresql://postgres:REDACTED_DB_PASS@localhost:5441/agents"
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
DATABASE_URL=postgresql://postgres:REDACTED_DB_PASS@localhost:5441/agents
EREBUS_DATABASE_URL=postgresql://postgres:REDACTED_DB_PASS@localhost:5440/erebus
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

**A2A Protocol v0.3.0 Integration (Full Stack):**
- ✅ Database schema with A2A fields (context_id, kind, status object, history JSONB, artifacts JSONB)
- ✅ TaskState enum with all 9 A2A states (submitted, working, input-required, completed, canceled, failed, rejected, auth-required, unknown)
- ✅ Message and Artifact types with Part union (Text, File, Data) in protocols service
- ✅ Repository methods: add_message_to_history(), add_artifact(), update_status_with_message()
- ✅ Task handlers populate history on creation (initial user message) and completion (agent response + artifact)
- ✅ Hecate agent execution adds A2A-compliant messages and artifacts with metadata
- ✅ Protocols service HTTP integration: get_task, list_tasks, cancel_task, resubscribe_task now proxy to Agents service (port 9003)
- ✅ Frontend TypeScript types updated with A2A interfaces (TaskState, TaskStatus, A2AMessage, A2AArtifact)
- ✅ **Axum 0.7 Router State Fix** - Fixed Router<AppState> → Router<()> type mismatch by calling .with_state() BEFORE .layer() middleware
- ✅ **Protocols Service Compilation** - All services now compile and run successfully (Port 8001)
- ✅ **A2A Endpoint Testing** - Validated end-to-end task flow: Create via Agents → Retrieve via A2A with history array populated
- ✅ **Protocols README** - Comprehensive documentation in svc/nullblock-protocols/README.md

**Infrastructure:**
- ✅ Source-agnostic user system with SourceType enum
- ✅ PostgreSQL logical replication for user sync (Erebus→Agents)

### In Progress 🔄

- 🔄 **Task State Naming Mismatch** - A2A protocol uses "working" state, but Hecate process endpoint expects "created" or "running" - need to align state transitions and validate task lifecycle
- 🔄 **Hecate Auto-Processing** - Tasks created with auto_start=true transition to "working" state but don't automatically process with Hecate agent

### Next Up 📋

**Immediate (Unblock Development):**
1. **Fix Task State Alignment** - Reconcile A2A state names ("working") with internal processing states ("running", "created"). Update task handlers to accept A2A states or create state mapping layer.
2. **Hecate Auto-Processing Flow** - Implement automatic task processing when auto_start=true. Current behavior: task transitions to "working" but Hecate doesn't execute. Need to trigger agent processing on state change.
3. **Task Processing Endpoint** - Update `/tasks/:id/process` to accept A2A states or add state normalization before validation
4. **Validate Artifact Population** - Once processing works, confirm Hecate adds completion artifacts with metadata (model, duration) to artifacts array

**Phase 1 - Streaming & Real-time (High Priority):**
5. Implement Server-Sent Events (SSE) for message/stream endpoint
6. Build Kafka → SSE bridge: Subscribe to task.lifecycle topic, stream updates to A2A clients
7. Implement tasks/resubscribe for resuming task status streams
8. Add connection management (timeouts, keep-alive, reconnection)

**Phase 2 - Message Handling (High Priority):**
9. Connect message/send to Agents service (create tasks from A2A messages)
10. Implement message routing to appropriate agents based on capabilities
11. Handle message context propagation through task lifecycle
12. Add message validation and error responses

**Phase 3 - Push Notifications (Medium Priority):**
13. Create push_notification_configs database table (task_id, webhook_url, events, headers)
14. Implement webhook delivery system with retry logic (exponential backoff)
15. Add event filtering (subscribe to specific task states)
16. Webhook authentication (HMAC signatures)

**Phase 4 - Security (Medium Priority):**
17. Implement authentication middleware (verify API keys, tokens)
18. Add security scheme support (API Key, OAuth2, Bearer tokens)
19. Agent Card signatures (JWS) for integrity verification
20. Rate limiting per client

**Phase 5 - Polish (Lower Priority):**
21. Standardize error handling across all A2A endpoints
22. Comprehensive A2A compliance testing suite
23. Performance optimization (caching, connection pooling)
24. Documentation and examples

---

_NullBlock implements a cyberpunk aesthetic with neon styling and maintains immersive error messages throughout the user experience while building toward a comprehensive MCP-powered agentic ecosystem._

## 📝 Implementation Notes

### A2A Task Schema Implementation Details

**Database Changes (svc/nullblock-agents/migrations/001_create_tasks_table.sql):**
- Added `context_id UUID` - Groups related tasks together
- Added `kind VARCHAR DEFAULT 'task'` - Always "task" per A2A spec
- Changed `status VARCHAR` to use A2A state values with CHECK constraint
- Added `status_message TEXT` - Optional human-readable status description
- Added `status_timestamp TIMESTAMPTZ` - When status was last updated
- Added `history JSONB DEFAULT '[]'` - Message array tracking conversation
- Added `artifacts JSONB DEFAULT '[]'` - Artifact array with task outputs

**Repository Methods (svc/nullblock-agents/src/database/repositories/tasks.rs):**
- `add_message_to_history(task_id, message)` - Appends message to history JSONB array using PostgreSQL || operator
- `add_artifact(task_id, artifact)` - Appends artifact to artifacts JSONB array
- `update_status_with_message(task_id, state, message)` - Updates status with optional message and timestamp

**Handler Integration (svc/nullblock-agents/src/handlers/tasks.rs):**
- Task creation adds initial user message to history with task description
- Message format: `{messageId, role: "user", parts: [{type: "text", text}], timestamp, taskId, contextId, kind: "message"}`

**Agent Integration (svc/nullblock-agents/src/agents/hecate.rs:745-807):**
- After task processing, adds agent response message to history with metadata (agent, model, processing_duration_ms)
- Creates completion artifact with result text and metadata (artifact_type: "completion_result", model, duration)
- Uses `update_status_with_message()` to set completion status with success message

**Protocols Service (svc/nullblock-protocols/):**
- Added reqwest HTTP client dependency for service-to-service communication
- AppState contains http_client and agents_service_url (http://localhost:9003)
- Task handlers proxy to Agents service: GET /api/agents/tasks/:id, GET /api/agents/tasks, POST /api/agents/tasks/:id/cancel
- JSON-RPC handlers updated to pass AppState to task functions
- Response parsing extracts task data from `{"success": true, "data": {...}}` wrapper

**Frontend Types (svc/hecate/src/types/tasks.ts):**
- TaskStatus changed from string to object `{state: TaskState, message?: string, timestamp?: string}`
- TaskState type union with 9 A2A values
- A2AMessage interface with MessagePart union type (text | file | data)
- A2AArtifact interface with parts array
- Task interface updated with contextId, kind, status object, history, artifacts