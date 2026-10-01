use nuxt_v8_view_worker::contract::{
    HOST_SCHEMA, HostConfig, Limits, REQUEST_SCHEMA, RenderRequest, SCRIPT_ID, SCRIPT_SHA256,
    ViewPayload,
};
use nuxt_v8_view_worker::{digest, host};
#[cfg(feature = "v8-engine")]
use std::io::Write;
use std::path::Path;
#[cfg(feature = "v8-engine")]
use std::path::PathBuf;

fn limits(timeout_ms: u64) -> Limits {
    Limits {
        input_bytes: 65_536,
        output_bytes: 65_536,
        heap_mib: 64,
        timeout_ms,
    }
}

fn request(timeout_ms: u64) -> RenderRequest {
    RenderRequest {
        schema: REQUEST_SCHEMA.into(),
        request_id: "integration-1".into(),
        script_id: SCRIPT_ID.into(),
        script_sha256: SCRIPT_SHA256.into(),
        limits: limits(timeout_ms),
        payload: ViewPayload {
            title: "<NuxtJP>".into(),
            summary: "Local rendering only".into(),
            status_label: "Ready".into(),
            highlighted: true,
        },
    }
}

fn config(worker: &Path, timeout_ms: u64) -> HostConfig {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    HostConfig {
        schema: HOST_SCHEMA.into(),
        worker_path: worker.display().to_string(),
        worker_sha256: digest::file(worker).unwrap(),
        expires_unix_seconds: now + 60,
        script_id: SCRIPT_ID.into(),
        script_sha256: SCRIPT_SHA256.into(),
        limits: limits(timeout_ms),
    }
}

#[cfg(feature = "v8-engine")]
fn worker() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_nuxt-v8-view-worker"))
}

#[cfg(feature = "v8-engine")]
fn temporary_dir() -> PathBuf {
    let name = format!(
        "nuxt-v8-worker-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let path = std::env::temp_dir().join(name);
    std::fs::create_dir(&path).unwrap();
    path
}

#[test]
#[cfg(feature = "v8-engine")]
fn renders_through_pinned_process_boundary() {
    let worker = worker();
    let response = host::launch(&config(&worker, 2_000), &request(2_000)).unwrap();
    assert_eq!(response.view.title, "<NuxtJP>");
    assert!(response.view.html.contains("&lt;NuxtJP&gt;"));
    assert!(!response.view.html.contains("<NuxtJP>"));
}

#[test]
#[cfg(feature = "v8-engine")]
fn rejects_a_tampered_worker_before_execution() {
    let original = worker();
    let directory = temporary_dir();
    let copy = directory.join("worker");
    std::fs::copy(&original, &copy).unwrap();
    let pinned = config(&copy, 2_000);
    std::fs::OpenOptions::new()
        .append(true)
        .open(&copy)
        .unwrap()
        .write_all(b"tampered")
        .unwrap();
    assert_eq!(
        host::launch(&pinned, &request(2_000)).unwrap_err(),
        "worker digest mismatch"
    );
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn host_kills_a_pinned_non_worker_on_timeout() {
    let path = Path::new("/usr/bin/yes");
    if !path.is_file() {
        return;
    }
    let error = host::launch(&config(path, 20), &request(20)).unwrap_err();
    assert_eq!(error, "worker exceeded host wall-clock limit");
}

#[test]
fn unknown_request_fields_are_rejected() {
    let json = format!(
        r#"{{"schema":"{REQUEST_SCHEMA}","request_id":"r","script_id":"{SCRIPT_ID}",
        "script_sha256":"{SCRIPT_SHA256}","limits":{{"input_bytes":4096,
        "output_bytes":4096,"heap_mib":16,"timeout_ms":100}},"payload":
        {{"title":"a","summary":"b","status_label":"c","highlighted":false}},
        "Command":"ignored"}}"#
    );
    assert!(serde_json::from_str::<RenderRequest>(&json).is_err());
}

#[test]
fn secret_url_path_and_command_values_are_rejected() {
    for value in [
        "secret=one",
        "https://outside",
        "/etc/passwd",
        "Command: curl",
    ] {
        let mut candidate = request(100);
        candidate.payload.summary = value.into();
        assert!(nuxt_v8_view_worker::input::validate_request(&candidate).is_err());
    }
}
