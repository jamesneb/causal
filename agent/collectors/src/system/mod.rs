// agent/collectors/src/system/mod.rs

//! System metrics collectors for monitoring system resources

pub mod cpu;
pub mod memory;
pub mod network;
pub mod process;

use std::collections::HashMap;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Common settings for system metrics collectors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemCollectorSettings {
    /// Collection interval in seconds
    pub interval_seconds: u64,
    /// Whether to collect per-core CPU metrics
    pub collect_per_core: bool,
    /// Whether to collect per-process metrics
    pub collect_per_process: bool,
    /// Maximum number of processes to monitor (by highest resource usage)
    pub max_processes: usize,
    /// Whether to collect detailed metrics
    pub collect_detailed: bool,
}

impl Default for SystemCollectorSettings {
    fn default() -> Self {
        Self {
            interval_seconds: 60,
            collect_per_core: true,
            collect_per_process: false,
            max_processes: 10,
            collect_detailed: false,
        }
    }
}

/// Helper function to merge collector settings from config
pub fn merge_system_settings(
    defaults: &SystemCollectorSettings,
    config_settings: Option<&HashMap<String, String>>,
) -> Result<SystemCollectorSettings> {
    let mut settings = defaults.clone();
    
    if let Some(cfg) = config_settings {
        // Process interval_seconds
        if let Some(interval) = cfg.get("interval_seconds") {
            if let Ok(seconds) = interval.parse::<u64>() {
                settings.interval_seconds = seconds;
            }
        }
        
        // Process collect_per_core
        if let Some(per_core) = cfg.get("collect_per_core") {
            settings.collect_per_core = per_core == "true" || per_core == "1";
        }
        
        // Process collect_per_process
        if let Some(per_process) = cfg.get("collect_per_process") {
            settings.collect_per_process = per_process == "true" || per_process == "1";
        }
        
        // Process max_processes
        if let Some(max_procs) = cfg.get("max_processes") {
            if let Ok(count) = max_procs.parse::<usize>() {
                settings.max_processes = count;
            }
        }
        
        // Process collect_detailed
        if let Some(detailed) = cfg.get("collect_detailed") {
            settings.collect_detailed = detailed == "true" || detailed == "1";
        }
    }
    
    Ok(settings)
}