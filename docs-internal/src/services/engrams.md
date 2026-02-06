# Engrams Service

**Universal Memory Layer** - Persistent context storage for all NullBlock agents, workflows, and COWs.

## Overview

| Property | Value |
|----------|-------|
| **Port** | 9004 |
| **Location** | `/svc/nullblock-engrams/` |
| **Database** | PostgreSQL (agents DB, port 5441) |
| **Role** | Context persistence, agent memory |

Engrams are the **"long-term scars" of the mesh** — shared, forkable, evolvable knowledge that any agent, workflow, or service can draw from and contribute to.

## Engram Types

| Type | Description | Example |
|------|-------------|---------|
| `persona` | Character/voice definitions | Twitter persona for Echo Factory |
| `preference` | User settings | UI theme, notification settings |
| `strategy` | Decision frameworks | Trading entry/exit rules |
| `knowledge` | Domain information | Blockchain mechanics, market data |
| `compliance` | Regulatory constraints | Jurisdiction restrictions |

## Key Field Design

The `key` field is a **namespaced lookup path**. Uniqueness is scoped to `(wallet_address, key, version)`.

### Convention

```
{domain}.{name}           → twitter.main_persona
{domain}.{sub}.{name}     → trading.strategies.momentum
{app}.{feature}.{id}      → echo.personas.crypto_sage
```

### Why Keys?

- **Human-readable**: "get my twitter persona"
- **Versioning**: same key can have v1, v2, v3...
- **Cross-agent lookup**: any agent can fetch wallet engrams

## API Endpoints

### CRUD

```bash
# Create
curl -X POST http://localhost:9004/engrams \
  -H "Content-Type: application/json" \
  -d '{
    "wallet_address": "0x742d35Cc...",
    "engram_type": "persona",
    "key": "twitter.crypto_sage",
    "content": {"name": "Crypto Sage", "voice": "casual"},
    "tags": ["twitter", "persona"]
  }'

# List all
curl http://localhost:9004/engrams

# Get by ID
curl http://localhost:9004/engrams/{id}

# Update (creates new version)
curl -X PUT http://localhost:9004/engrams/{id} \
  -d '{"content": {"name": "Crypto Sage v2"}}'

# Delete
curl -X DELETE http://localhost:9004/engrams/{id}
```

### Wallet Operations

```bash
# Get all for wallet
curl http://localhost:9004/engrams/wallet/0x742d35Cc...

# Get by wallet + key
curl http://localhost:9004/engrams/wallet/0x742d35Cc.../twitter.crypto_sage
```

### Search

```bash
curl -X POST http://localhost:9004/engrams/search \
  -d '{
    "wallet_address": "0x742d35Cc...",
    "engram_type": "persona",
    "tags": ["twitter"]
  }'
```

### Fork & Publish

```bash
# Fork to another wallet
curl -X POST http://localhost:9004/engrams/{id}/fork \
  -d '{"target_wallet": "0xNewWallet..."}'

# Make public
curl -X POST http://localhost:9004/engrams/{id}/publish
```

## Versioning

Each update creates a new version:

```
wallet: 0x742d35Cc...
key: twitter.crypto_sage
  version: 1  ← original
  version: 2  ← updated voice
  version: 3  ← added hashtags (current)
```

## Database Schema

```sql
CREATE TABLE engrams (
    id UUID PRIMARY KEY,
    wallet_address VARCHAR NOT NULL,
    engram_type VARCHAR NOT NULL,
    key VARCHAR NOT NULL,
    tags TEXT[] DEFAULT '{}',
    content JSONB NOT NULL,
    summary TEXT,
    version INTEGER DEFAULT 1,
    parent_id UUID,
    lineage_root_id UUID,
    is_public BOOLEAN DEFAULT false,
    is_mintable BOOLEAN DEFAULT false,
    price_mon DECIMAL(18, 8),
    royalty_percent INTEGER DEFAULT 5,
    UNIQUE (wallet_address, key, version)
);
```

## Use Cases

- **Echo Factory**: Store Twitter personas, content style
- **HECATE Memory**: Save conversation summaries
- **Cross-Agent Context**: Shared engram pool
- **Monetization**: Public engrams on Crossroads

## Configuration

```bash
DATABASE_URL=postgresql://postgres:REDACTED_DB_PASS@localhost:5441/agents
ENGRAMS_PORT=9004
```

## Related

- [Echo Factory Plan](../echo-factory/plan.md)
- [Architecture Overview](../architecture.md)
