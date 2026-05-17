# MESSAGING.md - Message Queue Patterns

**Authority:** guidance (comprehensive async messaging patterns with exact configurations)
**Layer:** Architecture
**Binding:** No
**Scope:** Kafka, RabbitMQ, SQS,nats patterns with exact specifications for pre-inference context

---

## 1. Apache Kafka

### 1.1 Topic Configuration

```yaml
# Topic creation with retention
kafka-topics.sh --create \
  --bootstrap-server kafka:9092 \
  --topic user-events \
  --partitions 12 \
  --replication-factor 3 \
  --config retention.ms=604800000 \
  --config retention.bytes=10737418240 \
  --config min.insync.replicas=2 \
  --config max.message.bytes=1048576

# Topic configuration properties
retention.ms: 604800000          # 7 days
retention.bytes: 10737418240     # 10GB per partition
min.insync.replicas: 2           # ACKs required
max.message.bytes: 1048576       # 1MB max message
cleanup.policy: delete            # delete | compact
segment.ms: 604800000            # Segment roll time
segment.bytes: 1073741824        # 1GB segment size
flush.messages: 10000            # Flush after N messages
flush.ms: 60000                  # Or flush after N ms
```

### 1.2 Producer Configuration

```yaml
# Kafka producer with exactly-once semantics
bootstrap.servers: kafka-1:9092,kafka-2:9092,kafka-3:9092

# Reliability
acks: all                      # 0, 1, all (-1)
enable.idempotence: true        # Exactly-once
max.in.flight.requests.per.connection: 5
retries: 3
retry.backoff.ms: 100

# Performance
batch.size: 65536               # 64KB
linger.ms: 5                    # Wait up to 5ms for batching
buffer.memory: 33554432         # 32MB
compression.type: lz4           # lz4, snappy, gzip, zstd

# Timeouts
request.timeout.ms: 30000
delivery.timeout.ms: 120000
max.block.ms: 60000

# Idempotence
transactional.id: producer-1   # For exactly-once across topics
```

### 1.3 Consumer Configuration

```yaml
# Kafka consumer with balanced parallelism
bootstrap.servers: kafka-1:9092,kafka-2:9092,kafka-3:9092

# Consumer group
group.id: order-processor
group.instance.id: ${HOSTNAME}  # Static membership

# Reliability
enable.auto.commit: false       # Manual commit
auto.offset.reset: earliest     # earliest | latest
auto.commit.interval.ms: 5000

# Fetch settings
fetch.min.bytes: 1
fetch.max.wait.ms: 500
max.partition.fetch.bytes: 1048576

# Session timeout
session.timeout.ms: 45000
heartbeat.interval.ms: 3000
max.poll.interval.ms: 300000

# Concurrency
concurrency: 3                  # Threads per consumer
```

### 1.4 Spring Kafka Implementation

