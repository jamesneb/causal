// agent/core/lib/platforms/lambda/mod.rs

//! AWS Lambda-specific functionality
//!
//! This module contains code that is specific to the AWS Lambda platform,
//! including preloaders, freeze/thaw detection, and other Lambda-specific optimizations.

mod preload;
mod minimal_deps;
mod freeze_thaw;

pub use preload::{
    LambdaRuntimePreloader,
    NetworkPreloader,
    LibraryPreloader,
    AwsSdkPreloader,
    LambdaExtensionApiPreloader
};
pub use minimal_deps::init_dependency_loader;
pub use freeze_thaw::FreezeThawDetector;