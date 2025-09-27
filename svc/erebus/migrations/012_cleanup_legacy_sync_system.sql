-- Migration: Clean up legacy Python sync system
-- This migration removes the old sync queue and trigger system that are no longer needed
-- since we've moved to PostgreSQL logical replication

-- Remove the legacy sync queue table
DO $$
BEGIN
    IF EXISTS (SELECT FROM information_schema.tables WHERE table_name = 'user_sync_queue') THEN
        DROP TABLE user_sync_queue CASCADE;
        RAISE NOTICE 'Dropped user_sync_queue table';
    ELSE
        RAISE NOTICE 'user_sync_queue table does not exist (already cleaned)';
    END IF;
END
$$;

-- Remove legacy sync functions
DROP FUNCTION IF EXISTS queue_user_sync(UUID, VARCHAR, JSONB) CASCADE;
DROP FUNCTION IF EXISTS trigger_queue_user_sync() CASCADE;
DROP FUNCTION IF EXISTS sync_user_to_agents(UUID, VARCHAR, VARCHAR, VARCHAR, VARCHAR, VARCHAR, JSONB, JSONB, JSONB) CASCADE;
DROP FUNCTION IF EXISTS trigger_sync_user_to_agents() CASCADE;

-- Remove legacy sync triggers
DROP TRIGGER IF EXISTS trigger_queue_user_sync ON user_references;
DROP TRIGGER IF EXISTS trigger_sync_user_to_agents ON user_references;

-- Clean up any legacy replication views
DROP VIEW IF EXISTS replication_status;

-- Show remaining triggers on user_references table
SELECT
    trigger_name,
    event_manipulation,
    action_statement
FROM information_schema.triggers
WHERE event_object_table = 'user_references'
ORDER BY trigger_name;

DO $$
BEGIN
    RAISE NOTICE '==========================================';
    RAISE NOTICE 'LEGACY SYNC SYSTEM CLEANUP COMPLETE';
    RAISE NOTICE '==========================================';
    RAISE NOTICE 'Removed components:';
    RAISE NOTICE '  ✅ user_sync_queue table';
    RAISE NOTICE '  ✅ Python sync scripts';
    RAISE NOTICE '  ✅ Legacy sync functions and triggers';
    RAISE NOTICE '  ✅ dblink-based sync system';
    RAISE NOTICE '';
    RAISE NOTICE 'Active replication:';
    RAISE NOTICE '  🔄 PostgreSQL logical replication';
    RAISE NOTICE '  📡 Publication: erebus_user_sync';
    RAISE NOTICE '  📥 Subscription: agents_user_sync';
    RAISE NOTICE '==========================================';
END
$$;