# CACHING.md - Caching Architecture (DENSE)

**Authority:** guidance (caching strategies, invalidation, and performance patterns)
**Layer:** Guides
**Binding:** No
**Scope:** caching patterns, cache levels, and invalidation strategies
**Non-goals:** specific cache implementations, cache-as-database patterns

---

## 1. Caching Principles

### 1.1 Cache Purpose
Cache is a **performance optimization**, not a:
- Source of truth
- Consistency mechanism
- Data storage layer
- Reliability guarantee

### 1.2 The Two Hard Problems
"There are only two hard things in Computer Science: cache invalidation and naming things."

**Design for invalidation first:**
- How will this cache entry be invalidated?
- What events trigger invalidation?
- How do we handle invalidation failures?
- What's the blast radius of stale data?

### 1.3 Cache Trade-offs Matrix

| Aspect | Cache Hit | Cache Miss | Stale Hit | Evicted Entry |
|--------|-----------|------------|-----------|---------------|
| Latency | Microseconds | Milliseconds | Milliseconds | Milliseconds |
| Throughput | High | Variable | Variable | Normal |
| Consistency | Stale | Fresh | Stale | N/A |
| Complexity | Low | Low | High | High |
| Cost | Low | High (origin load) | Low | Low |

### 1.4 Production Mindset
Before adding a cache, establish a performance budget and verify the cache is necessary:

- **Cache only when the system demands it:** If the system meets latency targets without a cache, adding one only introduces a failure mode. Measure first.
- **Stale data has a business cost:** The acceptable staleness window is a product decision, not an engineering default. A price shown 5 minutes late may be catastrophically wrong; a user's display name shown 5 minutes stale is harmless. Make this explicit.
- **A cache is a stateful dependency:** If the cache goes offline and the origin cannot absorb the resulting load, the cache has become load-bearing infrastructure — that is a fragile architecture. Design so the system degrades gracefully when the cache is cold or absent.
- **CDN vs application cache are different tools:** CDNs serve public, edge-delivered assets; distributed caches (Redis) handle session and application state. Using the wrong layer for the wrong data adds complexity and consistency bugs.
- **TTL is a fallback, not a strategy:** Time-based expiry is a safety net for when event-driven invalidation fails. For data with defined write paths, use explicit or event-driven invalidation and treat TTL as the last resort.
- **Measure total round-trip cost:** Serialization and deserialization often exceed the network round-trip for a direct DB read. Benchmark the full cache path before assuming it is faster.

---

## 2. Cache Levels

### 2.1 L1: In-Memory Cache Specification

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "L1InMemoryCacheConfig",
  "type": "object",
  "required": ["max_size", "eviction_policy"],
  "properties": {
    "max_size": {
      "type": "object",
      "properties": {
        "items": {"type": "integer", "description": "Max number of entries"},
        "bytes": {"type": "integer", "description": "Max bytes (if bounded by memory)"}
      }
    },
    "eviction_policy": {
      "type": "string",
      "enum": ["LRU", "LFU", "FIFO", "TTL", "Random", "ARC", "LRU_2"]
    },
    "ttl_seconds": {
      "type": "integer",
      "description": "Default TTL for all entries"
    },
    "initial_capacity": {
      "type": "integer",
      "description": "Initial bucket allocation"
    },
    "thread_safe": {
      "type": "boolean",
      "default": true
    },
    "statistics": {
      "type": "object",
      "properties": {
        "enabled": {"type": "boolean"},
        "hit_rate_window_seconds": {"type": "integer"}
      }
    }
  }
}
```

**Implementation Examples:**

```rust
// Rust: ConcurrentHashMap with LRU cache
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

struct LRUCache<K, V> {
    capacity: usize,
    cache: Mutex<HashMap<K, (V, Instant)>>,
    access_order: Mutex<Vec<K>>,
}

impl<K: Clone + Eq + std::hash::Hash, V: Clone> LRUCache<K, V> {
    fn new(capacity: usize) -> Self {
        Self {
            capacity,
            cache: Mutex::new(HashMap::with_capacity(capacity)),
            access_order: Mutex::new(Vec::with_capacity(capacity)),
        }
    }

