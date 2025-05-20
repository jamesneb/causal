// agent/platforms/aws-lambda/extension/src/main.rs (Optimized for Cold Start)

use anyhow::{Context, Result};
use lambda_extension::{Extension, LambdaEvent, NextEvent, SharedService, service_fn};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};
use tracing_subscriber::{filter::LevelFilter, EnvFilter, Registry};
use tracing_subscriber::layer::SubscriberExt;

// Import cold start optimization modules
use agent_core::startup::{
    is_cold_start, time_since_startup, StartupOptimizer,
    Lazy, ConditionalLazy, env_conditional, 
    prefetch_critical_memory, optimized_spin_wait, fast_memcpy, hardware_crc32,
    preload::{self, lambda::{
        LambdaRuntimePreloader, NetworkPreloader, LibraryPreloader, AwsSdkPreloader, LambdaExtensionApiPreloader,
    }},
    minimal_deps::lambda::init_dependency_loader,
};

// For access to internal implementation details
use agent_core::startup::preload::PRELOAD_COMPLETED;

// Import our modules lazily after critical initialization
mod state;
mod telemetry;

// Import specific items needed for early initialization
use state::LambdaStateManager;

// Configuration constants
const DEFAULT_METRICS_ENDPOINT: &str = "https://metrics.example.com/v1/lambda";
const DEFAULT_BATCH_SIZE: usize = 50;
const DEFAULT_FLUSH_INTERVAL_SECS: u64 = 5;

// Environment variables
const ENV_METRICS_ENDPOINT: &str = "METRICS_ENDPOINT";
const ENV_BATCH_SIZE: &str = "METRICS_BATCH_SIZE";
const ENV_FLUSH_INTERVAL_SECS: &str = "METRICS_FLUSH_INTERVAL_SECS";
const ENV_LOG_LEVEL: &str = "LOG_LEVEL";
const ENV_DISABLE_PRELOAD: &str = "DISABLE_PRELOAD";
const ENV_MEMORY_LIMIT_MB: &str = "PRELOAD_MEMORY_LIMIT_MB";

// Use minimal structs for cold start critical path
struct ExtensionConfig {
    metrics_endpoint: String,
    batch_size: usize,
    flush_interval: Duration,
    log_level: LevelFilter,
    preload_enabled: bool,
    preload_memory_limit_mb: Option<usize>,
}

impl ExtensionConfig {
    fn from_env() -> Self {
        // Fetch only required env vars during cold start
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
            
        let log_level = std::env::var(ENV_LOG_LEVEL)
            .unwrap_or_else(|_| "info".to_string())
            .parse::<LevelFilter>()
            .unwrap_or(LevelFilter::INFO);
            
        let preload_enabled = !std::env::var(ENV_DISABLE_PRELOAD)
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false);
            
        let preload_memory_limit_mb = std::env::var(ENV_MEMORY_LIMIT_MB)
            .ok()
            .and_then(|s| s.parse().ok());
            
        Self {
            metrics_endpoint,
            batch_size,
            flush_interval: Duration::from_secs(flush_interval),
            log_level,
            preload_enabled,
            preload_memory_limit_mb,
        }
    }
}

// Critical initialization that must happen before anything else
fn critical_init() -> Result<()> {
    // Initialize the dependency tracker
    init_dependency_loader();
    
    // Record cold start status early
    let cold_start = is_cold_start();
    
    if cold_start {
        // Enable more aggressive optimizations on cold start
        std::env::set_var("RUST_MIN_STACK", "131072"); // 128KB minimum stack
        
        // Prefetch critical memory regions for faster access
        unsafe {
            // Prefetch common environment variables
            for env_var in &[
                "METRICS_ENDPOINT", "METRICS_BATCH_SIZE", "METRICS_FLUSH_INTERVAL_SECS",
                "LOG_LEVEL", "PRELOAD_MEMORY_LIMIT_MB", "AWS_LAMBDA_FUNCTION_NAME",
                "AWS_LAMBDA_FUNCTION_VERSION", "AWS_REGION"
            ] {
                if let Ok(value) = std::env::var(env_var) {
                    prefetch_critical_memory(value.as_ptr(), value.len());
                }
            }
            
            // Prefetch process memory regions
            prefetch_critical_memory(
                &PRELOAD_COMPLETED as *const AtomicBool as *const u8,
                std::mem::size_of::<AtomicBool>()
            );
        }
    }
    
    Ok(())
}

