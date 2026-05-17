# SCALING.md - Scaling Architecture Standards

**Authority:** guidance (comprehensive topic with exact specifications)
**Layer:** Architecture
**Binding:** No
**Scope:** Comprehensive topic coverage for pre-inference context

---

## 1. Horizontal Pod Autoscaling (HPA)

### 1.1 HPA Manifest Specifications

```yaml
# Standard HPA for stateless service
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: api-autoscaler
  namespace: production
  labels:
    app: api
    tier: backend
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: api-deployment
  minReplicas: 3
  maxReplicas: 100
  behavior:
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
        - type: Percent
          value: 10
          periodSeconds: 60
        - type: Pods
          value: 2
          periodSeconds: 60
      selectPolicy: Min
    scaleUp:
      stabilizationWindowSeconds: 0
      policies:
        - type: Percent
          value: 100
          periodSeconds: 15
        - type: Pods
          value: 4
          periodSeconds: 15
      selectPolicy: Max
  metrics:
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 70
    - type: Resource
      resource:
        name: memory
        target:
          type: Utilization
          averageUtilization: 80
    - type: Pods
      pods:
        metric:
          name: http_requests_per_second
        target:
          type: AverageValue
          averageValue: "1000"
```

```yaml
# HPA with custom metrics
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: api-custom-metrics-hpa
  namespace: production
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: api-deployment
  minReplicas: 3
  maxReplicas: 50
  metrics:
    # CPU metric
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 60
    
    # Memory metric
    - type: Resource
      resource:
        name: memory
        target:
          type: Utilization
          averageUtilization: 70
    
    # Custom Prometheus metric
    - type: Pods
      pods:
        metric:
          name: request_queue_depth
          selector:
            matchLabels:
              queue: "important"
        target:
          type: AverageValue
          averageValue: "100"
    
    # External metric (e.g., queue depth in Redis)
    - type: External
      external:
        metric:
          name: redis_stream_length
          selector:
            matchLabels:
              stream_name: order_processing
        target:
          type: AverageValue
          averageValue: "1000"
```

```yaml
# HPA for specific deployment
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: worker-autoscaler
  namespace: production
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: worker-deployment
  minReplicas: 2
  maxReplicas: 20
  behavior:
    scaleDown:
      stabilizationWindowSeconds: 600
      policies:
        - type: Pods
          value: 1
          periodSeconds: 300
    scaleUp:
      stabilizationWindowSeconds: 30
      policies:
        - type: Pods
          value: 2
          periodSeconds: 60
  metrics:
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 50
    - type: Pods
      pods:
        metric:
          name: rabbitmq_queue_messages
        target:
          type: AverageValue
          averageValue: "50"
```

### 1.2 Vertical Pod Autoscaler (VPA)

```yaml
# VPA for resource optimization
apiVersion: autoscaling.k8s.io/v1
kind: VerticalPodAutoscaler
metadata:
  name: api-vpa
  namespace: production
spec:
  targetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: api-deployment
  updatePolicy:
    updateMode: "Auto"
    minRecheckDuration: 10m
    maxRecheckDuration: 1h
  resourcePolicy:
    containerPolicies:
      - containerName: '*'
        minAllowed:
          cpu: 100m
          memory: 128Mi
        maxAllowed:
          cpu: 4
          memory: 8Gi
        controlledResources: ["cpu", "memory"]
        controlledValues: RequestsAndLimits
```

```yaml
# VPA in Off mode (recommendation only)
apiVersion: autoscaling.k8s.io/v1
kind: VerticalPodAutoscaler
metadata:
  name: worker-vpa-recommendation
  namespace: production
spec:
  targetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: worker-deployment
  updatePolicy:
    updateMode: "Off"
  resourcePolicy:
    containerPolicies:
      - containerName: '*'
        minAllowed:
          cpu: 50m
          memory: 64Mi
        maxAllowed:
          cpu: 8
          memory: 32Gi
```

### 1.3 HPA with Multiple Metric Types

```yaml
# Complex HPA with multiple scaling signals
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: orderservice-comprehensive-hpa
  namespace: production
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: orderservice-deployment
  minReplicas: 5
  maxReplicas: 100
  
  metrics:
    # 1. CPU utilization as primary metric
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 65
    
    # 2. Memory utilization as secondary
    - type: Resource
      resource:
        name: memory
        target:
          type: Utilization
          averageUtilization: 75
    
    # 3. Custom application metric from Prometheus
    - type: Pods
      pods:
        metric:
          name: payment_request_duration_seconds_p99
          selector:
            matchLabels:
              app: orderservice
        target:
          type: AverageValue
          averageValue: "2"
    
    # 4. Database connection pool metric
    - type: Pods
      pods:
        metric:
          name: db_connection_pool_in_use
        target:
          type: AverageValue
          averageValue: "80"
    
    # 5. External queue depth
    - type: External
      external:
        metric:
          name: rabbitmq_messages_ready
          selector:
            matchLabels:
              queue: order_processing
        target:
          type: AverageValue
          averageValue: "500"
  
  # Scaling behavior configuration
  behavior:
    # Scale down slowly to prevent flapping
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
        # Can scale down by max 10% every minute
        - type: Percent
          value: 10
          periodSeconds: 60
        # Or max 2 pods every minute
        - type: Pods
          value: 2
          periodSeconds: 60
      selectPolicy: Min  # Take the smaller of the two policies
    
    # Scale up quickly to handle traffic spikes
    scaleUp:
      stabilizationWindowSeconds: 15
      policies:
        # Can double pods (100%) every 15 seconds
        - type: Percent
          value: 100
          periodSeconds: 15
        # Or add 4 pods every 15 seconds
        - type: Pods
          value: 4
          periodSeconds: 15
      selectPolicy: Max  # Take the larger of the two policies
```

## 2. Database Sharding

### 2.1 Sharding Architecture Patterns

