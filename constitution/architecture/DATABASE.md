# DATABASE.md - Database Architecture

**Authority:** guidance (comprehensive database patterns with exact schemas, queries, and configurations)
**Layer:** Architecture
**Binding:** No
**Scope:** SQL, NoSQL, time-series databases with exact specifications for pre-inference context

---

## 1. SQL Databases

### 1.1 PostgreSQL

#### Connection Pooling (PgBouncer)
```ini
; pgbouncer.ini
[databases]
; Database alias = connection string
production = host=postgres-primary port=5432 dbname=app
replica = host=postgres-replica1 port=5432 dbname=app

[pgbouncer]
listen_addr = 0.0.0.0
listen_port = 6432
auth_type = md5
auth_file = /etc/pgbouncer/userlist.txt
pool_mode = transaction
max_client_conn = 1000
default_pool_size = 25
min_pool_size = 5
reserve_pool_size = 5
reserve_pool_timeout = 3
max_db_connections = 100
log_connections = 0
log_disconnections = 0
log_pooler_errors = 1
server_reset_query = DISCARD ALL
server_check_delay = 30
server_lifetime = 3600
server_idle_timeout = 600
query_timeout = 30
query_wait_timeout = 30
client_idle_timeout = 0
```

#### Connection String Patterns
```yaml
# Standard connection
postgresql://user:password@localhost:5432/mydb

# With SSL
postgresql://user:password@localhost:5432/mydb?sslmode=require

# Connection pool (PgBouncer)
postgresql://user:password@localhost:6432/mydb

# Multiple hosts (candidates)
postgresql://user:password@primary:5432,replica1:5432,mreplica2:5432/mydb?target_session_attrs=any

# Kubernetes service
postgresql://user:password@postgres.production.svc.cluster.local:5432/mydb
```

#### Index Patterns
```sql
-- B-tree (default, most common)
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_orders_user_id ON orders(user_id);
CREATE INDEX idx_orders_status ON orders(status) WHERE status != 'completed';

-- Composite index (column order matters!)
-- For: WHERE status = 'pending' AND created_at > '2024-01-01'
-- Good: index on (status, created_at) - equality first, range second
CREATE INDEX idx_orders_status_created ON orders(status, created_at);

-- Partial index (smaller, faster)
CREATE INDEX idx_orders_pending ON orders(created_at) 
WHERE status = 'pending';

-- GIN index for JSONB
CREATE INDEX idx_users_metadata ON users USING GIN(metadata);

-- GiST index for full-text search
CREATE INDEX idx_posts_content_fts ON posts USING GIN(to_tsvector('english', content));

-- Covering index (includes all needed columns)
CREATE INDEX idx_orders_covering ON orders(user_id, created_at) 
INCLUDE (total, status);

-- Index with ILIKE (use pg_trgm for pattern matching)
CREATE EXTENSION IF NOT EXISTS pg_trgm;
CREATE INDEX idx_users_name_trgm ON users USING GIN(name gin_trgm_ops);
```

#### Query Patterns

##### Common Table Expression (CTE)
```sql
-- Recursive CTE for hierarchical data
WITH RECURSIVE org_tree AS (
    -- Base case: top-level managers
    SELECT id, name, manager_id, 1 AS depth
    FROM employees
    WHERE manager_id IS NULL
    
    UNION ALL
    
    -- Recursive case: employees under managers
    SELECT e.id, e.name, e.manager_id, ot.depth + 1
    FROM employees e
    INNER JOIN org_tree ot ON e.manager_id = ot.id
    WHERE ot.depth < 10  -- Prevent infinite recursion
)
SELECT * FROM org_tree ORDER BY depth, name;

-- Data migration with CTE
WITH updated AS (
    UPDATE products 
    SET price = price * 1.1
    WHERE category = 'electronics'
    RETURNING id, price
)
INSERT INTO price_history (product_id, old_price, new_price, changed_at)
SELECT id, price / 1.1, price, NOW()
FROM updated;
```

