# PERFORMANCE.md - Performance Optimization Standards

**Authority:** guidance (comprehensive topic with exact specifications)
**Layer:** Architecture
**Binding:** No
**Scope:** Comprehensive topic coverage for pre-inference context

---

## 1. Profiling Techniques

### 1.1 Go Profiling

```go
// profiling/setup.go - Complete profiling setup

package profiling

import (
    "context"
    "fmt"
    "net/http"
    "net/http/pprof"
    "runtime"
    "time"

    "github.com/pkg/profile"
)

type Profiler struct {
    enabled  bool
    pprofDir string
    memRate  int
}

func NewProfiler() *Profiler {
    return &Profiler{
        enabled:  false,
        pprofDir: "/tmp/pprof",
        memRate:  4096, // bytes between samples
    }
}

func (p *Profiler) Start(mode profile.Mode) (func(), error) {
    if p.enabled {
        return func() {}, nil
    }

    p.enabled = true

    // Configure memory profiler
    runtime.MemProfileRate = p.memRate

    // Start CPU profiling
    stop, err := profile.Start(
        mode,
        profile.ProfilePath(p.pprofDir),
        profile.NoShutdownHook,
    )

    if err != nil {
        return nil, fmt.Errorf("failed to start profiler: %w", err)
    }

    return func() {
        stop()
        p.enabled = false
    }, nil
}

func (p *Profiler) ServeHTTP() {
    // CPU profiling
    http.HandleFunc("/debug/pprof/profile", pprof.Profile)

    // Heap profiling
    http.HandleFunc("/debug/pprof/heap", pprof.Handler("heap").ServeHTTP)

    // Goroutine profiling
    http.HandleFunc("/debug/pprof/goroutine", pprof.Handler("goroutine").ServeHTTP)

    // Threadcreate profiling
    http.HandleFunc("/debug/pprof/threadcreate", pprof.Handler("threadcreate").ServeHTTP)

    // Block profiling
    http.HandleFunc("/debug/pprof/block", pprof.Handler("block").ServeHTTP)

    // Mutex profiling
    http.HandleFunc("/debug/pprof/mutex", pprof.Handler("mutex").ServeHTTP)

    // Symbol lookup
    http.HandleFunc("/debug/pprof/symbol", pprof.Symbol)
}

// pprof commands:
// go tool pprof http://localhost:8080/debug/pprof/profile?seconds=30
// go tool pprof -png http://localhost:8080/debug/pprof/heap  # Generate PNG
// go tool pprof -svg http://localhost:8080/debug/pprof/heap  # Generate SVG
// go tool pprof http://localhost:8080/debug/pprof/heap       # Interactive
```

### 1.2 Python Profiling

```python
# profiling/setup.py - Python profiling configuration

import cProfile
import pstats
import yappi
import memory_profiler
import time
from contextlib import contextmanager
from functools import wraps
import logging

logger = logging.getLogger(__name__)


class ProfilerManager:
    def __init__(self, output_dir: str = "/tmp/profiles"):
        self.output_dir = output_dir
        self.enabled = False
        self._profiler = None
        
    def start(self, profiler_type: str = "yappi"):
        """Start profiling"""
        self.enabled = True
        
        if profiler_type == "yappi":
            # Yappi for multi-threaded profiling
            yappi.set_clock_type("cpu")
            yappi.start()
            self._profiler = "yappi"
            
        elif profiler_type == "cprofile":
            self._profiler = cProfile.Profile()
            self._profiler.enable()
            
        elif profiler_type == "memory":
            # Memory profiling via memory_profiler
            pass
            
    def stop(self, output_file: str = None):
        """Stop profiling and save results"""
        if not self.enabled:
            return
            
        self.enabled = False
        
        if self._profiler == "yappi":
            stats = yappi.get_func_stats()
            if output_file:
                stats.save(output_file, type="pstat")
            else:
                stats.print(20)
            yappi.stop()
            
        elif isinstance(self._profiler, cProfile.Profile):
            self._profiler.disable()
            if output_file:
                self._profiler.dump_stats(output_file)
            else:
                stats = pstats.Stats(self._profiler)
                stats.sort_stats("cumulative")
                stats.print_stats(20)


@contextmanager
def profile_context(name: str, profiler_type: str = "yappi"):
    """Context manager for profiling a code block"""
    manager = ProfilerManager()
    
    logger.info(f"Starting profile for: {name}")
    manager.start(profiler_type)
    
    start_time = time.time()
    
    try:
        yield manager
    finally:
        duration = time.time() - start_time
        logger.info(f"Profile completed for: {name} (took {duration:.2f}s)")
        manager.stop(f"/tmp/profiles/{name}.prof")


def profile_func(func):
    """Decorator for profiling a function"""
    @wraps(func)
    def wrapper(*args, **kwargs):
        profiler = ProfilerManager()
        profiler.start()
        
        try:
            result = func(*args, **kwargs)
            return result
        finally:
            profiler.stop(f"/tmp/profiles/{func.__name__}.prof")
            
    return wrapper


def memory_profile(func):
    """Decorator for memory profiling a function"""
    @wraps(func)
    def wrapper(*args, **kwargs):
        profiler = memory_profiler.Profile()
        
        profiler.enable()
        try:
            result = func(*args, **kwargs)
            return result
        finally:
            profiler.disable()
            
        # Print memory stats
        from io import StringIO
        stream = StringIO()
        memory_profiler.print_profile_stream(profiler, stream=stream)
        logger.info(f"Memory profile for {func.__name__}:\n{stream.getvalue()}")
        
    return wrapper


# Line-by-line profiling
def profile_lines(func):
    """Profile line-by-line execution"""
    @wraps(func)
    def wrapper(*args, **kwargs):
        from line_profiler import LineProfiler
        
        lp = LineProfiler()
        lp_wrapper = lp(func)
        
        result = lp_wrapper(*args, **kwargs)
        
        lp.print_stats()
        
        return result
        
    return wrapper
```

