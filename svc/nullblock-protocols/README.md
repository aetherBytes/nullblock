# NullBlock Protocols Service

Multi-protocol server supporting A2A (Agent-to-Agent) and MCP (Model Context Protocol) for the NullBlock agentic platform.

## Overview

The Protocols service acts as a protocol gateway, exposing standardized APIs that route to internal NullBlock services. It provides:

- **A2A Protocol v0.3.0** compliance for agent-to-agent communication
- **MCP Protocol** support for model context management (planned)
- Unified protocol abstraction layer
- Service-to-service HTTP integration

## Architecture

```
External Clients
       ↓
┌─────────────────────────┐
│  Protocols Service      │
│  (Port 8001)           │
│                         │
│  ┌─────────┬─────────┐ │
│  │   A2A   │   MCP   │ │
│  └────┬────┴────┬────┘ │
└───────┼─────────┼───────┘
        ↓         ↓
  ┌──────────────────┐
  │  Agents Service  │
  │  (Port 9003)     │
  └──────────────────┘
```

## A2A Protocol Implementation

### Endpoints

**REST/HTTP+JSON:**
- `GET /v1/card` - Agent Card (public capabilities)
- `GET /.well-known/agent-card.json` - Agent Card (well-known location)
- `POST /v1/messages` - Send message
- `POST /v1/messages/stream` - Send streaming message (SSE)
- `GET /v1/tasks` - List tasks
- `GET /v1/tasks/:id` - Get task by ID
- `POST /v1/tasks/:id/cancel` - Cancel task
- `POST /v1/tasks/:id/subscribe` - Subscribe to task updates (SSE)

**JSON-RPC 2.0:**
- `POST /a2a/jsonrpc` - All A2A methods via JSON-RPC

### Supported Methods

| Method | Status | Description |
|--------|--------|-------------|
| `message/send` | ✅ Stub | Send message to agent |
| `message/stream` | ✅ Stub | Stream message responses |
| `tasks/get` | ✅ Implemented | Retrieve task by ID |
| `tasks/list` | ✅ Implemented | List tasks with filters |
| `tasks/cancel` | ✅ Implemented | Cancel running task |
| `tasks/resubscribe` | ✅ Stub | Resume task updates stream |
| `tasks/pushNotificationConfig/*` | ✅ Stub | Webhook management |
| `agent/getAuthenticatedExtendedCard` | ✅ Stub | Extended agent card |

### A2A Task Schema

