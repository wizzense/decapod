# KUBERNETES.md - Kubernetes Orchestration

**Authority:** guidance (comprehensive container orchestration with exact manifests)
**Layer:** Architecture
**Binding:** No
**Scope:** Kubernetes resources, operators, networking, storage, security, and operational patterns with exact specifications for pre-inference context

---

## 1. Core Resource Model

### 1.1 Workload Resources

#### Pod Specification
```yaml
apiVersion: v1
kind: Pod
metadata:
  name: web-server
  namespace: production
  labels:
    app: web-server
    version: v2.1.0
    environment: production
spec:
  restartPolicy: Always  # Always | OnFailure | Never
  terminationGracePeriodSeconds: 30  # graceful shutdown window
  affinity:
    nodeAffinity:
      requiredDuringSchedulingIgnoredDuringExecution:
        nodeSelectorTerms:
        - matchExpressions:
          - key: topology.kubernetes.io/zone
            operator: In
            values:
            - us-east-1a
            - us-east-1b
          - key: node.kubernetes.io/workload-type
            operator: NotIn
            values:
            - batch
      preferredDuringSchedulingIgnoredDuringExecution:
      - weight: 100
        preference:
          matchExpressions:
          - key: storage-node
            operator: In
            values:
            - "true"
    podAffinity:
      preferredDuringSchedulingIgnoredDuringExecution:
      - weight: 50
        podAffinityTerm:
          labelSelector:
            matchLabels:
              app: database
          topologyKey: topology.kubernetes.io/zone
    podAntiAffinity:
      requiredDuringSchedulingIgnoredDuringExecution:
      - labelSelector:
          matchLabels:
            app: web-server
        topologyKey: kubernetes.io/hostname
  tolerations:
  - key: "dedicated"
    operator: "Equal"
    value: "web-server"
    effect: "NoSchedule"
  - key: "gpu"
    operator: "Exists"
    effect: "NoSchedule"
  - key: "node.kubernetes.io/not-ready"
    operator: "Exists"
    effect: "NoExecute"
    tolerationSeconds: 300
  initContainers:
  - name: init-myservice
    image: busybox:1.36
    command:
    - sh
    - -c
    - |
      echo "Waiting for database to be ready..."
      until nslookup mysql.default.svc.cluster.local; do
        echo "DNS not ready, waiting..."
        sleep 5
      done
      echo "Database is ready!"
    resources:
      requests:
        memory: "16Mi"
        cpu: "50m"
      limits:
        memory: "32Mi"
        cpu: "100m"
    securityContext:
      runAsNonRoot: true
      runAsUser: 65534
      runAsGroup: 65534
      fsGroup: 65534
      readOnlyRootFilesystem: true
      capabilities:
        drop:
        - ALL
  containers:
  - name: nginx
    image: nginx:1.25-alpine
    ports:
    - name: http
      containerPort: 80
      protocol: TCP
    - name: https
      containerPort: 443
      protocol: TCP
    - name: metrics
      containerPort: 9090
      protocol: TCP
    env:
    - name: DATABASE_URL
      valueFrom:
        secretKeyRef:
          name: database-credentials
          key: url
    - name: REDIS_HOST
      valueFrom:
        configMapKeyRef:
          name: app-config
          key: redis.host
    - name: POD_IP
      valueFrom:
        fieldRef:
          fieldPath: status.podIP
    - name: NODE_NAME
      valueFrom:
        fieldRef:
          fieldPath: spec.nodeName
    - name: CPU_LIMIT
      valueFrom:
        resourceFieldRef:
          containerName: nginx
          resource: limits.cpu
          divisor: "1m"
    resources:
      requests:
        memory: "128Mi"
        cpu: "250m"
      limits:
        memory: "256Mi"
        cpu: "500m"
    livenessProbe:
      httpGet:
        path: /healthz/live
        port: http
        httpHeaders:
        - name: X-Custom-Header
          value: "liveness"
      initialDelaySeconds: 15
      periodSeconds: 10
      timeoutSeconds: 5
      failureThreshold: 3
      successThreshold: 1
    readinessProbe:
      httpGet:
        path: /healthz/ready
        port: http
      initialDelaySeconds: 5
      periodSeconds: 5
      timeoutSeconds: 3
      failureThreshold: 3
      successThreshold: 1
    startupProbe:
      httpGet:
        path: /healthz
        port: http
      initialDelaySeconds: 0
      periodSeconds: 5
      failureThreshold: 30
      timeoutSeconds: 3
    securityContext:
      allowPrivilegeEscalation: false
      readOnlyRootFilesystem: true
      capabilities:
        drop:
        - ALL
      seccompProfile:
        type: RuntimeDefault
    volumeMounts:
    - name: cache
      mountPath: /tmp
    - name: config
      mountPath: /etc/nginx/conf.d
      readOnly: true
    - name: tls-certs
      mountPath: /etc/nginx/ssl
      readOnly: true
  volumes:
  - name: cache
    emptyDir:
      medium: Memory
      sizeLimit: "256Mi"
  - name: config
    configMap:
      name: nginx-config
      items:
      - key: default.conf
        path: default.conf
      defaultMode: 0444
  - name: tls-certs
    secret:
      secretName: nginx-tls
      optional: true
      defaultMode: 0444
  dnsPolicy: ClusterFirst  # ClusterFirst | ClusterFirstWithHostNet | Default | None
  dnsConfig:
    nameservers:
    - 8.8.8.8
    - 8.8.4.4
    searches:
    - default.svc.cluster.local
    - svc.cluster.local
    options:
    - name: ndots
      value: "2"
    - name: edns0
  hostNetwork: false
  hostPID: false
  hostIPC: false
  imagePullSecrets:
  - name: registry-pull-secret
  nodeSelector:
    kubernetes.io/os: linux
  serviceAccountName: web-server
  automountServiceAccountToken: false
  hostAliases:
  - ip: "10.0.0.1"
    hostnames:
    - "internal-api.example.com"
```

#### Deployment Specification
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: web-server
  namespace: production
  labels:
    app: web-server
spec:
  replicas: 3
  revisionHistoryLimit: 5
  selector:
    matchLabels:
      app: web-server
  strategy:
    type: RollingUpdate  # RollingUpdate | Recreate | RBD (deprecated)
    rollingUpdate:
      maxSurge: 1  # 1 for default, can be percentage like "25%"
      maxUnavailable: 0  # 0 for zero-downtime, "25%" for percentage
  template:
    metadata:
      labels:
        app: web-server
        version: v2.1.0
    spec:
      # (same as Pod spec above)
