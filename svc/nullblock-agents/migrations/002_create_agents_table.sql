-- Migration: Create agents table
-- Agents service owns full CRUD for agent registration and management

CREATE TABLE agents (
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

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_agents_agent_type ON agents(agent_type);
CREATE INDEX idx_agents_status ON agents(status);
CREATE INDEX idx_agents_health_status ON agents(health_status);
CREATE INDEX idx_agents_last_health_check ON agents(last_health_check);
CREATE INDEX idx_agents_created_at ON agents(created_at);

-- Unique constraint on name within agent_type
CREATE UNIQUE INDEX idx_agents_name_type_unique ON agents(name, agent_type);

-- Update trigger for updated_at
CREATE TRIGGER update_agents_updated_at BEFORE UPDATE ON agents
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();