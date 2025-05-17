pub mod types;
pub mod utils;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
