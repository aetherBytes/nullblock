#!/bin/bash
# Run Agents database migrations

echo "=========================================="
echo "🔄 Running Agents Database Migrations"
echo "=========================================="

# Check if Agents database is accessible
if ! docker exec nullblock-postgres-agents pg_isready -U postgres > /dev/null 2>&1; then
    echo "❌ Agents database not accessible"
    echo "   Start it with: docker-compose up -d postgres-agents"
    exit 1
fi

echo "✅ Database connection OK"
echo ""

# Run each migration file by piping content (in order)
migrations=(
    "svc/nullblock-agents/migrations/001_create_tasks_table.sql"
    "svc/nullblock-agents/migrations/002_create_agents_table.sql"
    "svc/nullblock-agents/migrations/003_create_user_references_table.sql"
    "svc/nullblock-agents/migrations/004_add_tasks_foreign_keys.sql"
    "svc/nullblock-agents/migrations/006_fix_user_references_constraint_name.sql"
    "svc/nullblock-agents/migrations/005_setup_replication_subscription.sql"
)

migration_num=0
for migration in "${migrations[@]}"; do
    if [ -f "$migration" ]; then
        migration_num=$((migration_num + 1))
        echo "📄 Migration $migration_num: $(basename "$migration")"
        cat "$migration" | docker exec -i nullblock-postgres-agents psql -U postgres -d agents 2>&1 | \
            grep -E "(ERROR|NOTICE|Created|already exists|ready|synced)" | head -10 || echo "   ✅ Applied"
        echo ""
    fi
done

echo "=========================================="
echo "✅ Agents Migrations Complete ($migration_num files)"
echo "=========================================="

# Show status
echo ""
echo "Database Status:"
docker exec -i nullblock-postgres-agents psql -U postgres -d agents <<-EOSQL
    SELECT COUNT(*) as tables FROM information_schema.tables WHERE table_schema = 'public' AND table_type = 'BASE TABLE';
    SELECT COUNT(*) as subscriptions FROM pg_subscription WHERE subname = 'agents_user_sync';
    SELECT COUNT(*) as synced_users FROM user_references WHERE is_active = true;
    SELECT subenabled as subscription_enabled, srsubstate as replication_state FROM pg_subscription sub LEFT JOIN pg_subscription_rel rel ON sub.oid = rel.srsubid WHERE sub.subname = 'agents_user_sync';
EOSQL
echo ""
















