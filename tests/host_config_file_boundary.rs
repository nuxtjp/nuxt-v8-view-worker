//! Exercises host configuration admission without enabling the V8 engine.

use nuxt_v8_view_worker::host;
use std::path::{Path, PathBuf};

const LIMIT: usize = 65_536;
const FIXTURE: &str = "examples/host-config.example.json";

#[test]
fn accepts_a_host_configuration_at_the_exact_limit() {
    let root = temporary_dir();
    let path = root.join("exact.json");
    write_padded_fixture(&path, LIMIT);
    host::load_config(&path).expect("exact-limit host configuration");
    std::fs::remove_dir_all(root).expect("remove temporary directory");
}

#[test]
fn rejects_limit_plus_one_without_disclosing_content() {
    let root = temporary_dir();
    let path = root.join("oversized.json");
    write_padded_fixture(&path, LIMIT + 1);
    let error = host::load_config(&path).expect_err("oversized configuration must fail");
    assert!(error.contains("exceeds configured limit"));
    assert!(!error.contains("worker_path"));
    std::fs::remove_dir_all(root).expect("remove temporary directory");
}

#[test]
fn rejects_a_directory_without_reading_it() {
    let root = temporary_dir();
    let error = host::load_config(&root).expect_err("directory must fail");
    assert!(error.contains("regular non-symlink"));
    std::fs::remove_dir_all(root).expect("remove temporary directory");
}

#[test]
fn parse_errors_do_not_echo_configuration_content() {
    let root = temporary_dir();
    let path = root.join("invalid.json");
    std::fs::write(&path, br#"{"token":"do-not-disclose"}"#).expect("write invalid input");
    let error = host::load_config(&path).expect_err("invalid configuration must fail");
    assert!(!error.contains("do-not-disclose"));
    std::fs::remove_dir_all(root).expect("remove temporary directory");
}

#[cfg(unix)]
#[test]
fn rejects_an_initial_symlink() {
    use std::os::unix::fs::symlink;

    let root = temporary_dir();
    let target = root.join("target.json");
    let link = root.join("link.json");
    std::fs::copy(FIXTURE, &target).expect("copy fixture");
    symlink(&target, &link).expect("create symlink");
    assert!(host::load_config(&link).is_err());
    std::fs::remove_dir_all(root).expect("remove temporary directory");
}

fn write_padded_fixture(path: &Path, size: usize) {
    let mut source = std::fs::read(FIXTURE).expect("read fixture");
    assert!(source.len() <= size);
    source.resize(size, b' ');
    std::fs::write(path, source).expect("write padded fixture");
}

fn temporary_dir() -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "nuxt-v8-host-config-{}-{nonce}",
        std::process::id()
    ));
    std::fs::create_dir(&path).expect("create temporary directory");
    path
}
