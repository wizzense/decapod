# MEMORY.md - Memory Architecture (DENSE)

**Authority:** guidance (memory management, optimization, and resource patterns)
**Layer:** Guides
**Binding:** No
**Scope:** memory hierarchy, allocation strategies, and memory optimization
**Non-goals:** language-specific garbage collection details, premature optimization

---

## 1. Memory Hierarchy

### 1.1 The Memory Pyramid

```
Speed:    Fast ←———————————————————————————→ Slow
Size:     Small ←——————————————————————————→ Large
Cost:     High ←———————————————————————————→ Low

Registers    → L1 Cache → L2 Cache → L3 Cache → DRAM → SSD → HDD
1 KB         → 32 KB    → 256 KB   → 8 MB     → 64GB → 1TB → 10TB
1 cycle      → 4 cycles → 10 cycles→ 40 cycles→ 100ns→ 10μs→ 10ms
```

### 1.2 Memory Access Patterns

| Pattern | Latency Impact | Cache-Friendly | Example |
|---------|---------------|----------------|---------|
| Sequential read | ~1x | Yes | Array iteration |
| Sequential write | ~1x | Yes | Building result array |
| Random read | ~10x | No | Tree traversal |
| Random write | ~10x | No | Hash table insertion |
| Strided read | Varies | Depends on stride | Matrix row vs column |

### 1.3 Cache Line Behavior

```
Cache Line Size: 64 bytes (typical)

// GOOD: Sequential access, fits in cache lines
for (int i = 0; i < n; i++) {
    process(arr[i]);  // Each element = 1 cache line
}

// BAD: Strided access, poor cache utilization
for (int i = 0; i < n; i += 4) {
    process(arr[i]);  // Only 1 in 4 cache lines used
}

// GOOD: Contiguous struct, cache-friendly
struct Item { int x; int y; int z; }
std::vector<Item> items;

// BAD: Array of pointers, scattered heap
std::vector<Item*> items;  // Each Item on heap, bad locality
```

---

## 2. Memory Allocation Strategies

### 2.1 Stack vs Heap Decision Matrix

| Factor | Stack | Heap |
|--------|-------|------|
| Size limit | ~1-8 MB (configurable) | System RAM |
| Lifetime | Function scope | Manual or GC |
| Allocation | O(1), pointer bump | O(log n) to O(1) |
| Deallocation | Automatic | Manual/GC |
| Thread safety | Trivial | Requires sync |
| Use for | Local variables, small fixed-size | Dynamic size, long-lived |

### 2.2 Arena/Pool Allocation

```rust
// Arena allocator - fast allocation, bulk deallocation
use std::alloc::{alloc, Layout};
use std::ptr::NonNull;

pub struct Arena {
    ptr: NonNull<u8>,
    remaining: usize,
    capacity: usize,
}

impl Arena {
    pub fn new(capacity: usize) -> Self {
        let ptr = unsafe { alloc(Layout::array::<u8>(capacity).unwrap()) };
        Self {
            ptr: NonNull::new(ptr).unwrap(),
            remaining: capacity,
            capacity,
        }
    }
    
    pub fn alloc<T>(&mut self, value: &T) -> *mut T {
        let size = std::mem::size_of::<T>();
        let align = std::mem::align_of::<T>();
        
        // Align pointer
        let align_offset = (self.ptr.as_ptr() as usize) % align;
        let skip = if align_offset == 0 { 0 } else { align - align_offset };
        
        if self.remaining < size + skip {
            panic!("Arena out of memory");
        }
        
        let ptr = unsafe { self.ptr.as_ptr().add(skip) };
        self.ptr = NonNull::new(ptr.add(size)).unwrap();
        self.remaining -= size + skip;
        
        // Write value
        unsafe { ptr::write(ptr as *mut T, value.clone()) };
        ptr
    }
    
    pub fn clear(&mut self) {
        self.ptr = NonNull::new(unsafe { alloc(Layout::array::<u8>(self.capacity).unwrap()) }).unwrap();
        self.remaining = self.capacity;
    }
}

impl Drop for Arena {
    fn drop(&mut self) {
        unsafe { std::alloc::dealloc(self.ptr.as_ptr(), Layout::array::<u8>(self.capacity).unwrap()) };
    }
}

// Usage example
fn process_with_arena() {
    let mut arena = Arena::new(1024 * 1024); // 1MB arena
    
    let items: Vec<String> = vec![
        "item1".to_string(),
        "item2".to_string(),
        "item3".to_string(),
    ];
    
    for item in items {
        let _ptr = arena.alloc(&item); // O(1) allocation
    }
    
    // All memory freed at once when arena is dropped
}
```