```java
// Producer configuration
@Configuration
public class KafkaProducerConfig {
    
    @Bean
    public ProducerFactory<String, OrderEvent> producerFactory() {
        Map<String, Object> config = new HashMap<>();
        config.put(ProducerConfig.BOOTSTRAP_SERVERS_CONFIG, "kafka:9092");
        config.put(ProducerConfig.KEY_SERIALIZER_CLASS_CONFIG, StringSerializer.class);
        config.put(ProducerConfig.VALUE_SERIALIZER_CLASS_CONFIG, JsonSerializer.class);
        
        // Exactly-once
        config.put(ProducerConfig.ENABLE_IDEMPOTENCE_CONFIG, true);
        config.put(ProducerConfig.ACKS_CONFIG, "all");
        config.put(ProducerConfig.RETRIES_CONFIG, 3);
        
        // Performance
        config.put(ProducerConfig.BATCH_SIZE_CONFIG, 65536);
        config.put(ProducerConfig.LINGER_MS_CONFIG, 5);
        config.put(ProducerConfig.COMPRESSION_TYPE_CONFIG, "lz4");
        
        return new DefaultKafkaProducerFactory<>(config);
    }
    
    @Bean
    public KafkaTemplate<String, OrderEvent> kafkaTemplate() {
        return new KafkaTemplate<>(producerFactory());
    }
}

@Service
public class OrderEventProducer {
    
    private final KafkaTemplate<String, OrderEvent> template;
    
    public void sendOrderCreated(Order order) {
        OrderEvent event = new OrderEvent("ORDER_CREATED", order);
        
        // Send with routing key (partition by user for ordering)
        ListenableFuture<SendResult<String, OrderEvent>> future = 
            template.send("order-events", order.getUserId(), event);
        
        future.addCallback(
            result -> {
                // Record metadata
                String topic = result.getRecordMetadata().topic();
                int partition = result.getRecordMetadata().partition();
                long offset = result.getRecordMetadata().offset();
                log.info("Sent {} to {}-{}:{}", event.getType(), topic, partition, offset);
            },
            ex -> log.error("Failed to send order event", ex)
        );
    }
    
    // Transactional send across topics
    @Transactional("kafkaTransactionManager")
    public void sendOrderWithInventory(Order order, List<InventoryReservation> reservations) {
        // These will be committed atomically
        template.send("order-events", order.getUserId(), new OrderEvent("ORDER_CREATED", order));
        for (InventoryReservation r : reservations) {
            template.send("inventory-events", r.getProductId(), 
                new InventoryEvent("RESERVED", r));
        }
    }
}

// Consumer configuration
@Configuration
public class KafkaConsumerConfig {
    
    @Bean
    public ConsumerFactory<String, OrderEvent> consumerFactory() {
        Map<String, Object> config = new HashMap<>();
        config.put(ConsumerConfig.BOOTSTRAP_SERVERS_CONFIG, "kafka:9092");
        config.put(ConsumerConfig.GROUP_ID_CONFIG, "order-processor");
        config.put(ConsumerConfig.KEY_DESERIALIZER_CLASS_CONFIG, StringDeserializer.class);
        config.put(ConsumerConfig.VALUE_DESERIALIZER_CLASS_CONFIG, JsonDeserializer.class);
        config.put(ConsumerConfig.ENABLE_AUTO_COMMIT_CONFIG, false);
        config.put(ConsumerConfig.AUTO_OFFSET_RESET_CONFIG, "earliest");
        return new DefaultKafkaConsumerFactory<>(config);
    }
    
    @Bean
    public ConcurrentKafkaListenerContainerFactory<String, OrderEvent> 
            kafkaListenerContainerFactory() {
        ConcurrentKafkaListenerContainerFactory<String, OrderEvent> factory =
            new ConcurrentKafkaListenerContainerFactory<>();
        factory.setConsumerFactory(consumerFactory());
        factory.setConcurrency(3);
        factory.getContainerProperties().setAckMode(
            ContainerProperties.AckMode.MANUAL_IMMEDIATE);
        return factory;
    }
}

@Service
public class OrderEventConsumer {
    
    @KafkaListener(
        topics = "order-events",
        groupId = "order-processor",
        containerFactory = "kafkaListenerContainerFactory"
    )
    public void handleOrderEvent(
            @Payload OrderEvent event,
            @Header(KafkaHeaders.RECEIVED_PARTITION) int partition,
            @Header(KafkaHeaders.OFFSET) long offset,
            Acknowledgment ack) {
        
        try {
            switch (event.getType()) {
                case "ORDER_CREATED":
                    processOrderCreated(event.getOrder());
                    break;
                case "ORDER_CANCELLED":
                    processOrderCancelled(event.getOrder());
                    break;
                default:
                    log.warn("Unknown event type: {}", event.getType());
            }
            
            // Acknowledge after successful processing
            ack.acknowledge();
            
        } catch (Exception e) {
            log.error("Failed to process event at {}-{}", partition, offset, e);
            // Don't acknowledge - will be redelivered
            throw e;
        }
    }
}
```

### 1.5 Schema Registry

```yaml
# Schema configuration (Confluent)
schema.registry.url: http://schema-registry:8081
auto.register.schemas: false
subject.name.strategy: io.confluent.kafka.schemaregistry.storage.BeautifulSubjectNameStrategy

# Compatibility settings (backward, forward, full, none)
avro.compatibility.level: backward
```

