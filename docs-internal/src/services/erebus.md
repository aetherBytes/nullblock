# Erebus Router

**The unified gateway** - All frontend requests route through Erebus. No exceptions.

## Overview

| Property | Value |
|----------|-------|
| **Port** | 3000 |
| **Location** | `/svc/erebus/` |
| **Database** | PostgreSQL (port 5440) |
| **Role** | API gateway, user management, Crossroads marketplace |

## Golden Rule

```
Frontend → Erebus → {
  Wallet operations → Internal handlers
  Agent chat → Hecate (9003)
  A2A/MCP → Protocols (8001)
  Engrams → Engrams Service (9004)
  Marketplace → Crossroads (internal)
}
```

**NEVER** connect frontend directly to backend services.

## API Endpoints

### Users (Erebus-Owned)

```bash
POST /api/users/register     # Create/update user
POST /api/users/lookup       # Find by source identifier
GET  /api/users/:user_id     # Get by UUID
```

### Wallets

```bash
POST /api/wallets/challenge  # Request signature challenge
POST /api/wallets/verify     # Verify signature, get session
```

### Agents (Proxied to 9003)

```bash
POST /api/agents/chat        # Chat with agent
GET  /api/agents/tasks       # List tasks
POST /api/agents/tasks       # Create task
```

### Engrams (Proxied to 9004)

```bash
GET  /api/engrams            # List all
POST /api/engrams            # Create
GET  /api/engrams/:id        # Get by ID
GET  /api/engrams/wallet/:addr  # Get by wallet
```

### Marketplace

```bash
GET  /api/marketplace/listings     # Browse
POST /api/marketplace/listings     # Create listing
GET  /api/marketplace/search       # Search
GET  /api/marketplace/featured     # Featured services
```

### Discovery

```bash
GET  /api/discovery/agents         # Auto-discover agents
GET  /api/discovery/health/:endpoint  # Check service health
```

## Database Schema

Erebus owns the following tables:

- `user_references` - User identity (wallet, API key, OAuth)
- `wallets` - Web3 wallet connections
- `sessions` - Authentication sessions
- `crossroads_*` - Marketplace tables
- `agent_api_keys` - Encrypted LLM provider keys

## Web3 Authentication Flow

```
1. Wallet Connect →
2. POST /api/wallets/challenge →
3. User signs message →
4. POST /api/wallets/verify →
5. Auto-register user if new →
6. Return session token
```

## Configuration

```bash
# Environment
EREBUS_PORT=3000
DATABASE_URL=postgresql://postgres:postgres_secure_pass@localhost:5440/erebus

# Service URLs (for proxying)
AGENTS_SERVICE_URL=http://localhost:9003
PROTOCOLS_SERVICE_URL=http://localhost:8001
ENGRAMS_SERVICE_URL=http://localhost:9004
```

## Related

- [Architecture Overview](../architecture.md)
- [Crossroads Marketplace](./crossroads.md)
- [Wallet Integration](../reference/wallet.md)