### 2.3 Object Pool Pattern

```rust
// Object pool for expensive-to-create objects
use std::sync::{Arc, Mutex};

pub struct ObjectPool<T: Default> {
    available: Arc<Mutex<Vec<T>>>,
    in_use: Arc<Mutex<usize>>,
    max_size: usize,
}

impl<T: Default + 'static> ObjectPool<T> {
    pub fn new(max_size: usize) -> Self {
        let initial = (0..max_size)
            .map(|_| T::default())
            .collect();
        
        Self {
            available: Arc::new(Mutex::new(initial)),
            in_use: Arc::new(Mutex::new(0)),
            max_size,
        }
    }
    
    pub fn acquire(&self) -> PooledObject<T> {
        let obj = {
            let mut available = self.available.lock().unwrap();
            available.pop()
        };
        
        match obj {
            Some(obj) => {
                *self.in_use.lock().unwrap() += 1;
                PooledObject::new(self.clone(), obj)
            }
            None => {
                // Pool exhausted, create new object
                *self.in_use.lock().unwrap() += 1;
                PooledObject::new(self.clone(), T::default())
            }
        }
    }
    
    fn release(&self, obj: T) {
        let count = *self.in_use.lock().unwrap();
        if count > self.max_size {
            // Shrink pool
            drop(obj);
        } else {
            self.available.lock().unwrap().push(obj);
        }
        *self.in_use.lock().unwrap() -= 1;
    }
}

pub struct PooledObject<T> {
    pool: Option<ObjectPool<T>>,
    value: T,
}

impl<T> PooledObject<T> {
    fn new(pool: ObjectPool<T>, value: T) -> Self {
        Self { pool: Some(pool), value }
    }
    
    pub fn value(&self) -> &T {
        &self.value
    }
    
    pub fn value_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

impl<T> Drop for PooledObject<T> {
    fn drop(&mut self) {
        if let Some(pool) = self.pool.take() {
            pool.release(std::mem::take(&mut self.value));
        }
    }
}

// Usage: Database connection pool
struct DBConnection {
    connected: bool,
}

impl Default for DBConnection {
    fn default() -> Self {
        // Expensive: actually connect to DB
        println!("Creating new DB connection");
        Self { connected: true }
    }
}
```

---

## 3. Memory Optimization Patterns

### 3.1 Zero-Copy Techniques

