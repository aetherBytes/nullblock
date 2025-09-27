-- Migration: Sync Agents user_references schema with Erebus
-- This migration updates the Agents database schema to match Erebus for logical replication

-- First, drop the subscription to stop replication during schema changes
DROP SUBSCRIPTION IF EXISTS agents_user_sync;

-- Rename 'chain' to 'network' to match Erebus
ALTER TABLE user_references RENAME COLUMN chain TO network;

-- Add missing columns from Erebus schema
ALTER TABLE user_references ADD COLUMN IF NOT EXISTS created_at TIMESTAMPTZ DEFAULT NOW();
ALTER TABLE user_references ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ DEFAULT NOW();

-- Update constraints and indexes to match new column name
DROP INDEX IF EXISTS idx_user_references_chain;
DROP INDEX IF EXISTS idx_user_references_source_chain_unique;

CREATE INDEX IF NOT EXISTS idx_user_references_network ON user_references(network);
CREATE UNIQUE INDEX IF NOT EXISTS idx_user_references_source_network_unique
ON user_references(source_identifier, network)
WHERE source_identifier IS NOT NULL;

-- Add created_at and updated_at indexes to match Erebus
CREATE INDEX IF NOT EXISTS idx_user_references_created_at ON user_references(created_at);

-- Update any existing records to have created_at/updated_at values
UPDATE user_references
SET
    created_at = COALESCE(created_at, NOW()),
    updated_at = COALESCE(updated_at, NOW())
WHERE created_at IS NULL OR updated_at IS NULL;

-- Make timestamps NOT NULL to match Erebus
ALTER TABLE user_references ALTER COLUMN created_at SET NOT NULL;
ALTER TABLE user_references ALTER COLUMN updated_at SET NOT NULL;

-- Update column types and constraints to exactly match Erebus
ALTER TABLE user_references ALTER COLUMN source_identifier TYPE VARCHAR(255);
ALTER TABLE user_references ALTER COLUMN network TYPE VARCHAR(50);
ALTER TABLE user_references ALTER COLUMN user_type TYPE VARCHAR(50);
ALTER TABLE user_references ALTER COLUMN email TYPE VARCHAR(255);

-- Set default for user_type to match Erebus
ALTER TABLE user_references ALTER COLUMN user_type SET DEFAULT 'external';

-- Show the updated schema
\d user_references

-- Now recreate the subscription with the matching schema
CREATE SUBSCRIPTION agents_user_sync
CONNECTION 'host=nullblock-postgres-erebus port=5432 dbname=erebus user=nullblock_replicator password=nullblock_replication_secure_2024'
PUBLICATION erebus_user_sync
WITH (copy_data = true, create_slot = true);

-- Wait for initial sync
SELECT pg_sleep(5);

-- Verify the schema now matches
DO $$
DECLARE
    user_count int;
BEGIN
    SELECT COUNT(*) INTO user_count FROM user_references;
    RAISE NOTICE '==========================================';
    RAISE NOTICE 'SCHEMA SYNC AND REPLICATION SETUP COMPLETE';
    RAISE NOTICE '==========================================';
    RAISE NOTICE 'Schema updated to match Erebus';
    RAISE NOTICE 'Subscription recreated with copy_data = true';
    RAISE NOTICE 'Initial sync completed: % user records', user_count;
    RAISE NOTICE 'Logical replication is now active!';
    RAISE NOTICE '==========================================';
END
$$;