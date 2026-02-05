# Server Infrastructure Spec

## Overview

Deploy and manage Manifest infrastructure across dedicated server boxes using k3s, GitOps, and proper isolation between environments.

## Server Requirements

### Prerequisites
- SSH access as root (or sudo user)
- Domain (optional but recommended)
- GitHub org/repo for GitOps config

---

## Phase 1: Server Hardening

### User Setup
```bash
# Create manifest user with sudo
useradd -m -s /bin/bash manifest
usermod -aG sudo manifest

# Set up SSH key auth
mkdir -p /home/manifest/.ssh
# Add your public key to authorized_keys
chmod 700 /home/manifest/.ssh
chmod 600 /home/manifest/.ssh/authorized_keys
chown -R manifest:manifest /home/manifest/.ssh
```

### Security Configuration
```bash
# Disable password login
sed -i 's/PasswordAuthentication yes/PasswordAuthentication no/' /etc/ssh/sshd_config
systemctl restart sshd

# Enable firewall
ufw allow 22/tcp   # SSH
ufw allow 80/tcp   # HTTP
ufw allow 443/tcp  # HTTPS
ufw enable
```

### Checklist
- [ ] `manifest` user exists with sudo
- [ ] SSH key auth only (password disabled)
- [ ] Firewall enabled (22, 80, 443 open)

---

## Phase 2: k3s Installation

### Install k3s
```bash
curl -sfL https://get.k3s.io | sh -

# Wait for cluster to be ready
k3s kubectl get nodes
```

### Configure kubeconfig for remote access
```bash
# On server
cat /etc/rancher/k3s/k3s.yaml

# On your laptop, save as ~/.kube/manifest-prod (or similar)
# Replace 127.0.0.1 with server IP
```

### Verify Installation
```bash
kubectl get nodes        # Should show Ready
kubectl get pods -A      # CoreDNS should be running
kubectl get svc -A       # Traefik ingress should be running
```

### Checklist
- [ ] `kubectl get nodes` works from laptop
- [ ] CoreDNS running
- [ ] Ingress controller (Traefik) running

---

## Phase 3: Storage & Certificates

### Storage (Longhorn or local-path)

**Option A: Longhorn (recommended for production)**
```bash
kubectl apply -f https://raw.githubusercontent.com/longhorn/longhorn/v1.5.3/deploy/longhorn.yaml
```

**Option B: local-path (prototyping only)**
```bash
# Already included with k3s by default
kubectl get storageclass
```

### cert-manager (TLS)
```bash
kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.13.0/cert-manager.yaml

# Create Let's Encrypt issuer
cat <<EOF | kubectl apply -f -
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: letsencrypt-prod
spec:
  acme:
    server: https://acme-v02.api.letsencrypt.org/directory
    email: your-email@example.com
    privateKeySecretRef:
      name: letsencrypt-prod
    solvers:
    - http01:
        ingress:
          class: traefik
EOF
```

### Checklist
- [ ] PV provisioning works (test with a PVC)
- [ ] cert-manager installed
- [ ] Test certificate can be issued

---

## Phase 4: GitOps with ArgoCD

### Install ArgoCD
```bash
kubectl create namespace argocd
kubectl apply -n argocd -f https://raw.githubusercontent.com/argoproj/argo-cd/stable/manifests/install.yaml

# Get initial admin password
kubectl -n argocd get secret argocd-initial-admin-secret -o jsonpath="{.data.password}" | base64 -d
```

### Expose ArgoCD UI
```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: argocd-server
  namespace: argocd
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
    traefik.ingress.kubernetes.io/router.tls: "true"
spec:
  tls:
  - hosts:
    - argocd.yourdomain.com
    secretName: argocd-tls
  rules:
  - host: argocd.yourdomain.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: argocd-server
            port:
              number: 443
```

### Create App of Apps
```yaml
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: manifest-infra
  namespace: argocd
spec:
  project: default
  source:
    repoURL: https://github.com/YOUR_ORG/manifest-infra.git
    targetRevision: main
    path: clusters/prod
  destination:
    server: https://kubernetes.default.svc
    namespace: argocd
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
```

### Checklist
- [ ] ArgoCD UI reachable
- [ ] Can sync a test app from your repo

---

## Phase 5: Platform Services