Tasks follow [A2A Protocol v0.3.0](https://a2a-protocol.org/latest/specification/) specification:

```json
{
  "id": "uuid",
  "contextId": "uuid",
  "kind": "task",
  "status": {
    "state": "submitted|working|completed|...",
    "message": "Optional status message",
    "timestamp": "ISO8601"
  },
  "history": [
    {
      "messageId": "uuid",
      "role": "user|assistant",
      "parts": [
        {"type": "text", "text": "message content"}
      ],
      "taskId": "uuid",
      "contextId": "uuid",
      "kind": "message",
      "timestamp": "ISO8601"
    }
  ],
  "artifacts": [
    {
      "artifactId": "uuid",
      "parts": [
        {"type": "text", "text": "artifact content"}
      ],
      "metadata": {}
    }
  ]
}
```

### Task States

- `submitted` - Task created, not started
- `working` - Task in progress
- `input-required` - Waiting for user input
- `completed` - Task finished successfully
- `canceled` - Task canceled by user/system
- `failed` - Task failed with error
- `rejected` - Task rejected (validation/permissions)
- `auth-required` - Authentication needed
- `unknown` - Unknown/unspecified state

## Quick Start

### Prerequisites

- Rust 1.75+
- Running Agents service (port 9003)
- PostgreSQL database (optional, for direct DB access)

### Environment Variables

```bash
# Required
AGENTS_SERVICE_URL=http://localhost:9003  # Agents service endpoint

# Optional
PORT=8001                                  # Service port (default: 8001)
PROTOCOLS_SERVICE_URL=http://localhost:8001 # Own URL for logging
RUST_LOG=info                              # Log level
```

### Running

```bash
# Development
cd svc/nullblock-protocols
cargo run

# Production
cargo build --release
./target/release/nullblock-protocols
```

### Testing

```bash
# Health check
curl http://localhost:8001/health

# Get Agent Card
curl http://localhost:8001/v1/card

# Create task via Agents service
curl -X POST http://localhost:9003/tasks \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Test Task",
    "description": "A2A test",
    "task_type": "system",
    "category": "user_assigned",
    "priority": "medium"
  }'

# Retrieve via A2A protocol
curl http://localhost:8001/v1/tasks/{task_id}

# List tasks
curl -X POST http://localhost:8001/a2a/jsonrpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "method": "tasks/list",
    "params": {},
    "id": 1
  }'
```

## Implementation Status

### ✅ Completed
- Axum 0.7 router with stateful handlers
- HTTP client integration with Agents service
- A2A task GET/LIST/CANCEL endpoints
- JSON-RPC 2.0 request handling
- Agent Card endpoint
- Health monitoring
- CORS support
- Authentication middleware (pass-through)

### 🔄 In Progress
- Server-Sent Events (SSE) for streaming
- Message handling integration
- Push notification webhooks

### ❌ Not Implemented
- MCP protocol support
- Authentication/authorization
- Rate limiting
- Caching layer
- WebSocket support (if needed)

## Development

### Project Structure

```
svc/nullblock-protocols/
├── src/
│   ├── protocols/
│   │   ├── a2a/
│   │   │   ├── mod.rs          # A2A module root
│   │   │   ├── routes.rs       # Route definitions
│   │   │   ├── handlers/       # Request handlers
│   │   │   │   ├── card.rs     # Agent Card
│   │   │   │   ├── messages.rs # Message handling
│   │   │   │   ├── tasks.rs    # Task operations
│   │   │   │   └── push.rs     # Push notifications
│   │   │   ├── jsonrpc.rs      # JSON-RPC support
│   │   │   ├── types.rs        # A2A type definitions
│   │   │   └── auth.rs         # Auth middleware
│   │   └── mcp/                # MCP protocol (future)
│   ├── server.rs               # Server setup
│   ├── health.rs               # Health checks
│   ├── error.rs                # Error types
│   └── main.rs                 # Entry point
├── Cargo.toml
└── README.md
```

### Key Design Patterns

**Router State Management (Axum 0.7):**
```rust
// Pattern: Add state BEFORE middleware, return Router<()>
pub fn create_routes(state: AppState) -> Router {
    Router::new()
        .route("/endpoint", get(handler))
        .with_state(state)           // Convert to Router<AppState>
        .layer(middleware::from_fn(...)) // Back to Router<()>
}
```

**HTTP Proxying to Services:**
```rust
pub async fn get_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>
) -> Result<Json<Task>, ProtocolError> {
    let url = format!("{}/tasks/{}", state.agents_service_url, task_id);

    let response = state.http_client.get(&url).send().await?;
    let json: serde_json::Value = response.json().await?;

    // Extract from wrapper: {"success": true, "data": {...}}
    let task = serde_json::from_value(json["data"].clone())?;
    Ok(Json(task))
}
```

### Adding New Endpoints

1. Define types in `src/protocols/a2a/types.rs`
2. Add handler in `src/protocols/a2a/handlers/*.rs`
3. Register route in `src/protocols/a2a/routes.rs`
4. Add JSON-RPC method in `src/protocols/a2a/jsonrpc.rs` (if needed)

## Troubleshooting

### Router Type Errors

If you see `expected Router<()>, found Router<AppState>`:
- Ensure `.with_state(state)` is called BEFORE `.layer()`
- Check that subrouter returns `Router` (not `Router<AppState>`)

### 404 on A2A Endpoints

- Verify Agents service is running on correct port
- Check `AGENTS_SERVICE_URL` environment variable
- Ensure no `/api/agents/` prefix in handler URLs (use `/tasks/` directly)

### Compilation Issues

```bash
# Clean build
cargo clean
cargo build

# Check dependencies
cargo tree | grep axum
```

## References

- [A2A Protocol Specification v0.3.0](https://a2a-protocol.org/latest/specification/)
- [Axum Web Framework](https://docs.rs/axum/0.7/axum/)
- [NullBlock Architecture](../../CLAUDE.md)

## Contributing

This service follows NullBlock coding standards:
- No comments unless explicitly requested
- Prefer editing existing files over creating new ones
- Follow Rust conventions and clippy lints
- Test all protocol changes end-to-end
