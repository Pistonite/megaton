use crate::prelude::*;

use std::path::{Path, PathBuf};

use super::Error;

/// Path extensions
pub trait PathExt: Sized + AsRef<Path> {
    /// Get the parent path, create a report on error
    fn parent_or_err(&self) -> Result<&Path, Error>;
    /// Get the parent path, create a report on error
    fn into_parent(self) -> Result<PathBuf, Error>;
    /// Push `path` onto `self` and return the result
    fn into_joined(self, path: impl AsRef<Path>) -> PathBuf;
    /// Convert to an absolute path
    fn to_abs(&self) -> Result<PathBuf, Error> {
        let path = self.as_ref();
        dunce::canonicalize(path)
            .change_context_lazy(|| Error::Canonicalize(path.display().to_string()))
    }
    /// Convert to relative path from base.
    ///
    /// Both self and base should be absolute
    fn rebase(&self, base: impl AsRef<Path>) -> PathBuf {
        let path = self.as_ref();
        let base = base.as_ref();
        assert!(path.is_absolute());
        assert!(base.is_absolute());
        pathdiff::diff_paths(path, base).unwrap_or(path.to_path_buf())
    }

    fn to_utf8(&self) -> Result<String, Error> {
        self.as_ref()
            .as_os_str()
            .to_os_string()
            .into_string()
            .map_err(|_| report!(Error::NotUTF8(self.as_ref().display().to_string())))
    }
}

impl PathExt for PathBuf {
    fn parent_or_err(&self) -> Result<&Path, Error> {
        Ok(self.parent().ok_or(Error::ParentPath)?)
    }

    fn into_parent(mut self) -> Result<PathBuf, Error> {
        let ok = self.pop();
        if !ok {
            Err(Error::ParentPath)?;
        }
        Ok(self)
    }

    #[inline]
    fn into_joined(mut self, path: impl AsRef<Path>) -> PathBuf {
        self.push(path);
        self
    }
}

impl PathExt for &Path {
    fn parent_or_err(&self) -> Result<&Path, Error> {
        Ok(self.parent().ok_or(Error::ParentPath)?)
    }

    fn into_parent(self) -> Result<PathBuf, Error> {
        Ok(self.parent_or_err()?.to_path_buf())
    }

    fn into_joined(self, path: impl AsRef<Path>) -> PathBuf {
        self.join(path)
    }
}