    fn get(&self, key: &K) -> Option<V> {
        let mut cache = self.cache.lock().unwrap();
        let mut access_order = self.access_order.lock().unwrap();
        
        if let Some((value, _)) = cache.get(key) {
            // Move to end (most recently used)
            if let Some(pos) = access_order.iter().position(|k| k == key) {
                access_order.remove(pos);
                access_order.push(key.clone());
            }
            return Some(value.clone());
        }
        None
    }

    fn put(&self, key: K, value: V) {
        let mut cache = self.cache.lock().unwrap();
        let mut access_order = self.access_order.lock().unwrap();
        
        if cache.contains_key(&key) {
            // Update existing
            if let Some(pos) = access_order.iter().position(|k| k == &key) {
                access_order.remove(pos);
            }
        } else if cache.len() >= self.capacity {
            // Evict least recently used
            if let Some(lru_key) = access_order.first().cloned() {
                cache.remove(&lru_key);
                access_order.remove(0);
            }
        }
        
        cache.insert(key.clone(), (value, Instant::now()));
        access_order.push(key);
    }

    fn invalidate(&self, key: &K) {
        let mut cache = self.cache.lock().unwrap();
        let mut access_order = self.access_order.lock().unwrap();
        
        cache.remove(key);
        if let Some(pos) = access_order.iter().position(|k| k == key) {
            access_order.remove(pos);
        }
    }
}
```

```go
// Go: Thread-safe LRU cache with TTL
package cache

import (
    "container/list"
    "sync"
    "time"
)

type entry struct {
    key   string
    value interface{}
    ttl   time.Time
}

type TTLCache struct {
    mu       sync.Mutex
    capacity int
    items    map[string]*list.Element
    order    *list.List
    ttl      time.Duration
}

type Options struct {
    Capacity int
    TTL      time.Duration
}

func NewTTLCache(opts Options) *TTLCache {
    return &TTLCache{
        capacity: opts.Capacity,
        items:    make(map[string]*list.Element),
        order:    list.New(),
        ttl:      opts.TTL,
    }
}

func (c *TTLCache) Get(key string) (interface{}, bool) {
    c.mu.Lock()
    defer c.mu.Unlock()

    elem, exists := c.items[key]
    if !exists {
        return nil, false
    }

    e := elem.Value.(*entry)
    if time.Now().After(e.ttl) {
        c.removeElement(elem)
        return nil, false
    }

    // Move to front (most recently used)
    c.order.MoveToFront(elem)
    return e.value, true
}

func (c *TTLCache) Set(key string, value interface{}) {
    c.SetWithTTL(key, value, c.ttl)
}

func (c *TTLCache) SetWithTTL(key string, value interface{}, ttl time.Duration) {
    c.mu.Lock()
    defer c.mu.Unlock()

    if elem, exists := c.items[key]; exists {
        c.order.MoveToFront(elem)
        elem.Value.(*entry).value = value
        elem.Value.(*entry).ttl = time.Now().Add(ttl)
        return
    }

    // Add new entry
    if c.order.Len() >= c.capacity {
        c.removeOldest()
    }

    e := &entry{
        key:   key,
        value: value,
        ttl:   time.Now().Add(ttl),
    }
    elem := c.order.PushFront(e)
    c.items[key] = elem
}

func (c *TTLCache) Delete(key string) {
    c.mu.Lock()
    defer c.mu.Unlock()

    if elem, exists := c.items[key]; exists {
        c.removeElement(elem)
    }
}

func (c *TTLCache) removeOldest() {
    elem := c.order.Back()
    if elem != nil {
        c.removeElement(elem)
    }
}

func (c *TTLCache) removeElement(elem *list.Element) {
    c.order.Remove(elem)
    delete(c.items, elem.Value.(*entry).key)
}

