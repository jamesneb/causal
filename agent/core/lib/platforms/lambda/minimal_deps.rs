// agent/core/lib/platforms/lambda/minimal_deps.rs

use crate::startup::minimal_deps::DEPENDENCY_LOADER;
use tracing::{debug, info};

/// Initialize the dependency loader for Lambda
pub fn init_dependency_loader() {
    // Ensure the loader is initialized
    let loader = DEPENDENCY_LOADER.clone();
    
    // Allow Lambda-specific dependencies
    loader.allow(&[
        "lambda_runtime",
        "lambda_extension",
        "aws_lambda_events",
        "aws_config",
        "aws_sdk_lambda",
        "aws_sdk_cloudwatch",
    ]);
    
    // Automatically deactivate after cold start
    let loader_clone = loader.clone();
    std::thread::spawn(move || {
        // Wait for cold start to complete
        std::thread::sleep(std::time::Duration::from_secs(5));
        loader_clone.deactivate();
        loader_clone.log_statistics();
    });
}