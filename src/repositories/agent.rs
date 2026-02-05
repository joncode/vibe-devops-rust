use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;
use crate::models::{AgentTask, AgentTaskStatus, AgentToolExecution, AgentToolStatus, HexId};
pub struct AgentTaskRepository<'a> { pool: &'a PgPool }
impl<'a> AgentTaskRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self { Self { pool } }
    pub async fn create(&self, agent_type: &str, task_type: &str, input_message: &str) -> Result<AgentTask> {
        Ok(sqlx::query_as::<_, AgentTask>("INSERT INTO agent_tasks (hex_id,agent_type,task_type,input_message,status) VALUES ($1,$2,$3,$4,$5) RETURNING *")
            .bind(AgentTask::generate_hex_id()).bind(agent_type).bind(task_type).bind(input_message).bind(AgentTaskStatus::Pending).fetch_one(self.pool).await?) }
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<AgentTask>> {
        Ok(sqlx::query_as::<_, AgentTask>("SELECT * FROM agent_tasks WHERE id=$1 AND deleted_at IS NULL").bind(id).fetch_optional(self.pool).await?) }
    pub async fn find_by_hex_id(&self, hex_id: &str) -> Result<Option<AgentTask>> {
        Ok(sqlx::query_as::<_, AgentTask>("SELECT * FROM agent_tasks WHERE hex_id=$1 AND deleted_at IS NULL").bind(hex_id).fetch_optional(self.pool).await?) }
    pub async fn update_status(&self, id: Uuid, status: AgentTaskStatus, error_message: Option<&str>) -> Result<Option<AgentTask>> {
        let completed_at = if matches!(status, AgentTaskStatus::Completed|AgentTaskStatus::Failed|AgentTaskStatus::Cancelled) { Some(chrono::Utc::now()) } else { None };
        Ok(sqlx::query_as::<_, AgentTask>("UPDATE agent_tasks SET status=$2,error_message=COALESCE($3,error_message),completed_at=COALESCE($4,completed_at),updated_at=NOW() WHERE id=$1 AND deleted_at IS NULL RETURNING *")
            .bind(id).bind(status).bind(error_message).bind(completed_at).fetch_optional(self.pool).await?) }
    pub async fn update_result(&self, id: Uuid, result: serde_json::Value) -> Result<Option<AgentTask>> {
        Ok(sqlx::query_as::<_, AgentTask>("UPDATE agent_tasks SET result=$2,updated_at=NOW() WHERE id=$1 AND deleted_at IS NULL RETURNING *").bind(id).bind(result).fetch_optional(self.pool).await?) }
    pub async fn append_tool_call(&self, id: Uuid, tc: serde_json::Value) -> Result<Option<AgentTask>> {
        Ok(sqlx::query_as::<_, AgentTask>("UPDATE agent_tasks SET tool_calls=tool_calls||$2::jsonb,updated_at=NOW() WHERE id=$1 AND deleted_at IS NULL RETURNING *").bind(id).bind(serde_json::json!([tc])).fetch_optional(self.pool).await?) }
    pub async fn list(&self, status: Option<AgentTaskStatus>, agent_type: Option<&str>, limit: i64, offset: i64) -> Result<Vec<AgentTask>> {
        Ok(sqlx::query_as::<_, AgentTask>("SELECT * FROM agent_tasks WHERE deleted_at IS NULL AND ($1::agent_task_status IS NULL OR status=$1) AND ($2::text IS NULL OR agent_type=$2) ORDER BY created_at DESC LIMIT $3 OFFSET $4")
            .bind(status).bind(agent_type).bind(limit).bind(offset).fetch_all(self.pool).await?) }
    pub async fn count(&self, status: Option<AgentTaskStatus>, agent_type: Option<&str>) -> Result<i64> {
        let c: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM agent_tasks WHERE deleted_at IS NULL AND ($1::agent_task_status IS NULL OR status=$1) AND ($2::text IS NULL OR agent_type=$2)").bind(status).bind(agent_type).fetch_one(self.pool).await?; Ok(c.0) }}
pub struct AgentToolExecutionRepository<'a> { pool: &'a PgPool }
impl<'a> AgentToolExecutionRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self { Self { pool } }
    pub async fn create(&self, task_id: Uuid, tool_name: &str, tool_input: serde_json::Value) -> Result<AgentToolExecution> {
        Ok(sqlx::query_as::<_, AgentToolExecution>("INSERT INTO agent_tool_executions (hex_id,task_id,tool_name,tool_input,status) VALUES ($1,$2,$3,$4,$5) RETURNING *")
            .bind(AgentToolExecution::generate_hex_id()).bind(task_id).bind(tool_name).bind(tool_input).bind(AgentToolStatus::Pending).fetch_one(self.pool).await?) }
    pub async fn find_by_task_id(&self, task_id: Uuid) -> Result<Vec<AgentToolExecution>> {
        Ok(sqlx::query_as::<_, AgentToolExecution>("SELECT * FROM agent_tool_executions WHERE task_id=$1 AND deleted_at IS NULL ORDER BY created_at ASC").bind(task_id).fetch_all(self.pool).await?) }
    pub async fn mark_running(&self, id: Uuid) -> Result<Option<AgentToolExecution>> {
        Ok(sqlx::query_as::<_, AgentToolExecution>("UPDATE agent_tool_executions SET status=$2 WHERE id=$1 AND deleted_at IS NULL RETURNING *").bind(id).bind(AgentToolStatus::Running).fetch_optional(self.pool).await?) }
    pub async fn complete(&self, id: Uuid, output: serde_json::Value, dur: i32) -> Result<Option<AgentToolExecution>> {
        Ok(sqlx::query_as::<_, AgentToolExecution>("UPDATE agent_tool_executions SET status=$2,tool_output=$3,duration_ms=$4,completed_at=NOW() WHERE id=$1 AND deleted_at IS NULL RETURNING *").bind(id).bind(AgentToolStatus::Completed).bind(output).bind(dur).fetch_optional(self.pool).await?) }
    pub async fn fail(&self, id: Uuid, error: &str, dur: Option<i32>) -> Result<Option<AgentToolExecution>> {
        Ok(sqlx::query_as::<_, AgentToolExecution>("UPDATE agent_tool_executions SET status=$2,error_message=$3,duration_ms=$4,completed_at=NOW() WHERE id=$1 AND deleted_at IS NULL RETURNING *").bind(id).bind(AgentToolStatus::Failed).bind(error).bind(dur).fetch_optional(self.pool).await?) }}
