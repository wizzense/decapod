# METRICS.md - Metrics and Observability Architecture Standards

**Authority:** guidance (comprehensive topic with exact specifications)
**Layer:** Architecture
**Binding:** No
**Scope:** Comprehensive topic coverage for pre-inference context

---

## 1. SLI/SLO/SLA Definitions

### 1.1 Standard SLI Definitions

```yaml
# sli-definitions.yaml - Standard Service Level Indicators

apiVersion: v1
kind: ConfigMap
metadata:
  name: sli-definitions
  namespace: monitoring
data:
  # API Service SLIs
  api-availability: |
    name: API Availability
    description: Percentage of successful requests (2xx/3xx responses)
    query: |
      sum(rate(http_requests_total{status=~"2..|3.."}[5m]))
      /
      sum(rate(http_requests_total[5m]))
    good: Higher is better
    threshold: 99.9

  api-latency-p50: |
    name: API Latency P50
    description: 50th percentile response time
    query: |
      histogram_quantile(0.50, 
        sum(rate(http_request_duration_seconds_bucket[5m])) by (le)
      )
    good: Lower is better
    threshold: 100ms

  api-latency-p95: |
    name: API Latency P95
    description: 95th percentile response time
    query: |
      histogram_quantile(0.95,
        sum(rate(http_request_duration_seconds_bucket[5m])) by (le)
      )
    good: Lower is better
    threshold: 500ms

  api-latency-p99: |
    name: API Latency P99
    description: 99th percentile response time
    query: |
      histogram_quantile(0.99,
        sum(rate(http_request_duration_seconds_bucket[5m])) by (le)
      )
    good: Lower is better
    threshold: 1s

  api-errors: |
    name: API Error Rate
    description: Percentage of 5xx responses
    query: |
      sum(rate(http_requests_total{status=~"5.."}[5m]))
      /
      sum(rate(http_requests_total[5m]))
    good: Lower is better
    threshold: 0.1%

  # Database SLIs
  db-connections: |
    name: Database Connection Pool Utilization
    description: Percentage of used connections
    query: |
      pg_stat_activity_count / pg_settings_max_connections
    good: Lower is better
    threshold: 80%

  db-query-latency: |
    name: Database Query Latency P99
    description: 99th percentile query duration
    query: |
      histogram_quantile(0.99,
        sum(rate(pg_stat_statements_mean_exec_time[5m])) by (le)
      )
    good: Lower is better
    threshold: 1s

  # Infrastructure SLIs
  pod-restarts: |
    name: Pod Restart Rate
    description: Number of pod restarts per minute
    query: |
      sum(rate(kube_pod_container_status_restarts_total[5m])) by (pod, namespace)
    good: Lower is better
    threshold: 0.01

  node-cpu-usage: |
    name: Node CPU Usage
    description: Percentage of CPU used
    query: |
      1 - (sum(rate(node_cpu_seconds_total{mode="idle"}[5m])) by (instance) / count(sum(rate(node_cpu_seconds_total[5m])) by (instance)))
    good: Lower is better
    threshold: 85%

  node-memory-usage: |
    name: Node Memory Usage
    description: Percentage of memory used
    query: |
      1 - (node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes)
    good: Lower is better
    threshold: 85%
```

### 1.2 SLO Configuration

```yaml
# slo-config.yaml - Service Level Objectives

apiVersion: v1
kind: ConfigMap
metadata:
  name: slo-config
  namespace: monitoring
data:
  # Web Application SLOs
  web-availability-slo: |
    name: Web Availability
    target: 99.9%
    window: 30d
    sli: api-availability
    errorBudgetPolicy:
      burnRateThreshold: 14.4  # 1% of errors in 1 hour = 14.4x burn rate
      action: page
    alertRules:
      - name: web-availability-error-budget-90%
        severity: warning
        threshold: 90%
        action: notify
      - name: web-availability-error-budget-50%
        severity: critical
        threshold: 50%
        action: page

  web-latency-slo: |
    name: Web Latency
    target: 99%
    window: 30d
    sli: api-latency-p99
    threshold: 1s
    alertRules:
      - name: web-latency-error-budget-90%
        severity: warning
        threshold: 90%
        action: notify
      - name: web-latency-slo-breach
        severity: critical
        threshold: 100%
        action: page

  # Checkout Service SLOs (stricter)
  checkout-availability-slo: |
    name: Checkout Availability
    target: 99.95%
    window: 30d
    sli: api-availability
    alertRules:
      - name: checkout-availability-warning
        severity: warning
        threshold: 95% error budget consumed
        action: notify
      - name: checkout-availability-critical
        severity: critical
        threshold: 50% remaining error budget
        action: page

  checkout-latency-slo: |
    name: Checkout Latency
    target: 99.5%
    window: 30d
    sli: api-latency-p99
    threshold: 500ms
    alertRules:
      - name: checkout-latency-warning
        severity: warning
        action: notify
      - name: checkout-latency-critical
        severity: critical
        action: page

  # Infrastructure SLOs
  infrastructure-availability-slo: |
    name: Infrastructure Availability
    target: 99.99%
    window: 30d
    sli: node-cpu-usage
    # Alert when sustained high usage
```

### 1.3 SLA Document

```markdown
# Service Level Agreement (SLA)

## Service: API Platform
## Version: 1.0
## Effective Date: 2024-01-01

---

## 1. Service Scope

This SLA covers the following services:
- REST API (api.example.com)
- GraphQL API (api.example.com/graphql)
- WebSocket connections (ws.example.com)

## 2. Service Level Objectives

| Metric               | Objective    | Measurement |
|---------------------|---------------|-------------|
| Availability        | 99.9%         | Per month   |
| Error Rate          | < 0.1%        | Per month   |
| Latency P50         | < 100ms       | Per minute  |
| Latency P95         | < 500ms       | Per minute  |
| Latency P99         | < 1s          | Per minute  |

## 3. Definitions

**Availability** = (Total Requests - Failed Requests) / Total Requests

**Error Rate** = Failed Requests / Total Requests
- Failed requests: HTTP 5xx responses
- Excludes: Planned maintenance, client errors (4xx)

**Latency** = Time from request received to response sent

## 4. Exclusions

The following are excluded from SLA calculations:
1. Planned maintenance (with 48-hour notice)
2. Force majeure events
3. Third-party service failures
4. Client-side issues
5. DDoS attacks

## 5. Support

| Severity | Response Time | Resolution Time |
|----------|---------------|-----------------|
| Critical | 15 minutes    | 4 hours         |
| High     | 1 hour       | 24 hours        |
| Medium   | 4 hours      | 72 hours        |
| Low      | 24 hours     | 7 days          |

## 6. Credits

| Availability     | Credit      |
|------------------|--------------|
| 99.0% - 99.89%   | 10%          |
| 95.0% - 98.99%   | 25%          |
| 90.0% - 94.99%   | 50%          |
| < 90.0%          | 100%         |

Credits are applied as service credits on future invoices.

## 7. Maintenance Windows

- Weekly: Sunday 02:00-04:00 UTC (4 hours)
- Monthly: First Sunday 00:00-06:00 UTC (6 hours)

Emergency maintenance may be performed with customer notification.
```

