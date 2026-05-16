# CONCURRENCY.md - Concurrency & Parallelism Architecture (DENSE)

**Authority:** guidance (concurrency patterns, async discipline, and coordination models)
**Layer:** Guides
**Binding:** No
**Scope:** concurrency models, async patterns, background task discipline
**Non-goals:** language-specific runtime details, OS-level threading

---

## 1. Concurrency Models

### 1.1 Model Comparison Matrix

| Model | Pros | Cons | Use When |
|-------|------|------|----------|
| **Shared memory** | Fast, low overhead | Race conditions, deadlocks | Hot paths, read-heavy workloads |
| **Message passing** | Safe, composable | Overhead, channel complexity | Distributed state, coordination |
| **Actor model** | Isolated state, fault tolerant | Complexity, debugging | Distributed systems, agent loops |
| **CSP (channels)** | Explicit coordination | Channel management | Pipeline processing, fan-out/fan-in |

### 1.2 Async vs Threads Decision Matrix

| Factor | Threads | Async |
|--------|---------|-------|
| **CPU-bound work** | Good (GIL-free languages) | Poor (blocks executor) |
| **I/O-bound work** | Okay (blocked threads) | Excellent (non-blocking) |
| **Latency requirements** | Variable | Predictable |
| **Concurrency level** | 100s of connections | 10000s of connections |
| **Complexity** | Simple for basic use | Steeper learning curve |
| **Stack size** | 1-8 MB per thread | 0.5-2 KB per task |
| **Context switch** | ~1-2 μs | ~0.1-0.5 μs |

### 1.3 Production Mindset
Concurrency is one of the highest-leverage and highest-risk categories of engineering decisions:

- **Sequential first:** Do not reach for concurrent architectures until the sequential baseline is exhausted. The simplest correct program is single-threaded. Concurrency is justified by measured need, not anticipated scale.
- **Coordination is the bottleneck:** Amdahl's Law is a hard limit. If 10% of a workload is sequential, no amount of parallelism yields more than 10× improvement. Design to minimize the sequential fraction, and be explicit about where it lives.
- **Blast radius isolation:** A concurrency bug — deadlock, live-lock, data race — can bring down an entire process or starve a thread pool. Isolate concurrent subsystems behind clear boundaries so failures cannot cascade.
- **Backpressure is a correctness property:** A system that cannot say "no" when overloaded is not production-ready. Every concurrent queue must be bounded. Unbounded queues are memory leaks with a delayed fuse.
- **Immutability eliminates the problem class:** Shared mutable state is the root cause of most concurrency bugs. Prefer immutable data, message passing, and copy-on-write semantics. When mutable state is unavoidable, make lock discipline explicit and reviewed.
- **Explicit state machines over ad-hoc coordination:** Complex concurrent workflows modeled with boolean flags and informal protocols will contain bugs that cannot be reproduced or proven correct. Model them as explicit state machines with defined transitions.
- **Lock-free is not "free":** Lock-free data structures are expert territory. Unless implementing a low-level primitive where profiling justifies it, lock-free code introduces correctness hazards that testing rarely catches. Use well-tested library implementations.
- **Async is not free either:** Async runtimes have scheduling overhead. For CPU-bound work, async adds overhead without benefit; use dedicated thread pools. Watch stack sizes, allocation rates, and wake-up patterns under load.

---

## 2. Async Discipline

### 2.1 Lock Hygiene Rules

```rust
// Rule 1: Never hold locks across await points
// WRONG: Lock held during network I/O
async fn bad_example(mutex: Arc<Mutex<u64>>, client: &HttpClient) -> Result<()> {
    let value = {
        let guard = mutex.lock().await;  // Lock acquired
        *guard
    };  // Lock dropped here (before await)
    
    let response = client.get("/api").await?;  // await happens after lock
    Ok(())
}

// WRONG: Lock held across await
async fn wrong_example(mutex: Arc<Mutex<u64>>, client: &HttpClient) -> Result<()> {
    let mut guard = mutex.lock().await;
    let response = client.get("/api").await?;  // DANGER: lock held across await
    *guard += 1;
    drop(guard);
    Ok(())
}

// RIGHT: Short-lived lock scope
async fn good_example(mutex: Arc<Mutex<u64>>, client: &HttpClient) -> Result<()> {
    let value = {
        let guard = mutex.lock().await;
        *guard  // Lock dropped here
    };
    
    let response = client.get("/api").await?;  // No lock during I/O
    Ok(())
}

// Rule 2: Use RwLock for read-heavy workloads
async fn read_pattern(rwlock: Arc<RwLock<Config>>) -> Result<String> {
    let config = rwlock.read().await;  // Multiple readers allowed
    Ok(config.endpoint.clone())
}

async fn write_pattern(rwlock: Arc<RwLock<Config>>, new_config: Config) -> Result<()> {
    let mut config = rwlock.write().await;  // Exclusive access
    *config = new_config;
    Ok(())
}
```