```java
// Avro schema and serializer
@GenerateAvroSchema
public class OrderEvent {
    @AvroName("event_type")
    private String eventType;
    
    @AvroName("order_id")
    private String orderId;
    
    @AvroName("user_id")
    private String userId;
    
    @AvroName("total")
    private BigDecimal total;
    
    @AvroName("items")
    private List<OrderItem> items;
    
    @AvroName("created_at")
    private long createdAt;
}
```

---

## 2. RabbitMQ

### 2.1 Exchange and Queue Configuration

```yaml
# RabbitMQ definitions (imported via mgmt API or config)
{
  "rabbit_version": "3.12",
  "rabbitmq_version": "3.12.0",
  "users": [
    {
      "name": "producer",
      "password_hash": "...",
      "tags": ["producer"]
    },
    {
      "name": "consumer",
      "password_hash": "...",
      "tags": ["consumer"]
    }
  ],
  "vhosts": [
    {
      "name": "/"
    }
  ],
  "permissions": [
    {
      "user": "producer",
      "vhost": "/",
      "configure": "",
      "write": "order.*",
      "read": ""
    },
    {
      "user": "consumer",
      "vhost": "/",
      "configure": "",
      "write": "",
      "read": "order.*"
    }
  ],
  "topic_permissions": [],
  "parameters": [],
  "global_parameters": [
    {
      "name": "cluster_name",
      "value": "production-cluster"
    }
  ],
  "policies": [
    {
      "vhost": "/",
      "name": "ha-all",
      "pattern": "^(order|payment|shipment).*",
      "apply-to": "queues",
      "definition": {
        "ha-mode": "all",
        "ha-sync-mode": "automatic",
        "ha-promote-on-shutdown": "when-synced"
      },
      "priority": 10
    }
  ],
  "queues": [
    {
      "name": "order.created",
      "vhost": "/",
      "durable": true,
      "auto_delete": false,
      "arguments": {
        "x-message-ttl": 86400000,
        "x-dead-letter-exchange": "order.dlx",
        "x-dead-letter-routing-key": "order.created.dead"
      }
    },
    {
      "name": "order.created.dlq",
      "vhost": "/",
      "durable": true,
      "auto_delete": false,
      "arguments": {
        "x-message-ttl": 604800000
      }
    }
  ],
  "exchanges": [
    {
      "name": "order.events",
      "vhost": "/",
      "type": "topic",
      "durable": true,
      "auto_delete": false,
      "internal": false,
      "arguments": {}
    },
    {
      "name": "order.dlx",
      "vhost": "/",
      "type": "fanout",
      "durable": true,
      "auto_delete": false,
      "internal": false,
      "arguments": {}
    }
  ],
  "bindings": [
    {
      "source": "order.events",
      "vhost": "/",
      "destination": "order.created",
      "destination_type": "queue",
      "routing_key": "order.created",
      "arguments": {}
    },
    {
      "source": "order.events",
      "vhost": "/",
      "destination": "order.updated",
      "destination_type": "queue",
      "routing_key": "order.updated",
      "arguments": {}
    },
    {
      "source": "order.events",
      "vhost": "/",
      "destination": "order.*",
      "destination_type": "queue",
      "routing_key": "order.*",
      "arguments": {}
    },
    {
      "source": "order.dlx",
      "vhost": "/",
      "destination": "order.created.dlq",
      "destination_type": "queue",
      "routing_key": "",
      "arguments": {}
    }
  ]
}
```

### 2.2 Spring AMQP Implementation

