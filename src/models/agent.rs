//! Agent task and tool execution models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::hex_id::HexId;

// =============================================================================
// AGENT TASK STATUS
// =============================================================================

/// Agent task status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "agent_task_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum AgentTaskStatus {
    Pending,
    Running,
    Waiting,
    Completed,
    Failed,
    Cancelled,
}

impl Default for AgentTaskStatus {
    fn default() -> Self {
        Self::Pending
    }
}

impl AgentTaskStatus {
    /// Returns true if the task is in a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }

    /// Returns true if the task is actively processing
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Running | Self::Waiting)
    }
}

// =============================================================================
// TOOL EXECUTION STATUS
// =============================================================================

/// Tool execution status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "tool_execution_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ToolExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

impl Default for ToolExecutionStatus {
    fn default() -> Self {
        Self::Pending
    }
}

impl ToolExecutionStatus {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed)
    }
}

// =============================================================================
// AGENT TASK MODEL
// =============================================================================

/// Agent task record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AgentTask {
    pub id: Uuid,
    pub hex_id: String,
    pub agent_type: String,
    pub task_type: String,
    pub input_message: String,
    pub context: serde_json::Value,
    pub status: AgentTaskStatus,
    pub tool_calls: serde_json::Value,
    pub result: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl HexId for AgentTask {
    const PREFIX: &'static str = "atk";
}

// =============================================================================
// AGENT TOOL EXECUTION MODEL
// =============================================================================

/// Agent tool execution record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AgentToolExecution {
    pub id: Uuid,
    pub hex_id: String,
    pub task_id: Uuid,
    pub tool_name: String,
    pub tool_input: serde_json::Value,
    pub tool_output: Option<serde_json::Value>,
    pub status: ToolExecutionStatus,
    pub duration_ms: Option<i32>,
    pub error_message: Option<String>,
    pub retry_count: i32,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl HexId for AgentToolExecution {
    const PREFIX: &'static str = "tex";
}

// =============================================================================
// API REQUEST/RESPONSE TYPES
// =============================================================================

/// Request to deploy/execute an agent task
#[derive(Debug, Clone, Deserialize)]
pub struct CreateAgentTaskRequest {
    /// Type of agent to use (e.g., "chatgpt", "claude")
    pub agent_type: String,
    /// Type of task (e.g., "deploy", "rollback", "status", "custom")
    pub task_type: String,
    /// The message/prompt for the agent
    pub message: String,
    /// Additional context (environment, app name, etc.)
    #[serde(default)]
    pub context: serde_json::Value,
}

/// Response for agent task status
#[derive(Debug, Clone, Serialize)]
pub struct AgentTaskResponse {
    pub id: String,
    pub agent_type: String,
    pub task_type: String,
    pub status: AgentTaskStatus,
    pub input_message: String,
    pub result: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub tool_executions: Vec<ToolExecutionSummary>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl AgentTaskResponse {
    pub fn from_task_with_executions(task: AgentTask, executions: Vec<AgentToolExecution>) -> Self {
        Self {
            id: task.hex_id,
            agent_type: task.agent_type,
            task_type: task.task_type,
            status: task.status,
            input_message: task.input_message,
            result: task.result,
            error_message: task.error_message,
            tool_executions: executions.into_iter().map(ToolExecutionSummary::from).collect(),
            created_at: task.created_at,
            started_at: task.started_at,
            completed_at: task.completed_at,
        }
    }
}

/// Summary of a tool execution for API responses
#[derive(Debug, Clone, Serialize)]
pub struct ToolExecutionSummary {
    pub id: String,
    pub tool_name: String,
    pub status: ToolExecutionStatus,
    pub duration_ms: Option<i32>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<AgentToolExecution> for ToolExecutionSummary {
    fn from(exec: AgentToolExecution) -> Self {
        Self {
            id: exec.hex_id,
            tool_name: exec.tool_name,
            status: exec.status,
            duration_ms: exec.duration_ms,
            error_message: exec.error_message,
            created_at: exec.created_at,
        }
    }
}

/// Log entry for streaming execution logs
#[derive(Debug, Clone, Serialize)]
pub struct AgentLogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
    pub tool_name: Option<String>,
    pub execution_id: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

/// Query parameters for listing tasks
#[derive(Debug, Clone, Deserialize)]
pub struct ListAgentTasksQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub status: Option<AgentTaskStatus>,
    pub agent_type: Option<String>,
    pub task_type: Option<String>,
}

fn default_limit() -> i64 {
    20
}

/// Query parameters for listing logs
#[derive(Debug, Clone, Deserialize)]
pub struct ListAgentLogsQuery {
    #[serde(default = "default_log_limit")]
    pub limit: i64,
    pub since: Option<DateTime<Utc>>,
}

fn default_log_limit() -> i64 {
    100
}
