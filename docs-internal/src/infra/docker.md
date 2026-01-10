# Docker & Containers

**Container-First Architecture** - All infrastructure runs in Docker.

## Golden Rules

### 1. Container-to-Container Communication

```bash
# ✅ CORRECT - use container names
host=nullblock-postgres-erebus port=5432

# ❌ WRONG - never use localhost in containers
host=localhost port=5432

# ❌ WRONG - never use host.docker.internal
host=host.docker.internal port=5440
```

### 2. Network Configuration

All containers join `nullblock-network` bridge network:

```bash
docker run --network nullblock-network ...
```

### 3. Port Mapping

```
External (host) → Internal (container)
5440 → 5432   # Erebus PostgreSQL
5441 → 5432   # Agents PostgreSQL
9092 → 9092   # Kafka
2181 → 2181   # Zookeeper
```

### 4. System-Agnostic Design

- Identical behavior on macOS and Linux
- Use Docker networking, not OS workarounds
- Test on both platforms

## Container Setup

### Start Infrastructure

```bash
cd ~/nullblock && just start
```

This creates:
- `nullblock-network` bridge network
- `nullblock-postgres-erebus` (port 5440)
- `nullblock-postgres-agents` (port 5441)
- `nullblock-kafka` (port 9092)
- `nullblock-zookeeper` (port 2181)

### Verify Containers

```bash
docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
```

### Network Inspection

```bash
docker network inspect nullblock-network
```

## PostgreSQL Replication

Erebus → Agents user sync via logical replication:

```sql
-- Erebus: Creates publication
CREATE PUBLICATION erebus_user_sync FOR TABLE user_references;

-- Agents: Creates subscription (uses container names)
CREATE SUBSCRIPTION agents_user_sync
  CONNECTION 'host=nullblock-postgres-erebus port=5432 ...'
  PUBLICATION erebus_user_sync;
```

### Verify Replication

```bash
# Check subscription status
PGPASSWORD="postgres_secure_pass" psql -h localhost -p 5441 -U postgres -d agents \
  -c "SELECT subname, subenabled FROM pg_subscription;"

# Compare user counts
PGPASSWORD="postgres_secure_pass" psql -h localhost -p 5440 -U postgres -d erebus \
  -c "SELECT COUNT(*) FROM user_references;"
PGPASSWORD="postgres_secure_pass" psql -h localhost -p 5441 -U postgres -d agents \
  -c "SELECT COUNT(*) FROM user_references;"
```

## Troubleshooting

### Container Won't Start

```bash
# Check logs
docker logs nullblock-postgres-erebus

# Remove and recreate
docker rm -f nullblock-postgres-erebus
just start
```

### Network Issues

```bash
# Verify network exists
docker network ls | grep nullblock

# Recreate network
docker network rm nullblock-network
docker network create nullblock-network
```

### Replication Broken

```bash
# Drop and recreate subscription
PGPASSWORD="..." psql -h localhost -p 5441 -U postgres -d agents \
  -c "DROP SUBSCRIPTION agents_user_sync;"

# Run migration again
cd svc/nullblock-agents && sqlx migrate run
```

## Related

- [Database Architecture](./database.md)
- [Tmuxinator Setup](./tmuxinator.md)