```rust
// Zero-copy message parsing with lifetimes
use std::str;

struct Message<'a> {
    // Borrowed from raw bytes, no allocation
    payload: &'a str,
    headers: &'a [Header<'a>],
}

struct ParsedMessage<'a> {
    data: &'a [u8],
}

impl<'a> ParsedMessage<'a> {
    fn parse(&self) -> Result<Message<'a>, &'static str> {
        // Parse without copying
        let payload = str::from_utf8(self.data)
            .map_err(|_| "Invalid UTF-8")?;
        
        Ok(Message {
            payload,
            headers: &[],
        })
    }
}

// Zero-copy with bytes crate
use bytes::{Bytes, BytesMut};

fn parse_frames(data: &Bytes) -> Vec<&[u8]> {
    let mut frames = Vec::new();
    let mut cursor = 0;
    
    while cursor < data.len() {
        // Read frame length (4 bytes)
        let len = u32::from_be_bytes([
            data[cursor],
            data[cursor + 1],
            data[cursor + 2],
            data[cursor + 3],
        ]) as usize;
        
        cursor += 4;
        frames.push(&data[cursor..cursor + len]);
        cursor += len;
    }
    
    frames
}

// Zero-copy HTTP parsing
struct HttpRequest<'a> {
    method: &'a str,
    path: &'a str,
    version: &'a str,
    headers: &'a [(&'a str, &'a str)],
    body: Option<&'a [u8]>,
}

fn parse_http_request(buffer: &[u8]) -> Option<HttpRequest> {
    let end_of_headers = buffer.windows(4)
        .position(|w| w == b"\r\n\r\n")?;
    
    let header_section = &buffer[..end_of_headers];
    let mut lines = header_section.split(|&b| b == b'\n');
    
    // Parse request line
    let request_line = lines.next()?;
    let parts: Vec<&[u8]> = request_line.split(|&b| b == b' ').collect();
    if parts.len() != 3 { return None; }
    
    let method = std::str::from_utf8(parts[0]).ok()?;
    let path = std::str::from_utf8(parts[1]).ok()?;
    let version = std::str::from_utf8(parts[2]).ok()?;
    
    // Parse headers (zero-copy)
    let mut headers = Vec::new();
    for line in lines {
        if let Some((k, v)) = line.split_once(b": ") {
            let k = std::str::from_utf8(k).ok()?;
            let v = std::str::from_utf8(v).ok()?;
            headers.push((k, v.trim()));
        }
    }
    
    Some(HttpRequest {
        method,
        path,
        version: &version[..8],  // "HTTP/1.1"
        headers: Box::leak(headers.into_boxed_slice()),
        body: None,
    })
}
```

### 3.2 Small Buffer Optimization (SSBO/SO)

```rust
// Small buffer optimization for strings
use std::mem::{size_of, MaybeUninit};

const SSO_CAPACITY: usize = 22; // Fits in 3 pointers (on 64-bit)

pub enum SmallString {
    Stack {
        length: u8,
        data: [u8; SSO_CAPACITY],
    },
    Heap {
        ptr: Box<[u8]>,
        length: usize,
    },
}

impl SmallString {
    pub fn new(s: &str) -> Self {
        if s.len() <= SSO_CAPACITY {
            let mut data = [0u8; SSO_CAPACITY];
            data[..s.len()].copy_from_slice(s.as_bytes());
            SmallString::Stack {
                length: s.len() as u8,
                data,
            }
        } else {
            SmallString::Heap {
                ptr: s.as_bytes().to_vec().into_boxed_slice(),
                length: s.len(),
            }
        }
    }
    
    pub fn as_str(&self) -> &str {
        match self {
            SmallString::Stack { length, data } => {
                let slice = &data[..*length as usize];
                std::str::from_utf8(slice).unwrap()
            }
            SmallString::Heap { ptr, length } => {
                let slice = &ptr[..*length];
                std::str::from_utf8(slice).unwrap()
            }
        }
    }
}
```

### 3.3 Streaming/Chunked Processing

