#!/bin/bash
# ArbFarm Health Monitor - displays status only (risk config set dynamically at startup)

echo "⏳ Waiting for ArbFarm..."

# Wait for service to be available
while ! curl -s http://localhost:9007/health > /dev/null 2>&1; do
  echo "Waiting for ArbFarm service..."
  sleep 3
done

echo "✅ Service is up!"
sleep 2

# Note: Risk config is now set dynamically at startup based on wallet balance (1/15th, max 10 SOL)
echo "📊 Risk config is set automatically based on wallet balance"
curl -s http://localhost:9007/config/risk 2>/dev/null | jq -r '"Current: max_position=\(.max_position_sol) SOL, max_concurrent=\(.max_concurrent_positions)"' || echo "Failed to get risk"
echo ""
sleep 2

while true; do
  clear
  RISK_LEVEL=$(curl -s http://localhost:9007/config/risk 2>/dev/null | jq -r '.level // "unknown"' | tr '[:lower:]' '[:upper:]')
  echo "⚡ ArbFarm Status - $(date '+%H:%M:%S') [RISK: $RISK_LEVEL]"
  echo "================================"
  curl -s http://localhost:9007/health 2>/dev/null | jq -r '"Service: \(.status)"' || echo "❌ Not responding"
  echo ""
  curl -s http://localhost:9007/scanner/status 2>/dev/null | jq -r '"Scanner: Running=\(.is_running) | Signals=\(.stats.total_signals)"' || true
  curl -s http://localhost:9007/executor/stats 2>/dev/null | jq -r '"Executor: Running=\(.is_running) | Executed=\(.executions_succeeded)"' || true
  curl -s http://localhost:9007/positions/monitor/status 2>/dev/null | jq -r '"Monitor: Positions=\(.active_positions)"' || true
  echo ""
  echo "💰 Wallet"
  curl -s http://localhost:9007/wallet/balance 2>/dev/null | jq -r '"Balance: \(.balance_sol) SOL"' || echo "Unknown"
  echo ""
  echo "🔒 Strategy Configs"
  curl -s http://localhost:9007/strategies 2>/dev/null | jq -r '
    .strategies[]? |
    select(.strategy_type == "curve_arb" or .strategy_type == "graduation_snipe") |
    (if .strategy_type == "curve_arb" then "🔍 Scanner" else "🔫 Sniper " end) + ": " +
    "SL:\(.risk_params.stop_loss_percent // 0)% TP:\(.risk_params.take_profit_percent // 0)% Trail:\(.risk_params.trailing_stop_percent // 0)% Time:\(.risk_params.time_limit_minutes // 0)m"
  ' || echo "Failed to get strategies"
  sleep 5
done
