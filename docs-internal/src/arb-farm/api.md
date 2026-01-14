# ArbFarm API Reference

ArbFarm runs on port 9007. All endpoints are relative to `http://localhost:9007`.

## Health

```bash
GET /health
```

Returns service status.

**Response:**
```json
{
  "status": "ok",
  "service": "arb-farm",
  "version": "0.1.0"
}
```

## Scanner

Control the MEV opportunity scanner.

| Method | Path | Description |
|--------|------|-------------|
| GET | `/scanner/status` | Scanner status + statistics |
| POST | `/scanner/start` | Start scanning |
| POST | `/scanner/stop` | Stop scanning |
| GET | `/scanner/signals` | Get recent signals |

### Scanner Status Response

```json
{
  "running": true,
  "signals_detected": 1247,
  "signals_per_minute": 12.5,
  "venues_active": 5,
  "uptime_seconds": 3600
}
```

## Edges (Opportunities)

Detected MEV opportunities.

| Method | Path | Description |
|--------|------|-------------|
| GET | `/edges` | List detected edges |
| GET | `/edges/atomic` | List atomic (guaranteed profit) edges |
| GET | `/edges/:id` | Get edge details |
| POST | `/edges/:id/approve` | Approve for execution |
| POST | `/edges/:id/reject` | Reject with reason |
| POST | `/edges/:id/execute` | Execute immediately |
| POST | `/edges/:id/simulate` | Simulate execution |

### Query Parameters

```
?status=detected|pending_approval|executing|executed|expired|failed|rejected
?edge_type=dex_arb|curve_arb|liquidation|backrun|jit
?atomicity=fully_atomic|partially_atomic|non_atomic
?limit=50&offset=0
```

### Edge Response

```json
{
  "id": "uuid",
  "edge_type": "dex_arb",
  "execution_mode": "autonomous",
  "atomicity": "fully_atomic",
  "estimated_profit_lamports": 150000,
  "risk_score": 15,
  "status": "detected",
  "route_data": {
    "from_venue": "jupiter",
    "to_venue": "raydium",
    "token_pair": ["SOL", "BONK"]
  },
  "created_at": "2024-01-15T10:30:00Z",
  "expires_at": "2024-01-15T10:31:00Z"
}
```

## Trades

Trade execution history and statistics.

| Method | Path | Description |
|--------|------|-------------|
| GET | `/trades` | List trade history |
| GET | `/trades/:id` | Get trade details |
| GET | `/trades/stats` | P&L statistics |
| GET | `/trades/daily` | Daily statistics |

### Trade Stats Response

```json
{
  "total_trades": 145,
  "profitable_trades": 112,
  "win_rate": 0.77,
  "total_profit_lamports": 2500000,
  "total_gas_cost_lamports": 150000,
  "net_profit_lamports": 2350000,
  "avg_profit_bps": 35,
  "best_trade_lamports": 500000
}
```

## Bonding Curves

pump.fun, moonshot, and other bonding curve operations.

| Method | Path | Description |
|--------|------|-------------|
| GET | `/curves/tokens` | List tracked curve tokens |
| GET | `/curves/health` | Venue health status |
| GET | `/curves/graduation-candidates` | Tokens near graduation |
| GET | `/curves/cross-venue-arb` | Cross-venue arb opportunities |
| GET | `/curves/:mint/progress` | Graduation progress for token |
| GET | `/curves/:mint/holders` | Holder statistics |
| POST | `/curves/:mint/quote` | Get buy/sell quote |
| GET | `/curves/:mint/parameters` | Curve parameters |

### Graduation Progress Response

```json
{
  "token_mint": "ABC123...",
  "token_name": "MoonCoin",
  "progress_percent": 72.5,
  "bonding_curve_address": "DEF456...",
  "current_supply": 800000000,
  "virtual_sol_reserves": 30.5,
  "virtual_token_reserves": 200000000,
  "graduation_threshold_sol": 85.0,
  "estimated_graduation_at": "2024-01-15T14:00:00Z"
}
```

### Holder Stats Response

