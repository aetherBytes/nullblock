#!/bin/bash
# NullBlock Content Service Database Migration Script

set -e

MIGRATIONS_DIR="$HOME/nullblock/svc/nullblock-content/migrations"
DB_CONTAINER="nullblock-postgres-content"
DB_NAME="nullblock_content"
DB_USER="postgres"

echo "🗄️ Running Content Service migrations..."

if ! docker ps --format '{{.Names}}' | grep -q "^${DB_CONTAINER}$"; then
    echo "❌ Database container ${DB_CONTAINER} not running. Start with 'just start' first."
    exit 1
fi

for migration in "$MIGRATIONS_DIR"/*.sql; do
    if [ -f "$migration" ]; then
        filename=$(basename "$migration")
        echo "  📄 Running $filename..."
        docker exec -i "$DB_CONTAINER" psql -U "$DB_USER" -d "$DB_NAME" < "$migration" 2>&1 | grep -v "already exists" || true
    fi
done

echo "✅ Content Service migrations complete"

echo ""
echo "📊 Verifying Content tables..."
docker exec "$DB_CONTAINER" psql -U "$DB_USER" -d "$DB_NAME" -c "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public' AND table_name LIKE 'content_%'" | grep content_ | wc -l | xargs echo "   Found" && echo "content_* tables"
