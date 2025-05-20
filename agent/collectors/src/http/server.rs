// agent/collectors/src/http/server.rs

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
use super::{HttpCollectorSettings, merge_http_settings, schema::{HttpMethod, HttpRequestMetrics, HttpResponseMetrics}};

/// HTTP server collector settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpServerCollectorSettings {
    /// Base HTTP settings
    #[serde(flatten)]
    pub base: HttpCollectorSettings,
    /// Maximum number of requests to store in memory
    pub max_requests: usize,
    /// Whether to report per-path metrics
    pub report_per_path: bool,
    /// Whether to report per-status-code metrics
    pub report_per_status: bool,
    /// Paths to monitor (empty means all)
    pub include_paths: Vec<String>,
    /// Paths to exclude
    pub exclude_paths: Vec<String>,
}

impl Default for HttpServerCollectorSettings {
    fn default() -> Self {
        Self {
            base: HttpCollectorSettings::default(),
            max_requests: 100,
            report_per_path: true,
            report_per_status: true,
            include_paths: Vec::new(),
            exclude_paths: Vec::new(),
        }
    }
}

/// HTTP request/response pair
#[derive(Debug, Clone, Serialize, Deserialize)]
struct HttpTransaction {
    /// Request metrics
    pub request: HttpRequestMetrics,
    /// Response metrics
    pub response: Option<HttpResponseMetrics>,
    /// Duration
    pub duration_ms: u64,
}

/// HTTP server collector for monitoring incoming HTTP requests
pub struct HttpServerCollector {
    /// Collector configuration
    config: CollectorConfig,
    /// HTTP server collector settings
    settings: HttpServerCollectorSettings,
    /// Recent transactions
    transactions: RwLock<VecDeque<HttpTransaction>>,
    /// Active transactions (not yet completed)
    active: RwLock<HashMap<String, (HttpRequestMetrics, Instant)>>,
    /// Request count by path
    path_counts: RwLock<HashMap<String, u64>>,
    /// Request count by status code
    status_counts: RwLock<HashMap<u16, u64>>,
    /// Request count by method
    method_counts: RwLock<HashMap<String, u64>>,
    /// Total request count
    total_requests: RwLock<u64>,
    /// Total responses sent
    total_responses: RwLock<u64>,
}

impl HttpServerCollector {
    /// Create a new HTTP server collector
    pub fn new(config: CollectorConfig) -> Result<Self> {
        let default_settings = HttpServerCollectorSettings::default();
        let settings = merge_http_settings(&default_settings.base, config.settings.as_ref())
            .map(|base| HttpServerCollectorSettings {
                base,
                ..default_settings
            })?;
        
        Ok(Self {
            config,
            settings,
            transactions: RwLock::new(VecDeque::with_capacity(settings.max_requests)),
            active: RwLock::new(HashMap::new()),
            path_counts: RwLock::new(HashMap::new()),
            status_counts: RwLock::new(HashMap::new()),
            method_counts: RwLock::new(HashMap::new()),
            total_requests: RwLock::new(0),
            total_responses: RwLock::new(0),
        })
    }
    
