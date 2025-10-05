#!/bin/bash
echo "🔄 Monitoring Redis for Erebus (Sessions)..."
echo "Using Docker Redis container on port 6379..."
echo ""
echo "📊 Redis connection monitoring..."
while true; do
  if docker ps | grep -q nullblock-redis && docker exec nullblock-redis redis-cli ping > /dev/null 2>&1; then
    echo "$(date '+%H:%M:%S') ✅ Redis (Docker) connection healthy"
  else
    echo "$(date '+%H:%M:%S') ❌ Redis (Docker) connection failed"
  fi
  sleep 30
done
