# Manifest Deployer Agent Spec

## Overview

The Deployer Agent is the orchestration layer that translates deploy intent into infrastructure changes. It does not execute commands directly — it creates reviewable PRs and coordinates with GitOps reconcilers.

## Core Responsibilities

### 1. Turn Intent into Deploy Artifacts
- Receive commands like: "Deploy service X from repo Y at sha Z to staging"
- Generate/modify Kustomize overlays, Helm values, or Terraform configs
- Create PRs for review

### 2. Trigger Infrastructure Provisioning
- Detect when new resources are needed (namespace, DB, redis, bucket, DNS, cert)
- Generate Terraform changes or call Manifest control-plane API
- Handle: VPC, load balancers, managed DBs, IAM, certificates

### 3. Coordinate Environment Policy
- Enforce constraints: allowed images, resource limits, secrets handling, approved charts
- Implement approval gates for production deployments

---

## Tool Selection Matrix

### Terraform — Infrastructure Layer
Use when creating/modifying:
- Kubernetes clusters or node pools
- VPC/networking
- Load balancers / ingress controller dependencies
- Managed DB (RDS/CloudSQL) or managed Redis
- Buckets, queues, IAM/service accounts
- External DNS records
- Certificates

**Agent Output:** Terraform PR with plan summary for human review

### Helm — Platform Services
Use for shared/vendor services:
- ingress-nginx / traefik
- cert-manager
- external-dns
- prometheus stack / grafana
- loki / fluent-bit / vector
- postgres operator / redis operator
- sealed-secrets / external-secrets operator
- argo cd / flux

**Agent Output:** HelmRelease objects (Flux) or Argo CD Application manifests

### Kustomize — Application Deployments
Use for your own services:
```
apps/services/{service}/
  base/
    deployment.yaml
    service.yaml
    ingress.yaml
    kustomization.yaml
  overlays/
    staging/
      kustomization.yaml
      patch.yaml
    prod/
      kustomization.yaml
      patch.yaml
```

**Agent Output:** PR modifying base or overlays

---

## Deployment Workflow

### Phase 1: Plan
Agent computes deploy plan from intent:
1. Does infra exist? (namespace, secrets backend, DB)
2. Does platform dependency exist? (ingress, cert-manager, external-secrets)
3. Does app have base + overlays?

### Phase 2: PR Creation
Agent does NOT `kubectl apply` directly. Instead:
1. Create branch
2. Commit changes
3. Open PR
4. GitOps reconciler applies after merge

### Phase 3: Verification
Agent watches:
- GitOps reconciliation status
- Deployment health (pods ready, service endpoints, ingress up)
- Smoke tests / health checks (HTTP 200)

### Phase 4: Promotion
Agent promotes by:
- Copying image tag from staging overlay to prod
- Or bumping a "release version" value

---

## Repository Structure

```
infra/
  envs/
    staging/
    prod/
  modules/
    k8s-cluster/
    db/
    dns/

platform/
  clusters/
    manifest-staging/
      ingress/
      cert-manager/
      monitoring/
    manifest-prod/

apps/
  services/
    {service-name}/
      base/
      overlays/
        staging/
        prod/
```

---

## Control Plane API

Even with GitOps, the agent needs internal APIs:

### Data Model
- **Application**: name, default config
- **Environment**: staging, prod, etc.
- **Release**: specific version deployed

### Endpoints

| Endpoint | Purpose |
|----------|---------|
| `POST /deploy/plan` | Calculate changes needed |
| `POST /deploy/pr` | Create PR with changes |
| `GET /deploy/status?app=&env=&release=` | Check deployment status |

---

## Implementation in Rust

Add to vibe-devops-server:

1. **Models**: `DeployIntent`, `DeployPlan`, `DeploymentStatus`
2. **Services**: `DeployerService` with plan/execute/verify phases
3. **Handlers**: `/api/v1/deploy/*` endpoints
4. **Background Jobs**: Watch GitOps reconciliation, run health checks

### Database Tables

```sql
CREATE TABLE deployments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    hex_id VARCHAR(20) NOT NULL UNIQUE,
    app_name VARCHAR(255) NOT NULL,
    environment VARCHAR(50) NOT NULL,
    target_sha VARCHAR(40),
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    pr_url TEXT,
    plan_json JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE deployment_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    hex_id VARCHAR(20) NOT NULL UNIQUE,
    deployment_id UUID NOT NULL REFERENCES deployments(id),
    event_type VARCHAR(50) NOT NULL,
    event_data JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);
```
