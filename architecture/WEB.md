# WEB.md - Web Architecture (DENSE)

**Authority:** guidance (web protocols, API design, and stateless service patterns)
**Layer:** Guides
**Binding:** No
**Scope:** HTTP protocols, API patterns, and web service architecture
**Non-goals:** specific frameworks, frontend implementation details

---

## 1. Web Architecture Principles

### 1.1 Statelessness
**HTTP is stateless.** Server treats each request independently.
- Scalability: Any server can handle any request
- Reliability: No server affinity required
- Simplicity: No session state to manage

**State Management Options:**
| Storage | Scope | Lifetime | Security |
|---------|-------|----------|----------|
| JWT Token | Client | Expiry (15min) | Signed, encrypted |
| Refresh Token | Client + Server | 7 days | Rotated on use |
| HttpOnly Cookie | Client | Session | HttpOnly, Secure |
| Server Session | Server + Redis | Configurable | In-memory |

### 1.2 HTTP Protocol Versions

```json
{
  "HTTPVersions": {
    "HTTP/1.1": {
      "features": [
        "Persistent connections (keep-alive)",
        "Pipelining (partial support)",
        "Chunked transfer encoding",
        "Byte serving",
        "Compression (optional)"
      ],
      "limitations": [
        "Head-of-line blocking",
        "No multiplexing",
        "Plaintext headers"
      ],
      "use_when": "Legacy systems, simple APIs"
    },
    "HTTP/2": {
      "features": [
        "Multiplexing (multiple streams)",
        "Header compression (HPACK)",
        "Server push",
        "Binary framing",
        "Stream prioritization"
      ],
      "improvements": [
        "~40% header size reduction",
        "No head-of-line blocking",
        "Parallel requests on single connection"
      ],
      "use_when": "Modern web, multiple assets per page"
    },
    "HTTP/3": {
      "features": [
        "QUIC transport (UDP)",
        "0-RTT connection establishment",
        "Connection migration",
        "Built-in TLS 1.3",
        "Improved loss recovery"
      ],
      "improvements": [
        "~75% latency reduction on lossy networks",
        "No head-of-line blocking at transport",
        "Instant migration between networks"
      ],
      "use_when": "Mobile, high-latency networks"
    }
  }
}
```

### 1.3 Production Mindset
The web is a distributed, adversarial environment. APIs are long-lived contracts with operational, economic, and trust implications:

- **APIs are products with SLAs:** Every internal and external API has consumers who depend on its behavior. A breaking change without a deprecation period is a contract violation. Treat versioning, documentation, and backward compatibility as first-class engineering obligations.
- **Use HTTP semantics, not workarounds:** The protocol has well-defined methods, headers, and caching semantics. Re-inventing these as POST bodies or custom headers wastes the protocol's value and breaks standard tooling. Build with HTTP, not on top of it.
- **The network is hostile and unreliable:** Every external HTTP call must have a timeout, a retry policy with exponential backoff and jitter, and a circuit breaker. "It worked in staging" is not a resilience argument. Design for failure at the transport layer.
- **Rate limiting is not optional:** Any endpoint reachable from the internet without a rate limit is a denial-of-service vulnerability. Protect resources with per-user, per-IP, and per-endpoint limits. Return 429 with `Retry-After`.
- **Stateless servers are the only scalable servers:** Session state held in application memory breaks horizontal scaling and requires sticky session routing, which is a load-balancer anti-pattern. State belongs in the database or a distributed cache, never in local memory.
- **Idempotency is required for mutation endpoints:** In a distributed system, retries are not exceptional — they are expected. POST/PUT/DELETE operations must be idempotent or require an idempotency key. Non-idempotent mutations that can be retried will eventually be retried, with real consequences.
- **GraphQL vs REST is a capabilities match, not a style choice:** GraphQL provides value for highly relational data, flexible client queries, and mobile bandwidth constraints. It makes caching, rate limiting, and performance tracing significantly harder. REST remains the right default for simple CRUD and cacheable resources.
- **Error responses are part of the API contract:** A 500 is a bug, not an expected state. API errors must use consistent, machine-parseable structures (RFC 7807 or equivalent). Clients must be able to handle errors programmatically, not just display a generic message.