### 1.3 Node.js Profiling

```javascript
// profiling/setup.js - Node.js profiling

const { PerformanceObserver, performance } = require('perf_hooks');
const v8 = require('v8');
const fs = require('fs');
const path = require('path');

class ProfilerManager {
    constructor(options = {}) {
        this.outputDir = options.outputDir || '/tmp/profiles';
        this.enabled = false;
        
        // Ensure output directory exists
        if (!fs.existsSync(this.outputDir)) {
            fs.mkdirSync(this.outputDir, { recursive: true });
        }
    }

    startCPUProfile(name) {
        if (this.enabled) return;
        
        this.enabled = true;
        v8.startSampling();
        
        // Schedule profile dump
        this.cpuProfileName = name;
        this.cpuProfileStart = Date.now();
    }

    stopCPUProfile(name) {
        if (!this.enabled) return;
        
        v8.stopSampling();
        this.enabled = false;
        
        const filename = path.join(
            this.outputDir, 
            `${name}-${Date.now()}.cpuprofile`
        );
        
        const profile = v8.stopSampling();
        fs.writeFileSync(filename, JSON.stringify(profile));
        
        console.log(`CPU profile saved to: ${filename}`);
    }

    startMemoryTracking() {
        // Enable memory profiling
        if (global.gc) {
            global.gc(); // Run GC before starting
        }
        
        this.memorySnapshots = [];
        
        this.memoryInterval = setInterval(() => {
            if (global.gc) {
                global.gc();
            }
            
            const heapStats = v8.getHeapStatistics();
            this.memorySnapshots.push({
                timestamp: Date.now(),
                heapUsed: heapStats.used_heap_size,
                heapTotal: heapStats.total_heap_size,
                heapLimit: heapStats.heap_size_limit,
            });
        }, 5000);
    }

    stopMemoryTracking() {
        if (this.memoryInterval) {
            clearInterval(this.memoryInterval);
            this.memoryInterval = null;
        }
        
        return this.memorySnapshots;
    }

    takeHeapSnapshot(name) {
        const filename = path.join(
            this.outputDir, 
            `${name}-${Date.now()}.heapsnapshot`
        );
        
        const snapshot = v8.writeHeapSnapshot(filename);
        console.log(`Heap snapshot saved to: ${snapshot}`);
        
        return snapshot;
    }

    getHeapStatistics() {
        return v8.getHeapStatistics();
    }

    getSpaceStatistics() {
        return v8.getHeapSpaceStatistics();
    }
}

// Performance hooks for custom metrics
function setupPerformanceObservers() {
    const obs = new PerformanceObserver((items) => {
        items.getEntries().forEach(entry => {
            console.log('Performance entry:', {
                name: entry.name,
                duration: entry.duration,
                entryType: entry.entryType,
            });
        });
    });

    // Observe all performance events
    obs.observe({ entryTypes: ['measure', 'mark', 'navigation', 'resource'] });
}

// Custom timing helper
function measure(name, fn) {
    return async (...args) => {
        performance.mark(`${name}-start`);
        
        try {
            const result = await fn(...args);
            performance.mark(`${name}-end`);
            performance.measure(name, `${name}-start`, `${name}-end`);
            return result;
        } catch (error) {
            performance.mark(`${name}-error`);
            throw error;
        }
    };
}

// HTTP request timing middleware
function requestTimingMiddleware(req, res, next) {
    const start = process.hrtime.bigint();
    
    res.on('finish', () => {
        const end = process.hrtime.bigint();
        const durationMs = Number(end - start) / 1_000_000;
        
        console.log({
            method: req.method,
            url: req.url,
            status: res.statusCode,
            duration: `${durationMs.toFixed(2)}ms`,
        });
    });
    
    next();
}

module.exports = {
    ProfilerManager,
    setupPerformanceObservers,
    measure,
    requestTimingMiddleware,
};
```

## 2. Memory Optimization

### 2.1 Go Memory Management