```typescript
// sharding/shard-manager.ts - Sharding implementation

import { Pool } from 'pg';
import { crc32 } from './hash';

interface ShardConfig {
  id: number;
  host: string;
  port: number;
  database: string;
  user: string;
  password: string;
}

interface ShardMetadata {
  userIdRange: { min: number; max: number };
  shardId: number;
}

class ShardManager {
  private pools: Map<number, Pool> = new Map();
  private shardConfigs: ShardConfig[];
  
  constructor(shardConfigs: ShardConfig[]) {
    this.shardConfigs = shardConfigs;
    this.initializePools();
  }
  
  private async initializePools(): Promise<void> {
    for (const config of this.shardConfigs) {
      const pool = new Pool({
        host: config.host,
        port: config.port,
        database: config.database,
        user: config.user,
        password: config.password,
        max: 20,
        idleTimeoutMillis: 30000,
        connectionTimeoutMillis: 2000,
      });
      
      await pool.query('SELECT 1');
      this.pools.set(config.id, pool);
    }
  }
  
  // Consistent hashing to determine shard
  private getShardForKey(key: string, totalShards: number): number {
    const hash = crc32(key);
    return hash % totalShards;
  }
  
  // Get shard for user
  getShardForUserId(userId: string): number {
    return this.getShardForKey(userId, this.shardConfigs.length);
  }
  
  // Get pool for user
  async getPoolForUser(userId: string): Promise<Pool> {
    const shardId = this.getShardForUserId(userId);
    const pool = this.pools.get(shardId);
    if (!pool) {
      throw new Error(`No pool for shard ${shardId}`);
    }
    return pool;
  }
  
  // Execute query on specific shard
  async query<T>(
    userId: string,
    query: string,
    params?: unknown[]
  ): Promise<T[]> {
    const pool = await this.getPoolForUser(userId);
    const result = await pool.query(query, params);
    return result.rows as T[];
  }
  
  // Execute query across all shards
  async queryAllShards<T>(
    query: string,
    params?: unknown[]
  ): Promise<T[]> {
    const promises: Promise<T[]>[] = [];
    
    for (const [shardId, pool] of this.pools) {
      promises.push(
        pool.query(query, params).then(result => result.rows as T[])
      );
    }
    
    const results = await Promise.all(promises);
    return results.flat();
  }
  
  // Aggregation across shards
  async aggregateAllShards<T>(
    aggregator: (pool: Pool) => Promise<T>,
    reducer: (results: T[]) => T
  ): Promise<T> {
    const promises: Promise<T>[] = [];
    
    for (const [shardId, pool] of this.pools) {
      promises.push(aggregator(pool));
    }
    
    const results = await Promise.all(promises);
    return reducer(results);
  }
  
  // Rebalance shards (for adding/removing shards)
  async rebalance(
    newShards: ShardConfig[],
    migrationBatchSize: number = 1000
  ): Promise<void> {
    console.log('Starting shard rebalance...');
    
    for (const shardId of this.pools.keys()) {
      const pool = this.pools.get(shardId)!;
      await pool.end();
    }
    
    const newPools = new Map<number, Pool>();
    for (const config of newShards) {
      const pool = new Pool({
        host: config.host,
        port: config.port,
        database: config.database,
        user: config.user,
        password: config.password,
        max: 20,
      });
      await pool.query('SELECT 1');
      newPools.set(config.id, pool);
    }
    
    this.pools = newPools;
    this.shardConfigs = newShards;
    
    console.log('Shard rebalance completed');
  }
}

// Consistent hash for even distribution
class ConsistentHashRing<T> {
  private ring: Map<number, T> = new Map();
  private sortedKeys: number[] = [];
  private virtualNodes: number = 150;
  
  addNode(node: T, key: string): void {
    for (let i = 0; i < this.virtualNodes; i++) {
      const hash = this.hash(`${key}:${i}`);
      this.ring.set(hash, node);
    }
    this.sortedKeys = Array.from(this.ring.keys()).sort((a, b) => a - b);
  }
  
  removeNode(key: string): void {
    for (let i = 0; i < this.virtualNodes; i++) {
      const hash = this.hash(`${key}:${i}`);
      this.ring.delete(hash);
    }
    this.sortedKeys = Array.from(this.ring.keys()).sort((a, b) => a - b);
  }
  
  getNode(key: string): T | undefined {
    if (this.ring.size === 0) return undefined;
    
    const hash = this.hash(key);
    let idx = this.binarySearch(this.sortedKeys, hash);
    
    if (idx === this.sortedKeys.length) {
      idx = 0;
    }
    
    return this.ring.get(this.sortedKeys[idx]);
  }
  
  private hash(key: string): number {
    return crc32(key);
  }
  
  private binarySearch(arr: number[], target: number): number {
    let left = 0;
    let right = arr.length;
    
    while (left < right) {
      const mid = Math.floor((left + right) / 2);
      if (arr[mid] < target) {
        left = mid + 1;
      } else {
        right = mid;
      }
    }
    
    return left;
  }
}
```

### 2.2 Shard Router Implementation

