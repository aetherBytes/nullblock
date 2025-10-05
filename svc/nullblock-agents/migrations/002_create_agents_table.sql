-- Migration: Create agents table
-- Agents service owns full CRUD for agent registration and management

CREATE TABLE IF NOT EXISTS agents (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR NOT NULL,
    agent_type VARCHAR NOT NULL, -- hecate, arbitrage, social, portfolio, etc.
    description TEXT,
    status VARCHAR NOT NULL, -- active, inactive, maintenance, error

    -- Agent capabilities and configuration
    capabilities JSONB DEFAULT '[]'::jsonb, -- array of capability strings
    endpoint_url VARCHAR,
    metadata JSONB DEFAULT '{}'::jsonb,

    -- Performance and health monitoring
    performance_metrics JSONB DEFAULT '{}'::jsonb,
    last_health_check TIMESTAMPTZ,
    health_status VARCHAR DEFAULT 'unknown', -- healthy, unhealthy, unknown

    -- Activity tracking fields
    last_task_processed UUID,
    tasks_processed_count INTEGER DEFAULT 0,
    last_action_at TIMESTAMPTZ,
    average_processing_time BIGINT DEFAULT 0,
    total_processing_time BIGINT DEFAULT 0,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_agents_agent_type ON agents(agent_type);
CREATE INDEX IF NOT EXISTS idx_agents_status ON agents(status);
CREATE INDEX IF NOT EXISTS idx_agents_health_status ON agents(health_status);
CREATE INDEX IF NOT EXISTS idx_agents_last_health_check ON agents(last_health_check);
CREATE INDEX IF NOT EXISTS idx_agents_created_at ON agents(created_at);

-- Activity tracking indexes
CREATE INDEX IF NOT EXISTS idx_agents_last_action_at ON agents(last_action_at);
CREATE INDEX IF NOT EXISTS idx_agents_tasks_processed_count ON agents(tasks_processed_count);

-- Unique constraint on name within agent_type
CREATE UNIQUE INDEX IF NOT EXISTS idx_agents_name_type_unique ON agents(name, agent_type);

-- Enhanced update trigger with activity tracking
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

DROP TRIGGER IF EXISTS update_agents_activity ON agents;
CREATE TRIGGER update_agents_activity BEFORE UPDATE ON agents
    FOR EACH ROW EXECUTE FUNCTION update_agent_activity();