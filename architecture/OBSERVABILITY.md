# OBSERVABILITY.md - Observability Architecture (DENSE)

**Authority:** guidance (observability patterns, structured logging, and audit discipline)
**Layer:** Guides
**Binding:** No
**Scope:** logging, metrics, tracing, event sourcing, mechanical verification
**Non-goals:** specific monitoring tool configuration, alerting thresholds

---

## 1. Observability Principles

### 1.1 The Three Pillars

| Pillar | Purpose | Use For | Storage | Retention |
|--------|---------|---------|---------|-----------|
| **Metrics** | Aggregate numerical data | Dashboards, alerting, capacity planning | TSDB (Prometheus, InfluxDB) | 13+ months |
| **Logs** | Discrete events with context | Debugging, audit trails, forensics | Log aggregator (Loki, ELK) | 30-90 days hot, 1+ year cold |
| **Traces** | Request flow across services | Understanding latency, dependencies | Distributed tracing (Jaeger, Tempo) | 7-30 days |

### 1.2 Core Mandates

- **Structured logging is required; string parsing is prohibited.** Every log entry must be machine-parseable (JSON, key-value pairs, or structured format).
- **Alert on symptoms, not causes.** Users experience symptoms (latency, errors); investigate causes after alerting.
- **Sampling is acceptable for high-volume data.** 100% capture at low volume, statistical sampling at high volume.
- **Cost of observability < cost of not observing.** If you can't see it, you can't fix it.

### 1.3 Production Mindset
Observability is not a feature bolted on after the system is built — it is the primary mechanism by which a system proves it is operating correctly:

- **SLIs and SLOs are the engineering-business contract:** Service Level Indicators define what "working" means in measurable terms. SLOs define the acceptable threshold. When within error budget, ship features. When outside it, fix reliability. This is not optional and does not require negotiation.
- **Mean Time to Detection must approach zero:** The goal of observability is to know about a failure before the customer does. If the customer reports the issue first, the observability layer has already failed its primary function.
- **Telemetry must be correlated:** Metrics, logs, and traces in isolation are incomplete. A single trace ID must link a user-visible request to a specific log line and a spike in a latency histogram. Siloed observability is expensive noise.
- **Semantic logging, not mechanical logging:** Logs are data, not strings. A log entry should capture the intent and outcome of an operation, not just a sequential chronicle of function calls. Log what happened and why it matters, with machine-parseable fields.
- **Distributed tracing is mandatory in concurrent systems:** When a request touches multiple async components or services, debugging without a trace is guesswork. Instrument trace propagation at service boundaries from the start — it cannot be added cheaply after the fact.
- **Instrumentation is production code:** Observability code must be tested, reviewed, and maintained at the same standard as business logic. A silent failure caused by missing or broken instrumentation is a critical defect.
- **High-volume logs are noise:** Logging every function call or intermediate state is log pollution. It increases cost, slows queries, and buries real signals. Log at the appropriate level; sample traces aggressively at high volume.
- **The audit trail is the system of record:** In Decapod, observability is the mechanism by which completion is proved. An operation that is not in the audit log did not happen as far as the system is concerned.

---

## 2. Structured Logging

