//! Closed contracts for a short-lived, process-isolated V8 renderer.

pub mod contract;
pub mod digest;
pub mod host;
pub mod input;
mod process;
#[cfg(feature = "v8-engine")]
pub mod render;

pub use contract::{HostConfig, Limits, RenderRequest, RenderResponse, SCRIPT_ID, SCRIPT_SHA256};
pub use host::launch;
#[cfg(feature = "v8-engine")]
pub use render::render;
