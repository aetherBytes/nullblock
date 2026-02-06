CREATE TABLE IF NOT EXISTS content_queue (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    theme VARCHAR(50) NOT NULL,
    text TEXT NOT NULL,
    tags TEXT[] NOT NULL DEFAULT '{}',
    image_prompt TEXT,
    image_path TEXT,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    scheduled_at TIMESTAMPTZ,
    posted_at TIMESTAMPTZ,
    tweet_url TEXT,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_content_queue_status ON content_queue(status);
CREATE INDEX IF NOT EXISTS idx_content_queue_theme ON content_queue(theme);
CREATE INDEX IF NOT EXISTS idx_content_queue_created_at ON content_queue(created_at);
