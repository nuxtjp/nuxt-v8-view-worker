use crate::contract::{HostConfig, RESPONSE_SCHEMA, RenderRequest, RenderResponse};
use crate::{digest, input, process};
use std::fs::{File, Metadata, OpenOptions};

const CONFIG_MAX: u64 = 65_536;

/// Loads a bounded, regular, non-symlink host configuration.
///
/// # Errors
///
/// Returns an error when the file boundary or closed JSON contract is invalid.
pub fn load_config(path: &std::path::Path) -> Result<HostConfig, String> {
    if !path.is_absolute() {
        return Err("host config path must be absolute".into());
    }
    let mut file = open_config_no_follow(path)?;
    let metadata = file
        .metadata()
        .map_err(|_| "cannot inspect opened host config")?;
    if !opened_file_is_regular(&metadata) {
        return Err("host config must be a bounded regular non-symlink file".into());
    }
    let bytes = digest::bounded_read(&mut file, CONFIG_MAX)?;
    serde_json::from_slice(&bytes).map_err(|_| "host config is not closed JSON".into())
}

#[cfg(unix)]
fn open_config_no_follow(path: &std::path::Path) -> Result<File, String> {
    use std::os::unix::fs::OpenOptionsExt;

    OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC)
        .open(path)
        .map_err(|_| "cannot safely open host config".into())
}

#[cfg(windows)]
fn open_config_no_follow(path: &std::path::Path) -> Result<File, String> {
    use std::os::windows::fs::OpenOptionsExt;

    const FILE_FLAG_OPEN_REPARSE_POINT: u32 = 0x0020_0000;
    OpenOptions::new()
        .read(true)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)
        .map_err(|_| "cannot safely open host config".into())
}

#[cfg(not(any(unix, windows)))]
fn open_config_no_follow(_: &std::path::Path) -> Result<File, String> {
    Err("secure host config admission is unsupported on this platform".into())
}

fn opened_file_is_regular(metadata: &Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::FileTypeExt;
        let kind = metadata.file_type();
        return metadata.is_file() && !kind.is_symlink_dir() && !kind.is_symlink_file();
    }
    #[cfg(not(windows))]
    metadata.is_file()
}

/// Verifies and invokes exactly one pinned worker for one request.
///
/// # Errors
///
/// Returns an error for any contract, integrity, process, timeout, or response failure.
pub fn launch(config: &HostConfig, request: &RenderRequest) -> Result<RenderResponse, String> {
    input::validate_host(config)?;
    input::validate_request(request)?;
    if request.limits != config.limits {
        return Err("request limits do not match pinned host limits".into());
    }
    let path = std::path::Path::new(&config.worker_path);
    let mut worker = open_worker_no_follow(path)?;
    if !worker
        .metadata()
        .map_err(|_| "cannot inspect pinned worker")?
        .is_file()
    {
        return Err("opened worker is not a regular file".into());
    }
    if digest::reader(&mut worker)? != config.worker_sha256.to_ascii_lowercase() {
        return Err("worker digest mismatch".into());
    }
    let encoded = serde_json::to_vec(request).map_err(|_| "cannot encode closed request")?;
    if encoded.len() as u64 > config.limits.input_bytes {
        return Err("request exceeds pinned input limit".into());
    }
    let output_max = config.limits.output_bytes;
    let output = process::invoke(&worker, &encoded, output_max, config.limits.timeout_ms)?;
    if output.len() as u64 > output_max {
        return Err("worker response exceeds pinned output limit".into());
    }
    let response: RenderResponse =
        serde_json::from_slice(&output).map_err(|_| "worker returned invalid closed JSON")?;
    if response.schema != RESPONSE_SCHEMA
        || response.request_id != request.request_id
        || response.status != "rendered"
    {
        return Err("worker response violates correlation contract".into());
    }
    Ok(response)
}

#[cfg(unix)]
fn open_worker_no_follow(path: &std::path::Path) -> Result<File, String> {
    use std::os::unix::fs::OpenOptionsExt;

    OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC)
        .open(path)
        .map_err(|_| "cannot safely open pinned worker".into())
}

#[cfg(not(unix))]
fn open_worker_no_follow(_: &std::path::Path) -> Result<File, String> {
    Err("secure worker admission requires a Unix descriptor".into())
}

#[cfg(test)]
#[path = "host_tests.rs"]
mod tests;
