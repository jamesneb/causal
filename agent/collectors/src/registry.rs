// agent/collectors/src/registry.rs

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use anyhow::{Result, anyhow};
use tokio::sync::mpsc;
use tokio::time::{Duration, interval};
use tracing::{debug, error, info, warn};

use crate::{Collector, CollectorConfig, CollectorFactory, CollectionFrequency};

/// Registry of collectors that manages initialization and collection
pub struct CollectorRegistry {
    /// Map of collector ID to collector instance
    collectors: RwLock<HashMap<String, Box<dyn Collector>>>,
    /// Map of collector ID to sender for on-demand collection
    on_demand_channels: RwLock<HashMap<String, mpsc::Sender<()>>>,
    /// Whether the registry has been initialized
    initialized: RwLock<bool>,
    /// Whether scheduled collection is running
    running: RwLock<bool>,
}

impl CollectorRegistry {
    /// Create a new collector registry
    pub fn new() -> Self {
        Self {
            collectors: RwLock::new(HashMap::new()),
            on_demand_channels: RwLock::new(HashMap::new()),
            initialized: RwLock::new(false),
            running: RwLock::new(false),
        }
    }
    
    /// Register a collector factory
    pub fn register_factory(&self, factory: Box<dyn CollectorFactory>, config: Option<CollectorConfig>) -> Result<()> {
        let metadata = factory.metadata();
        let id = metadata.id.clone();
        
        debug!("Registering collector factory: {}", id);
        
        let config = match config {
            Some(c) => c,
            None => CollectorConfig {
                id: id.clone(),
                frequency: CollectionFrequency::PerInvocation,
                enabled: metadata.enabled_by_default,
                settings: None,
            },
        };
        
        // Create the collector instance
        let collector = tokio::runtime::Runtime::new()?.block_on(factory.create(config))?;
        
        // Register the collector
        let mut collectors = self.collectors.write().unwrap();
        collectors.insert(id, collector);
        
        Ok(())
    }
    
    /// Initialize all registered collectors
    pub async fn initialize(&self) -> Result<()> {
        let mut initialized = self.initialized.write().unwrap();
        if *initialized {
            return Ok(());
        }
        
        info!("Initializing {} collectors", self.collectors.read().unwrap().len());
        
        // Initialize each collector
        let mut collectors = self.collectors.write().unwrap();
        for (id, collector) in collectors.iter_mut() {
            debug!("Initializing collector: {}", id);
            
            match collector.initialize().await {
                Ok(_) => debug!("Collector initialized: {}", id),
                Err(e) => {
                    error!("Failed to initialize collector {}: {}", id, e);
                    // Continue with other collectors
                }
            }
            
            // Set up on-demand channel if needed
            if let CollectionFrequency::OnDemand = collector.config().frequency {
                let (tx, mut rx) = mpsc::channel(10);
                let collector_clone = Arc::new(collector.as_ref());
                
                tokio::spawn(async move {
                    while let Some(_) = rx.recv().await {
                        debug!("On-demand collection triggered for {}", id);
                        match collector_clone.collect().await {
                            Ok(metrics) => {
                                debug!("Collected {} metrics from {}", metrics.len(), id);
                                // Metrics would be processed here or sent to a central handler
                            }
                            Err(e) => error!("Failed to collect metrics from {}: {}", id, e),
                        }
                    }
                });
                
                self.on_demand_channels.write().unwrap().insert(id.clone(), tx);
            }
        }
        
        *initialized = true;
        info!("All collectors initialized");
        
        Ok(())
    }
    
    /// Start scheduled collection for interval-based collectors
    pub async fn start_scheduled_collection(&self) -> Result<()> {
        let mut running = self.running.write().unwrap();
        if *running {
            return Ok(());
        }
        
        info!("Starting scheduled collection");
        
        // Group collectors by interval
        let collectors = self.collectors.read().unwrap();
        let mut interval_groups: HashMap<Duration, Vec<String>> = HashMap::new();
        
        for (id, collector) in collectors.iter() {
            if let CollectionFrequency::Interval(duration) = collector.config().frequency {
                interval_groups
                    .entry(duration)
                    .or_insert_with(Vec::new)
                    .push(id.clone());
            }
        }
        
        // Start a task for each interval group
        for (duration, ids) in interval_groups {
            debug!("Starting collection task for {} collectors with interval {:?}", ids.len(), duration);
            
            let registry = Arc::new(self.clone());
            
            tokio::spawn(async move {
                let mut ticker = interval(duration);
                loop {
                    ticker.tick().await;
                    
                    for id in &ids {
                        if let Err(e) = registry.collect_from(id).await {
                            error!("Failed to collect from {}: {}", id, e);
                        }
                    }
                }
            });
        }
        
        *running = true;
        info!("Scheduled collection started");
        
        Ok(())
    }
    
