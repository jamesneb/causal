// agent/collectors/src/database/nosql.rs

use std::any::Any;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use agent_core::telemetry::metrics::MetricValue;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::{Collector, CollectorConfig, CollectorFactory, CollectorMetadata};
use super::{DatabaseCollectorSettings, merge_db_settings, schema::{DatabaseQueryMetrics, QueryType, TableMetrics}};

/// NoSQL database collector settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoSqlDatabaseCollectorSettings {
    /// Base database settings
    #[serde(flatten)]
    pub base: DatabaseCollectorSettings,
    /// Database type (e.g., "mongodb", "dynamodb", "cosmosdb")
    pub database_type: String,
    /// Database instance name
    pub instance_name: Option<String>,
    /// Connection string (sanitized)
    pub connection_string: Option<String>,
    /// Collections/tables to monitor (empty means all)
    pub collections: Vec<String>,
}

impl Default for NoSqlDatabaseCollectorSettings {
    fn default() -> Self {
        Self {
            base: DatabaseCollectorSettings::default(),
            database_type: "generic".to_string(),
            instance_name: None,
            connection_string: None,
            collections: Vec::new(),
        }
    }
}

/// NoSQL operation type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NoSqlOperationType {
    Find,
    Insert,
    Update,
    Delete,
    Aggregate,
    Count,
    Distinct,
    MapReduce,
    Scan,
    Query,
    BatchGet,
    BatchWrite,
    Other(String),
}

impl NoSqlOperationType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Find => "find",
            Self::Insert => "insert",
            Self::Update => "update",
            Self::Delete => "delete",
            Self::Aggregate => "aggregate",
            Self::Count => "count",
            Self::Distinct => "distinct",
            Self::MapReduce => "map_reduce",
            Self::Scan => "scan",
            Self::Query => "query",
            Self::BatchGet => "batch_get",
            Self::BatchWrite => "batch_write",
            Self::Other(s) => s,
        }
    }
}

/// NoSQL operation metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
struct NoSqlOperationMetrics {
    /// Operation ID
    pub operation_id: String,
    /// Connection ID
    pub connection_id: Option<String>,
    /// Operation type
    pub operation_type: NoSqlOperationType,
    /// Collection/table name
    pub collection: Option<String>,
    /// Operation timestamp
    pub timestamp: u64,
    /// Operation duration in milliseconds
    pub duration_ms: u64,
    /// Query filter (sanitized)
    pub filter: Option<String>,
    /// Number of documents affected
    pub docs_affected: Option<u64>,
    /// Number of documents returned
    pub docs_returned: Option<u64>,
    /// Execution info
    pub execution_info: Option<String>,
    /// Error message if any
    pub error: Option<String>,
    /// Operation-specific metrics
    pub metadata: Option<HashMap<String, String>>,
}

/// NoSQL database collector for monitoring NoSQL database operations
pub struct NoSqlDatabaseCollector {
    /// Collector configuration
    config: CollectorConfig,
    /// NoSQL database collector settings
    settings: NoSqlDatabaseCollectorSettings,
    /// Recent operations
    operations: RwLock<VecDeque<NoSqlOperationMetrics>>,
    /// Collection metrics
    collections: RwLock<HashMap<String, TableMetrics>>,
    /// Operation type counts
    operation_type_counts: RwLock<HashMap<String, u64>>,
    /// Total operation count
    total_operations: RwLock<u64>,
    /// Error count
    error_count: RwLock<u64>,
    /// Connection count
    connection_count: RwLock<u64>,
}

