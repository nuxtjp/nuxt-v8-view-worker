use sha2::{Digest, Sha256};
use std::io::{self, Read};
use std::path::Path;

#[must_use]
pub fn bytes(value: &[u8]) -> String {
    format!("{:x}", Sha256::digest(value))
}

/// Computes the SHA-256 of a file without retaining its contents.
///
/// # Errors
///
/// Returns an error when the file cannot be opened or read completely.
pub fn file(path: &Path) -> Result<String, String> {
    let mut source = std::fs::File::open(path).map_err(|_| "cannot open pinned worker")?;
    reader(&mut source)
}

pub(crate) fn reader(source: &mut impl Read) -> Result<String, String> {
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8_192];
    loop {
        let count = source
            .read(&mut buffer)
            .map_err(|_| "cannot hash pinned worker")?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

/// Reads no more than a caller-selected byte ceiling.
///
/// # Errors
///
/// Returns an error on I/O failure or when the source exceeds the ceiling.
pub fn bounded_read(reader: &mut impl Read, max: u64) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    reader
        .take(max + 1)
        .read_to_end(&mut bytes)
        .map_err(map_read)?;
    if bytes.len() as u64 > max {
        return Err("input exceeds configured limit".into());
    }
    Ok(bytes)
}

fn map_read(_: io::Error) -> String {
    "cannot read bounded input".into()
}