```

#### StatefulSet Specification
```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: mysql
  namespace: database
spec:
  serviceName: mysql-headless  # Must match a headless Service
  replicas: 3
  podManagementPolicy: OrderedReady  # OrderedReady | Parallel
  updateStrategy:
    type: RollingUpdate  # RollingUpdate | OnDelete
    rollingUpdate:
      maxUnavailable: 1
      # Only for partitions when using maxUnavailable
      # partition: 2  # For canary updates
  persistentVolumeClaimRetentionPolicy:
    whenDeleted: Retain  # Retain | Delete
    whenScaled: Retain  # Retain | Delete
  selector:
    matchLabels:
      app: mysql
  template:
    spec:
      terminationGracePeriodSeconds: 30
      affinity:
        podAntiAffinity:
          requiredDuringSchedulingIgnoredDuringExecution:
          - labelSelector:
              matchLabels:
                app: mysql
            topologyKey: kubernetes.io/hostname
      containers:
      - name: mysql
        image: mysql:8.0
        volumeMounts:
        - name: data
          mountPath: /var/lib/mysql
        - name: config
          mountPath: /etc/mysql/conf.d
        command:
        - bash
        - -c
        - |
          set -e
          # Initialize database if not already done
          if [ ! -d "/var/lib/mysql/mysql" ]; then
            echo "Initializing database..."
            mysql_install_db --user=mysql --datadir=/var/lib/mysql
            echo "Running mysqld..."
          fi
          exec mysqld --user=mysql --datadir=/var/lib/mysql
  volumeClaimTemplates:
  - metadata:
      name: data
    spec:
      accessModes: ["ReadWriteOnce"]
      storageClassName: fast-ssd
      resources:
        requests:
          storage: 100Gi
      selector:
        matchLabels:
          type: ssd
    status:
      phase: Pending
  - metadata:
      name: config
    spec:
      accessModes: ["ReadOnlyMany"]
      storageClassName: standard
      resources:
        requests:
          storage: 1Gi
```

#### DaemonSet Specification
```yaml
apiVersion: apps/v1
kind: DaemonSet
metadata:
  name: node-exporter
  namespace: monitoring
spec:
  selector:
    matchLabels:
      app: node-exporter
  template:
    metadata:
      labels:
        app: node-exporter
    spec:
      tolerations:
      - key: node.kubernetes.io/not-ready
        operator: Exists
        effect: NoSchedule
      - key: node-role.kubernetes.io/control-plane
        operator: Exists
        effect: NoSchedule
      containers:
      - name: node-exporter
        image: prom/node-exporter:v1.6.1
        args:
        - --path.procfs=/host/proc
        - --path.sysfs=/host/sys
        - --path.rootfs=/host
        - --collector.filesystem.mount-points-exclude=^/(dev|proc|sys|var/lib/docker/.+)($|/)
        - --web.listen-address=:9100
        securityContext:
          readOnlyRootFilesystem: true
        volumeMounts:
        - name: proc
          mountPath: /host/proc
          readOnly: true
        - name: sys
          mountPath: /host/sys
          readOnly: true
        - name: root
          mountPath: /host
          readOnly: true
      hostNetwork: true
      hostPID: true
      volumes:
      - name: proc
        hostPath:
          path: /proc
      - name: sys
        hostPath:
          path: /sys
      - name: root
        hostPath:
          path: /
```

### 1.2 Service Resources

#### ClusterIP Service
```yaml
apiVersion: v1
kind: Service
metadata:
  name: web-server-svc
  namespace: production
  labels:
    app: web-server
  annotations:
    prometheus.io/scrape: "true"
    prometheus.io/port: "9090"
spec:
  type: ClusterIP  # ClusterIP | NodePort | LoadBalancer | ExternalName | Headless (ClusterIP: None)
  clusterIP: 10.96.0.100  # Optional: specify fixed IP
  clusterIPs:
  - 10.96.0.100
  ports:
  - name: http
    port: 80
    targetPort: 80
    protocol: TCP
  - name: https
    port: 443
    targetPort: 443
    protocol: TCP
  - name: metrics
    port: 9090
    targetPort: 9090
    protocol: TCP
  selector:
    app: web-server
  publishNotReadyAddresses: false  # Don't include pods not yet ready
  sessionAffinity: None  # None | ClientIP
  sessionAffinityConfig:
    clientIP:
      timeoutSeconds: 10800  # 3 hours for ClientIP affinity
  internalTrafficPolicy: Cluster  # Cluster | Local (Local = only route to pods on same node)
  externalTrafficPolicy: Cluster  # Cluster | Local (preserves client IP when Local)
  healthCheckNodePort: 0  # Specify for externalTrafficPolicy=Local
  loadBalancerClass: ""  # For cloud-specific LB implementation
  externalName: ""  # For ExternalName type
  internalTrafficPolicy: Cluster
```

#### Headless Service (for StatefulSets)
```yaml
apiVersion: v1
kind: Service
metadata:
  name: mysql-headless
  namespace: database
spec:
  type: ClusterIP
  clusterIP: None  # This makes it headless
  ports:
  - name: mysql
    port: 3306
    targetPort: 3306
  selector:
    app: mysql
  # For StatefulSet, SRV records will be created for:
  # mysql-0.mysql-headless.database.svc.cluster.local
  # mysql-1.mysql-headless.database.svc.cluster.local
  # mysql-2.mysql-headless.database.svc.cluster.local
```

#### NodePort Service
```yaml
apiVersion: v1
kind: Service
metadata:
  name: web-server-nodeport
  namespace: production
spec:
  type: NodePort
  ports:
  - name: http
    port: 80
    targetPort: 80
    nodePort: 30080  # Optional: specify fixed port (30000-32767)
    protocol: TCP
  - name: https
    port: 443
    targetPort: 443
    nodePort: 30443
    protocol: TCP
  selector:
    app: web-server
