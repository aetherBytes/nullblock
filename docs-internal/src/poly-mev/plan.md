# PolySwarm COW Implementation Plan

**NullBlock's Polymarket Agent Swarm** - Autonomous multi-agent system for prediction market dominance.

## Vision

A no-failure, high-reliability agent swarm that dominates Polymarket with AI-powered insight, probability refinement, arbitrage detection, and autonomous execution. Dogfoods NullBlock infrastructure while generating revenue and viral growth.

## Status

| Phase | Status |
|-------|--------|
| **Phase 1: Service Scaffold** | ⏳ Planned |
| **Phase 2: Market Scanner** | ⏳ Planned → [Detailed Plan](./scanner.md) |
| **Phase 3: Probability Refiner** | ⏳ Pending |
| **Phase 4: Arb Detector + Execution** | ⏳ Pending |
| **Phase 5: Engram Generator + Overseer** | ⏳ Pending |
| **Phase 6: Monad Deploy + Crossroads** | ⏳ Pending |

> **Current Focus**: Market Scanner - a forkable COW tool for real-time Polymarket intelligence. See [scanner.md](./scanner.md) for detailed implementation plan.

## Swarm Architecture

```
                    ┌─────────────────────┐
                    │  Resilience Overseer │
                    │    (Meta-Agent)      │
                    └──────────┬──────────┘
                               │ monitors/recovers
           ┌───────────────────┼───────────────────┐
           │                   │                   │
    ┌──────▼──────┐    ┌──────▼──────┐    ┌──────▼──────┐
    │   Market    │    │ Probability │    │     Arb     │
    │   Scanner   │───▶│   Refiner   │───▶│  Detector   │
    └──────┬──────┘    └──────┬──────┘    └──────┬──────┘
           │                  │                   │
           │           ┌──────▼──────┐           │
           └──────────▶│  Execution  │◀──────────┘
                       │   Engine    │
                       └──────┬──────┘
                              │
                       ┌──────▼──────┐
                       │   Insight   │
                       │   Engram    │
                       │  Generator  │
                       └─────────────┘
```

## Swarm Components

| Agent | Role | Reliability |
|-------|------|-------------|
| **Market Scanner** | Real-time event/market fetch via Gamma API | Redundant polling, fallback nodes |
| **Probability Refiner** | Sentiment/news/onchain signal analysis | Engram-backed historical accuracy |
| **Arb Detector** | Cross-market mispricing detection | Multi-threaded with auto-retry |
| **Execution Engine** | CLOB order placement/cancellation | Gasless via relayer, onchain fallback |
| **Insight Engram Generator** | Distill edges into tradable engrams | Persistent, shareable |
| **Resilience Overseer** | Swarm health monitoring, auto-recovery | Meta-agent, always-on |

## No-Failure Policy

| Strategy | Implementation |
|----------|----------------|
| **Redundancy** | Duplicate agents run in parallel; majority vote |
| **Auto-Recovery** | API failure → cached engrams or delayed execution |
| **Engram Learning** | Bad trades → new avoidance engrams |
| **Testing** | 100% unit/integration coverage + simulated events |

## NullBlock Integration

| Component | Usage |
|-----------|-------|
| **HECATE** | Guides swarm composition ("Shall I weave a PolySwarm?") |
| **Engrams** | Persistent memory, failure learning, edge storage |
| **Crossroads** | Mint COW as NFT suite (base free, premium $0.2 SOL) |
| **Void Canvas** | Swarm visualization (orbs = agents, dendrites = interactions) |
| **Mem Cache** | Pinned markets, saved swarm configs |

## Polymarket API Integration

### Gamma API (Read)

```
GET /events              # List events
GET /markets             # Market data
GET /prices?markets=...  # Real-time prices
```

### CLOB API (Trade)

```
POST /order              # Place order
DELETE /order/:id        # Cancel order
GET /orderbook/:token_id # Order book depth
```

### WebSocket (Real-time)

```
wss://clob.polymarket.com  # Price updates
```

## Service Structure

```
svc/poly-mev/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── agents/
│   │   ├── scanner.rs
│   │   ├── refiner.rs
│   │   ├── arb_detector.rs
│   │   ├── executor.rs
│   │   ├── engram_generator.rs
│   │   └── overseer.rs
│   ├── api/
│   │   ├── gamma.rs          # Gamma API client
│   │   ├── clob.rs           # CLOB API client
│   │   └── websocket.rs      # WS price feeds
│   ├── models/
│   │   ├── market.rs
│   │   ├── order.rs
│   │   └── edge.rs
│   └── services/
│       ├── engram_client.rs
│       └── monad_client.rs
└── migrations/
```

## API Endpoints (via Erebus)

### Markets

```bash
GET  /api/poly/markets              # List tracked markets
GET  /api/poly/markets/:id          # Market details + odds
POST /api/poly/markets/:id/track    # Start tracking
```

### Swarm

```bash
POST /api/poly/swarm/deploy         # Deploy swarm config
GET  /api/poly/swarm/status         # Swarm health
POST /api/poly/swarm/pause          # Pause execution
```

### Edges

```bash
GET  /api/poly/edges                # List detected edges
POST /api/poly/edges/:id/execute    # Execute trade
GET  /api/poly/edges/:id/engram     # Get edge as engram
```

## Monad Integration

| Purpose | Contract |
|---------|----------|
| **NFT Minting** | ERC-721 for COW components |
| **Execution Proxy** | Simple proxy for swarm trades |
| **Fast Polling** | 10k TPS for real-time arb detection |

## Revenue Model

| Stream | Rate |
|--------|------|
| **Execution Fees** | 3% on trades |
| **Premium Engrams** | $10/mo subscription |
| **Crossroads NFTs** | Base free, premium agents $0.2 SOL |

## Viral Hooks

- Swarm posts insights to X/Telegram ("Edge detected - join NullBlock!")
- Tie to Echo Factory for automated posting
- Demo video showing arb on test market

## Tech Stack

| Layer | Technology |
|-------|------------|
| **Backend** | Rust (extend existing services) |
| **Frontend** | React + Three.js (void swarm viz) |
| **Onchain** | Monad (Foundry contracts, ethers.js) |
| **APIs** | Polymarket Gamma/CLOB + WebSockets |

## Related

- [Engrams Service](../services/engrams.md)
- [Crossroads Marketplace](../services/crossroads.md)
- [HECATE Agent](../agents/hecate.md)
