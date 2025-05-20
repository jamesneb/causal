// agent/collectors/src/system/network.rs

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

/// Network interface metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
struct NetworkInterfaceMetrics {
    /// Interface name
    pub name: String,
    /// Bytes received
    pub rx_bytes: u64,
    /// Bytes transmitted
    pub tx_bytes: u64,
    /// Packets received
    pub rx_packets: u64,
    /// Packets transmitted
    pub tx_packets: u64,
    /// Receive errors
    pub rx_errors: u64,
    /// Transmit errors
    pub tx_errors: u64,
    /// Receive drops
    pub rx_drops: u64,
    /// Transmit drops
    pub tx_drops: u64,
    /// Is interface up
    pub is_up: bool,
}

/// Network metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
struct NetworkMetrics {
    /// Timestamp
    pub timestamp: u64,
    /// Per-interface metrics
    pub interfaces: HashMap<String, NetworkInterfaceMetrics>,
    /// Total bytes received
    pub total_rx_bytes: u64,
    /// Total bytes transmitted
    pub total_tx_bytes: u64,
    /// Total packets received
    pub total_rx_packets: u64,
    /// Total packets transmitted
    pub total_tx_packets: u64,
}

/// Network collector for monitoring network usage
pub struct NetworkCollector {
    /// Collector configuration
    config: CollectorConfig,
    /// Network collector settings
    settings: SystemCollectorSettings,
    /// Last network metrics
    last_metrics: RwLock<Option<NetworkMetrics>>,
    /// Historical metrics (for intervals)
    history: RwLock<Vec<NetworkMetrics>>,
    /// Last collection time
    last_collection: RwLock<Option<Instant>>,
}

impl NetworkCollector {
    /// Create a new network collector
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
    
    /// Collect network metrics
    async fn collect_network_metrics(&self) -> Result<NetworkMetrics> {
        // On Linux, read from /proc/net/dev
        // On other platforms, use platform-specific APIs
        
        // This is a simplified implementation for demonstration
        // A real implementation would use something like netstat or sysinfo crate
        
        #[cfg(target_os = "linux")]
        {
            use std::fs::File;
            use std::io::{BufRead, BufReader};
            
            let file = File::open("/proc/net/dev")?;
            let reader = BufReader::new(file);
            
            let mut interfaces = HashMap::new();
            let mut total_rx_bytes = 0;
            let mut total_tx_bytes = 0;
            let mut total_rx_packets = 0;
            let mut total_tx_packets = 0;
            
            // Skip the first two lines (headers)
            let mut lines = reader.lines();
            let _ = lines.next();
            let _ = lines.next();
            
            for line in lines {
                let line = line?;
                
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() < 2 {
                    continue;
                }
                
                let name = parts[0].trim().to_string();
                let values: Vec<&str> = parts[1].split_whitespace().collect();
                
                if values.len() < 16 {
                    continue;
                }
                
                let rx_bytes = values[0].parse::<u64>().unwrap_or(0);
                let rx_packets = values[1].parse::<u64>().unwrap_or(0);
                let rx_errors = values[2].parse::<u64>().unwrap_or(0);
                let rx_drops = values[3].parse::<u64>().unwrap_or(0);
                
                let tx_bytes = values[8].parse::<u64>().unwrap_or(0);
                let tx_packets = values[9].parse::<u64>().unwrap_or(0);
                let tx_errors = values[10].parse::<u64>().unwrap_or(0);
                let tx_drops = values[11].parse::<u64>().unwrap_or(0);
                
                let interface_metrics = NetworkInterfaceMetrics {
                    name: name.clone(),
                    rx_bytes,
                    tx_bytes,
                    rx_packets,
                    tx_packets,
                    rx_errors,
                    tx_errors,
                    rx_drops,
                    tx_drops,
                    is_up: true, // Simplified
                };
                
                interfaces.insert(name, interface_metrics);
                
                total_rx_bytes += rx_bytes;
                total_tx_bytes += tx_bytes;
                total_rx_packets += rx_packets;
                total_tx_packets += tx_packets;
            }
            
            Ok(NetworkMetrics {
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
                interfaces,
                total_rx_bytes,
                total_tx_bytes,
                total_rx_packets,
                total_tx_packets,
            })
        }
        
        #[cfg(not(target_os = "linux"))]
        {
            // Simplified implementation for non-Linux platforms
            // A real implementation would use platform-specific APIs
            
            let mut interfaces = HashMap::new();
            let mut total_rx_bytes = 0;
            let mut total_tx_bytes = 0;
            let mut total_rx_packets = 0;
            let mut total_tx_packets = 0;
            
            // Simulate a couple of network interfaces
            for name in ["eth0", "lo"] {
                let rx_bytes = rand::random::<u64>() % 10_000_000;
                let tx_bytes = rand::random::<u64>() % 10_000_000;
                let rx_packets = rx_bytes / 1500;
                let tx_packets = tx_bytes / 1500;
                
                let interface_metrics = NetworkInterfaceMetrics {
                    name: name.to_string(),
                    rx_bytes,
                    tx_bytes,
                    rx_packets,
                    tx_packets,
                    rx_errors: rand::random::<u64>() % 100,
                    tx_errors: rand::random::<u64>() % 100,
                    rx_drops: rand::random::<u64>() % 100,
                    tx_drops: rand::random::<u64>() % 100,
                    is_up: true,
                };
                
                interfaces.insert(name.to_string(), interface_metrics);
                
                total_rx_bytes += rx_bytes;
                total_tx_bytes += tx_bytes;
                total_rx_packets += rx_packets;
                total_tx_packets += tx_packets;
            }
            
            Ok(NetworkMetrics {
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
                interfaces,
                total_rx_bytes,
                total_tx_bytes,
                total_rx_packets,
                total_tx_packets,
            })
        }
    }
    
