# CI_CD_PIPELINES.md - CI/CD Pipeline Design

**Authority:** guidance (comprehensive deployment pipeline patterns with exact configurations)
**Layer:** Architecture
**Binding:** No
**Scope:** GitHub Actions, GitLab CI, ArgoCD, deployment strategies with exact specifications

---

## 1. GitHub Actions

### 1.1 Complete Workflow Templates

#### Multi-Stage Production Pipeline
```yaml
# .github/workflows/deploy.yml
name: Deploy

on:
  push:
    branches:
      - main
    tags:
      - 'v*'
  workflow_dispatch:
    inputs:
      environment:
        description: 'Environment to deploy'
        required: true
        default: 'staging'
        type: choice
        options:
          - staging
          - production

env:
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}

jobs:
  # ============================================================
  # Stage 1: Quality Gates
  # ============================================================
  quality:
    name: Quality Checks
    runs-on: ubuntu-latest
    timeout-minutes: 30
    
    steps:
      - name: Checkout code
        uses: actions/checkout@v4
        with:
          fetch-depth: 0  # Full history for semantic-release
      
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
      
      - name: Install dependencies
        run: npm ci
      
      - name: Run lint
        run: npm run lint
      
      - name: Run type check
        run: npm run typecheck
      
      - name: Run unit tests
        run: npm test -- --coverage --ci
        env:
          NODE_ENV: test
          DATABASE_URL: postgresql://test:test@localhost:5432/test
      
      - name: Upload coverage
        uses: codecov/codecov-action@v4
        with:
          files: ./coverage/lcov.info
          fail_ci_if_error: true
          token: ${{ secrets.CODECOV_TOKEN }}
      
      - name: Run E2E tests
        if: github.event_name != 'pull_request'
        run: npm run test:e2e
        env:
          CYPRESS_BASE_URL: ${{ secrets.STAGING_URL }}
      
      - name: Security audit
        run: npm audit --audit-level=moderate
      
      - name: Dependency review
        uses: actions/dependency-review-action@v4

  # ============================================================
  # Stage 2: Build & Package
  # ============================================================
  build:
    name: Build & Package
    runs-on: ubuntu-latest
    needs: quality
    outputs:
      image-tag: ${{ steps.meta.outputs.tags }}
      digest: ${{ steps.build.outputs.digest }}
    
    steps:
      - name: Checkout code
        uses: actions/checkout@v4
      
      - name: Setup Docker Buildx
        uses: docker/setup-buildx-action@v3
      
      - name: Log in to Container Registry
        uses: docker/login-action@v3
        with:
          registry: ${{ env.REGISTRY }}
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}
      
      - name: Extract metadata
        id: meta
        uses: docker/metadata-action@v5
        with:
          images: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}
          tags: |
            type=sha,prefix=,format=short
            type=ref,event=branch
            type=semver,pattern={{version}}
            type=raw,value=latest,enable=${{ github.ref == 'refs/heads/main' }}
      
      - name: Build and push
        id: build
        uses: docker/build-push-action@v5
        with:
          context: .
          push: true
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
          cache-from: type=gha
          cache-to: type=gha,mode=max
          provenance: true
          sbom: true
      
      - name: Generate artifact
        run: |
          echo "${{ steps.build.outputs.digest }}" > artifact-digest.txt
          echo "tag=${{ steps.meta.outputs.tags }}" >> artifact-digest.txt
      
      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: build-artifact
          path: artifact-digest.txt
          retention-days: 7

  # ============================================================
  # Stage 3: Deploy to Staging
  # ============================================================
  deploy-staging:
    name: Deploy to Staging
    runs-on: ubuntu-latest
    needs: build
    environment:
      name: staging
      url: https://staging.example.com
    
    steps:
      - name: Download artifact
        uses: actions/download-artifact@v4
        with:
          name: build-artifact
      
      - name: Deploy to staging
        run: |
          # kubectl/helm/kustomize deployment
          kubectl set image deployment/api \
            api=${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}@${{ needs.build.outputs.digest }}
          
          # Wait for rollout
          kubectl rollout status deployment/api --timeout=10m
          
          # Run smoke tests
          ./scripts/smoke-test.sh https://staging.example.com

  # ============================================================
  # Stage 4: Integration Tests
  # ============================================================
  integration:
    name: Integration Tests
    runs-on: ubuntu-latest
    needs: deploy-staging
    if: github.event_name == 'push'
    
    steps:
      - name: Run integration suite
        run: |
          # Parallel test execution across services
          npm run test:integration -- --workers 4
      
      - name: Performance tests
        run: k6 run tests/performance/smoke.js
        env:
          K6_CLOUD_TOKEN: ${{ secrets.K6_CLOUD_TOKEN }}
          TARGET_URL: https://staging.example.com

  # ============================================================
  # Stage 5: Deploy to Production
  # ============================================================
  deploy-production:
    name: Deploy to Production
    runs-on: ubuntu-latest
    needs: [deploy-staging, integration]
    if: github.ref == 'refs/heads/main' || startsWith(github.ref, 'refs/tags/v')
    environment:
      name: production
      url: https://example.com
    
    steps:
      - name: Download artifact
        uses: actions/download-artifact@v4
        with:
          name: build-artifact
      
      - name: Deploy to production (blue-green)
        run: |
          # Deploy to canary (10% traffic)
          kubectl set image deployment/api \
            api=${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}@${{ needs.build.outputs.digest }}
          
          # Wait for canary
          kubectl rollout status deployment/api-canary --timeout=5m
          
          # Run validation
          ./scripts/validate.sh production
      
          # Full rollout
          kubectl patch deployment/api \
            -p '{"spec":{"strategy":{"type":"Recreate"}}}'
          kubectl set image deployment/api \
            api=${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}@${{ needs.build.outputs.digest }}
          kubectl rollout status deployment/api --timeout=15m
      
      - name: Notify success
        uses: slackapi/slack-github-action@v1
        with:
          payload: |
            {
              "text": "✅ Successfully deployed to production",
              "blocks": [
                {
                  "type": "section",
                  "text": {
                    "type": "mrkdwn",
                    "text": "*Deployment Successful*\n<${{ github.server_url }}/${{ github.repository }}/actions/runs/${{ github.run_id }}|View Run>"
                  }
                }
              ]
            }
        env:
          SLACK_WEBHOOK_URL: ${{ secrets.SLACK_WEBHOOK_URL }}
          SLACK_WEBHOOK_TYPE: INCOMING_WEBHOOK

  # ============================================================
  # Stage 6: Post-Deploy Verification
  # ============================================================
  verify:
    name: Post-Deploy Verification
    runs-on: ubuntu-latest
    needs: deploy-production
    if: always()
    
    steps:
      - name: Health check
        run: |
          for i in {1..5}; do
            if curl -sf https://example.com/healthz; then
              echo "Health check passed"
              exit 0
            fi
            echo "Attempt $i failed, retrying..."
            sleep 10
          done
          exit 1
      
      - name: Notify failure
        if: failure()
        uses: slackapi/slack-github-action@v1
        with:
          payload: |
            {
              "text": "❌ Deployment to production may have failed. Please verify.",
              "blocks": [
                {
                  "type": "section",
                  "text": {
                    "type": "mrkdwn",
                    "text": "*Deployment Warning*\n<${{ github.server_url }}/${{ github.repository }}/actions/runs/${{ github.run_id }}|View Run>"
                  }
                }
              ]
            }
        env:
          SLACK_WEBHOOK_URL: ${{ secrets.SLACK_WEBHOOK_URL }}
          SLACK_WEBHOOK_TYPE: INCOMING_WEBHOOK
```

