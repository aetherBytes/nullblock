#!/bin/bash
# ArbFarm Health Monitor - sets risk MEDIUM by default

echo "⏳ Waiting for ArbFarm..."

# Wait for service to be available
while ! curl -s http://localhost:9007/health > /dev/null 2>&1; do
  echo "Waiting for ArbFarm service..."
  sleep 3
done

echo "✅ Service is up!"
sleep 2

echo "🔒 Setting risk level to MEDIUM..."
curl -s -X POST http://localhost:9007/config/risk \
  -H "Content-Type: application/json" \
  -d '{"level": "medium"}' | jq -r '"Risk set: max_position=\(.params.max_position_sol) SOL, max_concurrent=\(.params.max_concurrent_positions)"' || echo "Failed to set risk"
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
  echo "🔒 Risk Config"
  curl -s http://localhost:9007/config/risk 2>/dev/null | jq -r '"Max: \(.max_position_sol) SOL | Concurrent: \(.max_concurrent_positions) | SL: \(.max_drawdown_percent)%"' || true
  sleep 5
done
