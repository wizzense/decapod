# DATA.md - Data Architecture (DENSE)

**Authority:** guidance (data storage, modeling, and governance patterns)
**Layer:** Guides
**Binding:** No
**Scope:** data architecture principles, storage selection, and data governance
**Non-goals:** specific database implementations, one-size-fits-all solutions

---

## 1. Data Architecture Principles

### 1.1 Data Longevity
**Data outlives code by orders of magnitude.** Design for data that will survive:
- Multiple code rewrites
- Technology stack changes
- Team turnover
- Business pivots

### 1.2 Schema as Contract
Schema is the interface between data producers and consumers:
- Schema changes are migrations, not patches
- Backward compatibility is required unless explicitly coordinated
- Schema versioning enables gradual evolution
- Documentation is part of the schema

### 1.3 Data Ownership
Every data entity has a single owner:
- Owner defines schema and access patterns
- Owner manages lifecycle (retention, archival)
- Owner handles migrations
- Other services access through defined interfaces

### 1.4 Production Mindset
Data decisions compound over years. Schema choices made at week one outlive three engineering teams:

- **Data is the primary asset:** The most durable output of any engineering effort is clean, structured, accessible data. Code is a snapshot; data persists. Decisions must be data-driven, which requires data to be high-fidelity.
- **Avoid proprietary data lock-in:** Core data should live in open, portable formats (Postgres, Parquet, Avro). Vendor-specific binary formats create migration debt that compounds as volume grows.
- **Schema before storage:** There is no such thing as "schemaless in production" — only schema that is unknown to the database and therefore unenforceable. Express schema explicitly using protobuf, JSON Schema, or equivalent. Unstructured data is just data whose structure you haven't modeled yet.
- **Privacy and deletion are architecture requirements:** Compliance (GDPR, CCPA, HIPAA) is the legal floor. Deletion and anonymization must be designed into the data model from the start, not retrofitted. Data that cannot be deleted on demand is an incident waiting to happen.
- **Consistency model is a design choice, not a default:** Understand where your system sits in the CAP theorem and make it explicit. Core transactional state requires consistency (CP). High-frequency event logs can tolerate availability-priority (AP). Never drift into an unexamined middle.
- **Design for the next migration:** Every data structure should be written with its own evolution in mind. If the schema cannot support two live versions simultaneously, the design is incomplete.
- **Referential integrity is absolute:** If the database supports foreign keys, use them. If it does not, enforce integrity in the application layer. Orphaned references are data rot, and data rot compounds silently until a system fails in an unrecoverable way.
- **N+1 is an architectural smell:** A loop that issues one query per item is not a performance optimization opportunity — it is a design defect. Use joins, batching, or projection. Catch it in review, not production.

---

## 2. Storage Selection Framework

### 2.1 Decision Matrix

| Use Case | Primary Choice | When to Consider Alternatives |
|----------|---------------|------------------------------|
| Transactional (ACID) | PostgreSQL | Scale > 10TB or extreme write throughput |
| Document (flexible schema) | MongoDB | Need complex transactions |
| Key-Value (caching/session) | Redis | Need persistence guarantees |
| Time-series (metrics/logs) | TimescaleDB/InfluxDB | Small scale (< 1M points/day) |
| Graph (relationships) | Neo4j | Relationships fit in relational model |
| Search (full-text) | Elasticsearch | Simple search fits in Postgres |
| Blob (files/images) | S3 | Need filesystem semantics |
| Queue (async work) | Kafka/RabbitMQ | Simple queues fit in Redis |

### 2.2 Database Schema Definitions

**PostgreSQL Table Schema:**

```sql
-- Users table with full audit trail
CREATE TABLE users (
    -- Primary key
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    -- Core attributes
    email VARCHAR(254) NOT NULL,
    email_verified BOOLEAN NOT NULL DEFAULT false,
    
    -- Authentication
    password_hash VARCHAR(255),
    mfa_secret VARCHAR(255),
    mfa_enabled BOOLEAN NOT NULL DEFAULT false,
    
    -- Profile
    display_name VARCHAR(100),
    avatar_url TEXT,
    timezone VARCHAR(50) DEFAULT 'UTC',
    locale VARCHAR(10) DEFAULT 'en-US',
    
    -- Status
    status VARCHAR(20) NOT NULL DEFAULT 'active' 
        CHECK (status IN ('pending', 'active', 'suspended', 'deleted')),
    
    -- Security
    failed_login_attempts INTEGER NOT NULL DEFAULT 0,
    locked_until TIMESTAMPTZ,
    last_login_at TIMESTAMPTZ,
    last_login_ip INET,
    
    -- Timestamps with versioning
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    version INTEGER NOT NULL DEFAULT 1,
    
    -- Constraints
    CONSTRAINT users_email_unique UNIQUE (email) WHERE deleted_at IS NULL
);

-- Indexes for common queries
CREATE INDEX idx_users_email ON users (email) WHERE deleted_at IS NULL;
CREATE INDEX idx_users_status ON users (status) WHERE deleted_at IS NULL;
CREATE INDEX idx_users_created_at ON users (created_at DESC);

-- Update trigger for updated_at and version
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    NEW.version = OLD.version + 1;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
```