## 2. Prometheus Configurations

### 2.1 Complete Prometheus Configuration

```yaml
# prometheus/prometheus.yaml - Complete Prometheus configuration

global:
  scrape_interval: 15s
  evaluation_interval: 15s
  external_labels:
    cluster: 'production'
    environment: 'prod'
  
  # Remote write configuration
  remote_write:
    - url: https://remote-write.grafana.net/api/v1/write
      bearer_token: ${GRAFANA_TOKEN}
      queue_config:
        capacity: 10000
        max_shards: 30
        min_shards: 5
        max_samples_per_send: 2000
        batch_send_deadline: 30s
        retry_on_http_429: true

# Alerting
alerting:
  alertmanagers:
    - static_configs:
        - targets:
            - alertmanager:9093

# Rules
rule_files:
  - /etc/prometheus/rules/*.yml
  - /etc/prometheus/rules.d/*.yml

# Scrape configs
scrape_configs:
  # Prometheus self-monitoring
  - job_name: 'prometheus'
    static_configs:
      - targets: ['localhost:9090']
    metrics_path: /metrics

  # Kubernetes API server
  - job_name: 'kubernetes-apiserver'
    kubernetes_sd_configs:
      - role: endpoints
    scheme: https
    tls_config:
      ca_file: /var/run/secrets/kubernetes.io/serviceaccount/ca.crt
    bearer_token_file: /var/run/secrets/kubernetes.io/serviceaccount/token
    relabel_configs:
      - source_labels: [__meta_kubernetes_namespace, __meta_kubernetes_service_name]
        action: keep
        regex: default;kubernetes

  # Kubernetes nodes
  - job_name: 'kubernetes-nodes'
    kubernetes_sd_configs:
      - role: node
    relabel_configs:
      - action: labelmap
        regex: __meta_kubernetes_node_label_(.+)
      - target_label: __address__
        replacement: kubernetes.default.svc:443
      - source_labels: [__meta_kubernetes_node_name]
        regex: (.+)
        target_label: __metrics_path__
        replacement: /api/v1/nodes/${1}/proxy/metrics

  # Kubernetes pods
  - job_name: 'kubernetes-pods'
    kubernetes_sd_configs:
      - role: pod
    relabel_configs:
      - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_scrape]
        action: keep
        regex: true
      - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_path]
        action: replace
        target_label: __metrics_path__
        regex: (.+)
      - source_labels: [__address__, __meta_kubernetes_pod_annotation_prometheus_io_port]
        action: replace
        regex: ([^:]+)(?::\d+)?;(\d+)
        replacement: $1:$2
        target_label: __address__
      - action: labelmap
        regex: __meta_kubernetes_pod_label_(.+)
      - source_labels: [__meta_kubernetes_namespace]
        action: replace
        target_label: kubernetes_namespace
      - source_labels: [__meta_kubernetes_pod_name]
        action: replace
        target_label: kubernetes_pod_name

  # Application metrics (annotated pods)
  - job_name: 'application-metrics'
    kubernetes_sd_configs:
      - role: pod
    relabel_configs:
      - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_scrape]
        action: keep
        regex: true
      - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_scheme]
        action: replace
        target_label: __scheme__
        regex: (https?)
      - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_path]
        action: replace
        target_label: __metrics_path__
        regex: (.+)
      - source_labels: [__address__, __meta_kubernetes_pod_annotation_prometheus_io_port]
        action: replace
        regex: ([^:]+)(?::\d+)?;(\d+)
        replacement: $1:$2
        target_label: __address__

  # Blackbox exporter for external targets
  - job_name: 'blackbox-exporter'
    metrics_path: /probe
    params:
      module: [http_2xx]
    static_configs:
      - targets:
          - https://api.example.com/health
    relabel_configs:
      - source_labels: [__address__]
        target_label: __param_target
      - target_label: __address__
        replacement: blackbox-exporter:9115

  # Redis metrics
  - job_name: 'redis'
    static_configs:
      - targets: ['redis:9121']

  # PostgreSQL metrics
  - job_name: 'postgresql'
    static_configs:
      - targets: ['postgres-exporter:9187']

  # RabbitMQ metrics
  - job_name: 'rabbitmq'
    static_configs:
      - targets: ['rabbitmq:15692']

  # Node exporter for host metrics
  - job_name: 'node-exporter'
    kubernetes_sd_configs:
      - role: node
    relabel_configs:
      - source_labels: [__meta_kubernetes_node_name]
        regex: (.+)
        replacement: /api/v1/nodes/$1/proxy/metrics
        target_label: __metrics_path__
      - source_labels: [__meta_kubernetes_node_name]
        action: replace
        target_label: node
```

### 2.2 Recording Rules

