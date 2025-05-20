// agent/telemetry/src/buffer.rs

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use tokio::sync::{RwLock, mpsc};
use tokio::time::interval;
use tracing::{debug, info, warn, error};

use crate::{TelemetryEvent, TelemetryBatch, TelemetryPipeline, TelemetryConfig};

/// Buffer for storing telemetry events
pub struct TelemetryBuffer {
    /// Event buffer
    buffer: RwLock<VecDeque<TelemetryEvent>>,
    /// Maximum buffer size
    max_size: usize,
    /// Flush threshold
    flush_threshold: usize,
    /// Linked pipeline
    pipeline: Option<Arc<TelemetryPipeline>>,
    /// Flush timer
    flush_interval: Duration,
    /// Last flush time
    last_flush: RwLock<Instant>,
    /// Event sender
    event_tx: mpsc::Sender<TelemetryEvent>,
    /// Event receiver
    event_rx: RwLock<mpsc::Receiver<TelemetryEvent>>,
    /// Running state
    running: RwLock<bool>,
}

impl TelemetryBuffer {
    /// Create a new telemetry buffer
    pub fn new(config: &TelemetryConfig) -> Self {
        let (tx, rx) = mpsc::channel(config.max_buffer_size);
        
        Self {
            buffer: RwLock::new(VecDeque::with_capacity(config.max_buffer_size)),
            max_size: config.max_buffer_size,
            flush_threshold: config.buffer_threshold,
            pipeline: None,
            flush_interval: config.buffer_flush_interval_duration(),
            last_flush: RwLock::new(Instant::now()),
            event_tx: tx,
            event_rx: RwLock::new(rx),
            running: RwLock::new(false),
        }
    }
    
    /// Set the pipeline to send events to
    pub async fn set_pipeline(&mut self, pipeline: Arc<TelemetryPipeline>) {
        self.pipeline = Some(pipeline);
    }
    
    /// Start the buffer processing
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            return Ok(());
        }
        
        *running = true;
        
        // Start the background processing task
        let event_rx = self.event_rx.write().await.split().1;
        let buffer = Arc::new(self.buffer.clone());
        let max_size = self.max_size;
        let flush_threshold = self.flush_threshold;
        let flush_interval = self.flush_interval;
        let pipeline = self.pipeline.clone();
        let last_flush = Arc::new(self.last_flush.clone());
        let running_flag = Arc::new(self.running.clone());
        
        tokio::spawn(async move {
            let mut ticker = interval(flush_interval);
            let mut event_rx = event_rx;
            
            loop {
                tokio::select! {
                    // Process incoming event
                    Some(event) = event_rx.recv() => {
                        let mut buffer_guard = buffer.write().await;
                        
                        // Handle buffer overflow
                        if buffer_guard.len() >= max_size {
                            // Remove oldest event
                            buffer_guard.pop_front();
                            warn!("Buffer overflow, dropping oldest event");
                        }
                        
                        // Add event to buffer
                        buffer_guard.push_back(event);
                        
                        // Check if we should flush
                        if buffer_guard.len() >= flush_threshold {
                            if let Err(e) = flush_buffer(&mut buffer_guard, &pipeline, last_flush.clone()).await {
                                error!("Failed to flush buffer: {}", e);
                            }
                        }
                    }
                    
                    // Handle interval-based flush
                    _ = ticker.tick() => {
                        let mut buffer_guard = buffer.write().await;
                        if !buffer_guard.is_empty() {
                            if let Err(e) = flush_buffer(&mut buffer_guard, &pipeline, last_flush.clone()).await {
                                error!("Failed to flush buffer: {}", e);
                            }
                        }
                    }
                    
                    // Exit if not running
                    else => {
                        if !*running_flag.read().await {
                            break;
                        }
                    }
                }
            }
        });
        
        Ok(())
    }
    
    /// Stop the buffer processing
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if !*running {
            return Ok(());
        }
        
        *running = false;
        
        // Flush any remaining events
        let mut buffer = self.buffer.write().await;
        if !buffer.is_empty() {
            if let Some(pipeline) = &self.pipeline {
                flush_buffer_to_pipeline(&mut buffer, pipeline).await?;
            }
        }
        
        Ok(())
    }
    
    /// Add an event to the buffer
    pub async fn add_event(&self, event: TelemetryEvent) -> Result<()> {
        self.event_tx.send(event).await.map_err(|e| anyhow::anyhow!("Failed to add event: {}", e))
    }
    
    /// Flush the buffer manually
    pub async fn flush(&self) -> Result<()> {
        let mut buffer = self.buffer.write().await;
        if buffer.is_empty() {
            return Ok(());
        }
        
        if let Some(pipeline) = &self.pipeline {
            flush_buffer_to_pipeline(&mut buffer, pipeline).await?;
            
            // Update last flush time
            let mut last_flush = self.last_flush.write().await;
            *last_flush = Instant::now();
        } else {
            warn!("No pipeline set, cannot flush buffer");
        }
        
        Ok(())
    }
}

/// Helper function to flush buffer
async fn flush_buffer(
    buffer: &mut VecDeque<TelemetryEvent>,
    pipeline: &Option<Arc<TelemetryPipeline>>,
    last_flush: Arc<RwLock<Instant>>,
) -> Result<()> {
    if buffer.is_empty() {
        return Ok(());
    }
    
    if let Some(pipeline) = pipeline {
        flush_buffer_to_pipeline(buffer, pipeline).await?;
        
        // Update last flush time
        let mut last_flush_guard = last_flush.write().await;
        *last_flush_guard = Instant::now();
    } else {
        warn!("No pipeline set, cannot flush buffer");
    }
    
    Ok(())
}

/// Helper function to flush buffer to pipeline
async fn flush_buffer_to_pipeline(
    buffer: &mut VecDeque<TelemetryEvent>,
    pipeline: &Arc<TelemetryPipeline>,
) -> Result<()> {
    if buffer.is_empty() {
        return Ok(());
    }
    
    // Take events from buffer
    let events: Vec<_> = buffer.drain(..).collect();
    
    // Create batch
    let batch = TelemetryBatch {
        id: uuid::Uuid::new_v4().to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64,
        source: "telemetry_buffer".to_string(),
        events,
        metadata: None,
    };
    
    // Process batch through pipeline
    pipeline.process_batch(batch).await
}