### 2.1 Structured Log Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "StructuredLogEntry",
  "type": "object",
  "required": ["timestamp", "level", "message", "service", "trace_id", "span_id"],
  "properties": {
    "timestamp": {
      "type": "string",
      "format": "date-time",
      "description": "ISO8601 UTC timestamp"
    },
    "level": {
      "type": "string",
      "enum": ["trace", "debug", "info", "warn", "error", "fatal"],
      "description": "Log severity level"
    },
    "message": {
      "type": "string",
      "description": "Human-readable summary of the event"
    },
    "service": {
      "type": "string",
      "description": "Service name that generated this log"
    },
    "version": {
      "type": "string",
      "description": "Service version (semver)"
    },
    "environment": {
      "type": "string",
      "enum": ["development", "staging", "production"]
    },
    "trace_id": {
      "type": "string",
      "description": "Distributed trace ID for correlation"
    },
    "span_id": {
      "type": "string",
      "description": "Span ID within the trace"
    },
    "parent_span_id": {
      "type": "string",
      "description": "Parent span ID for trace hierarchy"
    },
    "user_id": {
      "type": ["string", "null"],
      "description": "Authenticated user ID if available"
    },
    "session_id": {
      "type": ["string", "null"],
      "description": "Session ID for user correlation"
    },
    "request_id": {
      "type": ["string", "null"],
      "description": "HTTP request ID"
    },
    "duration_ms": {
      "type": ["number", "null"],
      "description": "Duration of the operation in milliseconds"
    },
    "component": {
      "type": "string",
      "description": "Component or module name"
    },
    "operation": {
      "type": "string",
      "description": "Operation name"
    },
    "status": {
      "type": "string",
      "enum": ["success", "failure", "partial"]
    },
    "error": {
      "type": ["object", "null"],
      "properties": {
        "type": {
          "type": "string",
          "description": "Error class or type"
        },
        "code": {
          "type": "string",
          "description": "Application error code"
        },
        "message": {
          "type": "string",
          "description": "Error message (no secrets)"
        },
        "stack_trace": {
          "type": "string",
          "description": "Stack trace (no secrets, only in debug/staging)"
        },
        "cause": {
          "type": "string",
          "description": "Root cause error type"
        }
      }
    },
    "metadata": {
      "type": "object",
      "description": "Additional structured context",
      "additionalProperties": true
    }
  }
}
```

**Log Entry Examples:**

```json
// Info log - successful operation
{
  "timestamp": "2026-05-16T10:30:00.123Z",
  "level": "info",
  "message": "User profile updated successfully",
  "service": "user-service",
  "version": "2.4.1",
  "environment": "production",
  "trace_id": "4bf92f3577b34da6a3ce929d0e0e4736",
  "span_id": "00f067aa0ba902b7",
  "user_id": "usr_abc123",
  "component": "user-profile",
  "operation": "update_profile",
  "status": "success",
  "duration_ms": 45,
  "metadata": {
    "fields_updated": ["display_name", "avatar_url"],
    "ip_address": "203.0.113.42",
    "user_agent": "Mozilla/5.0..."
  }
}

// Error log - failed operation
{
  "timestamp": "2026-05-16T10:30:05.456Z",
  "level": "error",
  "message": "Database query failed after retry exhaustion",
  "service": "user-service",
  "version": "2.4.1",
  "environment": "production",
  "trace_id": "4bf92f3577b34da6a3ce929d0e0e4736",
  "span_id": "00f067aa0ba902b8",
  "user_id": "usr_abc123",
  "component": "user-profile",
  "operation": "fetch_user_details",
  "status": "failure",
  "duration_ms": 5234,
  "error": {
    "type": "DatabaseConnectionError",
    "code": "DB_CONN_002",
    "message": "Connection refused after 3 retries",
    "cause": "TimeoutException"
  },
  "metadata": {
    "database_host": "db.example.com",
    "query_timeout_ms": 5000,
    "attempt": 3
  }
}

// Warn log - degraded operation
{
  "timestamp": "2026-05-16T10:30:10.789Z",
  "level": "warn",
  "message": "Cache miss rate exceeded threshold",
  "service": "api-gateway",
  "version": "1.8.3",
  "environment": "production",
  "trace_id": null,
  "span_id": "a1b2c3d4e5f6",
  "component": "cache-layer",
  "operation": "redis_get",
  "status": "success",
  "duration_ms": 12,
  "metadata": {
    "hit_rate_pct": 72,
    "threshold_pct": 85,
    "cache_host": "redis-cluster.example.com",
    "consecutive_misses": 5
  }
}
```

### 2.2 Log Level Guidelines

| Level | When to Use | Example Scenarios |
|-------|-------------|-------------------|
| **TRACE** | Detailed debugging, enter/exit functions | Function entry, loop iterations, variable dumps |
| **DEBUG** | Debug information for troubleshooting | Business logic steps, decision points |
| **INFO** | Normal operations, significant events | Service startup, user actions, transactions |
| **WARN** | Unexpected but handled situation | Degraded performance, retry attempts, fallbacks |
| **ERROR** | Operation failed, needs attention | Exceptions, failed validation, timeout |
| **FATAL** | Service cannot continue | Out of memory, cannot bind port, corruption |

### 2.3 Anti-Patterns

```
// WRONG: Unstructured string logging
log.Printf("User %s failed to login after %d attempts from IP %s", userId, attempts, ip)

// WRONG: Logging secrets
log.Printf("API call with key: %s", apiKey)

// RIGHT: Structured fields
log.Info("Login failed",
    "user_id", userId,
    "attempts", attempts,
    "ip_address", ip,
    "reason", "invalid_credentials",
)