```go
// memory/management.go - Go memory optimization patterns

package memory

import (
    "runtime"
    "runtime/debug"
    "sync"
    "time"
    "unsafe"
)

// Object pool for reducing allocations
type ObjectPool[T any] struct {
    pool sync.Pool
    new  func() *T
}

func NewObjectPool[T any](factory func() *T) *ObjectPool[T] {
    return &ObjectPool[T]{
        pool: sync.Pool{
            New: func() interface{} {
                return factory()
            },
        },
        new: factory,
    }
}

func (p *ObjectPool[T]) Get() *T {
    if val := p.pool.Get(); val != nil {
        return val.(*T)
    }
    return p.new()
}

func (p *ObjectPool[T]) Put(obj *T) {
    p.pool.Put(obj)
}

// Buffer pool for I/O operations
type BufferPool struct {
    sizes     []int
    pools     []*sync.Pool
    maxSize   int
}

func NewBufferPool(minSize, maxSize int, factor float64) *BufferPool {
    var sizes []int
    size := minSize
    
    for size < maxSize {
        sizes = append(sizes, size)
        size = int(float64(size) * factor)
    }
    
    pools := make([]*sync.Pool, len(sizes))
    for i, s := range sizes {
        sz := s
        pools[i] = &sync.Pool{
            New: func() interface{} {
                return make([]byte, sz)
            },
        }
    }
    
    return &BufferPool{
        sizes:   sizes,
        pools:   pools,
        maxSize: maxSize,
    }
}

func (p *BufferPool) Get(size int) []byte {
    for i, s := range p.sizes {
        if size <= s {
            return p.pools[i].Get().([]byte)[:size]
        }
    }
    return make([]byte, size)
}

func (p *BufferPool) Put(buf []byte) {
    for i, s := range p.sizes {
        if cap(buf) == s {
            p.pools[i].Put(buf[:cap(buf)])
            return
        }
    }
}

// Memory profiler with metrics
type MemoryProfiler struct {
    interval time.Duration
    stop     chan struct{}
    history  []MemorySnapshot
}

type MemorySnapshot struct {
    Timestamp  time.Time
    HeapAlloc  uint64
    HeapSys    uint64
    StackInuse uint64
    GCNum      uint32
    GCLatest   time.Time
}

func (m *MemoryProfiler) Start(interval time.Duration) {
    m.interval = interval
    m.stop = make(chan struct{})
    
    go m.collect()
}

func (m *MemoryProfiler) Stop() {
    if m.stop != nil {
        close(m.stop)
    }
}

func (m *MemoryProfiler) collect() {
    tick := time.NewTicker(m.interval)
    defer tick.Stop()
    
    for {
        select {
        case <-tick.C:
            m.record()
        case <-m.stop:
            return
        }
    }
}

func (m *MemoryProfiler) record() {
    var ms runtime.MemStats
    runtime.ReadMemStats(&ms)
    
    snapshot := MemorySnapshot{
        Timestamp:  time.Now(),
        HeapAlloc:   ms.HeapAlloc,
        HeapSys:     ms.HeapSys,
        StackInuse:  ms.StackInuse,
        GCNum:       ms.NumGC,
        GCLatest:    time.Unix(0, int64(ms.LastGC)),
    }
    
    m.history = append(m.history, snapshot)
    
    // Keep only last 1000 snapshots
    if len(m.history) > 1000 {
        m.history = m.history[len(m.history)-1000:]
    }
}

// GOGC tuning
func SetGOGC(percent int) {
    debug.SetGCPercent(percent)
}

func GetGOGC() int {
    return debug.ReadGCPercent()
}

// Preallocate slices for known capacity
func PreallocateSlice(size int) []byte {
    return make([]byte, 0, size)
}

// StringBuilder for string concatenation
func EfficientConcat(parts []string) string {
    var sb strings.Builder
    sb.Grow(len(parts) * 10) // Estimate size
    
    for _, part := range parts {
        sb.WriteString(part)
    }
    
    return sb.String()
}

// Memory-mapped files for large data
func MemoryMapFile(filename string) ([]byte, error) {
    f, err := os.Open(filename)
    if err != nil {
        return nil, err
    }
    defer f.Close()
    
    fi, err := f.Stat()
    if err != nil {
        return nil, err
    }
    
    return syscall.Mmap(
        int(f.Fd()),
        0,
        int(fi.Size()),
        syscall.PROT_READ,
        syscall.MAP_PRIVATE,
    )
}

// Cache with eviction
type Cache[K comparable, V any] struct {
    data     map[K]V
    maxSize  int
    mu       sync.RWMutex
    onEvict  func(K, V)
}

func NewCache[K comparable, V any](maxSize int, onEvict func(K, V)) *Cache[K, V] {
    return &Cache[K, V]{
        data:    make(map[K]V, maxSize),
        maxSize: maxSize,
        onEvict: onEvict,
    }
}

func (c *Cache[K, V]) Get(key K) (V, bool) {
    c.mu.RLock()
    defer c.mu.RUnlock()
    
    val, ok := c.data[key]
    return val, ok
}

func (c *Cache[K, V]) Set(key K, val V) {
    c.mu.Lock()
    defer c.mu.Unlock()
    
    if len(c.data) >= c.maxSize {
        // Evict oldest (simple FIFO, could use LRU)
        for k, v := range c.data {
            delete(c.data, k)
            if c.onEvict != nil {
                c.onEvict(k, v)
            }
            break
        }
    }
    
    c.data[key] = val
}
```

### 2.2 Memory Leak Prevention

```go
// memory/leak_prevention.go - Patterns to prevent memory leaks

package memory

import (
    "context"
    "runtime"
    "sync"
    "time"
)

// Context with cancellation to prevent goroutine leaks
func PreventGoroutineLeak() {
    ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
    defer cancel()
    
    done := make(chan struct{})
    
    go func() {
        // Long-running operation
        // Will be cancelled after 5 seconds
        select {
        case <-ctx.Done():
            // Clean up
        case <-done:
            // Normal completion
        }
    }()
}

// WaitGroup for tracking goroutine completion
func TrackGoroutines() {
    var wg sync.WaitGroup
    
    for i := 0; i < 10; i++ {
        wg.Add(1)
        go func(id int) {
            defer wg.Done()
            // Work
        }(i)
    }
    
    wg.Wait() // Block until all done
}

// Timer cleanup
func TimerCleanup() {
    timer := time.NewTimer(30 * time.Second)
    defer timer.Stop() // Always cleanup timers
    
    select {
    case <-timer.C:
        // Handle timeout
    case <-time.After(1 * time.Hour):
        // This would cause a leak if timer wasn't stopped
    }
}

// Resource cleanup pattern
type Resource struct {
    data []byte
}

func (r *Resource) Close() error {
    r.data = nil
    return nil
}

func UseResources() error {
    // Multi-resource cleanup
    f, err := os.Create("file.txt")
    if err != nil {
        return err
    }
    
    // Get connection
    conn, err := net.Dial("tcp", "localhost:8080")
    if err != nil {
        f.Close()
        return err
    }
    
    // Defer cleanup (LIFO order)
    defer conn.Close()
    defer f.Close()
    
    // Use resources...
    return nil
}

// Channel cleanup to prevent goroutine blocks
func ChannelCleanup() {
    ch := make(chan int, 100)
    
    // Producer
    go func() {
        for i := 0; i < 10; i++ {
            ch <- i
        }
        close(ch) // Always close channels
    }()
    
    // Consumer
    for val := range ch {
        // Process val
        _ = val
    }
}

// Map access pattern for concurrent access
func ConcurrentMapAccess() {
    var mu sync.RWMutex
    m := make(map[string]int)
    
    // Read
    mu.RLock()
    val := m["key"]
    mu.RUnlock()
    _ = val
    
    // Write
    mu.Lock()
    m["key"] = 42
    mu.Unlock()
}

// Periodic cleanup for caches
func StartPeriodicCleanup(cleanupFn func(), interval time.Duration) func() {
    stop := make(chan struct{})
    
    go func() {
        tick := time.NewTicker(interval)
        defer tick.Stop()
        
        for {
            select {
            case <-tick.C:
                cleanupFn()
            case <-stop:
                return
            }
        }
    }()
    
    return func() {
        close(stop)
    }
}
```