impl NoSqlDatabaseCollector {
    /// Create a new NoSQL database collector
    pub fn new(config: CollectorConfig) -> Result<Self> {
        let default_settings = NoSqlDatabaseCollectorSettings::default();
        let base = merge_db_settings(&default_settings.base, config.settings.as_ref())?;
        
        // Create settings with merged base settings
        let mut settings = NoSqlDatabaseCollectorSettings {
            base,
            ..default_settings
        };
        
        // Override other settings from config
        if let Some(cfg) = &config.settings {
            if let Some(db_type) = cfg.get("database_type") {
                settings.database_type = db_type.clone();
            }
            
            if let Some(instance) = cfg.get("instance_name") {
                settings.instance_name = Some(instance.clone());
            }
            
            if let Some(conn_str) = cfg.get("connection_string") {
                settings.connection_string = Some(super::sql::sanitize_connection_string(conn_str));
            }
            
            if let Some(collections) = cfg.get("collections") {
                settings.collections = collections
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }
        
        Ok(Self {
            config,
            settings,
            operations: RwLock::new(VecDeque::with_capacity(settings.base.max_queries)),
            collections: RwLock::new(HashMap::new()),
            operation_type_counts: RwLock::new(HashMap::new()),
            total_operations: RwLock::new(0),
            error_count: RwLock::new(0),
            connection_count: RwLock::new(0),
        })
    }
    
    /// Track a NoSQL operation
    pub async fn track_operation(
        &self,
        operation_type: NoSqlOperationType,
        collection: Option<&str>,
        duration_ms: u64,
        docs_affected: Option<u64>,
        docs_returned: Option<u64>,
        filter: Option<&str>,
        error: Option<String>,
    ) -> String {
        // Generate a unique operation ID
        let operation_id = Uuid::new_v4().to_string();
        
        // Update operation type counts
        {
            let mut counts = self.operation_type_counts.write().await;
            *counts.entry(operation_type.as_str().to_string()).or_default() += 1;
        }
        
        // Update total operation count
        {
            let mut total = self.total_operations.write().await;
            *total += 1;
        }
        
        // Update error count if needed
        if error.is_some() {
            let mut errors = self.error_count.write().await;
            *errors += 1;
        }
        
        // Create operation metrics
        let metrics = NoSqlOperationMetrics {
            operation_id: operation_id.clone(),
            connection_id: None,
            operation_type: operation_type.clone(),
            collection: collection.map(|s| s.to_string()),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            duration_ms,
            filter: filter.map(|f| {
                if self.settings.base.collect_queries {
                    f.to_string()
                } else {
                    "[redacted]".to_string()
                }
            }),
            docs_affected,
            docs_returned,
            execution_info: None,
            error,
            metadata: None,
        };
        
        // Update collection metrics if needed
        if self.settings.base.collect_table_metrics && collection.is_some() {
            let collection_name = collection.unwrap();
            
            // Check if this collection should be tracked
            let should_track = self.settings.collections.is_empty() || 
                self.settings.collections.iter().any(|c| c == collection_name);
            
            if should_track {
                let mut collections = self.collections.write().await;
                let collection_metrics = collections.entry(collection_name.to_string()).or_insert_with(|| TableMetrics {
                    table_name: collection_name.to_string(),
                    row_count: None,
                    size_bytes: None,
                    reads: 0,
                    writes: 0,
                    last_accessed: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64,
                    metadata: None,
                });
                
                // Update reads/writes
                match operation_type {
                    NoSqlOperationType::Find | 
                    NoSqlOperationType::Aggregate | 
                    NoSqlOperationType::Count |
                    NoSqlOperationType::Distinct |
                    NoSqlOperationType::MapReduce |
                    NoSqlOperationType::Scan |
                    NoSqlOperationType::Query |
                    NoSqlOperationType::BatchGet => {
                        collection_metrics.reads += 1;
                    }
                    NoSqlOperationType::Insert | 
                    NoSqlOperationType::Update |
                    NoSqlOperationType::Delete |
                    NoSqlOperationType::BatchWrite => {
                        collection_metrics.writes += 1;
                    }
                    _ => {}
                }
                
                // Update last accessed
                collection_metrics.last_accessed = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
            }
        }
        
        // Store operation metrics
        self.store_operation(metrics).await;
        
        operation_id
    }
    
    /// Store an operation
    async fn store_operation(&self, metrics: NoSqlOperationMetrics) {
        let mut operations = self.operations.write().await;
        
        // Remove oldest if at capacity
        if operations.len() >= self.settings.base.max_queries {
            operations.pop_front();
        }
        
        operations.push_back(metrics);
    }
    
    /// Track a new database connection
    pub async fn track_connection(&self) {
        let mut connections = self.connection_count.write().await;
        *connections += 1;
    }
}

#[async_trait]
impl Collector for NoSqlDatabaseCollector {
    fn metadata(&self) -> CollectorMetadata {
        CollectorMetadata {
            id: self.config.id.clone(),
            name: format!("{} NoSQL Database Collector", self.settings.database_type),
            description: format!("Collects metrics from {} NoSQL database operations", self.settings.database_type),
            source_type: "database.nosql".to_string(),
            enabled_by_default: true,
        }
    }
    
    fn config(&self) -> CollectorConfig {
        self.config.clone()
    }
    