// RIGHT: Using structured logger (Go example)
logger.Info("Login failed",
    zap.String("user_id", userId),
    zap.Int("attempts", attempts),
    zap.String("ip_address", ip),
    zap.String("reason", "invalid_credentials"),
)
```

---

## 3. Event Sourcing for Audit

### 3.1 Event Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "AuditEvent",
  "type": "object",
  "required": ["event_id", "event_type", "timestamp", "actor", "entity", "operation", "status"],
  "properties": {
    "event_id": {
      "type": "string",
      "pattern": "^[a-z0-9]{16,}$",
      "description": "Unique event identifier (ULID recommended)"
    },
    "event_type": {
      "type": "string",
      "pattern": "^[a-z][a-z0-9_]*\\.[a-z][a-z0-9_]*$",
      "description": "Dot-separated event category (e.g., user.created, order.completed)"
    },
    "event_version": {
      "type": "integer",
      "minimum": 1,
      "default": 1
    },
    "timestamp": {
      "type": "string",
      "format": "date-time",
      "description": "When the event occurred"
    },
    "actor": {
      "type": "object",
      "required": ["type", "id"],
      "properties": {
        "type": {
          "type": "string",
          "enum": ["user", "service", "system", "agent", "scheduler"]
        },
        "id": {"type": "string"},
        "name": {"type": "string"},
        "ip_address": {"type": "string"},
        "user_agent": {"type": "string"}
      }
    },
    "entity": {
      "type": "object",
      "required": ["type", "id"],
      "properties": {
        "type": {"type": "string"},
        "id": {"type": "string"},
        "name": {"type": "string"}
      }
    },
    "operation": {
      "type": "string",
      "enum": ["create", "read", "update", "delete", "execute", "approve", "reject", "retry", "rollback"]
    },
    "status": {
      "type": "string",
      "enum": ["success", "failure", "pending"]
    },
    "intent_ref": {
      "type": ["string", "null"],
      "description": "Reference to the original intent that caused this event"
    },
    "correlation_id": {
      "type": ["string", "null"],
      "description": "ID linking related events"
    },
    "changes": {
      "type": "object",
      "description": "Before/after state for state-changing operations",
      "properties": {
        "before": {
          "type": "object",
          "description": "State before the operation"
        },
        "after": {
          "type": "object",
          "description": "State after the operation"
        },
        "diff": {
          "type": "array",
          "description": "List of changed fields",
          "items": {
            "type": "object",
            "properties": {
              "field": {"type": "string"},
              "old": {},
              "new": {}
            }
          }
        }
      }
    },
    "metadata": {
      "type": "object",
      "additionalProperties": true
    },
    "proof": {
      "type": "object",
      "properties": {
        "validation_hash": {"type": "string"},
        "validated_by": {"type": "string"},
        "validation_timestamp": {"type": "string"}
      }
    }
  }
}
```

### 3.2 Event Examples

```json
{
  "event": {
    "event_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
    "event_type": "todo.status.changed",
    "event_version": 1,
    "timestamp": "2026-05-16T10:30:00.000Z",
    "actor": {
      "type": "agent",
      "id": "agent-claude",
      "name": "Claude Code",
      "ip_address": "10.50.1.100"
    },
    "entity": {
      "type": "todo",
      "id": "todo_01ARZ3NDEKTSV4RRFFQ69G5FAV",
      "name": "Expand architecture docs"
    },
    "operation": "update",
    "status": "success",
    "intent_ref": "intent_01ARZ3NDEKTSV4RRFFQ69G5FA0",
    "correlation_id": "corr_01ARZ3NDEKTSV4RRFFQ69G5FA1",
    "changes": {
      "before": {
        "status": "pending",
        "updated_at": "2026-05-16T09:00:00.000Z"
      },
      "after": {
        "status": "active",
        "updated_at": "2026-05-16T10:30:00.000Z"
      },
      "diff": [
        {"field": "status", "old": "pending", "new": "active"},
        {"field": "updated_at", "old": "2026-05-16T09:00:00.000Z", "new": "2026-05-16T10:30:00.000Z"}
      ]
    },
    "metadata": {
      "transition_reason": "Starting implementation",
      "workspace_id": "ws_01ARZ3NDEKTSV4RRFFQ69G5FA2"
    }
  }
}
```

```json
{
  "event": {
    "event_id": "01ARZ3NDEKTSV4RRFFQ69G5FBV",
    "event_type": "validation.passed",
    "event_version": 1,
    "timestamp": "2026-05-16T10:31:00.000Z",
    "actor": {
      "type": "system",
      "id": "decapod-validate",
      "name": "Decapod Validation System"
    },
    "entity": {
      "type": "change",
      "id": "change_01ARZ3NDEKTSV4RRFFQ69G5FA3",
      "name": "Expand CLOUD.md to dense knowledge base"
    },
    "operation": "execute",
    "status": "success",
    "metadata": {
      "validation_checks": 12,
      "checks_passed": 12,
      "checks_failed": 0,
      "duration_ms": 234,
      "gates": [
        {"name": "store_integrity", "status": "pass"},
        {"name": "health_purity", "status": "pass"},
        {"name": "namespace_hygiene", "status": "pass"}
      ]
    }
  }
}
```

