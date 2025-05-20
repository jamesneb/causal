// agent/core/lib/startup/mod.rs

//! Core startup optimization module for Lambda functions and extensions
//!
//! This module provides components for optimizing startup time, 
//! especially for Lambda cold starts.

mod asm_opt;
mod lazy_init;
mod preload;
mod minimal_deps;

pub use asm_opt::{is_cold_start_asm, prefetch_critical_memory, optimized_spin_wait, fast_memcpy, hardware_crc32};
pub use lazy_init::{Lazy, ConditionalLazy, LazyComponents, env_conditional};
pub use preload::{preload_components, Preloadable, PRELOAD_COMPLETED};
pub use minimal_deps::MinimalDependencyLoader;

// Re-export lambda-specific components when the lambda feature is enabled
#[cfg(feature = "lambda")]
pub use preload::lambda::{
    LambdaRuntimePreloader, 
    NetworkPreloader, 
    LibraryPreloader, 
    AwsSdkPreloader, 
    LambdaExtensionApiPreloader
};

#[cfg(feature = "lambda")]
pub use minimal_deps::lambda::init_dependency_loader;

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Global flag to track if we're in a cold start
static IS_COLD_START: AtomicBool = AtomicBool::new(true);

/// Global tracking of startup time
static mut STARTUP_TIME: Option<Instant> = Some(Instant::now());

/// Check if this is a cold start and updates internal state
/// 
/// Uses assembly-optimized implementation when available for enhanced performance
/// during cold start.
#[inline]
pub fn is_cold_start() -> bool {
    // Use the assembly-optimized version for better performance
    asm_opt::is_cold_start_asm()
}

/// Force reset cold start flag (for testing or container reuse detection)
#[inline]
pub fn reset_cold_start() {
    IS_COLD_START.store(true, Ordering::SeqCst);
}

/// Get time elapsed since process startup
pub fn time_since_startup() -> Duration {
    unsafe {
        STARTUP_TIME.as_ref()
            .expect("Startup time not initialized")
            .elapsed()
    }
}

/// Critical items that must be initialized during cold start
pub trait CriticalInitialization {
    /// Perform critical initialization
    fn initialize(&self);
    
    /// Get component name
    fn name(&self) -> &'static str;
}

/// Startup optimizer that controls initialization order and timing
pub struct StartupOptimizer {
    /// Critical components that must be initialized immediately
    critical_components: Vec<Box<dyn CriticalInitialization>>,
    /// Non-critical components that can be initialized lazily
    lazy_components: LazyComponents<Box<dyn std::any::Any>>,
    /// Whether to log detailed timing
    log_timing: bool,
}

impl StartupOptimizer {
    /// Create a new startup optimizer
    pub fn new() -> Self {
        Self {
            critical_components: Vec::new(),
            lazy_components: LazyComponents::new(),
            log_timing: cfg!(debug_assertions),
        }
    }
    
    /// Add a critical component that must be initialized during cold start
    pub fn add_critical<T: CriticalInitialization + 'static>(&mut self, component: T) -> &mut Self {
        self.critical_components.push(Box::new(component));
        self
    }
    
    /// Add a component for lazy initialization
    pub fn add_lazy<T: 'static, F>(&mut self, component: Lazy<T, F>) -> &mut Self 
    where
        F: FnOnce() -> T + 'static,
        T: 'static,
    {
        let boxed_component = Lazy::new(
            component.name,
            move || Box::new(component.get().clone()) as Box<dyn std::any::Any>,
        );
        
        self.lazy_components.add(boxed_component);
        self
    }
    
    /// Enable or disable detailed timing logs
    pub fn with_timing_logs(mut self, enable: bool) -> Self {
        self.log_timing = enable;
        self
    }
    
    /// Initialize only critical components (for cold start)
    pub fn initialize_critical(&self) {
        let start = Instant::now();
        
        for component in &self.critical_components {
            let component_start = Instant::now();
            debug!("Initializing critical component '{}'", component.name());
            
            component.initialize();
            
            if self.log_timing {
                let duration = component_start.elapsed();
                debug!("Component '{}' initialized in {:?}", component.name(), duration);
            }
        }
        
        if self.log_timing {
            let duration = start.elapsed();
            info!(
                "Initialized {} critical components in {:?}", 
                self.critical_components.len(),
                duration
            );
        }
    }
    
    /// Get total cold start duration
    pub fn cold_start_duration(&self) -> Duration {
        time_since_startup()
    }
    
    /// Log cold start metrics
    pub fn log_cold_start_metrics(&self) {
        if is_cold_start() {
            let duration = self.cold_start_duration();
            info!("Cold start complete in {:?}", duration);
            
            // Log lazy initialization metrics
            self.lazy_components.log_metrics();
        }
    }
}

