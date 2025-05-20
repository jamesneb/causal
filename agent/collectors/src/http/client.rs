// agent/collectors/src/http/client.rs

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

/// HTTP client collector settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpClientCollectorSettings {
    /// Base HTTP settings
    #[serde(flatten)]
    pub base: HttpCollectorSettings,
    /// Maximum number of requests to store in memory
    pub max_requests: usize,
    /// Whether to report per-domain metrics
    pub report_per_domain: bool,
    /// Whether to report per-status-code metrics
    pub report_per_status: bool,
    /// Domains to monitor (empty means all)
    pub include_domains: Vec<String>,
    /// Domains to exclude
    pub exclude_domains: Vec<String>,
}

impl Default for HttpClientCollectorSettings {
    fn default() -> Self {
        Self {
            base: HttpCollectorSettings::default(),
            max_requests: 100,
            report_per_domain: true,
            report_per_status: true,
            include_domains: Vec::new(),
            exclude_domains: Vec::new(),
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
    /// Whether the request failed
    pub failed: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// HTTP client collector for monitoring outgoing HTTP requests
pub struct HttpClientCollector {
    /// Collector configuration
    config: CollectorConfig,
    /// HTTP client collector settings
    settings: HttpClientCollectorSettings,
    /// Recent transactions
    transactions: RwLock<VecDeque<HttpTransaction>>,
    /// Active transactions (not yet completed)
    active: RwLock<HashMap<String, HttpRequestMetrics>>,
    /// Request count by domain
    domain_counts: RwLock<HashMap<String, u64>>,
    /// Request count by status code
    status_counts: RwLock<HashMap<u16, u64>>,
    /// Request count by method
    method_counts: RwLock<HashMap<String, u64>>,
    /// Total request count
    total_requests: RwLock<u64>,
    /// Total failed requests
    failed_requests: RwLock<u64>,
}

impl HttpClientCollector {
    /// Create a new HTTP client collector
    pub fn new(config: CollectorConfig) -> Result<Self> {
        let default_settings = HttpClientCollectorSettings::default();
        let settings = merge_http_settings(&default_settings.base, config.settings.as_ref())
            .map(|base| HttpClientCollectorSettings {
                base,
                ..default_settings
            })?;
        
        Ok(Self {
            config,
            settings,
            transactions: RwLock::new(VecDeque::with_capacity(settings.max_requests)),
            active: RwLock::new(HashMap::new()),
            domain_counts: RwLock::new(HashMap::new()),
            status_counts: RwLock::new(HashMap::new()),
            method_counts: RwLock::new(HashMap::new()),
            total_requests: RwLock::new(0),
            failed_requests: RwLock::new(0),
        })
    }
    
    /// Track an HTTP request
    pub async fn track_request(
        &self,
        method: &str,
        url: &str,
        headers: Option<HashMap<String, String>>,
        body_size: usize,
    ) -> String {
        // Generate a unique request ID
        let request_id = Uuid::new_v4().to_string();
        
        // Parse URL to get path
        let path = url
            .split('?')
            .next()
            .unwrap_or(url)
            .split('#')
            .next()
            .unwrap_or(url)
            .to_string();
        
        // Create request metrics
        let request = HttpRequestMetrics {
            request_id: request_id.clone(),
            method: HttpMethod::from(method),
            url: url.to_string(),
            path,
            headers,
            size_bytes: body_size,
            has_body: body_size > 0,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            duration_ms: None,
            remote_addr: None,
            tls_info: None,
        };
        
        // Update counts
        {
            // Increment total requests
            let mut total = self.total_requests.write().await;
            *total += 1;
            
            // Increment method count
            let mut methods = self.method_counts.write().await;
            *methods.entry(method.to_uppercase()).or_default() += 1;
            
            // Increment domain count if enabled
            if self.settings.report_per_domain {
                if let Some(domain) = extract_domain(url) {
                    // Check if this domain should be tracked
                    let include = if self.settings.include_domains.is_empty() {
                        true
                    } else {
                        self.settings.include_domains.iter().any(|d| domain.contains(d))
                    };
                    
                    let exclude = self.settings.exclude_domains.iter().any(|d| domain.contains(d));
                    
                    if include && !exclude {
                        let mut domains = self.domain_counts.write().await;
                        *domains.entry(domain).or_default() += 1;
                    }
                }
            }
        }
        
        // Store active request
        {
            let mut active = self.active.write().await;
            active.insert(request_id.clone(), request);
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
        if let Some(request) = active.remove(request_id) {
            // Calculate timing
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
            
            let total_time_ms = now.saturating_sub(request.timestamp);
            
            // Create response metrics
            let response = HttpResponseMetrics {
                request_id: request_id.to_string(),
                status_code,
                status_text: status_text(status_code),
                headers,
                size_bytes: body_size,
                has_body: body_size > 0,
                timestamp: now,
                ttfb_ms: None, // We don't have this information here
                total_time_ms,
            };
            
            // Update status counts if enabled
            if self.settings.report_per_status {
                let mut statuses = self.status_counts.write().await;
                *statuses.entry(status_code).or_default() += 1;
            }
            
            // Create transaction and store it
            let transaction = HttpTransaction {
                request,
                response: Some(response),
                failed: false,
                error: None,
            };
            
            self.store_transaction(transaction).await;
        }
    }
    
    /// Track a failed HTTP request
    pub async fn track_error(&self, request_id: &str, error: &str) {
        let mut active = self.active.write().await;
        
        // Find the active request
        if let Some(request) = active.remove(request_id) {
            // Increment failed requests count
            let mut failed = self.failed_requests.write().await;
            *failed += 1;
            
            // Create transaction and store it
            let transaction = HttpTransaction {
                request,
                response: None,
                failed: true,
                error: Some(error.to_string()),
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
impl Collector for HttpClientCollector {
    fn metadata(&self) -> CollectorMetadata {
        CollectorMetadata {
            id: self.config.id.clone(),
            name: "HTTP Client Collector".to_string(),
            description: "Collects metrics from outgoing HTTP requests".to_string(),
            source_type: "http.client".to_string(),
            enabled_by_default: true,
        }
    }
    
    fn config(&self) -> CollectorConfig {
        self.config.clone()
    }
    
    fn update_config(&mut self, config: CollectorConfig) -> Result<()> {
        // Update settings from config
        if let Some(settings) = &config.settings {
            let default_settings = HttpClientCollectorSettings::default();
            self.settings = merge_http_settings(&default_settings.base, Some(settings))
                .map(|base| HttpClientCollectorSettings {
                    base,
                    ..default_settings
                })?;
        }
        
        self.config = config;
        Ok(())
    }
    
    async fn initialize(&mut self) -> Result<()> {
        info!("Initializing HTTP client collector");
        Ok(())
    }
    
    async fn collect(&self) -> Result<HashMap<String, MetricValue>> {
        debug!("Collecting HTTP client metrics");
        
        let mut metrics = HashMap::new();
        
        // Add total request count
        let total = *self.total_requests.read().await;
        metrics.insert("http.client.requests.total".to_string(), MetricValue::Counter(total));
        
        // Add failed request count
        let failed = *self.failed_requests.read().await;
        metrics.insert("http.client.requests.failed".to_string(), MetricValue::Counter(failed));
        
        // Add method counts
        let methods = self.method_counts.read().await;
        for (method, count) in methods.iter() {
            metrics.insert(
                format!("http.client.requests.method.{}", method.to_lowercase()),
                MetricValue::Counter(*count),
            );
        }
        
        // Add domain counts if enabled
        if self.settings.report_per_domain {
            let domains = self.domain_counts.read().await;
            for (domain, count) in domains.iter() {
                metrics.insert(
                    format!("http.client.requests.domain.{}", sanitize_metric_name(domain)),
                    MetricValue::Counter(*count),
                );
            }
        }
        
        // Add status code counts if enabled
        if self.settings.report_per_status {
            let statuses = self.status_counts.read().await;
            for (status, count) in statuses.iter() {
                metrics.insert(
                    format!("http.client.requests.status.{}", status),
                    MetricValue::Counter(*count),
                );
            }
        }
        
        // Add active request count
        let active = self.active.read().await.len() as u64;
        metrics.insert("http.client.requests.active".to_string(), MetricValue::Gauge(active as f64));
        
        Ok(metrics)
    }
    
    async fn shutdown(&self) -> Result<()> {
        debug!("Shutting down HTTP client collector");
        Ok(())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Factory for creating HTTP client collectors
pub struct HttpClientCollectorFactory {
    metadata: CollectorMetadata,
}

impl HttpClientCollectorFactory {
    /// Create a new HTTP client collector factory
    pub fn new() -> Self {
        Self {
            metadata: CollectorMetadata {
                id: "http.client".to_string(),
                name: "HTTP Client Collector".to_string(),
                description: "Collects metrics from outgoing HTTP requests".to_string(),
                source_type: "http.client".to_string(),
                enabled_by_default: true,
            },
        }
    }
}

#[async_trait]
impl CollectorFactory for HttpClientCollectorFactory {
    async fn create(&self, config: CollectorConfig) -> Result<Box<dyn Collector>> {
        let collector = HttpClientCollector::new(config)?;
        Ok(Box::new(collector))
    }
    
    fn metadata(&self) -> CollectorMetadata {
        self.metadata.clone()
    }
}

/// Extract domain from URL
fn extract_domain(url: &str) -> Option<String> {
    // Extract domain from URL
    // This is a simple implementation that works for most cases
    if let Some(stripped) = url.strip_prefix("http://") {
        stripped.split('/').next().map(|s| s.to_string())
    } else if let Some(stripped) = url.strip_prefix("https://") {
        stripped.split('/').next().map(|s| s.to_string())
    } else {
        url.split('/').next().map(|s| s.to_string())
    }
}

/// Get status text for HTTP status code
fn status_text(status: u16) -> String {
    match status {
        100 => "Continue".to_string(),
        101 => "Switching Protocols".to_string(),
        102 => "Processing".to_string(),
        200 => "OK".to_string(),
        201 => "Created".to_string(),
        202 => "Accepted".to_string(),
        203 => "Non-Authoritative Information".to_string(),
        204 => "No Content".to_string(),
        205 => "Reset Content".to_string(),
        206 => "Partial Content".to_string(),
        300 => "Multiple Choices".to_string(),
        301 => "Moved Permanently".to_string(),
        302 => "Found".to_string(),
        303 => "See Other".to_string(),
        304 => "Not Modified".to_string(),
        307 => "Temporary Redirect".to_string(),
        308 => "Permanent Redirect".to_string(),
        400 => "Bad Request".to_string(),
        401 => "Unauthorized".to_string(),
        402 => "Payment Required".to_string(),
        403 => "Forbidden".to_string(),
        404 => "Not Found".to_string(),
        405 => "Method Not Allowed".to_string(),
        406 => "Not Acceptable".to_string(),
        407 => "Proxy Authentication Required".to_string(),
        408 => "Request Timeout".to_string(),
        409 => "Conflict".to_string(),
        410 => "Gone".to_string(),
        411 => "Length Required".to_string(),
        412 => "Precondition Failed".to_string(),
        413 => "Payload Too Large".to_string(),
        414 => "URI Too Long".to_string(),
        415 => "Unsupported Media Type".to_string(),
        416 => "Range Not Satisfiable".to_string(),
        417 => "Expectation Failed".to_string(),
        418 => "I'm a teapot".to_string(),
        422 => "Unprocessable Entity".to_string(),
        429 => "Too Many Requests".to_string(),
        500 => "Internal Server Error".to_string(),
        501 => "Not Implemented".to_string(),
        502 => "Bad Gateway".to_string(),
        503 => "Service Unavailable".to_string(),
        504 => "Gateway Timeout".to_string(),
        505 => "HTTP Version Not Supported".to_string(),
        _ => format!("Status {}", status),
    }
}

/// Sanitize a string for use in a metric name
fn sanitize_metric_name(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' => c,
            _ => '_',
        })
        .collect()
}