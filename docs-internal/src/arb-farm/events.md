# ArbFarm Event Bus

**GOLDEN RULE**: All agent-to-agent and tool communications use standardized protocols with subscribable events. This enables future agents to plug in and consume events without modifying existing code.

## Event Structure

```rust
pub struct ArbEvent {
    pub id: Uuid,
    pub event_type: String,
    pub source: EventSource,
    pub topic: String,
    pub payload: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: Option<Uuid>,
}

pub enum EventSource {
    Agent(AgentType),
    Tool(String),
    External(String),
    System,
}
```

## Event Topics

All events follow a hierarchical topic structure for pattern-based subscriptions.

### Scanner Events
```
arb.scanner.*                    # All scanner events
arb.scanner.signal.detected      # New signal detected
arb.scanner.venue.added          # New venue discovered
arb.scanner.started              # Scanner started
arb.scanner.stopped              # Scanner stopped
```

### Edge Events
```
arb.edge.*                       # All edge events
arb.edge.detected                # New opportunity found
arb.edge.approved                # Edge approved for execution
arb.edge.rejected                # Edge rejected
arb.edge.executing               # Trade in progress
arb.edge.executed                # Trade completed
arb.edge.failed                  # Trade failed
arb.edge.expired                 # Edge expired
```

### Strategy Events
```
arb.strategy.*                   # Strategy events
arb.strategy.created             # New strategy added
arb.strategy.updated             # Strategy modified
arb.strategy.triggered           # Strategy conditions met
arb.strategy.enabled             # Strategy enabled
arb.strategy.disabled            # Strategy disabled
```

### Research Events
```
arb.research.*                   # Research/DD events
arb.research.url.ingested        # URL analyzed
arb.research.strategy.discovered # New strategy found
arb.research.strategy.approved   # Strategy approved
arb.research.backtest.completed  # Backtest finished
```

### KOL Events
```
arb.kol.*                        # KOL tracking events
arb.kol.trade.detected           # KOL made a trade
arb.kol.trade.copied             # We copied the trade
arb.kol.trust.updated            # Trust score changed
arb.kol.added                    # New KOL tracked
arb.kol.removed                  # KOL removed
```

### Threat Events
```
arb.threat.*                     # Threat detection events
arb.threat.detected              # Threat identified
arb.threat.blocked               # Entity blocked
arb.threat.alert                 # Alert raised
arb.threat.whitelisted           # Entity whitelisted
```

### Engram Events
```
arb.engram.*                     # Engram events
arb.engram.created               # New engram stored
arb.engram.pattern.matched       # Pattern recognized
arb.engram.avoidance.created     # Avoidance engram created
arb.engram.strategy.optimized    # Strategy improved via engrams
```

### Swarm Events
```
arb.swarm.*                      # Swarm health events
arb.swarm.agent.started          # Agent spawned
arb.swarm.agent.stopped          # Agent stopped
arb.swarm.agent.failed           # Agent crashed
arb.swarm.agent.recovered        # Agent restarted
arb.swarm.paused                 # Swarm paused
arb.swarm.resumed                # Swarm resumed
```

## Subscribing to Events

### Pattern Matching

Topics support wildcard patterns:

```rust
// Subscribe to all edge events
event_bus.subscribe_to(vec!["arb.edge.*".to_string()])

// Subscribe to specific events
event_bus.subscribe_to(vec![
    "arb.edge.executed".to_string(),
    "arb.threat.detected".to_string(),
])
```

### Composability Example

A new agent can plug into the swarm by subscribing to relevant events:

```rust
// Content generation agent subscribes to interesting events
let subscription = event_bus.subscribe_to(vec![
    "arb.edge.executed",
    "arb.threat.detected",
    "arb.research.strategy.discovered",
]).await;

while let Some(event) = subscription.recv().await {
    match event.topic.as_str() {
        "arb.edge.executed" => {
            let trade: TradeResult = serde_json::from_value(event.payload)?;
            if trade.profit_sol > 0.1 {
                self.generate_win_tweet(trade).await;
            }
        }
        // ... handle other events
    }
}
```

## Event Persistence

All events are persisted to the database for:
- Historical analysis
- Replay from a specific point
- Debugging and auditing

```sql
SELECT * FROM arb_events 
WHERE topic LIKE 'arb.edge.%' 
ORDER BY created_at DESC 
LIMIT 100;
```

## Related

- [Service Architecture](./service.md)
- [API Reference](./api.md)