```rust
// Memory-bounded stream processing
use std::io::{BufRead, BufReader};
use std::fs::File;

struct ChunkedProcessor {
    chunk_size: usize,
    buffer: Vec<u8>,
}

impl ChunkedProcessor {
    pub fn new(chunk_size: usize) -> Self {
        Self {
            chunk_size,
            buffer: Vec::with_capacity(chunk_size),
        }
    }
    
    pub fn process_file<F>(&mut self, path: &str, mut processor: F) -> Result<(), std::io::Error>
    where
        F: FnMut(&[u8]) -> std::io::Result<()>,
    {
        let file = File::open(path)?;
        let reader = BufReader::with_capacity(self.chunk_size * 2, file);
        
        for line in reader.lines() {
            let line = line?;
            self.buffer.extend(line.as_bytes());
            self.buffer.push(b'\n');
            
            if self.buffer.len() >= self.chunk_size {
                processor(&self.buffer)?;
                self.buffer.clear();
            }
        }
        
        // Process remaining
        if !self.buffer.is_empty() {
            processor(&self.buffer)?;
        }
        
        Ok(())
    }
}

// External merge sort for large files
async fn external_merge_sort(
    input_path: &Path,
    output_path: &Path,
    chunk_size: usize,
    max_memory_bytes: usize,
) -> std::io::Result<()> {
    let mut temp_files = Vec::new();
    let mut reader = BufReader::new(File::open(input_path)?);
    
    // Phase 1: Sort chunks
    let mut current_chunk: Vec<String> = Vec::new();
    let mut current_size = 0;
    
    for line in reader.lines() {
        let line = line?;
        current_size += line.len() + 1;
        current_chunk.push(line);
        
        if current_size >= max_memory_bytes {
            current_chunk.sort();
            let temp_file = tempfile::NamedTempFile::new()?;
            let mut writer = BufWriter::new(&temp_file);
            for item in current_chunk.drain(..) {
                writeln!(writer, "{}", item)?;
            }
            writer.flush()?;
            temp_files.push(temp_file);
            current_size = 0;
        }
    }
    
    // Process remaining
    if !current_chunk.is_empty() {
        current_chunk.sort();
        let temp_file = tempfile::NamedTempFile::new()?;
        // ... write chunk
        temp_files.push(temp_file);
    }
    
    // Phase 2: K-way merge
    let mut heaps: BinaryHeap<(Reverse<String>, usize)> = BinaryHeap::new();
    
    // Open all temp files and read first line
    let mut file_handles: Vec<BufReader<File>> = temp_files
        .iter()
        .map(|f| BufReader::new(File::open(f.path()).unwrap()))
        .collect();
    
    for (i, reader) in file_handles.iter_mut().enumerate() {
        if let Some(line) = reader.lines().next() {
            if let Ok(line) = line {
                heaps.push((Reverse(line), i));
            }
        }
    }
    
    let mut output = BufWriter::new(File::create(output_path)?);
    
    while let Some((Reverse(line), file_idx)) = heaps.pop() {
        writeln!(output, "{}", line)?;
        
        if let Some(line) = file_handles[file_idx].lines().next() {
            if let Ok(line) = line {
                heaps.push((Reverse(line), file_idx));
            }
        }
    }
    
    Ok(())
}
```

---

## 4. Memory-Bounded Data Structures

### 4.1 Bounded LRU Cache

```rust
use std::collections::{HashMap, LinkedList};
use std::hash::Hash;

pub struct BoundedLRUCache<K, V> {
    capacity: usize,
    map: HashMap<K, LinkedListNode<K, V>>,
    list: LinkedList<(K, V)>,
}

struct LinkedListNode<K, V> {
    iter: std::collections::linked_list::Iter<'_, (K, V)>,
}

impl<K: Eq + Hash + Clone, V: Clone> BoundedLRUCache<K, V> {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            map: HashMap::with_capacity(capacity),
            list: LinkedList::new(),
        }
    }
    
    pub fn get(&mut self, key: &K) -> Option<V> {
        if let Some(pos) = self.map.get(key) {
            // Move to front (most recently used)
            let value = pos.1.clone();
            self.list.remove(pos.iter.clone());
            self.list.push_front((key.clone(), value.clone()));
            self.map.insert(key.clone(), self.list.iter().next().unwrap().0);
            Some(value)
        } else {
            None
        }
    }
    
    pub fn put(&mut self, key: K, value: V) {
        // If key exists, update and move to front
        if let Some(_) = self.map.get(&key) {
            self.list.remove(self.map.get(&key).unwrap().iter.clone());
            self.list.push_front((key.clone(), value));
            let new_iter = self.list.iter().next().unwrap().0;
            self.map.insert(key, new_iter);
            return;
        }
        
        // Evict if at capacity
        if self.map.len() >= self.capacity {
            if let Some((old_key, _)) = self.list.pop_back() {
                self.map.remove(&old_key);
            }
        }
        
        // Insert new
        self.list.push_front((key.clone(), value));
        let iter = self.list.iter().next().unwrap().0;
        self.map.insert(key, iter);
    }
}
```

### 4.2 Ring Buffer

