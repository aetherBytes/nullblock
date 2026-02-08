#!/usr/bin/env bash
set -e

echo "📝 Starting NullBlock Content Service (Rust)..."
echo "Social media content generation and posting service"
echo ""

cd ~/nullblock/svc/nullblock-content
mkdir -p logs

if [ -f ../../.env.dev ]; then
  echo "🔐 Loading environment variables from .env.dev..."
  set -a
  source ../../.env.dev
  set +a
fi

export PORT=8002
export DATABASE_URL="${DATABASE_URL:-postgresql://postgres:${POSTGRES_PASSWORD:-changeme}@localhost:5442/nullblock_content}"
echo "🚀 Starting Content service on port 8002..."
echo "📝 Logs will be written to logs/content.log"
echo "🗄️  Using PostgreSQL for content persistence"
echo ""
echo "🗄️  Waiting for database to be ready..."
while ! docker exec nullblock-postgres-content pg_isready -U postgres > /dev/null 2>&1; do
  echo "⏳ Waiting for PostgreSQL container to be ready..."
  sleep 2
done
echo "✅ Database connection ready"
echo ""
echo "📋 Running content service migrations..."
~/nullblock/scripts/migrate-content.sh || echo "⚠️  Migration warnings (may be expected)"
echo "✅ Content database schema ready"
echo ""
echo "🚀 Starting Content service..."
cargo run --release 2>&1 | tee logs/content.log
