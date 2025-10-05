-- Migration: Create tasks table aligned with A2A Protocol v0.3.0
-- Agents service owns full CRUD for task management
-- user_id references user_references.id (synced from Erebus)
-- assigned_agent_id references agents.id

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE tasks (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR NOT NULL,
    description TEXT,
    task_type VARCHAR NOT NULL, -- maps to TaskType enum: arbitrage, social, portfolio, mcp, system, user_assigned
    category VARCHAR NOT NULL,  -- maps to TaskCategory enum: autonomous, user_assigned, system_generated, event_triggered

    -- A2A Protocol required fields
    context_id UUID NOT NULL DEFAULT uuid_generate_v4(), -- groups related tasks
    kind VARCHAR NOT NULL DEFAULT 'task', -- always "task" per A2A spec
    status VARCHAR NOT NULL CHECK (status IN ('submitted', 'working', 'input-required', 'completed', 'canceled', 'failed', 'rejected', 'auth-required', 'unknown')), -- A2A TaskState
    status_message TEXT, -- optional human-readable status message
    status_timestamp TIMESTAMPTZ, -- when status was last updated

    -- A2A Protocol optional fields
    history JSONB DEFAULT '[]'::jsonb, -- Message[] array per A2A spec
    artifacts JSONB DEFAULT '[]'::jsonb, -- Artifact[] array per A2A spec

    priority VARCHAR NOT NULL,  -- maps to TaskPriority enum: low, medium, high, urgent, critical
    user_id UUID,              -- reference to user_references.id (synced from Erebus)
    assigned_agent_id UUID,     -- reference to agents.id

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,

    -- Progress and duration
    progress SMALLINT DEFAULT 0 CHECK (progress >= 0 AND progress <= 100),
    estimated_duration BIGINT, -- milliseconds
    actual_duration BIGINT,    -- milliseconds

    -- Task relationships and configuration
    sub_tasks JSONB DEFAULT '[]'::jsonb,
    dependencies JSONB DEFAULT '[]'::jsonb,
    context JSONB DEFAULT '{}'::jsonb,
    parameters JSONB DEFAULT '{}'::jsonb,
    outcome JSONB,
    logs JSONB DEFAULT '[]'::jsonb,
    triggers JSONB DEFAULT '[]'::jsonb,
    required_capabilities JSONB DEFAULT '[]'::jsonb,

    -- Retry and notification settings
    auto_retry BOOLEAN DEFAULT true,
    max_retries INTEGER DEFAULT 3,
    current_retries INTEGER DEFAULT 0,
    user_approval_required BOOLEAN DEFAULT false,
    user_notifications BOOLEAN DEFAULT true,

    -- Action tracking fields (when tasks are processed by agents)
    actioned_at TIMESTAMPTZ,
    action_result TEXT,
    action_metadata JSONB DEFAULT '{}'::jsonb,
    action_duration BIGINT, -- milliseconds

    -- Source tracking fields
    source_identifier VARCHAR,
    source_metadata JSONB DEFAULT '{}'::jsonb
);

-- Indexes for performance
CREATE INDEX idx_tasks_user_id ON tasks(user_id);
CREATE INDEX idx_tasks_assigned_agent_id ON tasks(assigned_agent_id);
CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_tasks_context_id ON tasks(context_id);
CREATE INDEX idx_tasks_kind ON tasks(kind);
CREATE INDEX idx_tasks_task_type ON tasks(task_type);
CREATE INDEX idx_tasks_category ON tasks(category);
CREATE INDEX idx_tasks_priority ON tasks(priority);
CREATE INDEX idx_tasks_created_at ON tasks(created_at);
CREATE INDEX idx_tasks_updated_at ON tasks(updated_at);

-- Action tracking indexes
CREATE INDEX idx_tasks_actioned_at ON tasks(actioned_at);
CREATE INDEX idx_tasks_action_result ON tasks(action_result) WHERE action_result IS NOT NULL;
CREATE INDEX idx_tasks_action_duration ON tasks(action_duration);
CREATE INDEX idx_tasks_unprocessed ON tasks(status, actioned_at) WHERE actioned_at IS NULL;
CREATE INDEX idx_tasks_agent_actioned ON tasks(assigned_agent_id, actioned_at);

-- Foreign key constraints moved to migration 004_add_tasks_foreign_keys.sql
-- They will be added after all tables are created

-- Update trigger for updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_tasks_updated_at BEFORE UPDATE ON tasks
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();