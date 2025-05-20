use anyhow::Result;
use serde_json::{json, Value};
use std::time::Instant;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use async_trait::async_trait;

// Trait for metrics collection
#[async_trait]
pub trait MetricsCollector: Send + Sync {
    async fn collect_metrics(&self) -> Result<Value>;
}

// Real metrics collector implementation
pub struct RuntimeMetricsCollector {
    start_time: Instant,
    invocation_count: AtomicU64,
}

impl RuntimeMetricsCollector {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            invocation_count: AtomicU64::new(0),
        }
    }
}

#[async_trait]
impl MetricsCollector for RuntimeMetricsCollector {
    async fn collect_metrics(&self) -> Result<Value> {
        // Increment invocation counter atomically
        let count = self.invocation_count.fetch_add(1, Ordering::SeqCst);
        
        // Get memory usage
        let memory_usage = capture_memory_usage()?;
        
        // Calculate uptime (performance optimization: use Duration directly)
        let uptime_ms = self.start_time.elapsed().as_millis() as u64;
        
        // Create timestamp only once
        let timestamp = chrono::Utc::now().timestamp_millis();
        
        // Collect metrics into a JSON object (with capacity hint)
        let metrics = json!({
            "memory_usage_mb": memory_usage,
            "uptime_ms": uptime_ms,
            "timestamp": timestamp,
            "invocation_count": count,
            "environment": {
                "function_name": std::env::var("AWS_LAMBDA_FUNCTION_NAME").unwrap_or_default(),
                "function_version": std::env::var("AWS_LAMBDA_FUNCTION_VERSION").unwrap_or_default(),
                "memory_size": std::env::var("AWS_LAMBDA_FUNCTION_MEMORY_SIZE").unwrap_or_default(),
                "region": std::env::var("AWS_REGION").unwrap_or_default(),
            }
        });
        
        Ok(metrics)
    }
}

// Memory usage capture optimized for Linux
fn capture_memory_usage() -> Result<f64> {
    #[cfg(target_os = "linux")]
    {
        // Read memory info from /proc - more efficient than spawning a process
        let statm = std::fs::read_to_string("/proc/self/statm")?;
        let values: Vec<&str> = statm.split_whitespace().collect();
        if values.len() >= 2 {
            // Second value is resident set size (RSS) in pages
            let rss_pages: u64 = values[1].parse()?;
            // Get actual page size from system for accuracy
            let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) } as u64;
            // Convert to MB with minimal floating point ops
            let memory_mb = (rss_pages * page_size) as f64 / (1024.0 * 1024.0);
            return Ok(memory_mb);
        }
    }
    
    // Fallback with reasonable default
    Ok(0.0)
}
