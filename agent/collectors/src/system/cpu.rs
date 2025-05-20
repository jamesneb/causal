// agent/collectors/src/system/cpu.rs

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

use crate::{Collector, CollectorConfig, CollectorFactory, CollectorMetadata, CollectionFrequency};
use super::{SystemCollectorSettings, merge_system_settings};

/// CPU metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CpuMetrics {
    /// Timestamp
    pub timestamp: u64,
    /// CPU usage (0.0-1.0)
    pub usage: f64,
    /// Per-core usage
    pub per_core: Option<Vec<f64>>,
    /// CPU user time (0.0-1.0)
    pub user: f64,
    /// CPU system time (0.0-1.0)
    pub system: f64,
    /// CPU idle time (0.0-1.0)
    pub idle: f64,
    /// CPU temperature (Celsius) if available
    pub temperature: Option<f64>,
    /// Load average (1 minute)
    pub load_avg_1min: Option<f64>,
    /// Load average (5 minutes)
    pub load_avg_5min: Option<f64>,
    /// Load average (15 minutes)
    pub load_avg_15min: Option<f64>,
    /// CPU frequency (MHz) if available
    pub frequency_mhz: Option<f64>,
}

/// CPU collector for monitoring CPU usage
pub struct CpuCollector {
    /// Collector configuration
    config: CollectorConfig,
    /// CPU collector settings
    settings: SystemCollectorSettings,
    /// Last CPU metrics
    last_metrics: RwLock<Option<CpuMetrics>>,
    /// Historical metrics (for intervals)
    history: RwLock<Vec<CpuMetrics>>,
    /// Last collection time
    last_collection: RwLock<Option<Instant>>,
    /// Number of CPU cores
    num_cores: usize,
}

impl CpuCollector {
    /// Create a new CPU collector
    pub fn new(config: CollectorConfig) -> Result<Self> {
        let default_settings = SystemCollectorSettings::default();
        let settings = merge_system_settings(&default_settings, config.settings.as_ref())?;
        
        // Detect number of CPU cores
        let num_cores = num_cpus::get();
        
        Ok(Self {
            config,
            settings,
            last_metrics: RwLock::new(None),
            history: RwLock::new(Vec::new()),
            last_collection: RwLock::new(None),
            num_cores,
        })
    }
    
    /// Collect CPU metrics
    async fn collect_cpu_metrics(&self) -> Result<CpuMetrics> {
        // On Linux, read from /proc/stat
        // On other platforms, use platform-specific APIs
        
        // This is a simplified implementation for demonstration
        // A real implementation would use something like sysinfo crate
        
        #[cfg(target_os = "linux")]
        {
            use std::fs::File;
            use std::io::{BufRead, BufReader};
            
            let file = File::open("/proc/stat")?;
            let reader = BufReader::new(file);
            
            let mut user = 0.0;
            let mut nice = 0.0;
            let mut system = 0.0;
            let mut idle = 0.0;
            let mut iowait = 0.0;
            let mut irq = 0.0;
            let mut softirq = 0.0;
            let mut steal = 0.0;
            
            let mut per_core_metrics = if self.settings.collect_per_core {
                Some(Vec::with_capacity(self.num_cores))
            } else {
                None
            };
            
            for line in reader.lines() {
                let line = line?;
                
                if line.starts_with("cpu ") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 8 {
                        user = parts[1].parse::<f64>()?;
                        nice = parts[2].parse::<f64>()?;
                        system = parts[3].parse::<f64>()?;
                        idle = parts[4].parse::<f64>()?;
                        iowait = parts[5].parse::<f64>()?;
                        irq = parts[6].parse::<f64>()?;
                        softirq = parts[7].parse::<f64>()?;
                        
                        if parts.len() >= 9 {
                            steal = parts[8].parse::<f64>()?;
                        }
                    }
                } else if let Some(per_core) = &mut per_core_metrics {
                    if line.starts_with("cpu") && line.len() > 3 {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 8 {
                            let core_user = parts[1].parse::<f64>().unwrap_or(0.0);
                            let core_nice = parts[2].parse::<f64>().unwrap_or(0.0);
                            let core_system = parts[3].parse::<f64>().unwrap_or(0.0);
                            let core_idle = parts[4].parse::<f64>().unwrap_or(0.0);
                            let core_iowait = parts[5].parse::<f64>().unwrap_or(0.0);
                            let core_irq = parts[6].parse::<f64>().unwrap_or(0.0);
                            let core_softirq = parts[7].parse::<f64>().unwrap_or(0.0);
                            let core_steal = if parts.len() >= 9 {
                                parts[8].parse::<f64>().unwrap_or(0.0)
                            } else {
                                0.0
                            };
                            
                            let core_total = core_user + core_nice + core_system + core_idle + 
                                            core_iowait + core_irq + core_softirq + core_steal;
                            
                            if core_total > 0.0 {
                                let core_usage = 1.0 - core_idle / core_total;
                                per_core.push(core_usage);
                            }
                        }
                    }
                }
            }
            
            let total = user + nice + system + idle + iowait + irq + softirq + steal;
            let usage = if total > 0.0 {
                1.0 - idle / total
            } else {
                0.0
            };
            
            let user_pct = if total > 0.0 { user / total } else { 0.0 };
            let system_pct = if total > 0.0 { system / total } else { 0.0 };
            let idle_pct = if total > 0.0 { idle / total } else { 1.0 };
            
            // Read load average
            let mut load_avg_1min = None;
            let mut load_avg_5min = None;
            let mut load_avg_15min = None;
            
            if let Ok(file) = File::open("/proc/loadavg") {
                let reader = BufReader::new(file);
                if let Some(Ok(line)) = reader.lines().next() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 3 {
                        load_avg_1min = parts[0].parse::<f64>().ok();
                        load_avg_5min = parts[1].parse::<f64>().ok();
                        load_avg_15min = parts[2].parse::<f64>().ok();
                    }
                }
            }
            
