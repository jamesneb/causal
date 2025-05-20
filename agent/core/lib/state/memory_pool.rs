// agent/core/lib/state/memory_pool.rs

use std::sync::atomic::{AtomicUsize, Ordering};
use std::marker::PhantomData;
use anyhow::{Context, Result};

// Memory pool configuration
pub const DEFAULT_POOL_SIZE: usize = 1024 * 1024 * 10; // 10MB
pub const MIN_POOL_SIZE: usize = 1024;                 // 1KB
pub const ALIGNMENT: usize = 16;                       // 16-byte alignment

// Memory pool for efficient reuse of allocations
pub struct MemoryPool {
    buffer: *mut u8,
    buffer_size: usize,
    next_free: AtomicUsize,
    max_used: AtomicUsize,
}

// RAII guard for pool allocations
pub struct PoolAllocation<'a> {
    ptr: *mut u8,
    size: usize,
    pool: &'a MemoryPool,
}

unsafe impl Send for MemoryPool {}
unsafe impl Sync for MemoryPool {}

impl MemoryPool {
    // Create a new memory pool with the specified size
    pub fn new(size: usize) -> Result<Self> {
        let buffer_size = size.max(MIN_POOL_SIZE);
        
        // Allocate aligned memory
        let layout = std::alloc::Layout::from_size_align(buffer_size, ALIGNMENT)
            .context("Invalid memory layout")?;
        
        let buffer = unsafe { std::alloc::alloc(layout) };
        if buffer.is_null() {
            return Err(anyhow::anyhow!("Failed to allocate memory pool"));
        }
        
        Ok(Self {
            buffer,
            buffer_size,
            next_free: AtomicUsize::new(0),
            max_used: AtomicUsize::new(0),
        })
    }
    
    // Allocate memory from the pool
    pub fn allocate<'a>(&'a self, size: usize) -> Result<PoolAllocation<'a>> {
        // Round up to alignment
        let aligned_size = (size + ALIGNMENT - 1) & !(ALIGNMENT - 1);
        
        // Atomically reserve space
        let offset = self.next_free.fetch_add(aligned_size, Ordering::SeqCst);
        
        // Check if we have enough space
        if offset + aligned_size > self.buffer_size {
            // Reset if we're out of memory
            self.next_free.store(0, Ordering::SeqCst);
            return Err(anyhow::anyhow!("Memory pool exhausted"));
        }
        
        // Track max usage
        let mut current_max = self.max_used.load(Ordering::Relaxed);
        while offset + aligned_size > current_max {
            match self.max_used.compare_exchange_weak(
                current_max,
                offset + aligned_size,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(new_max) => current_max = new_max,
            }
        }
        
        // Return allocation
        Ok(PoolAllocation {
            ptr: unsafe { self.buffer.add(offset) },
            size: aligned_size,
            pool: self,
        })
    }
    
    // Reset the pool
    pub fn reset(&self) {
        self.next_free.store(0, Ordering::SeqCst);
    }
    
    // Get current usage statistics
    pub fn usage(&self) -> (usize, usize) {
        (
            self.next_free.load(Ordering::Relaxed),
            self.max_used.load(Ordering::Relaxed),
        )
    }
}

impl<'a> PoolAllocation<'a> {
    // Get raw pointer
    pub fn as_ptr(&self) -> *mut u8 {
        self.ptr
    }
    
    // Get slice to the allocated memory
    pub fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr, self.size) }
    }
    
    // Get mutable slice to the allocated memory
    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr, self.size) }
    }
    
    // Get allocation size
    pub fn size(&self) -> usize {
        self.size
    }
}

impl Drop for MemoryPool {
    fn drop(&mut self) {
        if !self.buffer.is_null() {
            let layout = std::alloc::Layout::from_size_align(self.buffer_size, ALIGNMENT)
                .expect("Invalid memory layout");
            unsafe {
                std::alloc::dealloc(self.buffer, layout);
            }
        }
    }
}

// Type-safe memory pool for specific types
pub struct TypedMemoryPool<T> {
    pool: MemoryPool,
    phantom: PhantomData<T>,
}

// RAII guard for typed allocations
pub struct TypedAllocation<'a, T> {
    allocation: PoolAllocation<'a>,
    phantom: PhantomData<T>,
}

impl<T> TypedMemoryPool<T> {
    // Create a new typed memory pool
    pub fn new(count: usize) -> Result<Self> {
        let size = std::mem::size_of::<T>() * count;
        Ok(Self {
            pool: MemoryPool::new(size)?,
            phantom: PhantomData,
        })
    }
    
    // Allocate a typed object
    pub fn allocate<'a>(&'a self) -> Result<TypedAllocation<'a, T>> {
        let allocation = self.pool.allocate(std::mem::size_of::<T>())?;
        Ok(TypedAllocation {
            allocation,
            phantom: PhantomData,
        })
    }
    
    // Reset the pool
    pub fn reset(&self) {
        self.pool.reset();
    }
    
    // Get usage statistics
    pub fn usage(&self) -> (usize, usize) {
        self.pool.usage()
    }
}

impl<'a, T> TypedAllocation<'a, T> {
    // Get reference to the typed object
    pub fn as_ref(&self) -> &T {
        let ptr = self.allocation.as_ptr() as *const T;
        unsafe { &*ptr }
    }
    
    // Get mutable reference to the typed object
    pub fn as_mut(&mut self) -> &mut T {
        let ptr = self.allocation.as_ptr() as *mut T;
        unsafe { &mut *ptr }
    }
    
    // Initialize with a value
    pub fn initialize(&mut self, value: T) {
        let ptr = self.allocation.as_ptr() as *mut T;
        unsafe {
            std::ptr::write(ptr, value);
        }
    }
}

impl<'a, T> Drop for TypedAllocation<'a, T> {
    fn drop(&mut self) {
        // Drop any initialized value
        let ptr = self.allocation.as_ptr() as *mut T;
        unsafe {
            std::ptr::drop_in_place(ptr);
        }
    }
}
