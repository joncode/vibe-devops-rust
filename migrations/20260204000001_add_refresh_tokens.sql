-- Add refresh tokens table for token rotation and refresh flows

CREATE TABLE IF NOT EXISTS app_refresh_tokens (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES app_users(id) ON DELETE CASCADE,
    token_hash      VARCHAR(128) NOT NULL UNIQUE,
    session_id      UUID REFERENCES app_session_tokens(id) ON DELETE SET NULL,
    family_id       UUID NOT NULL,
    generation      INTEGER NOT NULL DEFAULT 1,
    ip_address      VARCHAR(45),  -- IPv6 max length
    user_agent      TEXT,
    expires_at      TIMESTAMPTZ NOT NULL,
    revoked_at      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for refresh tokens
CREATE INDEX IF NOT EXISTS idx_app_refresh_tokens_user_id ON app_refresh_tokens(user_id);
CREATE INDEX IF NOT EXISTS idx_app_refresh_tokens_family ON app_refresh_tokens(family_id);
CREATE INDEX IF NOT EXISTS idx_app_refresh_tokens_active ON app_refresh_tokens(user_id, expires_at) WHERE revoked_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_app_refresh_tokens_cleanup ON app_refresh_tokens(expires_at) WHERE revoked_at IS NULL;
