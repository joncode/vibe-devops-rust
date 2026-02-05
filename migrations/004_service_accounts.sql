-- Service Accounts for Claude Agent Deployment
CREATE TABLE app_service_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    hex_id VARCHAR(20) NOT NULL UNIQUE,
    user_id UUID NOT NULL REFERENCES app_users(id),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    api_key_hash VARCHAR(255) NOT NULL,
    api_key_prefix VARCHAR(10) NOT NULL,
    permissions JSONB NOT NULL DEFAULT '[]'::JSONB,
    rate_limit_per_minute INTEGER NOT NULL DEFAULT 60,
    last_used_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);
CREATE INDEX idx_sa_user ON app_service_accounts(user_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_sa_prefix ON app_service_accounts(api_key_prefix) WHERE deleted_at IS NULL;

CREATE TABLE app_api_key_usage (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    hex_id VARCHAR(20) NOT NULL UNIQUE,
    service_account_id UUID NOT NULL REFERENCES app_service_accounts(id),
    endpoint VARCHAR(255) NOT NULL,
    method VARCHAR(10) NOT NULL,
    status_code INTEGER NOT NULL,
    response_time_ms INTEGER,
    ip_address VARCHAR(45),
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);
CREATE INDEX idx_aku_sa ON app_api_key_usage(service_account_id) WHERE deleted_at IS NULL;
