# API_DESIGN.md - API Design Standards

**Authority:** guidance (comprehensive API design with exact specifications, schemas, and patterns)
**Layer:** Architecture
**Binding:** No
**Scope:** REST, GraphQL, gRPC API design with exact specifications for pre-inference context

---

## 1. REST API Design

### 1.1 Resource Naming Conventions

```yaml
# Rules:
# - Use nouns, not verbs (GET /users not GET /getUsers)
# - Use plural for collections (/users not /user)
# - Use kebab-case for multi-word paths (/user-profiles not /userProfiles)
# - Nest resources for relationships (max 2 levels deep)
# - Use query parameters for filtering, sorting, pagination

# Good examples:
GET    /users                    # List users
GET    /users/{userId}           # Get single user
POST   /users                    # Create user
PUT    /users/{userId}           # Full update (replace)
PATCH  /users/{userId}           # Partial update
DELETE /users/{userId}           # Delete user
GET    /users/{userId}/orders    # User's orders (nested)
GET    /users/{userId}/orders/{orderId}  # Specific order

# Bad examples:
GET /getUser?id=123              # Verb in path
GET /user/123                    # Singular
POST /createUser                 # Verb in path
DELETE /user/123/orders/all      # 3 levels deep

# Query parameters:
GET /users?status=active&sort=created_at:desc&limit=20&offset=0
GET /orders?created_after=2024-01-01&total_gt=100
GET /products?category=electronics&in_stock=true
GET /users?search=john&fields=id,name,email
```

### 1.2 HTTP Methods

```yaml
GET    # Retrieve resource(s) - idempotent, no body
POST   # Create new resource - not idempotent
PUT    # Replace resource entirely - idempotent
PATCH  # Partial update - idempotent (with proper semantics)
DELETE # Remove resource - idempotent
HEAD   # Like GET but headers only
OPTIONS # CORS preflight, supported methods

# Safe methods: GET, HEAD, OPTIONS (don't modify server state)
# Idempotent methods: GET, PUT, DELETE, HEAD, OPTIONS
# (Idempotent = same request = same result, even if called multiple times)
```

### 1.3 Complete Request/Response Examples

#### Create Resource (POST)
```http
POST /v1/users HTTP/1.1
Host: api.example.com
Content-Type: application/json
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
Accept: application/json
X-Request-ID: f47ac10b-58cc-4372-a567-0e02b2c3d479
X-Correlation-ID: abc123

{
  "data": {
    "type": "users",
    "attributes": {
      "email": "john.doe@example.com",
      "name": "John Doe",
      "role": "engineer",
      "department": "engineering",
      "metadata": {
        "hire_date": "2024-01-15",
        "location": "New York"
      }
    },
    "relationships": {
      "manager": {
        "data": { "type": "users", "id": "usr_789xyz" }
      },
      "teams": {
        "data": [
          { "type": "teams", "id": "team_alpha" },
          { "type": "teams", "id": "team_beta" }
        ]
      }
    }
  }
}
```

```http
HTTP/1.1 201 Created
Content-Type: application/vnd.api+json
Location: /v1/users/usr_abc123
X-Request-ID: f47ac10b-58cc-4372-a567-0e02b2c3d479
ETag: "v1"
Cache-Control: no-cache

{
  "data": {
    "id": "usr_abc123",
    "type": "users",
    "links": {
      "self": "/v1/users/usr_abc123"
    },
    "attributes": {
      "email": "john.doe@example.com",
      "name": "John Doe",
      "role": "engineer",
      "department": "engineering",
      "created_at": "2024-01-15T10:30:00Z",
      "updated_at": "2024-01-15T10:30:00Z",
      "metadata": {
        "hire_date": "2024-01-15",
        "location": "New York"
      }
    },
    "relationships": {
      "manager": {
        "links": {
          "related": "/v1/users/usr_abc123/manager"
        },
        "data": { "type": "users", "id": "usr_789xyz" }
      },
      "teams": {
        "links": {
          "related": "/v1/users/usr_abc123/teams"
        },
        "data": [
          { "type": "teams", "id": "team_alpha" },
          { "type": "teams", "id": "team_beta" }
        ]
      }
    },
    "meta": {
      "created_by": "usr_system",
      "version": 1
    }
  },
  "included": [
    {
      "id": "usr_789xyz",
      "type": "users",
      "attributes": {
        "name": "Jane Manager"
      }
    },
    {
      "id": "team_alpha",
      "type": "teams",
      "attributes": {
        "name": "Platform Team"
      }
    },
    {
      "id": "team_beta",
      "type": "teams",
      "attributes": {
        "name": "Infrastructure Team"
      }
    }
  ]
}
```

