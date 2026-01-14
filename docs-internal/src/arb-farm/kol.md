# KOL Tracking + Copy Trading

The KOL (Key Opinion Leader) module enables tracking of whale wallets and influential traders, with optional automated copy trading.

## Overview

```
Wallet/Handle → Track Trades → Calculate Trust → Optional Copy Trading
                     │                │
                     ▼                ▼
               Store Trades     Auto-Disable
               Update Stats    if Trust Falls
```

## Key Concepts

### KOL Entity Types

| Type | Description | Tracking Method |
|------|-------------|-----------------|
| **Wallet** | Solana wallet address | Helius webhooks |
| **TwitterHandle** | X/Twitter account | Social monitoring |

### Trust Score

Trust scores (0-100) are calculated from trading performance:

| Factor | Weight | Description |
|--------|--------|-------------|
| Win Rate | 30% | Profitable trades / Total trades |
| Trade Count | 10% | Experience (max at 100 trades) |
| Avg Profit | 10% | Average profit per trade |
| Drawdown | -20% | Penalty for max drawdown |
| Base | 50 | Starting score |

**Formula:**
```
score = 50 + (win_rate × 30) + (min(trades, 100)/100 × 10)
        + (min(avg_profit, 50)/50 × 10) - (max_drawdown/100 × 20)
```

### Copy Trade Config

When copy trading is enabled, these parameters control behavior:

| Parameter | Default | Description |
|-----------|---------|-------------|
| `max_position_sol` | 0.5 | Maximum SOL per copied trade |
| `delay_ms` | 500 | Wait before copying (avoid frontrun) |
| `min_trust_score` | 60 | Auto-disable below this threshold |
| `copy_percentage` | 10% | Percentage of KOL's position to copy |
| `token_whitelist` | None | Only copy these tokens |
| `token_blacklist` | None | Never copy these tokens |

## API Endpoints

### KOL Management

| Method | Path | Description |
|--------|------|-------------|
| GET | `/kol` | List tracked KOLs |
| POST | `/kol` | Add KOL to track |
| GET | `/kol/:id` | Get KOL details |
| PUT | `/kol/:id` | Update KOL settings |
| DELETE | `/kol/:id` | Stop tracking |

### KOL Data

| Method | Path | Description |
|--------|------|-------------|
| GET | `/kol/:id/trades` | KOL's trade history |
| GET | `/kol/:id/stats` | Performance statistics |
| GET | `/kol/:id/trust` | Trust score breakdown |

### Copy Trading

| Method | Path | Description |
|--------|------|-------------|
| POST | `/kol/:id/copy/enable` | Enable copy trading |
| POST | `/kol/:id/copy/disable` | Disable copy trading |
| GET | `/kol/:id/copy/history` | Copy trade history |
| GET | `/kol/copies/active` | All active copy positions |
| GET | `/kol/copies/stats` | Global copy trading stats |

## Usage Examples

### Add a KOL by Wallet

```bash
curl -X POST http://localhost:9007/kol \
  -H "Content-Type: application/json" \
  -d '{
    "wallet_address": "7Vk3...8xNp",
    "display_name": "Whale Watcher"
  }'
```

### Add a KOL by Twitter Handle

```bash
curl -X POST http://localhost:9007/kol \
  -H "Content-Type: application/json" \
  -d '{
    "twitter_handle": "@CryptoWhale",
    "display_name": "Crypto Whale"
  }'
```

### Enable Copy Trading

```bash
curl -X POST http://localhost:9007/kol/{id}/copy/enable \
  -H "Content-Type: application/json" \
  -d '{
    "max_position_sol": 0.5,
    "delay_ms": 1000,
    "min_trust_score": 65,
    "copy_percentage": 0.1
  }'
```

### Get Trust Score Breakdown

```bash
curl http://localhost:9007/kol/{id}/trust
```

**Response:**
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

### List KOLs with Copy Trading Enabled

```bash
curl "http://localhost:9007/kol?copy_enabled=true&min_trust_score=60"
```

### Get Copy Trading Statistics

```bash
curl http://localhost:9007/kol/copies/stats
```

**Response:**
```json
{
  "total_copies": 145,
  "executed": 130,
  "failed": 10,
  "skipped": 5,
  "total_profit_lamports": 2500000000,
  "avg_delay_ms": 523.5
}
```

## Copy Trade Flow

```
KOL Trade Detected (Helius Webhook)
         │
         ▼
┌──────────────────┐
│ Parse Trade Data │
│ - Token mint     │
│ - Amount         │
│ - Trade type     │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ Check Config     │
│ - Copy enabled?  │
│ - Trust >= min?  │
│ - Within limits? │
│ - Token allowed? │
└────────┬─────────┘
         │
    ┌────┴────┐
    │         │
   Pass     Fail
    │         │
    ▼         ▼
┌────────┐ Mark as
│ Delay  │ Skipped
│ (conf) │ + reason
└───┬────┘
    │
    ▼
┌────────────────┐
│ Execute Trade  │
│ (scaled amount)│
└───────┬────────┘
        │
   ┌────┴────┐
   │         │
Success    Fail
   │         │
   ▼         ▼
Record     Record
P&L        Error
```

## Copy Trade Status

| Status | Description |
|--------|-------------|
| `pending` | Detected, waiting to execute |
| `executing` | Trade in progress |
| `executed` | Successfully copied |
| `failed` | Copy attempt failed |
| `skipped` | Skipped (trust, limits, blacklist) |

## Events Emitted

| Topic | Description |
|-------|-------------|
| `arb.kol.added` | New KOL tracked |
| `arb.kol.removed` | KOL removed |
| `arb.kol.trade.detected` | KOL made a trade |
| `arb.kol.trade.copied` | We copied the trade |
| `arb.kol.trade.skipped` | Trade skipped |
| `arb.kol.trust.updated` | Trust score changed |
| `arb.kol.copy.enabled` | Copy trading enabled |
| `arb.kol.copy.disabled` | Copy trading disabled |
| `arb.kol.copy.auto_disabled` | Auto-disabled (low trust) |

## MCP Tools

| Tool | Description |
|------|-------------|
| `kol_track` | Start tracking a KOL |
| `kol_list` | List tracked KOLs |
| `kol_stats` | Get KOL performance stats |
| `kol_trades` | Get KOL trade history |
| `kol_trust_breakdown` | Get trust score details |
| `copy_enable` | Enable copy trading |
| `copy_disable` | Disable copy trading |
| `copy_active` | List active copy positions |
| `copy_stats` | Get copy trading stats |
| `copy_history` | Get copy trade history |

## Risk Management

### Auto-Disable

Copy trading automatically disables when:
- Trust score drops below `min_trust_score`
- KOL has consecutive losses
- KOL is flagged by threat detection

### Safety Features

- **Delay**: Configurable delay before copying
- **Max Position**: Hard limit on position size
- **Token Filters**: Whitelist/blacklist tokens
- **Trust Threshold**: Minimum trust requirement

## Related

- [Service Architecture](./service.md)
- [API Reference](./api.md)
- [Research Module](./research.md)
