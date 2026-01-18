#!/bin/bash
# ArbFarm Position & P&L Monitor

echo "📊 Positions & P&L"
echo "Waiting for service..."

# Wait for service to be available
while ! curl -s http://localhost:9007/health > /dev/null 2>&1; do
  sleep 3
done

echo "✅ Service is up!"
sleep 2

while true; do
  clear
  echo "📊 Open Positions - $(date '+%H:%M:%S')"
  echo "================================"
  positions=$(curl -s http://localhost:9007/positions 2>/dev/null)
  if [ -n "$positions" ]; then
    echo "$positions" | jq -r '.positions[]? | "[\(.status)] \(.token_symbol // .token_mint[0:8]) | Entry: \(.entry_amount_base | . * 1000 | floor / 1000) SOL | Now: \(.current_value_base | . * 1000 | floor / 1000) SOL | P&L: \(.unrealized_pnl | . * 10000 | floor / 10000) SOL (\(.unrealized_pnl_percent | . * 100 | floor / 100)%)"' 2>/dev/null || echo "No positions"
    echo ""
    echo "📈 Summary"
    echo "$positions" | jq -r '"Total: \(.positions | length) | Realized P&L: \(.stats.total_realized_pnl | . * 1000 | floor / 1000) SOL"' 2>/dev/null || true
  else
    echo "No positions"
  fi
  sleep 8
done
