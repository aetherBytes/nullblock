-- Migration: Fix user_references architecture in Agents database
-- The agents database should be a READ-ONLY sync cache from Erebus
-- This migration removes the ownership fields and makes it a proper cache

-- Drop the foreign key constraint from task_executions to user_references
-- since agents should not own user data
ALTER TABLE task_executions DROP CONSTRAINT IF EXISTS fk_tasks_user_id;

-- Remove the sync tracking fields that should only be in Erebus
ALTER TABLE user_references DROP COLUMN IF EXISTS synced_at;
ALTER TABLE user_references DROP COLUMN IF EXISTS erebus_created_at;
ALTER TABLE user_references DROP COLUMN IF EXISTS erebus_updated_at;

-- Remove the sync-related indexes
DROP INDEX IF EXISTS idx_user_references_synced_at;

-- Add a comment to clarify this is a read-only cache
COMMENT ON TABLE user_references IS 'READ-ONLY sync cache from Erebus user_references table. Do not modify directly.';

-- The user_references table in agents should be populated via sync from Erebus
-- The task_executions table will use source_identifier and source_metadata
-- instead of foreign keys to user_references



