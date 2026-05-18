# SECRETS.md - Secrets Management Architecture

**Authority:** guidance (comprehensive topic with exact specifications)
**Layer:** Architecture
**Binding:** No
**Scope:** Comprehensive topic coverage for pre-inference context

## Table of Contents

1. [Vault Patterns](#1-vault-patterns)
2. [AWS Secrets Manager](#2-aws-secrets-manager)
3. [Kubernetes Secrets](#3-kubernetes-secrets)
4. [Secret Rotation](#4-secret-rotation)
5. [SPIFFE/SPIRE](#5-spirffe-spire)
6. [Complete Configurations](#6-complete-configurations)
7. [Decision Matrices](#7-decision-matrices)
8. [Anti-Patterns and Failure Modes](#8-anti-patterns-and-failure-modes)
9. [Production Checklist](#9-production-checklist)
10. [References](#10-references)

---

## 1. Vault Patterns

### 1.1 HashiCorp Vault Architecture

Vault is a secrets management solution providing encryption, key management, and access control for secrets.

**Key Components:**
- **Storage Backend**: Where encrypted data is stored (Consul, S3, PostgreSQL, etc.)
- **Secret Engines**: Components that store, generate, or encrypt secrets
- **Auth Methods**: How applications authenticate to Vault
- **Audit Devices**: Logging of all requests and responses

### 1.2 Vault Server Configuration

```hcl
# /etc/vault/config.hcl

# Storage backend (Consul)
storage "consul" {
  address        = "consul.platform.svc.cluster.local:8500"
  scheme         = "https"
  token          = "your-consul-token"
  path           = "vault/"
  max_parallel   = 128
  
  # TLS configuration
  tls_ca_file     = "/etc/vault/tls/ca.crt"
  tls_cert_file   = "/etc/vault/tls/vault.crt"
  tls_key_file    = "/etc/vault/tls/vault.key"
  
  # High availability
  disable_registration  = false
  retry_join_etag       = true
}

# HA backend
ha_storage "consul" {
  address        = "consul.platform.svc.cluster.local:8500"
  scheme         = "https"
  token          = "your-consul-token"
  path           = "vault/"
}

# Listener configuration
listener "tcp" {
  address         = "[::]:8200"
  cluster_address = "[::]:8201"
  
  # TLS configuration
  tls_cert_file   = "/etc/vault/tls/vault.crt"
  tls_key_file    = "/etc/vault/tls/vault.key"
  tls_client_ca_file = "/etc/vault/tls/ca.crt"
  
  # Performance
  max_request_duration     = "90s"
  max_request_size         = 33554432  # 32MB
  request_timeout          = "60s"
  
  # Proxy protocol (for load balancers)
  proxy_protocol_behavior   = "deny_authorized"
  proxy_protocol_authorized_addrs = "10.0.0.0/8"
}

# Telemetry
telemetry {
  prometheus_retention_time = "30s"
  disable_hostname = true
  
  statsd_address = "statsd.honitoring.svc.cluster.local:9125"
}

# Logging
log_level = "INFO"
log_format = "json"
log_file = "/var/log/vault/vault.log"

# Seals (auto-unseal with AWS KMS)
seal "awskms" {
  region     = "us-east-1"
  kms_key_id = "alias/vault-kms-key"
}

# Cluster settings
cluster_addr = "https://vault-0.platform.svc.cluster.local:8201"
api_addr = "https://vault.platform.svc.cluster.local:8200"
ui = true
```

### 1.3 Vault Secret Engines Configuration

```yaml
# Kubernetes deployment for Vault with all secret engines configured
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: vault
  namespace: platform
spec:
  serviceName: vault
  replicas: 3
  podManagementPolicy: Parallel
  selector:
    matchLabels:
      app: vault
  template:
    metadata:
      labels:
        app: vault
    spec:
      securityContext:
        runAsNonRoot: true
        runAsUser: 100
        fsGroup: 1000
      serviceAccountName: vault
      containers:
      - name: vault
        image: hashicorp/vault:1.15.0
        command: ["vault", "server", "-config=/vault/config/config.hcl"]
        ports:
        - containerPort: 8200
          name: http
        - containerPort: 8201
          name: https-internal
        env:
        - name: VAULT_ADDR
          value: "https://vault.platform.svc.cluster.local:8200"
        - name: VAULT_CACERT
          value: /vault/tls/ca.crt
        - name: SKIP_CHOWN
          value: "true"
        - name: SKIP_SETCAP
          value: "true"
        - name: VAULT_SKIP_VERIFY
          value: "false"
        livenessProbe:
          httpGet:
            path: /v1/sys/health?standbyok=true&sealedcode=200&uninitcode=200
            port: 8200
          initialDelaySeconds: 10
          periodSeconds: 5
          failureThreshold: 3
        readinessProbe:
          httpGet:
            path: /v1/sys/health?standbyok=true
            port: 8200
          initialDelaySeconds: 5
          periodSeconds: 5
        resources:
          requests:
            cpu: 500m
            memory: 1Gi
          limits:
            cpu: 2000m
            memory: 4Gi
        securityContext:
          readOnlyRootFilesystem: false
          allowPrivilegeEscalation: false
          capabilities:
            drop:
              - ALL
        volumeMounts:
        - name: config
          mountPath: /vault/config
          readOnly: true
        - name: data
          mountPath: /vault/data
        - name: logs
          mountPath: /var/log/vault
        - name: tls
          mountPath: /vault/tls
          readOnly: true
      volumes:
      - name: config
        configMap:
          name: vault-config
      - name: tls
        secret:
          secretName: vault-tls
      - name: data
        persistentVolumeClaim:
          claimName: vault-data
---
# Vault Agent Injector deployment
apiVersion: apps/v1
kind: Deployment
metadata:
  name: vault-agent-injector
  namespace: platform
spec:
  replicas: 2
  selector:
    matchLabels:
      app: vault-agent-injector
  template:
    metadata:
      labels:
        app: vault-agent-injector
    spec:
      serviceAccountName: vault-agent-injector
      containers:
      - name: vault-agent-injector
        image: hashicorp/vault:1.15.0
        command: ["vault", "agent-injector", "-config=/vault/config/agent-config.hcl"]
        ports:
        - containerPort: 8080
          name: api
        env:
        - name: AGENT_INJECT_LISTEN
          value: ":8080"
        - name: AGENT_INJECT_VAULT_ADDR
          value: "https://vault.platform.svc.cluster.local:8200"
        - name: AGENT_INJECT_TLS_AUTO
          value: "vault-agent-injector-svc"
        - name: AGENT_INJECT_TLS_AUTO_HOSTS
          value: "vault-agent-injector,localhost"
        resources:
          requests:
            cpu: 100m
            memory: 128Mi
          limits:
            cpu: 500m
            memory: 512Mi
```

### 1.4 Vault Policies

```hcl
# vault-policy.hcl - Policy for application secrets

# Enable Kubernetes auth method for this namespace
path "auth/kubernetes/login" {
  capabilities = ["create", "read"]
}

# Database secrets
path "database/creds/order-service-role" {
  capabilities = ["read"]
}

path "database/creds/order-service-role/*" {
  capabilities = ["read"]
}

# Generic secrets
path "secret/data/platform/order-service/*" {
  capabilities = ["read", "list"]
}

path "secret/metadata/platform/order-service/*" {
  capabilities = ["list"]
}

# PKI secrets for certificates
path "pki/issue/order-service-domain" {
  capabilities = ["create", "update"]
}

path "pki/certs" {
  capabilities = ["read", "list"]
}

# Transit secrets for encryption
path "transit/encrypt/order-service-key" {
  capabilities = ["update"]
}

path "transit/decrypt/order-service-key" {
  capabilities = ["update"]
}

# AWS secrets
path "aws/creds/order-service-role" {
  capabilities = ["read"]
}

# AppRole for legacy systems
path "auth/approle/role/order-service" {
  capabilities = ["read"]
}

# Limit secret access to specific namespace labels
# This requires the namespace label to match
```

### 1.5 Vault Kubernetes Auth Configuration

```yaml
# Enable and configure Kubernetes auth method
apiVersion: v1
kind: ConfigMap
metadata:
  name: vault-k8s-config
data:
  config.yaml: |
    kubernetes:
      host: https://kubernetes.default.svc
      ca_cert: /var/run/secrets/kubernetes.io/serviceaccount/ca.crt
      token_reviewer_jwt: /var/run/secrets/token
      namespace: platform
    # Service account to validate tokens
    service_account_annotator: vault.hashicorp.com/service-account-name
      
---
# Vault Kubernetes auth role configuration
apiVersion: v1
kind: ServiceAccount
metadata:
  name: vault-auth
  namespace: platform
---
# Create a role that binds to the service account
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: vault-auth-role
  namespace: platform
rules:
  - apiGroups: [""]
    resources: ["serviceaccounts/token"]
    verbs: ["create"]
  - apiGroups: [""]
    resources: ["pods"]
    verbs: ["get", "list"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: vault-auth-rolebinding
  namespace: platform
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: Role
  name: vault-auth-role
subjects:
  - kind: ServiceAccount
    name: vault-auth
    namespace: platform
```

### 1.6 Dynamic Database Credentials

```yaml
# Database secret engine configuration
apiVersion: v1
kind: Secret
metadata:
  name: vault-database-config
type: Opaque
stringData:
  config.hcl: |
    # Configure PostgreSQL database secret engine
    # This would be done via Vault CLI or API
    
# Vault commands to set up database secrets:
# vault secrets enable -path=database database
# vault write database/config/postgresql \
#     plugin_name=postgresql-database-plugin \
#     connection_url="postgresql://{{username}}:{{password}}@postgres.platform.svc.cluster.local:5432/postgres?sslmode=require" \
#     allowed_roles="order-service-role" \
#     username="vault-admin" \
#     password="admin-password"
#
# vault write database/roles/order-service-role \
#     db_name=postgresql \
#     creation_statements="CREATE ROLE \"{{name}}\" WITH LOGIN PASSWORD '{{password}}' VALID UNTIL '{{expiration}}'; GRANT SELECT ON ALL TABLES IN SCHEMA public TO \"{{name}}\";" \
#     default_ttl="1h" \
#     max_ttl="24h"

---
# Kubernetes manifest for Vault database role binding
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: order-service-db-role
  namespace: platform
rules:
  - apiGroups: [""]
    resources: ["secrets"]
    verbs: ["create", "update", "get", "list"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: order-service-db-rolebinding
  namespace: platform
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: Role
  name: order-service-db-role
subjects:
  - kind: ServiceAccount
    name: order-service
    namespace: platform
```

---

## 2. AWS Secrets Manager

### 2.1 AWS Secrets Manager Configuration

```yaml
# AWS Secrets Manager configuration for Kubernetes
aws_secrets_manager:
  # Region and endpoint
  region: us-east-1
  endpoint: null  # Use AWS default
  
  # Authentication
  secret_arn: arn:aws:secretsmanager:us-east-1:123456789012:secret:order-service-creds
  secret_prefix: /platform/order-service/
  
  # Caching
  cache:
    enabled: true
    ttl: 3600  # 1 hour in seconds
    
  # Retry configuration
  retry:
    max_attempts: 3
    backoff: exponential
    initial_delay: 100ms
    max_delay: 5s
    
  # Version tracking
  version:
    stage: AWSCURRENT
    version_id: null  # Latest by default
    
  # Tags for organization
  tags:
    environment: production
    service: order-service
    managed-by: aws-secrets-manager

# CloudWatch Events for rotation
cloudwatch_events:
  enabled: true
  schedule: "rate(30 days)"
```

### 2.2 External Secrets Operator Configuration

```yaml
# External Secrets Operator ClusterSecretStore
apiVersion: external-secrets.io/v1beta1
kind: ClusterSecretStore
metadata:
  name: aws-secrets-manager
  namespace: platform
spec:
  provider:
    aws:
      service: SecretsManager
      region: us-east-1
      auth:
        jwt:
          serviceAccountRef:
            name: external-secrets-sa
            namespace: platform
---
# External Secrets Operator ExternalSecret
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: order-service-secrets
  namespace: platform
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: aws-secrets-manager
    kind: ClusterSecretStore
  target:
    name: order-service-secrets
    creationPolicy: Owner
    deletionPolicy: Retain
  data:
    - secretKey: database-url
      remoteRef:
        key: /platform/order-service/database
        property: url
    - secretKey: redis-password
      remoteRef:
        key: /platform/order-service/redis
        property: password
    - secretKey: kafka-credentials
      remoteRef:
        key: /platform/order-service/kafka
        property: password
        conversionStrategy: Default
    - secretKey: jwt-secret
      remoteRef:
        key: /platform/order-service/jwt
        property: secret
---
# External Secrets Operator PushSecret (for syncing k8s secrets to AWS)
apiVersion: external-secrets.io/v1beta1
kind: PushSecret
metadata:
  name: push-to-aws
  namespace: platform
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: aws-secrets-manager
    kind: ClusterSecretStore
  selector:
    secretTemplates:
    - matchRules:
        labelSelector:
          matchLabels:
            push-to-aws: "true"
    metadata:
      labels:
        created-by: pushsecret
  target:
    creationPolicy: Owner
    deletionPolicy: Delete
  data:
    - match:
        secretKey: database-credentials
        remoteRef:
          key: /platform/order-service/database-backup
```

---

## 3. Kubernetes Secrets

### 3.1 Kubernetes Secrets Configuration

```yaml
# Kubernetes Secrets with encryption at rest
apiVersion: v1
kind: Secret
metadata:
  name: order-service-secrets
  namespace: platform
  labels:
    app: order-service
    managed-by: vault
  annotations:
    kubernetes.io/description: "Secrets for order-service application"
type: Opaque
data:
  # Base64 encoded values - these should be generated, not hardcoded
  database-password: <base64-encoded-password>
  redis-password: <base64-encoded-password>
  jwt-secret: <base64-encoded-secret>
  api-keys: <base64-encoded-keys>
stringData:
  # Alternative: use stringData for plaintext (will be base64 encoded)
  database-username: "order-service"
---
# Encrypted Kubernetes Secret using Sealed Secrets
apiVersion: bitnami.com/v1alpha1
kind: SealedSecret
metadata:
  name: order-service-secrets
  namespace: platform
spec:
  encryptedData:
    database-password: AgA...  # Encrypted with Sealed Secrets public key
    redis-password: BhB...
    jwt-secret: ChC...
  template:
    metadata:
      labels:
        app: order-service
      annotations:
        sealedsecrets.bitnami.com/managed: "true"
---
# ESO-generated Secret (immutable once created)
apiVersion: v1
kind: Secret
metadata:
  name: order-service-secrets
  namespace: platform
  labels:
    app: order-service
  annotations:
    external-secrets.io/connection: aws-secrets-manager
    external-secrets.io/owner: platform/order-service-secrets
type: Opaque
data:
  database-url: <auto-populated>
  redis-password: <auto-populated>
```

### 3.2 Kubernetes Secrets Encryption Configuration

```yaml
# Enable encryption at rest for etcd
apiVersion: apiserver.config.k8s.io/v1
kind: EncryptionConfiguration
metadata:
  name: encryption-config
resources:
  - resources:
      - secrets
      - configmaps
    providers:
      # AES-GCM with 256-bit key (recommended for production)
      - aescbc:
          keys:
            - name: key1
              secret: <base64-encoded-256-bit-key>
      # AES-GCM with KMS plugin (for cloud deployments)
      - kms:
          name: vault-encryption-provider
          endpoint: unix:///var/run/kmsprovider.sock
          cachesize: 1000
          timeout: 3s
      # Encrypted identity (fallback, not recommended for secrets)
      - identity: {}
```

### 3.3 Vault Agent Injector Integration

```yaml
# Service with Vault annotations for automatic secret injection
apiVersion: apps/v1
kind: Deployment
metadata:
  name: order-service
  namespace: platform
spec:
  selector:
    matchLabels:
      app: order-service
  template:
    metadata:
      labels:
        app: order-service
      annotations:
        # Enable Vault agent injection
        vault.hashicorp.com/agent-inject: "true"
        
        # Vault address
        vault.hashicorp.com/agent-inject-address: "https://vault.platform.svc.cluster.local:8200"
        
        # Auth method
        vault.hashicorp.com/agent-inject-auth-method: "kubernetes"
        vault.hashicorp.com/agent-inject-auth-role: "order-service"
        
        # Template for database credentials
        vault.hashicorp.com/agent-inject-template-database-url: |
          {{- with secret "database/creds/order-service-role" -}}
          postgresql://{{ .Data.data.username }}:{{ .Data.data.password }}@postgres.platform.svc.cluster.local:5432/orders?sslmode=require
          {{- end }}
        
        # Database credentials (automatic injection)
        vault.hashicorp.com/agent-inject-secret-database-creds: "database/creds/order-service-role"
        
        # PKI certificates (automatic injection)
        vault.hashicorp.com/agent-inject-secret-tls-cert: "pki/issue/order-service-domain"
        vault.hashicorp.com/agent-inject-template-tls-cert: |
          {{- with secret "pki/issue/order-service-domain" "common_name=order-service.platform.svc.cluster.local" -}}
          {{ .Data.data.certificate }}{{ .Data.data.issuing_ca }}{{ .Data.data.private_key }}
          {{- end }}
        
        # Environment variable injection
        vault.hashicorp.com/agent-inject-env: "true"
        vault.hashicorp.com/agent-inject-env-DATABASE_URL: "database/creds/order-service-role"
        
        # Service account annotation
        vault.hashicorp.com/service-account-name: "order-service"
        
        # Pre-population
        vault.hashicorp.com/agent-pre-populate-only: "false"
        vault.hashicorp.com/agent-init-first: "true"
        
        # TLS configuration
        vault.hashicorp.com/agent-tls-ca-cert: /var/run/certs/vault-ca.crt
        vault.hashicorp.com/agent-tls-cert-file: /var/run/certs/vault.crt
        vault.hashicorp.com/agent-tls-key-file: /var/run/certs/vault.key
        vault.hashicorp.com/agent-tls-verify: "true"
    spec:
      serviceAccountName: order-service
      containers:
      - name: order-service
        image: order-service:1.2.3
        env:
        - name: DATABASE_URL
          value: /vault/secrets/database-creds
        - name: VAULT_CACERT
          value: /var/run/certs/vault-ca.crt
        volumeMounts:
        - name: vault-certs
          mountPath: /var/run/certs
        - name: vault-secrets
          mountPath: /vault/secrets
      volumes:
      - name: vault-certs
        secret:
          secretName: vault-tls
      - name: vault-secrets
        emptyDir:
          medium: Memory
```

---

## 4. Secret Rotation

### 4.1 Vault Dynamic Secret Rotation

```yaml
# Vault rotation configuration
rotation:
  # PostgreSQL credential rotation
  database:
    enabled: true
    rotation_period: 24h  # Rotate every 24 hours
    role: order-service-role
    provider: postgresql
    config:
      connection_url: postgresql://admin:password@postgres.platform.svc.cluster.local:5432/admin?sslmode=require
      max_connections: 10
      max_idle_connections: 2
      max_connection_lifetime: 1h
    hooks:
      pre_rotation:
        command: "/scripts/pre-rotation-hook.sh"
        timeout: 30s
      post_rotation:
        command: "/scripts/post-rotation-hook.sh"
        timeout: 30s
        
  # AWS credentials rotation
  aws:
    enabled: true
    rotation_period: 1h  # Rotate every hour
    role: order-service-role
    config:
      region: us-east-1
      iam_user_prefix: order-service
    hooks:
      pre_rotation:
        command: "/scripts/aws-pre-rotation.sh"
      post_rotation:
        command: "/scripts/aws-post-rotation.sh"
```

### 4.2 Database Password Rotation Procedure

```python
# Rotation script example for database credentials
import hvac
import psycopg2
from datetime import datetime
import os

class DatabaseCredentialRotator:
    def __init__(self, vault_addr, role_name, db_connection_url):
        self.vault_addr = vault_addr
        self.role_name = role_name
        self.db_connection_url = db_connection_url
        
    def rotate(self):
        # 1. Generate new credentials from Vault
        client = hvac.Client(url=self.vault_addr)
        response = client.secrets.database.generate_credentials(role_name=self.role_name)
        
        new_username = response['data']['username']
        new_password = response['data']['password']
        
        # 2. Create connection with new credentials
        new_db_url = self.db_connection_url.replace('{{username}}', new_username).replace('{{password}}', new_password)
        
        # 3. Test connection with new credentials
        try:
            conn = psycopg2.connect(new_db_url)
            conn.close()
        except Exception as e:
            raise Exception(f"New credentials failed validation: {e}")
        
        # 4. Revoke old credentials (this requires a hook system to ensure no disruption)
        # This should be done carefully to avoid breaking in-flight requests
        
        return {
            'username': new_username,
            'password': new_password,
            'rotated_at': datetime.utcnow().isoformat()
        }
```

### 4.3 Automatic Secret Rotation Configuration

```yaml
# Kubernetes CronJob for automatic secret rotation
apiVersion: batch/v1
kind: CronJob
metadata:
  name: secret-rotation
  namespace: platform
spec:
  schedule: "0 2 * * *"  # Daily at 2 AM
  concurrencyPolicy: Forbid
  jobTemplate:
    spec:
      template:
        spec:
          serviceAccountName: secret-rotation
          restartPolicy: OnFailure
          containers:
          - name: rotation
            image: vault:1.15.0
            command: ["vault", "operator", "rotate", "-format=json"]
            env:
            - name: VAULT_ADDR
              value: "https://vault.platform.svc.cluster.local:8200"
            - name: VAULT_TOKEN
              valueFrom:
                secretKeyRef:
                  name: vault-token
                  key: token
          - name: db-rotation
            image: your-rotation-app:latest
            args: ["--rotation-type=database", "--role=order-service-role"]
            env:
            - name: VAULT_ADDR
              value: "https://vault.platform.svc.cluster.local:8200"
```

---

## 5. SPIFFE/SPIRE

### 5.1 SPIFFE ID and Workload API

SPIFFE (Secure Production Identity Framework for Everyone) provides a standard for workload identity.

**SPIFFE ID Format**: `spiffe://<trust-domain>/<workload-namespace>/<workload-name>`

**Trust Domain**: The root of trust for your organization (e.g., `example.com`)

### 5.2 SPIRE Server and Agent Configuration

```yaml
# SPIRE Server configuration
apiVersion: v1
kind: ConfigMap
metadata:
  name: spire-server
  namespace: spire
data:
  server.conf: |
    server {
      bind_address = "0.0.0.0"
      bind_port = "8081"
      trust_domain = "example.com"
      data_dir = "/opt/spire/data/server"
      log_level = "INFO"
      database_url = "postgresql://spire:password@postgres.spire:5432/spire?sslmode=require"
      
      # Federation
      federation {
        bundle_endpoint_url = "https://spire-server.example.com:8443"
        # For cross-trust-domain communication
      }
    }
    
    plugins {
      DataStore "sql" {
        plugin_data {
          database_type = "postgresql"
          connection_string = "postgresql://spire:password@postgres.spire:5432/spire?sslmode=require"
        }
      }
      
      NodeAttestor "k8s_psat" {
        plugin_data {
          clusters = {
            "production" = {
              service_account_allow_list = ["platform:spire-agent"]
            }
          }
        }
      }
      
      NodeResolver "k8s_psat" {
        plugin_data {
          clusters = {
            "production" = {
              service_account_allow_list = ["platform:spire-agent"]
            }
          }
        }
      }
    }
    
    trust_ca:
      # Root CA for issuing workload identities
      subject = "CN=example.com SPIFFE CA,O=Example Inc"
      expiry = "87600h"  # 10 years
      
    # CA rotation
    ca_rotation {
      rotation_interval = "24h"
      validity_period = "72h"
    }
---
# SPIRE Agent configuration
apiVersion: v1
kind: ConfigMap
metadata:
  name: spire-agent
  namespace: spire
data:
  agent.conf: |
    agent {
      data_dir = "/opt/spire/data/agent"
      trust_domain = "example.com"
      trust_bundle_path = "/opt/spire/bundle/cert.pem"
      log_level = "INFO"
      
      # Workload API
      socket_path = "/run/spire/sockets/agent.sock"
      insecure_allow_unverified_verification = false
    }
    
    plugins {
      NodeAttestor "k8s_psat" {
        plugin_data {
          cluster = "production"
        }
      }
      
      WorkloadAttestor "k8s" {
        plugin_data {
          skip_kubelet_verification = false
          max_poll_interval = 60s
        }
      }
      
      WorkloadAttestor "unix" {
        plugin_data {
          use_new_cgroup = true
        }
      }
    }
```

### 5.3 SPIRE Registration and Workload Configuration

```yaml
# SPIRE Server Deployment
apiVersion: apps/v1
kind: Deployment
metadata:
  name: spire-server
  namespace: spire
spec:
  replicas: 2
  selector:
    matchLabels:
      app: spire-server
  template:
    metadata:
      labels:
        app: spire-server
    spec:
      serviceAccountName: spire-server
      containers:
      - name: spire-server
        image: gcr.io/spiffe-io/spire-server:1.6.3
        args:
          - -config
          - /opt/spire/config/server.conf
        ports:
        - containerPort: 8081
          name: grpc-api
        - containerPort: 8443
          name: federation-endpoint
        livenessProbe:
          httpGet:
            path: /liveness
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
        readinessProbe:
          httpGet:
            path: /readiness
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
        resources:
          requests:
            cpu: 100m
            memory: 256Mi
          limits:
            cpu: 500m
            memory: 1Gi
        volumeMounts:
        - name: spire-config
          mountPath: /opt/spire/config
          readOnly: true
        - name: spire-data
          mountPath: /opt/spire/data
        - name: spire-registration-socket
          mountPath: /run/spire
      volumes:
      - name: spire-config
        configMap:
          name: spire-server
      - name: spire-data
        persistentVolumeClaim:
          claimName: spire-data
      - name: spire-registration-socket
        hostPath:
          path: /run/spire/registration
          type: DirectoryOrCreate

---
# SPIRE Agent DaemonSet
apiVersion: apps/v1
kind: DaemonSet
metadata:
  name: spire-agent
  namespace: spire
spec:
  selector:
    matchLabels:
      app: spire-agent
  template:
    metadata:
      labels:
        app: spire-agent
    spec:
      serviceAccountName: spire-agent
      hostPID: true
      dnsPolicy: ClusterFirst
      containers:
      - name: spire-agent
        image: gcr.io/spiffe-io/spire-agent:1.6.3
        args:
          - -config
          - /opt/spire/config/agent.conf
        env:
        - name: SPIRE_AGENT_NODE_NAME
          valueFrom:
            fieldRef:
              fieldPath: spec.nodeName
        securityContext:
          privileged: true
        volumeMounts:
        - name: spire-config
          mountPath: /opt/spire/config
          readOnly: true
        - name: spire-data
          mountPath: /opt/spire/data
        - name: spire-socket
          mountPath: /run/spire/sockets
        - name: spire-agent-socket
          mountPath: /run/secrets/workload-api
        - name: kubelet-certs
          mountPath: /var/lib/kubelet/pki
          readOnly: true
      volumes:
      - name: spire-config
        configMap:
          name: spire-agent
      - name: spire-data
        hostPath:
          path: /opt/spire/data
          type: DirectoryOrCreate
      - name: spire-socket
        hostPath:
          path: /run/spire/sockets
          type: DirectoryOrCreate
      - name: spire-agent-socket
        hostPath:
          path: /run/secrets/workload-api
          type: DirectoryOrCreate
      - name: kubelet-certs
        hostPath:
          path: /var/lib/kubelet/pki
          type: Directory
```

### 5.4 SPIFFE Workload Registration

```yaml
# SPIRE Registration Entry for a Kubernetes workload
apiVersion: spire.spiffe.io/v1alpha1
kind: ClusterSPIFFEID
metadata:
  name: order-service-identity
  namespace: spire
spec:
  spiffeIDTemplate: "spiffe://example.com/platform/{{.PodMeta.Namespace}}/{{.PodMeta.Name}}"
  podSelector:
    matchLabels:
      app: order-service
  namespaceSelector:
    matchLabels:
      kubernetes.io/metadata.name: platform
  federatesWith:
    - "partner.example.com"
    - "legacy.example.com"
  sans:
    dnsNames:
      - order-service.platform.svc.cluster.local
      - order-service.platform
    ipAddresses:
      - "10.0.0.0"
      
---
# Registration entry for database access
apiVersion: spire.spiffe.io/v1alpha1
kind: ClusterSPIFFEID
metadata:
  name: postgres-identity
  namespace: spire
spec:
  spiffeIDTemplate: "spiffe://example.com/database/postgres"
  podSelector:
    matchLabels:
      app: postgresql
  namespaceSelector:
    matchLabels:
      kubernetes.io/metadata.name: platform
      
---
# Registration entry for service mesh mTLS
apiVersion: spire.spiffe.io/v1alpha1
kind: ClusterSPIFFEID
metadata:
  name: service-mesh-identity
  namespace: spire
spec:
  spiffeIDTemplate: "spiffe://example.com/service-mesh/{{.PodMeta.Namespace}}/{{.PodMeta.Name}}"
  podSelector: {}
  namespaceSelector:
    matchLabels:
      kubernetes.io/metadata.name: platform
  registerAmended: true
```

---

## 6. Complete Configurations

### 6.1 AWS Secrets Manager Secret Creation

```yaml
# Terraform configuration for AWS Secrets Manager
resource "aws_secretsmanager_secret" "order_service" {
  name                    = "/platform/order-service/database"
  description             = "Database credentials for order-service"
  recovery_window_in_days  = 30
  rotation_lambda_arn     = aws_lambda_function.rotation_lambda.arn
  
  tags = {
    Environment = "production"
    Service     = "order-service"
    ManagedBy   = "terraform"
  }
}

resource "aws_secretsmanager_secret_rotation" "order_service" {
  secret_id     = aws_secretsmanager_secret.order_service.id
  rotation_lambda_arn = aws_lambda_function.rotation_lambda.arn
  rotation_rules {
    automatically_after_days = 30
  }
}

resource "aws_secretsmanager_secret_version" "order_service" {
  secret_id = aws_secretsmanager_secret.order_service.id
  
  secret_string = jsonencode({
    username = "order_service"
    password = "initial-password"
    host     = "postgres.platform.svc.cluster.local"
    port     = 5432
    database = "orders"
    ssl_mode = "require"
  })
}

# Lambda function for automatic rotation
resource "aws_lambda_function" "rotation_lambda" {
  filename         = "rotation_function.zip"
  function_name    = "order-service-credentials-rotation"
  role            = aws_iam_role.rotation_lambda.arn
  handler         = "rotation_function.handler"
  source_code_hash = filebase64sha256("rotation_function.zip")
  runtime         = "python3.11"
  timeout         = 30
  
  environment {
    variables = {
      DB_HOST = "postgres.platform.svc.cluster.local"
      DB_PORT = "5432"
      DB_NAME = "orders"
    }
  }
}
```

### 6.2 Cross-Account Secret Access

```yaml
# Cross-account secret access via STS
resource "aws_iam_role" "cross_account_secrets" {
  name = "cross-account-secrets-access"
  
  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = "sts:AssumeRole"
        Principal = {
          AWS = "arn:aws:iam::123456789012:root"  # Source account
        }
        Condition = {
          StringEquals = {
            "sts:Externalid" = "order-service-external-id"
          }
        }
      }
    ]
  })
}

resource "aws_iam_role_policy" "cross_account_secrets" {
  name = "cross-account-secrets-policy"
  role = aws_iam_role.cross_account_secrets.id
  
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "secretsmanager:GetSecretValue",
          "secretsmanager:DescribeSecret"
        ]
        Resource = "arn:aws:secretsmanager:us-east-1:123456789012:secret:/platform/*"
      }
    ]
  })
}
```

---

## 7. Decision Matrices

### 7.1 Secrets Management Solution Selection

| Requirement | Kubernetes Secrets | Vault | AWS Secrets Manager | Azure Key Vault | GCP Secret Manager |
|-------------|-------------------|-------|---------------------|-----------------|-------------------|
| Encryption at rest | Partial | Full | Full | Full | Full |
| Dynamic secrets | No | Yes | Yes | Yes | Yes |
| Secret rotation | Manual | Automatic | Automatic | Automatic | Automatic |
| Audit logging | Limited | Full | Full | Full | Full |
| Multi-cloud | Yes | Yes | No | No | No |
| Cost | Low | Medium | Medium | Medium | Medium |
| Compliance | Limited | Full | Full | Full | Full |
| mTLS support | No | Yes (via PKI) | No | No | No |
| HSM support | No | Yes | Yes | Yes | Yes |

### 7.2 Secret Injection Methods

| Method | Pros | Cons | Best For |
|--------|------|------|----------|
| Env vars | Simple, standard | Logged by ps, less secure | Non-sensitive config |
| Volumes | Encrypted at rest | Slower startup | Certificates, keys |
| Vault Agent | Dynamic, automatic | Complex setup | Production secrets |
| ESO | External sync | Sync delay | Cloud secrets |
| SPIFFE | Workload identity | Complex | Service mesh |

---

## 8. Anti-Patterns and Failure Modes

### 8.1 Common Anti-Patterns

**Hardcoded Secrets**
```yaml
# BAD: Hardcoded secrets in deployment
apiVersion: apps/v1
kind: Deployment
metadata:
  name: bad-practice
spec:
  template:
    spec:
      containers:
      - name: app
        env:
        - name: API_KEY
          value: "super-secret-api-key"  # NEVER DO THIS
```

**Secrets in Git**
```yaml
# BAD: Base64-encoded secrets in git
apiVersion: v1
kind: Secret
metadata:
  name: bad-secret
data:
  password: c3VwZXItc2VjcmV0  # Decodes to "super-secret"
```

### 8.2 Failure Modes

**Vault Unavailable**
```
Error: "Error posting to Vault: dial tcp: lookup vault.platform.svc.cluster.local"
Cause: Vault service unavailable or network issue
Solution:
- Use Vault Agent with failover
- Configure Vault high availability
- Implement fallback to cached secrets
```

**Secret Not Synced**
```
Error: "secret is empty but was expected to have data"
Cause: ESO sync hasn't completed
Solution:
- Check ESO pod logs
- Verify ClusterSecretStore is valid
- Use correct secret template
```

---

## 9. Production Checklist

### 9.1 Security Checklist

- [ ] Secrets encrypted at rest (etcd encryption enabled)
- [ ] TLS enabled for all secret communication
- [ ] Vault running in HA mode with minimum 3 nodes
- [ ] Auto-unseal configured with KMS
- [ ] Audit logging enabled for all secret access
- [ ] Least privilege access policies in place
- [ ] Secret rotation configured for all long-lived credentials
- [ ] No hardcoded secrets in code or configuration
- [ ] Secrets scanned from git history
- [ ] SPIFFE/SPIRE workload identity deployed

### 9.2 Operational Checklist

- [ ] Backup and restore procedures documented
- [ ] Disaster recovery plan tested
- [ ] Monitoring and alerting for secret service health
- [ ] Runbook for secret rotation failures
- [ ] Emergency access procedure documented
- [ ] Regular security audits conducted

---

## 10. References

### HashiCorp Vault

- [Vault Documentation](https://developer.hashicorp.com/vault/docs)
- [Vault Kubernetes Deployment Guide](https://developer.hashicorp.com/vault/docs/platform/k8s)
- [Vault Database Secrets Engine](https://developer.hashicorp.com/vault/docs/secrets/databases)
- [Vault Agent Injector](https://developer.hashicorp.com/vault/docs/platform/k8s/injector)

### AWS Secrets Manager

- [AWS Secrets Manager Documentation](https://docs.aws.amazon.com/secretsmanager/)
- [External Secrets Operator](https://external-secrets.io/)
- [AWS Secrets Manager Lambda Rotation](https://docs.aws.amazon.com/secretsmanager/latest/userguide/rotate-secrets.html)

### Kubernetes Secrets

- [Kubernetes Secrets Documentation](https://kubernetes.io/docs/concepts/configuration/secret/)
- [Sealed Secrets for GitOps](https://github.com/bitnami-labs/sealed-secrets)

### SPIFFE/SPIRE

- [SPIFFE Specification](https://github.com/spiffe/spiffe/blob/main/standards/SPIFFE.md)
- [SPIRE Documentation](https://spiffe.io/docs/latest/)
- [SPIFFE Workload API](https://github.com/spiffe/spiffe/blob/main/standards/SPIFFE_Workload_API.md)