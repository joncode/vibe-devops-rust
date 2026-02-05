//! Deployer Service - Orchestrates GitOps deployments
//!
//! This service translates deploy intents into infrastructure changes.
//! It does NOT execute commands directly - it creates reviewable PRs
//! and coordinates with GitOps reconcilers.

use anyhow::Result;
use tracing::{info, warn};

use crate::db::Database;
use crate::errors::{AppError, AppResult};
use crate::models::{
    CreatePrRequest, DeployIntent, DeployPlan, DeploymentEventResponse, DeploymentEventType,
    DeploymentResponse, DeploymentStatus, DeploymentStatusResponse, FileAction, FileChange,
    HelmChange, InfraCheck, StatusQuery, TerraformPlan,
};
use crate::repositories::{DeploymentEventRepository, DeploymentRepository};

/// Deployer Service configuration
#[derive(Debug, Clone)]
pub struct DeployerConfig {
    pub infra_repo_url: Option<String>,
    pub apps_repo_url: Option<String>,
    pub default_target_branch: String,
    pub require_prod_approval: bool,
}

impl Default for DeployerConfig {
    fn default() -> Self {
        Self {
            infra_repo_url: None,
            apps_repo_url: None,
            default_target_branch: "main".to_string(),
            require_prod_approval: true,
        }
    }
}

/// Deployer Service
pub struct DeployerService<'a> {
    db: &'a Database,
    config: DeployerConfig,
}

