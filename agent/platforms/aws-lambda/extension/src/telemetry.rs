// agent/platforms/aws-lambda/extension/src/telemetry.rs

use anyhow::{Context, Result};
use chrono::Utc;
use lambda_extension::{Extension, LambdaEvent, LambdaExtension, NextEvent};
use once_cell::sync::Lazy;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

use crate::state::{is_cold_start, LambdaStateManager};
use agent_core::telemetry::protocol::binary::{MetricEntry, serialize_metrics_binary};
use agent_core::telemetry::protocol::registry::{FieldRegistry, FIELD_REGISTRY};
use agent_core::transport::buffer::TelemetryBuffer;
use agent_core::transport::retry::{ship_with_retry, ship_http_with_retry};
use agent_common::utils::compression::{compress_data, DEFAULT_COMPRESSION_LEVEL};
use agent_common::utils::error_correction::PROTOCOL_ERROR_CORRECTION_ENABLED;
use heapless::pool::singleton::{Pool, PoolPtr};

// Fixed-size memory pool for metrics
static METRICS_POOL: Pool<MetricEntry, 128> = Pool::uninit();

// Lambda-specific telemetry events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LambdaTelemetryEvent {
    Invoke {
        request_id: String,
        runtime: String,
        timestamp: u64,
        timeout: u32,
    },
    Response {
        request_id: String,
        status_code: u16,
        duration_ms: u64,
        memory_used_mb: u64,
    },
    Error {
        request_id: String,
        error_type: String,
        error_message: String,
        stack_trace: Option<String>,
    },
    Extension {
        event_type: String,
        duration_ms: u64,
        memory_used_mb: u64,
    },
    MetricsSnapshot {
        cpu_usage_percent: f64,
        memory_usage_mb: u64,
        network_rx_bytes: u64,
        network_tx_bytes: u64,
    },
}

// Trait for metrics shipping
#[async_trait::async_trait]
pub trait MetricsShipper: Send + Sync {
    async fn ship_metrics(&self, request_id: String, metrics: Value) -> Result<()>;
    async fn flush(&self) -> Result<()>;
}

// High-performance binary protocol implementation for Lambda
pub struct BinaryProtocolShipper {
    endpoint: String,
    buffer: Arc<TelemetryBuffer>,
    client: Client,
    compression_level: flate2::Compression,
    use_error_correction: bool,
    schema_endpoint: Option<String>,
    last_schema_update: std::sync::atomic::AtomicI64,
    schema_update_interval_secs: i64,
}

impl BinaryProtocolShipper {
    pub fn new(
        endpoint: String, 
        batch_size: usize, 
        client: Client,
        schema_endpoint: Option<String>,
    ) -> Self {
        // Initialize the pool (only needed once)
        unsafe { METRICS_POOL.init() };
        
        // Create buffer
        let buffer = Arc::new(TelemetryBuffer::new(
            batch_size,
            "/tmp/lambda-extension-metrics"
        ));
        
        Self {
            endpoint,
            buffer,
            client,
            compression_level: DEFAULT_COMPRESSION_LEVEL,
            use_error_correction: PROTOCOL_ERROR_CORRECTION_ENABLED,
            schema_endpoint,
            last_schema_update: std::sync::atomic::AtomicI64::new(0),
            schema_update_interval_secs: 3600, // Update schema once per hour
        }
    }
    
