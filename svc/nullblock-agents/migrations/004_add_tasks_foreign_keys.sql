-- Migration: Add foreign key constraints to tasks table
-- This migration must run after migrations 001, 002, and 003
-- It adds the foreign key constraints that reference agents and user_references tables

-- Tasks -> User References relationship
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'fk_tasks_user_id'
    ) THEN
        ALTER TABLE tasks
        ADD CONSTRAINT fk_tasks_user_id
        FOREIGN KEY (user_id) REFERENCES user_references(id)
        ON DELETE SET NULL;
    END IF;
END $$;

-- Tasks -> Agents relationship
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'fk_tasks_assigned_agent_id'
    ) THEN
        ALTER TABLE tasks
        ADD CONSTRAINT fk_tasks_assigned_agent_id
        FOREIGN KEY (assigned_agent_id) REFERENCES agents(id)
        ON DELETE SET NULL;
    END IF;
END $$;
