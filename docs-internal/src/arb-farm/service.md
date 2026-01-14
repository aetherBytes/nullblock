# ArbFarm Service Architecture

ArbFarm is a Solana MEV Agent Swarm service that provides autonomous multi-agent trading capabilities.

## Overview

| Aspect | Value |
|--------|-------|
| **Port** | 9007 |
| **Language** | Rust |
| **Database** | PostgreSQL (shared with agents on 5441) |
| **Direct Routes** | `/health`, `/scanner/*`, `/edges/*`, `/trades/*`, `/curves/*`, `/research/*`, `/kol/*`, `/threat/*` |

## Service Structure

```
svc/arb-farm/
├── Cargo.toml
├── .env.dev
└── src/
    ├── main.rs              # Service entrypoint + router
    ├── config.rs            # Configuration from environment
    ├── error.rs             # AppError enum with HTTP mapping
    ├── server.rs            # AppState + event bus
    │
    ├── agents/              # Agent implementations
    │   ├── mod.rs
    │   ├── scanner.rs       # Venue Scanner Agent
    │   └── ...
    │
    ├── events/              # Event bus (GOLDEN RULE)
    │   ├── mod.rs
    │   ├── types.rs         # ArbEvent, EventSource, AgentType
    │   ├── topics.rs        # Topic constants
    │   └── bus.rs           # Publish/subscribe implementation
    │
    ├── threat/              # Threat detection subsystem
    │   ├── mod.rs
    │   ├── score.rs         # ThreatScore calculation
    │   └── external/        # External API clients
    │       ├── rugcheck.rs  # RugCheck API
    │       ├── goplus.rs    # GoPlus Security
    │       └── birdeye.rs   # Birdeye holder analysis
    │
    ├── research/            # Research/DD subsystem
    │   ├── mod.rs           # Module exports
    │   ├── url_ingest.rs    # URL ingestion + HTML parsing
    │   ├── strategy_extract.rs  # LLM strategy extraction
    │   ├── backtest.rs      # Strategy backtesting
    │   └── social_monitor.rs    # X/Twitter monitoring
    │
    ├── venues/              # MEV venue implementations
    │   ├── mod.rs
    │   ├── traits.rs        # MevVenue trait
    │   ├── dex/             # DEX venues
    │   │   ├── jupiter.rs
    │   │   └── raydium.rs
    │   ├── curves/          # Bonding curves
    │   │   ├── pump_fun.rs
    │   │   └── moonshot.rs
    │   └── lending/         # Lending protocols
    │       └── marginfi.rs
    │
    ├── consensus/           # Multi-LLM consensus
    │   ├── mod.rs
    │   ├── engine.rs        # Consensus orchestration
    │   └── providers/       # LLM providers
    │       └── openrouter.rs
    │
    ├── execution/           # Trade execution
    │   ├── mod.rs
    │   ├── jito.rs          # Jito bundle submission
    │   └── simulation.rs    # Pre-execution simulation
    │
    ├── models/              # Core data models
    │   ├── mod.rs
    │   ├── edge.rs          # Edge (opportunity)
    │   ├── strategy.rs      # Trading strategy
    │   ├── trade.rs         # Trade execution record
    │   ├── signal.rs        # Scanner signal
    │   └── venue.rs         # MEV venue types
    │
    ├── handlers/            # HTTP handlers
    │   ├── mod.rs
    │   ├── health.rs        # Health check
    │   ├── scanner.rs       # Scanner endpoints
    │   ├── edges.rs         # Edge CRUD + execution
    │   ├── trades.rs        # Trade history + stats
    │   ├── curves.rs        # Bonding curve endpoints
    │   ├── research.rs      # Research/DD endpoints
    │   ├── kol.rs           # KOL tracking + copy trading
    │   ├── threat.rs        # Threat detection endpoints
    │   └── sse.rs           # Server-sent events
    │
    ├── mcp/                 # MCP tool definitions
    │   ├── mod.rs
    │   └── tools.rs         # Tool registration
    │
    └── database/            # Database layer
        └── mod.rs
```

## Configuration

All external API URLs are configurable via environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `ARB_FARM_PORT` | 9007 | Service port |
| `ARB_FARM_DATABASE_URL` | postgres://...5441/agents | Database connection |
| `JUPITER_API_URL` | https://quote-api.jup.ag/v6 | Jupiter DEX |
| `RAYDIUM_API_URL` | https://api-v3.raydium.io | Raydium AMM |
| `PUMP_FUN_API_URL` | https://pumpportal.fun/api | pump.fun bonding curves |
| `HELIUS_API_URL` | https://mainnet.helius-rpc.com | Helius RPC |
| `BIRDEYE_API_URL` | https://public-api.birdeye.so | Market data |
| `JITO_API_URL` | https://mainnet.block-engine.jito.wtf | Jito bundles |
| `RUGCHECK_API_URL` | https://api.rugcheck.xyz/v1 | Threat intel |
| `GOPLUS_API_URL` | https://api.gopluslabs.io/api/v1 | Honeypot detection |
| `OPENROUTER_API_URL` | https://openrouter.ai/api/v1 | Multi-LLM consensus |

## Agent Types

The swarm consists of 10 specialized agents:

| Agent | Purpose |
|-------|---------|
| **Scanner** | Venue scanning, signal detection |
| **Refiner** | Signal filtering and enrichment |
| **MevHunter** | Advanced MEV pattern detection |
| **Executor** | Trade execution via Jito |
| **StrategyEngine** | Strategy routing and classification |
| **ResearchDd** | URL ingestion, strategy extraction |
| **CopyTrade** | KOL tracking, copy trading |
| **ThreatDetector** | Rug pull, honeypot detection |
| **EngramHarvester** | Pattern extraction, engram storage |
| **Overseer** | Swarm health, auto-recovery |

## Atomicity Classification

Trades are classified by risk profile:

| Level | Description | Risk |
|-------|-------------|------|
| **FullyAtomic** | Flash loans, Jito bundles | Zero capital (reverts if unprofitable) |
| **PartiallyAtomic** | Some legs atomic | Mixed |
| **NonAtomic** | Cross-chain, delayed exit | Capital at risk |

Fully atomic trades are executed aggressively with lower profit thresholds since the only risk is gas cost.

## Implementation Status

| Phase | Description | Status |
|-------|-------------|--------|
| 1 | Service Scaffold + Core Models | ✅ Complete |
| 2 | Venue Scanner + Strategy Engine | ✅ Complete |
| 3 | Execution Engine + Risk Management | ✅ Complete |
| 4 | Bonding Curve Integration | ✅ Complete |
| 5 | MEV Detection (DEX Arb, Liquidations, JIT) | ✅ Complete |
| 6 | Research/DD Agent + URL Ingestion | ✅ Complete |
| 7 | KOL Tracking + Copy Trading | ✅ Complete |
| 8 | Threat Detection + Safety | ✅ Complete |
| 9 | Engram Integration + Learning | ✅ Complete |
| 10 | Swarm Orchestration + Resilience | ✅ Complete |
| 11 | Frontend Dashboard (MemCache Integration) | ✅ Complete |
| 12 | Crossroads Integration + Revenue | ⏳ Next |

## Running the Service

```bash
cd svc/arb-farm
cargo run
```

The service starts on port 9007. Health check: `curl http://localhost:9007/health`

## Related

- [Development Guide](./development.md)
- [Research Module](./research.md)
- [Bonding Curves](./curves.md)
- [KOL Tracking](./kol.md)
- [Threat Detection](./threat.md)
- [Event Bus](./events.md)
- [API Reference](./api.md)
- [Implementation Plan](./plan.md)
