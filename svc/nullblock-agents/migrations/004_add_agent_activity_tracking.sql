-- Migration: Add agent activity tracking fields
-- Tracks agent performance metrics and task processing history

-- Add activity tracking fields to agents table
ALTER TABLE agents ADD COLUMN last_task_processed UUID;
ALTER TABLE agents ADD COLUMN tasks_processed_count INTEGER DEFAULT 0;
ALTER TABLE agents ADD COLUMN last_action_at TIMESTAMPTZ;
ALTER TABLE agents ADD COLUMN average_processing_time BIGINT DEFAULT 0; -- milliseconds
ALTER TABLE agents ADD COLUMN total_processing_time BIGINT DEFAULT 0; -- milliseconds for calculating averages

-- Add foreign key constraint to last_task_processed
ALTER TABLE agents ADD CONSTRAINT fk_agents_last_task FOREIGN KEY (last_task_processed) REFERENCES tasks(id);

-- Add indexes for agent activity tracking
CREATE INDEX idx_agents_last_action_at ON agents(last_action_at);
CREATE INDEX idx_agents_tasks_processed ON agents(tasks_processed_count);
CREATE INDEX idx_agents_avg_processing_time ON agents(average_processing_time);
CREATE INDEX idx_agents_last_task_processed ON agents(last_task_processed);

-- Add composite index for finding active agents
CREATE INDEX idx_agents_active_processing ON agents(status, last_action_at) WHERE status = 'active';