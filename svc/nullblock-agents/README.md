# NullBlock Agents Service

**Multi-agent orchestration platform with persistent task management and LLM integration**

## Overview

The NullBlock Agents service is the core agent orchestration platform that manages specialized AI agents, handles task lifecycles, and provides seamless integration with LLM services. It serves as the central hub for agent-based automation within the NullBlock ecosystem.

### Key Features

- **🤖 Hecate Agent**: Primary conversational interface and orchestration engine
- **📋 Task Management**: Full CRUD operations with PostgreSQL persistence
- **⚡ Event Streaming**: Kafka-based task lifecycle events
- **🧠 Multi-LLM Support**: OpenRouter, OpenAI, Anthropic, Groq integration
- **🔄 Agent Orchestration**: Coordinate multiple specialized agents
- **📊 Performance Metrics**: Real-time monitoring and health checks

## Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Frontend      │    │     Erebus       │    │  Agents Service │
│   (Hecate UI)   │◄──►│   Router API     │◄──►│   Port 9003     │
│   Port 5173     │    │   Port 3000      │    │                 │
└─────────────────┘    └──────────────────┘    └─────────────────┘
                              │                         │
                              │                         ▼
                              │                ┌─────────────────┐
                              │                │   PostgreSQL    │
                              │                │  agents:5441    │
                              │                └─────────────────┘
                              │                         │
                              │                         ▼
                              │                ┌─────────────────┐
                              │                │     Kafka       │
                              │                │  localhost:9092 │
                              └────────────────┤                 │
                                               └─────────────────┘
```

## Quick Start

### Prerequisites

```bash
# Required services
docker-compose up postgres-agents kafka zookeeper -d

# Required environment variables
export DATABASE_URL="postgresql://postgres:$POSTGRES_PASSWORD@localhost:5441/agents"
export KAFKA_BOOTSTRAP_SERVERS="localhost:9092"
export OPENROUTER_API_KEY="your_openrouter_key"  # Optional but recommended
```

### Development Setup

```bash
# 1. Start the agents service
cd svc/nullblock-agents
cargo run

# 2. Verify health status
curl http://localhost:9003/health

# 3. Test task operations
curl http://localhost:9003/tasks
```

### Production Deployment

```bash
# Build optimized binary
cargo build --release

# Run with production config
export RUST_LOG=info
./target/release/nullblock-agents
```

## API Endpoints

### Core Endpoints

- **Health Check**: `GET /health` - Service health and component status
- **Tasks**: `GET/POST /tasks` - Task management operations
- **Agents**: `GET /agents/status` - Agent status and capabilities
- **Chat**: `POST /chat` - Direct Hecate agent interaction

### Task Management

```bash
# List tasks
GET /tasks?status=created&limit=10

# Create task
POST /tasks
{
  "name": "Example Task",
  "description": "Task description",
  "task_type": "system",
  "category": "user_assigned",
  "priority": "medium"
}

# Task lifecycle operations
POST /tasks/{id}/start
POST /tasks/{id}/pause
POST /tasks/{id}/resume
POST /tasks/{id}/cancel
DELETE /tasks/{id}
```

### Agent Interaction

```bash
# Chat with Hecate
POST /chat
{
  "message": "Hello Hecate!",
  "user_context": {
    "wallet_address": "0x1234...",
    "session_time": "5m"
  }
}

# Get agent model status
GET /agents/models
```

## Database Schema

### Core Tables

- **`tasks`**: Task management with full lifecycle support
- **`agents`**: Agent registry and health tracking
- **`user_references`**: Synced user data from Erebus service

### Key Relationships

- `tasks.assigned_agent_id` → `agents.id`
- `tasks.user_id` → `user_references.id`

## Configuration

### Environment Variables

| Variable | Description | Default | Required |
|----------|-------------|---------|----------|
| `DATABASE_URL` | PostgreSQL connection string | - | ✅ |
| `KAFKA_BOOTSTRAP_SERVERS` | Kafka cluster endpoints | `localhost:9092` | ❌ |
| `OPENROUTER_API_KEY` | OpenRouter API key for LLM access | - | ❌ |
| `RUST_LOG` | Logging level | `debug` | ❌ |
| `SERVER_HOST` | Server bind address | `0.0.0.0` | ❌ |
| `SERVER_PORT` | Server port | `9003` | ❌ |

### Agent Configuration

Hecate agent supports multiple personalities and LLM providers:

```rust
// Available personalities
- helpful_cyberpunk (default)
- technical_expert
- concise_assistant

