# NullBlock Protocols Service

Multi-protocol server supporting A2A (Agent-to-Agent) and MCP (Model Context Protocol) for the NullBlock agentic platform.

## Overview

The Protocols service acts as a protocol gateway, exposing standardized APIs that route to internal NullBlock services. It provides:

- **A2A Protocol v0.3.0** compliance for agent-to-agent communication
- **MCP Protocol 2025-11-25** support for model context management
- Unified protocol abstraction layer
- Service-to-service HTTP integration
- JSON-RPC 2.0 endpoints for both protocols

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
- A2A JSON-RPC 2.0 request handling
- Agent Card endpoint
- Health monitoring
- CORS support
- **MCP protocol support (Base Protocol + Server Features)**
  - ✅ MCP JSON-RPC 2.0 endpoint
  - ✅ Schema-compliant types (matches official TypeScript schema)
  - ✅ Resources (agent discovery and reading)
  - ✅ Tools (3 tools: agent messaging, task management)
  - ✅ Prompts (2 prompts: agent chat, task templates)
  - ✅ Lifecycle Management (initialize/initialized/ping)
  - ✅ ContentBlock unified content types
  - ✅ Annotations and metadata support
- **Authentication & Authorization**
  - ✅ OAuth 2.1-style Bearer token authentication
  - ✅ API key authentication
  - ✅ Service-to-service authentication
  - ✅ Configurable auth requirements (per-protocol)
  - ✅ Comprehensive logging

### 🔄 In Progress
- Server-Sent Events (SSE) for streaming
- Message handling integration
- Push notification webhooks
- MCP Client Features (sampling, roots)
- MCP Authorization framework

### ❌ Not Implemented (Optional MCP Features)
- Pagination (cursor-based)
- Resource subscriptions (subscribe/unsubscribe)
- List change notifications (tools/prompts/resources)
- Logging feature
- Completions feature (argument autocompletion)
- Sampling feature (client-side)
- Roots feature (client-side)
- Rate limiting
- Caching layer
- WebSocket transport

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
│   │   └── mcp/
│   │       ├── mod.rs          # MCP module root
│   │       ├── routes.rs       # Route definitions
│   │       ├── handlers.rs     # Request handlers
│   │       ├── jsonrpc.rs      # JSON-RPC support
│   │       └── types.rs        # MCP type definitions
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

## MCP Protocol Implementation

### Overview

