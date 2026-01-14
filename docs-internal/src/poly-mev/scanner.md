# PolySwarm Scanner

Real-time Polymarket intelligence scanner - the first forkable COW tool for prediction market data.

## Overview

The scanner connects to Polymarket WebSocket and REST APIs, processes real-time market data with maximum sensitivity, and streams events for consumption by:
- **Sandbox viewport** (MemCache tab) - live visual display
- **Agents via MCP** - automated decision-making
- **Engram storage** - persistent context

## Dogfooding Principle

**Critical**: This scanner uses only public APIs that any NullBlock user could access.

| Rule | Implementation |
|------|----------------|
| **Public APIs Only** | Polymarket Gamma/CLOB/WebSocket are public |
| **Erebus Gateway** | All requests flow through Erebus public API routes |
| **Tool Pattern** | Scanner is a Crossroads tool - users build similar tools |
| **No Internal DB Access** | Scanner uses its own DB, accessed via public endpoints |
| **Engram Storage** | Insights stored via public `/api/engrams/*` |

## Key Decisions

| Aspect | Decision |
|--------|----------|
| **Market Tracking** | Top volume markets by default + fuzzy search/slug paste |
| **UI Location** | Crossroads tool → MemCache → Sandbox tab |
| **Signal Sensitivity** | Maximum - capture all meaningful signals |
| **Backend** | Rust for performance, cloud-ready |
| **MCP Compatible** | Full MCP server with tools for agent discovery |
| **Forkable** | Users fork, customize config, get unique MCP endpoint |

## Architecture

```
Polymarket APIs                    NullBlock Backend                    Frontend
─────────────────────────────────────────────────────────────────────────────────
                                   ┌──────────────────┐
wss://ws-subscriptions-clob.       │   poly-mev       │
polymarket.com/ws/market  ────────▶│   (port 9006)    │
                                   │                  │
https://gamma-api.polymarket.com   │  - ws_client.rs  │────────┐
  /events, /markets       ────────▶│  - gamma.rs      │        │
                                   │  - scanner.rs    │        │
https://clob.polymarket.com        │  - sse.rs        │        │
  /orderbook              ────────▶│                  │        │
                                   └────────┬─────────┘        │
                                            │                  │
                                   ┌────────▼─────────┐        │
                                   │     Erebus       │◀───────┘
                                   │   (port 3000)    │    SSE proxy
                                   │ /api/poly/*      │
                                   └────────┬─────────┘
                                            │
                                   ┌────────▼─────────┐
                                   │     Hecate       │
                                   │   (port 5173)    │
                                   │                  │
                                   │ MemCache/Sandbox │
                                   │   usePolyScanner │
                                   └──────────────────┘
```

## Signal Detection

Maximum sensitivity thresholds for capturing all meaningful market signals:

| Signal Type | Threshold | Description |
|-------------|-----------|-------------|
| **Price Change** | 2% | Any price move ≥2% triggers event |
| **Volume Spike** | 1.5x | Volume increase of 50%+ |
| **Spread Opportunity** | 2% | Bid/ask spread ≥2% |
| **Depth Change** | 10% | Order book depth change |
| **Market Activity** | 5/min | High trade frequency |

```rust
pub struct SignalThresholds {
    pub price_change_percent: f64,      // 2.0
    pub volume_spike_multiplier: f64,   // 1.5
    pub spread_percent: f64,            // 2.0
    pub depth_change_percent: f64,      // 10.0
    pub activity_trades_per_min: u32,   // 5
}
```

## Forkable COW Architecture

Users can fork the scanner, customize it, and get their own MCP endpoint.

### Per-COW Endpoints

Each forked scanner gets dedicated endpoints:

| Endpoint | Purpose |
|----------|---------|
| `GET /api/poly/scanner/{cow_id}/stream` | SSE stream for this COW's events |
| `GET /api/poly/scanner/{cow_id}/status` | This COW's scanner status |
| `GET /api/poly/scanner/{cow_id}/config` | Get current config |
| `PUT /api/poly/scanner/{cow_id}/config` | Update config |
| `MCP: erebus://scanner/{cow_id}` | MCP resource for this COW |

### Fork Workflow

```
1. User forks base scanner in Crossroads
   POST /api/marketplace/fork
   → Returns: { "cow_id": "abc123" }

2. User customizes config via Sandbox UI
   PUT /api/poly/scanner/abc123/config

3. User's agents subscribe to their scanner
   MCP resources/list → "erebus://scanner/abc123"

4. Fixed endpoint for set-and-forget
   curl -N /api/poly/scanner/abc123/stream
```