```typescript
// sharding/shard-router.ts - Request routing

interface ShardRoute {
  shardId: number;
  connectionString: string;
}

interface UserShardMapping {
  userId: string;
  shardId: number;
  createdAt: Date;
}

class ShardRouter {
  private shardMap: Map<string, ShardRoute> = new Map();
  private userToShardCache: Cache<string, number>;
  
  constructor(
    private config: ShardConfig[],
    private connectionStringBuilder: (config: ShardConfig) => string,
    private metadataStore: MetadataStore
  ) {
    this.userToShardCache = new Cache({
      maxSize: 10000,
      ttl: 60 * 60 * 1000, // 1 hour
    });
    
    this.initializeShards();
  }
  
  private async initializeShards(): Promise<void> {
    for (const config of this.config) {
      const connectionString = this.connectionStringBuilder(config);
      this.shardMap.set(config.id, {
        shardId: config.id,
        connectionString,
      });
    }
  }
  
  // Get shard for user
  async getShardForUser(userId: string): Promise<ShardRoute> {
    // Check cache first
    const cachedShardId = this.userToShardCache.get(userId);
    if (cachedShardId !== undefined) {
      const route = this.shardMap.get(cachedShardId);
      if (route) return route;
    }
    
    // Check metadata store
    const mapping = await this.metadataStore.getUserShardMapping(userId);
    if (mapping) {
      this.userToShardCache.set(userId, mapping.shardId);
      return this.shardMap.get(mapping.shardId)!;
    }
    
    // Assign new user to shard with least users
    const shardId = await this.assignShardForUser(userId);
    const route = this.shardMap.get(shardId);
    if (!route) throw new Error(`Shard ${shardId} not found`);
    
    return route;
  }
  
  // Assign user to shard
  private async assignShardForUser(userId: string): Promise<number> {
    // Find shard with least users
    const shardCounts = await Promise.all(
      this.config.map(async config => {
        const count = await this.metadataStore.getUserCountForShard(config.id);
        return { shardId: config.id, count };
      })
    );
    
    const { shardId } = shardCounts.sort((a, b) => a.count - b.count)[0];
    
    // Save mapping
    await this.metadataStore.saveUserShardMapping({
      userId,
      shardId,
      createdAt: new Date(),
    });
    
    this.userToShardCache.set(userId, shardId);
    
    return shardId;
  }
  
  // Route database operation
  async routeOperation<T>(
    userId: string,
    operation: (connection: Pool) => Promise<T>
  ): Promise<T> {
    const route = await this.getShardForUser(userId);
    const pool = new Pool({ connectionString: route.connectionString });
    
    try {
      return await operation(pool);
    } finally {
      await pool.end();
    }
  }
  
  // Cross-shard query
  async routeCrossShardOperation<T>(
    userIds: string[],
    operation: (connections: Map<number, Pool>, userId: string) => Promise<T>
  ): Promise<Map<string, T>> {
    const connections = new Map<number, Pool>();
    const userToShard = new Map<string, number>();
    
    try {
      // Group userIds by shard
      for (const userId of userIds) {
        const route = await this.getShardForUser(userId);
        userToShard.set(userId, route.shardId);
        
        if (!connections.has(route.shardId)) {
          const pool = new Pool({
            connectionString: route.connectionString,
          });
          connections.set(route.shardId, pool);
        }
      }
      
      // Execute operations per shard
      const results = new Map<string, T>();
      
      for (const [userId, shardId] of userToShard) {
        const pool = connections.get(shardId)!;
        const result = await operation(connections, userId);
        results.set(userId, result);
      }
      
      return results;
    } finally {
      for (const pool of connections.values()) {
        await pool.end();
      }
    }
  }
  
  // Shard health check
  async healthCheck(): Promise<Map<number, boolean>> {
    const results = new Map<number, boolean>();
    
    const checks = this.config.map(async config => {
      const route = this.shardMap.get(config.id)!;
      const pool = new Pool({ connectionString: route.connectionString });
      
      try {
        await pool.query('SELECT 1');
        results.set(config.id, true);
      } catch {
        results.set(config.id, false);
      } finally {
        await pool.end();
      }
    });
    
    await Promise.all(checks);
    return results;
  }
  
  // Shutdown all connections
  async shutdown(): Promise<void> {
    this.userToShardCache.clear();
    // Close any open connections
  }
}
```

## 3. Read Replicas

### 3.1 Read Replica Configuration

```yaml
# Kubernetes service for read replica load balancing
apiVersion: v1
kind: Service
metadata:
  name: postgres-replicas
  namespace: production
  labels:
    app: postgres
    tier: database
    read: "true"
spec:
  type: ClusterIP
  selector:
    app: postgres
    role: replica
  ports:
    - name: postgres
      port: 5432
      targetPort: 5432
  # Session affinity for transactions
  sessionAffinity: ClientIP
  sessionAffinityConfig:
    clientIP:
      timeoutSeconds: 10800

---
# Endpoint for read replica discovery
apiVersion: v1
kind: Endpoints
metadata:
  name: postgres-replicas
  namespace: production
subsets:
  - addresses:
      - ip: 10.0.1.5
        targetRef:
          kind: Pod
          name: postgres-replica-1
          namespace: production
      - ip: 10.0.1.6
        targetRef:
          kind: Pod
          name: postgres-replica-2
          namespace: production
      - ip: 10.0.1.7
        targetRef:
          kind: Pod
          name: postgres-replica-3
          namespace: production
    ports:
      - port: 5432
        protocol: TCP
```

### 3.2 Read/Write Splitting Router

```typescript
// replication/read-write-splitter.ts

interface DatabaseConfig {
  host: string;
  port: number;
  primary: boolean;
}

class ReadWriteSplitter {
  private primaryPool: Pool;
  private replicaPools: Pool[];
  private replicaIndex: number = 0;
  
  constructor(config: {
    primary: DatabaseConfig;
    replicas: DatabaseConfig[];
  }) {
    // Create primary connection pool
    this.primaryPool = new Pool({
      host: config.primary.host,
      port: config.primary.port,
      database: 'mydb',
      max: 20,
      statement_timeout: 30000,
    });
    
    // Create replica connection pools
    this.replicaPools = config.replicas.map(replica =>
      new Pool({
        host: replica.host,
        port: replica.port,
        database: 'mydb',
        max: 10,
        statement_timeout: 30000,
      })
    );
  }
  
  // Determine if query is read-only
  private isReadOnlyQuery(sql: string): boolean {
    const normalizedSql = sql.trim().toUpperCase();
    const readKeywords = ['SELECT', 'SHOW', 'DESCRIBE', 'EXPLAIN', 'WITH'];
    
    for (const keyword of readKeywords) {
      if (normalizedSql.startsWith(keyword)) {
        return true;
      }
    }
    
    return false;
  }
  
  // Get next replica in round-robin
  private getNextReplica(): Pool {
    const pool = this.replicaPools[this.replicaIndex];
    this.replicaIndex = (this.replicaIndex + 1) % this.replicaPools.length;
    return pool;
  }
  
  // Route query to appropriate database
  async query<T>(
    sql: string,
    params?: unknown[],
    options?: { readOnly?: boolean }
  ): Promise<T[]> {
    const isReadOnly = options?.readOnly ?? this.isReadOnlyQuery(sql);
    
    let pool: Pool;
    if (isReadOnly && this.replicaPools.length > 0) {
      pool = this.getNextReplica();
    } else {
      pool = this.primaryPool;
    }
    
    const start = Date.now();
    try {
      const result = await pool.query(sql, params);
      return result.rows as T[];
    } finally {
      const duration = Date.now() - start;
      if (duration > 1000) {
        console.warn(`Slow query (${duration}ms): ${sql.substring(0, 100)}`);
      }
    }
  }
  
  // Transaction always goes to primary
  async transaction<T>(
    callback: (client: PoolClient) => Promise<T>
  ): Promise<T> {
    const client = await this.primaryPool.connect();
    
    try {
      await client.query('BEGIN');
      const result = await callback(client);
      await client.query('COMMIT');
      return result;
    } catch (error) {
      await client.query('ROLLBACK');
      throw error;
    } finally {
      client.release();
    }
  }
  
  // Health check for all databases
  async healthCheck(): Promise<{
    primary: boolean;
    replicas: boolean[];
  }> {
    const [primaryHealth, ...replicaHealth] = await Promise.all([
      this.checkPool(this.primaryPool),
      ...this.replicaPools.map(pool => this.checkPool(pool)),
    ]);
    
    return {
      primary: primaryHealth,
      replicas: replicaHealth,
    };
  }
  
  private async checkPool(pool: Pool): Promise<boolean> {
    try {
      await pool.query('SELECT 1');
      return true;
    } catch {
      return false;
    }
  }
}
```