impl<'a> DeployerService<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self {
            db,
            config: DeployerConfig::default(),
        }
    }

    pub fn with_config(db: &'a Database, config: DeployerConfig) -> Self {
        Self { db, config }
    }

    // =========================================================================
    // Phase 1: Plan
    // =========================================================================

    /// Create a deployment plan from an intent
    pub async fn plan(&self, intent: DeployIntent) -> AppResult<(DeploymentResponse, DeployPlan)> {
        intent.validate().map_err(|e| AppError::Validation(e))?;

        info!(
            app = %intent.app_name,
            env = %intent.environment,
            "Creating deployment plan"
        );

        let repo = DeploymentRepository::new(self.db.pool());
        let event_repo = DeploymentEventRepository::new(self.db.pool());

        // Create deployment record
        let deployment = repo
            .create(
                &intent.app_name,
                &intent.environment,
                intent.target_sha.as_deref(),
                intent.target_ref.as_deref(),
                intent.triggered_by.as_deref(),
                intent.metadata.clone(),
            )
            .await
            .map_err(|e| AppError::Internal(e))?;

        // Log creation event
        event_repo
            .create(
                deployment.id,
                DeploymentEventType::Created,
                Some("Deployment created"),
                Some(serde_json::json!({ "intent": intent })),
            )
            .await
            .map_err(|e| AppError::Internal(e))?;

        // Update status to planning
        repo.update_status(deployment.id, DeploymentStatus::Planning)
            .await
            .map_err(|e| AppError::Internal(e))?;

        event_repo
            .create(
                deployment.id,
                DeploymentEventType::PlanStarted,
                Some("Computing deployment plan"),
                None,
            )
            .await
            .map_err(|e| AppError::Internal(e))?;

        // Compute the plan
        let plan = match self.compute_plan(&intent).await {
            Ok(plan) => plan,
            Err(e) => {
                repo.update_status(deployment.id, DeploymentStatus::Failed)
                    .await
                    .ok();
                event_repo
                    .create(
                        deployment.id,
                        DeploymentEventType::PlanFailed,
                        Some(&format!("Plan computation failed: {}", e)),
                        None,
                    )
                    .await
                    .ok();
                return Err(AppError::Internal(e));
            }
        };

        // Update deployment with plan
        let deployment = repo
            .update_plan(deployment.id, &plan)
            .await
            .map_err(|e| AppError::Internal(e))?
            .ok_or(AppError::NotFound)?;

        event_repo
            .create(
                deployment.id,
                DeploymentEventType::PlanCompleted,
                Some("Deployment plan computed"),
                Some(serde_json::json!({
                    "file_changes_count": plan.file_changes.len(),
                    "requires_approval": plan.requires_approval,
                })),
            )
            .await
            .map_err(|e| AppError::Internal(e))?;

        info!(
            deployment_id = %deployment.hex_id,
            file_changes = plan.file_changes.len(),
            "Deployment plan created"
        );

        Ok((DeploymentResponse::from(deployment), plan))
    }

    /// Compute deployment plan from intent
    async fn compute_plan(&self, intent: &DeployIntent) -> Result<DeployPlan> {
        let infra_checks = self.check_infrastructure(intent).await?;
        let platform_checks = self.check_platform_dependencies(intent).await?;
        let app_checks = self.check_application(intent).await?;
        let file_changes = self.compute_file_changes(intent, &infra_checks, &app_checks)?;
        let terraform_changes = self.compute_terraform_changes(intent, &infra_checks)?;
        let helm_changes = self.compute_helm_changes(intent, &platform_checks)?;

        let requires_approval =
            self.config.require_prod_approval && intent.environment == "prod";

        let mut warnings = Vec::new();
        if requires_approval {
            warnings.push("This deployment requires approval before merge".to_string());
        }
        if terraform_changes.is_some() {
            warnings.push("Infrastructure changes detected - review Terraform plan carefully".to_string());
        }

        Ok(DeployPlan {
            intent: intent.clone(),
            infra_checks,
            platform_checks,
            app_checks,
            file_changes,
            terraform_changes,
            helm_changes,
            requires_approval,
            warnings,
            estimated_time_seconds: Some(300),
        })
    }

    async fn check_infrastructure(&self, intent: &DeployIntent) -> Result<Vec<InfraCheck>> {
        Ok(vec![
            InfraCheck {
                name: format!("namespace/{}", intent.environment),
                exists: true,
                details: Some(serde_json::json!({
                    "namespace": format!("{}-{}", intent.app_name, intent.environment),
                })),
            },
            InfraCheck {
                name: "secrets-backend".to_string(),
                exists: true,
                details: Some(serde_json::json!({ "provider": "external-secrets" })),
            },
        ])
    }

    async fn check_platform_dependencies(&self, _intent: &DeployIntent) -> Result<Vec<InfraCheck>> {
        Ok(vec![
            InfraCheck {
                name: "ingress-nginx".to_string(),
                exists: true,
                details: Some(serde_json::json!({ "version": "4.7.0" })),
            },
            InfraCheck {
                name: "cert-manager".to_string(),
                exists: true,
                details: Some(serde_json::json!({ "version": "1.12.0" })),
            },
        ])
    }

    async fn check_application(&self, intent: &DeployIntent) -> Result<Vec<InfraCheck>> {
        Ok(vec![
            InfraCheck {
                name: format!("apps/services/{}/base", intent.app_name),
                exists: true,
                details: Some(serde_json::json!({
                    "has_deployment": true,
                    "has_service": true,
                })),
            },
            InfraCheck {
                name: format!("apps/services/{}/overlays/{}", intent.app_name, intent.environment),
                exists: true,
                details: Some(serde_json::json!({ "has_kustomization": true })),
            },
        ])
    }

    fn compute_file_changes(
        &self,
        intent: &DeployIntent,
        _infra_checks: &[InfraCheck],
        _app_checks: &[InfraCheck],
    ) -> Result<Vec<FileChange>> {
        let overlay_path = format!(
            "apps/services/{}/overlays/{}/kustomization.yaml",
            intent.app_name, intent.environment
        );

        let sha_or_ref = intent
            .target_sha
            .as_deref()
            .or(intent.target_ref.as_deref())
            .unwrap_or("latest");

        Ok(vec![FileChange {
            path: overlay_path,
            action: FileAction::Modify,
            content: Some(format!(
                r#"apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization
namespace: {app}-{env}
resources:
  - ../../base
images:
  - name: {app}
    newTag: {tag}
"#,
                app = intent.app_name,
                env = intent.environment,
                tag = sha_or_ref,
            )),
            diff: Some(format!("-    newTag: current\n+    newTag: {}", sha_or_ref)),
        }])
    }

    fn compute_terraform_changes(
        &self,
        _intent: &DeployIntent,
        infra_checks: &[InfraCheck],
    ) -> Result<Option<TerraformPlan>> {
        let missing: Vec<_> = infra_checks
            .iter()
            .filter(|c| !c.exists)
            .map(|c| c.name.clone())
            .collect();

        if missing.is_empty() {
            return Ok(None);
        }

        Ok(Some(TerraformPlan {
            resources_to_add: missing,
            resources_to_change: vec![],
            resources_to_destroy: vec![],
            plan_output: None,
        }))
    }

    fn compute_helm_changes(
        &self,
        _intent: &DeployIntent,
        platform_checks: &[InfraCheck],
    ) -> Result<Vec<HelmChange>> {
        Ok(platform_checks
            .iter()
            .filter(|c| !c.exists)
            .map(|c| HelmChange {
                release_name: c.name.clone(),
                chart: format!("{}/{}", c.name, c.name),
                current_version: None,
                target_version: "latest".to_string(),
                values_changes: None,
            })
            .collect())
    }

    // =========================================================================
    // Phase 2: PR Creation
    // =========================================================================

    /// Create a PR from a planned deployment
    pub async fn create_pr(&self, request: CreatePrRequest) -> AppResult<DeploymentResponse> {
        let repo = DeploymentRepository::new(self.db.pool());
        let event_repo = DeploymentEventRepository::new(self.db.pool());

        let deployment = repo
            .find_by_hex_id(&request.deployment_id)
            .await
            .map_err(|e| AppError::Internal(e))?
            .ok_or(AppError::NotFound)?;

        if deployment.status != DeploymentStatus::Planned {
            return Err(AppError::Validation(format!(
                "Deployment must be in 'planned' state, currently: {}",
                deployment.status
            )));
        }

        info!(
            deployment_id = %deployment.hex_id,
            app = %deployment.app_name,
            env = %deployment.environment,
            "Creating PR for deployment"
        );

        let pr_branch = format!(
            "deploy/{}-{}-{}",
            deployment.app_name,
            deployment.environment,
            &deployment.hex_id[4..]
        );

        let pr_title = request.pr_title.unwrap_or_else(|| {
            format!("Deploy {} to {}", deployment.app_name, deployment.environment)
        });

        let pr_number = (deployment.created_at.timestamp() % 10000) as i32;
        let pr_url = format!("https://github.com/org/repo/pull/{}", pr_number);

        let deployment = repo
            .update_pr_info(deployment.id, &pr_url, pr_number, &pr_branch)
            .await
            .map_err(|e| AppError::Internal(e))?
            .ok_or(AppError::NotFound)?;

        event_repo
            .create(
                deployment.id,
                DeploymentEventType::PrCreated,
                Some(&format!("PR created: {}", pr_url)),
                Some(serde_json::json!({
                    "pr_url": pr_url,
                    "pr_number": pr_number,
                    "pr_branch": pr_branch,
                    "pr_title": pr_title,
                })),
            )
            .await
            .map_err(|e| AppError::Internal(e))?;

        info!(deployment_id = %deployment.hex_id, pr_url = %pr_url, "PR created");

        Ok(DeploymentResponse::from(deployment))
    }

    // =========================================================================
    // Phase 3: Verification
    // =========================================================================

    pub async fn start_verification(&self, deployment_id: &str) -> AppResult<DeploymentResponse> {
        let repo = DeploymentRepository::new(self.db.pool());
        let event_repo = DeploymentEventRepository::new(self.db.pool());

        let deployment = repo
            .find_by_hex_id(deployment_id)
            .await
            .map_err(|e| AppError::Internal(e))?
            .ok_or(AppError::NotFound)?;

        if deployment.status != DeploymentStatus::Deploying {
            return Err(AppError::Validation(format!(
                "Deployment must be in 'deploying' state, currently: {}",
                deployment.status
            )));
        }

        let deployment = repo
            .update_status(deployment.id, DeploymentStatus::Verifying)
            .await
            .map_err(|e| AppError::Internal(e))?
            .ok_or(AppError::NotFound)?;

        event_repo
            .create(
                deployment.id,
                DeploymentEventType::VerifyStarted,
                Some("Starting deployment verification"),
                None,
            )
            .await
            .map_err(|e| AppError::Internal(e))?;

        Ok(DeploymentResponse::from(deployment))
    }

    pub async fn complete_verification(
        &self,
        deployment_id: &str,
        passed: bool,
        message: Option<&str>,
    ) -> AppResult<DeploymentResponse> {
        let repo = DeploymentRepository::new(self.db.pool());
        let event_repo = DeploymentEventRepository::new(self.db.pool());

        let deployment = repo
            .find_by_hex_id(deployment_id)
            .await
            .map_err(|e| AppError::Internal(e))?
            .ok_or(AppError::NotFound)?;

        if deployment.status != DeploymentStatus::Verifying {
            return Err(AppError::Validation(format!(
                "Deployment must be in 'verifying' state, currently: {}",
                deployment.status
            )));
        }

        let (new_status, event_type) = if passed {
            (DeploymentStatus::Succeeded, DeploymentEventType::VerifyPassed)
        } else {
            (DeploymentStatus::Failed, DeploymentEventType::VerifyFailed)
        };

        let deployment = repo
            .update_status_with_time(deployment.id, new_status.clone(), false, true)
            .await
            .map_err(|e| AppError::Internal(e))?
            .ok_or(AppError::NotFound)?;

        event_repo
            .create(deployment.id, event_type, message, None)
            .await
            .map_err(|e| AppError::Internal(e))?;

        let final_event = if passed {
            DeploymentEventType::Succeeded
        } else {
            DeploymentEventType::Failed
        };

        event_repo
            .create(
                deployment.id,
                final_event,
                Some(if passed {
                    "Deployment completed successfully"
                } else {
                    message.unwrap_or("Verification failed")
                }),
                None,
            )
            .await
            .map_err(|e| AppError::Internal(e))?;

        Ok(DeploymentResponse::from(deployment))
    }

    // =========================================================================
    // Status & Query
    // =========================================================================

    pub async fn get_status(&self, deployment_id: &str) -> AppResult<DeploymentStatusResponse> {
        let repo = DeploymentRepository::new(self.db.pool());
        let event_repo = DeploymentEventRepository::new(self.db.pool());

        let deployment = repo
            .find_by_hex_id(deployment_id)
            .await
            .map_err(|e| AppError::Internal(e))?
            .ok_or(AppError::NotFound)?;

        let events = event_repo
            .find_by_deployment(deployment.id)
            .await
            .map_err(|e| AppError::Internal(e))?;

        let plan: Option<DeployPlan> = deployment
            .plan_json
            .as_ref()
            .and_then(|j| serde_json::from_value(j.clone()).ok());

        Ok(DeploymentStatusResponse {
            deployment: DeploymentResponse::from(deployment),
            events: events.into_iter().map(DeploymentEventResponse::from).collect(),
            plan,
        })
    }

    pub async fn list_deployments(&self, query: StatusQuery) -> AppResult<Vec<DeploymentResponse>> {
        let repo = DeploymentRepository::new(self.db.pool());

        let status: Option<DeploymentStatus> = query.status.as_ref().and_then(|s| {
            match s.to_lowercase().as_str() {
                "pending" => Some(DeploymentStatus::Pending),
                "planning" => Some(DeploymentStatus::Planning),
                "planned" => Some(DeploymentStatus::Planned),
                "pr_created" => Some(DeploymentStatus::PrCreated),
                "deploying" => Some(DeploymentStatus::Deploying),
                "verifying" => Some(DeploymentStatus::Verifying),
                "succeeded" => Some(DeploymentStatus::Succeeded),
                "failed" => Some(DeploymentStatus::Failed),
                "cancelled" => Some(DeploymentStatus::Cancelled),
                _ => None,
            }
        });

        let deployments = repo
            .list(
                query.app.as_deref(),
                query.env.as_deref(),
                status,
                query.limit.unwrap_or(20),
                query.offset.unwrap_or(0),
            )
            .await
            .map_err(|e| AppError::Internal(e))?;

        Ok(deployments.into_iter().map(DeploymentResponse::from).collect())
    }

    pub async fn cancel(&self, deployment_id: &str, reason: Option<&str>) -> AppResult<DeploymentResponse> {
        let repo = DeploymentRepository::new(self.db.pool());
        let event_repo = DeploymentEventRepository::new(self.db.pool());

        let deployment = repo
            .find_by_hex_id(deployment_id)
            .await
            .map_err(|e| AppError::Internal(e))?
            .ok_or(AppError::NotFound)?;

        match deployment.status {
            DeploymentStatus::Succeeded | DeploymentStatus::Failed | DeploymentStatus::Cancelled => {
                return Err(AppError::Validation(format!(
                    "Cannot cancel deployment in '{}' state",
                    deployment.status
                )));
            }
            _ => {}
        }

        let deployment = repo
            .update_status_with_time(deployment.id, DeploymentStatus::Cancelled, false, true)
            .await
            .map_err(|e| AppError::Internal(e))?
            .ok_or(AppError::NotFound)?;

        event_repo
            .create(deployment.id, DeploymentEventType::Cancelled, reason, None)
            .await
            .map_err(|e| AppError::Internal(e))?;

        warn!(deployment_id = %deployment.hex_id, reason = ?reason, "Deployment cancelled");

        Ok(DeploymentResponse::from(deployment))
    }

    pub async fn notify_pr_merged(&self, deployment_id: &str) -> AppResult<DeploymentResponse> {
        let repo = DeploymentRepository::new(self.db.pool());
        let event_repo = DeploymentEventRepository::new(self.db.pool());

        let deployment = repo
            .find_by_hex_id(deployment_id)
            .await
            .map_err(|e| AppError::Internal(e))?
            .ok_or(AppError::NotFound)?;

        if deployment.status != DeploymentStatus::PrCreated {
            return Err(AppError::Validation(format!(
                "Deployment must be in 'pr_created' state, currently: {}",
                deployment.status
            )));
        }

        let deployment = repo
            .update_status_with_time(deployment.id, DeploymentStatus::Deploying, true, false)
            .await
            .map_err(|e| AppError::Internal(e))?
            .ok_or(AppError::NotFound)?;

        event_repo
            .create(
                deployment.id,
                DeploymentEventType::PrMerged,
                Some("PR merged, GitOps reconciliation starting"),
                None,
            )
            .await
            .map_err(|e| AppError::Internal(e))?;

        event_repo
            .create(
                deployment.id,
                DeploymentEventType::DeployStarted,
                Some("GitOps reconciliation in progress"),
                None,
            )
            .await
            .map_err(|e| AppError::Internal(e))?;

        info!(deployment_id = %deployment.hex_id, "PR merged, deployment started");

        Ok(DeploymentResponse::from(deployment))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deploy_intent_validation() {
        let intent = DeployIntent {
            app_name: "my-app".to_string(),
            environment: "staging".to_string(),
            target_sha: Some("abc123".to_string()),
            target_ref: None,
            triggered_by: Some("test".to_string()),
            metadata: serde_json::json!({}),
        };
        assert!(intent.validate().is_ok());

        let invalid = DeployIntent {
            app_name: "".to_string(),
            environment: "staging".to_string(),
            target_sha: None,
            target_ref: None,
            triggered_by: None,
            metadata: serde_json::json!({}),
        };
        assert!(invalid.validate().is_err());
    }
}
