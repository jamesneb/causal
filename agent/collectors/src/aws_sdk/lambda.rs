// agent/collectors/src/aws_sdk/lambda.rs

use std::any::Any;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::time::Duration;

use agent_core::telemetry::metrics::MetricValue;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::{Collector, CollectorBuilder, CollectorConfig, CollectorFactory, CollectorMetadata};
use super::{AwsSdkCollectorSettings, merge_aws_settings};

/// Lambda collector settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LambdaCollectorSettings {
    /// Base AWS SDK settings
    #[serde(flatten)]
    pub base: AwsSdkCollectorSettings,
    /// Whether to collect function configuration
    pub collect_configuration: bool,
    /// Whether to collect invocation metrics
    pub collect_invocations: bool,
    /// Whether to collect cold start events
    pub collect_cold_starts: bool,
    /// Whether to collect function versions
    pub collect_versions: bool,
    /// Whether to collect execution environment info
    pub collect_environment_info: bool,
}

impl Default for LambdaCollectorSettings {
    fn default() -> Self {
        Self {
            base: AwsSdkCollectorSettings::default(),
            collect_configuration: true,
            collect_invocations: true,
            collect_cold_starts: true,
            collect_versions: true,
            collect_environment_info: true,
        }
    }
}

/// Lambda collector for monitoring Lambda execution environment
pub struct LambdaCollector {
    /// Collector configuration
    config: CollectorConfig,
    /// Lambda collector settings
    settings: LambdaCollectorSettings,
    /// Lambda function name
    function_name: Option<String>,
    /// Lambda function version
    function_version: Option<String>,
    /// Whether this is running in a Lambda
    is_lambda: bool,
    /// Memory limit in MB
    memory_limit: Option<u32>,
    /// Initialization type
    initialization_type: Option<String>,
    /// Cold start tracking
    cold_start: RwLock<bool>,
    /// Invocation count
    invocation_count: RwLock<u64>,
}

impl LambdaCollector {
    /// Create a new Lambda collector
    pub fn new(config: CollectorConfig) -> Result<Self> {
        let default_settings = LambdaCollectorSettings::default();
        let settings = merge_aws_settings(&default_settings.base, config.settings.as_ref())
            .map(|base| LambdaCollectorSettings {
                base,
                ..default_settings
            })?;
        
        // Check if we're running in Lambda
        let function_name = env::var("AWS_LAMBDA_FUNCTION_NAME").ok();
        let function_version = env::var("AWS_LAMBDA_FUNCTION_VERSION").ok();
        let memory_limit = env::var("AWS_LAMBDA_FUNCTION_MEMORY_SIZE")
            .ok()
            .and_then(|s| s.parse::<u32>().ok());
        let initialization_type = env::var("AWS_LAMBDA_INITIALIZATION_TYPE").ok();
        
        // Detect if we're running in Lambda
        let is_lambda = function_name.is_some() && function_version.is_some();
        
        Ok(Self {
            config,
            settings,
            function_name,
            function_version,
            is_lambda,
            memory_limit,
            initialization_type,
            cold_start: RwLock::new(true),
            invocation_count: RwLock::new(0),
        })
    }
    
    /// Get execution environment info
    async fn collect_environment_info(&self) -> HashMap<String, MetricValue> {
        let mut metrics = HashMap::new();
        
        // Basic Lambda environment info
        if self.is_lambda {
            // Memory limit
            if let Some(memory_mb) = self.memory_limit {
                metrics.insert("lambda.memory_limit_mb".to_string(), MetricValue::Gauge(memory_mb as f64));
            }
            
            // Function name
            if let Some(name) = &self.function_name {
                metrics.insert("lambda.function_name".to_string(), MetricValue::Set(vec![name.clone()]));
            }
            
            // Function version
            if let Some(version) = &self.function_version {
                metrics.insert("lambda.function_version".to_string(), MetricValue::Set(vec![version.clone()]));
            }
            
            // Region
            if let Ok(region) = env::var("AWS_REGION") {
                metrics.insert("lambda.region".to_string(), MetricValue::Set(vec![region]));
            }
            
            // Execution environment
            if let Ok(exec_env) = env::var("AWS_EXECUTION_ENV") {
                metrics.insert("lambda.execution_environment".to_string(), MetricValue::Set(vec![exec_env]));
            }
            
            // Log group/stream
            if let Ok(log_group) = env::var("AWS_LAMBDA_LOG_GROUP_NAME") {
                metrics.insert("lambda.log_group".to_string(), MetricValue::Set(vec![log_group]));
            }
            
            if let Ok(log_stream) = env::var("AWS_LAMBDA_LOG_STREAM_NAME") {
                metrics.insert("lambda.log_stream".to_string(), MetricValue::Set(vec![log_stream]));
            }
            
            // Initialization type
            if let Some(init_type) = &self.initialization_type {
                metrics.insert("lambda.initialization_type".to_string(), MetricValue::Set(vec![init_type.clone()]));
            }
        }
        
        metrics
    }
    
