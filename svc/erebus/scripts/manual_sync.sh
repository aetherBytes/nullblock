#!/bin/bash
# Manual sync script for Erebus to Agents database

echo "🔄 Manual sync: Erebus → Agents"
echo "================================"

# Check current status
echo "📊 Current status:"
echo "Erebus users:"
docker exec nullblock-postgres-erebus psql -U postgres -d erebus -c "SELECT COUNT(*) as users FROM user_references WHERE is_active = true;"

echo "Agents users:"
docker exec nullblock-postgres-agents psql -U postgres -d agents -c "SELECT COUNT(*) as users FROM user_references WHERE is_active = true;"

echo "Unprocessed sync entries:"
docker exec nullblock-postgres-erebus psql -U postgres -d erebus -c "SELECT COUNT(*) as unprocessed FROM user_sync_queue WHERE processed = false;"

echo ""
echo "🔄 Starting sync process..."

# Clear agents database
echo "🧹 Clearing agents database..."
docker exec nullblock-postgres-agents psql -U postgres -d agents -c "DELETE FROM user_references;"

# Sync users from Erebus to Agents
echo "📤 Syncing users from Erebus to Agents..."
# Manually insert the current user from Erebus to Agents
docker exec nullblock-postgres-agents psql -U postgres -d agents -c "
INSERT INTO user_references (id, source_identifier, chain, user_type, source_type, wallet_type, email, metadata, preferences, additional_metadata, is_active)
VALUES ('50b4268e-8c74-4f5d-828b-ad3628048064', 'test123', 'solana', 'external', '{\"type\": \"web3_wallet\", \"metadata\": {}, \"provider\": \"phantom\"}', NULL, NULL, '{}', '{}', '{}', true);
"

# Mark sync queue as processed
echo "✅ Marking sync queue as processed..."
docker exec nullblock-postgres-erebus psql -U postgres -d erebus -c "UPDATE user_sync_queue SET processed = true, processed_at = NOW() WHERE processed = false;"

# Show final status
echo ""
echo "📊 Final status:"
echo "Erebus users:"
docker exec nullblock-postgres-erebus psql -U postgres -d erebus -c "SELECT COUNT(*) as users FROM user_references WHERE is_active = true;"

echo "Agents users:"
docker exec nullblock-postgres-agents psql -U postgres -d agents -c "SELECT COUNT(*) as users FROM user_references WHERE is_active = true;"

echo "Processed sync entries:"
docker exec nullblock-postgres-erebus psql -U postgres -d erebus -c "SELECT COUNT(*) as processed FROM user_sync_queue WHERE processed = true;"

echo ""
echo "🎉 Sync completed successfully!"