```java
@Configuration
public class RabbitMQConfig {
    
    @Bean
    public ConnectionFactory connectionFactory() {
        CachingConnectionFactory factory = new CachingConnectionFactory("rabbitmq:5672");
        factory.setUsername("consumer");
        factory.setPassword("...");
        factory.setPublisherConfirmType(CachingConnectionFactory.ConfirmType.CORRELATED);
        factory.setPublisherReturns(true);
        return factory;
    }
    
    @Bean
    public RabbitTemplate rabbitTemplate(ConnectionFactory factory) {
        RabbitTemplate template = new RabbitTemplate(factory);
        template.setMandatory(true);
        template.setConfirmCallback((data, ack, cause) -> {
            if (!ack) {
                log.error("Message not acknowledged: {}", cause);
            }
        });
        template.setReturnsCallback(returned -> {
            log.error("Message returned: {} - {}", 
                returned.getMessage(), returned.getReplyText());
        });
        return template;
    }
    
    // DLQ configuration
    @Bean
    public DirectExchange deadLetterExchange() {
        return new DirectExchange("order.dlx");
    }
    
    @Bean
    public Queue deadLetterQueue() {
        return QueueBuilder
            .durable("order.created.dlq")
            .ttl(604800000) // 7 days
            .build();
    }
    
    @Bean
    public Binding deadLetterBinding() {
        return BindingBuilder
            .bind(deadLetterQueue())
            .to(deadLetterExchange())
            .with("order.created.dead");
    }
}

@Service
public class OrderEventPublisher {
    
    private final RabbitTemplate template;
    
    public void sendOrderCreated(Order order) {
        String routingKey = "order.created";
        
        MessageProperties props = new MessageProperties();
        props.setContentType("application/json");
        props.setDeliveryMode(MessageDeliveryMode.PERSISTENT);
        props.setMessageId(order.getId());
        props.setTimestamp(new Date());
        props.setHeader("user_id", order.getUserId());
        
        // Can add retry headers
        props.setHeader("x-retry-count", 0);
        
        Message message = new Message(
            new ObjectMapper().writeValueAsBytes(order),
            props
        );
        
        template.send("order.events", routingKey, message);
    }
    
    // With delay (requires delayed message plugin)
    public void sendDelayedMessage(Order order, int delayMs) {
        template.send("order.events", "order.delayed", message, msg -> {
            msg.getMessageProperties().setDelay(delayMs);
            return msg;
        });
    }
}

@Service
@RabbitListener(queues = "order.created")
public class OrderEventConsumer {
    
    @RabbitHandler
    public void handleOrderCreated(
            @Payload Order order,
            @Headers Map<String, Object> headers,
            Channel channel,
            @Header(AmqpHeaders.DELIVERY_TAG) long tag) {
        
        try {
            // Get retry count
            Integer retryCount = (Integer) headers.get("x-retry-count");
            
            processOrder(order);
            
            // Acknowledge
            channel.basicAck(tag, false);
            
        } catch (Exception e) {
            log.error("Failed to process order: {}", order.getId(), e);
            
            // Reject and requeue (if retries not exhausted)
            Integer retryCount = (Integer) headers.get("x-retry-count");
            if (retryCount != null && retryCount < 3) {
                // Requeue for retry
                channel.basicNack(tag, false, true);
            } else {
                // Send to DLQ
                channel.basicNack(tag, false, false);
            }
        }
    }
    
    // Concurrent consumers
    @RabbitListener(
        queues = "order.created",
        concurrency = "3-10",
        prefetch = "10"
    )
    public void handleWithConcurrency(Order order, Channel channel) {
        // Auto-acknowledged with manual ack in handler
        processOrder(order);
    }
}
```

---

## 3. AWS SQS

### 3.1 Queue Configuration

```yaml
# SQS queue (CloudFormation)
AWSTemplateFormatVersion: "2010-09-09"
Resources:
  OrderQueue:
    Type: AWS::SQS::Queue
    Properties:
      QueueName: order-processing.fifo
      FifoQueue: true
      ContentBasedDeduplication: true
      
      VisibilityTimeout: 300
      MessageRetentionPeriod: 1209600  # 14 days
      ReceiveMessageWaitTimeSeconds: 20  # Long polling
      
      RedrivePolicy:
        deadLetterTargetArn: !GetAtt OrderDeadLetterQueue.Arn
        maxReceiveCount: 5
      
      Tags:
        - Key: Environment
          Value: production
        - Key: Team
          Value: Platform

  OrderDeadLetterQueue:
    Type: AWS::SQS::Queue
    Properties:
      QueueName: order-processing.dlq.fifo
      FifoQueue: true
      MessageRetentionPeriod: 1209600
```

