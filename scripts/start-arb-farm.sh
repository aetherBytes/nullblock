#!/usr/bin/env bash
set -e

echo "🌾 Starting ArbFarm MEV Agent Swarm..."

# Kill any existing arb-farm process (debug or release)
if pgrep -f "arb-farm" > /dev/null 2>&1; then
  echo "🔪 Killing existing arb-farm process..."
  pkill -f "target/debug/arb-farm" || true
  pkill -f "target/release/arb-farm" || true
  sleep 2
fi

cd ~/nullblock/svc/arb-farm
mkdir -p logs

if [ -f ../../.env.dev ]; then
  echo "🔐 Loading environment variables from root .env.dev..."
  set -a
  source ../../.env.dev
  set +a
fi

if [ -f .env.dev ]; then
  echo "🔐 Loading environment variables from arb-farm .env.dev..."
  set -a
  source .env.dev
  set +a
fi

# Check for no-scan mode (set by just dev-mac no-scan)
if [ -f /tmp/arb-no-scan ]; then
  echo "🚫 No-scan mode enabled (scanner will not auto-start)"
  export ARB_SCANNER_AUTO_START=false
  rm /tmp/arb-no-scan  # Clean up flag file
fi

export DATABASE_URL="${DATABASE_URL:-postgresql://postgres:${POSTGRES_PASSWORD:-changeme}@localhost:5441/agents}"
export ARB_FARM_PORT=9007
echo "📝 Logs will be written to logs/arb-farm.log and /tmp/arb-farm.log"
echo ""

echo "🗄️ Waiting for database to be ready..."
while ! docker exec nullblock-postgres-agents pg_isready -U postgres > /dev/null 2>&1; do
  echo "⏳ Waiting for PostgreSQL container to be ready..."
  sleep 2
done
echo "✅ Database connection ready"
echo ""

echo "🚀 Starting ArbFarm server (release build)..."
# Use tee without -a to truncate log on each start
# Using --release for optimized build with all fixes
cargo run --release 2>&1 | tee logs/arb-farm.log | tee /tmp/arb-farm.log
