#!/bin/bash
# Run Agents database migrations

echo "🔄 Running Agents database migrations..."

# Check if Agents database is accessible
if ! docker exec nullblock-postgres-agents pg_isready -U postgres > /dev/null 2>&1; then
    echo "❌ Agents database not accessible"
    exit 1
fi

# Run each migration file
for migration in svc/nullblock-agents/migrations/*.sql; do
    if [ -f "$migration" ]; then
        echo "  📄 Applying $(basename "$migration")..."
        docker exec nullblock-postgres-agents psql -U postgres -d agents -f "/tmp/$(basename "$migration")" 2>/dev/null || echo "    ⚠️  Migration already applied or failed"
    fi
done

echo "✅ Agents migrations completed"