---

## 2. API Design Patterns

### 2.1 REST API Specification

```yaml
# OpenAPI 3.1 Specification for User Management API
openapi: 3.1.0
info:
  title: User Management API
  version: 1.0.0
  description: Complete user management API with CRUD operations
  contact:
    name: API Support
    email: api-support@example.com

servers:
  - url: https://api.example.com/v1
    description: Production
  - url: https://api-staging.example.com/v1
    description: Staging

paths:
  /users:
    get:
      operationId: listUsers
      summary: List all users
      description: Returns a paginated list of users
      security:
        - bearerAuth: []
      parameters:
        - name: page
          in: query
          schema:
            type: integer
            minimum: 1
            default: 1
        - name: limit
          in: query
          schema:
            type: integer
            minimum: 1
            maximum: 100
            default: 20
        - name: sort
          in: query
          schema:
            type: string
            enum: [created_at, email, name]
            default: created_at
        - name: order
          in: query
          schema:
            type: string
            enum: [asc, desc]
            default: desc
        - name: status
          in: query
          schema:
            type: string
            enum: [active, pending, suspended]
      responses:
        '200':
          description: Successful response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/UserListResponse'
        '401':
          $ref: '#/components/responses/Unauthorized'
        '403':
          $ref: '#/components/responses/Forbidden'
      tags:
        - Users

    post:
      operationId: createUser
      summary: Create a new user
      description: Creates a new user account
      security:
        - bearerAuth: []
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/CreateUserRequest'
          application/x-www-form-urlencoded:
            schema:
              $ref: '#/components/schemas/CreateUserRequest'
      responses:
        '201':
          description: User created successfully
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/UserResponse'
        '400':
          $ref: '#/components/responses/BadRequest'
        '409':
          $ref: '#/components/responses/Conflict'
      tags:
        - Users

  /users/{userId}:
    get:
      operationId: getUser
      summary: Get a user by ID
      security:
        - bearerAuth: []
      parameters:
        - name: userId
          in: path
          required: true
          schema:
            type: string
            format: uuid
      responses:
        '200':
          description: Successful response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/UserResponse'
        '404':
          $ref: '#/components/responses/NotFound'

    put:
      operationId: updateUser
      summary: Update a user
      security:
        - bearerAuth: []
      parameters:
        - name: userId
          in: path
          required: true
          schema:
            type: string
            format: uuid
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/UpdateUserRequest'
      responses:
        '200':
          description: User updated successfully
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/UserResponse'
        '404':
          $ref: '#/components/responses/NotFound'

    delete:
      operationId: deleteUser
      summary: Delete a user (soft delete)
      security:
        - bearerAuth: []
      parameters:
        - name: userId
          in: path
          required: true
          schema:
            type: string
            format: uuid
      responses:
        '204':
          description: User deleted successfully
        '404':
          $ref: '#/components/responses/NotFound'

components:
  securitySchemes:
    bearerAuth:
      type: http
      scheme: bearer
      bearerFormat: JWT
  
  schemas:
    User:
      type: object
      required:
        - id
        - email
        - created_at
      properties:
        id:
          type: string
          format: uuid
          example: 01ARZ3NDEKTSV4RRFFQ69G5FAV
        email:
          type: string
          format: email
          example: user@example.com
        display_name:
          type: string
          example: Jane Doe
        status:
          type: string
          enum: [active, pending, suspended]
        email_verified:
          type: boolean
        created_at:
          type: string
          format: date-time
        updated_at:
          type: string
          format: date-time
    
    UserListResponse:
      type: object
      properties:
        data:
          type: array
          items:
            $ref: '#/components/schemas/User'
        pagination:
          $ref: '#/components/schemas/Pagination'
    
    UserResponse:
      type: object
      properties:
        data:
          $ref: '#/components/schemas/User'
    
    CreateUserRequest:
      type: object
      required:
        - email
        - password
      properties:
        email:
          type: string
          format: email
        password:
          type: string
          format: password
          minLength: 12
        display_name:
          type: string
          minLength: 1
          maxLength: 100
    
    UpdateUserRequest:
      type: object
      properties:
        display_name:
          type: string
          minLength: 1
          maxLength: 100
        timezone:
          type: string
          example: America/Los_Angeles
    
    Pagination:
      type: object
      properties:
        page:
          type: integer
        limit:
          type: integer
        total:
          type: integer
        total_pages:
          type: integer
        next_cursor:
          type: string
          nullable: true
        has_more:
          type: boolean
  
  responses:
    BadRequest:
      description: Invalid request
      content:
        application/json:
          schema:
            $ref: '#/components/schemas/Error'
    
    Unauthorized:
      description: Authentication required
      content:
        application/json:
          schema:
            $ref: '#/components/schemas/Error'
    
    Forbidden:
      description: Insufficient permissions
      content:
        application/json:
          schema:
            $ref: '#/components/schemas/Error'
    
    NotFound:
      description: Resource not found
      content:
        application/json:
          schema:
            $ref: '#/components/schemas/Error'
    
    Conflict:
      description: Resource conflict
      content:
        application/json:
          schema:
            $ref: '#/components/schemas/Error'
  
  schemas:
    Error:
      type: object
      required:
        - error
      properties:
        error:
          type: object
          required:
            - code
            - message
          properties:
            code:
              type: string
              example: VALIDATION_ERROR
            message:
              type: string
              example: Email is required
            field:
              type: string
              example: email
            request_id:
              type: string
              example: req_01ARZ3NDEKTSV4RRFFQ69G5FA0
```

