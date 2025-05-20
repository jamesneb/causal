// agent/telemetry/src/manager.rs

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use agent_core::telemetry::metrics::MetricValue;
use anyhow::{Result, anyhow};
use causal_collectors::{
    Collector, CollectorConfig, CollectorFactory, CollectorMetadata, 
    CollectionFrequency, get_available_collector_factories
};
use serde_json::json;
use tokio::sync::{RwLock, mpsc};
use tokio::time::interval;
use tracing::{debug, info, warn, error};

use crate::{
    TelemetryEvent, TelemetryBatch, TelemetryEventBuilder,
    TelemetryRegistry, TelemetryPipeline, TelemetryConfig
};

/// Telemetry manager that coordinates collectors, processors, and transporters
pub struct TelemetryManager {
    /// Telemetry registry
    registry: TelemetryRegistry,
    /// Collection intervals
    intervals: RwLock<HashMap<u64, tokio::task::JoinHandle<()>>>,
    /// Buffer for events
    buffer: RwLock<Vec<TelemetryEvent>>,
    /// Event sender
    event_tx: mpsc::Sender<TelemetryEvent>,
    /// Event receiver
    event_rx: RwLock<mpsc::Receiver<TelemetryEvent>>,
    /// Running state
    running: RwLock<bool>,
    /// Manager configuration
    config: RwLock<TelemetryConfig>,
    /// Pipeline for processing and transporting events
    pipeline: RwLock<Option<Arc<TelemetryPipeline>>>,
}

impl TelemetryManager {
    /// Create a new telemetry manager
    pub fn new(registry: TelemetryRegistry) -> Self {
        let (tx, rx) = mpsc::channel(1000);
        
        Self {
            registry,
            intervals: RwLock::new(HashMap::new()),
            buffer: RwLock::new(Vec::new()),
            event_tx: tx,
            event_rx: RwLock::new(rx),
            running: RwLock::new(false),
            config: RwLock::new(TelemetryConfig::default()),
            pipeline: RwLock::new(None),
        }
    }
    
    /// Set the telemetry pipeline
    pub async fn set_pipeline(&self, pipeline: Arc<TelemetryPipeline>) {
        let mut p = self.pipeline.write().await;
        *p = Some(pipeline);
    }
    
    /// Register built-in collectors
    pub async fn register_built_in_collectors(&self) -> Result<()> {
        info!("Registering built-in collectors");
        
        let factories = get_available_collector_factories();
        
        for factory in factories {
            let metadata = factory.metadata();
            let id = metadata.id.clone();
            
            debug!("Registering collector factory: {}", id);
            
            let config = CollectorConfig {
                id: id.clone(),
                frequency: CollectionFrequency::Interval(Duration::from_secs(60)),
                enabled: metadata.enabled_by_default,
                settings: None,
            };
            
            let collector = factory.create(config).await?;
            
            self.registry.register_collector(id, collector).await?;
        }
        
        Ok(())
    }
    
    /// Initialize all registered collectors
    pub async fn initialize_collectors(&self) -> Result<()> {
        info!("Initializing collectors");
        
        self.registry.initialize_collectors().await?;
        
        Ok(())
    }
    
