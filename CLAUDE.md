# CLAUDE.md

```
 _   _       _ _ ____  _            _
| \ | |_   _| | | __ )| | ___   ___| | __
|  \| | | | | | |  _ \| |/ _ \ / __| |/ /
| |\  | |_| | | | |_) | | (_) | (__|   <
|_| \_|\__,_|_|_|____/|_|\___/ \___|_|\_\
```

**Mission**: Building the picks and axes for the onchain agent gold rush. NullBlock empowers builders with tools to create, deploy, and profit from intelligent agent workflows.

**Chain**: [Monad](https://monad.xyz) (exclusive) - High-performance EVM for agent transactions, NFT minting, and payments.

## Connect

- **Official**: [@Nullblock_io](https://x.com/Nullblock_io)
- **SDK**: [nullblock-sdk](https://github.com/aetherBytes/nullblock-sdk)
- **Site**: NullBlock.io _(Coming Soon)_

## Documentation

**Internal Docs**: `docs-internal/` (mdBook) - `just docs` serves at http://localhost:3001

| Quick Link | Path |
|------------|------|
| Architecture | [docs-internal/src/architecture.md](docs-internal/src/architecture.md) |
| Quick Start | [docs-internal/src/quickstart.md](docs-internal/src/quickstart.md) |
| Service Ports | [docs-internal/src/ports.md](docs-internal/src/ports.md) |
| API Reference | [docs-internal/src/reference/api.md](docs-internal/src/reference/api.md) |
| Environment Vars | [docs-internal/src/reference/env-vars.md](docs-internal/src/reference/env-vars.md) |

## Current Focus: ArbFarm (Solana MEV Agent Swarm)

Building ArbFarm - autonomous multi-agent system for capturing MEV opportunities on Solana. See [ArbFarm Plan](docs-internal/src/arb-farm/plan.md).

| Phase | Status |
|-------|--------|
| 1. Service Scaffold + Core Models | ✅ Complete |
| 2. Venue Scanner + Strategy Engine | ⚠️ Partial (curves work, DEX stubbed) |
| 3. Execution Engine + Risk Management | ✅ Complete |
| 4. Bonding Curve Integration | ✅ Complete |
| 5. MEV Detection | ⚠️ Partial (liquidations work, DEX arb stubbed) |
| 6. Research/DD Agent | ❌ Not Started |
| 7. KOL Tracking + Copy Trading | ✅ Complete (discovery + copy execution wired) |
| 8. Threat Detection | ❌ Not Started |
| 9. Engram Integration | ✅ Complete |
| 10. Swarm Orchestration | ✅ Complete |
| 11. Frontend Dashboard | ✅ Complete |
| 12. Crossroads Integration | ⏳ Next |

**Also Planned**: Poly Mev (Polymarket) - [plan](docs-internal/src/poly-mev/plan.md)

**Paused**: Echo Factory (X/Twitter COW) - [archived](docs-internal/src/archive/echo-factory/plan.md)

## Architecture

```
Frontend (5173) → Erebus (3000) → Backend Services
                       │
         ┌─────────────┼─────────────┐
         │             │             │
    Crossroads    Engrams (9004)   COWs (9005+)
```

**Golden Rule**: ALL frontend requests → Erebus (3000). NO direct service connections.

### Services & Ports

| Port | Service | Description |
|------|---------|-------------|
| 3000 | Erebus | Unified router + Crossroads |
| 5173 | Hecate | React frontend |
| 8001 | Protocols | A2A/MCP server |
| 9003 | Agents | HECATE agent API |
| 9004 | Engrams | Memory/context layer |
| 9007 | ArbFarm | Solana MEV agent swarm |
| 5440 | PostgreSQL | Erebus DB |
| 5441 | PostgreSQL | Agents DB |

## Quick Start

```bash
just dev-mac    # macOS (starts all services via tmuxinator)
just dev-linux  # Linux
just docs       # Serve internal docs at :3001
```

## Golden Rules

### Documentation Sync
- **ALWAYS** update [Trading Strategies](docs-internal/src/arb-farm/strategies.md) when modifying ArbFarm trading logic
- Strategy changes = doc changes in the **same commit**
- Includes: entry/exit logic, slippage, risk config, position management, new strategies
- Run `just docs` to verify changes render correctly

### Erebus Router
- **ALL** frontend → Erebus (3000) - NO EXCEPTIONS
- **ALL** user CRUD → `/api/users/*` - NO direct DB access

### Docker Containers
- ✅ Use container names: `nullblock-postgres-erebus:5432`
- ✅ Use internal ports (5432) for container-to-container
- ❌ Never use `localhost` or `host.docker.internal` between containers
- ❌ Never use external ports (5440) for container communication

### LLM API Keys
- **NEVER** put keys in `.env.dev`
- Keys stored in Erebus DB (`agent_api_keys` table)
- Seed with: `cd svc/erebus && cargo run --bin seed_agent_keys`

### Primitive-First Tool Architecture (Dog Food Everything)
- **ALL** tools start as generic base primitives — same CRUD operations everywhere
- Build primitives ourselves where absent; never skip straight to specialized tools
- COWs are composed from base primitives — they are strung-together suites of more basic tools
- Iterate on primitives to build sophisticated tool suites (e.g., ArbFarm's 97 MCP tools all build on the same engram CRUD base)
- Tags, metadata, and content differentiate use cases — not separate codepaths
- When adding a new COW or service, first check if existing primitives cover the need
- **We dog food everything** — NullBlock services consume the same tools we expose to users

### Code Standards
- **NEVER** add comments unless requested
- **ALWAYS** prefer editing over creating new files
- **NEVER** proactively create documentation files

## Key Concepts

### Engrams (Memory Layer)
Persistent, wallet-scoped context storage. Types: `persona`, `preference`, `strategy`, `knowledge`, `compliance`. See [Engrams Service](docs-internal/src/services/engrams.md).

### COWs (Constellations of Work)
Curated tool suites forming autonomous workflows. First-class Crossroads listing type. Forkable with engram inheritance.

### HECATE Agent
Von Neumann-class vessel AI. Default model: `cognitivecomputations/dolphin3.0-mistral-24b:free`. Timeout: 5min. Max tokens: 16384.

## API Endpoints

| Path | Service |
|------|---------|
| `/api/users/*` | User CRUD (Erebus owned) |
| `/api/wallets/*` | Authentication |
| `/api/agents/*` | Chat, tasks |
| `/api/engrams/*` | Memory/context |
| `/api/marketplace/*` | Crossroads listings |
| `/api/discovery/*` | Service discovery |
| `/api/arb/*` | ArbFarm MEV swarm |
| `/mcp/*` | MCP Protocol (2025-11-25) |
| `/a2a/*` | A2A Protocol |

## ArbFarm Development Status (Audit: 2026-02-01)

**Mode**: LOCAL DEVELOPMENT / PRE-PRODUCTION
**Overall**: LOCAL DEV READY — all code compiles, all features functional, production deployment blocked on security hardening

### Implemented (Core Trading Logic)
- ✅ Exit transactions saved to engrams (buy AND sell tracked)
- ✅ MCP tool annotations (97 tools with readOnlyHint/destructiveHint/idempotentHint)
- ✅ A2A tags exposed (`arbFarm.learning` on 6 learning tools)
- ✅ Wallet funding validated at startup (blocks if < 0.05 SOL)
- ✅ Position monitor (detection) + position executor (sell execution) auto-start with curve support + engrams
- ✅ Frontend service methods deduplicated
- ✅ Momentum exits fully implemented (position_manager.rs:1403-1500)
- ✅ Copy trading fully wired (webhooks.rs → kol_topics::TRADE_DETECTED → autonomous_executor)
- ✅ Webhook auth FAIL-CLOSED (returns 401 if HELIUS_WEBHOOK_AUTH_TOKEN not set)
- ✅ DATABASE_URL required (no hardcoded fallback - fails startup if missing)
- ✅ Signal deduplication in StrategyEngine (prevents duplicate buy attempts)
- ✅ Cooldown insert moved to post-success (failed TXs don't block retries)
- ✅ Engram recommendations applied to config
- ✅ Dead token salvage working
- ✅ Per-strategy capital allocation working
- ✅ Tiered exit strategy and Raydium Trade API for post-graduation sells
- ✅ Graduation snipe entry filter (50% pump allowed vs 15% for curve_arb)
- ✅ Copy sell timeout with emergency execution (30s timeout)
- ✅ Graceful shutdown handler (SIGTERM/SIGINT with in-flight drain)
- ✅ Daily risk stats persisted to DB (survives restarts)
- ✅ API rate limiting on scanner (150ms min per venue)
- ✅ Event broadcast failure logging (all event_tx.send calls now log errors)
- ✅ Signal staleness check (expired signals skipped)
- ✅ Confident-only recommendation threshold (>=70 score)
- ✅ Consolidated DB pools (30 connections)
- ✅ Inferred exit signatures flagged in DB (is_inferred_exit column)
- ✅ Position.opened event emitted on buy success (SSE-visible)
- ✅ Token age filter (stale tokens >48h at >=85% progress skipped in scanner)
- ✅ SSE-driven dashboard refresh (HomeTab no longer polls, reacts to SSE events)
- ✅ Strategy labels on trades and positions (strategy_type, signal_source, venue persisted)
- ✅ Webhook status indicators on behavioral strategies (Standby/Connected badges)
- ✅ signal_source and venue columns persisted to arb_positions (migration 016)
- ✅ RwLock poison safety in threat module (all unwraps replaced with safe fallbacks)
- ✅ Semaphore error handling in sell-all handler (no more panic on acquire failure)
- ✅ DB health check at startup (SELECT 1 with retry before accepting traffic)
- ✅ ALERTS_STORE capped at 10K entries with oldest-first eviction
- ✅ Strategy update failures logged (no more silent config loss)
- ✅ SSE reconnection backoff (exponential backoff prevents tight reconnect loops)
- ✅ WebSocket broadcast failures logged in laserstream (all 5 silent `let _ =` replaced)
- ✅ DB connection pool retry with exponential backoff (3 retries: 1s/2s/4s)
- ✅ SSE refresh race guard (skips fetchData during user actions: sell, sell-all, reconcile)
- ✅ Tmuxinator DATABASE_URL fixed (was pointing to Erebus DB, now points to Agents DB)
- ✅ `just migrate` includes ArbFarm migrations (scripts/migrate-arbfarm.sh added as Step 3)

### Known Issues — Fix Before Production Deployment

These issues are acceptable for local development but MUST be resolved before deploying with real capital:

| # | Issue | Severity | Why Deferred |
|---|-------|----------|-------------|
| P1 | API endpoints unauthenticated (wallet spoofing via x-wallet-address header) | CRITICAL | Local only — no external access |
| P2 | CORS allows any origin (`tower_http::cors::Any`) | CRITICAL | Local only — restrict to prod domain on deploy |
| P3 | Private keys / API keys in .env.dev (git-tracked) | CRITICAL | Dev keys only — rotate all keys before production, purge git history |
| P4 | HTTP endpoints not rate limited | HIGH | Local only — add Tower rate limiting middleware on deploy |
| P5 | No simulation before execute_edge_auto (buy without preflight) | HIGH | Add simulateTransaction call before real capital |
| P6 | No capital re-verification at execution time (race between check and TX) | HIGH | Re-check balance/allocation at TX submission |
| P7 | Emergency kill switch not prominent in UI | MEDIUM | Add top-level emergency stop button |
| P8 | Migrations not fully idempotent (ALTER TABLE without guards) | MEDIUM | Wrap in IF EXISTS checks |
| P9 | No auto migration runner at startup | MEDIUM | Add sqlx::migrate!() or document manual step |

## NullBlock Content Service

**Purpose**: Social media content generation + posting service for Nullblock brand
**Status**: ✅ Phase 1-3 COMPLETE (core infrastructure + content engine + API layer)
**Port**: 8002 (routes through Erebus at 3000)
**Database**: PostgreSQL (separate from Erebus/Agents)

**Agent Framework**: N.E.X.U.S.
- **N**etwork: Coordination ability
- **E**xecution: Task completion speed
- **X**pansion: Learning & scaling
- **U**tility: Practical ROI
- **S**ecurity: Verifiability & trust

### Themes (Tone: Cheerfully Inevitable + Professionally Apocalyptic)
1. **MORNING_INSIGHT** - Daily motivation (9 AM) - Infrastructure/protocols
2. **PROGRESS_UPDATE** - Dev milestones (3 PM) - Progress transparency
3. **EDUCATIONAL** - Deep dives (Wed noon) - Agentic workflows explainers
4. **EERIE_FUN** - Dark AI humor (Sun 6 PM) - Original voice, not Fallout refs
5. **COMMUNITY** - Engagement (6 PM daily) - Polls + questions

### Key Guidelines
- **Focus**: Infrastructure, protocols, agentic networks, open economy
- **Avoid**: Financial advice, token shilling, day trading mentions, Fallout IP references
- **Brand**: "Picks and shovels for the agentic age"
- **Tone**: Corporate dystopia + dark humor, cheerfully inevitable, professionally apocalyptic
- **Voice**: Original Nullblock (not Vault-Tec/Fallout lingo) - internal docs can reference the vibe, but posted content is pure Nullblock

### Implementation Status

**Phase 1 - Core Infrastructure ✓ Complete:**
- Database connection pool (20 connections, 5s timeout)
- ContentError enum with thiserror
- ContentRepository (14 CRUD methods)
- 3 SQL migrations (content_queue, content_metrics, content_templates)

**Phase 2 - Content Engine ✓ Complete:**
- 5 themes with content pools (MorningInsight, ProgressUpdate, Educational, EerieFun, Community)
- Template loader with JSON config + fallback defaults
- ContentGenerator with placeholder replacement
- Image prompt generation (retro-futuristic propaganda style)
- templates.json with 13 variants, 40+ unique content pieces

**Phase 3 - API Layer ✓ Complete:**
- Axum server on port 8002 with health check
- 7 REST endpoints (generate, queue CRUD, metrics, templates)
- Request handlers with proper error handling
- CORS enabled for development

**Phase 4 - Integration (In Progress):**
- Kafka event publishing (content.generated, content.posted, content.failed)
- MCP tool definitions via nullblock-protocols
- Erebus routing integration
- Crossroads marketplace listing

### Database Tables
- `content_queue` - Generated content pending review/posting (UUID, theme, text, tags, status, metadata)
- `content_metrics` - Engagement stats (likes, retweets, impressions, engagement_rate)
- `content_templates` - Theme definitions with JSONB variants

### API Routes (via Erebus)
- `POST /api/content/generate` - Generate content from theme
- `GET /api/content/queue` - List pending content
- `POST /api/content/queue/:id/post` - Post to X/Twitter
- `GET /api/content/posted` - List posted content with metrics

### MCP Tools (exposed via nullblock-protocols)
- `generate_content` - Generate themed content
- `post_content` - Post content to platform
- `schedule_content` - Schedule automated generation

### Crossroads Integration
- Listed as `Tool` type
- Metadata: themes, platforms, mcp_tools
- Ready for ClawHub import

### Audit False Alarms (Verified OK)
- Jito timeout — already has comprehensive Helius fallback (position_executor.rs:696-730)
- Exit queue unbounded — actually uses bounded mpsc channel, not Vec (position_executor.rs:50-67)
- Position persistence — correctly does DB-first, aborts on DB failure (position_manager.rs:1224-1236)

### Startup Safety Defaults (Automated Trading OFF)
All automated trading is **disabled by default** on every restart. The system starts in **OBSERVATION MODE**:
- All scanners/strategies active (find and display opportunities)
- All execution disabled (no buys, sells on existing positions only)

Enable execution via UI or env vars:

| Component | Default | Enable With |
|-----------|---------|-------------|
| Scanner | ON | Always runs (displays opportunities) |
| Execution Engine | OFF | `ARBFARM_ENABLE_EXECUTOR=1` or UI toggle |
| Graduation Sniper | ON (observe only) | `ARBFARM_SNIPER_ENTRY=1` to enable buys |
| Copy Trading | OFF | `ARBFARM_COPY_TRADING=1` to enable KOL copy |
| Volume Hunter | ON (observe only) | Execution via Execution Engine |
| Position Monitor | ON | Always runs (detects exit conditions, queues sells) |
| Position Executor | ON | Always runs (executes queued sell transactions) |
| Consensus Scheduler | OFF | Manual trigger only (UI/API), ignores DB state |

To disable sniper entirely (stop observing): `ARBFARM_ENABLE_SNIPER=0`

This prevents accidental automated trading after service restarts.

### Stubbed Features (Generate Warnings - Safe to Ignore)
These features are scaffolded but not implemented. Warnings are expected:

| Module | Warnings | Status |
|--------|----------|--------|
| `events/topics.rs` | 66 | Event constants for future features |
| `agents/mev_hunter.rs` | 16 | DEX arbitrage detection (stubbed) |
| `venues/dex/raydium.rs` | 8 | Raydium integration (stubbed) |
| `consensus/providers/*` | 14 | Anthropic/OpenAI direct providers (using OpenRouter) |
| `agents/graduation_tracker.rs` | 8 | Graduation tracking (stubbed) |

### Not Implemented (Non-Blocking)
- ❌ Research/DD Agent (Phase 6)
- ❌ Threat Detection (Phase 8)

### AWS Deployment TODO (Security + KOL Copy Trading)

**Status:** LOCAL DEV ONLY — resolve P1-P9 above before deploying with real capital

**Security hardening required before production:**
1. Add auth middleware or restrict API to localhost/VPN (P1)
2. Restrict CORS to production domain (P2)
3. Rotate ALL API keys, purge from git history with BFG/filter-branch (P3)
4. Add Tower rate limiting middleware on sensitive endpoints (P4)
5. Add simulateTransaction preflight on buys (P5)
6. Re-verify capital at TX submission time (P6)

**KOL Copy Trading** requires public URL for Helius webhooks:

**Setup Steps (do this when deploying to AWS):**

1. **Generate auth token:**
   ```bash
   openssl rand -hex 16
   # Example: a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6
   ```

2. **Add to production env:**
   ```bash
   HELIUS_WEBHOOK_AUTH_TOKEN=a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6
   ```

3. **Configure in Helius Dashboard:**
   - Webhook URL: `https://<your-aws-domain>/webhooks/helius`
   - Webhook Type: Enhanced Transaction
   - Auth Header: `Authorization: Bearer a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6`
   - Account Addresses: Add KOL wallet addresses to track

4. **Verify webhook works:**
   ```bash
   curl -X POST https://<your-aws-domain>/webhooks/helius \
     -H "Content-Type: application/json" \
     -H "Authorization: Bearer <your-token>" \
     -d '[]'
   # Should return 200
   ```

**What works WITHOUT webhooks (local testing):**
- ✅ Bonding Curve Scanner (polls pump.fun)
- ✅ Graduation Sniper (polls pump.fun)
- ✅ Position Management & Exits
- ✅ Buy/Sell Execution

**What REQUIRES webhooks (AWS only):**
- ⏳ KOL Copy Trading (Helius pushes wallet activity)

### Next Implementation Priorities
1. **Crossroads Integration** - Marketplace listing for ArbFarm COW
2. **DEX Arbitrage** - Complete `mev_hunter.rs` and Raydium venue
3. **Threat Detection** - Implement rug pull / honeypot detection
4. **Research Agent** - Social sentiment + alpha discovery

## ArbFarm MCP Tools for Learning

ArbFarm exposes MCP tools for AI-to-AI learning integration:

| Tool | Description |
|------|-------------|
| `engram_get_arbfarm_learning` | Fetch learning engrams (recommendations, conversations, patterns) |
| `engram_acknowledge_recommendation` | Update recommendation status (acknowledged/applied/rejected) |
| `engram_get_trade_history` | Get transaction summaries with PnL |
| `engram_get_errors` | Get execution error history |
| `engram_request_analysis` | Trigger consensus LLM analysis |

### /scrape-engrams Skill

The `/scrape-engrams` skill analyzes ArbFarm learning data and generates profit optimization recommendations.

**Usage**: Run `/scrape-engrams` in Claude Code to:
1. Fetch all engrams tagged `arbFarm.learning`
2. Analyze trade history and error patterns
3. Review pending LLM consensus recommendations
4. Generate actionable profit optimization plan

**Workflow**:
```
/scrape-engrams
├── Fetch learning engrams via engram_get_arbfarm_learning
├── Fetch trade history via engram_get_trade_history
├── Fetch error patterns via engram_get_errors
├── Cross-reference with current strategy configs
└── Generate profit maximization recommendations
```

**Output**: Markdown report with:
- Trade performance summary (win rate, PnL, patterns)
- Top pending recommendations from consensus LLM
- Suggested config changes with confidence scores
- Implementation steps for each recommendation

### MCP Server Configuration (Cursor IDE / Claude Desktop)

ArbFarm exposes 97 MCP tools via standard JSON-RPC at `/mcp/jsonrpc`. To use in external clients:

**Cursor IDE** (`.cursor/mcp.json` or `~/.cursor/mcp.json`):
```json
{
  "mcpServers": {
    "arbfarm": {
      "url": "http://localhost:9007/mcp/jsonrpc"
    }
  }
}
```

**Claude Desktop** (`~/Library/Application Support/Claude/claude_desktop_config.json`):
```json
{
  "mcpServers": {
    "arbfarm": {
      "url": "http://localhost:9007/mcp/jsonrpc"
    }
  }
}
```

**Available MCP Methods**:
- `initialize` - Server handshake with capabilities
- `tools/list` - List all 97 available tools
- `tools/call` - Execute any tool by name
- `resources/list` - List available resources (empty)
- `prompts/list` - List available prompts (empty)
- `ping` - Health check

**Protocol**: MCP 2025-11-25, JSON-RPC 2.0 over HTTP

**Test with curl**:
```bash
# Initialize
curl -X POST http://localhost:9007/mcp/jsonrpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}'

# List tools
curl -X POST http://localhost:9007/mcp/jsonrpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}'

# Call tool
curl -X POST http://localhost:9007/mcp/jsonrpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"engram_get_trade_history","arguments":{"limit":10}}}'
```

## Common Commands

```bash
# Dev environment
just dev-mac              # Start all services
just docs                 # Serve docs locally
just start                # Start infrastructure only

# Quality
cargo fmt && cargo clippy # Rust
ruff format . && ruff check . --fix # Python

# Health checks
curl http://localhost:3000/health   # Erebus
curl http://localhost:9003/health   # Agents
curl http://localhost:9004/health   # Engrams

# Database
just migrate              # Run all migrations
just wipe-db              # Fresh start (deletes data!)
```

## Archives

Detailed implementation notes moved to `archive/implementation-notes/`:
- `a2a-implementation.md` - A2A Protocol implementation details
- `development-status.md` - Detailed completion status

Historical design docs in `archive/crossroads/`:
- `CROSSROADS_UI_DESIGN.md`
- `CROSSROADS_BACKEND_PLAN.md`

---

_NullBlock: The void where agentic flows connect, modify, and evolve._
