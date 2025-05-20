// agent/core/lib/platforms/mod.rs

//! Platform-specific implementations for different runtime environments
//!
//! This module contains code that is specific to particular runtime platforms,
//! such as AWS Lambda, containers, or bare metal servers.

// Re-export platform-specific modules
#[cfg(feature = "lambda")]
pub mod lambda;