    /// Calculate rates between two metrics collections
    fn calculate_rates(
        &self,
        current: &NetworkMetrics,
        previous: &NetworkMetrics,
        elapsed_seconds: f64,
    ) -> HashMap<String, f64> {
        let mut rates = HashMap::new();
        
        if elapsed_seconds > 0.0 {
            // Calculate total rates
            rates.insert(
                "rx_bytes_per_second".to_string(),
                (current.total_rx_bytes as f64 - previous.total_rx_bytes as f64) / elapsed_seconds,
            );
            
            rates.insert(
                "tx_bytes_per_second".to_string(),
                (current.total_tx_bytes as f64 - previous.total_tx_bytes as f64) / elapsed_seconds,
            );
            
            rates.insert(
                "rx_packets_per_second".to_string(),
                (current.total_rx_packets as f64 - previous.total_rx_packets as f64) / elapsed_seconds,
            );
            
            rates.insert(
                "tx_packets_per_second".to_string(),
                (current.total_tx_packets as f64 - previous.total_tx_packets as f64) / elapsed_seconds,
            );
            
            // Calculate per-interface rates
            for (name, current_iface) in &current.interfaces {
                if let Some(previous_iface) = previous.interfaces.get(name) {
                    rates.insert(
                        format!("interface.{}.rx_bytes_per_second", name),
                        (current_iface.rx_bytes as f64 - previous_iface.rx_bytes as f64) / elapsed_seconds,
                    );
                    
                    rates.insert(
                        format!("interface.{}.tx_bytes_per_second", name),
                        (current_iface.tx_bytes as f64 - previous_iface.tx_bytes as f64) / elapsed_seconds,
                    );
                    
                    rates.insert(
                        format!("interface.{}.rx_packets_per_second", name),
                        (current_iface.rx_packets as f64 - previous_iface.rx_packets as f64) / elapsed_seconds,
                    );
                    
                    rates.insert(
                        format!("interface.{}.tx_packets_per_second", name),
                        (current_iface.tx_packets as f64 - previous_iface.tx_packets as f64) / elapsed_seconds,
                    );
                }
            }
        }
        
        rates
    }
}