#### Get Resource with Filtering (GET)
```http
GET /v1/users/usr_abc123?include=manager,teams&fields[users]=id,name,email,role HTTP/1.1
Host: api.example.com
Accept: application/json
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
```

```http
HTTP/1.1 200 OK
Content-Type: application/vnd.api+json
ETag: "v3"
Last-Modified: Mon, 15 Jan 2024 11:45:00 GMT
Cache-Control: private, max-age=300

{
  "data": {
    "id": "usr_abc123",
    "type": "users",
    "attributes": {
      "name": "John Doe",
      "email": "john.doe@example.com",
      "role": "engineer"
    },
    "relationships": {
      "manager": {
        "data": { "type": "users", "id": "usr_789xyz" }
      },
      "teams": {
        "data": [
          { "type": "teams", "id": "team_alpha" },
          { "type": "teams", "id": "team_beta" }
        ]
      }
    }
  },
  "included": [
    {
      "id": "usr_789xyz",
      "type": "users",
      "attributes": {
        "name": "Jane Manager",
        "email": "jane@example.com"
      }
    },
    {
      "id": "team_alpha",
      "type": "teams",
      "attributes": {
        "name": "Platform Team"
      }
    }
  ]
}
```

### 1.4 Pagination Specifications

#### Cursor-based Pagination (Preferred for large datasets)
```http
GET /v1/orders?page[limit]=25&page[cursor]=eyJpZCI6MTIzfQ== HTTP/1.1
```

```json
{
  "data": [...],
  "pagination": {
    "cursors": {
      "before": "eyJpZCI6MTAwfQ==",
      "after": "eyJpZCI6MTI1fQ=="
    },
    "has_more": true,
    "total": null
  },
  "links": {
    "first": "/v1/orders?page[limit]=25",
    "next": "/v1/orders?page[limit]=25&page[cursor]=eyJpZCI6MTI1fQ==",
    "prev": "/v1/orders?page[limit]=25&page[cursor]=eyJpZCI6MTAwfQ=="
  }
}
```

#### Offset-based Pagination (Simple use cases)
```http
GET /v1/users?page[limit]=20&page[offset]=0&page[number]=1 HTTP/1.1
```

```json
{
  "data": [...],
  "pagination": {
    "limit": 20,
    "offset": 0,
    "total": 1500,
    "current_page": 1,
    "total_pages": 75
  },
  "links": {
    "first": "/v1/users?page[limit]=20&page[offset]=0",
    "next": "/v1/users?page[limit]=20&page[offset]=20",
    "prev": null,
    "last": "/v1/users?page[limit]=20&page[offset]=1480"
  }
}
```

#### Keyset Pagination (For extreme performance)
```http
# Use compound sort keys for stable pagination
GET /v1/events?sort=created_at,id&after_id=evt_123&limit=50
# After getting results, use last item's sort keys for next page:
GET /v1/events?sort=created_at,id&after_created_at=2024-01-15T10:30:00Z&after_id=evt_456&limit=50
```

### 1.5 Response Envelope Patterns

```json
{
  "data": {...} | [...],  // Single resource or array
  "meta": {
    "request_id": "f47ac10b-58cc-4372-a567-0e02b2c3d479",
    "timestamp": "2024-01-15T10:30:00Z",
    "api_version": "v1",
    "pagination": {...} | null,
    "count": 150,
    "filters_applied": {
      "status": "active",
      "created_after": "2024-01-01"
    }
  },
  "error": null | {...},
  "included": [...],
  "links": {...}
}
```