### 2.2 Status Code Usage

| Code | Meaning | When to Use |
|------|---------|-------------|
| 200 | OK | Successful GET, PUT, PATCH |
| 201 | Created | Successful POST creating resource |
| 202 | Accepted | Async operation accepted |
| 204 | No Content | Successful DELETE |
| 301 | Moved Permanently | URL redirect (use 308) |
| 302 | Found | Temporary redirect (use 307) |
| 304 | Not Modified | Cached response still valid |
| 400 | Bad Request | Invalid input, validation failed |
| 401 | Unauthorized | Missing/invalid authentication |
| 403 | Forbidden | Authenticated but not authorized |
| 404 | Not Found | Resource doesn't exist |
| 409 | Conflict | Duplicate, state conflict |
| 410 | Gone | Resource deleted permanently |
| 422 | Unprocessable | Semantic validation errors |
| 429 | Too Many Requests | Rate limit exceeded |
| 500 | Internal Error | Unexpected server error |
| 502 | Bad Gateway | Upstream error |
| 503 | Service Unavailable | Maintenance, overload |
| 504 | Gateway Timeout | Upstream timeout |

---

## 3. Authentication & Security

### 3.1 JWT Token Implementation

```rust
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,           // Subject (user ID)
    pub email: String,
    pub roles: Vec<String>,
    pub iat: i64,              // Issued at
    pub exp: i64,              // Expiration
    pub jti: String,           // JWT ID (for revocation)
    pub iss: String,           // Issuer
    pub aud: String,           // Audience
}

pub struct JWTManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    access_token_expiry: Duration,
    refresh_token_expiry: Duration,
    issuer: String,
    audience: String,
}

impl JWTManager {
    pub fn new(secret: &str) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
            access_token_expiry: Duration::minutes(15),
            refresh_token_expiry: Duration::days(7),
            issuer: "api.example.com".to_string(),
            audience: "api.example.com".to_string(),
        }
    }
    
    pub fn generate_access_token(&self, user_id: &str, email: &str, roles: &[String]) -> Result<String, JWTError> {
        let now = Utc::now();
        let jti = Uuid::new_v4().to_string();
        
        let claims = Claims {
            sub: user_id.to_string(),
            email: email.to_string(),
            roles: roles.to_vec(),
            iat: now.timestamp(),
            exp: (now + self.access_token_expiry).timestamp(),
            jti,
            iss: self.issuer.clone(),
            aud: self.audience.clone(),
        };
        
        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| JWTError::EncodingError(e.to_string()))
    }
    
    pub fn verify_token(&self, token: &str) -> Result<Claims, JWTError> {
        let validation = Validation::default()
            .set_issuer(&[&self.issuer])
            .set_audience(&[&self.audience]);
        
        decode::<Claims>(token, &self.decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|e| JWTError::DecodingError(e.to_string()))
    }
}
```