```json
{
  "total_holders": 234,
  "top_10_concentration": 0.45,
  "creator_holdings_percent": 0.05,
  "holders": [
    {
      "address": "7Vk3...",
      "balance": 50000000,
      "percent": 0.125
    }
  ]
}
```

## Research/DD

URL ingestion, strategy discovery, and backtesting.

| Method | Path | Description |
|--------|------|-------------|
| POST | `/research/ingest` | Ingest URL for analysis |
| GET | `/research/discoveries` | List discovered strategies |
| GET | `/research/discoveries/:id` | Get discovery details |
| POST | `/research/discoveries/:id/approve` | Approve strategy |
| POST | `/research/discoveries/:id/reject` | Reject strategy |
| POST | `/research/backtest` | Run backtest |
| GET | `/research/backtest/:id` | Get backtest results |
| GET | `/research/sources` | List monitored sources |
| POST | `/research/sources` | Add source to monitor |
| GET | `/research/sources/:id` | Get source details |
| DELETE | `/research/sources/:id` | Remove source |
| POST | `/research/sources/:id/toggle` | Toggle source active |
| GET | `/research/alerts` | List social alerts |
| GET | `/research/stats` | Monitor statistics |
| POST | `/research/monitor` | Add account to monitor |

### Ingest URL Request

```json
{
  "url": "https://twitter.com/trader/status/123456789",
  "context": "Potential alpha from known profitable trader"
}
```

### Ingest URL Response

```json
{
  "id": "uuid",
  "url": "https://twitter.com/trader/status/123456789",
  "content_type": "tweet",
  "title": "Tweet by @trader",
  "summary": "Buy $BONK at curve progress 20%, sell at 80%",
  "tokens_mentioned": ["$BONK", "$SOL"],
  "addresses_found": [],
  "numbers_extracted": [
    {"value": 20.0, "context": "entry point"},
    {"value": 80.0, "context": "exit point"}
  ],
  "has_trading_signal": true,
  "analyzed_at": "2024-01-15T10:30:00Z"
}
```

### Backtest Request

```json
{
  "strategy_id": "uuid",
  "start_date": "2024-01-01",
  "end_date": "2024-01-15",
  "initial_capital_sol": 10.0
}
```

### Backtest Response

```json
{
  "id": "uuid",
  "strategy_id": "uuid",
  "status": "completed",
  "summary": {
    "total_trades": 45,
    "winning_trades": 32,
    "losing_trades": 13,
    "win_rate": 0.71,
    "total_profit_sol": 2.5,
    "total_return_percent": 25.0,
    "max_drawdown_percent": 8.5,
    "sharpe_ratio": 1.85,
    "sortino_ratio": 2.12,
    "calmar_ratio": 2.94
  },
  "metrics": {
    "avg_trade_profit_sol": 0.055,
    "best_trade_sol": 0.8,
    "worst_trade_sol": -0.3,
    "avg_trade_duration_hours": 4.2,
    "max_consecutive_wins": 8,
    "max_consecutive_losses": 3
  },
  "equity_curve": [
    {"timestamp": "2024-01-01T00:00:00Z", "equity_sol": 10.0},
    {"timestamp": "2024-01-15T00:00:00Z", "equity_sol": 12.5}
  ]
}
```

### Add Source Request

```json
{
  "source_type": "twitter",
  "handle_or_url": "@ZachXBT",
  "track_type": "threat"
}
```

## KOL Tracking + Copy Trading

Track whale wallets and influential traders with optional copy trading.

| Method | Path | Description |
|--------|------|-------------|
| GET | `/kol` | List tracked KOLs |
| POST | `/kol` | Add KOL to track |
| GET | `/kol/:id` | Get KOL details |
| PUT | `/kol/:id` | Update KOL settings |
| DELETE | `/kol/:id` | Stop tracking |
| GET | `/kol/:id/trades` | KOL trade history |
| GET | `/kol/:id/stats` | Performance statistics |
| GET | `/kol/:id/trust` | Trust score breakdown |
| POST | `/kol/:id/copy/enable` | Enable copy trading |
| POST | `/kol/:id/copy/disable` | Disable copy trading |
| GET | `/kol/:id/copy/history` | Copy trade history |
| GET | `/kol/copies/active` | Active copy positions |
| GET | `/kol/copies/stats` | Copy trading stats |