```rust
use std::fmt;

pub struct RingBuffer<T> {
    buffer: Vec<Option<T>>,
    write_idx: usize,
    read_idx: usize,
    len: usize,
    capacity: usize,
}

impl<T: Clone> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![None; capacity],
            write_idx: 0,
            read_idx: 0,
            len: 0,
            capacity,
        }
    }
    
    pub fn push(&mut self, item: T) -> Option<T> {
        let overwritten = self.buffer[self.write_idx].take();
        self.buffer[self.write_idx] = Some(item);
        self.write_idx = (self.write_idx + 1) % self.capacity;
        
        if self.len < self.capacity {
            self.len += 1;
        } else {
            // Buffer is full, advance read_idx
            self.read_idx = (self.read_idx + 1) % self.capacity;
        }
        
        overwritten
    }
    
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        
        let item = self.buffer[self.read_idx].take();
        self.read_idx = (self.read_idx + 1) % self.capacity;
        self.len -= 1;
        
        item
    }
    
    pub fn peek(&self) -> Option<&T> {
        if self.len == 0 {
            return None;
        }
        self.buffer[self.read_idx].as_ref()
    }
    
    pub fn len(&self) -> usize {
        self.len
    }
    
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

impl<T: Clone> Iterator for RingBuffer<T> {
    type Item = T;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.pop()
    }
}
```

### 4.3 Memory-Mapped File

```rust
use std::fs::{File, OpenOptions};
use std::io::{Read, Write, Seek, SeekFrom};
use std::path::Path;

pub struct MmappedFile {
    file: File,
    data: Vec<u8>,
    size: usize,
}

impl MmappedFile {
    pub fn open(path: &Path) -> std::io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;
        
        let metadata = file.metadata()?;
        let size = metadata.len() as usize;
        let data = unsafe {
            let mapped = memmap2::MmapOptions::new()
                .len(size.max(1))
                .map(&file)?;
            std::slice::from_raw_parts_mut(mapped.as_ptr(), size).to_vec()
        };
        
        Ok(Self { file, data, size })
    }
    
    pub fn read_at(&self, offset: usize, buf: &mut [u8]) -> std::io::Result<usize> {
        let end = (offset + buf.len()).min(self.size);
        let len = end - offset;
        buf[..len].copy_from_slice(&self.data[offset..end]);
        Ok(len)
    }
    
    pub fn write_at(&mut self, offset: usize, data: &[u8]) -> std::io::Result<()> {
        let end = (offset + data.len()).max(self.size);
        
        if end > self.size {
            self.file.set_len(end as u64)?;
            self.data.resize(end, 0);
            self.size = end;
        }
        
        self.data[offset..offset + data.len()].copy_from_slice(data);
        Ok(())
    }
    
    pub fn flush(&self) -> std::io::Result<()> {
        self.file.flush()?;
        // Also sync to disk
        #[cfg(unix)]
        unsafe {
            libc::fsync(self.file.as_raw_fd());
        }
        Ok(())
    }
}
```

---

## 5. Memory Safety

### 5.1 Use-After-Free Prevention

```rust
// RAII guard pattern
pub struct Guard<T> {
    data: Option<T>,
    cleanup: Box<dyn FnOnce(&T)>,
}

impl<T> Guard<T> {
    pub fn new(data: T, cleanup: impl FnOnce(&T) + 'static) -> Self {
        Self {
            data: Some(data),
            cleanup: Box::new(cleanup),
        }
    }
}

impl<T> Drop for Guard<T> {
    fn drop(&mut self) {
        if let Some(ref data) = self.data {
            (self.cleanup)(data);
        }
    }
}

// Owning raw pointer pattern
pub struct OwnedPtr<T> {
    ptr: *mut T,
}

impl<T> OwnedPtr<T> {
    pub fn new(value: T) -> Self {
        let ptr = Box::into_raw(Box::new(value));
        Self { ptr }
    }
    
    pub fn as_ref(&self) -> &T {
        unsafe { &*self.ptr }
    }
    
    pub fn as_mut(&mut self) -> &mut T {
        unsafe { &mut *self.ptr }
    }
}

impl<T> Drop for OwnedPtr<T> {
    fn drop(&mut self) {
        unsafe { Box::from_raw(self.ptr) };
    }
}

// Borrowing checker pattern (simpler than full borrow checker)
use std::cell::{Cell, RefCell};

pub struct CheckedPtr<T> {
    ptr: Cell<Option<*const T>>,
    borrowed: RefCell<bool>,
}

impl<T> CheckedPtr<T> {
    pub fn new(value: &T) -> Self {
        Self {
            ptr: Cell::new(Some(value as *const T)),
            borrowed: RefCell::new(false),
        }
    }
    
    pub fn borrow(&self) -> Option<&T> {
        if *self.borrowed.borrow() {
            return None;
        }
        *self.borrowed.borrow_mut() = true;
        self.ptr.get().map(|p| unsafe { &*p })
    }
    
    pub fn release(&self) {
        *self.borrowed.borrow_mut() = false;
    }
}

impl<T> Drop for CheckedPtr<T> {
    fn drop(&mut self) {
        *self.borrowed.borrow_mut() = false;
    }
}
```

