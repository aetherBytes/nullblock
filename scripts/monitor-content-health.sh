#!/usr/bin/env bash

echo "🏥 Content Service Health Monitor"
echo "================================="
echo ""

while true; do
  clear
  echo "🏥 Content Service Health Monitor - $(date '+%H:%M:%S')"
  echo "================================="
  echo ""

  if curl -s --max-time 2 http://localhost:8002/health > /dev/null 2>&1; then
    echo "✅ Service: HEALTHY"
    curl -s http://localhost:8002/health | python3 -m json.tool 2>/dev/null || echo "Service responding"
  else
    echo "❌ Service: DOWN"
  fi

  echo ""
  echo "📊 Queue Status:"
  curl -s --max-time 2 "http://localhost:8002/api/content/queue?status=pending" 2>/dev/null | python3 -c "import sys,json; data=json.load(sys.stdin); print(f'   Pending: {data.get(\"total\", 0)}')" 2>/dev/null || echo "   Unable to fetch"

  echo ""
  echo "Press Ctrl+C to exit"
  sleep 5
done