```

#### LoadBalancer Service
```yaml
apiVersion: v1
kind: Service
metadata:
  name: web-server-lb
  namespace: production
  annotations:
    # AWS specific
    service.beta.kubernetes.io/aws-load-balancer-type: "nlb"
    service.beta.kubernetes.io/aws-load-balancer-cross-zone-load-balancing-enabled: "true"
    service.beta.kubernetes.io/aws-load-balancer-backend-protocol: "tcp"
    # GCP specific
    cloud.google.com/load-balancer-type: "Internal"
    # Azure specific
    service.beta.kubernetes.io/azure-load-balancer-internal: "true"
spec:
  type: LoadBalancer
  ports:
  - name: https
    port: 443
    targetPort: 443
    protocol: TCP
  selector:
    app: web-server
  loadBalancerIP: ""  # For static IP allocation
  loadBalancerSourceRanges:
  - 10.0.0.0/8
  - 192.168.1.0/24
  externalTrafficPolicy: Cluster  # Preserve client IP
```

### 1.3 Ingress Resources

#### Ingress Specification (networking.k8s.io/v1)
```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: web-server-ingress
  namespace: production
  labels:
    app: web-server
  annotations:
    # Rewriting
    nginx.ingress.kubernetes.io/rewrite-target: /$2
    # SSL redirect
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    # Rate limiting
    nginx.ingress.kubernetes.io/limit-rps: "100"
    nginx.ingress.kubernetes.io/limit-connections: "50"
    # CORS
    nginx.ingress.kubernetes.io/enable-cors: "true"
    nginx.ingress.kubernetes.io/cors-allow-origin: "https://example.com"
    nginx.ingress.kubernetes.io/cors-allow-methods: "PUT, GET, POST, DELETE, PATCH"
    nginx.ingress.kubernetes.io/cors-allow-headers: "Authorization,Content-Type"
    # Timeouts
    nginx.ingress.kubernetes.io/proxy-connect-timeout: "30"
    nginx.ingress.kubernetes.io/proxy-read-timeout: "60"
    nginx.ingress.kubernetes.io/proxy-send-timeout: "60"
    # Buffer sizes
    nginx.ingress.kubernetes.io/proxy-body-size: "10m"
    # WebSocket
    nginx.ingress.kubernetes.io/use-regex: "true"
    # Custom max body size for file uploads
    nginx.ingress.kubernetes.io/proxy-buffer-size: "8k"
spec:
  ingressClassName: nginx  #.Specify ingress class (required in k8s 1.18+)
  defaultBackend:
    service:
      name: default-backend
      port:
        number: 80
  tls:
  - hosts:
    - web-server.example.com
    - api.example.com
    secretName: web-server-tls
  rules:
  - host: web-server.example.com
    http:
      paths:
      - path: /
        pathType: Prefix  # ImplementationSpecific | Prefix | Exact
        backend:
          service:
            name: web-server
            port:
              number: 80
      - path: /api/v1
        pathType: Prefix
        backend:
          service:
            name: api-gateway
            port:
              number: 8080
      - path: /ws
        pathType: Prefix
        backend:
          service:
            name: websocket-server
            port:
              number: 8081
  - host: api.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: api-gateway
            port:
              number: 8080
```

#### Ingress with mTLS (cert-manager + Istio example)
```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: secure-api-ingress
  namespace: production
  annotations:
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
    cert-manager.io/acme-challenge-type: "http01"
    nginx.ingress.kubernetes.io/auth-tls-verify-client: "on"
    nginx.ingress.kubernetes.io/auth-tls-secret: "production/ca-cert"
    nginx.ingress.kubernetes.io/auth-tls-verify-depth: "2"
    nginx.ingress.kubernetes.io/auth-tls-pass-certificate-to-upstream: "true"
spec:
  ingressClassName: nginx
  tls:
  - hosts:
    - secure-api.example.com
    secretName: secure-api-tls
  rules:
  - host: secure-api.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: api-gateway
            port:
              number: 8443
```

---

## 2. Configuration & Secrets

### 2.1 ConfigMap Patterns

#### ConfigMap with Fine-Grained File Mounts
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: app-config
  namespace: production
data:
  # Simple key-value (each key becomes a file)
  database.conf: |
    [database]
    host=postgres.example.com
    port=5432
    name=production_db
    max_connections=100
    
    [redis]
    host=redis.example.com
    port=6379
    db=0
  
  nginx.conf: |
    server {
      listen 80;
      server_name localhost;
      
      location / {
        root /usr/share/nginx/html;
        index index.html;
      }
      
      location /api {
        proxy_pass http://api-backend:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
      }
    }
  
  feature-flags.json: |
    {
      "new_checkout_flow": true,
      "dark_mode": false,
      "max_items_per_order": 100,
      "experimental_search": true
    }
  
  # Binary data (base64 encoded)
binaryData:
  random-bytes: SGVsbG8gV29ybGQh  # base64 encoded
immutable: false  # Prevent modifications after creation
```

#### ConfigMap Volume Mount with SubPath (Pitfalls)
```yaml
# PROBLEMATIC: Using subPath causes the file to be "orphaned" from configmap updates
# The mounted file will NOT be updated when ConfigMap changes
volumeMounts:
- name: config
  mountPath: /etc/app/config.json
  subPath: config.json  # BAD: Creates a symlink that won't update

# CORRECT: Mount entire directory, or use projected volumes
volumeMounts:
- name: config
  mountPath: /etc/app/config/
  readOnly: true
```

### 2.2 Secret Patterns

#### Generic Secret
```yaml
apiVersion: v1
kind: Secret
metadata:
  name: database-credentials
  namespace: production
type: Opaque  # Opaque | kubernetes.io/tls | kubernetes.io/basic-auth | kubernetes.io/ssh-auth | etc.
stringData:  # Write plain text (will be base64 encoded on create)
  username: db_user
  password: SuperSecretPassword123!
  url: "postgresql://db_user:SuperSecretPassword123!@postgres.example.com:5432/production_db"
data:  # Pre-encoded (base64)
  # echo -n 'password' | base64
  db-password: cGFzc3dvcmQ=
```

#### TLS Secret
```yaml
apiVersion: v1
kind: Secret
metadata:
  name: web-server-tls
  namespace: production
type: kubernetes.io/tls
data:
  # Certificate (base64 encoded)
  tls.crt: LS0tLS1CRUdJTiBDRVJUSUZJQ0FURS0tLS0tCk1JSUJ...
  # Private key (base64 encoded)
  tls.key: LS0tLS1CRUdJTiBQUklWQVRFIEtFWS0tLS0tCk1JSUV...
```

