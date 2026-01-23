# ArbFarm Profit Strategy - Complete Breakdown

## Overview

ArbFarm is an autonomous trading system for Solana pump.fun bonding curve tokens. It detects early-stage tokens, enters positions, and uses a tiered exit strategy with momentum-adaptive logic to maximize profits while protecting capital.

---

## Entry Strategy (Buy Criteria)

### Signal Detection Pipeline

```
Helius WebSocket → Pump.fun Venue → Scanner Agent → Strategy Engine → Opportunity Scorer → Executor
```

1. **Real-time Monitoring**: Helius WebSocket streams new token creations and trades
2. **Initial Filter**: Pump.fun venue filters for graduation progress 30-98%
3. **Signal Creation**: Scanner agent emits signals for matching tokens
4. **Strategy Match**: Strategy engine matches signals to active strategies
5. **Opportunity Scoring**: Curve scorer calculates comprehensive score (0-100)
6. **Execution Decision**: Only `StrongBuy` (≥80) or `Buy` (≥60) signals execute

---

## Opportunity Scoring System

### Score Thresholds
| Score | Recommendation | Action |
|-------|----------------|--------|
| ≥ 80 | **StrongBuy** | Execute immediately |
| ≥ 60 | **Buy** | Execute |
| ≥ 40 | **Hold** | Skip - monitor only |
| < 40 | **Avoid** | Skip - too risky |

### Scoring Weights (Default)
| Factor | Weight | Description |
|--------|--------|-------------|
| Graduation Progress | 30% | How close to Raydium migration |
| Volume | 20% | Trading activity and velocity |
| Holders | 20% | Distribution and growth |
| Momentum | 15% | Price action and buy pressure |
| Risk Penalty | 15% | Deductions for red flags |

### Scoring Weights (Aggressive)
| Factor | Weight | Description |
|--------|--------|-------------|
| Graduation Progress | 40% | Prioritize near-graduation |
| Volume | 15% | Less weight on volume |
| Holders | 15% | Less weight on holders |
| Momentum | 20% | More weight on momentum |
| Risk Penalty | 10% | Accept more risk |

### Scoring Weights (Conservative)
| Factor | Weight | Description |
|--------|--------|-------------|
| Graduation Progress | 20% | Less urgency |
| Volume | 20% | Steady activity |
| Holders | 30% | Strong distribution |
| Momentum | 10% | Less FOMO |
| Risk Penalty | 20% | Avoid risky tokens |

---

## Factor Scoring Details

### 1. Graduation Factor (0-100 points)

Measures progress toward Raydium migration (~$69K market cap):

| Progress | Score | Signal |
|----------|-------|--------|
| < 70% | 0 | Too early - skip |
| 70-80% | 0-30 | Early stage |
| 80-85% | 30-50 | Building momentum |
| 85-90% | 50-70 | Good opportunity |
| 90-95% | 70-90 | High priority |
| 95-99% | 90-100 | Imminent graduation |
| > 99% | 100 | "Imminent graduation" signal |

### 2. Volume Factor (0-100 points)

| Metric | Threshold | Points | Signal |
|--------|-----------|--------|--------|
| Volume 1h | > 10 SOL | +40 | "High volume" |
| Volume 1h | > 5 SOL | +25 | Good activity |
| Volume 1h | > 1 SOL | +15 | Minimum viable |
| Volume 1h | < 1 SOL | 0 | Skip - too quiet |
| Acceleration | > 1.0 | +30 | "Volume accelerating rapidly" |
| Acceleration | > 0.5 | +20 | "Volume trending up" |
| Acceleration | > 0.0 | +10 | Positive trend |
| Trade count 1h | > 50 | +20 | High activity |
| Trade count 1h | > 20 | +10 | Moderate activity |
| Avg trade size | > 0.5 SOL | +10 | "Large avg trade size" |

### 3. Holder Factor (0-100 points)

| Metric | Threshold | Points | Signal |
|--------|-----------|--------|--------|
| Total holders | ≥ 100 | +25 | Shows in positive signals |
| Total holders | ≥ 50 | +15 | Minimum viable |
| Total holders | < 50 | 10 max | "Low holder count" warning |
| Top 10 concentration | < 30% | +25 | "Well distributed holdings" |
| Top 10 concentration | < 50% | +15 | Acceptable distribution |
| Top 10 concentration | > 70% | warning | "High concentration" |
| Creator holdings | < 5% | +20 | Creator not dumping |
| Creator holdings | < 10% | +10 | Acceptable |
| Creator holdings | > 15% | warning | "Creator holds X%" |
| Gini coefficient | < 0.5 | +15 | "Fair distribution" |
| Gini coefficient | < 0.7 | +10 | Moderate inequality |
| Holder growth 1h | > 10 | +15 | "+X holders in 1h" |
| Holder growth 1h | > 0 | +10 | Growing |
| Holder growth 1h | < -5 | warning | "Holders declining" |

