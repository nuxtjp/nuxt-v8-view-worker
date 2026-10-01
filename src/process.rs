use std::io::{Read, Write};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

pub(crate) fn invoke(
    worker: &std::fs::File,
    request: &[u8],
    output_max: u64,
    timeout_ms: u64,
) -> Result<Vec<u8>, String> {
    let mut child = spawn(worker)?;
    let stdout = child.stdout.take().ok_or("worker stdout is unavailable")?;
    let stderr = child.stderr.take().ok_or("worker stderr is unavailable")?;
    let stdout_reader = std::thread::spawn(move || drain(stdout, output_max));
    let stderr_reader = std::thread::spawn(move || drain(stderr, 4_096));
    let status =
        write_request(&mut child, request).and_then(|()| wait_or_kill(&mut child, timeout_ms));
    let output = stdout_reader
        .join()
        .map_err(|_| "stdout reader failed closed")??;
    stderr_reader
        .join()
        .map_err(|_| "stderr reader failed closed")??;
    status?;
    if output.len() as u64 > output_max {
        return Err("worker response exceeds pinned output limit".into());
    }
    Ok(output)
}

#[cfg(target_os = "linux")]
fn spawn(worker: &std::fs::File) -> Result<Child, String> {
    use std::os::fd::AsRawFd;

    let pinned_path = format!("/proc/self/fd/{}", worker.as_raw_fd());
    Command::new(pinned_path)
        .env_clear()
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| "cannot start pinned worker".into())
}

#[cfg(not(target_os = "linux"))]
fn spawn(_: &std::fs::File) -> Result<Child, String> {
    Err("pinned descriptor execution requires Linux".into())
}

fn write_request(child: &mut Child, request: &[u8]) -> Result<(), String> {
    let result = child
        .stdin
        .take()
        .ok_or("worker stdin is unavailable")?
        .write_all(request);
    if result.is_err() {
        let _ = child.kill();
        let _ = child.wait();
        return Err("cannot send bounded worker request".into());
    }
    Ok(())
}

fn wait_or_kill(child: &mut Child, timeout_ms: u64) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_millis(timeout_ms);
    loop {
        match child.try_wait() {
            Ok(Some(status)) if status.success() => return Ok(()),
            Ok(Some(_)) => return Err("worker failed closed".into()),
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(2)),
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("worker exceeded host wall-clock limit".into());
            }
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("cannot observe worker status".into());
            }
        }
    }
}

fn drain(mut source: impl Read, max: u64) -> Result<Vec<u8>, String> {
    let mut kept = Vec::new();
    let mut buffer = [0_u8; 8_192];
    loop {
        let count = source
            .read(&mut buffer)
            .map_err(|_| "cannot read worker pipe")?;
        if count == 0 {
            return Ok(kept);
        }
        if kept.len() as u64 <= max {
            let remaining =
                usize::try_from((max + 1).saturating_sub(kept.len() as u64)).unwrap_or(usize::MAX);
            kept.extend_from_slice(&buffer[..count.min(remaining)]);
        }
    }
}