### 2.2 Cancellation Safety

```rust
use tokio::sync::CancellationToken;

pub struct CancellationScope {
    token: CancellationToken,
    children: Vec<CancellationToken>,
}

impl CancellationScope {
    pub fn new() -> Self {
        Self {
            token: CancellationToken::new(),
            children: Vec::new(),
        }
    }
    
    pub fn child_token(&mut self) -> CancellationToken {
        let child = self.token.child_token();
        self.children.push(child.clone());
        child
    }
    
    pub async fn run<F, T>(&self, future: F) -> Result<T,Cancelled>
    where
        F: Future<Output = T>,
    {
        tokio::select! {
            result = future => Ok(result),
            _ = self.token.cancelled() => Err(Cancelled),
        }
    }
    
    pub fn cancel(&self) {
        self.token.cancel();
    }
}

// Using with_timeout for deadline
async fn with_deadline<F, T>(
    future: F,
    duration: Duration,
) -> Result<T, TimeoutError>
where
    F: Future<Output = T>,
{
    tokio::time::timeout(duration, future).await?
}

// Composing cancellation with cleanup
async fn cancellable_operation(
    token: CancellationToken,
) -> Result<()> {
    let _guard = token.drop_guard();
    
    // Operation that can be cancelled
    loop {
        tokio::select! {
            result = do_work() => return result,
            _ = token.cancelled() => {
                // Cleanup before exit
                cleanup().await?;
                return Ok(());
            }
        }
    }
}
```

### 2.3 Error Handling in Async Context

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AsyncError {
    #[error("Operation timed out")]
    Timeout,
    
    #[error("Operation was cancelled")]
    Cancelled,
    
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, AsyncError>;

// Propagating errors through async layers
async fn top_level() -> Result<()> {
    let result = middle_layer().await?;
    Ok(())
}

async fn middle_layer() -> Result<()> {
    let result = bottom_layer().await?;
    Ok(result)
}

async fn bottom_layer() -> Result<()> {
    // Return concrete error type
    Err(AsyncError::ConnectionFailed("server unreachable".to_string()))
}

// Using anyhow for error propagation
pub type BoxError = Box<dyn std::error::Error + Send + Sync>;

async fn with_anyhow() -> anyhow::Result<()> {
    let result = may_fail().await?;
    Ok(result)
}

async fn may_fail() -> anyhow::Result<()> {
    // ...
    Ok(())
}
```

---

## 3. Background Task Discipline

### 3.1 Spawned Task Management

```rust
use tokio::task::{JoinSet, AbortHandle};
use std::sync::Arc;
use std::collections::HashMap;

pub struct TaskRegistry {
    tasks: Arc<tokio::sync::RwLock<HashMap<String, AbortHandle>>>,
}

impl TaskRegistry {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn spawn<F>(
        &self,
        name: String,
        future: F,
    ) -> Result<(), TaskError>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let handle = tokio::spawn(async move {
            if let Err(e) = std::panic::catch_unwind(future).await {
                tracing::error!(task = %name, error = ?e, "Task panicked");
            }
        }).abort_handle();
        
        let mut tasks = self.tasks.write().await;
        
        // Cancel existing task with same name
        if let Some(existing) = tasks.insert(name.clone(), handle) {
            existing.abort();
        }
        
        Ok(())
    }
    
    pub async fn cancel(&self, name: &str) -> bool {
        let mut tasks = self.tasks.write().await;
        if let Some(handle) = tasks.remove(name) {
            handle.abort();
            true
        } else {
            false
        }
    }
    
    pub async fn cancel_all(&self) {
        let mut tasks = self.tasks.write().await;
        for (_, handle) in tasks.drain() {
            handle.abort();
        }
    }
    
    pub async fn is_running(&self, name: &str) -> bool {
        let tasks = self.tasks.read().await;
        if let Some(handle) = tasks.get(name) {
            !handle.is_finished()
        } else {
            false
        }
    }
}
```

### 3.2 Graceful Shutdown

```rust
use tokio::signal;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct GracefulShutdown {
    shutdown_requested: Arc<AtomicBool>,
    shutdown_complete: Arc<AtomicBool>,
}

