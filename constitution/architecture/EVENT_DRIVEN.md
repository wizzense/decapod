# EVENT_DRIVEN.md - Event-Driven Architecture Patterns and Implementations

**Authority:** guidance (comprehensive topic with exact specifications)
**Layer:** Architecture
**Binding:** No
**Scope:** Comprehensive topic coverage for pre-inference context

## Table of Contents

1. [CQRS Patterns](#1-cqrs-patterns)
2. [Event Sourcing](#2-event-sourcing)
3. [Event Schema Design](#3-event-schema-design)
4. [Eventual Consistency](#4-eventual-consistency)
5. [Choreography vs Orchestration](#5-choreography-vs-orchestration)
6. [Kafka/Kinesis Event Schemas](#6-kafkakinesis-event-schemas)
7. [Event Processing Patterns](#7-event-processing-patterns)
8. [Decision Matrices](#8-decision-matrices)
9. [Anti-Patterns and Failure Modes](#9-anti-patterns-and-failure-modes)
10. [Production Implementation Guide](#10-production-implementation-guide)
11. [References](#11-references)

---

## 1. CQRS Patterns

### 1.1 CQRS Fundamentals

CQRS (Command Query Responsibility Segregation) separates read and write operations into distinct models. This allows independent optimization of each side.

**Command Side**
- Handles create, update, delete operations
- Returns void or single aggregate ID
- Can include complex business logic
- Validates business rules

**Query Side**
- Returns DTOs optimized for specific views
- Can use read-optimized storage
- Supports multiple representations of the same data
- Can include joins and aggregations

### 1.2 CQRS Implementation Patterns

```yaml
# CQRS basic architecture configuration
cqrs:
  # Command side configuration
  command:
    endpoint: /api/v1/commands
    model: aggregate_root
    validation:
      strict_mode: true
      validate_before_execution: true
      allowed_exceptions_are_serialized: false
    
    aggregate:
      persistence:
        type: event_store  # Options: event_store, document_db, relational
        event_store:
          provider: postgresql  # or: mongodb, eventstore
          connection_string: ${COMMAND_DB_URL}
          batch_size: 100
          bulk_insert: true
        snapshots:
          enabled: true
          frequency: every_10_events
          provider: postgresql
    
    handlers:
      concurrency:
        optimistic: true  # Optimistic concurrency with version field
        max_retry: 3
        retry_delay: 100ms
      timeout:
        command_timeout: 30s
        aggregate_lock_timeout: 5s
    
  # Query side configuration
  query:
    endpoints:
      - name: get-order
        path: /api/v1/queries/orders/{id}
        cache:
          enabled: true
          ttl: 30s
          invalidation: event_based  # Options: event_based, time_based, manual
      - name: list-orders
        path: /api/v1/queries/orders
        pagination:
          type: cursor  # Options: offset, cursor, keyset
          default_page_size: 20
          max_page_size: 100
    
    read_model:
      database:
        type: postgresql  # Options: postgresql, mongodb, elasticsearch, redis
        connection_string: ${QUERY_DB_URL}
        pool:
          min_size: 5
          max_size: 50
          idle_timeout: 30s
          max_lifetime: 1h
        
      projection:
        sync_mode: event_bus  # Options: event_bus, change_data_capture, polling
        batch_size: 100
        batch_timeout: 1s
        parallel_projections: true
        
    cache:
      redis:
        enabled: true
        connection_string: ${REDIS_URL}
        default_ttl: 300s
        cache_key_prefix: "query:"
        serialization: json
        
  # Event bus between command and query
  event_bus:
    type: kafka  # Options: kafka, rabbitmq, redis_streams, azure_event_hubs
    topic: cqrs.events
    consumer_group: cqrs-query-side
    serialization: avro
```

### 1.3 CQRS Command Model Implementation

```python
# Command model with aggregate root
from dataclasses import dataclass, field
from typing import List, Optional
from datetime import datetime
import uuid

@dataclass
class OrderLineItem:
    product_id: str
    quantity: int
    unit_price: float
    line_total: float = field(init=False)
    
    def __post_init__(self):
        self.line_total = self.quantity * self.unit_price

@dataclass
class ShippingAddress:
    street: str
    city: str
    state: str
    postal_code: str
    country: str

@dataclass
class OrderCreated:
    event_id: str = field(default_factory=lambda: str(uuid.uuid4()))
    occurred_at: datetime = field(default_factory=datetime.utcnow)
    order_id: str
    customer_id: str
    items: List[OrderLineItem]
    shipping_address: ShippingAddress
    total_amount: float

@dataclass 
class OrderConfirmed:
    event_id: str = field(default_factory=lambda: str(uuid.uuid4()))
    occurred_at: datetime = field(default_factory=datetime.utcnow)
    order_id: str
    confirmed_at: datetime

class OrderAggregate:
    """
    Aggregate root for order management.
    Manages state transitions and emits events.
    """
    
    def __init__(self, order_id: Optional[str] = None):
        self.order_id = order_id or str(uuid.uuid4())
        self.version = 0
        self.uncommitted_events: List = []
        
        # Internal state
        self._customer_id: Optional[str] = None
        self._items: List[OrderLineItem] = []
        self._shipping_address: Optional[ShippingAddress] = None
        self._status: str = "draft"
        self._total_amount: float = 0.0
        
    # State from events
    def apply_order_created(self, event: OrderCreated):
        self.order_id = event.order_id
        self._customer_id = event.customer_id
        self._items = event.items
        self._shipping_address = event.shipping_address
        self._status = "created"
        self._recalculate_total()
        
    def apply_order_confirmed(self, event: OrderConfirmed):
        self._status = "confirmed"
        
    def _recalculate_total(self):
        self._total_amount = sum(item.line_total for item in self._items)
        
    # Command handlers
    def create_order(
        self,
        customer_id: str,
        items: List[OrderLineItem],
        shipping_address: ShippingAddress
    ) -> OrderCreated:
        """Create a new order - returns event"""
        if self._status != "draft":
            raise InvalidOperationError(f"Cannot create order in status {self._status}")
        if not items:
            raise ValidationError("Order must have at least one item")
            
        event = OrderCreated(
            order_id=self.order_id,
            customer_id=customer_id,
            items=items,
            shipping_address=shipping_address
        )
        self.apply_order_created(event)
        self.uncommitted_events.append(event)
        self.version += 1
        return event
        
    def confirm(self) -> OrderConfirmed:
        """Confirm the order"""
        if self._status != "created":
            raise InvalidOperationError(f"Cannot confirm order in status {self._status}")
            
        event = OrderConfirmed(
            order_id=self.order_id,
            confirmed_at=datetime.utcnow()
        )
        self.apply_order_confirmed(event)
        self.uncommitted_events.append(event)
        self.version += 1
        return event
        
    def get_uncommitted_events(self) -> List:
        events = self.uncommitted_events
        self.uncommitted_events = []
        return events
    
    def rehydrate_from_events(self, events: List):
        """Reconstruct aggregate from event history"""
        for event in events:
            if isinstance(event, OrderCreated):
                self.apply_order_created(event)
            elif isinstance(event, OrderConfirmed):
                self.apply_order_confirmed(event)
                
@dataclass
class CommandResult:
    success: bool
    aggregate_id: str
    events: List
    version: int
    error: Optional[str] = None
    metadata: dict = field(default_factory=dict)
    
class CommandHandler:
    """Executes commands on aggregates and persists events"""
    
    def __init__(self, event_store):
        self.event_store = event_store
        
    async def handle_create_order(
        self,
        customer_id: str,
        items: List[OrderLineItem],
        shipping_address: ShippingAddress
    ) -> CommandResult:
        aggregate = OrderAggregate()
        
        try:
            events = [aggregate.create_order(customer_id, items, shipping_address)]
            
            # Persist events to event store
            await self.event_store.append_events(
                aggregate.order_id,
                aggregate.get_uncommitted_events(),
                expected_version=aggregate.version - len(events)
            )
            
            return CommandResult(
                success=True,
                aggregate_id=aggregate.order_id,
                events=events,
                version=aggregate.version
            )
            
        except ConcurrencyException as e:
            return CommandResult(
                success=False,
                aggregate_id=aggregate.order_id,
                events=[],
                version=0,
                error=f"Concurrency conflict: {e}"
            )
```

### 1.4 CQRS Query Model (Read Model)

```python
# Read model projections
from dataclasses import dataclass
from typing import List, Optional
from datetime import datetime

@dataclass
class OrderReadModel:
    """Read model for order queries"""
    order_id: str
    customer_id: str
    customer_name: str
    status: str
    items_count: int
    total_amount: float
    currency: str
    shipping_address: dict
    created_at: datetime
    updated_at: datetime
    confirmed_at: Optional[datetime]
    
@dataclass  
class OrderListItem:
    """Simplified order for list views"""
    order_id: str
    customer_name: str
    status: str
    total_amount: float
    created_at: datetime

class OrderReadModelRepository:
    """Repository for querying order read models"""
    
    def __init__(self, db_pool):
        self.db_pool = db_pool
        
    async def get_by_id(self, order_id: str) -> Optional[OrderReadModel]:
        """Get single order with full details"""
        async with self.db_pool.acquire() as conn:
            row = await conn.fetchrow("""
                SELECT 
                    o.id as order_id,
                    o.customer_id,
                    c.name as customer_name,
                    o.status,
                    o.total_items as items_count,
                    o.total_amount,
                    o.currency,
                    o.shipping_address,
                    o.created_at,
                    o.updated_at,
                    o.confirmed_at
                FROM orders o
                JOIN customers c ON o.customer_id = c.id
                WHERE o.id = $1
            """, order_id)
            
            if not row:
                return None
                
            return OrderReadModel(
                order_id=row['order_id'],
                customer_id=row['customer_id'],
                customer_name=row['customer_name'],
                status=row['status'],
                items_count=row['items_count'],
                total_amount=row['total_amount'],
                currency=row['currency'],
                shipping_address=row['shipping_address'],
                created_at=row['created_at'],
                updated_at=row['updated_at'],
                confirmed_at=row['confirmed_at']
            )
    
    async def list_orders(
        self,
        customer_id: Optional[str] = None,
        status: Optional[str] = None,
        limit: int = 20,
        cursor: Optional[str] = None
    ) -> List[OrderListItem]:
        """List orders with cursor-based pagination"""
        async with self.db_pool.acquire() as conn:
            query = """
                SELECT 
                    o.id as order_id,
                    c.name as customer_name,
                    o.status,
                    o.total_amount,
                    o.created_at
                FROM orders o
                JOIN customers c ON o.customer_id = c.id
                WHERE 1=1
            """
            params = []
            param_idx = 1
            
            if customer_id:
                query += f" AND o.customer_id = ${param_idx}"
                params.append(customer_id)
                param_idx += 1
                
            if status:
                query += f" AND o.status = ${param_idx}"
                params.append(status)
                param_idx += 1
                
            if cursor:
                query += f" AND o.created_at < ${param_idx}"
                params.append(datetime.fromisoformat(cursor))
                param_idx += 1
                
            query += """
                ORDER BY o.created_at DESC
                LIMIT $""" + str(param_idx)
            params.append(limit + 1)  # Fetch one extra to detect has_more
            
            rows = await conn.fetch(query, *params)
            
            has_more = len(rows) > limit
            if has_more:
                rows = rows[:limit]
                
            return [
                OrderListItem(
                    order_id=row['order_id'],
                    customer_name=row['customer_name'],
                    status=row['status'],
                    total_amount=row['total_amount'],
                    created_at=row['created_at']
                )
                for row in rows
            ], has_more
```

---

## 2. Event Sourcing

### 2.1 Event Sourcing Fundamentals

Event sourcing stores state as a sequence of events rather than current state. Every state change is captured as an immutable event record.

**Benefits:**
- Complete audit trail
- Temporal queries (state at any point in time)
- Event replay for debugging
- Multiple projections from same events
- Easy integration with event-driven architectures

**Trade-offs:**
- Event schema evolution complexity
- Projections for read models
- eventual consistency in queries
- Larger storage footprint (vs. point-in-time snapshots)

### 2.2 Event Store Implementation

```yaml
# Event Store PostgreSQL schema and configuration
event_store:
  # PostgreSQL schema for event storage
  schema:
    events_table: events
    snapshots_table: snapshots
    streams_table: streams
    
  # Stream configuration
  streams:
    order_stream:
      id: orders
      aggregate_type: order
      settings:
        max_age: 10y  # Keep events for 10 years
        max_count: 1000000
        cache_size: 10000
        
    inventory_stream:
      id: inventory
      aggregate_type: inventory_item
      settings:
        max_age: 3y
        cache_size: 5000
        
  # Snapshot configuration
  snapshots:
    enabled: true
    frequency: every_10_events
    strategy: when_useful  # Options: always, when_useful, never
    retention: 30_days
    
  # PostgreSQL connection
  postgres:
    host: ${EVENT_STORE_HOST}
    port: 5432
    database: event_store
    username: ${EVENT_STORE_USER}
    password: ${EVENT_STORE_PASSWORD}
    pool:
      min_connections: 10
      max_connections: 100
      connection_timeout: 30s
      idle_timeout: 5m
      max_lifetime: 1h
    options:
      sslmode: require
      application_name: event_store
      
  # Performance settings
  performance:
    batch_size: 500
    bulk_insert_threshold: 100
    parallel_projections: 4
    commit_interval: 100ms
    
  # Backup settings
  backup:
    enabled: true
    schedule: "0 2 * * *"  # Daily at 2 AM
    retention: 30_days
    destination: s3://event-store-backups/
    compression: lz4
```

```sql
-- Event Store PostgreSQL Schema
CREATE TABLE events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stream_name VARCHAR(255) NOT NULL,
    stream_version INTEGER NOT NULL,
    event_type VARCHAR(255) NOT NULL,
    event_data JSONB NOT NULL,
    metadata JSONB DEFAULT '{}',
    causation_id UUID,
    correlation_id UUID,
    user_id VARCHAR(255),
    trace_id VARCHAR(255),
    span_id VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT events_stream_version_unique UNIQUE (stream_name, stream_version),
    
    -- Indexes
    CONSTRAINT events_stream_name_check CHECK (char_length(stream_name) > 0),
    CONSTRAINT events_event_type_check CHECK (char_length(event_type) > 0)
);

-- Indexes for common query patterns
CREATE INDEX idx_events_stream_name ON events(stream_name);
CREATE INDEX idx_events_stream_version ON events(stream_name, stream_version);
CREATE INDEX idx_events_event_type ON events(event_type);
CREATE INDEX idx_events_correlation_id ON events(correlation_id) WHERE correlation_id IS NOT NULL;
CREATE INDEX idx_events_causation_id ON events(causation_id) WHERE causation_id IS NOT NULL;
CREATE INDEX idx_events_created_at ON events(created_at DESC);
CREATE INDEX idx_events_metadata_gin ON events USING GIN(metadata);

-- Snapshots table for fast aggregate reconstruction
CREATE TABLE snapshots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    stream_name VARCHAR(255) NOT NULL,
    aggregate_id VARCHAR(255) NOT NULL,
    aggregate_version INTEGER NOT NULL,
    snapshot_type VARCHAR(255) NOT NULL,
    snapshot_data JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT snapshots_stream_aggregate_unique UNIQUE (stream_name, aggregate_id),
    CONSTRAINT snapshots_version_check CHECK (aggregate_version >= 0)
);

CREATE INDEX idx_snapshots_stream_aggregate ON snapshots(stream_name, aggregate_id DESC);
CREATE INDEX idx_snapshots_aggregate_version ON snapshots(aggregate_id, aggregate_version DESC);

-- Streams metadata table
CREATE TABLE streams (
    stream_name VARCHAR(255) PRIMARY KEY,
    aggregate_type VARCHAR(255),
    stream_version INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'
);

-- Function to append events atomically
CREATE OR REPLACE FUNCTION append_events(
    p_stream_name VARCHAR,
    p_expected_version INTEGER,
    p_events JSONB,
    p_metadata JSONB DEFAULT '{}',
    p_corr_id UUID DEFAULT NULL,
    p_caus_id UUID DEFAULT NULL
) RETURNS TABLE (
    id UUID,
    stream_version INTEGER,
    event_type VARCHAR,
    created_at TIMESTAMPTZ
) AS $$
DECLARE
    v_next_version INTEGER;
    v_event JSONB;
    v_result RECORD;
BEGIN
    -- Calculate next version
    SELECT COALESCE(MAX(stream_version), -1) + 1
    INTO v_next_version
    FROM events
    WHERE stream_name = p_stream_name;
    
    -- Check for version conflict
    IF p_expected_version != v_next_version AND p_expected_version != -1 THEN
        RAISE EXCEPTION 'Optimistic concurrency violation: expected version % but stream is at version %',
            p_expected_version, v_next_version
            USING ERRCODE = '23505';  -- unique_violation
    END IF;
    
    -- Process each event
    FOR v_event IN SELECT * FROM jsonb_array_elements(p_events)
    LOOP
        INSERT INTO events (
            stream_name,
            stream_version,
            event_type,
            event_data,
            metadata,
            correlation_id,
            causation_id,
            created_at
        ) VALUES (
            p_stream_name,
            v_next_version,
            v_event->>'event_type',
            v_event->'event_data',
            p_metadata,
            p_corr_id,
            p_caus_id,
            NOW()
        )
        RETURNING id, stream_version, event_type, created_at
        INTO v_result;
        
        RETURN QUERY SELECT v_result.id, v_result.stream_version, v_result.event_type, v_result.created_at;
        
        v_next_version := v_next_version + 1;
    END LOOP;
    
    -- Update stream metadata
    UPDATE streams
    SET 
        stream_version = v_next_version - 1,
        updated_at = NOW()
    WHERE stream_name = p_stream_name;
    
    -- Insert stream if not exists
    INSERT INTO streams (stream_name, aggregate_type, stream_version)
    VALUES (p_stream_name, p_stream_name, v_next_version - 1)
    ON CONFLICT (stream_name) DO NOTHING;
END;
$$ LANGUAGE plpgsql;

-- Function to get aggregate events
CREATE OR REPLACE FUNCTION get_aggregate_events(
    p_stream_name VARCHAR,
    p_aggregate_id VARCHAR,
    p_from_version INTEGER DEFAULT 0
) RETURNS TABLE (
    id UUID,
    stream_version INTEGER,
    event_type VARCHAR,
    event_data JSONB,
    metadata JSONB,
    created_at TIMESTAMPTZ
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        e.id,
        e.stream_version,
        e.event_type,
        e.event_data,
        e.metadata,
        e.created_at
    FROM events e
    WHERE e.stream_name = p_stream_name
      AND e.stream_version > p_from_version
    ORDER BY e.stream_version ASC;
END;
$$ LANGUAGE plpgsql;

-- Function to get latest snapshot
CREATE OR REPLACE FUNCTION get_latest_snapshot(
    p_stream_name VARCHAR,
    p_aggregate_id VARCHAR
) RETURNS TABLE (
    id UUID,
    aggregate_version INTEGER,
    snapshot_data JSONB,
    created_at TIMESTAMPTZ
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        s.id,
        s.aggregate_version,
        s.snapshot_data,
        s.created_at
    FROM snapshots s
    WHERE s.stream_name = p_stream_name
      AND s.aggregate_id = p_aggregate_id
    ORDER BY s.aggregate_version DESC
    LIMIT 1;
END;
$$ LANGUAGE plpgsql;
```

### 2.3 Event Schema Evolution

```python
# Event versioning and upcasting
from abc import ABC, abstractmethod
from typing import Dict, Callable, Any
from dataclasses import dataclass

@dataclass
class EventEnvelope:
    """Wrapper for events with metadata"""
    event_id: str
    event_type: str
    event_version: int
    occurred_at: str
    stream_name: str
    stream_version: int
    event_data: Dict
    metadata: Dict = None

class EventUpcaster(ABC):
    """Base class for event upcasters"""
    
    @property
    @abstractmethod
    def event_type(self) -> str:
        pass
    
    @property
    @abstractmethod
    def from_version(self) -> int:
        pass
    
    @property
    @abstractmethod
    def to_version(self) -> int:
        pass
    
    @abstractmethod
    def upgrade(self, event_data: Dict) -> Dict:
        pass

class OrderCreatedUpcasterV1toV2(EventUpcaster):
    """Upcast OrderCreated from v1 to v2"""
    
    @property
    def event_type(self) -> str:
        return "OrderCreated"
    
    @property
    def from_version(self) -> int:
        return 1
    
    @property
    def to_version(self) -> int:
        return 2
    
    def upgrade(self, event_data: Dict) -> Dict:
        """
        V1 -> V2: Added 'priority' field
        V1: { customer_id, items, shipping_address }
        V2: { customer_id, items, shipping_address, priority }
        """
        upgraded = event_data.copy()
        if 'priority' not in upgraded:
            upgraded['priority'] = 'normal'
        return upgraded

class OrderCreatedUpcasterV2toV3(EventUpcaster):
    """Upcast OrderCreated from v2 to v3"""
    
    @property
    def event_type(self) -> str:
        return "OrderCreated"
    
    @property
    def from_version(self) -> int:
        return 2
    
    @property
    def to_version(self) -> int:
        return 3
    
    def upgrade(self, event_data: Dict) -> Dict:
        """
        V2 -> V3: Split shipping_address into separate fields
        V2: { ..., shipping_address: { street, city, state, postal_code, country } }
        V3: { ..., shipping_street, shipping_city, shipping_state, shipping_postal_code, shipping_country }
        """
        upgraded = event_data.copy()
        
        if 'shipping_address' in event_data:
            addr = event_data['shipping_address']
            upgraded['shipping_street'] = addr.get('street', '')
            upgraded['shipping_city'] = addr.get('city', '')
            upgraded['shipping_state'] = addr.get('state', '')
            upgraded['shipping_postal_code'] = addr.get('postal_code', '')
            upgraded['shipping_country'] = addr.get('country', '')
            del upgraded['shipping_address']
            
        return upgraded

class EventUpcasterChain:
    """Manages upcaster chain for event upgrades"""
    
    def __init__(self):
        self._upcasters: Dict[str, list] = {}
        
    def register(self, upcaster: EventUpcaster):
        key = f"{upcaster.event_type}_v{upcaster.from_version}"
        if key not in self._upcasters:
            self._upcasters[key] = []
        self._upcasters[key].append(upcaster)
        
    def upcast(self, event_type: str, event_version: int, event_data: Dict) -> Dict:
        """Upgrade event to latest version"""
        current_data = event_data
        current_version = event_version
        
        while True:
            key = f"{event_type}_v{current_version}"
            if key not in self._upcasters:
                break
                
            # Get all upcasters for this version transition
            applicable = [
                u for u in self._upcasters[key]
                if u.from_version == current_version
            ]
            
            if not applicable:
                break
                
            # Apply the upcaster
            upcaster = applicable[0]
            current_data = upcaster.upgrade(current_data)
            current_version = upcaster.to_version
            
        return current_data

# Usage
upcaster_chain = EventUpcasterChain()
upcaster_chain.register(OrderCreatedUpcasterV1toV2())
upcaster_chain.register(OrderCreatedUpcasterV2toV3())

# To upgrade an event
current_data = upcaster_chain.upcast("OrderCreated", 1, old_v1_event_data)
```

---

## 3. Event Schema Design

### 3.1 Event Schema Best Practices

**Naming Conventions:**
- Event types: Past tense, verb, noun (e.g., `OrderCreated`, `PaymentProcessed`)
- Namespaces: Dot-separated (e.g., `com.example.orders.OrderCreated`)
- Field names: snake_case for JSON, camelCase for protobuf

**Required Fields:**
- `event_id`: Globally unique identifier (UUID)
- `event_type`: Name of the event
- `event_version`: Schema version
- `occurred_at`: When event occurred
- `correlation_id`: For tracing related events
- `causation_id`: ID of the command that caused this event

### 3.2 Event Schema Examples

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "OrderCreatedEvent",
  "description": "Event emitted when a new order is successfully created",
  "type": "object",
  "x-struct": true,
  "x-events": {
    "currentVersion": 3,
    "migrationPath": ["OrderCreatedEventV1", "OrderCreatedEventV2"],
    "deprecatedVersions": [1, 2],
    "sunsetDate": "2027-01-01"
  },
  "required": [
    "eventId",
    "eventType",
    "eventVersion",
    "occurredAt",
    "correlationId",
    "payload"
  ],
  "properties": {
    "eventId": {
      "type": "string",
      "format": "uuid",
      "description": "Unique identifier for this event instance",
      "examples": ["550e8400-e29b-41d4-a716-446655440000"]
    },
    "eventType": {
      "type": "string",
      "const": "OrderCreated",
      "description": "The type of event"
    },
    "eventVersion": {
      "type": "integer",
      "minimum": 1,
      "maximum": 3,
      "description": "Schema version of this event"
    },
    "occurredAt": {
      "type": "string",
      "format": "date-time",
      "description": "ISO 8601 timestamp when event occurred",
      "examples": ["2026-01-15T10:30:00.000Z"]
    },
    "correlationId": {
      "type": "string",
      "format": "uuid",
      "description": "Groups related events together",
      "examples": ["660e8400-e29b-41d4-a716-446655440001"]
    },
    "causationId": {
      "type": "string",
      "format": "uuid",
      "description": "ID of the command that caused this event"
    },
    "payload": {
      "type": "object",
      "required": ["orderId", "customerId", "items", "shippingAddress", "totalAmount"],
      "properties": {
        "orderId": {
          "type": "string",
          "format": "uuid",
          "description": "Unique order identifier"
        },
        "orderNumber": {
          "type": "string",
          "pattern": "^ORD-[0-9]{10}$",
          "description": "Human-readable order number"
        },
        "customerId": {
          "type": "string",
          "format": "uuid"
        },
        "items": {
          "type": "array",
          "minItems": 1,
          "maxItems": 100,
          "items": {
            "$ref": "#/definitions/OrderLineItem"
          }
        },
        "shippingAddress": {
          "$ref": "#/definitions/ShippingAddress"
        },
        "totalAmount": {
          "$ref": "#/definitions/Money"
        },
        "priority": {
          "type": "string",
          "enum": ["low", "normal", "high", "urgent"],
          "default": "normal"
        },
        "notes": {
          "type": "string",
          "maxLength": 1000
        },
        "metadata": {
          "type": "object",
          "additionalProperties": true
        }
      }
    }
  },
  "definitions": {
    "OrderLineItem": {
      "type": "object",
      "required": ["lineItemId", "productId", "productName", "quantity", "unitPrice", "lineTotal"],
      "properties": {
        "lineItemId": {
          "type": "string",
          "format": "uuid"
        },
        "productId": {
          "type": "string"
        },
        "productName": {
          "type": "string",
          "maxLength": 200
        },
        "quantity": {
          "type": "integer",
          "minimum": 1,
          "maximum": 999
        },
        "unitPrice": {
          "$ref": "#/definitions/Money"
        },
        "lineTotal": {
          "$ref": "#/definitions/Money"
        },
        "discount": {
          "$ref": "#/definitions/Money"
        },
        "metadata": {
          "type": "object"
        }
      }
    },
    "ShippingAddress": {
      "type": "object",
      "required": ["street", "city", "state", "postalCode", "country"],
      "properties": {
        "street": {
          "type": "string",
          "maxLength": 200
        },
        "addressLine2": {
          "type": "string",
          "maxLength": 200
        },
        "city": {
          "type": "string",
          "maxLength": 100
        },
        "state": {
          "type": "string",
          "maxLength": 100
        },
        "postalCode": {
          "type": "string",
          "maxLength": 20
        },
        "country": {
          "type": "string",
          "minLength": 2,
          "maxLength": 2,
          "pattern": "^[A-Z]{2}$"
        },
        "phone": {
          "type": "string",
          "maxLength": 20
        },
        "instructions": {
          "type": "string",
          "maxLength": 500
        }
      }
    },
    "Money": {
      "type": "object",
      "required": ["amount", "currency"],
      "properties": {
        "amount": {
          "type": "string",
          "pattern": "^-?\\d+\\.\\d{2}$",
          "description": "Decimal string for precise arithmetic"
        },
        "currency": {
          "type": "string",
          "minLength": 3,
          "maxLength": 3,
          "pattern": "^[A-Z]{3}$",
          "examples": ["USD", "EUR", "GBP"]
        }
      }
    }
  }
}
```

### 3.3 Avro Schema for Kafka

```json
{
  "type": "record",
  "name": "OrderCreatedEvent",
  "namespace": "com.example.events.orders",
  "doc": "Event emitted when a new order is created",
  "aliases": ["OrderCreatedEvent", "com.example.orders.OrderCreated"],
  "version": "3",
  "fields": [
    {
      "name": "eventId",
      "type": {
        "type": "string",
        "logicalType": "uuid"
      },
      "doc": "Unique event identifier"
    },
    {
      "name": "eventType",
      "type": "string",
      "default": "OrderCreated"
    },
    {
      "name": "eventVersion",
      "type": "int",
      "default": 3
    },
    {
      "name": "occurredAt",
      "type": {
        "type": "long",
        "logicalType": "timestamp-millis"
      },
      "doc": "Event occurrence timestamp in milliseconds since epoch"
    },
    {
      "name": "correlationId",
      "type": {
        "type": "string",
        "logicalType": "uuid"
      }
    },
    {
      "name": "causationId",
      "type": ["null", {
        "type": "string",
        "logicalType": "uuid"
      }],
      "default": null
    },
    {
      "name": "payload",
      "type": {
        "type": "record",
        "name": "OrderCreatedPayload",
        "fields": [
          {
            "name": "orderId",
            "type": {
              "type": "string",
              "logicalType": "uuid"
            }
          },
          {
            "name": "orderNumber",
            "type": "string"
          },
          {
            "name": "customerId",
            "type": {
              "type": "string",
              "logicalType": "uuid"
            }
          },
          {
            "name": "items",
            "type": {
              "type": "array",
              "items": {
                "type": "record",
                "name": "OrderLineItem",
                "fields": [
                  {"name": "lineItemId", "type": "string"},
                  {"name": "productId", "type": "string"},
                  {"name": "productName", "type": "string"},
                  {"name": "quantity", "type": "int"},
                  {"name": "unitPrice", "type": "OrderMoney"},
                  {"name": "lineTotal", "type": "OrderMoney"}
                ]
              }
            }
          },
          {
            "name": "shippingAddress",
            "type": {
              "type": "record",
              "name": "ShippingAddress",
              "fields": [
                {"name": "street", "type": "string"},
                {"name": "city", "type": "string"},
                {"name": "state", "type": "string"},
                {"name": "postalCode", "type": "string"},
                {"name": "country", "type": "string"}
              ]
            }
          },
          {
            "name": "totalAmount",
            "type": "OrderMoney"
          },
          {
            "name": "priority",
            "type": {
              "type": "enum",
              "name": "OrderPriority",
              "symbols": ["LOW", "NORMAL", "HIGH", "URGENT"]
            },
            "default": "NORMAL"
          }
        ]
      }
    }
  ],
  "logicalTypes": {
    "OrderMoney": {
      "type": "record",
      "name": "OrderMoney",
      "fields": [
        {"name": "amount", "type": "string"},
        {"name": "currency", "type": "string"}
      ]
    }
  }
}
```

---

## 4. Eventual Consistency

### 4.1 Eventual Consistency Patterns

```yaml
# Eventual consistency configuration
eventual_consistency:
  # Read-your-writes consistency
  read_your_writes:
    enabled: true
    strategy: session_based  # Options: session_based, version_based, blocking
    session_timeout: 30m
    max_pending_reads: 100
    
  # Monotonic reads
  monotonic_reads:
    enabled: true
    strategy: version_tracking  # Options: version_tracking, sticky_server
    
  # Causal consistency
  causal_consistency:
    enabled: true
    vector_clock_based: true
    tracking_overhead_threshold: 1000  # Max tracked dependencies
    
  # Consistency guarantees by operation type
  operation_guarantees:
    strongly_consistent:
      - inventory_updates
      - payment_transactions
      - security_operations
      
    causal_consistent:
      - order_fulfillment
      - inventory_reservations
      - customer_profile_changes
      
    eventual_consistent:
      - search_indexes
      - analytics_views
      - notification_preferences
      - recommendation_models
```

### 4.2 Read-Your-Writes Implementation

```python
from typing import Optional
from dataclasses import dataclass
import time

@dataclass
class SessionConsistencyContext:
    """Context for read-your-writes consistency"""
    session_id: str
    user_id: str
    last_write_timestamp: float
    last_write_stream: Optional[str]
    last_write_version: Optional[int]

class ReadYourWritesConsistency:
    """Implements read-your-writes consistency"""
    
    def __init__(self, query_handler, event_store):
        self.query_handler = query_handler
        self.event_store = event_store
        self.sessions: dict = {}
        
    def read(
        self,
        stream_name: str,
        query_params: dict,
        session_context: SessionConsistencyContext
    ) -> Any:
        """
        Read with read-your-writes consistency.
        If we recently wrote to this stream, wait for event to propagate.
        """
        
        # Check if we need to wait
        if self._needs_wait(session_context, stream_name):
            # Wait for event propagation (async, with timeout)
            self._wait_for_propagation(session_context, stream_name)
            
        return self.query_handler.execute(stream_name, query_params)
    
    def _needs_wait(
        self,
        session: SessionConsistencyContext,
        stream_name: str
    ) -> bool:
        """Determine if we need to wait for propagation"""
        if session.last_write_stream != stream_name:
            return False
            
        if time.time() - session.last_write_timestamp > 30:
            # Allow eventual consistency after 30 seconds
            return False
            
        return True
    
    def _wait_for_propagation(
        self,
        session: SessionConsistencyContext,
        stream_name: str,
        timeout: float = 5.0
    ):
        """Wait for write to propagate to read replicas"""
        deadline = time.time() + timeout
        
        while time.time() < deadline:
            # Check if read replica is up to date
            current_version = self.event_store.get_stream_version(stream_name)
            
            if session.last_write_version is None:
                break
                
            if current_version >= session.last_write_version:
                return True
                
            time.sleep(0.1)  # Poll every 100ms
            
        return False  # Timed out, proceed anyway (eventual consistency)
```

---

## 5. Choreography vs Orchestration

### 5.1 Choreography Pattern

In choreography, services communicate by emitting and listening to events without a central coordinator.

```yaml
# Choreography configuration
choreography:
  # Event bus configuration
  event_bus:
    type: kafka
    topics:
      - orders.events
      - inventory.events
      - payments.events
      - notifications.events
      
    consumer_groups:
      order_service: orders.events
      inventory_service: orders.events, inventory.events
      payment_service: orders.events, payments.events
      notification_service: orders.events, payments.events, notifications.events
      
  # Event subscriptions
  subscriptions:
    order_service:
      topics:
        orders.events:
          filters:
            - eventType: OrderCreated
            - eventType: OrderCancelled
          concurrency: 10
          error_handling:
            strategy: retry_with_backoff
            max_retries: 3
            backoff: exponential
            
    inventory_service:
      topics:
        orders.events:
          filters:
            eventType: OrderCreated
          actions:
            - reserve_inventory
        inventory.events:
          filters:
            eventType: InventoryReserved
            correlationId: current_order_id
            
  # Dead letter queue
  dead_letter:
    enabled: true
    topic: choreography.dlq
    max_retries: 5
    retry_topic: choreography.retry
    retry_delays: [1s, 5s, 30s, 2m, 10m]
```

### 5.2 Orchestration Pattern

In orchestration, a central coordinator directs the flow of operations.

```yaml
# Orchestration configuration
orchestration:
  # Saga orchestrator
  saga_orchestrator:
    name: order-fulfillment-orchestrator
    persistence:
      enabled: true
      storage: postgresql
      connection_string: ${ORCHESTRATOR_DB_URL}
      table_name: saga_instances
      instance_ttl: 604800  # 7 days
      
    # Step definitions
    steps:
      - name: create_order
        command: CreateOrderCommand
        compensation: CancelOrderCommand
        timeout: 30s
        
      - name: reserve_inventory
        command: ReserveInventoryCommand
        compensation: ReleaseInventoryCommand
        timeout: 15s
        
      - name: process_payment
        command: ChargePaymentCommand
        compensation: RefundPaymentCommand
        timeout: 60s
        
      - name: confirm_order
        command: ConfirmOrderCommand
        compensation: null  # No compensation needed
        timeout: 10s
        
    # Recovery settings
    recovery:
      enabled: true
      interval: 60s  # Check for stuck sagas every minute
      resolution:
        in_progress_timeout: 30m  # Mark as failed if running longer
        compensate_on_recovery: true
        max_auto_compensation_attempts: 3
        
    # Observability
    observability:
      emit_state_changes: true
      emit_compensation_events: true
      trace_correlation: true
```

### 5.3 Comparison and Selection

| Criteria | Choreography | Orchestration |
|----------|-------------|----------------|
| Complexity | Low per service | High per orchestrator |
| Visibility | Low (scattered logic) | High (centralized state) |
| Coupling | Low | Higher (services know orchestrator) |
| Transaction scope | Limited | Full saga support |
| Debugging | Harder | Easier |
| Failure handling | Manual per service | Built-in compensation |
| Scalability | High | Medium |
| Best for | Simple, independent reactions | Complex multi-step workflows |

---

## 6. Kafka/Kinesis Event Schemas

### 6.1 Kafka Topic Configuration

```yaml
# Kafka cluster configuration
kafka:
  # Broker configuration
  brokers:
    - host: kafka-0.platform.svc.cluster.local
      port: 9092
      rack: us-east-1a
    - host: kafka-1.platform.svc.cluster.local
      port: 9092
      rack: us-east-1b
    - host: kafka-2.platform.svc.cluster.local
      port: 9092
      rack: us-east-1c
      
  # Security
  security:
    protocol: SASL_SSL
    sasl_mechanism: SCRAM-SHA-512
    tls:
      enabled: true
      cert_path: /etc/kafka/secrets/client.crt
      key_path: /etc/kafka/secrets/client.key
      ca_path: /etc/kafka/secrets/ca.crt
      
  # Producer configuration
  producer:
    acks: all  # Wait for all in-sync replicas
    retries: 3
    max_in_flight_requests_per_connection: 5
    enable_idempotence: true
    max_request_size: 1048576  # 1MB
    linger_ms: 5  # Batch for 5ms before sending
    batch_size: 16384  # 16KB batch size
    compression: lz4
    buffer_memory: 33554432  # 32MB buffer
    request_timeout_ms: 30000
    delivery_timeout_ms: 120000
    
  # Consumer configuration
  consumer:
    group_id: order-service-consumer
    auto_offset_reset: earliest
    enable_auto_commit: false
    auto_commit_interval_ms: 5000
    max_poll_records: 500
    max_poll_interval_ms: 300000
    session_timeout_ms: 30000
    heartbeat_interval_ms: 10000
    isolation_level: read_committed  # Only read committed transactions
    fetch_min_bytes: 1
    fetch_max_wait_ms: 500

# Kafka topics
kafka_topics:
  orders:
    name: orders.events
    partitions: 64
    replication_factor: 3
    configs:
      retention.ms: 604800000  # 7 days
      retention.bytes: -1  # Unlimited
      cleanup.policy: delete
      min.insync.replicas: "2"
      segment.bytes: 1073741824  # 1GB segments
      segment.ms: 3600000  # Roll every hour
      max.message.bytes: "1048576"  # 1MB
      
  inventory:
    name: inventory.events
    partitions: 48
    replication_factor: 3
    configs:
      retention.ms: 2592000000  # 30 days
      retention.bytes: -1
      cleanup.policy: delete
      
  payments:
    name: payments.events
    partitions: 32
    replication_factor: 3
    configs:
      retention.ms: 2592000000  # 30 days (financial data)
      retention.bytes: -1
      min.insync.replicas: "2"
      
  notifications:
    name: notifications.events
    partitions: 16
    replication_factor: 3
    configs:
      retention.ms: 86400000  # 1 day
      cleanup.policy: delete
      
  dead_letter:
    name: dead-letter
    partitions: 8
    replication_factor: 3
    configs:
      retention.ms: 604800000  # 7 days
```

### 6.2 Kafka Connect Configuration

```yaml
# Kafka Connect for CDC (Change Data Capture)
kafka_connect:
  # PostgreSQL source connector
  postgresql_source:
    name: postgresql-orders-source
    config:
      connector.class: io.confluent.connect.jdbc.JdbcSourceConnector
      tasks.max: 4
      
      # Database connection
      connection.url: jdbc:postgresql://postgres.platform.svc.cluster.local:5432/orders
      connection.user: ${POSTGRES_USER}
      connection.password: ${POSTGRES_PASSWORD}
      
      # Query configuration
      query: SELECT * FROM orders WHERE updated_at > ? ORDER BY updated_at ASC
      query.timeout.ms: 300000
      poll.interval.ms: 1000
      
      # Mode configuration
      mode: timestamp+incrementing
      incrementing.column.name: id
      timestamp.column.name: updated_at
      validate.non.null: false
      
      # Output configuration
      topic.prefix: cdc.
      batch.max.rows: 1000
      
      # Error handling
      errors.tolerance: all
      errors.log.enable: true
      errors.log.include.messages: true
      
  # Elasticsearch sink connector
  elasticsearch_sink:
    name: elasticsearch-orders-sink
    config:
      connector.class: io.confluent.connect.elasticsearch.ElasticsearchSinkConnector
      tasks.max: 4
      
      # Connection
      connection.url: https://elasticsearch.platform.svc.cluster.local:9200
      connection.username: ${ES_USER}
      connection.password: ${ES_PASSWORD}
      tls.enabled: true
      tls.truststore.path: /etc/connect/secrets/truststore.jks
      tls.truststore.password: ${TRUSTSTORE_PASSWORD}
      
      # Input
      topics: orders.events
      key.converter: org.apache.kafka.connect.storage.StringConverter
      value.converter: org.apache.kafka.connect.json.JsonConverter
      value.converter.schemas.enable: false
      
      # Index management
      index.name.mode: custom
      index.name.pattern: orders-${topic}
      type.name: _doc
      
      # Write behavior
      flush.timeout.ms: 10000
      max.retries: 10
      retry.backoff.ms: 1000
      
      # Data transformation
      transforms: insertKey
      transforms.insertKey.type: org.apache.kafka.connect.transforms.ValueToKey
      transforms.insertKey.fields: order_id
```

---

## 7. Event Processing Patterns

### 7.1 Event Processing Topologies

```yaml
# Stream processing configuration
stream_processing:
  # Flink job configuration
  flink:
    cluster:
      name: flink-cluster
      namespace: platform
      parallelism: 4
      restart_strategy: exponential
      min_pause_between_restarts: 10s
      max_restarts: 10
      delay: 30s
      
    jobs:
      order_analytics:
        jar: /opt/flink/jars/order-analytics.jar
        entry_class: com.example.OrderAnalyticsJob
        parallelism: 4
        checkpointing:
          enabled: true
          interval: 60s
          mode: EXACTLY_ONCE
          storage: filesystem
          checkpoint_dir: s3://flink-checkpoints/
          min_pause_between_checkpoints: 30s
          max_concurrent_checkpoints: 1
        state_backend:
          type: rocksdb
          rocksdb:
            memory: 2GB
            state_backend_dir: s3://flink-state/
        resources:
          memory: 4GB
          task_slots: 8
          
      inventory_replenishment:
        jar: /opt/flink/jars/inventory-replenishment.jar
        parallelism: 2
        window:
          type: tumbling
          size: 5m
        late_data:
          handling: allowed_lateness
          lateness: 1m
          side_output_late_events: true
```

### 7.2 Windowing Operations

```yaml
# Windowing configuration for stream processing
windowing:
  # Time windows
  time_windows:
    tumbling_5m:
      type: tumbling
      size: 5m
      watermark:
        delay: 30s
        alignment:
          enabled: true
          max_out_of_orderness: 10s
          
    sliding_1h_5m:
      type: sliding
      size: 1h
      slide: 5m
      watermark:
        delay: 30s
        
    session_10m:
      type: session
      gap: 10m
      timeout: 30s
      max_consecutive_gaps: 5
      
  # Count windows
  count_windows:
    count_1000:
      type: counting
      size: 1000
      greedy: true
      
  # Aggregation configuration
  aggregations:
    order_revenue:
      window: tumbling_5m
      metrics:
        total_revenue:
          type: sum
          field: total_amount
        order_count:
          type: count
        avg_order_value:
          type: avg
          field: total_amount
        max_order_value:
          type: max
          field: total_amount
        unique_customers:
          type: distinct_count
          field: customer_id
```

---

## 8. Decision Matrices

### 8.1 Event-Driven Pattern Selection

| Requirement | CQRS | Event Sourcing | Both | Neither |
|-------------|------|----------------|------|---------|
| Complex domain logic | ❌ | ❌ | ✅ | ❌ |
| Audit trail requirement | ❌ | ✅ | ✅ | ❌ |
| Multiple read models | ✅ | ❌ | ✅ | ❌ |
| Temporal queries | ❌ | ✅ | ✅ | ❌ |
| High write throughput | ❌ | ❌ | ❌ | ✅ |
| Simple CRUD with caching | ✅ | ❌ | ❌ | ❌ |
| Complex reporting | ✅ | ❌ | ✅ | ❌ |
| Point-in-time snapshots | ❌ | ✅ | ✅ | ❌ |

### 8.2 Event Storage Selection

| Factor | PostgreSQL (JSONB) | EventStoreDB | Kafka (with ksqlDB) | MongoDB |
|--------|-------------------|--------------|---------------------|---------|
| Schema evolution | Medium | Excellent | Medium | Medium |
| Query capability | Good | Good | Excellent | Good |
| Scalability | Medium | Medium | Excellent | High |
| Transaction support | Excellent | Good | Limited | Limited |
| Event replay | Good | Excellent | Excellent | Good |
| Operational complexity | Low | Medium | High | Low |
| Cost | Low | Medium | High | Low |

### 8.3 Messaging System Selection

| Requirement | Kafka | RabbitMQ | Redis Streams | Kinesis |
|-------------|-------|----------|--------------|---------|
| Exactly-once delivery | ✅ | ❌ | ❌ | ❌ |
| High throughput (1M+/s) | ✅ | ❌ | ❌ | ✅ |
| Message ordering | Partition key | Queue | Per stream | Shard key |
| Complex routing | ❌ | ✅ | ❌ | ❌ |
| Transaction support | ✅ | Basic | Limited | Limited |
| Latency | Low | Very Low | Very Low | Medium |
| Replay capability | ✅ | ❌ | ✅ | ❌ |
| Operational complexity | High | Medium | Low | Medium |

---

## 9. Anti-Patterns and Failure Modes

### 9.1 Anti-Patterns

**Chatty Event Chains**
```yaml
# PROBLEM: Too many small events creating tight coupling
chatty_pattern:
  events:
    - OrderCreated
    - OrderCreatedInventoryChecked
    - OrderCreatedInventoryReserved
    - OrderCreatedInventoryConfirmed
    - OrderCreatedPaymentInitiated
    - OrderCreatedPaymentConfirmed
    - OrderCreatedNotificationsQueued
    - OrderCreatedFulfillmentInitiated
    
# SOLUTION: Combine related events into meaningful aggregates
efficient_pattern:
  events:
    - OrderCreated  # Contains inventory and payment info
    - OrderConfirmed  # Indicates all checks passed
    - OrderFulfilled  # Indicates completion
```

**Eventual Consistency Without Bounds**
```yaml
# PROBLEM: No defined consistency windows
risky_pattern:
  reads: eventual_consistent
  write_wait: none
  consequence: "Users may see stale data indefinitely"

# SOLUTION: Define consistency bounds
safe_pattern:
  reads: read_your_writes  # Within session
  cross_session_consistency_window: 5s
  stale_threshold_alerts: true
  max_observed_staleness_metric: consistency_staleness_seconds
```

### 9.2 Common Failure Modes

**Event Loss**
```
Error: "Event not found in downstream projection"
Cause: Consumer offset not committed before crash
Solution: Ensure enable.auto.commit=false with manual commit after processing

Prevention:
- Use transactional outbox pattern
- Implement exactly-once semantics via idempotency
- Set appropriate replication factor (3+)
```

**Event Replay Storm**
```
Error: "Consumer lag suddenly zero, massive replay"
Cause: New consumer group starting from beginning
Solution: Set appropriate offset reset policy

Prevention:
- Use offset retention policies
- Implement consumer group monitoring
- Set up alerts for consumer lag
```

**Schema Version Conflicts**
```
Error: "Can't deserialize event - unknown field"
Cause: Consumers on old version processing new schema events
Solution: Implement backward-compatible schema evolution

Prevention:
- Always add optional fields (with defaults)
- Never rename fields (add alias)
- Version upcasters for all major changes
```

---

## 10. Production Implementation Guide

### 10.1 Event Processing Checklist

```yaml
production_checklist:
  event_schema:
    - [ ] All events have unique event_id
    - [ ] All events have occurred_at timestamp
    - [ ] All events have event_version for schema evolution
    - [ ] All events include correlation_id for tracing
    - [ ] Schema registry is configured
    - [ ] Backward compatibility is tested
    
  event_processing:
    - [ ] Consumers handle poison pills gracefully
    - [ ] Dead letter queue is configured
    - [ ] Consumer lag is monitored
    - [ ] Idempotency is implemented in handlers
    - [ ] Exactly-once semantics verified
    
  consistency:
    - [ ] Read-your-writes is implemented for user-facing operations
    - [ ] Consistency windows are defined and monitored
    - [ ] Stale reads are detected and alerted
    
  disaster_recovery:
    - [ ] Event store is backed up
    - [ ] Recovery procedures are documented
    - [ ] RTO and RPO are defined
    - [ ] Chaos testing includes event processing
```

### 10.2 Monitoring Configuration

```yaml
# Event processing observability
observability:
  # Lag monitoring
  consumer_lag:
    alert_threshold: 10000
    critical_threshold: 100000
    
  # Processing time
  processing_latency:
    p50_target: < 100ms
    p99_target: < 500ms
    p999_target: < 1s
    
  # Error rates
  error_rates:
    dlq_enqueue_rate:
      warning: 0.01  # 1%
      critical: 0.05  # 5%
      
  # Throughput
  throughput:
    events_per_second:
      warning: < 1000
      target: > 10000
```

---

## 11. References

### CQRS and Event Sourcing

- [CQRS](https://docs.microsoft.com/en-us/azure/architecture/patterns/cqrs) - Microsoft patterns & practices
- [Event Sourcing](https://docs.microsoft.com/en-us/azure/architecture/patterns/event-sourcing) - Microsoft patterns & practices
- [Event Sourcing pattern - Martin Fowler](https://martinfowler.com/eaaDev/EventSourcing.html)
- [CQRS - Martin Fowler](https://martinfowler.com/articles/cqrs.html)

### Event Schema

- [Confluent Schema Registry](https://docs.confluent.io/platform/current/schema-registry/index.html)
- [Avro Schema Resolution](https://avro.apache.org/docs/current/spec.html#Schema+Resolution)
- [JSON Schema](https://json-schema.org/specification.html)

### Streaming Platforms

- [Apache Kafka Documentation](https://kafka.apache.org/documentation/)
- [Confluent Kafka Documentation](https://docs.confluent.io/platform/current/)
- [Apache Flink Documentation](https://flink.apache.org/docs/)
- [Amazon Kinesis Data Streams](https://docs.aws.amazon.com/streams/latest/dev/)

### Event Processing Patterns

- [Streaming Systems](https://www.oreilly.com/library/view/streaming-systems/9781491983874/) - Tyler Akidau et al.
- [Apache Beam Documentation](https://beam.apache.org/documentation/)
- [Kafka Streams in Action](https://www.manning.com/books/kafka-streams-in-action)

### Production Considerations

- [Lessons from Building Event-Driven Systems](https://www.confluent.io/blog/lessons-learned-from-building-event-driven-systems/)
- [Event-Driven Microservices Anti-Patterns](https://solace.com/blog/event-driven-microservices-anti-patterns)