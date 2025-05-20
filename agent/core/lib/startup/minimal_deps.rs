// agent/core/lib/startup/minimal_deps.rs

use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tracing::{debug, info, warn};

/// Tracker for dependencies loaded during initialization
pub struct MinimalDependencyLoader {
    /// Whether dependency loading is active
    active: AtomicBool,
    /// Set of loaded dependencies
    loaded: parking_lot::RwLock<HashSet<&'static str>>,
    /// Allowed dependencies in minimal mode
    allowed: parking_lot::RwLock<HashSet<&'static str>>,
    /// Whether to warn or error on disallowed dependencies
    strict: bool,
}

impl MinimalDependencyLoader {
    /// Create a new dependency loader
    pub fn new(strict: bool) -> Self {
        let mut allowed = HashSet::new();
        
        // Always allowed core dependencies
        allowed.insert("std");
        allowed.insert("core");
        allowed.insert("alloc");
        allowed.insert("tokio");
        allowed.insert("tracing");
        allowed.insert("anyhow");
        allowed.insert("serde");
        allowed.insert("serde_json");
        allowed.insert("lazy_static");
        allowed.insert("once_cell");
        
        Self {
            active: AtomicBool::new(true),
            loaded: parking_lot::RwLock::new(HashSet::new()),
            allowed: parking_lot::RwLock::new(allowed),
            strict,
        }
    }
    
    /// Add allowed dependencies
    pub fn allow(&self, dependencies: &[&'static str]) {
        let mut allowed = self.allowed.write();
        for dep in dependencies {
            allowed.insert(*dep);
        }
    }
    
    /// Track a dependency load
    pub fn load_dependency(&self, name: &'static str) -> bool {
        // Skip if not active
        if !self.active.load(Ordering::Relaxed) {
            return true;
        }
        
        // Check if dependency is allowed
        let allowed = {
            let allowed = self.allowed.read();
            allowed.contains(name)
        };
        
        // Track loaded dependency
        let mut loaded = self.loaded.write();
        loaded.insert(name);
        
        if !allowed && self.strict {
            warn!("Strict mode: Disallowed dependency '{}' loaded during cold start", name);
            false
        } else {
            if !allowed {
                debug!("Non-essential dependency '{}' loaded during cold start", name);
            }
            true
        }
    }
    
    /// Deactivate dependency tracking
    pub fn deactivate(&self) {
        self.active.store(false, Ordering::Relaxed);
    }
    
    /// Get all loaded dependencies
    pub fn get_loaded_dependencies(&self) -> HashSet<&'static str> {
        self.loaded.read().clone()
    }
    
    /// Get statistics on dependencies
    pub fn get_statistics(&self) -> DependencyStats {
        let loaded = self.loaded.read();
        let allowed = self.allowed.read();
        
        let essential: HashSet<_> = loaded.intersection(&allowed).copied().collect();
        let non_essential: HashSet<_> = loaded.difference(&allowed).copied().collect();
        
        DependencyStats {
            total_loaded: loaded.len(),
            essential: essential.len(),
            non_essential: non_essential.len(),
            non_essential_deps: non_essential,
        }
    }
    
    /// Log dependency statistics
    pub fn log_statistics(&self) {
        let stats = self.get_statistics();
        
        info!(
            "Dependency loading: {} total, {} essential, {} non-essential",
            stats.total_loaded,
            stats.essential,
            stats.non_essential
        );
        
        if !stats.non_essential_deps.is_empty() {
            debug!("Non-essential dependencies loaded during cold start:");
            for dep in &stats.non_essential_deps {
                debug!("  - {}", dep);
            }
        }
    }
}

/// Dependency loading statistics
pub struct DependencyStats {
    /// Total number of loaded dependencies
    pub total_loaded: usize,
    /// Number of essential dependencies
    pub essential: usize,
    /// Number of non-essential dependencies
    pub non_essential: usize,
    /// Set of non-essential dependencies
    pub non_essential_deps: HashSet<&'static str>,
}

/// Global dependency loader
pub static DEPENDENCY_LOADER: once_cell::sync::Lazy<Arc<MinimalDependencyLoader>> = 
    once_cell::sync::Lazy::new(|| {
        let loader = MinimalDependencyLoader::new(
            std::env::var("STRICT_DEPENDENCIES")
                .map(|v| v == "1" || v.to_lowercase() == "true")
                .unwrap_or(false)
        );
        
        // Allow additional dependencies from environment
        if let Ok(deps) = std::env::var("ALLOWED_DEPENDENCIES") {
            let deps: Vec<&'static str> = deps.split(',')
                .map(|s| {
                    let s = s.trim();
                    // This leaks memory, but it's only done once at startup
                    // and the list should be small
                    Box::leak(s.to_string().into_boxed_str())
                })
                .collect();
            
            loader.allow(&deps);
        }
        
        Arc::new(loader)
    });
    
/// Track dependency loading for minimal cold start
#[macro_export]
macro_rules! track_dependency {
    ($name:expr) => {
        #[cfg(feature = "minimal_deps")]
        {
            use $crate::startup::minimal_deps::DEPENDENCY_LOADER;
            let _ = DEPENDENCY_LOADER.load_dependency($name);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_minimal_dependency_loader() {
        let loader = MinimalDependencyLoader::new(false);
        
        // Test basic loading
        assert!(loader.load_dependency("std"));
        assert!(loader.load_dependency("tokio"));
        
        // Test non-essential dependency
        assert!(loader.load_dependency("aws_sdk_s3"));
        
        // Check statistics
        let stats = loader.get_statistics();
        assert_eq!(stats.total_loaded, 3);
        assert_eq!(stats.essential, 2);
        assert_eq!(stats.non_essential, 1);
        assert!(stats.non_essential_deps.contains("aws_sdk_s3"));
        
        // Test adding allowed dependencies
        loader.allow(&["aws_sdk_s3"]);
        
        // Recalculate statistics
        let stats = loader.get_statistics();
        assert_eq!(stats.essential, 3);
        assert_eq!(stats.non_essential, 0);
        
        // Test deactivation
        loader.deactivate();
        assert!(loader.load_dependency("new_dependency"));
        
        // Should not have added the dependency
        assert!(!loader.get_loaded_dependencies().contains("new_dependency"));
    }
    
    #[test]
    fn test_strict_mode() {
        let loader = MinimalDependencyLoader::new(true);
        
        // Essential dependencies should load
        assert!(loader.load_dependency("std"));
        
        // Non-essential dependencies should fail in strict mode
        assert!(!loader.load_dependency("aws_sdk_s3"));
    }
}