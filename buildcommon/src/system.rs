//! Common system operations
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

use error_stack::{Result, ResultExt};

/// Error messages
#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("cannot find megaton repository root! bad setup?")]
    FindToolRoot,
    #[error("environment check failed. Please refer to the errors above")]
    CheckEnv,

    // === path operations ===
    #[error("failed to get current executable path")]
    CurrentExe,
    #[error("failed to get parent path")]
    ParentPath,

    // === file operations ===
    #[error("failed to read from `{0}`")]
    ReadFile(String),
    #[error("failed to read YAML from `{0}`")]
    ReadYaml(String),
    #[error("failed to write to `{0}`")]
    WriteFile(String),
    #[error("failed to write YAML to `{0}`")]
    WriteYaml(String),
}

/// Path extensions
pub trait PathExt: Sized {
    /// Get the parent path, create a report on error
    fn parent_or_err(&self) -> Result<&Path, Error>;
    /// Get the parent path, create a report on error
    fn into_parent(self) -> Result<PathBuf, Error>;
    /// Push `path` onto `self` and return the result
    fn into_joined(self, path: impl AsRef<Path>) -> PathBuf;
}

impl PathExt for PathBuf {
    fn parent_or_err(&self) -> Result<&Path, Error> {
        self.parent().ok_or(Error::ParentPath)
        .attach_printable_lazy(|| format!("getting parent of: {}", self.display()))
    }

    fn into_parent(mut self) -> Result<PathBuf, Error> {
        let ok = self.pop();
        if !ok {
            return Err(Error::ParentPath)
            .attach_printable(format!("getting parent of: {}", self.display()));
        }
        Ok(self)
    }

    #[inline]
    fn into_joined(mut self, path: impl AsRef<Path>) -> PathBuf {
        self.push(path);
        self
    }
}

/// Create file for buffered writing
pub fn buf_writer(path: impl AsRef<Path>) -> Result<BufWriter<File>, Error>
{
    let path = path.as_ref();
    let file = File::create(path)
        .change_context_lazy(|| Error::WriteFile(path.display().to_string()))?;
    Ok(BufWriter::new(file))
}

/// Open file for buffered reading
pub fn buf_reader(path: impl AsRef<Path>) -> Result<BufReader<File>, Error>
{
    let path = path.as_ref();
    let file = 
    File::open(path)
        .change_context_lazy(|| Error::ReadFile(path.display().to_string()))?;
    Ok(BufReader::new(file))
}
