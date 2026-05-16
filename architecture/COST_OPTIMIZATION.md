# COST_OPTIMIZATION.md - Cloud and Resource Cost Management (DENSE)

**Authority:** guidance (cost management)
**Layer:** Architecture
**Binding:** No
**Scope:** Cloud costs, resource allocation, and token economics

---

## 1. Cloud Cost Management

### 1.1 Cost Classification Framework

```json
{
  "CloudCostCategories": {
    "compute": {
      "subcategories": [
        {"name": "virtual_machines", "metrics": ["hours", "vCPU-hours", "GB-hours"]},
        {"name": "containers", "metrics": ["vCPU-seconds", "GB-seconds"]},
        {"name": "serverless", "metrics": ["invocations", "GB-seconds", "execution-minutes"]},
        {"name": "batch", "metrics": ["vCPU-hours", "GB-hours", "job-minutes"]}
      ],
      "optimization_strategies": [
        "right-sizing instances based on actual utilization",
        "using reserved capacity for steady-state workloads",
        "spot/preemptible instances for fault-tolerant batch workloads",
        "auto-scaling to match demand",
        "scale-to-zero for serverless when possible"
      ]
    },
    "storage": {
      "subcategories": [
        {"name": "block_storage", "metrics": ["GB-month"]},
        {"name": "object_storage", "metrics": ["GB-month", "PUT/COPY/POST requests", "GET/HEAD requests"]},
        {"name": "file_storage", "metrics": ["GB-month", "IOPS-hours"]},
        {"name": "archive", "metrics": ["GB-month", "retrieval-fee"]}
      ],
      "optimization_strategies": [
        "lifecycle policies to move cold data to cheaper tiers",
        "compression where applicable",
        "deduplication for backups",
        "choosing appropriate storage class for access patterns"
      ]
    },
    "network": {
      "subcategories": [
        {"name": "data_transfer", "metrics": ["GB-out", "GB-in", "inter-region"]},
        {"name": "load_balancing", "metrics": ["LCU-hours", "connections", "rules"]},
        {"name": "cdn", "metrics": ["GB-out", "requests"]}
      ],
      "optimization_strategies": [
        "minimizing data transfer out of cloud",
        "using internal endpoints for inter-service communication",
        "content compression to reduce transfer size",
        "CDN for static assets and API responses"
      ]
    },
    "database": {
      "subcategories": [
        {"name": "rds", "metrics": ["vCPU-hours", "GB-hours", "I/O-requests", "multi-AZ"]},
        {"name": "dynamodb", "metrics": ["write-capacity-units", "read-capacity-units", "storage"]},
        {"name": "elasticache", "metrics": ["node-hours", "data-transfer"]}
      ],
      "optimization_strategies": [
        "reserved capacity for predictable baseline",
        "on-demand for variable load",
        "auto-scaling with conservative floor",
        "choosing appropriate instance sizes"
      ]
    }
  }
}
```

### 1.2 Cost Optimization Decision Matrix

| Resource Type | Under-utilized Sign | Optimization Action | Expected Savings |
|---------------|--------------------|--------------------|------------------|
| EC2 Instance | CPU < 20% avg | Downsize or use spot | 30-60% |
| EBS Volume | Low disk I/O | Smaller volume or different type | 20-40% |
| RDS Instance | CPU < 30%, connections < 50% | Downsize | 20-40% |
| Load Balancer | Low traffic, few AZs | Fewer rules, single AZ | 30-50% |
| S3 Storage | Infrequent access | Move to IA/Glacier | 60-80% |
| NAT Gateway | Low traffic | VPC endpoints instead | 70-90% |

### 1.3 Resource Right-Sizing Specification

```yaml
# AWS Instance Right-Sizing Recommendations
RightSizingRecommendations:
  compute:
    cpu_utilization_threshold: 40
    memory_utilization_threshold: 50
    
    instance_types:
      - current: t3.large
        recommended: t3.medium
        condition: cpu_avg < 20 and memory_avg < 40
        monthly_savings: 45.50
        
      - current: m5.xlarge
        recommended: m5.large
        condition: cpu_avg < 30 and memory_avg < 30
        monthly_savings: 82.00
  
  database:
    rds_instances:
      - current: db.r5.xlarge
        recommended: db.r5.large
        condition: cpu_avg < 25 and connections_avg < 200
        monthly_savings: 156.00
      
    elasticache:
      - current: cache.r5.large
        recommended: cache.r5.small
        condition: cpu_avg < 20 and memory_avg < 40
        monthly_savings: 89.00
```