## 3. CPU Optimization

### 3.1 Goroutine Optimization

```go
// cpu/goroutine_optimization.go

package cpu

import (
    "runtime"
    "sync"
    "sync/atomic"
)

// Worker pool with bounded concurrency
type WorkerPool struct {
    work    chan func() error
    results chan error
    wg      sync.WaitGroup
}

func NewWorkerPool(workers, queueSize int) *WorkerPool {
    pool := &WorkerPool{
        work:    make(chan func() error, queueSize),
        results: make(chan error, queueSize),
    }
    
    for i := 0; i < workers; i++ {
        pool.wg.Add(1)
        go pool.worker()
    }
    
    return pool
}

func (p *WorkerPool) worker() {
    defer p.wg.Done()
    
    for work := range p.work {
        if err := work(); err != nil {
            p.results <- err
        }
    }
}

func (p *WorkerPool) Submit(work func() error) {
    p.work <- work
}

func (p *WorkerPool) Shutdown() {
    close(p.work)
    p.wg.Wait()
    close(p.results)
}

// Semaphore for limiting concurrency
type Semaphore struct {
    sem     chan struct{}
    count   int64
    maxSize int
}

func NewSemaphore(maxSize int) *Semaphore {
    return &Semaphore{
        sem:     make(chan struct{}, maxSize),
        maxSize: maxSize,
    }
}

func (s *Semaphore) Acquire() {
    s.sem <- struct{}{}
    atomic.AddInt64(&s.count, 1)
}

func (s *Semaphore) Release() {
    <-s.sem
    atomic.AddInt64(&s.count, -1)
}

func (s *Semaphore) Count() int64 {
    return atomic.LoadInt64(&s.count)
}

func (s *Semaphore) TryAcquire() bool {
    select {
    case s.sem <- struct{}{}:
        atomic.AddInt64(&s.count, 1)
        return true
    default:
        return false
    }
}

// Atomic operations for counters
type AtomicCounter struct {
    count int64
}

func (c *AtomicCounter) Increment() int64 {
    return atomic.AddInt64(&c.count, 1)
}

func (c *AtomicCounter) Decrement() int64 {
    return atomic.AddInt64(&c.count, -1)
}

func (c *AtomicCounter) Get() int64 {
    return atomic.LoadInt64(&c.count)
}

// Parallel processing with bounded memory
func ParallelProcess[T any, R any](
    items []T,
    fn func(T) R,
    workers int,
) []R {
    if len(items) == 0 {
        return nil
    }
    
    results := make([]R, len(items))
    
    // Determine chunk size
    chunkSize := (len(items) + workers - 1) / workers
    if chunkSize < 1 {
        chunkSize = 1
    }
    
    var wg sync.WaitGroup
    
    for i := 0; i < len(items); i += chunkSize {
        wg.Add(1)
        
        start := i
        end := i + chunkSize
        if end > len(items) {
            end = len(items)
        }
        
        go func(start, end int) {
            defer wg.Done()
            
            for j := start; j < end; j++ {
                results[j] = fn(items[j])
            }
        }(start, end)
    }
    
    wg.Wait()
    return results
}

// Batch processing to reduce overhead
func BatchProcess[T any](
    items []T,
    batchSize int,
    fn func([]T) error,
) error {
    for i := 0; i < len(items); i += batchSize {
        end := i + batchSize
        if end > len(items) {
            end = len(items)
        }
        
        if err := fn(items[i:end]); err != nil {
            return err
        }
    }
    
    return nil
}

// GOMAXPROCS configuration
func OptimizeCPU() {
    // Get number of CPU cores
    numCPU := runtime.NumCPU()
    
    // Set to use all cores
    runtime.GOMAXPROCS(numCPU)
    
    // Or limit for specific workloads
    // runtime.GOMAXPROCS(4)
}

// Mutex vs atomic selection guide
// Use atomic for: counters, flags, simple values
// Use mutex for: complex data structures, multiple fields

// Spinlock for short critical sections
type SpinLock struct {
    locked uint32
}

func (s *SpinLock) Lock() {
    for !atomic.CompareAndSwapUint32(&s.locked, 0, 1) {
        runtime.Gosched() // Yield
    }
}

func (s *SpinLock) Unlock() {
    atomic.StoreUint32(&s.locked, 0)
}
```