// Initialize logging with minimal overhead
fn init_logging(log_level: LevelFilter) -> Result<()> {
    let env_filter = EnvFilter::builder()
        .with_default_directive(log_level.into())
        .from_env_lossy();
        
    // Use a minimal logger during cold start
    let subscriber = Registry::default()
        .with(env_filter)
        .with(tracing_subscriber::fmt::Layer::default()
            .with_ansi(false) // Disable ANSI colors in Lambda
            .with_target(false) // Don't include targets
        );
        
    tracing::subscriber::set_global_default(subscriber)
        .context("Failed to set global default subscriber")?;
        
    Ok(())
}

// Lazily initialized Telemetry client
struct LazyTelemetry {
    client: Lazy<Arc<telemetry::LambdaTelemetryExtension>, Box<dyn FnOnce() -> Arc<telemetry::LambdaTelemetryExtension> + Send>>,
}

impl LazyTelemetry {
    fn new(
        config: &ExtensionConfig,
        extension_id: String,
        state_manager: Arc<Mutex<LambdaStateManager>>,
    ) -> Self {
        let metrics_endpoint = config.metrics_endpoint.clone();
        let batch_size = config.batch_size;
        let flush_interval = config.flush_interval;
        let state_manager_clone = state_manager.clone();
        
        Self {
            client: Lazy::new("telemetry_client", Box::new(move || {
                debug!("Initializing telemetry client");
                Arc::new(
                    telemetry::LambdaTelemetryExtension::new(
                        extension_id,
                        metrics_endpoint,
                        batch_size,
                        state_manager_clone,
                        flush_interval,
                    )
                )
            })),
        }
    }
    
    // Access the client, initializing if necessary
    fn client(&self) -> &Arc<telemetry::LambdaTelemetryExtension> {
        self.client.get()
    }
    
