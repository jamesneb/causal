// agent/core/lib/startup/lazy_init.rs

use std::sync::{Arc, Mutex, Once};
use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::time::{Duration, Instant};
use tracing::{debug, info, trace};

/// Metrics tracking startup initialization
struct LazyInitMetrics {
    /// Total initialization time
    total_init_time_ns: u64,
    /// Number of components initialized
    components_initialized: usize,
    /// Number of deferred initializations
    deferred_count: usize,
    /// Track component timing
    component_timings: Vec<(String, u64)>,
}

impl LazyInitMetrics {
    fn new() -> Self {
        Self {
            total_init_time_ns: 0,
            components_initialized: 0,
            deferred_count: 0,
            component_timings: Vec::new(),
        }
    }
    
    fn record_init(&mut self, component: &str, duration_ns: u64) {
        self.total_init_time_ns += duration_ns;
        self.components_initialized += 1;
        self.component_timings.push((component.to_string(), duration_ns));
    }
    
    fn increment_deferred(&mut self) {
        self.deferred_count += 1;
    }
    
    fn log_metrics(&self) {
        info!(
            "Lazy initialization metrics: {} components initialized, {} deferred, total time: {} µs",
            self.components_initialized,
            self.deferred_count,
            self.total_init_time_ns / 1_000
        );
        
        if debug_enabled() {
            // Sort by duration for easy analysis
            let mut timings = self.component_timings.clone();
            timings.sort_by(|a, b| b.1.cmp(&a.1));
            
            for (component, duration) in timings {
                debug!(
                    "Component '{}' initialization took {} µs",
                    component,
                    duration / 1_000
                );
            }
        }
    }
}

// Global metrics singleton
static INIT_METRICS: Mutex<LazyInitMetrics> = Mutex::new(LazyInitMetrics::new());

// Helper to check if debug logging is enabled
#[inline]
fn debug_enabled() -> bool {
    cfg!(debug_assertions) || std::env::var("DEBUG").unwrap_or_default() == "1"
}

/// Lazy initialization container that defers expensive initialization
/// until the component is actually needed
pub struct Lazy<T, F> {
    /// Initialization function
    init: UnsafeCell<Option<F>>,
    /// The value once initialized
    value: UnsafeCell<Option<T>>,
    /// Initialization flag
    init_once: Once,
    /// Component name for metrics
    name: &'static str,
    /// Dummy marker
    _marker: PhantomData<T>,
}

// Lazy<T> can be shared between threads as long as T is Send and Sync
unsafe impl<T, F> Send for Lazy<T, F> where T: Send {}
unsafe impl<T, F> Sync for Lazy<T, F> where T: Send + Sync, F: Send {}

impl<T, F> Lazy<T, F>
where
    F: FnOnce() -> T,
{
    /// Create a new lazy-initialized component
    pub const fn new(name: &'static str, init: F) -> Self {
        Self {
            init: UnsafeCell::new(Some(init)),
            value: UnsafeCell::new(None),
            init_once: Once::new(),
            name,
            _marker: PhantomData,
        }
    }
    
    /// Force initialization immediately
    pub fn initialize(&self) -> &T {
        self.init_once.call_once(|| {
            // Take the initialization function
            let init = unsafe {
                (*self.init.get()).take().expect("Lazy initialization function called twice")
            };
            
            // Record metrics for this initialization
            let start = Instant::now();
            trace!("Initializing component '{}'", self.name);
            
            // Run initialization
            let value = init();
            
            // Record duration
            let duration = start.elapsed().as_nanos() as u64;
            
            // Store value
            unsafe {
                *self.value.get() = Some(value);
            }
            
            // Update metrics
            let mut metrics = INIT_METRICS.lock().unwrap();
            metrics.record_init(self.name, duration);
            
            debug!("Component '{}' initialized in {} µs", self.name, duration / 1_000);
        });
        
        // Return reference to the value
        unsafe { (*self.value.get()).as_ref().unwrap() }
    }
    
    /// Check if the value has been initialized
    pub fn is_initialized(&self) -> bool {
        let mut initialized = false;
        self.init_once.call_once_force(|_| {
            initialized = true; // Will only be set to true if this is the first call
            
            // Record deferred initialization
            let mut metrics = INIT_METRICS.lock().unwrap();
            metrics.increment_deferred();
        });
        !initialized
    }
    
    /// Get a reference to the value, initializing if necessary
    pub fn get(&self) -> &T {
        self.initialize()
    }
}

/// A container that builds and manages a set of lazy-initialized components
pub struct LazyComponents<T> {
    components: Vec<Box<dyn LazyComponent<Output = T>>>,
    component_names: Vec<&'static str>,
}

/// Trait for lazy component initialization
pub trait LazyComponent {
    type Output;
    
    fn name(&self) -> &'static str;
    fn is_initialized(&self) -> bool;
    fn initialize(&self) -> &Self::Output;
}

impl<T, F> LazyComponent for Lazy<T, F>
where
    F: FnOnce() -> T,
{
    type Output = T;
    
    fn name(&self) -> &'static str {
        self.name
    }
    
    fn is_initialized(&self) -> bool {
        self.is_initialized()
    }
    
    fn initialize(&self) -> &Self::Output {
        self.initialize()
    }
}