#### ImagePullSecret
```yaml
apiVersion: v1
kind: Secret
metadata:
  name: registry-pull-secret
  namespace: production
type: kubernetes.io/dockerconfigjson
data:
  # echo -n '{"auths":{"ghcr.io":{"auth":"dXNlcjpwYXNz"}}}' | base64
  .dockerconfigjson: eyJhdXRocyI6eyJnaGNyLmlvIjp7ImF1dGgiOiJkWHBzWVc1blgxUnZjbVZ3In19fQ==
```

### 2.3 External Secrets Pattern (External Secrets Operator)

```yaml
apiVersion: external-secrets.io/v1beta1
kind: ExternalSecret
metadata:
  name: database-credentials
  namespace: production
spec:
  refreshInterval: 1h
  secretStoreRef:
    name: vault-backend
    kind: ClusterSecretStore
  target:
    name: database-credentials  # The created secret name
    creationPolicy: Owner  # Owner | Merge | Owner+ES | static
    deletionPolicy: Retain  # Retain | Delete
    template:
      type: Opaque
      data:
        username: "{{ .username }}"
        password: "{{ .password }}"
        url: "postgresql://{{ .username }}:{{ .password }}@{{ .host }}:5432/{{ .dbname }}"
  data:
  - secretKey: username
    remoteRef:
      key: production/database
      property: username
  - secretKey: password
    remoteRef:
      key: production/database
      property: password
  - secretKey: host
    remoteRef:
      key: production/database
      property: host
  - secretKey: dbname
    remoteRef:
      key: production/database
      property: dbname
```

---

## 3. Auto-scaling Resources

### 3.1 Horizontal Pod Autoscaler (HPA)

#### HPA with CPU and Memory
```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: web-server-hpa
  namespace: production
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: web-server
  minReplicas: 3
  maxReplicas: 100
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization  # Utilization | AverageValue | AverageUtilization (v2)
        averageUtilization: 70  # Scale when avg CPU > 70%
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80  # Scale when avg memory > 80%
  behavior:
    scaleDown:
      stabilizationWindowSeconds: 300  # 5 min cooldown before scaling down
      policies:
      - type: Percent
        value: 10
        periodSeconds: 60  # Max 10% pods removed per minute
      - type: Pods
        value: 4
        periodSeconds: 60  # OR max 4 pods removed per minute
      selectPolicy: Min  # Min | Max | Disabled (use most restrictive)
    scaleUp:
      stabilizationWindowSeconds: 0  # No cooldown for scale up
      policies:
      - type: Percent
        value: 100
        periodSeconds: 15  # Can double pods in 15 seconds
      - type: Pods
        value: 10
        periodSeconds: 15
      selectPolicy: Min
```

#### HPA with Custom Metrics (Prometheus)
```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: api-hpa
  namespace: production
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: api-server
  minReplicas: 3
  maxReplicas: 50
  metrics:
  # Standard resource metrics
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 60
  # Custom Prometheus metric
  - type: Pods
    pods:
      metric:
        name: http_requests_per_second
      target:
        type: AverageValue
        averageValue: "1k"  # 1000 RPS per pod
  - type: Pods
    pods:
      metric:
        name: request_latency_p99_seconds
      target:
        type: AverageValue
        averageValue: "100m"  # 100ms average P99
  behavior:
    scaleUp:
      policies:
      - type: Percent
        value: 100
        periodSeconds: 15
```

### 3.2 Vertical Pod Autoscaler (VPA)

```yaml
apiVersion: autoscaling.k8s.io/v1
kind: VerticalPodAutoscaler
metadata:
  name: api-server-vpa
  namespace: production
spec:
  targetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: api-server
  updatePolicy:
    updateMode: "Auto"  # Off | Initial | Recreate | Auto
  minAllowed:
    cpu: 100m
    memory: 128Mi
  maxAllowed:
    cpu: 4
    memory: 16Gi
  resourcePolicy:
    containerPolicies:
    - containerName: api-server
      minAllowed:
        cpu: 200m
        memory: 256Mi
      maxAllowed:
        cpu: 2
        memory: 8Gi
      controlledResources: ["cpu", "memory"]  # What to control
    - containerName: sidecar
      mode: "Off"  # Don't autoscale this container
```

### 3.3 Pod Disruption Budget (PDB)

```yaml
apiVersion: policy/v1
kind: PodDisruptionBudget
metadata:
  name: web-server-pdb
  namespace: production
spec:
  # At least N pods must remain available
  # Use minAvailable OR maxUnavailable, not both
  minAvailable: 2  # At least 2 pods must be available
  # OR
  # maxUnavailable: 1  # No more than 1 pod can be unavailable at a time
  # maxUnavailable: "50%"  # Percentage allowed
  
  # For zero-downtime deployments, use:
  # minAvailable: N where N = replicas - 1 (for single disruption)
  # OR use maxUnavailable: 1 with rolling update strategy
  
  selector:
    matchLabels:
      app: web-server
```

---

## 4. Networking

### 4.1 NetworkPolicy

#### Default Deny All Ingress
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: default-deny-ingress
  namespace: production
spec:
  podSelector: {}  # Selects all pods in namespace
  policyTypes:
  - Ingress  # Explicitly declare intent
```

#### Allow Ingress from Same Namespace
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-same-namespace
  namespace: production
spec:
  podSelector: {}  # All pods
  policyTypes:
  - Ingress
  ingress:
  - from:
    - podSelector: {}  # From pods in same namespace
    ports:
    - protocol: TCP
      port: 80
    - protocol: TCP
      port: 443
```

#### Web Server with Specific Allowed Sources
```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: web-server-netpol
  namespace: production
spec:
  podSelector:
    matchLabels:
      app: web-server
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: ingress-nginx
    - namespaceSelector:
        matchLabels:
          name: monitoring
      podSelector:
        matchLabels:
          app: prometheus
    ports:
    - protocol: TCP
      port: 80
    - protocol: TCP
      port: 443
    - protocol: TCP
      port: 9090
  egress:
  - to:
    - podSelector:
        matchLabels:
          app: api-server
    ports:
    - protocol: TCP
      port: 8080
  - to:
    - podSelector:
        matchLabels:
          app: redis
    ports:
    - protocol: TCP
      port: 6379
  - to:  # DNS is required
    - namespaceSelector: {}  # All namespaces (for DNS)
      podSelector:
        matchLabels:
          k8s-app: kube-dns
    ports:
    - protocol: UDP
      port: 53
  - to:
    - namespaceSelector: {}  # External internet
    ports:
    - protocol: TCP
      port: 443
    - protocol: TCP
      port: 80
```

