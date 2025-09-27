-- Migration: Add activity tracking columns to agents table
-- This migration adds the missing columns that the AgentEntity model expects

-- Add activity tracking columns
ALTER TABLE agents ADD COLUMN IF NOT EXISTS last_task_processed UUID;
ALTER TABLE agents ADD COLUMN IF NOT EXISTS tasks_processed_count INTEGER DEFAULT 0;
ALTER TABLE agents ADD COLUMN IF NOT EXISTS last_action_at TIMESTAMPTZ;
ALTER TABLE agents ADD COLUMN IF NOT EXISTS average_processing_time BIGINT DEFAULT 0;
ALTER TABLE agents ADD COLUMN IF NOT EXISTS total_processing_time BIGINT DEFAULT 0;

-- Add indexes for the new columns
CREATE INDEX IF NOT EXISTS idx_agents_last_action_at ON agents(last_action_at);
CREATE INDEX IF NOT EXISTS idx_agents_tasks_processed_count ON agents(tasks_processed_count);

-- Update the updated_at trigger to also update last_action_at when relevant
CREATE OR REPLACE FUNCTION update_agent_activity()
RETURNS TRIGGER AS $$
BEGIN
    -- Update last_action_at when status or health_status changes
    IF OLD.status != NEW.status OR OLD.health_status != NEW.health_status THEN
        NEW.last_action_at = NOW();
    END IF;
    
    -- Update updated_at
    NEW.updated_at = NOW();
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Drop the old trigger and create the new one
DROP TRIGGER IF EXISTS update_agents_updated_at ON agents;
CREATE TRIGGER update_agents_activity 
    BEFORE UPDATE ON agents
    FOR EACH ROW 
    EXECUTE FUNCTION update_agent_activity();
