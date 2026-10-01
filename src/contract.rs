use serde::{Deserialize, Serialize};

pub const REQUEST_SCHEMA: &str = "nuxtjp://v8-view-worker/request/v1";
pub const RESPONSE_SCHEMA: &str = "nuxtjp://v8-view-worker/response/v1";
pub const HOST_SCHEMA: &str = "nuxtjp://v8-view-worker/host-config/v1";
pub const SCRIPT_ID: &str = "nuxtjp-service-card-v1";
pub const SCRIPT_SHA256: &str = "bd954b068b08f679b2c90c788b907100a184097693bbf24ddd119937d4119b8e";
pub const HARD_INPUT_MAX: u64 = 1_048_576;
pub const HARD_OUTPUT_MAX: u64 = 262_144;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    pub input_bytes: u64,
    pub output_bytes: u64,
    pub heap_mib: u16,
    pub timeout_ms: u64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ViewPayload {
    pub title: String,
    pub summary: String,
    pub status_label: String,
    pub highlighted: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RenderRequest {
    pub schema: String,
    pub request_id: String,
    pub script_id: String,
    pub script_sha256: String,
    pub limits: Limits,
    pub payload: ViewPayload,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RenderedView {
    pub title: String,
    pub summary: String,
    pub status_label: String,
    pub highlighted: bool,
    pub html: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RenderResponse {
    pub schema: String,
    pub request_id: String,
    pub status: String,
    pub view: RenderedView,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HostConfig {
    pub schema: String,
    pub worker_path: String,
    pub worker_sha256: String,
    pub expires_unix_seconds: u64,
    pub script_id: String,
    pub script_sha256: String,
    pub limits: Limits,
}

impl Limits {
    /// Validates that all resource limits stay within the worker's hard ceiling.
    ///
    /// # Errors
    ///
    /// Returns an error when any limit is zero, excessive, or too small to operate safely.
    pub fn validate(&self) -> Result<(), String> {
        if self.input_bytes == 0 || self.input_bytes > HARD_INPUT_MAX {
            return Err("input limit is outside the closed range".into());
        }
        if self.output_bytes == 0 || self.output_bytes > HARD_OUTPUT_MAX {
            return Err("output limit is outside the closed range".into());
        }
        if !(8..=128).contains(&self.heap_mib) {
            return Err("heap limit is outside the closed range".into());
        }
        if !(10..=10_000).contains(&self.timeout_ms) {
            return Err("timeout is outside the closed range".into());
        }
        Ok(())
    }
}
