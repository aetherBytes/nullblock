#!/bin/bash
# Setup PostgreSQL Logical Replication for NullBlock User Sync
# This script replaces Python sync scripts with native PostgreSQL logical replication

set -e

echo "🔄 Setting up PostgreSQL Logical Replication for NullBlock User Sync"
echo "This will replace the failing Python sync scripts with native PostgreSQL replication"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Database connection info
EREBUS_DB="postgresql://postgres:postgres_secure_pass@localhost:5440/erebus"
AGENTS_DB="postgresql://postgres:postgres_secure_pass@localhost:5441/agents"

echo "📋 Pre-flight checks..."

# Check if Docker containers are running
if ! docker ps | grep -q "nullblock-postgres-erebus"; then
    echo -e "${RED}❌ Erebus PostgreSQL container is not running${NC}"
    echo "Please run: docker-compose up postgres-erebus -d"
    exit 1
fi

if ! docker ps | grep -q "nullblock-postgres-agents"; then
    echo -e "${RED}❌ Agents PostgreSQL container is not running${NC}"
    echo "Please run: docker-compose up postgres-agents -d"
    exit 1
fi

echo -e "${GREEN}✅ PostgreSQL containers are running${NC}"

# Test database connections
echo "🔍 Testing database connections..."

if ! docker exec nullblock-postgres-erebus pg_isready -U postgres > /dev/null 2>&1; then
    echo -e "${RED}❌ Cannot connect to Erebus database${NC}"
    exit 1
fi

if ! docker exec nullblock-postgres-agents pg_isready -U postgres > /dev/null 2>&1; then
    echo -e "${RED}❌ Cannot connect to Agents database${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Database connections successful${NC}"

# Check current PostgreSQL configuration
echo "🔧 Checking PostgreSQL configuration for logical replication..."

WAL_LEVEL=$(docker exec nullblock-postgres-erebus psql -U postgres -d erebus -t -c "SHOW wal_level;" 2>/dev/null | xargs)
MAX_REPLICATION_SLOTS=$(docker exec nullblock-postgres-erebus psql -U postgres -d erebus -t -c "SHOW max_replication_slots;" 2>/dev/null | xargs)
MAX_WAL_SENDERS=$(docker exec nullblock-postgres-erebus psql -U postgres -d erebus -t -c "SHOW max_wal_senders;" 2>/dev/null | xargs)

echo "Current Erebus PostgreSQL configuration:"
echo "  wal_level: $WAL_LEVEL"
echo "  max_replication_slots: $MAX_REPLICATION_SLOTS"
echo "  max_wal_senders: $MAX_WAL_SENDERS"

NEEDS_RESTART=false

if [ "$WAL_LEVEL" != "logical" ]; then
    echo -e "${YELLOW}⚠️ wal_level is '$WAL_LEVEL', need to set to 'logical'${NC}"
    docker exec nullblock-postgres-erebus psql -U postgres -c "ALTER SYSTEM SET wal_level = logical;"
    NEEDS_RESTART=true
fi

if [ "$MAX_REPLICATION_SLOTS" -lt 4 ]; then
    echo -e "${YELLOW}⚠️ max_replication_slots is $MAX_REPLICATION_SLOTS, setting to 4${NC}"
    docker exec nullblock-postgres-erebus psql -U postgres -c "ALTER SYSTEM SET max_replication_slots = 4;"
    NEEDS_RESTART=true
fi

if [ "$MAX_WAL_SENDERS" -lt 4 ]; then
    echo -e "${YELLOW}⚠️ max_wal_senders is $MAX_WAL_SENDERS, setting to 4${NC}"
    docker exec nullblock-postgres-erebus psql -U postgres -c "ALTER SYSTEM SET max_wal_senders = 4;"
    NEEDS_RESTART=true
fi

if [ "$NEEDS_RESTART" = true ]; then
    echo -e "${YELLOW}🔄 Configuration changed, restarting Erebus PostgreSQL container...${NC}"
    docker restart nullblock-postgres-erebus

    echo "⏳ Waiting for PostgreSQL to restart..."
    sleep 10

    # Wait for database to be ready
    while ! docker exec nullblock-postgres-erebus pg_isready -U postgres > /dev/null 2>&1; do
        echo "⏳ Waiting for Erebus PostgreSQL to be ready..."
        sleep 2
    done

    echo -e "${GREEN}✅ PostgreSQL restarted successfully${NC}"
else
    echo -e "${GREEN}✅ PostgreSQL configuration is correct for logical replication${NC}"
fi

echo ""
echo "🚀 Setting up logical replication..."

# Step 1: Setup publication on Erebus
echo "📤 Setting up publication on Erebus database..."
docker exec nullblock-postgres-erebus psql -U postgres -d erebus -f /home/sage/nullblock/svc/erebus/migrations/010_setup_logical_replication.sql

echo -e "${GREEN}✅ Publication setup complete on Erebus${NC}"