// Supported LLM providers
- OpenRouter (primary)
- OpenAI
- Anthropic
- Groq
```

## Data Flow

### Task Lifecycle

1. **Creation**: Task created via API or frontend
2. **Assignment**: Auto-assigned to appropriate agent (Hecate for user tasks)
3. **Processing**: Agent processes task using LLM capabilities
4. **Completion**: Results stored in task outcome/logs
5. **Events**: Kafka events published for each state change

### Agent Response Flow

1. **Task Pickup**: Agent receives assigned task
2. **LLM Processing**: Task content processed using selected model
3. **Response Storage**: Agent response stored in task outcome
4. **Status Update**: Task marked as completed/failed
5. **Event Publishing**: Lifecycle event sent to Kafka

## Development Guide

### Adding New Agents

1. **Create Agent Struct**:
```rust
pub struct MyAgent {
    pub name: String,
    pub capabilities: Vec<String>,
    // ... agent-specific fields
}
```

2. **Implement Agent Trait**:
```rust
impl Agent for MyAgent {
    async fn process_task(&mut self, task: &Task) -> AppResult<TaskOutcome> {
        // Implementation
    }
}
```

3. **Register in Database**:
```rust
// Add agent entry to agents table
let agent_entity = AgentEntity {
    name: "my-agent".to_string(),
    agent_type: "specialized".to_string(),
    status: "active".to_string(),
    // ... other fields
};
```

### Task Processing Pattern

```rust
// 1. Receive task
let task = task_repo.get_by_id(&task_id).await?;

// 2. Process with agent
let outcome = agent.process_task(&task).await?;

// 3. Update task status
task_repo.update_outcome(&task_id, outcome).await?;

// 4. Publish event
kafka_producer.publish_task_event(
    TaskLifecycleEvent::task_completed(task.id, outcome)
).await?;
```

## Troubleshooting

### Common Issues

**Database Connection Errors**
```bash
# Check PostgreSQL status
docker logs nullblock-postgres-agents
pg_isready -h localhost -p 5441 -U postgres
```

**Enum Serialization Errors**
```bash
# Fix data inconsistencies
PGPASSWORD="$POSTGRES_PASSWORD" psql -h localhost -p 5441 -U postgres -d agents \
  -c "UPDATE tasks SET category = 'user_assigned' WHERE category = 'userassigned';"
```

**Kafka Connection Issues**
```bash
# Verify Kafka is running
docker logs nullblock-kafka
```

### Debug Mode

```bash
# Enable detailed logging
export RUST_LOG=debug
cargo run

# Monitor logs
tail -f logs/hecate-server.log
```

## Creating Additional Agent Services

### Service Template

When creating new agent services (e.g., `nullblock-trading-agents`), follow this pattern:

1. **Repository Structure**:
```
svc/nullblock-{domain}-agents/
├── src/
│   ├── agents/           # Specialized agents
│   ├── handlers/         # HTTP endpoints
│   ├── database/         # Data layer
│   └── main.rs
├── migrations/           # Database schema
├── Cargo.toml           # Dependencies
└── README.md            # Service documentation
```

2. **Core Dependencies** (Cargo.toml):
```toml
[dependencies]
axum = "0.7"
sqlx = { version = "0.8", features = ["postgres", "uuid", "chrono"] }
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
tracing = "0.1"
```

3. **Database Setup**:
- Use dedicated PostgreSQL database: `postgres-{domain}-agents`
- Port allocation: 5441 (agents), 5442 (mcp), 5443 (trading), etc.
- Follow same migration pattern as this service

4. **Service Architecture**:
- Port allocation: 9003 (agents), 9004 (trading), 9005 (social), etc.
- Follow same handler/database/agent structure
- Implement health checks and logging patterns

5. **Integration Points**:
- Route through Erebus (GOLDEN RULE)
- Use same Kafka cluster for events
- Follow consistent API patterns
- Implement agent registration in shared agents table

## Monitoring & Operations

### Health Checks

- **Service Health**: `/health` endpoint with component status
- **Database**: Connection pool and query performance
- **LLM Services**: Model availability and API status
- **Agent Status**: Individual agent health and capabilities

### Performance Metrics

- **Request Latency**: Chat and task operation timing
- **Token Usage**: LLM consumption tracking
- **Task Throughput**: Completion rates and queue depths
- **Error Rates**: Failed requests and agent errors

### Logging

- **Structured Logs**: JSON format with request IDs
- **Chat Sessions**: Individual session files in `logs/chats/`
- **Cyberpunk Aesthetic**: Themed log messages and status indicators
- **Performance Data**: Latency, costs, and model usage

---

For questions or contributions, see the main [NullBlock documentation](../../CLAUDE.md) or contact the development team.