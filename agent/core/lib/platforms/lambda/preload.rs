// agent/core/lib/platforms/lambda/preload.rs

use std::collections::HashSet;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tracing::{debug, info, warn};

use crate::startup::asm_opt::{prefetch_critical_memory, optimized_spin_wait};
use crate::startup::Preloadable;

/// Preloads AWS Lambda runtime libraries that are commonly used
pub struct LambdaRuntimePreloader {
    preloaded: AtomicBool,
}

impl LambdaRuntimePreloader {
    pub fn new() -> Self {
        Self {
            preloaded: AtomicBool::new(false),
        }
    }
}

impl Preloadable for LambdaRuntimePreloader {
    fn preload(&self) {
        if self.preloaded.swap(true, Ordering::SeqCst) {
            return; // Already preloaded
        }
        
        // Common Lambda runtime directories to warm up
        let paths = [
            "/var/runtime",
            "/var/lang/bin",
            "/var/rapid",
            "/opt/extensions",
        ];
        
        for path in &paths {
            if Path::new(path).exists() {
                debug!("Preloading Lambda runtime path: {}", path);
                
                // Read directory contents to cache file metadata
                if let Ok(entries) = std::fs::read_dir(path) {
                    for entry in entries.flatten() {
                        let _ = entry.file_name();
                        let _ = entry.metadata();
                    }
                }
            }
        }
        
        // Preload common environment variables
        let env_vars = [
            "AWS_LAMBDA_FUNCTION_NAME",
            "AWS_LAMBDA_FUNCTION_VERSION",
            "AWS_LAMBDA_FUNCTION_MEMORY_SIZE",
            "AWS_REGION",
            "AWS_EXECUTION_ENV",
            "AWS_LAMBDA_INITIALIZATION_TYPE",
            "AWS_LAMBDA_LOG_GROUP_NAME",
            "AWS_LAMBDA_LOG_STREAM_NAME",
            "AWS_ACCESS_KEY_ID",
            "AWS_SECRET_ACCESS_KEY",
            "AWS_SESSION_TOKEN",
        ];
        
        for var in &env_vars {
            let _ = std::env::var(var);
        }
    }
    
    fn name(&self) -> &'static str {
        "lambda_runtime"
    }
    
    fn estimated_memory(&self) -> usize {
        // Rough estimate based on file descriptor cache and env vars
        64 * 1024 // 64 KB
    }
}

/// Preload TLS certificates and DNS resolution for Lambda
pub struct NetworkPreloader {
    preloaded: AtomicBool,
    endpoints: Vec<&'static str>,
}

impl NetworkPreloader {
    pub fn new(endpoints: Vec<&'static str>) -> Self {
        Self {
            preloaded: AtomicBool::new(false),
            endpoints,
        }
    }
    
    pub fn with_default_aws_endpoints() -> Self {
        Self::new(vec![
            "dynamodb.us-east-1.amazonaws.com",
            "s3.amazonaws.com",
            "sqs.us-east-1.amazonaws.com",
            "lambda.us-east-1.amazonaws.com",
            "logs.us-east-1.amazonaws.com",
        ])
    }
}

impl Preloadable for NetworkPreloader {
    fn preload(&self) {
        if self.preloaded.swap(true, Ordering::SeqCst) {
            return; // Already preloaded
        }
        
        // Resolve DNS for common endpoints
        let resolved = self.endpoints.iter()
            .filter_map(|endpoint| {
                match std::net::ToSocketAddrs::to_socket_addrs(&format!("{}:443", endpoint)) {
                    Ok(addrs) => {
                        let ips: Vec<_> = addrs.map(|a| a.ip()).collect();
                        if !ips.is_empty() {
                            debug!("Resolved {} to {} addresses", endpoint, ips.len());
                            Some(endpoint)
                        } else {
                            None
                        }
                    }
                    Err(_) => None,
                }
            })
            .collect::<Vec<_>>();
            
        debug!("Preloaded DNS for {}/{} endpoints", resolved.len(), self.endpoints.len());
    }
    
    fn name(&self) -> &'static str {
        "network_preload"
    }
    
    fn estimated_memory(&self) -> usize {
        // Rough estimate for DNS cache and TLS state
        32 * 1024 // 32 KB
    }
}

/// Preload commonly used dynamic libraries
pub struct LibraryPreloader {
    preloaded: AtomicBool,
    libraries: Vec<&'static str>,
}

impl LibraryPreloader {
    pub fn new(libraries: Vec<&'static str>) -> Self {
        Self {
            preloaded: AtomicBool::new(false),
            libraries,
        }
    }
    
    pub fn with_common_libraries() -> Self {
        Self::new(vec![
            "libssl.so",
            "libcrypto.so",
            "libz.so",
            "libc.so.6",
        ])
    }
}

