// agent/collectors/src/lib.rs

//! Collectors module for the Causal distributed systems debugger
//!
//! This module provides a pluggable architecture for collecting telemetry
//! from various sources in a distributed system.

mod aws_sdk;
mod database;
mod http;
mod system;
mod registry;

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use agent_core::telemetry::metrics::MetricValue;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// Collection frequency for metrics
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CollectionFrequency {
    /// Collect once on start
    Once,
    /// Collect on every invocation
    PerInvocation,
    /// Collect on a fixed interval
    Interval(Duration),
    /// Collect on demand only
    OnDemand,
}

/// Configuration for a collector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectorConfig {
    /// Unique collector ID
    pub id: String,
    /// Collection frequency
    pub frequency: CollectionFrequency,
    /// Whether this collector is enabled
    pub enabled: bool,
    /// Collector-specific configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<HashMap<String, String>>,
}

/// Metadata about a collector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectorMetadata {
    /// Unique collector ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Description
    pub description: String,
    /// Data source type
    pub source_type: String,
    /// Whether this collector is enabled by default
    pub enabled_by_default: bool,
}

/// Core trait for all telemetry collectors
#[async_trait]
pub trait Collector: Send + Sync {
    /// Get collector metadata
    fn metadata(&self) -> CollectorMetadata;
    
    /// Get current configuration
    fn config(&self) -> CollectorConfig;
    
    /// Update configuration
    fn update_config(&mut self, config: CollectorConfig) -> Result<()>;
    
    /// Initialize the collector
    async fn initialize(&mut self) -> Result<()>;
    
    /// Collect metrics
    async fn collect(&self) -> Result<HashMap<String, MetricValue>>;
    
    /// Shut down the collector
    async fn shutdown(&self) -> Result<()>;
    
    /// Get collector as Any for downcasting
    fn as_any(&self) -> &dyn Any;
}

// Re-export the collector registry
pub use registry::CollectorRegistry;

/// Builder pattern for creating collectors
#[derive(Debug, Default)]
pub struct CollectorBuilder {
    id: Option<String>,
    name: Option<String>,
    description: Option<String>,
    source_type: Option<String>,
    enabled: bool,
    frequency: CollectionFrequency,
    settings: HashMap<String, String>,
}

impl CollectorBuilder {
    /// Create a new collector builder
    pub fn new() -> Self {
        Self {
            id: None,
            name: None,
            description: None,
            source_type: None,
            enabled: true,
            frequency: CollectionFrequency::PerInvocation,
            settings: HashMap::new(),
        }
    }
    
    /// Set collector ID
    pub fn with_id(mut self, id: &str) -> Self {
        self.id = Some(id.to_string());
        self
    }
    
    /// Set display name
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }
    
    /// Set description
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }
    
    /// Set source type
    pub fn with_source_type(mut self, source_type: &str) -> Self {
        self.source_type = Some(source_type.to_string());
        self
    }
    
    /// Set enabled state
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
    
    /// Set collection frequency
    pub fn with_frequency(mut self, frequency: CollectionFrequency) -> Self {
        self.frequency = frequency;
        self
    }
    
    /// Add a setting
    pub fn with_setting(mut self, key: &str, value: &str) -> Self {
        self.settings.insert(key.to_string(), value.to_string());
        self
    }
    
    /// Add multiple settings
    pub fn with_settings(mut self, settings: HashMap<String, String>) -> Self {
        self.settings.extend(settings);
        self
    }
    
    /// Build metadata
    pub fn build_metadata(&self) -> Result<CollectorMetadata> {
        Ok(CollectorMetadata {
            id: self.id.clone().ok_or_else(|| anyhow::anyhow!("Collector ID is required"))?,
            name: self.name.clone().unwrap_or_else(|| self.id.clone().unwrap_or_default()),
            description: self.description.clone().unwrap_or_default(),
            source_type: self.source_type.clone().ok_or_else(|| anyhow::anyhow!("Source type is required"))?,
            enabled_by_default: self.enabled,
        })
    }
    
    /// Build configuration
    pub fn build_config(&self) -> Result<CollectorConfig> {
        Ok(CollectorConfig {
            id: self.id.clone().ok_or_else(|| anyhow::anyhow!("Collector ID is required"))?,
            frequency: self.frequency,
            enabled: self.enabled,
            settings: if self.settings.is_empty() {
                None
            } else {
                Some(self.settings.clone())
            },
        })
    }
}

/// Traits for collector factories
#[async_trait]
pub trait CollectorFactory: Send + Sync {
    /// Create a new collector instance
    async fn create(&self, config: CollectorConfig) -> Result<Box<dyn Collector>>;
    
    /// Get metadata for this collector type
    fn metadata(&self) -> CollectorMetadata;
}

/// Helper function to get all available collector factories
pub fn get_available_collector_factories() -> Vec<Box<dyn CollectorFactory>> {
    let mut factories: Vec<Box<dyn CollectorFactory>> = Vec::new();
    
    // Register AWS SDK collectors
    #[cfg(feature = "aws")]
    {
        factories.push(Box::new(aws_sdk::lambda::LambdaCollectorFactory::new()));
        factories.push(Box::new(aws_sdk::dynamodb::DynamoDbCollectorFactory::new()));
        factories.push(Box::new(aws_sdk::s3::S3CollectorFactory::new()));
    }
    
    // Register HTTP collectors
    #[cfg(feature = "http")]
    {
        factories.push(Box::new(http::client::HttpClientCollectorFactory::new()));
        factories.push(Box::new(http::server::HttpServerCollectorFactory::new()));
    }
    
    // Register database collectors
    #[cfg(feature = "database")]
    {
        factories.push(Box::new(database::sql::SqlDatabaseCollectorFactory::new()));
        factories.push(Box::new(database::nosql::NoSqlDatabaseCollectorFactory::new()));
    }
    
    // Register system collectors
    #[cfg(feature = "system")]
    {
        factories.push(Box::new(system::cpu::CpuCollectorFactory::new()));
        factories.push(Box::new(system::memory::MemoryCollectorFactory::new()));
        factories.push(Box::new(system::network::NetworkCollectorFactory::new()));
        factories.push(Box::new(system::process::ProcessCollectorFactory::new()));
    }
    
    factories
}