use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use super::hex_id::HexId;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "agent_task_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum AgentTaskStatus { Pending, Running, Completed, Failed, Cancelled }
impl Default for AgentTaskStatus { fn default() -> Self { Self::Pending } }
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "agent_tool_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum AgentToolStatus { Pending, Running, Completed, Failed }
impl Default for AgentToolStatus { fn default() -> Self { Self::Pending } }
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AgentTask { pub id: Uuid, pub hex_id: String, pub agent_type: String, pub task_type: String,
    pub input_message: String, pub status: AgentTaskStatus, pub tool_calls: serde_json::Value,
    pub result: Option<serde_json::Value>, pub error_message: Option<String>, pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>, pub completed_at: Option<DateTime<Utc>>, pub deleted_at: Option<DateTime<Utc>> }
impl HexId for AgentTask { const PREFIX: &'static str = "atk"; }
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AgentToolExecution { pub id: Uuid, pub hex_id: String, pub task_id: Uuid, pub tool_name: String,
    pub tool_input: serde_json::Value, pub tool_output: Option<serde_json::Value>, pub status: AgentToolStatus,
    pub duration_ms: Option<i32>, pub error_message: Option<String>, pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>, pub deleted_at: Option<DateTime<Utc>> }
impl HexId for AgentToolExecution { const PREFIX: &'static str = "ate"; }
impl AgentToolExecution {
    pub fn to_response(&self, task_hex_id: &str) -> AgentToolExecutionResponse {
        AgentToolExecutionResponse { id: self.hex_id.clone(), task_id: task_hex_id.to_string(), tool_name: self.tool_name.clone(),
            tool_input: self.tool_input.clone(), tool_output: self.tool_output.clone(), status: self.status, duration_ms: self.duration_ms,
            error_message: self.error_message.clone(), created_at: self.created_at, completed_at: self.completed_at }}}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTaskResponse { pub id: String, pub agent_type: String, pub task_type: String, pub input_message: String,
    pub status: AgentTaskStatus, pub tool_calls: serde_json::Value, pub result: Option<serde_json::Value>,
    pub error_message: Option<String>, pub created_at: DateTime<Utc>, pub completed_at: Option<DateTime<Utc>> }
impl From<AgentTask> for AgentTaskResponse { fn from(t: AgentTask) -> Self { Self { id: t.hex_id, agent_type: t.agent_type,
    task_type: t.task_type, input_message: t.input_message, status: t.status, tool_calls: t.tool_calls, result: t.result,
    error_message: t.error_message, created_at: t.created_at, completed_at: t.completed_at }}}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentToolExecutionResponse { pub id: String, pub task_id: String, pub tool_name: String,
    pub tool_input: serde_json::Value, pub tool_output: Option<serde_json::Value>, pub status: AgentToolStatus,
    pub duration_ms: Option<i32>, pub error_message: Option<String>, pub created_at: DateTime<Utc>, pub completed_at: Option<DateTime<Utc>> }
#[derive(Debug, Clone, Deserialize)]
pub struct CreateDeployTaskRequest { pub message: String, #[serde(default = "default_agent")] pub agent_type: String,
    pub app_name: Option<String>, pub environment: Option<String>, pub image_tag: Option<String> }
fn default_agent() -> String { "chatgpt".to_string() }
