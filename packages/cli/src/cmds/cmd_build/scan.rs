// This module handles scanning the mod/library source

use cu::{Error, Result};
use std::path::{PathBuf, Path};

use super::{RustCrate, SourceFile};

// Get every source file in the given directory, recursivly
pub fn discover_source(dir: impl AsRef<Path>) -> Result<Vec<SourceFile>> {
    // TODO: Implement
    Ok(todo!())
}

// Find a rust crate in the given directory, if one exists
pub fn discover_crate(dir: impl AsRef<Path>) -> Option<RustCrate> {
    // TODO: Implement
    Some(todo!())
}
