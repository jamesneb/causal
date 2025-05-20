// agent/telemetry/src/transport.rs

use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use async_trait::async_trait;
use serde::Serialize;
use tokio::sync::{RwLock, mpsc};
use tracing::{debug, info, warn, error};

use crate::{TelemetryBatch, TelemetryBuffer};

/// Trait for telemetry transporters
#[async_trait]
pub trait TelemetryTransport: Send + Sync {
    /// Send a batch of telemetry events
    async fn send_batch(&self, batch: TelemetryBatch) -> Result<()>;
    
    /// Get the transport name
    fn name(&self) -> &str;
    
    /// Get the transport type
    fn transport_type(&self) -> &str;
    
    /// Clone the transport as a boxed trait object
    fn clone_box(&self) -> Box<dyn TelemetryTransport>;
}

impl Clone for Box<dyn TelemetryTransport> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// HTTP transport for sending telemetry via HTTP
pub struct HttpTransport {
    /// Transport name
    name: String,
    /// Endpoint URL
    url: String,
    /// HTTP method
    method: String,
    /// HTTP headers
    headers: RwLock<Vec<(String, String)>>,
    /// Retry configuration
    retry_config: RetryConfig,
    /// Authentication type
    auth: Option<AuthConfig>,
    /// Enable compression
    compression: bool,
}

/// Authentication configuration
#[derive(Clone)]
pub enum AuthConfig {
    /// Basic authentication
    Basic {
        username: String,
        password: String,
    },
    /// Bearer token
    Bearer {
        token: String,
    },
    /// API key
    ApiKey {
        key: String,
        header_name: String,
    },
}

/// Retry configuration
#[derive(Clone, Copy)]
pub struct RetryConfig {
    /// Maximum number of retries
    pub max_retries: usize,
    /// Initial retry delay
    pub initial_delay: Duration,
    /// Maximum retry delay
    pub max_delay: Duration,
    /// Retry backoff factor
    pub backoff_factor: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            backoff_factor: 2.0,
        }
    }
}

impl HttpTransport {
    /// Create a new HTTP transport
    pub fn new(name: &str, url: &str) -> Self {
        Self {
            name: name.to_string(),
            url: url.to_string(),
            method: "POST".to_string(),
            headers: RwLock::new(vec![
                ("Content-Type".to_string(), "application/json".to_string()),
            ]),
            retry_config: RetryConfig::default(),
            auth: None,
            compression: false,
        }
    }
    
    /// Set HTTP method
    pub fn with_method(mut self, method: &str) -> Self {
        self.method = method.to_uppercase();
        self
    }
    
    /// Add an HTTP header
    pub async fn add_header(&self, name: &str, value: &str) {
        let mut headers = self.headers.write().await;
        headers.push((name.to_string(), value.to_string()));
    }
    
    /// Set authentication
    pub fn with_auth(mut self, auth: AuthConfig) -> Self {
        self.auth = Some(auth);
        self
    }
    
    /// Set retry configuration
    pub fn with_retry_config(mut self, retry_config: RetryConfig) -> Self {
        self.retry_config = retry_config;
        self
    }
    
    /// Enable or disable compression
    pub fn with_compression(mut self, enabled: bool) -> Self {
        self.compression = enabled;
        self
    }
    
    /// Send a batch via HTTP
    async fn send_http_request(&self, batch: &TelemetryBatch) -> Result<()> {
        // For now, just log that we would send the data
        // In a real implementation, this would use reqwest or another HTTP client
        info!(
            "Would send batch {} with {} events to {}",
            batch.id,
            batch.events.len(),
            self.url
        );
        
        // Simulate some network latency
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        Ok(())
    }
}