    // Upload field registry to schema endpoint
    async fn update_schema(&self) -> Result<()> {
        // Check if schema endpoint is configured
        let Some(schema_endpoint) = &self.schema_endpoint else {
            return Ok(());
        };
        
        // Check if update is needed
        let now = chrono::Utc::now().timestamp();
        let last_update = self.last_schema_update.load(std::sync::atomic::Ordering::Relaxed);
        if now - last_update < self.schema_update_interval_secs {
            return Ok(());
        }
        
        // Serialize field registry
        let registry = FIELD_REGISTRY.read().unwrap();
        let registry_data = registry.serialize();
        drop(registry);
        
        // Send to schema endpoint
        let response = self.client.post(schema_endpoint)
            .header("Content-Type", "application/octet-stream")
            .header("X-Probe-Schema-Version", agent_core::telemetry::protocol::registry::PROTOCOL_FIELD_REGISTRY_VERSION.to_string())
            .body(registry_data)
            .send()
            .await
            .context("Failed to update schema")?;
            
        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Schema update failed with status: {}", 
                response.status()
            ));
        }
        
        // Update last update time
        self.last_schema_update.store(now, std::sync::atomic::Ordering::Relaxed);
        
        Ok(())
    }
    
    // Check and send any backed up metrics
    async fn send_backed_up_metrics(&self) -> Result<()> {
        // Check if backup file exists and has content
        let temp_file_path = self.buffer.get_temp_file_path();
        if !temp_file_path.exists() {
            return Ok(());
        }
        
        let metadata = match std::fs::metadata(temp_file_path) {
            Ok(md) => md,
            Err(_) => return Ok(()),
        };
        
        if metadata.len() == 0 {
            return Ok(());
        }
        
        // Read file
        let mut file = match std::fs::File::open(temp_file_path) {
            Ok(f) => f,
            Err(_) => return Ok(()),
        };
        
        let mut reader = std::io::BufReader::new(file);
        
        // Read batches and send them
        let mut position = 0;
        let file_size = metadata.len() as usize;
        
        while position < file_size {
            // Read length prefix
            let mut length_bytes = [0u8; 4];
            if reader.read_exact(&mut length_bytes).is_err() {
                break;
            }
            position += 4;
            
            let batch_len = u32::from_le_bytes(length_bytes) as usize;
            if batch_len == 0 || position + batch_len > file_size {
                break;
            }
            
            // Read batch data
            let mut batch = vec![0u8; batch_len];
            if reader.read_exact(&mut batch).is_err() {
                break;
            }
            position += batch_len;
            
            // Send batch
            match self.client.post(&self.endpoint)
                .header("Content-Type", "application/octet-stream")
                .header("X-Probe-Protocol-Version", agent_core::telemetry::protocol::binary::PROTOCOL_VERSION.to_string())
                .body(batch)
                .send()
                .await
            {
                Ok(_) => {
                    // Successfully sent
                },
                Err(e) => {
                    tracing::warn!("Failed to send backed up metrics: {}", e);
                    return Ok(());
                }
            }
        }
        
        // If we get here, all backed up metrics were sent or processed
        // Delete the backup file
        let _ = std::fs::remove_file(temp_file_path);
        
        Ok(())
    }
}

#[async_trait::async_trait]
impl MetricsShipper for BinaryProtocolShipper {
    async fn ship_metrics(&self, request_id: String, metrics: Value) -> Result<()> {
        // Get entry from pool with zero allocation
        let entry = METRICS_POOL.alloc().ok_or_else(|| 
            anyhow::anyhow!("Metrics pool exhausted")
        )?;
        
        // Set values (reusing existing storage)
        entry.request_id.clear();
        entry.request_id.push_str(&request_id);
        entry.metrics = metrics;
        entry.timestamp = Utc::now();
        
        // Add to buffer and check if should flush
        let should_flush = self.buffer.add_metric(entry).await;
        
        // Flush outside the lock if needed
        if should_flush {
            self.flush().await?;
        }
        
        Ok(())
    }
    
    async fn flush(&self) -> Result<()> {
        // First, maybe update schema
        if let Err(e) = self.update_schema().await {
            tracing::warn!("Failed to update schema: {}", e);
            // Continue anyway
        }
        
        // Try to send any backed up metrics first (on cold start or after recovery)
        if is_cold_start() {
            if let Err(e) = self.send_backed_up_metrics().await {
                tracing::warn!("Failed to send backed up metrics: {}", e);
            }
        }
        
        // Flush buffer
        let entries = self.buffer.flush().await;
        
        if entries.is_empty() {
            return Ok(());
        }
        
        // Only log if we're actually sending something
        let metrics_count = entries.len();
        tracing::info!("Flushing {} metrics", metrics_count);
        
        // Get registry for serialization
        let registry = FIELD_REGISTRY.read().unwrap();
        
        // Serialize metrics in binary format
        let binary_data = serialize_metrics_binary(
            &entries, 
            &registry,
            self.compression_level,
            self.use_error_correction
        );
        
        drop(registry);
        
        // Send with retry logic for resilience
        let client = self.client.clone();
        let endpoint = self.endpoint.clone();
        let mut headers = std::collections::HashMap::new();
        headers.insert(
            "X-Probe-Protocol-Version".to_string(), 
            agent_core::telemetry::protocol::binary::PROTOCOL_VERSION.to_string()
        );
        
        let send_result = ship_http_with_retry(
            &client,
            &endpoint,
            binary_data.clone(),
            "application/octet-stream",
            3, // retry attempts
            500, // base retry delay in ms
            Some(headers)
        ).await;
        
        // Return pointers to the pool regardless of send success
        for entry in entries {
            unsafe { METRICS_POOL.free(entry) };
        }
        
        // Handle send result
        match send_result {
            Ok(_) => {
                // Successfully sent
            },
            Err(e) => {
                tracing::warn!("Failed to send metrics after retries: {}", e);
                
                // Backup metrics to disk if send failed
                if let Err(backup_err) = self.buffer.backup_buffer(&binary_data) {
                    tracing::error!("Failed to backup metrics: {}", backup_err);
                } else {
                    tracing::info!("Backed up metrics to disk for later retry");
                }
                
                return Err(e);
            }
        }
        
        Ok(())
    }
}

