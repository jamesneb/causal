// agent/collectors/src/database/mod.rs

//! Database collectors for monitoring database operations

pub mod sql;
pub mod nosql;

use std::collections::HashMap;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Common settings for database collectors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseCollectorSettings {
    /// Whether to collect query text
    pub collect_queries: bool,
    /// Maximum query length to collect
    pub max_query_length: usize,
    /// Whether to collect parameters
    pub collect_parameters: bool,
    /// Whether to collect query execution plans
    pub collect_execution_plans: bool,
    /// Whether to collect table metrics
    pub collect_table_metrics: bool,
    /// Maximum number of queries to store in memory
    pub max_queries: usize,
}

impl Default for DatabaseCollectorSettings {
    fn default() -> Self {
        Self {
            collect_queries: true,
            max_query_length: 1000,
            collect_parameters: false,
            collect_execution_plans: false,
            collect_table_metrics: true,
            max_queries: 100,
        }
    }
}

/// Helper function to merge collector settings from config
pub fn merge_db_settings(
    defaults: &DatabaseCollectorSettings,
    config_settings: Option<&HashMap<String, String>>,
) -> Result<DatabaseCollectorSettings> {
    let mut settings = defaults.clone();
    
    if let Some(cfg) = config_settings {
        // Process collect_queries
        if let Some(collect_queries) = cfg.get("collect_queries") {
            settings.collect_queries = collect_queries == "true" || collect_queries == "1";
        }
        
        // Process max_query_length
        if let Some(max_query_length) = cfg.get("max_query_length") {
            if let Ok(length) = max_query_length.parse::<usize>() {
                settings.max_query_length = length;
            }
        }
        
        // Process collect_parameters
        if let Some(collect_parameters) = cfg.get("collect_parameters") {
            settings.collect_parameters = collect_parameters == "true" || collect_parameters == "1";
        }
        
        // Process collect_execution_plans
        if let Some(collect_execution_plans) = cfg.get("collect_execution_plans") {
            settings.collect_execution_plans = collect_execution_plans == "true" || collect_execution_plans == "1";
        }
        
        // Process collect_table_metrics
        if let Some(collect_table_metrics) = cfg.get("collect_table_metrics") {
            settings.collect_table_metrics = collect_table_metrics == "true" || collect_table_metrics == "1";
        }
        
        // Process max_queries
        if let Some(max_queries) = cfg.get("max_queries") {
            if let Ok(max) = max_queries.parse::<usize>() {
                settings.max_queries = max;
            }
        }
    }
    
    Ok(settings)
}

/// Common schema for database metrics
pub mod schema {
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    use std::time::Duration;
    
    /// Database operation type
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum DatabaseOperationType {
        Query,
        Execute,
        Transaction,
        Prepare,
        Connect,
        Disconnect,
        Other(String),
    }
    
    /// Query type
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum QueryType {
        Select,
        Insert,
        Update,
        Delete,
        Create,
        Alter,
        Drop,
        Begin,
        Commit,
        Rollback,
        Other(String),
    }
    
    impl QueryType {
        pub fn from_query(query: &str) -> Self {
            let query = query.trim().to_uppercase();
            if query.starts_with("SELECT") {
                Self::Select
            } else if query.starts_with("INSERT") {
                Self::Insert
            } else if query.starts_with("UPDATE") {
                Self::Update
            } else if query.starts_with("DELETE") {
                Self::Delete
            } else if query.starts_with("CREATE") {
                Self::Create
            } else if query.starts_with("ALTER") {
                Self::Alter
            } else if query.starts_with("DROP") {
                Self::Drop
            } else if query.starts_with("BEGIN") {
                Self::Begin
            } else if query.starts_with("COMMIT") {
                Self::Commit
            } else if query.starts_with("ROLLBACK") {
                Self::Rollback
            } else {
                Self::Other(query.split_whitespace().next().unwrap_or("UNKNOWN").to_string())
            }
        }
    }
    
    /// Database query metrics
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DatabaseQueryMetrics {
        /// Query ID
        pub query_id: String,
        /// Connection ID
        pub connection_id: Option<String>,
        /// Query type
        pub query_type: QueryType,
        /// Query text
        pub query_text: Option<String>,
        /// Parameters
        pub parameters: Option<String>,
        /// Query timestamp
        pub timestamp: u64,
        /// Query duration in milliseconds
        pub duration_ms: u64,
        /// Number of rows affected
        pub rows_affected: Option<u64>,
        /// Number of rows returned
        pub rows_returned: Option<u64>,
        /// Execution plan
        pub execution_plan: Option<String>,
        /// Error message if any
        pub error: Option<String>,
        /// Database-specific metrics
        pub metadata: Option<HashMap<String, String>>,
    }
    
    /// Connection metrics
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ConnectionMetrics {
        /// Connection ID
        pub connection_id: String,
        /// Connection string (sanitized)
        pub connection_string: Option<String>,
        /// Database name
        pub database: Option<String>,
        /// User name
        pub user: Option<String>,
        /// Connected timestamp
        pub connected_at: u64,
        /// Disconnected timestamp (if applicable)
        pub disconnected_at: Option<u64>,
        /// Connection duration in milliseconds
        pub duration_ms: Option<u64>,
        /// Number of queries executed
        pub query_count: u64,
        /// Error message if any
        pub error: Option<String>,
    }
    
    /// Table metrics
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TableMetrics {
        /// Table name
        pub table_name: String,
        /// Row count estimate
        pub row_count: Option<u64>,
        /// Size in bytes
        pub size_bytes: Option<u64>,
        /// Number of reads
        pub reads: u64,
        /// Number of writes
        pub writes: u64,
        /// Last accessed timestamp
        pub last_accessed: u64,
        /// Table-specific metrics
        pub metadata: Option<HashMap<String, String>>,
    }
}