---

## 4. State Transition History

### 4.1 Transition History Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "StateTransition",
  "type": "object",
  "required": ["entity_type", "entity_id", "from_state", "to_state", "timestamp", "actor"],
  "properties": {
    "entity_type": {
      "type": "string",
      "description": "Type of entity (todo, claim, task, etc.)"
    },
    "entity_id": {
      "type": "string",
      "description": "Unique ID of the entity"
    },
    "from_state": {
      "type": ["string", "null"],
      "description": "Previous state (null for creation)"
    },
    "to_state": {
      "type": "string",
      "description": "New state"
    },
    "timestamp": {
      "type": "string",
      "format": "date-time"
    },
    "actor": {
      "type": "object",
      "required": ["type", "id"],
      "properties": {
        "type": {
          "type": "string",
          "enum": ["user", "service", "system", "agent", "scheduler"]
        },
        "id": {"type": "string"},
        "name": {"type": "string"}
      }
    },
    "reason": {
      "type": "string",
      "minLength": 1,
      "maxLength": 500,
      "description": "Mandatory reason for the transition"
    },
    "metadata": {
      "type": "object",
      "additionalProperties": true
    }
  }
}
```

**Transition History Example:**

```json
{
  "entity_type": "todo",
  "entity_id": "todo_01ARZ3NDEKTSV4RRFFQ69G5FAV",
  "from_state": null,
  "to_state": "pending",
  "timestamp": "2026-05-16T08:00:00.000Z",
  "actor": {"type": "user", "id": "usr_abc123"},
  "reason": "Created new task via API",
  "metadata": {"source": "api", "priority": "normal"}
}
{
  "entity_type": "todo",
  "entity_id": "todo_01ARZ3NDEKTSV4RRFFQ69G5FAV",
  "from_state": "pending",
  "to_state": "active",
  "timestamp": "2026-05-16T09:00:00.000Z",
  "actor": {"type": "agent", "id": "agent-claude"},
  "reason": "Starting implementation of feature X",
  "metadata": {"workspace_id": "ws_01ARZ3NDEKTSV4RRFFQ69G5FA2"}
}
{
  "entity_type": "todo",
  "entity_id": "todo_01ARZ3NDEKTSV4RRFFQ69G5FAV",
  "from_state": "active",
  "to_state": "pending",
  "timestamp": "2026-05-16T10:00:00.000Z",
  "actor": {"type": "agent", "id": "agent-claude"},
  "reason": "Blocked on dependency task",
  "metadata": {"blocked_by": ["todo_01ARZ3NDEKTSV4RRFFQ69G5FA1"]}
}
{
  "entity_type": "todo",
  "entity_id": "todo_01ARZ3NDEKTSV4RRFFQ69G5FAV",
  "from_state": "pending",
  "to_state": "active",
  "timestamp": "2026-05-16T11:00:00.000Z",
  "actor": {"type": "agent", "id": "agent-claude"},
  "reason": "Dependency completed, resuming work",
  "metadata": {}
}
```

---

## 5. Metrics and SLIs

### 5.1 Service Level Indicator Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "SLIConfiguration",
  "type": "object",
  "required": ["service", "indicators"],
  "properties": {
    "service": {
      "type": "string"
    },
    "indicators": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["name", "type", "query", "slo_target", "window"],
        "properties": {
          "name": {
            "type": "string",
            "description": "SLI name (e.g., 'availability', 'latency')"
          },
          "type": {
            "type": "string",
            "enum": ["availability", "latency", "throughput", "quality", "correctness"]
          },
          "query": {
            "type": "string",
            "description": "PromQL/MetricQL query defining the SLI"
          },
          "slo_target": {
            "type": "number",
            "minimum": 0,
            "maximum": 100,
            "description": "Target percentage (e.g., 99.9)"
          },
          "window": {
            "type": "string",
            "description": "Rolling window (e.g., '30d', '7d')"
          },
          "alert_threshold": {
            "type": "number",
            "description": "Alert when SLI drops below this"
          },
          "burn_rate_threshold": {
            "type": "number",
            "description": "Alert on fast burn rate"
          },
          "description": {
            "type": "string"
          }
        }
      }
    }
  }
}
```