### Dependency Order
1. Ingress (Traefik — comes with k3s)
2. cert-manager
3. Storage (Longhorn)
4. external-dns (optional)
5. Observability (Prometheus + Grafana)
6. Registry (optional, or use GHCR)
7. Your platform apps

### Observability Stack
```bash
# Add Prometheus community helm repo
helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
helm repo update

# Install kube-prometheus-stack
helm install monitoring prometheus-community/kube-prometheus-stack \
  --namespace monitoring \
  --create-namespace \
  --set grafana.adminPassword=YOUR_GRAFANA_PASSWORD
```

---

## Environment Isolation

### Two-Box Setup (Prod + Testnet)

| Resource | Prod Box | Testnet Box |
|----------|----------|-------------|
| Server | Separate physical | Separate physical |
| k3s cluster | Independent | Independent |
| PostgreSQL | Own instance | Own instance |
| Databases | `*_production` | `*_testnet` |
| Namespaces | `manifest`, `rails` | `manifest-testnet`, `rails-testnet` |
| Image tags | `:latest`, `:v1.2.3` | `:testnet` |
| Secrets | Prod passwords | Testnet passwords |
| DNS | `app.yourdomain.com` | `testnet-app.yourdomain.com` |
| Replicas | 2-10 (autoscaled) | 1 (fixed) |
| Backups | Daily automated | Not needed |
| Monitoring | Full stack | Minimal/none |

---

## Deployment Commands

### Deploy to Prod
```bash
kubectl -n manifest set image deployment/manifest-app \
  manifest-app=ghcr.io/USER/manifest-app:v1.2.3
```

### Deploy to Testnet
```bash
kubectl -n manifest-testnet set image deployment/manifest-app \
  manifest-app=ghcr.io/USER/manifest-app:testnet
```

### Useful kubectl Commands
```bash
# Check pod status
kubectl get pods -A

# View logs
kubectl -n manifest logs -f deployment/manifest-app

# Shell into pod
kubectl -n manifest exec -it deployment/manifest-app -- /bin/sh

# Check HPA status
kubectl -n manifest get hpa

# Rolling restart
kubectl -n manifest rollout restart deployment/manifest-app
```

---

## Repository Structure

```
manifest-infra/
├── clusters/
│   ├── prod/
│   │   ├── apps/
│   │   ├── platform/
│   │   └── kustomization.yaml
│   └── testnet/
│       ├── apps/
│       ├── platform/
│       └── kustomization.yaml
├── apps/
│   └── manifest-app/
│       ├── base/
│       │   ├── deployment.yaml
│       │   ├── service.yaml
│       │   ├── ingress.yaml
│       │   └── kustomization.yaml
│       └── overlays/
│           ├── prod/
│           └── testnet/
└── platform/
    ├── cert-manager/
    ├── monitoring/
    └── storage/
```

---

## Database Schema for Infrastructure Tracking

```sql
CREATE TABLE servers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    hex_id VARCHAR(20) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    hostname VARCHAR(255) NOT NULL,
    ip_address INET NOT NULL,
    environment VARCHAR(50) NOT NULL, -- 'prod', 'testnet', 'dev'
    status VARCHAR(50) NOT NULL DEFAULT 'provisioning',
    k3s_version VARCHAR(50),
    specs JSONB, -- CPU, RAM, disk
    last_heartbeat_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);

CREATE TABLE server_services (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    hex_id VARCHAR(20) NOT NULL UNIQUE,
    server_id UUID NOT NULL REFERENCES servers(id),
    service_name VARCHAR(255) NOT NULL,
    namespace VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'unknown',
    replicas_desired INTEGER,
    replicas_ready INTEGER,
    image_tag VARCHAR(255),
    last_deployed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);
```

---

## Backup Strategy (Production)

### Database Backups
```bash
# Daily PostgreSQL backup via cron
0 2 * * * pg_dump -h localhost -U manifest manifest_production | gzip > /backups/manifest-$(date +\%Y\%m\%d).sql.gz
```

### k3s Backup
```bash
# Backup etcd (k3s uses SQLite by default, but for HA setups)
k3s etcd-snapshot save --name pre-upgrade-$(date +%Y%m%d)
```

### Off-site Storage
- Sync backups to S3/Storj daily
- Retain 7 daily, 4 weekly, 12 monthly
