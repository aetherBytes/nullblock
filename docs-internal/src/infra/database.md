# Database Architecture

**Dual PostgreSQL** - Separated databases for service isolation with replication.

## Databases

| Database | Port | Owner | Contents |
|----------|------|-------|----------|
| **Erebus** | 5440 | Erebus | Users, wallets, marketplace, API keys |
| **Agents** | 5441 | Agents | Tasks, agents, engrams, user replica |

## Erebus Database (Port 5440)

### Tables

| Table | Purpose |
|-------|---------|
| `user_references` | User identity (wallet, API key, OAuth) |
| `wallets` | Web3 wallet connections |
| `sessions` | Auth sessions |
| `agent_api_keys` | Encrypted LLM provider keys |
| `crossroads_*` | Marketplace tables |

### Connection

```bash
PGPASSWORD="postgres_secure_pass" psql -h localhost -p 5440 -U postgres -d erebus
```

## Agents Database (Port 5441)

### Tables

| Table | Purpose |
|-------|---------|
| `tasks` | A2A protocol tasks |
| `engrams` | Memory/context storage |
| `engram_history` | Version audit trail |
| `user_references` | **READ-ONLY replica** |

### Connection

```bash
PGPASSWORD="postgres_secure_pass" psql -h localhost -p 5441 -U postgres -d agents
```

## Replication

PostgreSQL logical replication syncs `user_references` from Erebus to Agents.

### Setup (Automatic via Migrations)

1. Erebus migration creates publication
2. Agents migration creates subscription
3. Initial data backfilled automatically

### How It Works

```
Erebus INSERT/UPDATE/DELETE → Publication → Subscription → Agents replica
```

Latency: < 1 second

### Verify

```bash
# Check subscription
psql -p 5441 -d agents -c "SELECT * FROM pg_subscription;"

# Compare counts
psql -p 5440 -d erebus -c "SELECT COUNT(*) FROM user_references;"
psql -p 5441 -d agents -c "SELECT COUNT(*) FROM user_references;"
```

## Migrations

### Running Migrations

```bash
# Erebus
cd svc/erebus && sqlx migrate run

# Agents
cd svc/nullblock-agents && sqlx migrate run

# Engrams (handled by shell script)
./scripts/start-engrams.sh  # runs migrations before service
```

### Creating Migrations

```bash
sqlx migrate add -r <migration_name>
```

Use `IF NOT EXISTS` for idempotent migrations.

## Backup & Restore

### Backup

```bash
pg_dump -h localhost -p 5440 -U postgres erebus > erebus_backup.sql
pg_dump -h localhost -p 5441 -U postgres agents > agents_backup.sql
```

### Restore

```bash
psql -h localhost -p 5440 -U postgres erebus < erebus_backup.sql
psql -h localhost -p 5441 -U postgres agents < agents_backup.sql
```

## Related

- [Docker & Containers](./docker.md)
- [Service Ports](../ports.md)
