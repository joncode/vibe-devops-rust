-- Migration: Server Infrastructure
CREATE TYPE server_status AS ENUM ('provisioning', 'active', 'degraded', 'offline', 'maintenance', 'decommissioned');
CREATE TYPE server_environment AS ENUM ('prod', 'testnet', 'staging', 'dev');
CREATE TYPE service_status AS ENUM ('unknown', 'running', 'degraded', 'stopped', 'failed', 'deploying', 'scaling');

CREATE TABLE servers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(), hex_id VARCHAR(16) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL, hostname VARCHAR(255) NOT NULL, ip_address INET NOT NULL,
    environment server_environment NOT NULL, status server_status NOT NULL DEFAULT 'provisioning',
    k3s_version VARCHAR(50), specs JSONB DEFAULT '{}'::jsonb, metadata JSONB DEFAULT '{}'::jsonb,
    last_heartbeat_at TIMESTAMPTZ, created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(), deleted_at TIMESTAMPTZ
);

CREATE TABLE server_services (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(), hex_id VARCHAR(16) NOT NULL UNIQUE,
    server_id UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    service_name VARCHAR(255) NOT NULL, namespace VARCHAR(255) NOT NULL,
    status service_status NOT NULL DEFAULT 'unknown',
    replicas_desired INTEGER, replicas_ready INTEGER, image_tag VARCHAR(255),
    health_endpoint VARCHAR(512), metadata JSONB DEFAULT '{}'::jsonb,
    last_deployed_at TIMESTAMPTZ, last_health_check_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(), updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(), deleted_at TIMESTAMPTZ,
    CONSTRAINT unique_service_per_server_namespace UNIQUE NULLS NOT DISTINCT (server_id, namespace, service_name, deleted_at)
);

CREATE INDEX idx_servers_active ON servers(environment, status) WHERE deleted_at IS NULL;
CREATE INDEX idx_servers_hex_id ON servers(hex_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_server_services_by_server ON server_services(server_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_server_services_hex_id ON server_services(hex_id) WHERE deleted_at IS NULL;

CREATE OR REPLACE FUNCTION update_updated_at_column() RETURNS TRIGGER AS $$ BEGIN NEW.updated_at = NOW(); RETURN NEW; END; $$ LANGUAGE plpgsql;
CREATE TRIGGER update_servers_updated_at BEFORE UPDATE ON servers FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_server_services_updated_at BEFORE UPDATE ON server_services FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