### 1.4 Reserved Capacity Planning

```json
{
  "ReservedCapacitySchema": {
    "type": "object",
    "required": ["service", "resource_type", "baseline_utilization"],
    "properties": {
      "service": {
        "type": "string",
        "enum": ["ec2", "rds", "elasticache", "redshift", "dynamodb"]
      },
      "resource_type": {
        "type": "string",
        "description": "Specific resource type (e.g., t3.medium, db.r5.large)"
      },
      "baseline_utilization": {
        "type": "object",
        "properties": {
          "minimum": {
            "type": "integer",
            "description": "Minimum guaranteed usage"
          },
          "typical": {
            "type": "integer",
            "description": "Typical/median usage"
          },
          "peak": {
            "type": "integer",
            "description": "Peak usage"
          }
        }
      },
      "recommendation": {
        "type": "object",
        "properties": {
          "reserved_count": {
            "type": "integer",
            "description": "Number of reserved instances to purchase"
          },
          "reserved_unit_cost": {
            "type": "number",
            "description": "Cost per reserved unit per year"
          },
          "on_demand_unit_cost": {
            "type": "number",
            "description": "Cost per on-demand unit per year"
          },
          "upfront_payment": {
            "type": "number",
            "description": "Upfront payment for reserved"
          },
          "annual_savings": {
            "type": "number",
            "description": "Annual savings vs on-demand"
          },
          "break_even_months": {
            "type": "integer",
            "description": "Months until reserved pays off"
          }
        }
      }
    }
  }
}
```

**Example Reserved Capacity Calculation:**

```yaml
reserved_capacity_analysis:
  service: ec2
  resource_type: t3.medium
  utilization:
    minimum: 2
    typical: 4
    peak: 10
    
  on_demand_pricing:
    hourly: 0.0416
    monthly_at_4_instances: 120.62
    annual: 1447.44
  
  reserved_pricing:
    hourly_per_instance: 0.0236
    monthly_per_instance: 17.06
    annual_per_instance: 204.72
    3_year_partial_upfront: 0.0168
    3_year_all_upfront: 0.0142
  
  recommendation:
    quantity: 4
    term: 3 years
    payment_type: all_upfront
    upfront_cost: 204.72
    annual_cost: 204.72
    vs_on_demand_annual: 1242.72
    savings_percent: 85.8%
    break_even_months: 1.97
```

---

## 2. Token Economics

### 2.1 Context Efficiency Targets

```json
{
  "TokenBudgetSchema": {
    "type": "object",
    "properties": {
      "task_type": {
        "type": "string",
        "enum": [
          "code_completion",
          "code_review",
          "documentation",
          "refactoring",
          "debugging",
          "architecture_design",
          "general"
        ]
      },
      "target_tokens": {
        "type": "object",
        "properties": {
          "prompt": {
            "type": "integer",
            "description": "Target prompt token count"
          },
          "completion": {
            "type": "integer",
            "description": "Target completion token count"
          },
          "total": {
            "type": "integer",
            "description": "Total target tokens"
          }
        }
      },
      "optimization": {
        "type": "object",
        "properties": {
          "minimization_strategies": {
            "type": "array",
            "items": {"type": "string"},
            "examples": [
              "inject only relevant files",
              "reuse session context",
              "exclude documentation",
              "truncate verbose logs",
              "use concise code snippets"
            ]
          },
          "reuse_threshold": {
            "type": "number",
            "description": "Reuse context if similarity above threshold"
          }
        }
      }
    }
  }
}
```

**Token Budget by Task Type:**