impl Preloadable for LibraryPreloader {
    fn preload(&self) {
        if self.preloaded.swap(true, Ordering::SeqCst) {
            return; // Already preloaded
        }
        
        #[cfg(target_os = "linux")]
        {
            use std::os::unix::fs::MetadataExt;
            
            // Common library paths
            let lib_paths = [
                "/lib64",
                "/usr/lib64",
                "/var/runtime",
                "/var/lang/lib",
            ];
            
            let mut found_libs = HashSet::new();
            
            // Find and touch the libraries to ensure they're in page cache
            for path in &lib_paths {
                if !Path::new(path).exists() {
                    continue;
                }
                
                if let Ok(entries) = std::fs::read_dir(path) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                            for lib in &self.libraries {
                                if filename.starts_with(lib) || filename.contains(lib) {
                                    // Get file metadata to cache inode
                                    if let Ok(metadata) = entry.metadata() {
                                        let _ = metadata.ino();
                                        let _ = metadata.size();
                                        found_libs.insert(*lib);
                                        debug!("Preloaded library: {}", filename);
                                        
                                        // Read a small chunk to bring into page cache
                                        if let Ok(mut file) = std::fs::File::open(&path) {
                                            use std::io::Read;
                                            let mut buffer = [0u8; 4096];
                                            let _ = file.read(&mut buffer);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            debug!("Preloaded {}/{} libraries", found_libs.len(), self.libraries.len());
        }
    }
    
    fn name(&self) -> &'static str {
        "library_preload"
    }
    
    fn estimated_memory(&self) -> usize {
        // Rough estimate per library (metadata only, not full load)
        4 * 1024 * self.libraries.len() // 4 KB per library
    }
}

/// Preload the AWS SDK connection pool
pub struct AwsSdkPreloader {
    preloaded: AtomicBool,
    use_http_client: bool,
}

impl AwsSdkPreloader {
    pub fn new(use_http_client: bool) -> Self {
        Self {
            preloaded: AtomicBool::new(false),
            use_http_client,
        }
    }
}

impl Preloadable for AwsSdkPreloader {
    fn preload(&self) {
        if self.preloaded.swap(true, Ordering::SeqCst) {
            return; // Already preloaded
        }
        
        // Prime AWS_REGION and credentials
        let _ = std::env::var("AWS_REGION");
        let _ = std::env::var("AWS_ACCESS_KEY_ID");
        let _ = std::env::var("AWS_SECRET_ACCESS_KEY");
        let _ = std::env::var("AWS_SESSION_TOKEN");
        
        // Optionally initialize HTTP client
        if self.use_http_client {
            #[cfg(feature = "reqwest")]
            {
                use std::time::Duration;
                
                // Create a minimal client with common settings
                let client = reqwest::Client::builder()
                    .timeout(Duration::from_secs(5))
                    .tcp_keepalive(Some(Duration::from_secs(15)))
                    .pool_idle_timeout(Some(Duration::from_secs(30)))
                    .build();
                    
                if let Ok(client) = client {
                    // Prime the connection pool by making a small request
                    // to the AWS metadata service
                    let _result = std::thread::spawn(move || {
                        let rt = tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                            .unwrap();
                            
                        rt.block_on(async {
                            let _resp = client
                                .get("http://169.254.169.254/latest/meta-data/instance-id")
                                .timeout(Duration::from_millis(100))
                                .send()
                                .await;
                        });
                    }).join();
                    
                    debug!("Preloaded HTTP client");
                }
            }
        }
    }
    
    fn name(&self) -> &'static str {
        "aws_sdk"
    }
    
    fn estimated_memory(&self) -> usize {
        // HTTP client + connection pool
        if self.use_http_client {
            256 * 1024 // 256 KB
        } else {
            16 * 1024 // 16 KB
        }
    }
}

/// Preload the Lambda Extension API client
pub struct LambdaExtensionApiPreloader {
    preloaded: AtomicBool,
}

impl LambdaExtensionApiPreloader {
    pub fn new() -> Self {
        Self {
            preloaded: AtomicBool::new(false),
        }
    }
}

impl Preloadable for LambdaExtensionApiPreloader {
    fn preload(&self) {
        if self.preloaded.swap(true, Ordering::SeqCst) {
            return; // Already preloaded
        }
        
        // Prime Lambda Extension API environment variables
        let _ = std::env::var("AWS_LAMBDA_RUNTIME_API");
        
        #[cfg(feature = "lambda_extension")]
        {
            use std::time::Duration;
            
            // Create extension client but don't register
            let client = lambda_extension::client::ExtensionClient::builder()
                .timeout(Duration::from_secs(5))
                .build();
                
            if let Ok(_client) = client {
                debug!("Preloaded Lambda Extension API client");
            }
        }
    }
    
    fn name(&self) -> &'static str {
        "lambda_extension_api"
    }
    
    fn estimated_memory(&self) -> usize {
        32 * 1024 // 32 KB
    }
}