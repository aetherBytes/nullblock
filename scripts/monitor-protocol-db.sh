#!/bin/bash
echo "🗄️  Protocol Service Database Integration Monitoring"
echo "Monitoring Agents database connection for protocol service integration..."
echo ""
mkdir -p ~/nullblock/svc/nullblock-protocols/logs
while true; do
  echo "$(date '+%Y-%m-%d %H:%M:%S') [AGENTS-DB] Database status..." | tee -a ~/nullblock/svc/nullblock-protocols/logs/agents-db.log
  if docker exec nullblock-postgres-agents pg_isready -U postgres > /dev/null 2>&1; then
    echo "✅ PostgreSQL connection healthy" | tee -a ~/nullblock/svc/nullblock-protocols/logs/agents-db.log
  else
    echo "❌ PostgreSQL connection failed" | tee -a ~/nullblock/svc/nullblock-protocols/logs/agents-db.log
  fi
  sleep 45
done
