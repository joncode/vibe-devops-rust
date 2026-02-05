-- Migration: Add agent_tasks and agent_tool_executions tables
-- Supports ChatGPT/Claude agent deployments with auditable tool execution

-- =============================================================================
-- AGENT TASK STATUS ENUM
-- =============================================================================

DO $$ BEGIN
    CREATE TYPE agent_task_status AS ENUM (
        'pending',      -- Task created, not yet started
        'running',      -- Agent actively processing
        'waiting',      -- Waiting for tool results
        'completed',    -- Successfully completed
        'failed',       -- Failed with error
        'cancelled'     -- Manually cancelled
    );
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- =============================================================================
-- TOOL EXECUTION STATUS ENUM
-- =============================================================================

DO $$ BEGIN
    CREATE TYPE tool_execution_status AS ENUM (
        'pending',      -- Tool queued
        'running',      -- Tool executing
        'completed',    -- Tool completed successfully
        'failed'        -- Tool failed with error
    );
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- =============================================================================
-- AGENT TASKS TABLE
-- =============================================================================

CREATE TABLE IF NOT EXISTS agent_tasks (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    hex_id          VARCHAR(20) NOT NULL UNIQUE,
    
    -- Agent info
    agent_type      VARCHAR(50) NOT NULL,  -- 'chatgpt', 'claude', etc.
    task_type       VARCHAR(50) NOT NULL,  -- 'deploy', 'rollback', 'status', 'custom'
    
    -- Input/Output
    input_message   TEXT NOT NULL,
    context         JSONB NOT NULL DEFAULT '{}',  -- Additional context (env, app, etc.)
    
    -- Status tracking
    status          agent_task_status NOT NULL DEFAULT 'pending',
    
    -- Results
    tool_calls      JSONB NOT NULL DEFAULT '[]',  -- Array of tool call summaries
    result          JSONB,                         -- Final result
    error_message   TEXT,                          -- Error message if failed
    
    -- Timestamps
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at      TIMESTAMPTZ,                   -- When processing began
    completed_at    TIMESTAMPTZ,                   -- When task finished
    deleted_at      TIMESTAMPTZ,                   -- Soft delete
    
    -- Constraints
    CONSTRAINT agent_tasks_agent_type_not_empty CHECK (char_length(agent_type) >= 1),
    CONSTRAINT agent_tasks_task_type_not_empty CHECK (char_length(task_type) >= 1),
    CONSTRAINT agent_tasks_input_not_empty CHECK (char_length(input_message) >= 1)
);

-- =============================================================================
-- AGENT TOOL EXECUTIONS TABLE
-- =============================================================================

CREATE TABLE IF NOT EXISTS agent_tool_executions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    hex_id          VARCHAR(20) NOT NULL UNIQUE,
    
    -- References
    task_id         UUID NOT NULL REFERENCES agent_tasks(id) ON DELETE CASCADE,
    
    -- Tool info
    tool_name       VARCHAR(100) NOT NULL,
    tool_input      JSONB NOT NULL,
    tool_output     JSONB,
    
    -- Status tracking
    status          tool_execution_status NOT NULL DEFAULT 'pending',
    
    -- Metrics
    duration_ms     INTEGER,  -- Execution time in milliseconds
    
    -- Error handling
    error_message   TEXT,
    retry_count     INTEGER NOT NULL DEFAULT 0,
    
    -- Timestamps
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at      TIMESTAMPTZ,
    completed_at    TIMESTAMPTZ,
    deleted_at      TIMESTAMPTZ,
    
    -- Constraints
    CONSTRAINT agent_tool_executions_tool_name_not_empty CHECK (char_length(tool_name) >= 1)
);

-- =============================================================================
-- INDEXES
-- =============================================================================

-- agent_tasks indexes
CREATE INDEX IF NOT EXISTS idx_agent_tasks_hex_id 
    ON agent_tasks(hex_id);

CREATE INDEX IF NOT EXISTS idx_agent_tasks_status 
    ON agent_tasks(status) 
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_agent_tasks_agent_type 
    ON agent_tasks(agent_type) 
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_agent_tasks_task_type 
    ON agent_tasks(task_type) 
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_agent_tasks_created_at 
    ON agent_tasks(created_at DESC) 
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_agent_tasks_active 
    ON agent_tasks(status, created_at DESC) 
    WHERE deleted_at IS NULL AND status IN ('pending', 'running', 'waiting');

-- agent_tool_executions indexes
CREATE INDEX IF NOT EXISTS idx_agent_tool_executions_hex_id 
    ON agent_tool_executions(hex_id);

CREATE INDEX IF NOT EXISTS idx_agent_tool_executions_task_id 
    ON agent_tool_executions(task_id) 
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_agent_tool_executions_status 
    ON agent_tool_executions(status) 
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_agent_tool_executions_tool_name 
    ON agent_tool_executions(tool_name) 
    WHERE deleted_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_agent_tool_executions_created_at 
    ON agent_tool_executions(task_id, created_at ASC) 
    WHERE deleted_at IS NULL;
