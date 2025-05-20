// agent/collectors/src/aws_sdk/mod.rs

//! AWS SDK collectors for monitoring AWS services usage

pub mod lambda;
pub mod dynamodb;
pub mod s3;

use std::collections::HashMap;
use std::env;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Common settings for AWS SDK collectors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsSdkCollectorSettings {
    /// AWS region
    pub region: Option<String>,
    /// AWS profile
    pub profile: Option<String>,
    /// Whether to use environment credentials
    pub use_environment: bool,
    /// Whether to use instance profile
    pub use_instance_profile: bool,
    /// AWS endpoint override (useful for local testing)
    pub endpoint: Option<String>,
    /// Whether to collect detailed metrics
    pub detailed_metrics: bool,
}

impl Default for AwsSdkCollectorSettings {
    fn default() -> Self {
        Self {
            region: env::var("AWS_REGION").ok(),
            profile: None,
            use_environment: true,
            use_instance_profile: false,
            endpoint: None,
            detailed_metrics: false,
        }
    }
}

/// Helper function to merge collector settings from config
pub fn merge_aws_settings(
    defaults: &AwsSdkCollectorSettings,
    config_settings: Option<&HashMap<String, String>>,
) -> Result<AwsSdkCollectorSettings> {
    let mut settings = defaults.clone();
    
    if let Some(cfg) = config_settings {
        // Process region
        if let Some(region) = cfg.get("region") {
            settings.region = Some(region.clone());
        }
        
        // Process profile
        if let Some(profile) = cfg.get("profile") {
            settings.profile = Some(profile.clone());
        }
        
        // Process use_environment
        if let Some(use_env) = cfg.get("use_environment") {
            settings.use_environment = use_env == "true" || use_env == "1";
        }
        
        // Process use_instance_profile
        if let Some(use_profile) = cfg.get("use_instance_profile") {
            settings.use_instance_profile = use_profile == "true" || use_profile == "1";
        }
        
        // Process endpoint
        if let Some(endpoint) = cfg.get("endpoint") {
            settings.endpoint = Some(endpoint.clone());
        }
        
        // Process detailed_metrics
        if let Some(detailed) = cfg.get("detailed_metrics") {
            settings.detailed_metrics = detailed == "true" || detailed == "1";
        }
    }
    
    Ok(settings)
}

/// Common schema for AWS SDK metrics
pub mod schema {
    use serde::{Deserialize, Serialize};
    
    /// AWS API call result
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum ApiCallResult {
        Success,
        ClientError,
        ServerError,
        NetworkError,
        Timeout,
        Unknown,
    }
    
    /// AWS API call metrics
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ApiCallMetrics {
        /// Service name
        pub service: String,
        /// Operation name
        pub operation: String,
        /// Result status
        pub result: ApiCallResult,
        /// Duration in milliseconds
        pub duration_ms: u64,
        /// Retry count
        pub retry_count: u32,
        /// Error code if applicable
        pub error_code: Option<String>,
        /// Error message if applicable
        pub error_message: Option<String>,
        /// Request ID if available
        pub request_id: Option<String>,
    }
}