#### Pull Request Pipeline
```yaml
# .github/workflows/pr.yml
name: PR Checks

on:
  pull_request:
    types: [opened, synchronize, reopened]
    branches: [main, develop]

env:
  NODE_VERSION: '20'
  PYTHON_VERSION: '3.11'

jobs:
  pr-checks:
    name: PR Validation
    runs-on: ubuntu-latest
    permissions:
      contents: read
      pull-requests: write
      checks: write
    
    services:
      postgres:
        image: postgres:15-alpine
        env:
          POSTGRES_USER: test
          POSTGRES_PASSWORD: test
          POSTGRES_DB: test
        ports:
          - 5432:5432
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
      
      redis:
        image: redis:7-alpine
        ports:
          - 6379:6379
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
    
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      
      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: ${{ env.NODE_VERSION }}
          cache: 'npm'
      
      - name: Install dependencies
        run: npm ci
      
      - name: Run lint
        run: npm run lint
      
      - name: Type check
        run: npm run typecheck
      
      - name: Run tests
        run: npm test -- --ci
        env:
          DATABASE_URL: postgresql://test:test@localhost:5432/test
          REDIS_URL: redis://localhost:6379
          NODE_ENV: test
      
      - name: Build
        run: npm run build
      
      - name: Run Trivy vulnerability scanner
        uses: aquasecurity/trivy-action@master
        with:
          scan-type: 'fs'
          scan-ref: '.'
          format: 'sarif'
          output: 'trivy-results.sarif'
      
      - name: Upload Trivy results
        uses: github/codeql-action/upload-sarif@v2
        with:
          sarif_file: 'trivy-results.sarif'
      
      - name: Comment on PR with coverage
        uses: romeovs/lcov-reporter-action@v0.3
        if: always()
        with:
          lcov-file: ./coverage/lcov.info
          github-token: ${{ secrets.GITHUB_TOKEN }}
          delete-old-comments: true
      
      - name: Add PR comment
        if: always()
        uses: actions/github-script@v7
        with:
          script: |
            const { execSync } = require('child_process');
            const { getOctokit, context } = require('@actions/github');
            
            const octokit = getOctokit(process.env.GITHUB_TOKEN);
            
            // Get test results
            const results = {
              workflow: context.workflow,
              run_id: context.runId,
              sha: context.sha,
              ref: context.ref
            };
            
            await octokit.rest.issues.createComment({
              ...context.repo,
              issue_number: context.issue.number,
              body: `## PR Checks\n\n**Run ID:** ${results.run_id}\n\nWorkflow triggered successfully. Review results below.`
            });