### 3.2 OAuth 2.0 PKCE Flow

```json
{
  "OAuth2PKCEFlow": {
    "steps": [
      {
        "step": 1,
        "name": "Authorization Request",
        "client_action": "Generate code_verifier (random 43-128 chars)",
        "request": {
          "response_type": "code",
          "client_id": "your-client-id",
          "redirect_uri": "https://your-app.com/callback",
          "scope": "openid profile email",
          "state": "random-state-value",
          "code_challenge": "BASE64URL(SHA256(code_verifier))",
          "code_challenge_method": "S256"
        }
      },
      {
        "step": 2,
        "name": "Authorization Response",
        "provider_action": "User authenticates, consents",
        "response": {
          "code": "auth-code-from-provider",
          "state": "random-state-value"
        }
      },
      {
        "step": 3,
        "name": "Token Exchange",
        "client_action": "Send code + code_verifier",
        "request": {
          "grant_type": "authorization_code",
          "code": "auth-code-from-provider",
          "redirect_uri": "https://your-app.com/callback",
          "client_id": "your-client-id",
          "code_verifier": "original-code-verifier"
        },
        "response": {
          "access_token": "ey...",
          "token_type": "Bearer",
          "expires_in": 3600,
          "refresh_token": "rt_...",
          "id_token": "ey..."
        }
      },
      {
        "step": 4,
        "name": "Token Validation",
        "client_action": "Validate id_token, extract claims"
      }
    ],
    "security_properties": [
      "code_verifier prevents authorization code interception",
      "SHA256 ensures verifier cannot be derived from challenge",
      "State parameter prevents CSRF attacks"
    ]
  }
}
```

---

## 4. Resilience Patterns

### 4.1 Circuit Breaker

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CircuitState {
    Closed,      // Normal operation
    Open,        // Failing, reject immediately
    HalfOpen,    // Testing if service recovered
}

pub struct CircuitBreaker {
    state: std::sync::atomic::AtomicPtr<CircuitStateInner>,
}

struct CircuitStateInner {
    state: CircuitState,
    failure_count: u64,
    last_failure_time: Option<Instant>,
    last_state_change: Instant,
}

impl CircuitBreaker {
    pub fn new(
        failure_threshold: u64,
        recovery_timeout: Duration,
        half_open_requests: u64,
    ) -> Self {
        let inner = Box::new(CircuitStateInner {
            state: CircuitState::Closed,
            failure_count: 0,
            last_failure_time: None,
            last_state_change: Instant::now(),
        });
        
        Self {
            state: std::sync::atomic::AtomicPtr::new(Box::into_raw(inner)),
        }
    }
    
    pub fn call<F, R>(&self, operation: F) -> Result<R, CircuitBreakerError>
    where
        F: FnOnce() -> Result<R, std::error::Error>,
    {
        let state = self.get_state();
        
        match state.state {
            CircuitState::Open => {
                if state.last_state_change.elapsed() < Duration::from_secs(30) {
                    return Err(CircuitBreakerError::CircuitOpen);
                }
                // Transition to half-open
                self.transition_to(CircuitState::HalfOpen);
            }
            CircuitState::HalfOpen => {
                // Allow limited requests through
            }
            CircuitState::Closed => {}
        }
        
        match operation() {
            Ok(result) => {
                self.on_success();
                Ok(result)
            }
            Err(e) => {
                self.on_failure();
                Err(CircuitBreakerError::OperationFailed(e))
            }
        }
    }
    
