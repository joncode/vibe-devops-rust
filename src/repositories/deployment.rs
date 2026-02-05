//! Deployment repository - Database access for deployments and events

use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::{
    Deployment, DeploymentEvent, DeploymentEventType, DeploymentStatus, DeployPlan, HexId,
};

/// Deployment repository
pub struct DeploymentRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> DeploymentRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Create a new deployment
    pub async fn create(
        &self,
        app_name: &str,
        environment: &str,
        target_sha: Option<&str>,
        target_ref: Option<&str>,
        triggered_by: Option<&str>,
        metadata: serde_json::Value,
    ) -> Result<Deployment> {
        let hex_id = Deployment::generate_hex_id();

        let deployment = sqlx::query_as::<_, Deployment>(
            r#"
            INSERT INTO deployments (hex_id, app_name, environment, target_sha, target_ref, triggered_by, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
        )
        .bind(&hex_id)
        .bind(app_name)
        .bind(environment)
        .bind(target_sha)
        .bind(target_ref)
        .bind(triggered_by)
        .bind(&metadata)
        .fetch_one(self.pool)
        .await?;

        Ok(deployment)
    }

    /// Find deployment by hex_id
    pub async fn find_by_hex_id(&self, hex_id: &str) -> Result<Option<Deployment>> {
        let deployment = sqlx::query_as::<_, Deployment>(
            r#"
            SELECT * FROM deployments
            WHERE hex_id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(hex_id)
        .fetch_optional(self.pool)
        .await?;

        Ok(deployment)
    }

    /// Find deployment by UUID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Deployment>> {
        let deployment = sqlx::query_as::<_, Deployment>(
            r#"
            SELECT * FROM deployments
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id)
        .fetch_optional(self.pool)
        .await?;

        Ok(deployment)
    }

    /// List deployments with optional filtering
    pub async fn list(
        &self,
        app_name: Option<&str>,
        environment: Option<&str>,
        status: Option<DeploymentStatus>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Deployment>> {
        let deployments = sqlx::query_as::<_, Deployment>(
            r#"
            SELECT * FROM deployments
            WHERE deleted_at IS NULL
                AND ($1::VARCHAR IS NULL OR app_name = $1)
                AND ($2::VARCHAR IS NULL OR environment = $2)
                AND ($3::deployment_status IS NULL OR status = $3)
            ORDER BY created_at DESC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(app_name)
        .bind(environment)
        .bind(status)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool)
        .await?;

        Ok(deployments)
    }

    /// Update deployment status
    pub async fn update_status(
        &self,
        id: Uuid,
        status: DeploymentStatus,
    ) -> Result<Option<Deployment>> {
        let deployment = sqlx::query_as::<_, Deployment>(
            r#"
            UPDATE deployments
            SET status = $2, updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(status)
        .fetch_optional(self.pool)
        .await?;

        Ok(deployment)
    }

    /// Update deployment status with timestamps
    pub async fn update_status_with_time(
        &self,
        id: Uuid,
        status: DeploymentStatus,
        set_started: bool,
        set_completed: bool,
    ) -> Result<Option<Deployment>> {
        let deployment = sqlx::query_as::<_, Deployment>(
            r#"
            UPDATE deployments
            SET status = $2,
                updated_at = NOW(),
                started_at = CASE WHEN $3 THEN NOW() ELSE started_at END,
                completed_at = CASE WHEN $4 THEN NOW() ELSE completed_at END
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(status)
        .bind(set_started)
        .bind(set_completed)
        .fetch_optional(self.pool)
        .await?;

        Ok(deployment)
    }

    /// Update deployment plan
    pub async fn update_plan(&self, id: Uuid, plan: &DeployPlan) -> Result<Option<Deployment>> {
        let plan_json = serde_json::to_value(plan)?;

        let deployment = sqlx::query_as::<_, Deployment>(
            r#"
            UPDATE deployments
            SET plan_json = $2, status = 'planned', updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(&plan_json)
        .fetch_optional(self.pool)
        .await?;

        Ok(deployment)
    }

    /// Update PR information
    pub async fn update_pr_info(
        &self,
        id: Uuid,
        pr_url: &str,
        pr_number: i32,
        pr_branch: &str,
    ) -> Result<Option<Deployment>> {
        let deployment = sqlx::query_as::<_, Deployment>(
            r#"
            UPDATE deployments
            SET pr_url = $2, pr_number = $3, pr_branch = $4, status = 'pr_created', updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(pr_url)
        .bind(pr_number)
        .bind(pr_branch)
        .fetch_optional(self.pool)
        .await?;

        Ok(deployment)
    }

    /// Soft delete deployment
    pub async fn delete(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            r#"
            UPDATE deployments
            SET deleted_at = NOW(), updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id)
        .execute(self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}

/// Deployment event repository
pub struct DeploymentEventRepository<'a> {
    pool: &'a PgPool,
}

impl<'a> DeploymentEventRepository<'a> {
    pub fn new(pool: &'a PgPool) -> Self {
        Self { pool }
    }

    /// Create a new deployment event
    pub async fn create(
        &self,
        deployment_id: Uuid,
        event_type: DeploymentEventType,
        message: Option<&str>,
        event_data: Option<serde_json::Value>,
    ) -> Result<DeploymentEvent> {
        let hex_id = DeploymentEvent::generate_hex_id();

        let event = sqlx::query_as::<_, DeploymentEvent>(
            r#"
            INSERT INTO deployment_events (hex_id, deployment_id, event_type, message, event_data)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(&hex_id)
        .bind(deployment_id)
        .bind(event_type)
        .bind(message)
        .bind(&event_data)
        .fetch_one(self.pool)
        .await?;

        Ok(event)
    }

    /// Find events by deployment ID
    pub async fn find_by_deployment(&self, deployment_id: Uuid) -> Result<Vec<DeploymentEvent>> {
        let events = sqlx::query_as::<_, DeploymentEvent>(
            r#"
            SELECT * FROM deployment_events
            WHERE deployment_id = $1 AND deleted_at IS NULL
            ORDER BY created_at ASC
            "#,
        )
        .bind(deployment_id)
        .fetch_all(self.pool)
        .await?;

        Ok(events)
    }

    /// Find event by hex_id
    pub async fn find_by_hex_id(&self, hex_id: &str) -> Result<Option<DeploymentEvent>> {
        let event = sqlx::query_as::<_, DeploymentEvent>(
            r#"
            SELECT * FROM deployment_events
            WHERE hex_id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(hex_id)
        .fetch_optional(self.pool)
        .await?;

        Ok(event)
    }
}
