# CI_CD.md - CI/CD Practice Guide

**Authority:** guidance (delivery automation and release hygiene)
**Layer:** Guides
**Binding:** No
**Scope:** practical CI/CD patterns for production-grade software delivery
**Non-goals:** replacing release contracts or environment-specific runbooks

---

## Table of Contents

1. [CI/CD Mission](#1-cicd-mission)
2. [CI Baseline (Per-PR)](#2-ci-baseline-per-pr)
3. [CD Baseline (Post-Merge)](#3-cd-baseline-post-merge)
4. [Branch Strategy](#4-branch-strategy)
5. [Release Hygiene](#5-release-hygiene)
6. [Deployment Strategies](#6-deployment-strategies)
7. [Secrets Management](#7-secrets-management)
8. [Rollback Procedures](#8-rollback-procedures)
9. [Pipeline Maintenance](#9-pipeline-maintenance)
10. [Incident Integration](#10-incident-integration)
11. [Anti-Patterns](#11-anti-patterns)

---

## 1. CI/CD Mission

CI/CD should make high-quality delivery the default path:
- Every change is validated the same way
- Release risk is visible before merge
- Deployment outcomes are observable and reversible

The pipeline is not infrastructure — it is engineering discipline made executable. The following principles define what that means in practice.

### 1.1 Core Principles

**Deployment frequency is a competitive metric.**
The ability to ship to production ten times a day is not a technical indulgence — it is the mechanism by which an organization tests hypotheses faster than competitors who deploy monthly. Infrequent deployment is infrequent feedback.

**Releases must be boring non-events.**
A release that requires a war room, a release manager, or an after-hours window is a release that will cause an incident. If shipping is painful, teams will ship less. If teams ship less, every deployment becomes higher-stakes. The pipeline's job is to make this cycle impossible.

**CI is a practice, not a tool.**
Continuous Integration means merging to the main branch at least once per day. Long-lived feature branches are the opposite of integration — they are divergence accumulation. The discipline of small, frequent merges is the practice; the tool enforces it.

**Fail closed, recover fast.**
When deployment metrics degrade, the pipeline must halt the rollout and revert automatically. Mean Time to Recovery is more operationally important than Mean Time Between Failures. Optimize for fast recovery, not for preventing every failure.

**Build once, deploy everywhere.**
The same artifact that passes staging must be the artifact deployed to production. Environment-specific builds destroy the value of staging. Immutable, hash-verified artifacts are the only trustworthy promotion mechanism.

**Deployment and release are independent operations.**
Deploying code to a server is a technical operation. Releasing a feature to users is a product operation. Feature flags decouple them, enabling dark launches, gradual rollouts, and instant kill switches without a full redeployment.

**The pipeline is code.**
CI/CD configuration must live in the repository, versioned alongside application code, subject to the same review process. Pipelines that exist only in a CI provider's UI are unversioned infrastructure.

**A broken main branch stops all feature work.**
When the main branch build fails, it is the highest-priority incident for the entire engineering team. Not because it is urgent in isolation, but because it blocks all downstream work. Fix it before anything else.

---

## 2. CI Baseline (Per-PR)

### 2.1 Required Pipeline Stages

Every PR must pass through these stages:

| Stage | Purpose | Tools | Fail Behavior |
|-------|---------|-------|---------------|
| **Build** | Compile code, generate artifacts | `cargo build`, `npm build` | Block merge |
| **Static Analysis** | Catch obvious issues | `cargo clippy`, linters | Block merge |
| **Unit Tests** | Verify isolated behavior | `cargo test`, `npm test` | Block merge |
| **Integration Tests** | Verify component contracts | Test suite | Block merge |
| **Security Scan** | Find vulnerabilities | `cargo audit`, dependency check | Block merge |
| **Policy Checks** | Verify requirements | Custom validators | Block merge |

### 2.2 PR Pipeline Configuration

```yaml
# .github/workflows/pr-verify.yml
name: PR Verification

on:
  pull_request:
    branches: [main, master]

jobs:
  verify:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Build
        run: cargo build --release
        
      - name: Lint
        run: cargo clippy --all-targets -- -D warnings
        
      - name: Test
        run: cargo test --all-features
        
      - name: Integration Tests
        run: cargo test --test '*integration*'
        
      - name: Security Audit
        run: cargo audit
        
      - name: Validate
        run: decapod validate
```

### 2.3 When to Add More Checks

Add additional verification stages when:
- New language/framework is introduced
- Security requirements change
- Performance requirements are added
- New integration points are created

**Do not add stages that:**
- Take longer than 10 minutes total
- Require credentials/secrets in PR context
- Are redundant with existing stages
- Test implementation details

### 2.4 PR Merge Requirements

Before merging, all required stages must pass:
- Build succeeds
- All tests pass (unit, integration)
- Lint/format checks pass
- Security scan passes
- Policy checks pass
- At least one approval (if required)

---

## 3. CD Baseline (Post-Merge)

### 3.1 Pipeline Stages

| Stage | Purpose | Gates |
|-------|---------|-------|
| **Build & Hash** | Create immutable artifact | None (always runs) |
| **Test** | Verify artifact quality | Must pass |
| **Stage Deploy** | Deploy to staging environment | Must pass |
| **Smoke Tests** | Verify staging works | Must pass |
| **Production Deploy** | Deploy to production | Manual or automatic |
| **Health Check** | Verify production health | Must pass |
| **Monitor** | Watch for degradation | Always runs |

### 3.2 Artifact Promotion

```
Source Code → Build → Artifact #abc123
                             │
                             ▼
                      Deploy to Staging
                             │
                   ┌─────────┴─────────┐
                   ▼                   ▼
              Smoke Tests          Security Scan
                   │                   │
                   └─────────┬─────────┘
                             ▼
                    Deploy to Production
                             │
                             ▼
                       Health Check
                             │
                             ▼
                        Monitoring
```

### 3.3 Deployment Gate Configuration

```yaml
# .github/workflows/deploy.yml
name: Deploy

on:
  push:
    branches: [main]

jobs:
  build:
    runs-on: ubuntu-latest
    outputs:
      artifact_hash: ${{ steps.hash.outputs.hash }}
    steps:
      - uses: actions/checkout@v4
      - name: Build
        run: cargo build --release
      - name: Hash
        id: hash
        run: echo "hash=$(sha256sum target/release/binary | cut -d' ' -f1)" >> $GITHUB_OUTPUT

  deploy-staging:
    needs: build
    runs-on: ubuntu-latest
    environment: staging
    steps:
      - name: Deploy
        run: deploy.sh staging ${{ needs.build.outputs.artifact_hash }}
      - name: Smoke Tests
        run: smoke-tests.sh staging

  deploy-production:
    needs: [build, deploy-staging]
    runs-on: ubuntu-latest
    environment: production
    steps:
      - name: Deploy
        run: deploy.sh production ${{ needs.build.outputs.artifact_hash }}
      - name: Health Check
        run: health-check.sh production
```

---

## 4. Branch Strategy

### 4.1 Branch Types

| Branch | Purpose | Lifetime | Protection |
|--------|---------|----------|-----------|
| **main/master** | Production-ready code | Permanent | Required checks, no direct push |
| **release/*** | Release preparation | Until release | Required checks |
| **feature/*** | New feature development | Until merged | Optional checks |
| **bugfix/*** | Bug fixes | Until merged | Optional checks |
| **hotfix/*** | Emergency production fixes | Until merged | Required checks |

### 4.2 Branch Rules

1. **Short-lived feature branches**: Merge within 1-2 days
2. **Frequent integration**: Rebase onto main daily
3. **Protected branches**: Require PR and checks
4. **Direct commits**: Forbidden on protected branches

### 4.3 Git Workflow

```bash
# Start feature branch
git checkout main
git pull
git checkout -b feature/my-feature

# Work in small increments
git add .
git commit -m "Add initial implementation"
git push -u origin feature/my-feature

# Keep current with main
git fetch origin
git rebase origin/main

# When ready, create PR
# After approval, squash and merge
```

### 4.4 Commit Message Conventions

Follow conventional commits:

```
type(scope): description

[optional body]

[optional footer]
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

---

## 5. Release Hygiene

### 5.1 Release Process

1. **Tag creation**: Annotated tags with version
2. **Changelog**: Generate from conventional commits
3. **Artifact verification**: Ensure artifact matches tag
4. **Deployment**: Deploy with rollback plan
5. **Verification**: Health checks and smoke tests
6. **Announcement**: Notify stakeholders

### 5.2 Version Numbering

Follow semantic versioning (MAJOR.MINOR.PATCH):

| Component | Increment When |
|-----------|---------------|
| MAJOR | Breaking changes |
| MINOR | New functionality (backward compatible) |
| PATCH | Bug fixes (backward compatible) |

### 5.3 Release Checklist

- [ ] All tests pass on main
- [ ] Version bumped correctly
- [ ] Changelog updated
- [ ] Release notes written
- [ ] Artifact hash verified
- [ ] Deployment plan reviewed
- [ ] Rollback plan documented
- [ ] Monitoring alerts configured
- [ ] Stakeholders notified

### 5.4 Hotfix Process

```bash
# Create hotfix branch from production tag
git checkout -b hotfix/critical-bug v1.2.3
git cherry-pick <fix-commit>
git tag -a v1.2.4 -m "Critical bug fix"
git push origin hotfix/critical-bug v1.2.4

# Create PR to main after hotfix is deployed
```

---

## 6. Deployment Strategies

### 6.1 Rolling Deployment

**When to use:** Stateless services, canary releases

```yaml
strategy:
  type: rolling
  maxSurge: 25%
  maxUnavailable: 0
```

**Pros:** Simple, no downtime, gradual rollout
**Cons:** Hard to roll back, mixed versions during rollout

### 6.2 Blue-Green Deployment

**When to use:** State的服务, zero-downtime requirements

```yaml
strategy:
  type: blue-green
  activeDeadlineSeconds: 3600
```

**Pros:** Instant rollback, easy verification
**Cons:** Double infrastructure cost, potential for drift

### 6.3 Canary Deployment

**When to use:** High-risk changes, gradual rollout

```yaml
strategy:
  type: canary
  canary:
    weight: 10  # Start with 10% of traffic
    steps:
      - setWeight: 25
      - pause: {duration: 10m}
      - setWeight: 50
      - pause: {duration: 30m}
      - setWeight: 100
```

**Pros:** Real traffic testing, easy rollback
**Cons:** Complex, potential for partial failures

### 6.4 Feature Flags

Decouple deployment from release:

```rust
if feature_flags::is_enabled("new_checkout_flow", user_id) {
    new_checkout_flow()
} else {
    legacy_checkout_flow()
}
```

**Benefits:**
- Deploy without releasing
- Instant kill switch
- Gradual rollout
- A/B testing capability

---

## 7. Secrets Management

### 7.1 Secrets Pipeline

```
Development → Build Time → Runtime
   │              │              │
   ▼              ▼              ▼
.env         CI Secrets      Vault/KMS
```

### 7.2 Secrets Rules

1. **Never commit secrets**: Use `.gitignore`, pre-commit hooks
2. **Rotate regularly**: Automated rotation where possible
3. **Principle of least privilege**: Access only what you need
4. **Audit access**: Log all secret access
5. **Separate credentials**: Build vs. runtime secrets

### 7.3 Secret Storage

| Environment | Storage | Access |
|-------------|---------|--------|
| Development | `.env` file (local only) | Developer |
| CI | Secrets manager (GitHub Actions, etc.) | CI service |
| Staging | Secrets manager | CI + limited devs |
| Production | Vault/KMS | Runtime only |

### 7.4 Example: Vault Integration

```yaml
# In deployment config
env:
  DATABASE_PASSWORD:
    secret_ref: secret/data/production/db#password
```

---

## 8. Rollback Procedures

### 8.1 When to Rollback

Trigger rollback when:
- Error rate spikes above threshold
- Latency increases beyond SLA
- Health checks fail consistently
- Security incident detected
- Business metrics degrade

### 8.2 Rollback Process

```bash
# 1. Identify the issue
kubectl describe pod <pod-name> | grep -A 10 Events

# 2. Verify the last good deployment
decapod deploy history --service <name> --limit 5

# 3. Rollback to previous version
decapod deploy rollback --service <name>

# 4. Verify rollback
kubectl rollout status deployment/<name>
decapod validate --service <name>

# 5. Investigate while monitoring
```

### 8.3 Automatic Rollback Configuration

```yaml
# Kubernetes deployment with automatic rollback
spec:
  strategy:
    type: RollingUpdate
  rollbackTo:
    revision: 0  # Previous revision
```

### 8.4 Rollback Metrics

Track these to determine if rollback is needed:
- Error rate (5xx responses)
- Latency (p99 response time)
- Success rate (business metrics)
- Resource utilization

---

## 9. Pipeline Maintenance

### 9.1 Pipeline Health Metrics

| Metric | Target | Alert |
|--------|--------|-------|
| PR merge time | < 30 min | > 1 hour |
| Pipeline success rate | > 90% | < 80% |
| Failed PR rate | < 10% | > 20% |
| Mean time to restore | < 30 min | > 1 hour |

### 9.2 Pipeline Optimization

**Common optimizations:**
- Parallelize independent stages
- Cache dependencies between runs
- Reduce test execution time
- Optimize Docker layers
- Skip unnecessary checks

### 9.3 Pipeline Review

Quarterly review of:
- Build times and trends
- Failure modes and causes
- Required checks (remove unnecessary)
- Security scanning coverage
- Compliance requirements

---

## 10. Incident Integration

### 10.1 Pipeline Behavior During Incidents

During incidents:
- New PRs may be blocked or slowed
- Production deployments may require extra approval
- Focus is on resolution, not new features

### 10.2 Incident Deployment Rules

1. All incident fixes require at least two approvals
2. Hotfixes must include rollback plan
3. Monitor for 30 minutes after deployment
4. Post-mortem required for all incidents

### 10.3 Emergency Access

```bash
# Emergency access to production
eval $(decapod emergency access --service <name> --role developer)
```

---

## 11. Anti-Patterns

### 11.1 CI Anti-Patterns

**The 90-Minute Build**
- Too many checks in CI
- No parallelization
- Sequential test execution

**The Flaky Suite**
- Tests that fail randomly
- Network dependencies in tests
- Race conditions

**The Bypassed Pipeline**
- Force merges bypassing checks
- Disabled validation
- Secret workarounds

### 11.2 CD Anti-Patterns

**The Big Bang Deploy**
- Many changes at once
- No rollback plan
- Long deployment windows

**The Manual Step**
- Human intervention required
- Credentials entered manually
- Click-to-deploy

**The Snowflake Environment**
- Environment-specific differences
- Configuration drift
- "Works on my machine"

### 11.3 How to Fix

| Anti-Pattern | Fix |
|--------------|-----|
| 90-minute build | Parallelize, cache, reduce checks |
| Flaky suite | Fix tests, quarantine, don't ignore |
| Bypassed pipeline | Automate, enforce, monitor |
| Big bang deploy | Incremental, feature flags, canary |
| Manual step | Automate, self-service |
| Snowflake environment | Infrastructure as code, immutable |

---

## Links

### Core Router
- `core/DECAPOD.md` - **Router and navigation charter (START HERE)**
- `core/ENGINEERING_EXCELLENCE.md` - **Oracle for Engineering Standards**

### Authority (Constitution Layer)
- `specs/INTENT.md` - **Methodology contract (READ FIRST)**
- `specs/SYSTEM.md` - System definition and authority doctrine
- `specs/GIT.md` - Git workflow contract

### Registry (Core Indices)
- `core/PLUGINS.md` - Subsystem registry
- `core/METHODOLOGY.md` - Methodology guides index

### Contracts (Interfaces Layer)
- `interfaces/TESTING.md` - Testing contract
- `interfaces/CONTROL_PLANE.md` - Sequencing patterns
- `interfaces/CLAIMS.md` - Promises ledger
- `interfaces/DOC_RULES.md` - Doc compilation rules

### Practice (Methodology Layer - This Document)
- `methodology/SOUL.md` - Agent identity
- `methodology/ARCHITECTURE.md` - Architecture practice
- `methodology/TESTING.md` - Testing practice
- `methodology/KNOWLEDGE.md` - Knowledge curation
- `methodology/MEMORY.md` - Memory and learning

### Operations (Plugins Layer)
- `plugins/TODO.md` - Work tracking
- `plugins/VERIFY.md` - Validation subsystem
- `plugins/EMERGENCY_PROTOCOL.md` - Emergency protocols