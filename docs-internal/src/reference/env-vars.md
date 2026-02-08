# Environment Variables

Complete reference for NullBlock environment configuration.

## Backend Services

### Erebus

```bash
EREBUS_PORT=3000
DATABASE_URL=postgresql://postgres:$POSTGRES_PASSWORD@localhost:5440/erebus

# Service URLs (for proxying)
AGENTS_SERVICE_URL=http://localhost:9003
PROTOCOLS_SERVICE_URL=http://localhost:8001
ENGRAMS_SERVICE_URL=http://localhost:9004
```

### Agents

```bash
DATABASE_URL=postgresql://postgres:$POSTGRES_PASSWORD@localhost:5441/agents
EREBUS_BASE_URL=http://localhost:3000
KAFKA_BOOTSTRAP_SERVERS=localhost:9092

# LLM Configuration
DEFAULT_LLM_MODEL=cognitivecomputations/dolphin3.0-mistral-24b:free
LLM_REQUEST_TIMEOUT_MS=300000
```

### Engrams

```bash
DATABASE_URL=postgresql://postgres:$POSTGRES_PASSWORD@localhost:5441/agents
ENGRAMS_PORT=9004
```

### Protocols

```bash
PROTOCOLS_PORT=8001
AGENTS_SERVICE_URL=http://localhost:9003
```

## Frontend (Hecate)

```bash
VITE_EREBUS_API_URL=http://localhost:3000
VITE_PROTOCOLS_API_URL=http://localhost:8001
VITE_HECATE_API_URL=http://localhost:9003
CORS_ORIGINS=http://localhost:5173,http://localhost:3000
```

## Infrastructure

### PostgreSQL

```bash
# Erebus DB
POSTGRES_HOST=localhost
POSTGRES_PORT=5440
POSTGRES_USER=postgres
POSTGRES_PASSWORD=$POSTGRES_PASSWORD
POSTGRES_DB=erebus

# Agents DB
POSTGRES_HOST=localhost
POSTGRES_PORT=5441
POSTGRES_USER=postgres
POSTGRES_PASSWORD=$POSTGRES_PASSWORD
POSTGRES_DB=agents
```

### Kafka

```bash
KAFKA_BOOTSTRAP_SERVERS=localhost:9092
KAFKA_ZOOKEEPER=localhost:2181
```

### Redis (Optional)

```bash
REDIS_URL=redis://localhost:6379
```

## API Keys

**IMPORTANT**: LLM API keys are stored in Erebus database, NOT in `.env.dev`.

```bash
# Seed keys from secure storage
cd svc/erebus && cargo run --bin seed_agent_keys
```

### Where Keys Are Stored

| Key Type | Location |
|----------|----------|
| LLM Provider Keys | Erebus DB (`agent_api_keys` table) |
| User Session Keys | Runtime (memory) |
| OAuth Tokens | Echo Factory DB (encrypted) |

## Environment File Setup

Create `.env.dev` in project root:

```bash
# ~/nullblock/.env.dev
DATABASE_URL=postgresql://postgres:$POSTGRES_PASSWORD@localhost:5441/agents
EREBUS_BASE_URL=http://localhost:3000
# ... other non-secret config
```

Symlink to services:

```bash
ln -s ../../.env.dev svc/nullblock-agents/.env.dev
ln -s ../../.env.dev svc/erebus/.env.dev
ln -s ../../.env.dev svc/nullblock-engrams/.env.dev
```

## Docker Container Variables

When running in containers, use container names:

```bash
# ✅ Container-to-container
DATABASE_URL=postgresql://postgres:pass@nullblock-postgres-agents:5432/agents

# ❌ Host access (don't use in containers)
DATABASE_URL=postgresql://postgres:pass@localhost:5441/agents
```

## Related

- [Docker & Containers](../infra/docker.md)
- [Database Architecture](../infra/database.md)