```yaml
# prometheus/recording-rules.yaml

groups:
  - name: application-recording-rules
    interval: 30s
    rules:
      # Request rate
      - record: application:http_requests_total:rate5m
        expr: |
          sum(rate(http_requests_total[5m])) by (service, method, status)
      
      - record: application:http_requests_total:rate1h
        expr: |
          sum(rate(http_requests_total[1h])) by (service, method, status)

      # Request latency
      - record: application:http_request_duration_seconds:avg5m
        expr: |
          sum(rate(http_request_duration_seconds_sum[5m])) by (service, method)
          /
          sum(rate(http_request_duration_seconds_count[5m])) by (service, method)

      - record: application:http_request_duration_seconds:p955m
        expr: |
          histogram_quantile(0.95,
            sum(rate(http_request_duration_seconds_bucket[5m])) by (service, method, le)
          )

      - record: application:http_request_duration_seconds:p99_5m
        expr: |
          histogram_quantile(0.99,
            sum(rate(http_request_duration_seconds_bucket[5m])) by (service, method, le)
          )

      # Error rate
      - record: application:http_errors_total:rate5m
        expr: |
          sum(rate(http_requests_total{status=~"5.."}[5m])) by (service)

      - record: application:error_rate:ratio5m
        expr: |
          sum(rate(http_requests_total{status=~"5.."}[5m])) by (service)
          /
          sum(rate(http_requests_total[5m])) by (service)

  - name: business-metrics
    interval: 60s
    rules:
      # Order metrics
      - record: orders:created:rate5m
        expr: |
          sum(rate(orders_created_total[5m]))

      - record: orders:completed:rate5m
        expr: |
          sum(rate(orders_completed_total[5m]))

      - record: orders:failed:rate5m
        expr: |
          sum(rate(orders_failed_total[5m]))

      # Revenue metrics (assuming $ value in orders)
      - record: revenue:total:rate1h
        expr: |
          sum(rate(order_total_amount_sum[1h]))

      - record: revenue:average:rate1h
        expr: |
          sum(rate(order_total_amount_sum[1h]))
          /
          sum(rate(orders_completed_total[1h]))

      # User metrics
      - record: users:registered:rate1d
        expr: |
          sum(increase(users_registered_total[1d]))

      - record: users:active:rate5m
        expr: |
          sum(rate(users_active_sessions_total[5m])) by (service)

  - name: infrastructure-recording-rules
    interval: 30s
    rules:
      # Kubernetes pod resource usage
      - record: kubernetes:pods:cpu_usage:rate5m
        expr: |
          sum(rate(container_cpu_usage_seconds_total[5m])) by (namespace, pod)
          / on (namespace, pod) group_left()
          sum(kube_pod_container_resource_limits_cpu_cores) by (namespace, pod)

      - record: kubernetes:pods:memory_usage:ratio
        expr: |
          sum(container_memory_working_set_bytes) by (namespace, pod)
          / on (namespace, pod) group_left()
          sum(kube_pod_container_resource_limits_memory_bytes) by (namespace, pod)

      # Database connection pool
      - record: postgresql:connections:used_ratio
        expr: |
          pg_stat_activity_count
          /
          pg_settings_max_connections

      - record: postgresql:queries:running:rate5m
        expr: |
          sum(rate(pg_stat_activity_count{state="active"}[5m])) by (datname)

      # Queue depth
      - record: rabbitmq:queue:depth:rate5m
        expr: |
          sum(rate(rabbitmq_queue_messages{queue="orders"}[5m])) by (queue)
```

## 3. Alerting Rules

### 3.1 Complete Alert Configuration

```yaml
# prometheus/alert-rules.yaml

groups:
  - name: high-level-alerts
    interval: 30s
    rules:
      # Service Level Objective Alerts
      - alert: SLOServiceAvailabilityWarning
        expr: |
          1 - (
            sum(rate(http_requests_total{status=~"2..|3.."}[5m])) by (service)
            /
            sum(rate(http_requests_total[5m])) by (service)
          ) > 0.001  # 99.9% SLO warning at 90% budget consumed
        for: 5m
        labels:
          severity: warning
          team: platform
        annotations:
          summary: "Service {{ $labels.service }} availability below SLO target"
          description: "Current availability: {{ $value | humanizePercentage }} (SLO target: 99.9%)"
          runbook_url: "https://runbooks.example.com/availability-warning"

      - alert: SLOServiceAvailabilityCritical
        expr: |
          1 - (
            sum(rate(http_requests_total{status=~"2..|3.."}[5m])) by (service)
            /
            sum(rate(http_requests_total[5m])) by (service)
          ) > 0.005  # 99.5% SLO critical at 50% budget remaining
        for: 2m
        labels:
          severity: critical
          team: platform
        annotations:
          summary: "CRITICAL: Service {{ $labels.service }} availability severely degraded"
          description: "Current availability: {{ $value | humanizePercentage }} (SLO target: 99.9%)"
          runbook_url: "https://runbooks.example.com/availability-critical"

      - alert: SLOLatencyWarning
        expr: |
          histogram_quantile(0.99, 
            sum(rate(http_request_duration_seconds_bucket[5m])) by (service, le)
          ) > 1
        for: 5m
        labels:
          severity: warning
          team: platform
        annotations:
          summary: "Service {{ $labels.service }} latency above SLO target"
          description: "P99 latency: {{ $value | humanizeDuration }} (SLO target: 1s)"

  - name: infrastructure-alerts
    interval: 30s
    rules:
      # Kubernetes alerts
      - alert: KubePodNotReady
        expr: |
          sum by (namespace, pod) (kube_pod_status_phase{phase=~"Pending|Unknown"}) > 0
        for: 10m
        labels:
          severity: warning
          team: platform
        annotations:
          summary: "Pod {{ $labels.namespace }}/{{ $labels.pod }} is not ready"
          description: "Pod has been in non-ready state for more than 10 minutes"

      - alert: KubePodCrashLooping
        expr: |
          rate(kube_pod_container_status_restarts_total[5m]) > 0.1
        for: 5m
        labels:
          severity: warning
          team: platform
        annotations:
          summary: "Pod {{ $labels.namespace }}/{{ $labels.pod }} is crash looping"
          description: "Pod has restarted {{ $value | humanize }} times in the last 5 minutes"

      - alert: KubeDeploymentReplicasMismatch
        expr: |
          kube_deployment_spec_replicas != kube_deployment_status_replicas_available
        for: 10m
        labels:
          severity: warning
          team: platform
        annotations:
          summary: "Deployment {{ $labels.namespace }}/{{ $labels.deployment }} replica mismatch"
          description: "Expected {{ $value }} replicas but only {{ $value }} available"

      - alert: KubeHPA scaleLimiter
        expr: |
          kube_hpa_status_condition{condition="ScalingLimited"} == 1
        for: 5m
        labels:
          severity: warning
          team: platform
        annotations:
          summary: "HPA {{ $labels.namespace }}/{{ $labels.hpa }} is scale-limited"
          description: "HPA has hit scale limits and cannot scale"

      # Node alerts
      - alert: NodeHighCPU
        expr: |
          100 - (avg by (instance) (rate(node_cpu_seconds_total{mode="idle"}[5m])) * 100) > 85
        for: 10m
        labels:
          severity: warning
          team: platform
        annotations:
          summary: "Node {{ $labels.instance }} CPU usage high"
          description: "Node CPU usage is above 85% for 10 minutes"

      - alert: NodeHighMemory
        expr: |
          (1 - (node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes)) * 100 > 85
        for: 10m
        labels:
          severity: warning
          team: platform
        annotations:
          summary: "Node {{ $labels.instance }} memory usage high"
          description: "Node memory usage is above 85% for 10 minutes"

      - alert: NodeDiskSpaceLow
        expr: |
          (node_filesystem_avail_bytes{mountpoint="/"} / node_filesystem_size_bytes{mountpoint="/"}) * 100 < 15
        for: 5m
        labels:
          severity: warning
          team: platform
        annotations:
          summary: "Node {{ $labels.instance }} disk space low"
          description: "Disk space available is below 15%"

      # API/Application alerts
      - alert: APIHighErrorRate
        expr: |
          sum(rate(http_requests_total{status=~"5.."}[5m])) by (service) / sum(rate(http_requests_total[5m])) by (service) > 0.01
        for: 5m
        labels:
          severity: warning
          team: backend
        annotations:
          summary: "Service {{ $labels.service }} error rate high"
          description: "Error rate is above 1% for 5 minutes"

      - alert: APIHighLatency
        expr: |
          histogram_quantile(0.95, 
            sum(rate(http_request_duration_seconds_bucket[5m])) by (service, le)
          ) > 0.5
        for: 5m
        labels:
          severity: warning
          team: backend
        annotations:
          summary: "Service {{ $labels.service }} latency high"
          description: "P95 latency is above 500ms for 5 minutes"

      # Database alerts
      - alert: DatabaseConnectionsHigh
        expr: |
          pg_stat_activity_count / pg_settings_max_connections > 0.8
        for: 5m
        labels:
          severity: warning
          team: backend
        annotations:
          summary: "PostgreSQL connection pool high"
          description: "Database connections above 80% of max"

      - alert: DatabaseReplicationLag
        expr: |
          pg_replication_lag_seconds > 30
        for: 5m
        labels:
          severity: warning
          team: backend
        annotations:
          summary: "PostgreSQL replication lag"
          description: "Replica is {{ $value }}s behind primary"

      # Queue alerts
      - alert: QueueDepthHigh
        expr: |
          rabbitmq_queue_messages{queue="orders"} > 1000
        for: 10m
        labels:
          severity: warning
          team: backend
        annotations:
          summary: "Order queue depth high"
          description: "Order queue has {{ $value }} messages waiting"

  - name: security-alerts
    interval: 30s
    rules:
      - alert: FailedLoginsHigh
        expr: |
          sum(rate(login_failures_total[5m])) by (service) > 10
        for: 5m
        labels:
          severity: warning
          team: security
        annotations:
          summary: "High number of failed logins"
          description: "More than 10 failed logins per minute on {{ $labels.service }}"

      - alert: AuthTokenAbuse
        expr: |
          sum(rate(auth_token_refresh_failures_total[5m])) by (service) > 5
        for: 5m
        labels:
          severity: warning
          team: security
        annotations:
          summary: "Potential token abuse detected"
          description: "Token refresh failures are high on {{ $labels.service }}"
```