```yaml
token_budgets:
  code_completion:
    target_total: 30000
    prompt_target: 20000
    completion_target: 10000
    strategies:
      - inject only related files
      - provide relevant function signatures
      - exclude test files unless related
  
  code_review:
    target_total: 50000
    prompt_target: 40000
    completion_target: 10000
    strategies:
      - inject only changed files
      - provide relevant context
      - use diff format
  
  refactoring:
    target_total: 80000
    prompt_target: 60000
    completion_target: 20000
    strategies:
      - inject refactored files plus dependencies
      - provide architectural context
      - include constraints
  
  architecture_design:
    target_total: 100000
    prompt_target: 80000
    completion_target: 20000
    strategies:
      - inject relevant docs
      - provide system context
      - include requirements
```

### 2.2 Model Selection Framework

```json
{
  "ModelSelectionSchema": {
    "type": "object",
    "required": ["task_complexity", "requirements"],
    "properties": {
      "task_complexity": {
        "type": "string",
        "enum": ["simple", "moderate", "complex", "reasoning"],
        "criteria": {
          "simple": ["pattern matching", "basic formatting", "simple validation"],
          "moderate": ["code generation", "refactoring", "test writing"],
          "complex": ["debugging", "optimization", "complex refactoring"],
          "reasoning": ["architecture design", "multi-file refactoring", "problem solving"]
        }
      },
      "requirements": {
        "type": "object",
        "properties": {
          "speed": {
            "type": "string",
            "enum": ["fast", "medium", "slow_ok"],
            "impact": "Model tier selection"
          },
          "accuracy": {
            "type": "string",
            "enum": ["standard", "high"],
            "impact": "Whether to use advanced models"
          },
          "cost_sensitivity": {
            "type": "string",
            "enum": ["low", "medium", "high"]
          }
        }
      },
      "model_recommendation": {
        "type": "object",
        "properties": {
          "primary": {
            "type": "string",
            "description": "Recommended model"
          },
          "fallback": {
            "type": "string",
            "description": "Fallback if primary unavailable"
          },
          "cost_per_1k_tokens": {
            "type": "number"
          }
        }
      }
    }
  }
}
```

**Model Selection Decision Tree:**

```
Task Analysis
    │
    ├── Simple/Formatting Task?
    │   ├── YES → Use fast/cheap model
    │   │         (e.g., Haiku, gpt-3.5-turbo)
    │   │
    │   └── NO → Continue
    │
    ├── Code Completion (simple)?
    │   ├── YES → Use mid-tier model
    │   │         (e.g., Sonnet, gpt-4o-mini)
    │   │
    │   └── NO → Continue
    │
    ├── Complex/Reasoning Required?
    │   ├── YES → Use advanced model
    │   │         (e.g., Opus, gpt-4-turbo, claude-3-opus)
    │   │
    │   └── NO → Continue
    │
    └── Use mid-tier model
        (e.g., Sonnet 3.5, gpt-4o)
```

---

## 3. Agent-Specific Cost Optimization

### 3.1 Context Window Optimization

```json
{
  "ContextOptimization": {
    "file_selection": {
      "strategy": "relevance-based injection",
      "criteria": {
        "importance_score_threshold": 0.7,
        "max_files_per_task": 20,
        "max_file_size_kb": 100
      },
      "scoring": {
        "direct_imports": 1.0,
        "transitive_imports": 0.6,
        "recently_modified": 0.3,
        "test_files": -0.2,
        "generated_files": -0.3
      }
    },
    "content_truncation": {
      "strategies": [
        {"type": "head", "description": "Keep first N lines"},
        {"type": "tail", "description": "Keep last N lines"},
        {"type": "middle", "description": "Keep first and last N lines"},
        {"type": "relevant", "description": "Keep lines matching relevance query"}
      ],
      "defaults": {
        "large_file_threshold_kb": 10,
        "head_lines": 200,
        "tail_lines": 100
      }
    },
    "session_persistence": {
      "store_in_session": [
        "task_requirements",
        "resolved_dependencies",
        "architecture_decisions",
        "test_results"
      ],
      "exclude_from_session": [
        "temporary_files",
        "build_artifacts",
        "large_logs",
        "duplicate_context"
      ]
    }
  }
}
```

