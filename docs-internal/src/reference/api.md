# API Endpoints

Complete reference for all NullBlock API endpoints.

**All requests go through Erebus (port 3000).**

## Authentication

### Wallet Challenge

```bash
POST /api/wallets/challenge
Content-Type: application/json

{
  "wallet_address": "0x742d35Cc6634C0532925a3b844Bc9e7595f8fE00",
  "wallet_chain": "ethereum"
}

# Response
{
  "challenge": "Sign this message to authenticate: 0x..."
}
```

### Wallet Verify

```bash
POST /api/wallets/verify
Content-Type: application/json

{
  "wallet_address": "0x742d35Cc...",
  "signature": "0x..."
}

# Response
{
  "success": true,
  "session_token": "eyJ...",
  "user": {...}
}
```

## Users

### Register

```bash
POST /api/users/register
Content-Type: application/json

{
  "source_identifier": "0x742d35Cc...",
  "source_type": {
    "type": "web3_wallet",
    "provider": "metamask",
    "network": "ethereum"
  }
}
```

### Lookup

```bash
POST /api/users/lookup
Content-Type: application/json

{
  "source_identifier": "0x742d35Cc...",
  "source_type": "web3_wallet",
  "network": "ethereum"
}
```

## Agents

### Chat

```bash
POST /api/agents/chat
Content-Type: application/json

{
  "message": "Hello HECATE",
  "wallet_address": "0x742d35Cc...",
  "wallet_chain": "ethereum"
}

# Response
{
  "response": "Welcome to the void, visitor...",
  "model": "cognitivecomputations/dolphin3.0-mistral-24b:free"
}
```

### Tasks

```bash
# Create
POST /api/agents/tasks
{
  "name": "Test Task",
  "description": "A test task",
  "task_type": "system",
  "category": "user_assigned",
  "priority": "medium"
}

# List
GET /api/agents/tasks

# Get
GET /api/agents/tasks/:id

# Process
POST /api/agents/tasks/:id/process
```

## Engrams

### Create

```bash
POST /api/engrams
Content-Type: application/json

{
  "wallet_address": "0x742d35Cc...",
  "engram_type": "persona",
  "key": "twitter.crypto_sage",
  "content": {
    "name": "Crypto Sage",
    "voice": "casual"
  },
  "tags": ["twitter", "persona"]
}
```

### List

```bash
GET /api/engrams
GET /api/engrams/wallet/0x742d35Cc...
GET /api/engrams/wallet/0x742d35Cc.../twitter.crypto_sage
```

### Search

```bash
POST /api/engrams/search
Content-Type: application/json

{
  "wallet_address": "0x742d35Cc...",
  "engram_type": "persona",
  "tags": ["twitter"]
}
```

### Update

```bash
PUT /api/engrams/:id
Content-Type: application/json

{
  "content": {"name": "Crypto Sage v2"}
}
```

### Delete

```bash
DELETE /api/engrams/:id
```

## Marketplace

### Listings

```bash
# Browse
GET /api/marketplace/listings

# Create
POST /api/marketplace/listings
{
  "name": "My Agent",
  "description": "...",
  "category": "Agent",
  "price_type": "Free"
}

# Get
GET /api/marketplace/listings/:id

# Search
POST /api/marketplace/search
{
  "query": "trading",
  "category": "Agent"
}
```

### Discovery

```bash
GET /api/discovery/agents
GET /api/discovery/tools
GET /api/discovery/health/:endpoint
POST /api/discovery/scan
```

### Wallet Stash

```bash
# Get wallet's tool inventory, owned COWs, and unlock progress
GET /api/marketplace/wallet/:address/stash

# Response
{
  "wallet_address": "0x742d35Cc...",
  "owned_cows": [...],
  "owned_tools": [...],
  "unlocked_tabs": ["arbfarm"],
  "unlock_progress": [
    { "cowId": "arbfarm", "owned": 5, "required": 5, "percent": 100 },
    { "cowId": "polymev", "owned": 0, "required": 5, "percent": 0 }
  ]
}

# Get only unlocked tabs
GET /api/marketplace/wallet/:address/unlocks
```

## MCP Protocol

Model Context Protocol (2025-11-25) for AI tool integration.

### JSON-RPC

```bash
POST /mcp/jsonrpc
Content-Type: application/json

# Initialize session
{
  "jsonrpc": "2.0",
  "method": "initialize",
  "params": {
    "protocolVersion": "2025-11-25",
    "capabilities": {},
    "clientInfo": {"name": "my-client", "version": "1.0.0"}
  },
  "id": 1
}

# List tools
{
  "jsonrpc": "2.0",
  "method": "tools/list",
  "id": 2
}

# Call tool
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "list_engrams",
    "arguments": {"wallet_address": "0x742d35Cc..."}
  },
  "id": 3
}
```

### Convenience Endpoints

```bash
GET /mcp/tools       # List available tools (9 total)
GET /mcp/resources   # List available resources
GET /mcp/prompts     # List available prompts
GET /api/tools       # Alias for /mcp/tools
```

### Available Tools

| Tool | Description |
|------|-------------|
| `send_agent_message` | Send message to agent |
| `create_task` | Create new task |
| `get_task_status` | Get task by ID |
| `list_engrams` | List engrams for wallet |
| `get_engram` | Get engram by ID |
| `create_engram` | Create new engram |
| `update_engram` | Update engram |
| `delete_engram` | Delete engram (respects pin protection) |
| `search_engrams` | Search engrams |
| `user_profile_get` | Get user profile by wallet |
| `user_profile_update` | Update user profile field |
| `hecate_remember` | Save context to memory (auto-tagged) |
| `hecate_cleanup` | Compact old conversation sessions |
| `hecate_pin_engram` | Pin engram (protect from deletion) |
| `hecate_unpin_engram` | Remove pin protection |
| `hecate_set_model` | Switch LLM model conversationally |

