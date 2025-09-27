-- Migration: Add foreign key constraints
-- Now that all tables exist, we can add the foreign key relationships

-- Tasks -> User References relationship
ALTER TABLE tasks
ADD CONSTRAINT fk_tasks_user_id
FOREIGN KEY (user_id) REFERENCES user_references(id)
ON DELETE SET NULL; -- If user is deleted, set task user_id to NULL but keep task

-- Tasks -> Agents relationship
ALTER TABLE tasks
ADD CONSTRAINT fk_tasks_assigned_agent_id
FOREIGN KEY (assigned_agent_id) REFERENCES agents(id)
ON DELETE SET NULL; -- If agent is deleted, unassign from task but keep task