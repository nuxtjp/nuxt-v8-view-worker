use nuxt_v8_view_worker::contract::{HARD_INPUT_MAX, RenderRequest};
use nuxt_v8_view_worker::{digest, render};
use std::io::Write;

fn main() {
    if let Err(error) = run() {
        eprintln!("nuxt-v8-view-worker: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let input = digest::bounded_read(&mut std::io::stdin().lock(), HARD_INPUT_MAX)?;
    let request: RenderRequest =
        serde_json::from_slice(&input).map_err(|_| "request is not closed JSON")?;
    if input.len() as u64 > request.limits.input_bytes {
        return Err("request exceeds declared input limit".into());
    }
    let response = render(&request)?;
    let output = serde_json::to_vec(&response).map_err(|_| "cannot encode closed response")?;
    if output.len() as u64 > request.limits.output_bytes {
        return Err("response exceeds declared output limit".into());
    }
    std::io::stdout()
        .lock()
        .write_all(&output)
        .map_err(|_| "cannot write bounded response")?;
    Ok(())
}
