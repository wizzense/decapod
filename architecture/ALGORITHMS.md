# ALGORITHMS.md - Algorithms & Data Structures (DENSE)

**Authority:** guidance (algorithm selection, complexity analysis, and optimization)
**Layer:** Guides
**Binding:** No
**Scope:** algorithm patterns, complexity trade-offs, and data structure selection
**Non-goals:** academic proofs, premature optimization without measurement

---

## 1. Algorithm Selection Principles

### 1.1 Complexity Classes

| Class | Name | Practical Limit | Typical Algorithms |
|-------|------|-----------------|---------------------|
| O(1) | Constant | Unlimited | Hash lookup, array index |
| O(log n) | Logarithmic | Millions | Binary search, BST |
| O(n) | Linear | Billions | Single loop, linear search |
| O(n log n) | Linearithmic | Millions | Merge sort, heapsort |
| O(n²) | Quadratic | Thousands | Nested loops, quicksort worst |
| O(n³) | Cubic | Thousands | Floyd-Warshall, matrix multiply |
| O(2ⁿ) | Exponential | < 30 | Brute force subsets |
| O(n!) | Factorial | < 12 | Permutations |

### 1.2 Time-Space Tradeoff Matrix

| Algorithm | Time (avg) | Space | When to Use |
|-----------|-----------|-------|------------|
| Hash table | O(1) | O(n) | Fast lookups, unique keys |
| BST (balanced) | O(log n) | O(n) | Ordered data, range queries |
| Array | O(1) idx / O(n) search | O(1) | Frequent index access |
| Linked list | O(n) | O(n) | Frequent insert/delete |
| Heap | O(log n) | O(n) | Priority queue |
| Trie | O(m) | O(ALPHABET × m × n) | Prefix matching |
| Bloom filter | O(k) | O(n) | Membership test with false positives |

### 1.3 Production Mindset
The gap between academic algorithm knowledge and production engineering is real:

- **Standard libraries first:** Most business value lives in domain logic, not sorting internals. Use language-native, battle-tested implementations. Custom algorithms are warranted only when the standard approach imposes a measurable, load-bearing bottleneck.
- **Maintenance cost is a first-class constraint:** A clever algorithm maintained by one person is a single point of failure. Favor correct and readable over theoretically optimal.
- **Data locality beats asymptotic complexity for small n:** Most production operation sets are small (n < 1000). O(n²) with cache-friendly sequential access frequently outperforms O(n log n) with pointer chasing. The memory wall is the real bottleneck in modern hardware.
- **Prefer scale-out over scale-up:** An O(n log n) algorithm that parallelizes cleanly across 100 machines is often more practical than an O(n) algorithm that must remain single-threaded.
- **Determinism is a correctness property:** In a system governed by reproducible validation, algorithms must produce identical output for identical input. Avoid non-deterministic choices (e.g., unseed random pivots) anywhere output is compared or stored.
- **Resource budgets are not optional:** Every algorithm must have time and memory bounds enforced at the call site. An algorithm that may run forever or allocate without limit is a bug, not a performance risk.

---

## 2. Sorting Algorithms

### 2.1 Sorting Algorithm Comparison

| Algorithm | Best | Average | Worst | Space | Stable | Notes |
|-----------|------|---------|-------|-------|--------|-------|
| Timsort | O(n) | O(n log n) | O(n log n) | O(n) | Yes | Python, Java default |
| Quicksort | O(n log n) | O(n log n) | O(n²) | O(log n) | No | Fast in practice |
| Mergesort | O(n log n) | O(n log n) | O(n log n) | O(n) | Yes | Linked lists |
| Heapsort | O(n log n) | O(n log n) | O(n log n) | O(1) | No | Guaranteed O(n log n) |
| Radix (LSD) | O(nk) | O(nk) | O(nk) | O(n+k) | Yes | Integers only |
| Counting | O(n + k) | O(n + k) | O(n + k) | O(k) | Yes | Limited range |
| Insertion | O(n) | O(n²) | O(n²) | O(1) | Yes | Small/nearly sorted |