### Add KOL Request

```json
{
  "wallet_address": "7Vk3...8xNp",
  "display_name": "Whale Watcher"
}
```

Or by Twitter handle:

```json
{
  "twitter_handle": "@CryptoWhale",
  "display_name": "Crypto Whale"
}
```

### KOL Stats Response

```json
{
  "entity_id": "uuid",
  "display_name": "Whale Watcher",
  "identifier": "7Vk3...8xNp",
  "trust_score": 78.5,
  "total_trades": 145,
  "profitable_trades": 112,
  "win_rate": 0.77,
  "avg_profit_percent": 12.5,
  "total_volume_sol": 250.5,
  "our_copy_count": 45,
  "our_copy_profit_sol": 2.5,
  "copy_trading_enabled": true,
  "last_trade_at": "2024-01-15T10:30:00Z"
}
```

### Enable Copy Trading Request

```json
{
  "max_position_sol": 0.5,
  "delay_ms": 500,
  "min_trust_score": 60,
  "copy_percentage": 0.1,
  "token_blacklist": ["ScamToken..."]
}
```

### Trust Score Breakdown Response

```json
{
  "base_score": 50.0,
  "win_rate_factor": 23.4,
  "trade_count_factor": 7.5,
  "avg_profit_factor": 6.2,
  "drawdown_penalty": 4.5,
  "final_score": 82.6
}
```

## Threat Detection

Identify rug pulls, honeypots, scam wallets, and other threats.

| Method | Path | Description |
|--------|------|-------------|
| GET | `/threat/check/:mint` | Full threat analysis on token |
| GET | `/threat/wallet/:address` | Check wallet for scam history |
| GET | `/threat/blocked` | List blocked entities |
| POST | `/threat/report` | Report a threat |
| DELETE | `/threat/blocked/:address` | Remove from blocklist |
| GET | `/threat/blocked/:address/status` | Check if blocked |
| GET | `/threat/whitelist` | List whitelisted entities |
| POST | `/threat/whitelist` | Whitelist an entity |
| DELETE | `/threat/whitelist/:address` | Remove from whitelist |
| GET | `/threat/whitelist/:address/status` | Check if whitelisted |
| GET | `/threat/watch` | List watched wallets |
| POST | `/threat/watch` | Add wallet to watch list |
| GET | `/threat/alerts` | Get recent alerts |
| GET | `/threat/score/:mint/history` | Get threat score history |
| GET | `/threat/stats` | Get detection statistics |

### Threat Check Response

```json
{
  "success": true,
  "score": {
    "token_mint": "TokenMint...",
    "overall_score": 0.65,
    "factors": {
      "has_mint_authority": true,
      "has_freeze_authority": false,
      "top_10_concentration": 0.78,
      "rugcheck_score": 0.45,
      "goplus_honeypot": false,
      "wash_trade_likelihood": 0.2
    },
    "risk_level": "high",
    "recommendation": "CAUTION - Significant risk factors detected",
    "external_data": {
      "rugcheck": { ... },
      "goplus": { ... },
      "holder_analysis": { ... }
    }
  }
}
```

### Report Threat Request

```json
{
  "entity_type": "token",
  "address": "ScamToken...",
  "category": "rug_pull",
  "reason": "Creator dumped 80% of supply",
  "evidence_url": "https://solscan.io/tx/..."
}
```

### Watch Wallet Request

```json
{
  "wallet_address": "CreatorWallet...",
  "related_token_mint": "TokenMint...",
  "watch_reason": "Token creator - monitoring for dumps",
  "alert_on_sell": true,
  "alert_threshold_sol": 10.0
}
```

### Threat Stats Response

```json
{
  "total_tokens_checked": 1234,
  "threats_detected": 89,
  "rugs_prevented": 23,
  "blocked_tokens": 45,
  "blocked_wallets": 67,
  "whitelisted_count": 12,
  "watched_wallets": 34,
  "alerts_last_24h": 15
}
```

