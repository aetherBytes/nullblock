#!/bin/bash
# Setup script for automatic curve execution with minimal risk
# This configures the ArbFarm to automatically buy graduation candidates

BASE_URL="${ARB_FARM_URL:-http://localhost:9007}"

echo "🌾 ArbFarm Auto Curve Setup"
echo "============================="
echo "Base URL: $BASE_URL"
echo ""

# Check health
echo "1. Checking service health..."
HEALTH=$(curl -s "$BASE_URL/health" | jq -r '.status // "error"')
if [ "$HEALTH" != "ok" ]; then
    echo "   ❌ Service not healthy. Start with: cargo run"
    exit 1
fi
echo "   ✅ Service is healthy"

# Check wallet status
echo ""
echo "2. Checking wallet status..."
WALLET_STATUS=$(curl -s "$BASE_URL/wallet/status")
HAS_SIGNER=$(echo "$WALLET_STATUS" | jq -r '.has_signer // false')
WALLET_ADDR=$(echo "$WALLET_STATUS" | jq -r '.wallet_address // "none"')

if [ "$HAS_SIGNER" != "true" ]; then
    echo "   ❌ No wallet signer configured!"
    echo "   Set ARB_FARM_WALLET_PRIVATE_KEY in .env.dev"
    exit 1
fi
echo "   ✅ Wallet: $WALLET_ADDR"

# Get balance
BALANCE=$(curl -s "$BASE_URL/wallet/balance" | jq -r '.balance_sol // 0')
echo "   💰 Balance: $BALANCE SOL"

if (( $(echo "$BALANCE < 0.1" | bc -l) )); then
    echo "   ⚠️  WARNING: Low balance. Consider adding more SOL for testing."
fi

# Get strategies
echo ""
echo "3. Getting current strategies..."
STRATEGIES=$(curl -s "$BASE_URL/strategies")
CURVE_STRATEGY=$(echo "$STRATEGIES" | jq '.strategies[] | select(.strategy_type == "curve_arb")')
CURVE_ID=$(echo "$CURVE_STRATEGY" | jq -r '.id')

if [ -z "$CURVE_ID" ] || [ "$CURVE_ID" == "null" ]; then
    echo "   ❌ No curve strategy found. Creating one..."
    # Create a new curve strategy
    RESULT=$(curl -s -X POST "$BASE_URL/strategies" \
        -H "Content-Type: application/json" \
        -d '{
            "name": "Auto Curve Graduation",
            "strategy_type": "curve_arb",
            "venue_types": ["bondingcurve", "BondingCurve"],
            "execution_mode": "autonomous",
            "risk_params": {
                "max_position_sol": 0.05,
                "daily_loss_limit_sol": 0.1,
                "min_profit_bps": 100,
                "max_slippage_bps": 200,
                "max_risk_score": 50,
                "require_simulation": true,
                "auto_execute_atomic": false,
                "auto_execute_enabled": true,
                "require_confirmation": false,
                "staleness_threshold_hours": 24,
                "stop_loss_percent": 15.0,
                "take_profit_percent": 50.0,
                "time_limit_minutes": 30,
                "base_currency": "sol",
                "max_capital_allocation_percent": 10.0,
                "concurrent_positions": 1
            }
        }')
    CURVE_ID=$(echo "$RESULT" | jq -r '.strategy.id // .id')
    echo "   ✅ Created strategy: $CURVE_ID"
else
    echo "   Found existing strategy: $CURVE_ID"
fi

# Configure for minimal risk autonomous mode
echo ""
echo "4. Configuring strategy for autonomous mode with minimal risk..."
curl -s -X PUT "$BASE_URL/strategies/$CURVE_ID" \
    -H "Content-Type: application/json" \
    -d '{
        "execution_mode": "autonomous",
        "risk_params": {
            "max_position_sol": 0.05,
            "daily_loss_limit_sol": 0.1,
            "min_profit_bps": 100,
            "max_slippage_bps": 200,
            "max_risk_score": 50,
            "require_simulation": true,
            "auto_execute_atomic": false,
            "auto_execute_enabled": true,
            "require_confirmation": false,
            "staleness_threshold_hours": 24,
            "stop_loss_percent": 15.0,
            "take_profit_percent": 50.0,
            "time_limit_minutes": 30,
            "base_currency": "sol",
            "max_capital_allocation_percent": 10.0,
            "concurrent_positions": 1
        }
    }' > /dev/null
echo "   ✅ Strategy configured: 0.05 SOL max position, 50 max risk score"

# Enable the strategy
echo ""
echo "5. Enabling strategy..."
curl -s -X POST "$BASE_URL/strategies/$CURVE_ID/toggle" \
    -H "Content-Type: application/json" \
    -d '{"enabled": true}' > /dev/null
echo "   ✅ Strategy enabled"

# Start scanner
echo ""
echo "6. Starting scanner..."
curl -s -X POST "$BASE_URL/scanner/start" > /dev/null
echo "   ✅ Scanner started"

# Start autonomous executor
echo ""
echo "7. Starting autonomous executor..."
curl -s -X POST "$BASE_URL/executor/start" > /dev/null
echo "   ✅ Autonomous executor started"

# Final status
echo ""
echo "============================="
echo "✅ SETUP COMPLETE"
echo "============================="
echo ""
echo "📊 CONFIGURATION:"
echo "   Strategy:        Auto Curve Graduation"
echo "   Mode:            🤖 AUTONOMOUS"
echo "   Max Position:    0.05 SOL"
echo "   Max Risk Score:  50"
echo "   Stop Loss:       15%"
echo "   Take Profit:     50%"
echo ""
echo "📋 MONITORING COMMANDS:"
echo "   Watch logs:      (already in tmuxinator pane)"
echo "   Check status:    curl $BASE_URL/executor/stats"
echo "   Check scanner:   curl $BASE_URL/scanner/status"
echo "   Check edges:     curl $BASE_URL/edges"
echo "   Check positions: curl $BASE_URL/positions"
echo ""
echo "🔍 USEFUL ENDPOINTS:"
echo "   Graduation candidates: curl $BASE_URL/curves/graduation-candidates"
echo "   Top opportunities:     curl $BASE_URL/curves/top-opportunities"
echo "   Process signals:       curl -X POST $BASE_URL/scanner/process"
echo ""
echo "The system will now automatically:"
echo "  1. Scan for graduation candidates every 5 seconds"
echo "  2. Create edges for candidates matching the strategy"
echo "  3. Auto-execute buys up to 0.05 SOL per trade"
echo ""
echo "Watch the logs for 🚀 (auto-execution) and ✅ (success) messages"
