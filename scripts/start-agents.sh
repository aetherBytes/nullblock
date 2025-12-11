#!/bin/bash
# Start Hecate Agent Server (Rust) with database setup

echo "🎯 Starting Hecate Agent Server (Rust)..."
echo "🦀 High-performance Rust implementation with database & events"
echo ""
cd ~/nullblock/svc/nullblock-agents
mkdir -p logs

if [ -f ../../.env.dev ]; then
  echo "🔐 Loading environment variables from .env.dev..."
  set -a
  source ../../.env.dev
  set +a
else
  echo "⚠️  Warning: .env.dev file not found"
fi

export AGENTS_PORT=9003
export DATABASE_URL="postgresql://postgres:postgres_secure_pass@localhost:5441/agents"
export KAFKA_BOOTSTRAP_SERVERS="localhost:9092"
echo "🚀 Starting Rust agents service on port 9003..."
echo "📝 Logs will be written to logs/hecate-rust.log"
echo "🗄️  Using PostgreSQL for task persistence"
echo "📨 Using Kafka for event streaming"
echo ""

echo "🗄️  Setting up database schema..."
echo "📋 Waiting for database to be ready..."
while ! docker exec nullblock-postgres-agents pg_isready -U postgres > /dev/null 2>&1; do
  echo "⏳ Waiting for PostgreSQL container to be ready..."
  sleep 2
done
echo "✅ Database connection ready"
echo ""

echo "📋 Running database migrations..."
# Use the migration scripts directly instead of inline
~/nullblock/scripts/run-agents-migrations.sh
echo "✅ Database schema ready"
echo ""

echo "📨 Waiting for Kafka to be ready..."
while ! docker exec nullblock-kafka kafka-broker-api-versions --bootstrap-server localhost:9092 > /dev/null 2>&1; do
  echo "⏳ Waiting for Kafka container to be ready..."
  sleep 3
done
echo "✅ Kafka connection ready"
echo ""

echo "🚀 Starting Rust agents service..."
cargo run --release 2>&1 | tee logs/hecate-rust.log
