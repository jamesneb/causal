}use anyhow::Result;
use serde_json::{json, Value};
use std::env;
use std::sync::OnceLock;

// Cache environment variables to avoid repeated lookups
static CACHED_CONFIG: OnceLock<Value> = OnceLock::new();

pub fn get_config() -> Result<Value> {
    // Use cached config if available (thread-safe, zero-cost after init)
    if let Some(config) = CACHED_CONFIG.get() {
        return Ok(config.clone());
    }
    
    // Pre-allocate strings to avoid repeated small allocations
    let endpoint = env::var("TELEMETRY_ENDPOINT")
        .unwrap_or_else(|_| "https://example.com/telemetry".to_string());
    
    // Parse with proper error handling
    let sampling_rate = env::var("SAMPLING_RATE")
        .unwrap_or_else(|_| "1.0".to_string())
        .parse::<f64>()
        .unwrap_or(1.0);
        
    // Get batch size for efficient transfers
    let batch_size = env::var("BATCH_SIZE")
        .unwrap_or_else(|_| "10".to_string())
        .parse::<u64>()
        .unwrap_or(10);
    
    // Get Lambda function configuration (do this once at startup)
    let function_name = env::var("AWS_LAMBDA_FUNCTION_NAME").unwrap_or_default();
    let function_version = env::var("AWS_LAMBDA_FUNCTION_VERSION").unwrap_or_default();
    let memory_size = env::var("AWS_LAMBDA_FUNCTION_MEMORY_SIZE").unwrap_or_default();
    let region = env::var("AWS_REGION").unwrap_or_default();
    
    // Create configuration object with capacity hint
    let config = json!({
        "telemetry": {
            "endpoint": endpoint,
            "sampling_rate": sampling_rate,
            "batch_size": batch_size
        },
        "function": {
            "name": function_name,
            "version": function_version,
            "memory_size": memory_size,
            "region": region
        }
    });
    
    // Cache the config for future calls
    let _ = CACHED_CONFIG.set(config.clone());
    
    Ok(config)
}
