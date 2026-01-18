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

## Signals

Real-time market signals detected by the scanner.

| Method | Path | Description |
|--------|------|-------------|
| GET | `/signals` | List recent signals |
| GET | `/signals/stream` | SSE stream of live signals |
| GET | `/signals/:id` | Get signal details |

### Query Parameters

```
?signal_type=price_discrepancy|volume_spike|dex_arb|liquidation|curve_graduation|kol_signal
?venue_type=jupiter|raydium|pumpfun|moonshot|kamino|marginfi
?min_profit_bps=50
?min_confidence=0.8
?limit=50&offset=0
```

### Signal Response

```json
{
  "id": "uuid",
  "signal_type": "dex_arb",
  "venue_type": "jupiter",
  "estimated_profit_bps": 75,
  "confidence": 0.85,
  "token_pair": ["SOL", "BONK"],
  "detected_at": "2024-01-15T10:30:00Z",
  "expires_at": "2024-01-15T10:31:00Z",
  "metadata": {
    "price_a": 0.000025,
    "price_b": 0.0000255,
    "spread_bps": 200
  }
}
```

## Strategies

Strategy management for automated trading.

| Method | Path | Description |
|--------|------|-------------|
| GET | `/strategies` | List all strategies |
| POST | `/strategies` | Create new strategy |
| GET | `/strategies/:id` | Get strategy details |
| PUT | `/strategies/:id` | Update strategy |
| DELETE | `/strategies/:id` | Permanently delete strategy and data |
| POST | `/strategies/:id/toggle` | Enable/disable strategy (pause/resume) |
| POST | `/strategies/:id/kill` | Emergency stop - halt all running operations |
| GET | `/strategies/:id/stats` | Strategy performance stats |

### Kill Strategy Response

Immediately stops all running operations for a strategy (cancels pending approvals, halts executions) but keeps the strategy in your list for later use.

```json
{
  "success": true,
  "id": "uuid",
  "strategy_name": "Jupiter-Raydium Arb",
  "message": "Strategy killed - all operations halted",
  "action": "emergency_stop"
}
```

### Create Strategy Request

```json
{
  "name": "DEX Arbitrage - Jupiter/Raydium",
  "strategy_type": "dex_arb",
  "venue_types": ["dex_amm"],
  "execution_mode": "hybrid",
  "risk_params": {
    "max_position_sol": 0.5,
    "daily_loss_limit_sol": 2.0,
    "min_profit_bps": 50,
    "max_slippage_bps": 100
  }
}
```

### Strategy Response

```json
{
  "id": "uuid",
  "wallet_address": "5wrmi85pTPmB4NDv7rUYncEMi1KqVo93bZn3XtXSbjYT",
  "name": "DEX Arbitrage - Jupiter/Raydium",
  "strategy_type": "dex_arb",
  "venue_types": ["dex_amm"],
  "execution_mode": "hybrid",
  "risk_params": {
    "max_position_sol": 0.5,
    "daily_loss_limit_sol": 2.0,
    "min_profit_bps": 50,
    "max_slippage_bps": 100
  },
  "is_active": true,
  "stats": {
    "total_trades": 45,
    "win_rate": 0.78,
    "total_profit_sol": 1.25
  },
  "created_at": "2024-01-15T10:30:00Z",
  "updated_at": "2024-01-15T12:30:00Z"
}
```

## Approvals

Pending approval workflow for trades requiring manual confirmation.

| Method | Path | Description |
|--------|------|-------------|
| GET | `/approvals` | List all approvals |
| GET | `/approvals/pending` | List pending approvals only |
| GET | `/approvals/:id` | Get approval details |
| POST | `/approvals/:id/approve` | Approve for execution |
| POST | `/approvals/:id/reject` | Reject with reason |
| POST | `/approvals/cleanup` | Clean up expired approvals |
| POST | `/approvals/hecate-recommendation` | Add Hecate AI recommendation |

### Pending Approvals Response

```json
{
  "approvals": [
    {
      "id": "uuid",
      "edge_id": "uuid",
      "strategy_id": "uuid",
      "approval_type": "entry",
      "status": "pending",
      "expires_at": "2024-01-15T10:35:00Z",
      "hecate_decision": true,
      "hecate_reasoning": "High confidence opportunity with low risk score",
      "created_at": "2024-01-15T10:30:00Z"
    }
  ],
  "total": 1
}
```

### Approve Request

```json
{
  "notes": "Optional notes about approval decision"
}
```

### Reject Request

```json
{
  "reason": "Risk too high for current market conditions"
}
```

### Hecate Recommendation Request

```json
{
  "approval_id": "uuid",
  "decision": true,
  "reasoning": "Analysis indicates high probability of profit"
}
```

## Execution Config

Global execution toggle and configuration.

| Method | Path | Description |
|--------|------|-------------|
| GET | `/execution/config` | Get execution config |
| PUT | `/execution/config` | Update execution config |
| POST | `/execution/toggle` | Toggle auto-execution on/off |

### Execution Config Response

```json
{
  "auto_execution_enabled": false,
  "default_approval_timeout_secs": 300,
  "notify_hecate_on_pending": true
}
```

### Toggle Execution Request

```json
{
  "enabled": true
}
```

### Toggle Execution Response

```json
{
  "enabled": true,
  "message": "Auto-execution enabled"
}
```

## Settings

Global settings and configuration.

