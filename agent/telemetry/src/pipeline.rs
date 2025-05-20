// agent/telemetry/src/pipeline.rs

use std::sync::Arc;
use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, warn, error};

use crate::{TelemetryEvent, TelemetryBatch, TelemetryProcessor, TelemetryTransport};

/// Pipeline for processing and transporting telemetry data
pub struct TelemetryPipeline {
    /// Processors to apply to batches
    processors: RwLock<Vec<Arc<dyn TelemetryProcessor>>>,
    /// Transporters to send processed batches
    transports: RwLock<Vec<Arc<dyn TelemetryTransport>>>,
    /// Pipeline name
    name: String,
}

impl TelemetryPipeline {
    /// Create a new telemetry pipeline
    pub fn new(name: &str) -> Self {
        Self {
            processors: RwLock::new(Vec::new()),
            transports: RwLock::new(Vec::new()),
            name: name.to_string(),
        }
    }
    
    /// Add a processor to the pipeline
    pub async fn add_processor(&self, processor: Arc<dyn TelemetryProcessor>) {
        let mut processors = self.processors.write().await;
        processors.push(processor);
    }
    
    /// Add a transport to the pipeline
    pub async fn add_transport(&self, transport: Arc<dyn TelemetryTransport>) {
        let mut transports = self.transports.write().await;
        transports.push(transport);
    }
    
    /// Process a batch through the pipeline
    pub async fn process_batch(&self, mut batch: TelemetryBatch) -> Result<()> {
        // Apply processors
        {
            let processors = self.processors.read().await;
            for processor in &*processors {
                batch = processor.process_batch(batch).await?;
            }
        }
        
        // Send to transporters
        {
            let transports = self.transports.read().await;
            let mut futures = Vec::new();
            
            for transport in &*transports {
                let transport_clone = transport.clone();
                let batch_clone = batch.clone();
                
                let future = async move {
                    if let Err(e) = transport_clone.send_batch(batch_clone).await {
                        error!("Failed to send batch via transport {}: {}", 
                               transport_clone.name(), e);
                        Err(e)
                    } else {
                        Ok(())
                    }
                };
                
                futures.push(future);
            }
            
            // Wait for all transports to complete
            let results = join_all(futures).await;
            
            // Check if any transport succeeded
            let success = results.iter().any(|r| r.is_ok());
            
            if !success && !results.is_empty() {
                return Err(anyhow::anyhow!("All transports failed to send batch"));
            }
        }
        
        Ok(())
    }
    
    /// Get the pipeline name
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// Builder for creating telemetry pipelines
pub struct TelemetryPipelineBuilder {
    /// Pipeline name
    name: String,
    /// Processors to add
    processors: Vec<Arc<dyn TelemetryProcessor>>,
    /// Transporters to add
    transports: Vec<Arc<dyn TelemetryTransport>>,
}

impl TelemetryPipelineBuilder {
    /// Create a new pipeline builder
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            processors: Vec::new(),
            transports: Vec::new(),
        }
    }
    
    /// Add a processor to the pipeline
    pub fn with_processor(mut self, processor: Arc<dyn TelemetryProcessor>) -> Self {
        self.processors.push(processor);
        self
    }
    
    /// Add a transport to the pipeline
    pub fn with_transport(mut self, transport: Arc<dyn TelemetryTransport>) -> Self {
        self.transports.push(transport);
        self
    }
    
    /// Build the pipeline
    pub async fn build(self) -> Arc<TelemetryPipeline> {
        let pipeline = Arc::new(TelemetryPipeline::new(&self.name));
        
        for processor in self.processors {
            pipeline.add_processor(processor).await;
        }
        
        for transport in self.transports {
            pipeline.add_transport(transport).await;
        }
        
        pipeline
    }
}