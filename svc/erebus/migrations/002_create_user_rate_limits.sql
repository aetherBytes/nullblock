-- User Rate Limits Table
-- Tracks daily API usage per user per agent
-- Resets at midnight UTC

CREATE TABLE IF NOT EXISTS user_rate_limits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES user_references(id) ON DELETE CASCADE,
    agent_name VARCHAR NOT NULL,

    -- Daily usage tracking (resets at midnight UTC)
    daily_count INTEGER DEFAULT 0,
    last_reset_date DATE DEFAULT CURRENT_DATE,

    -- Configurable limits (default 100 calls/day for free tier)
    daily_limit INTEGER DEFAULT 100,

    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    -- One rate limit record per user per agent
    CONSTRAINT user_rate_limits_user_agent_unique UNIQUE (user_id, agent_name)
);

-- Index for efficient lookups
CREATE INDEX IF NOT EXISTS idx_user_rate_limits_user_agent ON user_rate_limits(user_id, agent_name);
CREATE INDEX IF NOT EXISTS idx_user_rate_limits_last_reset ON user_rate_limits(last_reset_date);
