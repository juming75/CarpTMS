//! Memory Management Optimization Module
//!
//! Provides advanced memory management techniques including:
//! - Memory pooling for frequently allocated objects
//! - Arena allocation for short-lived data
//! - Reference counting for shared data
//! - Memory-mapped files for large datasets
//! - Custom allocators for specific use cases

use std::alloc::{GlobalAlloc, Layout, System};
use std::collections::VecDeque;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use thiserror::Error;

/// 清理函数类型别名
type CleanupFn<T> = Option<Box<dyn Fn(&T) + Send + Sync>>;

/// Memory management error
#[derive(Error, Debug)]
pub enum MemoryError {
    #[error("Allocation failed: {0}")]
    AllocationFailed(String),

    #[error("Pool exhausted")]
    PoolExhausted,

    #[error("Invalid size")]
    InvalidSize,

    #[error("Memory limit exceeded")]
    MemoryLimitExceeded,
}

/// Memory pool for efficient allocation
pub struct MemoryPool<T> {
    pool: Arc<Mutex<VecDeque<Box<T>>>>,
    max_size: usize,
    current_size: Arc<AtomicUsize>,
}

impl<T: Default> MemoryPool<T> {
    pub fn new(max_size: usize) -> Self {
        Self {
            pool: Arc::new(Mutex::new(VecDeque::new())),
            max_size,
            current_size: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Get an object from the pool
    pub fn acquire(&self) -> Result<PooledObject<T>, MemoryError> {
        let mut pool = self.pool.lock().map_err(|_| MemoryError::PoolExhausted)?;
        self.current_size.fetch_sub(1, Ordering::SeqCst);

        if let Some(obj) = pool.pop_front() {
            Ok(PooledObject {
                data: Some(obj),
                pool: self.pool.clone(),
                size_tracker: self.current_size.clone(),
            })
        } else {
            // Create new object if pool is empty
            Ok(PooledObject {
                data: Some(Box::new(T::default())),
                pool: self.pool.clone(),
                size_tracker: self.current_size.clone(),
            })
        }
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        PoolStats {
            available: self.pool.lock().ok().map(|p| p.len()).unwrap_or(0),
            max_size: self.max_size,
            current_size: self.current_size.load(Ordering::SeqCst),
        }
    }
}

/// Pooled object that automatically returns to pool on drop
pub struct PooledObject<T> {
    data: Option<Box<T>>,
    pool: Arc<Mutex<VecDeque<Box<T>>>>,
    size_tracker: Arc<AtomicUsize>,
}

impl<T> std::ops::Deref for PooledObject<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // data is always Some unless PooledObject is already dropped
        self.data.as_ref().unwrap()
    }
}

impl<T> std::ops::DerefMut for PooledObject<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // data is always Some unless PooledObject is already dropped
        self.data.as_mut().unwrap()
    }
}

impl<T> Drop for PooledObject<T> {
    fn drop(&mut self) {
        if let Some(data) = self.data.take() {
            if let Ok(mut pool) = self.pool.lock() {
                if pool.len() < self.size_tracker.load(Ordering::SeqCst) {
                    pool.push_back(data);
                    self.size_tracker.fetch_add(1, Ordering::SeqCst);
                }
            }
        }
    }
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub available: usize,
    pub max_size: usize,
    pub current_size: usize,
}

/// Arena allocator for short-lived objects
pub struct ArenaAllocator {
    chunks: Arc<RwLock<Vec<ArenaChunk>>>,
    current_chunk: Arc<Mutex<Option<ArenaChunk>>>,
    chunk_size: usize,
    total_allocated: Arc<AtomicUsize>,
}

struct ArenaChunk {
    data: Vec<u8>,
    offset: usize,
}