### 3.2 AWS SDK Implementation

```java
// SQS producer (AWS SDK v2)
@Service
public class SqsOrderPublisher {
    
    private final SqsClient sqsClient;
    private final String queueUrl;
    
    public SqsOrderPublisher(SqsClient sqsClient, @Value("${order.queue.url}") String queueUrl) {
        this.sqsClient = sqsClient;
        this.queueUrl = queueUrl;
    }
    
    public void sendOrderCreated(Order order) {
        SendMessageRequest request = SendMessageRequest.builder()
            .queueUrl(queueUrl)
            .messageDeduplicationId(order.getId())
            .messageGroupId("order")
            .messageBody(toJson(order))
            .messageAttributes(
                MessageAttributeValue.builder()
                    .stringValue(order.getUserId())
                    .dataType("String")
                    .build()
            )
            .build();
        
        SendMessageResponse response = sqsClient.sendMessage(request);
        log.info("Sent message {} to {}", response.messageId(), queueUrl);
    }
    
    // Batch send (up to 10 messages)
    public void sendBatch(List<Order> orders) {
        List<SendMessageBatchRequestEntry> entries = orders.stream()
            .map(order -> SendMessageBatchRequestEntry.builder()
                .id(order.getId())
                .messageDeduplicationId(order.getId())
                .messageGroupId("order")
                .messageBody(toJson(order))
                .build())
            .collect(Collectors.toList());
        
        SendMessageBatchRequest batchRequest = SendMessageBatchRequest.builder()
            .queueUrl(queueUrl)
            .entries(entries)
            .build();
        
        SendMessageBatchResponse response = sqsClient.sendMessageBatch(batchRequest);
        
        if (!response.failed().isEmpty()) {
            log.error("Failed messages: {}", response.failed());
        }
    }
}

// SQS consumer
@Service
public class SqsOrderConsumer {
    
    private final SqsClient sqsClient;
    private final String queueUrl;
    
    @Scheduled(fixedDelayString = "${sqs.poll.interval:1000}")
    public void pollQueue() {
        ReceiveMessageRequest receiveRequest = ReceiveMessageRequest.builder()
            .queueUrl(queueUrl)
            .maxNumberOfMessages(10)
            .waitTimeSeconds(20)  // Long polling
            .visibilityTimeout(300)
            .messageAttributeNames("All")
            .build();
        
        ReceiveMessageResponse response = sqsClient.receiveMessage(receiveRequest);
        
        for (Message message : response.messages()) {
            try {
                Order order = fromJson(message.body());
                processOrder(order);
                
                // Delete message after successful processing
                sqsClient.deleteMessage(DeleteMessageRequest.builder()
                    .queueUrl(queueUrl)
                    .receiptHandle(message.receiptHandle())
                    .build());
                    
            } catch (Exception e) {
                log.error("Failed to process message: {}", message.messageId(), e);
                // Message will become visible after visibility timeout
            }
        }
    }
}
```

---

## 4. Design Patterns

### 4.1 Saga Pattern (Choreography)

```java
// OrderCreatedEvent triggers downstream services
// Each service publishes completion events

// Order Service
@Service
public class OrderService {
    
    @Autowired
    private KafkaTemplate<String, Object> template;
    
    public void createOrder(Order order) {
        // Create order in PENDING state
        order.setStatus(OrderStatus.PENDING);
        orderRepository.save(order);
        
        // Emit event for other services to handle
        OrderCreatedEvent event = new OrderCreatedEvent(order);
        template.send("order.events", order.getUserId(), event);
    }
    
    @KafkaListener(topics = "payment.events")
    public void handlePaymentCompleted(PaymentCompletedEvent event) {
        if (event.isSuccess()) {
            orderService.confirmOrder(event.getOrderId());
            orderService.emitOrderConfirmed(event);
        } else {
            orderService.cancelOrder(event.getOrderId(), event.getReason());
        }
    }
    
    @KafkaListener(topics = "inventory.events")
    public void handleInventoryReserved(InventoryReservedEvent event) {
        // Inventory reserved - could trigger shipment
    }
}

// Compensating transactions
public class OrderSaga {
    
    public void cancelOrder(String orderId, String reason) {
        Order order = orderRepository.findById(orderId);
        
        // Compensating transactions (reverse what was done)
        
        // 1. Cancel payment
        paymentService.cancel(orderId);
        
        // 2. Release inventory
        inventoryService.release(orderId);
        
        // 3. Update order status
        order.setStatus(OrderStatus.CANCELLED);
        order.setCancellationReason(reason);
        orderRepository.save(order);
    }
}
```