    // Log cold start metrics
    async fn log_cold_start(&self, duration: Duration) -> Result<()> {
        // Create cold start metrics
        let cold_start_metrics = serde_json::json!({
            "cold_start_duration_ms": duration.as_millis(),
            "initialization_type": "cold_start",
            "timestamp": chrono::Utc::now().timestamp_millis(),
        });
        
        // Ship metrics
        self.client().ship_custom_metrics("extension-init", cold_start_metrics).await?;
        Ok(())
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    // Record the start time for cold start metrics
    let start_time = Instant::now();
    
    // Perform critical initialization first
    critical_init()?;
    
    // Load configuration
    let config = ExtensionConfig::from_env();
    
    // Initialize logging
    init_logging(config.log_level)?;
    
    // Log startup
    info!("Lambda extension starting up");
    
    // Preload critical components if enabled
    if config.preload_enabled {
        let start = Instant::now();
        debug!("Preloading critical components");
        
        // Create preloaders
        let runtime_preloader = LambdaRuntimePreloader::new();
        let network_preloader = NetworkPreloader::with_default_aws_endpoints();
        let library_preloader = LibraryPreloader::with_common_libraries();
        let sdk_preloader = AwsSdkPreloader::new(true);
        let extension_preloader = LambdaExtensionApiPreloader::new();
        
        // Register preloadable components
        let preload_components: Vec<&dyn preload::Preloadable> = vec![
            &runtime_preloader,
            &library_preloader,
            &extension_preloader,
            &network_preloader,
            &sdk_preloader,
        ];
        
        // Perform preloading with memory limit
        preload::preload_components(preload_components, config.preload_memory_limit_mb);
        
        debug!("Preloading completed in {:?}", start.elapsed());
    }
    
    // Initialize state manager
    let state_manager = Arc::new(Mutex::new(
        // Use minimal error handling during cold start
        LambdaStateManager::new().unwrap_or_else(|_| {
            warn!("Failed to load state, using default");
            LambdaStateManager::default()
        })
    ));
    
    // Register the extension
    let extension_client = lambda_extension::client::ExtensionClient::new()
        .context("Failed to create Lambda extension client")?;
        
    let extension_id = extension_client.register().await
        .context("Failed to register extension")?;
        
    info!("Registered extension with ID: {}", extension_id);
    
    // Initialize telemetry lazily
    let telemetry = LazyTelemetry::new(
        &config,
        extension_id.clone(),
        state_manager.clone(),
    );
    
    // Log cold start metrics if this is a cold start
    if is_cold_start() {
        let cold_start_duration = start_time.elapsed();
        info!("Cold start complete in {:?}", cold_start_duration);
        
        // Log telemetry in background to avoid blocking
        tokio::spawn(async move {
            if let Err(e) = telemetry.log_cold_start(cold_start_duration).await {
                warn!("Failed to log cold start metrics: {}", e);
            }
        });
    }
    
    // Start auto-flush only when needed (lazy initialization)
    let telemetry_for_autoflush = telemetry.client.clone();
    tokio::spawn(async move {
        // Delay auto-flush start until after cold start
        // For very short durations (< 10ms), use optimized spin wait instead of async sleep
        // This reduces thread context switching overhead for critical timing paths
        let delay_duration = Duration::from_millis(100);
        if delay_duration < Duration::from_millis(10) {
            // Optimize short delays with assembly-optimized spin wait
            // This is more efficient for very short waits
            std::thread::spawn(move || {
                optimized_spin_wait(delay_duration.as_nanos() as u64 / 10);
                telemetry_for_autoflush.get().start_auto_flush();
                debug!("Auto-flush started (optimized wait)");
            });
        } else {
            tokio::time::sleep(delay_duration).await;
            telemetry_for_autoflush.get().start_auto_flush();
            debug!("Auto-flush started");
        }
    });
    
    // Create shared service closure
    let service = service_fn(move |event: LambdaEvent| {
        let telemetry = telemetry.client().clone();
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
        .with_shared_service(shared_service);
        
    // Run the extension
    if let Err(e) = extension.run().await {
        error!("Extension exited with error: {}", e);
        
        // Flush telemetry on error
        if let Err(flush_err) = telemetry.client().force_flush().await {
            error!("Error flushing telemetry during shutdown: {}", flush_err);
        }
        
        return Err(e.into());
    }
    
    // Final flush and shutdown
    if let Err(e) = telemetry.client().force_flush().await {
        error!("Error during final telemetry flush: {}", e);
    }
    
    info!("Extension shutting down gracefully");
    
    Ok(())
}

// Default implementation for LambdaStateManager
impl Default for LambdaStateManager {
    fn default() -> Self {
        // Create minimal state with default values
        Self::new_with_defaults()
    }
}

// Add default constructor to LambdaStateManager
impl LambdaStateManager {
    fn new_with_defaults() -> Self {
        let function_name = std::env::var("AWS_LAMBDA_FUNCTION_NAME")
            .unwrap_or_else(|_| "unknown".to_string());
            
        let function_version = std::env::var("AWS_LAMBDA_FUNCTION_VERSION")
            .unwrap_or_else(|_| "$LATEST".to_string());
            
        let region = std::env::var("AWS_REGION")
            .unwrap_or_else(|_| "us-east-1".to_string());
            
        let persistence = agent_core::state::persistence::StatePersistence::new(
            Some("/tmp/lambda-extension-state"),
            None,
            None,
        );
        
        Self::new_from_state(
            agent_core::state::persistence::StatePersistence::default_state(
                &function_name,
                &function_version,
                &region,
            ),
            persistence,
        )
    }
    
    // Add constructor that takes explicit parameters
    fn new_from_state(
        state: state::LambdaState,
        persistence: agent_core::state::persistence::StatePersistence<state::LambdaState>,
    ) -> Self {
        Self {
            state,
            persistence,
            current_request: None,
            shared_memory: None, // Initialize later if needed
        }
    }
}