##### Window Functions
```sql
-- Running total
SELECT 
    date,
    amount,
    SUM(amount) OVER (ORDER BY date) AS running_total
FROM transactions;

-- Partition by customer, running total per customer
SELECT 
    customer_id,
    date,
    amount,
    SUM(amount) OVER (
        PARTITION BY customer_id 
        ORDER BY date
        ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW
    ) AS customer_running_total
FROM orders;

-- Percent of total
SELECT 
    category,
    SUM(amount) AS total,
    SUM(amount) / SUM(SUM(amount)) OVER () * 100 AS percent_of_total
FROM sales
GROUP BY category;

-- Row number, rank, dense rank
SELECT 
    name,
    score,
    ROW_NUMBER() OVER (ORDER BY score DESC) AS row_num,
    RANK() OVER (ORDER BY score DESC) AS rank,
    DENSE_RANK() OVER (ORDER BY score DESC) AS dense_rank
FROM leaderboard;

-- Lag and Lead
SELECT 
    month,
    revenue,
    LAG(revenue, 1) OVER (ORDER BY month) AS prev_month,
    LEAD(revenue, 1) OVER (ORDER BY month) AS next_month,
    revenue - LAG(revenue, 1) OVER (ORDER BY month) AS mom_change
FROM monthly_revenue;
```

##### JSONB Operations
```sql
-- Create JSONB
SELECT jsonb_build_object(
    'name', name,
    'email', email,
    'roles', jsonb_build_array('user')
) FROM users WHERE id = 1;

-- Query JSONB
SELECT * FROM events 
WHERE metadata->>'action' = 'purchase';

SELECT * FROM events 
WHERE metadata @> '{"user_id": 123}';

SELECT * FROM events 
WHERE metadata ? 'subscription';

-- Update JSONB
UPDATE users 
SET metadata = jsonb_set(
    metadata,
    '{theme}',
    '"dark"'
)
WHERE id = 1;

-- Add to JSONB array
UPDATE users 
SET metadata = jsonb_insert(
    metadata,
    '{notifications, 0}',
    '"email"'
)
WHERE id = 1;

-- JSONB aggregation
SELECT 
    user_id,
    jsonb_agg(event_type) AS event_types,
    jsonb_object_agg(event_type, COUNT(*)) AS event_counts
FROM user_events
GROUP BY user_id;

-- JSONB path query
SELECT * FROM orders
WHERE data @> '{"shipping_address": {"country": "US"}}';
```

### 1.2 MySQL

#### Configuration (my.cnf)
```ini
[mysqld]
# Connection settings
max_connections = 500
wait_timeout = 600
interactive_timeout = 600

# InnoDB settings
innodb_buffer_pool_size = 80G
innodb_buffer_pool_instances = 8
innodb_log_file_size = 4G
innodb_log_files_in_group = 3
innodb_flush_log_at_trx_commit = 1
innodb_flush_method = O_DIRECT
innodb_file_per_table = 1
innodb_io_capacity = 4000
innodb_io_capacity_max = 8000

# Query cache (MySQL 8.0 removed this, but for older versions)
query_cache_type = 0
query_cache_size = 0

# Logging
slow_query_log = 1
slow_query_log_file = /var/log/mysql/slow.log
long_query_time = 1
log_queries_not_using_indexes = 0

# Character set
character_set_server = utf8mb4
collation_server = utf8mb4_unicode_ci

# SSL
require_secure_transport = ON
```

#### Common Patterns
```sql
-- UPSERT (MySQL 8.0+)
INSERT INTO users (id, email, name)
VALUES (1, 'test@example.com', 'Test')
ON DUPLICATE KEY UPDATE
    email = VALUES(email),
    name = VALUES(name),
    updated_at = NOW();

-- Multiple upsert
INSERT INTO items (sku, quantity, price)
VALUES ('SKU001', 10, 29.99), ('SKU002', 5, 49.99)
AS new
ON DUPLICATE KEY UPDATE
    quantity = new.quantity,
    price = new.price;

-- Window functions (MySQL 8.0+)
SELECT 
    customer_id,
    order_date,
    total,
    SUM(total) OVER (
        PARTITION BY customer_id 
        ORDER BY order_date
    ) AS running_total
FROM orders;

-- CTEs (MySQL 8.0+)
WITH recent_orders AS (
    SELECT customer_id, MAX(order_date) AS last_order
    FROM orders
    GROUP BY customer_id
)
SELECT c.*, ro.last_order
FROM customers c
JOIN recent_orders ro ON c.id = ro.customer_id;
```

---

## 2. NoSQL Databases

### 2.1 MongoDB

