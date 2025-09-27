-- Migration: Add task action tracking fields
-- Adds fields to track when tasks are actually processed by agents
-- and what results they produce, separate from lifecycle management

-- Add action tracking fields to tasks table
ALTER TABLE tasks ADD COLUMN actioned_at TIMESTAMPTZ;
ALTER TABLE tasks ADD COLUMN action_result TEXT;
ALTER TABLE tasks ADD COLUMN action_metadata JSONB DEFAULT '{}'::jsonb;
ALTER TABLE tasks ADD COLUMN action_duration BIGINT; -- milliseconds

-- Add indexes for action tracking queries
CREATE INDEX idx_tasks_actioned_at ON tasks(actioned_at);
CREATE INDEX idx_tasks_action_result ON tasks(action_result) WHERE action_result IS NOT NULL;
CREATE INDEX idx_tasks_action_duration ON tasks(action_duration);

-- Add composite index for finding unprocessed tasks
CREATE INDEX idx_tasks_unprocessed ON tasks(status, actioned_at) WHERE actioned_at IS NULL;

-- Add index for tasks by agent with action status
CREATE INDEX idx_tasks_agent_actioned ON tasks(assigned_agent_id, actioned_at);