| Method | Path | Description |
|--------|------|-------------|
| GET | `/settings` | Get all settings |
| GET | `/settings/risk` | Get global risk config |
| POST | `/settings/risk` | Update global risk config |
| GET | `/settings/venues` | Get venue settings |
| GET | `/settings/api-keys` | Get API key status |

### Execution Settings Response

```json
{
  "auto_execute_enabled": false,
  "auto_min_confidence": 0.8,
  "auto_max_position_sol": 0.5,
  "require_simulation": true,
  "default_execution_mode": "agent_directed"
}
```

### Update Execution Settings Request

```json
{
  "auto_execute_enabled": true,
  "auto_min_confidence": 0.85,
  "auto_max_position_sol": 0.3,
  "require_simulation": true
}
```

### Risk Config Response

```json
{
  "max_position_sol": 5.0,
  "daily_loss_limit_sol": 2.0,
  "min_profit_bps": 50,
  "max_slippage_bps": 100,
  "max_concurrent_positions": 10,
  "cooldown_after_loss_ms": 5000
}
```

## Risk Levels

Quick risk profile configuration with presets.

| Method | Path | Description |
|--------|------|-------------|
| GET | `/config/risk` | Get current risk level and config |
| POST | `/config/risk` | Set risk level preset (low/medium/high) |
| POST | `/config/risk/custom` | Set custom risk parameters |

### Risk Level Presets

| Level | max_position_sol | max_concurrent | daily_loss_limit |
|-------|------------------|----------------|------------------|
| low | 0.02 SOL | 2 | 0.1 SOL |
| medium | 0.25 SOL | 10 | 1.0 SOL |
| high | 1.0 SOL | 20 | 5.0 SOL |

### Get Risk Level Response

```json
{
  "level": "medium",
  "max_position_sol": 0.25,
  "max_concurrent_positions": 10,
  "daily_loss_limit_sol": 1.0,
  "max_drawdown_percent": 25.0,
  "max_position_per_token_sol": 0.25,
  "cooldown_after_loss_ms": 3000,
  "volatility_scaling_enabled": true,
  "auto_pause_on_drawdown": true
}
```

### Set Risk Level Request

```json
{
  "level": "medium"
}
```

### Set Custom Risk Request

```json
{
  "max_position_sol": 0.5,
  "max_concurrent_positions": 5,
  "daily_loss_limit_sol": 2.0,
  "max_drawdown_percent": 30.0
}
```

## Positions

Active position tracking and management.

| Method | Path | Description |
|--------|------|-------------|
| GET | `/positions` | List all positions with stats |
| GET | `/positions/:id` | Get position details |
| POST | `/positions/:id/close` | Close a position |
| GET | `/positions/history` | Closed position history |
| GET | `/positions/exposure` | Current exposure breakdown |
| GET | `/positions/pnl-summary` | P&L summary |
| POST | `/positions/reconcile` | Reconcile with wallet |
| GET | `/positions/monitor/status` | Position monitor status |
| POST | `/positions/monitor/start` | Start position monitor |
| POST | `/positions/monitor/stop` | Stop position monitor |
| POST | `/positions/emergency-close` | Emergency close all positions |
| POST | `/positions/sell-all` | Sell all wallet tokens |

### Positions Response

```json
{
  "positions": [
    {
      "id": "uuid",
      "edge_id": "uuid",
      "strategy_id": "uuid",
      "token_mint": "ABC123...",
      "token_symbol": "MOON",
      "entry_amount_base": 0.25,
      "entry_token_amount": 1000000,
      "entry_price": 0.00025,
      "entry_time": "2024-01-15T10:30:00Z",
      "entry_tx_signature": "sig...",
      "current_price": 0.00030,
      "current_value_base": 0.30,
      "unrealized_pnl": 0.05,
      "unrealized_pnl_percent": 20.0,
      "high_water_mark": 0.00032,
      "exit_config": {
        "stop_loss_percent": 15.0,
        "take_profit_percent": 50.0,
        "trailing_stop_percent": 10.0,
        "time_limit_minutes": 30
      },
      "partial_exits": [],
      "status": "open",
      "momentum": {
        "velocity": 1.5,
        "momentum_score": 25.0,
        "momentum_decay_count": 0
      },
      "remaining_amount_base": 0.25,
      "remaining_token_amount": 1000000
    }
  ],
  "stats": {
    "total_positions_opened": 50,
    "total_positions_closed": 45,
    "active_positions": 5,
    "total_realized_pnl": 0.15,
    "total_unrealized_pnl": 0.05,
    "stop_losses_triggered": 10,
    "take_profits_triggered": 8
  }
}
```

### Close Position Request

```json
{
  "slippage_bps": 100
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

**Strategy Tools:**
- `strategy_list` - List all strategies
- `strategy_create` - Create new strategy
- `strategy_details` - Get strategy details
- `strategy_update` - Update strategy parameters
- `strategy_toggle` - Enable/disable strategy (pause/resume)
- `strategy_kill` - Emergency stop all running operations
- `strategy_delete` - Permanently delete strategy and data
- `strategy_batch_toggle` - Batch enable/disable
- `strategy_save_to_engrams` - Persist to memory layer

**Approval Tools:**
- `approval_list` - List all approvals
- `approval_pending` - List pending approvals
- `approval_details` - Get approval details
- `approval_approve` - Approve for execution
- `approval_reject` - Reject with reason

**Execution Tools:**
- `execution_config_get` - Get execution config
- `execution_toggle` - Toggle auto-execution

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