// Lambda Extension integration for telemetry
pub struct LambdaTelemetryExtension {
    shipper: Arc<dyn MetricsShipper>,
    state_manager: Arc<Mutex<LambdaStateManager>>,
    extension_id: String,
    flush_interval: Duration,
    auto_flush_running: std::sync::atomic::AtomicBool,
}

impl LambdaTelemetryExtension {
    pub fn new(
        extension_id: String,
        endpoint: String,
        batch_size: usize,
        state_manager: Arc<Mutex<LambdaStateManager>>,
        flush_interval: Duration,
    ) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap();
            
        let shipper: Arc<dyn MetricsShipper> = Arc::new(
            BinaryProtocolShipper::new(
                endpoint,
                batch_size,
                client,
                None, // No schema endpoint for now
            )
        );
        
        Self {
            shipper,
            state_manager,
            extension_id,
            flush_interval,
            auto_flush_running: std::sync::atomic::AtomicBool::new(false),
        }
    }
    
    // Start auto-flush background task
    pub fn start_auto_flush(&self) {
        let was_running = self.auto_flush_running.swap(true, std::sync::atomic::Ordering::SeqCst);
        if was_running {
            return; // Already running
        }
        
        let shipper = self.shipper.clone();
        let interval = self.flush_interval;
        let auto_flush_running = &self.auto_flush_running;
        
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            
            while auto_flush_running.load(std::sync::atomic::Ordering::SeqCst) {
                interval_timer.tick().await;
                
                if let Err(e) = shipper.flush().await {
                    tracing::warn!("Auto-flush failed: {}", e);
                }
            }
        });
    }
    
    // Stop auto-flush background task
    pub fn stop_auto_flush(&self) {
        self.auto_flush_running.store(false, std::sync::atomic::Ordering::SeqCst);
    }
    
    // Process Lambda extension events
    pub async fn process_event(&self, event: LambdaEvent) -> Result<()> {
        match event.next {
            NextEvent::Shutdown(shutdown) => {
                // Flush all pending metrics before shutdown
                info!("Extension shutting down: {}", shutdown.shutdown_reason);
                
                if let Err(e) = self.shipper.flush().await {
                    error!("Failed to flush metrics on shutdown: {}", e);
                }
                
                self.stop_auto_flush();
            }
            NextEvent::Invoke(invoke) => {
                let request_id = invoke.request_id.clone();
                debug!("Invoke event: {}", request_id);
                
                // Capture invoke telemetry
                let invoke_event = LambdaTelemetryEvent::Invoke {
                    request_id: request_id.clone(),
                    runtime: "provided.al2".to_string(), // This could be obtained from env
                    timestamp: chrono::Utc::now().timestamp_millis() as u64,
                    timeout: std::env::var("AWS_LAMBDA_FUNCTION_TIMEOUT")
                        .unwrap_or_else(|_| "3".to_string())
                        .parse::<u32>()
                        .unwrap_or(3),
                };
                
                // Convert to JSON
                let metrics = serde_json::to_value(invoke_event)?;
                
                // Ship metrics
                if let Err(e) = self.shipper.ship_metrics(request_id.clone(), metrics).await {
                    warn!("Failed to ship invoke metrics: {}", e);
                }
                
                // Update state
                let mut state = self.state_manager.lock().await;
                state.record_invoke(&request_id);
            }
        }
        
        Ok(())
    }
    
    // Ship custom metrics
    pub async fn ship_custom_metrics(&self, request_id: &str, metrics: Value) -> Result<()> {
        self.shipper.ship_metrics(request_id.to_string(), metrics).await
    }
    
    // Ship response metrics
    pub async fn ship_response_metrics(
        &self, 
        request_id: &str, 
        status_code: u16, 
        duration_ms: u64,
        memory_used_mb: u64,
    ) -> Result<()> {
        let response_event = LambdaTelemetryEvent::Response {
            request_id: request_id.to_string(),
            status_code,
            duration_ms,
            memory_used_mb,
        };
        
        let metrics = serde_json::to_value(response_event)?;
        self.shipper.ship_metrics(request_id.to_string(), metrics).await
    }
    
    // Ship error metrics
    pub async fn ship_error_metrics(
        &self,
        request_id: &str,
        error_type: &str,
        error_message: &str,
        stack_trace: Option<String>,
    ) -> Result<()> {
        let error_event = LambdaTelemetryEvent::Error {
            request_id: request_id.to_string(),
            error_type: error_type.to_string(),
            error_message: error_message.to_string(),
            stack_trace,
        };
        
        let metrics = serde_json::to_value(error_event)?;
        self.shipper.ship_metrics(request_id.to_string(), metrics).await
    }
    
    // Force flush all pending metrics
    pub async fn force_flush(&self) -> Result<()> {
        self.shipper.flush().await
    }
}