**MongoDB Document Schema:**

```javascript
// users collection schema
{
  $jsonSchema: {
    bsonType: "object",
    required: ["_id", "email", "status", "createdAt", "updatedAt"],
    properties: {
      _id: {
        bsonType: "objectId"
      },
      email: {
        bsonType: "string",
        maxLength: 254,
        pattern: "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$"
      },
      emailVerified: {
        bsonType: "bool",
        default: false
      },
      passwordHash: {
        bsonType: "string"
      },
      mfa: {
        bsonType: "object",
        properties: {
          enabled: { bsonType: "bool" },
          secret: { bsonType: "string" },
          backupCodes: { bsonType: "array" }
        }
      },
      profile: {
        bsonType: "object",
        properties: {
          displayName: { bsonType: "string", maxLength: 100 },
          avatarUrl: { bsonType: "string" },
          timezone: { bsonType: "string", default: "UTC" },
          locale: { bsonType: "string", default: "en-US" },
          bio: { bsonType: "string", maxLength: 500 },
          socialLinks: {
            bsonType: "object",
            properties: {
              twitter: { bsonType: "string" },
              github: { bsonType: "string" },
              linkedin: { bsonType: "string" }
            }
          }
        }
      },
      status: {
        bsonType: "string",
        enum: ["pending", "active", "suspended", "deleted"],
        default: "pending"
      },
      security: {
        bsonType: "object",
        properties: {
          failedLoginAttempts: { bsonType: "int", minimum: 0 },
          lockedUntil: { bsonType: "date" },
          lastLoginAt: { bsonType: "date" },
          lastLoginIp: { bsonType: "string" }
        }
      },
      preferences: {
        bsonType: "object",
        properties: {
          notifications: {
            bsonType: "object",
            properties: {
              email: { bsonType: "bool" },
              push: { bsonType: "bool" }
            }
          },
          privacy: {
            bsonType: "object",
            properties: {
              profileVisibility: { 
                bsonType: "string", 
                enum: ["public", "private", "unlisted"] 
              }
            }
          }
        }
      },
      createdAt: { bsonType: "date" },
      updatedAt: { bsonType: "date" },
      deletedAt: { bsonType: "date" }
    }
  }
}

// Required indexes
db.users.createIndex({ email: 1 }, { unique: true, partialFilterExpression: { deletedAt: null } })
db.users.createIndex({ status: 1 })
db.users.createIndex({ createdAt: -1 })
```

**TimescaleDB Hypertable:**

```sql
-- Create TimescaleDB hypertable for metrics
CREATE TABLE metrics (
    time TIMESTAMPTZ NOT NULL,
    metric_name VARCHAR(255) NOT NULL,
    value DOUBLE PRECISION NOT NULL,
    labels JSONB,
    entity_id UUID,
    org_id UUID NOT NULL
);

-- Convert to hypertable with chunk interval
SELECT create_hypertable(
    'metrics', 
    'time', 
    chunk_time_interval => INTERVAL '1 day',
    migrate_data => true
);

-- Add compression for old data
ALTER TABLE metrics SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'metric_name, org_id'
);

-- Add continuous aggregate for 1-minute rollups
CREATE MATERIALIZED VIEW metrics_1m
WITH (timescaledb.continuous) AS
SELECT 
    time_bucket('1 minute', time) AS bucket,
    metric_name,
    org_id,
    AVG(value) as value_avg,
    MIN(value) as value_min,
    MAX(value) as value_max,
    COUNT(*) as sample_count
FROM metrics
GROUP BY 1, 2, 3
WITH NO DATA;

-- Refresh policy
SELECT add_continuous_aggregate_policy('metrics_1m',
    start_offset => INTERVAL '1 hour',
    end_offset => INTERVAL '1 minute',
    schedule_interval => INTERVAL '1 minute');
```

