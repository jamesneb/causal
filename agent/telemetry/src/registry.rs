// agent/telemetry/src/registry.rs

use std::collections::HashMap;
use std::sync::Arc;

use agent_core::telemetry::metrics::MetricValue;
use anyhow::{Result, anyhow};
use causal_collectors::Collector;
use tokio::sync::RwLock;
use tracing::{debug, info, warn, error};

/// Registry for telemetry collectors
#[derive(Clone)]
pub struct TelemetryRegistry {
    /// Registered collectors
    collectors: Arc<RwLock<HashMap<String, Box<dyn Collector>>>>,
}

impl TelemetryRegistry {
    /// Create a new telemetry registry
    pub fn new() -> Self {
        Self {
            collectors: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Register a collector
    pub async fn register_collector(&self, id: String, collector: Box<dyn Collector>) -> Result<()> {
        let mut collectors = self.collectors.write().await;
        collectors.insert(id, collector);
        Ok(())
    }
    
    /// Unregister a collector
    pub async fn unregister_collector(&self, id: &str) -> Result<()> {
        let mut collectors = self.collectors.write().await;
        if collectors.remove(id).is_none() {
            return Err(anyhow!("Collector not found: {}", id));
        }
        Ok(())
    }
    
    /// Initialize all registered collectors
    pub async fn initialize_collectors(&self) -> Result<()> {
        let mut collectors = self.collectors.write().await;
        let ids: Vec<String> = collectors.keys().cloned().collect();
        
        for id in &ids {
            if let Some(collector) = collectors.get_mut(id) {
                debug!("Initializing collector: {}", id);
                match collector.initialize().await {
                    Ok(_) => {
                        debug!("Collector initialized: {}", id);
                    }
                    Err(e) => {
                        error!("Failed to initialize collector {}: {}", id, e);
                        // Remove collector
                        collectors.remove(id);
                        // Continue with other collectors
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Collect metrics from all collectors
    pub async fn collect_metrics(&self) -> Result<HashMap<String, HashMap<String, MetricValue>>> {
        let collectors = self.collectors.read().await;
        let mut results = HashMap::new();
        
        for (id, collector) in collectors.iter() {
            match collector.collect().await {
                Ok(metrics) => {
                    if !metrics.is_empty() {
                        results.insert(id.clone(), metrics);
                    }
                }
                Err(e) => {
                    error!("Failed to collect metrics from {}: {}", id, e);
                }
            }
        }
        
        Ok(results)
    }
    
    /// Collect metrics from a specific collector
    pub async fn collect_from(&self, id: &str) -> Result<HashMap<String, MetricValue>> {
        let collectors = self.collectors.read().await;
        let collector = collectors.get(id).ok_or_else(|| anyhow!("Collector not found: {}", id))?;
        
        collector.collect().await
    }
    
    /// Get all registered collectors
    pub async fn get_collectors(&self) -> HashMap<String, Box<dyn Collector>> {
        self.collectors.read().await.clone()
    }
}