### 4.2 Service Mesh (Istio) VirtualService

```yaml
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: web-server-vs
  namespace: production
spec:
  hosts:
  - web-server
  - web-server.production.svc.cluster.local
  - "*.example.com"
  gateways:
  - web-server-gateway  # Reference to Gateway resource
  - mesh  # Include for internal mesh routing
  http:
  - name: default-route
    match:
    - uri:
        prefix: /
    route:
    - destination:
        host: web-server
        port:
          number: 80
      weight: 100
  - name: api-v1
    match:
    - uri:
        prefix: /api/v1
    route:
    - destination:
        host: api-server
        port:
          number: 8080
      weight: 90
    - destination:
        host: api-server-canary
        port:
          number: 8080
      weight: 10  # 10% traffic to canary
    retries:
      attempts: 3
      perTryTimeout: 2s
      retryOn: connect-failure,refused-stream,unavailable,cancelled,retriable-status-codes
    timeout: 10s
    fault:
      delay:
        percentage:
          value: 1.0  # 1% of requests
        fixedDelay: 5s
      # OR abort:
      #   percentage:
      #     value: 5.0  # 5% of requests
      #   httpStatus: 503
  - name: websocket-route
    match:
    - uri:
        prefix: /ws
    route:
    - destination:
        host: websocket-server
        port:
          number: 8081
    headers:
      response:
        set:
          X-Custom-Header: "websocket"
  tls:
  - match:
    - port: 443
      sniHosts:
      - secure.example.com
    route:
    - destination:
        host: secure-backend
        port:
          number: 8443
```

### 4.3 Gateway Resource (Istio)

```yaml
apiVersion: networking.istio.io/v1beta1
kind: Gateway
metadata:
  name: web-server-gateway
  namespace: production
spec:
  selector:
    istio: ingressgateway  # Pod labels to select
  servers:
  - port:
      number: 80
      name: http
      protocol: HTTP  # HTTP | HTTPS | HTTPS2 | TCP | TLS
    hosts:
    - "web-server.example.com"
    - "api.example.com"
    # Redirect HTTP to HTTPS
    # redirect:
    #   httpsPort: 443
    #   redirectCode: 301
  - port:
      number: 443
      name: https
      protocol: HTTPS
    hosts:
    - "web-server.example.com"
    tls:
      mode: SIMPLE  # NONE | SIMPLE | MUTUAL | AUTO_PASSTHROUGH
      credentialName: web-server-tls-cert  # Reference to Kubernetes Secret
      # For mutual TLS:
      # mode: MUTUAL
      # privateKey: /etc/certs/tls.key
      # serverCertificate: /etc/certs/tls.crt
      # caCertificates: /etc/certs/ca.crt
  - port:
      number: 9443
      name: grpc
      protocol: GRPC
    hosts:
    - "grpc.example.com"
    tls:
      mode: SIMPLE
      credentialName: grpc-tls-cert
```

---

## 5. Storage

### 5.1 PersistentVolumeClaim

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: data-pvc
  namespace: database
  labels:
    app: mysql
spec:
  accessModes:
  - ReadWriteOnce  # RWO | RWX | ROX | RWOP
  # RWO: Single node read-write
  # RWX: Multiple nodes read-write  
  # ROX: Multiple nodes read-only
  # RWOP: Single pod read-write (CSI only)
  storageClassName: fast-ssd
  resources:
    requests:
      storage: 100Gi
  dataSource:
    apiGroup: snapshot.storage.k8s.io
    kind: VolumeSnapshot
    name: mysql-snapshot-2024-01-15
  selector:
    matchLabels:
      type: ssd
      environment: production
```

### 5.2 StorageClass

```yaml
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: fast-ssd
  annotations:
    storageclass.kubernetes.io/is-default-class: "false"
provisioner: kubernetes.io/gce-pd  # aws-ebs | kubernetes.io/gce-pd | kubernetes.io/azure-disk | etc.
parameters:
  type: pd-ssd  # gp2 | gp3 | io1 | sc1 | st1 (AWS)
  # replication-type: regional-pd (GCP)
  # cachingMode: ReadNone | ReadWrite | ReadWriteSlower (Azure)
volumeBindingMode: WaitForFirstConsumer  # Immediate | WaitForFirstConsumer
# Immediate: Create PV immediately
# WaitForFirstConsumer: Delay until pod is scheduled (allows topology-aware provisioning)
allowVolumeExpansion: true
reclaimPolicy: Retain  # Delete | Retain
mountOptions:
- hard
- noatime
- nobarrier
- defaults
```

### 5.3 CSI Volume Templates (for StatefulSets)

```yaml
# For StatefulSet with CSI driver
volumeClaimTemplates:
- metadata:
    name: data
  spec:
    accessModes:
    - ReadWriteOnce
    storageClassName: csi-hostpath-sc
    resources:
      requests:
        storage: 10Gi
    dataSource:
      apiGroup: snapshot.storage.k8s.io
      kind: VolumeSnapshot
      name: my-snapshot
```

---

## 6. RBAC & Security Contexts

### 6.1 ServiceAccount with ClusterRoleBinding

```yaml
apiVersion: v1
kind: ServiceAccount
metadata:
  name: web-server
  namespace: production
  labels:
    app: web-server
secrets:
- name: web-server-token-xxxxx
imagePullSecrets:
- name: registry-secret
automountToken: false  # Don't mount SA token

---
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  name: web-server-role
  namespace: production
rules:
# Read pods and services in same namespace
- apiGroups: [""]
  resources: ["pods", "services"]
  verbs: ["get", "list", "watch"]
# Read specific configmaps
- apiGroups: [""]
  resources: ["configmaps"]
  resourceNames: ["app-config"]  # Limit to specific resources
  verbs: ["get"]
# Access to pods/logs
- apiGroups: [""]
  resources: ["pods/log"]
  verbs: ["get"]
