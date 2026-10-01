//! Proves that path replacement after admission cannot replace opened content.

use super::{open_config_no_follow, open_worker_no_follow};
use crate::digest;

#[cfg(unix)]
#[test]
fn reads_the_admitted_handle_after_the_path_is_replaced() {
    let root = temporary_dir();
    let admitted_path = root.join("host.json");
    let replacement_path = root.join("replacement.json");
    std::fs::write(&admitted_path, b"admitted").expect("write admitted input");
    std::fs::write(&replacement_path, b"replacement").expect("write replacement input");

    let mut admitted = open_config_no_follow(&admitted_path).expect("open admitted input");
    std::fs::rename(&replacement_path, &admitted_path).expect("replace admitted path");
    assert_eq!(
        digest::bounded_read(&mut admitted, 65_536).expect("read admitted handle"),
        b"admitted"
    );

    drop(admitted);
    std::fs::remove_dir_all(root).expect("remove temporary directory");
}

#[cfg(unix)]
#[test]
fn refuses_a_symlink_at_worker_open() {
    use std::os::unix::fs::symlink;

    let root = temporary_dir();
    let target = root.join("worker");
    let link = root.join("worker-link");
    std::fs::write(&target, b"worker").expect("write worker");
    symlink(&target, &link).expect("create worker symlink");
    let error = open_worker_no_follow(&link).expect_err("worker symlink must fail");
    assert_eq!(error, "cannot safely open pinned worker");
    std::fs::remove_dir_all(root).expect("remove temporary directory");
}

#[cfg(unix)]
fn temporary_dir() -> std::path::PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path =
        std::env::temp_dir().join(format!("nuxt-v8-host-race-{}-{nonce}", std::process::id()));
    std::fs::create_dir(&path).expect("create temporary directory");
    path
}
