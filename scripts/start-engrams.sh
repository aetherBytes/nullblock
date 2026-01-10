#!/usr/bin/env bash
set -e

echo "🧠 Starting Engram Service (Rust)..."
echo "Universal memory/context persistence layer for NullBlock"
echo ""

# Navigate to engrams service directory
cd ~/nullblock/svc/nullblock-engrams
mkdir -p logs

# Load environment variables
if [ -f ../../.env.dev ]; then
  echo "🔐 Loading environment variables from .env.dev..."
  set -a
  source ../../.env.dev
  set +a
fi

export ENGRAMS_PORT=9004
export DATABASE_URL="postgresql://postgres:REDACTED_DB_PASS@localhost:5441/agents"
echo "🚀 Starting Engram service on port 9004..."
echo "📝 Logs will be written to logs/engrams.log"
echo "🗄️  Using PostgreSQL for engram persistence"
echo ""
echo "🗄️  Waiting for database to be ready..."
while ! docker exec nullblock-postgres-agents pg_isready -U postgres > /dev/null 2>&1; do
  echo "⏳ Waiting for PostgreSQL container to be ready..."
  sleep 2
done
echo "✅ Database connection ready"
echo ""
echo "📋 Running engram migrations..."
cat ~/nullblock/svc/nullblock-engrams/migrations/001_create_engrams.sql | docker exec -i nullblock-postgres-agents psql -U postgres -d agents 2>&1 | grep -E "(ERROR|already exists)" || echo "✅ 001_create_engrams.sql applied"
echo "✅ Engram database schema ready"
echo ""
echo "🚀 Starting Engram service..."
cargo run 2>&1 | tee logs/engrams.log