func (c *TTLCache) Len() int {
    c.mu.Lock()
    defer c.mu.Unlock()
    return c.order.Len()
}
```

### 2.2 L2: Distributed Cache (Redis) Specification

```yaml
# Redis Cluster Configuration
RedisConfiguration:
  cluster_mode: true
  nodes:
    - host: redis-1.example.com
      port: 6379
      priority: 10
    - host: redis-2.example.com
      port: 6379
      priority: 5
    - host: redis-3.example.com
      port: 6379
      priority: 5
  
  replication:
    enabled: true
    replicas_per_master: 2
    min_replicas_to_write: 1
    repl_backup_mode: rdb
  
  persistence:
    rdb:
      enabled: true
      save_schedule: "900 1 300 100 60 10000"
      compression: yes
      checksum: yes
    aof:
      enabled: true
      fsync_strategy: everysec
      rewrite_percentage: 100
      rewrite_min_size: 64mb
  
  memory:
    maxmemory: 12gb
    maxmemory_policy: allkeys-lru
    maxmemory_samples: 5
  
  eviction_policy: lru
  
  connection:
    timeout: 5
    tcp_keepalive: 300
    max_clients: 10000
  
  slowlog:
    max_len: 128
    slowlog_log_slower_than: 10000
  
  latency_monitor_threshold: 100
```

**Redis Data Structure Selection:**

| Data Type | Use Case | Time Complexity | Max Size |
|-----------|----------|-----------------|----------|
| STRING | Session tokens, simple values | O(1) | 512MB |
| HASH | User profiles, objects | O(1) to O(N) | 2^32 fields |
| LIST | Queues, activity feeds | O(1) to O(N) | 2^32 items |
| SET | Tags, unique visitors | O(1) to O(N) | 2^32 members |
| ZSET | Rankings, priority queues | O(log N) | 2^32 members |
| BITMAP | Activity tracking, flags | O(1) to O(N) | 2^32 bits |
| HYPERLOGLOG | Unique counts | O(1) | 12KB |
| GEO | Location queries | O(log N) | 2^32 entries |

**Cache Key Schema:**

```json
{
  "CacheKeySchema": {
    "type": "object",
    "required": ["namespace", "version", "identifier"],
    "properties": {
      "namespace": {
        "type": "string",
        "pattern": "^[a-z][a-z0-9_]{1,30}$",
        "description": "Service or feature name"
      },
      "version": {
        "type": "string",
        "description": "Schema version for cache busting"
      },
      "identifier": {
        "type": "string",
        "description": "Entity-specific identifier(s)",
        "oneOf": [
          {"type": "string"},
          {"type": "array", "items": {"type": "string"}}
        ]
      },
      "qualifier": {
        "type": "string",
        "description": "Optional qualifier (e.g., 'v2', 'en')"
      }
    },
    "pattern": "{namespace}:{version}:{identifier}(:{qualifier})?"
  }
}
```

**Cache Key Examples:**

```
# Good cache keys
users:v1:12345                    # User with ID 12345
users:v1:12345:profile            # Profile of user 12345
products:v2:featured              # Featured products
session:v3:abc123def456          # Session token
rate_limit:v1:api:192.168.1.1     # Rate limit for IP
search:v2:products:category:shoes:page:1

# Bad cache keys
u:12345                           # Unclear namespace
data                             # No version, no identifier
my cache key with spaces          # Spaces, no structure
```

### 2.3 L3: CDN Configuration

```json
{
  "CDNConfiguration": {
    "provider": "cloudfront",
    "price_class": "PriceClass_All",
    "origin": {
      "domain_name": "api.example.com",
      "origin_path": "",
      "protocol": "https",
      "min_tls_version": "tls1.2",
      "ssl_protocols": ["TLSv1.2", "TLSv1.3"]
    },
    "caching": {
      "default_ttl": 86400,
      "max_ttl": 31536000,
      "min_ttl": 0,
      "forward_cookies": "none",
      "query_string_forwarding": "none",
      "compress": true,
      "allowed_http_methods": ["GET", "HEAD"]
    },
    "cache_policies": {
      "static_assets": {
        "path_pattern": "*.{jpg,jpeg,png,gif,webp,svg,css,js,woff,woff2}",
        "ttl": 31536000,
        "compress": true
      },
      "api_responses": {
        "path_pattern": "/api/*",
        "ttl": 0,
        "cache_policy": "Elemental-MediaStore"
      },
      "html_pages": {
        "path_pattern": "*.html",
        "ttl": 3600,
        "stale_while_revalidate": 60
      }
    },
    "geo_restriction": {
      "enabled": true,
      "restriction_type: whitelist",
      "countries": ["US", "CA", "GB"]
    },
    "security": {
      "waf_enabled": true,
      "shield_enabled": true,
      "https_only": true,
      "referer_validation": true
    }
  }
}
```

**Cache-Control Header Strategy:**

```
# Static assets - Cache long, immutable
Cache-Control: public, max-age=31536000, immutable