### 1.6 Error Response Patterns

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Request validation failed",
    "details": [
      {
        "field": "email",
        "code": "INVALID_FORMAT",
        "message": "Email format is invalid",
        "value": "not-an-email"
      },
      {
        "field": "age",
        "code": "OUT_OF_RANGE",
        "message": "Age must be between 0 and 150",
        "value": -5
      }
    ],
    "source": {
      "pointer": "/data/attributes/email",
      "parameter": "email"
    },
    "documentation_url": "https://api.example.com/docs/errors/VALIDATION_ERROR",
    "trace_id": "abc123",
    "request_id": "f47ac10b-58cc-4372-a567-0e02b2c3d479"
  },
  "meta": {
    "timestamp": "2024-01-15T10:30:00Z"
  }
}
```

#### HTTP Status Codes
```yaml
# 2xx Success
200 OK                    # GET, PUT, PATCH succeeded
201 Created               # POST created new resource
202 Accepted             # Async operation queued
204 No Content           # DELETE succeeded, no body

# 4xx Client Errors
400 Bad Request           # Malformed request, invalid syntax
401 Unauthorized          # No/invalid authentication
403 Forbidden             # Authenticated but not authorized
404 Not Found             # Resource doesn't exist
405 Method Not Allowed    # HTTP method not supported
409 Conflict              # State conflict (duplicate, version mismatch)
410 Gone                   # Resource permanently deleted
422 Unprocessable Entity  # Validation failed (semantic errors)
429 Too Many Requests     # Rate limit exceeded

# 5xx Server Errors
500 Internal Server Error # Unexpected error
501 Not Implemented       # Feature not implemented
502 Bad Gateway            # Upstream/service failure
503 Service Unavailable    # Temporarily unavailable
504 Gateway Timeout        # Upstream timeout
```

---

## 2. GraphQL API Design

### 2.1 Schema Design

```graphql
# schema.graphql

scalar DateTime
scalar JSON
scalar UUID

enum UserRole {
  ADMIN
  ENGINEER
  MANAGER
  VIEWER
}

enum OrderStatus {
  PENDING
  PROCESSING
  SHIPPED
  DELIVERED
  CANCELLED
}

type User {
  id: ID!
  email: String!
  name: String!
  role: UserRole!
  
  # Relations
  manager: User
  directReports: [User!]!
  orders: OrderConnection!
  
  # Computed
  fullName: String!
  isActive: Boolean!
  
  # Timestamps
  createdAt: DateTime!
  updatedAt: DateTime!
  
  # Meta
  metadata: JSON
}

type Order {
  id: ID!
  status: OrderStatus!
  total: Decimal!
  currency: String!
  
  # Relations
  user: User!
  items: [OrderItem!]!
  
  # Timestamps
  createdAt: DateTime!
  updatedAt: DateTime!
}

type OrderItem {
  id: ID!
  quantity: Int!
  unitPrice: Decimal!
  totalPrice: Decimal!
  product: Product!
}

type Product {
  id: ID!
  name: String!
  description: String
  price: Decimal!
  inStock: Boolean!
  category: Category!
}

type Category {
  id: ID!
  name: String!
  slug: String!
  parent: Category
  children: [Category!]!
  products: ProductConnection!
}

# Pagination
type UserConnection {
  edges: [UserEdge!]!
  pageInfo: PageInfo!
  totalCount: Int!
}

type UserEdge {
  node: User!
  cursor: String!
}

type PageInfo {
  hasNextPage: Boolean!
  hasPreviousPage: Boolean!
  startCursor: String
  endCursor: String
}

# Input types
input CreateUserInput {
  email: String!
  name: String!
  role: UserRole = VIEWER
  metadata: JSON
}

input UpdateUserInput {
  email: String
  name: String
  role: UserRole
  metadata: JSON
}

input UserFilterInput {
  role: UserRole
  search: String
  createdAfter: DateTime
  createdBefore: DateTime
}

input OrderByInput {
  field: OrderSortField!
  direction: SortDirection = ASC
}

enum OrderSortField {
  CREATED_AT
  UPDATED_AT
  TOTAL
}

enum SortDirection {
  ASC
  DESC
}
```

### 2.2 Complete Query/Mutation Examples

```graphql
# Query with nested relations and pagination
query GetUserWithOrders($userId: ID!, $orderLimit: Int = 10) {
  user(id: $userId) {
    id
    email
    name
    role
    manager {
      id
      name
      email
    }
    orders(first: $orderLimit, after: null, sort: [{ field: CREATED_AT, direction: DESC }]) {
      edges {
        node {
          id
          status
          total
          currency
          createdAt
          items {
            id
            quantity
            product {
              id
              name
            }
          }
        }
        cursor
      }
      pageInfo {
        hasNextPage
        endCursor
      }
    }
  }
}