impl<T> LazyComponents<T> {
    /// Create a new empty container
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            component_names: Vec::new(),
        }
    }
    
    /// Add a lazy component to the container
    pub fn add<F>(&mut self, component: Lazy<T, F>) -> &mut Self 
    where
        F: FnOnce() -> T + 'static,
        Lazy<T, F>: LazyComponent<Output = T>,
    {
        self.component_names.push(component.name());
        self.components.push(Box::new(component));
        self
    }
    
    /// Get a list of all component names
    pub fn component_names(&self) -> &[&'static str] {
        &self.component_names
    }
    
    /// Initialize all components
    pub fn initialize_all(&self) -> Vec<&T> {
        let start = Instant::now();
        
        let results: Vec<&T> = self.components
            .iter()
            .map(|c| c.initialize())
            .collect();
        
        let duration = start.elapsed();
        debug!(
            "Initialized all {} components in {} µs",
            self.components.len(),
            duration.as_micros()
        );
        
        results
    }
    
    /// Log initialization metrics
    pub fn log_metrics(&self) {
        INIT_METRICS.lock().unwrap().log_metrics();
    }
}

/// Initialize a component only if a condition is met
pub struct ConditionalLazy<T, F, P> {
    /// Inner lazy component
    inner: Lazy<T, F>,
    /// Predicate that determines if initialization should happen
    predicate: P,
    /// Whether the predicate has been evaluated
    evaluated: Mutex<bool>,
}

impl<T, F, P> ConditionalLazy<T, F, P>
where
    F: FnOnce() -> T,
    P: Fn() -> bool + Send + Sync,
{
    /// Create a new conditionally initialized component
    pub const fn new(name: &'static str, init: F, predicate: P) -> Self {
        Self {
            inner: Lazy::new(name, init),
            predicate,
            evaluated: Mutex::new(false),
        }
    }
    
    /// Get the value if the predicate is true, otherwise return None
    pub fn get(&self) -> Option<&T> {
        // Check if the predicate has been evaluated
        let mut evaluated = self.evaluated.lock().unwrap();
        
        if !*evaluated {
            // Evaluate predicate
            let should_init = (self.predicate)();
            *evaluated = true;
            
            if should_init {
                // Initialize if predicate is true
                return Some(self.inner.initialize());
            } else {
                debug!("Component '{}' initialization skipped due to predicate", self.inner.name);
                return None;
            }
        }
        
        // Return value if already initialized
        if self.inner.is_initialized() {
            Some(self.inner.get())
        } else {
            None
        }
    }
    
    /// Force initialization regardless of predicate
    pub fn force_initialize(&self) -> &T {
        let mut evaluated = self.evaluated.lock().unwrap();
        *evaluated = true;
        self.inner.initialize()
    }
}

/// Helper to create a conditionally initialized component based on environment variable
pub fn env_conditional<T, F>(
    name: &'static str,
    init: F,
    env_var: &'static str,
    default: bool,
) -> ConditionalLazy<T, F, impl Fn() -> bool + Send + Sync>
where
    F: FnOnce() -> T,
{
    ConditionalLazy::new(
        name,
        init,
        move || {
            std::env::var(env_var)
                .map(|val| val == "1" || val.to_lowercase() == "true")
                .unwrap_or(default)
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    // Test component that tracks if it was initialized
    struct ExpensiveComponent {
        id: usize,
        initialized: Arc<AtomicUsize>,
    }
    
    impl ExpensiveComponent {
        fn new(id: usize, initialized: Arc<AtomicUsize>) -> Self {
            // Simulate expensive initialization
            std::thread::sleep(Duration::from_millis(5));
            initialized.fetch_add(1, Ordering::SeqCst);
            Self { id, initialized }
        }
        
        fn id(&self) -> usize {
            self.id
        }
    }
    
    #[test]
    fn test_lazy_initialization() {
        let initialized = Arc::new(AtomicUsize::new(0));
        let init_clone = initialized.clone();
        
        // Create lazy component
        let lazy = Lazy::new("test_component", move || {
            ExpensiveComponent::new(42, init_clone)
        });
        
        // Should not be initialized yet
        assert_eq!(initialized.load(Ordering::SeqCst), 0);
        
        // Access value to trigger initialization
        let component = lazy.get();
        assert_eq!(component.id(), 42);
        
        // Should be initialized now
        assert_eq!(initialized.load(Ordering::SeqCst), 1);
        
        // Second access should not reinitialize
        let component2 = lazy.get();
        assert_eq!(component2.id(), 42);
        assert_eq!(initialized.load(Ordering::SeqCst), 1);
    }
    
    #[test]
    fn test_conditional_initialization() {
        let initialized = Arc::new(AtomicUsize::new(0));
        let init_clone = initialized.clone();
        
        // Create conditional component that should initialize
        let conditional_true = ConditionalLazy::new(
            "conditional_true",
            move || ExpensiveComponent::new(1, init_clone),
            || true,
        );
        
        // Create conditional component that should not initialize
        let init_clone2 = initialized.clone();
        let conditional_false = ConditionalLazy::new(
            "conditional_false",
            move || ExpensiveComponent::new(2, init_clone2),
            || false,
        );
        
        // Check values
        let component1 = conditional_true.get();
        assert!(component1.is_some());
        assert_eq!(component1.unwrap().id(), 1);
        
        let component2 = conditional_false.get();
        assert!(component2.is_none());
        
        // Only one should have initialized
        assert_eq!(initialized.load(Ordering::SeqCst), 1);
        
        // Force initialize the second component
        let forced = conditional_false.force_initialize();
        assert_eq!(forced.id(), 2);
        
        // Now both should be initialized
        assert_eq!(initialized.load(Ordering::SeqCst), 2);
    }
}