**SLI Configuration Example:**

```yaml
service: api-gateway
indicators:
  - name: request_success_rate
    type: availability
    description: "Percentage of successful HTTP requests"
    query: |
      1 - (
        sum(rate(http_requests_total{status=~"5.."}[5m]))
        /
        sum(rate(http_requests_total[5m]))
      )
    slo_target: 99.9
    window: 30d
    alert_threshold: 99.5
    
  - name: p99_request_duration
    type: latency
    description: "99th percentile request duration"
    query: |
      histogram_quantile(0.99,
        sum(rate(http_request_duration_seconds_bucket[5m])) by (le)
      )
    slo_target: 99.0
    window: 30d
    alert_threshold: 0.5
    burn_rate_threshold: 14
    
  - name: error_budget_remaining
    type: availability
    description: "Remaining error budget percentage"
    query: |
      1 - (
        sum(increase(http_requests_total{status=~"5.."}[30d]))
        /
        sum(increase(http_requests_total[30d]))
      ) > 0.999
    slo_target: 99.9
    window: 30d
```

### 5.2 Four Golden Signals

```yaml
GoldenSignals:
  latency:
    metrics:
      - name: request_duration_seconds
        type: histogram
        labels: [service, route, method, status_code]
        buckets: [0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1, 2.5, 5, 10]
      
      - name: request_duration_seconds_summary
        type: summary
        labels: [service, route, method]
        quantiles: [0.5, 0.9, 0.95, 0.99]
      
    sli:
      - name: p99_latency
        query: histogram_quantile(0.99, sum(rate(request_duration_seconds_bucket[5m])) by (le))
        target: 500ms
        window: 5m
        
  traffic:
    metrics:
      - name: requests_total
        type: counter
        labels: [service, route, method, status_code]
        
      - name: requests_per_second
        type: gauge
        labels: [service]
        
    sli:
      - name: request_rate
        query: sum(rate(requests_total[5m]))
        target: ">0"
        
  errors:
    metrics:
      - name: errors_total
        type: counter
        labels: [service, route, method, status_code, error_type]
        
      - name: error_rate
        type: gauge
        labels: [service]
        
    sli:
      - name: error_rate
        query: sum(rate(errors_total[5m])) / sum(rate(requests_total[5m]))
        target: "<0.001"
        
    error_budget:
      target: 0.1%
      window: 30d
      
  saturation:
    metrics:
      - name: cpu_usage_percent
        type: gauge
        labels: [service, instance]
        
      - name: memory_usage_bytes
        type: gauge
        labels: [service, instance]
        
      - name: queue_depth
        type: gauge
        labels: [service, queue_name]
        
      - name: connection_pool_usage
        type: gauge
        labels: [service, pool_name]
        buckets: [0, 0.5, 0.7, 0.8, 0.9, 0.95, 1]
        
    sli:
      - name: cpu_saturation
        query: cpu_usage_percent > 80
        target: "<80%"
        
      - name: queue_depth
        query: queue_depth / max_queue_depth
        target: "<0.9"
```

---

## 6. Distributed Tracing

