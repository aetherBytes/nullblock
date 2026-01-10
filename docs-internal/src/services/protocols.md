# Protocols Service

**The protocol bridge** - A2A and MCP protocol implementations.

## Overview

| Property | Value |
|----------|-------|
| **Port** | 8001 |
| **Location** | `/svc/nullblock-protocols/` |
| **Role** | Protocol server, agent interoperability |

## A2A Protocol

NullBlock implements [A2A Protocol v0.3.0](https://a2a-protocol.org/latest/specification/).

### JSON-RPC Methods

```
POST /a2a/jsonrpc
```

| Method | Description |
|--------|-------------|
| `message/send` | Send message to agent |
| `message/stream` | Stream message (SSE) |
| `tasks/get` | Get task by ID |
| `tasks/list` | List tasks |
| `tasks/cancel` | Cancel task |
| `tasks/resubscribe` | Resume task stream |

### REST Endpoints

```bash
GET  /a2a/v1/card           # Agent Card
POST /a2a/v1/messages       # Send message
GET  /a2a/v1/tasks/:id      # Get task
GET  /a2a/v1/tasks          # List tasks
```

### Agent Card

```json
{
  "name": "NullBlock Agent Hub",
  "description": "Multi-agent orchestration platform",
  "version": "0.3.0",
  "capabilities": {
    "tasks": true,
    "messages": true,
    "streaming": false
  }
}
```

## MCP Protocol

Model Context Protocol (MCP) 2025-11-25 implementation for AI tool integration.

**Specification**: [MCP 2025-11-25](https://modelcontextprotocol.io/specification/2025-11-25)

### Endpoint

```bash
POST /mcp/jsonrpc   # Main JSON-RPC 2.0 endpoint
GET  /mcp/tools     # Convenience endpoint for tools/list
GET  /mcp/resources # Convenience endpoint for resources/list
GET  /mcp/prompts   # Convenience endpoint for prompts/list
```

### Supported Methods

| Method | Description |
|--------|-------------|
| `initialize` | Initialize MCP session with capabilities |
| `initialized` | Client notification after init complete |
| `tools/list` | List available tools (9 tools) |
| `tools/call` | Execute a tool |
| `resources/list` | List available resources (2 resources) |
| `resources/read` | Read resource by URI |
| `prompts/list` | List available prompts (2 prompts) |
| `prompts/get` | Get prompt with arguments |
| `ping` | Health check |

### Available Tools (9)

| Tool | Description |
|------|-------------|
| `send_agent_message` | Send message to a NullBlock agent |
| `create_task` | Create a new task |
| `get_task_status` | Get task status by ID |
| `list_engrams` | List engrams for a wallet |
| `get_engram` | Get engram by ID |
| `create_engram` | Create a new engram |
| `update_engram` | Update an existing engram |
| `delete_engram` | Delete an engram |
| `search_engrams` | Search engrams by query |

### Available Resources (2)

| URI | Description |
|-----|-------------|
| `agent://hecate` | HECATE vessel AI agent |
| `agent://siren` | Siren marketing agent |

### Available Prompts (2)

| Prompt | Description |
|--------|-------------|
| `agent_chat` | Chat with a NullBlock agent |
| `task_template` | Create task from template |

### Server Capabilities

```json
{
  "tools": { "listChanged": false },
  "resources": { "subscribe": false, "listChanged": false },
  "prompts": { "listChanged": false }
}
```

### MCP Client (Hecate)

Hecate implements proper MCP client protocol:
- Sends `initialize` with protocol version and client info
- Sends `initialized` notification after handshake
- Caches tools with 5-minute TTL
- Can execute tools via `tools/call`

## Implementation Status

### Completed

- Task schema aligned with A2A spec
- Agent Card with full compliance
- JSON-RPC 2.0 endpoints
- HTTP REST endpoints
- Task handlers proxied to Agents service

### In Progress

- Server-Sent Events (SSE) streaming
- Push notifications (webhooks)

### Not Implemented

- Authentication middleware
- Agent Card signatures (JWS)

## Configuration

```bash
PROTOCOLS_PORT=8001
AGENTS_SERVICE_URL=http://localhost:9003
```

## Related

- [API Reference](../reference/api.md)
- [Agents Service](./agents.md)
