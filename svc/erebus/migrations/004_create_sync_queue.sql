-- Migration: Create sync queue for user data synchronization
-- This migration creates a sync queue table that can be processed by the application
-- to sync user data from Erebus to Agents database

-- Create sync queue table
CREATE TABLE IF NOT EXISTS user_sync_queue (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    operation VARCHAR NOT NULL, -- 'INSERT', 'UPDATE', 'DELETE'
    user_data JSONB NOT NULL,
    processed BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    processed_at TIMESTAMPTZ
);

-- Create indexes for the sync queue
CREATE INDEX idx_user_sync_queue_processed ON user_sync_queue(processed);
CREATE INDEX idx_user_sync_queue_created_at ON user_sync_queue(created_at);
CREATE INDEX idx_user_sync_queue_user_id ON user_sync_queue(user_id);

-- Create function to add user to sync queue
CREATE OR REPLACE FUNCTION queue_user_sync(
    p_user_id UUID,
    p_operation VARCHAR,
    p_user_data JSONB
) RETURNS UUID AS $$
DECLARE
    queue_id UUID;
BEGIN
    INSERT INTO user_sync_queue (user_id, operation, user_data)
    VALUES (p_user_id, p_operation, p_user_data)
    RETURNING id INTO queue_id;
    
    RETURN queue_id;
END;
$$ LANGUAGE plpgsql;

-- Create trigger to automatically queue user changes
CREATE OR REPLACE FUNCTION trigger_queue_user_sync() RETURNS TRIGGER AS $$
DECLARE
    user_data JSONB;
    operation VARCHAR;
BEGIN
    -- Determine operation type
    IF TG_OP = 'DELETE' THEN
        operation := 'DELETE';
        user_data := to_jsonb(OLD);
    ELSE
        operation := TG_OP;
        user_data := to_jsonb(NEW);
    END IF;
    
    -- Queue the sync operation
    PERFORM queue_user_sync(
        COALESCE(NEW.id, OLD.id),
        operation,
        user_data
    );
    
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

-- Create trigger on user_references table
DROP TRIGGER IF EXISTS trigger_queue_user_sync ON user_references;
CREATE TRIGGER trigger_queue_user_sync
    AFTER INSERT OR UPDATE OR DELETE ON user_references
    FOR EACH ROW
    EXECUTE FUNCTION trigger_queue_user_sync();

-- Add comments for documentation
COMMENT ON TABLE user_sync_queue IS 'Queue for syncing user data from Erebus to Agents database';
COMMENT ON FUNCTION queue_user_sync IS 'Adds user changes to sync queue for processing';
COMMENT ON FUNCTION trigger_queue_user_sync IS 'Automatically queues user changes for sync';