### 2.3 Multi-Model Considerations

```yaml
# Polyglot Persistence Architecture
MultiModelArchitecture:
  primary_database:
    type: postgresql
    host: postgres-primary.example.com
    port: 5432
    database: production
    pool:
      min_size: 10
      max_size: 100
      idle_timeout: 30000
    ssl:
      enabled: true
      mode: require
  
  search:
    type: elasticsearch
    hosts:
      - es-1.example.com:9200
      - es-2.example.com:9200
      - es-3.example.com:9200
    index_strategy:
      users:
        shards: 5
        replicas: 2
        refresh_interval: "5s"
      products:
        shards: 10
        replicas: 2
        refresh_interval: "1s"
    security:
      api_key_auth: true
  
  cache:
    type: redis
    cluster:
      enabled: true
      nodes:
        - redis-1.example.com:6379
        - redis-2.example.com:6379
        - redis-3.example.com:6379
    persistence:
      rdb_enabled: true
      aof_enabled: true
  
  time_series:
    type: timescaleDB
    host: timeseries.example.com
    port: 5432
    retention:
      raw_data: "30 days"
      aggregates: "1 year"
  
  blob_storage:
    type: s3
    bucket: production-assets
    region: us-east-1
    storage_classes:
      hot: STANDARD
      warm: STANDARD_IA
      cold: GLACIER
    lifecycle_rules:
      - prefix: "uploads/"
        transition_days: 90
        target_class: GLACIER
```

---

## 3. Data Modeling Patterns

### 3.1 Relational Modeling

```sql
-- E-commerce Order Schema (3NF)
CREATE TABLE customers (
    customer_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(254) NOT NULL UNIQUE,
    first_name VARCHAR(100),
    last_name VARCHAR(100),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE addresses (
    address_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id UUID NOT NULL REFERENCES customers(customer_id),
    address_type VARCHAR(20) CHECK (address_type IN ('shipping', 'billing')),
    street_line1 VARCHAR(255) NOT NULL,
    street_line2 VARCHAR(255),
    city VARCHAR(100) NOT NULL,
    state VARCHAR(100),
    postal_code VARCHAR(20) NOT NULL,
    country_code CHAR(2) NOT NULL,
    is_default BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE products (
    product_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sku VARCHAR(100) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    price DECIMAL(10, 2) NOT NULL CHECK (price >= 0),
    cost DECIMAL(10, 2) CHECK (cost >= 0),
    category_id UUID REFERENCES categories(category_id),
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE order_items (
    order_item_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id UUID NOT NULL REFERENCES orders(order_id),
    product_id UUID NOT NULL REFERENCES products(product_id),
    quantity INTEGER NOT NULL CHECK (quantity > 0),
    unit_price DECIMAL(10, 2) NOT NULL,
    discount DECIMAL(10, 2) DEFAULT 0,
    PRIMARY KEY (order_id, order_item_id)
);

CREATE TABLE orders (
    order_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    customer_id UUID NOT NULL REFERENCES customers(customer_id),
    shipping_address_id UUID REFERENCES addresses(address_id),
    billing_address_id UUID REFERENCES addresses(address_id),
    status VARCHAR(20) NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'confirmed', 'processing', 'shipped', 'delivered', 'cancelled')),
    subtotal DECIMAL(10, 2) NOT NULL,
    tax DECIMAL(10, 2) DEFAULT 0,
    shipping_cost DECIMAL(10, 2) DEFAULT 0,
    total DECIMAL(10, 2) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Query: Get customer order history with items
SELECT 
    c.customer_id,
    c.email,
    o.order_id,
    o.created_at,
    o.status,
    o.total,
    json_agg(
        json_build_object(
            'product_name', p.name,
            'quantity', oi.quantity,
            'unit_price', oi.unit_price
        )
    ) FILTER (WHERE oi.order_item_id IS NOT NULL) as items
FROM customers c
JOIN orders o ON c.customer_id = o.customer_id
LEFT JOIN order_items oi ON o.order_id = oi.order_id
LEFT JOIN products p ON oi.product_id = p.product_id
WHERE c.customer_id = $1
GROUP BY c.customer_id, c.email, o.order_id, o.created_at, o.status, o.total
ORDER BY o.created_at DESC;
```