## 4. Database Query Optimization

### 4.1 Query Optimization Patterns

```sql
-- Complete index creation examples

-- Basic index
CREATE INDEX idx_users_email ON users(email);

-- Composite index for multi-column queries
CREATE INDEX idx_orders_customer_status 
ON orders(customer_id, status, created_at DESC);

-- Partial index for specific query patterns
CREATE INDEX idx_orders_pending 
ON orders(created_at) 
WHERE status = 'PENDING';

-- Covering index (includes all columns needed by query)
CREATE INDEX idx_products_catalog 
ON products(category_id, status) 
INCLUDE (id, name, price, inventory);

-- Expression index for function-based queries
CREATE INDEX idx_users_email_lower ON users(LOWER(email));
CREATE INDEX idx_orders_year ON orders(DATE_PART('year', created_at));

-- Unique index
CREATE UNIQUE INDEX idx_users_email_unique ON users(LOWER(email));

-- Index with storage parameters
CREATE INDEX idx_large_table_text 
ON large_table(text_column) 
WITH (fillfactor = 80);

-- Concurrent index creation (non-blocking)
CREATE INDEX CONCURRENTLY idx_orders_customer_id 
ON orders(customer_id);

-- Drop index
DROP INDEX IF EXISTS idx_users_email;

-- Analyze table for query planning
ANALYZE VERBOSE users;

-- Reindex for maintenance
REINDEX INDEX idx_users_email;
REINDEX DATABASE mydb;

-- Query to find missing indexes
SELECT 
    schemaname,
    tablename,
    seq_scan - idx_scan AS missing_index_scans,
    idx_scan AS index_scans
FROM pg_stat_user_tables
WHERE seq_scan - idx_scan > 100
ORDER BY missing_index_scans DESC;

-- Query to find unused indexes
SELECT 
    schemaname || '.' || tablename AS table_name,
    indexname,
    idx_scan,
    pg_size_pretty(pg_relation_size(indexrelid)) AS index_size
FROM pg_stat_user_indexes
WHERE idx_scan = 0
  AND NOT indexname LIKE '%_pkey'
  AND NOT indexname LIKE '%_seq'
ORDER BY pg_relation_size(indexrelid) DESC;
```

### 4.2 Query Plan Analysis

```sql
-- EXPLAIN ANALYZE for query plan analysis

-- Basic analysis
EXPLAIN ANALYZE 
SELECT u.*, o.* 
FROM users u
LEFT JOIN orders o ON u.id = o.user_id
WHERE u.status = 'ACTIVE'
  AND o.created_at > NOW() - INTERVAL '30 days';

-- EXPLAIN with settings
EXPLAIN (ANALYZE, BUFFERS, FORMAT TEXT)
SELECT * FROM orders WHERE customer_id = 123;

-- Output format options
EXPLAIN (FORMAT JSON)
SELECT * FROM products WHERE category_id = 5;

EXPLAIN (FORMAT YAML)
SELECT * FROM orders WHERE status = 'PENDING';

-- Cost threshold
EXPLAIN (COSTS, VERBOSE, TIMING)
SELECT * FROM large_table WHERE key = 'value';

-- Common patterns to identify:

-- 1. Sequential scan on large table (consider index)
-- Seq Scan on orders  (cost=0.00..100000.00 rows=1000000)

-- 2. Nested loop join (good for small sets)
-- Nested Loop (cost=0.00..100.00 rows=10)

-- 3. Hash join (good for large sets)
-- Hash Join (cost=1000.00..5000.00 rows=10000)

-- 4. Merge join (good for pre-sorted)
-- Merge Join (cost=1000.00..5000.00 rows=10000)

-- Statistics query
SELECT 
    relname,
    reltuples::bigint AS estimated_rows,
    relpages AS page_count,
    pg_size_pretty(pg_relation_size(relid)) AS table_size
FROM pg_class
WHERE relnamespace = 'public'::regnamespace
  AND relkind = 'r'
ORDER BY pg_relation_size(relid) DESC;

-- Table bloat analysis
SELECT 
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) AS total_size,
    pg_size_pretty(pg_relation_size(schemaname||'.'||tablename)) AS table_size,
    n_dead_tup,
    n_live_tup,
    last_autovacuum,
    last_autoanalyze
FROM pg_stat_user_tables
WHERE n_dead_tup > 1000
ORDER BY n_dead_tup DESC;
```

### 4.2 Application-Level Caching