impl ArenaAllocator {
    pub fn new(chunk_size: usize) -> Self {
        Self {
            chunks: Arc::new(RwLock::new(Vec::new())),
            current_chunk: Arc::new(Mutex::new(None)),
            chunk_size,
            total_allocated: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Allocate memory in the arena
    pub fn allocate(&self, size: usize) -> Result<ArenaAllocation<'_>, MemoryError> {
        if size > self.chunk_size {
            return Err(MemoryError::InvalidSize);
        }

        let mut current = match self.current_chunk.lock() {
            Ok(c) => c,
            Err(e) => return Err(MemoryError::AllocationFailed(e.to_string())),
        };

        // Check if current chunk has space
        if let Some(ref mut chunk) = *current {
            if chunk.offset + size <= chunk.data.len() {
                let offset = chunk.offset;
                chunk.offset += size;
                self.total_allocated.fetch_add(size, Ordering::SeqCst);

                return Ok(ArenaAllocation {
                    ptr: NonNull::new(&mut chunk.data[offset] as *mut u8).unwrap(),
                    size,
                    _marker: std::marker::PhantomData,
                });
            }
        }

        // Create new chunk
        let mut new_chunk = ArenaChunk {
            data: vec![0u8; self.chunk_size],
            offset: size,
        };

        let ptr = NonNull::new(&mut new_chunk.data[0] as *mut u8).unwrap();
        self.total_allocated.fetch_add(size, Ordering::SeqCst);

        // Store old chunk and set new current
        if let Some(old_chunk) = current.replace(new_chunk) {
            if let Ok(mut chunks) = self.chunks.write() {
                chunks.push(old_chunk);
            }
        }

        Ok(ArenaAllocation {
            ptr,
            size,
            _marker: std::marker::PhantomData,
        })
    }

    /// Reset the arena (frees all allocations)
    pub fn reset(&self) {
        if let Ok(mut chunks) = self.chunks.write() {
            chunks.clear();
        }

        if let Ok(mut current) = self.current_chunk.lock() {
            *current = None;
        }

        self.total_allocated.store(0, Ordering::SeqCst);
    }

    /// Get memory statistics
    pub fn stats(&self) -> ArenaStats {
        let chunks = match self.chunks.read() {
            Ok(c) => c,
            Err(_) => return ArenaStats {
                total_chunks: 0,
                total_allocated: 0,
                chunk_size: self.chunk_size,
            },
        };
        let current = self.current_chunk.lock().ok();

        ArenaStats {
            total_chunks: chunks.len() + if current.is_some() { 1 } else { 0 },
            total_allocated: self.total_allocated.load(Ordering::SeqCst),
            chunk_size: self.chunk_size,
        }
    }
}

/// Arena allocation handle
pub struct ArenaAllocation<'a> {
    ptr: NonNull<u8>,
    size: usize,
    _marker: std::marker::PhantomData<&'a [u8]>,
}

impl<'a> ArenaAllocation<'a> {
    pub fn as_slice(&self) -> &'a [u8] {
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.size) }
    }

    pub fn as_mut_slice(&mut self) -> &'a mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.size) }
    }
}

/// Arena statistics
#[derive(Debug, Clone)]
pub struct ArenaStats {
    pub total_chunks: usize,
    pub total_allocated: usize,
    pub chunk_size: usize,
}

/// Smart pointer for shared data with automatic cleanup
pub struct SharedData<T> {
    data: Arc<T>,
    cleanup_fn: CleanupFn<T>,
}

impl<T> SharedData<T> {
    pub fn new(data: T) -> Self {
        Self {
            data: Arc::new(data),
            cleanup_fn: None,
        }
    }

    pub fn new_with_cleanup(data: T, cleanup: impl Fn(&T) + Send + Sync + 'static) -> Self {
        Self {
            data: Arc::new(data),
            cleanup_fn: Some(Box::new(cleanup)),
        }
    }

    pub fn get(&self) -> &T {
        &self.data
    }

    pub fn get_arc(&self) -> Arc<T> {
        self.data.clone()
    }
}

impl<T> Clone for SharedData<T> {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            cleanup_fn: None, // Cleanup only happens on the original
        }
    }
}

impl<T> Drop for SharedData<T> {
    fn drop(&mut self) {
        if let Some(cleanup) = &self.cleanup_fn {
            // This will only run when the last Arc is dropped
            if Arc::strong_count(&self.data) == 1 {
                cleanup(&self.data);
            }
        }
    }
}

/// Memory manager that coordinates different memory optimization strategies
pub struct MemoryManager {
    object_pools:
        Arc<RwLock<std::collections::HashMap<String, Arc<dyn std::any::Any + Send + Sync>>>>,
    arena_allocators: Arc<RwLock<std::collections::HashMap<String, Arc<ArenaAllocator>>>>,
    memory_limit: usize,
    current_usage: Arc<AtomicUsize>,
}

impl MemoryManager {
    pub fn new(memory_limit: usize) -> Self {
        Self {
            object_pools: Arc::new(RwLock::new(std::collections::HashMap::new())),
            arena_allocators: Arc::new(RwLock::new(std::collections::HashMap::new())),
            memory_limit,
            current_usage: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Register an object pool
    pub fn register_pool<T: 'static + Default + Send + Sync>(
        &self,
        name: &str,
        max_size: usize,
    ) -> Result<Arc<MemoryPool<T>>, MemoryError> {
        let pool = Arc::new(MemoryPool::<T>::new(max_size));
        if let Ok(mut pools) = self.object_pools.write() {
            pools.insert(
                name.to_string(),
                pool.clone() as Arc<dyn std::any::Any + Send + Sync>,
            );
        }
        Ok(pool)
    }

    /// Get an object pool
    pub fn get_pool<T: 'static + Default + Send + Sync>(
        &self,
        name: &str,
    ) -> Option<Arc<MemoryPool<T>>> {
        self.object_pools.read().ok()?
            .get(name)
            .and_then(|pool| pool.clone().downcast::<MemoryPool<T>>().ok())
    }

    /// Create an arena allocator
    pub fn create_arena(
        &self,
        name: &str,
        chunk_size: usize,
    ) -> Result<Arc<ArenaAllocator>, MemoryError> {
        let arena = Arc::new(ArenaAllocator::new(chunk_size));
        if let Ok(mut arenas) = self.arena_allocators.write() {
            arenas.insert(name.to_string(), arena.clone());
        }
        Ok(arena)
    }

