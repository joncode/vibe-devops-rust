-- ChatGPT Agent Deployment Tables
CREATE TYPE agent_task_status AS ENUM ('pending','running','completed','failed','cancelled');
CREATE TYPE agent_tool_status AS ENUM ('pending','running','completed','failed');

CREATE TABLE agent_tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    hex_id VARCHAR(20) NOT NULL UNIQUE,
    agent_type VARCHAR(50) NOT NULL,
    task_type VARCHAR(50) NOT NULL,
    input_message TEXT NOT NULL,
    status agent_task_status NOT NULL DEFAULT 'pending',
    tool_calls JSONB NOT NULL DEFAULT '[]',
    result JSONB,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ
);

CREATE TABLE agent_tool_executions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    hex_id VARCHAR(20) NOT NULL UNIQUE,
    task_id UUID NOT NULL REFERENCES agent_tasks(id) ON DELETE CASCADE,
    tool_name VARCHAR(100) NOT NULL,
    tool_input JSONB NOT NULL,
    tool_output JSONB,
    status agent_tool_status NOT NULL DEFAULT 'pending',
    duration_ms INTEGER,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_agent_tasks_hex_id ON agent_tasks(hex_id);
CREATE INDEX idx_agent_tasks_status ON agent_tasks(status) WHERE deleted_at IS NULL;
CREATE INDEX idx_agent_tasks_created_at ON agent_tasks(created_at DESC) WHERE deleted_at IS NULL;
CREATE INDEX idx_agent_tool_executions_task_id ON agent_tool_executions(task_id) WHERE deleted_at IS NULL;
