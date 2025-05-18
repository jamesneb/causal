use lambda_extension::{service_fn, Extension, LambdaEvent, NextEvent};
use anyhow::{Context, Result};
use agent_core::telemetry::Collector;
use agent_core::transport::Shipper;

mod metrics;
mod telemetry;
mod runtime;

async fn extension_handler(event: LambdaEvent<serde_json::Value>) -> Result<()> {
    match event.next {
        NextEvent::Invoke(invoke) => {
            // Capture Lambda invocation metrics
            let request_id = invoke.request_id.clone();
            let metrics = metrics::capture_runtime_metrics().await?;
            
            // Ship metrics to ingestion service
            telemetry::ship_metrics(request_id, metrics).await?;
        }
        NextEvent::Shutdown(shutdown) => {
            // Handle shutdown gracefully
            tracing::info!("Shutdown event received: {:?}", shutdown);
            // Flush any pending metrics
            telemetry::flush_metrics().await?;
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing::subscriber::set_global_default(
        tracing::subscriber::FmtSubscriber::builder()
            .with_max_level(tracing::Level::INFO)
            .finish(),
    )
    .context("Failed to set global tracing subscriber")?;
    
    // Get configuration from environment
    let config = runtime::get_config()?;
    
    // Initialize the extension
    tracing::info!("Initializing Causeway Lambda extension v{}", env!("CARGO_PKG_VERSION"));
    
    // Register the extension with Lambda Runtime API
    let handler = service_fn(extension_handler);
    Extension::new()
        .with_events(&["INVOKE", "SHUTDOWN"])
        .run(handler)
        .await
        .context("Failed to register Lambda extension")?;
    
    Ok(())
}
EOF
