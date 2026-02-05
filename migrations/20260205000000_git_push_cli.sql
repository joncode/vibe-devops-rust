-- Migration: Git Push CLI tables
-- Apps, Builds, Artifacts, Releases, Runs for git-push deployment workflow

-- =============================================================================
-- ENUMS
-- =============================================================================

-- Build status enum
CREATE TYPE build_status AS ENUM (
    'queued',
    'building',
    'succeeded',
    'failed',
    'cancelled'
);

-- Release status enum
CREATE TYPE release_status AS ENUM (
    'pending',
    'deploying',
    'active',
    'failed',
    'rolled_back'
);

-- Run status enum
CREATE TYPE run_status AS ENUM (
    'starting',
    'running',
    'succeeded',
    'failed',
    'cancelled'
);

-- Artifact type enum
CREATE TYPE artifact_type AS ENUM (
    'slug',
    'oci-image'
);

-- =============================================================================
-- TABLES
-- =============================================================================

-- Applications
CREATE TABLE apps (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    hex_id VARCHAR(20) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL UNIQUE,
    repo_url TEXT,
    default_stack VARCHAR(100) DEFAULT 'paketo',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- Builds
CREATE TABLE builds (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    hex_id VARCHAR(20) NOT NULL UNIQUE,
    app_id UUID NOT NULL REFERENCES apps(id),
    source_sha VARCHAR(40) NOT NULL,
    source_ref VARCHAR(255),
    status build_status NOT NULL DEFAULT 'queued',
    output_artifact_digest VARCHAR(255),
    logs_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    finished_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ
);

-- Artifacts
CREATE TABLE artifacts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    hex_id VARCHAR(20) NOT NULL UNIQUE,
    digest VARCHAR(255) NOT NULL UNIQUE,
    artifact_type artifact_type NOT NULL,
    size_bytes BIGINT,
    uri TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

-- Releases
CREATE TABLE releases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    hex_id VARCHAR(20) NOT NULL UNIQUE,
    app_id UUID NOT NULL REFERENCES apps(id),
    build_id UUID NOT NULL REFERENCES builds(id),
    artifact_digest VARCHAR(255) NOT NULL,
    version INTEGER NOT NULL,
    config_snapshot JSONB,
    status release_status NOT NULL DEFAULT 'pending',
    process_formation JSONB DEFAULT '{"web": 1}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    UNIQUE(app_id, version)
);

-- Runs (one-off commands)
CREATE TABLE runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    hex_id VARCHAR(20) NOT NULL UNIQUE,
    app_id UUID NOT NULL REFERENCES apps(id),
    release_id UUID NOT NULL REFERENCES releases(id),
    command TEXT[] NOT NULL,
    status run_status NOT NULL DEFAULT 'starting',
    exit_code INTEGER,
    logs_stream_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    finished_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ
);

-- =============================================================================
-- INDEXES
-- =============================================================================

-- Apps indexes
CREATE INDEX idx_apps_name ON apps(name) WHERE deleted_at IS NULL;
CREATE INDEX idx_apps_hex_id ON apps(hex_id);
CREATE INDEX idx_apps_deleted_at ON apps(deleted_at) WHERE deleted_at IS NOT NULL;

-- Builds indexes
CREATE INDEX idx_builds_app_id ON builds(app_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_builds_hex_id ON builds(hex_id);
CREATE INDEX idx_builds_status ON builds(status) WHERE deleted_at IS NULL;
CREATE INDEX idx_builds_source_sha ON builds(app_id, source_sha) WHERE deleted_at IS NULL;
CREATE INDEX idx_builds_deleted_at ON builds(deleted_at) WHERE deleted_at IS NOT NULL;

-- Artifacts indexes
CREATE INDEX idx_artifacts_hex_id ON artifacts(hex_id);
CREATE INDEX idx_artifacts_digest ON artifacts(digest) WHERE deleted_at IS NULL;
CREATE INDEX idx_artifacts_deleted_at ON artifacts(deleted_at) WHERE deleted_at IS NOT NULL;

-- Releases indexes
CREATE INDEX idx_releases_app_id ON releases(app_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_releases_hex_id ON releases(hex_id);
CREATE INDEX idx_releases_build_id ON releases(build_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_releases_status ON releases(status) WHERE deleted_at IS NULL;
CREATE INDEX idx_releases_app_version ON releases(app_id, version DESC) WHERE deleted_at IS NULL;
CREATE INDEX idx_releases_deleted_at ON releases(deleted_at) WHERE deleted_at IS NOT NULL;

-- Runs indexes
CREATE INDEX idx_runs_app_id ON runs(app_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_runs_hex_id ON runs(hex_id);
CREATE INDEX idx_runs_release_id ON runs(release_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_runs_status ON runs(status) WHERE deleted_at IS NULL;
CREATE INDEX idx_runs_deleted_at ON runs(deleted_at) WHERE deleted_at IS NOT NULL;

-- =============================================================================
-- FUNCTIONS
-- =============================================================================

-- Function to get the next release version for an app
CREATE OR REPLACE FUNCTION next_release_version(p_app_id UUID)
RETURNS INTEGER AS $$
DECLARE
    v_next INTEGER;
BEGIN
    SELECT COALESCE(MAX(version), 0) + 1 INTO v_next
    FROM releases
    WHERE app_id = p_app_id;
    RETURN v_next;
END;
$$ LANGUAGE plpgsql;

-- Function to get current release for an app
CREATE OR REPLACE FUNCTION current_release(p_app_id UUID)
RETURNS UUID AS $$
DECLARE
    v_release_id UUID;
BEGIN
    SELECT id INTO v_release_id
    FROM releases
    WHERE app_id = p_app_id 
      AND status = 'active'
      AND deleted_at IS NULL
    ORDER BY version DESC
    LIMIT 1;
    RETURN v_release_id;
END;
$$ LANGUAGE plpgsql;
