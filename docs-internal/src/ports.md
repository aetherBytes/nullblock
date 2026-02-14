# Service Ports

Quick reference for all NullBlock service ports.

## Production Services

| Port | Service | Description |
|------|---------|-------------|
| **3000** | Erebus | Unified router + Crossroads marketplace |
| **5173** | Hecate Frontend | React development server |
| **8001** | Protocols | A2A/MCP protocol server |
| **9003** | Agents | HECATE agent API |
| **9004** | Engrams | Memory/context layer |
| **8002** | Content | Social media content service |
| **9006** | Poly Mev | Polymarket scanner (planned) |
| **9007** | ArbFarm | Solana MEV agent swarm |

## Infrastructure

| Port | Service | Description |
|------|---------|-------------|
| **5440** | PostgreSQL (Erebus) | User data, marketplace, wallets |
| **5441** | PostgreSQL (Agents) | Tasks, agents, engrams |
| **9092** | Kafka | Event streaming |
| **2181** | Zookeeper | Kafka coordination |
| **6379** | Redis | Caching (optional) |

## Development Tools

| Port | Service | Description |
|------|---------|-------------|
| **3001** | mdBook | Internal documentation (this site) |

## Port Mapping

### Docker Container Ports

```
External (host) → Internal (container)
5440 → 5432   # Erebus PostgreSQL
5441 → 5432   # Agents PostgreSQL
9092 → 9092   # Kafka
2181 → 2181   # Zookeeper
```

### Container-to-Container Communication

```
# CORRECT - use container names + internal ports
host=nullblock-postgres-erebus port=5432

# WRONG - don't use localhost or external ports
host=localhost port=5440
```

## Health Check URLs

```bash
# All services
curl http://localhost:3000/health   # Erebus
curl http://localhost:9003/health   # Agents
curl http://localhost:9004/health   # Engrams
curl http://localhost:8001/health   # Protocols
curl http://localhost:9007/health   # ArbFarm
```
