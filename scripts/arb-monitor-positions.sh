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

# Get strategy IDs for lookup
get_strategy_type() {
  local strat_id="$1"
  local signal_src="$2"

  # Check signal_source first (most reliable for sniper)
  if [ "$signal_src" = "graduation_sniper" ]; then
    echo "🔫"
    return
  fi

  # Check for nil UUID (wallet discovery)
  if [ "$strat_id" = "00000000-0000-0000-0000-000000000000" ]; then
    echo "📦"
    return
  fi

  # Otherwise it's a scanner position
  echo "🔍"
}

while true; do
  clear
  echo "📊 Open Positions - $(date '+%H:%M:%S')"
  echo "Legend: 🔍=Scanner 🔫=Sniper 📦=Discovered"
  echo "================================"
  positions=$(curl -s http://localhost:9007/positions 2>/dev/null)
  if [ -n "$positions" ]; then
    # Show positions with strategy indicator, exit config, and momentum
    echo "$positions" | jq -r '
      .positions[]? |
      (if .signal_source == "graduation_sniper" then "🔫"
       elif .strategy_id == "00000000-0000-0000-0000-000000000000" then "📦"
       else "🔍" end) as $icon |
      (.momentum.momentum_score // 0 | . * 10 | floor / 10) as $mom |
      (if $mom >= 30 then "🚀" elif $mom >= 0 then "📈" elif $mom >= -30 then "📉" else "💀" end) as $mom_icon |
      "\($icon) [\(.status)] \(.token_symbol // .token_mint[0:8]) | \(.entry_amount_base | . * 1000 | floor / 1000) SOL → \(.current_value_base | . * 1000 | floor / 1000) SOL | P&L: \(.unrealized_pnl_percent | . * 100 | floor / 100)% | Mom:\($mom)\($mom_icon) | TP:\(.exit_config.take_profit_percent // 0)% SL:\(.exit_config.stop_loss_percent // 0)%"
    ' 2>/dev/null || echo "No positions"
    echo ""
    echo "📈 Summary"
    # Count by type
    scanner_count=$(echo "$positions" | jq '[.positions[]? | select(.signal_source != "graduation_sniper" and .strategy_id != "00000000-0000-0000-0000-000000000000")] | length')
    sniper_count=$(echo "$positions" | jq '[.positions[]? | select(.signal_source == "graduation_sniper")] | length')
    discovered_count=$(echo "$positions" | jq '[.positions[]? | select(.strategy_id == "00000000-0000-0000-0000-000000000000" and .signal_source != "graduation_sniper")] | length')
    echo "🔍 Scanner: $scanner_count | 🔫 Sniper: $sniper_count | 📦 Discovered: $discovered_count"
    echo "$positions" | jq -r '"Realized P&L: \(.stats.total_realized_pnl | . * 1000 | floor / 1000) SOL"' 2>/dev/null || true
  else
    echo "No positions"
  fi
  sleep 8
done
