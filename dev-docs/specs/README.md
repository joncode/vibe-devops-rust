# Vibe DevOps Server Specs

Organized from the VIBE CODE INFRA Notion page. These specs define the implementation roadmap for the Manifest deployment platform.

## Specs

| # | Spec | Description |
|---|------|-------------|
| 01 | [Deployer Agent](./01-deployer-agent.md) | Orchestration layer that translates deploy intent into GitOps PRs |
| 02 | [Git Push CLI](./02-git-push-cli.md) | Heroku-style `git push manifest main` workflow |
| 03 | [Claude Agent Deployment](./03-claude-agent-deployment.md) | Claude-based AI agents with structured tools |
| 04 | [ChatGPT Agent Deployment](./04-chatgpt-agent-deployment.md) | OpenAI-based agents with tool calling |
| 05 | [Server Infrastructure](./05-server-infrastructure.md) | k3s, GitOps, and multi-environment setup |

## Implementation Priority

### Phase 1: Foundation (Current)
- [x] Soft-delete across all tables
- [x] hex_id for all models
- [ ] Core deployment models (apps, builds, releases)

### Phase 2: Git Push Flow
- [ ] Build service with Paketo buildpacks
- [ ] Artifact storage
- [ ] Release management
- [ ] Basic CLI

### Phase 3: Agent Infrastructure
- [ ] Service accounts + API keys
- [ ] Tool execution framework
- [ ] Audit logging

### Phase 4: Full Agent Support
- [ ] Claude agent integration
- [ ] ChatGPT agent integration
- [ ] Policy engine

### Phase 5: Production Infrastructure
- [ ] Multi-server GitOps
- [ ] Observability stack
- [ ] Backup automation

## Source

These specs were synthesized from toggle lists in the [VIBE CODE INFRA Notion page](https://www.notion.so/VIBE-CODE-INFRA-2ee48ff42f35805386bbcf1c06b7b415).