### 2.2 Implementation Examples

```rust
// Quicksort with median-of-three pivot and three-way partition
pub fn quicksort<T: Ord + Clone>(arr: &mut [T]) {
    let len = arr.len();
    if len <= 1 {
        return;
    }
    
    // Median-of-three pivot selection
    let mid = len / 2;
    let (a, b, c) = (arr[0].clone(), arr[mid].clone(), arr[len - 1].clone());
    let pivot = vec![a, b, c];
    pivot.sort();
    let pivot_val = &pivot[1];
    
    // Three-way partition (Dutch National Flag)
    let mut lt = 0;  // Elements less than pivot
    let mut gt = len; // Elements greater than pivot
    let mut i = 0;   // Current element
    
    while i < gt {
        match arr[i].cmp(pivot_val) {
            std::cmp::Ordering::Less => {
                arr.swap(lt, i);
                lt += 1;
                i += 1;
            }
            std::cmp::Ordering::Greater => {
                gt -= 1;
                arr.swap(i, gt);
            }
            std::cmp::Ordering::Equal => {
                i += 1;
            }
        }
    }
    
    // Recursively sort partitions
    quicksort(&mut arr[..lt]);
    quicksort(&mut arr[gt..]);
}

// Mergesort for linked lists
pub struct ListNode<T> {
    val: T,
    next: Option<Box<ListNode<T>>>,
}

impl ListNode {
    pub fn merge_sort(&mut self, n: usize) -> Option<Box<ListNode<T>>> {
        if n <= 1 {
            return self.next.take();
        }
        
        let mid = n / 2;
        let mut left_tail = self;
        let mut count = mid;
        
        // Split list into two halves
        while count > 1 {
            if let Some(next) = &left_tail.next {
                left_tail = left_tail.next.as_mut().unwrap();
            }
            count -= 1;
        }
        
        let mut right = left_tail.next.take();
        if let Some(ref mut right_node) = right {
            right = right_node.merge_sort(n - mid);
        }
        
        // Merge sorted halves
        self.merge(right)
    }
    
    fn merge(&mut self, mut right: Option<Box<ListNode<T>>>) -> Option<Box<ListNode<T>>> {
        let mut dummy = Box::new(ListNode { val: self.val.take(), next: None });
        let mut tail = &mut dummy;
        
        let mut left = Some(Box::new(std::mem::replace(self, ListNode { val: T::default(), next: None })));
        
        while let (Some(l), Some(r)) = (&mut left, &mut right) {
            if l.val <= r.val {
                tail.next = left.take();
                if let Some(next) = tail.next.as_mut() {
                    left = next.next.take();
                    tail = tail.next.as_mut().unwrap();
                }
            } else {
                tail.next = right.take();
                if let Some(next) = tail.next.as_mut() {
                    right = next.next.take();
                    tail = tail.next.as_mut().unwrap();
                }
            }
        }
        
        if left.is_some() {
            tail.next = left;
        } else if right.is_some() {
            tail.next = right;
        }
        
        dummy.next
    }
}
```

### 2.3 External Sorting (for datasets larger than memory)

