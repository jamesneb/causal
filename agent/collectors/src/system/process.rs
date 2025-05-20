// agent/collectors/src/system/process.rs

use std::any::Any;
use std::collections::{HashMap, BTreeMap};
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

/// Process metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProcessMetrics {
    /// Process ID
    pub pid: u32,
    /// Process name
    pub name: String,
    /// Command line
    pub cmdline: Option<String>,
    /// User ID
    pub uid: Option<u32>,
    /// CPU usage (0.0-1.0)
    pub cpu_usage: f64,
    /// Memory usage in bytes
    pub memory_bytes: u64,
    /// Virtual memory in bytes
    pub virtual_memory_bytes: Option<u64>,
    /// Resident set size in bytes
    pub rss_bytes: Option<u64>,
    /// Number of threads
    pub threads: Option<u32>,
    /// Open file descriptors
    pub open_fds: Option<u32>,
    /// Process start time
    pub start_time: Option<u64>,
    /// Process state
    pub state: Option<String>,
}

/// System process metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SystemProcessMetrics {
    /// Timestamp
    pub timestamp: u64,
    /// Total number of processes
    pub total_processes: u32,
    /// Total number of threads
    pub total_threads: Option<u32>,
    /// Number of running processes
    pub running_processes: Option<u32>,
    /// Number of blocked processes
    pub blocked_processes: Option<u32>,
    /// Number of sleeping processes
    pub sleeping_processes: Option<u32>,
    /// Number of zombie processes
    pub zombie_processes: Option<u32>,
    /// Per-process metrics
    pub processes: BTreeMap<u32, ProcessMetrics>,
}

/// Process collector for monitoring processes
pub struct ProcessCollector {
    /// Collector configuration
    config: CollectorConfig,
    /// Process collector settings
    settings: SystemCollectorSettings,
    /// Last process metrics
    last_metrics: RwLock<Option<SystemProcessMetrics>>,
    /// Historical metrics (for intervals)
    history: RwLock<Vec<SystemProcessMetrics>>,
    /// Last collection time
    last_collection: RwLock<Option<Instant>>,
    /// Process filter (PIDs to include)
    process_filter: Option<Vec<u32>>,
}

impl ProcessCollector {
    /// Create a new process collector
    pub fn new(config: CollectorConfig) -> Result<Self> {
        let default_settings = SystemCollectorSettings::default();
        let settings = merge_system_settings(&default_settings, config.settings.as_ref())?;
        
        // Check for process filter in config
        let mut process_filter = None;
        if let Some(cfg) = &config.settings {
            if let Some(pids) = cfg.get("process_filter") {
                process_filter = Some(
                    pids.split(',')
                        .filter_map(|s| s.trim().parse::<u32>().ok())
                        .collect(),
                );
            }
        }
        
        Ok(Self {
            config,
            settings,
            last_metrics: RwLock::new(None),
            history: RwLock::new(Vec::new()),
            last_collection: RwLock::new(None),
            process_filter,
        })
    }
    
    /// Collect process metrics
    async fn collect_process_metrics(&self) -> Result<SystemProcessMetrics> {
        // On Linux, read from /proc
        // On other platforms, use platform-specific APIs
        
        // This is a simplified implementation for demonstration
        // A real implementation would use something like sysinfo or procfs crate
        
        #[cfg(target_os = "linux")]
        {
            use std::fs::{self, File};
            use std::io::{BufRead, BufReader};
            use std::path::Path;
            
            let proc_dir = Path::new("/proc");
            let mut processes = BTreeMap::new();
            
            let mut total_processes = 0;
            let mut total_threads = 0;
            let mut running_processes = 0;
            let mut blocked_processes = 0;
            let mut sleeping_processes = 0;
            let mut zombie_processes = 0;
            
            // Read overall process stats
            if let Ok(file) = File::open("/proc/stat") {
                let reader = BufReader::new(file);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        if line.starts_with("processes ") {
                            if let Some(count) = line.split_whitespace().nth(1) {
                                total_processes = count.parse::<u32>().unwrap_or(0);
                            }
                        } else if line.starts_with("procs_running ") {
                            if let Some(count) = line.split_whitespace().nth(1) {
                                running_processes = count.parse::<u32>().unwrap_or(0);
                            }
                        } else if line.starts_with("procs_blocked ") {
                            if let Some(count) = line.split_whitespace().nth(1) {
                                blocked_processes = count.parse::<u32>().unwrap_or(0);
                            }
                        }
                    }
                }
            }
            