#[async_trait]
impl TelemetryTransport for HttpTransport {
    async fn send_batch(&self, batch: TelemetryBatch) -> Result<()> {
        let mut retry_count = 0;
        let mut delay = self.retry_config.initial_delay;
        
        loop {
            match self.send_http_request(&batch).await {
                Ok(_) => return Ok(()),
                Err(e) => {
                    retry_count += 1;
                    if retry_count >= self.retry_config.max_retries {
                        return Err(e);
                    }
                    
                    warn!(
                        "Failed to send batch via HTTP, retrying {}/{}",
                        retry_count, self.retry_config.max_retries
                    );
                    
                    tokio::time::sleep(delay).await;
                    
                    // Calculate next delay with exponential backoff
                    let next_delay = delay.as_millis() as f64 * self.retry_config.backoff_factor;
                    delay = Duration::from_millis(
                        next_delay as u64
                            .min(self.retry_config.max_delay.as_millis() as u64),
                    );
                }
            }
        }
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn transport_type(&self) -> &str {
        "http"
    }
    
    fn clone_box(&self) -> Box<dyn TelemetryTransport> {
        Box::new(Self {
            name: self.name.clone(),
            url: self.url.clone(),
            method: self.method.clone(),
            headers: RwLock::new(
                self.headers
                    .try_read()
                    .unwrap_or_else(|_| panic!("Failed to read headers"))
                    .clone(),
            ),
            retry_config: self.retry_config,
            auth: self.auth.clone(),
            compression: self.compression,
        })
    }
}

/// File transport for writing telemetry to files
pub struct FileTransport {
    /// Transport name
    name: String,
    /// Output directory
    directory: String,
    /// File prefix
    prefix: String,
    /// File extension
    extension: String,
    /// Maximum file size in bytes
    max_file_size: usize,
    /// Time-based rotation interval
    rotation_interval: Option<Duration>,
    /// Maximum number of files to keep
    max_files: Option<usize>,
    /// Compression enabled
    compression: bool,
}

impl FileTransport {
    /// Create a new file transport
    pub fn new(name: &str, directory: &str) -> Self {
        Self {
            name: name.to_string(),
            directory: directory.to_string(),
            prefix: "telemetry".to_string(),
            extension: "json".to_string(),
            max_file_size: 10 * 1024 * 1024, // 10 MB
            rotation_interval: Some(Duration::from_secs(3600)), // 1 hour
            max_files: Some(24), // 24 files
            compression: false,
        }
    }
    
    /// Set file prefix
    pub fn with_prefix(mut self, prefix: &str) -> Self {
        self.prefix = prefix.to_string();
        self
    }
    
    /// Set file extension
    pub fn with_extension(mut self, extension: &str) -> Self {
        self.extension = extension.to_string();
        self
    }
    
    /// Set maximum file size
    pub fn with_max_file_size(mut self, max_file_size: usize) -> Self {
        self.max_file_size = max_file_size;
        self
    }
    
    /// Set rotation interval
    pub fn with_rotation_interval(mut self, interval: Duration) -> Self {
        self.rotation_interval = Some(interval);
        self
    }
    
    /// Set maximum number of files to keep
    pub fn with_max_files(mut self, max_files: usize) -> Self {
        self.max_files = Some(max_files);
        self
    }
    
    /// Enable or disable compression
    pub fn with_compression(mut self, enabled: bool) -> Self {
        self.compression = enabled;
        self
    }
    
    /// Write a batch to a file
    async fn write_to_file(&self, batch: &TelemetryBatch) -> Result<()> {
        // For now, just log that we would write the data
        // In a real implementation, this would write to a file
        info!(
            "Would write batch {} with {} events to {}/{}-{}.{}",
            batch.id,
            batch.events.len(),
            self.directory,
            self.prefix,
            chrono::Utc::now().format("%Y%m%d-%H%M%S"),
            self.extension
        );
        
        // Simulate some disk I/O
        tokio::time::sleep(Duration::from_millis(20)).await;
        
        Ok(())
    }
}

#[async_trait]
impl TelemetryTransport for FileTransport {
    async fn send_batch(&self, batch: TelemetryBatch) -> Result<()> {
        self.write_to_file(&batch).await
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn transport_type(&self) -> &str {
        "file"
    }
    
    fn clone_box(&self) -> Box<dyn TelemetryTransport> {
        Box::new(Self {
            name: self.name.clone(),
            directory: self.directory.clone(),
            prefix: self.prefix.clone(),
            extension: self.extension.clone(),
            max_file_size: self.max_file_size,
            rotation_interval: self.rotation_interval,
            max_files: self.max_files,
            compression: self.compression,
        })
    }
}