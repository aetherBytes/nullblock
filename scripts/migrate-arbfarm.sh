#!/bin/bash
# ArbFarm Database Migration Script

set -e

MIGRATIONS_DIR="$HOME/nullblock/svc/arb-farm/migrations"
DB_CONTAINER="nullblock-postgres-agents"
DB_NAME="agents"
DB_USER="postgres"

echo "🗄️ Running ArbFarm migrations..."

# Check if container is running
if ! docker ps --format '{{.Names}}' | grep -q "^${DB_CONTAINER}$"; then
    echo "❌ Database container ${DB_CONTAINER} not running. Start with 'just start' first."
    exit 1
fi

# Run each migration file
for migration in "$MIGRATIONS_DIR"/*.sql; do
    if [ -f "$migration" ]; then
        filename=$(basename "$migration")
        echo "  📄 Running $filename..."
        docker exec -i "$DB_CONTAINER" psql -U "$DB_USER" -d "$DB_NAME" < "$migration" 2>&1 | grep -v "already exists" || true
    fi
done

echo "✅ ArbFarm migrations complete"

# Verify tables exist
echo ""
echo "📊 Verifying ArbFarm tables..."
docker exec "$DB_CONTAINER" psql -U "$DB_USER" -d "$DB_NAME" -c "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public' AND table_name LIKE 'arb_%'" | grep arb_ | wc -l | xargs echo "   Found" && echo "arb_* tables"
