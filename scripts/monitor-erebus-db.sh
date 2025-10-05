#!/bin/bash
echo "🗄️  Monitoring PostgreSQL for Erebus (Users DB)..."
echo "Using Docker PostgreSQL container on port 5440..."
echo ""
echo "📊 Database connection monitoring..."
while true; do
  if docker ps | grep -q nullblock-postgres-erebus && docker exec nullblock-postgres-erebus pg_isready -U postgres > /dev/null 2>&1; then
    echo "$(date '+%H:%M:%S') ✅ Erebus PostgreSQL (Docker) connection healthy"
  else
    echo "$(date '+%H:%M:%S') ❌ Erebus PostgreSQL (Docker) connection failed"
  fi
  sleep 30
done