```typescript
// caching/database-cache.ts - Multi-level caching

interface CacheConfig {
  ttl: number;
  maxSize: number;
  staleWhileRevalidate: number;
}

class DatabaseQueryCache {
  private cache: Map<string, CacheEntry>;
  private maxSize: number;
  private ttl: number;
  
  constructor(config: CacheConfig) {
    this.cache = new Map();
    this.maxSize = config.maxSize;
    this.ttl = config.ttl * 1000;
  }
  
  async get<T>(key: string, fetcher: () => Promise<T>): Promise<T> {
    const entry = this.cache.get(key);
    const now = Date.now();
    
    if (entry && now - entry.timestamp < this.ttl) {
      return entry.value as T;
    }
    
    // Stale-while-revalidate
    if (entry && now - entry.timestamp < this.ttl * 2) {
      // Return stale, revalidate in background
      this.revalidate(key, fetcher);
      return entry.value as T;
    }
    
    const value = await fetcher();
    this.set(key, value);
    return value;
  }
  
  private async revalidate<T>(key: string, fetcher: () => Promise<T>): Promise<void> {
    try {
      const value = await fetcher();
      this.set(key, value);
    } catch (error) {
      console.error('Revalidation failed:', error);
    }
  }
  
  private set(key: string, value: unknown): void {
    if (this.cache.size >= this.maxSize) {
      // Evict oldest
      const oldest = Array.from(this.cache.entries())
        .sort((a, b) => a[1].timestamp - b[1].timestamp)[0];
      this.cache.delete(oldest[0]);
    }
    
    this.cache.set(key, {
      value,
      timestamp: Date.now(),
    });
  }
  
  invalidate(key: string): void {
    this.cache.delete(key);
  }
  
  invalidatePattern(pattern: string): void {
    const regex = new RegExp(pattern);
    for (const key of this.cache.keys()) {
      if (regex.test(key)) {
        this.cache.delete(key);
      }
    }
  }
  
  clear(): void {
    this.cache.clear();
  }
}

interface CacheEntry {
  value: unknown;
  timestamp: number;
}

// Cache-aside pattern
class CacheAsidePattern {
  constructor(
    private cache: DatabaseQueryCache,
    private db: DatabaseClient
  ) {}
  
  async getUser(userId: string): Promise<User | null> {
    return this.cache.get(
      `user:${userId}`,
      () => this.db.users.findById(userId)
    );
  }
  
  async getUserOrders(userId: string): Promise<Order[]> {
    return this.cache.get(
      `orders:${userId}`,
      () => this.db.orders.findByUserId(userId)
    );
  }
  
  async invalidateUser(userId: string): void {
    this.cache.invalidate(`user:${userId}`);
    this.cache.invalidatePattern(`orders:${userId}`);
  }
}

// Request coalescing for cache stampede prevention
class RequestCoalescingCache {
  private inflight: Map<string, Promise<unknown>> = new Map();
  
  async get<T>(key: string, fetcher: () => Promise<T>): Promise<T> {
    // Check if request is already in flight
    const existing = this.inflight.get(key);
    if (existing) {
      return existing as Promise<T>;
    }
    
    // Start new request
    const promise = fetcher().finally(() => {
      this.inflight.delete(key);
    }) as Promise<T>;
    
    this.inflight.set(key, promise);
    return promise;
  }
}
```

## 5. Benchmarking Patterns

### 5.1 Go Benchmarking

```go
// benchmarks/database_test.go

package benchmarks

import (
    "testing"
    "database/sql"
    "fmt"
)

func BenchmarkDatabaseQuery(b *testing.B) {
    db, _ := sql.Open("postgres", "connection-string")
    defer db.Close()
    
    // Warmup
    for i := 0; i < 100; i++ {
        db.QueryRow("SELECT * FROM users WHERE id = $1", i%1000)
    }
    
    b.ResetTimer()
    
    for i := 0; i < b.N; i++ {
        rows, err := db.Query("SELECT * FROM users WHERE id = $1", i%1000)
        if err != nil {
            b.Fatal(err)
        }
        rows.Close()
    }
}

func BenchmarkDatabaseQueryParallel(b *testing.B) {
    db, _ := sql.Open("postgres", "connection-string")
    defer db.Close()
    
    b.ResetTimer()
    b.RunParallel(func(pb *testing.PB) {
        i := 0
        for pb.Next() {
            rows, err := db.Query("SELECT * FROM users WHERE id = $1", i%1000)
            if err != nil {
                b.Fatal(err)
            }
            rows.Close()
            i++
        }
    })
}

func BenchmarkStringConcat(b *testing.B) {
    parts := []string{"hello", "world", "this", "is", "a", "test"}
    
    b.ResetTimer()
    
    for i := 0; i < b.N; i++ {
        var result string
        for _, part := range parts {
            result += part + " "
        }
    }
}

func BenchmarkStringBuilder(b *testing.B) {
    parts := []string{"hello", "world", "this", "is", "a", "test"}
    
    b.ResetTimer()
    
    for i := 0; i < b.N; i++ {
        var sb strings.Builder
        sb.Grow(100)
        for _, part := range parts {
            sb.WriteString(part)
            sb.WriteByte(' ')
        }
    }
}

func BenchmarkSliceAppend(b *testing.B) {
    b.ResetTimer()
    
    for i := 0; i < b.N; i++ {
        var s []int
        for j := 0; j < 1000; j++ {
            s = append(s, j)
        }
    }
}

func BenchmarkSlicePrealloc(b *testing.B) {
    b.ResetTimer()
    
    for i := 0; i < b.N; i++ {
        s := make([]int, 0, 1000)
        for j := 0; j < 1000; j++ {
            s = append(s, j)
        }
    }
}

// Run benchmarks with:
// go test -bench=. -benchmem -benchtime=5s
// go test -bench=BenchmarkDatabaseQuery -benchmem
// go test -bench=BenchmarkString -benchmem -cpuprofile=cpu.prof
// go tool pprof cpu.prof
```

### 5.2 Load Testing Configuration

