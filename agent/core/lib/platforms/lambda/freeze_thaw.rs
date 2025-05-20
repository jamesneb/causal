// agent/core/lib/platforms/lambda/freeze_thaw.rs

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;
use tracing::debug;

/// Detection of Lambda freeze/thaw cycles
pub struct FreezeThawDetector {
    /// Last activity timestamp
    last_activity: AtomicU64,
    /// Threshold for detecting freezes
    freeze_threshold_ms: u64,
}

impl FreezeThawDetector {
    /// Create a new detector with the specified threshold
    pub fn new(freeze_threshold_ms: u64) -> Self {
        Self {
            last_activity: AtomicU64::new(
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64
            ),
            freeze_threshold_ms,
        }
    }
    
    /// Update the activity timestamp
    pub fn record_activity(&self) {
        self.last_activity.store(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            Ordering::SeqCst
        );
    }
    
    /// Check if the function was likely frozen and thawed
    pub fn was_frozen(&self) -> bool {
        let last = self.last_activity.load(Ordering::SeqCst);
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
            
        let elapsed = now - last;
        
        if elapsed > self.freeze_threshold_ms {
            // If we were likely frozen, update the activity and notify
            self.record_activity();
            debug!("Detected likely Lambda freeze/thaw cycle after {} ms", elapsed);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_freeze_thaw_detection() {
        let detector = FreezeThawDetector::new(100);
        
        // Should not be frozen initially
        assert!(!detector.was_frozen());
        
        // Manipulate the last activity time to simulate a freeze
        detector.last_activity.store(
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64 - 200,
            Ordering::SeqCst
        );
        
        // Now should detect freeze
        assert!(detector.was_frozen());
        
        // Should not detect freeze again immediately
        assert!(!detector.was_frozen());
    }
}