### 3.3 Cached Read Replica Failover

```typescript
// replication/replica-failover.ts

class ReplicaFailoverManager {
  private primary: DatabaseConnection;
  private replicas: DatabaseConnection[];
  private replicaIndex: number = 0;
  private isPrimaryAvailable: boolean = true;
  private healthCheckInterval: number = 30000;
  
  constructor(config: DatabaseConfig[]) {
    this.primary = new DatabaseConnection(config[0]);
    this.replicas = config.slice(1).map(c => new DatabaseConnection(c));
    
    this.startHealthChecks();
    this.setupFailoverHandlers();
  }
  
  private startHealthChecks(): void {
    setInterval(async () => {
      const primaryHealthy = await this.primary.healthCheck();
      
      if (!primaryHealthy && this.isPrimaryAvailable) {
        console.error('Primary database is unhealthy!');
        await this.promoteReplica();
      } else if (primaryHealthy && !this.isPrimaryAvailable) {
        console.log('Primary database recovered');
        this.isPrimaryAvailable = true;
      }
      
      // Check replicas
      for (const replica of this.replicas) {
        const healthy = await replica.healthCheck();
        if (!healthy) {
          console.error(`Replica ${replica.id} is unhealthy`);
        }
      }
    }, this.healthCheckInterval);
  }
  
  private async promoteReplica(): Promise<void> {
    // Find most up-to-date replica
    let bestReplica: DatabaseConnection | null = null;
    let highestLag = Infinity;
    
    for (const replica of this.replicas) {
      const lag = await replica.getReplicationLag();
      if (lag !== null && lag < highestLag) {
        highestLag = lag;
        bestReplica = replica;
      }
    }
    
    if (!bestReplica) {
      throw new Error('No healthy replica available for promotion');
    }
    
    console.log(`Promoting replica ${bestReplica.id} to primary...`);
    
    // Wait for replica to catch up
    await bestReplica.waitForReplication(highestLag + 1);
    
    // Promote
    await bestReplica.promote();
    
    // Swap primary
    const oldPrimary = this.primary;
    this.primary = bestReplica;
    
    // Mark old primary as replica
    this.replicas = this.replicas.filter(r => r !== bestReplica);
    if (!oldPrimary.isReplica()) {
      this.replicas.push(oldPrimary);
    }
    
    this.isPrimaryAvailable = true;
    
    console.log('Replica promotion completed');
  }
  
  // Route query with automatic failover
  async query<T>(
    sql: string,
    readOnly: boolean = false
  ): Promise<T[]> {
    if (readOnly && this.isPrimaryAvailable) {
      // Try replicas first
      try {
        return await this.routeToReplica(sql);
      } catch (error) {
        console.warn('Replica query failed, falling back to primary');
        return await this.primary.query(sql);
      }
    }
    
    return await this.primary.query(sql);
  }
  
  private async routeToReplica<T>(sql: string): Promise<T[]> {
    const replica = this.replicas[this.replicaIndex];
    this.replicaIndex = (this.replicaIndex + 1) % this.replicas.length;
    
    return await replica.query(sql);
  }
}
```

## 4. CQRS for Scaling

### 4.1 CQRS Architecture

```typescript
// cqrs/command-handler.ts

interface Command {
  type: string;
  payload: unknown;
  metadata: {
    userId: string;
    correlationId: string;
    timestamp: Date;
  };
}

interface CommandHandler<T extends Command> {
  handle(command: T): Promise<CommandResult>;
}

interface CommandResult {
  success: boolean;
  data?: unknown;
  error?: {
    code: string;
    message: string;
    details?: unknown;
  };
}

// Create order command
interface CreateOrderCommand extends Command {
  type: 'CREATE_ORDER';
  payload: {
    customerId: string;
    items: Array<{
      productId: string;
      quantity: number;
      price: number;
    }>;
    shippingAddressId: string;
    paymentMethodId: string;
  };
}

// Create order command handler
class CreateOrderHandler implements CommandHandler<CreateOrderCommand> {
  constructor(
    private orderRepository: OrderRepository,
    private inventoryService: InventoryService,
    private paymentService: PaymentService,
    private eventBus: EventBus,
    private outboxStore: OutboxStore
  ) {}
  
  async handle(command: CreateOrderCommand): Promise<CommandResult> {
    const { customerId, items, shippingAddressId, paymentMethodId } = command.payload;
    
    // Start transaction
    const transaction = await this.orderRepository.beginTransaction();
    
    try {
      // 1. Validate inventory
      for (const item of items) {
        const available = await this.inventoryService.checkAvailability(
          item.productId,
          item.quantity
        );
        
        if (!available) {
          throw new InsufficientInventoryError(item.productId);
        }
      }
      
      // 2. Reserve inventory (soft lock)
      for (const item of items) {
        await this.inventoryService.reserve(
          item.productId,
          item.quantity,
          command.metadata.correlationId
        );
      }
      
      // 3. Process payment
      const paymentResult = await this.paymentService.charge(
        customerId,
        paymentMethodId,
        this.calculateTotal(items)
      );
      
      if (!paymentResult.success) {
        throw new PaymentFailedError(paymentResult.error);
      }
      
      // 4. Create order
      const order = await this.orderRepository.create({
        customerId,
        items,
        shippingAddressId,
        paymentTransactionId: paymentResult.transactionId,
        status: 'CONFIRMED',
      }, transaction);
      
      // 5. Record event in outbox for reliability
      await this.outboxStore.save({
        aggregateId: order.id,
        aggregateType: 'Order',
        eventType: 'ORDER_CREATED',
        payload: {
          orderId: order.id,
          customerId,
          total: this.calculateTotal(items),
        },
        metadata: command.metadata,
      }, transaction);
      
      // Commit transaction
      await this.orderRepository.commit(transaction);
      
      // Publish event (after commit)
      await this.eventBus.publish({
        type: 'ORDER_CREATED',
        payload: {
          orderId: order.id,
          customerId,
          items,
          total: this.calculateTotal(items),
        },
        metadata: {
          correlationId: command.metadata.correlationId,
          timestamp: new Date(),
        },
      });
      
      return {
        success: true,
        data: { orderId: order.id },
      };
      
    } catch (error) {
      await this.orderRepository.rollback(transaction);
      
      return {
        success: false,
        error: {
          code: error instanceof Error ? error.name : 'UNKNOWN',
          message: error instanceof Error ? error.message : 'Unknown error',
        },
      };
    }
  }
  
  private calculateTotal(items: Array<{ price: number; quantity: number }>): number {
    return items.reduce((sum, item) => sum + (item.price * item.quantity), 0);
  }
}
```