### 3.2 Document Modeling

```javascript
// E-commerce Order Document (MongoDB)
{
  "_id": ObjectId("..."),
  "orderNumber": "ORD-2026-001234567",
  "customer": {
    "customerId": "cust_abc123",
    "email": "customer@example.com",
    "name": {
      "first": "John",
      "last": "Doe"
    }
  },
  "items": [
    {
      "productId": "prod_xyz789",
      "sku": "WIDGET-001",
      "name": "Premium Widget",
      "quantity": 2,
      "unitPrice": 29.99,
      "discount": 0,
      "lineTotal": 59.98
    },
    {
      "productId": "prod_abc456",
      "sku": "GADGET-002",
      "name": "Super Gadget",
      "quantity": 1,
      "unitPrice": 49.99,
      "discount": 5.00,
      "lineTotal": 44.99
    }
  ],
  "shippingAddress": {
    "street": "123 Main St",
    "city": "San Francisco",
    "state": "CA",
    "postalCode": "94102",
    "country": "US"
  },
  "billingAddress": {
    "sameAsShipping": true
  },
  "pricing": {
    "subtotal": 104.97,
    "tax": 9.45,
    "taxRate": 0.09,
    "shippingCost": 5.99,
    "discount": 5.00,
    "total": 115.41
  },
  "payment": {
    "method": "credit_card",
    "last4": "4242",
    "brand": "visa"
  },
  "status": "processing",
  "statusHistory": [
    {
      "status": "pending",
      "timestamp": ISODate("2026-05-16T10:00:00Z"),
      "note": "Order placed"
    },
    {
      "status": "confirmed",
      "timestamp": ISODate("2026-05-16T10:01:00Z"),
      "note": "Payment confirmed"
    },
    {
      "status": "processing",
      "timestamp": ISODate("2026-05-16T10:05:00Z"),
      "note": "Fulfillment started"
    }
  ],
  "createdAt": ISODate("2026-05-16T10:00:00Z"),
  "updatedAt": ISODate("2026-05-16T10:05:00Z")
}
```

### 3.3 Event Sourcing

```json
{
  "EventSchema": {
    "$schema": "http://json-schema.org/draft-07/schema#",
    "title": "DomainEvent",
    "type": "object",
    "required": ["event_id", "event_type", "aggregate_id", "aggregate_type", "version", "timestamp", "data"],
    "properties": {
      "event_id": {
        "type": "string",
        "description": "Unique event identifier (ULID)"
      },
      "event_type": {
        "type": "string",
        "pattern": "^[A-Z][a-zA-Z0-9]*\\.[A-Z][a-zA-Z0-9]*$",
        "description": "Dot-separated: Entity.Action (e.g., User.EmailChanged)"
      },
      "aggregate_id": {
        "type": "string",
        "description": "ID of the aggregate this event belongs to"
      },
      "aggregate_type": {
        "type": "string",
        "description": "Type of aggregate (e.g., User, Order)"
      },
      "version": {
        "type": "integer",
        "minimum": 1,
        "description": "Sequence number for ordering"
      },
      "timestamp": {
        "type": "string",
        "format": "date-time"
      },
      "causation_id": {
        "type": "string",
        "description": "ID of the command that caused this event"
      },
      "correlation_id": {
        "type": "string",
        "description": "ID for correlating related events"
      },
      "actor": {
        "type": "object",
        "properties": {
          "type": {"type": "string"},
          "id": {"type": "string"}
        }
      },
      "data": {
        "type": "object",
        "description": "Event-specific payload"
      },
      "metadata": {
        "type": "object",
        "additionalProperties": true
      }
    }
  }
}
```

**Event Examples:**

```json
{
  "event_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
  "event_type": "Order.Created",
  "aggregate_id": "ord_01ARZ3NDEKTSV4RRFFQ69G5FA0",
  "aggregate_type": "Order",
  "version": 1,
  "timestamp": "2026-05-16T10:00:00.000Z",
  "correlation_id": "01ARZ3NDEKTSV4RRFFQ69G5FA1",
  "actor": {"type": "user", "id": "usr_abc123"},
  "data": {
    "orderNumber": "ORD-2026-001234567",
    "customerId": "cust_abc123",
    "items": [...],
    "total": 115.41
  }
}

{
  "event_id": "01ARZ3NDEKTSV4RRFFQ69G5FBV",
  "event_type": "Order.StatusChanged",
  "aggregate_id": "ord_01ARZ3NDEKTSV4RRFFQ69G5FA0",
  "aggregate_type": "Order",
  "version": 2,
  "timestamp": "2026-05-16T10:05:00.000Z",
  "causation_id": "cmd_01ARZ3NDEKTSV4RRFFQ69G5FA2",
  "actor": {"type": "service", "id": "fulfillment-service"},
  "data": {
    "previousStatus": "confirmed",
    "newStatus": "processing",
    "reason": "Fulfillment started"
  }
}
```