    fn on_success(&mut self) {
        // Reset failure count and close circuit
        let state = self.get_state_mut();
        state.failure_count = 0;
        if state.state != CircuitState::Closed {
            self.transition_to(CircuitState::Closed);
        }
    }
    
    fn on_failure(&mut self) {
        let state = self.get_state_mut();
        state.failure_count += 1;
        state.last_failure_time = Some(Instant::now());
        
        if state.failure_count >= 5 {
            self.transition_to(CircuitState::Open);
        }
    }
    
    fn transition_to(&self, new_state: CircuitState) {
        let state = self.get_state_mut();
        state.state = new_state;
        state.last_state_change = Instant::now();
    }
}
```

### 4.2 Retry with Backoff

```rust
use std::time::Duration;

pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            jitter: true,
        }
    }
}

pub async fn with_retry<F, Fut, T>(
    config: RetryConfig,
    mut operation: F,
) -> Result<T, RetryError>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, RetryError>>,
{
    let mut attempt = 0;
    let mut last_error = None;
    
    loop {
        attempt += 1;
        
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if is_retryable(&e) && attempt < config.max_attempts => {
                last_error = Some(e);
                let delay = calculate_delay(&config, attempt);
                tokio::time::sleep(delay).await;
            }
            Err(e) => return Err(e),
        }
    }
}

fn calculate_delay(config: &RetryConfig, attempt: u32) -> Duration {
    // Exponential backoff: base * 2^attempt
    let exp_delay = config.base_delay * 2u32.pow(attempt - 1);
    let capped = exp_delay.min(config.max_delay);
    
    if config.jitter {
        // Full jitter: 0 to capped
        let jitter = Duration::from_nanos(
            (rand::random::<u64>() % capped.as_nanos() as u64)
        );
        capped + jitter
    } else {
        capped
    }
}

fn is_retryable(error: &RetryError) -> bool {
    match error {
        RetryError::NetworkError => true,
        RetryError::Timeout => true,
        RetryError::ServiceUnavailable => true,
        RetryError::RateLimited => true,
        RetryError::Permanent => false,
    }
}
```

### 4.3 Timeout Configuration

```rust
pub struct TimeoutConfig {
    pub connection_timeout: Duration,
    pub request_timeout: Duration,
    pub read_timeout: Duration,
}

impl TimeoutConfig {
    pub fn http_defaults() -> Self {
        Self {
            connection_timeout: Duration::from_secs(5),
            request_timeout: Duration::from_secs(30),
            read_timeout: Duration::from_secs(60),
        }
    }
    
    pub fn database_defaults() -> Self {
        Self {
            connection_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(30),
            read_timeout: Duration::from_secs(60),
        }
    }
}

// Usage
pub async fn fetch_with_timeout(url: &str) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(30))
        .build()?;
    
    client.get(url).send().await?.text().await
}
```

---

## 5. Caching Headers

### 5.1 Cache-Control Directives

```
# Cache for 1 year (immutable assets)
Cache-Control: public, max-age=31536000, immutable

# Cache for 1 hour, stale-while-revalidate up to 1 day
Cache-Control: public, max-age=3600, stale-while-revalidate=86400

# Private (browser only, no CDN)
Cache-Control: private, max-age=3600

# No cache (always revalidate)
Cache-Control: no-cache, no-store, must-revalidate

# Proxy cache disabled
Cache-Control: private, no-cache

# ETag support
ETag: "33a64df551425fcc55e4d42a148795d9"

# Last modified
Last-Modified: Wed, 21 Oct 2015 07:28:00 GMT

