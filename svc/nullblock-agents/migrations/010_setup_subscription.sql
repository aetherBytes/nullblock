-- Migration: Setup PostgreSQL Logical Replication Subscription
-- This migration sets up the subscription on Agents database to receive user data from Erebus
-- Run this on the Agents database (port 5441) AFTER setting up publication on Erebus

-- =============================================================================
-- PART 2: AGENTS DATABASE SETUP (Target/Subscriber)
-- Run this on the Agents database (port 5441)
-- =============================================================================

-- Verify that user_references table exists and has correct structure
DO $$
DECLARE
    table_exists boolean;
BEGIN
    SELECT EXISTS (
        SELECT FROM information_schema.tables
        WHERE table_schema = 'public'
        AND table_name = 'user_references'
    ) INTO table_exists;

    IF NOT table_exists THEN
        RAISE EXCEPTION 'user_references table does not exist. Please run previous migrations first.';
    END IF;

    RAISE NOTICE 'user_references table exists and ready for replication';
END
$$;

-- Clear any existing data (fresh start for replication)
-- WARNING: This will delete all existing user reference data in Agents database
DO $$
DECLARE
    user_count int;
BEGIN
    SELECT COUNT(*) INTO user_count FROM user_references;

    IF user_count > 0 THEN
        RAISE NOTICE 'Found % existing user_references records. Clearing for fresh replication sync...', user_count;
        DELETE FROM user_references;
        RAISE NOTICE 'Cleared existing user_references data for clean replication start';
    ELSE
        RAISE NOTICE 'user_references table is empty, ready for initial sync';
    END IF;
END
$$;

-- Create subscription to Erebus database
-- This will automatically sync all data from the publication
DO $$
DECLARE
    subscription_exists boolean;
    connection_string text := 'host=localhost port=5440 dbname=erebus user=nullblock_replicator password=nullblock_replication_secure_2024';
BEGIN
    -- Check if subscription already exists
    SELECT EXISTS (
        SELECT FROM pg_subscription
        WHERE subname = 'agents_user_sync'
    ) INTO subscription_exists;

    IF subscription_exists THEN
        RAISE NOTICE 'Subscription agents_user_sync already exists. Dropping and recreating...';
        DROP SUBSCRIPTION agents_user_sync;
    END IF;

    -- Create the subscription
    EXECUTE format('CREATE SUBSCRIPTION agents_user_sync CONNECTION %L PUBLICATION erebus_user_sync', connection_string);

    RAISE NOTICE 'Created subscription: agents_user_sync';
    RAISE NOTICE 'Connection: %', connection_string;
    RAISE NOTICE 'Publication: erebus_user_sync';
END
$$;

-- Wait a moment for initial sync to complete
SELECT pg_sleep(2);

-- Verify subscription status
SELECT
    subname as subscription_name,
    subenabled as enabled,
    subconninfo as connection_info,
    subpublications as publications
FROM pg_subscription
WHERE subname = 'agents_user_sync';

-- Check replication status
SELECT
    subscription_name,
    received_lsn,
    last_msg_send_time,
    last_msg_receipt_time,
    latest_end_lsn,
    latest_end_time
FROM pg_stat_subscription
WHERE subscription_name = 'agents_user_sync';

-- Verify data was synced
DO $$
DECLARE
    synced_count int;
BEGIN
    SELECT COUNT(*) INTO synced_count FROM user_references;

    RAISE NOTICE '==========================================';
    RAISE NOTICE 'AGENTS LOGICAL REPLICATION SETUP COMPLETE';
    RAISE NOTICE '==========================================';
    RAISE NOTICE 'Subscription: agents_user_sync created';
    RAISE NOTICE 'Initial sync completed: % user records', synced_count;
    RAISE NOTICE '';
    RAISE NOTICE 'Real-time sync is now active!';
    RAISE NOTICE 'Changes in Erebus will automatically appear in Agents database';
    RAISE NOTICE '==========================================';
END
$$;

-- Create monitoring views for replication health
CREATE OR REPLACE VIEW subscription_health AS
SELECT
    s.subname as subscription_name,
    s.subenabled as enabled,
    ss.received_lsn,
    ss.last_msg_send_time,
    ss.last_msg_receipt_time,
    ss.latest_end_time,
    CASE
        WHEN ss.last_msg_receipt_time IS NULL THEN 'Never received'
        WHEN ss.last_msg_receipt_time < NOW() - INTERVAL '1 minute' THEN 'Possibly stale'
        ELSE 'Healthy'
    END as health_status
FROM pg_subscription s
LEFT JOIN pg_stat_subscription ss ON s.subname = ss.subscription_name
WHERE s.subname = 'agents_user_sync';

-- Create monitoring view for sync lag
CREATE OR REPLACE VIEW replication_lag AS
SELECT
    'agents_user_sync' as subscription_name,
    EXTRACT(EPOCH FROM (NOW() - ss.last_msg_receipt_time)) as lag_seconds,
    CASE
        WHEN ss.last_msg_receipt_time IS NULL THEN '❌ No messages received'
        WHEN EXTRACT(EPOCH FROM (NOW() - ss.last_msg_receipt_time)) < 5 THEN '✅ Real-time'
        WHEN EXTRACT(EPOCH FROM (NOW() - ss.last_msg_receipt_time)) < 60 THEN '⚠️ Minor lag'
        ELSE '🔴 Significant lag'
    END as status
FROM pg_stat_subscription ss
WHERE ss.subscription_name = 'agents_user_sync';

-- Add helpful comments
COMMENT ON SUBSCRIPTION agents_user_sync IS 'Logical replication subscription receiving user_references from Erebus database';

-- Show final status
SELECT * FROM subscription_health;
SELECT * FROM replication_lag;