#### Document Schema Patterns
```javascript
// User document
{
  "_id": ObjectId("..."),
  "email": "user@example.com",
  "name": {
    "first": "John",
    "last": "Doe"
  },
  "roles": ["admin", "user"],
  "profile": {
    "avatar": "https://...",
    "bio": "Engineer",
    "social": {
      "twitter": "@johndoe",
      "github": "johndoe"
    }
  },
  "preferences": {
    "theme": "dark",
    "notifications": {
      "email": true,
      "push": false
    }
  },
  "createdAt": ISODate("2024-01-15T10:30:00Z"),
  "updatedAt": ISODate("2024-01-15T10:30:00Z"),
  "lastLoginAt": ISODate("2024-01-20T14:22:00Z"),
  "status": "active"  // active, suspended, deleted
}

// Order document (referencing user)
{
  "_id": ObjectId("..."),
  "orderNumber": "ORD-2024-00001",
  "userId": ObjectId("..."),
  "items": [
    {
      "sku": "SKU001",
      "name": "Product Name",
      "quantity": 2,
      "unitPrice": 29.99,
      "total": 59.98
    }
  ],
  "shippingAddress": {
    "street": "123 Main St",
    "city": "New York",
    "state": "NY",
    "zip": "10001",
    "country": "US"
  },
  "totals": {
    "subtotal": 59.98,
    "tax": 5.40,
    "shipping": 10.00,
    "total": 75.38
  },
  "status": "pending",  // pending, processing, shipped, delivered, cancelled
  "createdAt": ISODate("2024-01-15T10:30:00Z"),
  "updatedAt": ISODate("2024-01-15T10:30:00Z")
}
```

#### Index Patterns
```javascript
// Single field index
db.users.createIndex({ "email": 1 }, { unique: true });
db.orders.createIndex({ "userId": 1 });
db.orders.createIndex({ "status": 1, "createdAt": -1 });

// Compound index (field order matters!)
// For: db.orders.find({ status: "pending" }).sort({ createdAt: -1 })
db.orders.createIndex({ "status": 1, "createdAt": -1 });

// Text index
db.posts.createIndex({ "title": "text", "content": "text" });

// Wildcard index (dynamic fields)
db.logs.createIndex({ "meta.$**": 1 });

// Geospatial index
db.places.createIndex({ "location": "2dsphere" });
db.places.find({
  location: {
    $near: {
      $geometry: { type: "Point", coordinates: [-73.97, 40.77] },
      $maxDistance: 1000  // meters
    }
  }
});

// Partial index
db.orders.createIndex(
  { "createdAt": 1 },
  { 
    partialFilterExpression: { "status": "pending" },
    expireAfterSeconds: 3600 * 24 * 30  // TTL index
  }
);

// Covered index
db.orders.createIndex(
  { "userId": 1, "status": 1 },
  { name: "user_status_covering", partialFilterExpression: { "status": { $exists: true } } }
);
```

#### Aggregation Pipeline
```javascript
// Pipeline stages: $match, $group, $sort, $limit, $project, $lookup, $unwind, $facet

// Example 1: User order summary with top products
db.orders.aggregate([
  // Stage 1: Filter
  { $match: { 
      "createdAt": { $gte: ISODate("2024-01-01") },
      "status": { $in: ["delivered", "shipped"] }
    }
  },
  // Stage 2: Unwind items array
  { $unwind: "$items" },
  // Stage 3: Group by user
  { $group: {
      _id: "$userId",
      totalSpent: { $sum: "$items.total" },
      orderCount: { $sum: 1 },
      products: { $addToSet: "$items.sku" }
    }
  },
  // Stage 4: Add computed fields
  { $addFields: {
      averageOrderValue: { $divide: ["$totalSpent", "$orderCount"] }
    }
  },
  // Stage 5: Sort and limit
  { $sort: { totalSpent: -1 } },
  { $limit: 10 },
  // Stage 6: Lookup user details
  { $lookup: {
      from: "users",
      localField: "_id",
      foreignField: "_id",
      as: "user"
    }
  },
  { $unwind: "$user" },
  // Stage 7: Project final shape
  { $project: {
      _id: 0,
      userId: "$_id",
      userName: "$user.name",
      userEmail: "$user.email",
      totalSpent: 1,
      orderCount: 1,
      averageOrderValue: { $round: ["$averageOrderValue", 2] },
      uniqueProducts: { $size: "$products" }
    }
  }
]);

// Example 2: Time series bucketing
db.events.aggregate([
  { $match: { "type": "pageview" } },
  { $group: {
      _id: {
        page: "$page",
        hour: { $dateToString: { format: "%Y-%m-%d %H:00", date: "$timestamp" } }
      },
      views: { $sum: 1 },
      uniqueUsers: { $addToSet: "$userId" }
    }
  },
  { $addFields: {
      uniqueUserCount: { $size: "$uniqueUsers" }
    }
  },
  { $sort: { "_id.hour": 1 } }
]);

// Example 3: Facet for multiple aggregations
db.orders.aggregate([
  { $match: { "createdAt": { $gte: ISODate("2024-01-01") } } },
  { $facet: {
      byStatus: [
        { $group: { _id: "$status", count: { $sum: 1 } } }
      ],
      byDay: [
        { $group: {
            _id: { $dateToString: { format: "%Y-%m-%d", date: "$createdAt" } },
            count: { $sum: 1 },
            total: { $sum: "$totals.total" }
          }
        }
      ],
      topUsers: [
        { $group: { _id: "$userId", total: { $sum: "$totals.total" } } },
        { $sort: { total: -1 } },
        { $limit: 5 }
      ]
    }
  }
]);
```