```

### 1.2 Reusable Workflows

```yaml
# .github/workflows/reusable-deploy.yml
on:
  workflow_call:
    inputs:
      environment:
        required: true
        type: string
      image-tag:
        required: true
        type: string
    secrets:
      KUBE_CONFIG:
        required: true
      SLACK_WEBHOOK:
        required: false

jobs:
  deploy:
    runs-on: ubuntu-latest
    environment: ${{ inputs.environment }}
    
    steps:
      - name: Setup kubectl
        uses: azure/setup-kubectl@v3
      
      - name: Configure kubectl
        run: |
          echo "${{ secrets.KUBE_CONFIG }}" | base64 -d > kubeconfig
          echo "KUBECONFIG=$(pwd)/kubeconfig" >> $GITHUB_ENV
      
      - name: Deploy
        run: |
          kubectl set image deployment/api \
            api=${{ inputs.image-tag }} \
            --namespace=${{ inputs.environment }}
          
          kubectl rollout status deployment/api \
            --namespace=${{ inputs.environment }} \
            --timeout=15m
      
      - name: Notify
        if: always()
        uses: slackapi/slack-github-action@v1
        with:
          payload: |
            {
              "text": "Deployment to ${{ inputs.environment }} completed",
              "blocks": [
                {
                  "type": "section",
                  "text": {
                    "type": "mrkdwn",
                    "text": "*Deploy: ${{ inputs.environment }}*\nImage: `${{ inputs.image-tag }}`"
                  }
                }
              ]
            }
        env:
          SLACK_WEBHOOK_URL: ${{ secrets.SLACK_WEBHOOK }}
```

---

## 2. ArgoCD (GitOps)

### 2.1 Application Manifests

```yaml
# argocd/app.yaml - Application definition
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: web-api
  namespace: argocd
  labels:
    app: web-api
    team: platform
  finalizers:
    - resources-finalizer.argocd.argoproj.io
spec:
  project: production
  
  source:
    repoURL: https://github.com/example/k8s-config.git
    targetRevision: HEAD
    path: apps/web-api/overlays/production
    kustomize:
      images:
        - api=ghcr.io/example/api:v1.2.3
    directory:
      recurse: true
  
  destination:
    server: https://kubernetes.default.svc
    namespace: production
  
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
      allowEmpty: false
    syncOptions:
      - CreateNamespace=true
      - PruneLast=true
      - ServerSideApply=true
      - Validate=true
    retry:
      limit: 5
      backoff:
        duration: 5s
        factor: 2
        maxDuration: 3m
    ignoreDifferences:
      - group: apps
        kind: Deployment
        jsonPointers:
          - /spec/replicas
      - group: ""
        kind: ServiceAccount
        jsonPointers:
          - /secrets
```

### 2.2 Kustomize Overlays

```yaml
# apps/web-api/base/kustomization.yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization

resources:
  - deployment.yaml
  - service.yaml
  - hpa.yaml
  - pdb.yaml
  - configmap.yaml
  - secret.yaml

