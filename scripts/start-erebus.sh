#!/usr/bin/env bash
set -e

echo "🔗 Starting Erebus Server (Rust)..."
cd svc/erebus
mkdir -p logs

if [ -f ../../.env.dev ]; then
  echo "🔐 Loading environment variables from .env.dev..."
  set -a
  source ../../.env.dev
  set +a
else
  echo "⚠️  Warning: .env.dev file not found - ENCRYPTION_MASTER_KEY may not be set"
fi

export EREBUS_HOST=127.0.0.1
export EREBUS_PORT=3000
export DATABASE_URL="postgresql://postgres:REDACTED_DB_PASS@localhost:5440/erebus"
export KAFKA_BOOTSTRAP_SERVERS="localhost:9092"
echo "📝 Logs will be written to logs/erebus.log"
echo ""
echo "🗄️  Waiting for Erebus database to be ready..."
while ! docker exec nullblock-postgres-erebus pg_isready -U postgres > /dev/null 2>&1; do
  echo "⏳ Waiting for Erebus PostgreSQL container to be ready..."
  sleep 2
done
echo "✅ Erebus database connection ready"
echo ""
echo "📋 Running Erebus database migrations..."
~/nullblock/scripts/run-erebus-migrations.sh
echo "✅ Erebus migrations complete"
echo ""
echo "📨 Waiting for Kafka to be ready..."
while ! docker exec nullblock-kafka kafka-broker-api-versions --bootstrap-server localhost:9092 > /dev/null 2>&1; do
  echo "⏳ Waiting for Kafka container to be ready..."
  sleep 3
done
echo "✅ Kafka connection ready"
echo ""
echo "🚀 Starting Erebus Rust server..."
cargo run 2>&1 | tee logs/erebus.log
