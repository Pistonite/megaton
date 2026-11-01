use std::hash::Hasher;
use std::path::{Path, PathBuf};

use cu::pre::*;
use rustc_hash::FxHasher;

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
