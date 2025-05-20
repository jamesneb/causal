// agent/telemetry/src/lib.rs

//! Telemetry system for the Causal distributed systems debugger
//!
//! This module provides the telemetry system that combines collectors,
//! processors, and transporters to collect, process, and transport
//! telemetry data.

mod manager;
mod processor;
mod transport;
mod pipeline;
mod config;
mod registry;
mod buffer;

use std::sync::Arc;
use agent_core::telemetry::metrics::MetricValue;
use anyhow::Result;
use causal_collectors::{Collector, CollectorRegistry};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

// Re-export the main components
pub use manager::TelemetryManager;
pub use processor::TelemetryProcessor;
pub use transport::TelemetryTransport;
pub use pipeline::TelemetryPipeline;
pub use config::TelemetryConfig;
pub use registry::TelemetryRegistry;
pub use buffer::TelemetryBuffer;

/// Core telemetry event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryEvent {
    /// Event ID
    pub id: String,
    /// Event timestamp
    pub timestamp: u64,
    /// Source of the event
    pub source: String,
    /// Resource ID
    pub resource_id: Option<String>,
    /// Resource name
    pub resource_name: Option<String>,
    /// Event name
    pub name: String,
    /// Event type
    pub event_type: String,
    /// Event data (metrics)
    pub data: serde_json::Value,
    /// Event metadata
    pub metadata: Option<serde_json::Value>,
}

/// Telemetry batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryBatch {
    /// Batch ID
    pub id: String,
    /// Batch timestamp
    pub timestamp: u64,
    /// Source of the batch
    pub source: String,
    /// Events in the batch
    pub events: Vec<TelemetryEvent>,
    /// Batch metadata
    pub metadata: Option<serde_json::Value>,
}

/// Initialize the telemetry system
pub async fn init() -> Result<Arc<TelemetryManager>> {
    // Initialize the registry
    let registry = TelemetryRegistry::new();
    
    // Initialize the manager
    let manager = TelemetryManager::new(registry);
    
    // Register built-in collectors
    manager.register_built_in_collectors().await?;
    
    // Initialize all registered collectors
    manager.initialize_collectors().await?;
    
    info!("Telemetry system initialized");
    
    Ok(Arc::new(manager))
}

/// Builder for creating telemetry events
pub struct TelemetryEventBuilder {
    event: TelemetryEvent,
}

impl TelemetryEventBuilder {
    /// Create a new event builder
    pub fn new(name: &str, event_type: &str) -> Self {
        use uuid::Uuid;
        use std::time::{SystemTime, UNIX_EPOCH};
        
        Self {
            event: TelemetryEvent {
                id: Uuid::new_v4().to_string(),
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
                source: "unknown".to_string(),
                resource_id: None,
                resource_name: None,
                name: name.to_string(),
                event_type: event_type.to_string(),
                data: serde_json::Value::Null,
                metadata: None,
            },
        }
    }
    
    /// Set source of the event
    pub fn with_source(mut self, source: &str) -> Self {
        self.event.source = source.to_string();
        self
    }
    
    /// Set resource ID
    pub fn with_resource_id(mut self, resource_id: &str) -> Self {
        self.event.resource_id = Some(resource_id.to_string());
        self
    }
    
    /// Set resource name
    pub fn with_resource_name(mut self, resource_name: &str) -> Self {
        self.event.resource_name = Some(resource_name.to_string());
        self
    }
    
    /// Set event data from metrics
    pub fn with_metrics(mut self, metrics: &HashMap<String, MetricValue>) -> Self {
        self.event.data = serde_json::to_value(metrics).unwrap_or(serde_json::Value::Null);
        self
    }
    
    /// Set event data from JSON value
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.event.data = data;
        self
    }
    
    /// Set event metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.event.metadata = Some(metadata);
        self
    }
    
    /// Build the event
    pub fn build(self) -> TelemetryEvent {
        self.event
    }
}

// Import std::collections for HashMap
use std::collections::HashMap;