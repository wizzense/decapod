# DISTRIBUTED_SYSTEMS.md - Distributed Systems Architecture for Microservices

**Authority:** guidance (comprehensive topic with exact specifications)
**Layer:** Architecture
**Binding:** No
**Scope:** Comprehensive topic coverage for pre-inference context

## Table of Contents

1. [Fundamental Theorems](#1-fundamental-theorems)
2. [Consensus Algorithms](#2-consensus-algorithms)
3. [Distributed Transactions](#3-distributed-transactions)
4. [Clock Synchronization](#4-clock-synchronization)
5. [CRDT Patterns](#5-crdt-patterns)
6. [Configuration Specifications](#6-configuration-specifications)
7. [Decision Matrix](#7-decision-matrix)
8. [Failure Modes and Recovery](#8-failure-modes-and-recovery)
9. [Production Implementation Guide](#9-production-implementation-guide)
10. [References](#10-references)

---

## 1. Fundamental Theorems

### 1.1 CAP Theorem

The CAP theorem states that a distributed data store can only guarantee two of three properties simultaneously:

- **Consistency (C)**: Every read receives the most recent write or an error
- **Availability (A)**: Every request receives a response, without guarantee that it contains the most recent write
- **Partition Tolerance (P)**: The system continues to operate despite network partitions

**Critical Insight**: Partitions are unavoidable in real systems. Therefore, the real choice is between:
- **CP Systems**: Sacrifice availability during partitions (e.g., ZooKeeper, etcd)
- **AP Systems**: Sacrifice strong consistency during partitions (e.g., Cassandra, DynamoDB)

### 1.2 PACELC Model

PACELC extends CAP with latency considerations:

```
IF network partition (P)
  THEN choose between Availability (A) or Consistency (C)
ELSE (E)
  THEN choose between Latency (L) or Consistency (C)
```

| System | Partition Behavior | Normal Operation (Latency vs Consistency) |
|--------|-------------------|------------------------------------------|
| DynamoDB | Available | Latency |
| Cassandra | Available | Latency |
| etcd | Consistent | Latency |
| ZooKeeper | Consistent | Latency |
| HBase | Consistent | Consistency |
| MongoDB | Available (eventual) | Latency |

### 1.3 Consistency Levels

**Strong Consistency**
- All reads see the same data immediately after any write
- Achieved via: Synchronous replication, consensus protocols
- Latency: High (network round-trips required)
- Use case: Financial transactions, inventory management

**Sequential Consistency**
- All processes see data in the same order across all nodes
- Weaker than strong consistency, stronger than eventual consistency
- Achieved via: Version vectors, vector clocks
- Use case: Cache invalidation, leader election

**Causal Consistency**
- Causally related operations are seen by all processes in order
- Non-causally related operations may be seen in different orders
- Achieved via: Vector clocks, tracking dependencies
- Use case: Social media feeds, comments on posts

**Eventual Consistency**
- All updates will eventually propagate to all replicas
- Property: If no new updates are made, eventually all reads will return the last written value
- Achieved via: Asynchronous replication, anti-entropy, Merkle trees
- Latency: Low (reads can be served locally)
- Use case: CDN content, user profiles, like counts

**Read-your-writes Consistency**
- A process always sees its own writes
- Achieved via: Sticky sessions, write-after-read tracking
- Use case: User sessions, shopping carts

**Monotonic Read Consistency**
- Once a process sees a particular value, it will never see older values
- Achieved via: Read timestamps, versioning
- Use case: DNS caching, distributed file systems

### 1.4 Consistency Level Configuration Examples

```yaml
# Cassandra consistency levels configuration
cassandra:
  consistency_levels:
    # Operations that require quorum for both read and write
    strongly_consistent:
      read: QUORUM
      write: QUORUM
      read_repair_chance: 0.9
      dc_local_read_timeout: 5000ms
    
    # Eventual consistency for non-critical data
    eventually_consistent:
      read: ONE
      write: ANY
      read_repair_chance: 0.1
      gc_grace_seconds: 864000  # 10 days
    
    # Write-heavy workload optimization
    write_optimized:
      read: LOCAL_ONE
      write: LOCAL_QUORUM
      write_timeout: 3000ms
      read_timeout: 2000ms
    
    # Linearizable consistency for leader elections
    linearizable:
      read: SERIAL
      write: SERIAL
      conditional_write_timeout: 5000ms

# DynamoDB consistency configuration
dynamodb:
  consistency_strategies:
    strong:
      read: strong
      write: transactional
      provisioned_throughput:
        read: 1000
        write: 1000
    
    eventual:
      read: eventual
      write: standard
      provisioned_throughput:
        read: 5000
        write: 1000
    
    adaptive:
      read_strategy: adaptive
      write_strategy: transactional
      fallback_read_on_retry: true
```

---

## 2. Consensus Algorithms

### 2.1 Raft Consensus Algorithm

Raft was designed to be more understandable than Paxos while providing the same guarantees. It decomposes consensus into three sub-problems:

1. **Leader Election**: Single leader manages replicated log
2. **Log Replication**: Leader replicates entries to followers
3. **Safety**: Consistent log across cluster

#### Raft States and Transitions

```
States: FOLLOWER | CANDIDATE | LEADER

Transitions:
- Follower -> Candidate: Election timeout expires without leader heartbeat
- Candidate -> Leader: Receives votes from majority of nodes
- Candidate -> Follower: Receives heartbeat from new leader
- Leader -> Follower: Receives higher term from peer
```

#### Raft Timing Parameters

| Parameter | Description | Typical Value |
|-----------|-------------|---------------|
| electionTimeout | Time before follower becomes candidate | 150-300ms random |
| heartbeatInterval | Leader sends append entries | 50-150ms |
| rpcTimeout | Timeout for RPC calls | 300ms |
| electionTimeoutUpperBound | Max election timeout | 300ms |
| minElectionTimeout | Minimum election timeout | 150ms |

#### Etcd Raft Configuration

```yaml
# etcd cluster configuration with Raft settings
apiVersion: v1
kind: ConfigMap
metadata:
  name: etcd-config
  namespace: platform
data:
  etcd.conf.yml: |
    # Cluster member configuration
    member:
      name: etcd-0
      data-dir: /var/lib/etcd
      wallet-dir: /var/lib/etcd/wal
      snapshot-count: 10000
      heartbeat-interval: 100
      election-timeout: 1000
      election-timeout-ms: 1000
      quota-backend-bytes: 8589934592  # 8GB
      max-request-bytes: 1572864  # 1.5MB
      max-mSnapshots: 5
      max-wals: 5
      cors: []
    
    # Peer configuration
    peer:
      auto-tls: false
      peer-client-tls-auth: true
      peer-trusted-ca-file: /etc/kubernetes/pki/etcd/ca.crt
      peer-cert-file: /etc/kubernetes/pki/etcd/peer.crt
      peer-key-file: /etc/kubernetes/pki/etcd/peer.key
    
    # Client configuration  
    client:
      auto-tls: false
      client-cert-auth: true
      trusted-ca-file: /etc/kubernetes/pki/etcd/ca.crt
      cert-file: /etc/kubernetes/pki/etcd/server.crt
      key-file: /etc/kubernetes/pki/etcd/server.key
      unauthenticated: false
      max-snapshots: 5
      max-wals: 5
      cipher-suites: ""
      advertise-client-urls: https://10.0.0.10:2379
      client-urls: https://0.0.0.0:2379
      secure-serving: true
      unix-socket: /var/run/etcd.sock
    
    # Logging configuration
    log:
      dir: /var/log/etcd
      level: info
      package-config: ""
      zap-output-format: json
      output-config: ""
    
    # Raft specific settings
    raft:
      election-timeout-ms: 1000
      heartbeat-interval-ms: 100
      max-inflight-msgs: 10
      max-snapshot-traverse: 10
      check-quorum: true
      pre-vote: true
      step-middle-commit-timeout: false
      leader-old-peer-check: false
      disable-commit-merged: false
      tick: heartBeat
      election: tick
      heartbeat: 1  # Number of ticks between heartbeats
      election: 10   # Number of ticks before election
    
    # Cluster configuration
    cluster:
      initial:
        cluster-state: new
        new-member-urls: https://10.0.0.10:2380
        initial-advertise-peer-urls: https://10.0.0.10:2380
      heartbeat: 100  # Heartbeat interval (ms) for discovery
      election: 1000  # Election timeout (ms) for discovery
      initial-cluster: etcd-0=https://10.0.0.10:2380,etcd-1=https://10.0.0.11:2380,etcd-2=https://10.0.0.12:2380
      initial-cluster-state: new
      initial-cluster-token: etcd-cluster
      discovery: ""
      discovery-fallback: exit
      discovery-dns: ""
      discovery-proxy: ""
      discovery-srv: ""
      auto-tls: false
      strict-reconfig-check: true
      remove-member-check: true
      prefix: /_etcd/rpc/
      compaction-batch-limit: 1000
      compaction-interval: 5000
      compaction-interval-h: "1h"
      pagination-batch-limit: 10000
      pagination-max: 10000

---
# Kubernetes etcd cluster setup
apiVersion: v1
kind: Secret
metadata:
  name: etcd-tls
  namespace: platform
type: kubernetes.io/tls
stringData:
  # Certificate configuration for etcd
  # Generated via: cfssl or similar PKI tool
  ca.crt: |
    -----BEGIN CERTIFICATE-----
    MIAGCSqGSIb3DQEHAqCAMIACAH2ghhOdHJ1c2tleTEiMCAGA1UEChMZZ295dGhp
    ... (truncated for brevity)
    -----END CERTIFICATE-----
---
# Etcd member pod
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: etcd
  namespace: platform
spec:
  serviceName: etcd
  replicas: 3
  podManagementPolicy: Parallel
  selector:
    matchLabels:
      app: etcd
  template:
    metadata:
      labels:
        app: etcd
    spec:
      containers:
      - name: etcd
        image: gcr.io/etcd-development/etcd:v3.5.12
        command:
        - /usr/local/bin/etcd
        - --name=$(HOSTNAME)
        - --data-dir=/var/lib/etcd
        - --wallet-dir=/var/lib/etcd/wal
        - --cert-file=/etc/ssl/certs/etcd/server.crt
        - --key-file=/etc/ssl/certs/etcd/server.key
        - --trusted-ca-file=/etc/ssl/certs/etcd/ca.crt
        - --client-cert-auth=true
        - --peer-cert-file=/etc/ssl/certs/etcd/peer.crt
        - --peer-key-file=/etc/ssl/certs/etcd/peer.key
        - --peer-trusted-ca-file=/etc/ssl/certs/etcd/ca.crt
        - --peer-client-cert-auth=true
        - --initial-advertise-peer-urls=https://$(HOSTNAME).etcd.platform.svc.cluster.local:2380
        - --listen-peer-urls=https://0.0.0.0:2380
        - --advertise-client-urls=https://$(HOSTNAME).etcd.platform.svc.cluster.local:2379
        - --listen-client-urls=https://0.0.0.0:2379
        - --heartbeat-interval=100
        - --election-timeout=1000
        - --snapshot-count=10000
        - --max-snapshots=5
        - --max-wals=5
        - --quota-backend-bytes=8589934592
        - --grpc-keepalive-timeout=20s
        - --grpc-keepalive-interval=2h
        - --peer-read-buffer-size=1048576
        - --peer-write-buffer-size=1048576
        - --backend-batch-interval=100ms
        - --backend-batch-limit=1000
        ports:
        - containerPort: 2379
          name: client
        - containerPort: 2380
          name: peer
        env:
        - name: HOSTNAME
          valueFrom:
            fieldRef:
              fieldPath: metadata.name
        - name: ETCD_NAME
          valueFrom:
            fieldRef:
              fieldPath: metadata.name
        - name: ETCD_INITIAL_CLUSTER
          value: "etcd-0=https://etcd-0.etcd.platform.svc.cluster.local:2380,etcd-1=https://etcd-1.etcd.platform.svc.cluster.local:2380,etcd-2=https://etcd-2.etcd.platform.svc.cluster.local:2380"
        - name: ETCD_INITIAL_CLUSTER_STATE
          value: new
        - name: ETCD_INITIAL_CLUSTER_TOKEN
          value: etcd-cluster
        - name: ETCDCTL_API
          value: "3"
        - name: ETCDCTL_CERT
          value: /etc/ssl/certs/etcd/client.crt
        - name: ETCDCTL_KEY
          value: /etc/ssl/certs/etcd/client.key
        - name: ETCDCTL_CACERT
          value: /etc/ssl/certs/etcd/ca.crt
        resources:
          requests:
            cpu: 500m
            memory: 2Gi
          limits:
            cpu: 2000m
            memory: 8Gi
        livenessProbe:
          exec:
            command:
            - /usr/local/bin/etcdctl
            - --endpoints=https://localhost:2379
            - --cacert=/etc/ssl/certs/etcd/ca.crt
            - --cert=/etc/ssl/certs/etcd/client.crt
            - --key=/etc/ssl/certs/etcd/client.key
            - endpoint health
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 3
        readinessProbe:
          exec:
            command:
            - /usr/local/bin/etcdctl
            - --endpoints=https://localhost:2379
            - --cacert=/etc/ssl/certs/etcd/ca.crt
            - --cert=/etc/ssl/certs/etcd/client.crt
            - --key=/etc/ssl/certs/etcd/client.key
            - endpoint health
            - --if-available
          initialDelaySeconds: 5
          periodSeconds: 5
          timeoutSeconds: 3
        volumeMounts:
        - name: etcd-data
          mountPath: /var/lib/etcd
        - name: etcd-wal
          mountPath: /var/lib/etcd/wal
        - name: etcd-certs
          mountPath: /etc/ssl/certs/etcd
        securityContext:
          runAsNonRoot: true
          runAsUser: 1000
          fsGroup: 1000
      volumes:
      - name: etcd-data
        persistentVolumeClaim:
          claimName: etcd-data
      - name: etcd-wal
        emptyDir:
          medium: Memory
          sizeLimit: 1Gi
      - name: etcd-certs
        secret:
          secretName: etcd-tls
```

### 2.2 Paxos Consensus Algorithm

Paxos is the foundational consensus algorithm. It operates in two phases:

**Phase 1 (Prepare)**
1. Proposer selects proposal number N
2. Proposer sends Prepare(N) to majority of acceptors
3. Acceptors respond with Promise if N > any previous prepare they've responded to

**Phase 2 (Accept)**
1. Proposer sends Accept(N, value) to majority
2. Acceptors accept if they haven't promised to a higher number
3. Once majority accepts, value is chosen

#### Multi-Paxos Optimization

In practice, systems use Multi-Paxos to elect a stable leader and batch operations:

```python
# Multi-Paxos leader lease implementation concept
class MultiPaxosNode:
    def __init__(self, node_id, peers):
        self.node_id = node_id
        self.peers = peers
        self.state = "follower"
        self.current_term = 0
        self.voted_for = None
        self.log = []
        self.commit_index = 0
        self.last_applied = 0
        self.leader_lease = None
        
    async def become_leader(self):
        """Optimized leader election with lease"""
        self.state = "leader"
        self.current_term += 1
        self.voted_for = self.node_id
        
        # Send AppendEntries to all peers to establish leadership
        await self.broadcast_heartbeat()
        
        # Acquire leader lease from majority
        lease_responses = await self.gather_leases()
        if len(lease_responses) >= len(self.peers) // 2 + 1:
            self.leader_lease = Lease(
                term=self.current_term,
                expiry=now() + LEASE_DURATION,
                leader_id=self.node_id
            )
        
    async def handle_prepare(self, proposal_id):
        """Phase 1 of classic Paxos"""
        if proposal_id.term < self.current_term:
            return PromiseRejected(term=self.current_term)
        
        if self.last_promised_proposal_id is None or proposal_id > self.last_promised_proposal_id:
            self.last_promised_proposal_id = proposal_id
            return PromiseAccepted(
                proposal_id=proposal_id,
                accepted_proposal_id=self.accepted_proposal_id,
                accepted_value=self.accepted_value
            )
        return PromiseRejected(proposal_id=self.last_promised_proposal_id)
    
    async def handle_accept(self, proposal_id, value):
        """Phase 2 of classic Paxos"""
        if proposal_id.term < self.current_term:
            return AcceptRejected(term=self.current_term)
        
        if self.last_promised_proposal_id is not None and proposal_id < self.last_promised_proposal_id:
            return AcceptRejected(proposal_id=self.last_promised_proposal_id)
        
        self.accepted_proposal_id = proposal_id
        self.accepted_value = value
        
        return AcceptAccepted(proposal_id=proposal_id)
    
    async def handle_learn(self, proposal_id, value):
        """Learn phase - value has been chosen"""
        if proposal_id > self.highest_learned_proposal_id:
            self.highest_learned_proposal_id = proposal_id
            self.commit_value(value)
```

### 2.3 Consensus Protocol Comparison

| Property | Raft | Paxos | Multi-Paxos | Zab |
|----------|------|-------|-------------|-----|
| Understandability | High | Low | Medium | Medium |
| Leader election | Strong leader | No inherent leader | Leader optimization | Strong leader |
| Log replication | Append-only | Generic | Append-only | Append-only |
| Membership changes | Joint quorum | Complex | Single server | Dynamic |
| Implementation complexity | Medium | High | High | Medium |
| Performance | Good | Poor (single decree) | Excellent | Excellent |
| Formal verification | Available | Classic | Extensions | Available |
| Examples | etcd, CockroachDB, TiKV | Chubby, LibPaxos | Spanner | ZooKeeper |

---

## 3. Distributed Transactions

### 3.1 Two-Phase Commit (2PC)

2PC is a atomic commitment protocol with two phases:

**Phase 1: Prepare**
1. Coordinator sends Prepare to all participants
2. Participants vote Yes/No
3. Participants write PREPARE to their log and lock resources

**Phase 2: Commit/Rollback**
1. Coordinator decides commit (if all Yes) or rollback
2. Coordinator writes COMMIT/ABORT to log
3. Coordinator sends decision to all participants
4. Participants commit/rollback and release locks

```yaml
# Two-Phase Commit configuration
two_phase_commit:
  coordinator:
    name: payment-coordinator
    transaction_timeout: 30s
    max_retries: 3
    retry_backoff: exponential
    initial_backoff: 1s
    max_backoff: 30s
    abort_on_timeout: true
    parallel_prepare: true
    parallel_commit: true
  
  participant:
    name: payment-service
    prepare_timeout: 10s
    commit_timeout: 15s
    rollback_timeout: 10s
    deadlock_detection_timeout: 60s
    lock_timeout: 300s
    heuristic_decision: rollback  # Options: rollback, commit, rollback_partial
    
  recovery:
    auto_recovery: true
    recovery_interval: 30s
    xa_recovery_interval: 60s
    in-doubt_transaction_timeout: 86400s  # 24 hours
    
  logging:
    log_dir: /var/log/2pc
    fsync_enabled: true
    trace_transactions: true
```

**2PC Failure Modes**

| Failure Point | Result | Recovery Action |
|---------------|--------|----------------|
| Coordinator crashes before prepare | Participants timeout, auto rollback | Coordinator recovers, completes rollback |
| Coordinator crashes after prepare, before commit | Participants in prepared state, blocked | Coordinator recovers, completes commit/rollback |
| Participant crashes before prepare | Coordinator timeout, rollback | Participant recovers, no action needed |
| Participant crashes after prepare | Coordinator commits | Participant recovers, applies commit |
| Network partition during commit | Coordinator can't reach majority | Participants block indefinitely |

### 3.2 Saga Pattern

Sagas replace ACID transactions with a sequence of local transactions, with compensating transactions for rollback.

**Choreography-Based Saga**
Services emit and listen to events without central coordinator.

```yaml
# Order Saga - Choreography based
order_saga:
  name: order-fulfillment-saga
  type: choreography
  
  steps:
    - name: create-order
      service: order-service
      action: create_order
      compensation: cancel_order
      timeout: 30s
      retry:
        max_attempts: 3
        backoff: exponential
        initial: 1s
        max: 30s
      
    - name: reserve-inventory
      service: inventory-service
      action: reserve_inventory
      compensation: release_inventory
      timeout: 15s
      retry:
        max_attempts: 3
        backoff: exponential
      
    - name: process-payment
      service: payment-service
      action: charge_customer
      compensation: refund_payment
      timeout: 30s
      retry:
        max_attempts: 3
        max_per_step_timeout: 120s
      
    - name: send-notification
      service: notification-service
      action: send_order_confirmation
      compensation: void_notification
      timeout: 10s
      compensation_not_required: true  # Notification doesn't need compensation
      
  error_handling:
    retryable_errors:
      - RESOURCE_TEMPORARILY_UNAVAILABLE
      - TIMEOUT
      - SERVICE_UNAVAILABLE
    non_retryable_errors:
      - INSUFFICIENT_INVENTORY
      - PAYMENT_DECLINED
      - INVALID_CUSTOMER
    default_on_non_retryable: compensate_from_current
    
  observability:
    saga_state_events: true
    compensation_events: true
    correlation_id_propagation: true
```

**Orchestration-Based Saga**
A central coordinator (saga orchestrator) directs the participants.

```yaml
# Order Saga - Orchestration based
apiVersion: microservices.io/v1alpha1
kind: SagaOrchestrator
metadata:
  name: order-fulfillment-orchestrator
  namespace: platform
spec:
  name: order-fulfillment-saga
  initialCommand:
    name: CreateOrderSaga
    payload:
      orderId: "{$.command.payload.orderId}"
      customerId: "{$.command.payload.customerId}"
      items: "{$.command.payload.items}"
  
  steps:
    - name: createOrder
      service: order-service
      command:
        name: CreateOrder
        parameters:
          customerId: "{$.command.payload.customerId}"
          items: "{$.command.payload.items}"
          idempotencyKey: "{$.command.payload.orderId}"
      compensate:
        service: order-service
        command:
          name: CancelOrder
          parameters:
            orderId: "{$.ctx.createOrder.orderId}"
      onSuccess: reserveInventory
      onError:
        then: compensateFromStep
        compensationOrder: []
      timeout: 30s
      
    - name: reserveInventory
      service: inventory-service
      command:
        name: ReserveInventory
        parameters:
          items: "{$.command.payload.items}"
          orderId: "{$.ctx.createOrder.orderId}"
          reservationTimeout: 3600s
      compensate:
        service: inventory-service
        command:
          name: ReleaseInventory
          parameters:
            reservationId: "{$.ctx.reserveInventory.reservationId}"
      onSuccess: processPayment
      onError:
        then: compensateFromStep
        compensationOrder: [createOrder]
      timeout: 15s
      
    - name: processPayment
      service: payment-service
      command:
        name: ChargePayment
        parameters:
          customerId: "{$.command.payload.customerId}"
          amount: "{$.ctx.createOrder.totalAmount}"
          currency: "{$.command.payload.currency}"
          orderId: "{$.ctx.createOrder.orderId}"
          paymentMethodId: "{$.command.payload.paymentMethodId}"
      compensate:
        service: payment-service
        command:
          name: RefundPayment
          parameters:
            transactionId: "{$.ctx.processPayment.transactionId}"
            amount: "{$.ctx.processPayment.chargedAmount}"
      onSuccess: confirmOrder
      onError:
        then: compensateFromStep
        compensationOrder: [reserveInventory, createOrder]
      timeout: 30s
      retry:
        maxAttempts: 5
        backoffMultiplier: 2
        initialInterval: 1s
        maxInterval: 60s
        retryableErrors:
          - PAYMENT_GATEWAY_TIMEOUT
          - PAYMENT_GATEWAY_UNAVAILABLE
          - INSUFFICIENT_FUNDS_RETRY
          
    - name: confirmOrder
      service: order-service
      command:
        name: ConfirmOrder
        parameters:
          orderId: "{$.ctx.createOrder.orderId}"
      compensate:
        service: order-service
        command:
          name: MarkOrderFailed
          parameters:
            orderId: "{$.ctx.createOrder.orderId}"
            reason: "Saga compensation"
      onSuccess: sendNotification
      onError:
        then: compensateFromStep
        compensationOrder: [processPayment, reserveInventory, createOrder]
      timeout: 10s
      
    - name: sendNotification
      service: notification-service
      command:
        name: SendOrderConfirmation
        parameters:
          orderId: "{$.ctx.createOrder.orderId}"
          customerEmail: "{$.ctx.confirmOrder.customerEmail}"
      compensate:
        service: notification-service
        command:
          name: VoidNotification
          parameters:
            notificationId: "{$.ctx.sendNotification.notificationId}"
      onSuccess: sagaComplete
      onError: sagaComplete  # Notifications failures are not critical
      timeout: 10s
      
  errorHandling:
    sagaError:
      strategy: compensate
      retryCompensation: true
      maxCompensationRetries: 3
      compensationTimeout: 60s
    unknownStateTimeout: 120s
    
  sagaStore:
    type: postgres
    connectionString: "${SAAGA_STORE_DB_URL}"
    tableName: saga_instances
    instanceTtl: 604800s  # 7 days
    
  endpoints:
    status: /saga/order-fulfillment/status/{sagaId}
    events: /saga/order-fulfillment/events
```

### 3.3 Saga vs 2PC Decision Matrix

| Criteria | 2PC | Saga |
|----------|-----|------|
| ACID compliance | Full ACID | Relaxed (no atomicity across services) |
| Blocking | Yes during commit | No, but compensating transactions |
| Latency | High (2 round trips to all participants) | Lower (parallel local transactions) |
| Scalability | Limited (all participants must be available) | High (services operate independently) |
| Consistency model | Strong consistency | Eventual consistency |
| Complexity | Low (protocol handles everything) | High (compensating logic required) |
| Failure handling | In-doubt transactions | Manual compensation |
| Best for | Short-duration transactions | Long-running business processes |
| Transaction scope | Single distributed unit | Multi-service workflows |

---

## 4. Clock Synchronization

### 4.1 Time in Distributed Systems

Distributed systems cannot rely on wall-clock time because:
- Clocks drift and skew between machines
- NTP synchronization has limited accuracy
- Leap seconds cause unexpected behavior
- Clock updates can go backward

### 4.2 Logical Clocks

**Lamport Timestamps**

```python
class LamportClock:
    def __init__(self):
        self.time = 0
        
    def tick(self):
        """Increment clock for local event"""
        self.time += 1
        return self.time
    
    def update(self, received_time):
        """Update clock when receiving message"""
        self.time = max(self.time, received_time) + 1
        return self.time
    
    def get(self):
        return self.time
    
    def compare(self, other):
        """Compare two Lamport timestamps"""
        if self.time < other:
            return -1
        elif self.time > other:
            return 1
        return 0

# Usage in message passing
def send_message(clock, message):
    clock.tick()
    return Message(payload=message, timestamp=clock.get())

def receive_message(clock, message):
    clock.update(message.timestamp)
    return clock.get()
```

**Vector Clocks**

Vector clocks track causality by maintaining a vector of timestamps:

```python
class VectorClock:
    def __init__(self, node_id, nodes):
        self.node_id = node_id
        self.clock = {node_id: 0 for node_id in nodes}
        
    def tick(self):
        """Increment local component for local event"""
        self.clock[self.node_id] += 1
        return dict(self.clock)
    
    def update(self, received_clock):
        """Merge with received vector clock"""
        for node, time in received_clock.items():
            self.clock[node] = max(self.clock.get(node, 0), time)
        self.clock[self.node_id] += 1
        return dict(self.clock)
    
    def happens_before(self, other_clock):
        """Check if self happens before other_clock"""
        self_less = any(
            self.clock.get(n, 0) <= other_clock.get(n, 0)
            for n in set(self.clock) | set(other_clock)
        )
        self_greater = any(
            self.clock.get(n, 0) > other_clock.get(n, 0)
            for n in set(self.clock) | set(other_clock)
        )
        return self_less and not self_greater
    
    def concurrent_with(self, other_clock):
        """Check if two clocks are concurrent (neither happens-before)"""
        return not self.happens_before(other_clock) and \
               not other_clock.happens_before(self.clock)
    
    def merge(self, other_clock):
        """Merge two vector clocks, taking max of each component"""
        all_nodes = set(self.clock.keys()) | set(other_clock.keys())
        merged = {
            n: max(self.clock.get(n, 0), other_clock.get(n, 0))
            for n in all_nodes
        }
        return merged

# Conflict detection with vector clocks
def detect_conflict(clock1, clock2):
    if clock1.concurrent_with(clock2):
        return ConflictDetected(
            causally_dependent=False,
            requires_merge=True,
            manual_resolution=True
        )
    return NoConflict()
```

### 4.3 Hybrid Logical Clocks (HLC)

HLC combines physical time with logical time:

```python
class HybridLogicalClock:
    def __init__(self):
        self.pt = 0  # Physical time (from NTP)
        self.lt = 0  # Logical time
        self.node_id = 0
        
    def tick(self):
        """Local event - increment logical time"""
        self.lt += 1
        return (self.pt, self.lt, self.node_id)
    
    def update(self, received_hlc):
        """Receive message with HLC timestamp"""
        recv_pt, recv_lt, recv_node = received_hlc
        
        # Update physical time if NTP sync provides new value
        self.pt = max(self.pt, recv_pt)
        
        if self.pt == recv_pt:
            self.lt = max(self.lt, recv_lt) + 1
        elif self.pt > recv_pt:
            self.lt += 1
        else:  # Should not happen with properly synced clocks
            self.pt = recv_pt
            self.lt = recv_lt + 1
            
        return (self.pt, self.lt, self.node_id)
    
    def to_wallclock(self):
        """Convert to approximate wall-clock time"""
        return datetime.fromtimestamp(self.pt / 1000.0)
    
    def compare(self, other):
        """Compare two HLC values"""
        if self.pt != other[0]:
            return self.pt - other[0]
        if self.lt != other[1]:
            return self.lt - other[1]
        return self.node_id - other[2]
```

### 4.4 TrueTime (Spanner-style)

TrueTime uses GPS and atomic clocks to bound clock uncertainty:

```python
from dataclasses import dataclass
from datetime import datetime
from typing import Optional

@dataclass
class TimeRange:
    """Represents a time interval between earliest and latest possible time"""
    earliest: datetime
    latest: datetime
    
    def contains(self, t: datetime) -> bool:
        return self.earliest <= t <= self.latest
    
    def midpoint(self) -> datetime:
        return self.earliest + (self.latest - self.earliest) / 2

class TrueTime:
    """
    TrueTime implementation concept. 
    Real implementations (Spanner) use specialized hardware.
    """
    def __init__(self, epsilon_ms: int = 10):
        self.epsilon_ms = epsilon_ms  # Maximum clock drift
    
    def now(self) -> TimeRange:
        """Return time interval with maximum error bound"""
        now = datetime.utcnow()
        epsilon = timedelta(milliseconds=self.epsilon_ms)
        return TimeRange(
            earliest=now - epsilon,
            latest=now + epsilon
        )
    
    def wait_for(self, target_time: TimeRange) -> None:
        """Block until we're confident we're past target time"""
        while True:
            current = self.now()
            if current.latest < target_time.earliest:
                # We're definitely before target
                sleep_duration = (target_time.earliest - current.latest).total_seconds()
                time.sleep(sleep_duration)
            elif current.earliest > target_time.latest:
                # We're definitely after target
                return
            else:
                # We're in the uncertainty interval
                # Wait until the uncertainty is resolved
                time.sleep(self.epsilon_ms / 1000.0)

# Using TrueTime for distributed transactions (Spanner-style)
def write_with_timestamp(true_time: TrueTime, data: dict) -> tuple:
    """
    Write data with TrueTime-based timestamp.
    Returns (commit_timestamp, data)
    """
    # Start the commit
    start_time = true_time.now()
    
    # ... perform write ...
    
    # Compute commit timestamp as after all reads
    commit_time = true_time.now()
    
    # Wait for commit timestamp to be definitely in the past
    true_time.wait_for(commit_time)
    
    return (commit_time.midpoint(), data)
```

### 4.5 NTP Configuration for Distributed Systems

```yaml
# NTP client configuration for distributed systems
ntp:
  servers:
    - server 0.pool.ntp.org
    - server 1.pool.ntp.org
    - server 2.pool.ntp.org
    - server 3.pool.ntp.org
    
  # Timing parameters
  driftfile: /var/lib/ntp/ntp.drift
  logfile: /var/log/ntp.log
  
  # Sync parameters
  minpoll: 4       # Minimum poll interval (16 seconds)
  maxpoll: 10      # Maximum poll interval (1024 seconds = ~17 min)
  iburst: true     # Burst sync on startup
  burst: false      # Continuous burst mode (use with caution)
  
  # Accuracy settings
  maxdist: 16       # Maximum distance for acceptable synchronization
  mindist: 0.01     # Minimum distance for step correction
  maxstep: 1000     # Maximum step size in seconds (0 = no limit)
  stepout: 0.128    # Step timeout in seconds
  
  # Security
  restrict:
    - restrict -4 default kod notrap nomodify nopeer noquery limited
    - restrict -6 default kod notrap nomodify nopeer noquery limited
    - restrict 127.0.0.1
    - restrict ::1
    
  # Authentication (if using symmetric key)
  trustedkey: [1, 2, 3]
  keys: /etc/ntp/ntp.keys
  trustedkey: 1
  
  # Monitoring
  statistics: loopstats peerstats clockstats
  filegen: loopstats type:day enable
  filegen: peerstats type:day enable
  filegen: clockstats type:day enable

# Kubernetes NTP daemonset for nodes needing time sync
apiVersion: apps/v1
kind: DaemonSet
metadata:
  name: ntp-sync
  namespace: platform
spec:
  selector:
    matchLabels:
      app: ntp-sync
  template:
    metadata:
      labels:
        app: ntp-sync
    spec:
      hostNetwork: true
      hostPID: true
      containers:
      - name: ntp
        image: alpine/ntp:3.17
        securityContext:
          privileged: true
        command:
        - /bin/sh
        - -c
        - |
          apk add --no-cache ntp
          ntpd -dn -p {{ range .Values.ntp.servers }}{{ . }} {{ end }}
        env:
        - name: POD_NAME
          valueFrom:
            fieldRef:
              fieldPath: metadata.name
        volumeMounts:
        - name: ntp-config
          mountPath: /etc/ntp.conf
      volumes:
      - name: ntp-config
        configMap:
          name: ntp-config
```

---

## 5. CRDT Patterns

### 5.1 CRDT Fundamentals

CRDTs (Conflict-free Replicated Data Types) enable eventual consistency without coordination.

**Two Types of CRDTs:**

1. **CmRDT (Commutative Replicated Data Types)**: Operations commute
2. **CvRDT (Convergent Replicated Data Types)**: State converges via merge

### 5.2 G-Counter (Grow-only Counter)

```python
from typing import Dict

class GCounter:
    """
    Grow-only counter that only increments.
    Converges to the sum of all node contributions.
    """
    
    def __init__(self, node_id: str):
        self.node_id = node_id
        self.counts: Dict[str, int] = {}
        
    def increment(self, amount: int = 1) -> 'GCounter':
        """Increment the local counter"""
        result = self.copy()
        result.counts[self.node_id] = self.counts.get(self.node_id, 0) + amount
        return result
    
    def merge(self, other: 'GCounter') -> 'GCounter':
        """Merge with another G-Counter (take max of each node)"""
        result = self.copy()
        for node_id, count in other.counts.items():
            result.counts[node_id] = max(
                result.counts.get(node_id, 0),
                count
            )
        return result
    
    def value(self) -> int:
        """Get the total counter value"""
        return sum(self.counts.values())
    
    def copy(self) -> 'GCounter':
        """Create a deep copy"""
        result = GCounter(self.node_id)
        result.counts = dict(self.counts)
        return result
    
    def compare(self, other: 'GCounter') -> int:
        """
        Compare two G-Counters:
        -1 if self < other
        0 if self == other  
        1 if self > other
        """
        self_total = self.value()
        other_total = other.value()
        if self_total < other_total:
            return -1
        elif self_total > other_total:
            return 1
        return 0
    
    def to_dict(self) -> Dict:
        return {'node_id': self.node_id, 'counts': dict(self.counts)}
    
    @classmethod
    def from_dict(cls, data: Dict) -> 'GCounter':
        counter = cls(data['node_id'])
        counter.counts = dict(data['counts'])
        return counter
```

### 5.3 PN-Counter (Positive-Negative Counter)

```python
from typing import Dict

class PNCounter:
    """
    Counter that can both increment and decrement.
    Uses two G-Counters: one for increments, one for decrements.
    """
    
    def __init__(self, node_id: str):
        self.node_id = node_id
        self.positive = GCounter(node_id)  # Tracks increments
        self.negative = GCounter(node_id)  # Tracks decrements
        
    def increment(self, amount: int = 1) -> 'PNCounter':
        result = self.copy()
        result.positive = result.positive.increment(amount)
        return result
    
    def decrement(self, amount: int = 1) -> 'PNCounter':
        result = self.copy()
        result.negative = result.negative.increment(amount)
        return result
    
    def merge(self, other: 'PNCounter') -> 'PNCounter':
        """Merge two PN-Counters"""
        result = self.copy()
        result.positive = result.positive.merge(other.positive)
        result.negative = result.negative.merge(other.negative)
        return result
    
    def value(self) -> int:
        """Get current value: sum of positive minus negative"""
        return self.positive.value() - self.negative.value()
    
    def copy(self) -> 'PNCounter':
        result = PNCounter(self.node_id)
        result.positive = self.positive.copy()
        result.negative = self.negative.copy()
        return result
    
    def to_dict(self) -> Dict:
        return {
            'node_id': self.node_id,
            'positive': self.positive.to_dict(),
            'negative': self.negative.to_dict()
        }
```

### 5.4 LWW-Register (Last-Write-Wins Register)

```python
from typing import Optional
from datetime import datetime

class LWWRegister:
    """
    Last-Write-Wins Register.
    On conflict, the value with the higher timestamp wins.
    """
    
    def __init__(self, node_id: str):
        self.node_id = node_id
        self.value: Optional[any] = None
        self.timestamp: float = 0.0
        
    def set(self, value: any, timestamp: Optional[float] = None) -> 'LWWRegister':
        """Set a new value with timestamp"""
        if timestamp is None:
            timestamp = datetime.utcnow().timestamp()
        result = self.copy()
        result.value = value
        result.timestamp = timestamp
        return result
    
    def merge(self, other: 'LWWRegister') -> 'LWWRegister':
        """Merge with another register - higher timestamp wins"""
        if self.timestamp > other.timestamp:
            return self.copy()
        return other.copy()
    
    def copy(self) -> 'LWWRegister':
        result = LWWRegister(self.node_id)
        result.value = self.value
        result.timestamp = self.timestamp
        return result
    
    def to_dict(self) -> Dict:
        return {
            'node_id': self.node_id,
            'value': self.value,
            'timestamp': self.timestamp
        }
```

### 5.5 OR-Set (Observed-Remove Set)

```python
from typing import Dict, Set, Tuple

class ORObject:
    """Single item in an OR-Set with unique tag"""
    def __init__(self, value: any, tag: str):
        self.value = value
        self.tag = tag

class ORSet:
    """
    Observed-Remove Set.
    Elements are added with unique tags.
    Elements are removed by tag, not by value.
    """
    
    def __init__(self, node_id: str):
        self.node_id = node_id
        self.adds: Dict[str, ORObject] = {}  # tag -> value
        self.removes: Set[str] = set()       # tags that have been removed
        
    def add(self, value: any, tag: Optional[str] = None) -> 'ORSet':
        """Add an element with a unique tag"""
        if tag is None:
            tag = f"{self.node_id}:{datetime.utcnow().timestamp()}"
        
        result = self.copy()
        result.adds[tag] = ORObject(value, tag)
        return result
    
    def remove(self, value: any) -> 'ORSet':
        """Remove all elements with this value"""
        result = self.copy()
        tags_to_remove = [
            tag for tag, obj in self.adds.items()
            if obj.value == value and tag not in self.removes
        ]
        result.removes.update(tags_to_remove)
        return result
    
    def remove_tag(self, tag: str) -> 'ORSet':
        """Remove by specific tag"""
        result = self.copy()
        if tag in result.adds:
            result.removes.add(tag)
        return result
    
    def merge(self, other: 'ORSet') -> 'ORSet':
        """
        Merge two OR-Sets.
        Union of adds, intersection of removes.
        """
        result = self.copy()
        
        # Merge adds (union)
        for tag, obj in other.adds.items():
            if tag not in result.removes:
                result.adds[tag] = obj
                
        # Merge removes (union)
        result.removes.update(other.removes)
        
        return result
    
    def query(self, value: any) -> bool:
        """Check if a value is in the set"""
        return any(
            obj.value == value and tag not in self.removes
            for tag, obj in self.adds.items()
        )
    
    def get(self) -> Set[any]:
        """Get all current values"""
        return {
            obj.value for tag, obj in self.adds.items()
            if tag not in self.removes
        }
    
    def copy(self) -> 'ORSet':
        result = ORSet(self.node_id)
        result.adds = dict(self.adds)
        result.removes = set(self.removes)
        return result
```

### 5.6 CRDT Selection Guide

| Use Case | CRDT Type | Rationale |
|----------|-----------|-----------|
| Like/reaction counts | G-Counter / PN-Counter | Only grows, commutative |
| User session data | LWW-Register | Last update wins |
| Shopping cart | OR-Set | Add/remove semantics |
| Document editing | RGA (Replicated Growable Array) | Ordered sequence |
| Distributed rate limiting | Sliding Window Counter | Time-based sliding window |
| Distributed cache | LWW-Map | Map with last-write-wins per key |
| Set membership | 2P-Set | Add-only then remove-only phases |
| Configuration flags | LWW-Register | Simple on/off with last writer wins |

### 5.7 CRDT Configuration in Production

```yaml
# CRDT-based distributed data store configuration
crdt:
  # Global CRDT store settings
  store:
    name: crdt-store
    nodes:
      - host: crdt-node-0.platform.svc.cluster.local
        port: 9090
      - host: crdt-node-1.platform.svc.cluster.local
        port: 9090
      - host: crdt-node-2.platform.svc.cluster.local
        port: 9090
    
    # Consistency settings
    consistency:
      read_repair_chance: 0.9  # 90% chance of read repair
      stale_read_threshold: 5s # Serve stale reads if within 5s
      
    # Sync settings
    sync:
      anti_entropy_interval: 30s
      merkle_tree_sync: true
      merkle_tree_depth: 16
      
    # Serialization
    serialization: protobuf
    compression: lz4
    
  # Counter instances
  counters:
    user_likes:
      type: pn_counter
      nodes:
        - user-like-counter-0
        - user-like-counter-1
        
    product_views:
      type: gc_counter
      nodes:
        - view-counter-0
        - view-counter-1
        
    rate_limiting:
      type: sliding_window_counter
      window_size: 60s
      buckets: 60
      
  # Register instances
  registers:
    user_preferences:
      type: lww_register
      default_timestamp_source: system
      clock_type: hybrid  # Options: lamport, vector, hybrid
      
    feature_flags:
      type: lww_register
      default_timestamp_source: system
      
  # Set instances
  sets:
    user_permissions:
      type: or_set
      
    product_tags:
      type: or_set
```

---

## 6. Configuration Specifications

### 6.1 Distributed Lock Configuration

```yaml
# Distributed lock using etcd
distributed_lock:
  etcd:
    endpoints:
      - https://etcd-0.platform.svc.cluster.local:2379
      - https://etcd-1.platform.svc.cluster.local:2379
      - https://etcd-2.platform.svc.cluster.local:2379
    dial_timeout: 5s
    call_timeout: 10s
    keepalive_time: 10s
    keepalive_timeout: 30s
    max_call_send_msg_size: 2097152
    max_call_recv_msg_size: 2097152
    
  lock_config:
    ttl: 30s
    session_timeout: 20s
    retry_count: 3
    retry_delay: 100ms
    retry_jitter: 0.2
    lock_order: fifo  # Options: fifo, random, priority
    
  lock_types:
    # Advisory lock for resource isolation
    resource_lock:
      ttl: 60s
      extensions_enabled: true
      extension_timeout: 30s
      extension_count: 5
      
    # Lease lock for leader election
    leader_election:
      ttl: 15s
      extensions_enabled: true
      extension_timeout: 5s
      extension_count: unlimited
      
    # Transaction lock for distributed transactions
    transaction_lock:
      ttl: 30s
      extensions_enabled: false
```

### 6.2 Service Discovery Configuration

```yaml
# Service discovery with Consul
service_discovery:
  consul:
    addresses:
      - consul-0.platform.svc.cluster.local:8500
      - consul-1.platform.svc.cluster.local:8500
      - consul-2.platform.svc.cluster.local:8500
    datacenter: us-east-1
    token: ""  # Use ACL token from environment
    enable_ssl: true
    ca_cert: /etc/consul/ca.pem
    client_cert: /etc/consul/client.pem
    client_key: /etc/consul/client-key.pem
    timeout: 5s
    
  service_definition:
    name: order-service
    id: order-service-{{.PodName}}
    tags:
      - production
      - v1.2.3
      - region-us-east
      - protocol-http
      - protocol-grpc
    meta:
      version: "1.2.3"
      team: orders
      domain: e-commerce
    port: 8080
    weights:
      passing: 10
      warning: 1
      
    checks:
      - name: health
        interval: 10s
        timeout: 5s
        method: GET
        path: /health/ready
        deregister_critical_service_after: 60s
        
  dns_config:
    enable_pagination: true
    allow_stale: true
    max_stale: 15s
    consistent: false
```

---

## 7. Decision Matrix

### 7.1 Consistency Model Selection

| Requirement | Recommended Model | Rationale |
|-------------|------------------|-----------|
| Financial transactions | Linearizable/Sequential | Consistency critical |
| Shopping cart | Eventual with causal | Can tolerate brief inconsistency |
| Social media likes | Eventual | Eventually consistent is acceptable |
| Inventory management | Strong consistency | Must prevent overselling |
| User profile | Read-your-writes | Session consistency important |
| CDN content | Eventual | High latency tolerance |
| Leaderboard scores | Eventual | Minor inconsistencies acceptable |
| Distributed locking | Linearizable | Lock integrity critical |

### 7.2 Consensus Algorithm Selection

| Criteria | Raft | Paxos | 2PC | Sagas |
|----------|------|-------|-----|-------|
| Latency tolerance | Medium | High | Low | Medium |
| Fault tolerance | High | High | Medium | High |
| Implementation complexity | Medium | High | Medium | High |
| Coordinator bottleneck | No | No | Yes | Optional |
| Block on failure | No | No | Yes | No |
| Best for | Config/leader election | Generic consensus | Short transactions | Long workflows |

### 7.3 Clock Selection

| Requirement | Clock Type | Accuracy | Overhead |
|-------------|-----------|----------|----------|
| Causality tracking | Vector clock | Perfect | High (O(n) storage) |
| Event ordering | Lamport timestamp | Perfect | Low (O(1) storage) |
| Approximate sync | NTP | 10-100ms | Low |
| Global ordering with uncertainty | Hybrid logical clock | Good | Medium |
| TrueTime bounds | GPS/Atomic | 7ms | High (special hardware) |

---

## 8. Failure Modes and Recovery

### 8.1 Network Partition Handling

```yaml
partition_handling:
  detection:
    timeout: 10s
    suspicion_multiplier: 2
    max_paranoia: 5
    check_interval: 1s
    
  behavior:
    when_partition_detected: close_quorum
    read_operations: stale_allowed  # Options: stale_allowed, unavailable
    write_operations: local_only  # Options: local_only, rejected
    allow_local_locks: true
    
  recovery:
    when_partition_healed: resync
    sync_strategy: anti_entropy  # Options: anti_entropy, full_state_transfer
    conflict_resolution: auto_merge  # Options: auto_merge, manual
    
  metrics:
    partition_count: true
    partition_duration: true
    split_vote_count: true
    missed_heartbeats: true
```

### 8.2 Failure Detection Configuration

```yaml
failure_detector:
  # SWIM-based failure detector (used in Consul, Cassandra)
  swim:
    protocol_period: 1s
    suspicion_timeout: 5s
    suspicion_max: 3
    suspicion_multiplier: 2
    
  # Phi Accrual failure detector (used in Akka, Cassandra)
  phi_accrual:
    threshold: 8
    max_sample_size: 1000
    min_std_deviation: 100ms
    acceptable_heartbeat_pause: 2s
    first_heartbeat_estimate: 1s
    
  # Eddie configurables
  eddie:
    heartbeat_interval: 1s
    timeout: 5s
    max_failures: 3
    cleanup_interval: 10s
    
  # Cloud-specific considerations
  cloud_provider_factors:
    aws:
      az_network_latency: 1-5ms
      region_network_latency: 50-100ms
      instance_failure_rate: 0.1%
    gcp:
      zone_network_latency: 1-2ms
      region_network_latency: 10-50ms
      instance_failure_rate: 0.05%
```

### 8.3 Specific Failure Mode Recovery Procedures

**Split-Brain Recovery**

```
Error: "Multiple leaders detected in cluster"
Cause: Network partition caused multiple nodes to believe they're the leader

Recovery Steps:
1. Stop all write operations
2. Identify the partition with majority (quorum)
3. Promote majority partition's leader to canonical leader
4. Replay logs on minority partition nodes to catch up
5. Merge divergent states using configured resolution policy
6. Resume normal operations
```

**Lost Update Recovery**

```
Error: "Concurrent modification detected on key orders:1234"
Cause: Two nodes updated the same key without coordination

Recovery Options (choose based on policy):
1. LWW: Accept highest timestamp value
2. Merge: Combine both values if possible
3. Manual: Flag for human resolution
4. Abort: Reject both, require retry
```

**In-Doubt Transaction Recovery (2PC)**

```
Error: "Transaction TX-12345 in prepared state after coordinator crash"
Cause: Coordinator crashed between prepare and commit phases

Recovery Steps:
1. Query coordinator log for transaction state
2. If COMMIT found: Complete commit on all participants
3. If ABORT found: Complete rollback on all participants  
4. If nothing found: Default to rollback after timeout
5. Log resolution for audit trail
```

---

## 9. Production Implementation Guide

### 9.1 Quorum Configuration

```yaml
# Distributed system quorum configuration
quorum:
  # For N nodes, configure for fault tolerance
  cluster_sizes:
    small:
      nodes: 3
      quorum_size: 2  # N/2 + 1
      fault_tolerance: 1
      
    medium:
      nodes: 5
      quorum_size: 3
      fault_tolerance: 2
      
    large:
      nodes: 7
      quorum_size: 4
      fault_tolerance: 3
      
  # Read/write quorum settings
  read_write_quorum:
    strong_consistency:
      read_quorum: QUORUM  # (N/2) + 1
      write_quorum: QUORUM
      read_repair: true
      
    eventual_consistency:
      read_quorum: ONE
      write_quorum: ALL
      read_repair: true
      
    fast_consistency:
      read_quorum: LOCAL_QUORUM
      write_quorum: LOCAL_QUORUM
      global_quorum_for_writes: true
```

### 9.2 Observability for Distributed Systems

```yaml
# Distributed tracing configuration
tracing:
  # OpenTelemetry configuration
  otel:
    exporter:
      type: otlp  # Options: otlp, jaeger, zipkin, data-dog
      endpoint: https://otel-collector.platform.svc.cluster.local:4317
      insecure: false
      timeout: 10s
      retry:
        max_attempts: 3
        initial_backoff: 1s
        max_backoff: 30s
        
    sampling:
      type: tail  # Options: always_on, always_off, trace_id_ratio, tail
      ratio: 0.1  # 10% sampling rate
      parent_based: true
      targets:
        - name: high_value_operations
          type: always_on
        - name: health_checks
          type: always_off
          
  # Baggage propagation
  baggage:
    enabled: true
    keys:
      - tenant_id
      - user_id
      - correlation_id
      - session_id
      
  # Service Mesh tracing
  service_mesh:
    istio:
      tracing:
        sampling: 10%
        lightstep: false
        datadog: false
        zipkin: false
        opentracing:
          enabled: true
        jaeger:
          enabled: true
```

---

## 10. References

### Fundamental Theory

- [CAP Twelve Years Later: How the "Rules" Have Changed](https://www.infoq.com/articles/cap-twelve-years-later/) - Eric Brewer
- [Perspectives on the CAP Theorem](https://groups.csail.mit.edu/tds/papers/Gilbert/Brewer2.pdf) - Gilbert & Lynch
- [A Critique of the CAP Theorem](https://arxiv.org/abs/1509.05393) - Kleppmann
- [PACELC: A Better Primitive for Consistent Distributed Systems](https://jsn.github.io/blog/2019/01/20/pacels-theorem-for-distributed-systems.html)

### Consensus Algorithms

- [In Search of an Understandable Consensus Algorithm](https://raft.github.io/raft.pdf) - Ongaro & Ousterhout (Raft paper)
- [The Paxos Made Simple paper](https://lamport.azurewebsites.net/pubs/paxos-simple.pdf) - Lamport
- [Multi-Paxos Made Simple](https://jsn.github.io/blog/2019/01/15/multi-paxos-made-simple/)
- [Raft Refloated](https://arxiv.org/abs/1804.04019) - Howard et al.
- [Zab: A Simple Total Order Broadcast Protocol](https://zookeeper.apache.org/doc/r3.5.5/zookeeperInternals.html)

### Distributed Transactions

- [Sagas](https://www.cs.cornell.edu/andru/cs711/2002fa/plus/saga.pdf) - Hector Garcia-Molina
- [Using Sagas to Maintain Data Consistency](https://www.datastax.com/blog/using-sagas-maintain-data-consistency)
- [Large-scale Incremental Processing Using Distributed Transactions](https://www.usenix.org/legacy/events/osdi10/tech/full_papers/Peng.pdf)

### CRDT

- [A comprehensive study of Convergent and Commutative Replicated Data Types](https://hal.inria.fr/file/index/docid/555588/filename/techreport.pdf) - Shapiro et al.
- [Conflict-free Replicated Data Types (CRDT)](https://crdt.tech/)
- [Delta State Replicated Data Types](https://arxiv.org/abs/1603.01529) -有效性

### Clock Synchronization

- [Time, Clocks, and Ordering of Events in a Distributed System](https://lamport.azurewebsites.net/pubs/time-clocks.pdf) - Lamport
- [Hybrid Logical Clocks](https://cse.buffalo.edu/tech-reports/2014-04.pdf) - Kulkarni et al.
- [Spanner: Google's Globally Distributed Database](https://research.google/pubs/pub39966/)
- [TrueTime API Reference](https://cloud.google.com/spanner/docs/reference/latest/spanner/time#method)

### Production Reference

- [etcd Documentation](https://etcd.io/docs/latest/)
- [Consul Documentation](https://developer.hashicorp.com/consul/docs)
- [FoundationDB Documentation](https://apple.github.io/foundationdb/)
- [CockroachDB Architecture](https://www.cockroachlabs.com/docs/v23.1/architecture/overview)