```yaml
# k6/load-test.js - k6 load testing script

import http from 'k6/http';
import { check, sleep, group } from 'k6';
import { Rate, Trend } from 'k6/metrics';

// Custom metrics
const errorRate = new Rate('errors');
const responseTime = new Trend('response_time');

// Test configuration
export const options = {
  scenarios: {
    // Smoke test
    smoke: {
      executor: 'constant-vus',
      vus: 5,
      duration: '1m',
    },
    
    // Load test
    load: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '2m', target: 50 },
        { duration: '5m', target: 50 },
        { duration: '2m', target: 0 },
      ],
    },
    
    // Stress test
    stress: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '2m', target: 100 },
        { duration: '5m', target: 100 },
        { duration: '2m', target: 200 },
        { duration: '5m', target: 200 },
        { duration: '2m', target: 0 },
      ],
    },
    
    // Spike test
    spike: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '1m', target: 100 },
        { duration: '1m', target: 1000 }, // Spike
        { duration: '5m', target: 1000 },
        { duration: '1m', target: 0 },
      ],
    },
    
    // Soak test
    soak: {
      executor: 'constant-vus',
      vus: 100,
      duration: '24h',
    },
  },
  
  thresholds: {
    // Global thresholds
    'http_req_duration': ['p(95)<500'],
    'http_req_failed': ['rate<0.01'],
    
    // Custom thresholds
    'errors': ['rate<0.1'],
    'response_time': ['p(99)<1000'],
  },
};

// Test data
const BASE_URL = 'https://api.example.com';
const TEST_USERS = ['user1@test.com', 'user2@test.com'];

export function setup() {
  // Login and get tokens
  const tokens = TEST_USERS.map(email => {
    const res = http.post(`${BASE_URL}/auth/login`, {
      email,
      password: 'testpass123',
    });
    return JSON.parse(res.body).token;
  });
  
  return { tokens };
}

export default function(data) {
  const token = data.tokens[Math.floor(Math.random() * data.tokens.length)];
  const headers = {
    'Authorization': `Bearer ${token}`,
    'Content-Type': 'application/json',
  };
  
  group('Health Check', () => {
    const res = http.get(`${BASE_URL}/health`);
    check(res, {
      'health check status is 200': (r) => r.status === 200,
    });
  });
  
  group('User Operations', () => {
    // Get user
    const userRes = http.get(`${BASE_URL}/users/me`, { headers });
    check(userRes, {
      'get user status is 200': (r) => r.status === 200,
    });
    errorRate.add(userRes.status !== 200);
    
    // Update user
    const updateRes = http.put(
      `${BASE_URL}/users/me`,
      JSON.stringify({ displayName: 'Updated Name' }),
      { headers }
    );
    check(updateRes, {
      'update user status is 200': (r) => r.status === 200,
    });
    errorRate.add(updateRes.status !== 200);
  });
  
  group('Product Operations', () => {
    // List products
    const listRes = http.get(`${BASE_URL}/products?limit=20`, { headers });
    check(listRes, {
      'list products status is 200': (r) => r.status === 200,
    });
    
    const products = JSON.parse(listRes.body);
    
    // Get single product
    if (products.length > 0) {
      const productRes = http.get(
        `${BASE_URL}/products/${products[0].id}`,
        { headers }
      );
      check(productRes, {
        'get product status is 200': (r) => r.status === 200,
      });
      responseTime.add(productRes.timings.duration);
    }
  });
  
  group('Order Operations', () => {
    // Create order
    const orderRes = http.post(
      `${BASE_URL}/orders`,
      JSON.stringify({
        items: [
          { productId: 'prod_123', quantity: 1 },
        ],
      }),
      { headers }
    );
    
    const orderCreated = check(orderRes, {
      'create order status is 201': (r) => r.status === 201,
    });
    errorRate.add(!orderCreated);
    
    if (orderCreated) {
      const orderId = JSON.parse(orderRes.body).id;
      
      // Get order
      const getRes = http.get(`${BASE_URL}/orders/${orderId}`, { headers });
      check(getRes, {
        'get order status is 200': (r) => r.status === 200,
      });
    }
  });
  
  sleep(1);
}

// Run custom scenarios
export function handleSummary(data) {
  return {
    'stdout': textSummary(data, { indent: ' ', enableColors: true }),
    'summary.json': JSON.stringify(data),
  };
}

function textSummary(data, options) {
  // Generate text summary
  return `
  Test Summary
  =============
  Requests: ${data.metrics.http_reqs.values.count}
  Failed: ${data.metrics.http_req_failed.values.passes}
  Duration: ${data.state.testMetrics.duration}
  
  Response Times:
  - Average: ${data.metrics.http_req_duration.values.avg}ms
  - P95: ${data.metrics.http_req_duration.values['p(95)']}ms
  - P99: ${data.metrics.http_req_duration.values['p(99)']}ms
  `;
}
```

## 6. Decision Matrices

