#!/bin/bash
# Run Erebus database migrations

echo "🔄 Running Erebus database migrations..."

# Check if Erebus database is accessible
if ! docker exec nullblock-postgres-erebus pg_isready -U postgres > /dev/null 2>&1; then
    echo "❌ Erebus database not accessible"
    exit 1
fi

# Run each migration file by piping content
for migration in svc/erebus/migrations/*.sql; do
    if [ -f "$migration" ]; then
        echo "  📄 Applying $(basename "$migration")..."
        cat "$migration" | docker exec -i nullblock-postgres-erebus psql -U postgres -d erebus 2>&1 | grep -E "(ERROR|NOTICE|already exists)" || echo "    ✅ Applied successfully"
    fi
done

echo "✅ Erebus migrations completed"