# User-specific content - No cache
Cache-Control: private, no-store, no-cache, must-revalidate

# API responses - Short cache with revalidation
Cache-Control: public, max-age=60, stale-while-revalidate=300

# Authenticated API - No cache
Cache-Control: private, no-cache

# Error responses - No cache
Cache-Control: no-store

# CDN refresh
Cache-Control: public, max-age=86400, s-maxage=3600
```

---

## 3. Caching Patterns

### 3.1 Cache-Aside (Lazy Loading)

```rust
// Cache-Aside Pattern Implementation
async fn get_user_profile(cache: &RedisCache, db: &Database, user_id: &str) -> Result<UserProfile> {
    let cache_key = format!("users:v1:{}:profile", user_id);
    
    // Step 1: Check cache
    if let Some(cached) = cache.get(&cache_key).await? {
        // Hit
        tracing::debug!(cache_key = %cache_key, "Cache hit");
        return serde_json::from_str(&cached);
    }
    
    // Step 2: Cache miss - fetch from DB
    tracing::debug!(cache_key = %cache_key, "Cache miss, fetching from DB");
    let profile = db.get_user_profile(user_id).await?;
    
    // Step 3: Store in cache (fire-and-forget with TTL)
    let serialized = serde_json::to_string(&profile)?;
    cache.set_ex(&cache_key, &serialized, 3600).await?; // 1 hour TTL
    
    Ok(profile)
}
```

**Cache-Aside Flow:**
```
Request → Check Cache → MISS → Fetch from DB → Store in Cache → Return
         ↓
        HIT → Return
```

### 3.2 Write-Through

```rust
async fn update_user_profile(
    cache: &RedisCache, 
    db: &Database, 
    user_id: &str, 
    updates: ProfileUpdate
) -> Result<UserProfile> {
    let cache_key = format!("users:v1:{}:profile", user_id);
    
    // Step 1: Write to database
    let profile = db.update_user_profile(user_id, updates).await?;
    
    // Step 2: Write to cache (synchronously)
    let serialized = serde_json::to_string(&profile)?;
    cache.set_ex(&cache_key, &serialized, 3600).await?;
    
    Ok(profile)
}
```

**Write-Through Flow:**
```
Request → Write to DB → Write to Cache → Return
```

### 3.3 Write-Behind (Write-Back)

```rust
struct WriteBehindCache {
    write_queue: Arc<Channel<CacheWrite>>,
    flush_interval: Duration,
}

impl WriteBehindCache {
    async fn write(&self, key: String, value: String) {
        self.write_queue.send(CacheWrite { key, value }).await;
    }
    
