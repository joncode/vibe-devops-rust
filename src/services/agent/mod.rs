pub mod tools;
pub use tools::*;
use std::sync::Arc; use std::time::Instant; use serde::{Deserialize, Serialize}; use serde_json::Value as JsonValue; use uuid::Uuid;
use crate::db::Database; use crate::errors::{AppError, AppResult};
use crate::models::{AgentTask, AgentTaskResponse, AgentTaskStatus, AgentToolExecution, AgentToolExecutionResponse, AgentToolStatus, CreateDeployTaskRequest};
use crate::repositories::{AgentTaskRepository, AgentToolExecutionRepository};
pub struct AgentService<'a> { db: &'a Database, tool_registry: Arc<ToolRegistry> }
impl<'a> AgentService<'a> {
    pub fn new(db: &'a Database) -> Self { Self { db, tool_registry: Arc::new(create_default_registry()) } }
    pub async fn create_task(&self, req: &CreateDeployTaskRequest) -> AppResult<AgentTask> {
        let task_type = self.infer_task_type(&req.message);
        Ok(AgentTaskRepository::new(self.db.pool()).create(&req.agent_type, task_type, &req.message).await.map_err(AppError::Internal)?) }
    pub async fn start_task(&self, task_id: Uuid) -> AppResult<AgentTask> {
        AgentTaskRepository::new(self.db.pool()).update_status(task_id, AgentTaskStatus::Running, None).await.map_err(AppError::Internal)?.ok_or(AppError::NotFound) }
    pub async fn execute_tool(&self, task_id: Uuid, tool_name: &str, tool_input: JsonValue, ctx: &ToolContext) -> AppResult<AgentToolExecution> {
        let repo = AgentTaskRepository::new(self.db.pool());
        let task = repo.find_by_id(task_id).await.map_err(AppError::Internal)?.ok_or(AppError::NotFound)?;
        if task.status != AgentTaskStatus::Running { return Err(AppError::Validation(format!("Task not running: {:?}", task.status))); }
        let exec_repo = AgentToolExecutionRepository::new(self.db.pool());
        let exec = exec_repo.create(task_id, tool_name, tool_input.clone()).await.map_err(AppError::Internal)?;
        exec_repo.mark_running(exec.id).await.map_err(AppError::Internal)?;
        let start = Instant::now(); let result = self.tool_registry.execute(tool_name, tool_input.clone(), ctx).await; let dur = start.elapsed().as_millis() as i32;
        Ok(match result { Ok(out) => { repo.append_tool_call(task_id, serde_json::json!({"tool":tool_name,"status":"completed"})).await.map_err(AppError::Internal)?;
                exec_repo.complete(exec.id, out, dur).await.map_err(AppError::Internal)?.ok_or(AppError::NotFound)? }
            Err(e) => { let m = e.to_string(); repo.append_tool_call(task_id, serde_json::json!({"tool":tool_name,"error":m,"status":"failed"})).await.map_err(AppError::Internal)?;
                exec_repo.fail(exec.id, &m, Some(dur)).await.map_err(AppError::Internal)?.ok_or(AppError::NotFound)? } }) }
    pub async fn complete_task(&self, task_id: Uuid, result: JsonValue) -> AppResult<AgentTask> {
        let repo = AgentTaskRepository::new(self.db.pool()); repo.update_result(task_id, result).await.map_err(AppError::Internal)?;
        repo.update_status(task_id, AgentTaskStatus::Completed, None).await.map_err(AppError::Internal)?.ok_or(AppError::NotFound) }
    pub async fn fail_task(&self, task_id: Uuid, error: &str) -> AppResult<AgentTask> {
        AgentTaskRepository::new(self.db.pool()).update_status(task_id, AgentTaskStatus::Failed, Some(error)).await.map_err(AppError::Internal)?.ok_or(AppError::NotFound) }
    pub async fn cancel_task(&self, task_id: Uuid) -> AppResult<AgentTask> {
        AgentTaskRepository::new(self.db.pool()).update_status(task_id, AgentTaskStatus::Cancelled, None).await.map_err(AppError::Internal)?.ok_or(AppError::NotFound) }
    pub async fn get_task(&self, hex_id: &str) -> AppResult<AgentTask> {
        AgentTaskRepository::new(self.db.pool()).find_by_hex_id(hex_id).await.map_err(AppError::Internal)?.ok_or(AppError::NotFound) }
    pub async fn get_task_status(&self, hex_id: &str) -> AppResult<TaskStatusResponse> {
        let task = self.get_task(hex_id).await?; Ok(TaskStatusResponse { task: task.clone().into(), tool_executions: self.get_task_executions(&task).await? }) }
    pub async fn get_task_executions(&self, task: &AgentTask) -> AppResult<Vec<AgentToolExecutionResponse>> {
        Ok(AgentToolExecutionRepository::new(self.db.pool()).find_by_task_id(task.id).await.map_err(AppError::Internal)?.into_iter().map(|e| e.to_response(&task.hex_id)).collect()) }
    pub async fn get_task_logs(&self, hex_id: &str, include_outputs: bool) -> AppResult<TaskLogsResponse> {
        let task = self.get_task(hex_id).await?;
        let execs = AgentToolExecutionRepository::new(self.db.pool()).find_by_task_id(task.id).await.map_err(AppError::Internal)?;
        Ok(TaskLogsResponse { task_id: task.hex_id.clone(), task_status: task.status, input_message: task.input_message.clone(),
            logs: execs.into_iter().map(|e| LogEntry { timestamp: e.created_at, tool_name: e.tool_name, status: e.status, duration_ms: e.duration_ms,
                input: if include_outputs { Some(e.tool_input) } else { None }, output: if include_outputs { e.tool_output } else { None }, error: e.error_message }).collect() }) }
    pub async fn list_tasks(&self, status: Option<AgentTaskStatus>, agent_type: Option<&str>, limit: i64, offset: i64) -> AppResult<TaskListResponse> {
        let repo = AgentTaskRepository::new(self.db.pool());
        let tasks = repo.list(status, agent_type, limit, offset).await.map_err(AppError::Internal)?;
        let total = repo.count(status, agent_type).await.map_err(AppError::Internal)?;
        Ok(TaskListResponse { tasks: tasks.into_iter().map(|t| t.into()).collect(), total, limit, offset }) }
    pub fn get_available_tools(&self) -> Vec<OpenAIToolDefinition> { self.tool_registry.to_openai_definitions() }
    fn infer_task_type(&self, message: &str) -> &'static str { let l = message.to_lowercase();
        if l.contains("deploy") { "deploy" } else if l.contains("rollback") { "rollback" } else if l.contains("status") { "status" } else if l.contains("log") { "fetch_logs" } else { "custom" } } }
#[derive(Debug, Serialize, Deserialize)]
pub struct TaskStatusResponse { pub task: AgentTaskResponse, pub tool_executions: Vec<AgentToolExecutionResponse> }
#[derive(Debug, Serialize, Deserialize)]
pub struct TaskLogsResponse { pub task_id: String, pub task_status: AgentTaskStatus, pub input_message: String, pub logs: Vec<LogEntry> }
#[derive(Debug, Serialize, Deserialize)]
pub struct LogEntry { pub timestamp: chrono::DateTime<chrono::Utc>, pub tool_name: String, pub status: AgentToolStatus, pub duration_ms: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")] pub input: Option<JsonValue>, #[serde(skip_serializing_if = "Option::is_none")] pub output: Option<JsonValue>, #[serde(skip_serializing_if = "Option::is_none")] pub error: Option<String> }
#[derive(Debug, Serialize, Deserialize)]
pub struct TaskListResponse { pub tasks: Vec<AgentTaskResponse>, pub total: i64, pub limit: i64, pub offset: i64 }
