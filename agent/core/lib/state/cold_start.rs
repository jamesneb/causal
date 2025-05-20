// agent/core/lib/state/cold_start.rs

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use once_cell::sync::Lazy;

// Atomic flag to track cold start state
pub static COLD_START: AtomicBool = AtomicBool::new(true);

// Track initialization time for performance metrics
pub static INIT_TIME: Lazy<Instant> = Lazy::new(Instant::now);

// Check if this is a cold start and update state
pub fn is_cold_start() -> bool {
    COLD_START.swap(false, Ordering::SeqCst)
}

// Get time since initialization (for tracking cold start overhead)
pub fn time_since_init() -> Duration {
    INIT_TIME.elapsed()
}

// Get duration of cold start
pub fn cold_start_duration() -> Option<Duration> {
    if is_cold_start() {
        Some(time_since_init())
    } else {
        None
    }
}

// Reset cold start state (useful for testing)
pub fn reset_cold_start() {
    COLD_START.store(true, Ordering::SeqCst);
}

// Enhanced cold start detection that accounts for Lambda freeze/thaw
pub struct ColdStartDetector {
    last_invocation: std::sync::Mutex<Option<Instant>>,
    freeze_threshold: Duration,
}

impl ColdStartDetector {
    pub fn new(freeze_threshold: Duration) -> Self {
        Self {
            last_invocation: std::sync::Mutex::new(None),
            freeze_threshold,
        }
    }
    
    // Check if this is a cold start or if the function was frozen
    pub fn is_cold_or_frozen(&self) -> bool {
        let mut last_invocation = self.last_invocation.lock().unwrap();
        
        match *last_invocation {
            None => {
                // First invocation, definitely a cold start
                *last_invocation = Some(Instant::now());
                true
            }
            Some(last) => {
                let now = Instant::now();
                let elapsed = now.duration_since(last);
                
                // Update last invocation time
                *last_invocation = Some(now);
                
                // Check if elapsed time exceeds the freeze threshold
                elapsed > self.freeze_threshold
            }
        }
    }
    
    // Reset the detector state
    pub fn reset(&self) {
        let mut last_invocation = self.last_invocation.lock().unwrap();
        *last_invocation = None;
    }
}

// Helper functions for Lambda specific cold start detection
#[cfg(feature = "lambda")]
pub mod lambda {
    use super::*;
    use std::env;
    
    // Lambda freezes after about 5-15 minutes of inactivity
    pub const DEFAULT_FREEZE_THRESHOLD: Duration = Duration::from_secs(60 * 10); // 10 minutes
    
    // Get environment-aware cold start detector
    pub fn get_detector() -> ColdStartDetector {
        let threshold = env::var("COLD_START_FREEZE_THRESHOLD_SECS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(DEFAULT_FREEZE_THRESHOLD.as_secs());
        
        ColdStartDetector::new(Duration::from_secs(threshold))
    }
    
    // Lambda-specific cold start detection logic
    pub fn is_lambda_cold_start() -> bool {
        static DETECTOR: Lazy<ColdStartDetector> = Lazy::new(get_detector);
        DETECTOR.is_cold_or_frozen()
    }
}
