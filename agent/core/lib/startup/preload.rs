// agent/core/lib/startup/preload.rs

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

// Import ASM optimizations
use crate::startup::asm_opt::{prefetch_critical_memory, optimized_spin_wait};

/// Components that perform prefetching of resources
pub trait Preloadable {
    /// Preload any resources needed by this component
    fn preload(&self);
    
    /// Get the name of this component for logging
    fn name(&self) -> &'static str;
    
    /// Estimated memory footprint of preloaded resources (bytes)
    fn estimated_memory(&self) -> usize {
        0 // Default implementation returns 0
    }
}

// Global preload state
pub static PRELOAD_COMPLETED: AtomicBool = AtomicBool::new(false);

/// Perform resource preloading for the specified components
pub fn preload_components<'a>(
    components: impl IntoIterator<Item = &'a (dyn Preloadable + 'a)>,
    memory_limit_mb: Option<usize>,
) {
    // Only preload once
    if PRELOAD_COMPLETED.swap(true, Ordering::SeqCst) {
        debug!("Preload already performed, skipping");
        return;
    }
    
    let start = Instant::now();
    let mut loaded_components = Vec::new();
    let mut total_memory: usize = 0;
    let memory_limit = memory_limit_mb.map(|mb| mb * 1024 * 1024);
    
    // Convert to Vec first to allow prefetching
    let components_vec: Vec<&dyn Preloadable> = components.into_iter().collect();
    
    // Prefetch component metadata for all components before starting
    for component in &components_vec {
        unsafe {
            // Prefetch component metadata to ensure it's in CPU cache
            prefetch_critical_memory(
                *component as *const _ as *const u8,
                std::mem::size_of_val(*component),
            );
        }
    }
    
    // Short spin-wait to ensure prefetch completes
    optimized_spin_wait(100);
    
    for component in components_vec {
        let component_start = Instant::now();
        let name = component.name();
        
        // Check if we're exceeding memory limit
        let estimated_memory = component.estimated_memory();
        if let Some(limit) = memory_limit {
            if total_memory + estimated_memory > limit {
                warn!(
                    "Skipping preload of '{}' to stay under memory limit ({} bytes)",
                    name,
                    limit
                );
                continue;
            }
        }
        
        // Prefetch the component implementation 
        unsafe {
            // Prefetch component name
            prefetch_critical_memory(
                name.as_ptr(),
                name.len(),
            );
        }
        
        // Perform preload
        debug!("Preloading component '{}'", name);
        component.preload();
        
        // Update tracking
        let duration = component_start.elapsed();
        total_memory += estimated_memory;
        loaded_components.push((name, duration, estimated_memory));
        
        debug!("Component '{}' preloaded in {:?}", name, duration);
    }
    
    let duration = start.elapsed();
    info!(
        "Preloaded {} components in {:?}, using approximately {} KB",
        loaded_components.len(),
        duration,
        total_memory / 1024
    );
    
    if duration > Duration::from_millis(100) {
        // Log details for slow preloads
        for (name, duration, memory) in loaded_components {
            debug!(
                "Component '{}' preload: {:?}, {} KB",
                name,
                duration,
                memory / 1024
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    
    struct TestPreloadable {
        name: &'static str,
        memory_usage: usize,
        preload_count: Arc<AtomicBool>,
    }
    
    impl TestPreloadable {
        fn new(name: &'static str, memory_usage: usize) -> Self {
            Self {
                name,
                memory_usage,
                preload_count: Arc::new(AtomicBool::new(false)),
            }
        }
    }
    
    impl Preloadable for TestPreloadable {
        fn preload(&self) {
            self.preload_count.store(true, Ordering::SeqCst);
        }
        
        fn name(&self) -> &'static str {
            self.name
        }
        
        fn estimated_memory(&self) -> usize {
            self.memory_usage
        }
    }
    
    #[test]
    fn test_preload_components() {
        // Reset global state for test
        PRELOAD_COMPLETED.store(false, Ordering::SeqCst);
        
        let component1 = TestPreloadable::new("test1", 1024);
        let component2 = TestPreloadable::new("test2", 1024);
        
        let components: Vec<&dyn Preloadable> = vec![&component1, &component2];
        
        // Preload with no memory limit
        preload_components(components.clone(), None);
        
        assert!(component1.preload_count.load(Ordering::SeqCst));
        assert!(component2.preload_count.load(Ordering::SeqCst));
        
        // Reset for memory limit test
        PRELOAD_COMPLETED.store(false, Ordering::SeqCst);
        component1.preload_count.store(false, Ordering::SeqCst);
        component2.preload_count.store(false, Ordering::SeqCst);
        
        let big_component = TestPreloadable::new("big", 1024 * 1024 * 10); // 10 MB
        let small_component = TestPreloadable::new("small", 1024); // 1 KB
        
        let limited_components: Vec<&dyn Preloadable> = vec![&big_component, &small_component];
        
        // Preload with a 5 MB memory limit
        preload_components(limited_components, Some(5));
        
        // Only the small component should be preloaded
        assert!(!big_component.preload_count.load(Ordering::SeqCst));
        assert!(small_component.preload_count.load(Ordering::SeqCst));
    }
}