### 3.2 Proof Generation Cost Optimization

```json
{
  "ProofOptimization": {
    "incremental_proofs": {
      "enabled": true,
      "strategy": "diff-based",
      "incremental_fields": [
        "changed_files",
        "new_dependencies",
        "modified_interfaces"
      ],
      "full_proof_triggers": [
        "schema_change",
        "permission_change",
        "security_critical_path_change"
      ]
    },
    "proof_caching": {
      "enabled": true,
      "cache_key_fields": [
        "file_hash",
        "function_signature_hash",
        "context_hash"
      ],
      "cache_ttl_hours": 24
    },
    "validation_scope": {
      "full_validation_triggers": [
        "PR to main/master",
        "security-related changes",
        "infrastructure changes"
      ],
      "incremental_validation": [
        "PR to feature branch",
        "draft PR",
        "WIP commits"
      ]
    }
  }
}
```

---

## 4. Governance and Budgets

### 4.1 Budget Alert Configuration

```json
{
  "BudgetAlertConfiguration": {
    "alert_levels": [
      {
        "name": "warning",
        "threshold_percent": 80,
        "actions": [
          "notify_team_channel",
          "create_ticket"
        ],
        "message": "Budget at {percent}% of monthly allocation"
      },
      {
        "name": "critical",
        "threshold_percent": 95,
        "actions": [
          "notify_team_channel",
          "page_on_call",
          "block_new_resources"
        ],
        "message": "Budget CRITICAL: {percent}% of allocation used"
      },
      {
        "name": "exceeded",
        "threshold_percent": 100,
        "actions": [
          "notify_leadership",
          "block_new_resources",
          "suspend_non_production"
        ],
        "message": "Budget EXCEEDED: Immediate action required"
      }
    ],
    "tracking": {
      "granularity": "daily",
      "attribution": {
        "enabled": true,
        "dimensions": ["team", "service", "environment", "project"]
      },
      "forecast": {
        "enabled": true,
        "method": "linear_regression",
        "confidence_threshold": 0.8
      }
    }
  }
}
```

### 4.2 Cost Attribution Framework

```json
{
  "CostAttributionSchema": {
    "required_tags": [
      {"name": "environment", "values": ["production", "staging", "development"]},
      {"name": "team", "values": null},
      {"name": "service", "values": null},
      {"name": "cost_center", "values": null}
    ],
    "optional_tags": [
      {"name": "project", "values": null},
      {"name": "owner", "values": null},
      {"name": "managed_by", "values": ["terraform", "manual", "auto"]}
    ],
    "enforcement": {
      "require_tags_on_create": true,
      "block_untagged_resources": true,
      "tag_validation_period_hours": 24
    },
    "reports": {
      "daily_summary": {
        "recipients": ["cloud-costs@example.com"],
        "format": "email"
      },
      "weekly_breakdown": {
        "recipients": ["team-leads@example.com"],
        "format": "pdf"
      },
      "monthly_invoice": {
        "recipients": ["finance@example.com"],
        "format": "csv"
      }
    }
  }
}
```

---

## 5. Optimization Playbooks

### 5.1 Compute Cost Reduction

```yaml
compute_optimization:
  step_1_discovery:
    actions:
      - name: "Export cost explorer data"
        command: >
          aws ce get-cost-and-usage
          --time-period Start=2024-01-01,End=2024-01-31
          --granularity MONTHLY
          --metrics "BlendedCost" "UnblendedCost" "UsageQuantity"
          --group-by Type=DIMENSION,Key=INSTANCE_TYPE
      
      - name: "Export rightsizing recommendations"
        command: >
          aws compute-optimizer get-ec2-instance-recommendations
      
    outputs:
      - underutilized_instances.csv
      - rightsizing_recommendations.json
  
  step_2_analysis:
    criteria:
      cpu_utilization_threshold: 40
      memory_utilization_threshold: 50
      network_utilization_threshold: 20
      
    instance_analysis:
      - name: "Identify over-provisioned"
        condition: "utilization < threshold"
        action: "schedule_rightsizing"
        
      - name: "Identify unused"
        condition: "utilization == 0 for 30 days"
        action: "schedule_termination"
        
      - name: "Identify batch workloads"
        condition: "pattern matches 'nightly', 'weekend'"
        action: "migrate_to_spot"
  
  step_3_implementation:
    actions:
      - name: "Implement auto-scaling"
        target: "all_stateful_services"
        
      - name: "Purchase reserved for baseline"
        target: "steady_state_instances"
        savings_target: 30
      
      - name: "Migrate batch to spot"
        target: "non-critical_batch_jobs"
        expected_savings: 60
  
  step_4_validation:
    metrics:
      - "Monthly spend vs previous month"
      - "Instance utilization distribution"
      - "Reserved coverage ratio"
```