### 4.2 Outbox Pattern

```sql
-- Outbox table
CREATE TABLE outbox (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    aggregate_type VARCHAR(100) NOT NULL,
    aggregate_id VARCHAR(100) NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    payload JSONB NOT NULL,
    created_at TIMESTAMP DEFAULT NOW(),
    published_at TIMESTAMP,
    INDEX idx_outbox_unpublished (published_at) WHERE published_at IS NULL
);

-- Transactional outbox write
BEGIN;
    -- Update order
    UPDATE orders SET status = 'CONFIRMED' WHERE id = '123';
    
    -- Write to outbox (same transaction)
    INSERT INTO outbox (aggregate_type, aggregate_id, event_type, payload)
    VALUES ('order', '123', 'ORDER_CONFIRMED', '{"orderId": "123"}');
COMMIT;

-- Outbox processor (runs as separate process)
SELECT * FROM outbox 
WHERE published_at IS NULL 
ORDER BY created_at 
LIMIT 100;

-- Mark as published
UPDATE outbox SET published_at = NOW() WHERE id = '...';
```

### 4.3 Circuit Breaker

```java
// Resilience4j circuit breaker
@CircuitBreaker(
    name = "messaging",
    fallbackMethod = "fallback"
)
public void sendMessage(OrderEvent event) {
    kafkaTemplate.send("order.events", event.getOrderId(), event);
}

public void fallback(OrderEvent event, Exception e) {
    // Store in local buffer for later retry
    messageBuffer.add(event);
    log.warn("Circuit open, message buffered: {}", event);
}
```

---

## 5. Decision Matrix

| Criteria | Kafka | RabbitMQ | SQS |
|----------|-------|----------|-----|
| Ordering | Per partition | Per queue | Per message group |
| Throughput | Very high | High | Medium |
| Latency | Low | Very low | Low |
| At-least-once | Yes | Yes | Yes |
| Exactly-once | Yes (with transactions) | No | No |
| Delayed messages | No (requires plugin) | Yes | No (use delay queue) |
| Priority queues | No | Yes | No |
| Multi-consumer | Yes (consumer groups) | Yes (shared queue) | Yes |
| Message retention | Configurable | Configurable | Up to 14 days |
| Best for | Event streaming, audit logs | Task queues, RPC | Fire-and-forget, async tasks |

---

## Links

### Core Router
- `core/DECAPOD.md` - **Router and navigation charter (START HERE)**
- `core/ENGINEERING_EXCELLENCE.md` - Engineering standards

### Architecture (This Section)
- `architecture/KUBERNETES.md` - Message queue operators
- `architecture/DATABASE.md` - Event store patterns
- `architecture/API_DESIGN.md` - Event-driven API design
- `architecture/CACHING.md` - Cache invalidation via events

### Authority (Constitution Layer)
- `specs/INTENT.md` - **Methodology contract (READ FIRST)**
- `specs/SYSTEM.md` - System definition and authority doctrine
- `specs/SECURITY.md` - Security doctrine

### Interface Contracts
- `interfaces/CLAIMS.md` - Promises ledger
- `interfaces/CONTROL_PLANE.md` - Agent sequencing patterns
- `interfaces/KNOWLEDGE_SCHEMA.md` - Knowledge event schemas

### Methodology
- `methodology/ARCHITECTURE.md` - Architecture decision methodology
- `methodology/CI_CD.md` - Event-driven CI/CD

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2024-01-16 | Initial comprehensive messaging reference |