## 4. Dashboard Design

### 4.1 Service Overview Dashboard

```json
{
  "title": "Service Overview",
  "uid": "service-overview",
  "panels": [
    {
      "title": "Request Rate",
      "type": "graph",
      "gridPos": {"x": 0, "y": 0, "w": 12, "h": 8},
      "targets": [
        {
          "expr": "sum(rate(http_requests_total[5m])) by (service)",
          "legendFormat": "{{service}}"
        }
      ],
      "yAxes": [
        {"label": "req/s", "min": 0},
        {"label": null}
      ]
    },
    {
      "title": "Error Rate",
      "type": "graph",
      "gridPos": {"x": 12, "y": 0, "w": 12, "h": 8},
      "targets": [
        {
          "expr": "sum(rate(http_requests_total{status=~\"5..\"}[5m])) by (service) / sum(rate(http_requests_total[5m])) by (service) * 100",
          "legendFormat": "{{service}}",
          "unit": "percent"
        }
      ]
    },
    {
      "title": "P99 Latency",
      "type": "graph",
      "gridPos": {"x": 0, "y": 8, "w": 12, "h": 8},
      "targets": [
        {
          "expr": "histogram_quantile(0.99, sum(rate(http_request_duration_seconds_bucket[5m])) by (service, le))",
          "legendFormat": "{{service}}",
          "unit": "s"
        }
      ]
    },
    {
      "title": "Apdex Score",
      "type": "stat",
      "gridPos": {"x": 12, "y": 8, "w": 6, "h": 4},
      "targets": [
        {
          "expr": "sum(rate(http_request_duration_seconds_bucket{le=\"0.5\"}[5m])) by (service) / sum(rate(http_request_duration_seconds_count[5m])) by (service)"
        }
      ],
      "fieldConfig": {
        "defaults": {
          "thresholds": {
            "steps": [
              {"value": 0, "color": "red"},
              {"value": 0.85, "color": "yellow"},
              {"value": 0.95, "color": "green"}
            ]
          }
        }
      }
    },
    {
      "title": "Active Pods",
      "type": "stat",
      "gridPos": {"x": 18, "y": 8, "w": 6, "h": 4},
      "targets": [
        {
          "expr": "sum(kube_pod_status_phase{phase=\"Running\"}) by (namespace)"
        }
      ]
    }
  ]
}
```

### 4.2 Business Metrics Dashboard

```json
{
  "title": "Business Metrics",
  "uid": "business-metrics",
  "panels": [
    {
      "title": "Revenue",
      "type": "stat",
      "gridPos": {"x": 0, "y": 0, "w": 6, "h": 4},
      "targets": [
        {
          "expr": "sum(increase(order_total_amount_sum[24h]))"
        }
      ],
      "fieldConfig": {
        "defaults": {
          "unit": "currencyUSD",
          "decimals": 2
        }
      }
    },
    {
      "title": "Orders (24h)",
      "type": "stat",
      "gridPos": {"x": 6, "y": 0, "w": 6, "h": 4},
      "targets": [
        {
          "expr": "sum(increase(orders_completed_total[24h]))"
        }
      ],
      "fieldConfig": {
        "defaults": {
          "unit": "none",
          "decimals": 0
        }
      }
    },
    {
      "title": "Conversion Rate",
      "type": "gauge",
      "gridPos": {"x": 12, "y": 0, "w": 6, "h": 4},
      "targets": [
        {
          "expr": "sum(rate(orders_completed_total[1h])) / sum(rate(page_views_total[1h])) * 100"
        }
      ],
      "fieldConfig": {
        "defaults": {
          "unit": "percent",
          "thresholds": {
            "steps": [
              {"value": 0, "color": "red"},
              {"value": 2, "color": "yellow"},
              {"value": 5, "color": "green"}
            ]
          }
        }
      }
    },
    {
      "title": "Active Users (Real-time)",
      "type": "stat",
      "gridPos": {"x": 18, "y": 0, "w": 6, "h": 4},
      "targets": [
        {
          "expr": "sum(users_active_sessions_total)"
        }
      ]
    },
    {
      "title": "Revenue Over Time",
      "type": "graph",
      "gridPos": {"x": 0, "y": 4, "w": 24, "h": 8},
      "targets": [
        {
          "expr": "sum(rate(order_total_amount_sum[1h]))",
          "legendFormat": "Revenue",
          "interval": "1h"
        }
      ]
    },
    {
      "title": "Orders Funnel",
      "type": "bargauge",
      "gridPos": {"x": 0, "y": 12, "w": 12, "h": 8},
      "targets": [
        {"expr": "sum(rate(page_views_total[1h]))", "legendFormat": "Views"},
        {"expr": "sum(rate(product_views_total[1h]))", "legendFormat": "Products Viewed"},
        {"expr": "sum(rate(add_to_cart_total[1h]))", "legendFormat": "Added to Cart"},
        {"expr": "sum(rate(checkout_started_total[1h]))", "legendFormat": "Checkout Started"},
        {"expr": "sum(rate(orders_completed_total[1h]))", "legendFormat": "Completed"}
      ]
    }
  ]
}
```