### 6.1 Trace Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "DistributedTrace",
  "type": "object",
  "required": ["trace_id", "spans"],
  "properties": {
    "trace_id": {
      "type": "string",
      "pattern": "^[a-f0-9]{16,32}$",
      "description": "Unique trace identifier"
    },
    "spans": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["span_id", "parent_span_id", "trace_id", "service", "operation", "start_time", "end_time"],
        "properties": {
          "span_id": {
            "type": "string",
            "pattern": "^[a-f0-9]{16}$"
          },
          "parent_span_id": {
            "type": ["string", "null"],
            "pattern": "^[a-f0-9]{16}$"
          },
          "trace_id": {
            "type": "string"
          },
          "service": {
            "type": "string",
            "description": "Service name that created this span"
          },
          "version": {
            "type": "string",
            "description": "Service version"
          },
          "operation": {
            "type": "string",
            "description": "Operation name (e.g., HTTP method + route)"
          },
          "start_time": {
            "type": "string",
            "format": "date-time"
          },
          "end_time": {
            "type": "string",
            "format": "date-time"
          },
          "duration_ms": {
            "type": "number"
          },
          "span_kind": {
            "type": "string",
            "enum": ["server", "client", "producer", "consumer", "internal"]
          },
          "status": {
            "type": "object",
            "properties": {
              "code": {
                "type": "integer",
                "enum": [0, 1, 2],
                "description": "0=OK, 1=ERROR, 2=UNKNOWN"
              },
              "message": {"type": "string"}
            }
          },
          "tags": {
            "type": "object",
            "additionalProperties": {
              "type": ["string", "number", "boolean"]
            }
          },
          "logs": {
            "type": "array",
            "items": {
              "type": "object",
              "properties": {
                "timestamp": {"type": "string"},
                "fields": {
                  "type": "object",
                  "additionalProperties": {}
                }
              }
            }
          }
        }
      }
    }
  }
}
```

**Trace Example (Jaeger format):**

```json
{
  "trace_id": "4bf92f3577b34da6a3ce929d0e0e4736",
  "spans": [
    {
      "span_id": "00f067aa0ba902b7",
      "parent_span_id": null,
      "trace_id": "4bf92f3577b34da6a3ce929d0e0e4736",
      "service": "api-gateway",
      "version": "1.8.3",
      "operation": "POST /api/v1/users",
      "start_time": "2026-05-16T10:30:00.000Z",
      "end_time": "2026-05-16T10:30:00.234Z",
      "duration_ms": 234,
      "span_kind": "server",
      "status": {"code": 0, "message": "OK"},
      "tags": {
        "http.method": "POST",
        "http.route": "/api/v1/users",
        "http.status_code": 201,
        "http.host": "api.example.com",
        "http.user_agent": "Mozilla/5.0...",
        "net.peer.ip": "203.0.113.42"
      },
      "logs": []
    },
    {
      "span_id": "4bf92f3577b34da6",
      "parent_span_id": "00f067aa0ba902b7",
      "trace_id": "4bf92f3577b34da6a3ce929d0e0e4736",
      "service": "user-service",
      "version": "2.4.1",
      "operation": "createUser",
      "start_time": "2026-05-16T10:30:00.050Z",
      "end_time": "2026-05-16T10:30:00.180Z",
      "duration_ms": 130,
      "span_kind": "client",
      "status": {"code": 0},
      "tags": {
        "db.system": "postgresql",
        "db.statement": "INSERT INTO users ...",
        "db.operation": "INSERT",
        "db.sql_table": "users"
      },
      "logs": []
    },
    {
      "span_id": "a3ce929d0e0e4736",
      "parent_span_id": "00f067aa0ba902b7",
      "trace_id": "4bf92f3577b34da6a3ce929d0e0e4736",
      "service": "notification-service",
      "version": "1.2.0",
      "operation": "sendWelcomeEmail",
      "start_time": "2026-05-16T10:30:00.190Z",
      "end_time": "2026-05-16T10:30:00.220Z",
      "duration_ms": 30,
      "span_kind": "client",
      "status": {"code": 0},
      "tags": {
        "messaging.system": "sns",
        "messaging.destination": "welcome-email-queue",
        "messaging.operation": "publish"
      },
      "logs": []
    }
  ]
}
```

---

## 7. Mechanical Verification

### 7.1 Grep-Based Checks

```bash
#!/bin/bash
# observability_checks.sh - Automated observability verification

set -e

echo "Running observability mechanical checks..."

# Check 1: No panics in production code
echo "Checking for unwrap/expect in code..."
if grep -rnE '\.unwrap\(|\.expect\(' src/ --include='*.rs' 2>/dev/null; then
    echo "ERROR: Found unwrap/expect in source code"
    exit 1
fi

# Check 2: No secrets in source
echo "Checking for hardcoded secrets..."
SECRET_PATTERNS="(sk-|AKIA|ghp_|xox[baprs]-|AIza|password\s*=\s*['\"]|-----BEGIN.*PRIVATE KEY-----)"
if grep -rnE "$SECRET_PATTERNS" src/ --include='*.rs' --include='*.py' --include='*.ts' 2>/dev/null; then
    echo "ERROR: Found potential secrets in source"
    exit 1
fi

# Check 3: Structured logging exists
echo "Checking for logging statements..."
if ! grep -rnE 'log\.(info|warn|error|debug)' src/ --include='*.rs' 2>/dev/null | head -5; then
    echo "WARNING: No structured logging found"
fi

# Check 4: Trace propagation exists
echo "Checking for trace propagation..."
if ! grep -rnE 'trace_id|span_id|tracing' src/ --include='*.rs' 2>/dev/null | head -5; then
    echo "WARNING: No trace propagation found"
fi

# Check 5: All state enums have transition tables
echo "Checking state enum transition tables..."
TRANSITION_TABLES=$(grep -rn 'can_transition_to\|StateTransition' src/ --include='*.rs' 2>/dev/null | wc -l)
if [ "$TRANSITION_TABLES" -eq 0 ]; then
    echo "WARNING: No state transition tables found"