---

## 4. Data Governance

### 4.1 Data Classification Schema

```json
{
  "DataClassificationSchema": {
    "type": "object",
    "required": ["classification", "owner", "retention"],
    "properties": {
      "classification": {
        "type": "string",
        "enum": ["public", "internal", "confidential", "restricted"]
      },
      "owner": {
        "type": "object",
        "required": ["team", "contact"],
        "properties": {
          "team": {"type": "string"},
          "contact": {"type": "string"}
        }
      },
      "retention": {
        "type": "object",
        "required": ["period"],
        "properties": {
          "period": {"type": "integer"},
          "unit": {"type": "string", "enum": ["days", "months", "years"]},
          "archive_after": {"type": "integer"},
          "delete_after": {"type": "integer"}
        }
      },
      "encryption": {
        "type": "object",
        "properties": {
          "at_rest": {"type": "boolean"},
          "in_transit": {"type": "boolean"},
          "algorithm": {"type": "string"}
        }
      },
      "pii_fields": {
        "type": "array",
        "items": {
          "type": "object",
          "properties": {
            "field_name": {"type": "string"},
            "pii_type": {
              "type": "string",
              "enum": ["name", "email", "phone", "address", "ssn", "dob", "financial", "health", "biometric"]
            },
            "masking_required": {"type": "boolean"},
            "consent_required": {"type": "boolean"}
          }
        }
      },
      "access_control": {
        "type": "object",
        "properties": {
          "role_based": {"type": "boolean"},
          "allowed_roles": {"type": "array", "items": {"type": "string"}},
          "audit_required": {"type": "boolean"}
        }
      }
    }
  }
}
```

**Data Classification Examples:**

```yaml
data_classifications:
  - table: users
    classification: restricted
    owner:
      team: identity-platform
      contact: identity-team@example.com
    retention:
      period: 2555  # 7 years after account deletion
      unit: days
    encryption:
      at_rest: true
      algorithm: AES-256-GCM
    pii_fields:
      - field_name: email
        pii_type: email
        masking_required: false
        consent_required: true
      - field_name: phone_number
        pii_type: phone
        masking_required: true
        consent_required: true
      - field_name: date_of_birth
        pii_type: dob
        masking_required: false
        consent_required: true
    access_control:
      role_based: true
      allowed_roles: [identity-admin, billing-service]
      audit_required: true
      
  - table: user_sessions
    classification: internal
    owner:
      team: identity-platform
      contact: identity-team@example.com
    retention:
      period: 30
      unit: days
    encryption:
      at_rest: true
      in_transit: true
    access_control:
      role_based: false
      audit_required: false
      
  - table: products
    classification: public
    owner:
      team: catalog-platform
      contact: catalog-team@example.com
    retention:
      period: -1  # indefinite
      unit: days
    access_control:
      role_based: false
```

### 4.2 Data Retention Policies

```json
{
  "DataRetentionPolicySchema": {
    "type": "object",
    "properties": {
      "policies": {
        "type": "array",
        "items": {
          "type": "object",
          "required": ["name", "source", "retention", "storage_tier"],
          "properties": {
            "name": {"type": "string"},
            "description": {"type": "string"},
            "source": {
              "type": "object",
              "properties": {
                "type": {"type": "string"},
                "table": {"type": "string"},
                "filter": {"type": "string"}
              }
            },
            "retention": {
              "type": "object",
              "properties": {
                "hot": {
                  "type": "object",
                  "properties": {
                    "duration": {"type": "integer"},
                    "unit": {"type": "string"}
                  }
                },
                "warm": {
                  "type": "object",
                  "properties": {
                    "duration": {"type": "integer"},
                    "unit": {"type": "string"}
                  }
                },
                "cold": {
                  "type": "object",
                  "properties": {
                    "duration": {"type": "integer"},
                    "unit": {"type": "string"}
                  }
                },
                "delete": {
                  "type": "object",
                  "properties": {
                    "enabled": {"type": "boolean"},
                    "method": {"type": "string"}
                  }
                }
              }
            },
            "storage_tier": {
              "type": "string",
              "enum": ["standard", "ia", "glacier", "deep_archive"]
            }
          }
        }
      }
    }
  }
}
```

