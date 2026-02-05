# Claude Agent Deployment Spec

## Overview

Deploy Claude-based AI agents to Manifest infrastructure with proper authentication, authorization, and programmatic access for automated deployments.

## Authentication & Authorization

### Service Account System

Service accounts enable programmatic access for AI agents and CI/CD systems.

```sql
CREATE TABLE app_service_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    hex_id VARCHAR(20) NOT NULL UNIQUE,
    user_id UUID NOT NULL REFERENCES users(id),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    api_key_hash VARCHAR(255) NOT NULL,
    api_key_prefix VARCHAR(10) NOT NULL, -- for identification
    permissions JSONB NOT NULL DEFAULT '[]',
    rate_limit_per_minute INTEGER DEFAULT 60,
    last_used_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_service_accounts_user_id ON app_service_accounts(user_id);
CREATE INDEX idx_service_accounts_key_prefix ON app_service_accounts(api_key_prefix);
```

### API Key Usage Tracking

```sql
CREATE TABLE app_api_key_usage (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    hex_id VARCHAR(20) NOT NULL UNIQUE,
    service_account_id UUID NOT NULL REFERENCES app_service_accounts(id),
    endpoint VARCHAR(255) NOT NULL,
    method VARCHAR(10) NOT NULL,
    status_code INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE INDEX idx_api_key_usage_account_id ON app_api_key_usage(service_account_id);
```

### Permission Scopes

| Scope | Description |
|-------|-------------|
| `deploy:code` | Deploy code to nodes |
| `deploy:service` | Deploy services/containers |
| `deploy:full` | Deploy full codebases |
| `manage:nodes` | Create/update/delete nodes |
| `read:status` | Read deployment status |
| `manage:deployments` | Full deployment lifecycle |

---

## Claude Agent Architecture

### Agent Runner Service

A dedicated service that:
1. Receives deployment tasks
2. Calls Claude API with structured tools
3. Executes allowed operations against Manifest infrastructure
4. Streams logs back to UI/notifications

### Tool Definitions for Claude

Rather than free-form access, expose structured tools:

```json
{
  "tools": [
    {
      "name": "create_deployment_pr",
      "description": "Create a PR with deployment changes",
      "parameters": {
        "app": "string",
        "environment": "string",
        "changes": "object"
      }
    },
    {
      "name": "run_ci",
      "description": "Trigger CI workflow",
      "parameters": {
        "workflow_id": "string",
        "ref": "string"
      }
    },
    {
      "name": "deploy_service",
      "description": "Deploy a service to an environment",
      "parameters": {
        "service": "string",
        "environment": "string",
        "image_tag": "string"
      }
    },
    {
      "name": "rollback",
      "description": "Rollback to a previous version",
      "parameters": {
        "service": "string",
        "environment": "string",
        "version": "string"
      }
    },
    {
      "name": "get_cluster_status",
      "description": "Get status of a cluster/environment",
      "parameters": {
        "environment": "string"
      }
    },
    {
      "name": "fetch_logs",
      "description": "Fetch service logs",
      "parameters": {
        "service": "string",
        "environment": "string",
        "since": "string"
      }
    }
  ]
}
```

---

## Deployment APIs

### Deploy Code/Service

```http
POST /api/v1/deployments
Authorization: Bearer <api_key>
Content-Type: application/json

{
  "type": "code" | "service" | "codebase",
  "target_node_id": "uuid",
  "name": "my-service",
  "source": {
    "type": "git" | "archive" | "inline",
    "repository": "https://github.com/user/repo",
    "branch": "main",
    "path": "/path/to/service",
    "commit": "abc123"
  },
  "config": {
    "runtime": "node" | "python" | "docker",
    "entrypoint": "index.js",
    "environment": {...},
    "resources": {
      "cpu": "500m",
      "memory": "512Mi"
    }
  },
  "build": {
    "command": "npm install && npm run build",
    "dockerfile": "path/to/Dockerfile"
  }
}
```

### Deploy Full Codebase

```http
POST /api/v1/deployments/codebase
Authorization: Bearer <api_key>

{
  "target_node_id": "uuid",
  "name": "full-app-deployment",
  "source": {
    "type": "git",
    "repository": "https://github.com/user/full-app",
    "branch": "main"
  },
  "services": [
    {
      "name": "frontend",
      "path": "./frontend",
      "config": {...}
    },
    {
      "name": "backend",
      "path": "./backend",
      "config": {...}
    }
  ],
  "infrastructure": {
    "database": true,
    "cache": true
  }
}
```

---

## GitOps Integration

Claude agents work best with GitOps for safety:

### Recommended Flow
1. Agent creates PR for deployment change
2. CI runs validations
3. Human merges (or auto-merge on green for staging)
4. GitOps reconciler (ArgoCD/Flux) deploys

### Why GitOps for Agents
- Agent can't "freestyle" production
- All changes are reviewable diffs
- Audit trail built-in
- Easy rollback via git revert

---

## Security & Credentials

### Dedicated Service Account
- Create `claude-deployer` service account
- Grant minimum required permissions
- Set reasonable rate limits

### Short-lived Credentials
- Use OIDC where possible
- Rotate API keys regularly
- Set expiration dates

### Secrets Management
- Agent never reads secrets directly
- Tools fetch secrets at runtime from Vault/AWS Secrets Manager
- Secrets referenced by name, not value

---

## Environment Policies

### Development
- Auto-approve all actions
- No human review required

### Staging
- Auto-approve PR creation
- Auto-merge on green CI
- Agent monitors rollout

### Production
- Agent creates PR only
- Requires human approval
- Change ticket required
- Agent monitors but doesn't auto-remediate

---

## Observability Requirements

### Audit Logging
Every tool call logged:
- Who (service account)
- What (action + parameters)
- When (timestamp)
- Result (success/failure)
- Links (PR URLs, CI runs, etc.)

### Deployment History
- Replayable history per deployment
- Link to git commits
- Link to ArgoCD sync status

---

## Implementation in Rust

### New Modules

1. **`src/services/claude_agent.rs`**
   - Tool execution logic
   - Claude API integration
   - Response parsing

2. **`src/api/handlers/agent.rs`**
   - `/api/v1/agent/deploy` — trigger agent deployment
   - `/api/v1/agent/status` — check agent task status
   - `/api/v1/agent/logs` — stream agent execution logs

3. **`src/models/service_account.rs`**
   - ServiceAccount struct with HexId + SoftDelete

4. **`src/repositories/service_account.rs`**
   - CRUD for service accounts
   - API key validation

### Middleware
- `AgentAuthMiddleware` — validate service account tokens
- `RateLimitMiddleware` — enforce per-account limits
- `AuditMiddleware` — log all agent actions
