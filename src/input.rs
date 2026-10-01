use crate::contract::{
    HOST_SCHEMA, HostConfig, REQUEST_SCHEMA, RenderRequest, SCRIPT_ID, SCRIPT_SHA256,
};

const SECRET_MARKERS: &[&str] = &[
    "password",
    "passwd",
    "secret",
    "api_key",
    "apikey",
    "access_token",
    "refresh_token",
    "bearer ",
    "private key",
];
const COMMAND_MARKERS: &[&str] = &[
    "command",
    "cmd.exe",
    "powershell",
    "sh -c",
    "bash -c",
    "process.",
];

/// Validates the fixed request identity, limits, and scalar payload policy.
///
/// # Errors
///
/// Returns an error when any request value falls outside the closed contract.
pub fn validate_request(request: &RenderRequest) -> Result<(), String> {
    if request.schema != REQUEST_SCHEMA {
        return Err("unsupported request schema".into());
    }
    if request.script_id != SCRIPT_ID || request.script_sha256 != SCRIPT_SHA256 {
        return Err("renderer identity is not pinned".into());
    }
    request.limits.validate()?;
    token(&request.request_id, 64)?;
    safe_text(&request.payload.title, 120)?;
    safe_text(&request.payload.summary, 2_000)?;
    safe_text(&request.payload.status_label, 80)
}

/// Validates expiry, identity, limits, and the worker filesystem boundary.
///
/// # Errors
///
/// Returns an error when the host configuration is unsafe or stale.
pub fn validate_host(config: &HostConfig) -> Result<(), String> {
    if config.schema != HOST_SCHEMA {
        return Err("unsupported host config schema".into());
    }
    if config.script_id != SCRIPT_ID || config.script_sha256 != SCRIPT_SHA256 {
        return Err("host renderer identity is not pinned".into());
    }
    config.limits.validate()?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| "system clock is invalid")?
        .as_secs();
    if config.expires_unix_seconds <= now {
        return Err("host config has expired".into());
    }
    let path = std::path::Path::new(&config.worker_path);
    if !path.is_absolute() {
        return Err("worker path must be absolute".into());
    }
    let metadata = std::fs::symlink_metadata(path).map_err(|_| "worker is unavailable")?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err("worker must be a regular non-symlink file".into());
    }
    digest(&config.worker_sha256)
}

fn token(value: &str, max: usize) -> Result<(), String> {
    if value.is_empty()
        || value.len() > max
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err("identifier is not a closed token".into());
    }
    Ok(())
}

fn digest(value: &str) -> Result<(), String> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("SHA-256 digest is malformed".into());
    }
    Ok(())
}

fn safe_text(value: &str, max: usize) -> Result<(), String> {
    let lower = value.to_ascii_lowercase();
    let path_like = value.starts_with('/')
        || value.starts_with("./")
        || value.starts_with("../")
        || value.contains('\\');
    let forbidden = SECRET_MARKERS
        .iter()
        .chain(COMMAND_MARKERS)
        .any(|item| lower.contains(item));
    if value.is_empty()
        || value.len() > max
        || value.chars().any(char::is_control)
        || path_like
        || lower.contains("://")
        || lower.starts_with("file:")
        || forbidden
    {
        return Err("payload contains prohibited or oversized text".into());
    }
    Ok(())
}
