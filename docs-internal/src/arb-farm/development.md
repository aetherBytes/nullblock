# ArbFarm Development Guide

This guide covers how to develop and extend the ArbFarm service.

## Prerequisites

- Rust 1.75+
- PostgreSQL 15+
- Helius API key (for RPC)
- OpenRouter API key (for LLM consensus)

## Quick Start

```bash
cd svc/arb-farm
cp .env.example .env.dev

# Edit .env.dev with your API keys

cargo run
```

The service starts on port 9007.

## Project Structure

```
src/
├── main.rs          # Router setup, service entrypoint
├── config.rs        # Environment configuration
├── error.rs         # Error types with HTTP mapping
├── server.rs        # AppState, database pool, event bus
├── handlers/        # HTTP route handlers
├── models/          # Data structures
├── events/          # Event bus (pub/sub)
├── venues/          # MEV venue implementations
├── research/        # Research/DD subsystem
├── threat/          # Threat detection
├── consensus/       # Multi-LLM consensus
├── execution/       # Trade execution
└── mcp/             # MCP tool definitions
```

## Adding a New Handler

1. Create handler file in `src/handlers/`:

```rust
// src/handlers/my_feature.rs
use axum::{extract::State, Json};
use crate::{error::AppResult, server::AppState};

pub async fn get_something(
    State(state): State<AppState>,
) -> AppResult<Json<serde_json::Value>> {
    // Implementation
    Ok(Json(serde_json::json!({"status": "ok"})))
}
```

2. Export from `src/handlers/mod.rs`:

```rust
pub mod my_feature;
```

3. Add route in `src/main.rs`:

```rust
.route("/my-feature", get(my_feature::get_something))
```

## Adding a New Venue

Venues implement the `MevVenue` trait:

```rust
// src/venues/my_venue.rs
use async_trait::async_trait;
use crate::venues::traits::{MevVenue, VenueType};
use crate::models::{Edge, Signal};
use crate::error::AppResult;

pub struct MyVenue {
    api_url: String,
    client: reqwest::Client,
}

#[async_trait]
impl MevVenue for MyVenue {
    fn venue_type(&self) -> VenueType {
        VenueType::Custom("my_venue".to_string())
    }

    async fn scan(&self) -> AppResult<Vec<Signal>> {
        // Scan for opportunities
        Ok(vec![])
    }

    async fn get_quote(&self, token: &str, amount: u64, is_buy: bool) -> AppResult<u64> {
        // Get price quote
        Ok(0)
    }
}
```

## Adding MCP Tools

1. Define tool in `src/mcp/tools.rs`:

```rust
fn get_my_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "my_tool".to_string(),
            description: "Does something useful".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "param1": {"type": "string", "description": "First param"}
                },
                "required": ["param1"]
            }),
        }
    ]
}
```

2. Add to `get_all_tools()`:

```rust
pub fn get_all_tools() -> Vec<McpTool> {
    let mut tools = vec![];
    tools.extend(get_scanner_tools());
    tools.extend(get_my_tools()); // Add here
    tools
}
```

## Working with the Event Bus

### Publishing Events

```rust
use crate::events::{ArbEvent, EventSource, AgentType};

// In a handler or agent
let event = ArbEvent::new(
    "arb.edge.detected",
    EventSource::Agent(AgentType::Scanner),
    serde_json::json!({
        "edge_id": edge.id,
        "edge_type": edge.edge_type,
        "estimated_profit": edge.estimated_profit_lamports
    }),
);

state.event_bus.publish(event).await;
```

### Subscribing to Events

```rust
// Subscribe to specific topics
let mut rx = state.event_bus.subscribe_to(vec![
    "arb.edge.*".to_string(),
    "arb.threat.detected".to_string(),
]);

// Process events
while let Some(event) = rx.recv().await {
    match event.topic.as_str() {
        t if t.starts_with("arb.edge.") => {
            // Handle edge events
        }
        "arb.threat.detected" => {
            // Handle threat
        }
        _ => {}
    }
}
```

## Research Module

### URL Ingestion

```rust
use crate::research::UrlIngester;

let ingester = UrlIngester::new(client);
let result = ingester.ingest("https://twitter.com/...").await?;

// Result contains:
// - content_type (Tweet, Thread, Article, etc.)
// - title, summary
// - tokens_mentioned ($BONK, $SOL, etc.)
// - addresses_found (Solana addresses)
// - numbers_extracted (prices, percentages)
```

### Strategy Extraction

