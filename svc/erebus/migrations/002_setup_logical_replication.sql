-- Migration: Setup PostgreSQL Logical Replication for User Sync
-- This migration replaces the Python sync scripts with native PostgreSQL logical replication
-- to sync user_references from Erebus (source) to Agents (target) database

-- =============================================================================
-- PART 1: EREBUS DATABASE SETUP (Source/Publisher)
-- Run this on the Erebus database (port 5440)
-- =============================================================================

-- Create replication user with appropriate permissions
DO $$
BEGIN
    -- Create replication user if it doesn't exist
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'nullblock_replicator') THEN
        CREATE USER nullblock_replicator WITH REPLICATION PASSWORD 'nullblock_replication_secure_2024';
        RAISE NOTICE 'Created replication user: nullblock_replicator';
    ELSE
        RAISE NOTICE 'Replication user nullblock_replicator already exists';
    END IF;
END
$$;

-- Grant necessary permissions to replication user
GRANT CONNECT ON DATABASE erebus TO nullblock_replicator;
GRANT USAGE ON SCHEMA public TO nullblock_replicator;
GRANT SELECT ON user_references TO nullblock_replicator;

-- Create publication for user_references table
DO $$
BEGIN
    -- Check if publication already exists
    IF NOT EXISTS (SELECT 1 FROM pg_publication WHERE pubname = 'erebus_user_sync') THEN
        CREATE PUBLICATION erebus_user_sync FOR TABLE user_references;
        RAISE NOTICE 'Created publication: erebus_user_sync for table user_references';
    ELSE
        RAISE NOTICE 'Publication erebus_user_sync already exists';
    END IF;
END
$$;

-- Verify publication was created successfully
SELECT
    pubname as publication_name,
    pubowner::regrole as owner,
    puballtables as all_tables,
    pubinsert as insert_enabled,
    pubupdate as update_enabled,
    pubdelete as delete_enabled
FROM pg_publication
WHERE pubname = 'erebus_user_sync';

-- Show which tables are included in the publication
SELECT
    pub.pubname as publication_name,
    pt.tablename,
    pt.schemaname
FROM pg_publication pub
JOIN pg_publication_tables pt ON pub.pubname = pt.pubname
WHERE pub.pubname = 'erebus_user_sync';

-- =============================================================================
-- CONFIGURATION NOTES FOR EREBUS DATABASE
-- =============================================================================
--
-- The following PostgreSQL configuration changes are required in postgresql.conf:
--
-- # Enable logical replication
-- wal_level = logical
-- max_replication_slots = 4
-- max_wal_senders = 4
-- max_logical_replication_workers = 4
--
-- After making these changes, restart PostgreSQL service:
--
-- Docker: docker restart nullblock-postgres-erebus
--
-- =============================================================================

-- Verify current configuration (informational only)
SELECT
    name,
    setting,
    CASE
        WHEN name = 'wal_level' THEN CASE WHEN setting = 'logical' THEN '✅ Correct' ELSE '❌ Should be: logical' END
        WHEN name = 'max_replication_slots' THEN CASE WHEN setting::int >= 4 THEN '✅ Correct' ELSE '❌ Should be >= 4' END
        WHEN name = 'max_wal_senders' THEN CASE WHEN setting::int >= 4 THEN '✅ Correct' ELSE '❌ Should be >= 4' END
        ELSE 'ℹ️ For reference'
    END as status
FROM pg_settings
WHERE name IN ('wal_level', 'max_replication_slots', 'max_wal_senders', 'max_logical_replication_workers')
ORDER BY name;

-- Show current replication status
SELECT
    slot_name,
    plugin,
    slot_type,
    database,
    active,
    restart_lsn,
    confirmed_flush_lsn
FROM pg_replication_slots;

-- Add comments for documentation
COMMENT ON PUBLICATION erebus_user_sync IS 'Logical replication publication for syncing user_references to Agents database';

-- Create monitoring view for replication status
CREATE OR REPLACE VIEW replication_status AS
SELECT
    'erebus_user_sync' as publication_name,
    COUNT(*) as published_tables,
    string_agg(pt.schemaname || '.' || pt.tablename, ', ') as table_list
FROM pg_publication_tables pt
WHERE pt.pubname = 'erebus_user_sync';

-- Final verification and status report
DO $$
DECLARE
    pub_count int;
    table_count int;
BEGIN
    SELECT COUNT(*) INTO pub_count FROM pg_publication WHERE pubname = 'erebus_user_sync';
    SELECT COUNT(*) INTO table_count FROM pg_publication_tables WHERE pubname = 'erebus_user_sync';

    RAISE NOTICE '==========================================';
    RAISE NOTICE 'EREBUS LOGICAL REPLICATION SETUP COMPLETE';
    RAISE NOTICE '==========================================';
    RAISE NOTICE 'Publications created: %', pub_count;
    RAISE NOTICE 'Tables in publication: %', table_count;
    RAISE NOTICE '';
    RAISE NOTICE 'Next steps:';
    RAISE NOTICE '1. Ensure PostgreSQL config has wal_level = logical';
    RAISE NOTICE '2. Restart Erebus PostgreSQL container if config changed';
    RAISE NOTICE '3. Run setup_subscription.sql on Agents database (port 5441)';
    RAISE NOTICE '';
    RAISE NOTICE 'Replication user: nullblock_replicator';
    RAISE NOTICE 'Publication: erebus_user_sync';
    RAISE NOTICE '==========================================';
END
$$;