impl GracefulShutdown {
    pub fn new() -> Self {
        Self {
            shutdown_requested: Arc::new(AtomicBool::new(false)),
            shutdown_complete: Arc::new(AtomicBool::new(false)),
        }
    }
    
    pub fn is_shutdown_requested(&self) -> bool {
        self.shutdown_requested.load(Ordering::SeqCst)
    }
    
    pub async fn wait_for_shutdown(&self) {
        signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
        self.shutdown_requested.store(true, Ordering::SeqCst);
    }
    
    pub fn set_shutdown_complete(&self) {
        self.shutdown_complete.store(true, Ordering::SeqCst);
    }
    
    pub async fn run_with_shutdown<F, Fut>(&self, f: F) -> Result<(), ShutdownError>
    where
        F: FnOnce(ShutdownHandle) -> Fut,
        Fut: Future<Output = Result<()>>,
    {
        let handle = ShutdownHandle {
            shutdown_requested: self.shutdown_requested.clone(),
        };
        
        tokio::select! {
            result = f(handle) => result,
            _ = self.wait_for_shutdown() => {
                tracing::info!("Shutdown signal received");
                Err(ShutdownError::ShutdownRequested)
            }
        }
    }
}

pub struct ShutdownHandle {
    shutdown_requested: Arc<AtomicBool>,
}

impl ShutdownHandle {
    pub fn is_shutdown_requested(&self) -> bool {
        self.shutdown_requested.load(Ordering::SeqCst)
    }
    
    pub fn check_and_yield(&self) -> Result<(), ShutdownError> {
        if self.is_shutdown_requested() {
            Err(ShutdownError::ShutdownRequested)
        } else {
            Ok(())
        }
    }
}

#[derive(Error, Debug)]
pub enum ShutdownError {
    #[error("Shutdown was requested")]
    ShutdownRequested,
}
```

### 3.3 Bounded Channel Configuration

```rust
use tokio::sync::mpsc;

pub struct BoundedChannel<T> {
    sender: mpsc::Sender<T>,
    receiver: Option<mpsc::Receiver<T>>,
}

impl<T> BoundedChannel<T> {
    pub fn new(capacity: usize) -> Self {
        let (sender, receiver) = mpsc::channel(capacity);
        Self {
            sender,
            receiver: Some(receiver),
        }
    }
    
    pub async fn send(&self, value: T) -> Result<(), SendError<T>> {
        self.sender.send(value).await
    }
    
    pub fn receiver(&mut self) -> Option<mpsc::Receiver<T>> {
        self.receiver.take()
    }
}

// Channel size guidelines:
// - CPU-bound work: small buffer (1-10)
// - I/O-bound work: medium buffer (100-1000)
// - Batch processing: large buffer or unbounded (with backpressure)
```

---

## 4. Coordination Patterns

### 4.1 Semaphore for Concurrency Limiting

```rust
use tokio::sync::Semaphore;

pub struct ConcurrencyLimiter {
    semaphore: Arc<Semaphore>,
    permits: usize,
}

impl ConcurrencyLimiter {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            permits: max_concurrent,
        }
    }
    
    pub async fn acquire(&self) -> ConcurrencyPermit<'_> {
        ConcurrencyPermit {
            permit: self.semaphore.acquire().await.unwrap(),
        }
    }
    
    pub fn available_permits(&self) -> usize {
        self.semaphore.available_permits()
    }
    
    pub async fn run_concurrent<F, T, E>(&self, futures: Vec<F>) -> Vec<Result<T, E>>
    where
        F: Future<Output = Result<T, E>>,
    {
        let mut handles = Vec::new();
        
        for future in futures {
            let sem = self.semaphore.clone();
            handles.push(tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                future.await
            }));
        }
        
        let mut results = Vec::new();
        for handle in handles {
            results.push(handle.await.unwrap());
        }
        results
    }
}