#[async_trait]
impl Collector for NetworkCollector {
    fn metadata(&self) -> CollectorMetadata {
        CollectorMetadata {
            id: self.config.id.clone(),
            name: "Network Collector".to_string(),
            description: "Collects network usage metrics".to_string(),
            source_type: "system.network".to_string(),
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
        info!("Initializing network collector");
        
        // Collect initial metrics
        let metrics = self.collect_network_metrics().await?;
        *self.last_metrics.write().await = Some(metrics);
        *self.last_collection.write().await = Some(Instant::now());
        
        Ok(())
    }
    
    async fn collect(&self) -> Result<HashMap<String, MetricValue>> {
        debug!("Collecting network metrics");
        
        // Collect new metrics
        let metrics = self.collect_network_metrics().await?;
        
        // Get previous metrics for rate calculation
        let previous_metrics = self.last_metrics.read().await.clone();
        let previous_collection = self.last_collection.read().await.clone();
        
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
        
        // Add total network metrics
        result.insert("system.network.rx_bytes".to_string(), MetricValue::Counter(metrics.total_rx_bytes));
        result.insert("system.network.tx_bytes".to_string(), MetricValue::Counter(metrics.total_tx_bytes));
        result.insert("system.network.rx_packets".to_string(), MetricValue::Counter(metrics.total_rx_packets));
        result.insert("system.network.tx_packets".to_string(), MetricValue::Counter(metrics.total_tx_packets));
        
        // Add per-interface metrics
        for (name, iface) in &metrics.interfaces {
            result.insert(
                format!("system.network.interface.{}.rx_bytes", name),
                MetricValue::Counter(iface.rx_bytes),
            );
            
            result.insert(
                format!("system.network.interface.{}.tx_bytes", name),
                MetricValue::Counter(iface.tx_bytes),
            );
            
            result.insert(
                format!("system.network.interface.{}.rx_packets", name),
                MetricValue::Counter(iface.rx_packets),
            );
            
            result.insert(
                format!("system.network.interface.{}.tx_packets", name),
                MetricValue::Counter(iface.tx_packets),
            );
            
            result.insert(
                format!("system.network.interface.{}.rx_errors", name),
                MetricValue::Counter(iface.rx_errors),
            );
            
            result.insert(
                format!("system.network.interface.{}.tx_errors", name),
                MetricValue::Counter(iface.tx_errors),
            );
            
            result.insert(
                format!("system.network.interface.{}.rx_drops", name),
                MetricValue::Counter(iface.rx_drops),
            );
            
            result.insert(
                format!("system.network.interface.{}.tx_drops", name),
                MetricValue::Counter(iface.tx_drops),
            );
            
            result.insert(
                format!("system.network.interface.{}.is_up", name),
                MetricValue::Gauge(if iface.is_up { 1.0 } else { 0.0 }),
            );
        }
        
        // Calculate rates if we have previous metrics
        if let (Some(prev_metrics), Some(prev_collection)) = (previous_metrics, previous_collection) {
            let elapsed = Instant::now().duration_since(prev_collection).as_secs_f64();
            let rates = self.calculate_rates(&metrics, &prev_metrics, elapsed);
            
            for (key, value) in rates {
                result.insert(format!("system.network.{}", key), MetricValue::Gauge(value));
            }
        }
        
        Ok(result)
    }
    
    async fn shutdown(&self) -> Result<()> {
        debug!("Shutting down network collector");
        Ok(())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Factory for creating network collectors
pub struct NetworkCollectorFactory {
    metadata: CollectorMetadata,
}

impl NetworkCollectorFactory {
    /// Create a new network collector factory
    pub fn new() -> Self {
        Self {
            metadata: CollectorMetadata {
                id: "system.network".to_string(),
                name: "Network Collector".to_string(),
                description: "Collects network usage metrics".to_string(),
                source_type: "system.network".to_string(),
                enabled_by_default: true,
            },
        }
    }
}

#[async_trait]
impl CollectorFactory for NetworkCollectorFactory {
    async fn create(&self, config: CollectorConfig) -> Result<Box<dyn Collector>> {
        let collector = NetworkCollector::new(config)?;
        Ok(Box::new(collector))
    }
    
    fn metadata(&self) -> CollectorMetadata {
        self.metadata.clone()
    }
}