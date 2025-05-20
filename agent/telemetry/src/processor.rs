// agent/telemetry/src/processor.rs

use std::sync::Arc;
use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use tracing::{debug, info, warn};

use crate::{TelemetryEvent, TelemetryBatch};

/// Trait for telemetry processors
#[async_trait]
pub trait TelemetryProcessor: Send + Sync {
    /// Process a batch of telemetry events
    async fn process_batch(&self, batch: TelemetryBatch) -> Result<TelemetryBatch>;
    
    /// Get the processor name
    fn name(&self) -> &str;
}

/// Filtering processor to filter events by criteria
pub struct FilteringProcessor {
    /// Processor name
    name: String,
    /// Filter function
    filter: Arc<dyn Fn(&TelemetryEvent) -> bool + Send + Sync>,
}

impl FilteringProcessor {
    /// Create a new filtering processor
    pub fn new(name: &str, filter: impl Fn(&TelemetryEvent) -> bool + Send + Sync + 'static) -> Self {
        Self {
            name: name.to_string(),
            filter: Arc::new(filter),
        }
    }
    
    /// Create a processor that filters by source
    pub fn source(sources: Vec<String>) -> Self {
        let sources = Arc::new(sources);
        Self::new(
            "source_filter",
            move |event| sources.contains(&event.source),
        )
    }
    
    /// Create a processor that filters by event type
    pub fn event_type(types: Vec<String>) -> Self {
        let types = Arc::new(types);
        Self::new(
            "event_type_filter",
            move |event| types.contains(&event.event_type),
        )
    }
    
    /// Create a processor that filters by event name
    pub fn event_name(names: Vec<String>) -> Self {
        let names = Arc::new(names);
        Self::new(
            "event_name_filter",
            move |event| names.contains(&event.name),
        )
    }
}

#[async_trait]
impl TelemetryProcessor for FilteringProcessor {
    async fn process_batch(&self, batch: TelemetryBatch) -> Result<TelemetryBatch> {
        // Filter events
        let filtered_events = batch.events
            .into_iter()
            .filter(|event| (self.filter)(event))
            .collect::<Vec<_>>();
        
        // Create new batch
        Ok(TelemetryBatch {
            id: batch.id,
            timestamp: batch.timestamp,
            source: batch.source,
            events: filtered_events,
            metadata: Some(json!({
                "processor": self.name,
                "original_count": batch.events.len(),
                "filtered_count": filtered_events.len(),
            })),
        })
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

/// Transformation processor to transform events
pub struct TransformationProcessor {
    /// Processor name
    name: String,
    /// Transform function
    transform: Arc<dyn Fn(TelemetryEvent) -> Result<TelemetryEvent> + Send + Sync>,
}

impl TransformationProcessor {
    /// Create a new transformation processor
    pub fn new(
        name: &str,
        transform: impl Fn(TelemetryEvent) -> Result<TelemetryEvent> + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: name.to_string(),
            transform: Arc::new(transform),
        }
    }
    
    /// Create a processor that adds metadata to events
    pub fn add_metadata(metadata: serde_json::Value) -> Self {
        let metadata = Arc::new(metadata);
        Self::new(
            "add_metadata",
            move |mut event| {
                // Update metadata
                match &mut event.metadata {
                    Some(existing) => {
                        if let Some(obj) = existing.as_object_mut() {
                            if let Some(add_obj) = metadata.as_object() {
                                for (key, value) in add_obj {
                                    obj.insert(key.clone(), value.clone());
                                }
                            }
                        }
                    }
                    None => {
                        event.metadata = Some(metadata.clone());
                    }
                }
                
                Ok(event)
            },
        )
    }
    
    /// Create a processor that sanitizes sensitive data
    pub fn sanitize_sensitive_data(sensitive_keys: Vec<String>) -> Self {
        let sensitive_keys = Arc::new(sensitive_keys);
        Self::new(
            "sanitize_sensitive_data",
            move |mut event| {
                // TODO: Implement sanitization logic
                // This would recursively scan the event data and metadata
                // to replace sensitive values
                
                Ok(event)
            },
        )
    }
}

#[async_trait]
impl TelemetryProcessor for TransformationProcessor {
    async fn process_batch(&self, batch: TelemetryBatch) -> Result<TelemetryBatch> {
        // Transform events
        let mut transformed_events = Vec::with_capacity(batch.events.len());
        let mut errors = 0;
        
        for event in batch.events {
            match (self.transform)(event) {
                Ok(transformed) => transformed_events.push(transformed),
                Err(_) => errors += 1,
            }
        }
        
        // Create new batch
        Ok(TelemetryBatch {
            id: batch.id,
            timestamp: batch.timestamp,
            source: batch.source,
            events: transformed_events,
            metadata: Some(json!({
                "processor": self.name,
                "original_count": transformed_events.len() + errors,
                "transformed_count": transformed_events.len(),
                "error_count": errors,
            })),
        })
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}

/// Enrichment processor to add information to events
pub struct EnrichmentProcessor {
    /// Processor name
    name: String,
    /// Enrichment function
    enrich: Arc<dyn Fn(&mut TelemetryEvent) -> Result<()> + Send + Sync>,
}

impl EnrichmentProcessor {
    /// Create a new enrichment processor
    pub fn new(
        name: &str,
        enrich: impl Fn(&mut TelemetryEvent) -> Result<()> + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: name.to_string(),
            enrich: Arc::new(enrich),
        }
    }
    