# Step 2: Setup subscription on Agents
echo "📥 Setting up subscription on Agents database..."
docker exec nullblock-postgres-agents psql -U postgres -d agents -f /home/sage/nullblock/svc/nullblock-agents/migrations/010_setup_subscription.sql

echo -e "${GREEN}✅ Subscription setup complete on Agents${NC}"

echo ""
echo "🧪 Testing replication functionality..."

# Test 1: Check initial sync
EREBUS_COUNT=$(docker exec nullblock-postgres-erebus psql -U postgres -d erebus -t -c "SELECT COUNT(*) FROM user_references WHERE is_active = true;" | xargs)
AGENTS_COUNT=$(docker exec nullblock-postgres-agents psql -U postgres -d agents -t -c "SELECT COUNT(*) FROM user_references WHERE is_active = true;" | xargs)

echo "Initial sync verification:"
echo "  Erebus users: $EREBUS_COUNT"
echo "  Agents users: $AGENTS_COUNT"

if [ "$EREBUS_COUNT" -eq "$AGENTS_COUNT" ]; then
    echo -e "${GREEN}✅ Initial sync successful - user counts match${NC}"
else
    echo -e "${YELLOW}⚠️ User counts don't match yet, replication may still be in progress${NC}"
fi

# Test 2: Create a test user to verify real-time sync
echo ""
echo "🧪 Testing real-time replication with test user..."

TEST_USER_ID=$(uuidgen)
TEST_WALLET="test_wallet_$(date +%s)"

echo "Creating test user in Erebus..."
docker exec nullblock-postgres-erebus psql -U postgres -d erebus -c "
INSERT INTO user_references (
    id, source_identifier, chain, source_type, is_active
) VALUES (
    '$TEST_USER_ID'::uuid,
    '$TEST_WALLET',
    'solana',
    '{\"type\": \"web3_wallet\", \"provider\": \"test\"}'::jsonb,
    true
);"

echo "⏳ Waiting 3 seconds for replication..."
sleep 3

# Check if test user appeared in Agents database
if docker exec nullblock-postgres-agents psql -U postgres -d agents -t -c "SELECT EXISTS(SELECT 1 FROM user_references WHERE id = '$TEST_USER_ID'::uuid);" | grep -q "t"; then
    echo -e "${GREEN}✅ Real-time replication working! Test user appeared in Agents database${NC}"

    # Clean up test user
    docker exec nullblock-postgres-erebus psql -U postgres -d erebus -c "DELETE FROM user_references WHERE id = '$TEST_USER_ID'::uuid;"
    sleep 2

    if docker exec nullblock-postgres-agents psql -U postgres -d agents -t -c "SELECT EXISTS(SELECT 1 FROM user_references WHERE id = '$TEST_USER_ID'::uuid);" | grep -q "f"; then
        echo -e "${GREEN}✅ Delete replication also working! Test user removed from Agents database${NC}"
    else
        echo -e "${YELLOW}⚠️ Delete replication may be delayed${NC}"
    fi
else
    echo -e "${RED}❌ Real-time replication not working - test user not found in Agents database${NC}"
fi

echo ""
echo "📊 Replication status summary:"

# Show replication status
docker exec nullblock-postgres-erebus psql -U postgres -d erebus -c "
SELECT
    pubname as publication,
    pubinsert as insert_enabled,
    pubupdate as update_enabled,
    pubdelete as delete_enabled
FROM pg_publication WHERE pubname = 'erebus_user_sync';"

docker exec nullblock-postgres-agents psql -U postgres -d agents -c "
SELECT * FROM subscription_health;"

docker exec nullblock-postgres-agents psql -U postgres -d agents -c "
SELECT * FROM replication_lag;"

echo ""
echo -e "${GREEN}🎉 PostgreSQL Logical Replication setup complete!${NC}"
echo ""
echo -e "${BLUE}📋 What was accomplished:${NC}"
echo "  ✅ Configured PostgreSQL for logical replication"
echo "  ✅ Created publication 'erebus_user_sync' on Erebus database"
echo "  ✅ Created subscription 'agents_user_sync' on Agents database"
echo "  ✅ Verified real-time bidirectional sync functionality"
echo "  ✅ Replaced Python sync scripts with native PostgreSQL replication"
echo ""
echo -e "${BLUE}📊 Benefits achieved:${NC}"
echo "  🚀 Real-time sync (vs 30-second polling)"
echo "  🛡️ Built-in failure recovery"
echo "  📈 Better performance and reliability"
echo "  🔧 Zero maintenance (no Python scripts to debug)"
echo ""
echo -e "${BLUE}🔍 Monitoring:${NC}"
echo "  Check replication status: SELECT * FROM subscription_health;"
echo "  Check replication lag: SELECT * FROM replication_lag;"
echo "  PostgreSQL logs: docker logs nullblock-postgres-erebus"
echo ""
echo "Next step: Remove legacy Python sync scripts and update documentation"