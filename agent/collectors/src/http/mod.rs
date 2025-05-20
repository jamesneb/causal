// agent/collectors/src/http/mod.rs

//! HTTP collectors for monitoring HTTP client and server operations

pub mod client;
pub mod server;

use std::collections::HashMap;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Common settings for HTTP collectors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpCollectorSettings {
    /// Whether to collect headers
    pub collect_headers: bool,
    /// Whether to collect request/response bodies
    pub collect_bodies: bool,
    /// Maximum size of collected bodies in bytes
    pub max_body_size: usize,
    /// List of headers to collect (empty means all)
    pub include_headers: Vec<String>,
    /// List of headers to exclude
    pub exclude_headers: Vec<String>,
    /// Whether to collect TLS/SSL information
    pub collect_tls_info: bool,
}

impl Default for HttpCollectorSettings {
    fn default() -> Self {
        Self {
            collect_headers: true,
            collect_bodies: false,
            max_body_size: 1024, // 1KB
            include_headers: vec![
                "content-type".to_string(),
                "content-length".to_string(),
                "user-agent".to_string(),
                "x-request-id".to_string(),
                "x-trace-id".to_string(),
            ],
            exclude_headers: vec![
                "authorization".to_string(),
                "cookie".to_string(),
                "set-cookie".to_string(),
                "x-api-key".to_string(),
            ],
            collect_tls_info: true,
        }
    }
}

/// Helper function to merge collector settings from config
pub fn merge_http_settings(
    defaults: &HttpCollectorSettings,
    config_settings: Option<&HashMap<String, String>>,
) -> Result<HttpCollectorSettings> {
    let mut settings = defaults.clone();
    
    if let Some(cfg) = config_settings {
        // Process collect_headers
        if let Some(collect_headers) = cfg.get("collect_headers") {
            settings.collect_headers = collect_headers == "true" || collect_headers == "1";
        }
        
        // Process collect_bodies
        if let Some(collect_bodies) = cfg.get("collect_bodies") {
            settings.collect_bodies = collect_bodies == "true" || collect_bodies == "1";
        }
        
        // Process max_body_size
        if let Some(max_body_size) = cfg.get("max_body_size") {
            if let Ok(size) = max_body_size.parse::<usize>() {
                settings.max_body_size = size;
            }
        }
        
        // Process include_headers
        if let Some(include_headers) = cfg.get("include_headers") {
            settings.include_headers = include_headers
                .split(',')
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect();
        }
        
        // Process exclude_headers
        if let Some(exclude_headers) = cfg.get("exclude_headers") {
            settings.exclude_headers = exclude_headers
                .split(',')
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect();
        }
        
        // Process collect_tls_info
        if let Some(collect_tls_info) = cfg.get("collect_tls_info") {
            settings.collect_tls_info = collect_tls_info == "true" || collect_tls_info == "1";
        }
    }
    
    Ok(settings)
}

/// Common schema for HTTP metrics
pub mod schema {
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    use std::time::Duration;
    
    /// HTTP method
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub enum HttpMethod {
        Get,
        Post,
        Put,
        Delete,
        Head,
        Options,
        Patch,
        Other(String),
    }
    
    impl From<&str> for HttpMethod {
        fn from(method: &str) -> Self {
            match method.to_uppercase().as_str() {
                "GET" => Self::Get,
                "POST" => Self::Post,
                "PUT" => Self::Put,
                "DELETE" => Self::Delete,
                "HEAD" => Self::Head,
                "OPTIONS" => Self::Options,
                "PATCH" => Self::Patch,
                _ => Self::Other(method.to_string()),
            }
        }
    }
    
    /// HTTP request metrics
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct HttpRequestMetrics {
        /// Request ID (generated or from headers)
        pub request_id: String,
        /// HTTP method
        pub method: HttpMethod,
        /// Request URL
        pub url: String,
        /// Request path (without query)
        pub path: String,
        /// Request headers
        pub headers: Option<HashMap<String, String>>,
        /// Request size in bytes
        pub size_bytes: usize,
        /// Whether the request has a body
        pub has_body: bool,
        /// Request timestamp
        pub timestamp: u64,
        /// Request duration
        pub duration_ms: Option<u64>,
        /// Remote address
        pub remote_addr: Option<String>,
        /// TLS information
        pub tls_info: Option<TlsInfo>,
    }
    
    /// HTTP response metrics
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct HttpResponseMetrics {
        /// Request ID (matches the request)
        pub request_id: String,
        /// HTTP status code
        pub status_code: u16,
        /// Status text
        pub status_text: String,
        /// Response headers
        pub headers: Option<HashMap<String, String>>,
        /// Response size in bytes
        pub size_bytes: usize,
        /// Whether the response has a body
        pub has_body: bool,
        /// Response timestamp
        pub timestamp: u64,
        /// Response time (time to first byte)
        pub ttfb_ms: Option<u64>,
        /// Total response time
        pub total_time_ms: u64,
    }
    
    /// TLS/SSL information
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TlsInfo {
        /// TLS version
        pub version: Option<String>,
        /// Cipher suite
        pub cipher: Option<String>,
        /// Certificate issuer
        pub cert_issuer: Option<String>,
        /// Certificate subject
        pub cert_subject: Option<String>,
        /// Certificate not before
        pub cert_not_before: Option<u64>,
        /// Certificate not after
        pub cert_not_after: Option<u64>,
    }
}