```rust
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;

pub struct ExternalMergeSort {
    temp_dir: std::path::PathBuf,
    chunk_size: usize,  // Max bytes per chunk
}

impl ExternalMergeSort {
    pub fn new(temp_dir: &Path, chunk_size: usize) -> Self {
        std::fs::create_dir_all(temp_dir).unwrap();
        Self {
            temp_dir: temp_dir.to_path_buf(),
            chunk_size,
        }
    }
    
    pub fn sort(&self, input_path: &Path, output_path: &Path, max_memory: usize) -> std::io::Result<()> {
        // Phase 1: Create sorted chunks
        let chunks = self.create_sorted_chunks(input_path, max_memory)?;
        
        // Phase 2: K-way merge
        self.merge_chunks(&chunks, output_path)?;
        
        // Cleanup temp files
        for chunk in &chunks {
            let _ = std::fs::remove_file(chunk);
        }
        
        Ok(())
    }
    
    fn create_sorted_chunks(&self, input_path: &Path, max_memory: usize) -> std::io::Result<Vec<std::path::PathBuf>> {
        let mut chunks = Vec::new();
        let file = File::open(input_path)?;
        let reader = BufReader::with_capacity(max_memory, file);
        
        let mut current_chunk = Vec::new();
        let mut chunk_idx = 0;
        
        for line in reader.lines() {
            let line = line?;
            current_chunk.push(line);
            
            if current_chunk.iter().map(|s| s.len()).sum::<usize>() >= self.chunk_size {
                current_chunk.sort();
                let chunk_path = self.temp_dir.join(format!("chunk_{}.txt", chunk_idx));
                Self::write_chunk(&current_chunk, &chunk_path)?;
                chunks.push(chunk_path);
                current_chunk.clear();
                chunk_idx += 1;
            }
        }
        
        // Handle remaining lines
        if !current_chunk.is_empty() {
            current_chunk.sort();
            let chunk_path = self.temp_dir.join(format!("chunk_{}.txt", chunk_idx));
            Self::write_chunk(&current_chunk, &chunk_path)?;
            chunks.push(chunk_path);
        }
        
        Ok(chunks)
    }
    
    fn write_chunk(lines: &[String], path: &Path) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        for line in lines {
            writeln!(writer, "{}", line)?;
        }
        writer.flush()
    }
    
    fn merge_chunks(&self, chunks: &[std::path::PathBuf], output_path: &Path) -> std::io::Result<()> {
        use std::collections::BinaryHeap;
        use std::cmp::Ordering;
        
        #[derive(Eq)]
        struct ChunkItem<'a> {
            value: String,
            chunk_idx: usize,
            line: String,
        }
        
        impl<'a> PartialEq for ChunkItem<'a> {
            fn eq(&self, other: &Self) -> bool {
                self.value == other.value
            }
        }
        
        impl<'a> Ord for ChunkItem<'a> {
            fn cmp(&self, other: &Self) -> Ordering {
                other.value.cmp(&self.value)  // Reverse for min-heap
            }
        }
        
        impl<'a> PartialOrd for ChunkItem<'a> {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }
        
        let output = File::create(output_path)?;
        let mut writer = BufWriter::new(output);
        
        // Open all chunk files
        let mut readers: Vec<BufReader<File>> = Vec::new();
        for chunk in chunks {
            readers.push(BufReader::new(File::open(chunk)?));
        }
        
        // Initialize heap with first line from each chunk
        let mut heap: BinaryHeap<ChunkItem> = BinaryHeap::new();
        for (idx, reader) in readers.iter_mut().enumerate() {
            if let Some(Ok(line)) = std::io::BufRead::lines(reader).next() {
                heap.push(ChunkItem {
                    value: line.clone(),
                    chunk_idx: idx,
                    line,
                });
            }
        }
        
        // K-way merge
        while let Some(item) = heap.pop() {
            writeln!(writer, "{}", item.value)?;
            
            // Read next line from same chunk
            if let Some(Ok(next_line)) = readers[item.chunk_idx].lines().next() {
                heap.push(ChunkItem {
                    value: next_line.clone(),
                    chunk_idx: item.chunk_idx,
                    line: next_line,
                });
            }
        }
        
        writer.flush()
    }
}
```

---

## 3. Searching Algorithms

### 3.1 Binary Search Variants