### 2.2 Redis

#### Data Structures and Commands
```redis
# String (most common)
SET user:123:token "abc123" EX 3600
GET user:123:token
SETNX user:123:token "abc123"  # Set if not exists (returns 1 if set)

# String with counter
INCR pageviews:2024:01:15
INCRBY pageviews:2024:01:15 100
DECR pageviews:2024:01:15
INCRBYFLOAT price:SKU001 0.50

# Hash (like dict/object)
HSET user:123 name "John" email "john@example.com" role "admin"
HGET user:123 name
HGETALL user:123
HMGET user:123 name email
HINCRBY user:123 login_count 1
HKEYS user:123
HVALS user:123
HEXISTS user:123 email  # Returns 1 if exists

# List (ordered, can have duplicates)
LPUSH notifications:123 "New order" "Payment received"
RPUSH notifications:123 "Shipment dispatched"
LRANGE notifications:123 0 -1  # Get all
LLEN notifications:123
LPOP notifications:123
RPOP notifications:123
LTRIM notifications:123 0 99  # Keep only first 100

# Set (unordered, unique)
SADD user:123:roles "admin" "user"
SMEMBERS user:123:roles
SISMEMBER user:123:roles "admin"  # Returns 1 if member
SREM user:123:roles "guest"
SUNION user:123:roles user:456:roles  # Union of sets
SINTER user:123:permissions admin:permissions  # Intersection
SCARD user:123:roles  # Count

# Sorted Set (leaderboards, priority queues)
ZADD leaderboard:2024 1000 "player1" 1500 "player2" 1200 "player3"
ZREVRANGE leaderboard:2024 0 9 WITHSCORES  # Top 10
ZRANGE leaderboard:2024 0 9 WITHSCORES  # Bottom 10
ZINCRBY leaderboard:2024 100 "player1"  # Increment score
ZRANK leaderboard:2024 "player1"  # Get rank (0-indexed)
ZREVRANK leaderboard:2024 "player1"  # Get rank (descending)
ZSCORE leaderboard:2024 "player1"  # Get score
ZRANGEBYSCORE leaderboard:2024 1000 2000  # By score range

# Bitmap (efficient for boolean flags)
SETBIT user:123:daily:login:2024:01:15 0 1  # Set bit 0 to 1
GETBIT user:123:daily:login:2024:01:15 0  # Get bit 0
BITCOUNT user:123:daily:login:2024:01:15  # Count set bits

# HyperLogLog (cardinality estimation)
PFADD pageviews:2024:01:15 "192.168.1.1" "192.168.1.2"
PFCOUNT pageviews:2024:01:15  # Approximate unique count

# Geospatial
GEOADD locations:user -122.4194 37.7749 "user:123"
GEOPOS locations:user "user:123"  # Get position
GEODIST locations:user "user:123" "user:456" km  # Distance
GEORADIUS locations:user -122.4194 37.7749 10 km  # Search radius
GEOSEARCH locations:user FROMLONLAT -122.4194 37.7749 BYRADIUS 10 km WITHDIST
```