---

## 5. Migration Strategies

### 5.1 Zero-Downtime Migration Pattern

```sql
-- Phase 1: Add new column (nullable)
ALTER TABLE users 
ADD COLUMN display_name VARCHAR(100);

-- Phase 2: Backfill data
UPDATE users 
SET display_name = first_name || ' ' || last_name
WHERE display_name IS NULL 
  AND first_name IS NOT NULL;

-- Phase 3: Add NOT NULL constraint (requires backfill first)
ALTER TABLE users 
ALTER COLUMN display_name SET NOT NULL;

-- Phase 4: Add CHECK constraint
ALTER TABLE users 
ADD CONSTRAINT users_display_name_length 
CHECK (LENGTH(display_name) >= 1);

-- Phase 5: Add index
CREATE INDEX idx_users_display_name 
ON users (display_name);
```

### 5.2 Expand-Contract Migration

```json
{
  "migration_strategy": {
    "name": "Rename user.display_name to user.name",
    "phases": [
      {
        "phase": 1,
        "name": "Expand - Add new column",
        "direction": "expand",
        "sql": "ALTER TABLE users ADD COLUMN name VARCHAR(100)",
        "backfill": "UPDATE users SET name = display_name WHERE display_name IS NOT NULL",
        "rollback": "ALTER TABLE users DROP COLUMN IF EXISTS name"
      },
      {
        "phase": 2,
        "name": "Dual write",
        "direction": "expand",
        "description": "Application writes to both columns",
        "code_change": {
          "before": "user.displayName = value",
          "after": "user.displayName = value; user.name = value"
        }
      },
      {
        "phase": 3,
        "name": "Read from new column",
        "direction": "migrate",
        "description": "Application reads from new column, falls back to old",
        "code_change": {
          "read_logic": "user.name ?: user.displayName"
        }
      },
      {
        "phase": 4,
        "name": "Verify migration complete",
        "direction": "migrate",
        "checks": [
          "SELECT COUNT(*) FROM users WHERE name IS NULL AND display_name IS NOT NULL = 0"
        ]
      },
      {
        "phase": 5,
        "name": "Stop writing to old column",
        "direction": "contract",
        "code_change": {
          "before": "user.displayName = value; user.name = value",
          "after": "user.name = value"
        }
      },
      {
        "phase": 6,
        "name": "Remove old column",
        "direction": "contract",
        "sql": "ALTER TABLE users DROP COLUMN display_name",
        "requires_maintenance_window": true
      }
    ]
  }
}
```

---

## 6. Data Access Patterns

### 6.1 Repository Pattern

```rust
// Repository trait
trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<User>>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>>;
    async fn create(&self, user: &User) -> Result<User>;
    async fn update(&self, user: &User) -> Result<User>;
    async fn delete(&self, id: &Uuid) -> Result<()>;
    async fn list(&self, filter: &UserFilter) -> Result<Vec<User>>;
}

// Postgres implementation
struct PostgresUserRepository {
    pool: PgPool,
}

impl UserRepository for PostgresUserRepository {
    async fn find_by_id(&self, id: &Uuid) -> Result<Option<User>> {
        let row: Option<UserRow> = sqlx_query_as!(
            UserRow,
            r#"
            SELECT id, email, email_verified, password_hash, mfa_enabled,
                   display_name, avatar_url, timezone, locale, status,
                   failed_login_attempts, locked_until, last_login_at,
                   created_at, updated_at, deleted_at
            FROM users
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(row.map(|r| r.into()))
    }
    
    async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let row: Option<UserRow> = sqlx_query_as!(
            UserRow,
            r#"
            SELECT id, email, email_verified, password_hash, mfa_enabled,
                   display_name, avatar_url, timezone, locale, status,
                   failed_login_attempts, locked_until, last_login_at,
                   created_at, updated_at, deleted_at
            FROM users
            WHERE email = $1 AND deleted_at IS NULL
            "#,
            email
        )
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(row.map(|r| r.into()))
    }
    
    async fn create(&self, user: &User) -> Result<User> {
        let row: UserRow = sqlx_query_as!(
            UserRow,
            r#"
            INSERT INTO users (id, email, password_hash, display_name, status)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, email, email_verified, password_hash, mfa_enabled,
                      display_name, avatar_url, timezone, locale, status,
                      failed_login_attempts, locked_until, last_login_at,
                      created_at, updated_at, deleted_at
            "#,
            user.id,
            user.email,
            user.password_hash,
            user.display_name,
            user.status.as_str()
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(row.into())
    }
    
    async fn update(&self, user: &User) -> Result<User> {
        let row: UserRow = sqlx_query_as!(
            UserRow,
            r#"
            UPDATE users
            SET email = $2,
                display_name = $3,
                avatar_url = $4,
                timezone = $5,
                locale = $6,
                updated_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, email, email_verified, password_hash, mfa_enabled,
                      display_name, avatar_url, timezone, locale, status,
                      failed_login_attempts, locked_until, last_login_at,
                      created_at, updated_at, deleted_at
            "#,
            user.id,
            user.email,
            user.display_name,
            user.avatar_url,
            user.timezone,
            user.locale
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(row.into())
    }
}
```