fi

echo "All checks passed!"
```

### 7.2 Validation as Observability

The validation harness (`decapod validate`) is itself an observability tool. It makes invisible invariants visible:

```json
{
  "validation_gates": [
    {
      "name": "store_integrity",
      "description": "Deterministic rebuild from events produces identical state",
      "check": "replay_all_events_from_empty_should_reconstruct_current_state",
      "severity": "critical"
    },
    {
      "name": "health_purity",
      "description": "Health status is derived from proof events, not manual values",
      "check": "no_manual_health_status_values_in_store",
      "severity": "critical"
    },
    {
      "name": "namespace_hygiene",
      "description": "No legacy namespace references remain",
      "check": "all_namespace_references_are_current",
      "severity": "high"
    },
    {
      "name": "schema_determinism",
      "description": "Stable output across validation runs",
      "check": "validate_produces_deterministic_output",
      "severity": "medium"
    },
    {
      "name": "event_immutability",
      "description": "Events cannot be modified after creation",
      "check": "no_update_operations_on_events",
      "severity": "critical"
    },
    {
      "name": "audit_trail_completeness",
      "description": "All state changes have corresponding events",
      "check": "all_mutations_have_audit_events",
      "severity": "high"
    }
  ]
}
```

---

## 8. Alerting Patterns

### 8.1 Alert Configuration Schema

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "AlertConfiguration",
  "type": "object",
  "properties": {
    "alert_name": {"type": "string"},
    "severity": {
      "type": "string",
      "enum": ["critical", "high", "medium", "low", "info"]
    },
    "service": {"type": "string"},
    "description": {"type": "string"},
    "runbook_url": {"type": "string"},
    "expr": {
      "type": "string",
      "description": "PromQL query"
    },
    "for": {
      "type": "string",
      "description": "Duration condition (e.g., '5m')"
    },
    "labels": {
      "type": "object",
      "additionalProperties": {"type": "string"}
    },
    "annotations": {
      "type": "object",
      "properties": {
        "summary": {"type": "string"},
        "description": {"type": "string"},
        "runbook_url": {"type": "string"}
      }
    },
    "notifications": {
      "type": "object",
      "properties": {
        "slack_channel": {"type": "string"},
        "pagerduty_severity": {"type": "string"},
        "email": {"type": "string"}
      }
    },
    "throttling_seconds": {
      "type": "integer",
      "default": 300
    },
    "no_data_behavior": {
      "type": "string",
      "enum": ["ok", "no_data", " alerting"]
    }
  }
}
```

**Alert Examples:**

```yaml
alerts:
  - name: HighErrorRate
    severity: critical
    service: api-gateway
    description: "API error rate exceeds 1%"
    runbook_url: "https://runbooks.example.com/high-error-rate"
    expr: |
      sum(rate(http_requests_total{status=~"5.."}[5m])) 
      / 
      sum(rate(http_requests_total[5m])) > 0.01
    for: 2m
    labels:
      team: platform
      severity: p1
    annotations:
      summary: "High error rate on {{ $labels.service }}"
      description: "Error rate is {{ $value | humanizePercentage }} (threshold: 1%)"
    notifications:
      slack_channel: "#alerts-critical"
      pagerduty_severity: critical
    throttling_seconds: 300
    
  - name: SLOServiceLatencyP99
    severity: high
    service: user-service
    description: "P99 latency exceeds SLO target"
    expr: |
      histogram_quantile(0.99, 
        sum(rate(http_request_duration_seconds_bucket{service="user-service"}[5m])) by (le)
      ) > 0.5
    for: 10m
    labels:
      team: backend
    annotations:
      summary: "P99 latency is {{ $value | humanizeDuration }}"
      description: "SLO target is 500ms"
    no_data_behavior: no_data
    
  - name: ErrorBudgetExhausted
    severity: critical
    service: api-gateway
    description: "Error budget will be exhausted within 1 hour at current burn rate"
    expr: |
      (
        sum(increase(http_requests_total{status=~"5.."}[1h]))
        /
        sum(increase(http_requests_total[1h]))
      ) > 0.01
      and
      (
        sum(increase(http_requests_total{status=~"5.."}[1h]))
        /
        sum(increase(http_requests_total[1h]))
      ) > (
        1 - (0.001 * 24)
      )
    for: 5m
    labels:
      team: platform
    annotations:
      summary: "Error budget exhaustion imminent"
      description: "At the current burn rate, the error budget will be exhausted in less than 1 hour"
    notifications:
      slack_channel: "#alerts-critical"
      pagerduty_severity: critical
```

