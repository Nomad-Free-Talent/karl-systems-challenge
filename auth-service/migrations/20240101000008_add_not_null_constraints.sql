-- Add NOT NULL constraints to columns that should never be NULL
-- This migration fixes the schema to match the Rust struct definitions

-- Update any NULL values to defaults before adding constraints
UPDATE users SET created_at = NOW() WHERE created_at IS NULL;
UPDATE users SET updated_at = NOW() WHERE updated_at IS NULL;
UPDATE users SET is_active = TRUE WHERE is_active IS NULL;

UPDATE roles SET created_at = NOW() WHERE created_at IS NULL;

UPDATE permissions SET created_at = NOW() WHERE created_at IS NULL;

-- Add NOT NULL constraints
ALTER TABLE users 
    ALTER COLUMN created_at SET NOT NULL,
    ALTER COLUMN updated_at SET NOT NULL,
    ALTER COLUMN is_active SET NOT NULL;

ALTER TABLE roles 
    ALTER COLUMN created_at SET NOT NULL;

ALTER TABLE permissions 
    ALTER COLUMN created_at SET NOT NULL;

