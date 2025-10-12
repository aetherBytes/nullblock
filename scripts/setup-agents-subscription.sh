#!/bin/bash

# Setup PostgreSQL Logical Replication Subscription
# This script creates the subscription in Agents DB to sync from Erebus DB

set -e

echo "=========================================="
echo "Setting up Agents DB User Sync Subscription"
echo "=========================================="

# Connection details
EREBUS_HOST="${EREBUS_HOST:-localhost}"
EREBUS_PORT="${EREBUS_PORT:-5440}"
AGENTS_HOST="${AGENTS_HOST:-localhost}"
AGENTS_PORT="${AGENTS_PORT:-5441}"
POSTGRES_USER="${POSTGRES_USER:-postgres}"
POSTGRES_PASSWORD="${POSTGRES_PASSWORD:-REDACTED_DB_PASS}"

echo "📋 Configuration:"
echo "  Erebus DB: ${EREBUS_HOST}:${EREBUS_PORT}"
echo "  Agents DB: ${AGENTS_HOST}:${AGENTS_PORT}"
echo ""

# Check if publication exists on Erebus
echo "🔍 Checking Erebus publication..."
EREBUS_PUB_COUNT=$(PGPASSWORD="$POSTGRES_PASSWORD" psql -h "$EREBUS_HOST" -p "$EREBUS_PORT" -U "$POSTGRES_USER" -d erebus -tAc \
  "SELECT COUNT(*) FROM pg_publication WHERE pubname = 'erebus_user_sync';")

if [ "$EREBUS_PUB_COUNT" -eq "0" ]; then
  echo "❌ ERROR: Publication 'erebus_user_sync' does not exist on Erebus DB"
  echo "   Please run the Erebus migration first: svc/erebus/migrations/002_setup_logical_replication.sql"
  exit 1
fi

echo "✅ Erebus publication 'erebus_user_sync' exists"

# Check if table exists in Agents DB
echo "🔍 Checking Agents DB table..."
AGENTS_TABLE_COUNT=$(PGPASSWORD="$POSTGRES_PASSWORD" psql -h "$AGENTS_HOST" -p "$AGENTS_PORT" -U "$POSTGRES_USER" -d agents -tAc \
  "SELECT COUNT(*) FROM information_schema.tables WHERE table_name = 'erebus_user_references';")

if [ "$AGENTS_TABLE_COUNT" -eq "0" ]; then
  echo "❌ ERROR: Table 'erebus_user_references' does not exist in Agents DB"
  echo "   Please run the Agents migration first: svc/nullblock-agents/migrations/003_create_user_references_table.sql"
  exit 1
fi

echo "✅ Agents table 'erebus_user_references' exists"

# Drop existing subscription if it exists
echo "🗑️  Dropping existing subscription (if any)..."
PGPASSWORD="$POSTGRES_PASSWORD" psql -h "$AGENTS_HOST" -p "$AGENTS_PORT" -U "$POSTGRES_USER" -d agents <<-EOSQL
  DO \$\$
  BEGIN
    IF EXISTS (SELECT 1 FROM pg_subscription WHERE subname = 'agents_user_sync') THEN
      DROP SUBSCRIPTION agents_user_sync;
      RAISE NOTICE 'Dropped existing subscription: agents_user_sync';
    ELSE
      RAISE NOTICE 'No existing subscription to drop';
    END IF;
  END \$\$;
EOSQL

# Create subscription
echo "📡 Creating subscription..."
PGPASSWORD="$POSTGRES_PASSWORD" psql -h "$AGENTS_HOST" -p "$AGENTS_PORT" -U "$POSTGRES_USER" -d agents <<-EOSQL
  CREATE SUBSCRIPTION agents_user_sync
  CONNECTION 'host=nullblock-postgres-erebus port=5432 dbname=erebus user=postgres password=REDACTED_DB_PASS'
  PUBLICATION erebus_user_sync
  WITH (copy_data = true, create_slot = true, enabled = true);
EOSQL

echo "✅ Subscription created successfully"

# Verify subscription
echo "🔍 Verifying subscription status..."
PGPASSWORD="$POSTGRES_PASSWORD" psql -h "$AGENTS_HOST" -p "$AGENTS_PORT" -U "$POSTGRES_USER" -d agents <<-EOSQL
  SELECT
    subname as subscription_name,
    subenabled as enabled,
    subpublications as publications
  FROM pg_subscription
  WHERE subname = 'agents_user_sync';
EOSQL

# Check replication slot on Erebus
echo "🔍 Checking replication slot on Erebus..."
PGPASSWORD="$POSTGRES_PASSWORD" psql -h "$EREBUS_HOST" -p "$EREBUS_PORT" -U "$POSTGRES_USER" -d erebus <<-EOSQL
  SELECT
    slot_name,
    plugin,
    active,
    restart_lsn IS NOT NULL as has_data
  FROM pg_replication_slots
  WHERE slot_name LIKE '%agents_user_sync%';
EOSQL

# Check user count in both databases
echo "📊 Checking user counts..."
EREBUS_COUNT=$(PGPASSWORD="$POSTGRES_PASSWORD" psql -h "$EREBUS_HOST" -p "$EREBUS_PORT" -U "$POSTGRES_USER" -d erebus -tAc \
  "SELECT COUNT(*) FROM user_references WHERE is_active = true;")
AGENTS_COUNT=$(PGPASSWORD="$POSTGRES_PASSWORD" psql -h "$AGENTS_HOST" -p "$AGENTS_PORT" -U "$POSTGRES_USER" -d agents -tAc \
  "SELECT COUNT(*) FROM erebus_user_references WHERE is_active = true;")

echo "  Erebus DB: $EREBUS_COUNT active users"
echo "  Agents DB: $AGENTS_COUNT active users (synced)"

if [ "$EREBUS_COUNT" -eq "$AGENTS_COUNT" ]; then
  echo "✅ User counts match - replication working!"
else
  echo "⚠️  User counts differ - replication may still be in progress"
  echo "   Wait a few seconds and check again"
fi

echo ""
echo "=========================================="
echo "✅ SUBSCRIPTION SETUP COMPLETE"
echo "=========================================="
echo ""
echo "Next steps:"
echo "1. Drop and recreate Agents DB: docker exec nullblock-postgres-agents psql -U postgres -d agents -c 'DROP TABLE IF EXISTS tasks CASCADE; DROP TABLE IF EXISTS agents CASCADE; DROP TABLE IF EXISTS erebus_user_references CASCADE;'"
echo "2. Run Agents migrations: cd svc/nullblock-agents && DATABASE_URL='postgresql://postgres:REDACTED_DB_PASS@localhost:5441/agents' sqlx migrate run"
echo "3. Re-run this script to set up subscription"
echo "4. Test user creation in Erebus and verify it syncs to Agents"
echo ""