### 4. Momentum Factor (0-100 points)

| Metric | Threshold | Points | Signal |
|--------|-----------|--------|--------|
| Price momentum 1h | > 20% | +35 | "+X% price 1h" |
| Price momentum 1h | > 10% | +25 | Shows in signals |
| Price momentum 1h | > 5% | +15 | Positive |
| Price momentum 1h | > 0% | +10 | Stable |
| Price momentum 1h | < -10% | 0 | Skip - dumping |
| Buy/sell ratio 1h | > 2.0 | +30 | "Strong buy pressure: Xx" |
| Buy/sell ratio 1h | > 1.5 | +20 | "Healthy buy pressure" |
| Buy/sell ratio 1h | > 1.2 | +10 | Slight buy pressure |
| Buy/sell ratio 1h | < 0.8 | cap at 10 | Sell pressure |
| Volume velocity | high | +0-20 | Bonus for fast volume |
| Unique buyers 1h | > 15 | +15 | "X unique buyers 1h" |
| Unique buyers 1h | ≥ 5 | +10 | Minimum viable |

### 5. Risk Penalty (0-100% reduction)

| Risk Factor | Threshold | Penalty | Warning |
|-------------|-----------|---------|---------|
| Wash trading | > 60% likelihood | -30% | "Wash trading suspected" |
| Wash trading | > 30% likelihood | -10% | "Minor wash trading signals" |
| Low liquidity | < 5 SOL reserves | -20% | "Low liquidity: X SOL" |
| Low market cap | < 10 SOL | -15% | "Low market cap" |
| Top 10 concentration | > 80% | -25% | Whale risk |
| Top 10 concentration | > 70% | -15% | Concentration risk |
| Low activity | < 0.5 SOL vol + < 5 trades | -20% | "Very low activity" |
| Stalled graduation | > 98% + < 2 SOL vol | -15% | "Near graduation but low volume" |
| Creator holdings | > 20% | -20% | "Creator holds significant supply" |

---

## Minimum Thresholds (Hard Filters)

Tokens are rejected if they fail ANY of these:

| Metric | Minimum | Maximum |
|--------|---------|---------|
| Graduation progress | 70% | 99% |
| Volume 1h | 1 SOL | - |
| Holder count | 50 | - |
| Top 10 concentration | - | 70% |
| Creator holdings | - | 15% |
| Wash trade likelihood | - | 60% |
| Unique buyers 1h | 5 | - |

---

## Position Sizing

### Dynamic Sizing
- **Base**: 3% of wallet balance per trade
- **Max position**: Configurable cap (default: 0.3 SOL)
- **Max concurrent**: Limits simultaneous open positions (default: 10)
- **Cooldown**: 300 seconds between trades on same token

### Size Adjustments
- Score 80+ (StrongBuy): Full position size
- Score 60-79 (Buy): 75% position size
- High volatility: Reduced size
- Low liquidity: Reduced size to limit price impact

---

## Exit Strategy (Tiered Profit-Taking)

### Phase 1: Capital Recovery (100% gain / 2x)
| Parameter | Value | Purpose |
|-----------|-------|---------|
| Target | +100% (2x) | Double your money |
| Exit Size | 50% of position | Recover initial capital |
| Result | Break-even secured, 50% "free" tokens remain |

### Phase 2: Pre-Migration Lock (150% gain)
| Parameter | Value | Purpose |
|-----------|-------|---------|
| Target | +150% | Near bonding curve completion |
| Exit Size | 25% of position | Lock additional profits |
| Result | 25% "moon bag" remains for potential graduation |

### Phase 3: Moon Bag Management
The remaining 25% position is managed by:
1. **Extended Take Profit**: +300% target (if strong momentum)
2. **Trailing Stop**: 20% from peak price
3. **Time Limit**: 15 minutes max hold time

---

## Risk Management

### Stop Loss
| Parameter | Value | Notes |
|-----------|-------|-------|
| Stop Loss | -30% | Allows bonding curve volatility |
| Activation | Immediate | Triggers full exit |

