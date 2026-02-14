# NullBlock Internal Documentation

```
 _   _       _ _ ____  _            _
| \ | |_   _| | | __ )| | ___   ___| | __
|  \| | | | | | |  _ \| |/ _ \ / __| |/ /
| |\  | |_| | | | |_) | | (_) | (__|   <
|_| \_|\__,_|_|_|____/|_|\___/ \___|_|\_\
```

**The Agent Mesh Platform for Onchain Automation**

---

## Mission

In a rapidly expanding onchain automated world, we are building the picks and axes for this digital gold rush. NullBlock empowers builders with the essential tools to create, deploy, and profit from intelligent agent workflows.

---

## Quick Links

| I want to... | Go to... |
|--------------|----------|
| **Understand the architecture** | [Architecture Overview](./architecture.md) |
| **Start the dev environment** | [Quick Start](./quickstart.md) |
| **Learn about Engrams** | [Engrams Service](./services/engrams.md) |
| **See ArbFarm plan** | [ArbFarm Plan](./arb-farm/plan.md) |
| **Check API endpoints** | [API Reference](./reference/api.md) |

---

## Current Development Focus

### ArbFarm (Solana MEV Agent Swarm)

Autonomous multi-agent system for capturing MEV opportunities on Solana. See [ArbFarm Plan](./arb-farm/plan.md).

| Phase | Status |
|-------|--------|
| 1-5. Core + Curves + Execution | ✅ Complete |
| 6. Research/DD Agent | ❌ Not Started |
| 7. KOL Tracking + Copy Trading | ✅ Complete |
| 8. Threat Detection | ❌ Not Started |
| 9-11. Engrams + Swarm + Dashboard | ✅ Complete |
| 12. Crossroads Integration | ⏳ Next |

### Also Active

- **NullBlock Content Service** — Social media content generation (Phase 1-5 Complete)
- **Poly Mev** — Polymarket trading (planned)

### Paused

- **Echo Factory** — X/Twitter COW ([archived](./archive/echo-factory/plan.md))

---

## Service Status

| Service | Port | Description |
|---------|------|-------------|
| **Erebus** | 3000 | Unified router + Crossroads |
| **Hecate Frontend** | 5173 | React interface |
| **Protocols** | 8001 | A2A/MCP server |
| **Agents** | 9003 | HECATE agent API |
| **Engrams** | 9004 | Memory/context layer |
| **ArbFarm** | 9007 | Solana MEV agent swarm |
| **Content** | 8002 | Social media content service |

---

## Connect

- **Official**: [@Nullblock_io](https://x.com/Nullblock_io)
- **SDK**: [nullblock-sdk](https://github.com/aetherBytes/nullblock-sdk)
- **Site**: NullBlock.io _(Coming Soon)_
