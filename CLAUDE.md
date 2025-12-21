# CLAUDE.md

```
 _   _       _ _ ____  _            _
| \ | |_   _| | | __ )| | ___   ___| | __
|  \| | | | | | |  _ \| |/ _ \ / __| |/ /
| |\  | |_| | | | |_) | | (_) | (__|   <
|_| \_|\__,_|_|_|____/|_|\___/ \___|_|\_\
```

**🎯 Mission**:
In a rapidly expanding onchain automated world, we are building the picks and axes
for this digital gold rush.
NullBlock empowers builders with the essential tools to create, deploy,and
profit from intelligent agent workflows.
Together, we shape the future of autonomous commerce.

## 🌐 Connect & Follow

**🏢 Official**: [@Nullblock_io](https://x.com/Nullblock_io)
**📦 SDK**: [nullblock-sdk](https://github.com/aetherBytes/nullblock-sdk)
**🌍 Site**: NullBlock.io _(Coming Soon)_

### 🚧 Current Development Focus

**Priority Tasks:**

1. **🔵 Get X Account Verified** - Obtain verified status for [@Nullblock_io](https://x.com/Nullblock_io) to establish credibility and official presence
2. **💰 Marketing & Token Strategy** - Develop marketing strategy, purchase additional supply in dev wallet, implement token lock mechanisms
3. **💬 Community Channels** - Set up Discord and Telegram channels - the people are demanding it!
4. **🌐 WEB3 WILDS X Community** - Start planning the "WEB3 WILDS" X community initiative - branding, content strategy, launch timeline, and engagement mechanics
5. **🤖 Agent Model Selection** - Make sure Siren / other agents do not get stuck on a default model. Should follow model selections of user.
6. **🔄 Crossroads Login Reload** - Crossroads needs to reload content after success on login
7. **✅ A2A Task Schema & Integration** - COMPLETED: Full A2A Protocol v0.3.0 compliance with task schema, repository methods, handler population of history/artifacts, and Protocols→Agents HTTP integration
8. **✅ Docker & Replication Infrastructure** - COMPLETED: Fixed PostgreSQL logical replication with container-first architecture, system-agnostic design working on macOS and Linux
9. **🔄 Task State Alignment** - A2A protocol uses "working" state but Hecate expects "created"/"running" - reconcile state transitions and implement auto-processing for auto_start=true tasks
10. **📋 A2A Streaming (SSE)** - Implement Server-Sent Events for message/stream and tasks/resubscribe, bridge Kafka task.lifecycle events to SSE streams for real-time updates

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

## 🐳 Docker & Container Golden Rules

### Container-First Architecture 🚨

**CRITICAL**: All infrastructure runs in Docker containers with container-to-container communication.

#### Golden Rules:

1. **🔗 Container-to-Container Communication**
   - ✅ **ALWAYS** use container names for service-to-service communication
   - ✅ **ALWAYS** use internal ports (5432, not 5440) for container communication
   - ✅ **ALWAYS** use Docker bridge networks (never `--network=host` unless absolutely necessary)
   - ❌ **NEVER** use `localhost` or `127.0.0.1` for container-to-container communication
   - ❌ **NEVER** use `host.docker.internal` (macOS-specific, breaks on Linux)
   - ❌ **NEVER** use external ports (5440/5441) for container communication

2. **📦 Network Configuration**
   - All containers MUST join the `nullblock-network` bridge network
   - External access uses port mapping: `-p 5440:5432` (host:container)
   - Internal access uses container name: `nullblock-postgres-erebus:5432`
   - Docker DNS automatically resolves container names within the network

3. **🔧 Configuration Examples**
   ```bash
   # ✅ CORRECT - Container to container
   docker run --network nullblock-network postgres
   CONNECTION='host=nullblock-postgres-erebus port=5432'

   # ✅ CORRECT - Host to container (external)
   psql -h localhost -p 5440

   # ❌ WRONG - Using host.docker.internal
   CONNECTION='host=host.docker.internal port=5440'

   # ❌ WRONG - Using localhost in container
   CONNECTION='host=localhost port=5432'
   ```

4. **🎯 System-Agnostic Design**
   - Container configurations MUST work identically on macOS and Linux
   - Use Docker networking features, not OS-specific workarounds
   - Test on both platforms before committing

5. **🗄️ Database Replication Example**
   - Erebus DB publishes: `nullblock-postgres-erebus:5432`
   - Agents DB subscribes: `host=nullblock-postgres-erebus port=5432`
   - External access: `localhost:5440` (Erebus), `localhost:5441` (Agents)

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

**📖 For detailed agent documentation, see [AGENTS.md](./AGENTS.md)**

### Hecate Agent

- **Purpose**: Main conversational interface, orchestration engine
- **Default Model**: `cognitivecomputations/dolphin3.0-mistral-24b:free` (override via `DEFAULT_LLM_MODEL` env var)
- **Features**: Multi-model LLM, intent analysis, delegation, task management
- **Timeout**: 5-minute for thinking models (configurable via `LLM_REQUEST_TIMEOUT_MS`)
- **Max Tokens**: 16384 (required for base64 image responses)

### LLM Factory & Model Selection

**Location**: `svc/nullblock-agents/src/llm/factory.rs`

**Providers**: OpenRouter (primary), OpenAI, Anthropic, Groq, HuggingFace
**Strategies**: Quality, speed, cost, balanced

**Model Selection Flow**:
1. Hard-coded default model loaded from `DEFAULT_LLM_MODEL` or falls back to `cognitivecomputations/dolphin3.0-mistral-24b:free`
2. Startup validation: Test query sent to verify model availability
3. On failure: Fetch live free models from OpenRouter, sort by context window
4. Runtime routing: Weighted scoring (quality 40pts, reliability 30pts, cost optimization, tier bonuses)
5. Fallback chain: Automatically tries alternative models if primary fails
6. Empty response detection: Skips models returning 0 completion tokens

### Specialized Agents

- **Siren Marketing**: Content generation, Twitter posts, project analysis

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

**Container Communication**:
- **Internal (container-to-container)**: `nullblock-postgres-erebus:5432`, `nullblock-postgres-agents:5432`
- **External (host access)**: `localhost:5440` (Erebus), `localhost:5441` (Agents)
- **Network**: All containers on `nullblock-network` bridge network
- **Replication**: Agents container subscribes to Erebus via `host=nullblock-postgres-erebus port=5432`

**Setup (Automatic via Migrations)**:
1. Infrastructure: `just start` creates network and containers with replication config
2. Erebus migration `002_setup_logical_replication.sql` creates publication
3. Agents migration `005_setup_replication_subscription.sql` creates subscription
4. Replication starts automatically with initial data backfill

**Benefits**: Service isolation, real-time sync (<1s), automatic recovery, persistent across container restarts, system-agnostic (works on macOS and Linux)

**Monitoring**:
```bash
# Check replication status
PGPASSWORD="REDACTED_DB_PASS" psql -h localhost -p 5441 -U postgres -d agents -c \
  "SELECT subname, subenabled FROM pg_subscription WHERE subname = 'agents_user_sync';"

# Verify user counts match
PGPASSWORD="REDACTED_DB_PASS" psql -h localhost -p 5440 -U postgres -d erebus -c \
  "SELECT COUNT(*) as erebus_users FROM user_references;"
PGPASSWORD="REDACTED_DB_PASS" psql -h localhost -p 5441 -U postgres -d agents -c \
  "SELECT COUNT(*) as agents_users FROM user_references;"
```

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

## 🌌 Void Experience (Home Screen)

The Void Experience is the immersive Three.js post-login home screen. Users awaken in a living agent mesh void.

### Architecture

```
THE VOID
├── ParticleField (stars) - Ambient drifting particles
├── NeuralLines (constellations) - Minor nodes representing Tools/Services/Servers
├── CrossroadsOrb (center) - The Crossroads Bazaar marketplace hub
└── AgentClusters (major nodes) - AI Agents orbiting the center
    ├── Hecate (orchestrator) - Gold glow
    ├── Siren (marketing) - Purple accent
    └── Erebus (router) - Blue accent
```

### Node Types

| Type | Component | Represents |
|------|-----------|------------|
| **Center** | `CrossroadsOrb` | Crossroads Bazaar (marketplace) |
| **Major** | `AgentCluster` | AI Agents (clickable, opens panel) |
| **Minor** | `NeuralLines` nodes | Tools, Services, MCP Servers |
| **Background** | `ParticleField` | Ambient stars |

### Camera Behavior

- **Pre-login**: Position `[4, 3, 12]` - Far back, offset view (all visible, non-interactive, slow auto-rotate)
- **Post-login**: Position `[0, 0.5, 6]` - Centered on Crossroads (interactive, static camera, no wobble)
- **Logout**: Smooth zoom-out animation back to pre-login position
- **Cluster Click**: Camera zooms to cluster, cluster freezes in place, panel opens

### File Structure

```
svc/hecate/src/components/void-experience/
├── VoidExperience.tsx       # Canvas wrapper, state management
├── scene/
│   ├── CrossroadsOrb.tsx    # Central bazaar node
│   ├── AgentCluster.tsx     # Individual agent node
│   ├── AgentClusters.tsx    # Agent collection manager
│   ├── NeuralLines.tsx      # Service constellation network
│   ├── ParticleField.tsx    # Ambient star particles
│   └── CameraController.tsx # Smooth camera traversal
├── hooks/
│   └── useAgentClusters.ts  # Fetch from /api/discovery/agents
└── chat/
    └── VoidChatHUD.tsx      # Chat input overlay
```

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
VITE_API_GATEWAY=https://randomuser.me/api
VITE_FAST_API_BACKEND_URL=http://localhost:8000
CORS_ORIGINS=http://localhost:5173,http://localhost:3000
```

**⚠️ TODO**: Update all local API URL references to use environment variables consistently. Currently some services may have hardcoded URLs that need to be migrated to use the VITE_* environment variables above.

### LLM APIs

```bash
OPENAI_API_KEY=
ANTHROPIC_API_KEY=
GROQ_API_KEY=
OPENROUTER_API_KEY=          # REQUIRED - Get from https://openrouter.ai/
DEFAULT_LLM_MODEL=cognitivecomputations/dolphin3.0-mistral-24b:free
LLM_REQUEST_TIMEOUT_MS=300000  # 5 minutes for thinking models
```

**CRITICAL**: `OPENROUTER_API_KEY` is REQUIRED for reliable LLM access. Without it, you'll hit severe rate limits on free models.

**Environment File Location**: Create `.env.dev` in project root. Services running in subdirectories need symlinks:
```bash
ln -s ../../.env.dev svc/nullblock-agents/.env.dev
ln -s ../../.env.dev svc/erebus/.env.dev
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

**Docker & Container Architecture (October 2025):**

- ✅ **Container-First Architecture** - All infrastructure runs in Docker containers with bridge networking
- ✅ **System-Agnostic Design** - Identical behavior on macOS and Linux using Docker networking features
- ✅ **PostgreSQL Replication Fix** - Fixed subscription to use container names (`nullblock-postgres-erebus:5432`) instead of OS-specific workarounds (`host.docker.internal`)
- ✅ **Network Configuration** - All containers join `nullblock-network` bridge network for container-to-container communication
- ✅ **Justfile Updates** - Both `start-mac` and `start-linux` now use Docker bridge networking with proper port mapping
- ✅ **Replication Verification** - Tested and confirmed <1 second replication latency from Erebus→Agents
- ✅ **Container Golden Rules** - Added comprehensive documentation to CLAUDE.md preventing future networking issues
- ✅ **Migration Script Updates** - Subscription migration now uses internal container ports (5432) instead of external host ports

**LLM & Error Handling (December 2025):**

- ✅ **OpenRouter API Key Validation** - Startup validation detects missing/placeholder keys with clear error messages
- ✅ **Anonymous Access Detection** - Runtime detection when API key not loaded properly (detects `user_*` anonymous IDs)
- ✅ **Empty Response Validation** - Fallback chain skips models returning 0 completion tokens
- ✅ **Environment File Symlinks** - Services access `.env.dev` via symlinks for API key loading
- ✅ **Enhanced Error Logging** - Critical errors include actionable configuration steps and partial key verification
- ✅ **Task Creation Fix** - User reference creation now includes required `network` field in `source_type` object
- ✅ **Model Selection Documentation** - Comprehensive documentation of startup validation, fallback chains, and scoring algorithm

### In Progress 🔄

- 🔄 **Task State Naming Mismatch** - A2A protocol uses "working" state, but Hecate process endpoint expects "created" or "running" - need to align state transitions and validate task lifecycle
- 🔄 **Hecate Auto-Processing** - Tasks created with auto_start=true transition to "working" state but don't automatically process with Hecate agent

### Next Up 📋

**Immediate (Unblock Development):**

1. **Fix Task State Alignment** - Reconcile A2A state names ("working") with internal processing states ("running", "created"). Update task handlers to accept A2A states or create state mapping layer.
2. **Hecate Auto-Processing Flow** - Implement automatic task processing when auto_start=true. Current behavior: task transitions to "working" but Hecate doesn't execute. Need to trigger agent processing on state change.
3. **Task Processing Endpoint** - Update `/tasks/:id/process` to accept A2A states or add state normalization before validation
4. **Validate Artifact Population** - Once processing works, confirm Hecate adds completion artifacts with metadata (model, duration) to artifacts array
5. **Service Container Integration** - Update Erebus, Agents, and Protocols Rust services to use container names for inter-service communication (e.g., `http://nullblock-protocols:8001` instead of `http://localhost:8001`)
6. **Fix Image Generation** - Three issues blocking image display: (1) useChat.ts parseContentForImages() removes image markdown from content string, (2) max_tokens too low (4096) for base64 images (need 16384+), (3) No error handling for truncated/timeout responses. Fix: Keep images in markdown for MarkdownRenderer, increase token limit, add validation and timeout handling

**Phase 1 - Streaming & Real-time (High Priority):** 5. Implement Server-Sent Events (SSE) for message/stream endpoint 6. Build Kafka → SSE bridge: Subscribe to task.lifecycle topic, stream updates to A2A clients 7. Implement tasks/resubscribe for resuming task status streams 8. Add connection management (timeouts, keep-alive, reconnection)

**Phase 2 - Message Handling (High Priority):** 9. Connect message/send to Agents service (create tasks from A2A messages) 10. Implement message routing to appropriate agents based on capabilities 11. Handle message context propagation through task lifecycle 12. Add message validation and error responses

**Phase 3 - Push Notifications (Medium Priority):** 13. Create push_notification_configs database table (task_id, webhook_url, events, headers) 14. Implement webhook delivery system with retry logic (exponential backoff) 15. Add event filtering (subscribe to specific task states) 16. Webhook authentication (HMAC signatures)

**Phase 4 - Security (Medium Priority):** 17. Implement authentication middleware (verify API keys, tokens) 18. Add security scheme support (API Key, OAuth2, Bearer tokens) 19. Agent Card signatures (JWS) for integrity verification 20. Rate limiting per client

**Phase 5 - Polish (Lower Priority):** 21. Standardize error handling across all A2A endpoints 22. Comprehensive A2A compliance testing suite 23. Performance optimization (caching, connection pooling) 24. Documentation and examples

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

### Recent Bug Fixes & Improvements

**Task Creation with User References** (`svc/erebus/src/resources/agents/routes.rs:38-42`):
- Fixed missing `network` field in `source_type` object causing task creation failures
- Error was: `"Failed to deserialize the JSON body into the target type: source_type: missing field 'network'"`
- Solution: Added `"network": wallet_chain` to default_source_type JSON

**LLM API Key Management** (`svc/nullblock-agents/src/llm/factory.rs` & `providers.rs`):
- Added startup validation to detect invalid/placeholder API keys
- Implemented anonymous access detection via OpenRouter `user_id` field
- Enhanced error messages with actionable configuration steps
- Log partial API keys for verification (e.g., `sk-or-v1-49a74f...7c7a`)

**Empty Response Handling** (`svc/nullblock-agents/src/llm/factory.rs:161-164`):
- Added validation to detect models returning 0 completion tokens
- Fallback chain now skips empty responses and tries next model
- Prevents silent failures with clear warning logs

**Environment File Access** (`svc/nullblock-agents/src/main.rs:30-37`):
- Improved .env.dev loading warnings with specific guidance
- Created symlinks from service directories to project root .env.dev
- Services now consistently load API keys from centralized configuration
