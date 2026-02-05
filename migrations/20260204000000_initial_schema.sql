-- Initial schema for Vibe DevOps Server
-- Creates core tables for users, identifiers, and sessions

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "citext";

-- User status enum
DO $$ BEGIN
    CREATE TYPE user_status AS ENUM ('active', 'inactive', 'suspended', 'pending');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- Identifier type enum
DO $$ BEGIN
    CREATE TYPE identifier_type AS ENUM (
        'email', 'phone', 'wallet', 
        'oauth_google', 'oauth_github', 'oauth_apple', 'username'
    );
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- =============================================================================
-- APP USER TABLES
-- =============================================================================

-- Main app users table
CREATE TABLE IF NOT EXISTS app_users (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    display_name    VARCHAR(255),
    avatar_url      TEXT,
    metadata        JSONB NOT NULL DEFAULT '{}',
    status          user_status NOT NULL DEFAULT 'pending',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ,
    
    CONSTRAINT app_users_display_name_length CHECK (
        display_name IS NULL OR char_length(display_name) >= 1
    )
);

-- Social identifiers (email, phone, wallet, oauth)
CREATE TABLE IF NOT EXISTS app_social_identifiers (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id             UUID NOT NULL REFERENCES app_users(id) ON DELETE CASCADE,
    identifier_type     identifier_type NOT NULL,
    identifier_value    CITEXT NOT NULL,
    verified            BOOLEAN NOT NULL DEFAULT FALSE,
    verified_at         TIMESTAMPTZ,
    is_primary          BOOLEAN NOT NULL DEFAULT FALSE,
    metadata            JSONB NOT NULL DEFAULT '{}',
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT unique_identifier UNIQUE (identifier_type, identifier_value),
    CONSTRAINT identifier_value_not_empty CHECK (char_length(identifier_value) >= 1)
);

-- Password storage (separate for security)
CREATE TABLE IF NOT EXISTS app_user_passwords (
    user_id         UUID PRIMARY KEY REFERENCES app_users(id) ON DELETE CASCADE,
    password_hash   VARCHAR(255) NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Session tokens
CREATE TABLE IF NOT EXISTS app_session_tokens (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES app_users(id) ON DELETE CASCADE,
    token_hash      VARCHAR(128) NOT NULL UNIQUE,
    device_info     JSONB NOT NULL DEFAULT '{}',
    ip_address      VARCHAR(45),  -- IPv6 max length
    user_agent      TEXT,
    expires_at      TIMESTAMPTZ NOT NULL,
    revoked_at      TIMESTAMPTZ,
    last_used_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- =============================================================================
-- INDEXES
-- =============================================================================

-- App Users indexes
CREATE INDEX IF NOT EXISTS idx_app_users_status ON app_users(status) WHERE deleted_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_app_users_created_at ON app_users(created_at);
CREATE INDEX IF NOT EXISTS idx_app_users_metadata ON app_users USING GIN (metadata);

-- App Social Identifiers indexes
CREATE INDEX IF NOT EXISTS idx_app_social_identifiers_user_id ON app_social_identifiers(user_id);
CREATE INDEX IF NOT EXISTS idx_app_social_identifiers_lookup ON app_social_identifiers(identifier_type, identifier_value);
CREATE INDEX IF NOT EXISTS idx_app_social_identifiers_primary ON app_social_identifiers(user_id) WHERE is_primary = TRUE;

-- App Session Tokens indexes
CREATE INDEX IF NOT EXISTS idx_app_session_tokens_user_id ON app_session_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_app_session_tokens_active ON app_session_tokens(user_id, expires_at) WHERE revoked_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_app_session_tokens_cleanup ON app_session_tokens(expires_at) WHERE revoked_at IS NULL;

-- =============================================================================
-- FUNCTIONS & TRIGGERS
-- =============================================================================

-- Auto-update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply to tables with updated_at
DROP TRIGGER IF EXISTS update_app_users_updated_at ON app_users;
CREATE TRIGGER update_app_users_updated_at
    BEFORE UPDATE ON app_users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_app_social_identifiers_updated_at ON app_social_identifiers;
CREATE TRIGGER update_app_social_identifiers_updated_at
    BEFORE UPDATE ON app_social_identifiers
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

DROP TRIGGER IF EXISTS update_app_user_passwords_updated_at ON app_user_passwords;
CREATE TRIGGER update_app_user_passwords_updated_at
    BEFORE UPDATE ON app_user_passwords
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