# Variables:
{
  "userId": "usr_abc123",
  "orderLimit": 20
}

# Mutation with input and error handling
mutation CreateOrder($input: CreateOrderInput!) {
  createOrder(input: $input) {
    order {
      id
      status
      total
      items {
        id
        quantity
        product {
          id
          name
        }
      }
    }
    user {
      id
      email
      loyaltyPoints
    }
    errors {
      field
      message
      code
    }
  }
}

# Input:
{
  "input": {
    "userId": "usr_abc123",
    "items": [
      { "productId": "prod_xyz", "quantity": 2 },
      { "productId": "prod_abc", "quantity": 1 }
    ],
    "shippingAddress": {
      "street": "123 Main St",
      "city": "New York",
      "state": "NY",
      "zip": "10001",
      "country": "US"
    }
  }
}

# Response:
{
  "data": {
    "createOrder": {
      "order": {
        "id": "ord_123",
        "status": "PENDING",
        "total": "149.99",
        "items": [
          {
            "id": "item_1",
            "quantity": 2,
            "product": { "id": "prod_xyz", "name": "Widget Pro" }
          }
        ]
      },
      "user": {
        "id": "usr_abc123",
        "email": "user@example.com",
        "loyaltyPoints": 150
      },
      "errors": null
    }
  }
}
```

### 2.3 DataLoader Pattern (N+1 Prevention)

```python
# DataLoader: Batch and cache database queries to prevent N+1
from dataloader import DataLoader
from functools import cached_property

class UserLoader(DataLoader):
    @cached_property
    def batch_load_fn(self):
        async def batch_load(ids):
            users = await User.query.where(User.id.in_(ids)).fetch_all()
            return [next((u for u in users if u.id == id), None) for id in ids]
        return batch_load

class OrderLoader(DataLoader):
    @cached_property
    def batch_load_fn(self):
        async def batch_load(user_ids):
            orders = await Order.query.where(Order.user_id.in_(user_ids)).fetch_all()
            # Group by user_id
            orders_by_user = {}
            for order in orders:
                if order.user_id not in orders_by_user:
                    orders_by_user[order.user_id] = []
                orders_by_user[order.user_id].append(order)
            return [orders_by_user.get(uid, []) for uid in user_ids]
        return batch_load

# Usage in resolver
class UserType:
    @staticmethod
    async def resolve_orders(user, info):
        loader = info.context.loaders.order_loader
        return await loader.load(user.id)
```

### 2.4 GraphQL Error Handling

```python
# Custom error types
class GraphQLError(Exception):
    def __init__(self, message, code, field=None, details=None):
        self.message = message
        self.code = code
        self.field = field
        self.details = details or {}

# Union type for errors
class Error:
    pass

class ValidationError(Error):
    field: str
    message: str

class NotFoundError(Error):
    message: str

class UnauthorizedError(Error):
    message: str

type CreateOrderResult {
    order: Order
    errors: [ValidationError!]
}

# Use in mutation
async def resolve_create_order(_, info, input):
    errors = []
    
    # Validate input
    if not input.get('userId'):
        errors.append({'field': 'userId', 'message': 'Required'})
    
    # Check product availability
    for item in input.get('items', []):
        product = await get_product(item.productId)
        if not product:
            errors.append({
                'field': f'items.{item.productId}',
                'message': 'Product not found'
            })
    
    if errors:
        return {'order': None, 'errors': errors}
    
    # Create order
    order = await order_service.create(input)
    return {'order': order, 'errors': None}
```

---

## 3. gRPC & Protocol Buffers

### 3.1 Proto Schema Design

```protobuf
// user_service.proto
syntax = "proto3";

package user.v1;

import "google/protobuf/timestamp.proto";
import "google/protobuf/field_mask.proto";
import "google/protobuf/empty.proto";
import "validate/validate.proto";

option go_package = "github.com/example/user/v1;userpb";

