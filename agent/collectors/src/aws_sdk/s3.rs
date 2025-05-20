// agent/collectors/src/aws_sdk/s3.rs

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use agent_core::telemetry::metrics::MetricValue;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::{Collector, CollectorConfig, CollectorFactory, CollectorMetadata};
use super::{AwsSdkCollectorSettings, merge_aws_settings};

/// S3 collector settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3CollectorSettings {
    /// Base AWS SDK settings
    #[serde(flatten)]
    pub base: AwsSdkCollectorSettings,
    /// Whether to collect bucket metrics
    pub collect_bucket_metrics: bool,
    /// Buckets to monitor (empty means all buckets)
    pub buckets: Vec<String>,
}

impl Default for S3CollectorSettings {
    fn default() -> Self {
        Self {
            base: AwsSdkCollectorSettings::default(),
            collect_bucket_metrics: true,
            buckets: Vec::new(),
        }
    }
}

/// S3 collector for monitoring S3 operations
pub struct S3Collector {
    /// Collector configuration
    config: CollectorConfig,
    /// S3 collector settings
    settings: S3CollectorSettings,
    /// S3 operation stats
    operations: RwLock<HashMap<String, u64>>,
}

impl S3Collector {
    /// Create a new S3 collector
    pub fn new(config: CollectorConfig) -> Result<Self> {
        let default_settings = S3CollectorSettings::default();
        let settings = merge_aws_settings(&default_settings.base, config.settings.as_ref())
            .map(|base| S3CollectorSettings {
                base,
                ..default_settings
            })?;
            
        Ok(Self {
            config,
            settings,
            operations: RwLock::new(HashMap::new()),
        })
    }
}

#[async_trait]
impl Collector for S3Collector {
    fn metadata(&self) -> CollectorMetadata {
        CollectorMetadata {
            id: self.config.id.clone(),
            name: "AWS S3 Collector".to_string(),
            description: "Collects metrics from AWS S3 operations".to_string(),
            source_type: "aws.s3".to_string(),
            enabled_by_default: true,
        }
    }
    
    fn config(&self) -> CollectorConfig {
        self.config.clone()
    }
    
    fn update_config(&mut self, config: CollectorConfig) -> Result<()> {
        // Update settings from config
        if let Some(settings) = &config.settings {
            let default_settings = S3CollectorSettings::default();
            self.settings = merge_aws_settings(&default_settings.base, Some(settings))
                .map(|base| S3CollectorSettings {
                    base,
                    ..default_settings
                })?;
        }
        
        self.config = config;
        Ok(())
    }
    
    async fn initialize(&mut self) -> Result<()> {
        info!("Initializing S3 collector");
        Ok(())
    }
    
    async fn collect(&self) -> Result<HashMap<String, MetricValue>> {
        debug!("Collecting S3 metrics");
        
        // For now, just return basic operation counts
        let operations = self.operations.read().await;
        let mut metrics = HashMap::new();
        
        for (op, count) in operations.iter() {
            metrics.insert(format!("s3.operation.{}", op), MetricValue::Counter(*count));
        }
        
        Ok(metrics)
    }
    
    async fn shutdown(&self) -> Result<()> {
        debug!("Shutting down S3 collector");
        Ok(())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Factory for creating S3 collectors
pub struct S3CollectorFactory {
    metadata: CollectorMetadata,
}

impl S3CollectorFactory {
    /// Create a new S3 collector factory
    pub fn new() -> Self {
        Self {
            metadata: CollectorMetadata {
                id: "aws.s3".to_string(),
                name: "AWS S3 Collector".to_string(),
                description: "Collects metrics from AWS S3 operations".to_string(),
                source_type: "aws.s3".to_string(),
                enabled_by_default: true,
            },
        }
    }
}

#[async_trait]
impl CollectorFactory for S3CollectorFactory {
    async fn create(&self, config: CollectorConfig) -> Result<Box<dyn Collector>> {
        let collector = S3Collector::new(config)?;
        Ok(Box::new(collector))
    }
    
    fn metadata(&self) -> CollectorMetadata {
        self.metadata.clone()
    }
}