    async fn flush_loop(&self, db: Arc<Database>) {
        let mut interval = time::interval(self.flush_interval);
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    self.flush_pending_writes(&db).await;
                }
                Some(write) = self.write_queue.recv() => {
                    self.pending_writes.push(write);
                }
            }
        }
    }
}
```

**Write-Behind Flow:**
```
Request → Write to Cache → Return (async) → Background: Write to DB
```

### 3.4 Refresh-Ahead

```rust
async fn refresh_ahead_cache(
    cache: &RedisCache,
    db: &Database,
    key: &str,
    ttl: u64,
    refresh_threshold: f32, // e.g., 0.8 = refresh at 80% of TTL
) {
    // Check if TTL is below threshold
    if let Some(ttl_remaining) = cache.ttl(key).await {
        let total_ttl = cache.get_ttl(key).await.unwrap_or(ttl);
        if (ttl - ttl_remaining) as f32 / total_ttl as f32 >= refresh_threshold {
            // Background refresh
            let key_clone = key.to_string();
            tokio::spawn(async move {
                if let Ok(fresh_data) = db.fetch(&key_clone).await {
                    let _ = cache.set_ex(&key_clone, &fresh_data, ttl).await;
                }
            });
        }
    }
}
```

---

## 4. Cache Invalidation Strategies

### 4.1 TTL Configuration Schema

```json
{
  "TTLConfigurationSchema": {
    "type": "object",
    "required": ["category", "ttl_seconds", "reason"],
    "properties": {
      "category": {
        "type": "string",
        "description": "Cache category name"
      },
      "ttl_seconds": {
        "type": "integer",
        "minimum": 0,
        "maximum": 31536000
      },
      "jitter_percent": {
        "type": "number",
        "minimum": 0,
        "maximum": 50,
        "default": 0,
        "description": "Random jitter to prevent thundering herd"
      },
      "reason": {
        "type": "string",
        "description": "Why this TTL was chosen"
      },
      "staleness_acceptable": {
        "type": "boolean",
        "default": false,
        "description": "Is stale data acceptable for this category?"
      },
      "max_stale_seconds": {
        "type": "integer",
        "description": "Max acceptable staleness if allowed"
      }
    }
  }
}
```

**TTL Configuration Examples:**

```yaml
ttl_policies:
  - category: user_profile
    ttl_seconds: 3600
    jitter_percent: 10
    reason: "User profiles change infrequently, 1 hour freshness is acceptable"
    staleness_acceptable: true
    max_stale_seconds: 300
  
  - category: product_catalog
    ttl_seconds: 300
    jitter_percent: 5
    reason: "Price/availability changes should propagate within 5 minutes"
    staleness_acceptable: false
  
  - category: session_token
    ttl_seconds: 86400
    jitter_percent: 0
    reason: "Session tokens have explicit expiry matching token TTL"
    staleness_acceptable: false
  
  - category: feature_flags
    ttl_seconds: 60
    jitter_percent: 20
    reason: "Flag changes should take effect quickly for kill switches"
    staleness_acceptable: false
  
  - category: geo_location
    ttl_seconds: 604800
    jitter_percent: 0
    reason: "Geo data changes rarely, cache for 1 week"
    staleness_acceptable: true
    max_stale_seconds: 86400
```

### 4.2 Event-Driven Invalidation

```rust
// Event-driven cache invalidation
async fn handle_invalidation_event(
    cache: &RedisCache,
    event: &CacheInvalidationEvent,
) -> Result<()> {
    match event {
        CacheInvalidationEvent::EntityUpdated { entity_type, entity_id, .. } => {
            let key = format!("{}:v1:{}:profile", entity_type, entity_id);
            cache.delete(&key).await?;
            tracing::info!(key = %key, "Invalidated cache on entity update");
        }
        CacheInvalidationEvent::EntityDeleted { entity_type, entity_id } => {
            let key = format!("{}:v1:{}:profile", entity_type, entity_id);
            cache.delete(&key).await?;
        }
        CacheInvalidationEvent::BulkInvalidate { pattern } => {
            cache.delete_pattern(pattern).await?;
        }
        CacheInvalidationEvent::TagUpdated { tag, .. } => {
            // Invalidate all items with this tag
            let tag_key = format!("tag:{}:items", tag);
            if let Some(item_keys) = cache.smembers(&tag_key).await? {
                for key in item_keys {
                    cache.delete(&key).await?;
                }
            }
        }
    }
    Ok(())
}
```

**Pub/Sub Invalidation Pattern:**

```rust
// Publisher (when data changes)
async fn on_user_updated(user: &User) -> Result<()> {
    // Update database first
    db.update_user(user).await?;
    
    // Publish invalidation event
    redis.publish(
        "cache:invalidate",
        serde_json::json!({
            "event": "entity_updated",
            "entity_type": "user",
            "entity_id": user.id
        })
    ).await?;
    
    Ok(())
}

// Subscriber (invalidation worker)
async fn invalidation_worker(redis: RedisClient) {
    let mut subscriber = redis.subscribe("cache:invalidate").await;
    
    while let Some(msg) = subscriber.next().await {
        let event: InvalidationEvent = serde_json::from_str(&msg).unwrap();
        process_invalidation(&redis, &event).await;
    }
}
```

### 4.3 Version-Based Invalidation

```rust
// Version-based cache key management
struct VersionedCache {
    namespace: String,
    current_version: String,
}

impl VersionedCache {
    fn key(&self, identifier: &str) -> String {
        format!("{}:v{}:{}", self.namespace, self.current_version, identifier)
    }
    