# Conditional request
If-None-Match: "33a64df551425fcc55e4d42a148795d9"
If-Modified-Since: Wed, 21 Oct 2015 07:28:00 GMT
```

### 5.2 Response Cache Headers by Resource Type

```yaml
CacheHeaders:
  static_assets:
    description: Images, fonts, JS, CSS with content hashing
    headers:
      Cache-Control: public, max-age=31536000, immutable
      Vary: Accept-Encoding
  
  html_pages:
    description: Dynamic HTML
    headers:
      Cache-Control: no-cache, no-store, must-revalidate
      Pragma: no-cache
      Expires: 0
  
  api_responses:
    description: Dynamic API responses
    headers:
      Cache-Control: private, no-cache
      Vary: Authorization, Content-Type
  
  user_specific:
    description: Personalized content
    headers:
      Cache-Control: private, max-age=0, no-cache, no-store
      Vary: Authorization, Cookie
  
  cdn_cached:
    description: CDN-cached API responses
    headers:
      Cache-Control: public, max-age=60, stale-while-revalidate=300
      Vary: Accept-Encoding
```

---

## 6. Rate Limiting

### 6.1 Rate Limit Response Headers

```
# All responses should include these headers
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 999
X-RateLimit-Reset: 1715856000

# When rate limited (429 response)
Retry-After: 60
X-RateLimit-Retry-After-Seconds: 60

# Custom rate limit headers for different tiers
X-RateLimit-Limit-Read: 100
X-RateLimit-Limit-Write: 10
```

### 6.2 Rate Limit Implementation

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

pub struct RateLimiter {
    requests: Arc<RwLock<HashMap<String, RateLimitEntry>>>,
    config: RateLimitConfig,
}

pub struct RateLimitConfig {
    pub requests_per_window: u64,
    pub window_seconds: u64,
    pub burst_size: u64,
}

struct RateLimitEntry {
    tokens: u64,
    last_update: Instant,
}

impl RateLimiter {
    pub async fn check(&self, key: &str) -> Result<u64, RateLimitExceeded> {
        let mut entries = self.requests.write().await;
        let entry = entries.entry(key.to_string()).or_insert_with(|| {
            RateLimitEntry {
                tokens: self.config.requests_per_window,
                last_update: Instant::now(),
            }
        });
        
        // Replenish tokens
        let elapsed = entry.last_update.elapsed().as_secs();
        let replenished = elapsed * self.config.requests_per_window / self.config.window_seconds;
        entry.tokens = (entry.tokens + replenished).min(self.config.requests_per_window);
        entry.last_update = Instant::now();
        
        if entry.tokens > 0 {
            entry.tokens -= 1;
            Ok(entry.tokens)
        } else {
            let retry_after = self.config.window_seconds - elapsed;
            Err(RateLimitExceeded {
                limit: self.config.requests_per_window,
                remaining: 0,
                reset_at: entry.last_update + Duration::from_secs(self.config.window_seconds),
                retry_after: Duration::from_secs(retry_after.max(1)),
            })
        }
    }
}
```

---

## 7. Anti-Patterns

| Anti-Pattern | Specific Failure Mode | Prevention |
|--------------|----------------------|------------|
| **Session state in server memory** | Breaks horizontal scaling, sticky sessions needed | Use database or Redis for session |
| **Chatty APIs** | Many round trips, latency | Batch operations, composite APIs |
| **GET for mutations** | Caching issues, security | Use proper HTTP methods |
| **200 for errors** | Client can't detect failure | Use appropriate status codes |
| **No versioning** | Breaking changes affect clients | URL or header versioning |
| **Exposing internal IDs** | Enumeration attacks, info leak | Use opaque IDs or UUIDs |
| **No rate limiting** | DoS vulnerability | Implement rate limiting |
| **Synchronous chains** | Cascading latency | Async/parallel where possible |
| **No timeouts** | Resource exhaustion | Always set timeouts |
| **Verbose error messages** | Information disclosure | Generic messages in production |

---

## Links

### Core Router
- `core/DECAPOD.md` - **Router and navigation charter (START HERE)**
- `core/ENGINEERING_EXCELLENCE.md` - Engineering standards

### Architecture (This Section)
- `architecture/SECURITY.md` - Web security
- `architecture/CACHING.md` - HTTP caching
- `architecture/FRONTEND.md` - Frontend architecture
- `architecture/CLOUD.md` - Cloud deployment

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
- `methodology/API_DESIGN.md` - API design patterns