### 6.1 Optimization Technique Selection

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                          Optimization Technique Selection Matrix                         │
├─────────────────────────────────────────────────────────────────────────────────────────┤
│ Issue                       │ First Try              │ If First Fails           │
├─────────────────────────────┼────────────────────────┼────────────────────────────┤
│ Slow DB queries             │ Add indexes            │ Query optimization        │
│                             │ Analyze execution plan │ Connection pooling       │
│                             │                        │ Read replicas            │
├─────────────────────────────┼────────────────────────┼────────────────────────────┤
│ High memory usage           │ Reduce allocations     │ Use object pools         │
│                             │ Clear caches           │ Profile heap             │
│                             │                        │ Increase GOGC           │
├─────────────────────────────┼────────────────────────┼────────────────────────────┤
│ High CPU usage              │ Optimize hot paths     │ Parallelize work         │
│                             │ Reduce allocations     │ Bump GOMAXPROCS          │
│                             │                        │ Consider caching         │
├─────────────────────────────┼────────────────────────┼────────────────────────────┤
│ Slow response times        │ Cache frequent queries │ Add CDN                   │
│                             │ Database optimization  │ Optimize client-side     │
│                             │                        │ Use connection pooling   │
├─────────────────────────────┼────────────────────────┼────────────────────────────┤
│ Memory leaks                │ Profile heap           │ Find unbounded growth    │
│                             │ Check goroutine count  │ Add cleanup handlers     │
│                             │                        │ Use leak detection       │
├─────────────────────────────┼────────────────────────┼────────────────────────────┤
│ Connection exhaustion       │ Connection pooling     │ Tune pool sizes          │
│                             │ Close connections       │ Use proxy/pooler         │
│                             │                        │ Check connection limits  │
└─────────────────────────────┴────────────────────────┴────────────────────────────┘
```

### 6.2 Caching Strategy Selection

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                            Caching Strategy Selection Matrix                             │
├─────────────────────────────────────────────────────────────────────────────────────────┤
│ Data Type                  │ Cache Strategy          │ TTL Recommendation          │
├─────────────────────────────┼────────────────────────┼─────────────────────────────┤
│ User sessions              │ Redis                  │ 24 hours                   │
├─────────────────────────────┼────────────────────────┼─────────────────────────────┤
│ User profiles              │ Cache-aside            │ 1 hour, stale-while-reval  │
├─────────────────────────────┼────────────────────────┼─────────────────────────────┤
│ Product catalog            │ CDN + Redis            │ 24 hours                   │
├─────────────────────────────┼────────────────────────┼─────────────────────────────┤
│ API responses              │ Gateway cache          │ Varies by endpoint         │
├─────────────────────────────┼────────────────────────┼─────────────────────────────┤
│ Database query results     │ Application cache      │ 5-30 minutes               │
├─────────────────────────────┼────────────────────────┼─────────────────────────────┤
│ Static assets              │ CDN                    │ 1 year                     │
├─────────────────────────────┼────────────────────────┼─────────────────────────────┤
│ Real-time data             │ In-memory only         │ No persistent cache        │
└─────────────────────────────┴────────────────────────┴─────────────────────────────┘
```

## 7. Anti-Patterns

### 7.1 Performance Anti-Patterns

```
┌─────────────────────────────────────────────────────────────────────────────────────────┐
│                           Performance Anti-Patterns to Avoid                              │
├─────────────────────────────────────────────────────────────────────────────────────────┤
│ Anti-Pattern                    │ Problem                       │ Solution                │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Premature optimization          │ Complex, hard to maintain     │ Profile first           │
│                                 │ Wasted effort on rare paths   │ Optimize what matters   │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ N+1 queries                     │ Database overload             │ Use JOINs               │
│                                 │ Latency multiplication         │ Use DataLoader          │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ String concatenation in loop   │ Memory allocation spam        │ Use strings.Builder     │
│                                 │ Garbage collection overhead    │ Or bytes.Buffer         │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Synchronous file I/O            │ Thread blocking               │ Use async I/O          │
│                                 │ Poor concurrency              │ Or worker threads      │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Large object allocations        │ GC pressure                   │ Reuse objects           │
│ in hot paths                    │ Memory fragmentation           │ Use pools              │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ No connection pooling          │ Connection overhead            │ Use pool               │
│                                 │ Latency on each request        │ Tune pool sizes        │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Unbounded caches               │ Memory exhaustion              │ Set max size            │
│                                 │ OOM crashes                    │ Implement eviction     │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ No index on WHERE/JOIN cols    │ Full table scans               │ Analyze queries         │
│                                 │ Query timeout                  │ Create proper indexes   │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Deep recursion                 │ Stack overflow                 │ Use iteration          │
│                                 │ Memory heavy                   │ Tail call optimization  │
├─────────────────────────────────┼───────────────────────────────┼────────────────────────┤
│ Serial processing             │ CPU underutilization            │ Parallelize            │
│                                 │ Slower processing              │ Use workers/pipelines │
└─────────────────────────────────┴───────────────────────────────┴────────────────────────┘
```

---

## Links

### Profiling Tools
- [Go pprof](https://github.com/google/pprof)
- [Py-spy](https://github.com/benfred/py-spy)
- [pyflame](https://github.com/uber-archive/pyflame)
- [Node.js profiler](https://nodejs.org/en/docs/guides/simple-profiling/)
- [async-profiler](https://github.com/async-profiler/async-profiler)

### Memory Management
- [Go Memory Model](https://go.dev/ref/mem)
- [GOGC Tuning](https://pkg.go.dev/runtime/debug#SetGCPercent)
- [pprof Memory Documentation](https://github.com/google/pprof/blob/main/doc/README.md)
- [Python memory management](https://docs.python.org/3/c-api/memory.html)

### Database Optimization
- [PostgreSQL EXPLAIN](https://www.postgresql.org/docs/current/sql-explain.html)
- [Query Planning](https://www.postgresql.org/docs/current/planner-optimizer.html)
- [Index Types](https://www.postgresql.org/docs/current/indexes-types.html)
- [MySQL Optimization](https://dev.mysql.com/doc/refman/8.0/en/optimization.html)

### Benchmarking
- [Go Testing/Benchmarking](https://go.dev/doc/testing)
- [k6 Load Testing](https://k6.io/docs/)
- [wrk HTTP Benchmarking](https://github.com/wg/wrk)
- [ab (Apache Bench)](https://httpd.apache.org/docs/2.4/programs/ab.html)

### Caching
- [Redis Documentation](https://redis.io/documentation)
- [Memcached Documentation](https://memcached.org/)
- [HTTP Caching](https://developer.mozilla.org/en-US/docs/Web/HTTP/Caching)
- [CDN Best Practices](https://developer.mozilla.org/en-US/docs/Web/HTML/Optimizing_your_pages_for_speed)

### Performance Tools
- [Prometheus](https://prometheus.io/)
- [Grafana](https://grafana.com/)
- [Datadog](https://www.datadoghq.com/)
- [New Relic](https://newrelic.com/)
- [APM Comparison](https://en.wikipedia.org/wiki/Application_performance_management)