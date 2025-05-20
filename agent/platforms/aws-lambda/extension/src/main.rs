// agent/platforms/aws-lambda/extension/src/main.rs

use anyhow::{Context, Result};
use lambda_extension::{Extension, LambdaEvent, NextEvent, SharedService, service_fn};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};
use tracing_subscriber::{filter::LevelFilter, EnvFilter, Registry};
use tracing_subscriber::layer::SubscriberExt;

mod state;
mod telemetry;

use state::{LambdaStateManager, is_cold_start};
use telemetry::LambdaTelemetryExtension;

// Configuration constants
const DEFAULT_METRICS_ENDPOINT: &str = "https://metrics.example.com/v1/lambda";
const DEFAULT_BATCH_SIZE: usize = 50;
const DEFAULT_FLUSH_INTERVAL_SECS: u64 = 5;

// Environment variables
const ENV_METRICS_ENDPOINT: &str = "METRICS_ENDPOINT";
const ENV_BATCH_SIZE: &str = "METRICS_BATCH_SIZE";
const ENV_FLUSH_INTERVAL_SECS: &str = "METRICS_FLUSH_INTERVAL_SECS";
const ENV_LOG_LEVEL: &str = "LOG_LEVEL";

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let log_level = std::env::var(ENV_LOG_LEVEL)
        .unwrap_or_else(|_| "info".to_string())
        .parse::<LevelFilter>()
        .unwrap_or(LevelFilter::INFO);
        
    let env_filter = EnvFilter::builder()
        .with_default_directive(log_level.into())
        .from_env_lossy();
        
    let subscriber = Registry::default()
        .with(env_filter)
        .with(tracing_subscriber::fmt::Layer::default());
        
    tracing::subscriber::set_global_default(subscriber)
        .context("Failed to set global default subscriber")?;
    
    info!("Lambda extension starting up");
    
    // Initialize state manager
    let state_manager = Arc::new(Mutex::new(
        LambdaStateManager::new().context("Failed to initialize state manager")?
    ));
    
    // Get configuration from environment
    let metrics_endpoint = std::env::var(ENV_METRICS_ENDPOINT)
        .unwrap_or_else(|_| DEFAULT_METRICS_ENDPOINT.to_string());
        
    let batch_size = std::env::var(ENV_BATCH_SIZE)
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(DEFAULT_BATCH_SIZE);
        
    let flush_interval = std::env::var(ENV_FLUSH_INTERVAL_SECS)
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(DEFAULT_FLUSH_INTERVAL_SECS);
    
    // Register the extension
    let extension_client = lambda_extension::client::ExtensionClient::new()
        .context("Failed to create Lambda extension client")?;
        
    let extension_id = extension_client.register().await
        .context("Failed to register extension")?;
        
    info!("Registered extension with ID: {}", extension_id);
    
    // Initialize telemetry
    let telemetry = Arc::new(
        LambdaTelemetryExtension::new(
            extension_id.clone(),
            metrics_endpoint,
            batch_size,
            state_manager.clone(),
            Duration::from_secs(flush_interval),
        )
    );
    
    // Start auto-flush
    telemetry.start_auto_flush();
    
    // Log cold start metrics
    if is_cold_start() {
        info!("Cold start detected, logging initialization metrics");
        
        let cold_start_duration = agent_core::state::cold_start::time_since_init();
        
        // Record cold start metrics
        let cold_start_metrics = serde_json::json!({
            "cold_start_duration_ms": cold_start_duration.as_millis(),
            "initialization_type": "cold_start",
        });
        
        if let Err(e) = telemetry.ship_custom_metrics("extension-init", cold_start_metrics).await {
            warn!("Failed to ship cold start metrics: {}", e);
        }
    }
    
    // Create shared service closure
    let service = service_fn(|event: LambdaEvent| {
        let telemetry = telemetry.clone();
        let state_manager = state_manager.clone();
        
        async move {
            // Process the event in both state manager and telemetry
            let mut state = state_manager.lock().await;
            state.process_event(&event);
            
            if let Err(e) = telemetry.process_event(event.clone()).await {
                error!("Error processing event: {}", e);
            }
            
            // If shutdown event, flush all telemetry
            if matches!(event.next, NextEvent::Shutdown(_)) {
                debug!("Shutdown event received, flushing telemetry");
                if let Err(e) = telemetry.force_flush().await {
                    error!("Error flushing telemetry: {}", e);
                }
            }
            
            Ok::<(), anyhow::Error>(())
        }
    });
    
    // Start the extension
    let shared_service = SharedService::new(service);
    let extension = Extension::new()
        .with_client(extension_client)
        .with_service(shared_service.clone());
        
    // Run the extension
    if let Err(e) = extension.run().await {
        error!("Extension exited with error: {}", e);
        
        // Flush telemetry on error
        if let Err(flush_err) = telemetry.force_flush().await {
            error!("Error flushing telemetry during shutdown: {}", flush_err);
        }
        
        return Err(e.into());
    }
    
    // Final flush and shutdown
    if let Err(e) = telemetry.force_flush().await {
        error!("Error during final telemetry flush: {}", e);
    }
    
    info!("Extension shutting down gracefully");
    
    Ok(())
}
