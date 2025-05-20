// agent/collectors/src/aws_sdk/dynamodb.rs

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

/// DynamoDB collector settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamoDbCollectorSettings {
    /// Base AWS SDK settings
    #[serde(flatten)]
    pub base: AwsSdkCollectorSettings,
    /// Whether to collect table metrics
    pub collect_table_metrics: bool,
    /// Tables to monitor (empty means all tables)
    pub tables: Vec<String>,
}

impl Default for DynamoDbCollectorSettings {
    fn default() -> Self {
        Self {
            base: AwsSdkCollectorSettings::default(),
            collect_table_metrics: true,
            tables: Vec::new(),
        }
    }
}

/// DynamoDB collector for monitoring DynamoDB operations
pub struct DynamoDbCollector {
    /// Collector configuration
    config: CollectorConfig,
    /// DynamoDB collector settings
    settings: DynamoDbCollectorSettings,
    /// DynamoDB operation stats
    operations: RwLock<HashMap<String, u64>>,
}

impl DynamoDbCollector {
    /// Create a new DynamoDB collector
    pub fn new(config: CollectorConfig) -> Result<Self> {
        let default_settings = DynamoDbCollectorSettings::default();
        let settings = merge_aws_settings(&default_settings.base, config.settings.as_ref())
            .map(|base| DynamoDbCollectorSettings {
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
impl Collector for DynamoDbCollector {
    fn metadata(&self) -> CollectorMetadata {
        CollectorMetadata {
            id: self.config.id.clone(),
            name: "AWS DynamoDB Collector".to_string(),
            description: "Collects metrics from AWS DynamoDB operations".to_string(),
            source_type: "aws.dynamodb".to_string(),
            enabled_by_default: true,
        }
    }
    
    fn config(&self) -> CollectorConfig {
        self.config.clone()
    }
    
    fn update_config(&mut self, config: CollectorConfig) -> Result<()> {
        // Update settings from config
        if let Some(settings) = &config.settings {
            let default_settings = DynamoDbCollectorSettings::default();
            self.settings = merge_aws_settings(&default_settings.base, Some(settings))
                .map(|base| DynamoDbCollectorSettings {
                    base,
                    ..default_settings
                })?;
        }
        
        self.config = config;
        Ok(())
    }
    
    async fn initialize(&mut self) -> Result<()> {
        info!("Initializing DynamoDB collector");
        Ok(())
    }
    
    async fn collect(&self) -> Result<HashMap<String, MetricValue>> {
        debug!("Collecting DynamoDB metrics");
        
        // For now, just return basic operation counts
        let operations = self.operations.read().await;
        let mut metrics = HashMap::new();
        
        for (op, count) in operations.iter() {
            metrics.insert(format!("dynamodb.operation.{}", op), MetricValue::Counter(*count));
        }
        
        Ok(metrics)
    }
    
    async fn shutdown(&self) -> Result<()> {
        debug!("Shutting down DynamoDB collector");
        Ok(())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Factory for creating DynamoDB collectors
pub struct DynamoDbCollectorFactory {
    metadata: CollectorMetadata,
}

impl DynamoDbCollectorFactory {
    /// Create a new DynamoDB collector factory
    pub fn new() -> Self {
        Self {
            metadata: CollectorMetadata {
                id: "aws.dynamodb".to_string(),
                name: "AWS DynamoDB Collector".to_string(),
                description: "Collects metrics from AWS DynamoDB operations".to_string(),
                source_type: "aws.dynamodb".to_string(),
                enabled_by_default: true,
            },
        }
    }
}

#[async_trait]
impl CollectorFactory for DynamoDbCollectorFactory {
    async fn create(&self, config: CollectorConfig) -> Result<Box<dyn Collector>> {
        let collector = DynamoDbCollector::new(config)?;
        Ok(Box::new(collector))
    }
    
    fn metadata(&self) -> CollectorMetadata {
        self.metadata.clone()
    }
}