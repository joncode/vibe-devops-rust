use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;
use crate::errors::AppError;
pub type ToolResult<T> = Result<T, ToolError>;
#[derive(Debug, thiserror::Error)]
pub enum ToolError { #[error("Invalid: {0}")] InvalidInput(String), #[error("Failed: {0}")] ExecutionFailed(String), #[error("Not impl: {0}")] NotImplemented(String) }
impl From<ToolError> for AppError { fn from(e: ToolError) -> Self { match e { ToolError::InvalidInput(m) => AppError::Validation(m), _ => AppError::Internal(anyhow::anyhow!("{}", e)) } } }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIToolDefinition { #[serde(rename = "type")] pub tool_type: String, pub function: OpenAIFunctionDefinition }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIFunctionDefinition { pub name: String, pub description: String, pub parameters: JsonValue }
#[derive(Debug, Clone)]
pub struct ToolContext { pub task_id: uuid::Uuid, pub environment: Option<String>, pub user_id: Option<uuid::Uuid>, pub metadata: HashMap<String, String> }
impl ToolContext { pub fn new(task_id: uuid::Uuid) -> Self { Self { task_id, environment: None, user_id: None, metadata: HashMap::new() } }
    pub fn with_environment(mut self, env: &str) -> Self { self.environment = Some(env.to_string()); self } }
#[async_trait]
pub trait AgentTool: Send + Sync {
    fn name(&self) -> &str; fn description(&self) -> &str; fn parameters_schema(&self) -> JsonValue;
    async fn execute(&self, input: JsonValue, ctx: &ToolContext) -> ToolResult<JsonValue>;
    fn to_openai_definition(&self) -> OpenAIToolDefinition { OpenAIToolDefinition { tool_type: "function".to_string(), function: OpenAIFunctionDefinition { name: self.name().to_string(), description: self.description().to_string(), parameters: self.parameters_schema() } } } }
pub struct ToolRegistry { tools: HashMap<String, Arc<dyn AgentTool>> }
impl Default for ToolRegistry { fn default() -> Self { Self::new() } }
impl ToolRegistry {
    pub fn new() -> Self { Self { tools: HashMap::new() } }
    pub fn register<T: AgentTool + 'static>(&mut self, tool: T) { self.tools.insert(tool.name().to_string(), Arc::new(tool)); }
    pub fn to_openai_definitions(&self) -> Vec<OpenAIToolDefinition> { self.tools.values().map(|t| t.to_openai_definition()).collect() }
    pub async fn execute(&self, name: &str, input: JsonValue, ctx: &ToolContext) -> ToolResult<JsonValue> {
        self.tools.get(name).ok_or_else(|| ToolError::NotImplemented(format!("Tool: {}", name)))?.execute(input, ctx).await } }
pub struct GetDeploymentStatusTool;
#[async_trait]
impl AgentTool for GetDeploymentStatusTool { fn name(&self) -> &str { "get_deployment_status" } fn description(&self) -> &str { "Get status" }
    fn parameters_schema(&self) -> JsonValue { serde_json::json!({"type":"object","properties":{"app_name":{"type":"string"},"environment":{"type":"string"}},"required":["app_name","environment"]}) }
    async fn execute(&self, input: JsonValue, _ctx: &ToolContext) -> ToolResult<JsonValue> {
        let app = input["app_name"].as_str().ok_or_else(|| ToolError::InvalidInput("app_name".into()))?;
        let env = input["environment"].as_str().ok_or_else(|| ToolError::InvalidInput("env".into()))?;
        Ok(serde_json::json!({"app_name":app,"environment":env,"status":"running","health":"healthy"})) } }
pub struct CreateDeploymentPrTool;
#[async_trait]
impl AgentTool for CreateDeploymentPrTool { fn name(&self) -> &str { "create_deployment_pr" } fn description(&self) -> &str { "Create PR" }
    fn parameters_schema(&self) -> JsonValue { serde_json::json!({"type":"object","properties":{"app_name":{"type":"string"},"environment":{"type":"string"},"image_tag":{"type":"string"}},"required":["app_name","environment","image_tag"]}) }
    async fn execute(&self, input: JsonValue, _ctx: &ToolContext) -> ToolResult<JsonValue> {
        let app = input["app_name"].as_str().ok_or_else(|| ToolError::InvalidInput("app_name".into()))?;
        let _tag = input["image_tag"].as_str().ok_or_else(|| ToolError::InvalidInput("image_tag".into()))?;
        Ok(serde_json::json!({"pr_number":123,"pr_url":format!("https://github.com/org/{}-infra/pull/123",app)})) } }
pub struct TriggerRollbackTool;
#[async_trait]
impl AgentTool for TriggerRollbackTool { fn name(&self) -> &str { "trigger_rollback" } fn description(&self) -> &str { "Rollback" }
    fn parameters_schema(&self) -> JsonValue { serde_json::json!({"type":"object","properties":{"app_name":{"type":"string"},"environment":{"type":"string"}},"required":["app_name","environment"]}) }
    async fn execute(&self, input: JsonValue, _ctx: &ToolContext) -> ToolResult<JsonValue> {
        let app = input["app_name"].as_str().ok_or_else(|| ToolError::InvalidInput("app_name".into()))?;
        Ok(serde_json::json!({"app_name":app,"rollback_initiated":true})) } }
pub struct FetchAppLogsTool;
#[async_trait]
impl AgentTool for FetchAppLogsTool { fn name(&self) -> &str { "fetch_app_logs" } fn description(&self) -> &str { "Logs" }
    fn parameters_schema(&self) -> JsonValue { serde_json::json!({"type":"object","properties":{"app_name":{"type":"string"},"environment":{"type":"string"}},"required":["app_name","environment"]}) }
    async fn execute(&self, input: JsonValue, _ctx: &ToolContext) -> ToolResult<JsonValue> {
        let app = input["app_name"].as_str().ok_or_else(|| ToolError::InvalidInput("app_name".into()))?;
        Ok(serde_json::json!({"app_name":app,"logs":[]})) } }
pub fn create_default_registry() -> ToolRegistry { let mut r = ToolRegistry::new(); r.register(GetDeploymentStatusTool); r.register(CreateDeploymentPrTool); r.register(TriggerRollbackTool); r.register(FetchAppLogsTool); r }
