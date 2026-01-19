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
| 7. KOL Tracking + Copy Trading | ⚠️ Partial (discovery works, copy stubbed) |
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

## ArbFarm Production Status (Audit: 2026-01-19)

**Overall**: 95% Production Ready

### Implemented (Live Trading Ready)
- ✅ Exit transactions saved to engrams (buy AND sell tracked)
- ✅ MCP tool annotations (97 tools with readOnlyHint/destructiveHint/idempotentHint)
- ✅ A2A tags exposed (`arbFarm.learning` on 6 learning tools)
- ✅ Wallet funding validated at startup (blocks if < 0.05 SOL)
- ✅ Position monitor auto-starts with curve support + engrams
- ✅ Frontend service methods deduplicated

### Stubbed Features (Generate Warnings - Safe to Ignore)
These features are scaffolded but not implemented. Warnings are expected:

| Module | Warnings | Status |
|--------|----------|--------|
| `events/topics.rs` | 66 | Event constants for future features |
| `agents/mev_hunter.rs` | 16 | DEX arbitrage detection (stubbed) |
| `venues/dex/raydium.rs` | 8 | Raydium integration (stubbed) |
| `consensus/providers/*` | 14 | Anthropic/OpenAI direct providers (using OpenRouter) |
| `agents/graduation_tracker.rs` | 8 | Graduation tracking (stubbed) |

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