    /// Track cold start
    async fn track_invocation(&self) -> HashMap<String, MetricValue> {
        let mut metrics = HashMap::new();
        
        // Increment invocation count
        let mut count = self.invocation_count.write().await;
        *count += 1;
        metrics.insert("lambda.invocation_count".to_string(), MetricValue::Counter(*count));
        
        // Check cold start
        let mut cold_start = self.cold_start.write().await;
        if *cold_start {
            metrics.insert("lambda.cold_start".to_string(), MetricValue::Counter(1));
            *cold_start = false;
        }
        
        metrics
    }
}

#[async_trait]
impl Collector for LambdaCollector {
    fn metadata(&self) -> CollectorMetadata {
        CollectorMetadata {
            id: self.config.id.clone(),
            name: "AWS Lambda Collector".to_string(),
            description: "Collects metrics from AWS Lambda execution environment".to_string(),
            source_type: "aws.lambda".to_string(),
            enabled_by_default: true,
        }
    }
    
    fn config(&self) -> CollectorConfig {
        self.config.clone()
    }
    
    fn update_config(&mut self, config: CollectorConfig) -> Result<()> {
        // Update settings from config
        if let Some(settings) = &config.settings {
            let default_settings = LambdaCollectorSettings::default();
            self.settings = merge_aws_settings(&default_settings.base, Some(settings))
                .map(|base| LambdaCollectorSettings {
                    base,
                    ..default_settings
                })?;
        }
        
        self.config = config;
        Ok(())
    }
    
    async fn initialize(&mut self) -> Result<()> {
        info!("Initializing Lambda collector");
        
        // Reset cold start flag
        *self.cold_start.write().await = true;
        
        Ok(())
    }
    
    async fn collect(&self) -> Result<HashMap<String, MetricValue>> {
        debug!("Collecting Lambda metrics");
        
        let mut metrics = HashMap::new();
        
        // Track invocation and cold start
        if self.settings.collect_invocations {
            metrics.extend(self.track_invocation().await);
        }
        
        // Collect environment info
        if self.settings.collect_environment_info {
            metrics.extend(self.collect_environment_info().await);
        }
        
        Ok(metrics)
    }
    
    async fn shutdown(&self) -> Result<()> {
        debug!("Shutting down Lambda collector");
        Ok(())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Factory for creating Lambda collectors
pub struct LambdaCollectorFactory {
    metadata: CollectorMetadata,
}

impl LambdaCollectorFactory {
    /// Create a new Lambda collector factory
    pub fn new() -> Self {
        Self {
            metadata: CollectorMetadata {
                id: "aws.lambda".to_string(),
                name: "AWS Lambda Collector".to_string(),
                description: "Collects metrics from AWS Lambda execution environment".to_string(),
                source_type: "aws.lambda".to_string(),
                enabled_by_default: true,
            },
        }
    }
}

#[async_trait]
impl CollectorFactory for LambdaCollectorFactory {
    async fn create(&self, config: CollectorConfig) -> Result<Box<dyn Collector>> {
        let collector = LambdaCollector::new(config)?;
        Ok(Box::new(collector))
    }
    
    fn metadata(&self) -> CollectorMetadata {
        self.metadata.clone()
    }
}