### 5.2 Buffer Overflow Protection

```rust
// Safe string operations
pub fn safe_copy(src: &[u8], dst: &mut [u8]) -> usize {
    let len = src.len().min(dst.len());
    dst[..len].copy_from_slice(&src[..len]);
    len
}

pub fn safe_concat(a: &str, b: &str, max_len: usize) -> String {
    let mut result = String::with_capacity(max_len);
    for (i, c) in a.chars().chain(b.chars()).take(max_len).enumerate() {
        result.push(c);
    }
    result
}

// Length-checked operations
pub trait LengthChecked {
    fn checked_copy(&self, dst: &mut [u8]) -> Result<(), ()>;
    fn checked_clone(&self, max_len: usize) -> Result<Self, ()>
    where Self: Sized;
}

impl LengthChecked for str {
    fn checked_copy(&self, dst: &mut [u8]) -> Result<(), ()> {
        if self.len() > dst.len() {
            return Err(());
        }
        dst[..self.len()].copy_from_slice(self.as_bytes());
        Ok(())
    }
    
    fn checked_clone(&self, max_len: usize) -> Result<String, ()> {
        if self.len() > max_len {
            return Err(());
        }
        Ok(self.to_string())
    }
}
```

---

## 6. GC-Friendly Patterns

### 6.1 Object Retention Analysis

```rust
// GC pressure analysis
pub struct GCStats {
    pub allocated_bytes: u64,
    pub retained_bytes: u64,
    pub collections: u64,
    pub pause_time_ms: f64,
}

pub fn analyze_retenion<T>(object: &T) -> usize
where
    T: ?Sized,
{
    // Estimate memory retained by this object
    std::mem::size_of_val(object)
}

// Pattern: Weak references for caches
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;

pub struct WeakCache<K, V> {
    map: RefCell<HashMap<K, std::rc::Weak<V>>>,
}

impl<K, V> WeakCache<K, V>
where
    K: Eq + Hash,
{
    pub fn new() -> Self {
        Self {
            map: RefCell::new(HashMap::new()),
        }
    }
    
    pub fn get(&self, key: &K) -> Option<V>
    where
        V: Clone,
    {
        let map = self.map.borrow();
        map.get(key)
            .and_then(|w| w.upgrade())
            .map(|v| (*v).clone())
    }
    
    pub fn insert(&self, key: K, value: V) {
        let mut map = self.map.borrow_mut();
        map.insert(key, std::rc::Rc::downgrade(&std::rc::Rc::new(value)));
    }
    
    pub fn cleanup(&self) {
        let mut map = self.map.borrow_mut();
        map.retain(|_, v| v.upgrade().is_some());
    }
}
```

### 6.2 Object Graph Optimization

```rust
// Avoid deep object graphs - use IDs instead
#[derive(Clone, Copy)]
pub struct UserId(Uuid);

pub struct Post {
    pub id: Uuid,
    pub author_id: UserId,  // Reference by ID, not &User
    pub title: String,
    pub content: String,
}

// Bad: Deep coupling
pub struct BadPost {
    pub author: Box<User>,  // Strong reference
}

// Good: ID-based reference
pub struct GoodPost {
    pub author_id: UserId,
}

// Use indices for related data
pub struct PostStore {
    posts: Vec<Post>,
    by_author: HashMap<UserId, Vec<usize>>,  // Index
}
```