### 4.2 Event Sourcing with CQRS

```typescript
// cqrs/event-sourced-aggregate.ts

interface Event {
  type: string;
  aggregateId: string;
  aggregateVersion: number;
  payload: unknown;
  metadata: {
    timestamp: Date;
    userId?: string;
    correlationId?: string;
  };
}

interface Aggregate<T> {
  id: string;
  version: number;
  state: T;
  apply(event: Event): void;
  uncommittedEvents: Event[];
  markCommitted(): void;
}

class OrderAggregate implements Aggregate<OrderState> {
  id: string;
  version: number = 0;
  state: OrderState;
  private _uncommittedEvents: Event[] = [];
  
  constructor(id: string, initialState?: OrderState) {
    this.id = id;
    this.state = initialState || this.createInitialState();
  }
  
  get uncommittedEvents(): Event[] {
    return [...this._uncommittedEvents];
  }
  
  private createInitialState(): OrderState {
    return {
      customerId: '',
      items: [],
      status: 'DRAFT',
      total: 0,
      createdAt: new Date(),
      updatedAt: new Date(),
    };
  }
  
  // Command: Place order
  placeOrder(
    customerId: string,
    items: OrderItem[],
    shippingAddress: Address
  ): void {
    if (this.state.status !== 'DRAFT') {
      throw new InvalidOperationError('Order cannot be placed from current status');
    }
    
    if (items.length === 0) {
      throw new ValidationError('Order must have at least one item');
    }
    
    const event = this.createEvent('ORDER_PLACED', {
      customerId,
      items,
      shippingAddress,
      total: this.calculateTotal(items),
      placedAt: new Date(),
    });
    
    this.apply(event);
    this._uncommittedEvents.push(event);
  }
  
  // Command: Confirm order
  confirm(paymentTransactionId: string): void {
    if (this.state.status !== 'PLACED') {
      throw new InvalidOperationError('Order cannot be confirmed from current status');
    }
    
    const event = this.createEvent('ORDER_CONFIRMED', {
      paymentTransactionId,
      confirmedAt: new Date(),
    });
    
    this.apply(event);
    this._uncommittedEvents.push(event);
  }
  
  // Command: Cancel order
  cancel(reason: string, cancelledBy: string): void {
    if (['DELIVERED', 'CANCELLED', 'REFUNDED'].includes(this.state.status)) {
      throw new InvalidOperationError('Order cannot be cancelled from current status');
    }
    
    const event = this.createEvent('ORDER_CANCELLED', {
      reason,
      cancelledBy,
      cancelledAt: new Date(),
      refundAmount: this.calculateRefundAmount(),
    });
    
    this.apply(event);
    this._uncommittedEvents.push(event);
  }
  
  // Event application
  apply(event: Event): void {
    this.version++;
    
    switch (event.type) {
      case 'ORDER_PLACED':
        this.state = {
          ...this.state,
          customerId: event.payload.customerId,
          items: event.payload.items,
          shippingAddress: event.payload.shippingAddress,
          total: event.payload.total,
          status: 'PLACED',
          placedAt: event.payload.placedAt,
          updatedAt: new Date(),
        };
        break;
        
      case 'ORDER_CONFIRMED':
        this.state = {
          ...this.state,
          status: 'CONFIRMED',
          paymentTransactionId: event.payload.paymentTransactionId,
          confirmedAt: event.payload.confirmedAt,
          updatedAt: new Date(),
        };
        break;
        
      case 'ORDER_CANCELLED':
        this.state = {
          ...this.state,
          status: 'CANCELLED',
          cancellation: {
            reason: event.payload.reason,
            cancelledBy: event.payload.cancelledBy,
            cancelledAt: event.payload.cancelledAt,
            refundAmount: event.payload.refundAmount,
          },
          updatedAt: new Date(),
        };
        break;
        
      case 'ORDER_SHIPPED':
        this.state = {
          ...this.state,
          status: 'SHIPPED',
          shippingInfo: event.payload,
          shippedAt: event.payload.shippedAt,
          updatedAt: new Date(),
        };
        break;
        
      case 'ORDER_DELIVERED':
        this.state = {
          ...this.state,
          status: 'DELIVERED',
          deliveredAt: event.payload.deliveredAt,
          updatedAt: new Date(),
        };
        break;
    }
  }
  
  markCommitted(): void {
    this._uncommittedEvents = [];
  }
  
  private createEvent(type: string, payload: unknown): Event {
    return {
      type,
      aggregateId: this.id,
      aggregateVersion: this.version + 1,
      payload,
      metadata: {
        timestamp: new Date(),
      },
    };
  }
  
  private calculateTotal(items: OrderItem[]): number {
    return items.reduce((sum, item) => sum + (item.price * item.quantity), 0);
  }
  
  private calculateRefundAmount(): number {
    if (this.state.status === 'CONFIRMED') {
      return this.state.total;
    }
    return 0;
  }
}

// Query side - materialized view
class OrderQueryModel {
  private projections: Map<string, OrderReadModel> = new Map();
  
  applyEvent(event: Event): void {
    switch (event.type) {
      case 'ORDER_PLACED':
      case 'ORDER_CONFIRMED':
      case 'ORDER_CANCELLED':
      case 'ORDER_SHIPPED':
      case 'ORDER_DELIVERED':
        this.updateProjection(event.aggregateId, event);
        break;
    }
  }
  
  private updateProjection(orderId: string, event: Event): void {
    let projection = this.projections.get(orderId);
    
    if (!projection) {
      projection = new OrderReadModel(orderId);
      this.projections.set(orderId, projection);
    }
    
    projection.apply(event);
  }
  
  getOrder(orderId: string): OrderReadModel | undefined {
    return this.projections.get(orderId);
  }
  
  getOrdersByCustomer(customerId: string): OrderReadModel[] {
    return Array.from(this.projections.values())
      .filter(o => o.customerId === customerId);
  }
}
```

### 4.3 CQRS Event Bus