### Trailing Stop
| Parameter | Value | Notes |
|-----------|-------|-------|
| Trail Percent | 20% | Distance from high water mark |
| Activation | Only when profitable | Protects gains, not losses |

### Time-Based Exit
| Parameter | Value | Notes |
|-----------|-------|-------|
| Time Limit | 15 minutes | Prevents stuck capital |
| Activation | After time elapsed | Full exit regardless of P&L |

### Daily Limits
| Parameter | Value | Notes |
|-----------|-------|-------|
| Daily Loss Limit | 1.0 SOL (medium) | Pauses trading if exceeded |
| Max Drawdown | 30% | Per-position max loss |

---

## Momentum-Adaptive Logic

### Momentum Tracking
The system tracks price momentum in real-time:

```
Velocity = (Price Change %) / Time (minutes)
Momentum Score = Acceleration (change in velocity)
```

### Momentum Strength Classification

| Strength | Velocity | Momentum Score | Behavior |
|----------|----------|----------------|----------|
| **Strong** | > 2.0%/min | > 30 | Extend targets +50%, sell less (0.5x) |
| **Normal** | 0.3-2.0%/min | 5-30 | Use base targets |
| **Weak** | < 0.3%/min | < 5 | Reduce targets -30%, sell more (1.3x) |
| **Reversing** | < -0.5%/min | < -30 | Immediate exit (1.5x sell size) |

### Momentum Reversal Exit
Triggers immediate 100% exit when ALL conditions met:
1. Velocity < -0.5%/min (price falling)
2. Momentum Score < -30 (acceleration negative)
3. 4+ consecutive negative readings (sustained reversal)
4. Current P&L > 5% (only protects actual profits)

### Adaptive Target Adjustment

**Strong Momentum Example:**
- Base first target: 100% → Adjusted: 150% (extended by 50%)
- Base exit size: 50% → Adjusted: 25% (sell less, let it run)

**Weak Momentum Example:**
- Base first target: 100% → Adjusted: 70% (reduced by 30%)
- Base exit size: 50% → Adjusted: 65% (sell more, take profits early)

---

## Exit Priority Order

The system checks exit conditions in this order (first match wins):

1. **Stop Loss** (-30%) → Full exit, Critical urgency
2. **Time Limit** (15 min) → Full exit, High urgency
3. **Momentum Reversal** (falling + profitable) → Full exit, Critical urgency
4. **Adaptive Partial #1** (~100% adjusted) → Partial exit
5. **Adaptive Partial #2** (~150% adjusted) → Partial exit
6. **Extended Take Profit** (300% + strong momentum) → Full exit
7. **Standard Partial #1** (100%) → Partial exit
8. **Standard Partial #2** (150%) → Partial exit
9. **Full Take Profit** (100%) → Full exit
10. **Trailing Stop** (20% from peak, profitable only) → Full exit

---

## Slippage Management

### Profit-Aware Slippage
Slippage tolerance scales with profitability:

```
If Profitable:
  Slippage = (P&L% × 25%) × 100 bps
  Example: 50% profit → 12.5% slippage tolerance

If Losing/Break-even:
  Slippage = 5% minimum floor
```

### Urgency Multipliers
| Urgency | Multiplier | Use Case |
|---------|------------|----------|
| Normal | 1.0x | Standard exits |
| High | 1.25x | Take profits, trailing stops |
| Critical | 1.5x | Stop loss, momentum reversal |

### Slippage Bounds
- **Minimum**: 5% (500 bps) - pump.fun curves move 10-20% in seconds
- **Maximum**: 20% (2000 bps) - prioritize execution over retention
- **Emergency**: 25% (2500 bps) - for failed retry attempts

---

## Configuration Reference

### Risk Levels (Presets)

#### Low Risk
```json
{
  "max_position_sol": 0.02,
  "max_concurrent_positions": 2,
  "stop_loss_pct": 15,
  "take_profit_pct": 10,
  "trailing_stop_pct": 8,
  "time_limit_minutes": 5,
  "daily_loss_limit_sol": 0.1
}
```

#### Medium Risk (Default)
```json
{
  "max_position_sol": 0.3,
  "max_concurrent_positions": 10,
  "stop_loss_pct": 30,
  "take_profit_pct": 100,
  "trailing_stop_pct": 20,
  "time_limit_minutes": 15,
  "daily_loss_limit_sol": 1.0
}
```