```rust
// Standard binary search - returns index or None
pub fn binary_search<T: Ord>(arr: &[T], target: &T) -> Option<usize> {
    let mut left = 0;
    let mut right = arr.len();
    
    while left < right {
        let mid = left + (right - left) / 2;
        match arr[mid].cmp(target) {
            std::cmp::Ordering::Less => left = mid + 1,
            std::cmp::Ordering::Equal => return Some(mid),
            std::cmp::Ordering::Greater => right = mid,
        }
    }
    
    None
}

// Lower bound - first index where arr[i] >= target
pub fn lower_bound<T: Ord>(arr: &[T], target: &T) -> usize {
    let mut left = 0;
    let mut right = arr.len();
    
    while left < right {
        let mid = left + (right - left) / 2;
        if arr[mid] < *target {
            left = mid + 1;
        } else {
            right = mid;
        }
    }
    
    left
}

// Upper bound - first index where arr[i] > target
pub fn upper_bound<T: Ord>(arr: &[T], target: &T) -> usize {
    let mut left = 0;
    let mut right = arr.len();
    
    while left < right {
        let mid = left + (right - left) / 2;
        if arr[mid] <= *target {
            left = mid + 1;
        } else {
            right = mid;
        }
    }
    
    left
}

// Range search - returns (lower, upper) indices
pub fn equal_range<T: Ord>(arr: &[T], target: &T) -> (usize, usize) {
    (lower_bound(arr, target), upper_bound(arr, target))
}
```

### 3.2 Search on Unknown Data Structures

```rust
// Exponential search - for unbounded sorted arrays
pub fn exponential_search<T: Ord>(arr: &[T], target: &T) -> Option<usize> {
    if arr.is_empty() {
        return None;
    }
    
    // Find range
    let mut bound = 1;
    while bound < arr.len() && arr[bound] < *target {
        bound *= 2;
    }
    
    // Binary search in range
    let left = bound / 2;
    let right = bound.min(arr.len());
    
    binary_search(&arr[left..right], target).map(|i| left + i)
}

// Interpolation search - for uniformly distributed data
pub fn interpolation_search<T: Ord + std::ops::Sub<Output = T> + std::ops::Div<Output = T> + std::ops::Add<Output = T> + std::ops::Mul<Output = T> + From<u8> + Copy>(
    arr: &[T],
    target: &T,
) -> Option<usize>
where
    T: TryInto<usize>,
    <T as TryInto<usize>>::Error: std::fmt::Debug,
{
    let mut left = 0;
    let mut right = arr.len().saturating_sub(1);
    
    while left < right && arr[left] <= *target && arr[right] >= *target {
        let left_val = arr[left];
        let right_val = arr[right];
        
        // Interpolation formula
        let range = right_val - left_val;
        let target_offset = *target - left_val;
        
        // Avoid division by zero
        let pos = if range == T::from(0) {
            left
        } else {
            let pos_f = (target_offset * T::from(right - left)) / range;
            left + std::cmp::min(
                pos_f.try_into().unwrap_or(0),
                right - left
            )
        };
        
        match arr[pos].cmp(target) {
            std::cmp::Ordering::Less => left = pos + 1,
            std::cmp::Ordering::Equal => return Some(pos),
            std::cmp::Ordering::Greater => right = pos.saturating_sub(1),
        }
    }
    
    if left < arr.len() && arr[left] == *target {
        Some(left)
    } else {
        None
    }
}
```

---

## 4. Graph Algorithms

### 4.1 Graph Representations

