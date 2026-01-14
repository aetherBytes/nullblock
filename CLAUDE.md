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

## Current Focus: Poly Mev (Polymarket Trading Agent)

Building tools to facilitate Polymarket trading - a new agent service for prediction market intelligence. See [Poly Mev Plan](docs-internal/src/poly-mev/plan.md).

| Phase | Status |
|-------|--------|
| 1. Service Scaffold | ⏳ Next |
| 2. Polymarket API Integration | ⏳ Pending |
| 3. Market Analysis Tools | ⏳ Pending |
| 4. Trading Strategies | ⏳ Pending |
| 5. Frontend | ⏳ Pending |

**Paused**: Echo Factory (X/Twitter COW) - [archived plan](docs-internal/src/archive/echo-factory/plan.md)

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
| `/mcp/*` | MCP Protocol (2025-11-25) |
| `/a2a/*` | A2A Protocol |

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
