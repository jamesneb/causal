// agent/platforms/aws-lambda/extension/src/state.rs

use anyhow::Result;
use lambda_extension::{Extension, LambdaEvent, NextEvent};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

use agent_core::state::persistence::StatePersistence;
use agent_core::startup::is_cold_start;

/// Lambda state for tracking execution environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LambdaState {
    /// Lambda function name
    pub function_name: String,
    /// Lambda function version
    pub function_version: String,
    /// AWS region
    pub region: String,
    /// Invocation count
    pub invocation_count: u64,
    /// Extension startup timestamp
    pub startup_time: u64,
    /// Last invocation timestamp
    pub last_invocation: Option<u64>,
    /// Last shutdown timestamp
    pub last_shutdown: Option<u64>,
    /// Cold start count
    pub cold_start_count: u64,
    /// Execution environment ID
    pub execution_env_id: Option<String>,
    /// Initialization type
    pub initialization_type: Option<String>,
    /// Platform capabilities
    pub platform_capabilities: Vec<String>,
}

/// Current Lambda request state
#[derive(Debug, Clone)]
pub struct RequestState {
    /// Request ID
    pub request_id: String,
    /// Start time
    pub start_time: Instant,
    /// Deadline (ms since epoch)
    pub deadline_ms: u64,
    /// Tracing context
    pub tracing_context: Option<String>,
}

/// State manager for Lambda extension
pub struct LambdaStateManager {
    /// Persistent Lambda state
    pub state: LambdaState,
    /// State persistence
    pub persistence: StatePersistence<LambdaState>,
    /// Current request being processed
    pub current_request: Option<RequestState>,
    /// Shared memory for function communication
    pub shared_memory: Option<SharedMemory>,
}

/// Shared memory for function communication
pub struct SharedMemory {
    /// Memory region name
    pub name: String,
    /// Memory size
    pub size: usize,
    /// Memory pointer
    pub pointer: *mut u8,
}

// Ensure pointer can be sent between threads
unsafe impl Send for SharedMemory {}
unsafe impl Sync for SharedMemory {}

impl LambdaStateManager {
    /// Create a new state manager
    pub fn new() -> Result<Self> {
        // Load function information from environment
        let function_name = std::env::var("AWS_LAMBDA_FUNCTION_NAME")
            .unwrap_or_else(|_| "unknown".to_string());
            
        let function_version = std::env::var("AWS_LAMBDA_FUNCTION_VERSION")
            .unwrap_or_else(|_| "$LATEST".to_string());
            
        let region = std::env::var("AWS_REGION")
            .unwrap_or_else(|_| "us-east-1".to_string());
            
        let initialization_type = std::env::var("AWS_LAMBDA_INITIALIZATION_TYPE").ok();
        
        // Create persistence layer
        let mut persistence = StatePersistence::new(
            Some("/tmp/lambda-extension-state"),
            Some(function_name.clone()),
            Some(function_version.clone()),
        );
        
        // Try to load existing state
        let state = match persistence.load() {
            Ok(state) => {
                debug!("Loaded existing state");
                
                // Update state with current values
                let mut state = state;
                state.function_name = function_name;
                state.function_version = function_version;
                state.region = region;
                state.initialization_type = initialization_type;
                state.cold_start_count += 1;
                
                state
            }
            Err(_) => {
                debug!("Creating new state");
                
                // Create new state
                LambdaState {
                    function_name,
                    function_version,
                    region,
                    invocation_count: 0,
                    startup_time: SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                    last_invocation: None,
                    last_shutdown: None,
                    cold_start_count: 1,
                    execution_env_id: std::env::var("AWS_LAMBDA_FUNCTION_NAME").ok(),
                    initialization_type,
                    platform_capabilities: Vec::new(),
                }
            }
        };
        
        // Save initial state
        persistence.save(&state)?;
        
        Ok(Self {
            state,
            persistence,
            current_request: None,
            shared_memory: None,
        })
    }
    
    /// Process a Lambda event
    pub fn process_event(&mut self, event: &LambdaEvent) {
        match &event.next {
            NextEvent::Invoke(invoke) => {
                // Increment invocation count
                self.state.invocation_count += 1;
                
                // Update last invocation timestamp
                self.state.last_invocation = Some(
                    SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                );
                
                // Store current request state
                self.current_request = Some(RequestState {
                    request_id: invoke.request_id.clone(),
                    start_time: Instant::now(),
                    deadline_ms: invoke.deadline_ms,
                    tracing_context: invoke.tracing.as_ref().map(|tc| tc.to_string()),
                });
                
                // Save state
                if let Err(e) = self.persistence.save(&self.state) {
                    warn!("Failed to save state: {}", e);
                }
            }
            NextEvent::Shutdown(shutdown) => {
                // Update last shutdown timestamp
                self.state.last_shutdown = Some(
                    SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                );
                
                // Save state
                if let Err(e) = self.persistence.save(&self.state) {
                    warn!("Failed to save state: {}", e);
                }
                
                // Clear current request
                self.current_request = None;
            }
        }
    }
    
    /// Get current request duration
    pub fn get_current_request_duration(&self) -> Option<Duration> {
        self.current_request.as_ref().map(|req| req.start_time.elapsed())
    }
    
    /// Get time remaining until deadline
    pub fn get_time_remaining(&self) -> Option<Duration> {
        self.current_request.as_ref().map(|req| {
            let now_ms = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;
                
            if req.deadline_ms > now_ms {
                Duration::from_millis(req.deadline_ms - now_ms)
            } else {
                Duration::from_millis(0)
            }
        })
    }
    
    /// New with defaults helper
    pub fn new_with_defaults() -> Self {
        let function_name = std::env::var("AWS_LAMBDA_FUNCTION_NAME")
            .unwrap_or_else(|_| "unknown".to_string());
            
        let function_version = std::env::var("AWS_LAMBDA_FUNCTION_VERSION")
            .unwrap_or_else(|_| "$LATEST".to_string());
            
        let region = std::env::var("AWS_REGION")
            .unwrap_or_else(|_| "us-east-1".to_string());
            
        let persistence = StatePersistence::new(
            Some("/tmp/lambda-extension-state"),
            None,
            None,
        );
        
        Self {
            state: LambdaState {
                function_name,
                function_version,
                region,
                invocation_count: 0,
                startup_time: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                last_invocation: None,
                last_shutdown: None,
                cold_start_count: 1,
                execution_env_id: std::env::var("AWS_LAMBDA_FUNCTION_NAME").ok(),
                initialization_type: std::env::var("AWS_LAMBDA_INITIALIZATION_TYPE").ok(),
                platform_capabilities: Vec::new(),
            },
            persistence,
            current_request: None,
            shared_memory: None,
        }
    }
}

impl Default for LambdaStateManager {
    fn default() -> Self {
        Self::new_with_defaults()
    }
}
