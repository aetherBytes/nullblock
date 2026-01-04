-- Agents Replication Subscription
-- Syncs user_references from Erebus database
-- Requires: Erebus must have publication 'erebus_user_sync' created
-- Requires: PostgreSQL containers on nullblock-network

DROP SUBSCRIPTION IF EXISTS agents_user_sync;

CREATE SUBSCRIPTION agents_user_sync
CONNECTION 'host=nullblock-postgres-erebus port=5432 dbname=erebus user=postgres password=postgres_secure_pass'
PUBLICATION erebus_user_sync
WITH (
    copy_data = true,
    create_slot = true,
    enabled = true,
    slot_name = 'agents_user_sync'
);
