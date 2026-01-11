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
| `delete_engram` | Delete engram |
| `search_engrams` | Search engrams |

### Hecate Tools Endpoint

```bash
GET /hecate/tools    # Returns MCP tools via Hecate agent
```

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

## Health Checks

```bash
curl http://localhost:3000/health   # Erebus
curl http://localhost:9003/health   # Agents
curl http://localhost:9004/health   # Engrams
curl http://localhost:8001/health   # Protocols
```

## Related

- [Erebus Router](../services/erebus.md)
- [Wallet Integration](./wallet.md)