    async fn invalidate_version(&self, cache: &RedisCache) -> Result<String> {
        // Generate new version
        let new_version = Uuid::new_v4().to_string()[..8].to_string();
        
        // The old version will naturally expire via TTL
        // New entries use the new version
        
        tracing::info!(
            old_version = %self.current_version,
            new_version = %new_version,
            "Cache version rotated"
        );
        
        self.current_version = new_version;
        Ok(new_version)
    }
}
```

---

## 5. Cache Stampede Prevention

### 5.1 Thundering Herd Problem

```
Problem: Cache expires at time T
         1000 requests arrive at exactly T
         All 1000 hit database simultaneously
         Database overload → cascade failure
```

### 5.2 Prevention Strategies

```rust
// Strategy 1: Per-Item TTL Jitter
fn calculate_jittered_ttl(base_ttl: u64, jitter_percent: f32) -> u64 {
    let jitter_range = (base_ttl as f32 * jitter_percent / 100.0) as u64;
    let jitter = rand::thread_rng().gen_range(0..=jitter_range);
    base_ttl + jitter
}

// Strategy 2: Probabilistic Early Expiration
async fn get_with_probabilistic_refresh(
    cache: &RedisCache,
    db: &Database,
    key: &str,
    base_ttl: u64,
) -> Result<String> {
    // Check if value exists
    if let Some(value) = cache.get(key).await? {
        // Check if should refresh early (probabilistic)
        let should_refresh = compute_early_refresh_probability(key, base_ttl)?;
        if should_refresh {
            // Spawn background refresh
            let key_clone = key.to_string();
            tokio::spawn(async move {
                if let Ok(fresh) = db.get(&key_clone).await {
                    let _ = cache.set_ex(&key_clone, &fresh, base_ttl).await;
                }
            });
        }
        return Ok(value);
    }
    
    // Normal cache miss handling
    get_or_compute(cache, db, key).await
}

fn compute_early_refresh_probability(key: &str, ttl: u64) -> Result<bool> {
    // XFetch algorithm: probability increases as TTL decreases
    let ttl_remaining = cache.ttl(key).await?.unwrap_or(0);
    let elapsed = ttl - ttl_remaining;
    
    // Probability formula: P = elapsed / (ttl * alpha)
    let alpha = 2.0;
    let probability = elapsed as f64 / (ttl as f64 * alpha);
    
    Ok(rand::random::<f64>() < probability)
}

// Strategy 3: Mutex-based single fetcher
use tokio::sync::Mutex;

struct MutexCache {
    locks: Arc<Mutex<HashMap<String, Arc<Mutex<()>>>>>,
}

impl MutexCache {
    async fn get_or_fetch<F, Fut>(&self, cache: &RedisCache, key: &str, fetcher: F) -> Result<String>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<String>>,
    {
        // Check cache first
        if let Some(value) = cache.get(key).await? {
            return Ok(value);
        }
        
        // Get or create lock for this key
        let lock = {
            let mut locks = self.locks.lock().await;
            locks.entry(key.to_string())
                .or_insert_with(|| Arc::new(Mutex::new(())))
                .clone()
        };
        
        // Only one request computes the value
        let _guard = lock.lock().await;
        
        // Double-check cache (another request may have populated it)
        if let Some(value) = cache.get(key).await? {
            return Ok(value);
        }
        
        let value = fetcher().await?;
        cache.set_ex(key, &value, 3600).await?;
        Ok(value)
    }
}
```

---

## 6. Redis Distributed Locking

### 6.1 Distributed Lock Schema

```json
{
  "DistributedLockSchema": {
    "type": "object",
    "required": ["name", "ttl_ms", "retry_count"],
    "properties": {
      "name": {
        "type": "string",
        "pattern": "^lock:[a-z0-9:_]+$"
      },
      "ttl_ms": {
        "type": "integer",
        "minimum": 1000,
        "maximum": 60000,
        "description": "Lock TTL in milliseconds"
      },
      "retry_count": {
        "type": "integer",
        "minimum": 0,
        "maximum": 10
      },
      "retry_delay_ms": {
        "type": "integer",
        "minimum": 10,
        "maximum": 5000
      },
      "extension_enabled": {
        "type": "boolean"
      },
      "extension_threshold_ms": {
        "type": "integer"
      }
    }
  }
}
```

**Distributed Lock Implementation:**

```rust
use tokio::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