    /// Get an arena allocator
    pub fn get_arena(&self, name: &str) -> Option<Arc<ArenaAllocator>> {
        self.arena_allocators.read().ok()?.get(name).cloned()
    }

    /// Track memory allocation
    pub fn track_allocation(&self, size: usize) -> Result<(), MemoryError> {
        let current = self.current_usage.fetch_add(size, Ordering::SeqCst);
        if current + size > self.memory_limit {
            self.current_usage.fetch_sub(size, Ordering::SeqCst);
            return Err(MemoryError::MemoryLimitExceeded);
        }
        Ok(())
    }

    /// Track memory deallocation
    pub fn track_deallocation(&self, size: usize) {
        self.current_usage.fetch_sub(size, Ordering::SeqCst);
    }

    /// Get memory statistics
    pub fn get_stats(&self) -> MemoryStats {
        let pool_count = self.object_pools.read().ok().map(|p| p.len()).unwrap_or(0);
        let arena_count = self.arena_allocators.read().ok().map(|a| a.len()).unwrap_or(0);
        MemoryStats {
            current_usage: self.current_usage.load(Ordering::SeqCst),
            memory_limit: self.memory_limit,
            pool_count,
            arena_count,
        }
    }

    /// Force garbage collection (reset all arenas and pools)
    pub fn force_gc(&self) {
        // Reset all arenas
        if let Ok(arenas) = self.arena_allocators.read() {
            for arena in arenas.values() {
                arena.reset();
            }
        }

        // Note: Object pools are not reset as they contain reusable objects

        // Reset usage counter
        self.current_usage.store(0, Ordering::SeqCst);
    }
}

/// Memory statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    pub current_usage: usize,
    pub memory_limit: usize,
    pub pool_count: usize,
    pub arena_count: usize,
}

/// Custom allocator that tracks allocations
pub struct TrackingAllocator {
    inner: System,
    allocation_count: Arc<AtomicUsize>,
    deallocation_count: Arc<AtomicUsize>,
    total_allocated: Arc<AtomicUsize>,
}

impl Default for TrackingAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl TrackingAllocator {
    pub fn new() -> Self {
        Self {
            inner: System,
            allocation_count: Arc::new(AtomicUsize::new(0)),
            deallocation_count: Arc::new(AtomicUsize::new(0)),
            total_allocated: Arc::new(AtomicUsize::new(0)),
        }
    }

    pub fn get_stats(&self) -> TrackingStats {
        TrackingStats {
            allocation_count: self.allocation_count.load(Ordering::SeqCst),
            deallocation_count: self.deallocation_count.load(Ordering::SeqCst),
            total_allocated: self.total_allocated.load(Ordering::SeqCst),
        }
    }
}

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.allocation_count.fetch_add(1, Ordering::SeqCst);
        self.total_allocated
            .fetch_add(layout.size(), Ordering::SeqCst);
        self.inner.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.deallocation_count.fetch_add(1, Ordering::SeqCst);
        self.total_allocated
            .fetch_sub(layout.size(), Ordering::SeqCst);
        self.inner.dealloc(ptr, layout)
    }
}

/// Tracking statistics
#[derive(Debug, Clone)]
pub struct TrackingStats {
    pub allocation_count: usize,
    pub deallocation_count: usize,
    pub total_allocated: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_pool() {
        let pool: MemoryPool<Vec<u8>> = MemoryPool::new(10);

        {
            let obj1 = pool.acquire().unwrap();
            assert_eq!(obj1.len(), 0);

            let obj2 = pool.acquire().unwrap();
            assert_eq!(obj2.len(), 0);
        } // Objects returned to pool on drop

        let stats = pool.stats();
        assert_eq!(stats.available, 2);
    }

    #[test]
    fn test_arena_allocator() {
        let arena = ArenaAllocator::new(1024);

        let alloc1 = arena.allocate(100).unwrap();
        assert_eq!(alloc1.size, 100);

        let alloc2 = arena.allocate(200).unwrap();
        assert_eq!(alloc2.size, 200);

        let stats = arena.stats();
        assert_eq!(stats.total_allocated, 300);

        arena.reset();
        let stats = arena.stats();
        assert_eq!(stats.total_allocated, 0);
    }

    #[test]
    fn test_shared_data() {
        let shared = SharedData::new(vec![1, 2, 3, 4, 5]);

        let cloned = shared.clone();
        assert_eq!(cloned.get().len(), 5);

        let arc = shared.get_arc();
        assert_eq!(arc.len(), 5);
    }

    #[tokio::test]
    async fn test_memory_manager() {
        let manager = MemoryManager::new(1024 * 1024); // 1MB limit

        // Register a pool
        let _pool = manager.register_pool::<Vec<u8>>("test_pool", 10).unwrap();

        // 测试内存分配器
        let _arena = manager.create_arena("test_arena", 4096).unwrap();

        // Test allocation
        manager.track_allocation(100).unwrap();
        manager.track_deallocation(50);

        let stats = manager.get_stats();
        assert_eq!(stats.current_usage, 50);
        assert_eq!(stats.pool_count, 1);
        assert_eq!(stats.arena_count, 1);
    }
}