pub struct ConcurrencyPermit<'a> {
    permit: tokio::sync::SemaphorePermit<'a>,
}

// Usage
async fn process_items(items: Vec<Item>) -> Results {
    let limiter = ConcurrencyLimiter::new(10);  // Max 10 concurrent
    
    let futures: Vec<_> = items
        .into_iter()
        .map(|item| {
            let limiter = ConcurrencyLimiter::new(10);
            async move {
                let _permit = limiter.acquire().await;
                process_item(item).await
            }
        })
        .collect();
    
    futures::future::join_all(futures).await
}
```

### 4.2 Barrier Synchronization

```rust
use tokio::sync::Barrier;

pub struct PhaseBarrier {
    name: String,
    barrier: Arc<Barrier>,
    phase: Arc<std::sync::atomic::AtomicUsize>,
}

impl PhaseBarrier {
    pub fn new(name: String, participants: usize) -> Self {
        Self {
            name,
            barrier: Arc::new(Barrier::new(participants)),
            phase: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }
    
    pub async fn wait_phase(&self, expected_phase: usize) {
        let current_phase = self.phase.load(std::sync::atomic::Ordering::SeqCst);
        
        if current_phase != expected_phase {
            // Wait for all parties to reach this phase
            self.barrier.wait().await;
            
            // Increment phase
            self.phase.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
    }
}

// Barrier usage example
async fn distributed_join(workers: usize) {
    let barrier = Arc::new(Barrier::new(workers));
    
    let handles: Vec<_> = (0..workers)
        .map(|i| {
            let barrier = barrier.clone();
            tokio::spawn(async move {
                tracing::info!("Worker {} - Phase 1 start", i);
                // Do phase 1 work
                barrier.wait().await;
                tracing::info!("Worker {} - Phase 1 complete", i);
                
                tracing::info!("Worker {} - Phase 2 start", i);
                // Do phase 2 work
                barrier.wait().await;
                tracing::info!("Worker {} - Phase 2 complete", i);
            })
        })
        .collect();
    
    for handle in handles {
        handle.await.unwrap();
    }
}
```

### 4.3 Once Cell / Lazy Initialization

```rust
use tokio::sync::OnceCell;

pub struct Lazy<T> {
    cell: OnceCell<T>,
    init: std::sync::Arc<std::sync::Mutex<Option<T>>>,
}

impl<T> Lazy<T> {
    pub fn new() -> Self {
        Self {
            cell: OnceCell::new(),
            init: std::sync::Arc::new(std::sync::Mutex::new(None)),
        }
    }
    
    pub async fn get<F, Fut>(&self, init: F) -> &T
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = T>,
    {
        self.cell
            .get_or_init(async { init().await })
            .await
    }
    
    pub fn get_blocking<F>(&self, init: F) -> &T
    where
        F: FnOnce() -> T,
    {
        self.cell.get_or_init(|| init())
    }
}

// Usage with async initialization
static DATABASE: Lazy<DatabaseConnection> = Lazy::new();

async fn get_database() -> &'static DatabaseConnection {
    DATABASE.get(|| async {
        DatabaseConnection::connect("postgres://...").await
    }).await
}
```

---

## 5. Testing Concurrent Code

### 5.1 Race Condition Testing

```rust
#[cfg(test)]
mod tests {
    use tokio::sync::Mutex;
    use std::sync::atomic::{AtomicU64, Ordering};
    
    #[tokio::test]
    async fn test_concurrent_increment() {
        let counter = Arc::new(AtomicU64::new(0));
        let num_tasks = 1000;
        let increments_per_task = 100;
        
        let handles: Vec<_> = (0..num_tasks)
            .map(|_| {
                let counter = counter.clone();
                tokio::spawn(async move {
                    for _ in 0..increments_per_task {
                        counter.fetch_add(1, Ordering::SeqCst);
                    }
                })
            })
            .collect();
        
        for handle in handles {
            handle.await.unwrap();
        }
        
        let final_count = counter.load(Ordering::SeqCst);
        assert_eq!(final_count, num_tasks * increments_per_task);
    }
    