commonLabels:
  app: web-api
  managed-by: argocd

images:
  - name: api
    newName: ghcr.io/example/api
    newTag: latest

configMapGenerator:
  - name: api-config
    literals:
      - ENVIRONMENT=production
      - LOG_LEVEL=info
    files:
      - config.json=config.json

secretGenerator:
  - name: api-secrets
    envs:
      - secrets.env
    options:
      disableNameSuffixHash: false

replicas:
  - name: api
    count: 3

vars:
  - name: API_VERSION
    objref:
      kind: ConfigMap
      name: api-config
      apiVersion: v1
      fieldpath: data.API_VERSION
```

```yaml
# apps/web-api/overlays/staging/kustomization.yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization

bases:
  - ../../base

patchesStrategicMerge:
  - deployment-patch.yaml

patches:
  - patch: |
      - op: replace
        path: /spec/replicas
        value: 2
    target:
      kind: Deployment
  - patch: |
      - op: replace
        path: /spec/template/spec/containers/0/resources/requests/cpu
        value: "100m"
    target:
      kind: Deployment

replicas:
  - name: api
    count: 2

commonLabels:
  env: staging

images:
  - name: api
    newTag: staging-latest

configMapGenerator:
  - name: api-config
    behavior: replace
    literals:
      - ENVIRONMENT=staging
      - LOG_LEVEL=debug
```

### 2.3 ArgoCD ApplicationSet (Multi-Cluster)

```yaml
# argocd/appset.yaml
apiVersion: argoproj.io/v1alpha1
kind: ApplicationSet
metadata:
  name: web-api-multicluster
  namespace: argocd
