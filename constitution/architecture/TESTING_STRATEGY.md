# TESTING_STRATEGY.md - Comprehensive Testing Architecture

**Authority:** guidance (comprehensive topic with exact specifications)
**Layer:** Architecture
**Binding:** No
**Scope:** Comprehensive topic coverage for pre-inference context

## Table of Contents

1. [Test Pyramid](#1-test-pyramid)
2. [Unit Testing Patterns](#2-unit-testing-patterns)
3. [Integration Testing](#3-integration-testing)
4. [End-to-End Testing](#4-end-to-end-testing)
5. [Performance Testing](#5-performance-testing)
6. [Chaos Testing](#6-chaos-testing)
7. [Test Infrastructure](#7-test-infrastructure)
8. [Test Code Examples](#8-test-code-examples)
9. [Decision Matrices](#9-decision-matrices)
10. [Production Checklist](#10-production-checklist)
11. [References](#11-references)

---

## 1. Test Pyramid

### 1.1 Test Pyramid Overview

The test pyramid is a framework for structuring automated tests. The shape represents the proportion of tests at each layer.

```
                    ┌─────────────┐
                    │     E2E     │  5-10% - Few, slow, high confidence
                   ┌┴─────────────┴┐
                   │   Integration │  20-30% - Medium quantity, moderate speed
                  ┌┴───────────────┴┐
                  │      Unit       │  60-70% - Many, fast, isolated
                 ┌┴─────────────────┴┐
                 │   Component      │  ~10% - Optional layer for complex components
                ┌┴───────────────────┴┐
```

### 1.2 Layer Definitions

**Unit Tests (60-70%)**
- Test individual functions, methods, and classes
- Run in isolation without external dependencies
- Execute in milliseconds
- Written by developers
- High coverage target: 80%+

**Integration Tests (20-30%)**
- Test interactions between components
- May use real dependencies (database, message broker)
- Execute in seconds to minutes
- Written by developers and QA
- Cover critical paths

**End-to-End Tests (5-10%)**
- Test complete user flows
- Use real infrastructure
- Execute in minutes
- Written by QA and SDETs
- Cover happy paths and critical user journeys

### 1.3 Test Strategy Configuration

```yaml
# Testing strategy configuration
test_strategy:
  # Coverage requirements
  coverage:
    unit:
      minimum: 80
      target: 90
      methods_per_file:
        minimum: 70
        
    integration:
      minimum: 60
      target: 75
      critical_paths: 100
      
    e2e:
      minimum: 50
      target: 70
      critical_user_journeys: 100
      
  # Test execution
  execution:
    unit:
      parallel: true
      workers: 4
      rerun_failed: false
      timeout: 30s
      
    integration:
      parallel: true
      workers: 2
      rerun_failed: true
      timeout: 300s
      
    e2e:
      parallel: false
      workers: 1
      rerun_failed: false
      timeout: 600s
      
  # Quality gates
  quality_gates:
    unit:
      pass_rate: 100
      no_flaky_tests: true
      
    integration:
      pass_rate: 100
      flaky_detection: true
      retry_count: 2
      
    e2e:
      pass_rate: 95
      flaky_detection: true
      retry_count: 2
```

---

## 2. Unit Testing Patterns

### 2.1 Unit Test Structure

```python
# Standard unit test structure (AAA pattern)
# Arrange: Set up test data and dependencies
# Act: Execute the code under test
# Assert: Verify the results

class TestOrderService:
    """Unit tests for OrderService"""
    
    def test_create_order_with_valid_items_succeeds(self):
        # Arrange
        customer_id = uuid.uuid4()
        items = [
            OrderLineItem(product_id="SKU123", quantity=2, unit_price=29.99),
            OrderLineItem(product_id="SKU456", quantity=1, unit_price=49.99),
        ]
        shipping_address = ShippingAddress(
            street="123 Main St",
            city="San Francisco",
            state="CA",
            postal_code="94102",
            country="US"
        )
        
        mock_repo = Mock(spec=OrderRepository)
        mock_event_publisher = Mock(spec=EventPublisher)
        service = OrderService(
            repository=mock_repo,
            event_publisher=mock_event_publisher
        )
        
        # Act
        result = service.create_order(
            customer_id=customer_id,
            items=items,
            shipping_address=shipping_address
        )
        
        # Assert
        assert result.order_id is not None
        assert result.status == OrderStatus.CREATED
        assert result.total_amount == 109.97  # 2*29.99 + 49.99
        assert mock_repo.save.call_count == 1
        assert mock_event_publisher.publish.call_count == 1
        
    def test_create_order_with_empty_items_raises_error(self):
        # Arrange
        customer_id = uuid.uuid4()
        items = []
        shipping_address = ShippingAddress(
            street="123 Main St",
            city="San Francisco", 
            state="CA",
            postal_code="94102",
            country="US"
        )
        
        mock_repo = Mock(spec=OrderRepository)
        mock_event_publisher = Mock(spec=EventPublisher)
        service = OrderService(
            repository=mock_repo,
            event_publisher=mock_event_publisher
        )
        
        # Act & Assert
        with pytest.raises(ValidationError) as exc_info:
            service.create_order(
                customer_id=customer_id,
                items=items,
                shipping_address=shipping_address
            )
        assert "at least one item" in str(exc_info.value)
```

### 2.2 Test Doubles (Mocks, Stubs, Fakes)

```python
from unittest.mock import Mock, MagicMock, patch, call
from pytest import fixture

# Mock - Mock object with callable assertions
# Use when: You need to verify interactions occurred

def test_order_repository_save_is_called(self):
    mock_repo = Mock(spec=OrderRepository)
    mock_repo.save.return_value = Order(order_id="123")
    
    service = OrderService(repository=mock_repo)
    service.create_order(customer_id="cust1", items=[], shipping_address=addr)
    
    mock_repo.save.assert_called_once()

# Stub - Pre-programmed responses, no verification
# Use when: You just need the mock to return specific values

def test_order_repository_returns_stubbed_data(self):
    stub_repo = Mock(spec=OrderRepository)
    stub_repo.get_by_id.return_value = Order(order_id="123", status=OrderStatus.CREATED)
    
    service = OrderService(repository=stub_repo)
    order = service.get_order("123")
    
    assert order.order_id == "123"

# Fake - Working implementation (in-memory database)
# Use when: You need real behavior without external dependencies

class FakeOrderRepository:
    def __init__(self):
        self._orders = {}
        
    def save(self, order: Order) -> Order:
        self._orders[order.order_id] = order
        return order
        
    def get_by_id(self, order_id: str) -> Order:
        return self._orders.get(order_id)

def test_create_and_retrieve_order_with_fake():
    fake_repo = FakeOrderRepository()
    service = OrderService(repository=fake_repo)
    
    order = service.create_order(customer_id="cust1", items=[item], shipping_address=addr)
    retrieved = service.get_order(order.order_id)
    
    assert retrieved.order_id == order.order_id

# Spy - Wraps real object, tracks method calls
# Use when: You want real behavior but also verification

def test_event_publisher_spy_records_calls(self):
    spy_publisher = MagicMock(spec=EventPublisher)
    spy_publisher.publish.side_effect = lambda e: print(f"Published: {e}")
    
    service = OrderService(event_publisher=spy_publisher)
    service.create_order(customer_id="cust1", items=[], shipping_address=addr)
    
    assert spy_publisher.publish.call_count == 1
    call_args = spy_publisher.publish.call_args[0][0]
    assert call_args.event_type == "OrderCreated"
```

### 2.3 Parameterized Tests

```python
import pytest
from itertools import combinations

class TestOrderPricing:
    """Parameterized tests for pricing calculations"""
    
    @pytest.mark.parametrize("quantity,unit_price,expected_total", [
        (1, 10.00, 10.00),
        (2, 10.00, 20.00),
        (10, 5.50, 55.00),
        (100, 1.99, 199.00),
        (0, 10.00, 0.00),  # Edge case: zero quantity
    ])
    def test_line_item_total_calculation(self, quantity, unit_price, expected_total):
        item = OrderLineItem(
            product_id="SKU123",
            quantity=quantity,
            unit_price=unit_price
        )
        assert item.line_total == pytest.approx(expected_total)
        
    @pytest.mark.parametrize("discount_percent,expected_discount", [
        (0, 0.00),
        (10, 10.00),
        (25, 25.00),
        (50, 50.00),
        (100, 100.00),
    ])
    def test_discount_application(self, discount_percent, expected_discount):
        price = 100.00
        discount = price * (discount_percent / 100)
        assert discount == pytest.approx(expected_discount)
        
    @pytest.mark.parametrize("item_count,discount_threshold,expected_discount", [
        (1, 5, 0),    # No discount for single item
        (5, 5, 5),    # Exactly 5 items gets discount
        (10, 5, 10),  # 10% discount for 5+ items
        (20, 5, 10),  # 10% discount capped at 10%
    ])
    def test_bulk_discount_calculation(self, item_count, discount_threshold, expected_discount):
        total = item_count * 10.00
        discount = 0
        
        if item_count >= discount_threshold:
            discount = min(total * 0.1, 10.00)  # 10% discount, max $10
            
        assert discount == expected_discount
        
    # Test state transitions
    @pytest.mark.parametrize("current_status,action,expected_status", [
        (OrderStatus.DRAFT, "submit", OrderStatus.SUBMITTED),
        (OrderStatus.SUBMITTED, "confirm", OrderStatus.CONFIRMED),
        (OrderStatus.CONFIRMED, "ship", OrderStatus.SHIPPED),
        (OrderStatus.SHIPPED, "deliver", OrderStatus.DELIVERED),
        (OrderStatus.CONFIRMED, "cancel", OrderStatus.CANCELLED),
        (OrderStatus.SHIPPED, "cancel", OrderStatus.CANCELLED_PENDING),  # Requires return
    ])
    def test_order_status_transitions(self, current_status, action, expected_status):
        order = Order(status=current_status)
        order.transition(action)
        assert order.status == expected_status
```

### 2.4 Test Fixtures

```python
import pytest
from dataclasses import dataclass, field
from typing import List

@dataclass
class TestOrder:
    order_id: str = "test-order-123"
    customer_id: str = "test-customer-456"
    status: str = "CREATED"
    items: List = field(default_factory=list)
    total_amount: float = 0.0

@pytest.fixture
def sample_order_line_items():
    """Fixture providing sample line items"""
    return [
        OrderLineItem(
            product_id="SKU001",
            product_name="Widget A",
            quantity=2,
            unit_price=19.99
        ),
        OrderLineItem(
            product_id="SKU002",
            product_name="Widget B",
            quantity=1,
            unit_price=29.99
        ),
    ]

@pytest.fixture
def sample_shipping_address():
    """Fixture providing sample address"""
    return ShippingAddress(
        street="123 Test Street",
        city="Test City",
        state="CA",
        postal_code="90210",
        country="US"
    )

@pytest.fixture
def order_service(sample_order_line_items, sample_shipping_address):
    """Fixture providing configured OrderService"""
    mock_repo = Mock(spec=OrderRepository)
    mock_event_publisher = Mock(spec=EventPublisher)
    return OrderService(
        repository=mock_repo,
        event_publisher=mock_event_publisher
    )

class TestOrderServiceWithFixtures:
    def test_create_order_uses_fixtures(
        self,
        order_service,
        sample_order_line_items,
        sample_shipping_address
    ):
        result = order_service.create_order(
            customer_id="test-customer",
            items=sample_order_line_items,
            shipping_address=sample_shipping_address
        )
        
        assert result.order_id is not None
        assert result.items == sample_order_line_items
        
    def test_order_with_fixture_values(self, sample_order_line_items):
        total = sum(item.line_total for item in sample_order_line_items)
        assert total == pytest.approx(69.97)

# Fixture scopes
@pytest.fixture(scope="session")
def db_connection():
    """Session-scoped fixture - created once per test session"""
    conn = create_test_database()
    yield conn
    conn.close()

@pytest.fixture(scope="module")
def test_data():
    """Module-scoped fixture - created once per test module"""
    return load_test_data("module_data.json")

@pytest.fixture(scope="function")
def clean_order_repository():
    """Function-scoped fixture - created for each test"""
    repo = InMemoryOrderRepository()
    yield repo
    repo.clear()  # Clean up after test

@pytest.fixture(scope="function", autouse=True)
def reset_singleton_state():
    """Auto-use fixture that runs before each test"""
    SingletonClass.reset_instance()
    yield
    SingletonClass.reset_instance()
```

---

## 3. Integration Testing

### 3.1 Integration Test Configuration

```yaml
# Integration test configuration
integration_tests:
  # Testcontainers configuration
  testcontainers:
    enabled: true
    images:
      postgres:
        image: postgres:15-alpine
        tag: "15"
        environment:
          POSTGRES_DB: testdb
          POSTGRES_USER: testuser
          POSTGRES_PASSWORD: testpass
        ports:
          - 5432
        tmpfs:
          - /var/lib/postgresql/data
          
      redis:
        image: redis:7-alpine
        tag: "7"
        ports:
          - 6379
        command: redis-server --appendonly yes
          
      kafka:
        image: confluentinc/cp-kafka:7.5.0
        tag: "7.5.0"
        ports:
          - 9092
          - 29092
        environment:
          KAFKA_BROKER_ID: 1
          KAFKA_ZOOKEEPER_CONNECT: zookeeper:2181
          KAFKA_ADVERTISED_LISTENERS: PLAINTEXT://localhost:29092
          KAFKA_OFFSETS_TOPIC_REPLICATION_FACTOR: 1
          KAFKA_AUTO_CREATE_TOPICS_ENABLE: "true"
          KAFKA_GROUP_INITIAL_REBALANCE_DELAY_MS: 0
          
      elasticsearch:
        image: docker.elastic.co/elasticsearch/elasticsearch:8.10.0
        tag: "8.10.0"
        environment:
          discovery.type: single-node
          xpack.security.enabled: false
          ES_JAVA_OPTS: "-Xms512m -Xmx512m"
        ports:
          - 9200
          
  # Database migration
  migrations:
    auto_migrate: true
    migrate_before_each_test: false
    seed_data: true
    
  # Network configuration
  network:
    enable_networking: true
    dns_resolver: 8.8.8.8
    
  # Test isolation
  isolation:
    use_transaction_rollback: true
    cleanup_after_test: true
```

### 3.2 Integration Test Implementation

```python
import pytest
import testcontainers
from testcontainers.postgres import PostgresContainer
from testcontainers.redis import RedisContainer
from testcontainers.kafka import KafkaContainer
from sqlalchemy import create_engine, text
from sqlalchemy.orm import sessionmaker
import fakeredis

class TestDatabaseIntegration:
    """Integration tests with real database"""
    
    @pytest.fixture(scope="class")
    def postgres(self):
        """Start PostgreSQL container"""
        with PostgresContainer("postgres:15-alpine") as pg:
            yield pg
            
    @pytest.fixture(scope="class")
    def db_engine(self, postgres):
        """Create SQLAlchemy engine"""
        engine = create_engine(postgres.get_connection_url())
        yield engine
        engine.dispose()
        
    @pytest.fixture(scope="function")
    def db_session(self, db_engine):
        """Create fresh database session for each test"""
        # Run migrations
        with db_engine.begin() as conn:
            conn.execute(text("CREATE EXTENSION IF NOT EXISTS pgcrypto"))
            conn.execute(text("""
                CREATE TABLE IF NOT EXISTS orders (
                    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                    customer_id UUID NOT NULL,
                    status VARCHAR(50) NOT NULL,
                    total_amount DECIMAL(12, 2) NOT NULL,
                    created_at TIMESTAMPTZ DEFAULT NOW(),
                    updated_at TIMESTAMPTZ DEFAULT NOW()
                )
            """))
            
        Session = sessionmaker(bind=db_engine)
        session = Session()
        
        yield session
        
        session.rollback()
        session.close()

class TestOrderRepositoryIntegration(TestDatabaseIntegration):
    """Integration tests for OrderRepository with PostgreSQL"""
    
    def test_save_and_retrieve_order(self, db_session):
        # Arrange
        order = Order(
            customer_id=uuid.uuid4(),
            status=OrderStatus.CREATED,
            total_amount=109.99
        )
        
        # Act
        db_session.add(order)
        db_session.commit()
        
        # Assert
        retrieved = db_session.query(Order).filter_by(id=order.id).first()
        assert retrieved is not None
        assert retrieved.id == order.id
        assert retrieved.total_amount == 109.99
        
    def test_update_order_status(self, db_session):
        # Arrange
        order = Order(
            customer_id=uuid.uuid4(),
            status=OrderStatus.CREATED,
            total_amount=50.00
        )
        db_session.add(order)
        db_session.commit()
        
        # Act
        order.status = OrderStatus.CONFIRMED
        db_session.commit()
        
        # Assert
        db_session.refresh(order)
        assert order.status == OrderStatus.CONFIRMED
        
    def test_concurrent_updates_handled(self, db_session):
        # Arrange
        order = Order(
            customer_id=uuid.uuid4(),
            status=OrderStatus.CREATED,
            total_amount=100.00
        )
        db_session.add(order)
        db_session.commit()
        order_id = order.id
        
        # Create separate sessions to simulate concurrent access
        Session2 = sessionmaker(bind=db_session.get_bind())
        session2 = Session2()
        
        # Act - First transaction
        order1 = db_session.query(Order).filter_by(id=order_id).first()
        order1.total_amount = 110.00
        db_session.commit()
        
        # Second transaction should detect conflict
        order2 = session2.query(Order).filter_by(id=order_id).first()
        order2.total_amount = 120.00
        
        # Assert
        with pytest.raises(StaleDataError):
            session2.commit()
            
        session2.close()


class TestRedisCacheIntegration:
    """Integration tests for Redis caching"""
    
    @pytest.fixture(scope="class")
    def redis(self):
        """Start Redis container"""
        with RedisContainer("redis:7-alpine") as redis:
            yield redis
            
    @pytest.fixture
    def redis_client(self, redis):
        """Create Redis client"""
        import redis as redis_lib
        client = redis_lib.Redis.from_url(redis.get_connection_url())
        yield client
        client.flushdb()
        
    def test_cache_order(self, redis_client):
        # Arrange
        order_id = "order-123"
        order_data = {"id": order_id, "total": 99.99}
        
        # Act
        redis_client.hset("orders", order_id, json.dumps(order_data))
        
        # Assert
        cached = redis_client.hget("orders", order_id)
        assert cached is not None
        assert json.loads(cached) == order_data
        
    def test_cache_invalidation(self, redis_client):
        # Arrange
        order_id = "order-123"
        redis_client.hset("orders", order_id, json.dumps({"id": order_id}))
        
        # Act
        redis_client.hdel("orders", order_id)
        
        # Assert
        assert redis_client.hget("orders", order_id) is None
        
    def test_cache_ttl(self, redis_client):
        # Arrange
        order_id = "order-123"
        redis_client.setex(f"order:{order_id}", 1, "test")  # 1 second TTL
        
        # Assert initial
        assert redis_client.get(f"order:{order_id}") == b"test"
        
        import time
        time.sleep(1.1)
        
        # Assert expired
        assert redis_client.get(f"order:{order_id}") is None


class TestKafkaIntegration:
    """Integration tests with Kafka"""
    
    @pytest.fixture(scope="class")
    def kafka(self):
        """Start Kafka container"""
        with KafkaContainer("confluentinc/cp-kafka:7.5.0") as kafka:
            yield kafka
            
    @pytest.fixture
    def kafka_producer(self, kafka):
        """Create Kafka producer"""
        from confluent_kafka import Producer
        
        conf = {
            'bootstrap.servers': kafka.get_bootstrap_server(),
            'client.id': 'test-producer',
        }
        producer = Producer(conf)
        yield producer
        producer.flush()
        
    @pytest.fixture
    def kafka_consumer(self, kafka):
        """Create Kafka consumer"""
        from confluent_kafka import Consumer
        
        conf = {
            'bootstrap.servers': kafka.get_bootstrap_server(),
            'group.id': 'test-group',
            'auto.offset.reset': 'earliest',
            'enable.auto.commit': True,
        }
        consumer = Consumer(conf)
        consumer.subscribe(['test-topic'])
        yield consumer
        consumer.close()
        
    def test_produce_and_consume_message(self, kafka_producer, kafka_consumer):
        # Arrange
        test_message = {"order_id": "123", "amount": 99.99}
        
        # Act
        kafka_producer.produce(
            'test-topic',
            key='order-123',
            value=json.dumps(test_message).encode('utf-8')
        )
        kafka_producer.flush()
        
        # Poll for message
        msg = kafka_consumer.poll(timeout=5.0)
        
        # Assert
        assert msg is not None
        assert json.loads(msg.value().decode('utf-8')) == test_message
```

---

## 4. End-to-End Testing

### 4.1 E2E Test Configuration

```yaml
# E2E test configuration
e2e_tests:
  # Test environment
  environment:
    type: kubernetes  # Options: local, kubernetes, docker-compose
    namespace: e2e-test
    service_account: e2e-test-runner
    
  # Browser automation
  browsers:
    chrome:
      enabled: true
      version: 120
      headless: true
      args:
        - "--no-sandbox"
        - "--disable-dev-shm-usage"
        - "--disable-gpu"
        - "--window-size=1920,1080"
        
    firefox:
      enabled: true
      version: 121
      headless: true
      
    safari:
      enabled: false
      
  # Mobile emulation
  mobile:
    iphone:
      enabled: true
      user_agent: "Mozilla/5.0 (iPhone; CPU iPhone OS 16_0 like Mac OS X)"
      
    android:
      enabled: true
      
  # Viewport sizes
  viewports:
    desktop:
      width: 1920
      height: 1080
    tablet:
      width: 768
      height: 1024
    mobile:
      width: 375
      height: 667
      
  # Wait times (milliseconds)
  waits:
    implicit: 5000
    explicit: 10000
    page_load: 30000
    
  # Recording
  video:
    enabled: true
    record_on_failure_only: true
    save_path: /test-results/videos
    
  # Screenshots
  screenshots:
    enabled: true
    on_failure: true
    on_success: false
    full_page: true
```

### 4.2 E2E Test Implementation

```python
import pytest
from playwright.sync_api import sync_playwright, expect
from dataclasses import dataclass

@dataclass
class TestUser:
    email: str
    password: str
    name: str

@pytest.fixture
def browser_context():
    """Configure browser context"""
    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        context = browser.new_context(
            viewport={"width": 1920, "height": 1080},
            record_video_dir="/test-results/videos",
            record_video_size={"width": 1920, "height": 1080},
        )
        yield context
        context.close()
        browser.close()

@pytest.fixture
def authenticated_context(browser_context):
    """Create authenticated context"""
    page = browser_context.new_page()
    
    # Perform login
    page.goto("https://app.example.com/login")
    page.fill('[name="email"]', "test@example.com")
    page.fill('[name="password"]', "testpassword")
    page.click('[type="submit"]')
    
    # Wait for redirect
    page.wait_for_url("**/dashboard")
    
    yield page
    
    page.close()

class TestOrderWorkflowE2E:
    """End-to-end tests for order workflow"""
    
    def test_complete_order_flow(self, authenticated_context):
        """Test complete order creation flow"""
        page = authenticated_context
        
        # 1. Navigate to order page
        page.click('[data-testid="new-order-btn"]')
        page.wait_for_url("**/orders/new")
        
        # 2. Add items to cart
        page.fill('[data-testid="product-search"]', "Widget A")
        page.wait_for_selector('[data-testid="search-results"]')
        page.click('[data-testid="product-Widget-A"] [data-testid="add-btn"]')
        
        # Verify item added
        expect(page.locator('[data-testid="cart-items"]')).to_contain_text("Widget A")
        
        # 3. Adjust quantity
        page.fill('[data-testid="quantity-input"]', "3")
        page.click('[data-testid="update-quantity-btn"]')
        
        # 4. Proceed to checkout
        page.click('[data-testid="checkout-btn"]')
        page.wait_for_url("**/checkout")
        
        # 5. Fill shipping address
        page.fill('[name="street"]', "123 Test Street")
        page.fill('[name="city"]', "San Francisco")
        page.fill('[name="state"]', "CA")
        page.fill('[name="postalCode"]', "94102")
        page.fill('[name="country"]', "US")
        
        # 6. Select payment method
        page.click('[data-testid="payment-method-card"]')
        
        # 7. Review order
        page.click('[data-testid="review-order-btn"]')
        page.wait_for_url("**/review")
        
        # 8. Submit order
        page.click('[data-testid="submit-order-btn"]')
        
        # 9. Verify confirmation
        page.wait_for_url("**/confirmation/**")
        expect(page.locator('[data-testid="confirmation-message"]')).to_contain_text("Order placed successfully")
        
        # Extract order number
        order_number = page.locator('[data-testid="order-number"]').text_content()
        assert order_number.startswith("ORD-")
        
    def test_order_cancellation_flow(self, authenticated_context):
        """Test order cancellation"""
        page = authenticated_context
        
        # Navigate to existing order
        page.goto("https://app.example.com/orders")
        page.click('[data-testid="order-ORD-123"]')
        
        # Wait for order details
        page.wait_for_selector('[data-testid="order-details"]')
        
        # Cancel order
        page.click('[data-testid="cancel-order-btn"]')
        
        # Confirm cancellation
        page.click('[data-testid="confirm-cancel-btn"]')
        
        # Verify cancelled status
        expect(page.locator('[data-testid="order-status"]')).to_contain_text("Cancelled")
        
    def test_payment_failure_handling(self, authenticated_context):
        """Test handling of payment failure"""
        page = authenticated_context
        
        # Navigate to checkout with insufficient funds card
        page.goto("https://app.example.com/checkout")
        
        # Fill invalid card details
        page.fill('[name="cardNumber"]', "4000000000000002")  # Stripe test decline card
        page.fill('[name="expiry"]', "12/25")
        page.fill('[name="cvc"]', "123")
        
        # Submit order
        page.click('[data-testid="submit-payment-btn"]')
        
        # Verify error message
        expect(page.locator('[data-testid="payment-error"]')).to_contain_text("Your card was declined")
        
        # Verify order is not created
        page.goto("https://app.example.com/orders")
        assert page.locator('[data-testid="order-ORD-new"]').count() == 0


class TestAPIIntegrationE2E:
    """API integration tests using Playwright"""
    
    def test_api_health_check(self, authenticated_context):
        """Verify API health endpoint"""
        page = authenticated_context
        
        response = page.request.get("https://api.example.com/health")
        assert response.status == 200
        assert response.json()["status"] == "healthy"
        
    def test_api_authentication(self, authenticated_context):
        """Verify API authentication works"""
        page = authenticated_context
        
        # Make authenticated API request
        response = page.request.get(
            "https://api.example.com/v1/orders",
            headers={"Authorization": f"Bearer {page.context.token}"}
        )
        
        assert response.status == 200
```

### 4.3 API Contract Testing

```python
import pytest
from pact import Pact, Verifier

class TestOrderServiceContract:
    """Contract tests for Order Service"""
    
    @pytest.fixture
    def pact(self):
        return Pact(
            consumer="web-frontend",
            provider="order-service",
            host="localhost",
            port=8080
        )
        
    def test_order_creation_contract(self, pact):
        """Test contract for order creation"""
        (pact
         .given("a customer exists")
         .upon_receiving("a request to create an order")
         .with_request(
             method="POST",
             path="/v1/orders",
             headers={"Content-Type": "application/json"},
             body={
                 "customerId": "customer-123",
                 "items": [
                     {"productId": "SKU001", "quantity": 2, "unitPrice": 29.99}
                 ],
                 "shippingAddress": {
                     "street": "123 Test St",
                     "city": "Test City",
                     "state": "CA",
                     "postalCode": "90210",
                     "country": "US"
                 }
             }
         )
         .will_respond_with(
             status=201,
             headers={"Content-Type": "application/json"},
             body={
                 "orderId": pact.term(r"[a-f0-9-]{36}", "order-123-uuid"),
                 "status": "CREATED",
                 "totalAmount": 59.98,
                 "createdAt": pact.term(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z", "2024-01-15T10:30:00Z")
             }
         ))
```

---

## 5. Performance Testing

### 5.1 Performance Test Configuration

```yaml
# Performance test configuration
performance_tests:
  # Load testing
  load_test:
    engine: k6  # Options: k6, gatling, locust, artillery
    
    # Test scenarios
    scenarios:
      light_load:
        duration: 60s
        vus: 10
        think_time: 2s
        
      normal_load:
        duration: 300s
        stages:
          - duration: 60s
            target: 50
          - duration: 180s
            target: 50
          - duration: 60s
            target: 0
        think_time: 1s
            
      peak_load:
        duration: 120s
        stages:
          - duration: 30s
            target: 100
          - duration: 60s
            target: 200
          - duration: 30s
            target: 0
        think_time: 0.5s
            
      stress_test:
        duration: 300s
        stages:
          - duration: 60s
            target: 100
          - duration: 120s
            target: 500
          - duration: 60s
            target: 1000
          - duration: 60s
            target: 0
        think_time: 0s
        
      spike_test:
        duration: 120s
        stages:
          - duration: 30s
            target: 50
          - duration: 10s
            target: 500
          - duration: 60s
            target: 500
          - duration: 20s
            target: 0
            
      soak_test:
        duration: 24h
        target: 100
        think_time: 1s
        
  # Thresholds
  thresholds:
    http_req_duration:
      p95: 200ms
      p99: 500ms
      avg: 100ms
    http_req_failed:
      rate: 0.01  # 1% failure rate max
    checks:
      health_check:
        threshold: 0.95  # 95% of checks must pass
        
  # Metrics collection
  metrics:
    influxdb:
      enabled: true
      url: http://influxdb.monitoring.svc.cluster.local:8086
      database: k6
    prometheus:
      enabled: true
      pushgateway: http://pushgateway.monitoring.svc.cluster.local:9091
    datadog:
      enabled: false
```

### 5.2 k6 Performance Test Scripts

```javascript
// order_service_load_test.js
import http from 'k6/http';
import { check, sleep, group } from 'k6';
import { Rate, Trend, Counter } from 'k6/metrics';

// Custom metrics
const orderCreationDuration = new Trend('order_creation_duration');
const orderRetrievalDuration = new Trend('order_retrieval_duration');
const orderListDuration = new Trend('order_list_duration');
const errorRate = new Rate('errors');

// Test configuration
export const options = {
  stages: [
    { duration: '60s', target: 50 },
    { duration: '180s', target: 50 },
    { duration: '60s', target: 0 },
  ],
  thresholds: {
    'http_req_duration': ['p(95)<500', 'p(99)<1000'],
    'http_req_failed': ['rate<0.01'],
    'order_creation_duration': ['p(95)<300'],
    'order_retrieval_duration': ['p(95)<100'],
  },
};

const BASE_URL = __ENV.TARGET_URL || 'https://api.example.com';

// Test data generation
function generateOrderItems() {
  const items = [];
  const numItems = Math.floor(Math.random() * 5) + 1;
  
  for (let i = 0; i < numItems; i++) {
    items.push({
      productId: `SKU${Math.floor(Math.random() * 1000)}`,
      quantity: Math.floor(Math.random() * 5) + 1,
      unitPrice: Math.random() * 100
    });
  }
  return items;
}

export function setup() {
  // Create test data
  const authResponse = http.post(`${BASE_URL}/v1/auth/token`, {
    grant_type: 'client_credentials',
    client_id: __ENV.CLIENT_ID,
    client_secret: __ENV.CLIENT_SECRET,
  });
  
  return {
    token: authResponse.json().access_token,
    customerIds: Array.from({ length: 100 }, (_, i) => `customer-${i}`),
  };
}

export default function(data) {
  const headers = {
    'Authorization': `Bearer ${data.token}`,
    'Content-Type': 'application/json',
    'X-Correlation-ID': `${__VU}-${__ITER}-${Date.now()}`,
  };
  
  // Scenario 1: Create Order
  group('Order Creation', () => {
    const orderPayload = {
      customerId: data.customerIds[Math.floor(Math.random() * data.customerIds.length)],
      items: generateOrderItems(),
      shippingAddress: {
        street: '123 Test Street',
        city: 'San Francisco',
        state: 'CA',
        postalCode: '94102',
        country: 'US',
      },
    };
    
    const startTime = Date.now();
    const response = http.post(
      `${BASE_URL}/v1/orders`,
      JSON.stringify(orderPayload),
      { headers }
    );
    orderCreationDuration.add(Date.now() - startTime);
    
    const success = check(response, {
      'order created with status 201': (r) => r.status === 201,
      'order has id': (r) => r.json('orderId') !== undefined,
      'order status is CREATED': (r) => r.json('status') === 'CREATED',
    });
    
    errorRate.add(!success);
    
    if (response.status === 201) {
      return response.json('orderId');
    }
    return null;
  });
  
  // Scenario 2: Retrieve Order
  group('Order Retrieval', () => {
    // First create an order to retrieve
    const orderPayload = {
      customerId: data.customerIds[0],
      items: generateOrderItems(),
      shippingAddress: {
        street: '123 Test Street',
        city: 'San Francisco',
        state: 'CA',
        postalCode: '94102',
        country: 'US',
      },
    };
    
    const createResponse = http.post(
      `${BASE_URL}/v1/orders`,
      JSON.stringify(orderPayload),
      { headers }
    );
    
    if (createResponse.status !== 201) {
      return;
    }
    
    const orderId = createResponse.json('orderId');
    
    // Now retrieve it
    const startTime = Date.now();
    const response = http.get(
      `${BASE_URL}/v1/orders/${orderId}`,
      { headers }
    );
    orderRetrievalDuration.add(Date.now() - startTime);
    
    check(response, {
      'order retrieved with status 200': (r) => r.status === 200,
      'order data matches': (r) => r.json('orderId') === orderId,
    });
  });
  
  // Scenario 3: List Orders
  group('Order Listing', () => {
    const startTime = Date.now();
    const response = http.get(
      `${BASE_URL}/v1/orders?page=1&pageSize=20`,
      { headers }
    );
    orderListDuration.add(Date.now() - startTime);
    
    check(response, {
      'orders listed with status 200': (r) => r.status === 200,
      'pagination present': (r) => r.json('pagination') !== undefined,
    });
  });
  
  // Scenario 4: Update Order Status
  group('Order Status Update', () => {
    // Create order first
    const orderPayload = {
      customerId: data.customerIds[0],
      items: generateOrderItems(),
      shippingAddress: {
        street: '123 Test Street',
        city: 'San Francisco',
        state: 'CA',
        postalCode: '94102',
        country: 'US',
      },
    };
    
    const createResponse = http.post(
      `${BASE_URL}/v1/orders`,
      JSON.stringify(orderPayload),
      { headers }
    );
    
    if (createResponse.status !== 201) {
      return;
    }
    
    const orderId = createResponse.json('orderId');
    
    // Update status
    const updateResponse = http.patch(
      `${BASE_URL}/v1/orders/${orderId}/status`,
      JSON.stringify({ status: 'CONFIRMED' }),
      { headers }
    );
    
    check(updateResponse, {
      'order updated with status 200': (r) => r.status === 200,
      'status updated': (r) => r.json('status') === 'CONFIRMED',
    });
  });
  
  sleep(1);
}

export function handleSummary(data) {
  return {
    'stdout': textSummary(data, { indent: ' ', enableColors: true }),
    'summary.json': JSON.stringify(data),
  };
}
```

### 5.3 Database Performance Testing

```sql
-- Database performance test queries

-- Test query: Order lookup by customer
EXPLAIN ANALYZE
SELECT 
    o.id,
    o.order_number,
    o.status,
    o.total_amount,
    o.created_at,
    json_agg(
        json_build_object(
            'product_id', oi.product_id,
            'product_name', p.name,
            'quantity', oi.quantity,
            'unit_price', oi.unit_price
        )
    ) as items
FROM orders o
JOIN order_items oi ON o.id = oi.order_id
JOIN products p ON oi.product_id = p.id
WHERE o.customer_id = 'customer-123'
  AND o.created_at > NOW() - INTERVAL '30 days'
GROUP BY o.id, o.order_number, o.status, o.total_amount, o.created_at
ORDER BY o.created_at DESC
LIMIT 20;

-- Test query: Aggregate revenue by product category
EXPLAIN ANALYZE
SELECT 
    p.category,
    COUNT(DISTINCT o.id) as order_count,
    SUM(oi.quantity) as total_units_sold,
    SUM(oi.quantity * oi.unit_price) as total_revenue
FROM orders o
JOIN order_items oi ON o.id = oi.order_id
JOIN products p ON oi.product_id = p.id
WHERE o.status IN ('CONFIRMED', 'SHIPPED', 'DELIVERED')
  AND o.created_at > NOW() - INTERVAL '7 days'
GROUP BY p.category
ORDER BY total_revenue DESC;
```

---

## 6. Chaos Testing

### 6.1 Chaos Engineering Configuration

```yaml
# Chaos engineering configuration
chaos_engineering:
  # Framework: Chaos Monkey, Gremlin, Litmus, Chaos Mesh
  framework: chaos_mesh
  
  # Experiment configuration
  experiments:
    # Network chaos
    network_partition:
      enabled: true
      probability: 0.01  # 1% chance per minute
      duration: 30s
      target:
        services:
          - order-service
          - payment-service
        namespaces:
          - platform
      action:
        delay:
          enabled: true
          latency: 500ms
          jitter: 100ms
        loss:
          enabled: false
          rate: 10
        corrupt:
          enabled: false
          rate: 5
          
    # Pod failure
    pod_kill:
      enabled: true
      probability: 0.001  # 0.1% chance per minute
      target:
        services:
          - order-service
          - inventory-service
      action:
        kill_count: 1
        grace_period: 30s
        
    # Resource exhaustion
    resource_exhaustion:
      enabled: true
      probability: 0.005
      target:
        services:
          - order-service
      action:
        cpu_stress:
          enabled: true
          workers: 2
          load: 80
        memory_stress:
          enabled: true
          workers: 1
          size: 1GB
          
    # Dependency failure
    database_failure:
      enabled: true
      probability: 0.001
      target:
        services:
          - postgres
      action:
        connection_pool_exhaustion:
          enabled: true
          max_connections: 100%
        query_latency:
          enabled: true
          latency: 5000ms
          probability: 50
          
    # DNS failure
    dns_failure:
      enabled: true
      probability: 0.005
      target:
        services:
          - order-service
      action:
        error_rate: 100
        timeout: 5000ms
        nxdomain: false
        
    # Latency injection
    latency_injection:
      enabled: true
      probability: 0.01
      target:
        services:
          - order-service
      action:
        delay: 2000ms
        jitter: 500ms
        target_port: 8080
        
    # Message broker failure
    kafka_failure:
      enabled: true
      probability: 0.001
      target:
        services:
          - kafka
      action:
        partition_leader_election_delay:
          enabled: true
          delay: 30000ms
        broker_pod_kill:
          enabled: true
          kill_count: 1
          
  # Scheduling
  scheduling:
    enabled: true
    schedule: "0 * * * *"  # Every hour
    random_time_range: 600  # Randomize up to 10 minutes
    
  # Safety
  safety:
    max_concurrent_experiments: 1
    experiment_timeout: 5m
    auto_rollback: true
    blast_radius_limit:
      max_affected_pods: 1
      max_affected_percentage: 10
    notification:
      enabled: true
      channels:
        - slack: "#chaos-alerts"
        - pagerduty: true
        
  # Steady state hypothesis
  steady_state:
    order_service_health:
      - name: api_responds
        probe:
          type: http
          url: http://order-service.platform.svc.cluster.local:8080/health/ready
          timeout: 5s
          expected_status: 200
          
      - name: p99_under_500ms
        probe:
          type: metric
          query: histogram_quantile(0.99, rate(http_request_duration_seconds_bucket{service="order-service"}[5m])) < 0.5
          
    order_creation_works:
      - name: create_order_succeeds
        probe:
          type: http
          method: POST
          url: http://order-service.platform.svc.cluster.local:8080/v1/orders
          body:
            customerId: "test-customer"
            items:
              - productId: "SKU001"
                quantity: 1
                unitPrice: 10.00
          timeout: 10s
          expected_status: 201
```

### 6.2 Chaos Experiment Implementation

```python
# chaos_experiments.py

from chaosmesh import experiment
from chaosmesh.experiments import podkill, networkdelay, networkloss
from chaosmesh.targerts import pods
from kubernetes import client, config

# Load kubernetes config
config.load_incluster_config()

class ChaosExperimentRunner:
    """Run chaos experiments against the platform"""
    
    def __init__(self, namespace="platform"):
        self.namespace = namespace
        self.core_v1 = client.CoreV1Api()
        
    @experiment(
        name="order-service-pod-kill",
        description="Kill order-service pods to test resilience",
        steady_state_probe=order_service_steady_state,
    )
    def order_service_pod_kill(self):
        """Kill 1 order-service pod"""
        target = pods(
            namespace=self.namespace,
            label_selectors={"app": "order-service"}
        )
        
        podkill(
            target=target,
            count=1,
            grace_period=30,
        )
        
    @experiment(
        name="order-service-network-delay",
        description="Inject network delay to test timeout handling",
        steady_state_probe=order_service_steady_state,
    )
    def order_service_network_delay(self):
        """Add 2 second delay to order-service"""
        target = pods(
            namespace=self.namespace,
            label_selectors={"app": "order-service"}
        )
        
        networkdelay(
            target=target,
            delay=2000,  # 2 seconds
            jitter=500,
            duration=60,
        )
        
    @experiment(
        name="database-connection-exhaustion",
        description="Simulate database connection pool exhaustion",
        steady_state_probe=order_service_steady_state,
    )
    def database_connection_exhaustion(self):
        """Inject connection delays to database"""
        target = pods(
            namespace=self.namespace,
            label_selectors={"app": "postgres"}
        )
        
        networkdelay(
            target=target,
            delay=5000,  # 5 second delay
            duration=120,
        )
```

---

## 7. Test Infrastructure

### 7.1 Test Environment Configuration

```yaml
# Test infrastructure configuration
test_infrastructure:
  # CI/CD integration
  ci:
    provider: github_actions  # Options: github_actions, gitlab_ci, jenkins, argo
  
  # Container registry
  container_registry:
    url: ghcr.io/example
    username: ${CI_REGISTRY_USER}
    token: ${CI_REGISTRY_TOKEN}
    
  # Test execution
  execution:
    parallelization:
      unit: 8
      integration: 4
      e2e: 1
      
    retry:
      unit: 0
      integration: 2
      e2e: 2
      
    timeout:
      unit: 5m
      integration: 30m
      e2e: 60m
      
  # Test data management
  test_data:
    generation:
      enabled: true
      strategy: synthetic
      cleanup: after_each_test
    seeding:
      enabled: true
      snapshot_based: true
      
  # Quality gates
  quality_gates:
    unit:
      min_coverage: 80
      max_complexity: 15
      max_duplication: 5
      
    integration:
      min_coverage: 60
      max_flaky_rate: 5
      
    e2e:
      min_coverage: 50
      max_flaky_rate: 5
      
  # Notifications
  notifications:
    slack:
      webhook: ${SLACK_WEBHOOK}
      channel: "#test-results"
      
    email:
      smtp_host: smtp.example.com
      recipients:
        - platform-team@example.com
```

---

## 8. Test Code Examples

### 8.1 Test Class Patterns

```python
# test_order_service.py - Comprehensive test class example

import pytest
from unittest.mock import Mock, MagicMock, AsyncMock, patch
from dataclasses import dataclass, field
from datetime import datetime, timedelta
from typing import List, Optional
import uuid

# Import the system under test
from order_service import OrderService, Order, OrderStatus, ValidationError
from event_publisher import EventPublisher, Event
from repository import OrderRepository

# ============================================================================
# FIXTURES
# ============================================================================

@pytest.fixture
def mock_repository():
    """Create mock repository"""
    repo = Mock(spec=OrderRepository)
    repo.save = MagicMock()
    repo.get_by_id = MagicMock(return_value=None)
    repo.list_by_customer = MagicMock(return_value=[])
    return repo

@pytest.fixture
def mock_event_publisher():
    """Create mock event publisher"""
    publisher = Mock(spec=EventPublisher)
    publisher.publish = MagicMock()
    publisher.publish_batch = MagicMock()
    return publisher

@pytest.fixture
def order_service(mock_repository, mock_event_publisher):
    """Create OrderService with mocked dependencies"""
    return OrderService(
        repository=mock_repository,
        event_publisher=mock_event_publisher,
        config=OrderServiceConfig(
            max_items_per_order=100,
            max_retry_attempts=3,
            event_publish_timeout=5,
        )
    )

@pytest.fixture
def valid_customer_id():
    return str(uuid.uuid4())

@pytest.fixture
def valid_order_items():
    return [
        OrderLineItem(product_id="SKU001", quantity=2, unit_price=29.99),
        OrderLineItem(product_id="SKU002", quantity=1, unit_price=49.99),
    ]

@pytest.fixture
def valid_shipping_address():
    return ShippingAddress(
        street="123 Test Street",
        city="San Francisco",
        state="CA",
        postal_code="94102",
        country="US"
    )

# ============================================================================
# TEST CLASS: Order Creation
# ============================================================================

class TestOrderCreation:
    """Tests for order creation functionality"""
    
    def test_create_order_with_valid_input_succeeds(
        self,
        order_service,
        valid_customer_id,
        valid_order_items,
        valid_shipping_address
    ):
        """
        Test that a valid order can be created successfully.
        
        Expected behavior:
        - Order is created with generated ID
        - Status is set to CREATED
        - Total is calculated correctly
        - Repository save is called
        - OrderCreated event is published
        """
        # Act
        result = order_service.create_order(
            customer_id=valid_customer_id,
            items=valid_order_items,
            shipping_address=valid_shipping_address,
            notes="Test order"
        )
        
        # Assert
        assert result.order_id is not None
        assert result.status == OrderStatus.CREATED
        assert result.customer_id == valid_customer_id
        assert len(result.items) == 2
        assert result.total_amount == pytest.approx(109.97)  # 2*29.99 + 49.99
        assert result.created_at is not None
        
        # Verify interactions
        order_service.repository.save.assert_called_once()
        order_service.event_publisher.publish.assert_called_once()
        
        # Verify event content
        published_event = order_service.event_publisher.publish.call_args[0][0]
        assert published_event.event_type == "OrderCreated"
        assert published_event.payload["order_id"] == result.order_id
        
    def test_create_order_with_empty_items_raises_error(
        self,
        order_service,
        valid_customer_id,
        valid_shipping_address
    ):
        """Test that creating order with no items raises ValidationError"""
        with pytest.raises(ValidationError) as exc_info:
            order_service.create_order(
                customer_id=valid_customer_id,
                items=[],
                shipping_address=valid_shipping_address
            )
            
        assert "at least one item" in str(exc_info.value).lower()
        
    def test_create_order_with_too_many_items_raises_error(
        self,
        order_service,
        valid_customer_id,
        valid_shipping_address
    ):
        """Test that creating order with too many items raises ValidationError"""
        too_many_items = [
            OrderLineItem(product_id=f"SKU{i}", quantity=1, unit_price=10.00)
            for i in range(150)  # Exceeds 100 item limit
        ]
        
        with pytest.raises(ValidationError) as exc_info:
            order_service.create_order(
                customer_id=valid_customer_id,
                items=too_many_items,
                shipping_address=valid_shipping_address
            )
            
        assert "too many items" in str(exc_info.value).lower()
        
    def test_create_order_with_invalid_shipping_address_raises_error(
        self,
        order_service,
        valid_customer_id,
        valid_order_items
    ):
        """Test that invalid shipping address raises ValidationError"""
        invalid_address = ShippingAddress(
            street="",
            city="",
            state="",
            postal_code="",
            country=""
        )
        
        with pytest.raises(ValidationError) as exc_info:
            order_service.create_order(
                customer_id=valid_customer_id,
                items=valid_order_items,
                shipping_address=invalid_address
            )
            
        assert "shipping address" in str(exc_info.value).lower()

# ============================================================================
# TEST CLASS: Order Retrieval
# ============================================================================

class TestOrderRetrieval:
    """Tests for order retrieval functionality"""
    
    def test_get_order_by_id_existing_order_returns_order(
        self,
        order_service,
        valid_customer_id
    ):
        """Test that getting existing order returns order data"""
        # Arrange
        expected_order = Order(
            order_id="order-123",
            customer_id=valid_customer_id,
            status=OrderStatus.CREATED,
            total_amount=99.99,
            items=[],
        )
        order_service.repository.get_by_id.return_value = expected_order
        
        # Act
        result = order_service.get_order("order-123")
        
        # Assert
        assert result is not None
        assert result.order_id == "order-123"
        order_service.repository.get_by_id.assert_called_once_with("order-123")
        
    def test_get_order_by_id_non_existing_order_returns_none(
        self,
        order_service
    ):
        """Test that getting non-existing order returns None"""
        order_service.repository.get_by_id.return_value = None
        
        result = order_service.get_order("non-existent")
        
        assert result is None

# ============================================================================
# TEST CLASS: Order Updates
# ============================================================================

class TestOrderUpdates:
    """Tests for order update functionality"""
    
    def test_confirm_order_transitions_status(
        self,
        order_service,
        valid_customer_id,
        valid_order_items,
        valid_shipping_address
    ):
        """Test that confirming order transitions status to CONFIRMED"""
        # Arrange
        order = Order(
            order_id="order-123",
            customer_id=valid_customer_id,
            status=OrderStatus.CREATED,
            total_amount=99.99,
            items=[],
        )
        order_service.repository.get_by_id.return_value = order
        
        # Act
        result = order_service.confirm_order("order-123")
        
        # Assert
        assert result.status == OrderStatus.CONFIRMED
        order_service.repository.save.assert_called()
        
        # Verify event published
        published_event = order_service.event_publisher.publish.call_args[0][0]
        assert published_event.event_type == "OrderConfirmed"
        
    def test_confirm_already_confirmed_order_raises_error(
        self,
        order_service
    ):
        """Test that confirming already confirmed order raises error"""
        order = Order(
            order_id="order-123",
            customer_id="customer-1",
            status=OrderStatus.CONFIRMED,
            total_amount=99.99,
            items=[],
        )
        order_service.repository.get_by_id.return_value = order
        
        with pytest.raises(InvalidOperationError) as exc_info:
            order_service.confirm_order("order-123")
            
        assert "already confirmed" in str(exc_info.value).lower()

# ============================================================================
# TEST CLASS: Error Handling
# ============================================================================

class TestErrorHandling:
    """Tests for error handling scenarios"""
    
    def test_repository_save_failure_raises_error(
        self,
        order_service,
        valid_customer_id,
        valid_order_items,
        valid_shipping_address
    ):
        """Test that repository save failure propagates as error"""
        order_service.repository.save.side_effect = DatabaseError("Connection failed")
        
        with pytest.raises(DatabaseError):
            order_service.create_order(
                customer_id=valid_customer_id,
                items=valid_order_items,
                shipping_address=valid_shipping_address
            )
            
    def test_event_publish_failure_does_not_fail_order_creation(
        self,
        order_service,
        valid_customer_id,
        valid_order_items,
        valid_shipping_address
    ):
        """Test that event publish failure doesn't fail order creation"""
        order_service.event_publisher.publish.side_effect = EventPublishError("Queue full")
        
        # Should not raise - order should still be created
        result = order_service.create_order(
            customer_id=valid_customer_id,
            items=valid_order_items,
            shipping_address=valid_shipping_address
        )
        
        assert result is not None
        assert result.order_id is not None
        
    def test_timeout_handling(
        self,
        order_service,
        valid_customer_id,
        valid_order_items,
        valid_shipping_address
    ):
        """Test that operations timeout correctly"""
        order_service.repository.save.side_effect = TimeoutError("Operation timed out")
        
        with pytest.raises(TimeoutError):
            order_service.create_order(
                customer_id=valid_customer_id,
                items=valid_order_items,
                shipping_address=valid_shipping_address
            )
```

---

## 9. Decision Matrices

### 9.1 Test Type Selection

| Requirement | Unit | Integration | E2E | Performance | Chaos |
|-------------|------|-------------|-----|-------------|-------|
| Code coverage | ✅ Essential | ✅ Helpful | ⚠️ Limited | ❌ No | ❌ No |
| API contract validation | ⚠️ Mocked | ✅ Real | ✅ Best | ❌ No | ❌ No |
| Database logic | ✅ Essential | ✅ Real DB | ⚠️ Via API | ⚠️ Simulated | ❌ No |
| Network resilience | ❌ No | ⚠️ Simulated | ✅ Real | ❌ No | ✅ Best |
| UI/UX validation | ❌ No | ⚠️ Headless | ✅ Essential | ❌ No | ❌ No |
| Load handling | ❌ No | ❌ No | ⚠️ Limited | ✅ Essential | ⚠️ Useful |
| Security validation | ⚠️ Mocked | ✅ Real | ✅ Best | ❌ No | ⚠️ Limited |

### 9.2 Test Framework Selection

| Language | Unit | Integration | E2E | Performance |
|----------|------|-------------|-----|-------------|
| Python | pytest | pytest, testcontainers | Playwright, Selenium | k6, locust |
| Go | testing, testify | go-playwright | Playwright | k6 |
| Java | JUnit, TestNG | Testcontainers | Playwright, Selenium | JMeter, k6 |
| JavaScript | Jest, Mocha | Jest + supertest | Playwright, Cypress | k6, Artillery |
| Rust | tokio-test, proptest | testcontainers | Playwright | k6 |

---

## 10. Production Checklist

### 10.1 Test Strategy Checklist

- [ ] Test pyramid defined and documented
- [ ] Unit test coverage > 80%
- [ ] Integration tests for all critical paths
- [ ] E2E tests for all critical user journeys
- [ ] Performance tests in CI/CD pipeline
- [ ] Chaos experiments scheduled and monitored
- [ ] Test data management strategy in place
- [ ] Flaky test tracking and remediation process
- [ ] Test execution reports automated

### 10.2 Quality Gates Checklist

- [ ] All unit tests pass before merge
- [ ] All integration tests pass before merge
- [ ] No new flaky tests introduced
- [ ] Code coverage maintained above threshold
- [ ] Performance baselines defined and enforced
- [ ] Chaos experiments have steady state hypotheses
- [ ] Test infrastructure has DR plan

---

## 11. References

### Testing Fundamentals

- [Test Pyramid - Martin Fowler](https://martinfowler.com/articles/practical-test-pyramid.html)
- [xUnit Test Patterns](https://martinfowler.com/books/umlst.html)
- [Arrange-Act-Assert](https://automationpanda.com/2020/07/07/arrange-act-assert-a-pattern-for-writing-good-tests/)

### Unit Testing

- [pytest Documentation](https://docs.pytest.org/)
- [JUnit Documentation](https://junit.org/junit5/)
- [Google Test](https://google.github.io/googletest/)

### Integration Testing

- [Testcontainers](https://testcontainers.com/)
- [Contracts - Pact](https://docs.pact.io/)

### E2E Testing

- [Playwright](https://playwright.dev/)
- [Cypress](https://www.cypress.io/)
- [Selenium](https://www.selenium.dev/)

### Performance Testing

- [k6 Documentation](https://k6.io/docs/)
- [Gatling](https://gatling.io/)
- [JMeter](https://jmeter.apache.org/)

### Chaos Engineering

- [Chaos Mesh](https://chaos-mesh.org/)
- [Litmus](https://litmuschaos.io/)
- [Gremlin](https://www.gremlin.com/)