use aws_sdk_lambda::{Client as LambdaClient, Error as LambdaError};
use anyhow::{Context, Result};
use clap::Parser;

mod discovery;
mod attachment;
mod config;

/// Causeway Lambda Discovery - Auto-discovers and instruments Lambda functions
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// AWS region
    #[arg(short, long)]
    region: Option<String>,
    
    /// Run in scan-only mode (no modifications)
    #[arg(long)]
    scan_only: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    // Parse command line args
    let args = Args::parse();
    
    // Initialize AWS SDK
    let config = aws_config::from_env()
        .region(args.region.map(aws_smithy_types::region::Region::new))
        .load()
        .await;
    let lambda_client = LambdaClient::new(&config);
    
    tracing::info!("Starting Causeway Lambda discovery service");
    
    // Discover Lambda functions
    let functions = discovery::scan_lambda_functions(&lambda_client).await
        .context("Failed to scan Lambda functions")?;
    
    tracing::info!("Found {} Lambda functions", functions.len());
    
    if args.scan_only {
        tracing::info!("Running in scan-only mode, no modifications will be made");
        return Ok(());
    }
    
    // Attach extension to functions
    let results = attachment::attach_to_functions(&lambda_client, &functions).await
        .context("Failed to attach extension")?;
    
    tracing::info!("Attached extension to {} functions", results.success_count);
    if results.failure_count > 0 {
        tracing::warn!("{} functions failed to instrument", results.failure_count);
    }
    
    Ok(())
}