## 5. Complete Metric Definitions

### 5.1 Custom Metrics Implementation

```typescript
// metrics/application-metrics.ts - Complete metrics implementation

import { Registry, Counter, Histogram, Gauge, Summary } from 'prom-client';

const register = new Registry();

// Add default metrics
import { collectDefaultMetrics } from 'prom-client';
collectDefaultMetrics({ register });

// HTTP request metrics
const httpRequestsTotal = new Counter({
  name: 'http_requests_total',
  help: 'Total number of HTTP requests',
  labelNames: ['method', 'path', 'status'] as const,
  registers: [register],
});

const httpRequestDuration = new Histogram({
  name: 'http_request_duration_seconds',
  help: 'HTTP request duration in seconds',
  labelNames: ['method', 'path', 'status'] as const,
  buckets: [0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1, 2.5, 5, 10],
  registers: [register],
});

// Business metrics
const ordersCreated = new Counter({
  name: 'orders_created_total',
  help: 'Total number of orders created',
  labelNames: ['source', 'status'] as const,
  registers: [register],
});

const ordersCompleted = new Counter({
  name: 'orders_completed_total',
  help: 'Total number of completed orders',
  labelNames: ['payment_method'] as const,
  registers: [register],
});

const orderTotalAmount = new Summary({
  name: 'order_total_amount_dollars',
  help: 'Order total amount in dollars',
  labelNames: ['currency'] as const,
  percentiles: [0.25, 0.5, 0.75, 0.95, 0.99],
  registers: [register],
});

const activeUsers = new Gauge({
  name: 'users_active_sessions',
  help: 'Number of active user sessions',
  labelNames: ['service'] as const,
  registers: [register],
});

// Database metrics
const dbQueryDuration = new Histogram({
  name: 'db_query_duration_seconds',
  help: 'Database query duration in seconds',
  labelNames: ['operation', 'table'] as const,
  buckets: [0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1, 5],
  registers: [register],
});

const dbConnectionPoolSize = new Gauge({
  name: 'db_connection_pool_size',
  help: 'Database connection pool size',
  labelNames: ['state'] as const, // 'active' | 'idle' | 'total'
  registers: [register],
});

// Queue metrics
const queueDepth = newGauge({
  name: 'queue_messages_pending',
  help: 'Number of messages pending in queue',
  labelNames: ['queue', 'consumer'] as const,
  registers: [register],
});

const queueProcessingTime = new Histogram({
  name: 'queue_message_processing_seconds',
  help: 'Time to process a message',
  labelNames: ['queue', 'success'] as const,
  buckets: [0.01, 0.05, 0.1, 0.5, 1, 5, 10],
  registers: [register],
});

// Cache metrics
const cacheHits = new Counter({
  name: 'cache_hits_total',
  help: 'Total cache hits',
  labelNames: ['cache', 'key'] as const,
  registers: [register],
});

const cacheMisses = new Counter({
  name: 'cache_misses_total',
  help: 'Total cache misses',
  labelNames: ['cache', 'key'] as const,
  registers: [register],
});

// Middleware for HTTP metrics
function metricsMiddleware(req: Request, res: Response, next: NextFunction) {
  const start = process.hrtime.bigint();
  
  res.on('finish', () => {
    const end = process.hrtime.bigint();
    const duration = Number(end - start) / 1e9; // Convert to seconds
    
    const path = req.route?.path || req.path;
    const labels = {
      method: req.method,
      path: normalizePath(path),
      status: res.statusCode.toString(),
    };
    
    httpRequestsTotal.inc(labels);
    httpRequestDuration.observe(labels, duration);
  });
  
  next();
}

// Normalize paths to prevent high cardinality
function normalizePath(path: string): string {
  return path
    .replace(/\/user\/[^\/]+/, '/user/:id')
    .replace(/\/order\/[^\/]+/, '/order/:id')
    .replace(/\/product\/[^\/]+/, '/product/:id');
}

// Usage tracking helpers
function trackOrderCreated(order: Order): void {
  ordersCreated.inc({
    source: order.source,
    status: 'pending',
  });
}

function trackOrderCompleted(order: Order): void {
  ordersCompleted.inc({
    payment_method: order.paymentMethod,
  });
  
  orderTotalAmount.observe(
    { currency: order.currency },
    order.total
  );
}

function trackDbQuery(operation: string, table: string, duration: number): void {
  dbQueryDuration.observe({ operation, table }, duration);
}

function trackCacheAccess(cacheName: string, hit: boolean): void {
  if (hit) {
    cacheHits.inc({ cache: cacheName });
  } else {
    cacheMisses.inc({ cache: cacheName });
  }
}

// Export for Prometheus scraping
async function getMetrics(): Promise<string> {
  return register.metrics();
}

function getContentType(): string {
  return register.contentType;
}

export {
  register,
  httpRequestsTotal,
  httpRequestDuration,
  ordersCreated,
  ordersCompleted,
  orderTotalAmount,
  activeUsers,
  dbQueryDuration,
  dbConnectionPoolSize,
  queueDepth,
  queueProcessingTime,
  cacheHits,
  cacheMisses,
  metricsMiddleware,
  trackOrderCreated,
  trackOrderCompleted,
  trackDbQuery,
  trackCacheAccess,
  getMetrics,
  getContentType,
};
```

### 5.2 RED Metrics Implementation

