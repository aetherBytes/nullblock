#!/usr/bin/env bash
set -e

echo "🌾 Starting ArbFarm MEV Agent Swarm..."

# Kill any existing arb-farm process
if pgrep -f "target/debug/arb-farm" > /dev/null 2>&1; then
  echo "🔪 Killing existing arb-farm process..."
  pkill -f "target/debug/arb-farm" || true
  sleep 2
fi

cd ~/nullblock/svc/arb-farm
mkdir -p logs

if [ -f ../../.env.dev ]; then
  echo "🔐 Loading environment variables from .env.dev..."
  set -a
  source ../../.env.dev
  set +a
fi

echo "📝 Logs will be written to logs/arb-farm.log and /tmp/arb-farm.log"
echo ""

echo "🗄️ Waiting for database to be ready..."
while ! docker exec nullblock-postgres-agents pg_isready -U postgres > /dev/null 2>&1; do
  echo "⏳ Waiting for PostgreSQL container to be ready..."
  sleep 2
done
echo "✅ Database connection ready"
echo ""

echo "🚀 Starting ArbFarm server..."
# Use tee without -a to truncate log on each start
cargo run 2>&1 | tee logs/arb-farm.log | tee /tmp/arb-farm.log