/// Detection of Lambda freeze/thaw cycles
pub struct FreezeThawDetector {
    /// Last activity timestamp
    last_activity: std::sync::atomic::AtomicU64,
    /// Threshold for detecting freezes
    freeze_threshold_ms: u64,
}

impl FreezeThawDetector {
    /// Create a new detector with the specified threshold
    pub fn new(freeze_threshold_ms: u64) -> Self {
        Self {
            last_activity: std::sync::atomic::AtomicU64::new(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64
            ),
            freeze_threshold_ms,
        }
    }
    
    /// Update the activity timestamp
    pub fn record_activity(&self) {
        self.last_activity.store(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            Ordering::SeqCst
        );
    }
    
    /// Check if the function was likely frozen and thawed
    pub fn was_frozen(&self) -> bool {
        let last = self.last_activity.load(Ordering::SeqCst);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
            
        let elapsed = now - last;
        
        if elapsed > self.freeze_threshold_ms {
            // If we were likely frozen, update the activity and notify
            self.record_activity();
            debug!("Detected likely Lambda freeze/thaw cycle after {} ms", elapsed);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    
    struct TestComponent {
        name: &'static str,
        initialized: Arc<AtomicUsize>,
    }
    
    impl TestComponent {
        fn new(name: &'static str, initialized: Arc<AtomicUsize>) -> Self {
            initialized.fetch_add(1, Ordering::SeqCst);
            Self { name, initialized }
        }
    }
    
    impl CriticalInitialization for TestComponent {
        fn initialize(&self) {
            // Already initialized in constructor
        }
        
        fn name(&self) -> &'static str {
            self.name
        }
    }
    
    impl Clone for TestComponent {
        fn clone(&self) -> Self {
            self.initialized.fetch_add(1, Ordering::SeqCst);
            Self {
                name: self.name,
                initialized: self.initialized.clone(),
            }
        }
    }
    
    #[test]
    fn test_cold_start_detection() {
        assert!(is_cold_start(), "First call should be a cold start");
        assert!(!is_cold_start(), "Second call should not be a cold start");
        
        reset_cold_start();
        assert!(is_cold_start(), "After reset should be a cold start");
    }
    
    #[test]
    fn test_startup_optimizer() {
        let critical_init = Arc::new(AtomicUsize::new(0));
        let lazy_init = Arc::new(AtomicUsize::new(0));
        
        let critical_clone = critical_init.clone();
        let lazy_clone = lazy_init.clone();
        
        let mut optimizer = StartupOptimizer::new();
        
        // Add critical component
        optimizer.add_critical(TestComponent::new("critical", critical_clone));
        
        // Add lazy component
        optimizer.add_lazy(Lazy::new(
            "lazy",
            move || TestComponent::new("lazy", lazy_clone),
        ));
        
        // Initialize critical components
        optimizer.initialize_critical();
        
        // Check initialization state
        assert_eq!(critical_init.load(Ordering::SeqCst), 1, "Critical component should be initialized");
        assert_eq!(lazy_init.load(Ordering::SeqCst), 0, "Lazy component should not be initialized yet");
    }
    
    #[test]
    fn test_freeze_thaw_detection() {
        let detector = FreezeThawDetector::new(100);
        
        // Should not be frozen initially
        assert!(!detector.was_frozen());
        
        // Manipulate the last activity time to simulate a freeze
        detector.last_activity.store(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64 - 200,
            Ordering::SeqCst
        );
        
        // Now should detect freeze
        assert!(detector.was_frozen());
        
        // Should not detect freeze again immediately
        assert!(!detector.was_frozen());
    }
}
