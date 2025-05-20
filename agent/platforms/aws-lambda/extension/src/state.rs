// agent/platforms/aws-lambda/extension/src/state.rs

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use lambda_extension::{LambdaEvent, NextEvent};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

use agent_core::state::cold_start::{is_cold_start, lambda::is_lambda_cold_start};
use agent_core::state::persistence::{PersistentState, StatePersistence};
use agent_core::state::memory_pool::{MemoryPool, TypedMemoryPool};

// Lambda-specific persistent state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LambdaState {
    pub function_name: String,
    pub function_version: String,
    pub function_memory_mb: u32,
    pub region: String,
    pub account_id: String,
    pub cold_starts: u64,
    pub invoke_count: u64,
    pub last_invoke_time: Option<DateTime<Utc>>,
    pub errors: u64,
    pub avg_duration_ms: f64,
    pub max_duration_ms: u64,
    pub version: u32,
    pub extra_data: HashMap<String, String>,
    pub creation_time: DateTime<Utc>,
    pub metrics_endpoint: Option<String>,
    pub log_level: String,
}

impl Default for LambdaState {
    fn default() -> Self {
        let function_name = std::env::var("AWS_LAMBDA_FUNCTION_NAME")
            .unwrap_or_else(|_| "unknown".to_string());
            
        let function_version = std::env::var("AWS_LAMBDA_FUNCTION_VERSION")
            .unwrap_or_else(|_| "$LATEST".to_string());
            
        let function_memory_mb = std::env::var("AWS_LAMBDA_FUNCTION_MEMORY_SIZE")
            .unwrap_or_else(|_| "128".to_string())
            .parse::<u32>()
            .unwrap_or(128);
            
        let region = std::env::var("AWS_REGION")
            .unwrap_or_else(|_| "us-east-1".to_string());
            
        let account_id = "unknown".to_string(); // Requires additional API call
        
        Self {
            function_name,
            function_version,
            function_memory_mb,
            region,
            account_id,
            cold_starts: 1, // This is a default, so it's a cold start
            invoke_count: 0,
            last_invoke_time: None,
            errors: 0,
            avg_duration_ms: 0.0,
            max_duration_ms: 0,
            version: 1,
            extra_data: HashMap::new(),
            creation_time: Utc::now(),
            metrics_endpoint: None,
            log_level: "info".to_string(),
        }
    }
}

impl PersistentState for LambdaState {
    fn state_id() -> &'static str {
        "lambda-extension"
    }
    
    fn validate(&self) -> Result<()> {
        if self.version < 1 {
            return Err(anyhow::anyhow!("Invalid state version"));
        }
        Ok(())
    }
}

// Runtime state snapshot for current invocation
#[derive(Debug, Clone)]
pub struct StateSnapshot {
    pub request_id: String,
    pub start_time: Instant,
    pub cold_start: bool,
    pub memory_usage_mb: u64,
    pub cpu_usage_percent: f64,
}

// Lambda state manager
pub struct LambdaStateManager {
    state: LambdaState,
    persistence: StatePersistence<LambdaState>,
    current_request: Option<StateSnapshot>,
    shared_memory: Option<SharedMemory>,
}

impl LambdaStateManager {
    // Create a new state manager
    pub fn new() -> Result<Self> {
        // Initialize persistence
        let persistence = StatePersistence::<LambdaState>::new(
            None, // Use default directory
            None, // Use default state file
            None, // Use default backup file
        );
        
        // Load existing state or create new
        let mut state = persistence.load()?;
        
        // Check if this is a cold start
        if is_lambda_cold_start() {
            state.cold_starts += 1;
        }
        
        // Try to initialize shared memory
        let shared_memory = match SharedMemory::new() {
            Ok(mem) => Some(mem),
            Err(e) => {
                warn!("Failed to initialize shared memory: {}", e);
                None
            }
        };
        
        Ok(Self {
            state,
            persistence,
            current_request: None,
            shared_memory,
        })
    }
    
