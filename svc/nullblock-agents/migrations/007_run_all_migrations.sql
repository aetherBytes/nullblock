-- Migration: Run all database migrations in order
-- This script ensures all migrations are applied in the correct sequence

-- First, ensure we're in the correct database
\c nullblock_agents;

-- Run migrations in order
\i 001_create_tasks_table.sql
\i 002_create_agents_table.sql
\i 003_create_user_references_table.sql
\i 003_add_task_action_tracking.sql
\i 004_add_agent_activity_tracking.sql
\i 004_add_foreign_key_constraints.sql
\i 005_rename_tasks_to_task_executions.sql
\i 006_create_database_sync_functions.sql
\i 008_update_user_references_schema.sql

-- Verify the final schema
SELECT 
    table_name,
    column_name,
    data_type,
    is_nullable
FROM information_schema.columns 
WHERE table_name IN ('task_executions', 'agents', 'user_references')
ORDER BY table_name, ordinal_position;

-- Show indexes
SELECT 
    schemaname,
    tablename,
    indexname,
    indexdef
FROM pg_indexes 
WHERE tablename IN ('task_executions', 'agents', 'user_references')
ORDER BY tablename, indexname;