### 6.2 N+1 Query Prevention

```rust
// BAD: N+1 query problem
async fn get_user_orders_with_products(user_id: &Uuid) -> Result<Vec<Order>> {
    let orders = db.find_orders_by_user(user_id).await?;
    
    for order in &mut orders {
        // This causes N queries!
        order.products = db.find_products_by_order(&order.id).await?;
    }
    
    Ok(orders)
}

// GOOD: Single join query
async fn get_user_orders_with_products(user_id: &Uuid) -> Result<Vec<Order>> {
    let rows: Vec<OrderWithProductsRow> = sqlx_query_as!(
        OrderWithProductsRow,
        r#"
        SELECT 
            o.id, o.order_number, o.status, o.total, o.created_at,
            json_agg(
                json_build_object(
                    'product_id', p.id,
                    'product_name', p.name,
                    'quantity', oi.quantity,
                    'unit_price', oi.unit_price
                )
            ) FILTER (WHERE p.id IS NOT NULL) as products
        FROM orders o
        LEFT JOIN order_items oi ON o.id = oi.order_id
        LEFT JOIN products p ON oi.product_id = p.id
        WHERE o.customer_id = $1
        GROUP BY o.id, o.order_number, o.status, o.total, o.created_at
        ORDER BY o.created_at DESC
        "#,
        user_id
    )
    .fetch_all(&self.pool)
    .await?;
    
    Ok(rows.into_iter().map(|r| r.into()).collect())
}

// GOOD: Batched queries (DataLoader pattern)
async fn get_orders_products(&self, order_ids: &[Uuid]) -> Result<HashMap<Uuid, Vec<Product>>> {
    let products: Vec<ProductRow> = sqlx_query_as!(
        ProductRow,
        r#"
        SELECT p.id, p.name, p.price, oi.order_id, oi.quantity
        FROM order_items oi
        JOIN products p ON oi.product_id = p.id
        WHERE oi.order_id = ANY($1)
        ORDER BY oi.order_id
        "#,
        &order_ids
    )
    .fetch_all(&self.pool)
    .await?;
    
    let mut map: HashMap<Uuid, Vec<Product>> = HashMap::new();
    for product in products {
        map.entry(product.order_id)
           .or_default()
           .push(product.into());
    }
    
    Ok(map)
}
```

---

## 7. Performance & Scaling

### 7.1 Connection Pool Configuration

```json
{
  "ConnectionPoolConfiguration": {
    "type": "object",
    "required": ["min_connections", "max_connections", "acquire_timeout"],
    "properties": {
      "min_connections": {
        "type": "integer",
        "minimum": 0,
        "maximum": 100,
        "description": "Always-open connections"
      },
      "max_connections": {
        "type": "integer",
        "minimum": 1,
        "maximum": 1000,
        "description": "Maximum concurrent connections"
      },
      "acquire_timeout": {
        "type": "integer",
        "minimum": 1000,
        "maximum": 60000,
        "description": "Max wait time for connection in ms"
      },
      "idle_timeout": {
        "type": "integer",
        "minimum": 0,
        "description": "Close idle connections after ms"
      },
      "max_lifetime": {
        "type": "integer",
        "minimum": 0,
        "description": "Recycle connections after ms"
      },
      "validation": {
        "type": "object",
        "properties": {
          "on_acquire": {"type": "boolean"},
          "on_release": {"type": "boolean"}
        }
      }
    }
  }
}
```