# Update configmaps (for dynamic config reload)
- apiGroups: [""]
  resources: ["configmaps"]
  verbs: ["update", "patch"]

---
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: web-server-rolebinding
  namespace: production
subjects:
- kind: ServiceAccount
  name: web-server
  namespace: production
roleRef:
  kind: Role
  name: web-server-role
  apiGroup: rbac.authorization.k8s.io

---
# For cluster-wide access, use ClusterRole and ClusterRoleBinding
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: metrics-reader
rules:
- apiGroups: [""]
  resources: ["nodes", "pods"]
  verbs: ["get", "list"]
- apiGroups: ["metrics.k8s.io"]
  resources: ["pods", "nodes"]
  verbs: ["get", "list"]

---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: metrics-reader-binding
subjects:
- kind: ServiceAccount
  name: prometheus
  namespace: monitoring
roleRef:
  kind: ClusterRole
  name: metrics-reader
  apiGroup: rbac.authorization.k8s.io
```

### 6.2 Pod Security Standards (PSS)

```yaml
# Pod security admission label (Kubernetes 1.25+)
# Valid options: privileged | baseline | restricted
apiVersion: v1
kind: Namespace
metadata:
  name: production
  labels:
    # Enforce baseline restrictions
    pod-security.kubernetes.io/enforce: baseline
    pod-security.kubernetes.io/enforce-version: v1.25
    # Audit restricted violations (log but don't block)
    pod-security.kubernetes.io/audit: restricted
    pod-security.kubernetes.io/audit-version: v1.25
    # Warn users about restricted violations
    pod-security.kubernetes.io/warn: restricted
    pod-security.kubernetes.io/warn-version: v1.25
```

### 6.3 Pod Security Context

```yaml
spec:
  securityContext:
    runAsNonRoot: true
    runAsUser: 65534  # nobody
    runAsGroup: 65534
    runAsNonRoot: true
    fsGroup: 65534  # Group for mounted volumes
    suppementalGroups: [65534]
    seccompProfile:
      type: RuntimeDefault  # RuntimeDefault | Unconfined | Custom (filename)
    seLinuxOptions:
      level: "s0:c123,c456"
      role: "object_r"
      type: "svirt_sandbox_file_t"
      user: "system_u"
    windowsOptions:
      gmsaCredentialSpecName: "web-app-gmsa"
      gmsaCredentialSpec: '{"Name":"web-app-gmsa","DNS":"web-app.domain"}'
      hostProcess: false
      runAsUserName: "NT AUTHORITY\\LocalService"
  containers:
  - name: web
    securityContext:
      allowPrivilegeEscalation: false
      capabilities:
        drop:
        - ALL
        add:  # Only add what's strictly necessary
        - NET_BIND_SERVICE
      privileged: false
      readOnlyRootFilesystem: true
      # For writable rootfs with specific safe paths
      # writableRootFilesystem: false  (default)
      procMount: Default  # Default | Unmasked
```

---

## 7. Resource Quotas & Limits

### 7.1 ResourceQuota

```yaml
apiVersion: v1
kind: ResourceQuota
metadata:
  name: production-quota
  namespace: production
spec:
  hard:
    # Compute resources
    requests.cpu: "20"
    requests.memory: 40Gi
    limits.cpu: "40"
    limits.memory: 80Gi
    # Count quota
    persistentvolumeclaims: "10"
    services.loadbalancers: "2"
    services.nodeports: "5"
    pods: "50"
    replicationcontrollers: "10"
    resourcequotas: "1"
    secrets: "20"
    configmaps: "30"
    # Storage
    requests.storage: "500Gi"
    # For GKE/GCP
    # compute.googleapis.com/regional固态硬盘: "100Gi"
  scopeSelector:
    matchExpressions:
    - operator: In
      scopeName: PriorityClass
      values: ["high-priority"]
  - operator: Exists
    scopeName: ScopeName
    values: ["Terminating"]
status:
  hard:
    requests.cpu: "20"
    requests.memory: 40Gi
    pods: "50"
  used:
    requests.cpu: "4"
    requests.memory: 8Gi
    pods: "12"
```

### 7.2 LimitRange

```yaml
apiVersion: v1
kind: LimitRange
metadata:
  name: production-limits
  namespace: production
spec:
  limits:
  # Default limits for containers
  - type: Container
    default:
      cpu: 500m
      memory: 512Mi
    defaultRequest:
      cpu: 100m
      memory: 128Mi
    # Factor to multiply requests by for limits
    # defaultRequest is often set to match guaranteed QoS
    # QoS: requests == limits = Guaranteed
    #       requests > limits = Burstable (or BestEffort if no requests)
    # For guaranteed QoS, both must be set equal
    min:
      cpu: 50m
      memory: 32Mi
    max:
      cpu: "4"
      memory: 16Gi
    maxLimitRequestRatio:
      cpu: "4"  # Limit cannot exceed request by more than 4x
      memory: "4"
  # Default limits for pods
  - type: Pod
    max:
      cpu: "8"
      memory: 32Gi
  # Default limits for PVCs
  - type: PersistentVolumeClaim
    min:
      storage: 1Gi
    max:
      storage: 100Gi
```

---

## 8. Operators & Custom Resources

### 8.1 Custom Resource Definition (CRD)

```yaml
apiVersion: apiextensions.k8s.io/v1
kind: CustomResourceDefinition
metadata:
  name: databases.example.com
  labels:
    app: database-operator
spec:
  group: example.com
  names:
    kind: Database
    plural: databases
    singular: database
    shortNames:
    - db
    categories:
    - all
  scope: Namespaced  # Namespaced | Cluster
  versions:
  - name: v1
    served: true
    storage: true  # Only ONE version should have this true
    schema:
      openAPIV3Schema:
        type: object
        properties:
          spec:
            type: object
            required:
            - engine
            - version
            properties:
              engine:
                type: string
                enum:
                - postgresql
                - mysql
                - mongodb
              version:
                type: string
                pattern: "^[0-9]+\\.[0-9]+$"
              replicas:
                type: integer
                minimum: 1
                maximum: 10
                default: 1
              storage:
                type: object
                properties:
                  size:
                    type: string
                    pattern: "^[0-9]+Gi$"
                  storageClass:
                    type: string
              backupEnabled:
                type: boolean
                default: true
          status:
            type: object
            properties:
              phase:
                type: string
              readyReplicas:
                type: integer
              masterEndpoint:
                type: string
    additionalPrinterColumns:
    - name: Engine
      type: string
      jsonPath: .spec.engine
    - name: Version
      type: string
      jsonPath: .spec.version
    - name: Replicas
      type: integer
      jsonPath: .spec.replicas
    - name: Status
      type: string
      jsonPath: .status.phase
    - name: Age
      type: date
      jsonPath: .metadata.creationTimestamp
  conversion:
    strategy: Webhook  # None | Webhook
    webhook:
      conversionReviewVersions: ["v1", "v1beta1"]
      clientConfig:
        service:
          name: database-operator
          namespace: operators
          path: /convert
        caBundle: LS0tLS1CRUdJTiB...
  preserveUnknownFields: false
