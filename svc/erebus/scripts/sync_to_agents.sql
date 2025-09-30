-- Sync script to process user_sync_queue and sync data to agents database
-- This script processes the sync queue and syncs user data from Erebus to Agents

-- First, let's see what's in the sync queue
SELECT 'Sync Queue Status:' as status;
SELECT COUNT(*) as total_entries, 
       COUNT(CASE WHEN processed = false THEN 1 END) as unprocessed,
       COUNT(CASE WHEN processed = true THEN 1 END) as processed
FROM user_sync_queue;

-- Show unprocessed entries
SELECT 'Unprocessed Entries:' as status;
SELECT id, user_id, operation, created_at 
FROM user_sync_queue 
WHERE processed = false 
ORDER BY created_at;

-- For now, let's manually sync the current user from Erebus to Agents
-- This is a temporary solution until we have a proper sync processor

-- Get the current user from Erebus
SELECT 'Current Erebus User:' as status;
SELECT id, source_identifier, chain, source_type 
FROM user_references 
WHERE is_active = true;

-- Insert/Update the user in Agents database
-- We'll use a simple approach: delete existing and insert fresh
DELETE FROM user_references WHERE source_identifier = 'test123';

-- Insert the current user from Erebus into Agents
INSERT INTO user_references (
    id, source_identifier, chain, user_type, source_type, wallet_type, 
    email, metadata, preferences, additional_metadata, is_active
)
SELECT 
    id, source_identifier, chain, user_type, source_type, wallet_type,
    email, metadata, preferences, additional_metadata, is_active
FROM dblink('host=localhost port=5440 dbname=erebus user=postgres password=REDACTED_DB_PASS',
    'SELECT id, source_identifier, chain, user_type, source_type, wallet_type, 
            email, metadata, preferences, additional_metadata, is_active 
     FROM user_references WHERE is_active = true'
) AS erebus_user (
    id uuid,
    source_identifier text,
    chain text,
    user_type text,
    source_type jsonb,
    wallet_type text,
    email text,
    metadata jsonb,
    preferences jsonb,
    additional_metadata jsonb,
    is_active boolean
);

-- Mark sync queue entries as processed
UPDATE user_sync_queue SET processed = true, processed_at = NOW() WHERE processed = false;

-- Show final status
SELECT 'Final Status:' as status;
SELECT 'Erebus users:' as database, COUNT(*) as count FROM dblink('host=localhost port=5440 dbname=erebus user=postgres password=REDACTED_DB_PASS', 'SELECT COUNT(*) FROM user_references WHERE is_active = true') AS erebus_count(count bigint)
UNION ALL
SELECT 'Agents users:' as database, COUNT(*) as count FROM user_references WHERE is_active = true;