### 7.2 Read Replica Routing

```rust
// Read replica router
struct ReplicaRouter {
    primary: PgPool,
    replicas: Vec<PgPool>,
    read_only_routes: HashMap<String, Route>,
}

enum PoolSelector {
    Primary,
    Replica(RoundRobin),
}

impl ReplicaRouter {
    fn select_pool(&self, query: &str) -> &PgPool {
        let is_read_only = query.trim().to_uppercase().starts_with("SELECT")
            && !query.contains("FOR UPDATE")
            && !query.contains("FOR SHARE");
        
        if is_read_only {
            // Round-robin to replicas
            self.replicas.first().unwrap_or(&self.primary)
        } else {
            &self.primary
        }
    }
}
```

---

## 8. Security & Compliance

### 8.1 Column-Level Encryption

```sql
-- Enable pgcrypto extension
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- Add encrypted column
ALTER TABLE users 
ADD COLUMN ssn_encrypted BYTEA;

-- Add key ID column for key rotation
ALTER TABLE users 
ADD COLUMN ssn_key_id VARCHAR(50);

-- Function to encrypt SSN
CREATE OR REPLACE FUNCTION encrypt_ssn(plaintext TEXT, key_id TEXT)
RETURNS BYTEA AS $$
DECLARE
    key_bytes BYTEA;
    iv BYTEA;
    encrypted BYTEA;
BEGIN
    -- Get key from key management system (simplified)
    key_bytes := get_encryption_key(key_id);
    
    -- Generate random IV
    iv := gen_random_bytes(12);
    
    -- Encrypt using AES-256-GCM
    encrypted := encrypt_iv(plaintext::BYTEA, iv, key_bytes, 'aes-gcm');
    
    -- Return IV || encrypted data
    RETURN iv || encrypted;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Function to decrypt SSN
CREATE OR REPLACE FUNCTION decrypt_ssn(ciphertext BYTEA, key_id TEXT)
RETURNS TEXT AS $$
DECLARE
    key_bytes BYTEA;
    iv BYTEA;
    encrypted BYTEA;
    decrypted BYTEA;
BEGIN
    key_bytes := get_encryption_key(key_id);
    
    -- Extract IV (first 12 bytes)
    iv := ciphertext[1:12];
    encrypted := ciphertext[13:];
    
    -- Decrypt
    decrypted := decrypt_iv(encrypted, iv, key_bytes, 'aes-gcm');
    
    RETURN decrypted::TEXT;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

-- Application-level encryption example
async fn save_user_ssn(user_id: Uuid, ssn: &str) -> Result<()> {
    let key_id = "key-2026-001";
    let encrypted = encrypt_ssn(ssn, key_id)?;
    
    sqlx_query!(
        "UPDATE users SET ssn_encrypted = $2, ssn_key_id = $3 WHERE id = $1",
        user_id,
        encrypted,
        key_id
    )
    .execute(&pool)
    .await?;
    
    Ok(())
}
```

---

## 9. Anti-Patterns

| Anti-Pattern | Specific Failure Mode | Prevention |
|--------------|----------------------|------------|
| **SELECT *** | Returns too much data, breaks projections | Always specify columns |
| **N+1 queries** | Performance degrades linearly with data | Use joins, DataLoader |
| **No indexes** | Full table scans, slow queries | Index strategy per query |
| **No connection limits** | Resource exhaustion, cascading failure | Connection pool with limits |
| **Storing files in DB** | Bloats database, slow backups | Use object storage (S3) |
| **No backups** | Data loss on corruption/failure | Automated, tested backups |
| **Hard deletes** | Can't audit, can't recover | Soft delete with deleted_at |
| **No validation** | Bad data, constraint violations | Validate at every boundary |
| **No foreign keys** | Orphaned records, data rot | Use constraints or application enforcement |
| **Schema drift** | Prod differs from migrations | All changes via migrations |

---

## Links

### Core Router
- `core/DECAPOD.md` - **Router and navigation charter (START HERE)**
- `core/ENGINEERING_EXCELLENCE.md` - Engineering standards

### Architecture (This Section)
- `architecture/CACHING.md` - Caching patterns
- `architecture/SECURITY.md` - Security architecture
- `architecture/OBSERVABILITY.md` - Data observability
- `architecture/CLOUD.md` - Cloud data services

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
- `methodology/DATA_MODELING.md` - Data modeling patterns