// Service definition
service UserService {
  // Unary RPC
  rpc GetUser(GetUserRequest) returns (User);
  
  // Server streaming
  rpc ListUsers(ListUsersRequest) returns (stream User);
  
  // Client streaming
  rpc CreateUsers(stream CreateUserRequest) returns (CreateUsersResponse);
  
  // Bidirectional streaming
  rpc StreamUserUpdates(StreamUserUpdatesRequest) returns (stream User);
  
  // Batch operations
  rpc BatchGetUsers(BatchGetUsersRequest) returns (BatchGetUsersResponse);
}

message User {
  string id = 1 [(validate.rules).string = {
    min_len: 3,
    max_len: 50,
    pattern: "^usr_[a-zA-Z0-9]+$"
  }];
  
  string email = 2 [
    (validate.rules).string.email = true,
    (validate.rules).string.ignore_empty = false
  ];
  
  string name = 3 [(validate.rules).string = {
    min_len: 1,
    max_len: 200
  }];
  
  UserRole role = 4 [(validate.rules).enum.defined_only = true];
  
  map<string, string> metadata = 5;
  
  google.protobuf.Timestamp created_at = 6;
  google.protobuf.Timestamp updated_at = 7;
}

enum UserRole {
  USER_ROLE_UNSPECIFIED = 0;
  USER_ROLE_VIEWER = 1;
  USER_ROLE_ENGINEER = 2;
  USER_ROLE_MANAGER = 3;
  USER_ROLE_ADMIN = 4;
}

// Request/Response messages
message GetUserRequest {
  string id = 1;
  oneof identifier {
    string user_id = 2;
    string email = 3;
  }
  // Field selection
  google.protobuf.FieldMask field_mask = 4;
}

message ListUsersRequest {
  int32 page_size = 1 [(validate.rules).int32 = {
    gte: 1,
    lte: 100
  }];
  string page_token = 2;
  string filter = 3 [(validate.rules).string.max_len = 500];
  bool include_deleted = 4;
  
  // Sorting
  message OrderBy {
    string field = 1;
    bool descending = 2;
  }
  repeated OrderBy order_by = 5;
}

message ListUsersResponse {
  repeated User users = 1;
  string next_page_token = 2;
  int32 total_size = 3;
}

message CreateUserRequest {
  string email = 1 [(validate.rules).string.email = true];
  string name = 2 [(validate.rules).string.min_len = 1];
  UserRole role = 3;
  map<string, string> metadata = 4;
}

message CreateUsersResponse {
  message CreateResult {
    User user = 1;
    string error = 2;
  }
  repeated CreateResult results = 1;
  int32 success_count = 2;
  int32 failure_count = 3;
}

message BatchGetUsersRequest {
  repeated string ids = 1 [(validate.rules).repeated.max_items = 100];
}

message BatchGetUsersResponse {
  map<string, User> users = 1;
  repeated string not_found = 2;
}

message StreamUserUpdatesRequest {
  repeated string user_ids = 1;
}
```

### 3.2 gRPC Streaming Patterns

```python
# Server streaming: GetUserOrders
async def stream_user_orders(request, context):
    """Stream orders for a user."""
    user_id = request.user_id
    
    async for order in order_service.stream_orders(user_id):
        yield order
        # Check for cancellation
        if context.cancelled():
            return

# Client streaming: CreateUsers
async def create_users(stub, user_requests):
    """Send multiple user creation requests."""
    async def request_generator():
        for user_data in user_requests:
            yield user_data
            # Simulate delay between requests
            await asyncio.sleep(0.1)
    
    response = await stub.CreateUsers(request_generator())
    return response

# Bidirectional streaming: StreamUserUpdates
async def stream_user_updates(stub, user_ids):
    """Real-time user update stream with subscription management."""
    async def request_generator():
        for user_id in user_ids:
            yield StreamUserUpdatesRequest(user_id=user_id)
            await asyncio.sleep(30)  # Heartbeat
    
    responses = stub.StreamUserUpdates(request_generator())
    
    async for response in responses:
        if response.HasField('update'):
            print(f"User update: {response.update}")
        elif response.HasField('delete'):
            print(f"User deleted: {response.delete}")