### Scanner Config Model

```rust
pub struct ScannerConfig {
    pub cow_id: Uuid,
    pub owner_wallet: String,
    pub name: String,

    // Market Selection
    pub tracked_markets: Vec<String>,
    pub track_top_by_volume: bool,
    pub top_market_count: u32,
    pub category_filters: Vec<String>,

    // Signal Thresholds (customizable)
    pub price_change_threshold: f64,
    pub volume_spike_multiplier: f64,
    pub spread_threshold: f64,
    pub activity_threshold: u32,

    // Output Config
    pub emit_low_significance: bool,
    pub emit_medium_significance: bool,
    pub emit_high_significance: bool,
    pub emit_critical_only: bool,

    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

## MCP Integration

### Scanner Tools

| Tool | Description |
|------|-------------|
| `scanner_status` | Get scanner status and statistics |
| `scanner_events` | Get recent events with filtering |
| `scanner_track_market` | Track a market by slug |
| `scanner_search_markets` | Fuzzy search markets |
| `scanner_save_to_engram` | Save event as engram |

### Scanner Resources

| URI | Description |
|-----|-------------|
| `erebus://scanner` | Scanner status and config |
| `erebus://scanner/{cow_id}` | User's forked scanner |
| `erebus://scanner/events` | Live event stream (SSE) |

### Event Format (MCP Compatible)

```json
{
  "id": "uuid-here",
  "event_type": "price_change",
  "timestamp": "2024-01-15T10:30:00Z",
  "significance": "high",
  "market": {
    "id": "condition_id",
    "slug": "market-slug",
    "title": "Will X happen?"
  },
  "data": {
    "outcome": "Yes",
    "old_price": "0.45",
    "new_price": "0.52",
    "change_percent": "15.56"
  }
}
```

## Service Structure

```
svc/poly-mev/
├── Cargo.toml
├── .env.example
├── migrations/
│   └── 001_create_scanner_tables.sql
└── src/
    ├── main.rs
    ├── config.rs
    ├── error.rs
    ├── server.rs
    │
    ├── api/
    │   ├── mod.rs
    │   ├── gamma.rs          # Gamma API (markets, events)
    │   ├── clob.rs           # CLOB API (orderbook, prices)
    │   └── websocket.rs      # WebSocket client
    │
    ├── models/
    │   ├── mod.rs
    │   ├── market.rs
    │   ├── order.rs
    │   └── scanner_event.rs
    │
    ├── scanner/
    │   ├── mod.rs
    │   ├── engine.rs         # Main scanner loop
    │   ├── signals.rs        # Signal detection
    │   └── processor.rs      # Event normalization
    │
    ├── handlers/
    │   ├── mod.rs
    │   ├── health.rs
    │   ├── markets.rs
    │   ├── scanner.rs
    │   └── sse.rs            # SSE streaming
    │
    └── database/
        ├── mod.rs
        └── repositories/
            ├── mod.rs
            ├── markets.rs
            └── events.rs
```

## API Endpoints

### Scanner Control

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Service health |
| GET | `/scanner/stream` | SSE event stream |
| GET | `/scanner/status` | Scanner status + stats |
| POST | `/scanner/start` | Start scanning |
| POST | `/scanner/stop` | Stop scanning |

### Markets

| Method | Path | Description |
|--------|------|-------------|
| GET | `/markets` | List tracked markets |
| GET | `/markets/search?q=` | Fuzzy search markets |
| POST | `/markets/track` | Track by slug or condition_id |
| DELETE | `/markets/:id/track` | Untrack market |
| GET | `/markets/top?limit=50` | Top markets by volume |

### Events

| Method | Path | Description |
|--------|------|-------------|
| GET | `/events?limit=100` | Recent scanner events |

## Frontend Components

### Sandbox Tab (MemCache)

New first tab in MemCache submenu containing:
- **Viewport** - Live scanner feed display
- **Tool Builder** - Build custom tools/COWs
- **Engram Creator** - Create engrams from scanner insights

### New Hook: usePolyScanner

```typescript
interface UsePolyScannerReturn {
  // Event streaming
  events: ScannerEvent[];
  isConnected: boolean;
  connectToStream: () => void;
  disconnectFromStream: () => void;
  clearEvents: () => void;

  // Scanner status
  status: ScannerStatus | null;

  // Market management
  trackedMarkets: TrackedMarket[];
  topMarkets: TrackedMarket[];
  searchMarkets: (query: string) => Promise<TrackedMarket[]>;
  trackMarket: (slugOrId: string) => Promise<boolean>;
  untrackMarket: (id: string) => Promise<boolean>;

  // Filtering
  getEventsByType: (type: string) => ScannerEvent[];
  getHighSignificanceEvents: () => ScannerEvent[];
}
```