    // Record an invoke event
    pub fn record_invoke(&mut self, request_id: &str) {
        self.state.invoke_count += 1;
        self.state.last_invoke_time = Some(Utc::now());
        
        // Create snapshot of current state
        let snapshot = StateSnapshot {
            request_id: request_id.to_string(),
            start_time: Instant::now(),
            cold_start: is_cold_start(),
            memory_usage_mb: self.get_memory_usage().unwrap_or(0),
            cpu_usage_percent: self.get_cpu_usage().unwrap_or(0.0),
        };
        
        self.current_request = Some(snapshot);
        
        // Save state after updating
        if let Err(e) = self.persistence.save(&self.state) {
            error!("Failed to save state: {}", e);
        }
    }
    
    // Record response
    pub fn record_response(&mut self, duration_ms: u64, status_code: u16) {
        // Update average duration
        let old_avg = self.state.avg_duration_ms;
        let old_count = self.state.invoke_count - 1; // We already incremented in record_invoke
        
        self.state.avg_duration_ms = (old_avg * old_count as f64 + duration_ms as f64) / 
            self.state.invoke_count as f64;
            
        // Update max duration
        if duration_ms > self.state.max_duration_ms {
            self.state.max_duration_ms = duration_ms;
        }
        
        // Clear current request
        self.current_request = None;
        
        // Save state after updating
        if let Err(e) = self.persistence.save(&self.state) {
            error!("Failed to save state: {}", e);
        }
    }
    
    // Record error
    pub fn record_error(&mut self, error_type: &str) {
        self.state.errors += 1;
        
        // Add error to extra data for tracking
        let key = format!("last_error_{}", error_type);
        self.state.extra_data.insert(key, Utc::now().to_rfc3339());
        
        // Save state after updating
        if let Err(e) = self.persistence.save(&self.state) {
            error!("Failed to save state: {}", e);
        }
    }
    
    // Get current memory usage
    pub fn get_memory_usage(&self) -> Result<u64> {
        let mut usage = 0u64;
        
        // Try to use procfs first
        if let Ok(memory) = std::fs::read_to_string("/proc/self/statm") {
            let parts: Vec<&str> = memory.split_whitespace().collect();
            if parts.len() >= 2 {
                if let Ok(rss_pages) = parts[1].parse::<u64>() {
                    // Convert pages to MB
                    usage = rss_pages * 4096 / 1024 / 1024; // 4KB page size
                }
            }
        }
        
        // Fallback to shared memory if available
        if usage == 0 {
            if let Some(ref shared_mem) = self.shared_memory {
                usage = shared_mem.get_memory_usage_mb()?;
            }
        }
        
        Ok(usage)
    }
    
    // Get current CPU usage
    pub fn get_cpu_usage(&self) -> Result<f64> {
        let mut usage = 0.0;
        
        // Try to use procfs first
        if let Ok(stat) = std::fs::read_to_string("/proc/self/stat") {
            let parts: Vec<&str> = stat.split_whitespace().collect();
            if parts.len() >= 15 {
                if let (Ok(utime), Ok(stime)) = (parts[13].parse::<u64>(), parts[14].parse::<u64>()) {
                    // Calculate CPU usage
                    let total_time = utime + stime;
                    let uptime = std::fs::read_to_string("/proc/uptime")
                        .ok()
                        .and_then(|s| s.split_whitespace().next())
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.0);
                        
                    if uptime > 0.0 {
                        let cores = num_cpus::get() as f64;
                        usage = 100.0 * (total_time as f64 / sysconf::page::pagesize() as f64) / (uptime * cores);
                    }
                }
            }
        }
        
        // Fallback to shared memory if available
        if usage == 0.0 {
            if let Some(ref shared_mem) = self.shared_memory {
                usage = shared_mem.get_cpu_usage_percent()?;
            }
        }
        
