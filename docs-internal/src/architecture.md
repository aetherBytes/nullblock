# Architecture Overview

## System Diagram

```
Frontend (5173) → Erebus (3000) → Backend Services
                       │
         ┌─────────────┼─────────────┐
         │             │             │
    Crossroads    Engrams (9004)   Services
    (Internal)                        │
                           ┌──────────┼──────────┐
                           │          │          │
                      Agents (9003) ArbFarm   Content
                      Protocols     (9007)    (8002)
                      (8001)
```

## Golden Rule

> **ALL frontend requests MUST route through Erebus (port 3000). NO direct service connections.**

```
Frontend → Erebus → {
  Wallet operations → Internal handlers
  Agent chat → Hecate (9003)
  A2A/MCP → Protocols (8001)
  Engrams → Engrams Service (9004)
  Marketplace → Crossroads (internal)
}
```

## Key Features

- **Agent Orchestration**: Multi-model LLM coordination via Hecate
- **Unified Router**: Single entry point through Erebus (Port 3000)
- **Marketplace**: Crossroads AI service discovery and monetization
- **Engrams**: Universal memory layer for persistent context
- **COWs**: Constellations of Work - curated tool suites
- **Protocol Agnostic**: A2A, MCP, custom protocols
- **Real-time**: WebSocket chat, live task management

## Core Services

### Production-Ready ✅

| Service | Location | Description |
|---------|----------|-------------|
| **Protocols** | `/svc/nullblock-protocols/` | Multi-protocol server (A2A, MCP) |
| **Agents** | `/svc/nullblock-agents/` | Agent suite (HECATE, Siren, LLM) |
| **Erebus** | `/svc/erebus/` | Unified routing server |
| **Crossroads** | `/svc/erebus/src/resources/crossroads/` | Marketplace subsystem |
| **Hecate Frontend** | `/svc/hecate/` | React interface |
| **Engrams** | `/svc/nullblock-engrams/` | Memory/context layer |

### In Development 🔄

| Service | Location | Description |
|---------|----------|-------------|
| **ArbFarm** | `/svc/arb-farm/` | Solana MEV agent swarm |
| **Content** | `/svc/nullblock-content/` | Social media content service |
