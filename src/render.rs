use crate::contract::{
    RESPONSE_SCHEMA, RenderRequest, RenderResponse, RenderedView, SCRIPT_SHA256,
};
use crate::{digest, input};
use std::sync::{
    Arc, Once,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use std::time::Duration;

const RENDERER: &str = include_str!("renderer.js");
static V8_INIT: Once = Once::new();

/// Executes the embedded renderer against a validated scalar payload.
///
/// # Errors
///
/// Returns an error for contract, digest, V8, timeout, or output failures.
pub fn render(request: &RenderRequest) -> Result<RenderResponse, String> {
    input::validate_request(request)?;
    if digest::bytes(RENDERER.as_bytes()) != SCRIPT_SHA256 {
        return Err("embedded renderer digest does not match contract".into());
    }
    let payload =
        serde_json::to_string(&request.payload).map_err(|_| "cannot serialize closed payload")?;
    let encoded = serde_json::to_string(&payload).map_err(|_| "cannot quote closed payload")?;
    let source =
        format!("(() => {{ const input = Object.freeze(JSON.parse({encoded}));\n{RENDERER}\n}})()");
    let output = execute(&source, &request.limits)?;
    if output.len() as u64 > request.limits.output_bytes {
        return Err("renderer output exceeds configured limit".into());
    }
    let view: RenderedView =
        serde_json::from_str(&output).map_err(|_| "renderer returned an invalid closed view")?;
    Ok(RenderResponse {
        schema: RESPONSE_SCHEMA.into(),
        request_id: request.request_id.clone(),
        status: "rendered".into(),
        view,
    })
}

fn execute(source: &str, limits: &crate::contract::Limits) -> Result<String, String> {
    initialize();
    let heap = usize::from(limits.heap_mib) * 1024 * 1024;
    let params = v8::CreateParams::default().heap_limits(0, heap);
    let mut isolate = v8::Isolate::new(params);
    let handle = isolate.thread_safe_handle();
    let timed_out = Arc::new(AtomicBool::new(false));
    let timeout_flag = Arc::clone(&timed_out);
    let (done_tx, done_rx) = mpsc::channel();
    let duration = Duration::from_millis(limits.timeout_ms);
    let watchdog = std::thread::spawn(move || {
        if done_rx.recv_timeout(duration).is_err() {
            timeout_flag.store(true, Ordering::Release);
            handle.terminate_execution();
        }
    });
    let result = execute_in_isolate(&mut isolate, source);
    let _ = done_tx.send(());
    watchdog.join().map_err(|_| "watchdog failed closed")?;
    if timed_out.load(Ordering::Acquire) {
        return Err("renderer exceeded wall-clock limit".into());
    }
    result
}

fn execute_in_isolate(isolate: &mut v8::OwnedIsolate, source: &str) -> Result<String, String> {
    v8::scope!(let handle_scope, isolate);
    let context = v8::Context::new(handle_scope, v8::ContextOptions::default());
    let scope = &v8::ContextScope::new(handle_scope, context);
    let code = v8::String::new(scope, source).ok_or("renderer source allocation failed")?;
    let script = v8::Script::compile(scope, code, None).ok_or("renderer compilation failed")?;
    let value = script.run(scope).ok_or("renderer execution failed")?;
    let output = value
        .to_string(scope)
        .ok_or("renderer returned a non-string value")?;
    Ok(output.to_rust_string_lossy(scope))
}

fn initialize() {
    V8_INIT.call_once(|| {
        let platform = v8::new_default_platform(0, false).make_shared();
        v8::V8::initialize_platform(platform);
        v8::V8::initialize();
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::Limits;

    #[test]
    fn terminates_non_yielding_javascript() {
        let limits = Limits {
            input_bytes: 4096,
            output_bytes: 4096,
            heap_mib: 16,
            timeout_ms: 20,
        };
        assert_eq!(
            execute("while (true) {}", &limits).unwrap_err(),
            "renderer exceeded wall-clock limit"
        );
    }

    #[test]
    fn context_exposes_no_host_bindings() {
        let limits = Limits {
            input_bytes: 4096,
            output_bytes: 4096,
            heap_mib: 16,
            timeout_ms: 100,
        };
        let probe = "JSON.stringify([typeof process,typeof require,typeof fetch,typeof Deno])";
        assert_eq!(
            execute(probe, &limits).unwrap(),
            r#"["undefined","undefined","undefined","undefined"]"#
        );
    }
}
