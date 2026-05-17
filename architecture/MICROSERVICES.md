# MICROSERVICES.md - Service Decomposition and Inter-Service Communication Architecture

**Authority:** guidance (comprehensive topic with exact specifications)
**Layer:** Architecture
**Binding:** No
**Scope:** Comprehensive topic coverage for pre-inference context

## Table of Contents

1. [Service Decomposition Strategies](#1-service-decomposition-strategies)
2. [Bounded Contexts and Domain Boundaries](#2-bounded-contexts-and-domain-boundaries)
3. [Inter-Service Communication Patterns](#3-inter-service-communication-patterns)
4. [Service Mesh Patterns](#4-service-mesh-patterns)
5. [Resilience Patterns](#5-resilience-patterns)
6. [Service Definition YAML Specifications](#6-service-definition-yaml-specifications)
7. [Decision Matrix](#7-decision-matrix)
8. [Anti-Patterns and Failure Modes](#8-anti-patterns-and-failure-modes)
9. [Production Checklist](#9-production-checklist)
10. [References](#10-references)

---

## 1. Service Decomposition Strategies

### 1.1 Decomposition by Business Capability

Service decomposition follows the principle of finding natural boundaries in the business domain. The key metrics for successful decomposition are:

- **Independent Deployability**: Each service can be deployed without coordinating with other teams
- **Technology Heterogeneity**: Services can use different programming languages, frameworks, or databases
- **Scalability**: Services can scale independently based on their specific load patterns
- **Team Boundaries**: Services align with team ownership and responsibility

### 1.2 Domain-Driven Design Bounded Contexts

Bounded contexts are the primary unit of decomposition in microservices architecture. Each bounded context encapsulates:

- A distinct domain model
- A ubiquitous language specific to that context
- An explicit boundary around the model
- A dedicated team ownership

### 1.3 Decomposition Anti-Patterns

**God Service Anti-Pattern**
A service that encompasses too many responsibilities. This creates:
- Deployment coupling (entire service must be deployed for any change)
- Team contention (multiple teams fighting for the same service)
- Scaling inefficiency (the entire service scales even if only one feature is stressed)
- Failure blast radius (failure in one feature affects all features)

**Shared Database Anti-Pattern**
Multiple services directly sharing the same database schema. Problems include:
- Implicit coupling through schema changes
- No service can evolve independently
- Data ownership is unclear
- Transactions spanning service boundaries become necessary

**Chatty Service Anti-Pattern**
Services that require many sequential calls to complete a single operation. This causes:
- High latency due to network round-trips
- Tight temporal coupling between services
- Increased failure probability (more network calls = more failure points)
- Resource consumption from maintaining many connections

### 1.4 Decomposition Metrics

Use these metrics to evaluate decomposition quality:

| Metric | Formula | Target Range |
|--------|---------|--------------|
| Service Coupling Index (SCI) | (Direct dependencies × API changes) / Autonomous changes | < 0.3 |
| Change Failure Rate | Failed deployments / Total deployments | < 0.15 |
| Deploy Frequency | Number of deployments per day per service | > 1 |
| Lead Time for Changes | Time from commit to production | < 7 days |
| Memory Size per Service | Megabytes of memory allocated | 256MB - 4GB |

---

## 2. Bounded Contexts and Domain Boundaries

### 2.1 Context Mapping Patterns

**Partnership Relationship**
Two contexts collaborate on a specific relationship. Changes require coordination but each context maintains its autonomy.

**Customer-Supplier Relationship**
One context (supplier) provides APIs that another context (customer) consumes. Customer needs are prioritized in supplier's roadmap.

**Conformist Relationship**
One context adopts the model of another context without transformation. Used when integration cost must be minimized.

**Anticorruption Layer**
A translation layer that isolates one context from the model of another. Essential when integrating with legacy systems.

**Open Host Service**
A service defined as a published protocol that any external context can use. Changes must be backward compatible.

**Published Language**
A shared language (schema, API contract) that multiple contexts use for communication.

### 2.2 Boundary Identification heuristics

Strong candidates for service boundaries:
- Different rate of change (one domain evolves faster than others)
- Different team ownership (different squads own different parts)
- Different security requirements (PCI, HIPAA, SOC2 compliance boundaries)
- Different scaling requirements (some features are read-heavy, others write-heavy)
- Different availability requirements (critical path vs background processing)

### 2.3 Subdomain Classification

| Subdomain Type | Characteristics | Decomposition Guidance |
|----------------|-----------------|------------------------|
| Core Domain | Unique business value, competitive advantage | Highest investment, most stable APIs |
| Supporting Domain | Required for core domain, not differentiating | Standard investment, stable interfaces |
| Generic Domain | Commodity functionality (billing, notifications) | Consider off-the-shelf solutions or shared libraries |

---

## 3. Inter-Service Communication Patterns

### 3.1 Synchronous Communication Patterns

#### REST/gRPC

**REST Characteristics**
- Resource-oriented model
- JSON or XML payload format
- HTTP 1.1/2.0 transport
- Idempotent operations where applicable
- Cacheable responses

**gRPC Characteristics**
- Contract-first API design with Protobuf
- Binary serialization (smaller payloads, faster parsing)
- HTTP/2 transport (multiplexing, header compression)
- Bi-directional streaming support
- Strong typing with code generation

**When to Use REST vs gRPC**

| Scenario | Recommended Protocol |
|----------|----------------------|
| External-facing APIs (browsers, mobile) | REST with JSON |
| Internal service-to-service with strict latency requirements | gRPC |
| Streaming (bidirectional) | gRPC |
| When debugging is critical (human-readable payloads) | REST with JSON |
| Polyglot environment with many languages | gRPC (better multi-language support) |
| Existing REST infrastructure | REST |

#### Request-Response Pattern

```yaml
# OpenAPI 3.0 specification for REST endpoint
openapi: 3.0.3
info:
  title: Order Service API
  version: 1.0.0
  description: |
    Order management service API for the e-commerce platform.
    This API follows REST conventions and uses JSON for request/response bodies.
servers:
  - url: https://api.example.com/v1
    description: Production server
  - url: https://staging-api.example.com/v1
    description: Staging server
paths:
  /orders:
    get:
      operationId: listOrders
      summary: List orders with pagination
      description: |
        Returns a paginated list of orders. Supports filtering by status,
        date range, and customer ID. Results are sorted by creation date
        descending by default.
      tags:
        - Orders
      parameters:
        - name: page
          in: query
          description: Page number (1-indexed)
          required: false
          schema:
            type: integer
            minimum: 1
            default: 1
            example: 1
        - name: page_size
          in: query
          description: Number of items per page
          required: false
          schema:
            type: integer
            minimum: 1
            maximum: 100
            default: 20
            example: 20
        - name: status
          in: query
          description: Filter by order status
          required: false
          schema:
            type: string
            enum: [pending, confirmed, processing, shipped, delivered, cancelled]
        - name: customer_id
          in: query
          description: Filter by customer ID (UUID format)
          required: false
          schema:
            type: string
            format: uuid
        - name: created_after
          in: query
          description: Filter orders created after this timestamp (ISO 8601)
          required: false
          schema:
            type: string
            format: date-time
        - name: created_before
          in: query
          description: Filter orders created before this timestamp (ISO 8601)
          required: false
          schema:
            type: string
            format: date-time
      responses:
        '200':
          description: Successful response with paginated order list
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/OrderListResponse'
              example:
                data:
                  - id: "550e8400-e29b-41d4-a716-446655440000"
                    customer_id: "123e4567-e89b-12d3-a456-426614174000"
                    status: "confirmed"
                    total_amount: 159.99
                    currency: "USD"
                    items_count: 3
                    created_at: "2026-01-15T10:30:00Z"
                    updated_at: "2026-01-15T10:35:00Z"
                pagination:
                  page: 1
                  page_size: 20
                  total_items: 1523
                  total_pages: 77
        '400':
          description: Invalid request parameters
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ErrorResponse'
        '401':
          description: Authentication required
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ErrorResponse'
        '429':
          description: Rate limit exceeded
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ErrorResponse'
        '500':
          description: Internal server error
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ErrorResponse'
    post:
      operationId: createOrder
      summary: Create a new order
      description: |
        Creates a new order with the specified items. This is an idempotent
        operation - multiple requests with the same idempotency_key will return
        the same order without creating duplicates.
      tags:
        - Orders
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/CreateOrderRequest'
            example:
              customer_id: "123e4567-e89b-12d3-a456-426614174000"
              idempotency_key: "order-create-2026-01-15-abc123"
              items:
                - product_id: "prod_12345"
                  quantity: 2
                  unit_price: 49.99
                - product_id: "prod_67890"
                  quantity: 1
                  unit_price: 60.01
              shipping_address:
                street: "123 Main Street"
                city: "San Francisco"
                state: "CA"
                postal_code: "94102"
                country: "US"
      responses:
        '201':
          description: Order created successfully
          headers:
            Location:
              description: URL of the newly created order
              schema:
                type: string
                format: uri
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/OrderResponse'
        '400':
          description: Invalid order data
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ErrorResponse'
        '409':
          description: Conflict - order with idempotency key already exists
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/OrderResponse'
  /orders/{order_id}:
    get:
      operationId: getOrder
      summary: Get order by ID
      description: |
        Retrieves the complete order details including all line items,
        shipping information, and payment status.
      tags:
        - Orders
      parameters:
        - name: order_id
          in: path
          required: true
          description: Order UUID
          schema:
            type: string
            format: uuid
            example: "550e8400-e29b-41d4-a716-446655440000"
      responses:
        '200':
          description: Order found
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/OrderResponse'
        '404':
          description: Order not found
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ErrorResponse'
    patch:
      operationId: updateOrder
      summary: Update order status
      description: |
        Updates specific fields of an order. Only certain status transitions
        are allowed. This operation is partial - only provided fields are updated.
      tags:
        - Orders
      parameters:
        - name: order_id
          in: path
          required: true
          description: Order UUID
          schema:
            type: string
            format: uuid
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/UpdateOrderRequest'
      responses:
        '200':
          description: Order updated successfully
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/OrderResponse'
        '400':
          description: Invalid update request
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ErrorResponse'
        '409':
          description: Invalid status transition
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ErrorResponse'
```

### 3.2 Asynchronous Communication Patterns

#### Message Queue Patterns

**Point-to-Point (P2P)**
- One producer, one consumer
- Message is processed exactly once
- Use case: task processing, order fulfillment

**Pub/Sub (Publish-Subscribe)**
- One producer, multiple consumers
- Each consumer receives a copy of the message
- Use case: notifications, event broadcasting

#### Event-Driven Architecture

Events are the core primitive of event-driven systems:

```yaml
# Kafka topic configuration for order events
apiVersion: kafka.apache.org/v1alpha1
kind: KafkaTopic
metadata:
  name: orders.order-events
  namespace: platform
  labels:
    app: order-service
    domain: e-commerce
spec:
  topicName: orders.order-events
  partitions: 48
  replicationFactor: 3
  configs:
    retention.ms: "604800000"  # 7 days
    retention.bytes: "-1"      # unlimited
    cleanup.policy: "delete"
    min.insync.replicas: "2"
    unclean.leader.election.enable: "false"
    segment.ms: "3600000"       # 1 hour segment rotation
    max.message.bytes: "1048576"  # 1MB max message size
---
# Kafka topic configuration for inventory events
apiVersion: kafka.apache.org/v1alpha1
kind: KafkaTopic
metadata:
  name: inventory.stock-events
  namespace: platform
  labels:
    app: inventory-service
    domain: e-commerce
spec:
  topicName: inventory.stock-events
  partitions: 64
  replicationFactor: 3
  configs:
    retention.ms: "2592000000"  # 30 days for inventory
    retention.bytes: "-1"
    cleanup.policy: "delete"
    min.insync.replicas: "2"
```

#### Message Schema Design

```json
{
  "schema": {
    "type": "record",
    "name": "OrderCreatedEvent",
    "namespace": "com.example.orders.events",
    "doc": "Event emitted when a new order is successfully created in the system",
    "version": "1",
    "fields": [
      {
        "name": "event_id",
        "type": {
          "type": "string",
          "logicalType": "uuid"
        },
        "doc": "Globally unique identifier for this event instance"
      },
      {
        "name": "event_type",
        "type": "string",
        "doc": "The type of event that occurred"
      },
      {
        "name": "event_version",
        "type": "string",
        "doc": "Schema version for this event type"
      },
      {
        "name": "occurred_at",
        "type": {
          "type": "long",
          "logicalType": "timestamp-millis"
        },
        "doc": "Unix timestamp in milliseconds when the event occurred"
      },
      {
        "name": "correlation_id",
        "type": {
          "type": "string",
          "logicalType": "uuid"
        },
        "doc": "ID for correlating related events across services"
      },
      {
        "name": "causation_id",
        "type": {
          "type": "string",
          "logicalType": "uuid"
        },
        "doc": "ID of the command or event that caused this event"
      },
      {
        "name": "payload",
        "type": {
          "type": "record",
          "name": "OrderPayload",
          "fields": [
            {
              "name": "order_id",
              "type": {
                "type": "string",
                "logicalType": "uuid"
              }
            },
            {
              "name": "customer_id",
              "type": {
                "type": "string",
                "logicalType": "uuid"
              }
            },
            {
              "name": "order_number",
              "type": "string"
            },
            {
              "name": "status",
              "type": "string",
              "enum": ["pending", "confirmed", "processing", "shipped", "delivered", "cancelled"]
            },
            {
              "name": "total_amount",
              "type": {
                "type": "bytes",
                "logicalType": "decimal",
                "precision": 12,
                "scale": 2
              }
            },
            {
              "name": "currency",
              "type": "string",
              "logicalType": "iso-4217-currency-code"
            },
            {
              "name": "items",
              "type": {
                "type": "array",
                "items": {
                  "type": "record",
                  "name": "OrderLineItem",
                  "fields": [
                    {"name": "line_item_id", "type": "string"},
                    {"name": "product_id", "type": "string"},
                    {"name": "product_name", "type": "string"},
                    {"name": "quantity", "type": "int"},
                    {"name": "unit_price", "type": {"type": "bytes", "logicalType": "decimal", "precision": 10, "scale": 2}}
                  ]
                }
              }
            },
            {
              "name": "shipping_address",
              "type": {
                "type": "record",
                "name": "ShippingAddress",
                "fields": [
                  {"name": "street", "type": "string"},
                  {"name": "city", "type": "string"},
                  {"name": "state", "type": "string"},
                  {"name": "postal_code", "type": "string"},
                  {"name": "country", "type": "string"}
                ]
              }
            }
          ]
        }
      }
    ]
  }
}
```

---

## 4. Service Mesh Patterns

### 4.1 Service Mesh Architecture

A service mesh provides a dedicated infrastructure layer for handling service-to-service communication. The data plane handles actual traffic, while the control plane manages configuration and policy.

**Data Plane Components**
- Sidecar proxies (Envoy, HAProxy)
- Local traffic interception
- Encryption (mTLS)
- Observability (metrics, traces, logs)
- Load balancing
- Circuit breaking

**Control Plane Components**
- Service discovery
- Configuration management
- Certificate management
- Policy enforcement
- Identity management

### 4.2 Istio Service Mesh Configuration

```yaml
# Istio Control Plane configuration (istiod)
apiVersion: install.istio.io/v1alpha1
kind: IstioOperator
metadata:
  name: istio-control-plane
  namespace: istio-system
spec:
  profile: default
  version: 1.20.0
  meshConfig:
    enableAutoMtls: true
    defaultConfig:
      proxyMetadata:
        ISTIO_META_DNS_CAPTURE: "true"
        ISTIO_META_DNS_AUTO_ALLOCATE: "true"
      tracing:
        sampling: 10.0
        zipkin:
          address: jaeger-collector.observability:9411
      binaryPollingInterval: 10s
      drainDuration: 45s
      parentShutdownDuration: 60s
      readinessFailureThreshold: 5
      readinessInitialDelaySeconds: 5
      readinessPeriodSeconds: 5
    localityLbSetting:
      enabled: true
      failover:
        - from: region/us-east
          to: region/us-west
        - from: region/eu-west
          to: region/eu-central
    extensionProviders:
    - name: prometheus
      prometheus:
        metricsPath: /metrics
    - name: jaeger
      jaeger:
        service: jaeger-collector.observability
        port: 9411
  values:
    global:
      imagePullPolicy: IfNotPresent
      istioNamespace: istio-system
      meshID: production-mesh
      multiCluster:
        clusterName: us-east-1
      network: main-network
    pilot:
      autoscaleEnabled: true
      autoscaleMin: 2
      autoscaleMax: 5
      configMap: true
      env:
        PILOT_ENABLE_CONFIG_SOURCE_PRIORITY: "true"
        PILOT_SEND_XDS_TIMEOUT: "10s"
        PILOT_MAX_FIELD_INSTANCES: 200000
      resources:
        requests:
          cpu: 500m
          memory: 2048Mi
        limits:
          cpu: 2000m
          memory: 4Gi
    istiod:
      enableAnalysis: true
    gateway:
      autoscaleEnabled: true
---
# Gateway configuration for ingress
apiVersion: networking.istio.io/v1beta1
kind: Gateway
metadata:
  name: public-gateway
  namespace: istio-ingress
spec:
  selector:
    istio: ingressgateway
  servers:
  - port:
      number: 80
      name: http
      protocol: HTTP
    tls:
      httpsRedirect: true
    hosts:
    - "*.example.com"
  - port:
      number: 443
      name: https
      protocol: HTTPS
    tls:
      mode: SIMPLE
      credentialName: example-com-tls-cert
      minProtocolVersion: TLSV1_2
      cipherSuites:
      - TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256
      - TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384
      - TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256
      - TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384
    hosts:
    - "*.example.com"
---
# VirtualService for routing
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: order-service-route
  namespace: platform
spec:
  hosts:
  - order-service.platform.svc.cluster.local
  - order-service.example.com
  http:
  - name: api-routes
    match:
    - uri:
        prefix: /v1/orders
      headers:
        x-api-version:
          exact: "1"
    route:
    - destination:
        host: order-service.platform.svc.cluster.local
        port:
          number: 8080
      weight: 100
    retries:
      attempts: 3
      perTryTimeout: 10s
      retryOn: connect-failure,refused-stream,unavailable,cancelled,retriable-status-codes
      retryRemoteLocalities: true
    timeout: 30s
    corsPolicy:
      allowOrigins:
      - origin: "https://www.example.com"
      - origin: "https://app.example.com"
      allowMethods:
      - GET
      - POST
      - PUT
      - PATCH
      - DELETE
      - OPTIONS
      allowHeaders:
      - Authorization
      - Content-Type
      - X-Request-ID
      - X-Correlation-ID
      - X-Idempotency-Key
      exposeHeaders:
      - X-Request-ID
      maxAge: 86400s
  - name: health-routes
    match:
    - uri:
        prefix: /health
    - uri:
        prefix: /ready
    route:
    - destination:
        host: order-service.platform.svc.cluster.local
        port:
          number: 8080
    retries:
      attempts: 0
---
# DestinationRule for connection pooling and circuit breaking
apiVersion: networking.istio.io/v1beta1
kind: DestinationRule
metadata:
  name: order-service-destination
  namespace: platform
spec:
  host: order-service.platform.svc.cluster.local
  trafficPolicy:
    connectionPool:
      tcp:
        maxConnections: 1000
        connectTimeout: 10s
      http:
        h2UpgradePolicy: UPGRADE
        http1MaxPendingRequests: 1000
        http2MaxRequests: 1000
        maxRequestsPerConnection: 10000
        maxRetries: 10
    loadBalancer:
      simple: LEAST_CONN
      localityLbSetting:
        enabled: true
        distribute:
        - from: region/us-east-1/*
          to:
            "region/us-east-1/*": 100
    outlierDetection:
      consecutive5xxErrors: 5
      interval: 30s
      baseEjectionTime: 60s
      maxEjectionPercent: 50
      minHealthPercent: 30
    tls:
      mode: ISTIO_MUTUAL
      clientCertificate: /etc/istio/auth/default/tls.crt
      privateKey: /etc/istio/auth/default/tls.key
      caCertificates: /etc/istio/auth/default/ca.crt
      subjectAltNames:
      - order-service.platform.svc.cluster.local
      - order-service
```

### 4.3 Linkerd Service Mesh Configuration

```yaml
# Linkerd installation configuration
apiVersion: linkerd.io/v1alpha1
kind: LinkerD
metadata:
  name: linkerd-config
  namespace: linkerd
spec:
  addons:
    grafana:
      enabled: true
    jaeger:
      enabled: true
      collector:
        url: http://jaeger-collector.observability.svc.cluster.local:14268
    prometheus:
      enabled: true
  controlPlaneVersion: 2.14.0
  flags:
  - name: cluster-domain
    value: cluster.local
  - name: identity-trust-anchors-file
    value: /var/run/linkerd/io.root-ca.crt
  - name: identity-trust-domain
    value: cluster.local
  - name: enable-h2-upgrade
    value: true
  - name: enable-ipv6
    value: false
  profileValidator:
    enabled: true
  proxy:
    accessLog: ""
    await: true
    capabilities: null
    defaultInboundPolicy: ""
    defaultOutboundPolicy: ""
    disableExternalProfileAnnotation: false
    enableDebugSidecar: false
    enableEndpointSlices: true
    enableH2Upgrade: true
    enablePrometheusMetrics: true
    enableRepresentation: false
    enableSecurityContexts: true
    enableSpeakingEngine: true
    image:
      name: ghcr.io/linkerd/proxy
      pullPolicy: IfNotPresent
      version: 2.14.0
    logFormat: plain
    logLevel: warn,linkerd=info
    memory:
      limit: 250Mi
      request: 20Mi
    mountPath: /var/run/linkerd
    ocniAddress: ""
    outboundConnectTimeout: 1000ms
    podInboundPorts: ""
    ports:
      admin: 4191
      control: 4190
      inbound: 4143
      outbound: 4140
    proxyCompatibilityDate: 2024-01-22
    readinessProbe:
      initialDelaySeconds: 10
      maxDelaySeconds: 15
    requireIdentityOnInboundPorts: ""
    resource:
      cpu:
        limit: ""
        request: 100m
      memory:
        limit: ""
        request: 20Mi
    runAsRoot: false
    seccompProfile:
      type: RuntimeDefault
    timeout:
      connect: 1000ms
      request: 10000ms
      minRequestSeconds: 3
    uid: 2102
  proxyInjector:
    await: true
    defaultInboundPolicy: null
    enabled: true
    objectSelector:
      matchExpressions: null
      matchLabels: null
    tls:
      provided: null
      trusted: null
  publicAPI:
    gatewayPort: 443
    proxyPort: 4143
    tap:
      port: 8089
    webPort: 8084
  version: stable-2.14.0
---
# ServiceProfile for per-route metrics and retries
apiVersion: linkerd.io/v1alpha1
kind: ServiceProfile
metadata:
  name: order-service.platform.svc.cluster.local
  namespace: platform
spec:
  routes:
  - condition:
      requestHeaders:
        :method:
          exact: GET
        :path:
          regex: "^/v1/orders.*"
    responseClasses:
    - condition:
        status:
          min: 200
          max: 299
      isFailureClass: false
    - condition:
        status:
          min: 500
          max: 599
      isFailureClass: true
    timeout:
      duration: 30s
  - condition:
      requestHeaders:
        :method:
          exact: POST
        :path:
          exact: "/v1/orders"
    responseClasses:
    - condition:
        status:
          min: 200
          max: 299
      isFailureClass: false
    retry:
      budget:
        minRetriesPerSecond: 10
        percent: 20
        retryPercent: 50
      isRetryable:
        all1xx: true
        GET: true
        POST: true
        PUT: true
        DELETE: true
        PATCH: true
        statusCodes:
        - 429
        - 503
        - 504
      timeout:
        duration: 60s
```

---

## 5. Resilience Patterns

### 5.1 Circuit Breaker Pattern

The circuit breaker prevents cascading failures by failing fast when a downstream service is unhealthy.

**States:**
- **CLOSED**: Normal operation, requests pass through
- **OPEN**: Downstream is failing, requests fail immediately
- **HALF-OPEN**: Testing if downstream has recovered

```yaml
# Circuit breaker configuration for resilient client
apiVersion: v1
kind: ConfigMap
metadata:
  name: resilience-config
  namespace: platform
data:
  circuit-breaker.yml: |
    circuit_breakers:
      order-service:
        enabled: true
        initial_state: closed
        failure_threshold:
          consecutive_failures: 5
          failure_ratio: 0.5
        success_threshold:
          consecutive_successes: 3
        open_state:
          duration: 30s
          fallback:
            enabled: true
            fallback_method: GET
            fallback_endpoint: /v1/orders/fallback
        half_open_state:
          max_requests: 10
          duration: 10s
        error_codes:
          retryable:
            - 408  # Request Timeout
            - 429  # Too Many Requests
            - 500  # Internal Server Error
            - 502  # Bad Gateway
            - 503  # Service Unavailable
            - 504  # Gateway Timeout
          non_retryable:
            - 400  # Bad Request
            - 401  # Unauthorized
            - 403  # Forbidden
            - 404  # Not Found
            - 409  # Conflict
    latency_budgets:
      order-service:
        timeout:
          connect: 2s
          request: 5s
          idle: 30s
        slow_request_threshold: 3s
```

### 5.2 Bulkhead Pattern

Isolates failures by limiting the number of concurrent requests to a downstream service.

```yaml
# Bulkhead configuration
bulkhead:
  order-service:
    max_concurrent_calls: 100
    max_queue_size: 50
    queue_timeout: 5s
    thread_pool:
      core_size: 20
      max_size: 100
      keep_alive: 60s
      queue_size: 1000
  inventory-service:
    max_concurrent_calls: 50
    max_queue_size: 25
    queue_timeout: 3s
  payment-service:
    max_concurrent_calls: 10
    max_queue_size: 5
    queue_timeout: 10s
    thread_pool:
      core_size: 5
      max_size: 20
      keep_alive: 120s
      queue_size: 100
```

### 5.3 Retry Pattern with Backoff

```yaml
# Retry configuration
retry_policy:
  global:
    max_attempts: 3
    exponential_backoff:
      base_delay: 100ms
      max_delay: 30s
      multiplier: 2.0
      jitter: 0.2
    retry_on:
      - connect-failure
      - timeout
      - reset
      - retriable-status-codes
      - retriable-headers
    idempotent: true
  service_overrides:
    payment-service:
      max_attempts: 5
      base_delay: 500ms
      max_delay: 60s
    notification-service:
      max_attempts: 2
      base_delay: 1s
      non_retryable_errors:
        - INVALID_PHONE_NUMBER
        - INVALID_EMAIL_FORMAT
        - TEMPLATE_NOT_FOUND
```

### 5.4 Fallback Pattern

```yaml
# Fallback configurations for degraded mode
fallbacks:
  order-service:
    get-order:
      primary: /v1/orders/{id}
      fallback:
        type: cache
        cache_key: "order:{id}"
        cache_ttl: 300s
        stale_while_revalidate: 60s
      circuit_breaker_mode: failure_count
    list-orders:
      primary: /v1/orders
      fallback:
        type: static
        response:
          data: []
          pagination:
            page: 1
            page_size: 20
            total_items: 0
            total_pages: 0
          meta:
            degraded: true
            message: "Service is operating in degraded mode"
    create-order:
      fallback:
        type: queue
        queue_endpoint: /v1/orders/pending
        max_queue_size: 1000
        ttl: 3600s
```

---

## 6. Service Definition YAML Specifications

### 6.1 Kubernetes Service Deployment

```yaml
# Complete Kubernetes deployment for a microservice
apiVersion: apps/v1
kind: Deployment
metadata:
  name: order-service
  namespace: platform
  labels:
    app: order-service
    version: v1.2.3
    team: orders
    domain: e-commerce
    managed-by: flux
  annotations:
    prometheus.io/scrape: "true"
    prometheus.io/port: "9090"
    prometheus.io/path: "/metrics"
    linkerd.io/inject: "enabled"
    config.kubernetes.io/track: "true"
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  selector:
    matchLabels:
      app: order-service
      version: v1.2.3
  template:
    metadata:
      labels:
        app: order-service
        version: v1.2.3
        team: orders
        domain: e-commerce
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "9090"
        linkerd.io/inject: "enabled"
    spec:
      serviceAccountName: order-service
      securityContext:
        runAsNonRoot: true
        runAsUser: 1000
        runAsGroup: 1000
        fsGroup: 1000
        seccompProfile:
          type: RuntimeDefault
      affinity:
        podAntiAffinity:
          preferredDuringSchedulingIgnoredDuringExecution:
          - weight: 100
            podAffinityTerm:
              labelSelector:
                matchLabels:
                  app: order-service
              topologyKey: kubernetes.io/hostname
        podAffinity:
          preferredDuringSchedulingIgnoredDuringExecution:
          - weight: 50
            podAffinityTerm:
              labelSelector:
                matchLabels:
                  app: postgres-client
              topologyKey: topology.kubernetes.io/zone
      topologySpreadConstraints:
      - maxSkew: 1
        topologyKey: topology.kubernetes.io/zone
        whenUnsatisfiable: ScheduleAnyway
        labelSelector:
          matchLabels:
            app: order-service
      - maxSkew: 1
        topologyKey: kubernetes.io/hostname
        whenUnsatisfiable: ScheduleAnyway
        labelSelector:
          matchLabels:
            app: order-service
      tolerations:
      - key: "node-type"
        operator: "Equal"
        value: "application"
        effect: "NoSchedule"
      initContainers:
      - name: schema-migration
        image: order-service-migrations:1.2.3
        command: ["/app/bin/migrate"]
        args: ["up", "--timeout=60s"]
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: order-service-db-credentials
              key: url
        - name: MIGRATION_LOCK_TIMEOUT
          value: "30s"
        resources:
          requests:
            cpu: 100m
            memory: 64Mi
          limits:
            cpu: 500m
            memory: 256Mi
        securityContext:
          allowPrivilegeEscalation: false
          readOnlyRootFilesystem: true
          capabilities:
            drop:
            - ALL
      containers:
      - name: order-service
        image: order-service:1.2.3
        imagePullPolicy: Always
        ports:
        - name: http
          containerPort: 8080
          protocol: TCP
        - name: grpc
          containerPort: 9090
          protocol: TCP
        - name: admin
          containerPort: 8081
          protocol: TCP
        env:
        - name: SERVICE_NAME
          value: "order-service"
        - name: SERVICE_VERSION
          value: "1.2.3"
        - name: POD_NAME
          valueFrom:
            fieldRef:
              fieldPath: metadata.name
        - name: POD_NAMESPACE
          valueFrom:
            fieldRef:
              fieldPath: metadata.namespace
        - name: POD_IP
          valueFrom:
            fieldRef:
              fieldPath: status.podIP
        - name: NODE_NAME
          valueFrom:
            fieldRef:
              fieldPath: spec.nodeName
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: order-service-db-credentials
              key: url
        - name: KAFKA_BOOTSTRAP_SERVERS
          valueFrom:
            configMapKeyRef:
              name: kafka-config
              key: bootstrap_servers
        - name: REDIS_URL
          valueFrom:
            secretKeyRef:
              name: order-service-redis-credentials
              key: url
        - name: JAEGER_ENDPOINT
          value: "http://jaeger-agent.observability:6831"
        - name: OTEL_EXPORTER_OTLP_ENDPOINT
          value: "http://otel-collector.observability:4317"
        - name: LOG_LEVEL
          value: "info"
        - name: LOG_FORMAT
          value: "json"
        - name: GOMAXPROCS
          value: "4"
        - name: GOMEMLIMIT
          value: "2GiB"
        - name: HEALTH_PORT
          value: "8081"
        - name: METRICS_PORT
          value: "9090"
        - name: GRACEFUL_SHUTDOWN_TIMEOUT
          value: "30s"
        - name: READ_TIMEOUT
          value: "30s"
        - name: WRITE_TIMEOUT
          value: "30s"
        - name: IDLE_TIMEOUT
          value: "120s"
        - name: KEEP_ALIVE
          value: "90s"
        - name: MAX_HEADER_BYTES
          value: "16384"
        - name: API_RATE_LIMIT
          value: "1000"
        - name: API_RATE_LIMIT_BURST
          value: "100"
        resources:
          requests:
            cpu: 500m
            memory: 512Mi
          limits:
            cpu: 2000m
            memory: 2Gi
        livenessProbe:
          httpGet:
            path: /health/live
            port: admin
            httpHeaders:
            - name: X-Health-Check
              value: "true"
          initialDelaySeconds: 10
          periodSeconds: 15
          timeoutSeconds: 5
          failureThreshold: 3
          successThreshold: 1
        readinessProbe:
          httpGet:
            path: /health/ready
            port: admin
            httpHeaders:
            - name: X-Health-Check
              value: "true"
          initialDelaySeconds: 5
          periodSeconds: 10
          timeoutSeconds: 3
          failureThreshold: 3
          successThreshold: 1
        startupProbe:
          httpGet:
            path: /health/started
            port: admin
          initialDelaySeconds: 0
          periodSeconds: 5
          timeoutSeconds: 3
          failureThreshold: 30
          successThreshold: 1
        securityContext:
          allowPrivilegeEscalation: false
          readOnlyRootFilesystem: true
          capabilities:
            drop:
            - ALL
        volumeMounts:
        - name: tmp
          mountPath: /tmp
        - name: cache
          mountPath: /app/cache
        - name: config
          mountPath: /app/config
          readOnly: true
        - name: certificates
          mountPath: /etc/ssl/certs
          readOnly: true
      - name: envoy-proxy
        image: envoyproxy/envoy:v1.28.0
        args:
        - -c
        - /etc/envoy/envoy.yaml
        - --service-cluster
        - order-service
        - --service-node
        - $(POD_NAME).$(POD_NAMESPACE)
        env:
        - name: POD_NAME
          valueFrom:
            fieldRef:
              fieldPath: metadata.name
        - name: POD_NAMESPACE
          valueFrom:
            fieldRef:
              fieldPath: metadata.namespace
        ports:
        - name: envoy-http
          containerPort: 15001
          protocol: TCP
        - name: envoy-admin
          containerPort: 15000
          protocol: TCP
        resources:
          requests:
            cpu: 100m
            memory: 128Mi
          limits:
            cpu: 500m
            memory: 512Mi
        readinessProbe:
          tcpSocket:
            port: envoy-http
          initialDelaySeconds: 5
          periodSeconds: 10
        securityContext:
          runAsUser: 0
          allowPrivilegeEscalation: false
          readOnlyRootFilesystem: true
          capabilities:
            drop:
            - ALL
        volumeMounts:
        - name: envoy-config
          mountPath: /etc/envoy
      volumes:
      - name: tmp
        emptyDir:
          medium: Memory
          sizeLimit: 256Mi
      - name: cache
        emptyDir:
          medium: Memory
          sizeLimit: 512Mi
      - name: config
        configMap:
          name: order-service-config
          optional: true
      - name: certificates
        configMap:
          name: public-certs
          optional: true
      - name: envoy-config
        configMap:
          name: order-service-envoy-config
      dnsPolicy: ClusterFirst
      hostNetwork: false
      restartPolicy: Always
      terminationGracePeriodSeconds: 60
---
# Kubernetes Service definition
apiVersion: v1
kind: Service
metadata:
  name: order-service
  namespace: platform
  labels:
    app: order-service
    team: orders
  annotations:
    prometheus.io/scrape: "true"
    prometheus.io/port: "9090"
spec:
  type: ClusterIP
  clusterIP: None
  ports:
  - name: http
    port: 80
    targetPort: 8080
    protocol: TCP
  - name: grpc
    port: 9091
    targetPort: 9090
    protocol: TCP
  - name: admin
    port: 8081
    targetPort: 8081
    protocol: TCP
  - name: metrics
    port: 9090
    targetPort: 9090
    protocol: TCP
  selector:
    app: order-service
  publishNotReadyAddresses: false
  sessionAffinity: ClientIP
  sessionAffinityConfig:
    clientIP:
      timeoutSeconds: 10800
---
# Headless service for stateful sets
apiVersion: v1
kind: Service
metadata:
  name: order-service-headless
  namespace: platform
  labels:
    app: order-service
spec:
  type: ClusterIP
  clusterIP: None
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
---
# HorizontalPodAutoscaler
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: order-service-hpa
  namespace: platform
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: order-service
  minReplicas: 3
  maxReplicas: 50
  metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
  - type: Pods
    pods:
      metric:
        name: http_requests_per_second
      target:
        type: AverageValue
        averageValue: "1000"
  - type: External
    external:
      metric:
        name: queue_depth
        selector:
          matchLabels:
            queue: "orders"
      target:
        type: AverageValue
        averageValue: "100"
  behavior:
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
      - type: Percent
        value: 10
        periodSeconds: 60
      - type: Pods
        value: 2
        periodSeconds: 60
      selectPolicy: Max
    scaleUp:
      stabilizationWindowSeconds: 0
      policies:
      - type: Percent
        value: 100
        periodSeconds: 15
      - type: Pods
        value: 10
        periodSeconds: 15
      selectPolicy: Max
---
# PodDisruptionBudget
apiVersion: policy/v1
kind: PodDisruptionBudget
metadata:
  name: order-service-pdb
  namespace: platform
spec:
  maxUnavailable: 1
  selector:
    matchLabels:
      app: order-service
```

### 6.2 ServiceAccount and RBAC

```yaml
# ServiceAccount
apiVersion: v1
kind: ServiceAccount
metadata:
  name: order-service
  namespace: platform
  labels:
    app: order-service
  annotations:
    eks.amazonaws.com/role-arn: arn:aws:iam::123456789012:role/order-service-role
---
# ClusterRole for service permissions
apiVersion: rbac.authorization.k8s.io/v1
kind: ClusterRole
metadata:
  name: order-service
  labels:
    app: order-service
rules:
- apiGroups: [""]
  resources: ["configmaps"]
  verbs: ["get", "list", "watch"]
- apiGroups: [""]
  resources: ["secrets"]
  verbs: ["get", "list", "watch"]
- apiGroups: [""]
  resources: ["services"]
  verbs: ["get", "list", "watch"]
- apiGroups: ["networking.k8s.io"]
  resources: ["endpoints"]
  verbs: ["get", "list", "watch"]
- apiGroups: ["coordination.k8s.io"]
  resources: ["leases"]
  verbs: ["get", "create", "update"]
- apiGroups: ["discovery.k8s.io"]
  resources: ["endpointslices"]
  verbs: ["get", "list", "watch"]
---
# RoleBinding
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: order-service
  namespace: platform
subjects:
- kind: ServiceAccount
  name: order-service
  namespace: platform
roleRef:
  kind: ClusterRole
  name: order-service
  apiGroup: rbac.authorization.k8s.io
```

---

## 7. Decision Matrix

### 7.1 Service Decomposition Decision Matrix

| Scenario | Recommended Approach | Rationale |
|----------|---------------------|-----------|
| Team size < 5, simple domain | Single service or 2-3 services | Low complexity, minimize operational overhead |
| Multiple teams (> 10) | Strong bounded context boundaries | Team autonomy is critical |
| Rapid growth phase | Smaller services with clear boundaries | Enable independent scaling |
| Stability-focused phase | Consolidate related services | Reduce operational complexity |
| High regulatory requirements | Strict service isolation | Contain blast radius of compliance scope |
| Event-driven domain | Event-first decomposition | Natural event boundaries become service boundaries |
| Transactional domain | Aggregate-first with careful saga design | Minimize distributed transaction complexity |

### 7.2 Communication Protocol Decision Matrix

| Requirement | REST | gRPC | Messaging |
|-------------|------|------|----------|
| Latency (< 10ms) | ❌ | ✅ | ❌ |
| Streaming | ❌ | ✅ | ✅ (Kafka) |
| Browser clients | ✅ | ⚠️ (gRPC-Web) | ❌ |
| Debugging (human-readable) | ✅ | ❌ | ⚠️ |
| Strong typing | ❌ | ✅ | ⚠️ |
| Fire-and-forget | ❌ | ❌ | ✅ |
| Exactly-once delivery | ❌ | ❌ | ✅ |
| Schema evolution | ⚠️ | ✅ | ✅ |

### 7.3 Service Mesh Decision Matrix

| Requirement | Istio | Linkerd | No Mesh |
|-------------|-------|---------|---------|
| Complex routing rules | ✅ | ⚠️ | ❌ |
| mTLS minimal config | ✅ | ✅ | ❌ |
| Low resource overhead | ❌ | ✅ | N/A |
| Multi-cluster support | ✅ | ✅ | ❌ |
| WebAssembly extensibility | ✅ | ❌ | N/A |
| Simple operations | ⚠️ | ✅ | ✅ |
| Kubernetes only | ⚠️ | ⚠️ | ✅ |

---

## 8. Anti-Patterns and Failure Modes

### 8.1 Common Anti-Patterns

**Nanoservice Anti-Pattern**
Splitting services too finely creates:
- Excessive network hops
- Distributed transaction complexity
- Operational overhead explosion
- Harder debugging across service boundaries

**Monolithic Data Access Anti-Pattern**
Services accessing each other's databases directly creates:
- Implicit coupling through schema
- Impossible to enforce data consistency boundaries
- Race conditions on shared data
- Inability to evolve services independently

**Shared Library Coupling Anti-Pattern**
Over-sharing libraries between services causes:
- Version coupling (all services must upgrade together)
- Deployment coupling (a bug in shared lib affects all)
- Technology coupling (stuck with same language/framework)

### 8.2 Specific Failure Modes and Error Messages

**Failure Mode: Connection Pool Exhaustion**

```
Error: "dial tcp 10.0.0.50:8080: connect: cannot assign requested address"
Cause: Too many concurrent connections exhausting available ports
Solution: Implement connection pooling, bulkhead pattern

Error: "context deadline exceeded: client timeout"
Cause: Server not responding within timeout window
Solution: Increase timeout, check circuit breaker state, scale service

Error: "upstream connect error or disconnect/reset before headers"
Cause: Backend service crashed or is starting up
Solution: Configure proper readiness probes, increase failure threshold
```

**Failure Mode: Cascading Failures**

```
Error: "circuit breaker open: fast failure for order-service"
Cause: Downstream service returning errors above threshold
Symptom: Requests fail immediately instead of retrying
Solution: Set appropriate circuit breaker thresholds, implement fallback

Error: "retry exhausted after 3 attempts"
Cause: All retry attempts failed
Solution: Implement exponential backoff, check for systematic issues
```

**Failure Mode: Data Inconsistency**

```
Error: "optimistic lock failed: concurrent modification detected"
Cause: Two services modifying same entity simultaneously
Solution: Implement proper locking, use saga pattern for multi-service updates

Error: "message not found in log"
Cause: Event consumed multiple times or lost
Solution: Implement idempotency, use exactly-once delivery semantics
```

---

## 9. Production Checklist

### 9.1 Service Design Checklist

- [ ] Service has single responsibility within bounded context
- [ ] API contracts are versioned from the start
- [ ] Idempotency keys supported for all mutation operations
- [ ] Pagination implemented for all list endpoints
- [ ] Rate limiting configured at service and endpoint level
- [ ] Health endpoints implemented (/health/live, /health/ready)
- [ ] Graceful shutdown implemented with configurable timeout
- [ ] Structured logging with correlation IDs
- [ ] Distributed tracing configured
- [ ] Metrics exported in Prometheus format

### 9.2 Resilience Checklist

- [ ] Circuit breaker configured for all downstream calls
- [ ] Retry policy with exponential backoff and jitter
- [ ] Bulkhead isolation for critical downstream calls
- [ ] Fallback responses for degraded mode
- [ ] Timeout configured for all network calls
- [ ] Connection pooling implemented
- [ ] Load shedding configured for overload protection

### 9.3 Security Checklist

- [ ] mTLS enabled between services
- [ ] ServiceAccount with minimal permissions (RBAC)
- [ ] Network policies restricting traffic
- [ ] Secrets accessed viaVault or cloud secret manager
- [ ] No hardcoded credentials in code or config
- [ ] TLS 1.2+ enforced for external connections
- [ ] SecurityContext configured (non-root, read-only filesystem)

### 9.4 Operational Checklist

- [ ] Kubernetes deployment with proper resource limits
- [ ] HorizontalPodAutoscaler configured
- [ ] PodDisruptionBudget configured
- [ ] PodAntiAffinity for high availability
- [ ] Readiness and liveness probes configured
- [ ] Init container for database migrations
- [ ] Service monitor for Prometheus scraping
- [ ] Alerting rules configured

---

## 10. References

### Core References

- [Domain-Driven Design: Tackling Complexity in the Heart of Software](https://www.amazon.com/Domain-Driven-Design-Tackling-Complexity/dp/0321125215) - Eric Evans
- [Building Microservices: Designing Fine-Grained Systems](https://www.amazon.com/Building-Microservices-Designing-Fine-Grained-Systems/dp/1492034029) - Sam Newman
- [Implementing Domain-Driven Design](https://www.amazon.com/Implementing-Domain-Driven-Design-Vaughn-Vernon/dp/0321834577) - Vaughn Vernon
- [Microservices Patterns](https://www.microservices.io/patterns/index.html) - Chris Richardson

### Service Mesh References

- [Istio Documentation](https://istio.io/latest/docs/)
- [Linkerd Documentation](https://linkerd.io/2.14/overview/)
- [Envoy Proxy Documentation](https://www.envoyproxy.io/docs/envoy/latest/)

### API Design References

- [OpenAPI Specification](https://spec.openapis.org/oas/v3.0.3)
- [Google API Design Guide](https://cloud.google.com/apis/design)
- [REST API Design Rulebook](https://www.oreilly.com/library/view/rest-api-design/9781449317904/)

### Resilience Patterns References

- [Pattern: Circuit Breaker](https://docs.microsoft.com/en-us/azure/architecture/patterns/circuit-breaker)
- [Pattern: Bulkhead](https://docs.microsoft.com/en-us/azure/architecture/patterns/bulkhead)
- [Pattern: Retry](https://docs.microsoft.com/en-us/azure/architecture/patterns/retry)
- [Pattern: Fallback](https://docs.microsoft.com/en-us/azure/architecture/patterns/fallback)

### Kubernetes References

- [Kubernetes Documentation](https://kubernetes.io/docs/)
- [Production Kubernetes](https://www.oreilly.com/library/view/production-kubernetes/9781492055536/)

### Tooling References

- [Envoy Proxy](https://www.envoyproxy.io/)
- [Jaeger: Distributed Tracing](https://www.jaegertracing.io/)
- [Prometheus](https://prometheus.io/)
- [Grafana](https://grafana.com/)