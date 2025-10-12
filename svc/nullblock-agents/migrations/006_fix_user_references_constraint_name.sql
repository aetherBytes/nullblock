-- Migration: Fix user_references primary key constraint name
-- This fixes installations where the table was previously named erebus_user_references
-- and the constraint name didn't update when the table was renamed

-- Check if old constraint name exists and rename it
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'erebus_user_references_pkey'
        AND conrelid = 'user_references'::regclass
    ) THEN
        ALTER TABLE user_references
        RENAME CONSTRAINT erebus_user_references_pkey TO user_references_pkey;

        RAISE NOTICE 'Renamed constraint erebus_user_references_pkey to user_references_pkey';
    ELSE
        RAISE NOTICE 'Constraint erebus_user_references_pkey does not exist, no action needed';
    END IF;
END $$;

-- Verify constraint name
SELECT conname, contype
FROM pg_constraint
WHERE conrelid = 'user_references'::regclass
AND contype = 'p';