```typescript
// metrics/red-metrics.ts - Request/Error/Duration (RED) metrics

class REDMetrics {
  private requestCounter: Counter;
  private errorCounter: Counter;
  private durationHistogram: Histogram;
  
  constructor(serviceName: string) {
    this.requestCounter = new Counter({
      name: `${serviceName}_requests_total`,
      help: 'Total requests',
      labelNames: ['method', 'path', 'status'],
    });
    
    this.errorCounter = new Counter({
      name: `${serviceName}_errors_total`,
      help: 'Total errors',
      labelNames: ['method', 'path', 'error_type'],
    });
    
    this.durationHistogram = new Histogram({
      name: `${serviceName}_request_duration_seconds`,
      help: 'Request duration',
      labelNames: ['method', 'path'],
      buckets: [0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1, 2.5, 5, 10],
    });
  }
  
  recordRequest(
    method: string,
    path: string,
    status: number,
    durationMs: number
  ): void {
    const labels = { method, path, status: status.toString() };
    
    this.requestCounter.inc(labels);
    
    if (status >= 500) {
      this.errorCounter.inc({ ...labels, error_type: 'server_error' });
    } else if (status >= 400) {
      this.errorCounter.inc({ ...labels, error_type: 'client_error' });
    }
    
    this.durationHistogram.observe(labels, durationMs / 1000);
  }
  
  recordError(
    method: string,
    path: string,
    errorType: string
  ): void {
    this.errorCounter.inc({
      method,
      path,
      error_type: errorType,
    });
  }
}

// USE Metrics (Utilization, Saturation, Errors)
class USEMetrics {
  private cpuUtilization: Gauge;
  private memoryUtilization: Gauge;
  private saturation: Gauge;
  
  constructor() {
    this.cpuUtilization = new Gauge({
      name: 'system_cpu_utilization',
      help: 'CPU utilization percentage',
    });
    
    this.memoryUtilization = new Gauge({
      name: 'system_memory_utilization',
      help: 'Memory utilization percentage',
    });
    
    this.saturation = new Gauge({
      name: 'system_saturation',
      help: 'System saturation (0-1)',
    });
  }
  
  recordCPU(percent: number): void {
    this.cpuUtilization.set(percent);
  }
  
  recordMemory(percent: number): void {
    this.memoryUtilization.set(percent);
  }
  
  recordSaturation(value: number): void {
    this.saturation.set(value);
  }
}
```

## 6. Decision Matrices

### 6.1 Alert Severity Matrix

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                              Alert Severity Decision Matrix                              │
├─────────────────────────────────────────────────────────────────────────────────────────┤
│ Impact                      │ Duration    │ Severity    │ Response              │
├─────────────────────────────┼────────────┼─────────────┼────────────────────────┤
│ Complete outage             │ Any        │ P1 Critical │ Immediate (< 15 min)  │
│ Major feature broken        │ > 5 min    │ P1 Critical │ Immediate (< 15 min)  │
│ Partial outage             │ > 15 min   │ P2 High     │ < 30 min              │
│ Performance degradation     │ > 5 min    │ P2 High     │ < 30 min              │
│ Minor feature broken        │ > 30 min   │ P3 Medium   │ < 4 hours             │
│ Non-critical issue         │ > 1 hour   │ P3 Medium   │ < 4 hours             │
│ Warning/threshold breach   │ Sustained  │ P4 Low      │ Next business day     │
│ Informational              │ Any        │ P5 Info     │ Weekly review         │
└─────────────────────────────┴────────────┴─────────────┴────────────────────────┘
```

### 6.2 Metric Selection Matrix

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                             Metric Selection Decision Matrix                             │
├─────────────────────────────────────────────────────────────────────────────────────────┤
│ Purpose                     │ Recommended Metrics              │ Collection Method      │
├─────────────────────────────┼──────────────────────────────────┼────────────────────────┤
│ Availability monitoring    │ Request success rate              │ APM/Access logs        │
│                             │ Error rate by type               │ Synthetic monitoring   │
│                             │ Endpoint health checks           │                        │
├─────────────────────────────┼──────────────────────────────────┼────────────────────────┤
│ Performance monitoring     │ Latency (P50, P95, P99)           │ APM/Access logs        │
│                             │ Throughput (req/s)               │                        │
│                             │ Saturation metrics               │                        │
├─────────────────────────────┼──────────────────────────────────┼────────────────────────┤
│ Resource monitoring        │ CPU utilization                   │ Infrastructure agents  │
│                             │ Memory utilization               │                        │
│                             │ Disk I/O                         │                        │
│                             │ Network I/O                      │                        │
├─────────────────────────────┼──────────────────────────────────┼────────────────────────┤
│ Business monitoring        │ Revenue                          │ Application metrics    │
│                             │ Conversions                      │                        │
│                             │ Active users                     │                        │
│                             │ Custom business events           │                        │
├─────────────────────────────┼──────────────────────────────────┼────────────────────────┤
│ Security monitoring        │ Failed login attempts             │ Auth service logs      │
│                             │ Auth failures                    │                        │
│                             │ Suspicious patterns              │                        │
└─────────────────────────────┴──────────────────────────────────┴────────────────────────┘
```

## 7. Anti-Patterns

### 7.1 Metrics Anti-Patterns

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                             Metrics Anti-Patterns to Avoid                               │
├─────────────────────────────────────────────────────────────────────────────────────────┤
│ Anti-Pattern                    │ Problem                       │ Solution                │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Too many metrics               │ Cost/performance issues        │ Curate metrics          │
│                                 │ Alert fatigue                  │ Prioritize key metrics  │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ High cardinality labels        │ Cardinality explosion          │ Normalize labels        │
│                                 │ Memory exhaustion              │ Use low-cardinality     │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ No metric naming convention   │ Confusion, duplication          │ Use prefixes            │
│                                 │ Hard to find metrics           │ service_metric_type     │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Missing error categorization  │ Can't distinguish error types  │ Label errors properly   │
│                                 │ Hard to triage                  │ By type, severity      │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Not tracking SLO metrics      │ Unknown service health          │ Define SLOs and SLIs    │
│                                 │ Alerting becomes arbitrary     │ Track error budget      │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Alerts without runbooks       │ Slower response                  │ Create runbook for      │
│                                 │ Misunderstood alerts            │ every alert             │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ No dashboard ownership        │ Stale dashboards                │ Assign ownership        │
│                                 │ Information overload            │ Regular reviews         │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Collecting but not using       │ Wasted resources                │ Regular metric review   │
│                                 │ Storage costs                   │ Remove unused metrics   │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ No latency histogram percentiles│ Can't identify P99 issues      │ Include P50/P95/P99    │
│                                 │ Miss slow requests              │ In histogram            │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Not normalizing paths         │ Cardinality explosion            │ Normalize paths         │
│                                 │ Label explosion                  │ /user/:id not /user/123 │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Missing infrastructure metrics│ Can't debug resource issues     │ Include node/k8s metrics│
│                                 │                                 │                         │
└─────────────────────────────────┴───────────────────────────────┴────────────────────────┘
```

---

## Links

### Prometheus
- [Prometheus Documentation](https://prometheus.io/docs/)
- [Prometheus Best Practices](https://prometheus.io/docs/practices/)
- [Prometheus Recording Rules](https://prometheus.io/docs/prometheus/latest/recording_rules/)
- [Alertmanager Documentation](https://prometheus.io/docs/alerting/latest/alertmanager/)

### Grafana
- [Grafana Documentation](https://grafana.com/docs/)
- [Grafana Dashboards](https://grafana.com/grafana/dashboards)
- [Grafana Loki](https://grafana.com/oss/loki/)
- [Grafana Tempo](https://grafana.com/oss/tempo/)

### SLI/SLO
- [Google SRE Book - SLIs](https://sre.google/sre-book/part-III/part3-chapter-11/)
- [Site Reliability Engineering](https://sre.google/sre-book/table-of-contents/)
- [SLO Certification](https://www.oreilly.com/live-events/slo-based-engineering-c/)

### OpenTelemetry
- [OpenTelemetry Documentation](https://opentelemetry.io/docs/)
- [Collector Documentation](https://opentelemetry.io/docs/collector/)
- [Specification](https://opentelemetry.io/docs/specs/otel/)

### Observability
- [Observability Engineering](https://www.oreilly.com/library/view/observability-engineering/9781492076438/)
- [Honeycomb Observability](https://www.honeycomb.io/)
- [Lightstep](https://lightstep.com/)

### APM Tools
- [Datadog APM](https://www.datadoghq.com/apm/)
- [New Relic](https://newrelic.com/)
- [AWS X-Ray](https://aws.amazon.com/xray/)
- [Jaeger](https://www.jaegertracing.io/)

### Service Level Objectives
- [Definitive SLO Guide](https://sre.google/resources/practices-and-processes/building-slos/)
- [Error Budget Calculator](https://error-budget-calculator.com/)
- [SLO Generator](https://github.com/Nike-Inc/gimme-slo)

---

## 8. Additional Reference Materials

### 8.1 Common Metric Patterns

```typescript
// metrics/common-patterns.ts - Common metric patterns

