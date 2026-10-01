use nuxt_v8_view_worker::contract::{HARD_INPUT_MAX, RenderRequest};
use nuxt_v8_view_worker::{digest, host};
use std::io::Write;
use std::path::Path;

fn main() {
    if let Err(error) = run() {
        eprintln!("nuxt-v8-view-host: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut arguments = std::env::args_os();
    let _program = arguments.next();
    let config_path = arguments
        .next()
        .ok_or("exactly one host config path is required")?;
    if arguments.next().is_some() {
        return Err("exactly one host config path is required".into());
    }
    let config = host::load_config(Path::new(&config_path))?;
    let bytes = digest::bounded_read(&mut std::io::stdin().lock(), HARD_INPUT_MAX)?;
    let request: RenderRequest =
        serde_json::from_slice(&bytes).map_err(|_| "request is not closed JSON")?;
    let response = host::launch(&config, &request)?;
    let output = serde_json::to_vec(&response).map_err(|_| "cannot encode host response")?;
    std::io::stdout()
        .lock()
        .write_all(&output)
        .map_err(|_| "cannot write host response")?;
    Ok(())
}