## SSE Streams

Real-time Server-Sent Event streams.

| Path | Description |
|------|-------------|
| `/scanner/stream` | Scanner signals in real-time |
| `/edges/stream` | New edge opportunities |
| `/events/stream` | All system events |
| `/threat/stream` | Threat alerts |

### Connecting to SSE

```bash
curl -N http://localhost:9007/edges/stream
```

### Event Format

```json
{
  "event_type": "edge.detected",
  "data": {
    "id": "uuid",
    "edge_type": "dex_arb",
    "estimated_profit_lamports": 150000
  },
  "timestamp": "2024-01-15T10:30:00Z"
}
```

## MCP Tools

ArbFarm exposes MCP tools for agent integration.

| Method | Path | Description |
|--------|------|-------------|
| GET | `/mcp/manifest` | MCP manifest |
| GET | `/mcp/tools` | List available tools |

### Available MCP Tools

**Scanner Tools:**
- `scanner_status` - Get scanner status
- `scanner_signals` - Get recent signals
- `scanner_start` - Start scanner
- `scanner_stop` - Stop scanner

**Edge Tools:**
- `edge_list` - List edges with filtering
- `edge_details` - Get edge details
- `edge_approve` - Approve edge
- `edge_reject` - Reject edge
- `edge_list_atomic` - List atomic edges
- `edge_simulate` - Simulate execution
- `edge_execute` - Execute edge

**Curve Tools:**
- `curve_tokens` - List curve tokens
- `curve_progress` - Get graduation progress
- `curve_holders` - Get holder stats
- `curve_quote` - Get buy/sell quote
- `curve_graduation_candidates` - Find graduation opportunities

**Research Tools:**
- `research_ingest_url` - Analyze URL
- `research_list_discoveries` - List discovered strategies
- `research_approve_discovery` - Approve strategy
- `research_reject_discovery` - Reject strategy
- `research_backtest_strategy` - Run backtest
- `research_list_sources` - List monitored sources
- `research_alerts` - Get social alerts
- `research_stats` - Monitor statistics

**Trade Tools:**
- `trade_history` - Get trade history
- `trade_stats` - Get P&L statistics

**KOL Tools:**
- `kol_track` - Start tracking a KOL
- `kol_list` - List tracked KOLs
- `kol_stats` - Get KOL performance stats
- `kol_trades` - Get KOL trade history
- `kol_trust_breakdown` - Get trust score details
- `copy_enable` - Enable copy trading
- `copy_disable` - Disable copy trading
- `copy_active` - List active copy positions
- `copy_stats` - Get copy trading stats
- `copy_history` - Get copy trade history

**Threat Tools:**
- `threat_check_token` - Full threat analysis on a token
- `threat_check_wallet` - Analyze wallet for scam history
- `threat_list_blocked` - List blocked entities
- `threat_report` - Report a threat
- `threat_whitelist` - Whitelist an entity
- `threat_watch_wallet` - Add wallet to monitoring
- `threat_alerts` - Get recent threat alerts
- `threat_score_history` - Get threat score history
- `threat_stats` - Get detection statistics
- `threat_is_blocked` - Check if address is blocked
- `threat_is_whitelisted` - Check if address is whitelisted

## Error Responses

All errors follow a consistent format:

```json
{
  "error": "Description of the error",
  "code": 400
}
```

| Code | Description |
|------|-------------|
| 400 | Bad Request - Invalid parameters |
| 401 | Unauthorized |
| 403 | Forbidden - Threat detected |
| 404 | Not Found |
| 409 | Conflict - Consensus failed |
| 422 | Validation Error |
| 429 | Rate Limited |
| 500 | Internal Server Error |
| 502 | External API Error |

## Related

- [Service Architecture](./service.md)
- [Research Module](./research.md)
- [Bonding Curves](./curves.md)
- [KOL Tracking](./kol.md)
- [Threat Detection](./threat.md)
- [Event Bus](./events.md)