```rust
// Adjacency List
pub struct Graph {
    pub nodes: usize,
    pub edges: Vec<Vec<usize>>,
}

impl Graph {
    pub fn new(nodes: usize) -> Self {
        Self {
            nodes,
            edges: vec![Vec::new(); nodes],
        }
    }
    
    pub fn add_edge(&mut self, from: usize, to: usize) {
        self.edges[from].push(to);
    }
    
    pub fn bfs(&self, start: usize) -> Vec<usize> {
        use std::collections::{Deque, VecDeque};
        
        let mut visited = vec![false; self.nodes];
        let mut result = Vec::new();
        let mut queue = VecDeque::from([start]);
        visited[start] = true;
        
        while let Some(node) = queue.pop_front() {
            result.push(node);
            
            for &neighbor in &self.edges[node] {
                if !visited[neighbor] {
                    visited[neighbor] = true;
                    queue.push_back(neighbor);
                }
            }
        }
        
        result
    }
    
    pub fn dfs(&self, start: usize) -> Vec<usize> {
        let mut visited = vec![false; self.nodes];
        let mut result = Vec::new();
        self.dfs_rec(start, &mut visited, &mut result);
        result
    }
    
    fn dfs_rec(&self, node: usize, visited: &mut [bool], result: &mut Vec<usize>) {
        visited[node] = true;
        result.push(node);
        
        for &neighbor in &self.edges[node] {
            if !visited[neighbor] {
                self.dfs_rec(neighbor, visited, result);
            }
        }
    }
}

// Weighted Graph
pub struct WeightedGraph {
    pub nodes: usize,
    pub edges: Vec<Vec<(usize, i64)>>,  // (neighbor, weight)
}

impl WeightedGraph {
    pub fn new(nodes: usize) -> Self {
        Self {
            nodes,
            edges: vec![Vec::new(); nodes],
        }
    }
    
    pub fn add_edge(&mut self, from: usize, to: usize, weight: i64) {
        self.edges[from].push((to, weight));
    }
    
    // Dijkstra's algorithm - O((V + E) log V)
    pub fn dijkstra(&self, start: usize, end: usize) -> Option<(i64, Vec<usize>)> {
        use std::collections::{BinaryHeap, VecDeque};
        use std::cmp::Reverse;
        
        const INF: i64 = i64::MAX;
        
        let mut dist = vec![INF; self.nodes];
        let mut prev = vec![None; self.nodes];
        let mut pq: BinaryHeap<Reverse<(i64, usize)>> = BinaryHeap::new();
        
        dist[start] = 0;
        pq.push(Reverse((0, start)));
        
        while let Some(Reverse((d, u))) = pq.pop() {
            if d > dist[u] {
                continue;
            }
            
            if u == end {
                break;
            }
            
            for &(v, weight) in &self.edges[u] {
                let new_dist = dist[u] + weight;
                if new_dist < dist[v] {
                    dist[v] = new_dist;
                    prev[v] = Some(u);
                    pq.push(Reverse((new_dist, v)));
                }
            }
        }
        
        if dist[end] == INF {
            return None;
        }
        
        // Reconstruct path
        let mut path = VecDeque::new();
        let mut current = Some(end);
        while let Some(node) = current {
            path.push_front(node);
            current = prev[node];
        }
        
        Some((dist[end], path.into_iter().collect()))
    }
    
    // Bellman-Ford - O(VE), handles negative weights
    pub fn bellman_ford(&self, start: usize) -> Option<Vec<i64>> {
        const INF: i64 = i64::MAX;
        
        let mut dist = vec![INF; self.nodes];
        dist[start] = 0;
        
        // Relax edges V-1 times
        for _ in 0..self.nodes - 1 {
            for u in 0..self.nodes {
                for &(v, weight) in &self.edges[u] {
                    if dist[u] != INF && dist[u] + weight < dist[v] {
                        dist[v] = dist[u] + weight;
                    }
                }
            }
        }
        
        // Check for negative cycles
        for u in 0..self.nodes {
            for &(v, weight) in &self.edges[u] {
                if dist[u] != INF && dist[u] + weight < dist[v] {
                    return None;  // Negative cycle detected
                }
            }
        }
        
        Some(dist)
    }
}
```

### 4.2 Topological Sort