The Model Context Protocol (MCP) enables standardized communication between AI applications and context providers. All MCP messages follow the [JSON-RPC 2.0](https://www.jsonrpc.org/specification) specification.

### Specification & SDK

- **Official MCP Specification**: [MCP Base Protocol 2025-11-25](https://modelcontextprotocol.io/specification/2025-11-25/basic)
- **Official Anthropic Rust SDK**: [modelcontextprotocol/rust-sdk](https://github.com/modelcontextprotocol/rust-sdk) - `rmcp` crate v0.8.0
  - Available as optional dependency with `official-mcp-sdk` feature flag
  - Core protocol implementation with tokio async runtime
  - Procedural macros for tool generation via `rmcp-macros`

### Implementation Approach

NullBlock's MCP implementation uses a **custom JSON-RPC 2.0 gateway** approach rather than the official SDK:

**Why Custom Implementation:**
- Integrates seamlessly with existing Axum HTTP router
- Provides HTTP/JSON-RPC gateway to internal services
- Matches our A2A protocol architecture pattern
- Allows fine-grained control over service proxying
- No additional async runtime complexity

**Official SDK Available For:**
- Future migration if needed
- Reference implementation for protocol compliance
- Alternative transport mechanisms (stdio, WebSocket)
- Native MCP server functionality

The `rmcp` SDK is included as an optional dependency for future use or reference. For detailed information about the SDK and potential migration paths, see [RMCP_SDK_INTEGRATION.md](./RMCP_SDK_INTEGRATION.md).

### Key Components

**Required (MUST implement):**
- **Base Protocol**: Core JSON-RPC message types (requests, responses, notifications)
- **Lifecycle Management**: Connection initialization, capability negotiation, session control

**Optional (MAY implement):**
- **Authorization**: Authentication and authorization framework for HTTP-based transports
- **Server Features**: Resources, prompts, and tools exposed by servers
- **Client Features**: Sampling and root directory lists provided by clients
- **Utilities**: Cross-cutting concerns like logging and argument completion

### Endpoints

**JSON-RPC 2.0:**
- `POST /mcp/jsonrpc` - MCP JSON-RPC endpoint for all methods

### Supported Methods

| Method | Status | Description |
|--------|--------|-------------|
| `initialize` | ✅ Implemented | Initialize MCP session with capability negotiation |
| `initialized` | ✅ Implemented | Notification that client initialization is complete |
| `resources/list` | ✅ Implemented | List available resources (agents) |
| `resources/read` | ✅ Implemented | Read resource content by URI |
| `tools/list` | ✅ Implemented | List available tools |
| `tools/call` | ✅ Implemented | Execute a tool |
| `prompts/list` | ✅ Implemented | List available prompts |
| `prompts/get` | ✅ Implemented | Get a prompt by name |
| `ping` | ✅ Implemented | Health check ping |

### Available Tools

1. **send_agent_message** - Send a message to a NullBlock agent
   - Parameters: `agent_name` (string), `message` (string)
   
2. **create_task** - Create a new task in the task system
   - Parameters: `name` (string), `description` (string), `priority` (optional: low/medium/high/critical)
   
3. **get_task_status** - Get the status of a task
   - Parameters: `task_id` (string UUID)

### Available Prompts

1. **agent_chat** - Chat with a NullBlock agent
   - Arguments: `agent` (required), `context` (optional)
   
2. **task_template** - Create a task from a template
   - Arguments: `type` (required: analysis/research/development)

### Resources

Resources are exposed as URIs in the format `agent://{agent_name}`. The MCP server automatically discovers available agents from the Agents service and exposes them as readable resources.

### Example Usage

**Initialize session:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "protocolVersion": "2025-11-25",
    "capabilities": {},
    "clientInfo": {
      "name": "my-client",
      "version": "1.0.0"
    }
  }
}
```

**List available tools:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/list"
}
```

**Call a tool:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "send_agent_message",
    "arguments": {
      "agent_name": "hecate",
      "message": "Hello, how can you help me?"
    }
  }
}
```

**List resources:**
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "resources/list"
}
```

## Authentication & Authorization

The protocols service implements a comprehensive authentication system supporting:

- **OAuth 2.1-style Bearer tokens** (MCP specification compliant)
- **API Key authentication** for trusted clients
- **Service-to-service authentication** between NullBlock services

**Configuration**:
- `REQUIRE_AUTH` - Global auth requirement
- `REQUIRE_MCP_AUTH` - MCP endpoint auth
- `REQUIRE_A2A_AUTH` - A2A endpoint auth
- `SERVICE_SECRET` - Shared secret for service-to-service calls
- `API_KEYS` - Comma-separated API keys
- `ENABLE_BEARER_TOKENS` - Enable OAuth-style bearer tokens

**📚 See [AUTHENTICATION.md](./AUTHENTICATION.md) for complete documentation including:**
- All authentication mechanisms
- Configuration examples
- Security best practices
- Testing guide
- Troubleshooting

## References

### MCP Resources
- [MCP Protocol Specification 2025-11-25](https://modelcontextprotocol.io/specification/2025-11-25/basic)
- [Official TypeScript Schema](https://github.com/modelcontextprotocol/modelcontextprotocol/blob/main/schema/2025-11-25/schema.ts) - Source of truth for all types
- [Official Anthropic Rust SDK (rmcp)](https://github.com/modelcontextprotocol/rust-sdk)
- [rmcp crate documentation](https://docs.rs/rmcp/latest/rmcp/)
- [JSON-RPC 2.0 Specification](https://www.jsonrpc.org/specification)
- [OAuth 2.1 Authorization](https://modelcontextprotocol.io/specification/2025-11-25/authorization)

### A2A Resources
- [A2A Protocol Specification v0.3.0](https://a2a-protocol.org/latest/specification/)

### Framework & Architecture
- [Axum Web Framework](https://docs.rs/axum/0.7/axum/)
- [NullBlock Architecture](../../CLAUDE.md)

## Contributing

This service follows NullBlock coding standards:
- No comments unless explicitly requested
- Prefer editing existing files over creating new ones
- Follow Rust conventions and clippy lints
- Test all protocol changes end-to-end