---

## 7. Memory Monitoring

### 7.1 Memory Metrics

```rust
pub struct MemoryMetrics {
    pub rss_bytes: u64,           // Resident set size
    pub heap_used_bytes: u64,
    pub heap_total_bytes: u64,
    pub virtual_memory_bytes: u64,
    pub major_gcs: u64,
    pub minor_gcs: u64,
    pub gc_pause_ms: f64,
}

#[cfg(unix)]
pub fn get_memory_metrics() -> MemoryMetrics {
    let mut rss = 0u64;
    let status = std::fs::read_to_string("/proc/self/status").unwrap();
    
    for line in status.lines() {
        if line.starts_with("VmRSS:") {
            rss = line.split_whitespace()
                .nth(1)
                .and_then(|s| s.parse().ok())
                .unwrap_or(0) * 1024;
        }
    }
    
    MemoryMetrics {
        rss_bytes: rss,
        heap_used_bytes: 0,  // Would need jemalloc or similar
        heap_total_bytes: 0,
        virtual_memory_bytes: 0,
        major_gcs: 0,
        minor_gcs: 0,
        gc_pause_ms: 0.0,
    }
}
```

### 7.2 Leak Detection

```rust
// Simple leak detector
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicUsize, Ordering};

static ALLOCATED: AtomicUsize = AtomicUsize::new(0);
static CURRENT: AtomicUsize = AtomicUsize::new(0);

pub struct LeakDetector<T> {
    _phantom: std::marker::PhantomData<T>,
    id: usize,
}

static mut LEAK_MAP: Option<HashSet<usize>> = None;

impl<T> LeakDetector<T> {
    pub fn new() -> Self {
        static ID: AtomicUsize = AtomicUsize::new(0);
        let id = ID.fetch_add(1, Ordering::SeqCst);
        
        unsafe {
            if LEAK_MAP.is_none() {
                LEAK_MAP = Some(HashSet::new());
            }
            LEAK_MAP.as_mut().unwrap().insert(id);
        }
        
        ALLOCATED.fetch_add(1, Ordering::SeqCst);
        CURRENT.fetch_add(1, Ordering::SeqCst);
        
        Self {
            _phantom: std::marker::PhantomData,
            id,
        }
    }
}

impl<T> Drop for LeakDetector<T> {
    fn drop(&mut self) {
        CURRENT.fetch_sub(1, Ordering::SeqCst);
        
        unsafe {
            if let Some(ref mut map) = LEAK_MAP {
                map.remove(&self.id);
            }
        }
    }
}

pub fn get_leak_stats() -> (usize, usize) {
    (ALLOCATED.load(Ordering::SeqCst), CURRENT.load(Ordering::SeqCst))
}
```

---

## 8. Anti-Patterns

| Anti-Pattern | Specific Failure Mode | Prevention |
|--------------|----------------------|------------|
| **Premature optimization** | Wasted effort, complex code | Measure first, optimize bottleneck |
| **Memory hoarding** | OOM on long-running processes | Clear/release when done |
| **Giant objects** | Cache line waste, GC pressure | Small, focused structures |
| **Allocation in hot loop** | GC pressure, slow performance | Reuse objects, pool |
| **Ignoring memory hierarchy** | Poor cache utilization | Sequential access, locality |
| **No bounds checking** | Buffer overflow, security bug | Safe abstractions |
| **Deep call stacks** | Stack overflow | Iterative alternatives |
| **Unbounded caches** | Memory leak | Size limits + eviction |
| **Finalizers for cleanup** | Resurrection, unpredictable | Explicit close methods |
| **String concatenation in loop** | O(n²) memory allocation | StringBuilder |

---

## Links

### Core Router
- `core/DECAPOD.md` - **Router and navigation charter (START HERE)**
- `core/ENGINEERING_EXCELLENCE.md` - Engineering standards

### Architecture (This Section)
- `architecture/DATA.md` - Data architecture
- `architecture/CACHING.md` - Caching patterns
- `architecture/CONCURRENCY.md` - Concurrent memory access

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
- `methodology/PERFORMANCE_OPTIMIZATION.md` - Memory optimization