```rust
pub fn topological_sort(graph: &[Vec<usize>]) -> Option<Vec<usize>> {
    let n = graph.len();
    let mut in_degree = vec![0; n];
    
    // Calculate in-degrees
    for edges in graph {
        for &to in edges {
            in_degree[to] += 1;
        }
    }
    
    // Start with nodes that have no incoming edges
    let mut queue: Vec<usize> = (0..n)
        .filter(|&i| in_degree[i] == 0)
        .collect();
    
    let mut result = Vec::with_capacity(n);
    
    while let Some(node) = queue.pop() {
        result.push(node);
        
        for &neighbor in &graph[node] {
            in_degree[neighbor] -= 1;
            if in_degree[neighbor] == 0 {
                queue.push(neighbor);
            }
        }
    }
    
    // If we processed all nodes, we have a valid topological order
    if result.len() == n {
        Some(result)
    } else {
        None  // Cycle detected
    }
}

// Kahn's algorithm with path reconstruction
pub fn topological_sort_kahn(graph: &[Vec<usize>]) -> Option<Vec<usize>> {
    let n = graph.len();
    let mut in_degree = vec![0; n];
    let mut result = Vec::with_capacity(n);
    
    // Calculate in-degrees
    for edges in graph {
        for &to in edges {
            in_degree[to] += 1;
        }
    }
    
    // Priority queue for deterministic ordering
    use std::collections::BinaryHeap;
    use std::cmp::Reverse;
    let mut pq: BinaryHeap<Reverse<usize>> = BinaryHeap::new();
    
    for i in 0..n {
        if in_degree[i] == 0 {
            pq.push(Reverse(i));
        }
    }
    
    while let Some(Reverse(node)) = pq.pop() {
        result.push(node);
        
        for &neighbor in &graph[node] {
            in_degree[neighbor] -= 1;
            if in_degree[neighbor] == 0 {
                pq.push(Reverse(neighbor));
            }
        }
    }
    
    if result.len() == n {
        Some(result)
    } else {
        None
    }
}
```

---

## 5. Dynamic Programming

### 5.1 Classic DP Problems

```rust
// Longest Common Subsequence
pub fn lcs(s1: &[u8], s2: &[u8]) -> (usize, Vec<u8>) {
    let m = s1.len();
    let n = s2.len();
    
    // dp[i][j] = LCS length of s1[0..i] and s2[0..j]
    let mut dp = vec![vec![0; n + 1]; m + 1];
    
    for i in 1..=m {
        for j in 1..=n {
            if s1[i - 1] == s2[j - 1] {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }
    
    // Backtrack to find actual LCS
    let mut lcs = Vec::new();
    let mut i = m;
    let mut j = n;
    
    while i > 0 && j > 0 {
        if s1[i - 1] == s2[j - 1] {
            lcs.push(s1[i - 1]);
            i -= 1;
            j -= 1;
        } else if dp[i - 1][j] > dp[i][j - 1] {
            i -= 1;
        } else {
            j -= 1;
        }
    }
    
    lcs.reverse();
    (dp[m][n], lcs)
}

// Edit Distance (Levenshtein)
pub fn edit_distance(s1: &str, s2: &str) -> usize {
    let m = s1.len();
    let n = s2.len();
    let s1 = s1.as_bytes();
    let s2 = s2.as_bytes();
    
    // dp[i][j] = minimum edits to convert s1[0..i] to s2[0..j]
    let mut dp = vec![vec![0; n + 1]; m + 1];
    
    // Base cases
    for i in 0..=m {
        dp[i][0] = i;  // Delete all characters
    }
    for j in 0..=n {
        dp[0][j] = j;  // Insert all characters
    }
    
    for i in 1..=m {
        for j in 1..=n {
            if s1[i - 1] == s2[j - 1] {
                dp[i][j] = dp[i - 1][j - 1];
            } else {
                dp[i][j] = 1 + dp[i - 1][j - 1].min(dp[i - 1][j]).min(dp[i][j - 1]);
            }
        }
    }
    
    dp[m][n]
}

// 0/1 Knapsack
pub fn knapsack(items: &[(i64, i64)], capacity: i64) -> i64 {
    // (weight, value)
    let n = items.len();
    let capacity = capacity as usize;
    
    // dp[i][w] = max value using first i items with capacity w
    let mut dp = vec![vec![0; capacity + 1]; n + 1];
    
    for i in 1..=n {
        let (weight, value) = items[i - 1];
        let weight = weight as usize;
        let value = *value;
        
        for w in 0..=capacity {
            if w >= weight {
                dp[i][w] = dp[i - 1][w].max(dp[i - 1][w - weight] + value);
            } else {
                dp[i][w] = dp[i - 1][w];
            }
        }
    }
    
    dp[n][capacity]
}

// Longest Increasing Subsequence
pub fn lis(arr: &[i64]) -> usize {
    let n = arr.len();
    if n == 0 {
        return 0;
    }
    
    // dp[i] = length of LIS ending at index i
    let mut dp = vec![1; n];
    let mut result = 1;
    
    for i in 1..n {
        for j in 0..i {
            if arr[j] < arr[i] {
                dp[i] = dp[i].max(dp[j] + 1);
            }
        }
        result = result.max(dp[i]);
    }
    
    result
}
```

