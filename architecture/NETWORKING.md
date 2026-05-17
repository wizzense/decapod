# NETWORKING.md - Network Architecture for Kubernetes and Microservices

**Authority:** guidance (comprehensive topic with exact specifications)
**Layer:** Architecture
**Binding:** No
**Scope:** Comprehensive topic coverage for pre-inference context

## Table of Contents

1. [DNS Patterns](#1-dns-patterns)
2. [Load Balancing Algorithms](#2-load-balancing-algorithms)
3. [Service Discovery](#3-service-discovery)
4. [Ingress Controllers](#4-ingress-controllers)
5. [Network Policies](#5-network-policies)
6. [Service Mesh Networking](#6-service-mesh-networking)
7. [Complete YAML Manifests](#7-complete-yaml-manifests)
8. [Decision Matrices](#8-decision-matrices)
9. [Troubleshooting Guide](#9-troubleshooting-guide)
10. [References](#10-references)

---

## 1. DNS Patterns

### 1.1 Kubernetes DNS Configuration

CoreDNS is the DNS server for Kubernetes clusters. CoreDNS replaces kube-dns as the default DNS provider.

```yaml
# CoreDNS ConfigMap for custom DNS configuration
apiVersion: v1
kind: ConfigMap
metadata:
  name: coredns-custom
  namespace: kube-system
  labels:
    k8s-app: kube-dns
data:
  # Custom Corefile extensions
  # These overrides take precedence over the default Corefile
  custom.server: |
    # Cache middleware
    cache 30 {
      success 8254
      denial 2184
    }
    
    # Forward external domains to upstream DNS
    forward . /etc/resolv.conf {
      policy round_robin
    }
    
    # Log configuration
    log {
      class error
    }
    
    # Errors logging
    errors

  # Rewrite rules for service discovery
  rewrite name order-service.platform.svc.cluster.local order-service.platform.svc.cluster.local
    
  # Health check endpoint
  health: |
    lameduck 5s

---
# Corefile with full configuration
apiVersion: v1
kind: ConfigMap
metadata:
  name: coredns
  namespace: kube-system
data:
  Corefile: |
    .:53 {
      errors
      health {
         lameduck 5s
      }
      ready
      
      kubernetes cluster.local in-addr.arpa ip6.arpa {
        pods verified
        fallthrough in-addr.arpa ip6.arpa
        ttl 30
      }
      
      prometheus :9153
      
      forward . /etc/resolv.conf {
        policy round_robin
        max_concurrent 1000
      }
      
      cache 30
      
      reload
      loadbalance
    }
```

### 1.2 External DNS Configuration

```yaml
# ExternalDNS for automatic DNS record management
apiVersion: apps/v1
kind: Deployment
metadata:
  name: external-dns
  namespace: platform
  labels:
    app: external-dns
spec:
  strategy:
    type: Recreate
  selector:
    matchLabels:
      app: external-dns
  template:
    metadata:
      labels:
        app: external-dns
    spec:
      serviceAccountName: external-dns
      containers:
      - name: external-dns
        image: registry.k8s.io/external-dns/external-dns:v0.13.5
        args:
        - --source=service
        - --source=ingress
        - --source=awsloudancer-target-group
        - --domain-filter=example.com
        - --zone-id-filter=Z1234567890ABC
        - --provider=aws
        - --aws-zone-type=public
        - --aws-assume-role=external-dns
        - --policy=upsert-only
        - --registry=txt
        - --txt-owner-id=external-dns
        - --txt-prefix=external-dns-
        - --interval=1m
        - --log-level=info
        - --events
        - --metrics
        resources:
          requests:
            cpu: 100m
            memory: 128Mi
          limits:
            cpu: 500m
            memory: 256Mi
        securityContext:
          readOnlyRootFilesystem: true
          runAsUser: 1000
          fsGroup: 1000
      volumes:
      - name: aws-credentials
        secret:
          secretName: external-dns-aws-credentials
```

### 1.3 Headless Service DNS

Headless services return endpoints directly for pod discovery.

```yaml
# Headless service for stateful service discovery
apiVersion: v1
kind: Service
metadata:
  name: kafka-headless
  namespace: platform
  labels:
    app: kafka
spec:
  clusterIP: None  # Makes this headless
  publishNotReadyAddresses: false
  ports:
  - name: kafka
    port: 9092
    targetPort: 9092
  - name: internal
    port: 9093
    targetPort: 9093
  selector:
    app: kafka
    tier: messaging

# This creates DNS records like:
# kafka-0.kafka-headless.platform.svc.cluster.local -> pod IP
# kafka-1.kafka-headless.platform.svc.cluster.local -> pod IP
```

---

## 2. Load Balancing Algorithms

### 2.1 Load Balancing Types

| Algorithm | Description | Use Case | Trade-offs |
|-----------|-------------|----------|------------|
| Round Robin | Sequential distribution | Simple stateless services | No consideration for load |
| Weighted Round Robin | Weighted sequential | Servers with different capacities | Static weights |
| Least Connections | Routes to fewest active connections | Long-lived connections | Memory overhead |
| Weighted Least Connections | Weighted by capacity | Heterogeneous server capacity | Complex tuning |
| IP Hash | Hash of client IP | Session affinity | Uneven distribution |
| Random | Random selection | Simple, works well with many nodes | No consistency |
| Consistent Hash | Hash ring distribution | Cache lookup, distributed caching | Rebalancing complexity |

### 2.2 Nginx Load Balancing Configuration

```yaml
# Nginx upstream with multiple algorithms
# This would be in a ConfigMap for Nginx Ingress Controller
upstream order-backend {
    # Least connections algorithm
    least_conn;
    
    # Server configuration
    server order-service-0.order-service.platform.svc.cluster.local:8080 weight=5 max_fails=3 fail_timeout=30s;
    server order-service-1.order-service.platform.svc.cluster.local:8080 weight=5 max_fails=3 fail_timeout=30s;
    server order-service-2.order-service.platform.svc.cluster.local:8080 weight=5 max_fails=3 fail_timeout=30s;
    
    # Keepalive for connection pooling
    keepalive 32;
    keepalive_timeout 60s;
    keepalive_requests 1000;
}

upstream payment-backend {
    # IP hash for session affinity
    ip_hash;
    
    server payment-service-0.payment-service.platform.svc.cluster.local:8080 max_fails=2 fail_timeout=10s;
    server payment-service-1.payment-service.platform.svc.cluster.local:8080 max_fails=2 fail_timeout=10s;
    server payment-service-2.payment-service.platform.svc.cluster.local:8080 max_fails=2 fail_timeout=10s backup;
}

upstream websocket-backend {
    # Hash based on $connection for WebSocket affinity
    hash $remote_addr consistent;
    
    server ws-service-0.ws-service.platform.svc.cluster.local:8080;
    server ws-service-1.ws-service.platform.svc.cluster.local:8080;
    server ws-service-2.ws-service.platform.svc.cluster.local:8080;
}

upstream cache-backend {
    # Random with two random choices, then pick better one
    random two least_time=last_byte;
    
    server redis-0.redis.platform.svc.cluster.local:6379;
    server redis-1.redis.platform.svc.cluster.local:6379;
    server redis-2.redis.platform.svc.cluster.local:6379;
}
```

### 2.3 Kubernetes Service Load Balancing

```yaml
# Service with session affinity configuration
apiVersion: v1
kind: Service
metadata:
  name: order-service
  namespace: platform
  labels:
    app: order-service
spec:
  type: ClusterIP
  sessionAffinity: ClientIP
  sessionAffinityConfig:
    clientIP:
      timeoutSeconds: 10800  # 3 hours
  ports:
  - name: http
    port: 80
    targetPort: 8080
    protocol: TCP
  - name: grpc
    port: 9091
    targetPort: 9090
    protocol: TCP
  selector:
    app: order-service
  externalTrafficPolicy: Cluster
  # Options: Cluster (default) or Local
  # Local preserves client source IP but requires pod scheduling
  
# For external traffic policy Local
apiVersion: v1
kind: Service
metadata:
  name: order-service-external
  namespace: platform
spec:
  type: LoadBalancer
  externalTrafficPolicy: Local
  healthCheckNodePort: 32456
  ports:
  - name: http
    port: 80
    targetPort: 8080
    protocol: TCP
  selector:
    app: order-service
```

---

## 3. Service Discovery

### 3.1 Consul Service Discovery

```yaml
# Consul service registration
apiVersion: v1
kind: Service
metadata:
  name: order-service
  namespace: platform
  labels:
    app: order-service
  annotations:
    consul.hashicorp.com/service-name: order-service
    consul.hashicorp.com/service-port: "8080"
    consul.hashicorp.com/service-meta-environment: production
    consul.hashicorp.com/service-tags: "v1.2.3,backend,http"
    consul.hashicorp.com/health-check-id: order-service-health
spec:
  type: ClusterIP
  ports:
  - name: http
    port: 80
    targetPort: 8080
  selector:
    app: order-service

---
# Consul Intentions (network policies)
apiVersion: consul.hashicorp.com/v1alpha1
kind: ServiceIntentions
metadata:
  name: order-to-inventory
  namespace: platform
spec:
  destination:
    name: inventory-service
  sources:
  - name: order-service
    action: allow
    
---
# Consul config entry for service resolver (canary routing)
apiVersion: consul.hashicorp.com/v1alpha1
kind: ServiceResolver
metadata:
  name: order-service
  namespace: platform
spec:
  defaultSubset: v1
  subsets:
    v1:
      filter: Service.Meta.version == v1
    v2:
      filter: Service.Meta.version == v2
  redirect:
    service: order-service
```

### 3.2 Kubernetes Native Service Discovery

```yaml
# EndpointSlice for service discovery
apiVersion: discovery.k8s.io/v1
kind: EndpointSlice
metadata:
  name: order-service-example
  namespace: platform
  labels:
    kubernetes.io/service-name: order-service
    endpointslice.kubernetes.io/managed-by: endpointslice-controller
addressType: IPv4
ports:
  - name: http
    port: 8080
    protocol: TCP
  - name: grpc
    port: 9090
    protocol: TCP
endpoints:
  - addresses:
      - "10.1.2.3"
    conditions:
      ready: true
      serving: true
      terminating: false
    hostname: order-service-abc123
    nodeName: node-1
    targetRef:
      kind: Pod
      name: order-service-abc123
      namespace: platform
      uid: 12345678-1234-1234-1234-123456789012
    topology:
      kubernetes.io/hostname: node-1
      topology.kubernetes.io/zone: us-east-1a
  - addresses:
      - "10.1.2.4"
    conditions:
      ready: true
      serving: true
      terminating: false
    hostname: order-service-def456
    nodeName: node-2
    targetRef:
      kind: Pod
      name: order-service-def456
      namespace: platform
      uid: 12345678-1234-1234-1234-123456789013
    topology:
      kubernetes.io/hostname: node-2
      topology.kubernetes.io/zone: us-east-1b
```

---

## 4. Ingress Controllers

### 4.1 Nginx Ingress Controller

```yaml
# Nginx Ingress Controller installation
apiVersion: v1
kind: Namespace
metadata:
  name: ingress-nginx
  labels:
    app.kubernetes.io/name: ingress-nginx
    app.kubernetes.io/instance: ingress-nginx
---
apiVersion: v1
kind: ServiceAccount
metadata:
  name: ingress-nginx
  namespace: ingress-nginx
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: ingress-nginx
rules:
  - apiGroups: [""]
    resources: ["configmaps", "endpoints", "nodes", "pods", "secrets", "namespaces"]
    verbs: ["list", "watch"]
  - apiGroups: [""]
    resources: ["nodes"]
    verbs: ["get"]
  - apiGroups: [""]
    resources: ["services"]
    verbs: ["get", "list", "watch"]
  - apiGroups: ["networking.k8s.io"]
    resources: ["ingresses", "ingressclasses"]
    verbs: ["get", "list", "watch"]
  - apiGroups: [""]
    resources: ["configmaps", "events"]
    verbs: ["create", "patch"]
  - apiGroups: ["coordination.k8s.io"]
    resources: ["leases"]
    verbs: ["get", "create", "update"]
  - apiGroups: ["discovery.k8s.io"]
    resources: ["endpointslices"]
    verbs: ["list", "watch", "get"]
---
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRoleBinding
metadata:
  name: ingress-nginx
roleRef:
  apiGroup: rbac.authorization.k8s.io
  kind: ClusterRole
  name: ingress-nginx
subjects:
  - kind: ServiceAccount
    name: ingress-nginx
    namespace: ingress-nginx
---
apiVersion: v1
kind: ConfigMap
metadata:
  name: ingress-nginx-controller
  namespace: ingress-nginx
  labels:
    app.kubernetes.io/name: ingress-nginx
    app.kubernetes.io/component: controller
data:
  allow-snippet-annotations: "true"
  use-forwarded-headers: "true"
  compute-full-forwarded-for: "true"
  use-proxy-protocol: "false"
  enable-underscores-in-headers: "true"
  large-client-header-buffers: "4 16k"
  client-header-buffer-size: "4k"
  keep-alive: "75"
  keep-alive-requests: "1000"
  upstream-keepalive-connections: "1000"
  upstream-keepalive-timeout: "60s"
  upstream-keepalive-requests: "10000"
  proxy-connect-timeout: "10s"
  proxy-send-timeout: "60s"
  proxy-read-timeout: "60s"
  proxy-buffering: "on"
  proxy-buffer-size: "16k"
  proxy-buffers: "4 16k"
  proxy-max-temp-file-size: "1024m"
  ssl-protocols: "TLSv1.2 TLSv1.3"
  ssl-ciphers: "ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256"
  ssl-prefer-server-ciphers: "false"
  use-http2: "true"
  gzip-level: "5"
  gzip-types: "application/json application/xml text/plain text/css application/javascript"
  log-format-upstream: '$remote_addr - $remote_user [$time_local] "$request" $status $body_bytes_sent "$http_referer" "$http_user_agent" $request_length $request_time [$proxy_upstream_name] [$proxy_alternative_upstream_name] $upstream_addr $upstream_response_length $upstream_response_time $upstream_rtt $upstream_status $latency'
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: ingress-nginx-controller
  namespace: ingress-nginx
spec:
  replicas: 3
  selector:
    matchLabels:
      app.kubernetes.io/name: ingress-nginx
      app.kubernetes.io/component: controller
  template:
    metadata:
      labels:
        app.kubernetes.io/name: ingress-nginx
        app.kubernetes.io/component: controller
    spec:
      serviceAccountName: ingress-nginx
      terminationGracePeriodSeconds: 300
      containers:
      - name: controller
        image: registry.k8s.io/ingress-nginx/controller:v1.9.4
        args:
          - /nginx-ingress-controller
          - --publish-service=$(POD_NAMESPACE)/ingress-nginx-controller
          - --election-id=ingress-controller-leader
          - --controller-class=k8s.io/ingress-nginx
          - --ingress-class=nginx
          - --configmap=$(POD_NAMESPACE)/ingress-nginx-controller
          - --watch-ingress-without-class=true
        securityContext:
          capabilities:
            drop:
              - ALL
            add:
              - NET_BIND_SERVICE
          runAsUser: 101
          allowPrivilegeEscalation: true
        env:
          - name: POD_NAME
            valueFrom:
              fieldRef:
                fieldPath: metadata.name
          - name: POD_NAMESPACE
            valueFrom:
              fieldRef:
                fieldPath: metadata.namespace
          - name: LD_PRELOAD
            value: /usr/local/lib/libmimalloc.so
        ports:
          - name: http
            containerPort: 80
            protocol: TCP
          - name: https
            containerPort: 443
            protocol: TCP
          - name: metrics
            containerPort: 10254
            protocol: TCP
          - name: webhook
            containerPort: 8443
            protocol: TCP
        livenessProbe:
          httpGet:
            path: /healthz
            port: 10254
            scheme: HTTP
          initialDelaySeconds: 10
          periodSeconds: 10
          timeoutSeconds: 1
          successThreshold: 1
          failureThreshold: 5
        readinessProbe:
          httpGet:
            path: /healthz
            port: 10254
            scheme: HTTP
          initialDelaySeconds: 10
          periodSeconds: 10
          timeoutSeconds: 1
          successThreshold: 1
          failureThreshold: 3
        resources:
          requests:
            cpu: 100m
            memory: 90Mi
          limits:
            cpu: 1000m
            memory: 1Gi
        volumeMounts:
        - name: webhook-cert
          mountPath: /usr/local/certificates
          readOnly: true
      volumes:
      - name: webhook-cert
        secret:
          secretName: ingress-nginx-admission
---
apiVersion: v1
kind: Service
metadata:
  name: ingress-nginx-controller
  namespace: ingress-nginx
  labels:
    app.kubernetes.io/name: ingress-nginx
    app.kubernetes.io/component: controller
spec:
  type: LoadBalancer
  externalTrafficPolicy: Local
  ports:
    - name: http
      port: 80
      targetPort: http
      protocol: TCP
    - name: https
      port: 443
      targetPort: https
      protocol: TCP
  selector:
    app.kubernetes.io/name: ingress-nginx
    app.kubernetes.io/component: controller
```

### 4.2 Complete Ingress Resource

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: order-service-ingress
  namespace: platform
  labels:
    app: order-service
  annotations:
    # SSL/TLS Configuration
    cert-manager.io/cluster-issuer: letsencrypt-prod
    acme.cert-manager.io/http01-ingress-class: nginx
    
    # Rate Limiting
    nginx.ingress.kubernetes.io/limit-rps: "100"
    nginx.ingress.kubernetes.io/limit-rpm: "1000"
    nginx.ingress.kubernetes.io/limit-connections: "50"
    nginx.ingress.kubernetes.io/limit-burst-multiplier: "2"
    nginx.ingress.kubernetes.io/limit-rate: "0"
    nginx.ingress.kubernetes.io/limit-rate-after: "0"
    
    # Proxy Configuration
    nginx.ingress.kubernetes.io/proxy-body-size: "10m"
    nginx.ingress.kubernetes.io/proxy-buffer-size: "16k"
    nginx.ingress.kubernetes.io/proxy-connect-timeout: "10"
    nginx.ingress.kubernetes.io/proxy-send-timeout: "60"
    nginx.ingress.kubernetes.io/proxy-read-timeout: "60"
    nginx.ingress.kubernetes.io/proxy-next-upstream: "error timeout http_502 http_503 http_504"
    nginx.ingress.kubernetes.io/proxy-next-upstream-tries: "3"
    
    # CORS Configuration
    nginx.ingress.kubernetes.io/enable-cors: "true"
    nginx.ingress.kubernetes.io/cors-allow-origin: "https://example.com"
    nginx.ingress.kubernetes.io/cors-allow-methods: "GET PUT POST DELETE PATCH OPTIONS"
    nginx.ingress.kubernetes.io/cors-allow-headers: "Authorization,Content-Type,Accept,Origin,User-Agent,Cache-Control,Keep-Alive,X-Requested-With"
    nginx.ingress.kubernetes.io/cors-expose-headers: "X-Request-ID"
    nginx.ingress.kubernetes.io/cors-max-age: "86400"
    
    # Session Affinity
    nginx.ingress.kubernetes.io/affinity: "cookie"
    nginx.ingress.kubernetes.io/session-cookie-name: "route"
    nginx.ingress.kubernetes.io/session-cookie-expires: "172800"
    nginx.ingress.kubernetes.io/session-cookie-max-age: "172800"
    nginx.ingress.kubernetes.io/session-cookie-change-on-failure: "true"
    
    # Custom headers
    nginx.ingress.kubernetes.io/add-headers: "X-Frame-Options:SAMEORIGIN,X-Content-Type-Options:nosniff,X-XSS-Protection:1; mode=block,Strict-Transport-Security:max-age=31536000; includeSubDomains"
    
    # Canary/Routing
    nginx.ingress.kubernetes.io/canary: "false"
    
    # Rewrite
    nginx.ingress.kubernetes.io/rewrite-target: /
    nginx.ingress.kubernetes.io/use-regex: "true"
    
    # WebSocket
    nginx.ingress.kubernetes.io/proxy-http-version: "1.1"
    nginx.ingress.kubernetes.io/upstream-hash-by: "$remote_addr"
    
    # Logging
    nginx.ingress.kubernetes.io/log-format-upstream: '{"time":"$time_iso8601","remote_addr":"$remote_addr","x-forwarded-for":"$proxy_add_x_forwarded_for","request_id":"$req_id","geoip_country":"$geoip_country_code","remote_user":"$remote_user","body_bytes_sent":"$body_bytes_sent","request_time":"$request_time","status":"$status","request_uri":"$request_uri","request_method":"$request_method","host":"$host","upstream_addr":"$upstream_addr","upstream_status":"$upstream_status","upstream_response_length":"$upstream_response_length","upstream_response_time":"$upstream_response_time","upstream_connect_time":"$upstream_connect_time"}'
    
    # Health check
    nginx.ingress.kubernetes.io/server-snippet: |
      location /health {
        access_log off;
        return 200 "healthy\n";
        add_header Content-Type text/plain;
      }
spec:
  ingressClassName: nginx
  tls:
  - hosts:
      - orders.example.com
    secretName: orders-tls-secret
  rules:
  - host: orders.example.com
    http:
      paths:
      # API v1
      - path: /v1/orders
        pathType: Prefix
        backend:
          service:
            name: order-service
            port:
              number: 8080
      # WebSocket endpoint
      - path: /ws
        pathType: Prefix
        backend:
          service:
            name: order-service-ws
            port:
              number: 8080
      # Health check
      - path: /health
        pathType: Exact
        backend:
          service:
            name: order-service
            port:
              number: 8081
      # Metrics
      - path: /metrics
        pathType: Prefix
        backend:
          service:
            name: order-service
            port:
              number: 9090
```

---

## 5. Network Policies

### 5.1 Default Deny All

```yaml
# NetworkPolicy: Default deny all ingress and egress
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: default-deny-all
  namespace: platform
spec:
  podSelector: {}
  policyTypes:
    - Ingress
    - Egress
---
# NetworkPolicy: Default allow DNS
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-dns
  namespace: platform
spec:
  podSelector: {}
  policyTypes:
    - Egress
  egress:
    # Allow DNS resolution
    - to:
        - namespaceSelector:
            matchLabels:
              kubernetes.io/metadata.name: kube-system
      ports:
        - protocol: UDP
          port: 53
        - protocol: TCP
          port: 53
    # Allow NTP for time synchronization
    - to:
        - ipBlock:
            cidr: 0.0.0.0/0
            except:
              - 10.0.0.0/8
              - 172.16.0.0/12
              - 192.168.0.0/16
      ports:
        - protocol: UDP
          port: 123
```

### 5.2 Application Network Policies

```yaml
# Frontend to API communication
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: frontend-to-api
  namespace: platform
spec:
  podSelector:
    matchLabels:
      app: frontend
  policyTypes:
    - Egress
  egress:
    - to:
        - podSelector:
            matchLabels:
              app: order-service
      ports:
        - protocol: TCP
          port: 8080
    - to:
        - podSelector:
            matchLabels:
              app: inventory-service
      ports:
        - protocol: TCP
          port: 8080
    - to:
        - podSelector:
            matchLabels:
              app: payment-service
      ports:
        - protocol: TCP
          port: 8080

---
# API to Database communication
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: api-to-database
  namespace: platform
spec:
  podSelector:
    matchLabels:
      tier: database
  policyTypes:
    - Ingress
  ingress:
    - from:
        - namespaceSelector:
            matchLabels:
              kubernetes.io/metadata.name: platform
          podSelector:
            matchLabels:
              tier: application
      ports:
        - protocol: TCP
          port: 5432
    - from:
        - namespaceSelector:
            matchLabels:
              kubernetes.io/metadata.name: platform
          podSelector:
            matchLabels:
              app: backup-agent
      ports:
        - protocol: TCP
          port: 5432

---
# API to Message Queue
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: api-to-messaging
  namespace: platform
spec:
  podSelector:
    matchLabels:
      app: kafka
  policyTypes:
    - Ingress
    - Egress
  ingress:
    - from:
        - namespaceSelector:
            matchLabels:
              kubernetes.io/metadata.name: platform
          podSelector:
            matchLabels:
              tier: application
      ports:
        - protocol: TCP
          port: 9092
        - protocol: TCP
          port: 9093
  egress:
    # Allow connecting to other Kafka brokers
    - to:
        - podSelector:
            matchLabels:
              app: kafka
      ports:
        - protocol: TCP
          port: 9092
```

### 5.3 CNI-Specific Network Policy Implementation

```yaml
# Calico NetworkPolicy (uses NetworkPolicy API)
apiVersion: projectcalico.org/v3
kind: NetworkPolicy
metadata:
  name: frontend-to-api-calico
  namespace: platform
spec:
  order: 100
  selector: app == 'frontend'
  types:
    - Egress
  egress:
    - action: Allow
      destination:
        selector: app == 'order-service'
        ports:
          - 8080
    - action: Allow
      destination:
        selector: app == 'inventory-service'
        ports:
          - 8080
    - action: Allow
      destination:
        namespaceSelector: kubernetes.io/metadata.name == 'kube-system'
        ports:
          - 53

---
# Cilio NetworkPolicy
apiVersion: cilium.io/v2
kind: CiliumNetworkPolicy
metadata:
  name: frontend-to-api-cilium
  namespace: platform
spec:
  endpointSelector:
    matchLabels:
      app: frontend
  egress:
    - toPorts:
        - ports:
            - port: "8080"
              protocol: TCP
      toEndpoints:
        - matchLabels:
            app: order-service
    - toFQDNs:
        - matchPattern: "*.cluster.local"
      toPorts:
        - ports:
            - port: "53"
              protocol: UDP
```

---

## 6. Service Mesh Networking

### 6.1 Istio Service Mesh Configuration

```yaml
# Istio Authorization Policy
apiVersion: security.istio.io/v1beta1
kind: AuthorizationPolicy
metadata:
  name: order-service-authz
  namespace: platform
spec:
  selector:
    matchLabels:
      app: order-service
  action: ALLOW
  rules:
    # Allow ingress gateway
    - from:
        - source:
            principals: ["cluster.local/ns/istio-ingress/sa/istio-ingressgateway"]
      to:
        - operation:
            ports: ["8080", "9090"]
    # Allow own namespace
    - from:
        - source:
            namespaces: ["platform"]
      to:
        - operation:
            ports: ["8080"]
    # Allow monitoring
    - from:
        - source:
            namespaces: ["monitoring"]
      to:
        - operation:
            ports: ["9090"]
    # Deny all else
    - to:
        - operation:
            ports: ["8080", "9090"]
---
# Istio PeerAuthentication (mTLS mode)
apiVersion: security.istio.io/v1beta1
kind: PeerAuthentication
metadata:
  name: default-mutual-tls
  namespace: platform
spec:
  mtls:
    mode: STRICT

---
# Istio RequestAuthentication (JWT validation)
apiVersion: security.istio.io/v1beta1
kind: RequestAuthentication
metadata:
  name: order-service-jwt
  namespace: platform
spec:
  selector:
    matchLabels:
      app: order-service
  jwtRules:
    - issuer: "https://auth.example.com"
      audiences:
        - "order-service"
      forwardOriginalToken: true
      preserveExistingClaimsOnError: true
      fromHeaders:
        - name: Authorization
          prefix: "Bearer "
      jwksUri: https://auth.example.com/.well-known/jwks.json
      claimToHeaders:
        - claim: sub
          header: X-User-ID
        - claim: email
          header: X-User-Email
```

---

## 7. Complete YAML Manifests

### 7.1 MetalLB Configuration

```yaml
# MetalLB IPAddressPool
apiVersion: metallb.io/v1beta1
kind: IPAddressPool
metadata:
  name: production-pool
  namespace: metallb-system
spec:
  addresses:
    - 10.0.100.1-10.0.100.50  # Reserved IPs for LoadBalancer
    - 192.168.1.100-192.168.1.150
  autoAssign: true
  avoidBuggyIPs: true
  serviceAllocation:
    namespaceSelectors:
      - matchLabels:
          app: production
    podSelectors:
      - matchLabels:
          tier: frontend
---
# L2Advertisement for ARP
apiVersion: metallb.io/v1beta1
kind: L2Advertisement
metadata:
  name: production-l2
  namespace: metallb-system
spec:
  ipAddressPools:
    - production-pool
  interfaces:
    - eth0
  nodeSelectors:
    - matchLabels:
        node-role.kubernetes.io/worker: ""
  # For VRRP (keepalived), specify VIPs
  vrrpIPs:
    - 10.0.100.1
```

### 7.2 AWS Load Balancer Controller

```yaml
# AWS Load Balancer Controller ServiceAccount with IRSA
apiVersion: v1
kind: ServiceAccount
metadata:
  name: aws-load-balancer-controller
  namespace: kube-system
  annotations:
    eks.amazonaws.com/role-arn: arn:aws:iam::123456789012:role/aws-load-balancer-controller-role
---
# AWS Load Balancer Controller Deployment
apiVersion: apps/v1
kind: Deployment
metadata:
  name: aws-load-balancer-controller
  namespace: kube-system
spec:
  replicas: 2
  selector:
    matchLabels:
      app: aws-load-balancer-controller
  template:
    metadata:
      labels:
        app: aws-load-balancer-controller
    spec:
      serviceAccountName: aws-load-balancer-controller
      containers:
        - name: controller
          image: amazon/aws-load-balancer-controller:v2.6.0
          args:
            - --cluster-name=production
            - --ingress-class-rule-default=alb
            - --controller-name=k8s.io/aws-alb-ingress-controller
            - --aws-vpc-id=vpc-0123456789abcdef0
            - --aws-region=us-east-1
            - --feature-gates=WS=true
            - --feature-gates=ListenerRulesTagging=true
          ports:
            - name: controller
              containerPort: 9443
              protocol: TCP
            - name: metrics
              containerPort: 8080
              protocol: TCP
          env:
            - name: AWS_REGION
              value: us-east-1
            - name: AWS_STS_REGIONAL_ENDPOINTS
              value: regional
          livenessProbe:
            httpGet:
              path: /healthz
              port: 9443
            initialDelaySeconds: 10
            periodSeconds: 10
          readinessProbe:
            httpGet:
              path: /readyz
              port: 9443
            initialDelaySeconds: 10
            periodSeconds: 10
          resources:
            requests:
              cpu: 100m
              memory: 256Mi
            limits:
              cpu: 500m
              memory: 512Mi
          securityContext:
            readOnlyRootFilesystem: true
            capabilities:
              drop:
                - ALL
          volumeMounts:
            - name: cert
              mountPath: /tmp/cert
              readOnly: true
      volumes:
        - name: cert
          emptyDir: {}
---
# IngressClass for ALB
apiVersion: networking.k8s.io/v1
kind: IngressClass
metadata:
  name: alb
  labels:
    app.kubernetes.io/name: aws-load-balancer-controller
spec:
  controller: ingress.k8s.aws/alb
  parameters:
    apiGroup: elbv2.k8s.aws
    kind: IngressClassParams
    name: alb
---
# IngressClassParams
apiVersion: elbv2.k8s.aws/v1beta1
kind: IngressClassParams
metadata:
  name: alb
  labels:
    app.kubernetes.io/name: aws-load-balancer-controller
spec:
  group:
    name: application
  scheme: internet-facing
  ipAddressType: ipv4
  tags:
    Project: decapod
    Environment: production
  loadBalancerAttributes:
    - key: deletion_protection.enabled
      value: "true"
    - key: access_logs.s3.enabled
      value: "true"
    - key: access_logs.s3.bucket
      value: "alb-access-logs"
    - key: access_logs.s3.prefix
      value: "production"
```

---

## 8. Decision Matrices

### 8.1 Load Balancer Selection

| Requirement | NGINX Ingress | AWS ALB | GCE Ingress | Azure AGW |
|-------------|--------------|---------|-------------|-----------|
| Kubernetes native | Yes | Yes | Yes | Yes |
| gRPC routing | Limited | Yes | Yes | Yes |
| WebSocket support | Yes | Yes | Yes | Yes |
| Multi-tenant | Limited | Yes | Yes | Yes |
| Cost | Low (infra) | Medium | Medium | Medium |
| SSL termination | Yes | Yes | Yes | Yes |
| mTLS | Yes | No | No | Yes |
| WAF integration | Limited | Yes | Yes | Yes |
| Access logs | Yes | Yes | Yes | Yes |
| Custom headers | Yes | Limited | Limited | Limited |

### 8.2 Service Discovery Selection

| Requirement | Kubernetes DNS | Consul | etcd | Eureka |
|-------------|---------------|--------|------|--------|
| Setup complexity | None | Medium | High | Medium |
| Service health checks | Basic | Advanced | None | Advanced |
| Multi-cluster | Limited | Yes | Yes | No |
| DNS support | Yes | Yes | Limited | No |
| Configuration sync | No | Yes | Yes | Yes |
| Service mesh integration | Limited | Yes | Limited | No |

### 8.3 Network Policy Engine Selection

| Feature | Calico | Cilium | Weave | kube-router |
|---------|--------|--------|-------|-------------|
| Policy enforcement | Yes | Yes | Yes | Yes |
| eBPF-based | No | Yes | No | No |
| IPv6 support | Yes | Yes | Yes | Yes |
| Multi-cluster | Yes | Yes | Limited | No |
| Network visualization | Yes | Limited | Yes | No |
| Performance | Good | Excellent | Good | Good |
| BGP support | Yes | Yes | Yes | Yes |

---

## 9. Troubleshooting Guide

### 9.1 Common DNS Issues

```bash
# Check CoreDNS logs
kubectl logs -n kube-system -l k8s-app=kube-dns -c coredns

# Debug DNS resolution from a pod
kubectl exec -it test-pod -- nslookup kubernetes.default
kubectl exec -it test-pod -- nslookup order-service.platform.svc.cluster.local

# Check DNS resolution with dig
kubectl exec -it test-pod -- dig +short order-service.platform.svc.cluster.local

# Test connectivity
kubectl exec -it test-pod -- curl -v http://order-service.platform.svc.cluster.local

# Check EndpointSlices
kubectl get endpoints -n platform
kubectl get endpointslice -n platform -l kubernetes.io/service-name=order-service
```

### 9.2 Common Ingress Issues

```bash
# Check ingress controller logs
kubectl logs -n ingress-nginx -l app.kubernetes.io/name=ingress-nginx

# Check ingress status
kubectl describe ingress order-service-ingress -n platform

# Check certificate status
kubectl get certificate -n platform
kubectl describe certificate orders-tls-secret -n platform

# Test locally
curl -v -H "Host: orders.example.com" https://<ingress-ip>/health
```

### 9.3 Network Policy Debugging

```bash
# Check applied policies
kubectl get networkpolicy -n platform
kubectl describe networkpolicy default-deny-all -n platform

# Verify policy is applied (requires network policy aware CNI)
kubectl exec -it test-pod -- nc -zv destination-service 8080

# Check CNI status
kubectl logs -n kube-system -l k8s-app=cilium-agent
```

---

## 10. References

### Load Balancing

- [NGINX Ingress Controller Documentation](https://kubernetes.github.io/ingress-nginx/)
- [AWS Load Balancer Controller](https://kubernetes-sigs.github.io/aws-load-balancer-controller/)
- [MetalLB Documentation](https://metallb.universe.tf/)
- [HAProxy Ingress](https://haproxy-ingress.github.io/)

### Service Discovery

- [Kubernetes DNS Documentation](https://kubernetes.io/docs/concepts/services-networking/dns-pod-service/)
- [CoreDNS Documentation](https://coredns.io/docs/)
- [Consul Service Mesh](https://www.consul.io/docs/k8s)

### Network Policies

- [Kubernetes Network Policies](https://kubernetes.io/docs/concepts/services-networking/network-policies/)
- [Calico Documentation](https://docs.tigera.io/calico/)
- [Cilium Documentation](https://docs.cilium.io/)
- [Network Policy Recipes](https://github.com/ahmetb/kubernetes-network-policy-recipes)

### Service Mesh

- [Istio Documentation](https://istio.io/latest/docs/)
- [Linkerd Documentation](https://linkerd.io/2.14/overview/)
- [Ambassador Documentation](https://www.getambassador.io/docs/)

### Performance

- [HTTP/2 Performance](https://www.http2 explained.org/)
- [gRPC Performance](https://grpc.io/docs/guides/performance/)
- [WebSocket Performance](https://www.websocket.org/performance.html)