```

### 3.3 gRPC Error Handling

```python
from grpc import StatusCode
from grpc StatusError

class GrpcError(Exception):
    def __init__(self, code, message, details=None):
        self.code = code
        self.message = message
        self.details = details or {}

# Server-side error raising
async def get_user(request, context):
    user = await user_service.get_user(request.id)
    
    if not user:
        context.abort(
            StatusCode.NOT_FOUND,
            f"User {request.id} not found"
        )
    
    if not user.active:
        context.abort(
            StatusCode.FAILED_PRECONDITION,
            "User account is inactive",
            details=[{"type": "user_inactive", "user_id": request.id}]
        )
    
    return user

# Client-side error handling
try:
    response = await stub.GetUser(request)
except grpc.RpcError as e:
    if e.code() == StatusCode.NOT_FOUND:
        logger.warning(f"User not found: {e.details()}")
    elif e.code() == StatusCode.UNAUTHENTICATED:
        # Re-authenticate and retry
        await refresh_token()
        response = await stub.GetUser(request)
    elif e.code() == StatusCode.DEADLINE_EXCEEDED:
        logger.error(f"Request timed out: {e.details()}")
    else:
        raise
```

---

## 4. API Versioning

### 4.1 Versioning Strategies

```yaml
# Strategy 1: URL Path Versioning (Most common)
GET /v1/users
GET /v2/users

# Pros: Easy to route, visible in logs
# Cons: URL changes, more complex routing

# Strategy 2: Header Versioning
GET /users
Accept: application/vnd.example.v2+json
API-Version: 2024-01-01

# Pros: Clean URLs
# Cons: Hidden, harder to test

# Strategy 3: Query Parameter
GET /users?version=2

# Pros: Easy to add
# Cons: Clutters URLs, caching issues

# Recommended: URL Path + Header for deprecation
# URL for routing, Header for fine-grained control
```

### 4.2 Deprecation Policy

```yaml
# Minimum version support: 2 versions active
# Deprecation timeline:
# - Announce deprecation: 6 months before sunset
# - Maintain old version: Minimum 12 months
# - Sunset old version: After new version stable

# Deprecation headers:
Deprecation: true
Sunset: Sat, 31 Dec 2024 23:59:59 GMT
Link: <https://api.example.com/docs/v2>; rel="deprecation"; type="text/html"
X-API-Deprecated: true
X-API-Sunset-Date: 2024-12-31

# Error response for deprecated API:
{
  "error": {
    "code": "DEPRECATED_VERSION",
    "message": "API version v1 is deprecated",
    "details": {
      "sunset_date": "2024-12-31",
      "migration_guide": "https://api.example.com/docs/migration/v1-to-v2"
    }
  }
}
```

---

## 5. Authentication & Authorization Headers

### 5.1 Standard Auth Headers

```http
# Bearer Token (JWT, OAuth)
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...

# Basic Auth (rarely used for APIs)
Authorization: Basic dXNlcm5hbWU6cGFzc3dvcmQ=

# API Key
X-API-Key: sk_live_abc123def456
# OR
Authorization: ApiKey sk_live_abc123def456

# Mutual TLS (no header, uses client cert)
```

### 5.2 Custom Headers Convention

```http
# Request tracing
X-Request-ID: f47ac10b-58cc-4372-a567-0e02b2c3d479
X-Correlation-ID: abc123
X-Forwarded-For: 203.0.113.195, 70.41.3.18, 150.172.238.178
X-Real-IP: 203.0.113.195

# Feature flags / context
X-Tenant-ID: tenant_abc123
X-Feature-Dark-Mode: true
X-Preferred-Language: en-US

# Rate limiting (response)
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 999
X-RateLimit-Reset: 1706703600
Retry-After: 60

# Pagination
X-Total-Count: 1500
X-Page-Limit: 20
X-Page-Offset: 0
```

---

## 6. CORS Configuration

### 6.1 CORS Headers

```http
# Response headers for CORS
Access-Control-Allow-Origin: https://app.example.com
# OR for multiple origins (must validate in application):
Access-Control-Allow-Origin: https://app.example.com

