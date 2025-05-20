// agent/collectors/src/system/memory.rs

use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use agent_core::telemetry::metrics::MetricValue;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::{Collector, CollectorConfig, CollectorFactory, CollectorMetadata};
use super::{SystemCollectorSettings, merge_system_settings};

/// Memory metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MemoryMetrics {
    /// Timestamp
    pub timestamp: u64,
    /// Total physical memory in bytes
    pub total_bytes: u64,
    /// Used physical memory in bytes
    pub used_bytes: u64,
    /// Free physical memory in bytes
    pub free_bytes: u64,
    /// Cached memory in bytes
    pub cached_bytes: Option<u64>,
    /// Buffers in bytes
    pub buffers_bytes: Option<u64>,
    /// Active memory in bytes
    pub active_bytes: Option<u64>,
    /// Inactive memory in bytes
    pub inactive_bytes: Option<u64>,
    /// Total swap in bytes
    pub swap_total_bytes: Option<u64>,
    /// Used swap in bytes
    pub swap_used_bytes: Option<u64>,
    /// Free swap in bytes
    pub swap_free_bytes: Option<u64>,
}

/// Memory collector for monitoring memory usage
pub struct MemoryCollector {
    /// Collector configuration
    config: CollectorConfig,
    /// Memory collector settings
    settings: SystemCollectorSettings,
    /// Last memory metrics
    last_metrics: RwLock<Option<MemoryMetrics>>,
    /// Historical metrics (for intervals)
    history: RwLock<Vec<MemoryMetrics>>,
    /// Last collection time
    last_collection: RwLock<Option<Instant>>,
}

impl MemoryCollector {
    /// Create a new memory collector
    pub fn new(config: CollectorConfig) -> Result<Self> {
        let default_settings = SystemCollectorSettings::default();
        let settings = merge_system_settings(&default_settings, config.settings.as_ref())?;
        
        Ok(Self {
            config,
            settings,
            last_metrics: RwLock::new(None),
            history: RwLock::new(Vec::new()),
            last_collection: RwLock::new(None),
        })
    }
    
    /// Collect memory metrics
    async fn collect_memory_metrics(&self) -> Result<MemoryMetrics> {
        // On Linux, read from /proc/meminfo
        // On other platforms, use platform-specific APIs
        
        // This is a simplified implementation for demonstration
        // A real implementation would use something like sysinfo crate
        
        #[cfg(target_os = "linux")]
        {
            use std::fs::File;
            use std::io::{BufRead, BufReader};
            
            let file = File::open("/proc/meminfo")?;
            let reader = BufReader::new(file);
            
            let mut mem_total = 0;
            let mut mem_free = 0;
            let mut mem_available = 0;
            let mut cached = None;
            let mut buffers = None;
            let mut active = None;
            let mut inactive = None;
            let mut swap_total = None;
            let mut swap_free = None;
            
            for line in reader.lines() {
                let line = line?;
                
                if line.starts_with("MemTotal:") {
                    mem_total = parse_meminfo_value(&line);
                } else if line.starts_with("MemFree:") {
                    mem_free = parse_meminfo_value(&line);
                } else if line.starts_with("MemAvailable:") {
                    mem_available = parse_meminfo_value(&line);
                } else if line.starts_with("Cached:") {
                    cached = Some(parse_meminfo_value(&line));
                } else if line.starts_with("Buffers:") {
                    buffers = Some(parse_meminfo_value(&line));
                } else if line.starts_with("Active:") {
                    active = Some(parse_meminfo_value(&line));
                } else if line.starts_with("Inactive:") {
                    inactive = Some(parse_meminfo_value(&line));
                } else if line.starts_with("SwapTotal:") {
                    swap_total = Some(parse_meminfo_value(&line));
                } else if line.starts_with("SwapFree:") {
                    swap_free = Some(parse_meminfo_value(&line));
                }
            }
            
            // Convert from KB to bytes
            let total_bytes = mem_total * 1024;
            let free_bytes = mem_free * 1024;
            let used_bytes = if mem_available > 0 {
                total_bytes - (mem_available * 1024)
            } else {
                total_bytes - free_bytes - cached.unwrap_or(0) * 1024 - buffers.unwrap_or(0) * 1024
            };
            
            let swap_used_bytes = match (swap_total, swap_free) {
                (Some(total), Some(free)) => Some((total - free) * 1024),
                _ => None,
            };
            
            Ok(MemoryMetrics {
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
                total_bytes,
                used_bytes,
                free_bytes,
                cached_bytes: cached.map(|c| c * 1024),
                buffers_bytes: buffers.map(|b| b * 1024),
                active_bytes: active.map(|a| a * 1024),
                inactive_bytes: inactive.map(|i| i * 1024),
                swap_total_bytes: swap_total.map(|t| t * 1024),
                swap_free_bytes: swap_free.map(|f| f * 1024),
                swap_used_bytes,
            })
        }
        
        #[cfg(not(target_os = "linux"))]
        {
            // Simplified implementation for non-Linux platforms
            // A real implementation would use platform-specific APIs
            
            // Simulate memory usage
            let total_bytes = 16 * 1024 * 1024 * 1024; // 16 GB
            let used_bytes = (total_bytes as f64 * (rand::random::<f64>() * 0.5 + 0.2)) as u64;
            let free_bytes = total_bytes - used_bytes;
            
            Ok(MemoryMetrics {
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
                total_bytes,
                used_bytes,
                free_bytes,
                cached_bytes: Some((total_bytes as f64 * 0.2) as u64),
                buffers_bytes: Some((total_bytes as f64 * 0.05) as u64),
                active_bytes: Some((total_bytes as f64 * 0.4) as u64),
                inactive_bytes: Some((total_bytes as f64 * 0.3) as u64),
                swap_total_bytes: Some(8 * 1024 * 1024 * 1024), // 8 GB
                swap_used_bytes: Some((8 * 1024 * 1024 * 1024) as f64 * 0.1) as u64,
                swap_free_bytes: Some((8 * 1024 * 1024 * 1024) as f64 * 0.9) as u64,
            })
        }
    }
}