### Hecate Tools Endpoint

```bash
GET /hecate/tools    # Returns MCP tools via Hecate agent
```

## User API Keys

Per-user API key management for bypassing free-tier LLM limits. Keys are encrypted at rest via AES-256-GCM in Erebus.

```bash
# List user's API keys (masked)
GET /api/users/:user_id/api-keys

# Add API key
POST /api/users/:user_id/api-keys
{
  "provider": "anthropic",
  "api_key": "sk-ant-...",
  "key_name": "My Anthropic Key"
}

# Delete API key
DELETE /api/users/:user_id/api-keys/:key_id
```

Supported providers: `openrouter`, `anthropic`, `openai`, `groq`

## A2A Protocol

### JSON-RPC

```bash
POST /a2a/jsonrpc
Content-Type: application/json

{
  "jsonrpc": "2.0",
  "method": "tasks/get",
  "params": {"id": "uuid"},
  "id": 1
}
```

### REST

```bash
GET /a2a/v1/card
GET /a2a/v1/tasks/:id
GET /a2a/v1/tasks
POST /a2a/v1/messages
```

## ArbFarm (Solana MEV)

All ArbFarm endpoints require authentication via wallet session.

### Scanner

```bash
# Get scanner status
GET /api/arb/scanner/status

# Start/stop scanner
POST /api/arb/scanner/start
POST /api/arb/scanner/stop

# SSE event stream
GET /api/arb/scanner/stream
```

### Edges (Opportunities)

```bash
# List detected edges
GET /api/arb/edges
GET /api/arb/edges?status=detected&edge_type=dex_arb

# Get edge details
GET /api/arb/edges/:id

# Approve/reject/execute
POST /api/arb/edges/:id/approve
POST /api/arb/edges/:id/reject
{
  "reason": "Too risky"
}
POST /api/arb/edges/:id/execute
{
  "max_slippage_bps": 50
}
```

### Strategies

```bash
# List strategies
GET /api/arb/strategies

# Create strategy
POST /api/arb/strategies
{
  "name": "DEX Arb Strategy",
  "strategy_type": "dex_arb",
  "venue_types": ["dex_amm"],
  "execution_mode": "autonomous",
  "risk_params": {
    "max_position_sol": 1.0,
    "min_profit_bps": 25,
    "max_slippage_bps": 50
  }
}

# Toggle enable/disable
POST /api/arb/strategies/:id/toggle
{
  "enabled": true
}
```

### Swarm Management

```bash
# Get swarm health
GET /api/arb/swarm/status

# Pause/resume execution
POST /api/arb/swarm/pause
POST /api/arb/swarm/resume

# List agent statuses
GET /api/arb/swarm/agents
```

### Threat Detection

```bash
# Check token threat score
GET /api/arb/threat/check/:mint

# Check wallet for scam history
GET /api/arb/threat/wallet/:address

# List blocked entities
GET /api/arb/threat/blocked

# Report a threat
POST /api/arb/threat/report
{
  "entity_type": "token",
  "address": "TokenMint...",
  "category": "rug_pull",
  "reason": "Creator dumped 80% of supply"
}

# Get recent alerts
GET /api/arb/threat/alerts
```

### COW Marketplace

```bash
# List all COWs
GET /api/marketplace/arbfarm/cows
GET /api/marketplace/arbfarm/cows?filter=forkable

# Create a COW
POST /api/marketplace/arbfarm/cows
{
  "name": "My DEX Arb Strategy",
  "description": "Custom DEX arbitrage",
  "strategies": [...],
  "venue_types": ["dex_amm"],
  "risk_profile": {
    "profile_type": "balanced",
    "max_position_sol": 2.0
  },
  "price_sol": 0,
  "is_public": true,
  "is_forkable": true
}

# Get COW details
GET /api/marketplace/arbfarm/cows/:id

# Fork a COW
POST /api/marketplace/arbfarm/cows/:id/fork
{
  "name": "My Forked Strategy",
  "risk_profile_overrides": {
    "profile_type": "conservative"
  },
  "inherit_engrams": true
}

# Get COW strategies
GET /api/marketplace/arbfarm/cows/:id/strategies

# Get COW forks
GET /api/marketplace/arbfarm/cows/:id/forks

# Get COW revenue
GET /api/marketplace/arbfarm/cows/:id/revenue

# Get wallet earnings
GET /api/marketplace/arbfarm/earnings/:wallet_address

# Get ArbFarm stats
GET /api/marketplace/arbfarm/stats
```

### Trades

```bash
# Get trade history
GET /api/arb/trades

# Get trade details
GET /api/arb/trades/:id

# Get P&L statistics
GET /api/arb/trades/stats
GET /api/arb/trades/stats?period=7d
```

## Health Checks

```bash
curl http://localhost:3000/health   # Erebus
curl http://localhost:9003/health   # Agents
curl http://localhost:9004/health   # Engrams
curl http://localhost:8001/health   # Protocols
curl http://localhost:9007/health   # ArbFarm
```

## Related

- [Erebus Router](../services/erebus.md)
- [Wallet Integration](./wallet.md)
