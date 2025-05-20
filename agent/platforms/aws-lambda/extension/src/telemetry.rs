// agent/platforms/aws-lambda/extension/src/telemetry.rs

use anyhow::Result;
use lambda_extension::{Extension, LambdaEvent, NextEvent};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

use crate::state::LambdaStateManager;
use agent_core::startup::is_cold_start;
use causal_telemetry::{TelemetryTransport, TelemetryPipeline, TelemetryEvent, TelemetryBatch, TelemetryEventBuilder};

/// Lambda-specific telemetry events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LambdaTelemetryEvent {
    /// Invocation event
    Invoke {
        /// Request ID
        request_id: String,
        /// Lambda runtime
        runtime: String,
        /// Timestamp
        timestamp: u64,
        /// Function timeout
        timeout: u32,
    },
    /// Response event
    Response {
        /// Request ID
        request_id: String,
        /// Status code
        status_code: u16,
        /// Duration in milliseconds
        duration_ms: u64,
        /// Memory used in MB
        memory_used_mb: u64,
    },
    /// Error event
    Error {
        /// Request ID
        request_id: String,
        /// Error type
        error_type: String,
        /// Error message
        error_message: String,
        /// Stack trace
        stack_trace: Option<String>,
    },
    /// Extension event
    Extension {
        /// Event type
        event_type: String,
        /// Duration in milliseconds
        duration_ms: u64,
        /// Memory used in MB
        memory_used_mb: u64,
    },
    /// Metrics snapshot
    MetricsSnapshot {
        /// CPU usage percent
        cpu_usage_percent: f64,
        /// Memory usage in MB
        memory_usage_mb: u64,
        /// Network receive bytes
        network_rx_bytes: u64,
        /// Network transmit bytes
        network_tx_bytes: u64,
    },
}

/// Lambda Extension integration for telemetry
pub struct LambdaTelemetryExtension {
    /// Telemetry pipeline
    pipeline: Arc<TelemetryPipeline>,
    /// State manager
    state_manager: Arc<Mutex<LambdaStateManager>>,
    /// Extension ID
    extension_id: String,
    /// Flush interval
    flush_interval: Duration,
    /// Auto-flush running flag
    auto_flush_running: AtomicBool,
}

impl LambdaTelemetryExtension {
    /// Create a new Lambda telemetry extension
    pub fn new(
        extension_id: String,
        endpoint: String,
        batch_size: usize,
        state_manager: Arc<Mutex<LambdaStateManager>>,
        flush_interval: Duration,
    ) -> Self {
        // Create HTTP transport
        let transport = Arc::new(
            causal_telemetry::transport::HttpTransport::new("lambda-http", &endpoint)
                .with_retry_config(causal_telemetry::transport::RetryConfig {
                    max_retries: 3,
                    initial_delay: Duration::from_millis(100),
                    max_delay: Duration::from_secs(5),
                    backoff_factor: 2.0,
                })
                .with_compression(true)
        ) as Arc<dyn TelemetryTransport>;
        
        // Create pipeline
        let pipeline = TelemetryPipeline::new("lambda-telemetry");
        let pipeline = Arc::new(pipeline);
        
        // Add transport to pipeline
        tokio::spawn(async move {
            pipeline.add_transport(transport).await;
        });
        
        Self {
            pipeline,
            state_manager,
            extension_id,
            flush_interval,
            auto_flush_running: AtomicBool::new(false),
        }
    }
    
    /// Start auto-flush background task
    pub fn start_auto_flush(&self) {
        let was_running = self.auto_flush_running.swap(true, std::sync::atomic::Ordering::SeqCst);
        if was_running {
            return; // Already running
        }
        
        let pipeline = self.pipeline.clone();
        let interval = self.flush_interval;
        let auto_flush_running = &self.auto_flush_running;
        
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            
            while auto_flush_running.load(std::sync::atomic::Ordering::SeqCst) {
                interval_timer.tick().await;
                
                if let Err(e) = Self::flush_pipeline(&pipeline).await {
                    warn!("Auto-flush failed: {}", e);
                }
            }
        });
    }
    
    /// Stop auto-flush background task
    pub fn stop_auto_flush(&self) {
        self.auto_flush_running.store(false, std::sync::atomic::Ordering::SeqCst);
    }
    
    /// Process Lambda extension events
    pub async fn process_event(&self, event: LambdaEvent) -> Result<()> {
        match event.next {
            NextEvent::Shutdown(shutdown) => {
                // Flush all pending metrics before shutdown
                info!("Extension shutting down: {}", shutdown.shutdown_reason);
                
                if let Err(e) = Self::flush_pipeline(&self.pipeline).await {
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
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64,
                    timeout: std::env::var("AWS_LAMBDA_FUNCTION_TIMEOUT")
                        .unwrap_or_else(|_| "3".to_string())
                        .parse::<u32>()
                        .unwrap_or(3),
                };
                
                // Convert to JSON
                let metrics = serde_json::to_value(invoke_event)?;
                
                // Create telemetry event
                let event = TelemetryEventBuilder::new("lambda.invoke", "metrics")
                    .with_source("lambda.extension")
                    .with_resource_id(&request_id)
                    .with_data(metrics)
                    .build();
                
                // Ship metrics
                if let Err(e) = self.ship_event(event).await {
                    warn!("Failed to ship invoke metrics: {}", e);
                }
            }
        }
        
        Ok(())
    }
    
    /// Ship a telemetry event
    async fn ship_event(&self, event: TelemetryEvent) -> Result<()> {
        // Create batch with single event
        let batch = TelemetryBatch {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            source: "lambda.extension".to_string(),
            events: vec![event],
            metadata: None,
        };
        
        // Process batch through pipeline
        self.pipeline.process_batch(batch).await
    }
    
    /// Helper to flush pipeline
    async fn flush_pipeline(pipeline: &Arc<TelemetryPipeline>) -> Result<()> {
        // No direct flush in pipeline, but we could implement if needed
        Ok(())
    }
    
    /// Ship custom metrics
    pub async fn ship_custom_metrics(&self, request_id: &str, metrics: Value) -> Result<()> {
        let event = TelemetryEventBuilder::new("lambda.custom", "metrics")
            .with_source("lambda.extension")
            .with_resource_id(request_id)
            .with_data(metrics)
            .build();
            
        self.ship_event(event).await
    }
    
    /// Ship response metrics
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
        
        let event = TelemetryEventBuilder::new("lambda.response", "metrics")
            .with_source("lambda.extension")
            .with_resource_id(request_id)
            .with_data(metrics)
            .build();
            
        self.ship_event(event).await
    }
    
    /// Ship error metrics
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
        
        let event = TelemetryEventBuilder::new("lambda.error", "metrics")
            .with_source("lambda.extension")
            .with_resource_id(request_id)
            .with_data(metrics)
            .build();
            
        self.ship_event(event).await
    }
    
    /// Force flush all pending metrics
    pub async fn force_flush(&self) -> Result<()> {
        Self::flush_pipeline(&self.pipeline).await
    }
}
