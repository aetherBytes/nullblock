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

Model Context Protocol support for tool integration.

### Available Tools

Tools exposed via MCP for external agents:

- **Engram tools**: create, get, search, update
- **Task tools**: create, list, get status
- **Agent tools**: chat, query capabilities

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