            // Iterate through process directories
            if let Ok(entries) = fs::read_dir(proc_dir) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        
                        // Check if this is a process directory (numeric name)
                        if let Some(name) = path.file_name() {
                            if let Some(name_str) = name.to_str() {
                                if let Ok(pid) = name_str.parse::<u32>() {
                                    // Check if we should include this process
                                    if let Some(filter) = &self.process_filter {
                                        if !filter.contains(&pid) {
                                            continue;
                                        }
                                    }
                                    
                                    let stat_path = path.join("stat");
                                    let status_path = path.join("status");
                                    let cmdline_path = path.join("cmdline");
                                    
                                    let mut process_name = String::new();
                                    let mut cmdline = None;
                                    let mut uid = None;
                                    let mut cpu_usage = 0.0;
                                    let mut memory_bytes = 0;
                                    let mut virtual_memory_bytes = None;
                                    let mut rss_bytes = None;
                                    let mut threads = None;
                                    let mut open_fds = None;
                                    let mut start_time = None;
                                    let mut state = None;
                                    
                                    // Read process name and state from stat
                                    if let Ok(file) = File::open(&stat_path) {
                                        let reader = BufReader::new(file);
                                        if let Some(Ok(line)) = reader.lines().next() {
                                            let parts: Vec<&str> = line.split_whitespace().collect();
                                            if parts.len() > 2 {
                                                // Process name is in the form (name)
                                                if let Some(name_start) = line.find('(') {
                                                    if let Some(name_end) = line[name_start..].find(')') {
                                                        process_name = line[name_start + 1..name_start + name_end].to_string();
                                                    }
                                                }
                                                
                                                // Process state
                                                if parts.len() > 3 {
                                                    state = Some(parts[2].to_string());
                                                    
                                                    match parts[2] {
                                                        "R" => running_processes += 1,
                                                        "D" => blocked_processes += 1,
                                                        "S" | "I" => sleeping_processes += 1,
                                                        "Z" => zombie_processes += 1,
                                                        _ => {}
                                                    }
                                                }
                                                
                                                // Threads count
                                                if parts.len() > 19 {
                                                    if let Ok(thread_count) = parts[19].parse::<u32>() {
                                                        threads = Some(thread_count);
                                                        total_threads += thread_count;
                                                    }
                                                }
                                                
                                                // Start time
                                                if parts.len() > 21 {
                                                    if let Ok(time) = parts[21].parse::<u64>() {
                                                        start_time = Some(time);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    
                                    // Read additional info from status
                                    if let Ok(file) = File::open(&status_path) {
                                        let reader = BufReader::new(file);
                                        for line in reader.lines() {
                                            if let Ok(line) = line {
                                                if line.starts_with("Uid:") {
                                                    if let Some(uid_str) = line.split_whitespace().nth(1) {
                                                        uid = uid_str.parse::<u32>().ok();
                                                    }
                                                } else if line.starts_with("VmSize:") {
                                                    if let Some(vm_str) = line.split_whitespace().nth(1) {
                                                        virtual_memory_bytes = vm_str.parse::<u64>().ok().map(|kb| kb * 1024);
                                                    }
                                                } else if line.starts_with("VmRSS:") {
                                                    if let Some(rss_str) = line.split_whitespace().nth(1) {
                                                        rss_bytes = rss_str.parse::<u64>().ok().map(|kb| kb * 1024);
                                                        memory_bytes = rss_bytes.unwrap_or(0);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    
                                    // Read command line
                                    if let Ok(mut file) = File::open(&cmdline_path) {
                                        use std::io::Read;
                                        let mut buffer = Vec::new();
                                        if file.read_to_end(&mut buffer).is_ok() {
                                            // Replace null bytes with spaces
                                            for byte in buffer.iter_mut() {
                                                if *byte == 0 {
                                                    *byte = b' ';
                                                }
                                            }
                                            
                                            if !buffer.is_empty() {
                                                cmdline = String::from_utf8_lossy(&buffer).to_string().into();
                                            }
                                        }
                                    }
                                    
                                    // Count open file descriptors
                                    if let Ok(entries) = fs::read_dir(path.join("fd")) {
                                        open_fds = Some(entries.count() as u32);
                                    }
                                    
                                    let process_metrics = ProcessMetrics {
                                        pid,
                                        name: process_name,
                                        cmdline,
                                        uid,
                                        cpu_usage, // Will be calculated later
                                        memory_bytes,
                                        virtual_memory_bytes,
                                        rss_bytes,
                                        threads,
                                        open_fds,
                                        start_time,
                                        state,
                                    };
                                    
                                    processes.insert(pid, process_metrics);
                                }
                            }
                        }
                    }
                }
            }
            
            Ok(SystemProcessMetrics {
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
                total_processes,
                total_threads: Some(total_threads),
                running_processes: Some(running_processes),
                blocked_processes: Some(blocked_processes),
                sleeping_processes: Some(sleeping_processes),
                zombie_processes: Some(zombie_processes),
                processes,
            })
        }
        
        #[cfg(not(target_os = "linux"))]
        {
            // Simplified implementation for non-Linux platforms
            // A real implementation would use platform-specific APIs
            
            let mut processes = BTreeMap::new();
            let total_processes = 100;
            let total_threads = 500;
            
            // Generate some sample processes
            for pid in 1..20 {
                let process_metrics = ProcessMetrics {
                    pid,
                    name: format!("process-{}", pid),
                    cmdline: Some(format!("/usr/bin/process-{} --arg1 --arg2", pid)),
                    uid: Some(1000),
                    cpu_usage: rand::random::<f64>() * 0.2,
                    memory_bytes: rand::random::<u64>() % 1_000_000_000,
                    virtual_memory_bytes: Some(rand::random::<u64>() % 2_000_000_000),
                    rss_bytes: Some(rand::random::<u64>() % 1_000_000_000),
                    threads: Some(rand::random::<u32>() % 10 + 1),
                    open_fds: Some(rand::random::<u32>() % 100),
                    start_time: Some(
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs() as u64 - rand::random::<u64>() % 86400,
                    ),
                    state: Some("S".to_string()),
                };
                
                processes.insert(pid, process_metrics);
            }
            
            Ok(SystemProcessMetrics {
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64,
                total_processes,
                total_threads: Some(total_threads),
                running_processes: Some(5),
                blocked_processes: Some(2),
                sleeping_processes: Some(90),
                zombie_processes: Some(3),
                processes,
            })
        }
    }
    
    /// Calculate CPU usage based on previous metrics
    fn calculate_cpu_usage(
        &self,
        current: &SystemProcessMetrics,
        previous: &SystemProcessMetrics,
        elapsed_seconds: f64,
    ) -> SystemProcessMetrics {
        let mut updated = current.clone();
        
        if elapsed_seconds > 0.0 {
            for (pid, process) in &mut updated.processes {
                if let Some(prev_process) = previous.processes.get(pid) {
                    if let (Some(current_time), Some(prev_time)) = (process.start_time, prev_process.start_time) {
                        if current_time == prev_time {
                            // Same process, calculate CPU usage
                            // In a real implementation, we would read CPU time from /proc/[pid]/stat
                            // and calculate the difference
                            
                            // For now, use a simplified model
                            process.cpu_usage = prev_process.cpu_usage * 0.8 + rand::random::<f64>() * 0.2;
                        }
                    }
                }
            }
        }
        
        updated
    }
}

#[async_trait]
impl Collector for ProcessCollector {
    fn metadata(&self) -> CollectorMetadata {
        CollectorMetadata {
            id: self.config.id.clone(),
            name: "Process Collector".to_string(),
            description: "Collects process metrics".to_string(),
            source_type: "system.process".to_string(),
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
            
            // Check for process filter update
            if let Some(pids) = settings.get("process_filter") {
                self.process_filter = Some(
                    pids.split(',')
                        .filter_map(|s| s.trim().parse::<u32>().ok())
                        .collect(),
                );
            } else {
                self.process_filter = None;
            }
        }
        
        self.config = config;
        Ok(())
    }
    
    async fn initialize(&mut self) -> Result<()> {
        info!("Initializing process collector");
        
        // Collect initial metrics
        let metrics = self.collect_process_metrics().await?;
        *self.last_metrics.write().await = Some(metrics);
        *self.last_collection.write().await = Some(Instant::now());
        
        Ok(())
    }
    
    async fn collect(&self) -> Result<HashMap<String, MetricValue>> {
        debug!("Collecting process metrics");
        
        // Collect new metrics
        let metrics = self.collect_process_metrics().await?;
        
        // Get previous metrics for CPU usage calculation
        let previous_metrics = self.last_metrics.read().await.clone();
        let previous_collection = self.last_collection.read().await.clone();
        
        // Calculate CPU usage if we have previous metrics
        let metrics = if let (Some(prev_metrics), Some(prev_collection)) = (previous_metrics, previous_collection) {
            let elapsed = Instant::now().duration_since(prev_collection).as_secs_f64();
            self.calculate_cpu_usage(&metrics, &prev_metrics, elapsed)
        } else {
            metrics
        };
        
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
        
        // Add overall process metrics
        result.insert("system.process.total".to_string(), MetricValue::Gauge(metrics.total_processes as f64));
        
        if let Some(threads) = metrics.total_threads {
            result.insert("system.process.threads".to_string(), MetricValue::Gauge(threads as f64));
        }
        
        if let Some(running) = metrics.running_processes {
            result.insert("system.process.running".to_string(), MetricValue::Gauge(running as f64));
        }
        
        if let Some(blocked) = metrics.blocked_processes {
            result.insert("system.process.blocked".to_string(), MetricValue::Gauge(blocked as f64));
        }
        
        if let Some(sleeping) = metrics.sleeping_processes {
            result.insert("system.process.sleeping".to_string(), MetricValue::Gauge(sleeping as f64));
        }
        
        if let Some(zombie) = metrics.zombie_processes {
            result.insert("system.process.zombie".to_string(), MetricValue::Gauge(zombie as f64));
        }
        
        // Add per-process metrics if enabled
        if self.settings.collect_per_process {
            // Sort processes by CPU usage
            let mut sorted_processes: Vec<_> = metrics.processes.values().collect();
            sorted_processes.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap_or(std::cmp::Ordering::Equal));
            
            // Take top N processes
            let top_processes = sorted_processes.iter().take(self.settings.max_processes);
            
            for process in top_processes {
                let prefix = format!("system.process.{}", process.pid);
                
                result.insert(format!("{}.cpu", prefix), MetricValue::Gauge(process.cpu_usage));
                result.insert(format!("{}.memory_bytes", prefix), MetricValue::Gauge(process.memory_bytes as f64));
                
                if let Some(threads) = process.threads {
                    result.insert(format!("{}.threads", prefix), MetricValue::Gauge(threads as f64));
                }
                
                if let Some(fds) = process.open_fds {
                    result.insert(format!("{}.open_fds", prefix), MetricValue::Gauge(fds as f64));
                }
                
                if let Some(virt_mem) = process.virtual_memory_bytes {
                    result.insert(format!("{}.virtual_memory_bytes", prefix), MetricValue::Gauge(virt_mem as f64));
                }
                
                if let Some(rss) = process.rss_bytes {
                    result.insert(format!("{}.rss_bytes", prefix), MetricValue::Gauge(rss as f64));
                }
                
                // Add process name as a set
                result.insert(
                    format!("{}.name", prefix),
                    MetricValue::Set(vec![process.name.clone()]),
                );
                
                // Add cmdline as a set if available
                if let Some(cmdline) = &process.cmdline {
                    result.insert(
                        format!("{}.cmdline", prefix),
                        MetricValue::Set(vec![cmdline.clone()]),
                    );
                }
            }
        }
        
        Ok(result)
    }
    
    async fn shutdown(&self) -> Result<()> {
        debug!("Shutting down process collector");
        Ok(())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Factory for creating process collectors
pub struct ProcessCollectorFactory {
    metadata: CollectorMetadata,
}

impl ProcessCollectorFactory {
    /// Create a new process collector factory
    pub fn new() -> Self {
        Self {
            metadata: CollectorMetadata {
                id: "system.process".to_string(),
                name: "Process Collector".to_string(),
                description: "Collects process metrics".to_string(),
                source_type: "system.process".to_string(),
                enabled_by_default: true,
            },
        }
    }
}

#[async_trait]
impl CollectorFactory for ProcessCollectorFactory {
    async fn create(&self, config: CollectorConfig) -> Result<Box<dyn Collector>> {
        let collector = ProcessCollector::new(config)?;
        Ok(Box::new(collector))
    }
    
    fn metadata(&self) -> CollectorMetadata {
        self.metadata.clone()
    }
}