// Counter pattern for things that only increase
const requestCounter = new Counter({
  name: 'http_requests_total',
  help: 'Total HTTP requests',
  labelNames: ['method', 'endpoint', 'status_code'],
});

// Gauge pattern for things that go up and down
const currentConnections = new Gauge({
  name: 'active_connections',
  help: 'Number of active connections',
  labelNames: ['service'],
});

// Histogram pattern for distributions
const requestDuration = new Histogram({
  name: 'http_request_duration_seconds',
  help: 'HTTP request duration',
  labelNames: ['method', 'endpoint'],
  buckets: [0.01, 0.05, 0.1, 0.25, 0.5, 1, 2.5, 5, 10],
});

// Summary pattern for pre-computed percentiles
const responseSize = new Summary({
  name: 'http_response_size_bytes',
  help: 'HTTP response size in bytes',
  labelNames: ['method', 'endpoint'],
  percentiles: [0.25, 0.5, 0.75, 0.95, 0.99],
});

// Best practices:
// 1. Use counters for things that only increase
// 2. Use gauges for things that fluctuate
// 3. Use histograms for latency/response size
// 4. Avoid high-cardinality labels
// 5. Normalize path parameters

// Bad: path="/user/123456" (high cardinality)
// Good: path="/user/:id" (low cardinality)

// Example: Correct path normalization
function normalizePath(path: string): string {
  return path
    .replace(/\/user\/\d+/, '/user/:id')
    .replace(/\/order\/\d+/, '/order/:id')
    .replace(/\/product\/\d+/, '/product/:id');
}

// Example: Timing wrapper
async function withMetrics<T>(
  operation: () => Promise<T>,
  labels: Record<string, string>
): Promise<T> {
  const start = Date.now();
  try {
    return await operation();
  } finally {
    const duration = (Date.now() - start) / 1000;
    requestDuration.observe(labels, duration);
  }
}
```

### 8.2 Alert Response Playbooks

```yaml
# Runbook: High Error Rate Alert
# Severity: P2 - High
# Response Time: < 30 minutes

## Symptoms
- Error rate > 1% for 5+ minutes
- HTTP 5xx responses increasing
- User-facing errors reported

## Investigation Steps
1. Check service health
   - Review pod logs: kubectl logs -n production -l app=api --tail=100
   - Check pod status: kubectl get pods -n production -l app=api
   - Review recent deployments

2. Check dependencies
   - Database connectivity
   - Cache availability
   - External API status

3. Check metrics
   - Identify which endpoints are failing
   - Check error types
   - Compare to baseline

## Resolution Steps
1. If deployment-related: Rollback last deployment
   ```
   kubectl rollout undo deployment/api -n production
   ```

2. If database-related:
   - Check connection pool
   - Review slow queries
   - Consider scaling

3. If external dependency:
   - Enable circuit breaker
   - Fall back to cached data

## Post-Incident
- Update monitoring if new error pattern discovered
- Add new alert if needed
- Document in incident report

---

```yaml
# Runbook: High Latency Alert
# Severity: P2 - High
# Response Time: < 30 minutes

## Symptoms
- P99 latency > 1s for 5+ minutes
- P95 latency increasing
- User complaints of slow responses

## Investigation Steps
1. Identify slow endpoints
   - Check which paths are slow
   - Compare to baseline latency

2. Check resource utilization
   - CPU usage: kubectl top pods
   - Memory: check for OOM events
   - Network: check for saturation

3. Check database
   - Slow query log
   - Connection pool
   - Replication lag

4. Check external services
   - Third-party API latency
   - CDN performance

## Resolution Steps
1. If resource-constrained:
   - Scale horizontally: kubectl scale deployment/api --replicas=10
   - Check resource limits

2. If database-related:
   - Identify slow queries
   - Add indexes
   - Consider read replicas

3. If code-related:
   - Enable caching
   - Optimize queries
   - Deploy fix

## Post-Incident
- Add to performance test suite
- Schedule optimization work
- Update SLIs if needed
```

### 8.3 Custom Exporter Example

