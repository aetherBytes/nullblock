#!/bin/bash
# Run Erebus database migrations

echo "=========================================="
echo "🔄 Running Erebus Database Migrations"
echo "=========================================="

# Check if Erebus database is accessible
if ! docker exec nullblock-postgres-erebus pg_isready -U postgres > /dev/null 2>&1; then
    echo "❌ Erebus database not accessible"
    echo "   Start it with: docker-compose up -d postgres-erebus"
    exit 1
fi

echo "✅ Database connection OK"
echo ""

# Run each migration file by piping content
migration_count=0
for migration in svc/erebus/migrations/*.sql; do
    if [ -f "$migration" ]; then
        migration_count=$((migration_count + 1))
        echo "📄 Migration $migration_count: $(basename "$migration")"
        cat "$migration" | docker exec -i nullblock-postgres-erebus psql -U postgres -d erebus 2>&1 | \
            grep -E "(ERROR|NOTICE|Created|already exists)" | head -10 || echo "   ✅ Applied"
        echo ""
    fi
done

echo "=========================================="
echo "✅ Erebus Migrations Complete ($migration_count files)"
echo "=========================================="

# Show status
echo ""
echo "Database Status:"
docker exec -i nullblock-postgres-erebus psql -U postgres -d erebus <<-EOSQL
    SELECT COUNT(*) as tables FROM information_schema.tables WHERE table_schema = 'public' AND table_type = 'BASE TABLE';
    SELECT COUNT(*) as publications FROM pg_publication WHERE pubname = 'erebus_user_sync';
    SELECT COUNT(*) as users FROM user_references WHERE is_active = true;
EOSQL
echo ""