            // No simple way to get CPU temperature or frequency from /proc
            // A real implementation would use something like lm-sensors
            
            Ok(CpuMetrics {
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
                usage,
                per_core: per_core_metrics,
                user: user_pct,
                system: system_pct,
                idle: idle_pct,
                temperature: None,
                load_avg_1min,
                load_avg_5min,
                load_avg_15min,
                frequency_mhz: None,
            })
        }
        
        #[cfg(not(target_os = "linux"))]
        {
            // Simplified implementation for non-Linux platforms
            // A real implementation would use platform-specific APIs
            
            let mut per_core = if self.settings.collect_per_core {
                let cores = self.num_cores;
                let mut per_core_vec = Vec::with_capacity(cores);
                for _ in 0..cores {
                    // Simulate some random CPU usage per core
                    per_core_vec.push(rand::random::<f64>() * 0.5 + 0.1);
                }
                Some(per_core_vec)
            } else {
                None
            };
            
            // Simulate overall CPU usage
            let usage = rand::random::<f64>() * 0.5 + 0.1;
            let user = usage * 0.7;
            let system = usage * 0.3;
            let idle = 1.0 - usage;
            
            Ok(CpuMetrics {
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
                usage,
                per_core,
                user,
                system,
                idle,
                temperature: None,
                load_avg_1min: Some(rand::random::<f64>() * 2.0),
                load_avg_5min: Some(rand::random::<f64>() * 1.5),
                load_avg_15min: Some(rand::random::<f64>() * 1.0),
                frequency_mhz: None,
            })
        }
    }
}

#[async_trait]
impl Collector for CpuCollector {
    fn metadata(&self) -> CollectorMetadata {
        CollectorMetadata {
            id: self.config.id.clone(),
            name: "CPU Collector".to_string(),
            description: "Collects CPU usage metrics".to_string(),
            source_type: "system.cpu".to_string(),
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
        info!("Initializing CPU collector");
        
        // Collect initial metrics
        let metrics = self.collect_cpu_metrics().await?;
        *self.last_metrics.write().await = Some(metrics);
        *self.last_collection.write().await = Some(Instant::now());
        
        Ok(())
    }
    
    async fn collect(&self) -> Result<HashMap<String, MetricValue>> {
        debug!("Collecting CPU metrics");
        
        // Collect new metrics
        let metrics = self.collect_cpu_metrics().await?;
        
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
        
        // Add overall CPU metrics
        result.insert("system.cpu.usage".to_string(), MetricValue::Gauge(metrics.usage));
        result.insert("system.cpu.user".to_string(), MetricValue::Gauge(metrics.user));
        result.insert("system.cpu.system".to_string(), MetricValue::Gauge(metrics.system));
        result.insert("system.cpu.idle".to_string(), MetricValue::Gauge(metrics.idle));
        
        // Add per-core metrics if available
        if let Some(per_core) = metrics.per_core {
            for (i, usage) in per_core.iter().enumerate() {
                result.insert(
                    format!("system.cpu.core.{}.usage", i),
                    MetricValue::Gauge(*usage),
                );
            }
        }
        
        // Add load average if available
        if let Some(load1) = metrics.load_avg_1min {
            result.insert("system.cpu.load.1m".to_string(), MetricValue::Gauge(load1));
        }
        
        if let Some(load5) = metrics.load_avg_5min {
            result.insert("system.cpu.load.5m".to_string(), MetricValue::Gauge(load5));
        }
        
        if let Some(load15) = metrics.load_avg_15min {
            result.insert("system.cpu.load.15m".to_string(), MetricValue::Gauge(load15));
        }
        
        // Add temperature if available
        if let Some(temp) = metrics.temperature {
            result.insert("system.cpu.temperature".to_string(), MetricValue::Gauge(temp));
        }
        
        // Add frequency if available
        if let Some(freq) = metrics.frequency_mhz {
            result.insert("system.cpu.frequency_mhz".to_string(), MetricValue::Gauge(freq));
        }
        
        // Add number of cores
        result.insert("system.cpu.cores".to_string(), MetricValue::Gauge(self.num_cores as f64));
        
        Ok(result)
    }
    
    async fn shutdown(&self) -> Result<()> {
        debug!("Shutting down CPU collector");
        Ok(())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Factory for creating CPU collectors
pub struct CpuCollectorFactory {
    metadata: CollectorMetadata,
}

impl CpuCollectorFactory {
    /// Create a new CPU collector factory
    pub fn new() -> Self {
        Self {
            metadata: CollectorMetadata {
                id: "system.cpu".to_string(),
                name: "CPU Collector".to_string(),
                description: "Collects CPU usage metrics".to_string(),
                source_type: "system.cpu".to_string(),
                enabled_by_default: true,
            },
        }
    }
}

#[async_trait]
impl CollectorFactory for CpuCollectorFactory {
    async fn create(&self, config: CollectorConfig) -> Result<Box<dyn Collector>> {
        let collector = CpuCollector::new(config)?;
        Ok(Box::new(collector))
    }
    
    fn metadata(&self) -> CollectorMetadata {
        self.metadata.clone()
    }
}