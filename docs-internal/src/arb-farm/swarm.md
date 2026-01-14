# Swarm Orchestration & Resilience

The Swarm Orchestration system provides monitoring, health management, and fault tolerance for the ArbFarm agent ecosystem.

## Components

### Resilience Overseer (Meta-Agent)

The Resilience Overseer is a meta-agent that monitors and manages all other agents in the swarm.

```rust
pub struct ResilienceOverseer {
    id: Uuid,
    agents: Arc<RwLock<HashMap<Uuid, AgentStatus>>>,
    config: OverseerConfig,
    is_paused: Arc<RwLock<bool>>,
    event_tx: broadcast::Sender<ArbEvent>,
}
```

**Responsibilities**:
- Monitor agent heartbeats
- Track agent health states
- Coordinate swarm-wide pause/resume
- Identify agents needing recovery
- Emit swarm health events

### Agent Health States

```rust
pub enum AgentHealth {
    Healthy,    // Operating normally
    Degraded,   // 1-2 consecutive failures
    Unhealthy,  // 3-5 consecutive failures
    Dead,       // 6+ consecutive failures
}
```

### Agent Status Tracking

Each registered agent has status tracked:

```rust
pub struct AgentStatus {
    pub agent_type: AgentType,
    pub agent_id: Uuid,
    pub health: AgentHealth,
    pub last_heartbeat: Instant,
    pub consecutive_failures: u32,
    pub restart_count: u32,
    pub started_at: DateTime<Utc>,
    pub error_message: Option<String>,
}
```

### Overseer Configuration

```rust
pub struct OverseerConfig {
    pub heartbeat_interval_secs: u64,    // Default: 10
    pub heartbeat_timeout_secs: u64,     // Default: 30
    pub max_restart_attempts: u32,       // Default: 3
    pub restart_cooldown_secs: u64,      // Default: 60
    pub auto_recovery_enabled: bool,     // Default: true
}
```

## Circuit Breakers

Circuit breakers prevent cascading failures by temporarily disabling failing integrations.

### Circuit States

```rust
pub enum CircuitState {
    Closed,    // Normal operation - requests flow through
    Open,      // Failing - requests blocked
    HalfOpen,  // Testing recovery - limited requests allowed
}
```

### Circuit Breaker Configuration

```rust
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,      // Failures to open (default: 5)
    pub success_threshold: u32,      // Successes to close (default: 3)
    pub timeout_duration: Duration,  // Time before half-open (default: 30s)
    pub half_open_max_calls: u32,    // Test calls in half-open (default: 3)
}
```

### Usage Pattern

```rust
let breaker = CircuitBreaker::new("jupiter_api", config);

// Execute with circuit breaker protection
match breaker.execute(|| async {
    jupiter_client.get_quote(params).await
}).await {
    Ok(quote) => process_quote(quote),
    Err(CircuitBreakerError::Open) => {
        // Circuit is open, use fallback
        use_cached_route()
    }
    Err(CircuitBreakerError::Inner(e)) => {
        // Underlying error, circuit recorded failure
        handle_api_error(e)
    }
}
```

### Circuit Breaker Registry

Centralized management of all circuit breakers:

```rust
let registry = CircuitBreakerRegistry::new(default_config);

// Get or create breaker by name
let breaker = registry.get_or_create("helius_rpc").await;

// Reset specific breaker
if let Some(breaker) = registry.get("jupiter_api").await {
    breaker.reset().await;
}

// Reset all breakers (emergency recovery)
registry.reset_all().await;

// Get all states for monitoring
let states = registry.get_all_states().await;
```

## API Endpoints

### Swarm Status

```bash
# Full swarm status with agents and circuit breakers
GET /swarm/status

# Response
{
  "health": {
    "total_agents": 5,
    "healthy_agents": 4,
    "degraded_agents": 1,
    "unhealthy_agents": 0,
    "dead_agents": 0,
    "overall_health": "Degraded",
    "is_paused": false
  },
  "agents": [...],
  "circuit_breakers": [...]
}
```

### Swarm Health (Quick Check)

```bash
GET /swarm/health

# Response
{
  "total_agents": 5,
  "healthy_agents": 5,
  "overall_health": "Healthy",
  "is_paused": false
}
```

### Agent Management

```bash
# List all agents
GET /swarm/agents

# Get specific agent
GET /swarm/agents/:id

# Record heartbeat
POST /swarm/heartbeat
{
  "agent_id": "uuid-here"
}

# Report failure
POST /swarm/failure
{
  "agent_id": "uuid-here",
  "error": "Connection timeout to Jupiter API"
}
```

### Swarm Control

```bash
# Pause all trading activity
POST /swarm/pause

# Resume trading activity
POST /swarm/resume
```

### Circuit Breaker Management

```bash
# List all circuit breakers
GET /swarm/circuit-breakers

# Response
[
  { "name": "jupiter_api", "state": "Closed" },
  { "name": "helius_rpc", "state": "HalfOpen" },
  { "name": "birdeye_api", "state": "Open" }
]

# Reset specific breaker
POST /swarm/circuit-breakers/:name/reset

# Reset all breakers
POST /swarm/circuit-breakers/reset-all
```

## Event Topics

The overseer emits events on these topics:

| Topic | Description |
|-------|-------------|
| `arb.swarm.agent.started` | Agent registered with overseer |
| `arb.swarm.agent.stopped` | Agent unregistered |
| `arb.swarm.agent.failed` | Agent health degraded |
| `arb.swarm.agent.recovered` | Agent recovered/restarted |
| `arb.swarm.paused` | Swarm-wide pause activated |
| `arb.swarm.resumed` | Swarm-wide operations resumed |

## Health Check Flow

```
Agent Running
     │
     ├─── Every N seconds ───▶ Send Heartbeat
     │                              │
     │                              ▼
     │                        Overseer Records
     │                        Last Heartbeat
     │
     ▼
 Operation Fails
     │
     ├─── Report Failure ───▶ Overseer Records
     │                        Consecutive Failures
     │                              │
     │                              ▼
     │                        Update Health State
     │                        (Degraded/Unhealthy/Dead)
     │                              │
     │                              ▼
     │                        Emit AGENT_FAILED Event
     │
     ▼
 Heartbeat Timeout
     │
     └─── Overseer Detects ───▶ Mark as Stale
                                    │
                                    ▼
                              Auto-Recovery
                              (if enabled)
```

## Auto-Recovery

When `auto_recovery_enabled` is true, the overseer identifies agents needing restart:

```rust
// Get agents that need restart
let agents_to_restart = overseer.get_agents_needing_restart().await;

for (agent_id, agent_type) in agents_to_restart {
    // Restart logic - respects max_restart_attempts
    restart_agent(agent_type).await;
    overseer.record_agent_recovery(agent_id).await;
}
```

**Restart Conditions**:
- Agent health is `Dead` (6+ consecutive failures)
- Restart count < `max_restart_attempts`
- Respects `restart_cooldown_secs` between attempts