    /// Start the telemetry manager
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(());
        }
        
        info!("Starting telemetry manager");
        
        // Start the event processing task
        self.start_event_processor().await?;
        
        // Start collection intervals
        self.start_collection_intervals().await?;
        
        *running = true;
        
        Ok(())
    }
    
    /// Stop the telemetry manager
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if !*running {
            return Ok(());
        }
        
        info!("Stopping telemetry manager");
        
        // Stop collection intervals
        let mut intervals = self.intervals.write().await;
        for (_, handle) in intervals.drain() {
            handle.abort();
        }
        
        // Flush buffer
        self.flush_buffer().await?;
        
        *running = false;
        
        Ok(())
    }
    
    /// Add a telemetry event
    pub async fn add_event(&self, event: TelemetryEvent) -> Result<()> {
        if !*self.running.read().await {
            return Err(anyhow!("Telemetry manager is not running"));
        }
        
        self.event_tx.send(event).await.map_err(|e| anyhow!("Failed to send event: {}", e))
    }
    
    /// Collect metrics from all collectors
    pub async fn collect_metrics(&self) -> Result<HashMap<String, HashMap<String, MetricValue>>> {
        self.registry.collect_metrics().await
    }
    
    /// Collect metrics from a specific collector
    pub async fn collect_from(&self, collector_id: &str) -> Result<HashMap<String, MetricValue>> {
        self.registry.collect_from(collector_id).await
    }
    
    /// Start event processor task
    async fn start_event_processor(&self) -> Result<()> {
        // Clone receiver
        let mut event_rx = self.event_rx.write().await.split().1;
        
        let event_buffer = Arc::new(self.buffer.clone());
        let running = Arc::new(self.running.clone());
        let pipeline = Arc::new(self.pipeline.clone());
        let config = Arc::new(self.config.clone());
        
        // Spawn event processing task
        tokio::spawn(async move {
            let mut flush_interval = interval(Duration::from_secs(5));
            
            loop {
                tokio::select! {
                    // Process incoming event
                    Some(event) = event_rx.recv() => {
                        let mut buffer = event_buffer.write().await;
                        buffer.push(event);
                        
                        // Flush if buffer is large enough
                        let threshold = {
                            let cfg = config.read().await;
                            cfg.buffer_threshold
                        };
                        
                        if buffer.len() >= threshold {
                            if let Err(e) = flush_buffer_internal(&mut buffer, &pipeline).await {
                                error!("Failed to flush buffer: {}", e);
                            }
                        }
                    }
                    
                    // Flush on interval
                    _ = flush_interval.tick() => {
                        let mut buffer = event_buffer.write().await;
                        if !buffer.is_empty() {
                            if let Err(e) = flush_buffer_internal(&mut buffer, &pipeline).await {
                                error!("Failed to flush buffer: {}", e);
                            }
                        }
                    }
                    
                    // Exit if not running
                    else => {
                        let is_running = *running.read().await;
                        if !is_running {
                            break;
                        }
                    }
                }
            }
        });
        
        Ok(())
    }
    
    /// Start collection intervals
    async fn start_collection_intervals(&self) -> Result<()> {
        let collectors = self.registry.get_collectors().await;
        let mut interval_groups: HashMap<u64, Vec<String>> = HashMap::new();
        
        // Group collectors by interval
        for (id, collector) in &collectors {
            match collector.config().frequency {
                CollectionFrequency::Interval(duration) => {
                    let seconds = duration.as_secs();
                    interval_groups.entry(seconds).or_default().push(id.clone());
                }
                _ => {} // Ignore other collection frequencies
            }
        }
        
        // Start a task for each interval group
        let mut intervals = self.intervals.write().await;
        
        for (seconds, collector_ids) in interval_groups {
            let registry_clone = self.registry.clone();
            let event_tx = self.event_tx.clone();
            
            let handle = tokio::spawn(async move {
                let mut ticker = interval(Duration::from_secs(seconds));
                
                loop {
                    ticker.tick().await;
                    
                    for id in &collector_ids {
                        match registry_clone.collect_from(id).await {
                            Ok(metrics) => {
                                if !metrics.is_empty() {
                                    // Create and send event
                                    let event = TelemetryEventBuilder::new("metrics.collected", "metrics")
                                        .with_source(id)
                                        .with_metrics(&metrics)
                                        .build();
                                    
                                    if let Err(e) = event_tx.send(event).await {
                                        error!("Failed to send event: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                error!("Failed to collect from {}: {}", id, e);
                            }
                        }
                    }
                }
            });
            
            intervals.insert(seconds, handle);
        }
        
        Ok(())
    }
    
    /// Flush the event buffer
    pub async fn flush_buffer(&self) -> Result<()> {
        let mut buffer = self.buffer.write().await;
        let pipeline = self.pipeline.clone();
        
        flush_buffer_internal(&mut buffer, &pipeline).await
    }
}

/// Helper function to flush buffer
async fn flush_buffer_internal(
    buffer: &mut Vec<TelemetryEvent>,
    pipeline: &RwLock<Option<Arc<TelemetryPipeline>>>,
) -> Result<()> {
    if buffer.is_empty() {
        return Ok(());
    }
    
    // Create batch with events
    let events = std::mem::take(buffer);
    
    let batch = TelemetryBatch {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64,
        source: "telemetry_manager".to_string(),
        events,
        metadata: Some(json!({
            "event_count": buffer.len(),
        })),
    };
    
    // Process batch through pipeline if available
    let pipeline_guard = pipeline.read().await;
    if let Some(pipeline) = &*pipeline_guard {
        pipeline.process_batch(batch).await?;
    } else {
        // No pipeline, just log
        debug!("No pipeline configured, discarding {} events", batch.events.len());
    }
    
    Ok(())
}