    /// Track an incoming HTTP request
    pub async fn track_request(
        &self,
        method: &str,
        path: &str,
        headers: Option<HashMap<String, String>>,
        remote_addr: Option<String>,
        body_size: usize,
    ) -> String {
        // Generate a unique request ID
        let request_id = Uuid::new_v4().to_string();
        
        // Create request metrics
        let request = HttpRequestMetrics {
            request_id: request_id.clone(),
            method: HttpMethod::from(method),
            url: path.to_string(), // For server, URL is the same as path
            path: path.to_string(),
            headers,
            size_bytes: body_size,
            has_body: body_size > 0,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            duration_ms: None,
            remote_addr,
            tls_info: None, // We don't have this information here
        };
        
        // Update counts
        {
            // Increment total requests
            let mut total = self.total_requests.write().await;
            *total += 1;
            
            // Increment method count
            let mut methods = self.method_counts.write().await;
            *methods.entry(method.to_uppercase()).or_default() += 1;
            
            // Increment path count if enabled
            if self.settings.report_per_path {
                // Check if this path should be tracked
                let include = if self.settings.include_paths.is_empty() {
                    true
                } else {
                    self.settings.include_paths.iter().any(|p| path.starts_with(p))
                };
                
                let exclude = self.settings.exclude_paths.iter().any(|p| path.starts_with(p));
                
                if include && !exclude {
                    let mut paths = self.path_counts.write().await;
                    *paths.entry(normalize_path(path)).or_default() += 1;
                }
            }
        }
        
        // Store active request
        {
            let mut active = self.active.write().await;
            active.insert(request_id.clone(), (request, Instant::now()));
        }
        
        request_id
    }
    
    /// Track an HTTP response
    pub async fn track_response(
        &self,
        request_id: &str,
        status_code: u16,
        headers: Option<HashMap<String, String>>,
        body_size: usize,
    ) {
        let mut active = self.active.write().await;
        
        // Find the active request
        if let Some((request, start_time)) = active.remove(request_id) {
            // Calculate timing
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            
            let duration_ms = start_time.elapsed().as_millis() as u64;
            
            // Create response metrics
            let response = HttpResponseMetrics {
                request_id: request_id.to_string(),
                status_code,
                status_text: super::client::status_text(status_code),
                headers,
                size_bytes: body_size,
                has_body: body_size > 0,
                timestamp: now,
                ttfb_ms: None, // We don't have this information here
                total_time_ms: duration_ms,
            };
            
            // Update status counts if enabled
            if self.settings.report_per_status {
                let mut statuses = self.status_counts.write().await;
                *statuses.entry(status_code).or_default() += 1;
            }
            
            // Increment total responses
            {
                let mut total = self.total_responses.write().await;
                *total += 1;
            }
            
            // Create transaction and store it
            let transaction = HttpTransaction {
                request,
                response: Some(response),
                duration_ms,
            };
            
            self.store_transaction(transaction).await;
        }
    }
    
    /// Store a completed transaction
    async fn store_transaction(&self, transaction: HttpTransaction) {
        let mut transactions = self.transactions.write().await;
        
        // Remove oldest if at capacity
        if transactions.len() >= self.settings.max_requests {
            transactions.pop_front();
        }
        
        transactions.push_back(transaction);
    }
}

#[async_trait]
impl Collector for HttpServerCollector {
    fn metadata(&self) -> CollectorMetadata {
        CollectorMetadata {
            id: self.config.id.clone(),
            name: "HTTP Server Collector".to_string(),
            description: "Collects metrics from incoming HTTP requests".to_string(),
            source_type: "http.server".to_string(),
            enabled_by_default: true,
        }
    }
    
    fn config(&self) -> CollectorConfig {
        self.config.clone()
    }
    
    fn update_config(&mut self, config: CollectorConfig) -> Result<()> {
        // Update settings from config
        if let Some(settings) = &config.settings {
            let default_settings = HttpServerCollectorSettings::default();
            self.settings = merge_http_settings(&default_settings.base, Some(settings))
                .map(|base| HttpServerCollectorSettings {
                    base,
                    ..default_settings
                })?;
        }
        
        self.config = config;
        Ok(())
    }
    
    async fn initialize(&mut self) -> Result<()> {
        info!("Initializing HTTP server collector");
        Ok(())
    }
    