## Database Schema

```sql
-- Tracked markets
CREATE TABLE tracked_markets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    market_id VARCHAR(255) NOT NULL UNIQUE,
    condition_id VARCHAR(255),
    slug VARCHAR(255),
    title TEXT NOT NULL,
    category VARCHAR(100),
    last_price DECIMAL(20, 8),
    volume_24h DECIMAL(20, 8),
    is_top_market BOOLEAN DEFAULT false,
    tracked_since TIMESTAMPTZ DEFAULT NOW(),
    is_active BOOLEAN DEFAULT true,
    metadata JSONB DEFAULT '{}'
);

-- Scanner events (for history/analysis)
CREATE TABLE scanner_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type VARCHAR(50) NOT NULL,
    market_id VARCHAR(255) NOT NULL,
    significance VARCHAR(20) NOT NULL,
    event_data JSONB NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Market snapshots (for trend detection)
CREATE TABLE market_snapshots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    market_id VARCHAR(255) NOT NULL,
    price DECIMAL(20, 8),
    volume_24h DECIMAL(20, 8),
    liquidity DECIMAL(20, 8),
    spread DECIMAL(20, 8),
    snapshot_at TIMESTAMPTZ DEFAULT NOW()
);
```

## Implementation Phases

### Phase 1: Backend Foundation
1. Create `svc/poly-mev/` directory structure
2. Set up Cargo.toml with dependencies
3. Implement config, error, server modules
4. Create all model files
5. Implement health endpoint

### Phase 2: Polymarket API Clients
1. Implement `api/gamma.rs` - market fetching
2. Implement `api/clob.rs` - orderbook data
3. Implement `api/websocket.rs` - real-time stream
4. Test connections to all APIs

### Phase 3: Scanner Engine
1. Implement `scanner/signals.rs` - detection algorithms
2. Implement `scanner/processor.rs` - event normalization
3. Implement `scanner/engine.rs` - main loop
4. Set up broadcast channel for SSE

### Phase 4: HTTP Handlers + Erebus Integration
1. Implement SSE streaming handler
2. Implement market CRUD + search
3. Create Erebus proxy routes at `/api/poly/*`
4. Add database repositories

### Phase 5: Frontend - Sandbox Tab
1. Add "sandbox" to MemCacheSection type
2. Create Sandbox.tsx container
3. Create ScannerViewport.tsx
4. Create MarketSearch.tsx
5. Implement usePolyScanner hook

### Phase 6: Forkable COW + MCP
1. Create ScannerConfig model
2. Implement per-COW SSE endpoints
3. Add scanner tools to Erebus MCP handler
4. Implement fork workflow in Crossroads
5. Test agent subscription

## Verification

### Backend Tests

```bash
# Health check
curl http://localhost:9006/health

# Scanner status
curl http://localhost:9006/scanner/status

# Top markets
curl http://localhost:9006/markets/top?limit=20

# Search markets
curl "http://localhost:9006/markets/search?q=trump"

# SSE stream (keep open)
curl -N http://localhost:9006/scanner/stream
```

### MCP Tests

```bash
# Get scanner status via MCP
curl -X POST http://localhost:3000/mcp/jsonrpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call",
    "params": {
      "name": "scanner_status",
      "arguments": {}
    }
  }'

# Get high significance events
curl -X POST http://localhost:3000/mcp/jsonrpc \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/call",
    "params": {
      "name": "scanner_events",
      "arguments": { "significance": "high", "limit": 5 }
    }
  }'
```

### Forkable COW Tests

```bash
# Fork the base scanner
curl -X POST http://localhost:3000/api/marketplace/fork \
  -H "Content-Type: application/json" \
  -d '{
    "listing_id": "scanner-base",
    "wallet_address": "0xtest...",
    "custom_name": "My Politics Scanner"
  }'

# Subscribe to custom scanner SSE
curl -N http://localhost:3000/api/poly/scanner/{cow_id}/stream
```

## Related

- [Poly Mev Plan](./plan.md)
- [Engrams Service](../services/engrams.md)
- [Crossroads Marketplace](../services/crossroads.md)
- [MCP Servers](../reference/mcp-servers.md)