#### Patterns
```redis
-- Rate limiting
-- Window: 100 requests per minute per IP
-- Key: rate:ip:2024:01:15:10:30 (minute granularity)
-- Lua script for atomicity:
local key = KEYS[1]
local limit = tonumber(ARGV[1])
local window = tonumber(ARGV[2])
local current = tonumber(redis.call('GET', key) or '0')
if current >= limit then
    return 0
end
current = redis.call('INCR', key)
if current == 1 then
    redis.call('EXPIRE', key, window)
end
return current

-- Distributed lock
-- SET lock:resource_name unique_value NX EX 30
SET lock:order:123 unique_token NX EX 30
-- Release: check value and delete (must be atomic, use Lua)
if redis.call("GET", KEYS[1]) == ARGV[1] then
    return redis.call("DEL", KEYS[1])
else
    return 0
end

-- Cache with semaphore
SETNX cache:hot:data 1  -- Acquire semaphore
EXPIRE cache:hot:data 10  -- Auto-release
-- If SETNX returns 0, another process is updating

-- Pub/Sub channels
PUBLISH user:123:notifications "New message"
SUBSCRIBE user:123:notifications
PSUBSCRIBE user:123:*  # Pattern subscription

-- Streams (event sourcing, message queues)
XADD stream:orders "*" user-id "123" total "75.38"
XREAD STREAMS stream:orders $  # Read new
XREAD STREAMS stream:orders 0-0  # Read all
XRANGE stream:orders 0-0 + COUNT 10
XGROUP CREATE stream:orders consumers $  # Consumer group
XREADGROUP GROUP consumers worker1 STREAMS stream:orders >
```

---

## 3. Time-Series Databases

### 3.1 TimescaleDB (PostgreSQL Extension)

```sql
-- Create hypertable (partitioned by time)
SELECT create_hypertable('measurements', 'time', 
    chunk_time_interval => INTERVAL '1 day',
    migrate_data => true
);

-- Hypertable with additional partitioning
SELECT create_hypertable('device_readings', 'time',
    chunk_time_interval => INTERVAL '1 hour',
    partitioning_column => 'device_id',
    number_partitions => 4,
    migrate_data => true
);

-- Create index on hypertable
CREATE INDEX ON measurements (device_id, time DESC);

-- Continuous aggregate (materialized view)
CREATE MATERIALIZED VIEW hourly_stats
WITH (timescaledb.continuous) AS
SELECT 
    time_bucket('1 hour', time) AS hour,
    device_id,
    AVG(temperature) AS avg_temp,
    MIN(temperature) AS min_temp,
    MAX(temperature) AS max_temp,
    COUNT(*) AS reading_count
FROM measurements
GROUP BY 1, 2
WITH NO DATA;

-- Refresh policy
SELECT add_continuous_aggregate_policy('hourly_stats',
    start_offset => INTERVAL '3 hours',
    end_offset => INTERVAL '1 hour',
    schedule_interval => INTERVAL '1 hour'
);

-- Compression policy
ALTER TABLE measurements SET (
    timescaledb.compress,
    timescaledb.compress_segmentby = 'device_id'
);

SELECT add_compression_policy('measurements', INTERVAL '7 days');

-- Retention policy
SELECT add_retention_policy('measurements', INTERVAL '30 days');

-- Query with time_bucket
SELECT 
    time_bucket('5 minutes', time) AS interval,
    device_id,
    AVG(sensor_value) AS avg_value,
    -- Percentiles
    PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY sensor_value) AS median,
    PERCENTILE_CONT(0.95) WITHIN GROUP (ORDER BY sensor_value) AS p95,
    PERCENTILE_CONT(0.99) WITHIN GROUP (ORDER BY sensor_value) AS p99
FROM measurements
WHERE time >= NOW() - INTERVAL '1 day'
    AND device_id = 'sensor-001'
GROUP BY 1, 2
ORDER BY 1;

-- Gap filling
SELECT 
    time_bucket('5 minutes', time) AS interval,
    LOCF(AVG(sensor_value)) AS value  -- Last observation carried forward
FROM measurements
WHERE device_id = 'sensor-001'
    AND time >= NOW() - INTERVAL '1 day'
GROUP BY 1
ORDER BY 1;
```

---

## 4. Query Optimization

### 4.1 EXPLAIN Analysis

