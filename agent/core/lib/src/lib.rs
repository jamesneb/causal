pub mod telemetry;
pub mod config;
pub mod transport;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
