// agent/telemetry/src/config.rs

use std::time::Duration;
use serde::{Deserialize, Serialize};

/// Telemetry system configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    /// Buffer flush threshold (number of events)
    pub buffer_threshold: usize,
    /// Buffer flush interval (seconds)
    pub buffer_flush_interval: u64,
    /// Maximum buffer size
    pub max_buffer_size: usize,
    /// Maximum batch size
    pub max_batch_size: usize,
    /// Enable compression
    pub compression: bool,
    /// Enable encryption
    pub encryption: bool,
    /// Sampling rate (0.0-1.0)
    pub sampling_rate: f64,
    /// Maximum retry attempts
    pub max_retry_attempts: usize,
    /// Initial retry delay (milliseconds)
    pub initial_retry_delay_ms: u64,
    /// Maximum retry delay (milliseconds)
    pub max_retry_delay_ms: u64,
    /// Collection interval (seconds)
    pub collection_interval: u64,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            buffer_threshold: 100,
            buffer_flush_interval: 5,
            max_buffer_size: 1000,
            max_batch_size: 200,
            compression: true,
            encryption: false,
            sampling_rate: 1.0,
            max_retry_attempts: 3,
            initial_retry_delay_ms: 100,
            max_retry_delay_ms: 5000,
            collection_interval: 60,
        }
    }
}

impl TelemetryConfig {
    /// Create a minimal configuration for testing
    pub fn minimal() -> Self {
        Self {
            buffer_threshold: 10,
            buffer_flush_interval: 1,
            max_buffer_size: 100,
            max_batch_size: 50,
            compression: false,
            encryption: false,
            sampling_rate: 1.0,
            max_retry_attempts: 1,
            initial_retry_delay_ms: 10,
            max_retry_delay_ms: 100,
            collection_interval: 10,
        }
    }
    
    /// Create a configuration for high throughput
    pub fn high_throughput() -> Self {
        Self {
            buffer_threshold: 1000,
            buffer_flush_interval: 10,
            max_buffer_size: 10000,
            max_batch_size: 500,
            compression: true,
            encryption: false,
            sampling_rate: 0.1, // Only sample 10% of events
            max_retry_attempts: 5,
            initial_retry_delay_ms: 50,
            max_retry_delay_ms: 10000,
            collection_interval: 300, // 5 minutes
        }
    }
    
    /// Create a configuration for high reliability
    pub fn high_reliability() -> Self {
        Self {
            buffer_threshold: 50,
            buffer_flush_interval: 2,
            max_buffer_size: 5000,
            max_batch_size: 100,
            compression: true,
            encryption: true,
            sampling_rate: 1.0, // Collect all events
            max_retry_attempts: 10,
            initial_retry_delay_ms: 100,
            max_retry_delay_ms: 30000,
            collection_interval: 30,
        }
    }
    
    /// Get buffer flush interval as duration
    pub fn buffer_flush_interval_duration(&self) -> Duration {
        Duration::from_secs(self.buffer_flush_interval)
    }
    
    /// Get initial retry delay as duration
    pub fn initial_retry_delay(&self) -> Duration {
        Duration::from_millis(self.initial_retry_delay_ms)
    }
    
    /// Get maximum retry delay as duration
    pub fn max_retry_delay(&self) -> Duration {
        Duration::from_millis(self.max_retry_delay_ms)
    }
    
    /// Get collection interval as duration
    pub fn collection_interval_duration(&self) -> Duration {
        Duration::from_secs(self.collection_interval)
    }
}