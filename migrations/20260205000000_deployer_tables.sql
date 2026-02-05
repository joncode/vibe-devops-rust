-- Migration: Add deployments and deployment_events tables
-- Part of the Deployer Agent feature for orchestrating GitOps deployments

-- =============================================================================
-- ENUMS
-- =============================================================================

-- Deployment status enum
CREATE TYPE deployment_status AS ENUM (
    'pending',
    'planning',
    'planned',
    'pr_created',
    'deploying',
    'verifying',
    'succeeded',
    'failed',
    'cancelled'
);

-- Deployment event type enum
CREATE TYPE deployment_event_type AS ENUM (
    'created',
    'plan_started',
    'plan_completed',
    'plan_failed',
    'pr_created',
    'pr_merged',
    'pr_closed',
    'deploy_started',
    'deploy_progressing',
    'verify_started',
    'verify_passed',
    'verify_failed',
    'succeeded',
    'failed',
    'cancelled',
    'rollback_started',
    'rollback_completed',
    'comment'
);

-- =============================================================================
-- TABLES
-- =============================================================================

-- Deployments table
CREATE TABLE deployments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    hex_id VARCHAR(20) NOT NULL UNIQUE,
    app_name VARCHAR(255) NOT NULL,
    environment VARCHAR(50) NOT NULL,
    target_sha VARCHAR(40),
    target_ref VARCHAR(255),
    status deployment_status NOT NULL DEFAULT 'pending',
    pr_url TEXT,
    pr_number INTEGER,
    pr_branch VARCHAR(255),
    plan_json JSONB,
    triggered_by VARCHAR(255),
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ,
    CONSTRAINT deployments_sha_or_ref CHECK (target_sha IS NOT NULL OR target_ref IS NOT NULL)
);

-- Deployment events table (audit log)
CREATE TABLE deployment_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    hex_id VARCHAR(20) NOT NULL UNIQUE,
    deployment_id UUID NOT NULL REFERENCES deployments(id),
    event_type deployment_event_type NOT NULL,
    event_data JSONB,
    message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- =============================================================================
-- INDEXES
-- =============================================================================

CREATE INDEX idx_deployments_hex_id ON deployments(hex_id);
CREATE INDEX idx_deployments_app_env ON deployments(app_name, environment);
CREATE INDEX idx_deployments_status ON deployments(status) WHERE deleted_at IS NULL;
CREATE INDEX idx_deployments_active ON deployments(created_at DESC) WHERE deleted_at IS NULL;
CREATE INDEX idx_deployment_events_hex_id ON deployment_events(hex_id);
CREATE INDEX idx_deployment_events_deployment_id ON deployment_events(deployment_id);
CREATE INDEX idx_deployment_events_type ON deployment_events(event_type);