#[async_trait]
impl Collector for MemoryCollector {
    fn metadata(&self) -> CollectorMetadata {
        CollectorMetadata {
            id: self.config.id.clone(),
            name: "Memory Collector".to_string(),
            description: "Collects memory usage metrics".to_string(),
            source_type: "system.memory".to_string(),
            enabled_by_default: true,
        }
    }
    
    fn config(&self) -> CollectorConfig {
        self.config.clone()
    }
    
    fn update_config(&mut self, config: CollectorConfig) -> Result<()> {
        // Update settings from config
        if let Some(settings) = &config.settings {
            self.settings = merge_system_settings(&self.settings, Some(settings))?;
        }
        
        self.config = config;
        Ok(())
    }
    
    async fn initialize(&mut self) -> Result<()> {
        info!("Initializing memory collector");
        
        // Collect initial metrics
        let metrics = self.collect_memory_metrics().await?;
        *self.last_metrics.write().await = Some(metrics);
        *self.last_collection.write().await = Some(Instant::now());
        
        Ok(())
    }
    
    async fn collect(&self) -> Result<HashMap<String, MetricValue>> {
        debug!("Collecting memory metrics");
        
        // Collect new metrics
        let metrics = self.collect_memory_metrics().await?;
        
        // Store metrics
        {
            let mut last_metrics = self.last_metrics.write().await;
            *last_metrics = Some(metrics.clone());
            
            let mut last_collection = self.last_collection.write().await;
            *last_collection = Some(Instant::now());
            
            let mut history = self.history.write().await;
            history.push(metrics.clone());
            
            // Limit history size
            while history.len() > 60 {
                history.remove(0);
            }
        }
        
        // Convert to metrics map
        let mut result = HashMap::new();
        
        // Add memory metrics
        result.insert("system.memory.total_bytes".to_string(), MetricValue::Gauge(metrics.total_bytes as f64));
        result.insert("system.memory.used_bytes".to_string(), MetricValue::Gauge(metrics.used_bytes as f64));
        result.insert("system.memory.free_bytes".to_string(), MetricValue::Gauge(metrics.free_bytes as f64));
        
        // Calculate usage percentage
        let usage_pct = if metrics.total_bytes > 0 {
            metrics.used_bytes as f64 / metrics.total_bytes as f64
        } else {
            0.0
        };
        result.insert("system.memory.usage_pct".to_string(), MetricValue::Gauge(usage_pct));
        
        // Add cached/buffers if available
        if let Some(cached) = metrics.cached_bytes {
            result.insert("system.memory.cached_bytes".to_string(), MetricValue::Gauge(cached as f64));
        }
        
        if let Some(buffers) = metrics.buffers_bytes {
            result.insert("system.memory.buffers_bytes".to_string(), MetricValue::Gauge(buffers as f64));
        }
        
        // Add active/inactive if available
        if let Some(active) = metrics.active_bytes {
            result.insert("system.memory.active_bytes".to_string(), MetricValue::Gauge(active as f64));
        }
        
        if let Some(inactive) = metrics.inactive_bytes {
            result.insert("system.memory.inactive_bytes".to_string(), MetricValue::Gauge(inactive as f64));
        }
        
        // Add swap metrics if available
        if let Some(swap_total) = metrics.swap_total_bytes {
            result.insert("system.memory.swap.total_bytes".to_string(), MetricValue::Gauge(swap_total as f64));
        }
        
        if let Some(swap_used) = metrics.swap_used_bytes {
            result.insert("system.memory.swap.used_bytes".to_string(), MetricValue::Gauge(swap_used as f64));
            
            // Calculate swap usage percentage
            if let Some(swap_total) = metrics.swap_total_bytes {
                if swap_total > 0 {
                    let swap_usage_pct = swap_used as f64 / swap_total as f64;
                    result.insert("system.memory.swap.usage_pct".to_string(), MetricValue::Gauge(swap_usage_pct));
                }
            }
        }
        
        if let Some(swap_free) = metrics.swap_free_bytes {
            result.insert("system.memory.swap.free_bytes".to_string(), MetricValue::Gauge(swap_free as f64));
        }
        
        Ok(result)
    }
    
    async fn shutdown(&self) -> Result<()> {
        debug!("Shutting down memory collector");
        Ok(())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Factory for creating memory collectors
pub struct MemoryCollectorFactory {
    metadata: CollectorMetadata,
}

impl MemoryCollectorFactory {
    /// Create a new memory collector factory
    pub fn new() -> Self {
        Self {
            metadata: CollectorMetadata {
                id: "system.memory".to_string(),
                name: "Memory Collector".to_string(),
                description: "Collects memory usage metrics".to_string(),
                source_type: "system.memory".to_string(),
                enabled_by_default: true,
            },
        }
    }
}

#[async_trait]
impl CollectorFactory for MemoryCollectorFactory {
    async fn create(&self, config: CollectorConfig) -> Result<Box<dyn Collector>> {
        let collector = MemoryCollector::new(config)?;
        Ok(Box::new(collector))
    }
    
    fn metadata(&self) -> CollectorMetadata {
        self.metadata.clone()
    }
}

/// Parse a value from /proc/meminfo
#[cfg(target_os = "linux")]
fn parse_meminfo_value(line: &str) -> u64 {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 2 {
        parts[1].parse::<u64>().unwrap_or(0)
    } else {
        0
    }
}