        Ok(usage)
    }
    
    // Get current state
    pub fn get_state(&self) -> &LambdaState {
        &self.state
    }
    
    // Get current request snapshot
    pub fn get_current_request(&self) -> Option<&StateSnapshot> {
        self.current_request.as_ref()
    }
    
    // Set metrics endpoint
    pub fn set_metrics_endpoint(&mut self, endpoint: String) {
        self.state.metrics_endpoint = Some(endpoint);
        
        // Save state after updating
        if let Err(e) = self.persistence.save(&self.state) {
            error!("Failed to save state: {}", e);
        }
    }
    
    // Process Extension events
    pub fn process_event(&mut self, event: &LambdaEvent) {
        match &event.next {
            NextEvent::Shutdown(shutdown) => {
                info!("Received shutdown event: {}", shutdown.shutdown_reason);
                
                // Save state before shutting down
                if let Err(e) = self.persistence.save(&self.state) {
                    error!("Failed to save state before shutdown: {}", e);
                }
            }
            NextEvent::Invoke(invoke) => {
                debug!("Received invoke event for request {}", invoke.request_id);
                self.record_invoke(&invoke.request_id);
            }
        }
    }
}

// Shared memory implementation for communicating with the runtime
pub struct SharedMemory {
    memory_pool: TypedMemoryPool<SharedMemoryData>,
    allocation: Option<Box<dyn std::any::Any>>, // Store the allocation to keep it alive
}

// Data structure for shared memory
#[repr(C)]
pub struct SharedMemoryData {
    memory_usage_mb: u64,
    cpu_usage_percent: f64,
    request_id: [u8; 64],
    is_cold_start: u8,
    last_updated: u64, // timestamp
}

impl SharedMemory {
    // Create new shared memory region
    pub fn new() -> Result<Self> {
        // Create memory pool for 1 item
        let memory_pool = TypedMemoryPool::<SharedMemoryData>::new(1)
            .context("Failed to create memory pool")?;
            
        // Allocate data structure
        let mut allocation = memory_pool.allocate()
            .context("Failed to allocate shared memory")?;
            
        // Initialize with defaults
        allocation.initialize(SharedMemoryData {
            memory_usage_mb: 0,
            cpu_usage_percent: 0.0,
            request_id: [0; 64],
            is_cold_start: 1,
            last_updated: chrono::Utc::now().timestamp() as u64,
        });
        
        // Box the allocation to keep it alive as long as the SharedMemory instance exists
        let allocation_box = Box::new(allocation) as Box<dyn std::any::Any>;
        
        Ok(Self {
            memory_pool,
            allocation: Some(allocation_box),
        })
    }
    
    // Get memory usage from shared memory
    pub fn get_memory_usage_mb(&self) -> Result<u64> {
        let ptr = self.memory_pool.pool.allocate(std::mem::size_of::<SharedMemoryData>())?;
        let data = unsafe { &*(ptr.as_ptr() as *const SharedMemoryData) };
        Ok(data.memory_usage_mb)
    }
    
    // Get CPU usage from shared memory
    pub fn get_cpu_usage_percent(&self) -> Result<f64> {
        let ptr = self.memory_pool.pool.allocate(std::mem::size_of::<SharedMemoryData>())?;
        let data = unsafe { &*(ptr.as_ptr() as *const SharedMemoryData) };
        Ok(data.cpu_usage_percent)
    }
    
    // Update shared memory with current metrics
    pub fn update_metrics(&self, memory_mb: u64, cpu_percent: f64, request_id: &str) -> Result<()> {
        let ptr = self.memory_pool.pool.allocate(std::mem::size_of::<SharedMemoryData>())?;
        let data = unsafe { &mut *(ptr.as_ptr() as *mut SharedMemoryData) };
        
        data.memory_usage_mb = memory_mb;
        data.cpu_usage_percent = cpu_percent;
        
        // Copy request ID into fixed-size buffer
        let bytes = request_id.as_bytes();
        let copy_len = bytes.len().min(data.request_id.len() - 1); // Leave room for null terminator
        data.request_id[..copy_len].copy_from_slice(&bytes[..copy_len]);
        data.request_id[copy_len] = 0; // Null terminator
        
        data.is_cold_start = if is_cold_start() { 1 } else { 0 };
        data.last_updated = chrono::Utc::now().timestamp() as u64;
        
        Ok(())
    }
}
