// agent/core/lib/transport/buffer.rs

use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::Mutex;
use chrono::{DateTime, Utc};
use std::path::PathBuf;

use crate::telemetry::protocol::binary::MetricEntry;
use heapless::pool::singleton::PoolPtr;

// Buffer implementation for telemetry data
pub struct TelemetryBuffer {
    buffer: Mutex<Vec<PoolPtr<MetricEntry>>>,
    batch_size: usize,
    metrics_count: AtomicUsize,
    temp_file_path: PathBuf,
}

impl TelemetryBuffer {
    pub fn new(batch_size: usize, temp_dir_path: &str) -> Self {
        // Create temp directory if it doesn't exist
        let temp_dir = PathBuf::from(temp_dir_path);
        std::fs::create_dir_all(&temp_dir).unwrap_or_default();
        let temp_file = temp_dir.join("metrics-buffer.bin");
        
        // Pre-allocate buffer with capacity to avoid resizing
        let buffer = Mutex::new(Vec::with_capacity(batch_size * 2));
        
        Self {
            buffer,
            batch_size,
            metrics_count: AtomicUsize::new(0),
            temp_file_path: temp_file,
        }
    }
    
    // Get the current buffer size
    pub fn get_metrics_count(&self) -> usize {
        self.metrics_count.load(Ordering::Relaxed)
    }
    
    // Add metric to buffer
    pub async fn add_metric(&self, entry: PoolPtr<MetricEntry>) -> bool {
        // Increment metrics count
        self.metrics_count.fetch_add(1, Ordering::Relaxed);
        
        // Lock buffer only when necessary and minimize critical section
        let should_flush;
        {
            let mut buffer = self.buffer.lock().await;
            buffer.push(entry);
            should_flush = buffer.len() >= self.batch_size;
        }
        
        should_flush
    }
    
    // Flush the buffer and return the entries
    pub async fn flush(&self) -> Vec<PoolPtr<MetricEntry>> {
        let entries = {
            let mut buffer = self.buffer.lock().await;
            if buffer.is_empty() {
                return Vec::new();
            }
            // Use drain which is more efficient than clone+clear
            buffer.drain(..).collect::<Vec<_>>()
        };
        
        entries
    }
    
    // Get the temporary file path for backup
    pub fn get_temp_file_path(&self) -> &PathBuf {
        &self.temp_file_path
    }
    
    // Get the batch size
    pub fn get_batch_size(&self) -> usize {
        self.batch_size
    }
    
    // Backup metrics to disk in case of failed transmission
    pub fn backup_buffer(&self, data: &[u8]) -> anyhow::Result<()> {
        // Skip if empty
        if data.is_empty() {
            return Ok(());
        }
        
        // Serialize directly to file
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(&self.temp_file_path)?;
            
        // Use BufWriter for performance
        let mut writer = std::io::BufWriter::new(file);
        
        // Write length prefix (4 bytes)
        writer.write_all(&(data.len() as u32).to_le_bytes())?;
        
        // Write batch data
        writer.write_all(data)?;
        
        Ok(())
    }
    
    // Check if there's backup data to recover
    pub fn has_backup_data(&self) -> bool {
        if !self.temp_file_path.exists() {
            return false;
        }
        
        match std::fs::metadata(&self.temp_file_path) {
            Ok(metadata) => metadata.len() > 0,
            Err(_) => false,
        }
    }
}

// A structure to implement auto-flushing behavior
pub struct AutoFlushBuffer {
    buffer: TelemetryBuffer,
    flush_interval: std::time::Duration,
    running: std::sync::atomic::AtomicBool,
}

impl AutoFlushBuffer {
    pub fn new(buffer: TelemetryBuffer, flush_interval: std::time::Duration) -> Self {
        Self {
            buffer,
            flush_interval,
            running: std::sync::atomic::AtomicBool::new(false),
        }
    }
    
    // Start the auto-flush task in background
    pub fn start_auto_flush<F>(&self, flush_fn: F) 
    where
        F: Fn() -> futures::future::BoxFuture<'static, ()> + Send + Sync + 'static,
    {
        let was_running = self.running.swap(true, Ordering::SeqCst);
        if was_running {
            return; // Already running
        }
        
        let interval = self.flush_interval;
        let running_ref = &self.running;
        
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            
            while running_ref.load(Ordering::SeqCst) {
                interval_timer.tick().await;
                flush_fn().await;
            }
        });
    }
    
    // Stop the auto-flush task
    pub fn stop_auto_flush(&self) {
        self.running.store(false, Ordering::SeqCst);
    }
    
    // Get reference to the underlying buffer
    pub fn get_buffer(&self) -> &TelemetryBuffer {
        &self.buffer
    }
}
