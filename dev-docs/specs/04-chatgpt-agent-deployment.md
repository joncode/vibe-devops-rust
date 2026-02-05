# ChatGPT Agent Deployment Spec

## Overview

Deploy ChatGPT/OpenAI-based agents to Manifest infrastructure. The key insight: **you control the runner and tools, not OpenAI.**

## Architecture

### Agent Runner Service

A service you control that:
1. Receives a task ("deploy app X to cluster Y")
2. Calls the OpenAI Responses API with tool definitions
3. Executes allowed tools against Manifest infrastructure
4. Streams logs back to your UI/Slack

OpenAI's function/tool calling + structured outputs bridge model reasoning to real actions.

### Key Principle

Instead of giving the model free-form SSH, expose **structured, auditable tools**:
- `create_pr(changes)` — GitOps
- `run_ci(workflow_id)` — trigger pipelines
- `deploy_service(service, env, image_tag)` — deploy
- `rollback(service, env, version)` — rollback
- `get_cluster_status(env)` — read status
- `fetch_logs(service, env, since)` — debugging

---

## Infrastructure Requirements

### Agent Runner Location
- Lives inside Manifest network (or has VPN access)
- Has outbound access to OpenAI API
- Has access to internal tools (Git, CI, cluster API)
- Does NOT have unfettered kubectl access

### Backend Systems
Choose your stack:
- **GitOps:** ArgoCD or Flux
- **CI/CD:** GitHub Actions, Buildkite, etc.
- **Kubernetes:** k3s, EKS, GKE, etc.
- Your own Manifest API/Control plane

### Config Management
- Helm for vendor stacks
- Kustomize for your apps
- Minimal templating (env files + unit files)

---

## Deployment Flow (GitOps)

### For Kubernetes Services
1. Dev merges PR → "environment repo" changes (Helm/Kustomize values)
2. CI builds image(s) and pushes to registry
3. ArgoCD detects change in repo
4. ArgoCD applies manifests to cluster
5. Drift detection + auto-correction

### For Edge/Bare Metal Services
1. Dev merges PR → release spec update (version + target nodes)
2. CI produces signed artifact bundle
3. Edge Agent on node pulls bundle, validates signature
4. Agent applies systemd update
5. Reports status to Manifest API

This gives you **infrastructure as code** + **GitOps** + **zero-touch rollout**.

---

## Safe Rollout Pattern

### Phase 1: Human-in-the-loop
1. Agent creates PR for deployment change
2. CI runs
3. **Human merges**
4. GitOps reconciler deploys

### Phase 2: Staging Auto-deploy
Once stable, allow "auto-merge on green" for staging only.

### Phase 3: Controlled Prod
- Agent creates PR
- Requires 2-person approval
- Change ticket created automatically
- Agent monitors rollout

---

## Environment Policies

### Development
- Agent can: PR + merge + deploy
- No approval required

### Staging  
- Agent can: PR + trigger CI
- Auto-merge on green
- Agent monitors rollout

### Production
- Agent can: PR only
- Requires: approval + change ticket
- Human merges
- Agent monitors but doesn't auto-remediate

---

## Post-Deploy Verification

Agent should monitor after deploy:
- Health checks
- Smoke tests
- Log scanning for errors
- Automatic rollback if SLOs fail

---

## Chat Interface

Your team interacts naturally:
- "Deploy manifest-api to staging from main"
- "Promote build 8127 to prod"
- "Rollback to last known good"
- "What's the status of the frontend deploy?"

---

## Tool Definitions (OpenAI Format)

```json
{
  "tools": [
    {
      "type": "function",
      "function": {
        "name": "create_deployment_pr",
        "description": "Create a pull request with deployment changes for GitOps",
        "parameters": {
          "type": "object",
          "properties": {
            "app_name": {
              "type": "string",
              "description": "Name of the application to deploy"
            },
            "environment": {
              "type": "string",
              "enum": ["dev", "staging", "prod"],
              "description": "Target environment"
            },
            "image_tag": {
              "type": "string",
              "description": "Docker image tag to deploy"
            },
            "changes": {
              "type": "object",
              "description": "Additional config changes (replicas, env vars, etc.)"
            }
          },
          "required": ["app_name", "environment", "image_tag"]
        }
      }
    },
    {
      "type": "function",
      "function": {
        "name": "get_deployment_status",
        "description": "Get the current deployment status for an app in an environment",
        "parameters": {
          "type": "object",
          "properties": {
            "app_name": {"type": "string"},
            "environment": {"type": "string"}
          },
          "required": ["app_name", "environment"]
        }
      }
    },
    {
      "type": "function",
      "function": {
        "name": "trigger_rollback",
        "description": "Rollback an application to a previous version",
        "parameters": {
          "type": "object",
          "properties": {
            "app_name": {"type": "string"},
            "environment": {"type": "string"},
            "target_version": {"type": "string", "description": "Version or 'previous'"}
          },
          "required": ["app_name", "environment"]
        }
      }
    },
    {
      "type": "function",
      "function": {
        "name": "fetch_app_logs",
        "description": "Fetch recent logs for an application",
        "parameters": {
          "type": "object",
          "properties": {
            "app_name": {"type": "string"},
            "environment": {"type": "string"},
            "since": {"type": "string", "description": "Time range like '5m', '1h'"},
            "filter": {"type": "string", "description": "Log filter pattern"}
          },
          "required": ["app_name", "environment"]
        }
      }
    }
  ]
}
```

---

## Secrets & Credentials

### Dedicated Service Identity
- Create `chatgpt-deployer` service account
- Scoped permissions per environment
- Short-lived tokens where possible

### Secrets Architecture
- Agent never "reads" secrets directly
- Tools fetch secrets at runtime from Vault/Secrets Manager
- Secrets referenced by name in configs

---

## Manifest Control Plane Integration

The cleanest architecture:

```
[ChatGPT Agent Runner]
        |
        v
[Manifest Deploy MCP / Internal API]
        |
        +---> GitHub (PRs)
        +---> ArgoCD (sync status)
        +---> K8s API (health checks)
        +---> Vault (secrets)
```

Build one internal "control plane" API:
- `POST /deploy/plan`
- `POST /deploy/pr`
- `GET /deploy/status`

Then expose MCP tools that call that API. This keeps credentials and complexity out of the model loop.

---

## Implementation Checklist

| Component | Description |
|-----------|-------------|
| Agent Runner | Container/service calling OpenAI + executing tools |
| Orchestration | Queue for deploy tasks, status tracking |
| Deploy method | GitOps (ArgoCD/Flux) recommended |
| Policy engine | Environment-based approval rules |
| Audit system | Log every tool call with context |
| Secrets | Vault integration, no secrets in model context |

---

## Database Schema

```sql
CREATE TABLE agent_tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    hex_id VARCHAR(20) NOT NULL UNIQUE,
    agent_type VARCHAR(50) NOT NULL, -- 'chatgpt', 'claude', etc.
    task_type VARCHAR(50) NOT NULL, -- 'deploy', 'rollback', 'status'
    input_message TEXT NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    tool_calls JSONB DEFAULT '[]',
    result JSONB,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ
);

CREATE TABLE agent_tool_executions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    hex_id VARCHAR(20) NOT NULL UNIQUE,
    task_id UUID NOT NULL REFERENCES agent_tasks(id),
    tool_name VARCHAR(100) NOT NULL,
    tool_input JSONB NOT NULL,
    tool_output JSONB,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    duration_ms INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);
```

---

## Note on Assistants API

OpenAI has deprecated Assistants in favor of the Responses API with built-in tool use. Use the modern approach with structured tool definitions.