spec:
  generators:
    - matrix:
        generators:
          - clusters:
              selector:
                matchLabels:
                  environment: production
          - git:
              repoURL: https://github.com/example/k8s-config.git
              revision: HEAD
              paths:
                - clusters/*/web-api/*
  
  template:
    metadata:
      name: '{{name}}-web-api'
    spec:
      project: '{{metadata.labels.environment}}'
      source:
        repoURL: https://github.com/example/k8s-config.git
        targetRevision: HEAD
        path: 'clusters/{{metadata.labels.cluster}}/web-api'
      destination:
        server: '{{server}}'
        namespace: production
      syncPolicy:
        automated:
          prune: true
          selfHeal: true
```

---

## 3. Deployment Strategies

### 3.1 Blue-Green Deployment

```yaml
# Blue-green with nginx ingress
apiVersion: v1
kind: Service
metadata:
  name: api-bluegreen
  labels:
    app: api
spec:
  selector:
    role: api
    # Switch between blue and green
    slot: green
  ports:
    - port: 80
      targetPort: 8080
---
# Ingress with canary weight
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: api-ingress
  annotations:
    nginx.ingress.kubernetes.io/canary: "true"
    nginx.ingress.kubernetes.io/canary-weight: "10"  # 10% to new
spec:
  ingressClassName: nginx
  rules:
  - host: api.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: api-canary
            port:
              number: 80
```

```bash
# Deployment script
#!/bin/bash
set -euo pipefail

NEW_VERSION=$1
NAMESPACE=production

# Deploy new version (green)
kubectl set image deployment/api \
  api=ghcr.io/example/api:${NEW_VERSION} \
  --namespace=${NAMESPACE} \
  --selector=slot=green

# Wait for green to be ready
kubectl rollout status deployment/api \
  --namespace=${NAMESPACE} \
  --selector=slot=green \
  --timeout=10m

# Switch traffic (update service selector)
kubectl patch service api-bluegreen \
  --namespace=${NAMESPACE} \
  --type=merge \
  --patch='{"spec":{"selector":{"slot":"green"}}}'

# Wait a moment
sleep 30

# Run smoke tests
./smoke-tests.sh

# Scale down old version (blue)
kubectl scale deployment/api \
  --namespace=${NAMESPACE} \
  --replicas=0 \
  --selector=slot=blue

# Update deployment for next time
kubectl patch deployment api \
  --namespace=${NAMESPACE} \
  --type=merge \
  --patch='{"spec":{"selector":{"slot":"blue"}}}'
```

### 3.2 Canary Deployment

```yaml
# Canary deployment with HPA integration
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: api-canary
  namespace: production
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: api-canary
  minReplicas: 1
  maxReplicas: 10
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 50
---
# VirtualService for traffic splitting (Istio)
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: api-vs
  namespace: production
spec:
  hosts:
  - api.example.com
  http:
  - name: default
    route:
    - destination:
        host: api
        port:
          number: 80
      weight: 90
    - destination:
        host: api-canary
        port:
          number: 80
      weight: 10
  - name: specific-routes
    match:
    - headers:
        x-canary:
          exact: "true"
    route:
    - destination:
        host: api-canary
        port:
          number: 80
      weight: 100
```

### 3.3 Rolling Update with PDB

```yaml
# Deployment with rolling update
apiVersion: apps/v1
kind: Deployment
metadata:
  name: api
  namespace: production
spec:
  replicas: 10
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 2        # Can have 12 total during update
      maxUnavailable: 0   # Always maintain 10
  minReadySeconds: 30
  progressDeadlineSeconds: 600
  selector:
    matchLabels:
      app: api
  template:
    spec:
      topologySpreadConstraints:
      - maxSkew: 1
        topologyKey: topology.kubernetes.io/zone
        whenUnsatisfiable: DoNotSchedule
        labelSelector:
          matchLabels:
            app: api
---
# PodDisruptionBudget
apiVersion: policy/v1
kind: PodDisruptionBudget
metadata:
  name: api-pdb
  namespace: production
spec:
  minAvailable: 8  # At least 8 pods during disruptions
  selector:
    matchLabels:
      app: api
```

---

## 4. Secret Management

### 4.1 External Secrets Operator

```yaml
# external-secret.yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: api-secrets
  namespace: production
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: vault-backend
    kind: ClusterSecretStore
  target:
    name: api-secrets
    creationPolicy: Owner
    deletionPolicy: Retain
  data:
  - secretKey: DATABASE_URL
    remoteRef:
      key: production/api
      property: database_url
  - secretKey: STRIPE_KEY
    remoteRef:
      key: production/api
      property: stripe_key
  - secretKey: JWT_SECRET
    remoteRef:
      key: production/api
      property: jwt_secret
  # Template for complex secrets
  - secretKey: config.json
    remoteRef:
      key: production/api-config
    templating:
      engine: jsonata
      expression: |
        $$.config
```

---

## 5. Decision Frameworks

### 5.1 Strategy Selection

| Scenario | Recommended Strategy |
|----------|---------------------|
| Database schema changes | Blue-green (instant switch) |
| Major version upgrades | Blue-green |
| Hotfix emergency | Rolling with extra caution |
| New feature rollout | Canary (gradual) |
| A/B testing | Canary with traffic splitting |
| Zero-downtime required | Blue-green or canary |
| Low-risk minor update | Rolling |
| State-heavy services | Blue-green |

### 5.2 Pipeline Stage Checklist

```yaml
# Required stages:
1. Source: Checkout, dependency restore
2. Quality: Lint, type check, test, security scan
3. Build: Compile, package, containerize
4. Security: Scan image, sign, push to registry
5. Staging: Deploy to staging, integration tests
6. Production: Deploy, smoke tests, monitoring
7. Verify: Post-deploy checks, rollback capability

# Optional stages based on risk:
- Performance testing (major releases)
- Chaos testing (new infrastructure)
- Contract testing (API changes)
- Regression testing (user acceptance)
```

---

## Links

### Core Router
- `core/DECAPOD.md` - **Router and navigation charter (START HERE)**
- `core/ENGINEERING_EXCELLENCE.md` - Engineering standards

### Architecture (This Section)
- `architecture/KUBERNETES.md` - Deployment targets, GitOps
- `architecture/DATABASE.md` - Database migrations
- `architecture/AUTH.md` - Secret management in pipelines
- `architecture/MESSAGING.md` - Pipeline event triggers

### Authority (Constitution Layer)
- `specs/INTENT.md` - **Methodology contract (READ FIRST)**
- `specs/SYSTEM.md` - System definition and authority doctrine
- `specs/SECURITY.md` - Security doctrine
- `specs/GIT.md` - Git workflow contracts

### Interface Contracts
- `interfaces/CLAIMS.md` - Promises ledger
- `interfaces/CONTROL_PLANE.md` - Agent sequencing patterns
- `interfaces/TESTING.md` - Testing contracts

### Methodology
- `methodology/ARCHITECTURE.md` - Architecture decision methodology
- `methodology/CI_CD.md` - CI/CD methodology guides
- `methodology/RELEASE_MANAGEMENT.md` - Release procedures

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2024-01-16 | Initial comprehensive CI/CD reference |