use std::fmt::Write as _;
use std::hash::Hasher;
use std::path::{Path, PathBuf};

use cu::pre::*;
use rustc_hash::FxHasher;
use sha2::{Digest, Sha256};

/// Get a collision resistant output path in `out_dir` for the input `path`,
/// in the form of `out_dir/<file_base>_<hash>.<ext>`
pub fn hashed_output_path(path: &Path, out_dir: &Path) -> cu::Result<PathBuf> {
    let file_base = cu::check!(path.file_stem(), "cannot hash path without a file name: '{}'", path.display())?;

    let mut hasher = FxHasher::default();
    hasher.write(path.as_os_str().as_encoded_bytes());
    let hash = hasher.finish();

    let mut file_name = file_base.to_os_string();
    file_name.push(format!("_{hash:016x}"));
    if let Some(ext) = path.extension() {
        file_name.push(".");
        file_name.push(ext);
    }

    Ok(out_dir.join(file_name))
}

/// Compute the SHA-256 of a file as a lowercase hex string, streaming the
/// content so the file does not need to fit in memory
pub async fn co_file_sha256(path: &Path) -> cu::Result<String> {
    let mut reader = cu::fs::co_reader(path).await?;
    let mut hasher = Sha256::new();
    // larger than the reader's internal buffer, so reads go straight to the file
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        let n = cu::check!(reader.read(&mut buf).await, "failed to read file '{}'", path.display())?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }

    let mut hex = String::with_capacity(64);
    for b in hasher.finalize() {
        // write to string cannot fail
        let _ = write!(hex, "{b:02x}");
    }
    Ok(hex)
}
