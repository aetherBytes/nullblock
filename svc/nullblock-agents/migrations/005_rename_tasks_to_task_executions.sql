-- Migration: Rename tasks table to task_executions and add source tracking
-- This migration renames the table to be more generic and adds fields for tracking
-- the source of task creation (wallet, agent, API, etc.) with metadata

-- First, rename the table
ALTER TABLE tasks RENAME TO task_executions;

-- Add new source tracking fields
ALTER TABLE task_executions 
ADD COLUMN source_identifier VARCHAR, -- Generic identifier (wallet address, agent UUID, API key, etc.)
ADD COLUMN source_metadata JSONB DEFAULT '{}'::jsonb; -- JSON metadata about the source

-- Add indexes for the new fields
CREATE INDEX idx_task_executions_source_identifier ON task_executions(source_identifier);
CREATE INDEX idx_task_executions_source_metadata ON task_executions USING GIN (source_metadata);

-- Update the foreign key constraint name if it exists
-- (This will be handled by the foreign key constraints migration)

-- Add comments for documentation
COMMENT ON TABLE task_executions IS 'Task execution records with source tracking for audit trails';
COMMENT ON COLUMN task_executions.source_identifier IS 'Generic identifier for the source that created this task (wallet address, agent UUID, API key, etc.)';
COMMENT ON COLUMN task_executions.source_metadata IS 'JSON metadata about the source including type, chain, wallet_type, etc.';

-- Example source_metadata structure:
-- {
--   "type": "web3_wallet",
--   "chain": "solana", 
--   "wallet_type": "phantom",
--   "user_id": "uuid-here",
--   "session_id": "session-uuid",
--   "ip_address": "192.168.1.1",
--   "user_agent": "Mozilla/5.0..."
-- }
