// agent/collectors/src/database/sql.rs

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

/// SQL database collector settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqlDatabaseCollectorSettings {
    /// Base database settings
    #[serde(flatten)]
    pub base: DatabaseCollectorSettings,
    /// Database type
    pub database_type: String,
    /// Database instance name
    pub instance_name: Option<String>,
    /// Connection string (sanitized)
    pub connection_string: Option<String>,
    /// Tables to monitor (empty means all)
    pub tables: Vec<String>,
}

impl Default for SqlDatabaseCollectorSettings {
    fn default() -> Self {
        Self {
            base: DatabaseCollectorSettings::default(),
            database_type: "generic".to_string(),
            instance_name: None,
            connection_string: None,
            tables: Vec::new(),
        }
    }
}

/// SQL database collector for monitoring SQL database operations
pub struct SqlDatabaseCollector {
    /// Collector configuration
    config: CollectorConfig,
    /// SQL database collector settings
    settings: SqlDatabaseCollectorSettings,
    /// Recent queries
    queries: RwLock<VecDeque<DatabaseQueryMetrics>>,
    /// Table metrics
    tables: RwLock<HashMap<String, TableMetrics>>,
    /// Query type counts
    query_type_counts: RwLock<HashMap<String, u64>>,
    /// Total query count
    total_queries: RwLock<u64>,
    /// Error count
    error_count: RwLock<u64>,
    /// Connection count
    connection_count: RwLock<u64>,
}

impl SqlDatabaseCollector {
    /// Create a new SQL database collector
    pub fn new(config: CollectorConfig) -> Result<Self> {
        let default_settings = SqlDatabaseCollectorSettings::default();
        let base = merge_db_settings(&default_settings.base, config.settings.as_ref())?;
        
        // Create settings with merged base settings
        let mut settings = SqlDatabaseCollectorSettings {
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
                settings.connection_string = Some(sanitize_connection_string(conn_str));
            }
            
            if let Some(tables) = cfg.get("tables") {
                settings.tables = tables
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }
        
        Ok(Self {
            config,
            settings,
            queries: RwLock::new(VecDeque::with_capacity(settings.base.max_queries)),
            tables: RwLock::new(HashMap::new()),
            query_type_counts: RwLock::new(HashMap::new()),
            total_queries: RwLock::new(0),
            error_count: RwLock::new(0),
            connection_count: RwLock::new(0),
        })
    }
    
