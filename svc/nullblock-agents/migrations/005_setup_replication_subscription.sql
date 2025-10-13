-- Migration: Setup PostgreSQL Logical Replication Subscription
-- This migration creates the subscription to sync user_references from Erebus database
-- Requires: Erebus database must have publication 'erebus_user_sync' already created
-- Requires: PostgreSQL containers must be on same Docker network (nullblock-network)

-- =============================================================================
-- AGENTS DATABASE SUBSCRIPTION SETUP (Target/Subscriber)
-- This creates the subscription to receive user data from Erebus
-- =============================================================================

-- Drop existing subscription if present (idempotent migration)
-- Note: CREATE SUBSCRIPTION cannot be executed from a function/DO block,
-- so we use DROP IF EXISTS + CREATE pattern for idempotency
DROP SUBSCRIPTION IF EXISTS agents_user_sync;

-- Create subscription to Erebus publication
-- This will automatically create a replication slot on Erebus
-- Using container name for Docker DNS resolution (system-agnostic, works on all platforms)
-- Port 5432 is internal container port for container-to-container communication
CREATE SUBSCRIPTION agents_user_sync
CONNECTION 'host=nullblock-postgres-erebus port=5432 dbname=erebus user=postgres password=postgres_secure_pass'
PUBLICATION erebus_user_sync
WITH (
    copy_data = true,          -- Copy existing data on initial sync
    create_slot = true,        -- Create replication slot on publisher
    enabled = true,            -- Enable immediately
    slot_name = 'agents_user_sync'
);

-- Wait briefly for initial sync to start
SELECT pg_sleep(2);

-- Verify subscription was created successfully
SELECT
    subname as subscription_name,
    subenabled as enabled,
    subpublications as publications
FROM pg_subscription
WHERE subname = 'agents_user_sync';

-- Check subscription replication state
-- States: i=initialize, d=data copy, s=synchronized, r=ready
SELECT
    CASE srsubstate
        WHEN 'i' THEN 'initializing'
        WHEN 'd' THEN 'copying data'
        WHEN 's' THEN 'synchronized'
        WHEN 'r' THEN 'ready'
        ELSE 'unknown'
    END as replication_state,
    srrelid::regclass as table_name
FROM pg_subscription_rel
WHERE srsubid = (SELECT oid FROM pg_subscription WHERE subname = 'agents_user_sync');

-- Show synced user count
SELECT
    COUNT(*) as synced_users,
    COUNT(*) FILTER (WHERE is_active = true) as active_users
FROM user_references;

-- Add comment for documentation
COMMENT ON SUBSCRIPTION agents_user_sync IS 'Logical replication subscription receiving user data from Erebus database. Updates occur in real-time with <1 second latency.';

-- Final status report
DO $$
DECLARE
    sub_count int;
    user_count int;
BEGIN
    SELECT COUNT(*) INTO sub_count FROM pg_subscription WHERE subname = 'agents_user_sync';
    SELECT COUNT(*) INTO user_count FROM user_references;

    RAISE NOTICE '==========================================';
    RAISE NOTICE 'AGENTS REPLICATION SUBSCRIPTION SETUP COMPLETE';
    RAISE NOTICE '==========================================';
    RAISE NOTICE 'Subscription created: %', CASE WHEN sub_count > 0 THEN 'YES' ELSE 'NO' END;
    RAISE NOTICE 'Users synced: %', user_count;
    RAISE NOTICE '';
    RAISE NOTICE 'Monitoring:';
    RAISE NOTICE '  SELECT * FROM pg_subscription WHERE subname = ''agents_user_sync'';';
    RAISE NOTICE '  SELECT * FROM pg_subscription_rel;';
    RAISE NOTICE '  SELECT COUNT(*) FROM user_references;';
    RAISE NOTICE '';
    RAISE NOTICE 'The user_references table is now READ-ONLY and synced';
    RAISE NOTICE 'from Erebus in real-time. All user CRUD operations must';
    RAISE NOTICE 'go through Erebus /api/users/* endpoints.';
    RAISE NOTICE '==========================================';
END
$$;