```

### 8.2 Implementing the Operator (Controller Pattern)

```go
// Typical operator reconciliation loop structure
package controller

import (
    context "context"
    fmt "fmt"
    metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
    ctrl "sigs.k8s.io/controller-runtime"
   "sigs.k8s.io/controller-runtime/pkg/client"
    examplecomv1 "github.com/example/database-operator/api/v1"
)

type DatabaseReconciler struct {
    client.Client
}

func (r *DatabaseReconciler) Reconcile(ctx context.Context, req ctrl.Request) (ctrl.Result, error) {
    log := ctrl.LoggerFrom(ctx)
    
    // 1. Fetch the custom resource
    db := &examplecomv1.Database{}
    if err := r.Get(ctx, req.NamespacedName, db); err != nil {
        return ctrl.Result{}, client.IgnoreNotFound(err)
    }
    
    // 2. Create or update child resources based on spec
    // Create StatefulSet
    ss := r.statefulSetForDatabase(db)
    if err := r.createOrUpdate(ctx, ss, func() error {
        // Update spec fields that might have changed
        ss.Spec.Replicas = db.Spec.Replicas
        return nil
    }); err != nil {
        return ctrl.Result{}, fmt.Errorf("failed to reconcile StatefulSet: %w", err)
    }
    
    // Create Service
    svc := r.serviceForDatabase(db)
    if err := r.createOrUpdate(ctx, svc, nil); err != nil {
        return ctrl.Result{}, fmt.Errorf("failed to reconcile Service: %w", err)
    }
    
    // 3. Update status
    db.Status.Phase = "Running"
    db.Status.ReadyReplicas = *ss.Spec.Replicas
    if err := r.Status().Update(ctx, db); err != nil {
        return ctrl.Result{}, fmt.Errorf("failed to update status: %w", err)
    }
    
    return ctrl.Result{RequeueAfter: 30 * time.Second}, nil
}
```

---

## 9. Common Patterns & Anti-Patterns

### 9.1 Production Patterns

#### Pattern: Graceful Shutdown with PreStop Hook
```yaml
spec:
  containers:
  - name: nginx
    lifecycle:
      preStop:
        exec:
          command:
          - /bin/sh
          - -c
          - |
            echo "Starting graceful shutdown..."
            # Stop accepting new connections
            nginx -s quit
            # Wait for existing connections (max 65s)
            sleep 60
            # Force exit if still running
            kill -QUIT $PID
      postStart:
        exec:
          command:
          - /bin/sh
          - -c
          - |
            echo "Container started, registering with service discovery..."
            # Register with consul, etcd, etc.
```

#### Pattern: PodDisruptionBudget with Rolling Update
```yaml
# For 3 replicas, this ensures at least 2 pods are always available
spec:
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1        # Can have 4 pods during update
      maxUnavailable: 0  # Never have fewer than desired
---
# PDB ensures at least 2 pods available
spec:
  minAvailable: 2  # Or maxUnavailable: 1
```

#### Pattern: Init Container for Migration/Setup
```yaml
initContainers:
- name: wait-for-db
  image: postgres:15
  command:
  - sh
  - -c
  - |
    until psql -h "$DB_HOST" -U "$DB_USER" -d postgres -c '\q'; do
      echo "Waiting for database..."
      sleep 2
    done
    echo "Database is ready"
- name: run-migrations
  image: myapp:migrations
  env:
  - name: DB_HOST
    valueFrom:
      secretKeyRef:
        name: db-creds
        key: host
  command:
  - sh
  - -c
  - |
    echo "Running database migrations..."
    /app/migrate.sh
    echo "Migrations complete"
```

### 9.2 Anti-Patterns & Pitfalls

#### Anti-Pattern: Not Setting Resource Limits
```yaml
# BAD: No limits means pod can consume unlimited resources
containers:
- name: web
  image: nginx
  resources:
    requests:  # Only requests, no limits
      memory: "128Mi"
      cpu: "100m"
# This causes:
# - Pod scheduled based on requests
# - No throttling/termination when exceeding limits (since none set)
# - Potential resource starvation for other pods
# - BestEffort QoS class (first to be evicted)

# GOOD: Always set both requests AND limits
resources:
  requests:
    memory: "128Mi"
    cpu: "100m"
  limits:
    memory: "256Mi"
    cpu: "200m"
```

#### Anti-Pattern: Using Latest Tag
```yaml
# BAD: Latest tag is mutable, unpredictable
image: nginx:latest
image: myapp:latest

# Issues:
# - Image changes between deployments
# - No reproducibility
# - Cache invalidation issues
# - Security: might pull vulnerable version

# GOOD: Use specific immutable tags
image: nginx:1.25-alpine
image: myapp:v2.1.0@sha256:abc123...
```

#### Anti-Pattern: No Liveness/Readiness Probes
```yaml
# BAD: No probes means kubelet can't determine pod health
containers:
- name: web
  image: nginx
  # No livenessProbe
  # No readinessProbe

# Issues:
# - Kubelet will restart containers arbitrarily
# - Traffic sent to pods that aren't ready
# - No graceful handling of slow startup

# GOOD: Always define appropriate probes
livenessProbe:
  httpGet:
    path: /healthz/live
    port: 8080
  initialDelaySeconds: 15
  periodSeconds: 10
readinessProbe:
  httpGet:
    path: /healthz/ready
    port: 8080
  initialDelaySeconds: 5
  periodSeconds: 5