```typescript
// cqrs/event-bus.ts

interface EventSubscriber<T extends Event = Event> {
  handle(event: T): Promise<void>;
  subscribedTo(): string[];
  name: string;
}

class InMemoryEventBus implements EventBus {
  private subscribers: Map<string, EventSubscriber[]> = new Map();
  private deadLetterQueue: Array<{
    event: Event;
    error: Error;
    failedAt: Date;
    retries: number;
  }> = [];
  private maxRetries: number = 3;
  
  subscribe(subscriber: EventSubscriber): void {
    const eventTypes = subscriber.subscribedTo();
    
    for (const type of eventTypes) {
      if (!this.subscribers.has(type)) {
        this.subscribers.set(type, []);
      }
      this.subscribers.get(type)!.push(subscriber);
    }
  }
  
  unsubscribe(subscriber: EventSubscriber): void {
    for (const [type, subs] of this.subscribers) {
      const index = subs.findIndex(s => s.name === subscriber.name);
      if (index !== -1) {
        subs.splice(index, 1);
      }
    }
  }
  
  async publish<T extends Event>(event: T): Promise<void> {
    const subscribers = this.subscribers.get(event.type) || [];
    
    const publishPromises = subscribers.map(async subscriber => {
      try {
        await subscriber.handle(event);
      } catch (error) {
        console.error(`Subscriber ${subscriber.name} failed to handle ${event.type}:`, error);
        this.handleFailure(event, error as Error);
      }
    });
    
    await Promise.allSettled(publishPromises);
  }
  
  private handleFailure(event: Event, error: Error): void {
    const existing = this.deadLetterQueue.find(
      dle => dle.event.aggregateId === event.aggregateId &&
             dle.event.type === event.type
    );
    
    if (existing) {
      existing.retries++;
      existing.failedAt = new Date();
      existing.error = error;
    } else {
      this.deadLetterQueue.push({
        event,
        error,
        failedAt: new Date(),
        retries: 1,
      });
    }
    
    if (existing && existing.retries >= this.maxRetries) {
      console.error(`Event ${event.type}:${event.aggregateId} moved to DLQ after ${this.maxRetries} retries`);
    }
  }
}

// Kafka event bus for production
class KafkaEventBus implements EventBus {
  private producer: KafkaProducer;
  private consumer: KafkaConsumer;
  private subscriberOffsets: Map<string, Map<string, number>> = new Map();
  
  constructor(private config: KafkaConfig) {
    this.producer = new KafkaProducer({
      'bootstrap.servers': config.brokers,
      'security.protocol': 'SASL_SSL',
      'sasl.mechanism': 'SCRAM-SHA-512',
    });
  }
  
  async publish<T extends Event>(event: T): Promise<void> {
    await this.producer.send({
      topic: this.getTopicForEvent(event.type),
      messages: [
        {
          key: event.aggregateId,
          value: JSON.stringify(event),
          headers: {
            'event-type': event.type,
            'correlation-id': event.metadata.correlationId || '',
            'timestamp': event.metadata.timestamp.toISOString(),
          },
        },
      ],
    });
  }
  
  private getTopicForEvent(type: string): string {
    // Topic naming: {domain}.{entity}.{event}
    return `commerce.orders.${type.toLowerCase()}`;
  }
}
```

## 5. Complete Scaling Manifests

### 5.1 Kubernetes HPA with Multiple Scaling Triggers

```yaml
# k8s/comprehensive-hpa.yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: api-comprehensive-hpa
  namespace: production
  annotations:
    # Enable HPA visibility in metrics server
    metric-config.alpha.kubernetes.io/prometheus: '{"queries":[{"type":"promQL","expression":"..."}]}'
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: api-deployment
  minReplicas: 3
  maxReplicas: 100
  metrics:
    # CPU metric with custom threshold
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 70
    
    # Memory metric
    - type: Resource
      resource:
        name: memory
        target:
          type: Utilization
          averageUtilization: 80
    
    # Custom Prometheus metric - HTTP request rate
    - type: Pods
      pods:
        metric:
          name: http_requests_total
          selector:
            matchLabels:
              app: api
        target:
          type: AverageValue
          averageValue: "500"
    
    # Custom Prometheus metric - Error rate
    - type: Pods
      pods:
        metric:
          name: http_requests_errors_total
          selector:
            matchLabels:
              app: api
        target:
          type: AverageValue
          averageValue: "10"
    
    # Queue depth from Redis
    - type: External
      external:
        metric:
          name: redis_connected_clients
          selector:
            matchLabels:
              role: queue
        target:
          type: AverageValue
          averageValue: "1000"
  
  behavior:
    scaleDown:
      # 5 minute stabilization window
      stabilizationWindowSeconds: 300
      policies:
        # No more than 10% scale down per minute
        - type: Percent
          value: 10
          periodSeconds: 60
        # No more than 2 pods per minute
        - type: Pods
          value: 2
          periodSeconds: 60
      selectPolicy: Min
    
    scaleUp:
      # Immediate scale up (no stabilization)
      stabilizationWindowSeconds: 0
      policies:
        # Can double (100%) pods every 15 seconds
        - type: Percent
          value: 100
          periodSeconds: 15
        # Can add 4 pods every 15 seconds
        - type: Pods
          value: 4
          periodSeconds: 15
      selectPolicy: Max

---
# Prometheus metric scraper for custom metrics
apiVersion: v1
kind: ConfigMap
metadata:
  name: custom-metrics-config
  namespace: production
data:
  metric-names: |
    http_requests_total
    http_requests_errors_total
    queue_depth
    db_connection_pool_size
```

### 5.2 Database Scaling Configuration

