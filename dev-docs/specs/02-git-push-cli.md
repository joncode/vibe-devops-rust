# Manifest Git Push CLI Spec

## Overview

Enable `git push manifest main` deployment workflow — the Heroku-style experience for pushing code directly to Manifest infrastructure.

## Core Components

### 1. Git Receiver
Service that accepts git pushes and triggers builds.

### 2. Build Service
Handles source-to-artifact transformation using buildpacks or Dockerfile.

### 3. Artifact Store
Stores built slugs or OCI images.

### 4. Release + Runtime Controller
Manages releases and runs processes on the cluster.

---

## Data Model

### Applications
```sql
CREATE TABLE apps (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    hex_id VARCHAR(20) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL UNIQUE,
    repo_url TEXT,
    default_stack VARCHAR(100) DEFAULT 'paketo',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);
```

### Builds
```sql
CREATE TABLE builds (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    hex_id VARCHAR(20) NOT NULL UNIQUE,
    app_id UUID NOT NULL REFERENCES apps(id),
    source_sha VARCHAR(40) NOT NULL,
    source_ref VARCHAR(255),
    status VARCHAR(50) NOT NULL DEFAULT 'queued',
    output_artifact_digest VARCHAR(255),
    logs_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    finished_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ
);
```

### Artifacts
```sql
CREATE TABLE artifacts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    hex_id VARCHAR(20) NOT NULL UNIQUE,
    digest VARCHAR(255) NOT NULL UNIQUE,
    artifact_type VARCHAR(50) NOT NULL, -- 'slug' or 'oci-image'
    size_bytes BIGINT,
    uri TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ
);
```

### Releases
```sql
CREATE TABLE releases (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    hex_id VARCHAR(20) NOT NULL UNIQUE,
    app_id UUID NOT NULL REFERENCES apps(id),
    build_id UUID NOT NULL REFERENCES builds(id),
    artifact_digest VARCHAR(255) NOT NULL,
    version INTEGER NOT NULL,
    config_snapshot JSONB,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    process_formation JSONB DEFAULT '{"web": 1}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    UNIQUE(app_id, version)
);
```

### Runs (one-off commands)
```sql
CREATE TABLE runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    hex_id VARCHAR(20) NOT NULL UNIQUE,
    app_id UUID NOT NULL REFERENCES apps(id),
    release_id UUID NOT NULL REFERENCES releases(id),
    command TEXT[] NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'starting',
    exit_code INTEGER,
    logs_stream_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    finished_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ
);
```

---

## CLI Commands

### `manifest create <app-name>`
Creates a new application.

### `manifest git:remote -a <app-name>`
Adds the Manifest git remote to your repo.

### `git push manifest main`
The core workflow:
1. Git receiver accepts push
2. Creates a Build record (status: queued)
3. Streams build logs to terminal
4. On success, creates Artifact record
5. Creates Release record
6. Applies formation changes
7. Prints app URL + release id

### `manifest run <command>`
Runs a one-off command:
- Resolves current release (or `--release <id>`)
- Starts a Run record
- Attaches terminal to logs/stdin/stdout
- Exits with remote exit code

**Flags:**
- `-a, --app <name>` — target app
- `-r, --release <id>` — specific release
- `-e, --env KEY=VAL` — ephemeral env overrides
- `--size / --cpu / --mem` — resource allocation
- `--attach / --no-attach` — stream output
- `--timeout 600` — max runtime
- `--pty` — interactive mode (bash)

### `manifest releases`
Lists releases for an app.

### `manifest rollback <version>`
Rolls back to a previous release.

### `manifest logs`
Streams application logs.

---

## API Endpoints

### Apps
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/v1/apps` | Create app |
| GET | `/v1/apps/{app}` | Get app details |
| DELETE | `/v1/apps/{app}` | Delete app |

### Builds
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/v1/apps/{app}/builds` | Trigger build |
| GET | `/v1/apps/{app}/builds/{id}` | Get build status |
| GET | `/v1/apps/{app}/builds/{id}/logs` | Stream build logs |

### Releases
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/v1/apps/{app}/releases` | Create release |
| GET | `/v1/apps/{app}/releases` | List releases |
| GET | `/v1/apps/{app}/releases/{id}` | Get release |
| GET | `/v1/apps/{app}/releases/current` | Get current release |

### Runs
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/v1/apps/{app}/runs` | Start one-off run |
| GET | `/v1/apps/{app}/runs/{id}` | Get run status |
| WS | `/v1/apps/{app}/runs/{id}/attach` | Stream stdin/stdout/stderr |

### Authentication
Bearer token: `MANIFEST_TOKEN` scoped by app/role.

---

## Git Receiver Implementation

### Option A: Self-hosted Git Server
Host `git.manifest.run/<app>.git` with pre-receive hook:
```bash
#!/bin/bash
# pre-receive hook
while read oldrev newrev refname; do
  # Validate push
  # Trigger build via API
  curl -X POST "http://localhost:8080/v1/apps/$APP/builds" \
    -H "Authorization: Bearer $INTERNAL_TOKEN" \
    -d "{\"sha\": \"$newrev\"}"
done
```

**Pros:** Identical UX to Heroku
**Cons:** Must run/scale Git receiver service

### Option B: GitHub Webhooks
Use GitHub push webhooks to trigger builds:
1. User pushes to GitHub
2. Webhook fires to Manifest API
3. API clones repo and starts build

**Pros:** Leverage existing GitHub infrastructure
**Cons:** Slightly different UX (push to GitHub, not manifest)

---

## Build Process (Paketo/Cloud Native Buildpacks)

```yaml
# Build Job Template
apiVersion: batch/v1
kind: Job
metadata:
  name: build-${APP}-${BUILD_ID}
spec:
  template:
    spec:
      containers:
      - name: builder
        image: paketobuildpacks/builder:base
        command: ["/cnb/lifecycle/creator"]
        args:
          - "-app=/workspace"
          - "-cache-dir=/cache"
          - "-run-image=paketobuildpacks/run:base"
          - "registry.manifest.run/${APP}:${SHA}"
        volumeMounts:
        - name: source
          mountPath: /workspace
        - name: cache
          mountPath: /cache
      restartPolicy: Never
```

---

## Implementation Priority

1. **Phase 1:** Apps CRUD, Build triggering, Artifact storage
2. **Phase 2:** Releases, Formation management
3. **Phase 3:** Git receiver (webhook-based first)
4. **Phase 4:** One-off runs with attach
5. **Phase 5:** Self-hosted git server (optional)