#### Aggressive Risk
```json
{
  "max_position_sol": 10.0,
  "max_concurrent_positions": 20,
  "stop_loss_pct": 30,
  "take_profit_pct": 100,
  "trailing_stop_pct": 20,
  "time_limit_minutes": 15,
  "daily_loss_limit_sol": 5.0
}
```

### Tiered Exit Configuration
```json
{
  "partial_take_profit": {
    "first_target_percent": 100.0,
    "first_exit_percent": 50.0,
    "second_target_percent": 150.0,
    "second_exit_percent": 25.0
  },
  "adaptive_partial_tp": {
    "first_target_percent": 100.0,
    "first_exit_percent": 50.0,
    "second_target_percent": 150.0,
    "second_exit_percent": 25.0,
    "third_target_percent": 300.0,
    "third_exit_percent": 100.0,
    "enable_extended_targets": true
  }
}
```

### Momentum Configuration
```json
{
  "momentum_adaptive": {
    "strong_velocity_threshold": 2.0,
    "weak_velocity_threshold": 0.3,
    "reversal_velocity_threshold": -0.5,
    "strong_momentum_score": 30.0,
    "weak_momentum_score": 5.0,
    "reversal_momentum_score": -30.0,
    "strong_exit_multiplier": 0.5,
    "normal_exit_multiplier": 1.0,
    "weak_exit_multiplier": 1.3,
    "reversing_exit_multiplier": 1.5,
    "strong_target_extension_percent": 50.0,
    "weak_target_reduction_percent": 30.0,
    "reversal_confirmation_count": 4,
    "reversal_immediate_exit": true,
    "min_profit_for_momentum_exit": 5.0
  }
}
```

---

## Transaction Execution

### Buy Flow
1. Signal detected → Edge created
2. Position sizing calculated (3% of wallet, max 0.3 SOL)
3. Build pump.fun buy transaction
4. Sign via Turnkey wallet
5. Submit via Helius RPC
6. Create position tracker with exit config

### Sell Flow
1. Exit condition triggered → Exit signal created
2. Calculate profit-aware slippage
3. Build pump.fun sell transaction (or Jupiter if graduated)
4. Sign via Turnkey wallet
5. Submit via Jito bundle (or Helius fallback)
6. Update position (partial) or close (full)
7. Save transaction to engrams for learning

### Retry Logic
- **Max retries**: 3 attempts
- **Slippage escalation**: Jump to emergency (25%) after first failure
- **Priority queue**: Failed exits get high-priority retry
- **Dead token detection**: Skip tokens with zero liquidity

---

## Key Metrics Tracked

| Metric | Purpose |
|--------|---------|
| Entry Price | Calculate P&L |
| Current Price | Real-time from curve state |
| High Water Mark | Trailing stop reference |
| Unrealized P&L % | Trigger exit conditions |
| Velocity (%/min) | Momentum direction |
| Momentum Score | Acceleration/deceleration |
| Consecutive Negatives | Reversal confirmation |
| Partial Exits | Track which phases completed |
| Remaining Amount | Accurate P&L after partials |

---

## Example Trade Lifecycle

```
1. [00:00] Signal: New token ABC detected
2. [00:01] Entry: Buy 0.3 SOL worth at price 0.0001
3. [00:02] Monitoring: Price updates every 1-2 seconds
4. [00:05] +100% gain: Sell 50% (0.15 SOL) → Recovered capital
5. [00:08] +150% gain: Sell 25% (0.075 SOL) → Locked profits
6. [00:12] Price peaks at +200%, starts falling
7. [00:13] Trailing stop: 20% from peak → Sell remaining 25%
8. [00:13] Position closed: Total P&L = +0.375 SOL (+125%)
```

---

## Tuning Recommendations

### For More Conservative Trading
- Decrease `time_limit_minutes` to 5-10
- Increase `stop_loss_pct` to 20% (tighter)
- Decrease `first_target_percent` to 50-75%
- Increase `first_exit_percent` to 60-70%

### For More Aggressive Trading
- Increase `time_limit_minutes` to 20-30
- Keep `stop_loss_pct` at 30% (allow volatility)
- Increase `third_target_percent` to 500%
- Decrease exit percentages to ride winners longer

### For High-Volume Markets
- Decrease `reversal_confirmation_count` to 2-3
- Increase `reversal_velocity_threshold` to -1.0
- Tighten trailing stop to 15%

### For Low-Volume Markets
- Increase `reversal_confirmation_count` to 5-6
- Decrease slippage bounds
- Extend time limits