    /// Track a SQL query
    pub async fn track_query(
        &self,
        query_text: &str,
        duration_ms: u64,
        rows_affected: Option<u64>,
        rows_returned: Option<u64>,
        error: Option<String>,
    ) -> String {
        // Generate a unique query ID
        let query_id = Uuid::new_v4().to_string();
        
        // Determine query type
        let query_type = QueryType::from_query(query_text);
        let query_type_str = match &query_type {
            QueryType::Select => "select",
            QueryType::Insert => "insert",
            QueryType::Update => "update",
            QueryType::Delete => "delete",
            QueryType::Create => "create",
            QueryType::Alter => "alter",
            QueryType::Drop => "drop",
            QueryType::Begin => "begin",
            QueryType::Commit => "commit",
            QueryType::Rollback => "rollback",
            QueryType::Other(s) => s.to_lowercase().as_str(),
        };
        
        // Update query type counts
        {
            let mut counts = self.query_type_counts.write().await;
            *counts.entry(query_type_str.to_string()).or_default() += 1;
        }
        
        // Update total query count
        {
            let mut total = self.total_queries.write().await;
            *total += 1;
        }
        
        // Update error count if needed
        if error.is_some() {
            let mut errors = self.error_count.write().await;
            *errors += 1;
        }
        
        // Create query metrics
        let metrics = DatabaseQueryMetrics {
            query_id: query_id.clone(),
            connection_id: None,
            query_type,
            query_text: if self.settings.base.collect_queries {
                Some(truncate_query(query_text, self.settings.base.max_query_length))
            } else {
                None
            },
            parameters: None,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            duration_ms,
            rows_affected,
            rows_returned,
            execution_plan: None,
            error,
            metadata: None,
        };
        
        // Update table metrics if needed
        if self.settings.base.collect_table_metrics {
            let table_name = extract_table_name(query_text);
            if let Some(table) = table_name {
                // Check if this table should be tracked
                let should_track = self.settings.tables.is_empty() || 
                    self.settings.tables.iter().any(|t| t == &table);
                
                if should_track {
                    let mut tables = self.tables.write().await;
                    let table_metrics = tables.entry(table.clone()).or_insert_with(|| TableMetrics {
                        table_name: table.clone(),
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
                    match query_type {
                        QueryType::Select => table_metrics.reads += 1,
                        QueryType::Insert | QueryType::Update | QueryType::Delete => {
                            table_metrics.writes += 1;
                        }
                        _ => {}
                    }
                    
                    // Update last accessed
                    table_metrics.last_accessed = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64;
                }
            }
        }
        
        // Store query metrics
        self.store_query(metrics).await;
        
        query_id
    }
    
    /// Store a query
    async fn store_query(&self, metrics: DatabaseQueryMetrics) {
        let mut queries = self.queries.write().await;
        
        // Remove oldest if at capacity
        if queries.len() >= self.settings.base.max_queries {
            queries.pop_front();
        }
        
        queries.push_back(metrics);
    }
    
    /// Track a new database connection
    pub async fn track_connection(&self) {
        let mut connections = self.connection_count.write().await;
        *connections += 1;
    }
}

#[async_trait]
impl Collector for SqlDatabaseCollector {
    fn metadata(&self) -> CollectorMetadata {
        CollectorMetadata {
            id: self.config.id.clone(),
            name: format!("{} SQL Database Collector", self.settings.database_type),
            description: format!("Collects metrics from {} SQL database operations", self.settings.database_type),
            source_type: "database.sql".to_string(),
            enabled_by_default: true,
        }
    }
    
    fn config(&self) -> CollectorConfig {
        self.config.clone()
    }
    
    fn update_config(&mut self, config: CollectorConfig) -> Result<()> {
        // Update settings from config
        if let Some(settings) = &config.settings {
            let default_settings = SqlDatabaseCollectorSettings::default();
            let base = merge_db_settings(&default_settings.base, Some(settings))?;
            
            self.settings = SqlDatabaseCollectorSettings {
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
                self.settings.connection_string = Some(sanitize_connection_string(conn_str));
            }
            
            if let Some(tables) = settings.get("tables") {
                self.settings.tables = tables
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
        info!("Initializing SQL database collector for {}", self.settings.database_type);
        Ok(())
    }
    
    async fn collect(&self) -> Result<HashMap<String, MetricValue>> {
        debug!("Collecting SQL database metrics");
        
        let mut metrics = HashMap::new();
        let prefix = format!("database.sql.{}", self.settings.database_type.to_lowercase());
        
        // Add total query count
        let total = *self.total_queries.read().await;
        metrics.insert(format!("{}.queries.total", prefix), MetricValue::Counter(total));
        
        // Add error count
        let errors = *self.error_count.read().await;
        metrics.insert(format!("{}.queries.errors", prefix), MetricValue::Counter(errors));
        
        // Add connection count
        let connections = *self.connection_count.read().await;
        metrics.insert(format!("{}.connections", prefix), MetricValue::Counter(connections));
        
        // Add query type counts
        let query_types = self.query_type_counts.read().await;
        for (query_type, count) in query_types.iter() {
            metrics.insert(
                format!("{}.queries.type.{}", prefix, query_type),
                MetricValue::Counter(*count),
            );
        }
        
        // Add query duration metrics
        let queries = self.queries.read().await;
        if !queries.is_empty() {
            let durations: Vec<f64> = queries
                .iter()
                .map(|q| q.duration_ms as f64)
                .collect();
            
            metrics.insert(
                format!("{}.query.duration_ms", prefix),
                MetricValue::Histogram(durations),
            );
        }
        
        // Add table metrics if enabled
        if self.settings.base.collect_table_metrics {
            let tables = self.tables.read().await;
            for (name, table) in tables.iter() {
                let table_prefix = format!("{}.table.{}", prefix, sanitize_metric_name(name));
                
                metrics.insert(
                    format!("{}.reads", table_prefix),
                    MetricValue::Counter(table.reads),
                );
                
                metrics.insert(
                    format!("{}.writes", table_prefix),
                    MetricValue::Counter(table.writes),
                );
                
                if let Some(row_count) = table.row_count {
                    metrics.insert(
                        format!("{}.rows", table_prefix),
                        MetricValue::Gauge(row_count as f64),
                    );
                }
                
                if let Some(size) = table.size_bytes {
                    metrics.insert(
                        format!("{}.size_bytes", table_prefix),
                        MetricValue::Gauge(size as f64),
                    );
                }
            }
        }
        
        Ok(metrics)
    }
    
    async fn shutdown(&self) -> Result<()> {
        debug!("Shutting down SQL database collector");
        Ok(())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Factory for creating SQL database collectors
pub struct SqlDatabaseCollectorFactory {
    metadata: CollectorMetadata,
}

impl SqlDatabaseCollectorFactory {
    /// Create a new SQL database collector factory
    pub fn new() -> Self {
        Self {
            metadata: CollectorMetadata {
                id: "database.sql".to_string(),
                name: "SQL Database Collector".to_string(),
                description: "Collects metrics from SQL database operations".to_string(),
                source_type: "database.sql".to_string(),
                enabled_by_default: true,
            },
        }
    }
}

#[async_trait]
impl CollectorFactory for SqlDatabaseCollectorFactory {
    async fn create(&self, config: CollectorConfig) -> Result<Box<dyn Collector>> {
        let collector = SqlDatabaseCollector::new(config)?;
        Ok(Box::new(collector))
    }
    
    fn metadata(&self) -> CollectorMetadata {
        self.metadata.clone()
    }
}

/// Sanitize a connection string by removing sensitive information
fn sanitize_connection_string(conn_str: &str) -> String {
    // Common patterns to sanitize
    let patterns = [
        ("password=", "password=*****"),
        ("pwd=", "pwd=*****"),
        ("user id=", "user id=*****"),
        ("uid=", "uid=*****"),
        ("username=", "username=*****"),
        ("apikey=", "apikey=*****"),
        ("api_key=", "api_key=*****"),
        ("secret=", "secret=*****"),
    ];
    
    let mut sanitized = conn_str.to_lowercase();
    for (pattern, replacement) in patterns.iter() {
        if let Some(pos) = sanitized.find(pattern) {
            // Find the value part
            let start = pos + pattern.len();
            let end = sanitized[start..]
                .find(|c| c == ';' || c == ' ')
                .map(|p| p + start)
                .unwrap_or(sanitized.len());
            
            // Replace the value with *****
            sanitized = format!(
                "{}{}{}",
                &sanitized[0..start],
                "*****",
                &sanitized[end..]
            );
        }
    }
    
    sanitized
}

/// Extract table name from a SQL query
fn extract_table_name(query: &str) -> Option<String> {
    // This is a very simplistic implementation
    // A real implementation would use a SQL parser
    
    let query = query.trim().to_uppercase();
    
    // Handle SELECT
    if query.starts_with("SELECT") {
        // Try to find FROM clause
        if let Some(from_pos) = query.find(" FROM ") {
            let after_from = &query[from_pos + 6..];
            return after_from
                .split_whitespace()
                .next()
                .map(|s| s.trim().to_string());
        }
    }
    
    // Handle INSERT
    if query.starts_with("INSERT INTO") {
        let after_insert = &query[12..];
        return after_insert
            .split_whitespace()
            .next()
            .map(|s| s.trim().to_string());
    }
    
    // Handle UPDATE
    if query.starts_with("UPDATE") {
        let after_update = &query[7..];
        return after_update
            .split_whitespace()
            .next()
            .map(|s| s.trim().to_string());
    }
    
    // Handle DELETE
    if query.starts_with("DELETE FROM") {
        let after_delete = &query[12..];
        return after_delete
            .split_whitespace()
            .next()
            .map(|s| s.trim().to_string());
    }
    
    None
}

/// Truncate a SQL query to a maximum length
fn truncate_query(query: &str, max_length: usize) -> String {
    if query.len() <= max_length {
        query.to_string()
    } else {
        format!("{}...", &query[0..max_length])
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