---

## 9. Anti-Patterns

| Anti-Pattern | Why It's Dangerous | Alternative |
|---|---|---|
| **Unstructured logs** | Can't query, can't alert | Structured logging with typed fields |
| **Logging secrets** | Security breach | Redact or use SecretString wrappers |
| **No event sourcing** | Can't audit, can't replay | Broker pattern for all mutations |
| **Manual health values** | Drift from reality | Derive health from proof events |
| **Alert fatigue** | Real alerts ignored | Alert on symptoms, tune thresholds |
| **No transition history** | Can't debug state issues | Record every state transition |
| **Logging too much** | Cost explosion, noise | Log at appropriate level, sample |
| **No trace propagation** | Can't correlate distributed requests | Propagate trace context at all boundaries |
| **Hardcoded alert thresholds** | Don't adapt to traffic | Use SLI-based thresholds |
| **Ignoring no-data alerts** | System silently broken | Handle no-data as alert condition |

---

## 10. Observability Stack Architecture

```yaml
# Observability Stack Architecture
ObservabilityStack:
  collection:
    metrics:
      agent: prometheus
      scrape_interval: 15s
      remote_write:
        - url: https://metrics-backend.example.com/api/v1/push
          tls:
            enabled: true
            cert: /etc/prometheus/cert.pem
            key: /etc/prometheus/key.pem
    
    logs:
      agent: vector
      sources:
        - type: file
          paths:
            - /var/log/**/*.log
          multiline: true
      transforms:
        - type: remap
          source: parse_json(message)
        - type: add_fields
          fields:
            environment: production
            datacenter: us-east-1a
      sinks:
        - type: loki
          url: https://loki.example.com
          auth:
            strategy: bearer
            token: ${LOKI_API_KEY}
    
    traces:
      collector: otel-collector
      receivers:
        - type: otlp
          endpoint: 0.0.0.0:4317
        - type: jaeger
          endpoint: 0.0.0.0:14250
      exporters:
        - type: jaeger
          endpoint: https://jaeger.example.com:14250
  
  storage:
    metrics:
      backend: thanos
      retention: 13months
      compaction:
        block_duration: 2h
        retention_resolution_raw: 30d
        retention_resolution_5m: 1y
        retention_resolution_1h: 3y
    
    logs:
      backend: loki
      retention: 30d_hot
      90d_cold
      schema: 
        object_storage: s3
        s3_bucket: logs-archive.example.com
    
    traces:
      backend: jaeger
      retention: 14d
      max_block_age: 12h
  
  visualization:
    dashboards:
      tool: grafana
      default_dashboard_folder: General
      playlist:
        - name: SLO Overview
          interval: 30s
          dashboards:
            - uid: slo-overview
            - uid: error-budget
            - uid: latency-distribution
    
  alerting:
    manager: alertmanager
    receivers:
      - name: slack
        slack_configs:
          - channel: "#alerts"
            send_resolved: true
      - name: pagerduty
        pagerduty_configs:
          - service_key: ${PAGERDUTY_KEY}
            severity: critical
      - name: email
        email_configs:
          - to: ops@example.com
            send_resolved: true
    route:
      group_by: ['alertname', 'service']
      group_wait: 30s
      group_interval: 5m
      repeat_interval: 4h
      routes:
        - match:
            severity: critical
          receiver: pagerduty
          continue: true
        - match:
            severity: high
          receiver: slack
```

---

## Links

### Core Router
- `core/DECAPOD.md` - **Router and navigation charter (START HERE)**
- `core/ENGINEERING_EXCELLENCE.md` - Engineering standards

### Architecture (This Section)
- `architecture/SECURITY.md` - Security monitoring patterns
- `architecture/CLOUD.md` - Cloud observability
- `architecture/CONCURRENCY.md` - Distributed tracing patterns
- `architecture/DATA.md` - Data observability

### Authority (Constitution Layer)
- `specs/INTENT.md` - **Methodology contract (READ FIRST)**
- `specs/SYSTEM.md` - System definition
- `specs/SECURITY.md` - Security doctrine

### Interface Contracts
- `interfaces/CLAIMS.md` - Promises ledger
- `interfaces/CONTROL_PLANE.md` - Agent sequencing
- `interfaces/STORE_MODEL.md` - Store semantics

### Methodology
- `methodology/ARCHITECTURE.md` - Architecture methodology
- `methodology/OBSERVABILITY.md` - Observability practice