    #[tokio::test]
    async fn test_mutex_exclusive_access() {
        let data = Arc::new(Mutex::new(0u64));
        let num_tasks = 100;
        let increments_per_task = 100;
        
        let handles: Vec<_> = (0..num_tasks)
            .map(|_| {
                let data = data.clone();
                tokio::spawn(async move {
                    for _ in 0..increments_per_task {
                        let mut guard = data.lock().await;
                        *guard += 1;
                    }
                })
            })
            .collect();
        
        for handle in handles {
            handle.await.unwrap();
        }
        
        let final_value = *data.lock().await;
        assert_eq!(final_value, num_tasks * increments_per_task);
    }
    
    #[tokio::test]
    async fn test_deadlock_prevention() {
        // Test that lock ordering is consistent
        let resource_a = Arc::new(Mutex::new(()));
        let resource_b = Arc::new(Mutex::new(()));
        
        // Task 1 acquires A then B
        let handle1 = {
            let a = resource_a.clone();
            let b = resource_b.clone();
            tokio::spawn(async move {
                let _a = a.lock().await;
                tokio::task::yield_now().await;
                let _b = b.lock().await;
            })
        };
        
        // Task 2 acquires A then B (same order - no deadlock)
        let handle2 = {
            let a = resource_a.clone();
            let b = resource_b.clone();
            tokio::spawn(async move {
                let _a = a.lock().await;
                tokio::task::yield_now().await;
                let _b = b.lock().await;
            })
        };
        
        // Both should complete without deadlock
        let (r1, r2) = tokio::join!(handle1, handle2);
        r1.unwrap();
        r2.unwrap();
    }
}
```

### 5.2 Stress Testing

```rust
pub struct StressTestConfig {
    pub num_iterations: usize,
    pub num_concurrent_tasks: usize,
    pub timeout: Duration,
}

pub async fn stress_test<F, Fut>(config: StressTestConfig, operation: F)
where
    F: Fn() -> Fut + Send + Sync,
    Fut: Future<Output = Result<()>> + Send,
{
    use tokio::task::JoinSet;
    
    let mut join_set = JoinSet::new();
    
    for iteration in 0..config.num_iterations {
        // Spawn concurrent tasks
        for _ in 0..config.num_concurrent_tasks {
            let op = operation();
            join_set.spawn(async move {
                let result = tokio::time::timeout(config.timeout, op).await;
                match result {
                    Ok(Ok(())) => Ok(()),
                    Ok(Err(e)) => Err(e),
                    Err(_) => Err(StressTestError::Timeout),
                }
            });
        }
        
        // Collect results
        while let Some(result) = join_set.join_next().await {
            if let Err(e) = result.unwrap() {
                panic!("Stress test failed: {:?}", e);
            }
        }
        
        if iteration % 100 == 0 {
            tracing::info!("Completed {} iterations", iteration);
        }
    }
}
```

---

## 6. Anti-Patterns

| Anti-Pattern | Specific Failure Mode | Prevention |
|--------------|----------------------|------------|
| **Locks held across await** | Deadlocks, contention | Short-lived lock scopes |
| **Unbounded channels** | Memory leak under load | Bounded channels with backpressure |
| **Silent spawn failures** | Invisible bugs, lost work | Log all errors from spawned tasks |
| **No timeouts on I/O** | Hung tasks, resource exhaustion | Timeout every external call |
| **Shared mutable state** | Race conditions | Message passing, immutable data |
| **Thread-per-request** | Resource exhaustion at scale | Thread pools with bounded concurrency |
| **Fire-and-forget async** | Lost errors, no cancellation | Always await or log result |
| **Blocking in async** | Starves executor | Use async equivalents, spawn_blocking |
| **Ignoring CancellationToken** | Lingering work | Propagate cancellation |
| **Over-synchronization** | Performance bottleneck | Minimize critical sections |

---

## Links

### Core Router
- `core/DECAPOD.md` - **Router and navigation charter (START HERE)**
- `core/ENGINEERING_EXCELLENCE.md` - Engineering standards

### Architecture (This Section)
- `architecture/ALGORITHMS.md` - Algorithm selection
- `architecture/CLOUD.md` - Cloud infrastructure patterns
- `architecture/OBSERVABILITY.md` - Monitoring and debugging

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
- `methodology/CONCURRENCY_PRACTICE.md` - Concurrency patterns