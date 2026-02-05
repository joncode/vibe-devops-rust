-- Migration: Add soft-delete and hex_id to all tables
-- This migration ensures no data is ever truly deleted and provides
-- human-readable, URL-safe IDs for all records

-- =============================================================================
-- SOFT DELETE: Add deleted_at to tables that don't have it
-- =============================================================================

-- app_social_identifiers: add deleted_at
ALTER TABLE app_social_identifiers 
ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

-- app_user_passwords: add deleted_at
ALTER TABLE app_user_passwords 
ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

-- app_session_tokens: add deleted_at (separate from revoked_at)
-- revoked_at = session invalidated but retained for audit
-- deleted_at = soft-deleted record
ALTER TABLE app_session_tokens 
ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

-- app_refresh_tokens: add deleted_at
ALTER TABLE app_refresh_tokens 
ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;

-- =============================================================================
-- HEX_ID: Add hex_id column to all tables
-- Format: {prefix}_{10 random chars from safe charset}
-- Safe charset: a-k, m-z, 0, 2-9 (34 chars, no l/1 confusion)
-- =============================================================================

-- app_users: hex_id (prefix: usr_)
ALTER TABLE app_users 
ADD COLUMN IF NOT EXISTS hex_id VARCHAR(16) UNIQUE;

-- app_social_identifiers: hex_id (prefix: sid_)
ALTER TABLE app_social_identifiers 
ADD COLUMN IF NOT EXISTS hex_id VARCHAR(16) UNIQUE;

-- app_user_passwords: hex_id (prefix: pwd_)
ALTER TABLE app_user_passwords 
ADD COLUMN IF NOT EXISTS hex_id VARCHAR(16) UNIQUE;

-- app_session_tokens: hex_id (prefix: stk_)
ALTER TABLE app_session_tokens 
ADD COLUMN IF NOT EXISTS hex_id VARCHAR(16) UNIQUE;

-- app_refresh_tokens: hex_id (prefix: rtk_)
ALTER TABLE app_refresh_tokens 
ADD COLUMN IF NOT EXISTS hex_id VARCHAR(16) UNIQUE;

-- =============================================================================
-- INDEXES for soft-delete and hex_id
-- =============================================================================

-- Soft-delete indexes (partial indexes for active records only)
CREATE INDEX IF NOT EXISTS idx_app_social_identifiers_active 
ON app_social_identifiers(user_id) WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_app_user_passwords_active 
ON app_user_passwords(user_id) WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_app_session_tokens_soft_deleted 
ON app_session_tokens(deleted_at) WHERE deleted_at IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_app_refresh_tokens_soft_deleted 
ON app_refresh_tokens(deleted_at) WHERE deleted_at IS NOT NULL;

-- hex_id indexes (for lookups by public ID)
CREATE INDEX IF NOT EXISTS idx_app_users_hex_id 
ON app_users(hex_id) WHERE hex_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_app_social_identifiers_hex_id 
ON app_social_identifiers(hex_id) WHERE hex_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_app_user_passwords_hex_id 
ON app_user_passwords(hex_id) WHERE hex_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_app_session_tokens_hex_id 
ON app_session_tokens(hex_id) WHERE hex_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_app_refresh_tokens_hex_id 
ON app_refresh_tokens(hex_id) WHERE hex_id IS NOT NULL;

-- =============================================================================
-- BACKFILL: Generate hex_ids for existing records
-- Uses PostgreSQL's random() for initial backfill
-- Production inserts will use application-generated secure random IDs
-- =============================================================================

-- Helper function to generate safe random string (for backfill only)
CREATE OR REPLACE FUNCTION generate_safe_random_string(length INTEGER DEFAULT 10)
RETURNS TEXT AS $$
DECLARE
    -- Safe chars: a-k, m-z, 0, 2-9 (no l or 1)
    chars TEXT := 'abcdefghijkmnopqrstuvwxyz023456789';
    result TEXT := '';
    i INTEGER;
BEGIN
    FOR i IN 1..length LOOP
        result := result || substr(chars, floor(random() * length(chars) + 1)::integer, 1);
    END LOOP;
    RETURN result;
END;
$$ LANGUAGE plpgsql;

-- Backfill hex_ids for existing records
UPDATE app_users SET hex_id = 'usr_' || generate_safe_random_string(10) WHERE hex_id IS NULL;
UPDATE app_social_identifiers SET hex_id = 'sid_' || generate_safe_random_string(10) WHERE hex_id IS NULL;
UPDATE app_user_passwords SET hex_id = 'pwd_' || generate_safe_random_string(10) WHERE hex_id IS NULL;
UPDATE app_session_tokens SET hex_id = 'stk_' || generate_safe_random_string(10) WHERE hex_id IS NULL;
UPDATE app_refresh_tokens SET hex_id = 'rtk_' || generate_safe_random_string(10) WHERE hex_id IS NULL;

-- Make hex_id NOT NULL after backfill (for future inserts)
-- Note: This may fail if there are concurrent inserts during migration
-- In production, do this in a separate migration after backfill is verified
ALTER TABLE app_users ALTER COLUMN hex_id SET NOT NULL;
ALTER TABLE app_social_identifiers ALTER COLUMN hex_id SET NOT NULL;
ALTER TABLE app_user_passwords ALTER COLUMN hex_id SET NOT NULL;
ALTER TABLE app_session_tokens ALTER COLUMN hex_id SET NOT NULL;
ALTER TABLE app_refresh_tokens ALTER COLUMN hex_id SET NOT NULL;

-- Drop the helper function (no longer needed after backfill)
DROP FUNCTION IF EXISTS generate_safe_random_string(INTEGER);