```typescript
// metrics/custom-exporter.ts - Example custom Prometheus exporter

import { Registry, Gauge, Counter, collectDefaultMetrics } from 'prom-client';

class CustomExporter {
  private registry: Registry;
  private httpRequests: Counter;
  private queueDepth: Gauge;
  private processingTime: Summary;
  
  constructor() {
    this.registry = new Registry();
    
    // Collect default metrics (CPU, memory, etc)
    collectDefaultMetrics({ register: this.registry });
    
    // Custom metrics
    this.httpRequests = new Counter({
      name: 'myapp_http_requests_total',
      help: 'Total HTTP requests',
      labelNames: ['method', 'path', 'status'],
      registers: [this.registry],
    });
    
    this.queueDepth = new Gauge({
      name: 'myapp_queue_depth',
      help: 'Current queue depth',
      labelNames: ['queue_name'],
      registers: [this.registry],
    });
    
    this.processingTime = new Summary({
      name: 'myapp_processing_seconds',
      help: 'Processing time in seconds',
      labelNames: ['operation'],
      percentiles: [0.5, 0.9, 0.99],
      registers: [this.registry],
    });
    
    // Start collecting queue metrics
    this.startQueueMetrics();
  }
  
  private startQueueMetrics(): void {
    setInterval(() => {
      const queues = ['orders', 'notifications', 'emails'];
      
      for (const queue of queues) {
        const depth = this.getQueueDepth(queue); // Implement actual collection
        this.queueDepth.set({ queue_name: queue }, depth);
      }
    }, 10000);
  }
  
  recordHttpRequest(method: string, path: string, status: number): void {
    this.httpRequests.inc({ method, path, status });
  }
  
  recordProcessingTime(operation: string, durationMs: number): void {
    this.processingTime.observe({ operation }, durationMs / 1000);
  }
  
  async getMetrics(): Promise<string> {
    return this.registry.metrics();
  }
  
  getContentType(): string {
    return this.registry.contentType;
  }
}
```

### 8.4 Distributed Tracing Integration

```typescript
// metrics/distributed-tracing.ts - OpenTelemetry integration

import { NodeSDK } from '@opentelemetry/sdk-node';
import { Resource } from '@opentelemetry/resources';
import { SemanticResourceAttributes } from '@opentelemetry/semantic-conventions';
import { JaegerExporter } from '@opentelemetry/exporter-jaeger';
import { ZipkinExporter } from '@opentelemetry/exporter-zipkin';
import { getNodeAutoInstrumentations } from '@opentelemetry/auto-instrumentations-node';
import { PrometheusExporter } from '@opentelemetry/exporter-prometheus';

const sdk = new NodeSDK({
  resource: new Resource({
    [SemanticResourceAttributes.SERVICE_NAME]: 'my-service',
    [SemanticResourceAttributes.SERVICE_VERSION]: process.env.VERSION || '1.0.0',
    [SemanticResourceAttributes.DEPLOYMENT_ENVIRONMENT]: process.env.ENV || 'development',
  }),
  
  // Trace exporter (Jaeger/Zipkin)
  traceExporter: new JaegerExporter({
    endpoint: process.env.JAEGER_ENDPOINT || 'http://localhost:14268/api/traces',
  }),
  
  // Metrics exporter (Prometheus)
  metricExporter: new PrometheusExporter({
    port: 9464,
    startMetricServer: true,
  }),
  
  // Auto-instrumentation
  instrumentations: [
    getNodeAutoInstrumentations({
      '@opentelemetry/instrumentation-fs': { enabled: false },
    }),
  ],
});

sdk.start();

// Graceful shutdown
process.on('SIGTERM', () => {
  sdk.shutdown()
    .then(() => console.log('SDK shut down successfully'))
    .catch((error) => console.log('Error shutting down SDK', error))
    .finally(() => process.exit(0));
});
```

### 8.5 Log Correlation

```yaml
# Structured logging with correlation

# Configuration for correlated logging
logging:
  format: json
  level: info
  
  correlation:
    enabled: true
    header: X-Request-ID
    generate_if_missing: true
    
  fields:
    - timestamp
    - level
    - message
    - request_id
    - trace_id
    - span_id
    - user_id
    - service
    - version
    - environment

# Example log entry:
# {
#   "timestamp": "2024-01-15T10:30:00.000Z",
#   "level": "info",
#   "message": "Request completed",
#   "request_id": "abc123",
#   "trace_id": "xyz789",
#   "span_id": "span456",
#   "user_id": "user123",
#   "service": "api",
#   "version": "1.0.0",
#   "environment": "production",
#   "duration_ms": 245,
#   "status_code": 200
# }

# Integration with metrics:
# - Log entry includes trace_id
# - Trace includes request_id
# - Metrics include request_id label for correlation
# - Enables drill-down from metric to log to trace
```

### 8.6 Grafana Dashboard Variables

```json
{
  "templating": {
    "list": [
      {
        "name": "service",
        "type": "query",
        "query": "label_values(http_requests_total, service)",
        "multi": true,
        "allValue": ".*"
      },
      {
        "name": "environment",
        "type": "query", 
        "query": "label_values(http_requests_total, env)",
        "multi": true,
        "includeAll": true
      },
      {
        "name": "alertname",
        "type": "query",
        "query": "label_values(ALERTS{alertstate=\"firing\"}, alertname)",
        "multi": true,
        "allValue": ".*"
      }
    ]
  }
}
```

---

## Links

### Prometheus
- [Prometheus Documentation](https://prometheus.io/docs/)
- [Prometheus Best Practices](https://prometheus.io/docs/practices/)
- [Prometheus Recording Rules](https://prometheus.io/docs/prometheus/latest/recording_rules/)
- [Alertmanager Documentation](https://prometheus.io/docs/alerting/latest/alertmanager/)

### Grafana
- [Grafana Documentation](https://grafana.com/docs/)
- [Grafana Dashboards](https://grafana.com/grafana/dashboards)
- [Grafana Loki](https://grafana.com/oss/loki/)
- [Grafana Tempo](https://grafana.com/oss/tempo/)

### SLI/SLO
- [Google SRE Book - SLIs](https://sre.google/sre-book/part-III/part3-chapter-11/)
- [Site Reliability Engineering](https://sre.google/sre-book/table-of-contents/)
- [SLO Certification](https://www.oreilly.com/live-events/slo-based-engineering-c/)

### OpenTelemetry
- [OpenTelemetry Documentation](https://opentelemetry.io/docs/)
- [Collector Documentation](https://opentelemetry.io/docs/collector/)
- [Specification](https://opentelemetry.io/docs/specs/otel/)

### Observability
- [Observability Engineering](https://www.oreilly.com/library/view/observability-engineering/9781492076438/)
- [Honeycomb Observability](https://www.honeycomb.io/)
- [Lightstep](https://lightstep.com/)

### APM Tools
- [Datadog APM](https://www.datadoghq.com/apm/)
- [New Relic](https://newrelic.com/)
- [AWS X-Ray](https://aws.amazon.com/xray/)
- [Jaeger](https://www.jaegertracing.io/)

### Service Level Objectives
- [Definitive SLO Guide](https://sre.google/resources/practices-and-processes/building-slos/)
- [Error Budget Calculator](https://error-budget-calculator.com/)
- [SLO Generator](https://github.com/Nike-Inc/gimme-slo)