struct DistributedLock {
    redis: RedisClient,
    name: String,
    ttl_ms: u64,
    value: String,
    acquired: AtomicBool,
}

impl DistributedLock {
    async fn acquire(&self) -> Result<bool> {
        // SET key value NX PX ttl_ms
        let result = redis::cmd("SET")
            .arg(&self.name)
            .arg(&self.value)
            .arg("NX")
            .arg("PX")
            .arg(self.ttl_ms)
            .query_async::<Option<String>>(&self.redis)
            .await?;
        
        let acquired = result.is_some();
        self.acquired.store(acquired, Ordering::SeqCst);
        Ok(acquired)
    }
    
    async fn release(&self) -> Result<()> {
        // Lua script to release only if we own the lock
        let script = r#"
            if redis.call("get", KEYS[1]) == ARGV[1] then
                return redis.call("del", KEYS[1])
            else
                return 0
            end
        "#;
        
        redis::Script::new(script)
            .key(&self.name)
            .arg(&self.value)
            .invoke_async(&self.redis)
            .await?;
        
        self.acquired.store(false, Ordering::SeqCst);
        Ok(())
    }
    
    async fn extend(&self, ttl_ms: u64) -> Result<bool> {
        // Lua script to extend only if we own the lock
        let script = r#"
            if redis.call("get", KEYS[1]) == ARGV[1] then
                return redis.call("pexpire", KEYS[1], ARGV[2])
            else
                return 0
            end
        "#;
        
        let result: i32 = redis::Script::new(script)
            .key(&self.name)
            .arg(&self.value)
            .arg(ttl_ms)
            .invoke_async(&self.redis)
            .await?;
        
        Ok(result == 1)
    }
}
```

---

## 7. Session Store Patterns

### 7.1 Session Store Schema

```json
{
  "SessionStoreSchema": {
    "type": "object",
    "required": ["session_id", "user_id", "created_at", "expires_at"],
    "properties": {
      "session_id": {
        "type": "string",
        "pattern": "^sess_[a-zA-Z0-9]{32,}$"
      },
      "user_id": {
        "type": "string"
      },
      "created_at": {
        "type": "string",
        "format": "date-time"
      },
      "last_accessed_at": {
        "type": "string",
        "format": "date-time"
      },
      "expires_at": {
        "type": "string",
        "format": "date-time"
      },
      "ip_address": {
        "type": "string"
      },
      "user_agent": {
        "type": "string"
      },
      "data": {
        "type": "object",
        "additionalProperties": true
      },
      "security": {
        "type": "object",
        "properties": {
          "mfa_verified": {"type": "boolean"},
          "trust_level": {"type": "string"},
          "credential_id": {"type": "string"}
        }
      }
    }
  }
}
```

**Session Storage in Redis:**

```rust
// Redis session storage
const SESSION_PREFIX = "session:v3:";
const SESSION_TTL: u64 = 86400; // 24 hours

async fn create_session(redis: &RedisClient, user_id: &str) -> Result<Session> {
    let session_id = format!("sess_{}", Uuid::new_v4().to_string());
    let now = Utc::now();
    let session = Session {
        session_id: session_id.clone(),
        user_id: user_id.to_string(),
        created_at: now,
        last_accessed_at: now,
        expires_at: now + Duration::hours(24),
        data: HashMap::new(),
        security: SecurityContext::default(),
    };
    
    let key = format!("{}{}", SESSION_PREFIX, session_id);
    let value = serde_json::to_string(&session)?;
    
    redis.set_ex(&key, &value, SESSION_TTL).await?;
    
    Ok(session)
}

async fn get_session(redis: &RedisClient, session_id: &str) -> Result<Option<Session>> {
    let key = format!("{}{}", SESSION_PREFIX, session_id);
    let value: Option<String> = redis.get(&key).await?;
    
    match value {
        Some(v) => {
            let mut session: Session = serde_json::from_str(&v)?;
            session.last_accessed_at = Utc::now();
            
            // Touch TTL on access (sliding expiration)
            redis.expire(&key, SESSION_TTL).await?;
            
            Ok(Some(session))
        }
        None => Ok(None)
    }
}

