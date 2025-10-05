#!/bin/bash
# Run Agents database migrations

echo "🔄 Running Agents database migrations..."

# Check if Agents database is accessible
if ! docker exec nullblock-postgres-agents pg_isready -U postgres > /dev/null 2>&1; then
    echo "❌ Agents database not accessible"
    exit 1
fi

# Run each migration file by piping content (in order)
migrations=(
    "svc/nullblock-agents/migrations/001_create_tasks_table.sql"
    "svc/nullblock-agents/migrations/002_create_agents_table.sql"
    "svc/nullblock-agents/migrations/003_create_user_references_table.sql"
    "svc/nullblock-agents/migrations/004_add_tasks_foreign_keys.sql"
)

for migration in "${migrations[@]}"; do
    if [ -f "$migration" ]; then
        echo "  📄 Applying $(basename "$migration")..."
        cat "$migration" | docker exec -i nullblock-postgres-agents psql -U postgres -d agents 2>&1 | grep -E "(ERROR|NOTICE|already exists)" || echo "    ✅ Applied successfully"
    fi
done

echo "✅ Agents migrations completed"