    /// Create a processor that adds environment information
    pub fn add_environment_info() -> Self {
        Self::new(
            "add_environment_info",
            move |event| {
                let mut env_info = HashMap::new();
                
                // Add hostname
                if let Ok(hostname) = std::env::var("HOSTNAME") {
                    env_info.insert("hostname".to_string(), hostname);
                }
                
                // Add AWS region if available
                if let Ok(region) = std::env::var("AWS_REGION") {
                    env_info.insert("aws_region".to_string(), region);
                }
                
                // Add Lambda function name if available
                if let Ok(function_name) = std::env::var("AWS_LAMBDA_FUNCTION_NAME") {
                    env_info.insert("lambda_function_name".to_string(), function_name);
                }
                
                // Add environment
                if let Ok(env) = std::env::var("NODE_ENV") {
                    env_info.insert("node_env".to_string(), env);
                } else if let Ok(env) = std::env::var("RUST_ENV") {
                    env_info.insert("rust_env".to_string(), env);
                }
                
                // Add to metadata
                if !env_info.is_empty() {
                    match &mut event.metadata {
                        Some(metadata) => {
                            if let Some(obj) = metadata.as_object_mut() {
                                obj.insert("environment".to_string(), json!(env_info));
                            }
                        }
                        None => {
                            event.metadata = Some(json!({
                                "environment": env_info
                            }));
                        }
                    }
                }
                
                Ok(())
            },
        )
    }
}

#[async_trait]
impl TelemetryProcessor for EnrichmentProcessor {
    async fn process_batch(&self, mut batch: TelemetryBatch) -> Result<TelemetryBatch> {
        // Enrich events
        let mut errors = 0;
        
        for event in &mut batch.events {
            if let Err(_) = (self.enrich)(event) {
                errors += 1;
            }
        }
        
        // Update batch metadata
        match &mut batch.metadata {
            Some(metadata) => {
                if let Some(obj) = metadata.as_object_mut() {
                    obj.insert("processor".to_string(), json!(self.name));
                    obj.insert("error_count".to_string(), json!(errors));
                }
            }
            None => {
                batch.metadata = Some(json!({
                    "processor": self.name,
                    "error_count": errors,
                }));
            }
        }
        
        Ok(batch)
    }
    
    fn name(&self) -> &str {
        &self.name
    }
}