async fn delete_session(redis: &RedisClient, session_id: &str) -> Result<()> {
    let key = format!("{}{}", SESSION_PREFIX, session_id);
    redis.del(&key).await?;
    Ok(())
}
```

---

## 8. Monitoring & Alerting

### 8.1 Cache Metrics Schema

```json
{
  "CacheMetricsSchema": {
    "type": "object",
    "metrics": [
      {
        "name": "cache_hits_total",
        "type": "counter",
        "labels": ["cache_name", "cache_layer"],
        "description": "Total number of cache hits"
      },
      {
        "name": "cache_misses_total",
        "type": "counter",
        "labels": ["cache_name", "cache_layer"],
        "description": "Total number of cache misses"
      },
      {
        "name": "cache_hit_rate",
        "type": "gauge",
        "labels": ["cache_name"],
        "description": "Current hit rate percentage"
      },
      {
        "name": "cache_operations_total",
        "type": "counter",
        "labels": ["cache_name", "operation", "status"],
        "description": "Total cache operations by type and status"
      },
      {
        "name": "cache_operation_duration_seconds",
        "type": "histogram",
        "labels": ["cache_name", "operation"],
        "description": "Cache operation latency"
      },
      {
        "name": "cache_memory_used_bytes",
        "type": "gauge",
        "labels": ["cache_name"],
        "description": "Memory used by cache"
      },
      {
        "name": "cache_entries",
        "type": "gauge",
        "labels": ["cache_name"],
        "description": "Number of entries in cache"
      },
      {
        "name": "cache_evictions_total",
        "type": "counter",
        "labels": ["cache_name", "eviction_policy"],
        "description": "Total evictions"
      },
      {
        "name": "cache_stale_responses_total",
        "type": "counter",
        "labels": ["cache_name"],
        "description": "Responses served with stale data"
      }
    ]
  }
}
```

**Cache Alert Rules:**

```yaml
alerts:
  - name: CacheHitRateTooLow
    expr: cache_hit_rate < 0.8
    for: 10m
    severity: warning
    annotations:
      summary: "Cache hit rate is below 80%"
      description: "Current hit rate: {{ $value | humanizePercentage }}"
    
  - name: CacheMemoryUsageHigh
    expr: cache_memory_used_bytes / cache_memory_limit_bytes > 0.85
    for: 5m
    severity: warning
    annotations:
      summary: "Cache memory usage above 85%"
    
  - name: CacheEvictionsSpike
    expr: rate(cache_evictions_total[5m]) > 100
    for: 2m
    severity: warning
    annotations:
      summary: "Cache eviction rate spike detected"
    
  - name: CacheConnectionErrors
    expr: rate(cache_operations_total{status="error"}[5m]) > 0
    for: 1m
    severity: critical
    annotations:
      summary: "Cache connection errors detected"
```

---

## 9. Anti-Patterns

| Anti-Pattern | Specific Failure Mode | Prevention |
|--------------|----------------------|------------|
| **Cache as database** | Data loss on cache failure | Use cache as optimization only, DB is source of truth |
| **No TTL** | Memory grows forever, OOM | Always set TTL on all entries |
| **No invalidation** | Stale data served indefinitely | Event-driven invalidation on write |
| **Over-caching** | Complexity explosion, invalidation nightmares | Cache deliberately, not everything |
| **Cache bypass** | Origin overload on cache miss | Warm cache, use mutex pattern |
| **Large objects** | Serialization cost, memory pressure | Cache small, frequently accessed items |
| **No monitoring** | Silent performance degradation | Track hit rate, latency, memory |
| **Single cache server** | SPOF for performance | Redis Cluster, multi-AZ |
| **Synchronous cache write** | Write latency increase | Async write-through, write-behind |
| **Ignoring serialization cost** | Cache operation slower than DB | Benchmark full path |

---

## Links

### Core Router
- `core/DECAPOD.md` - **Router and navigation charter (START HERE)**
- `core/ENGINEERING_EXCELLENCE.md` - Engineering standards

### Architecture (This Section)
- `architecture/DATA.md` - Data architecture
- `architecture/MEMORY.md` - Memory management
- `architecture/CONCURRENCY.md` - Concurrent cache access
- `architecture/WEB.md` - HTTP caching

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
- `methodology/CACHING_BEST_PRACTICES.md` - Caching patterns