    fn update_config(&mut self, config: CollectorConfig) -> Result<()> {
        // Update settings from config
        if let Some(settings) = &config.settings {
            let default_settings = NoSqlDatabaseCollectorSettings::default();
            let base = merge_db_settings(&default_settings.base, Some(settings))?;
            
            self.settings = NoSqlDatabaseCollectorSettings {
                base,
                ..self.settings.clone()
            };
            
            // Update other settings
            if let Some(db_type) = settings.get("database_type") {
                self.settings.database_type = db_type.clone();
            }
            
            if let Some(instance) = settings.get("instance_name") {
                self.settings.instance_name = Some(instance.clone());
            }
            
            if let Some(conn_str) = settings.get("connection_string") {
                self.settings.connection_string = Some(super::sql::sanitize_connection_string(conn_str));
            }
            
            if let Some(collections) = settings.get("collections") {
                self.settings.collections = collections
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }
        
        self.config = config;
        Ok(())
    }
    
    async fn initialize(&mut self) -> Result<()> {
        info!("Initializing NoSQL database collector for {}", self.settings.database_type);
        Ok(())
    }
    
    async fn collect(&self) -> Result<HashMap<String, MetricValue>> {
        debug!("Collecting NoSQL database metrics");
        
        let mut metrics = HashMap::new();
        let prefix = format!("database.nosql.{}", self.settings.database_type.to_lowercase());
        
        // Add total operation count
        let total = *self.total_operations.read().await;
        metrics.insert(format!("{}.operations.total", prefix), MetricValue::Counter(total));
        
        // Add error count
        let errors = *self.error_count.read().await;
        metrics.insert(format!("{}.operations.errors", prefix), MetricValue::Counter(errors));
        
        // Add connection count
        let connections = *self.connection_count.read().await;
        metrics.insert(format!("{}.connections", prefix), MetricValue::Counter(connections));
        
        // Add operation type counts
        let operation_types = self.operation_type_counts.read().await;
        for (op_type, count) in operation_types.iter() {
            metrics.insert(
                format!("{}.operations.type.{}", prefix, op_type),
                MetricValue::Counter(*count),
            );
        }
        
        // Add operation duration metrics
        let operations = self.operations.read().await;
        if !operations.is_empty() {
            let durations: Vec<f64> = operations
                .iter()
                .map(|o| o.duration_ms as f64)
                .collect();
            
            metrics.insert(
                format!("{}.operation.duration_ms", prefix),
                MetricValue::Histogram(durations),
            );
        }
        
        // Add collection metrics if enabled
        if self.settings.base.collect_table_metrics {
            let collections = self.collections.read().await;
            for (name, collection) in collections.iter() {
                let collection_prefix = format!("{}.collection.{}", prefix, sanitize_metric_name(name));
                
                metrics.insert(
                    format!("{}.reads", collection_prefix),
                    MetricValue::Counter(collection.reads),
                );
                
                metrics.insert(
                    format!("{}.writes", collection_prefix),
                    MetricValue::Counter(collection.writes),
                );
                
                if let Some(doc_count) = collection.row_count {
                    metrics.insert(
                        format!("{}.documents", collection_prefix),
                        MetricValue::Gauge(doc_count as f64),
                    );
                }
                
                if let Some(size) = collection.size_bytes {
                    metrics.insert(
                        format!("{}.size_bytes", collection_prefix),
                        MetricValue::Gauge(size as f64),
                    );
                }
            }
        }
        
        Ok(metrics)
    }
    
    async fn shutdown(&self) -> Result<()> {
        debug!("Shutting down NoSQL database collector");
        Ok(())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Factory for creating NoSQL database collectors
pub struct NoSqlDatabaseCollectorFactory {
    metadata: CollectorMetadata,
}

impl NoSqlDatabaseCollectorFactory {
    /// Create a new NoSQL database collector factory
    pub fn new() -> Self {
        Self {
            metadata: CollectorMetadata {
                id: "database.nosql".to_string(),
                name: "NoSQL Database Collector".to_string(),
                description: "Collects metrics from NoSQL database operations".to_string(),
                source_type: "database.nosql".to_string(),
                enabled_by_default: true,
            },
        }
    }
}

#[async_trait]
impl CollectorFactory for NoSqlDatabaseCollectorFactory {
    async fn create(&self, config: CollectorConfig) -> Result<Box<dyn Collector>> {
        let collector = NoSqlDatabaseCollector::new(config)?;
        Ok(Box::new(collector))
    }
    
    fn metadata(&self) -> CollectorMetadata {
        self.metadata.clone()
    }
}

/// Sanitize a string for use in a metric name
fn sanitize_metric_name(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' => c.to_ascii_lowercase(),
            _ => '_',
        })
        .collect()
}