Access-Control-Allow-Methods: GET, POST, PUT, PATCH, DELETE, OPTIONS
Access-Control-Allow-Headers: Content-Type, Authorization, X-Request-ID, X-Correlation-ID
Access-Control-Expose-Headers: X-Request-ID, X-RateLimit-*
Access-Control-Allow-Credentials: true
Access-Control-Max-Age: 86400  # 24 hours, cache preflight

# Preflight request (OPTIONS)
OPTIONS /v1/users HTTP/1.1
Origin: https://app.example.com
Access-Control-Request-Method: POST
Access-Control-Request-Headers: Content-Type, Authorization
```

---

## 7. API Security Checklist

```yaml
# Authentication
- [ ] Require authentication for all non-public endpoints
- [ ] Validate tokens on every request
- [ ] Use short-lived access tokens (15-60 min)
- [ ] Implement refresh token rotation
- [ ] Support API key rotation

# Authorization
- [ ] Check permissions on every request
- [ ] Use least-privilege scopes
- [ ] Implement resource-level access control
- [ ] Log all authorization failures

# Input Validation
- [ ] Validate request body against schema
- [ ] Sanitize all string inputs
- [ ] Limit request body size
- [ ] Validate content-type header
- [ ] Check for SQL injection in query params

# Rate Limiting
- [ ] Implement per-user rate limits
- [ ] Implement per-IP rate limits for unauthenticated
- [ ] Return 429 with Retry-After header
- [ ] Consider burst limits

# Security Headers
- [ ] Content-Security-Policy (if serving HTML)
- [ ] X-Content-Type-Options: nosniff
- [ ] X-Frame-Options: DENY
- [ ] Strict-Transport-Security (HSTS)
- [ ] X-XSS-Protection (legacy browsers)

# Logging & Monitoring
- [ ] Log all authentication failures
- [ ] Log all authorization failures
- [ ] Log suspicious activity (unusual patterns)
- [ ] Alert on rate limit hits
- [ ] Alert on error rate spikes
```

---

## 8. API Design Anti-Patterns

```yaml
# ❌ Chasing the own tail (circular dependency)
# API calls itself through an alias
# User A -> /users -> /users
GET /users
Response: { "aliases": ["/users"] }

# ❌ Random batching
# Batch endpoint that does unrelated operations
POST /api/batch
Body: { "operations": [
    { "op": "get_user", "id": "123" },
    { "op": "delete_order", "id": "456" }
  ]}
# Should be separate calls or use GraphQL

# ❌ Version in body
POST /api/users
Body: { "version": "2.0", "data": {...} }

# ❌ Wrong HTTP status codes
# 200 for errors
# 500 for validation errors
# 404 for authorization (should be 403)

# ❌ Nested resources too deep
# Bad: /orgs/{org}/teams/{team}/members/{member}/roles/{role}
# Better: /members/{member}?include=roles

# ❌ Inconsistent naming
# /getUser, /list_users, /fetchUserOrders, /userList
# Should all use same convention: GET /users, GET /users/{id}, GET /users/{id}/orders

# ❌ Sensitive data in URLs or logs
# GET /users/123?token=xyz
# Authorization header is better (not logged by default)

# ❌ No pagination on large collections
# Returning 100,000 users in one response
# Must implement pagination
```

---

## Links

### Core Router
- `core/DECAPOD.md` - **Router and navigation charter (START HERE)**
- `core/ENGINEERING_EXCELLENCE.md` - Engineering standards

### Architecture (This Section)
- `architecture/WEB.md` - Web API patterns
- `architecture/AUTH.md` - Authentication patterns
- `architecture/MESSAGING.md` - Async API patterns
- `architecture/KUBERNETES.md` - API gateway in K8s

### Authority (Constitution Layer)
- `specs/INTENT.md` - **Methodology contract (READ FIRST)**
- `specs/SYSTEM.md` - System definition and authority doctrine
- `specs/SECURITY.md` - Security doctrine

### Interface Contracts
- `interfaces/CLAIMS.md` - Promises ledger
- `interfaces/CONTROL_PLANE.md` - Agent sequencing patterns
- `interfaces/DOC_RULES.md` - Doc compilation rules

### Methodology
- `methodology/ARCHITECTURE.md` - Architecture decision methodology
- `methodology/TESTING.md` - API testing methodology

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2024-01-15 | Expanded comprehensive API design reference |