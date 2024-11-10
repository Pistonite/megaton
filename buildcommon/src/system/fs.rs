use crate::prelude::*;

use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

use filetime::FileTime;

use super::Error;

/// Create file for buffered writing
pub fn buf_writer(path: impl AsRef<Path>) -> Result<BufWriter<File>, Error> {
    let path = path.as_ref();
    let file =
        File::create(path).change_context_lazy(|| Error::WriteFile(path.display().to_string()))?;
    Ok(BufWriter::new(file))
}

/// Open file for buffered reading
pub fn buf_reader(path: impl AsRef<Path>) -> Result<BufReader<File>, Error> {
    let path = path.as_ref();
    let file =
        File::open(path).change_context_lazy(|| Error::ReadFile(path.display().to_string()))?;
    Ok(BufReader::new(file))
}

/// Write content to a file
pub fn write_file(path: impl AsRef<Path>, content: impl AsRef<[u8]>) -> Result<(), Error> {
    let path = path.as_ref();
    std::fs::write(path, content)
        .change_context_lazy(|| Error::WriteFile(path.display().to_string()))
}

/// Read file as string
pub fn read_file(path: impl AsRef<Path>) -> Result<String, Error> {
    let path = path.as_ref();
    std::fs::read_to_string(path)
        .change_context_lazy(|| Error::ReadFile(path.display().to_string()))
}

/// Copy a file
pub fn copy_file(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<(), Error> {
    verboseln!(
        "cp '{}' '{}'",
        from.as_ref().display(),
        to.as_ref().display()
    );
    let from = from.as_ref();
    let to = to.as_ref();
    std::fs::copy(from, to).change_context_lazy(|| Error::WriteFile(to.display().to_string()))?;
    Ok(())
}

/// Get the modified time for a file.
///
/// If the file doesn't exist, None is returned
pub fn get_mtime(path: impl AsRef<Path>) -> Result<Option<FileTime>, Error> {
    let path = path.as_ref();
    if !path.exists() {
        return Ok(None);
    }

    let metadata = path
        .metadata()
        .change_context_lazy(|| Error::GetMTime(path.display().to_string()))?;

    Ok(Some(FileTime::from_last_modification_time(&metadata)))
}

/// Set the modified time for a file
pub fn set_mtime(path: impl AsRef<Path>, time: FileTime) -> Result<(), Error> {
    let path = path.as_ref();
    filetime::set_file_mtime(path, time)
        .change_context_lazy(|| Error::SetMTime(path.display().to_string()))
}

/// Return true if the build edge in -> out is up-to-date
#[inline]
pub fn up_to_date(in_mtime: Option<FileTime>, out_mtime: Option<FileTime>) -> bool {
    match (in_mtime, out_mtime) {
        (Some(in_mtime), Some(out_mtime)) => in_mtime <= out_mtime,
        _ => false,
    }
}

/// Convenience wrapper for std::fs::remove_dir_all
pub fn remove_directory(path: impl AsRef<Path>) -> Result<(), Error> {
    let path = path.as_ref();
    verboseln!("rm -r '{}'", path.display());
    if !path.exists() {
        return Ok(());
    }
    std::fs::remove_dir_all(path)
        .change_context_lazy(|| Error::RemoveDirectory(path.display().to_string()))
}

/// Convenience wrapper for std::fs::remove_file
pub fn remove_file(path: impl AsRef<Path>) -> Result<(), Error> {
    let path = path.as_ref();
    verboseln!("rm '{}'", path.display());
    if !path.exists() {
        return Ok(());
    }
    std::fs::remove_file(path).change_context_lazy(|| Error::RemoveFile(path.display().to_string()))
}

/// Convenience wrapper for std::fs::create_dir_all
pub fn ensure_directory(path: impl AsRef<Path>) -> Result<(), Error> {
    let path = path.as_ref();
    if path.exists() {
        return Ok(());
    }
    verboseln!("mkdir -p '{}'", path.display());
    std::fs::create_dir_all(path)
        .change_context_lazy(|| Error::CreateDirectory(path.display().to_string()))
}