### 5.2 Space-Optimized DP

```rust
// Fibonacci with O(1) space (iterative)
pub fn fibonacci(n: u64) -> u64 {
    if n <= 1 {
        return n;
    }
    
    let mut prev = 0;
    let mut curr = 1;
    
    for _ in 2..=n {
        let next = prev + curr;
        prev = curr;
        curr = next;
    }
    
    curr
}

// Fibonacci with matrix exponentiation O(log n)
pub fn fib_fast(n: u64) -> u64 {
    if n <= 1 {
        return n;
    }
    
    #[derive(Clone, Copy)]
    struct Matrix([[u64; 2]; 2]);
    
    impl Matrix {
        fn mul(self, other: Matrix) -> Matrix {
            Matrix([
                [
                    self.0[0][0] * other.0[0][0] + self.0[0][1] * other.0[1][0],
                    self.0[0][0] * other.0[0][1] + self.0[0][1] * other.0[1][1],
                ],
                [
                    self.0[1][0] * other.0[0][0] + self.0[1][1] * other.0[1][0],
                    self.0[1][0] * other.0[0][1] + self.0[1][1] * other.0[1][1],
                ],
            ])
        }
    }
    
    let base = Matrix([[1, 1], [1, 0]]);
    let mut result = Matrix([[1, 0], [0, 1]]);  // Identity
    
    let mut power = n - 1;
    let mut base = base;
    
    while power > 0 {
        if power & 1 == 1 {
            result = result.mul(base);
        }
        base = base.mul(base);
        power >>= 1;
    }
    
    result.0[0][0]
}
```

---

## 6. Probabilistic Data Structures

### 6.1 Bloom Filter

