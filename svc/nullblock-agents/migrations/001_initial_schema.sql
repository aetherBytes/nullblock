-- Agents Initial Schema
-- Tables: tasks, agents, user_references (read-only replica)
-- user_references is synced from Erebus via logical replication

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- =============================================================================
-- TASKS (A2A Protocol v0.3.0 compliant)
-- =============================================================================

CREATE TABLE IF NOT EXISTS tasks (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR NOT NULL,
    description TEXT,
    task_type VARCHAR NOT NULL,
    category VARCHAR NOT NULL,

    -- A2A Protocol fields
    context_id UUID NOT NULL DEFAULT uuid_generate_v4(),
    kind VARCHAR NOT NULL DEFAULT 'task',
    status VARCHAR NOT NULL CHECK (status IN ('submitted', 'working', 'input-required', 'completed', 'canceled', 'failed', 'rejected', 'auth-required', 'unknown')),
    status_message TEXT,
    status_timestamp TIMESTAMPTZ,
    history JSONB DEFAULT '[]'::jsonb,
    artifacts JSONB DEFAULT '[]'::jsonb,

    priority VARCHAR NOT NULL,
    user_id UUID,
    assigned_agent_id UUID,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,

    -- Progress
    progress SMALLINT DEFAULT 0 CHECK (progress >= 0 AND progress <= 100),
    estimated_duration BIGINT,
    actual_duration BIGINT,

    -- Task configuration
    sub_tasks JSONB DEFAULT '[]'::jsonb,
    dependencies JSONB DEFAULT '[]'::jsonb,
    context JSONB DEFAULT '{}'::jsonb,
    parameters JSONB DEFAULT '{}'::jsonb,
    outcome JSONB,
    logs JSONB DEFAULT '[]'::jsonb,
    triggers JSONB DEFAULT '[]'::jsonb,
    required_capabilities JSONB DEFAULT '[]'::jsonb,

    -- Retry settings
    auto_retry BOOLEAN DEFAULT true,
    max_retries INTEGER DEFAULT 3,
    current_retries INTEGER DEFAULT 0,
    user_approval_required BOOLEAN DEFAULT false,
    user_notifications BOOLEAN DEFAULT true,

    -- Action tracking
    actioned_at TIMESTAMPTZ,
    action_result TEXT,
    action_metadata JSONB DEFAULT '{}'::jsonb,
    action_duration BIGINT,

    -- Source tracking
    source_identifier VARCHAR,
    source_metadata JSONB DEFAULT '{}'::jsonb
);

CREATE INDEX IF NOT EXISTS idx_tasks_user_id ON tasks(user_id);
CREATE INDEX IF NOT EXISTS idx_tasks_assigned_agent_id ON tasks(assigned_agent_id);
CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
CREATE INDEX IF NOT EXISTS idx_tasks_context_id ON tasks(context_id);
CREATE INDEX IF NOT EXISTS idx_tasks_created_at ON tasks(created_at);

-- =============================================================================
-- AGENTS (agent registration and management)
-- =============================================================================

CREATE TABLE IF NOT EXISTS agents (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR NOT NULL,
    agent_type VARCHAR NOT NULL,
    description TEXT,
    status VARCHAR NOT NULL,
    capabilities JSONB DEFAULT '[]'::jsonb,
    endpoint_url VARCHAR,
    metadata JSONB DEFAULT '{}'::jsonb,
    performance_metrics JSONB DEFAULT '{}'::jsonb,
    last_health_check TIMESTAMPTZ,
    health_status VARCHAR DEFAULT 'unknown',
    last_task_processed UUID,
    tasks_processed_count INTEGER DEFAULT 0,
    last_action_at TIMESTAMPTZ,
    average_processing_time BIGINT DEFAULT 0,
    total_processing_time BIGINT DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_agents_agent_type ON agents(agent_type);
CREATE INDEX IF NOT EXISTS idx_agents_status ON agents(status);
CREATE UNIQUE INDEX IF NOT EXISTS idx_agents_name_type_unique ON agents(name, agent_type);

-- =============================================================================
-- USER REFERENCES (read-only replica from Erebus)
-- =============================================================================

CREATE TABLE IF NOT EXISTS user_references (
    id UUID PRIMARY KEY,
    source_identifier VARCHAR(255) NOT NULL,
    network VARCHAR(50) NOT NULL,
    user_type VARCHAR(50) DEFAULT 'external',
    email VARCHAR(255),
    metadata JSONB DEFAULT '{}'::jsonb,
    preferences JSONB DEFAULT '{}'::jsonb,
    source_type JSONB DEFAULT '{"type": "web3_wallet", "provider": "unknown"}'::jsonb,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    is_active BOOLEAN DEFAULT true
);

CREATE INDEX IF NOT EXISTS idx_user_references_source ON user_references(source_identifier, network);
CREATE INDEX IF NOT EXISTS idx_user_references_email ON user_references(email) WHERE email IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS user_references_source_network_unique
    ON user_references(source_identifier, network) WHERE is_active = true;

COMMENT ON TABLE user_references IS 'READ-ONLY: Synced from Erebus via logical replication. All user CRUD goes through Erebus /api/users/*';

-- =============================================================================
-- FOREIGN KEYS
-- =============================================================================

ALTER TABLE tasks ADD CONSTRAINT fk_tasks_user_id
    FOREIGN KEY (user_id) REFERENCES user_references(id) ON DELETE SET NULL;

ALTER TABLE tasks ADD CONSTRAINT fk_tasks_assigned_agent_id
    FOREIGN KEY (assigned_agent_id) REFERENCES agents(id) ON DELETE SET NULL;

-- =============================================================================
-- TRIGGER FUNCTIONS
-- =============================================================================

CREATE OR REPLACE FUNCTION update_updated_at_column() RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS update_tasks_updated_at ON tasks;
CREATE TRIGGER update_tasks_updated_at
    BEFORE UPDATE ON tasks FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_agents_updated_at ON agents;
CREATE TRIGGER update_agents_updated_at
    BEFORE UPDATE ON agents FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
