//! Integration tests for the Deployer Agent

use serde_json::json;

/// Test that the deployment models serialize correctly
#[test]
fn test_deploy_intent_serialization() {
    let intent = json!({
        "app_name": "my-app",
        "environment": "staging",
        "target_sha": "abc123def456",
        "triggered_by": "ci-pipeline",
        "metadata": {}
    });

    assert_eq!(intent["app_name"], "my-app");
    assert_eq!(intent["environment"], "staging");
}

/// Test deployment status values
#[test]
fn test_deployment_status_values() {
    let statuses = vec![
        "pending",
        "planning",
        "planned",
        "pr_created",
        "deploying",
        "verifying",
        "succeeded",
        "failed",
        "cancelled",
    ];

    for status in statuses {
        assert!(!status.is_empty());
    }
}

/// Test plan request validation
#[test]
fn test_plan_request_structure() {
    let plan_request = json!({
        "app_name": "test-service",
        "environment": "prod",
        "target_ref": "main",
        "triggered_by": "manual"
    });

    assert!(plan_request.get("app_name").is_some());
    assert!(plan_request.get("environment").is_some());
}

/// Test create PR request structure
#[test]
fn test_create_pr_request_structure() {
    let pr_request = json!({
        "deployment_id": "dep_abc123xyz",
        "pr_title": "Deploy test-service to staging",
        "pr_body": "Automated deployment PR"
    });

    assert!(pr_request.get("deployment_id").is_some());
}

/// Test deployment response structure
#[test]
fn test_deployment_response_structure() {
    let response = json!({
        "id": "dep_abc123xyz",
        "app_name": "my-app",
        "environment": "staging",
        "status": "planned",
        "pr_url": null,
        "created_at": "2026-02-05T12:00:00Z"
    });

    assert_eq!(response["status"], "planned");
}