    /// Collect metrics from a specific collector
    pub async fn collect_from(&self, id: &str) -> Result<HashMap<String, agent_core::telemetry::metrics::MetricValue>> {
        let collectors = self.collectors.read().unwrap();
        let collector = collectors.get(id).ok_or_else(|| anyhow!("Collector not found: {}", id))?;
        
        if !collector.config().enabled {
            debug!("Collector {} is disabled, skipping collection", id);
            return Ok(HashMap::new());
        }
        
        debug!("Collecting metrics from {}", id);
        collector.collect().await
    }
    
    /// Trigger on-demand collection for a specific collector
    pub async fn trigger_on_demand(&self, id: &str) -> Result<()> {
        let channels = self.on_demand_channels.read().unwrap();
        let sender = channels.get(id).ok_or_else(|| anyhow!("No on-demand channel for collector: {}", id))?;
        
        sender.send(()).await.map_err(|_| anyhow!("Failed to send on-demand trigger to collector: {}", id))?;
        
        Ok(())
    }
    
    /// Collect metrics from all collectors
    pub async fn collect_all(&self) -> Result<HashMap<String, HashMap<String, agent_core::telemetry::metrics::MetricValue>>> {
        let collectors = self.collectors.read().unwrap();
        let mut results = HashMap::new();
        
        for (id, collector) in collectors.iter() {
            if !collector.config().enabled {
                continue;
            }
            
            match collector.collect().await {
                Ok(metrics) => {
                    if !metrics.is_empty() {
                        results.insert(id.clone(), metrics);
                    }
                }
                Err(e) => error!("Failed to collect metrics from {}: {}", id, e),
            }
        }
        
        Ok(results)
    }
    
    /// Shut down all collectors
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down collectors");
        
        let mut collectors = self.collectors.write().unwrap();
        for (id, collector) in collectors.iter() {
            debug!("Shutting down collector: {}", id);
            
            if let Err(e) = collector.shutdown().await {
                error!("Failed to shut down collector {}: {}", id, e);
                // Continue with other collectors
            }
        }
        
        collectors.clear();
        self.on_demand_channels.write().unwrap().clear();
        *self.initialized.write().unwrap() = false;
        *self.running.write().unwrap() = false;
        
        info!("All collectors shut down");
        
        Ok(())
    }
    
    /// Get all registered collector IDs
    pub fn get_collector_ids(&self) -> Vec<String> {
        self.collectors.read().unwrap().keys().cloned().collect()
    }
    
    /// Get collector configuration
    pub fn get_collector_config(&self, id: &str) -> Option<CollectorConfig> {
        let collectors = self.collectors.read().unwrap();
        collectors.get(id).map(|c| c.config())
    }
    
    /// Update collector configuration
    pub fn update_collector_config(&self, id: &str, config: CollectorConfig) -> Result<()> {
        let mut collectors = self.collectors.write().unwrap();
        let collector = collectors.get_mut(id).ok_or_else(|| anyhow!("Collector not found: {}", id))?;
        
        collector.update_config(config)
    }
}

impl Clone for CollectorRegistry {
    fn clone(&self) -> Self {
        Self {
            collectors: RwLock::new(self.collectors.read().unwrap().clone()),
            on_demand_channels: RwLock::new(self.on_demand_channels.read().unwrap().clone()),
            initialized: RwLock::new(*self.initialized.read().unwrap()),
            running: RwLock::new(*self.running.read().unwrap()),
        }
    }
}

// Global collector registry
lazy_static::lazy_static! {
    pub static ref GLOBAL_COLLECTOR_REGISTRY: CollectorRegistry = CollectorRegistry::new();
}