```yaml
# k8s/database-scaling.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: postgres-config
  namespace: production
data:
  POSTGRES_MAX_CONNECTIONS: "200"
  POSTGRES_SHARED_BUFFERS: "2GB"
  POSTGRES_EFFECTIVE_CACHE_SIZE: "6GB"
  POSTGRES_MAINTENANCE_WORK_MEM: "512MB"
  POSTGRES_WORK_MEM: "16MB"
  POSTGRES_MIN_WAL_SIZE: "1GB"
  POSTGRES_MAX_WAL_SIZE: "4GB"
  POSTGRES_CHECKPOINT_COMPLETION_TARGET: "0.9"
  POSTGRES_WAL_BUFFFS: "16MB"
  POSTGRES_DEFAULT_STATISTICS_TARGET: "100"

---
# PostgreSQL statefulset with read replicas
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: postgres-primary
  namespace: production
spec:
  serviceName: postgres-primary
  replicas: 1
  selector:
    matchLabels:
      app: postgres
      role: primary
  template:
    metadata:
      labels:
        app: postgres
        role: primary
    spec:
      containers:
        - name: postgres
          image: postgres:15-alpine
          ports:
            - containerPort: 5432
          env:
            - name: POSTGRES_DB
              value: app
            - name: POSTGRES_USER
              valueFrom:
                secretKeyRef:
                  name: postgres-secrets
                  key: username
            - name: POSTGRES_PASSWORD
              valueFrom:
                secretKeyRef:
                  name: postgres-secrets
                  key: password
          resources:
            requests:
              cpu: "2"
              memory: 4Gi
            limits:
              cpu: "4"
              memory: 8Gi
          volumeMounts:
            - name: postgres-data
              mountPath: /var/lib/postgresql/data
          livenessProbe:
            exec:
              command: ["pg_isready", "-U", "app"]
            initialDelaySeconds: 30
            periodSeconds: 10
          readinessProbe:
            exec:
              command: ["pg_isready", "-U", "app", "-d", "app"]
            initialDelaySeconds: 5
            periodSeconds: 5
  volumeClaimTemplates:
    - metadata:
        name: postgres-data
      spec:
        accessModes: ["ReadWriteOnce"]
        storageClassName: fast-ssd
        resources:
          requests:
            storage: 100Gi

---
# Read replica deployment
apiVersion: apps/v1
kind: Deployment
metadata:
  name: postgres-replica
  namespace: production
spec:
  replicas: 3
  selector:
    matchLabels:
      app: postgres
      role: replica
  template:
    metadata:
      labels:
        app: postgres
        role: replica
    spec:
      containers:
        - name: postgres
          image: postgres:15-alpine
          command:
            - sh
            - -c
            - |
              exec postgres \
                -c shared_buffers=1GB \
                -c max_connections=100 \
                -c hot_standby=on \
                -c primary_conninfo='host=postgres-primary port=5432 user=replica'
          ports:
            - containerPort: 5432
          resources:
            requests:
              cpu: "1"
              memory: 2Gi
            limits:
              cpu: "2"
              memory: 4Gi
```

### 5.3 CronJob for Database Maintenance

```yaml
# k8s/database-maintenance.yaml
apiVersion: batch/v1
kind: CronJob
metadata:
  name: postgres-maintenance
  namespace: production
spec:
  schedule: "0 2 * * *"  # 2 AM daily
  concurrencyPolicy: Forbid
  successfulJobsHistoryLimit: 3
  failedJobsHistoryLimit: 3
  jobTemplate:
    spec:
      backoffLimit: 2
      template:
        spec:
          serviceAccountName: postgres-maintenance
          containers:
            - name: maintenance
              image: postgres:15-alpine
              command:
                - sh
                - -c
                - |
                  # Analyze tables for query optimization
                  psql -c "ANALYZE;"
                  
                  # Vacuum with aggressive cleanup
                  psql -c "VACUUM (FULL, ANALYZE, VERBOSE);"
                  
                  # Reindex bloated indexes
                  psql -c "REINDEX DATABASE app;"
                  
                  # Check for bloated tables
                  psql -c "SELECT tablename, pg_size_pretty(pg_total_relation_size(tablename::regclass)) AS size FROM pg_tables WHERE schemaname = 'public' ORDER BY pg_total_relation_size(tablename::regclass) DESC LIMIT 10;"
              env:
                - name: PGHOST
                  value: postgres-primary
                - name: PGDATABASE
                  value: app
                - name: PGUSER
                  valueFrom:
                    secretKeyRef:
                      name: postgres-secrets
                      key: username
                - name: PGPASSWORD
                  valueFrom:
                    secretKeyRef:
                      name: postgres-secrets
                      key: password
          restartPolicy: OnFailure
```

## 6. Decision Matrices

### 6.1 Scaling Strategy Selection Matrix

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                           Scaling Strategy Selection Matrix                              │
├─────────────────────────────────────────────────────────────────────────────────────────┤
│ Factor                        │ Vertical     │ Horizontal  │ Database   │ Caching   │
│                               │ Scaling      │ Scaling     │ Scaling    │ Scaling   │
├───────────────────────────────┼──────────────┼─────────────┼────────────┼──────────┤
│ Simple implementation        │ Best (1 param)│ Moderate    │ Complex    │ Moderate  │
├───────────────────────────────┼──────────────┼─────────────┼────────────┼──────────┤
│ Cost efficiency (small load) │ Best          │ Higher cost  │ Higher cost│ Best     │
├───────────────────────────────┼──────────────┼─────────────┼────────────┼──────────┤
│ Performance (large load)     │ Limited       │ Best        │ Best       │ Best     │
├───────────────────────────────┼──────────────┼─────────────┼────────────┼──────────┤
│ Availability/Fault tolerance │ No improvement│ Best        │ Moderate   │ Moderate │
├───────────────────────────────┼──────────────┼─────────────┼────────────┼──────────┤
│ Data isolation               │ Good          │ No change   │ Challenge  │ N/A      │
├───────────────────────────────┼──────────────┼─────────────┼────────────┼──────────┤
│ Consistency guarantees       │ No change     │ No change   │ Complex    │ Stale    │
├───────────────────────────────┼──────────────┼─────────────┼────────────┼──────────┤
│ Operational complexity       │ Low           │ Medium      │ High       │ Medium   │
└───────────────────────────────┴──────────────┴─────────────┴────────────┴──────────┘
```

### 6.2 Autoscaling Metric Selection

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                          Autoscaling Metric Selection Matrix                             │
├─────────────────────────────────────────────────────────────────────────────────────────┤
│ Metric Type                   │ When to Use                    │ When NOT to Use        │
├───────────────────────────────┼────────────────────────────────┼────────────────────────┤
│ CPU Utilization              │ Compute-bound workloads         │ I/O bound, waiting for │
│                               │ Fast response needed            │ external services      │
├───────────────────────────────┼────────────────────────────────┼────────────────────────┤
│ Memory Utilization           │ Memory leaks, caches            │ Memory stable but CPU  │
│                               │ Stateful services               │ high                   │
├───────────────────────────────┼────────────────────────────────┼────────────────────────┤
│ Request per second           │ HTTP services with known        │ Variable response size │
│                               │ consistent response time        │ or complexity          │
├───────────────────────────────┼────────────────────────────────┼────────────────────────┤
│ Queue depth                 │ Background workers              │ Request-response apps  │
│                               │ Batch processing                │                         │
├───────────────────────────────┼────────────────────────────────┼────────────────────────┤
│ Custom business metric       │ Domain-specific thresholds      │ Generic infrastructure │
│                               │ (cart size, conversion)        │ monitoring             │
├───────────────────────────────┼────────────────────────────────┼────────────────────────┤
│ Response time (latency)      │ User-facing services            │ Services with variable │
│                               │ SLO-based scaling               │ upstream dependencies  │
├───────────────────────────────┼────────────────────────────────┼────────────────────────┤
│ Error rate                   │ Reliability-focused scaling     │ When errors are part   │
│                               │ Error budget awareness          │ of normal operation    │
└───────────────────────────────┴────────────────────────────────┴────────────────────────┘
```