### 5.2 Storage Cost Reduction

```yaml
storage_optimization:
  lifecycle_policies:
    standard_to_ia:
      source_tier: "STANDARD"
      target_tier: "STANDARD_IA"
      trigger_days: 30
      
    ia_to_glacier:
      source_tier: "STANDARD_IA"
      target_tier: "GLACIER"
      trigger_days: 90
      
    glacier_to_deep_archive:
      source_tier: "GLACIER"
      target_tier: "DEEP_ARCHIVE"
      trigger_days: 365
  
  cleanup_actions:
    - name: "Delete old backups"
      resource: "automated_backups"
      retention_days: 7
      
    - name: "Remove test data"
      resource: "test_environments"
      pattern: "*_test_*"
      
    - name: "Empty deleted buckets"
      resource: "deleted_buckets"
      grace_period_days: 14
  
  monitoring:
    - metric: "storage.utilization.growth_rate"
      alert_threshold: "20% increase in 30 days"
      
    - metric: "storage.untagged.resources"
      alert_threshold: "any untagged resource"
```

### 5.3 Network Cost Reduction

```yaml
network_optimization:
  data_transfer:
    optimization_strategies:
      - name: "Use VPC endpoints"
        services: ["s3", "dynamodb", "sqs"]
        savings: "eliminate NAT gateway fees for internal traffic"
        
      - name: "Use internal ALB"
        pattern: "internal services via internal ALB, not public"
        savings: "reduce data transfer costs"
        
      - name: "Enable CDN"
        pattern: "static assets, API responses"
        savings: "reduce origin data transfer"
  
  cdn_optimization:
    cache_rules:
      - path_pattern: "*.css"
        ttl: 31536000  # 1 year (immutable)
        
      - path_pattern: "*.js"
        ttl: 31536000
        
      - path_pattern: "images/*"
        ttl: 86400
        
      - path_pattern: "api/*"
        ttl: 0  # No cache
        
    compression:
      enabled: true
      minimum_size_bytes: 1024
      algorithms: ["gzip", "brotli"]
```

---

## 6. Anti-Patterns

| Anti-Pattern | Specific Failure Mode | Prevention |
|--------------|----------------------|------------|
| **Over-provisioning** | Wasted spend on idle capacity | Right-size based on metrics |
| **No reserved capacity** | Paying premium for steady-state | Buy reserved for baseline |
| **Using prod for dev** | Dev workloads on expensive infra | Use dev-tier resources |
| **No auto-scaling** | Paying for peak when idle | Scale to match demand |
| **Inefficient data transfer** | High egress charges | Use VPC endpoints, CDN |
| **Unoptimized storage tiers** | Paying hot storage for cold data | Lifecycle policies |
| **No budget alerts** | Surprise bills | Set up proactive alerts |
| **Unused resources** | Zombie resources billing | Regular cleanup audits |
| **No tagging** | Can't attribute costs | Enforce tag requirements |
| **No cost visibility** | No one knows who spends what | Implement chargeback/showback |

---

## Links

### Core Router
- `core/DECAPOD.md` - **Router and navigation charter (START HERE)**
- `core/ENGINEERING_EXCELLENCE.md` - Engineering standards

### Architecture (This Section)
- `architecture/CLOUD.md` - Cloud infrastructure
- `architecture/DATA.md` - Data storage
- `architecture/SECURITY.md` - Security architecture

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
- `methodology/CLOUD_COST_MANAGEMENT.md` - Cost optimization