```rust
pub struct BloomFilter {
    bits: Vec<bool>,
    num_bits: usize,
    num_hashes: usize,
    hash_funcs: Vec<Box<dyn Fn(&[u8]) -> usize + Send + Sync>>,
}

impl BloomFilter {
    pub fn new(expected_items: usize, false_positive_rate: f64) -> Self {
        // m = -n * ln(p) / (ln(2)^2)
        let num_bits = ((-1.0 * expected_items as f64 * false_positive_rate.ln()) 
            / (2.0_f64.ln().powi(2))).ceil() as usize;
        
        // k = (m/n) * ln(2)
        let num_hashes = ((num_bits as f64 / expected_items as f64) * 2.0_f64.ln()).ceil() as usize;
        
        let mut hash_funcs: Vec<Box<dyn Fn(&[u8]) -> usize + Send + Sync>> = Vec::new();
        
        // Use two hash functions and generate k
        for i in 0..num_hashes {
            let i = i as u64;
            hash_funcs.push(Box::new(move |data: &[u8]| {
                // MurmurHash3-like combination
                let h1 = Self::hash64(data, 0x9e3779b9);
                let h2 = Self::hash64(data, 0x7f4a7c15);
                ((h1.wrapping_add(i.wrapping_mul(h2))) % num_bits as u64) as usize
            }));
        }
        
        Self {
            bits: vec![false; num_bits],
            num_bits,
            num_hashes,
            hash_funcs,
        }
    }
    
    fn hash64(data: &[u8], seed: u64) -> u64 {
        let mut h = seed;
        for chunk in data.chunks(8) {
            let mut buf = [0u8; 8];
            buf[..chunk.len()].copy_from_slice(chunk);
            let k = u64::from_le_bytes(buf);
            h ^= k.rotate_left(27).wrapping_mul(0x9e3779b97f4a7c15);
            h = h.rotate_left(31).wrapping_mul(5).wrapping_add(0xe6546b64);
        }
        h ^ (h >> 15).wrapping_mul(0x287b745);
        h
    }
    
    pub fn insert(&mut self, data: &[u8]) {
        for hash_func in &self.hash_funcs {
            let idx = hash_func(data);
            self.bits[idx] = true;
        }
    }
    
    pub fn contains(&self, data: &[u8]) -> bool {
        for hash_func in &self.hash_funcs {
            let idx = hash_func(data);
            if !self.bits[idx] {
                return false;
            }
        }
        true  // Possibly present
    }
}

// HyperLogLog
pub struct HyperLogLog {
    registers: Vec<u8>,  // M registers, each stores max count
    m: usize,
    alpha: f64,
}

impl HyperLogLog {
    pub fn new(precision: u8) -> Self {
        let m = 1 << precision;  // 2^precision registers
        let alpha = if m == 16 {
            0.673
        } else if m == 32 {
            0.697
        } else if m == 64 {
            0.709
        } else {
            0.7213 / (1.0 + 1.079 / m as f64)
        };
        
        Self {
            registers: vec![0; m],
            m,
            alpha,
        }
    }
    
    fn rho(w: u64) -> u8 {
        (64 - w.leading_zeros()) as u8
    }
    
    pub fn add(&mut self, data: &[u8]) {
        // Use first bits as register index, rest for count
        let hash = Self::hash64(data);
        let idx = (hash & (self.m as u64 - 1)) as usize;
        let w = hash >> (64 - self.registers[0].bit_len() as u32);
        self.registers[idx] = self.registers[idx].max(Self::rho(w));
    }
    
    pub fn count(&self) -> u64 {
        let inv_m = 1.0 / self.m as f64;
        let mut sum = 0.0;
        
        for &reg in &self.registers {
            sum += 2_f64.powi(-reg as i32);
        }
        
        let estimate = self.alpha * inv_m * inv_m / sum;
        (estimate as u64).max(self.m as u64 * 2)  // Small range correction
    }
    
    fn hash64(data: &[u8]) -> u64 {
        let mut h = 0x9e3779b97f4a7c15;
        for chunk in data.chunks(8) {
            let mut buf = [0u8; 8];
            buf[..chunk.len()].copy_from_slice(chunk);
            let k = u64::from_le_bytes(buf);
            h ^= k.rotate_left(27).wrapping_mul(0x9e3779b97f4a7c15);
            h = h.rotate_left(31).wrapping_mul(5).wrapping_add(0xe6546b64);
        }
        h ^ (h >> 15).wrapping_mul(0x287b745)
    }
}
```

---

## 7. Anti-Patterns

| Anti-Pattern | Specific Failure Mode | Prevention |
|--------------|----------------------|------------|
| **Premature optimization** | Wasted effort, complex code | Profile first, optimize the bottleneck |
| **Wrong data structure** | O(n²) when O(n log n) possible | Choose based on access patterns |
| **Brute force** | Slow, unscalable | Use appropriate algorithm |
| **Ignoring cache locality** | Poor performance | Use sequential access patterns |
| **Recursion without base case** | Stack overflow | Convert to iteration |
| **Unbounded recursion** | Stack overflow | Convert large n to iteration |
| **No early termination** | Wasted computation | Exit when answer is known |
| **Recomputing values** | Exponential time | Memoize overlapping subproblems |
| **Over-engineering** | Complex for simple case | Use simpler algorithm if adequate |
| **Floating point equality** | Incorrect comparisons | Use epsilon comparison |

---

## Links

### Core Router
- `core/DECAPOD.md` - **Router and navigation charter (START HERE)**
- `core/ENGINEERING_EXCELLENCE.md` - Engineering standards

### Architecture (This Section)
- `architecture/MEMORY.md` - Memory and cache efficiency
- `architecture/CONCURRENCY.md` - Parallel algorithms
- `architecture/DATA.md` - Data structures for storage

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
- `methodology/PERFORMANCE_OPTIMIZATION.md` - Performance patterns