```sql
-- Basic explain
EXPLAIN SELECT * FROM orders WHERE user_id = 123;

-- With costs
EXPLAIN (ANALYZE, COSTS, VERBOSE, BUFFERS, FORMAT TEXT)
SELECT * FROM orders WHERE status = 'pending' ORDER BY created_at DESC;

-- JSON format for programmatic analysis
EXPLAIN (FORMAT JSON)
SELECT * FROM orders WHERE user_id = 123;

-- Key things to look for:
-- - seq scan (bad for large tables)
-- - high estimated rows vs actual (outdated stats)
-- - high actual rows vs estimated (underestimation)
-- - Nested Loop (can be bad with large outer sets)
-- - Hash Join vs Merge Join (hash usually better for small sets)
```

### 4.2 Performance Patterns

```sql
-- Bulk insert (batch)
INSERT INTO orders (user_id, total)
SELECT user_id, SUM(total)
FROM cart_items
GROUP BY user_id
WHERE created_at > NOW() - INTERVAL '1 hour';

-- Partition pruning example
-- For: SELECT * FROM orders WHERE created_at >= '2024-01-01' AND created_at < '2024-01-02'
-- PostgreSQL will only scan the partition for that day

-- WITH CHECK OPTION for views
CREATE VIEW active_users AS
SELECT * FROM users WHERE status = 'active'
WITH LOCAL CHECK OPTION;

-- Materialized view refresh
REFRESH MATERIALIZED VIEW CONCURRENTLY hourly_stats;

-- Advisory lock for coordination
SELECT pg_advisory_lock(12345);  -- Lock
SELECT pg_advisory_unlock(12345);  -- Unlock
SELECT pg_try_advisory_lock(12345);  -- Non-blocking lock
```

---

## 5. Decision Matrix

### 5.1 When to Use Which Database

| Use Case | Recommended | Why |
|----------|-------------|-----|
| User data, transactions | PostgreSQL | ACID, complex queries, JSONB |
| Read-heavy, caching | Redis | In-memory, rich data structures |
| Document storage | MongoDB | Flexible schema, nested docs |
| Time-series metrics | TimescaleDB | Automatic partitioning, compression |
| Full-text search | Elasticsearch | Optimized for search, relevance |
| Graph relationships | Neo4j | Native graph traversal |
| Key-value, sessions | Redis | Fast, TTL support |
| Analytics, OLAP | ClickHouse/Redshift | Columnar, massive parallelism |
| Search, facets | Elasticsearch/Meilisearch | Ranking, filters, autocomplete |

### 5.2 Anti-Patterns

```yaml
# ❌ Don't use NoSQL when you need ACID transactions
# MongoDB's transactions are slower than PostgreSQL

# ❌ Don't embed everything in MongoDB
# Bad: Orders with embedded customer, items, shipping, payment
# If customer info changes, need to update all orders
# Better: Reference by ID, use $lookup when needed

# ❌ Don't over-index in MongoDB
# Each index consumes memory and slows writes
# Profile with explain() before adding

# ❌ Don't use Redis as primary data store without persistence
# AOF + RDB for durability, or accept data loss risk

# ❌ Don't store large blobs in PostgreSQL
# Use S3 + store URL in database
# Exception: Files under 1MB that are accessed frequently

# ❌ Don't use single-document MongoDB for many-to-many
# Use junction collections or array of refs with $lookup
```

---

## Links

### Core Router
- `core/DECAPOD.md` - **Router and navigation charter (START HERE)**
- `core/ENGINEERING_EXCELLENCE.md` - Engineering standards

### Architecture (This Section)
- `architecture/KUBERNETES.md` - Database StatefulSets, persistent volumes
- `architecture/CACHING.md` - Cache invalidation patterns
- `architecture/MESSAGING.md` - Event-driven database updates
- `architecture/CLOUD.md` - Managed database services

### Authority (Constitution Layer)
- `specs/INTENT.md` - **Methodology contract (READ FIRST)**
- `specs/SYSTEM.md` - System definition and authority doctrine
- `specs/SECURITY.md` - Security doctrine

### Interface Contracts
- `interfaces/CLAIMS.md` - Promises ledger
- `interfaces/CONTROL_PLANE.md` - Agent sequencing patterns
- `interfaces/STORE_MODEL.md` - State management contracts

### Methodology
- `methodology/ARCHITECTURE.md` - Architecture decision methodology
- `methodology/CI_CD.md` - Database migration CI/CD

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2024-01-16 | Initial comprehensive database reference |