```rust
use crate::research::StrategyExtractor;

let extractor = StrategyExtractor::new(openrouter_key);
let strategy = extractor.extract(&ingest_result).await?;

// Strategy contains:
// - strategy_type (DexArbitrage, BondingCurve, etc.)
// - entry_conditions
// - exit_conditions
// - risk_parameters
// - confidence (High, Medium, Low)
```

### Backtesting

```rust
use crate::research::{BacktestEngine, BacktestConfig};

let engine = BacktestEngine::new();
let config = BacktestConfig {
    initial_capital_sol: 10.0,
    start_date: "2024-01-01".to_string(),
    end_date: "2024-01-15".to_string(),
    ..Default::default()
};

let result = engine.run_backtest(&strategy, &config).await?;

// Result contains:
// - total_trades, win_rate
// - total_profit, total_return_percent
// - max_drawdown, sharpe_ratio, sortino_ratio
// - equity_curve
```

## Error Handling

Use the `AppError` enum for all errors:

```rust
use crate::error::{AppError, AppResult};

fn do_something() -> AppResult<String> {
    // Return specific errors
    Err(AppError::NotFound("Item not found".to_string()))?;
    Err(AppError::BadRequest("Invalid parameter".to_string()))?;
    Err(AppError::Configuration("Missing API key".to_string()))?;

    Ok("success".to_string())
}
```

Error types and HTTP codes:

| Error | HTTP Code | Use Case |
|-------|-----------|----------|
| `BadRequest` | 400 | Invalid input |
| `Unauthorized` | 401 | Auth required |
| `NotFound` | 404 | Resource missing |
| `Validation` | 422 | Invalid data |
| `RateLimited` | 429 | Too many requests |
| `ExternalApi` | 502 | Third-party failure |
| `ThreatDetected` | 403 | Blocked entity |
| `ConsensusFailed` | 409 | LLM disagreement |

## Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_url_ingestion

# Run with output
cargo test -- --nocapture
```

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_my_feature() {
        let result = do_something().await;
        assert!(result.is_ok());
    }
}
```

## Environment Variables

Required:

| Variable | Description |
|----------|-------------|
| `ARB_FARM_DATABASE_URL` | PostgreSQL connection |
| `HELIUS_API_KEY` | Helius RPC access |
| `OPENROUTER_API_KEY` | LLM consensus |

Optional:

| Variable | Default | Description |
|----------|---------|-------------|
| `ARB_FARM_PORT` | 9007 | Service port |
| `JUPITER_API_URL` | jup.ag | Jupiter API |
| `PUMP_FUN_API_URL` | pumpportal.fun | pump.fun API |

## Debugging

### Enable Debug Logging

```bash
RUST_LOG=debug cargo run
```

### Check Event Bus

```bash
# Stream all events
curl -N http://localhost:9007/events/stream
```

### Test MCP Tools

```bash
# List available tools
curl http://localhost:9007/mcp/tools | jq
```

## KOL Tracking

### Adding a KOL to Track

```rust
use crate::models::{KolEntity, KolEntityType, AddKolRequest};

let request = AddKolRequest {
    wallet_address: Some("7Vk3...8xNp".to_string()),
    twitter_handle: None,
    display_name: Some("Whale Watcher".to_string()),
};

// POST to /kol
```

### Enabling Copy Trading

```rust
use crate::models::EnableCopyRequest;

let config = EnableCopyRequest {
    max_position_sol: Some(0.5),
    delay_ms: Some(500),
    min_trust_score: Some(60.0),
    copy_percentage: Some(0.1),
    token_blacklist: None,
};

// POST to /kol/:id/copy/enable
```

## Threat Detection

### Checking Token Safety

```rust
use crate::threat::ThreatDetector;

let detector = ThreatDetector::default();
let score = detector.check_token("TokenMint...").await?;

if score.overall_score >= 0.7 {
    // High risk - avoid trading
    println!("DANGER: {}", score.recommendation);
}
```

### Reporting a Threat

```rust
use crate::models::{ThreatCategory, ThreatEntityType};

detector.block_entity(
    ThreatEntityType::Token,
    "ScamTokenMint...".to_string(),
    ThreatCategory::RugPull,
    "Creator dumped 80% of supply".to_string(),
    "user_report".to_string(),
);
```

### Watching a Creator Wallet

```rust
use crate::models::WatchedWallet;

let watched = WatchedWallet::new(
    "CreatorWallet...".to_string(),
    "Monitoring for dumps".to_string(),
);
detector.add_watched_wallet(watched);
```

## Related

- [Service Architecture](./service.md)
- [API Reference](./api.md)
- [Research Module](./research.md)
- [Bonding Curves](./curves.md)
- [KOL Tracking](./kol.md)
- [Threat Detection](./threat.md)
- [Event Bus](./events.md)