## 7. Anti-Patterns

### 7.1 Scaling Anti-Patterns

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                              Scaling Anti-Patterns to Avoid                              │
├─────────────────────────────────────────────────────────────────────────────────────────┤
│ Anti-Pattern                    │ Problem                       │ Solution                │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Scaling without metrics        │ Wrong decisions               │ Implement observability│
│                                 │ Can't measure impact          │ first                  │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ No scaling cooldown            │ Flapping, instability         │ Set stabilization      │
│                                 │ Resource thrashing            │ windows                │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Scaling on single metric       │ Missed signals                │ Use multiple metrics   │
│                                 │ Bottleneck moves              │ with weightings        │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Max replicas too low           │ Can't handle peak              │ Set based on capacity  │
│                                 │ Service degradation           │ planning               │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ No resource limits            │ Resource exhaustion            │ Set memory/CPU limits  │
│                                 │ OOM kills                     │ on all workloads       │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Scaling stateless apps         │ State loss                     │ External state store   │
│ without state separation       │                               │ (Redis, DB)            │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Database bottleneck ignored    │ Apps scale, DB doesn't         │ Scale database first   │
│                                 │ Latency increases             │ or implement caching   │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ No connection pooling         │ Connection exhaustion          │ Use poolers            │
│                                 │ Latency spikes                │ (PgBouncer, etc)       │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Synchronous cross-service      │ Blocking, cascading failures  │ Use async messaging    │
│ calls                          │                               │ for dependencies       │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ No read/write splitting       │ Read load on primary           │ Implement CQRS pattern │
│                                 │ Replication lag issues         │ for read replicas      │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Sharding too early            │ Complexity explosion           │ Scale reads/writes     │
│                                 │ Cross-shard queries slow       │ separately first       │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ No circuit breaker           │ Cascade failures               │ Implement circuit      │
│                                 │ Service unavailability        │ breaker pattern        │
└─────────────────────────────────┴───────────────────────────────┴────────────────────────┘
```

### 7.2 Database Scaling Mistakes

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                           Database Scaling Mistakes to Avoid                             │
├─────────────────────────────────────────────────────────────────────────────────────────┤
│ Mistake                       │ Problem                       │ Solution                  │
├───────────────────────────────┼───────────────────────────────┼──────────────────────────┤
│ Adding replicas without      │ Replication lag               │ Use connection poolers   │
│ connection pooling           │ Connection exhaustion         │ and read/write splitting │
├───────────────────────────────┼───────────────────────────────┼──────────────────────────┤
│ Sharding without clear       │ Cross-shard queries           │ Choose shard key based   │
│ shard key strategy           │ Data hotspots                 │ on access patterns       │
├───────────────────────────────┼───────────────────────────────┼──────────────────────────┤
│ Vertical scaling as default  │ Hardware limits               │ Plan for horizontal      │
│ approach                     │ Expensive                     │ scaling from start       │
├───────────────────────────────┼───────────────────────────────┼──────────────────────────┤
│ Ignoring query optimization  │ Index bloat                   │ Analyze slow queries     │
│ before scaling               │ Full table scans              │ and optimize before scale│
├───────────────────────────────┼───────────────────────────────┼──────────────────────────┤
│ No caching strategy         │ Database overload             │ Implement multi-level    │
│                               │ High latency                  │ caching (app, CDN, etc)  │
├───────────────────────────────┼───────────────────────────────┼──────────────────────────┤
│ Using DB for sessions        │ Session load on DB            │ Use Redis/memcached     │
│                               │ Replication issues            │ for session storage      │
└───────────────────────────────┴───────────────────────────────┴──────────────────────────┘
```

---

## Links

### Kubernetes Autoscaling
- [HPA Documentation](https://kubernetes.io/docs/tasks/run-application/horizontal-pod-autoscale/)
- [VPA Documentation](https://github.com/kubernetes/autoscaler/tree/master/vertical-pod-autoscaler)
- [KEDA - Event-driven autoscaling](https://keda.sh/)
- [Custom Metrics API](https://github.com/kubernetes/metrics)

### Database Scaling
- [Citus - PostgreSQL extension for sharding](https://www.citusdata.com/)
- [Vitess - Database clustering for MySQL](https://vitess.io/)
- [TiDB - Distributed SQL database](https://tidb.apache.org/)
- [PlanetScale - MySQL-compatible serverless database](https://planetscale.com/)

### Read Replicas
- [AWS RDS Read Replicas](https://docs.aws.amazon.com/AmazonRDS/latest/UserGuide/USER_ReadRepl.html)
- [Cloudflare Database Connector](https://developers.cloudflare.com/database-connector/)
- [PgBouncer - Connection pooler](https://www.pgbouncer.org/)

### CQRS & Event Sourcing
- [CQRS Pattern - Microsoft](https://docs.microsoft.com/en-us/azure/architecture/patterns/cqrs)
- [Event Sourcing Pattern - Microsoft](https://docs.microsoft.com/en-us/azure/architecture/patterns/event-sourcing)
- [Axon Framework](https://axoniq.io/)
- [EventStoreDB](https://eventstore.com/)

### Load Balancing
- [Envoy Proxy](https://www.envoyproxy.io/)
- [Traefik](https://traefik.io/)
- [NGINX Load Balancing](https://docs.nginx.com/nginx/admin-guide/load-balancer/)

### Metrics & Monitoring
- [Prometheus](https://prometheus.io/)
- [Grafana](https://grafana.com/)
- [Datadog](https://www.datadoghq.com/)
- [New Relic](https://newrelic.com/)

### Performance
- [Google SRE Book - Scaling](https://sre.google/sre-book/scaling/)
- [High Scalability Blog](http://highscalability.com/)
- [AWS Well-Architected - Performance](https://docs.aws.amazon.com/wellarchitected/latest/framework/welcome.html)