```

#### Anti-Pattern: HostPath Volume in Production
```yaml
# BAD: HostPath creates pod-node coupling
volumes:
- name: data
  hostPath:
    path: /data
    type: DirectoryOrCreate

# Issues:
# - Pod bound to specific node
# - Data loss if node fails
# - Security: pod can access host filesystem
# - Not portable across cloud providers

# GOOD: Use PersistentVolumeClaim
volumes:
- name: data
  persistentVolumeClaim:
    claimName: data-pvc
```

#### Anti-Pattern: Running as Root
```yaml
# BAD: Running as root is security risk
containers:
- name: web
  image: nginx
  securityContext:
    runAsUser: 0  # Running as root!

# Issues:
# - Container escape gives host access
# - Permission issues with volumes
# - Violates principle of least privilege

# GOOD: Run as non-root
securityContext:
  runAsNonRoot: true
  runAsUser: 1000
  runAsGroup: 1000
  allowPrivilegeEscalation: false
```

---

## 10. Debugging & Diagnostics

### 10.1 Common Commands

```bash
# Get pod status with events
kubectl get pod nginx-7fb96c846b-abc123 -o wide
kubectl describe pod nginx-7fb96c846b-abc123 -n production

# Check logs (all containers in pod)
kubectl logs nginx-7fb96c846b-abc123 --all-containers=true
kubectl logs nginx-7fb96c846b-abc123 --previous  # Previous container instance
kubectl logs nginx-7fb96c846b-abc123 -f --tail=100

# Execute into container
kubectl exec -it nginx-7fb96c846b-abc123 -n production -- /bin/sh

# Port forward for local debugging
kubectl port-forward nginx-7fb96c846b-abc123 8080:80 -n production

# Copy files from container
kubectl cp production/nginx-7fb96c846b-abc123:/var/log/nginx/error.log ./error.log

# Check resource usage
kubectl top pod -n production
kubectl top nodes

# Check HPA status
kubectl get hpa -n production
kubectl describe hpa web-server-hpa -n production

# Check PV/PVC status
kubectl get pv,pvc -n production
kubectl describe pvc data-pvc -n production

# Network debugging
kubectl run tmp-shell --rm -i --tty --image=nicolaka/netshoot -- /bin/bash
# Inside netshoot: dig, nslookup, nc, tcpdump, etc.
```

### 10.2 Common Error Messages

| Error | Cause | Solution |
|-------|-------|----------|
| `ImagePullBackOff` | Can't pull image | Check image name, registry auth, network |
| `CrashLoopBackOff` | Container keeps crashing | Check logs, app startup command |
| `OomKilled` | Memory limit exceeded | Increase memory limit or optimize app |
| `Terminating` | Pod stuck terminating | Force delete or check finalizers |
| `Pending` | Can't schedule pod | Check resources, node selector, taints |
| `ContainerCreating` | Init problem | Check volumes, secrets, configmaps |
| `Evicted` | Node pressure | Reduce resource requests or add nodes |

### 10.3 Network Debugging Checklist

```bash
# 1. Check if DNS resolution works
kubectl exec -it test-pod -- nslookup web-server.production.svc.cluster.local
kubectl exec -it test-pod -- cat /etc/resolv.conf

# 2. Check if service IP is reachable
kubectl exec -it test-pod -- curl -v http://10.96.0.100:80

# 3. Check endpoint slices
kubectl get endpoints web-server -n production

# 4. Check network policies
kubectl get networkpolicy -n production
kubectl describe networkpolicy web-server-netpol -n production

# 5. Check ingress status
kubectl describe ingress web-server-ingress -n production
kubectl get ingressclass

# 6. Check service port configuration
kubectl get svc web-server -n production -o yaml
```

---

## 11. Key Decision Frameworks

### 11.1 When to Use Each Workload Type

| Workload | Use Case | Key Characteristics |
|----------|----------|-------------------|
| **Deployment** | Stateless services | Rolling updates, multiple replicas, no persistent state |
| **StatefulSet** | Databases, queues | Stable network IDs, persistent storage, ordered deployment/scaling |
| **DaemonSet** | Node-level daemons | One pod per node, node selector support, log collectors, monitoring agents |
| **Job** | One-time tasks | Runs to completion, can parallelize, batch processing |
| **CronJob** | Scheduled tasks | Time-based schedules, Job controller |
| **ReplicaSet** | Rarely used directly | Usually managed by Deployment |

### 11.2 Service Type Selection

| Type | Use Case | External Access | Best For |
|------|----------|-----------------|----------|
| **ClusterIP** | Internal only | No | Backend services, databases |
| **NodePort** | Simple external access | Port on every node | Dev, simple deployments |
| **LoadBalancer** | Cloud-managed LB | Cloud LB | Production with cloud integration |
| **ExternalName** | CNAME alias | DNS only | External service mapping |
| **Headless** | StatefulSet discovery | No | DNS-based service discovery |

### 11.3 Storage Selection Matrix

| Need | Recommended | Considerations |
|------|-------------|---------------|
| Block storage | CSI (aws-ebs, gce-pd, azuredisk) | Single attach only |
| Shared storage | NFS, CephFS, Azure Files | Multiple read-write |
| Ephemeral fast storage | emptyDir with memory medium | Lost on pod restart, RAM disk |
| Database storage | Block CSI with ReadWriteOnce | Performance critical |
| File storage | Shared CSI (NFS, CephFS) | Shared access needed |

### 11.4 Scaling Decision Tree

```
Start with HPA (Horizontal Pod Autoscaler)
    │
    ├── CPU/Memory based scaling
    │       └── Simple, always start here
    │
    └── Custom metrics based scaling
            │
            ├── Prometheus metrics
            │       └── Use KEDA or custom metrics API
            │
            ├── Request rate based
            │       └── nginx-ingress or service mesh metrics
            │
            └── Queue depth based
                    └── Apache Kafka lag, RabbitMQ depth, AWS SQS
```

---

## Links

- `architecture/CLOUD.md` - Cloud-specific Kubernetes (EKS, GKE, AKS)
- `architecture/CONTAINERS.md` - Container runtime and OCI
- `architecture/OBSERVABILITY.md` - Kubernetes monitoring and logging
- `architecture/CACHING.md` - Caching strategies for K8s
- `core/ENGINEERING_EXCELLENCE.md` - Engineering standards

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2024-01-15 | Initial comprehensive Kubernetes reference |