    async fn collect(&self) -> Result<HashMap<String, MetricValue>> {
        debug!("Collecting HTTP server metrics");
        
        let mut metrics = HashMap::new();
        
        // Add total request/response counts
        let total_req = *self.total_requests.read().await;
        metrics.insert("http.server.requests.total".to_string(), MetricValue::Counter(total_req));
        
        let total_resp = *self.total_responses.read().await;
        metrics.insert("http.server.responses.total".to_string(), MetricValue::Counter(total_resp));
        
        // Add method counts
        let methods = self.method_counts.read().await;
        for (method, count) in methods.iter() {
            metrics.insert(
                format!("http.server.requests.method.{}", method.to_lowercase()),
                MetricValue::Counter(*count),
            );
        }
        
        // Add path counts if enabled
        if self.settings.report_per_path {
            let paths = self.path_counts.read().await;
            for (path, count) in paths.iter() {
                metrics.insert(
                    format!("http.server.requests.path.{}", sanitize_metric_name(path)),
                    MetricValue::Counter(*count),
                );
            }
        }
        
        // Add status code counts if enabled
        if self.settings.report_per_status {
            let statuses = self.status_counts.read().await;
            for (status, count) in statuses.iter() {
                metrics.insert(
                    format!("http.server.responses.status.{}", status),
                    MetricValue::Counter(*count),
                );
            }
        }
        
        // Add active request count
        let active = self.active.read().await.len() as u64;
        metrics.insert("http.server.requests.active".to_string(), MetricValue::Gauge(active as f64));
        
        // Add latency metrics from transactions
        let transactions = self.transactions.read().await;
        if !transactions.is_empty() {
            // Collect all latencies
            let latencies: Vec<f64> = transactions
                .iter()
                .map(|t| t.duration_ms as f64)
                .collect();
            
            metrics.insert(
                "http.server.latency.ms".to_string(),
                MetricValue::Histogram(latencies),
            );
        }
        
        Ok(metrics)
    }
    
    async fn shutdown(&self) -> Result<()> {
        debug!("Shutting down HTTP server collector");
        Ok(())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Factory for creating HTTP server collectors
pub struct HttpServerCollectorFactory {
    metadata: CollectorMetadata,
}

impl HttpServerCollectorFactory {
    /// Create a new HTTP server collector factory
    pub fn new() -> Self {
        Self {
            metadata: CollectorMetadata {
                id: "http.server".to_string(),
                name: "HTTP Server Collector".to_string(),
                description: "Collects metrics from incoming HTTP requests".to_string(),
                source_type: "http.server".to_string(),
                enabled_by_default: true,
            },
        }
    }
}

#[async_trait]
impl CollectorFactory for HttpServerCollectorFactory {
    async fn create(&self, config: CollectorConfig) -> Result<Box<dyn Collector>> {
        let collector = HttpServerCollector::new(config)?;
        Ok(Box::new(collector))
    }
    
    fn metadata(&self) -> CollectorMetadata {
        self.metadata.clone()
    }
}

/// Normalize a path for metrics
fn normalize_path(path: &str) -> String {
    // This is a simple implementation that works for common cases
    // For a real system, you might want more sophisticated normalization
    
    // Split path into segments
    let segments: Vec<&str> = path.split('/').collect();
    
    // If there are no segments, return "/"
    if segments.is_empty() {
        return "/".to_string();
    }
    
    // Rebuild the path with normalized segments
    let mut normalized = String::new();
    for segment in segments {
        if segment.is_empty() {
            continue;
        }
        
        // If segment looks like an ID, replace with :id
        if segment.chars().all(|c| c.is_ascii_digit()) {
            normalized.push_str("/:id");
        } else if segment.len() >= 32 && segment.chars().all(|c| c.is_ascii_hexdigit() || c == '-') {
            // Looks like a UUID or hash
            normalized.push_str("/:uuid");
        } else {
            normalized.push('/');
            normalized.push_str(segment);
        }
    }
    
    // If normalized is empty, return "/"
    if normalized.is_empty() {
        return "/".to_string();
    }
    
    normalized
}

/// Sanitize a string for use in a metric name
fn sanitize_metric_name(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' => c,
            '/' => '.',
            _ => '_',
        })
        .collect()
}